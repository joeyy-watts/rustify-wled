use crate::lib::artnet::anim::effects::base::effect::RenderedEffect;
use crate::lib::artnet::anim::effects::playback::PlaybackEffects;
use crate::lib::controllers::artnet::ArtNetController2D;
use crate::lib::models::animation::Animation;
use crate::lib::models::playback_state::PlaybackState;
use crate::utils::image::get_image_pixels;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

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
    pub size: (u8, u8),
    artnet_controller: Arc<ArtNetController2D>,
    rx_app: Arc<Mutex<Receiver<AnimationControllerMessage>>>,
}

impl AnimationController {
    pub fn new(rx_app: Receiver<AnimationControllerMessage>, config: AnimationControllerConfig) -> Self {
        let artnet_controller = ArtNetController2D::new(config.target, config.size);

        Self { 
            size: config.size, 
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
    fn play_from_playback(artnet_controller: &ArtNetController2D, playback: PlaybackState) {
        // if some animation is already playing, stop it
        if artnet_controller.is_playing.load(Ordering::Relaxed) {
            artnet_controller.stop_animation();
            
            while artnet_controller.is_playing.load(Ordering::Relaxed) {
            }
        }

        let image = get_image_pixels(playback.cover_url.unwrap().as_ref(), &32, &32).unwrap();
                
        let effect: RenderedEffect = match (playback.is_playing, playback.features) {
            (true, Some(features)) => {
                PlaybackEffects::play_features(30, features)
            },
            (true, None) => {
                PlaybackEffects::play(30)
            },
            (false, _) => {
                PlaybackEffects::pause(30)
            }
        };

        let animation = Animation::new(image, 30, effect);

        let frame_interval = 1.0 / animation.target_fps as f64;

        artnet_controller.send_animation(animation, frame_interval);
    }

    pub fn stop_animation(&self) {
        self.artnet_controller.stop_animation();
    }
}
