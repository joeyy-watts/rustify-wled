use crate::lib::artnet::anim::frame::AnimationFrame;

pub struct BrightnessEffect {

}

impl BrightnessEffect {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn apply(&self, image: &Vec<u8>) -> Vec<AnimationFrame> {
        let multipliers = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4];
        let mut result = vec![];

        for m in multipliers {
            result.push(AnimationFrame::new(&(image.iter().map(|x| (*x as f32 * m) as u8).collect())));
        }

        result
    }
}