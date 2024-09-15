use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::sync::Arc;
use std::thread::{self};
use std::time::Duration;
use rspotify::model::{AdditionalType, TrackId};
use rspotify::{AuthCodeSpotify, ClientError, Token};
use rspotify::clients::{BaseClient, OAuthClient};
use rspotify::model::PlayableItem::Track;
use crate::lib::models::app_channels::{self, AppChannels};
use crate::lib::models::playback_state::PlaybackState;
use crate::settings::SETTINGS;
use crate::utils::image::precache_image;
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
    playback_tx: Arc<Sender<PlaybackState>>,
    sp_msg_rx: Arc<Mutex<Receiver<SpotifyControllerMessage>>>,
}


impl SpotifyController {
    /////////////////////////////////////////
    /// Public Functions
    /////////////////////////////////////////

    pub fn new(playback_tx: Sender<PlaybackState>, sp_msg_rx: Receiver<SpotifyControllerMessage>) -> Self {
        Self { 
            client: Arc::new(get_client()),
            playback_tx: Arc::new(playback_tx),
            sp_msg_rx: Arc::new(Mutex::new(sp_msg_rx)),
        }
    }

    pub fn start(&self) {
        // initialization, send None first
        let _ = self.playback_tx.send(PlaybackState::none());

        let local_client = self.client.clone();
        let local_sender = self.playback_tx.clone();
        let local_receiver = self.sp_msg_rx.clone();

        thread::spawn(move || {
            // Mutex guard for receiver's use while inside this thread
            let receiver_guard = local_receiver.lock().unwrap();
            SpotifyController::playback_loop(&receiver_guard, &local_client, &local_sender);
        });
    }

    fn playback_loop(
        receiver_guard: &Receiver<SpotifyControllerMessage>,
        client: &AuthCodeSpotify,
        sender: &Sender<PlaybackState>
    ) {
        let mut current_playing: PlaybackState = PlaybackState::none();

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
                                if PlaybackState::eq(&current_playing.clone(), &PlaybackState::none()) {
                                    println!("Idled for too long, TIMEDOUT");
                                    break;
                                }
                                println!("Received TIMEOUT, but ignoring");
                            },
                            // no message, do nothing
                            _ => {},
                        }

                        // check if track has changed
                        match SpotifyController::track_changed(&client, &current_playing) {
                            (true, new_playback) => {
                                let local_client = client.clone();

                                // precache image
                                thread::spawn(move || {
                                    SpotifyController::precache_queue(&local_client);
                                });

                                // send new playback state
                                current_playing = new_playback;
                                match sender.send(current_playing.clone()) {
                                    Ok(_) => {},
                                    Err(_) => {},
                                }
                            },
                            (false, _) => {},
                        }

                        thread::sleep(Duration::from_secs(SETTINGS.read().unwrap().spotify.polling_seconds));
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
                // no message, do nothing
                Err(mpsc::RecvError) => {},
            }
        }
    }

    fn precache_queue(client: &AuthCodeSpotify) {
        let cache_count = SETTINGS.read().unwrap().spotify.precache_albums;

        match cache_count {
            Some(count) => {
                let queue = client.current_user_queue().unwrap();

                // precache only specified number of images
                for track in queue.queue.iter().take(count as usize) {
                    match track {
                        // if Track
                        Track(track) => {
                            let _ = precache_image(&track.album.images.first().unwrap().url);
                        },
                        // if Episode
                        _ => {},
                    }
                }
            },
            None => {}
        }
    }


    ///
    /// Determines whether the controller should update animation.
    /// 
    /// Returns:
    ///    - bool: whether the controller should update animation
    ///    - PlaybackState: the current playback state
    fn track_changed(client: &AuthCodeSpotify, current_playing: &PlaybackState) -> (bool, PlaybackState) {
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

    pub fn refresh_token(&self) -> Result<(), ClientError> {
        self.client.refresh_token()
    }
}