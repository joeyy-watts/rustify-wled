use config::{Config, ConfigError, File};
use serde_derive::Deserialize;


#[derive(Debug, Deserialize)]
pub struct App {
    pub(crate) callback_url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Target {
    pub(crate) host: String,
    pub(crate) size: (u8, u8),
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub(crate) target: Target,
    pub(crate) app: App,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("config/config.toml"))
            .build()?;

        s.try_deserialize()
    }
}