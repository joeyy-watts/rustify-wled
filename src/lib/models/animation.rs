use crate::lib::artnet::anim::effects::base::effect::RenderedEffect;
use super::frame::AnimationFrame;

#[derive(Clone)]
pub struct Animation {
    pub target_fps: u8,
    pub frames_loop: Vec<AnimationFrame>,
    // optional in/out transition frames
    pub frames_in: Option<Vec<AnimationFrame>>,
    pub frames_out: Option<Vec<AnimationFrame>>,
    image: Vec<u8>,
}

impl Animation {
    pub fn new(image: Vec<u8>, target_fps: u8, effect: RenderedEffect) -> Self {
        let frames_loop = effect.apply(&image);
        Self { target_fps, frames_loop, frames_in: None, frames_out: None, image }
    }

    pub fn add_transition_in(&mut self, effect: RenderedEffect) {
        self.frames_in = Some(effect.apply(&self.image));
    }

    pub fn add_transition_out(&mut self, effect: RenderedEffect) {
        self.frames_out = Some(effect.apply(&self.image));
    }
}