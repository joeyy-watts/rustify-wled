/// Static effect to display static image
/// 
use crate::lib::artnet::anim::{effects::effect::Effect, frame::AnimationFrame};

pub struct StaticEffect;

impl Effect for StaticEffect {
    fn apply(&self, image: &Vec<u8>, target_fps: &u8) -> Vec<AnimationFrame> {
        let frames = vec![image; 10];
        let mut result = vec![];

        for frame in frames {
            result.push(AnimationFrame::new(&frame));
        }

        result
    }
}