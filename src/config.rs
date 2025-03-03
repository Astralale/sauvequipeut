use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self};

/// Structure de configuration du client, chargée depuis `config.toml`
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server_address: String,
    pub team_name: String,
    pub navigation_mode: String,
    pub save_progress: bool,
}

impl Config {
    /// Charge la configuration depuis `config.toml`
    pub fn load() -> Result<Self, io::Error> {
        let config_data = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_data).expect("Erreur lors du parsing TOML");
        Ok(config)
    }
}
