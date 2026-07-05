use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
pub struct PolyqueryConfig {
    #[serde(default)]
    pub profile: Option<ProfileConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ProfileConfig {
    pub driver: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub user: Option<String>,
    pub password_env: Option<String>,
}

impl PolyqueryConfig {
    pub fn load(path: &Path) -> Self {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        toml::from_str(&content).unwrap_or_default()
    }
}
