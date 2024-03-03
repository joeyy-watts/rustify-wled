// base mathematical functions for animations
use std::f64::consts::PI;

pub struct Math;


/// Math functions for generating waveforms
/// 
/// These functions are not to be called directly, but will be called by
/// effects to generate the correct number of multipliers.
/// 
impl Math {
    pub fn sin_wave(i: f64, amplitude: f64, period: f64, vertical_offset: f64, exponent: f64) -> f64 {
        amplitude * ((2.0 * PI * i / period).sin()).powf(exponent) + vertical_offset
    }

    pub fn trunc_sin_wave(i: f64, amplitude: f64, period: f64, vertical_offset: f64, exponent: f64) -> f64 {
        amplitude * (((2.0 * PI * i / period).sin()).powf(exponent)).abs() + vertical_offset
    }

    pub fn sawtooth(i: f64, amplitude: f64, period: f64, vertical_offset: f64) -> f64 {
        2.0 * amplitude * (i - (0.5 + i).floor()) + vertical_offset
    }
}