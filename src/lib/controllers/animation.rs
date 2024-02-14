use crate::lib::controllers::artnet::ArtNetController2D;
use crate::lib::artnet::anim::animation::Animation;
use std::sync::atomic::Ordering;

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
    pub fn play_animation(&self, animation: Animation) {
        // if some animation is already playing, stop it gracefuly first
        // TODO: dynamic transitions; if play -> pause change gracefully, if change cover -> change immediately?
        if self.artnet_controller.is_playing.load(Ordering::Relaxed) {
            self.artnet_controller.stop_animation();

                while self.artnet_controller.is_playing.load(Ordering::Relaxed) {
                    // wait for the the animation to stop gracefully
                }  
        }

        let frame_interval = 1.0 / animation.target_fps as f64;

        self.artnet_controller.send_frames(animation.frames, frame_interval);
    }

    pub fn stop_animation(&self) {
        self.artnet_controller.stop_animation();
    }
}
