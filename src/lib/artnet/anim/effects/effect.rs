use crate::lib::artnet::anim::frame::AnimationFrame;


/// Brightness varying effect that is ready to be applied to image to generate animation
pub struct RenderedEffect {
    pub multipliers: Vec<f64>,
}

impl RenderedEffect {
    fn apply(&self, image: &Vec<u8>, target_fps: &u8) -> Vec<AnimationFrame> {
        let mut result = vec![];

        for i in 0..self.multipliers.len() {
            let frame = image.iter().map(|x| (*x as f64 * self.multipliers[i]) as u8).collect();
            result.push(AnimationFrame::new(&frame));
        }

        result
    }
}


pub trait WaveformEffect {
    fn render(&self, target_fps: &u8, slice_factor: f64, waveform_params: WaveformParameters) -> RenderedEffect {
        let num_factors = (((f64::from(*target_fps) * self.period).round())) as u16;

        let multipliers = self.calculate_multipliers(num_factors, waveform_params, Some(slice_factor));
        RenderedEffect { multipliers }
    }

    fn calculate_multipliers(&self, num_factors: u16, waveform_params: WaveformParameters, slice_factor: Option<f64>) -> Vec<f64> {
        let mut result = vec![];

        let slice_factor = slice_factor.unwrap_or(1.0);

        let range = (num_factors as f64 * slice_factor).round() as u16;

        for i in 0..range {
            let multiplier = self.math_func(waveform_params);
            result.push(multiplier);
        }

        result
    }

    fn math_func(&self, waveform_params: WaveformParameters) -> f64;
}

pub struct WaveformParameters {
    pub amplitude: f64,
    pub period: f64,
    pub offset: f64,
    pub exponent: f64,
}

pub struct WaveformEffectElement {
    effect: Box<dyn WaveformEffect>,
    parameters: WaveformParameters,
    // 0.0 - 1.0, how much of the waveform effect is needed
    slice_factor: f64,
}


/// EffectBuilder
/// 
/// TODO: add docs here
pub struct EffectBuilder {
    target_fps: u8,
    elements: Vec<WaveformEffectElement>,
}

impl EffectBuilder {
    pub fn new(target_fps: u8) -> Self {
        Self { target_fps, elements: Vec::new() }
    }

    // TODO: currently, this is only for waveform brightness effects
    // this will add effect one after the other, they don't mix
    // i'll have to implement overlay, etc. in the future
    // which will add effects on top of each other
    pub fn add_brightness_effect(&mut self, effect: dyn WaveformEffect, parameters: WaveformParameters, slice_factor: f64) {
        self.elements.push(WaveformEffectElement { effect, parameters, slice_factor });
    }

    // EffectElement also needs to have the waveform already with the parameters, just the struct
    pub fn build(&self) -> RenderedEffect {
        let mut result = vec![];

        // calculate the multiplier slices for each effect
        for element in self.elements {
            // for each render we need the parameters, and the slice factor
            result.push(element.effect.render(&self.target_fps, element.slice_factor, element.parameters));
        }

        RenderedEffect { multipliers: result }
    }
}