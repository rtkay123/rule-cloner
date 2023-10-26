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
    #[serde(rename = "ref")]
    pub git_ref: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Override {
    pub rule: String,
    #[serde(flatten)]
    pub version: RuleVersion,
    #[serde(rename = "executor-ref")]
    pub git_ref: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RuleVersion {
    Source {
        git: String,
    },
    Version {
        #[serde(flatten)]
        source: RuleSource,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rules {
    pub rules: Vec<String>,
    #[serde(flatten)]
    pub source: RuleSource,
    #[serde(rename = "override")]
    pub override_field: Vec<Override>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleSource {
    #[serde(default = "default_version")]
    pub version: String,
    pub registry: String,
    pub scope: Option<String>,
    pub prefix: String,
}

fn default_version() -> String {
    "@latest".to_string()
}
