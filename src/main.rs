mod config;
mod docker;
mod log;

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
    info!(repo = %config.executor.source, "cloning");
    Repository::clone(&config.executor.source, &folder)?;

    let ovrd = config
        .rules
        .override_field
        .iter()
        .find(|ovrd| ovrd.rule == rule);

    match ovrd {
        Some(_data) => {
            todo!("build a package")
        }
        None => {
            let package = format!(
                "rule@npm:{}/{}{}@latest",
                config.rules.scope, config.rules.prefix, rule
            );
            debug!(rule = package, "installing");
            Command::new("npm")
                .arg("install")
                .arg("--prefix")
                .arg(&folder)
                .arg(package)
                .output()
                .expect("failed to execute process");
        }
    }

    let mut file = compose.lock().await;
    file.services.insert(
        format!("{}{rule}", config.rules.prefix),
        ComposeService::from((folder.to_string_lossy().to_string(), rule)).await?,
    );

    Ok(())
}
