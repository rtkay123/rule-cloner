mod config;
mod docker;
mod log;
mod node_pkg;

use std::{io::Write, path::PathBuf, process::Command, sync::Arc};

use crate::{config::cli, docker::ComposeService};

use self::{config::file::AppConfig, docker::Compose};
use anyhow::Result;
use clap::Parser;
use futures_util::future;
use git2::Repository;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    log::init();

    tokio::fs::create_dir_all(&cli.output).await?;

    let config = Arc::new(AppConfig::try_from(&cli.config)?);

    let compose = Arc::new(Mutex::new(Compose::default()));

    let mut handles = Vec::with_capacity(config.rules.rules.len());

    for rule in &config.rules.rules {
        let config = Arc::clone(&config);
        let folder = cli.dest_folder(rule);
        let compose = Arc::clone(&compose);
        handles.push(tokio::spawn(clone_repo(
            config,
            rule.to_owned(),
            folder,
            compose,
        )));
    }

    if let Err(e) = future::try_join_all(handles).await {
        error!("{e}");
    }

    let handle = compose.lock().await;

    let file_name = "compose.yaml";

    debug!(file = file_name, "creating compose file");

    let mut compose_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_name)?;

    let data = serde_yaml::to_string(&*handle)?;
    debug!(file = file_name, "writing compose file");
    compose_file.write_all(data.as_bytes())?;
    debug!(file = file_name, "compose file successfully written");

    Ok(())
}

async fn clone_repo(
    config: Arc<AppConfig>,
    rule: String,
    folder: PathBuf,
    compose: Arc<Mutex<Compose>>,
) -> Result<()> {
    debug!(repo = %config.executor.source, dest = %folder.display(), "cloning");
    match Repository::clone(&config.executor.source, &folder) {
        Ok(repo) => {
            let ovrd = config
                .rules
                .override_field
                .iter()
                .find(|ovrd| ovrd.rule == rule);

            let package = format!(
                "rule@{}",
                match ovrd {
                    Some(data) => {
                        checkout_rev(repo, &data.git_ref)?;
                        match &data.version {
                            config::file::RuleVersion::Source { git } => git.to_owned(),
                            config::file::RuleVersion::Version { source } => {
                                let registry = &source.registry;
                                let prefix = &source.prefix;
                                node_pkg::build_name(
                                    config.rules.source.scope.as_deref(),
                                    registry,
                                    &rule,
                                    prefix,
                                    &config.rules.source.version,
                                )
                            }
                        }
                    }
                    None => {
                        checkout_rev(repo, &config.executor.git_ref)?;
                        node_pkg::build_name(
                            config.rules.source.scope.as_deref(),
                            &config.rules.source.registry,
                            &rule,
                            &config.rules.source.prefix,
                            &config.rules.source.version,
                        )
                    }
                }
            );

            debug!(rule = package, dest = %folder.display(), "installing npm package");

            let folder_str = folder.to_str().expect("to have valid str");

            let command = format!("npm install --prefix {folder_str} {package}");

            if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/C", &command])
                    .output()
                    .expect("failed to execute process")
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()
                    .expect("failed to execute process")
            };
            info!(rule = package, "ready");

            let mut file = compose.lock().await;
            file.services.insert(
                format!("{}{rule}", config.rules.source.prefix),
                ComposeService::from((folder.to_string_lossy().to_string(), rule)).await?,
            );

            Ok(())
        }
        Err(e) => {
            error!("{e}");
            anyhow::bail!("{e}")
        }
    }
}

fn checkout_rev(repo: Repository, rev: &str) -> Result<()> {
    let (object, reference) = repo.revparse_ext(rev)?;

    repo.checkout_tree(&object, None)?;

    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
    }?;
    Ok(())
}
