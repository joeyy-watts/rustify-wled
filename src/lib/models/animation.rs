use crate::lib::artnet::anim::effects::base::effect::RenderedEffect;
use super::frame::AnimationFrame;

static VALUES_PER_PIXEL: usize = 3;

#[derive(Clone)]
pub struct Animation {
    pub frames_loop: Vec<AnimationFrame>,
    // optional in/out transition frames
    pub frames_in: Option<Vec<AnimationFrame>>,
    pub frames_out: Option<Vec<AnimationFrame>>,
    pub target: String,
    image: Vec<u8>,
}

impl Animation {
    pub fn new(target: String, image: Vec<u8>, effect: RenderedEffect) -> Self {
        let frames_loop = effect.apply(&image);
        Self { frames_loop, frames_in: None, frames_out: None, target, image }
    }

    pub fn add_transition_in(&mut self, effect: RenderedEffect) {
        self.frames_in = Some(effect.apply(&self.image));
    }

    pub fn add_transition_out(&mut self, effect: RenderedEffect) {
        self.frames_out = Some(effect.apply(&self.image));
    }

    ///
    /// Returns the number of pixels in a single frame of the animation
    pub fn get_frame_pixels(&self) -> u16 {
        (self.frames_loop.clone().get(0).unwrap().data.len() / VALUES_PER_PIXEL) as u16
    }
}