/// Static effect to display static image
/// 
use crate::lib::artnet::anim::frame::AnimationFrame;

pub struct StaticEffect {

}

impl StaticEffect {
    pub fn new() -> Self {
        Self {}
    }

    pub fn apply(&self, image: Vec<u8>) -> Vec<AnimationFrame> {
        let frames = vec![image; 10];
        let mut result = vec![];

        for frame in frames {
            result.push(AnimationFrame::new(&frame));
        }

        result
    }
}