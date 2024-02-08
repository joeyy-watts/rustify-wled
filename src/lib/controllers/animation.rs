use crate::lib::controllers::artnet::ArtNetController2D;
use crate::lib::artnet::anim::animation::Animation;


/// Controller for playing animations to target ArtNet devices
/// 
/// `artnet_controller` - the controller for the target ArtNet device
/// `active_animation` - thread of the currently playing animation
/// 
pub struct AnimationController {
    pub active_animation: Option<Animation>,
    artnet_controller: ArtNetController2D,
}

impl AnimationController {
    pub fn new(target: String, dimensions: (u16, u16)) -> Self {
        let artnet_controller = ArtNetController2D::new(target, dimensions);
        Self { 
            active_animation: None, 
            artnet_controller: artnet_controller,
        }
    }

    /// Plays the given animation to the target device
    /// 
    /// `animation` - the animation to be played
    /// 
    /// Returns:
    ///     A Result indicating the success of the operation
    /// 
    pub fn play_animation(&mut self, animation: Animation) {
        // if some animation is already playing, stop it
        if !self.active_animation.is_none() {
            self.stop_animation();
        }

        self.active_animation = Some(animation);

        let frame_interval = 1.0 / self.active_animation.as_ref().unwrap().target_fps as f64;

        self.artnet_controller.send_frames(self.active_animation.as_ref().unwrap().frames.clone(), frame_interval);
    }

    pub fn stop_animation(&self) {
        self.artnet_controller.stop_animation();
    }
}
