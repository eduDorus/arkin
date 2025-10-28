use std::collections::HashMap;

use tracing::{error, warn};

/// Basic statistical functions
pub fn min(data: &[f64]) -> f64 {
    // data.iter().fold(|| f64::MAX, |a, &b| a.min(b)).reduce(|| f64::MAX, f64::min)
    data.iter().cloned().fold(f64::INFINITY, f64::min)
}
pub fn max(data: &[f64]) -> f64 {
    // data.iter().fold(|| f64::MIN, |a, &b| a.max(b)).reduce(|| f64::MIN, f64::max)
    data.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
}

pub fn absolut_range(data: &[f64]) -> f64 {
    max(data) - min(data)
}

pub fn relative_range(data: &[f64]) -> f64 {
    absolut_range(data) / mean(data)
}

pub fn relative_position(data: &[f64]) -> f64 {
    let value = if let Some(last) = data.last() {
        *last
    } else {
        return f64::NAN;
    };
    let min = min(data);
    let max = max(data);
    (value - min) / (max - min)
}

pub fn sum(data: &[f64]) -> f64 {
    data.iter().sum()
}

pub fn sum_positive(data: &[f64]) -> f64 {
    data.iter().filter(|&&x| x > 0.0).sum()
}

pub fn sum_negative(data: &[f64]) -> f64 {
    data.iter().filter(|&&x| x < 0.0).sum()
}

pub fn sum_abs(data: &[f64]) -> f64 {
    data.iter().map(|x| x.abs()).sum()
}

pub fn sum_abs_positive(data: &[f64]) -> f64 {
    data.iter().filter(|&&x| x > 0.0).map(|x| x.abs()).sum()
}

pub fn sum_abs_negative(data: &[f64]) -> f64 {
    data.iter().filter(|&&x| x < 0.0).map(|x| x.abs()).sum()
}

pub fn mean(data: &[f64]) -> f64 {
    let sum: f64 = sum(data);
    let n = data.len() as f64;
    sum / n
}

/// Calculate weighted mean (e.g., VWAP)
/// values: the values to average (e.g., prices)
/// weights: the weights for each value (e.g., volumes or notional)
pub fn weighted_mean(values: &[f64], weights: &[f64]) -> f64 {
    if values.len() != weights.len() {
        warn!("weighted_mean: values and weights must have the same length");
        return f64::NAN;
    }

    if values.is_empty() {
        return f64::NAN;
    }

    // Calculate sum of (value * weight) and sum of weights
    // let (weighted_sum, total_weight): (f64, f64) = values
    //     .iter()
    //     .zip(weights.iter())
    //     .map(|(&v, &w)| (v * w.abs(), w.abs())) // Use absolute value of weights
    //     .reduce(|| (0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));
    let (weighted_sum, total_weight) = values
        .iter()
        .zip(weights.iter())
        .fold((0.0, 0.0), |(sum, total), (&v, &w)| (sum + v * w.abs(), total + w.abs()));

    if total_weight == 0.0 {
        warn!("weighted_mean: total weight is zero");
        return f64::NAN;
    }

    weighted_sum / total_weight
}

pub fn median(data: &[f64]) -> f64 {
    let mut sorted_data = data.to_vec();
    sorted_data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted_data.len() / 2;
    if sorted_data.len().is_multiple_of(2) {
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
    let max_freq = *freq_map.values().max().unwrap_or(&0);
    freq_map
        .into_iter()
        .filter(|&(_, freq)| freq == max_freq)
        .map(|(bits, _)| f64::from_bits(bits))
        .collect()
}

/// Distribution Metrics
pub fn variance(data: &[f64]) -> f64 {
    let mean = mean(data);
    let sum_sq_diff: f64 = data.iter().map(|x| (x - mean).powi(2)).sum();
    let n = data.len() as f64 - 1.0;
    sum_sq_diff / n
}

pub fn std_dev(data: &[f64]) -> f64 {
    variance(data).sqrt()
}

/// Annualize volatility based on the interval of the data
/// For crypto 1-minute data: 365 days × 24 hours × 60 minutes = 525,600 periods/year
pub fn annualized_volatility(data: &[f64]) -> f64 {
    let period_std_dev = std_dev(data);
    // Crypto trades 24/7: 365 days × 24 hours × 60 minutes per year
    period_std_dev * (525_600.0f64).sqrt()
}

pub fn skew(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    let mean = mean(data);
    let std_dev = std_dev(data);
    let sum_cube_diff: f64 = data.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum();
    (n / ((n - 1.0) * (n - 2.0))) * sum_cube_diff
}

pub fn kurtosis(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    let mean = mean(data);
    let std_dev = std_dev(data);
    let sum_fourth_diff: f64 = data.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum();
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
pub fn abs_change(value: f64, prev_value: f64) -> f64 {
    value - prev_value
}

pub fn pct_change(value: f64, prev_value: f64) -> f64 {
    (value / prev_value) - 1.0
}

pub fn log_change(value: f64, prev_value: f64) -> f64 {
    value.ln() - prev_value.ln()
}

pub fn difference(value: f64, prev_value: f64) -> f64 {
    value - prev_value
}

/// Relationship Metrics
pub fn covariance(data1: &[f64], data2: &[f64]) -> f64 {
    let mean1 = mean(data1);
    let mean2 = mean(data2);
    let sum = data1.iter().zip(data2).map(|(&x, &y)| (x - mean1) * (y - mean2)).sum::<f64>();
    sum / (data1.len() as f64 - 1.0)
}

pub fn correlation(data1: &[f64], data2: &[f64]) -> f64 {
    covariance(data1, data2) / (std_dev(data1) * std_dev(data2))
}

pub fn cosine_similarity(data1: &[f64], data2: &[f64]) -> f64 {
    let dot_product: f64 = data1.iter().zip(data2).map(|(&x, &y)| x * y).sum();
    let norm1: f64 = data1.iter().map(|&x| x.powi(2)).sum::<f64>().sqrt();
    let norm2: f64 = data2.iter().map(|&x| x.powi(2)).sum::<f64>().sqrt();
    dot_product / (norm1 * norm2)
}

pub fn beta(data1: &[f64], data2: &[f64]) -> f64 {
    let cov = covariance(data1, data2);
    let var = variance(data2);
    cov / var
}

/// Elasticity
/// It shows how responsive delta value 1 is to delta value 2 in percentage terms, which is great for comparing across different scales or contexts.
/// Elasticity = % change in data1 / % change in data2
pub fn elasticity(data1: f64, data2: f64) -> f64 {
    data1 / data2
}

/// Regression Analysis
/// Here we do a simple regression between data1 dependent vs data2 independent variable
/// Returns beta the regression coefficient
// pub fn regression_analysis(data1: &[f64], data2: &[f64]) -> f64 {}

pub fn autocorrelation(data: &[f64], k: usize) -> f64 {
    let mean = mean(data);
    let sum_sq_diff: f64 = data.iter().map(|x| (x - mean).powi(2)).sum(); // Correct denominator
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
/// This can be used to compare dominance of one or another. Mostly used for quantities but could also be used on prices.
pub fn imbalance(value_1: f64, value_2: f64) -> f64 {
    (value_1 - value_2) / (value_1 + value_2)
}

pub fn coefficient_of_variation(data: &[f64]) -> f64 {
    let mean_val = mean(data);
    if mean_val == 0.0 {
        error!("Mean is zero, coefficient of variation is undefined");
        return f64::NAN;
    }
    std_dev(data) / mean_val
}

// Linear interpolation function
pub fn interp(x: f64, xp: &[f64], fp: &[f64]) -> f64 {
    assert!(
        xp.len() == fp.len() && xp.len() >= 2,
        "xp and fp must have same length and at least 2 elements"
    );
    if x.is_nan() {
        warn!("Input to interpolation is NaN, returning NaN");
        return f64::NAN; // Return NaN if input is NaN
    }
    if x <= xp[0] {
        fp[0] // Return the lower bound if x is below the smallest quantile
    } else if x >= xp[xp.len() - 1] {
        fp[fp.len() - 1] // Return the upper bound if x is above the largest quantile
    } else {
        let i = xp.iter().position(|&v| v > x).unwrap() - 1;
        let x0 = xp[i];
        let x1 = xp[i + 1];
        let f0 = fp[i];
        let f1 = fp[i + 1];
        f0 + (x - x0) * (f1 - f0) / (x1 - x0) // Linear interpolation
    }
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
        assert_eq!(absolut_range(&data), 4.0, "range failed");

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
        assert_eq!(difference(1.0, 0.5), 0.5, "difference failed");

        // Test inbalance
        assert!((imbalance(10.0, 5.0) - 0.3333333333).abs() < 1e-10, "inbalance failed");

        // Test quantile (median)
        assert_eq!(quantile(&data, 0.5), 3.0, "quantile failed");

        // Test IQR
        assert_eq!(iqr(&data), 2.0, "iqr failed");

        // Test coefficient of variation
        assert!(
            (coefficient_of_variation(&data) - 0.5270462767).abs() < 1e-10,
            "coef_variation failed"
        );

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
