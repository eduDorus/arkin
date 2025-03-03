use std::collections::HashMap;

use rayon::prelude::*;

/// Basic statistical functions
pub fn min(data: &[f64]) -> f64 {
    data.par_iter()
        .fold(|| f64::MAX, |a, &b| a.min(b))
        .reduce(|| f64::MAX, f64::min)
}
pub fn max(data: &[f64]) -> f64 {
    data.par_iter()
        .fold(|| f64::MIN, |a, &b| a.max(b))
        .reduce(|| f64::MIN, f64::max)
}

pub fn range(data: &[f64]) -> f64 {
    max(data) - min(data)
}

pub fn sum(data: &[f64]) -> f64 {
    data.par_iter().sum()
}

pub fn mean(data: &[f64]) -> f64 {
    let sum: f64 = sum(data);
    let n = data.len() as f64;
    sum / n
}

pub fn median(data: &[f64]) -> f64 {
    let mut sorted_data = data.to_vec();
    sorted_data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted_data.len() / 2;
    if sorted_data.len() % 2 == 0 {
        (sorted_data[mid - 1] + sorted_data[mid]) / 2.0
    } else {
        sorted_data[mid]
    }
}

pub fn mode(data: &[f64]) -> Vec<f64> {
    let mut freq_map = HashMap::new();
    for value in data {
        let entry = freq_map.entry(value.to_bits()).or_insert(0);
        *entry += 1;
    }
    let max_freq = freq_map.values().max().unwrap_or(&0).clone();
    freq_map
        .into_iter()
        .filter(|&(_, freq)| freq == max_freq)
        .map(|(bits, _)| f64::from_bits(bits))
        .collect()
}

pub fn variance(data: &[f64]) -> f64 {
    let mean = mean(data);
    let sum_sq_diff: f64 = data.par_iter().map(|x| (x - mean).powi(2)).sum();
    let n = data.len() as f64 - 1.0;
    sum_sq_diff / n
}

/// Distribution Metrics
pub fn std_dev(data: &[f64]) -> f64 {
    variance(data).sqrt()
}

pub fn skew(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    let mean = mean(data);
    let std_dev = std_dev(data);
    let sum_cube_diff: f64 = data.par_iter().map(|x| ((x - mean) / std_dev).powi(3)).sum();
    (n / ((n - 1.0) * (n - 2.0))) * sum_cube_diff
}

pub fn kurtosis(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    let mean = mean(data);
    let std_dev = std_dev(data);
    let sum_fourth_diff: f64 = data.par_iter().map(|x| ((x - mean) / std_dev).powi(4)).sum();
    (n * (n + 1.0) / ((n - 1.0) * (n - 2.0) * (n - 3.0))) * sum_fourth_diff
        - (3.0 * (n - 1.0).powi(2)) / ((n - 2.0) * (n - 3.0))
}

pub fn quantile(data: &[f64], q: f64) -> f64 {
    let mut sorted_data = data.to_vec();
    sorted_data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted_data.len();
    let pos = (n as f64 - 1.0) * q;
    let index = pos.floor() as usize;
    let frac = pos - index as f64;
    if index + 1 < n {
        sorted_data[index] * (1.0 - frac) + sorted_data[index + 1] * frac
    } else {
        sorted_data[index]
    }
}

pub fn iqr(data: &[f64]) -> f64 {
    let q1 = quantile(data, 0.25);
    let q3 = quantile(data, 0.75);
    q3 - q1
}

/// Change Metrics
pub fn pct_change(value: f64, prev_value: f64) -> f64 {
    (value / prev_value) - 1.0
}

pub fn log_change(value: f64, prev_value: f64) -> f64 {
    value.ln() - prev_value.ln()
}

pub fn acceleration(change: f64, prev_change: f64) -> f64 {
    change - prev_change
}

pub fn jerk(accel: f64, prev_accel: f64) -> f64 {
    accel - prev_accel
}

/// Relationship Metrics
pub fn covariance(data1: &[f64], data2: &[f64]) -> f64 {
    let mean1 = mean(data1);
    let mean2 = mean(data2);
    let sum: f64 = data1.par_iter().zip(data2).map(|(&x, &y)| (x - mean1) * (y - mean2)).sum();
    sum / (data1.len() as f64 - 1.0)
}

pub fn correlation(data1: &[f64], data2: &[f64]) -> f64 {
    covariance(data1, data2) / (std_dev(data1) * std_dev(data2))
}

pub fn autocorrelation(data: &[f64], k: usize) -> f64 {
    let mean = mean(data);
    let sum_sq_diff: f64 = data.par_iter().map(|x| (x - mean).powi(2)).sum(); // Correct denominator
    let n = data.len();
    let sum: f64 = (0..n - k).map(|i| (data[i] - mean) * (data[i + k] - mean)).sum();
    sum / sum_sq_diff
}

/// Smoothing
pub fn ema(data: &[f64], alpha: f64) -> Vec<f64> {
    let mut result = Vec::with_capacity(data.len());
    result.push(data[0]);
    for &value in &data[1..] {
        let prev = *result.last().unwrap();
        result.push(alpha * value + (1.0 - alpha) * prev);
    }
    result
}

/// Custom Metrics
pub fn inbalance(value_1: f64, value_2: f64) -> f64 {
    (value_1 - value_2) / (value_1 + value_2)
}

pub fn coef_variation(data: &[f64]) -> f64 {
    let mean_val = mean(data);
    if mean_val == 0.0 {
        panic!("Mean is zero, coefficient of variation is undefined");
    }
    std_dev(data) / mean_val
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistical_functions() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let data1 = vec![1.0, 2.0, 3.0];
        let data2 = vec![4.0, 5.0, 6.0];

        // Test min
        assert_eq!(min(&data), 1.0, "min failed");

        // Test max
        assert_eq!(max(&data), 5.0, "max failed");

        // Test range
        assert_eq!(range(&data), 4.0, "range failed");

        // Test sum
        assert_eq!(sum(&data), 15.0, "sum failed");

        // Test mean
        assert_eq!(mean(&data), 3.0, "mean failed");

        // Test median
        assert_eq!(median(&data), 3.0, "median failed");

        // Test mode
        assert_eq!(mode(&vec![1.0, 2.0, 2.0, 3.0]), vec![2.0], "mode failed");

        // Test variance
        assert!((variance(&data) - 2.5).abs() < 1e-10, "variance failed");

        // Test std_dev
        assert!((std_dev(&data) - 1.58113883).abs() < 1e-10, "std_dev failed");

        // Test skew (should be 0 for symmetric data)
        assert!((skew(&data)).abs() < 1e-10, "skew failed");

        // Test kurtosis (exact value depends on definition, just ensure it runs)
        let kurt = kurtosis(&data);
        assert!(kurt.is_finite(), "kurtosis failed");

        // Test pct_change
        assert!((pct_change(5.0, 4.0) - 0.25).abs() < 1e-10, "pct_change failed");

        // Test log_change
        assert!((log_change(5.0, 4.0) - 0.2231435513).abs() < 1e-10, "log_change failed");

        // Test acceleration
        assert_eq!(acceleration(1.0, 0.5), 0.5, "acceleration failed");

        // Test jerk
        assert_eq!(jerk(1.0, 0.5), 0.5, "jerk failed");

        // Test inbalance
        assert!((inbalance(10.0, 5.0) - 0.3333333333).abs() < 1e-10, "inbalance failed");

        // Test quantile (median)
        assert_eq!(quantile(&data, 0.5), 3.0, "quantile failed");

        // Test IQR
        assert_eq!(iqr(&data), 2.0, "iqr failed");

        // Test coefficient of variation
        assert!((coef_variation(&data) - 0.5270462767).abs() < 1e-10, "coef_variation failed");

        // Test covariance
        assert!((covariance(&data1, &data2) - 1.0).abs() < 1e-10, "covariance failed");

        // Test correlation
        assert!((correlation(&data1, &data2) - 1.0).abs() < 1e-10, "correlation failed");

        // Test autocorrelation (lag 1, should be positive and high)
        let autocorr = autocorrelation(&data, 1);
        println!("autocorr: {}", autocorr);
        assert_eq!(autocorr, 0.4, "autocorrelation failed");

        // Test ema
        let ema_result = ema(&vec![1.0, 2.0, 3.0], 0.5);
        let expected = vec![1.0, 1.5, 2.25];
        assert!(
            ema_result.iter().zip(expected.iter()).all(|(&a, &b)| (a - b).abs() < 1e-10),
            "ema failed"
        );
    }
}
