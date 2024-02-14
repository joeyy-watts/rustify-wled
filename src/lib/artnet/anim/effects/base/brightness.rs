use crate::lib::artnet::anim::effects::effect::{WaveformEffect, WaveformParameters};
use self::super::math::Math;


struct SinEffect;
struct TruncSinEffect;

impl WaveformEffect for SinEffect {
    fn math_func(&self, i: u16, waveform_params: WaveformParameters) -> f64 {
        Math::sin_wave(i as f64, waveform_params.amplitude, waveform_params.period, waveform_params.offset, waveform_params.exponent)
    }
}

impl WaveformEffect for TruncSinEffect {
    fn math_func(&self, i: u16, waveform_params: WaveformParameters) -> f64 {
        Math::trunc_sin_wave(i as f64, waveform_params.amplitude, waveform_params.period, waveform_params.offset, waveform_params.exponent)
    }
}