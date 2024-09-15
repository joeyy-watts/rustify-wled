use rspotify::model::AudioFeatures;

use super::{base::effect::{EffectBuilder, RenderedEffect}, waveforms::{waveform::WaveformParameters, waveform_impl::{SinEffect, TruncSinEffect}}};


/// Playback effects built from effects in base module.
/// 
/// Effects here use EffectBuilder to create dynamic effects.
/// These should not contain any math functions.
pub struct PlaybackEffects;

impl PlaybackEffects {
    pub fn play() -> RenderedEffect {
        let mut builder = EffectBuilder::new();
        builder.add_brightness_effect(TruncSinEffect, WaveformParameters { amplitude: 0.5, period: 2.0, v_offset: 0.5, h_offset: 0.0, exponent: 1.0 }, 0.5);

        builder.build()
    }

    pub fn pause() -> RenderedEffect {
        let mut builder = EffectBuilder::new();

        // breathing effect
        builder.add_brightness_effect(
            TruncSinEffect, 
            WaveformParameters { amplitude: 0.3, period: 1.0, v_offset: 0.7, h_offset: 0.0, exponent: 1.0 }, 
            1.0
        );

        // full sin wave between breathe
        builder.add_brightness_effect(
            SinEffect,
            WaveformParameters { amplitude: 0.3, period: 2.0, v_offset: 0.7, h_offset: 0.0, exponent: 1.0 },
            1.0
        );

        builder.build()
    }

    pub fn play_features(features: AudioFeatures) -> RenderedEffect {
        // period is doubled since the sin wave crest needs to correspond to each beat
        let period: f64 = (1.0 / (features.tempo / (60.0 * 2.0))) as f64;
        let exponent: f64 = (features.energy * 10.0).round() as f64;

        let mut builder = EffectBuilder::new();
        builder.add_brightness_effect(
            TruncSinEffect, 
            WaveformParameters { amplitude: -0.3, period, v_offset: 0.6, h_offset: 0.0, exponent },
            1.0
        );

        builder.build()
    }
}