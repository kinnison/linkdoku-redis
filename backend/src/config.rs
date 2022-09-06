//! Configuration for linkdoku backend
//!

use std::{collections::HashMap, path::PathBuf};

use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct OpenIDProvider {
    pub client_id: String,
    pub client_secret: String,
    pub discovery_doc: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub resources: PathBuf,
    pub port: u16,
    pub redis_url: Url,
    pub redirect_url: String,
    pub cookie_secret: String,
    pub openid: HashMap<String, OpenIDProvider>,
}

const BASE_ENV: &str = "dev";

pub fn load_configuration() -> Result<Configuration, ConfigError> {
    let config = Config::builder()
        .add_source(File::new(
            &format!("linkdoku-config-{}.yaml", BASE_ENV),
            FileFormat::Yaml,
        ))
        .add_source(
            Environment::default()
                .prefix("LINKDOKU")
                .separator("_")
                .try_parsing(true),
        );

    config.build()?.try_deserialize()
}
