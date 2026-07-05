use serde::Deserialize;
use std::path::Path;

/// Top-level configuration for polyquery, loaded from a TOML file.
#[derive(Debug, Default, Deserialize)]
pub struct PolyqueryConfig {
    /// Optional database profile configuration.
    #[serde(default)]
    pub profile: Option<ProfileConfig>,
}

/// Configuration for a database profile, specifying connection details.
#[derive(Debug, Deserialize)]
pub struct ProfileConfig {
    /// The database driver to use (e.g., "postgres", "mysql", "sqlite").
    pub driver: Option<String>,
    /// The database host address.
    pub host: Option<String>,
    /// The database port number.
    pub port: Option<u16>,
    /// The name of the database to connect to.
    pub database: Option<String>,
    /// The username for authentication.
    pub user: Option<String>,
    /// Environment variable name containing the database password.
    pub password_env: Option<String>,
}

impl PolyqueryConfig {
    /// Loads configuration from a TOML file at the given path.
    ///
    /// Returns a default configuration if the file cannot be read or parsed.
    pub fn load(path: &Path) -> Self {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        toml::from_str(&content).unwrap_or_default()
    }
}
