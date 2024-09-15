use crate::lib::artnet::anim::effects::base::effect::RenderedEffect;
use crate::lib::artnet::anim::effects::playback::PlaybackEffects;
use crate::lib::controllers::artnet::ArtNetController;
use crate::lib::models::animation::Animation;
use crate::lib::models::playback_state::PlaybackState;
use crate::utils::image::get_image_pixels;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use color_thief::{ColorFormat, get_palette};
use log::{info, trace};
use rocket::http::hyper::body::HttpBody;
use crate::settings::{SETTINGS, Target};
use crate::utils::network::resolve_ip;

/////////////////////////////////////////
/// Public Structs/Enums
/////////////////////////////////////////
pub struct AnimationControllerConfig {
    pub target: String,
    pub size: (u8, u8),
}

#[derive(Clone)]
pub enum AnimationControllerMessage {
    Animate(PlaybackState),      // start playing animation
    Stop,       // stop animation
    Timeout,    // timeout animation
    Terminate,  // terminate the message loop
}


/// Controller for playing animations to target ArtNet devices
///
/// `artnet_controller` - the controller for the target ArtNet device
/// `active_animation` - thread of the currently playing animation
///
pub struct AnimationController {
    artnet_controller: Arc<ArtNetController>,
    rx_app: Arc<Mutex<Receiver<AnimationControllerMessage>>>,
}

impl AnimationController {
    pub fn new(rx_app: Receiver<AnimationControllerMessage>) -> Self {
        let artnet_controller = ArtNetController::new();

        Self {
            artnet_controller: Arc::new(artnet_controller),
            rx_app: Arc::new(Mutex::new(rx_app))
        }
    }

    pub fn start(&self) {
        let local_artnet_controller = self.artnet_controller.clone();
        let local_receiver = self.rx_app.clone();

        thread::spawn(move || {
            let mut current_playing: PlaybackState = PlaybackState::none();
            // Mutex guard for receiver's use while inside this thread
            let receiver_guard = local_receiver.lock().unwrap();

            loop {
                match receiver_guard.recv() {
                    Ok(AnimationControllerMessage::Animate(playback)) => {
                        AnimationController::play_from_playback(local_artnet_controller.as_ref(), playback.clone());
                        current_playing = playback;
                    },
                    // for handling messages when loop is not running
                    Ok(AnimationControllerMessage::Stop) => {
                        local_artnet_controller.stop_animation();
                    },
                    // timeout signal received
                    Ok(AnimationControllerMessage::Timeout) => {
                        if PlaybackState::eq(&current_playing, &PlaybackState::none()) {
                            local_artnet_controller.stop_animation();
                        }
                    },
                    // terminate the entire controller
                    Ok(AnimationControllerMessage::Terminate) => {
                        break;
                    },
                    Err(mpsc::RecvError) => {
                        // empty, do nothing
                    },
                }
            }
        });
    }

    /// Plays the given animation to the target device.
    ///
    /// If an animation is already playing, it set the stop flag, wait for it to complete,
    /// then starts the new animation.
    ///
    /// `animation` - the animation to be played
    ///
    /// Returns:
    ///     A Result indicating the success of the operation
    ///
    /// Plays animation according to the given PlaybackState
    fn play_from_playback(artnet_controller: &ArtNetController, playback: PlaybackState) {
        let image_thread = thread::spawn(move || {
            let image = get_image_pixels(playback.cover_url, &32, &32).unwrap();
            image
        });

        let effect_thread = thread::spawn(move || {
            let effect: RenderedEffect = match (playback.is_playing, playback.features) {
                (true, Some(features)) => {
                    PlaybackEffects::play_features(features)
                },
                (true, None) => {
                    PlaybackEffects::play()
                },
                (false, _) => {
                    PlaybackEffects::pause()
                }
            };
            effect
        });

        let animation_thread: JoinHandle<Vec<Animation>> = thread::spawn(move || {
            let devices = SETTINGS.read().unwrap().targets.to_vec();
            let image = image_thread.join().unwrap();
            let effect = effect_thread.join().unwrap();

            devices.iter().map(|device| {
                AnimationController::get_animation_for_device(device, &image, &effect)
            }).collect::<Vec<Animation>>()
        });

        // if some animation is already playing, stop it
        if artnet_controller.any_playing() {
            // don't stop animation until next one is rendered
            while !animation_thread.is_finished() {}
            trace!("Next animation rendered, stopping previous animation");

            artnet_controller.stop_animation();
            trace!("Set stop flag; waiting for previous animation to stop");

            // wait for all previous animation to stop before sending new one
            while artnet_controller.any_playing() {}
        }

        artnet_controller.send_animations(animation_thread.join().unwrap());
        trace!("New animation sent");
    }

    pub fn stop_animation(&self) {
        self.artnet_controller.stop_animation();
    }

    ///
    /// Renders an animation for the given device, image, and effect.
    fn get_animation_for_device(device: &Target, image: &Vec<u8>, effect: &RenderedEffect) -> Animation {
        match device.size {
            // 1-dimensional effect
            // NOTE: currently this only supports DMX mode `Single RGB`, not `Multi RGB` (one color for the entire target)
            // TODO: add support for WLED `Effect` ArtNet mode
            (_, 0) => {
                let palette = get_palette(&image, ColorFormat::Rgb, 1, 2)
                    .unwrap()  // TODO: add default palette
                    .into_iter()
                    .nth(0).unwrap();

                Animation::new(
                    resolve_ip(device.host.clone().as_str()).unwrap(),  // Clone the host to avoid moving it
                    vec![palette.r, palette.g, palette.b],        // Clone the image so it can be reused
                    effect.clone(),       // Clone the effect so it can be reused
                )
            },
            // 2-dimensional effect
            (_, _) => Animation::new(
                resolve_ip(device.host.clone().as_str()).unwrap(),  // Clone the host to avoid moving it
                image.clone(),        // Clone the image so it can be reused
                effect.clone(),       // Clone the effect so it can be reused
            ),
        }
    }
}
