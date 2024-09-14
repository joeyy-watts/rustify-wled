use rspotify::model::{AudioFeatures, CurrentPlaybackContext, Id, PlayableItem};
use crate::settings::SETTINGS;

/// State of the current playback, to be tracked
#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub track_name: Option<String>,
    pub track_id: Option<String>,
    pub cover_url: Option<String>,
    pub features: Option<AudioFeatures>,
}

impl PartialEq for PlaybackState {
    fn eq(&self, other: &Self) -> bool {
        self.is_playing == other.is_playing &&
        self.track_id == other.track_id
    }
}

impl PlaybackState {
    /// Creates PlaybackState from a CurrentPlaybackContext
    /// 
    /// Does not contain AudioFeatures.
    pub fn from_playback_context(context: CurrentPlaybackContext) -> Self {
        match context.item {
            Some(PlayableItem::Track(track)) => {
                Self {
                    is_playing: context.is_playing,
                    track_name: Some(String::from(track.name)),
                    track_id: Some(String::from(track.id.unwrap().id())),
                    cover_url: Some(track.album.images.first().unwrap().url.clone()),
                    features: None,
                    }
            },
            Some(PlayableItem::Episode(_)) => PlaybackState::none(),
            None => PlaybackState::none(),
        }
    }

    pub fn add_features(&mut self, features: Option<AudioFeatures>) {
        self.features = features;
    }

    pub fn none() -> Self {
        Self {
                is_playing: false,
                track_name: None,
                track_id: None,
                cover_url: SETTINGS.read().unwrap().app.idle_image_url.clone(),
                features: None,
            }
    }
}