use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs::{File, OpenOptions};

#[derive(Deserialize, Serialize, Default)]
pub struct Compose {
    pub services: BTreeMap<String, ComposeService>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct ComposeService {
    pub build: Build,
    pub env_file: Vec<String>,
    pub restart: String,
    pub depends_on: Vec<String>,
}

impl ComposeService {
    pub async fn from(value: (String, String)) -> Result<Self> {
        let (path, rule) = value;
        create_env(&rule, &path).await?;

        Ok(Self {
            build: Build {
                context: path.clone(),
                args: vec!["GH_TOKEN".to_owned()],
            },
            env_file: vec![format!("{path}/.env"), ".env".into()],
            restart: "always".into(),
            depends_on: vec!["redis".to_owned(), "arango".to_owned()],
        })
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct Build {
    pub context: String,
    pub args: Vec<String>,
}

async fn write(file: &mut File, key: &'static str, value: String) -> Result<()> {
    crate::async_writeln!(file, "{key}={value}")
}

async fn create_env(rule: &str, destination: &str) -> Result<()> {
    let path = format!("{destination}/.env");

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .await?;

    // rule needs to be number

    let _ = write(&mut file, "FUNCTION_NAME", format!("rule-{rule}-rel-1-0-0")).await;

    let _ = write(&mut file, "NODE_ENV", "dev".into()).await;
    let _ = write(&mut file, "RULE_VERSION", "1.0.0".into()).await;
    let _ = write(&mut file, "RULE_NAME", rule.into()).await;
    let _ = write(&mut file, "QUOTING", "false".into()).await;

    let _ = write(&mut file, "REDIS_DB", "0".into()).await;
    let _ = write(&mut file, "REDIS_AUTH", "".into()).await;

    let servers: String = r#"[{"host": "redis", "port":6379}]"#.into();

    let _ = write(&mut file, "REDIS_SERVERS", servers).await;
    let _ = write(&mut file, "REDIS_IS_CLUSTER", "false".to_owned()).await;

    let _ = write(&mut file, "STARTUP_TYPE", "nats".into()).await;
    let _ = write(&mut file, "SERVER_URL", "nats:4222".into()).await;
    let _ = write(&mut file, "PRODUCER_STREAM", format!("RuleResponse{rule}")).await;
    let _ = write(&mut file, "CONSUMER_STREAM", format!("RuleRequest{rule}")).await;

    let _ = write(
        &mut file,
        "TRANSACTION_HISTORY_DATABASE_NAME",
        "transactionHistory".into(),
    )
    .await;
    let _ = write(
        &mut file,
        "TRANSACTION_HISTORY_DATABASE_URL",
        "tcp://arango:8529".into(),
    )
    .await;
    let _ = write(
        &mut file,
        "TRANSACTION_HISTORY_DATABASE_USER",
        "root".into(),
    )
    .await;
    let _ = write(
        &mut file,
        "TRANSACTION_HISTORY_DATABASE_PASSWORD",
        "".into(),
    )
    .await;
    let _ = write(
        &mut file,
        "TRANSACTION_HISTORY_DATABASE_CERT_PATH",
        "".into(),
    )
    .await;
    let _ = write(&mut file, "CONFIG_DATABASE", "Configuration".into()).await;
    let _ = write(&mut file, "CONFIG_COLLECTION", "configuration".into()).await;

    let _ = write(&mut file, "PSEUDONYMS_DATABASE", "pseudonyms".into()).await;
    let _ = write(
        &mut file,
        "PSEUDONYMS_DATABASE_URL",
        "tcp://arango:8529".into(),
    )
    .await;
    let _ = write(&mut file, "PSEUDONYMS_DATABASE_USER", "root".into()).await;
    let _ = write(&mut file, "PSEUDONYMS_DATABASE_PASSWORD", "".into()).await;

    let _ = write(&mut file, "CACHE_TTL", "300".into()).await;

    let _ = write(&mut file, "APM_ACTIVE", "false".into()).await;

    Ok(())
}

#[macro_export]
macro_rules! async_writeln {
    ($dst: expr) => {
        {
            tokio::io::AsyncWriteExt::write_all(&mut $dst, b"\n").await
        }
    };
    ($dst: expr, $fmt: expr) => {
        {
            use std::io::Write;
            let mut buf = Vec::<u8>::new();
            writeln!(buf, $fmt)?;
            tokio::io::AsyncWriteExt::write_all($dst, &buf).await.map_err(|e| e.into())
        }
    };
    ($dst: expr, $fmt: expr, $($arg: tt)*) => {
        {
            use std::io::Write;
            let mut buf = Vec::<u8>::new();
            writeln!(buf, $fmt, $( $arg )*)?;
            tokio::io::AsyncWriteExt::write_all(&mut $dst, &buf).await
        }
    };
}
