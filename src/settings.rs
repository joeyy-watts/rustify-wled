use std::sync::RwLock;
use once_cell::sync::Lazy;
use config::{Config, ConfigError, File};
use serde_derive::Deserialize;


#[derive(Debug, Deserialize)]
pub struct App {
    pub(crate) callback_url: String,
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<String>,
    pub(crate) idle_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Spotify {
    pub(crate) polling_seconds: u64,
    pub(crate) precache_albums: Option<u8>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Target {
    pub(crate) host: String,
    pub(crate) size: (u8, u8),
}

#[derive(Debug, Deserialize)]
#[allow(unused)]

pub struct Animation {
    pub(crate) target_fps: u8,
    #[serde(skip)]
    pub(crate) frame_interval: f64,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub(crate) targets: Vec<Target>,
    pub(crate) spotify: Spotify,
    pub(crate) app: App,
    pub(crate) animation: Animation,
}

impl Settings {
    fn new() -> Result<Self, ConfigError> {

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("config/config.toml"))
            .build()?;

        let mut settings = s.try_deserialize::<Settings>()?;

        settings.animation.frame_interval = 1.0 / settings.animation.target_fps as f64;

        Ok(settings)
    }
}

pub static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| {
    let settings = Settings::new().expect("Failed to load settings");
    RwLock::new(settings)
});