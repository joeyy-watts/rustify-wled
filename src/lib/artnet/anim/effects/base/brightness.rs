use crate::lib::artnet::anim::frame::AnimationFrame;
use crate::lib::artnet::anim::effects::effect::Effect;
use self::super::math::Math;

pub struct SinBrightnessEffect {
    pub period: f64,
    pub amplitude: f64,
    pub offset: f64,

}

impl Effect for SinBrightnessEffect {
    fn apply(&self, image: &Vec<u8>, target_fps: &u8) -> Vec<AnimationFrame> {
        let mut result = vec![];
        let num_factors = (((f64::from(*target_fps) * self.period).round())) as u16;

        // calculate multipliers from sine wave
        for i in 0..num_factors {
            let multiplier = Math::sin_wave((i as f64 / num_factors as f64), self.amplitude, self.period, self.offset, 1.0);
            result.push(AnimationFrame::new(&(image.iter().map(|x| (*x as f64 * multiplier) as u8).collect())));
        }

        result
    }
}