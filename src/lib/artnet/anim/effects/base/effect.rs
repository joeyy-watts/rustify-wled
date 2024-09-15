use crate::lib::{artnet::anim::effects::waveforms::waveform::{WaveformEffect, WaveformParameters}, models::frame::AnimationFrame};
use crate::settings::SETTINGS;

/// Brightness varying effect that is ready to be applied to image to generate animation
pub struct RenderedEffect {
    pub multipliers: Vec<f64>,
}

impl RenderedEffect {
    pub fn apply(&self, image: &Vec<u8>) -> Vec<AnimationFrame> {
        let mut result = vec![];

        for i in 0..self.multipliers.len() {
            let frame = image.iter().map(|x| (*x as f64 * self.multipliers[i]) as u8).collect();
            result.push(AnimationFrame::new(&frame));
        }

        result
    }
}

pub struct WaveformEffectElement {
    effect: Box<dyn WaveformEffect>,
    parameters: WaveformParameters,
    // 0.0 - 1.0, how much of the waveform effect is needed
    // TODO: either make this a tuple to get specific part of slice (start, end)
    // or have to implement horizontal offset in effects
    slice_factor: f64,
}


/// EffectBuilder
/// 
/// TODO: add docs here
pub struct EffectBuilder {
    elements: Vec<WaveformEffectElement>,
}

impl EffectBuilder {
    pub fn new() -> Self {
        Self { elements: Vec::new() }
    }

    // TODO: currently, this is only for waveform brightness effects
    // this will add effect one after the other, they don't mix
    // i'll have to implement overlay, etc. in the future
    // which will add effects on top of each other
    pub fn add_brightness_effect(&mut self, effect: impl WaveformEffect + 'static, parameters: WaveformParameters, slice_factor: f64) {
        self.elements.push(WaveformEffectElement { effect: Box::new(effect), parameters, slice_factor });
    }

    // EffectElement also needs to have the waveform already with the parameters, just the struct
    pub fn build(&self) -> RenderedEffect {
        let mut result = Vec::new();

        // calculate the multiplier slices for each effect
        for element in &self.elements {
            // for each render we need the parameters, and the slice factor
            result.extend(element.effect.render(element.slice_factor, element.parameters));
        }

        RenderedEffect { multipliers: result }
    }
}