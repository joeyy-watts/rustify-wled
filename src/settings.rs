use std::sync::RwLock;
use once_cell::sync::Lazy;
use config::{Config, ConfigError, File};
use log::warn;
use serde_derive::Deserialize;
use crate::utils::network::resolve_ip;

static SPOTIFY_POLLING_SECONDS_WARNING: u64 = 1;


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

        Self::verify_settings(settings)
    }

    fn verify_settings(settings: Settings) -> Result<(Settings), ConfigError> {
        if settings.spotify.polling_seconds <= 0 {
            return Err(ConfigError::Message("Polling seconds must be greater than 0".to_string()));
        } else if settings.spotify.polling_seconds < SPOTIFY_POLLING_SECONDS_WARNING {
            warn!("Be careful with short polling times! You may hit Spotify API rate limit. (current: {} seconds)", settings.spotify.polling_seconds);
        }

        if settings.targets.is_empty() {
            return Err(ConfigError::Message("No targets defined".to_string()));
        }

        if settings.animation.target_fps <= 0 {
            return Err(ConfigError::Message("Target FPS must be greater than 0".to_string()));
        } else if settings.animation.target_fps > 40 {
            warn!("The ArtNet protocol does not exceed 40 FPS, you may be wasting processing power (current: {} FPS)", settings.animation.target_fps);
        }

        for target in settings.targets.iter() {
            if target.size.0 < 1 || target.size.1 < 0 {
                return Err(ConfigError::Message(("Invalid target size {} for {}", &target.size, &target.host).to_string()));
            } else if (target.size.0 * target.size.1) as u16 > 1500 as u16 {
                warn!("Target size {} x {} for device {} exceeds the maximum number of LEDs WLED can drive: https://kno.wled.ge/interfaces/e1.31-dmx/, normal behavior is NOT GUARANTEED", &target.size.0, &target.size.1, &target.host);
            }

            resolve_ip(&target.host).expect(format!("Target address {} is unreachable! Please ensure proper connection or remove the device.", &target.host).as_str());
        }

        Ok(settings)
    }
}

pub static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| {
    let settings = Settings::new().expect("Failed to load settings");
    RwLock::new(settings)
});