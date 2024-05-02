use rocket::futures::future::Either;
use rocket::http::Status;
use rocket::response::Redirect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use rspotify::ClientError;

use crate::lib::models::app_channels::AppChannels;
use crate::lib::models::playback_state::PlaybackState;

use super::animation::{AnimationController, AnimationControllerMessage};
use super::spotify::{SpotifyController, SpotifyControllerMessage};


pub struct ApplicationController {
    animation_controller: Arc<AnimationController>,
    spotify_controller: Arc<SpotifyController>,
    stop_flag: Arc<AtomicBool>,
    playback_rx: Arc<Mutex<Receiver<PlaybackState>>>,
    sp_msg_tx: Sender<SpotifyControllerMessage>,
    anim_msg_tx: Sender<AnimationControllerMessage>,
}

///
///  Main backend controller for rustify-wled
/// 
impl ApplicationController {
    pub fn new(
        target: String, 
        size: (u8, u8), 
        animation: AnimationController, 
        spotify: SpotifyController,
        playback_rx: Receiver<PlaybackState>,
        sp_msg_tx: Sender<SpotifyControllerMessage>,
        anim_msg_tx: Sender<AnimationControllerMessage>,
    ) -> ApplicationController {
        let stop_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        ApplicationController {
            animation_controller: Arc::new(animation),
            spotify_controller: Arc::new(spotify),
            stop_flag: stop_flag.clone(),
            playback_rx: Arc::new(Mutex::new(playback_rx)),
            sp_msg_tx: sp_msg_tx,
            anim_msg_tx: anim_msg_tx,
        }
    }

    pub fn start(&self) -> Result<Either<Redirect, String>, Status> {
        self.spotify_controller.start();
        self.animation_controller.start();

        // if already authenticated, start loop (polls Spotify, plays Animation)
        // and return Ok()
        // if not, return redirect to Spotify auth (this will be called again after auth)

        // if token does not exist, redirect to Spotify auth
        match self.spotify_controller.get_token() {
            Some(token) if !token.is_expired() => {
                self.sp_msg_tx.send(SpotifyControllerMessage::Start).unwrap();
                self.start_loop();

                Ok(Either::Right("start!".to_string()))
            },
            Some(token) if token.is_expired() => {
                // refresh token first
                let _ = self.spotify_controller.refresh_token();
                
                self.sp_msg_tx.send(SpotifyControllerMessage::Start).unwrap();
                self.start_loop();

                Ok(Either::Right("started with refreshed token!".to_string()))
            },
            Some(_) => {
                Ok(Either::Right("shouldn't be here m8".to_string()))
            },
            None => {
                let auth_url = self.spotify_controller.get_authorize_url();

                // redirect to Spotify auth
                Ok(Either::Left(Redirect::to(auth_url)))
            }
        }
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.animation_controller.stop_animation();
        self.sp_msg_tx.send(SpotifyControllerMessage::Terminate).unwrap();
    }

    // ///
    // /// Request access token using callback response
    // /// 
    pub fn callback(&self, code: &str) -> Result<(), ClientError> {
        self.spotify_controller.get_access_token(code)
    }

    fn start_loop(&self) {
        let local_stop_flag = self.stop_flag.clone();
        let local_receiver: Arc<Mutex<Receiver<PlaybackState>>> = self.playback_rx.clone();
        let local_anim_msg_tx = self.anim_msg_tx.clone();
        let local_sp_msg_tx = self.sp_msg_tx.clone();

        thread::spawn(move || {
            while !local_stop_flag.load(Ordering::Relaxed) {
                // asynchronously try to get data from sender
                match local_receiver.lock().unwrap().try_recv() {
                    // new playback state found, play it
                    Ok(new_playback) => {
                        local_anim_msg_tx.send(AnimationControllerMessage::Animate(new_playback.clone())).unwrap();

                        if PlaybackState::eq(&new_playback, &PlaybackState::none()) {
                            let local_local_sp_msg_tx = local_sp_msg_tx.clone();
                            let local_local_anim_msg_tx = local_anim_msg_tx.clone();

                            thread::spawn(move || {
                                thread::sleep(Duration::from_secs(5 * 60));
                                if PlaybackState::eq(&new_playback, &PlaybackState::none()) {
                                    local_local_sp_msg_tx.send(SpotifyControllerMessage::Timeout).unwrap();
                                    local_local_anim_msg_tx.send(AnimationControllerMessage::Timeout).unwrap();
                                }
                            });
                        }
                    }
                    // empty, move on
                    Err(mpsc::TryRecvError::Empty) => {
                        // Channel is empty, continue with the next iteration
                    }
                    // channel disconnected, stop loop
                    Err(mpsc::TryRecvError::Disconnected) => {
                        println!("SpotifyController disconnected. Stopping application loop.");
                        break;
                    }
                }

                thread::sleep(Duration::from_secs_f64(0.5));
            }

            local_stop_flag.store(false, Ordering::Relaxed);
        });
    }
}