// base mathematical functions for animations
use std::f64::consts::PI;

pub struct Math;


/// Math functions for generating waveforms
/// 
/// These functions are not to be called directly, but will be called by
/// effects to generate the correct number of multipliers.
/// 
impl Math {
    pub fn sin_wave(i: f64, amplitude: f64, period: f64, v_offset: f64, h_offset: f64, exponent: f64) -> f64 {
        amplitude * ((2.0 * PI * (i + h_offset) / period).sin()).powf(exponent) + v_offset
    }

    pub fn trunc_sin_wave(i: f64, amplitude: f64, period: f64, v_offset: f64, h_offset: f64, exponent: f64) -> f64 {
        amplitude * (((2.0 * PI * (i + h_offset) / period).sin()).powf(exponent)).abs() + v_offset
    }

    pub fn sawtooth(i: f64, amplitude: f64, period: f64, v_offset: f64, h_offset: f64) -> f64 {
        2.0 * amplitude * ((i + h_offset) - (0.5 + (i + h_offset)).floor()) + v_offset
    }
}