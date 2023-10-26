use std::path::PathBuf;

use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub executor: Executor,
    pub rules: Rules,
}

impl TryFrom<&PathBuf> for AppConfig {
    type Error = anyhow::Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let path = value.to_string_lossy();
        debug!(path = %path, "reading config file");
        let str = std::fs::read_to_string(value)?;
        let config: Self = toml::from_str(&str)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Executor {
    pub source: String,
    pub branch: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Override {
    pub rule: String,
    #[serde(flatten)]
    pub version: RuleVersion,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RuleVersion {
    Source {
        url: String,
    },
    Version {
        version: String,
        registry: String,
        scope: String,
        prefix: String,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rules {
    pub registry: String,
    pub scope: String,
    pub prefix: String,
    pub rules: Vec<String>,
    #[serde(rename = "override")]
    pub override_field: Vec<Override>,
}
