use super::{base::effect::{EffectBuilder, RenderedEffect}, waveforms::{waveform::WaveformParameters, waveform_impl::SawtoothEffect}};

pub struct TransitionEffects;

impl TransitionEffects {
    pub fn fade_in(target_fps: u8, fade_time: f64) -> RenderedEffect {
        let mut builder = EffectBuilder::new(target_fps);

        builder.add_brightness_effect(
            SawtoothEffect,
            WaveformParameters { amplitude: 0.5, period: fade_time, v_offset: 0.5, h_offset: 0.5, exponent: 1.0 },
            1.0
        );

        builder.build()
    }

    pub fn fade_out(target_fps: u8, fade_time: f64) -> RenderedEffect {
        let mut builder = EffectBuilder::new(target_fps);

        builder.add_brightness_effect(
            SawtoothEffect,
            WaveformParameters { amplitude: -0.5, period: fade_time, v_offset: 0.5, h_offset: 0.5, exponent: 1.0 },
            1.0
        );

        builder.build()
    }
}