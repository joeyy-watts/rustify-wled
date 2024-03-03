use crate::lib::artnet::anim::effects::base::effect::RenderedEffect;
use crate::lib::artnet::anim::effects::playback::PlaybackEffects;
use crate::lib::controllers::artnet::ArtNetController2D;
use crate::lib::artnet::anim::animation::Animation;
use crate::utils::image::get_image_pixels;
use std::sync::atomic::Ordering;

use super::spotify::PlaybackState;

/// Controller for playing animations to target ArtNet devices
/// 
/// `artnet_controller` - the controller for the target ArtNet device
/// `active_animation` - thread of the currently playing animation
/// 
pub struct AnimationController {
    pub size: (u8, u8),
    artnet_controller: ArtNetController2D,
}

impl AnimationController {
    pub fn new(target: String, size: (u8, u8)) -> Self {
        let artnet_controller = ArtNetController2D::new(target, size);
        Self { size, artnet_controller }
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
    pub fn play_from_playback(&self, playback: PlaybackState) {
        // if some animation is already playing, stop it
        if self.artnet_controller.is_playing.load(Ordering::Relaxed) {
            self.artnet_controller.stop_animation();
            
            while self.artnet_controller.is_playing.load(Ordering::Relaxed) {
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

        self.artnet_controller.send_animation(animation, frame_interval);
    }

    pub fn stop_animation(&self) {
        self.artnet_controller.stop_animation();
    }
}
