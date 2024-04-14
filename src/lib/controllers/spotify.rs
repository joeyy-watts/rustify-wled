use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::{atomic::AtomicBool, Arc};
use std::thread::{self, current};
use std::time::Duration;
use rspotify::model::{AdditionalType, TrackId};
use rspotify::{AuthCodeSpotify, ClientError, Token};
use rspotify::clients::{BaseClient, OAuthClient};

use crate::lib::models::playback_state::PlaybackState;
use crate::utils::spotify::get_client;


pub struct SpotifyController {
    // we need AuthCodeSpotify as we need private info for currently playing
    pub client: Arc<AuthCodeSpotify>,
    stop_flag: Arc<AtomicBool>,
    sender: Arc<Sender<PlaybackState>>,
    current_playing: Arc<PlaybackState>,
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
        let local_client = self.client.clone();
        let local_sender = self.sender.clone();
        let local_stop_flag = self.stop_flag.clone();

        
        thread::spawn(move || {
            // keeps the current playback state
            let mut current_playing: PlaybackState = PlaybackState::none();

            while !local_stop_flag.load(Ordering::Relaxed) {
                let update = SpotifyController::should_update(&local_client, &current_playing);

                match update {
                    (true, new_playback) => {
                        current_playing = new_playback;
                        match local_sender.send(current_playing.clone()) {
                            Ok(_) => {},
                            Err(_) => {},
                        }
                    },
                    (false, _) => {},
                }

                thread::sleep(Duration::from_secs(2));
            }

            local_stop_flag.store(false, Ordering::Relaxed);
        });
    }


    ///
    /// Determines whether the controller should update animation.
    /// 
    /// Returns:
    ///    - bool: whether the controller should update animation
    ///    - PlaybackState: the current playback state
    fn should_update(client: &AuthCodeSpotify, current_playing: &PlaybackState) -> (bool, PlaybackState) {
        // current playback context from rspotify client
        let context = client.current_playback(
            None, 
            None::<Vec<&AdditionalType>>
        ).unwrap();

        // convert context to PlaybackState
        let mut new_playback = match context {
            Some(context) => {
                PlaybackState::from_playback_context(context)
            },
            None => {
                PlaybackState::none()
            }
        };

        // check if state has changed
        if !PlaybackState::eq(&new_playback, &current_playing) {
            // if state has changed, get audio features and return `true`
            let track_id: Option<TrackId> = match new_playback.track_id.as_ref() {
                Some(id) => Some(TrackId::from_id(id).unwrap()),
                None => None,
            };
            
            match track_id {
                Some(id) => {
                    new_playback.add_features(
                        Some(client.track_features(id).unwrap())
                    );
                },
                None => {},
            }

            (true, new_playback)
        } else {
            (false, new_playback)
        }
    }

    /////////////////////////////////////////
    /// rspotify Client-related Functions
    /////////////////////////////////////////
    
    pub fn get_token(&self) -> Option<Token> {
        self.client.get_token().lock().unwrap().clone()
    }
    
    pub fn get_authorize_url(&self) -> String {
        self.client.get_authorize_url(false).unwrap()
    }
    
    pub fn get_access_token(&self, code: &str) -> Result<(), ClientError> {
        self.client.request_token(code)
    }
}