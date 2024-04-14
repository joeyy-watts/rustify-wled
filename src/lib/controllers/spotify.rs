use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::{atomic::AtomicBool, Arc};
use std::thread;
use std::time::Duration;
use rspotify::model::{AdditionalType, AudioFeatures, CurrentPlaybackContext, Id, TrackId, PlayableItem};
use rspotify::AuthCodeSpotify;
use rspotify::clients::{BaseClient, OAuthClient};

use crate::utils::spotify::get_client;


pub struct SpotifyController {
    // we need AuthCodeSpotify as we need private info for currently playing
    pub client: Arc<AuthCodeSpotify>,
    stop_flag: Arc<AtomicBool>,
    sender: Arc<Sender<PlaybackState>>,
    current_playing: Arc<PlaybackState>,
}

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
                cover_url: Some(String::from("https://play-lh.googleusercontent.com/cShys-AmJ93dB0SV8kE6Fl5eSaf4-qMMZdwEDKI5VEmKAXfzOqbiaeAsqqrEBCTdIEs")),
                features: None,
            }
    }
}

impl SpotifyController {
    /////////////////////////////////////////
    /// Public Functions
    /////////////////////////////////////////

    pub fn new(anim_sender: Sender<PlaybackState>) -> Self {
        let client = Arc::new(get_client());

        let stop_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        // sender to send message to AnimationController
        let sender: Arc<Sender<PlaybackState>> = Arc::new(anim_sender);

        let current_playing = Arc::new(PlaybackState::none());
   
        Self { client, stop_flag, sender, current_playing }
    }

    pub fn start(&self) {
        // initialization, send None first
        let _ = self.sender.send(PlaybackState::none());

        // spawn thread for Spotify polling
        self.spawn_thread();
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    /////////////////////////////////////////
    /// Internal Functions
    /////////////////////////////////////////

    fn spawn_thread(&self) {
        // owned by controller
        let local_client = self.client.clone();
        // already valid; clone the sender
        let local_sender = self.sender.clone();
        // still need; this is valid
        let local_stop_flag = self.stop_flag.clone();

        // should be owned by thread
        let mut local_current_playing = self.current_playing.clone();
        
        thread::spawn(move || {
            while !local_stop_flag.load(Ordering::Relaxed) {
                let new_playing = local_client.current_playback(
                    None, 
                    None::<Vec<&AdditionalType>>
                ).unwrap(); 
                
                let mut new_playback = match new_playing {
                    Some(context) => {
                        PlaybackState::from_playback_context(context)
                    },
                    None => {
                        PlaybackState::none()
                    }
                };

                // if playback state has changed: get audio features, update self and send it
                if !PlaybackState::eq(&new_playback, &local_current_playing) {
                    let track_id: Option<TrackId> = match new_playback.track_id.as_ref() {
                        Some(id) => Some(TrackId::from_id(id).unwrap()),
                        None => None,
                    };
                    
                    match track_id {
                        Some(id) => {
                            new_playback.add_features(
                                Some(local_client.track_features(id).unwrap())
                            );
                        },
                        None => {},
                    }

                    // clone new playback state into 2 variables, one to self, one to sender
                    // a bit messy but i can't figure it out for the life of me
                    local_current_playing = Arc::new(new_playback.clone());

                    match local_sender.send(new_playback) {
                        // TODO: add error handling
                        Ok(_) => {},
                        Err(_) => {},
                    }
                }
                
                thread::sleep(Duration::from_secs(2));
            }

            local_stop_flag.store(false, Ordering::Relaxed);
        });
    }
}