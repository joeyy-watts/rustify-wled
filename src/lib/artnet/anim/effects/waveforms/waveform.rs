pub trait WaveformEffect {
    fn render(&self, target_fps: &u8, slice_factor: f64, waveform_params: WaveformParameters) -> Vec<f64> {
        let num_factors = (((f64::from(*target_fps) * waveform_params.period).round())) as u16;

        let multipliers = self.calculate_multipliers(num_factors, waveform_params, Some(slice_factor));
        
        multipliers
    }

    fn calculate_multipliers(&self, num_factors: u16, waveform_params: WaveformParameters, slice_factor: Option<f64>) -> Vec<f64> {
        let mut result = vec![];

        let slice_factor = slice_factor.unwrap_or(1.0);

        let range = (num_factors as f64 * slice_factor).round() as u16;

        for i in 0..range {
            let multiplier = self.math_func(waveform_params.period * ((i as f64) / (num_factors as f64)), waveform_params);
            result.push(multiplier);
        }

        result
    }

    fn math_func(&self, i: f64, waveform_params: WaveformParameters) -> f64;
}

#[derive(Copy, Clone)]
pub struct WaveformParameters {
    pub amplitude: f64,
    pub period: f64,
    pub v_offset: f64,
    pub h_offset: f64,
    pub exponent: f64,
}