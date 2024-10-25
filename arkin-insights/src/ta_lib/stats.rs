use rust_decimal::prelude::*;
use tracing::warn;

/// Calculates the arithmetic mean (average) of a slice of `Decimal` values.
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values.
///
/// # Returns
///
/// * `Option<Decimal>` - The mean value, or `None` if the data slice is empty.
///
/// # Example
///
/// ```
/// let data = vec![dec!(1.0), dec!(2.0), dec!(3.0)];
/// let mean_value = mean(&data).unwrap();
/// ```
pub fn mean(data: &[Decimal]) -> Option<Decimal> {
    if data.is_empty() {
        warn!("Data slice is empty, cannot calculate mean");
        return None;
    }

    let sum: Decimal = data.iter().sum();
    let count = Decimal::from(data.len());

    Some(sum / count)
}

/// Calculates the percentage change of a slice of `Decimal` values.
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values.
///
/// # Returns
///
/// * `Option<Vec<Decimal>>` - The percentage change values, or `None` if the data slice is empty.
///
/// # Example
///
/// ```
/// let data = vec![dec!(1.0), dec!(2.0), dec!(3.0)];
/// let pct_change_values = percentage_change(&data).unwrap();
/// ```
pub fn pct_change(data: &[Decimal]) -> Option<Vec<Decimal>> {
    if data.len() < 2 {
        warn!("Data slice is to small, cannot calculate percentage change");
        return None;
    }

    let mut result = Vec::with_capacity(data.len() - 1);
    for i in 1..data.len() {
        let prev = data[i - 1];
        let curr = data[i];
        let pct_change = if !prev.is_zero() {
            (curr - prev) / prev
        } else {
            Decimal::ZERO
        };
        result.push(pct_change);
    }

    Some(result)
}

/// Calculates the variance of a slice of `Decimal` values.
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values.
///
/// # Returns
///
/// * `Option<Decimal>` - The variance, or `None` if the data slice has fewer than two elements.
///
/// # Example
///
/// ```
/// let data = vec![dec!(1.0), dec!(2.0), dec!(3.0)];
/// let variance_value = variance(&data).unwrap();
/// ```
pub fn variance(data: &[Decimal]) -> Option<Decimal> {
    if data.len() < 2 {
        warn!("Data slice has fewer than two elements, cannot calculate variance");
        return None;
    }

    let data_mean = mean(data)?;
    let squared_diffs: Decimal = data.iter().map(|value| (*value - data_mean).powi(2)).sum();
    let count = Decimal::from(data.len() - 1); // Sample variance

    Some(squared_diffs / count)
}

/// Calculates the standard deviation of a slice of `Decimal` values.
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values.
///
/// # Returns
///
/// * `Option<Decimal>` - The standard deviation, or `None` if the data slice has fewer than two elements.
///
/// # Example
///
/// ```
/// let data = vec![dec!(1.0), dec!(2.0), dec!(3.0)];
/// let std_dev_value = standard_deviation(&data).unwrap();
/// ```
pub fn std_dev(data: &[Decimal]) -> Option<Decimal> {
    let var = variance(data)?;
    var.sqrt()
}

/// Calculates Pearson's Correlation Coefficient between two slices of `Decimal` values.
///
/// # Arguments
///
/// * `x` - A slice of `Decimal` values representing the first dataset.
/// * `y` - A slice of `Decimal` values representing the second dataset.
///
/// # Returns
///
/// * `Option<Decimal>` - The correlation coefficient, or `None` if calculation is not possible.
///
/// # Example
///
/// ```
/// let x = vec![dec!(1.0), dec!(2.0), dec!(3.0)];
/// let y = vec![dec!(2.0), dec!(4.0), dec!(6.0)];
/// let correlation = pearson_correlation(&x, &y).unwrap();
/// ```
pub fn pearson_correlation(x: &[Decimal], y: &[Decimal]) -> Option<Decimal> {
    if x.len() != y.len() {
        warn!("Input data slices have different lengths, cannot calculate Pearson's correlation");
        return None;
    }

    if x.len() < 2 {
        warn!("Data slices have fewer than two elements, cannot calculate Pearson's correlation");
        return None;
    }

    let n = Decimal::from(x.len());
    let sum_x: Decimal = x.iter().sum();
    let sum_y: Decimal = y.iter().sum();
    let sum_x_sq: Decimal = x.iter().map(|&xi| xi * xi).sum();
    let sum_y_sq: Decimal = y.iter().map(|&yi| yi * yi).sum();
    let sum_xy: Decimal = x.iter().zip(y.iter()).map(|(&xi, &yi)| xi * yi).sum();

    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator_left = n * sum_x_sq - sum_x * sum_x;
    let denominator_right = n * sum_y_sq - sum_y * sum_y;

    let denominator = (denominator_left * denominator_right).sqrt()?;

    if denominator.is_zero() {
        warn!("Denominator is zero in Pearson's correlation calculation");
        return None;
    }

    Some(numerator / denominator)
}

/// Calculates the covariance between two slices of `Decimal` values.
///
/// # Arguments
///
/// * `x` - A slice of `Decimal` values representing the first dataset.
/// * `y` - A slice of `Decimal` values representing the second dataset.
///
/// # Returns
///
/// * `Option<Decimal>` - The covariance, or `None` if calculation is not possible.
///
/// # Example
///
/// ```
/// let x = vec![dec!(1.0), dec!(2.0), dec!(3.0)];
/// let y = vec![dec!(2.0), dec!(4.0), dec!(6.0)];
/// let cov = covariance(&x, &y).unwrap();
/// ```
pub fn covariance(x: &[Decimal], y: &[Decimal]) -> Option<Decimal> {
    if x.len() != y.len() {
        warn!("Input data slices have different lengths, cannot calculate covariance");
        return None;
    }

    if x.len() < 2 {
        warn!("Data slices have fewer than two elements, cannot calculate covariance");
        return None;
    }

    let n = Decimal::from(x.len());
    let mean_x = mean(x)?;
    let mean_y = mean(y)?;

    let cov_sum: Decimal = x.iter().zip(y.iter()).map(|(&xi, &yi)| (xi - mean_x) * (yi - mean_y)).sum();
    let covariance = cov_sum / (n - Decimal::ONE);

    Some(covariance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_mean() {
        let data = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)];
        let result = mean(&data).unwrap();
        assert_eq!(result, dec!(3.0));
    }

    #[test]
    fn test_variance() {
        let data = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)];
        let result = variance(&data).unwrap();
        assert_eq!(result, dec!(2.5));
    }

    #[test]
    fn test_standard_deviation() {
        let data = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)];
        let result = std_dev(&data).unwrap();
        // Since sqrt(2.5) ≈ 1.5811
        assert_eq!(result.round_dp(4), dec!(1.5811));
    }

    #[test]
    fn test_pearson_correlation_perfect_positive() {
        let x = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)];
        let y = vec![dec!(2.0), dec!(4.0), dec!(6.0), dec!(8.0), dec!(10.0)];
        let result = pearson_correlation(&x, &y).unwrap();
        assert_eq!(result, Decimal::ONE); // Perfect positive correlation
    }

    #[test]
    fn test_covariance() {
        let x = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)];
        let y = vec![dec!(2.0), dec!(4.0), dec!(6.0), dec!(8.0), dec!(10.0)];
        let result = covariance(&x, &y).unwrap();
        assert_eq!(result, dec!(5.0));
    }

    #[test]
    fn test_empty_data_mean() {
        let data: Vec<Decimal> = vec![];
        let result = mean(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_insufficient_data_variance() {
        let data = vec![dec!(1.0)];
        let result = variance(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_insufficient_data_std_dev() {
        let data = vec![dec!(1.0)];
        let result = std_dev(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_pearson_correlation_zero_correlation() {
        let x = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)];
        let y = vec![dec!(2.0); 5];
        let result = pearson_correlation(&x, &y);
        assert!(result.is_none()); // No correlation
    }

    #[test]
    fn test_covariance_zero() {
        let x = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)];
        let y = vec![dec!(2.0); 5];
        let result = covariance(&x, &y).unwrap();
        assert_eq!(result, dec!(0.0));
    }
}
