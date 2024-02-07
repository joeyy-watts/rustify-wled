use crate::lib::artnet::anim::frame::AnimationFrame;

pub trait Effect {
    fn apply(&self, image: &Vec<u8>, target_fps: &u8) -> Vec<AnimationFrame>;
}