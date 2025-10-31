use itertools::izip;
use rust_decimal::prelude::*;
use tracing::warn;

use super::stats;

/// Enum representing different smoothing methods.
pub enum SmoothingMethod {
    SMA(usize),                    // Simple Moving Average with period
    EMA(usize),                    // Exponential Moving Average with period
    KAMA(usize, usize, usize),     // Kaufman's Adaptive Moving Average with period, fast_period, slow_period
    DEMA(usize),                   // Double Exponential Moving Average with period
    TEMA(usize),                   // Tripple Exponential Moving Average with period
    ALMA(usize, Decimal, Decimal), // Arnaud Legoux Moving Average with period, offset, sigma
}

impl SmoothingMethod {
    /// Calculates a series of smoothed values over the data.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice of `Decimal` values to be smoothed.
    ///
    /// # Returns
    ///
    /// * `Option<Vec<Decimal>>` - A vector of smoothed values, or `None` if calculation is not possible.
    pub fn calc(&self, data: &[Decimal]) -> Option<Vec<Decimal>> {
        match self {
            SmoothingMethod::SMA(period) => sma(data, *period),
            SmoothingMethod::EMA(period) => ema(data, *period),
            SmoothingMethod::KAMA(period, fast_period, slow_period) => kama(data, *period, *fast_period, *slow_period),
            SmoothingMethod::DEMA(period) => dema(data, *period),
            SmoothingMethod::TEMA(period) => tema(data, *period),
            SmoothingMethod::ALMA(period, offset, sigma) => alma(data, *period, *offset, *sigma),
        }
    }

    /// Returns the maximum period required by the smoothing method.
    pub fn max_period(&self) -> usize {
        match self {
            SmoothingMethod::SMA(period) => *period,
            SmoothingMethod::EMA(period) => *period,
            SmoothingMethod::KAMA(period, _, _) => *period,
            SmoothingMethod::DEMA(period) => *period,
            SmoothingMethod::TEMA(period) => *period,
            SmoothingMethod::ALMA(period, _, _) => *period,
        }
    }
}

/// Calculates the Simple Moving Average (SMA) series over a given period.
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values representing the time series data.
/// * `period` - The number of periods over which to calculate the SMA.
///
/// # Returns
///
/// * `Option<Vec<Decimal>>` - The SMA series if calculation is possible, `None` otherwise.
///
/// # Example
///
/// ```
/// let data = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)];
/// let sma_series = sma(&data, 3).unwrap();
/// // sma_series = [2.0, 3.0, 4.0]
/// ```
pub fn sma(data: &[Decimal], period: usize) -> Option<Vec<Decimal>> {
    if data.is_empty() {
        warn!("Data slice is empty, cannot calculate SMA");
        return None;
    }

    let mut result = Vec::new();

    for idx in 1..data.len() {
        let window_idx = if idx < period { 0 } else { idx - period };
        let window = &data[window_idx..=idx];
        let mean = stats::mean(window)?;
        result.push(mean);
    }

    Some(result)
}

/// Calculates the Exponential Moving Average (EMA) series over a given period.
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values representing the time series data.
/// * `period` - The number of periods over which to calculate the EMA.
///
/// # Returns
///
/// * `Option<Vec<Decimal>>` - The EMA series if calculation is possible, `None` otherwise.
///
/// # Example
///
/// ```
/// let data = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)];
/// let ema_series = ema(&data, 3).unwrap();
/// // ema_series = [2.0, 3.0, 4.0]
/// ```
pub fn ema(data: &[Decimal], period: usize) -> Option<Vec<Decimal>> {
    if data.is_empty() {
        warn!("Data slice is empty, cannot calculate EMA");
        return None;
    }

    let mut result = Vec::with_capacity(data.len());

    // Calculate initial SMA to start EMA calculation
    let first_value = data.first().cloned()?;
    result.push(first_value);

    let mut ema_prev = first_value;
    let alpha = Decimal::from(2) / Decimal::from(period + 1);

    // Apply the EMA formula to the rest of the data
    for &price in &data[1..] {
        let ema = alpha * price + (Decimal::ONE - alpha) * ema_prev;
        result.push(ema);
        ema_prev = ema;
    }

    Some(result)
}

/// Calculates the Double Exponential Moving Average (DEMA) series.
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values representing the time series data.
/// * `period` - The number of periods over which to calculate the DEMA.
///
/// # Returns
///
/// * `Option<Vec<Decimal>>` - The DEMA series if calculation is possible, `None` otherwise.
///
/// # Example
///
/// ```
/// let data = vec![/* data */];
/// let dema_series = dema(&data, 10).unwrap();
/// ```
pub fn dema(data: &[Decimal], period: usize) -> Option<Vec<Decimal>> {
    if data.len() < period {
        warn!("Data length is insufficient for DEMA calculation");
        return None;
    }

    // Compute EMA1 over the data
    let ema1_values = ema(data, period)?;
    info!("EMA 1: {:?}", ema1_values);

    // Compute EMA2 over EMA1
    let ema2_values = ema(&ema1_values, period)?;
    info!("EMA 2: {:?}", ema2_values);

    // DEMA = 2 * EMA1 - EMA2
    let dema_values = izip!(ema1_values, ema2_values)
        .map(|(ema1, ema2)| Decimal::from(2) * ema1 - ema2)
        .collect();

    Some(dema_values)
}

/// Calculates the Tripple Exponential Moving Average (TEMA) series.
/// The TEMA formula: [3 x (1st EMA)] – [3 x (2nd EMA)] + (3rd EMA)
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values representing the time series data.
/// * `period` - The number of periods over which to calculate the DEMA.
///
/// # Returns
///
/// * `Option<Vec<Decimal>>` - The TEMA series if calculation is possible, `None` otherwise.
///
/// # Example
///
/// ```
/// let data = vec![/* data */];
/// let tema_series = dema(&data, 10).unwrap();
/// ```
pub fn tema(data: &[Decimal], period: usize) -> Option<Vec<Decimal>> {
    if data.is_empty() {
        warn!("Data length is insufficient for DEMA calculation");
        return None;
    }

    // Compute EMA1 over the data
    let ema1_values = ema(data, period)?;

    // Compute EMA2 over EMA1
    let ema2_values = ema(&ema1_values, period)?;

    // Compute EMA3 over EMA@
    let ema3_values = ema(&ema2_values, period)?;

    // TEMA = 3 * EMA1 - 3 * EMA2 + EMA3
    let tema_values = izip!(ema1_values, ema2_values, ema3_values)
        .map(|(ema1, ema2, ema3)| (Decimal::from(3) * ema1) - (Decimal::from(3) * ema2) + ema3)
        .collect();

    Some(tema_values)
}

/// Calculates the Kaufman's Adaptive Moving Average (KAMA) series.
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values representing the time series data.
/// * `period` - The number of periods over which to calculate the KAMA.
/// * `fast_period` - The period for the fastest EMA (typically 2).
/// * `slow_period` - The period for the slowest EMA (typically 30).
///
/// # Returns
///
/// * `Option<Vec<Decimal>>` - The KAMA series if calculation is possible, `None` otherwise.
///
/// # Example
///
/// ```
/// let data = vec![/* data */];
/// let kama_series = kama(&data, 10, 2, 30).unwrap();
/// ```
pub fn kama(data: &[Decimal], period: usize, fast_period: usize, slow_period: usize) -> Option<Vec<Decimal>> {
    // Validate period parameters
    if fast_period >= slow_period {
        warn!("fast_period should be less than slow_period for KAMA calculation");
        return None;
    }
    if period == 0 || fast_period == 0 || slow_period == 0 {
        warn!("Period parameters must be greater than zero");
        return None;
    }

    // Determine the maximum period required
    let max_period = period.max(fast_period).max(slow_period);

    if data.len() < max_period {
        warn!(
            "Data length ({}) is less than required maximum period ({}) for KAMA calculation",
            data.len(),
            max_period
        );
        return None;
    }

    let fast_alpha = Decimal::from(2) / Decimal::from(fast_period + 1);
    let slow_alpha = Decimal::from(2) / Decimal::from(slow_period + 1);

    let mut kama_values = Vec::with_capacity(data.len() - period + 1);

    // Initialize KAMA with the initial SMA
    let initial_sma = stats::mean(&data[..period])?;
    let mut kama = initial_sma;
    kama_values.push(kama);

    for i in period..data.len() {
        // Efficiency Ratio (ER)
        let change = (data[i] - data[i - period]).abs();
        let volatility: Decimal = data[i - period + 1..=i].windows(2).map(|w| (w[1] - w[0]).abs()).sum();

        let er = if volatility.is_zero() {
            Decimal::ZERO
        } else {
            change / volatility
        };

        // Smoothing Constant (SC)
        let sc = (er * (fast_alpha - slow_alpha) + slow_alpha).powi(2);

        // Calculate KAMA
        kama = kama + sc * (data[i] - kama);
        kama_values.push(kama);
    }

    Some(kama_values)
}

/// Calculates the Arnaud Legoux Moving Average (ALMA) series.
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values representing the time series data.
/// * `period` - The number of periods over which to calculate the ALMA.
/// * `offset` - The offset parameter (between 0 and 1).
/// * `sigma` - The sigma parameter (positive value).
///
/// # Returns
///
/// * `Option<Vec<Decimal>>` - The ALMA series if calculation is possible, `None` otherwise.
///
/// # Example
///
/// ```
/// let data = vec![/* data */];
/// let alma_series = alma(&data, 9, dec!(0.85), dec!(6.0)).unwrap();
/// ```
pub fn alma(data: &[Decimal], period: usize, offset: Decimal, sigma: Decimal) -> Option<Vec<Decimal>> {
    if data.len() < period {
        warn!("Data length is less than period for ALMA calculation");
        return None;
    }
    if !(Decimal::ZERO..=Decimal::ONE).contains(&offset) {
        warn!("Offset parameter should be between 0 and 1");
        return None;
    }
    if sigma <= Decimal::ZERO {
        warn!("Sigma parameter should be positive");
        return None;
    }

    let m = offset * Decimal::from(period - 1);
    let s = Decimal::from(period) / sigma;

    let mut alma_values = Vec::with_capacity(data.len() - period + 1);

    for i in period - 1..data.len() {
        let window = &data[i + 1 - period..=i];
        let mut sum = Decimal::ZERO;
        let mut norm = Decimal::ZERO;

        for (j, &price) in window.iter().enumerate() {
            let idx = Decimal::from(j);
            let exponent = -((idx - m).powi(2)) / (Decimal::from(2) * s.powi(2));
            let weight = exponent.exp();
            sum += price * weight;
            norm += weight;
        }

        if norm.is_zero() {
            warn!("Normalization factor is zero in ALMA calculation");
            return None;
        }

        let alma = sum / norm;
        alma_values.push(alma);
    }

    Some(alma_values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_sma_insufficient_data() {
        let data = vec![dec!(1.0), dec!(2.0)];
        let period = 3;
        let result = sma(&data, period);
        assert!(result.is_none());
    }

    #[test]
    fn test_sma_sufficient_data() {
        let data = vec![dec!(1.0), dec!(2.0), dec!(3.0)];
        let period = 2;
        let result = sma(&data, period).unwrap();
        assert_eq!(result, vec![dec!(1.5), dec!(2.5)]); // SMA series: [ (1+2)/2, (2+3)/2 ]
    }

    #[test]
    fn test_sma_known_values() {
        let data = vec![dec!(10.0), dec!(20.0), dec!(30.0), dec!(40.0), dec!(50.0)];
        let period = 3;
        let result = sma(&data, period).unwrap();
        assert_eq!(result, vec![dec!(20.0), dec!(30.0), dec!(40.0)]); // Only one value: average of all five elements
    }

    #[test]
    fn test_sma_edge_case_single_value() {
        let data = vec![dec!(42.0)];
        let period = 1;
        let result = sma(&data, period).unwrap();
        assert_eq!(result, vec![dec!(42.0)]);
    }

    #[test]
    fn test_ema_insufficient_data() {
        let data = vec![dec!(1.0), dec!(2.0)];
        let period = 3;
        let result = ema(&data, period);
        assert!(result.is_none());
    }

    #[test]
    fn test_ema_sufficient_data() {
        let data = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0)];
        let period = 3;
        let result = ema(&data, period).unwrap();
        // Expected EMA series:
        // Initial SMA (first EMA value): (1+2+3)/3 = 2.0
        // Next EMA: (4 - 2.0) * 0.5 + 2.0 = 3.0
        assert_eq!(result, vec![dec!(1.0), dec!(1.5), dec!(2.25), dec!(3.125)]);
    }

    #[test]
    fn test_ema_known_values() {
        let data = vec![
            dec!(22.27),
            dec!(22.19),
            dec!(22.08),
            dec!(22.17),
            dec!(22.18),
            dec!(22.13),
            dec!(22.23),
            dec!(22.43),
            dec!(22.24),
            dec!(22.29),
            dec!(22.15),
            dec!(22.39),
            dec!(22.38),
            dec!(22.61),
            dec!(23.36),
        ];
        let period = 10;
        let result = ema(&data, period).unwrap();
        // The last EMA value should be approximately 22.5184
        let last_ema = result.last().unwrap();
        assert_eq!(last_ema.round_dp(4), dec!(22.5271));
    }

    #[test]
    fn test_ema_edge_case_constant_values() {
        let data = vec![dec!(5.0); 20];
        let period = 5;
        let result = ema(&data, period).unwrap();
        // All EMA values should be 5.0
        assert!(result.iter().all(|&x| x == dec!(5.0)));
    }

    #[test]
    fn test_kama_insufficient_data() {
        let data = vec![dec!(1.0); 9];
        let result = kama(&data, 10, 2, 30);
        assert!(result.is_none());
    }

    #[test]
    fn test_kama_invalid_periods() {
        let data = vec![dec!(1.0); 50];

        // fast_period >= slow_period
        let result = kama(&data, 10, 30, 30);
        assert!(result.is_none());

        // fast_period > slow_period
        let result = kama(&data, 10, 31, 30);
        assert!(result.is_none());

        // Period parameters are zero
        let result = kama(&data, 0, 2, 30);
        assert!(result.is_none());

        // Data length less than max_period
        let result = kama(&data[..20], 10, 2, 30);
        assert!(result.is_none());
    }

    #[test]
    fn test_kama_valid_parameters() {
        let data = vec![dec!(1.0); 50];
        let result = kama(&data, 10, 2, 30).unwrap();
        // All KAMA values should be 1.0
        assert!(result.iter().all(|&x| x == dec!(1.0)));
    }

    #[test]
    fn test_kama_sufficient_data() {
        let data = vec![
            dec!(110.0),
            dec!(112.0),
            dec!(111.0),
            dec!(115.0),
            dec!(118.0),
            dec!(117.0),
            dec!(119.0),
            dec!(120.0),
            dec!(121.0),
            dec!(122.0),
            dec!(123.0),
            dec!(124.0),
        ];
        let result = kama(&data, 5, 2, 12).unwrap();
        let last_kama = result.last().unwrap();
        assert_eq!(last_kama.round_dp(2), dec!(123.50));
    }

    #[test]
    fn test_kama_known_values() {
        let data = vec![
            dec!(10.0),
            dec!(10.5),
            dec!(11.0),
            dec!(11.5),
            dec!(12.0),
            dec!(12.5),
            dec!(13.0),
            dec!(13.5),
            dec!(14.0),
            dec!(14.5),
            dec!(15.0),
            dec!(15.5),
            dec!(16.0),
            dec!(16.5),
            dec!(17.0),
            dec!(17.5),
            dec!(18.0),
            dec!(18.5),
            dec!(19.0),
            dec!(19.5),
            dec!(20.0),
            dec!(20.5),
            dec!(21.0),
            dec!(21.5),
            dec!(22.0),
            dec!(22.5),
            dec!(23.0),
            dec!(23.5),
            dec!(24.0),
            dec!(24.5),
            dec!(25.0),
        ];
        let result = kama(&data, 10, 2, 30).unwrap();
        let last_kama = result.last().unwrap();
        assert_eq!(last_kama.round_dp(1), dec!(24.6));
    }

    #[test]
    fn test_kama_edge_case_no_volatility() {
        let data = vec![dec!(100.0); 31];
        let result = kama(&data, 10, 2, 30).unwrap();
        // All KAMA values should be 100.0
        assert!(result.iter().all(|&x| x == dec!(100.0)));
    }

    #[test]
    fn test_dema_insufficient_data() {
        let data = vec![dec!(1.0); 2];
        let result = dema(&data, 3);
        assert!(result.is_none());
    }

    #[test]
    fn test_dema_sufficient_data() {
        let data = vec![
            dec!(127.75),
            dec!(129.02),
            dec!(127.85),
            dec!(130.08),
            dec!(129.79),
            dec!(129.06),
            dec!(129.11),
            dec!(129.39),
            dec!(129.29),
            dec!(128.71),
            dec!(128.47),
            dec!(128.65),
            dec!(128.05),
            dec!(127.09),
            dec!(127.59),
            dec!(128.06),
            dec!(128.13),
            dec!(127.72),
            dec!(128.09),
            dec!(127.72),
        ];
        let result = dema(&data, 10).unwrap();
        let last_dema = result.last().unwrap();
        assert_eq!(last_dema.round_dp(2), dec!(127.80));
    }

    #[test]
    fn test_dema_known_values() {
        let data = vec![
            dec!(1.0),
            dec!(2.0),
            dec!(3.0),
            dec!(4.0),
            dec!(5.0),
            dec!(6.0),
            dec!(7.0),
            dec!(8.0),
            dec!(9.0),
            dec!(10.0),
            dec!(15.0),
        ];
        let result = dema(&data, 3).unwrap();
        info!("Result: {:?}", result);
        let last_dema = result.last().unwrap();
        assert_eq!(last_dema.round_dp(2), dec!(14.00));
    }

    #[test]
    fn test_dema_edge_case_constant_values() {
        let data = vec![dec!(5.0); 10];
        let result = dema(&data, 3).unwrap();
        // All DEMA values should be 5.0
        assert!(result.iter().all(|&x| x.round_dp(2) == dec!(5.0)));
    }

    #[test]
    fn test_tema_insufficient_data() {
        let data = vec![dec!(1.0); 2];
        let result = tema(&data, 3);
        assert!(result.is_none());
    }

    #[test]
    fn test_tema_sufficient_data() {
        let data = vec![
            dec!(127.75),
            dec!(129.02),
            dec!(127.85),
            dec!(130.08),
            dec!(129.79),
            dec!(129.06),
            dec!(129.11),
            dec!(129.39),
            dec!(129.29),
            dec!(128.71),
            dec!(128.47),
            dec!(128.65),
            dec!(128.05),
            dec!(127.09),
            dec!(127.59),
            dec!(128.06),
            dec!(128.13),
            dec!(127.72),
            dec!(128.09),
            dec!(127.72),
        ];
        let result = tema(&data, 10).unwrap();
        let last_dema = result.last().unwrap();
        assert_eq!(last_dema.round_dp(2), dec!(127.69));
    }

    #[test]
    fn test_tema_known_values() {
        let data = vec![
            dec!(1.0),
            dec!(2.0),
            dec!(3.0),
            dec!(4.0),
            dec!(5.0),
            dec!(6.0),
            dec!(7.0),
            dec!(8.0),
            dec!(9.0),
            dec!(10.0),
            dec!(15.0),
        ];
        let result = tema(&data, 3).unwrap();
        info!("Result: {:?}", result);
        let last_dema = result.last().unwrap();
        assert_eq!(last_dema.round_dp(2), dec!(14.51));
    }

    #[test]
    fn test_tema_edge_case_constant_values() {
        let data = vec![dec!(5.0); 10];
        let result = tema(&data, 3).unwrap();
        // All DEMA values should be 5.0
        assert!(result.iter().all(|&x| x.round_dp(2) == dec!(5.0)));
    }

    #[test]
    fn test_alma_insufficient_data() {
        let data = vec![dec!(1.0); 4];
        let result = alma(&data, 5, dec!(0.85), dec!(6.0));
        assert!(result.is_none());
    }

    #[test]
    fn test_alma_sufficient_data() {
        let data = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)];
        let result = alma(&data, 5, dec!(0.85), dec!(6.0)).unwrap();
        let last_alma = result.last().unwrap();
        assert_eq!(last_alma.round_dp(2), dec!(4.26));
    }

    #[test]
    fn test_alma_known_values() {
        let data = vec![
            dec!(10.0),
            dec!(11.0),
            dec!(12.0),
            dec!(13.0),
            dec!(14.0),
            dec!(15.0),
            dec!(16.0),
            dec!(17.0),
            dec!(18.0),
            dec!(19.0),
            dec!(20.0),
        ];
        let result = alma(&data, 5, dec!(0.85), dec!(6.0)).unwrap();
        let last_alma = result.last().unwrap();
        assert_eq!(last_alma.round_dp(2), dec!(19.26));
    }

    #[test]
    fn test_alma_edge_case_invalid_parameters() {
        let data = vec![dec!(1.0); 10];
        let result = alma(&data, 5, dec!(1.5), dec!(6.0));
        assert!(result.is_none()); // Offset parameter is invalid
    }
}
