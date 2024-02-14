use crate::lib::artnet::anim::frame::AnimationFrame;
use crate::lib::artnet::anim::effects::effect::{WaveformEffect, WaveformParameters};
use self::super::math::Math;


struct SinEffect;
struct TruncSinEffect;

impl WaveformEffect for SinEffect {
    fn math_func(&self, waveform_params: WaveformParameters) -> f64 {
        Math::sin_wave(waveform_params.amplitude, waveform_params.period, waveform_params.offset, waveform_params.exponent)
    }
}

impl WaveformEffect for TruncSinEffect {
    fn math_func(&self, waveform_params: WaveformParameters) -> f64 {
        Math::trunc_sin_wave(waveform_params.amplitude, waveform_params.period, waveform_params.offset, waveform_params.exponent)
    }
}