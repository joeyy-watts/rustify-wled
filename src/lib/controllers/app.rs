use rocket::futures::future::Either;
use rocket::http::Status;
use rocket::response::Redirect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use rspotify::ClientError;

use super::animation::AnimationController;
use super::spotify::{PlaybackState, SpotifyController};


pub struct ApplicationController {
    animation_controller: Arc<AnimationController>,
    spotify_controller: Arc<SpotifyController>,
    stop_flag: Arc<AtomicBool>,
    receiver: Arc<Mutex<Receiver<PlaybackState>>>,
}

///
///  Main backend controller for rustify-wled
/// 
impl ApplicationController {
    pub fn new(target: String, size: (u8, u8)) -> ApplicationController {
        let animation_controller = Arc::new(AnimationController::new(target, size));

        let (tx, rx): (Sender<PlaybackState>, Receiver<PlaybackState>) = mpsc::channel();

        let spotify_controller: Arc<SpotifyController> = Arc::new(SpotifyController::new(tx));

        let stop_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        Self { animation_controller, spotify_controller, stop_flag, receiver: Arc::new(Mutex::new(rx)) }
    }

    pub fn start(&self) -> Result<Either<Redirect, String>, Status> {
        // if already authenticated, start loop (polls Spotify, plays Animation)
        // and return Ok()
        // if not, return redirect to Spotify auth (this will be called again after auth)

        // if token does not exist, redirect to Spotify auth
        match self.spotify_controller.get_token() {
            Some(token) if !token.is_expired() => {
                self.spotify_controller.start();
                self.start_loop();

                Ok(Either::Right("start!".to_string()))
            },
            Some(token) if token.is_expired() => {
                // refresh token first
                let _ = self.spotify_controller.get_access_token(token.refresh_token.unwrap().as_ref());
                
                self.spotify_controller.start();
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
        self.spotify_controller.stop();
    }

    // ///
    // /// Request access token using callback response
    // /// 
    pub fn callback(&self, code: &str) -> Result<(), ClientError> {
        self.spotify_controller.get_access_token(code)
    }

    fn start_loop(&self) {
        let local_stop_flag = self.stop_flag.clone();
        let local_receiver: Arc<Mutex<Receiver<PlaybackState>>> = self.receiver.clone();
        let local_animation_controller = self.animation_controller.clone();

        thread::spawn(move || {
            while !local_stop_flag.load(Ordering::Relaxed) {
                // asynchronously try to get data from sender
                match local_receiver.lock().unwrap().try_recv() {
                    // new playback state found, play it
                    Ok(new_playback) => {
                        local_animation_controller.play_from_playback(new_playback);
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