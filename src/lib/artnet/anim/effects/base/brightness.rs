use crate::lib::artnet::anim::frame::AnimationFrame;
use crate::lib::artnet::anim::effects::effect::Effect;

pub struct BrightnessEffect;

impl Effect for BrightnessEffect {
    fn apply(&self, image: &Vec<u8>) -> Vec<AnimationFrame> {
        let multipliers = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4];
        let mut result = vec![];

        for m in multipliers {
            result.push(AnimationFrame::new(&(image.iter().map(|x| (*x as f32 * m) as u8).collect())));
        }

        result
    }
}