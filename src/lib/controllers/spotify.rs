use std::error::Error;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::{atomic::AtomicBool, Arc};
use std::thread;
use std::time::Duration;

use rspotify::model::{AdditionalType, AudioFeatures, CurrentPlaybackContext, Id, TrackId, PlayableItem};
use rspotify::{scopes, AuthCodeSpotify, ClientError, Config, Credentials, OAuth, Token};
use rspotify::clients::{BaseClient, OAuthClient};


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

    pub fn add_features(&mut self, features: AudioFeatures) {
        self.features = Some(features);
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
    pub fn new(sender: Sender<PlaybackState>) -> Self {
        let config = Config {
            token_cached: true,
            ..Default::default()
        };

        let credentials: Credentials = match Credentials::from_env() {
            Some(_) => {
                Credentials::from_env().unwrap()
            },
            None => panic!("Environment variable RSPOTIFY_CLIENT_ID and/or RSPOTIFY_CLIENT_SECRET not found"),
        };

        let oauth: OAuth = OAuth {
            redirect_uri: "http://localhost:8000/callback".to_string(),
            scopes: scopes!(
                "user-read-playback-state",
                "user-read-currently-playing"
            ),
            ..Default::default()
          
        };

        let client = Arc::new(AuthCodeSpotify::with_config(credentials, oauth, config));

        let stop_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let sender: Arc<Sender<PlaybackState>> = Arc::new(sender);

        let current_playing = Arc::new(PlaybackState::none());
   
        Self { client, stop_flag, sender, current_playing }
    }

    pub fn get_token(&self) -> Option<Token> {
         self.client.get_token().lock().unwrap().clone()
    }

    pub fn get_authorize_url(&self) -> String {
        self.client.get_authorize_url(false).unwrap()
    }

    pub fn get_access_token(&self, response_code: &str) -> Result<(), ClientError> {
        self.client.request_token(response_code)
    }

    pub fn start_listening(&self) {
        // initialize loop, send None first
        self.sender.send(PlaybackState::none());

        let local_client = self.client.clone();
        let mut local_current_playing = self.current_playing.clone();
        let local_sender = self.sender.clone();
        let local_stop_flag = self.stop_flag.clone();

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
                    let track_id = TrackId::from_id(new_playback.track_id.as_ref().unwrap()).unwrap();
                    new_playback.add_features(
                        local_client.track_features(track_id).unwrap()
                    );

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

    pub fn stop_listening(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}