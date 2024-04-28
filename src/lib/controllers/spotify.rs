use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::sync::Arc;
use std::thread::{self};
use std::time::Duration;
use rspotify::model::{AdditionalType, TrackId};
use rspotify::{AuthCodeSpotify, ClientError, Token};
use rspotify::clients::{BaseClient, OAuthClient};

use crate::lib::models::playback_state::PlaybackState;
use crate::utils::spotify::get_client;

#[derive(Clone, Copy)]
pub enum SpotifyControllerMessage {
    Start,      // start Spotify polling loop
    Stop,       // stop Spotify polling loop
    Terminate,  // terminate the message loop
    Timeout,    // timeout signal
}

pub struct SpotifyController {
    // we need AuthCodeSpotify as we need private info for currently playing
    pub client: Arc<AuthCodeSpotify>,
    tx_app: Arc<Sender<PlaybackState>>,
    rx_app: Arc<Mutex<Receiver<SpotifyControllerMessage>>>,
}


impl SpotifyController {
    /////////////////////////////////////////
    /// Public Functions
    /////////////////////////////////////////

    pub fn new(tx_app: Sender<PlaybackState>, rx_app: Receiver<SpotifyControllerMessage>) -> Self {
        Self { 
            client: Arc::new(get_client()),
            tx_app: Arc::new(tx_app),
            rx_app: Arc::new(Mutex::new(rx_app)), 
        }
    }

    pub fn start(&self) {
        // initialization, send None first
        let _ = self.tx_app.send(PlaybackState::none());

        let local_client = self.client.clone();
        let local_sender = self.tx_app.clone();
        let local_receiver = self.rx_app.clone();

        thread::spawn(move || {
            let mut current_playing: PlaybackState = PlaybackState::none();
            // Mutex guard for receiver's use while inside this thread
            let receiver_guard = local_receiver.lock().unwrap();
            
            loop {
                match receiver_guard.recv() {
                    Ok(SpotifyControllerMessage::Start) => {
                        println!("got start message");
                        loop {
                            match receiver_guard.try_recv() {
                                Ok(SpotifyControllerMessage::Stop) => {
                                    println!("Received STOP command");
                                    break;
                                },
                                Ok(SpotifyControllerMessage::Terminate) => {
                                    println!("Received TERMINATE command");
                                    break;
                                },
                                Ok(SpotifyControllerMessage::Timeout) => {
                                    if PlaybackState::eq(&current_playing, &PlaybackState::none()) {
                                        println!("Idled for too long, TIMEDOUT");
                                        break;
                                    }
                                    println!("Received TIMEOUT, but ignoring");
                                },
                                _ => {
                                    // normal loop, not breaking
                                },
                            }

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
                    },
                    // for handling messages when loop is not running
                    Ok(SpotifyControllerMessage::Stop) => {
                        println!("Received STOP, but no running polling loop.")
                    },
                    // terminate the entire controller
                    Ok(SpotifyControllerMessage::Terminate) => {
                        break;
                    },
                    Ok(SpotifyControllerMessage::Timeout) => {
                    },
                    Err(mpsc::RecvError) => {
                        // empty, do nothing
                    },
                }
            }
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