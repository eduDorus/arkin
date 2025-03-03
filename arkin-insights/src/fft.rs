use rayon::prelude::*;
use rustfft::{num_complex::Complex, FftPlanner};

/// Struct to hold the FFT-based features extracted from the price window.
#[derive(Debug)]
pub struct FftFeatures {
    /// The index of the dominant frequency (1-based, excluding DC component).
    pub dominant_frequency: usize,
    /// Energy in the low-frequency band (frequencies 1 to 5).
    pub low_freq_energy: f64,
    /// Energy in the medium-frequency band (frequencies 6 to 15).
    pub medium_freq_energy: f64,
    /// Energy in the high-frequency band (frequencies 16 to max_freq).
    pub high_freq_energy: f64,
    /// Spectral entropy of the normalized magnitudes (excluding DC).
    pub spectral_entropy: f64,
}

/// Computes FFT-based features from a window of price data.
///
/// # Arguments
/// * `prices` - A slice of `f64` representing the price data (minimum length of 2).
///
/// # Panics
/// Panics if the input slice `prices` has fewer than 2 elements, as returns cannot be computed.
///
/// # Returns
/// An `FftFeatures` struct containing the computed features.
///
/// # Features Computed
/// - **Dominant Frequency**: The frequency index with the highest magnitude (excluding DC).
/// - **Spectral Energy**: Sum of squared magnitudes in low (1-5), medium (6-15), and high (16-max_freq) frequency bands.
/// - **Spectral Entropy**: Entropy of the normalized magnitudes (excluding DC), computed as -∑(p * ln(p)).
///
/// # Example
/// ```rust
/// let prices = vec![100.0, 101.0, 102.0, 100.5, 103.0];
/// let features = compute_fft_features(&prices);
/// println!("Dominant Frequency: {}", features.dominant_frequency);
/// println!("Spectral Entropy: {}", features.spectral_entropy);
/// ```
pub fn compute_fft_features(data: &[f64]) -> FftFeatures {
    // Convert data to complex numbers (imaginary part = 0.0)
    let mut input: Vec<Complex<f64>> = data.par_iter().map(|&x| Complex::new(x, 0.0)).collect();

    // Plan and execute the FFT
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(data.len());
    fft.process(&mut input);

    // Step 4: Compute magnitudes up to Nyquist frequency
    let n = data.len();
    let max_freq = n / 2; // floor(N/2)
    let magnitudes: Vec<f64> = input
        .par_iter()
        .take(max_freq + 1) // Include DC up to max_freq
        .map(|c| c.norm())
        .collect();

    // Exclude DC component (frequency 0) for feature computation
    let mag_no_dc: Vec<f64> = magnitudes[1..].to_vec();

    // Step 5: Compute features

    // Dominant frequency: Index of max magnitude (1-based, since DC is excluded)
    let dominant_freq = mag_no_dc
        .par_iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, _)| i + 1)
        .unwrap_or(1); // Default to 1 if empty (won't happen since N >= 2)

    // Define frequency band limits, adjusting for small window sizes
    let low_end = 5.min(max_freq);
    let medium_end = 15.min(max_freq);
    let high_end = max_freq;

    // Compute spectral energy as sum of squared magnitudes in each band
    let low_freq_energy: f64 = mag_no_dc[0..low_end].par_iter().map(|&m| m.powi(2)).sum();
    let medium_freq_energy: f64 = if medium_end > 5 {
        mag_no_dc[5..medium_end].par_iter().map(|&m| m.powi(2)).sum()
    } else {
        0.0
    };
    let high_freq_energy: f64 = if high_end > 15 {
        mag_no_dc[15..high_end].par_iter().map(|&m| m.powi(2)).sum()
    } else {
        0.0
    };

    // Compute spectral entropy
    let sum_mag: f64 = mag_no_dc.par_iter().sum();
    let spectral_entropy = if sum_mag > 0.0 {
        let normalized_mag: Vec<f64> = mag_no_dc.par_iter().map(|&m| m / sum_mag).collect();
        -normalized_mag
            .par_iter()
            .map(|&p| if p > 0.0 { p * p.ln() } else { 0.0 })
            .sum::<f64>()
    } else {
        0.0 // Avoid division by zero; entropy is 0 if all magnitudes are 0
    };

    // Return the features in a struct
    FftFeatures {
        dominant_frequency: dominant_freq,
        low_freq_energy,
        medium_freq_energy,
        high_freq_energy,
        spectral_entropy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_fft_features() {
        // Simple test with a small window
        let prices = vec![100.0, 101.0, 102.0, 100.5, 103.0];
        let features = compute_fft_features(&prices);
        assert!(features.dominant_frequency >= 1);
        assert!(features.low_freq_energy >= 0.0);
        assert!(features.medium_freq_energy >= 0.0);
        assert!(features.high_freq_energy >= 0.0);
        assert!(features.spectral_entropy >= 0.0);
    }

    #[test]
    #[should_panic]
    fn test_insufficient_data() {
        let prices = vec![100.0];
        compute_fft_features(&prices);
    }

    #[test]
    fn test_sine_wave() {
        use std::f64::consts::PI;

        // Parameters
        let f = 1.0; // Frequency: 1 Hz
        let fs = 60.0; // Sampling frequency: 60 Hz
        let n_samples = 60; // Window size: 60 samples

        // Generate sine wave: sin(2 * pi * f / fs * n)
        let sine_wave: Vec<f64> = (0..n_samples).map(|n| (2.0 * PI * f / fs * n as f64).sin()).collect();

        // Compute FFT features
        let features = compute_fft_features(&sine_wave);

        // Verify the results
        assert_eq!(features.dominant_frequency, 1, "Dominant frequency should be 1");
        assert!(features.low_freq_energy > 0.0, "Low-frequency energy should be positive");
        assert!(features.medium_freq_energy < 1e-10, "Medium-frequency energy should be zero");
        assert!(features.high_freq_energy < 1e-10, "High-frequency energy should be zero");
        assert!(features.spectral_entropy < 0.1, "Spectral entropy should be low");
    }
}
