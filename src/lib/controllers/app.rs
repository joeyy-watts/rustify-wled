use rocket::futures::future::Either;
use rocket::http::Status;
use rocket::response::Redirect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use rspotify::ClientError;

use crate::lib::artnet::anim::animation::Animation;
use crate::lib::artnet::anim::effects::base::brightness::SinBrightnessEffect;
use crate::utils::image::get_image_pixels;

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
    pub fn new(target: String, size: (u16, u16)) -> ApplicationController {
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
        // TODO: if token exists but is expired, refresh token
        if self.spotify_controller.get_token().is_none() {
            let auth_url = self.spotify_controller.get_authorize_url();

            // redirect to Spotify auth
            Ok(Either::Left(Redirect::to(auth_url)))
        } else {
            // start listening loop
            self.spotify_controller.start_listening();
            self.start_loop();

            Ok(Either::Right("start!".to_string()))
        }
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.spotify_controller.stop_listening();
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
                let new_playback = local_receiver.lock().unwrap().recv().unwrap();

                // play new animation
                let image = get_image_pixels(new_playback.cover_url.unwrap().as_ref(), &32, &32).unwrap();
                let effect = SinBrightnessEffect {period: 1.0, amplitude: 0.5, offset: 0.5};
                let animation = Animation::new(image, (32, 32), 30, &effect);
                local_animation_controller.play_animation(animation);
            }

            local_stop_flag.store(false, Ordering::Relaxed);
        });
    }
}