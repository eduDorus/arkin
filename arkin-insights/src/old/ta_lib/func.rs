use itertools::izip;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tracing::warn;

use super::{mean, pct_change, std_dev, SmoothingMethod};

/// Calculates the True Range (TR) for a series of price data.
///
/// # Arguments
///
/// * `high` - A slice of `Decimal` values representing the high prices.
/// * `low` - A slice of `Decimal` values representing the low prices.
/// * `close` - A slice of `Decimal` values representing the closing prices.
///
/// # Returns
///
/// * `Option<Vec<Decimal>>` - A vector containing the True Range values for each period after the first one, or `None` if calculation is not possible.
///
/// # Example
///
/// ```
/// let high_prices = vec![dec!(15.0), dec!(16.0), dec!(17.0)];
/// let low_prices = vec![dec!(14.0), dec!(15.0), dec!(16.0)];
/// let close_prices = vec![dec!(14.5), dec!(15.5), dec!(16.5)];
/// let tr_values = true_range(&high_prices, &low_prices, &close_prices).unwrap();
/// ```
pub fn true_range(high: &[Decimal], low: &[Decimal], close: &[Decimal]) -> Option<Vec<Decimal>> {
    if high.len() != low.len() || low.len() != close.len() {
        warn!("Input data lengths do not match for True Range calculation");
        return None;
    }

    if high.len() < 2 {
        warn!("Not enough data to calculate True Range");
        return None;
    }

    let mut tr_values = Vec::with_capacity(high.len() - 1);

    for i in 1..high.len() {
        let current_high = high[i];
        let current_low = low[i];
        let previous_close = close[i - 1];

        let high_low = current_high - current_low;
        let high_close = (current_high - previous_close).abs();
        let low_close = (current_low - previous_close).abs();

        let true_range = Decimal::max(high_low, Decimal::max(high_close, low_close));
        tr_values.push(true_range);
    }

    Some(tr_values)
}

/// Calculates the Bollinger Bands.
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values representing the time series data (e.g., closing prices).
/// * `std_dev_multiplier` - The standard deviation multiplier for the upper and lower bands (typically 2).
/// * `smoothing_method` - The smoothing method to use for the middle band.
///
/// # Returns
///
/// * `Option<(Vec<Decimal>, Vec<Decimal>, Vec<Decimal>)>` - A tuple containing vectors of the middle band, upper band, and lower band, oscillator, width values.
///
/// # Example
///
/// ```
/// let closing_prices = vec![/* Close price data */];
/// let (lower_band, middle_band, upper_band) = bollinger_bands(&closing_prices, dec!(2.0), SmoothingMethod::SMA(20)).unwrap();
/// ```
pub fn bollinger_bands(
    data: &[Decimal],
    std_dev_multiplier: Decimal,
    smoothing_method: SmoothingMethod,
) -> Option<(Vec<Decimal>, Vec<Decimal>, Vec<Decimal>, Vec<Decimal>, Vec<Decimal>)> {
    let period = smoothing_method.max_period();
    if data.len() < period {
        warn!("Insufficient data for Bollinger Bands calculation");
        return None;
    }

    let mut middle_band = smoothing_method.calc(data)?;
    let mut upper_band = Vec::with_capacity(middle_band.len() - 1);
    let mut lower_band = Vec::with_capacity(middle_band.len() - 1);
    let mut oscillator = Vec::with_capacity(middle_band.len() - 1);
    let mut width = Vec::with_capacity(middle_band.len() - 1);

    info!("middle_band: {:?}", middle_band);
    info!("data: {:?}", data);
    let offset = data.len() - middle_band.len();
    let data = &data[offset..];
    info!("data after: {:?}", data);

    for i in 1..middle_band.len() {
        let idx = if period < i { i - period } else { 0 };
        let slice = &data[idx..=i];
        info!("Slice: {:?}", slice);
        let p = data[i];
        let mb = middle_band[i];
        info!("mb: {:?}", mb);
        let std_dev = std_dev(slice)?;
        let ub = mb + std_dev_multiplier * std_dev;
        let lb = mb - std_dev_multiplier * std_dev;

        let diff = ub - lb;
        let oscillator_value = if !diff.is_zero() {
            (p - lb) / diff
        } else {
            dec!(0.5)
        };

        let width_value = if mb.is_zero() {
            Decimal::zero()
        } else {
            diff / mb
        };

        upper_band.push(ub);
        lower_band.push(lb);
        oscillator.push(oscillator_value);
        width.push(width_value);
    }
    // remove first element from middle band
    middle_band.remove(0);

    info!("upper_band: {:?}", upper_band);
    info!("middle_band: {:?}", middle_band);
    info!("lower_band: {:?}", lower_band);
    info!("oscillator: {:?}", oscillator);
    info!("width: {:?}", width);

    Some((middle_band, upper_band, lower_band, oscillator, width))
}

/// Calculates the Relative Strength Index (RSI).
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values representing the closing prices.
/// * `smoothing_method` - The smoothing method to use for average gains and losses (typically EMA).
///
/// # Returns
///
/// * `Option<Vec<Decimal>>` - A vector containing the RSI values.
///
/// # Example
///
/// ```
/// let closing_prices = vec![/* Close price data */];
/// let rsi_values = rsi(&closing_prices, SmoothingMethod::EMA(14)).unwrap();
/// ```
pub fn rsi(data: &[Decimal], smoothing_method: SmoothingMethod) -> Option<Vec<Decimal>> {
    let period = smoothing_method.max_period();
    if data.len() < period + 1 {
        warn!("Insufficient data for RSI calculation");
        return None;
    }

    info!("data: {:?}", data);
    let pct_changes = pct_change(data)?;
    info!("pct_changes: {:?}", pct_changes.len());

    // Separate gains and losses
    let mut rsi = Vec::with_capacity(pct_changes.len());
    for i in 0..pct_changes.len() - period {
        info!("i: {:?}", i);
        info!("upper bound: {:?}", i + period);
        let slice = &pct_changes[i..i + period];
        assert!(slice.len() == period);
        info!("slice: {:?}", slice);

        let mut gains = Vec::with_capacity(period);
        let mut losses = Vec::with_capacity(period);

        for r in slice {
            if *r > Decimal::ZERO {
                gains.push(*r);
            } else {
                losses.push(r.abs());
            }
        }

        let last_gain = gains.pop().unwrap_or_default();
        let last_loss = losses.pop().unwrap_or_default();
        let prev_avg_gain = mean(&gains).unwrap_or_default();
        let prev_avg_loss = mean(&losses).unwrap_or_default();

        // Calculate the RSI
        let rsi_gain = prev_avg_gain * Decimal::from(period - 1) + last_gain;
        let rsi_loss = prev_avg_loss * Decimal::from(period - 1) + last_loss;
        info!("rsi_gain: {:?}", rsi_gain);
        info!("rsi_loss: {:?}", rsi_loss);

        // Zero loss edge case
        let rsi_value = match (rsi_gain.is_zero(), rsi_loss.is_zero()) {
            (true, true) => dec!(50),
            (false, true) => dec!(100),
            _ => {
                let ratio = rsi_gain / rsi_loss;
                Decimal::from(100) - (Decimal::from(100) / (Decimal::from(1) + ratio))
            }
        };
        info!("rsi_value: {:?}", rsi_value);

        rsi.push(rsi_value);
    }
    info!("rsi: {:?}", rsi.len());
    let rsi_smoothing = smoothing_method.calc(&rsi)?;
    info!("rsi: {:?}", rsi_smoothing);
    Some(rsi_smoothing)
}

/// Calculates the Moving Average Convergence Divergence (MACD) and Signal Line.
///
/// # Arguments
///
/// * `data` - A slice of `Decimal` values representing the closing prices.
/// * `fast` - The smoothing method for the fast line (e.g., `SmoothingMethod::EMA(12)`).
/// * `slow` - The smoothing method for the slow line (e.g., `SmoothingMethod::EMA(26)`).
/// * `signal` - The smoothing method for the signal line (e.g., `SmoothingMethod::EMA(9)`).
///
/// # Returns
///
/// * `Option<(Vec<Decimal>, Vec<Decimal>, Vec<Decimal>)>` - A tuple containing vectors of the MACD line, Signal line, and Histogram values.
///
/// # Example
///
/// ```
/// let closing_prices = vec![/* Close price data */];
/// let (macd_line, signal_line, histogram) = macd(
///     &closing_prices,
///     SmoothingMethod::EMA(12),
///     SmoothingMethod::EMA(26),
///     SmoothingMethod::EMA(9),
/// ).unwrap();
/// ```
pub fn macd(
    data: &[Decimal],
    fast: SmoothingMethod,
    slow: SmoothingMethod,
    signal: SmoothingMethod,
) -> Option<(Vec<Decimal>, Vec<Decimal>, Vec<Decimal>)> {
    let required_period = slow.max_period();
    if data.len() < required_period {
        warn!("Insufficient data for MACD calculation");
        return None;
    }

    // Calculate fast and slow series
    let fast_values = fast.calc(data)?;
    let slow_values = slow.calc(data)?;

    // Align the series
    let offset = fast_values.len() - slow_values.len();
    let fast_aligned = &fast_values[offset..];
    let slow_aligned = &slow_values;
    info!("fast_aligned: {:?}", fast_aligned);
    info!("slow_aligned: {:?}", slow_aligned);

    // Calculate MACD line
    let macd_line: Vec<Decimal> = izip!(fast_aligned, slow_aligned)
        .map(|(fast_val, slow_val)| fast_val - slow_val)
        .collect();

    // Calculate Signal line
    info!("MACD line: {:?}", macd_line);
    let signal_line = signal.calc(&macd_line)?;
    info!("signal_line: {:?}", signal_line);

    // Align the MACD line and Signal line
    let diff = macd_line.len() - signal_line.len();
    let macd_line_aligned = &macd_line[diff..];

    // Calculate Histogram
    let histogram: Vec<Decimal> = macd_line_aligned
        .iter()
        .zip(signal_line.iter())
        .map(|(&macd_val, &signal_val)| macd_val - signal_val)
        .collect();

    Some((macd_line_aligned.to_vec(), signal_line, histogram))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_true_range_known_values() {
        let high = vec![dec!(15.0), dec!(16.0), dec!(17.0), dec!(18.0)];
        let low = vec![dec!(14.0), dec!(15.0), dec!(16.0), dec!(17.0)];
        let close = vec![dec!(14.5), dec!(15.5), dec!(16.5), dec!(17.5)];

        // Manually calculate expected True Range values
        // TR[1] = max(16 - 15, abs(16 - 14.5), abs(15 - 14.5)) = max(1, 1.5, 0.5) = 1.5
        // TR[2] = max(17 - 16, abs(17 - 15.5), abs(16 - 15.5)) = max(1, 1.5, 0.5) = 1.5
        // TR[3] = max(18 - 17, abs(18 - 16.5), abs(17 - 16.5)) = max(1, 1.5, 0.5) = 1.5
        let expected_tr = vec![dec!(1.5), dec!(1.5), dec!(1.5)];

        let tr_values = true_range(&high, &low, &close).unwrap();

        assert_eq!(tr_values, expected_tr);
    }

    #[test]
    fn test_true_range_insufficient_data() {
        let high = vec![dec!(15.0)];
        let low = vec![dec!(14.0)];
        let close = vec![dec!(14.5)];

        let tr_values = true_range(&high, &low, &close);
        assert!(tr_values.is_none());
    }

    #[test]
    fn test_true_range_mismatched_lengths() {
        let high = vec![dec!(15.0), dec!(16.0)];
        let low = vec![dec!(14.0)];
        let close = vec![dec!(14.5), dec!(15.5)];

        let tr_values = true_range(&high, &low, &close);
        assert!(tr_values.is_none());
    }

    #[test]
    fn test_true_range_zero_volatility() {
        let high = vec![dec!(10.0), dec!(10.0), dec!(10.0)];
        let low = vec![dec!(10.0), dec!(10.0), dec!(10.0)];
        let close = vec![dec!(10.0), dec!(10.0), dec!(10.0)];

        let expected_tr = vec![dec!(0.0), dec!(0.0)];

        let tr_values = true_range(&high, &low, &close).unwrap();

        assert_eq!(tr_values, expected_tr);
    }

    #[test]
    fn test_bollinger_bands_known_values_sma() {
        let data = vec![
            dec!(20.0),
            dec!(22.0),
            dec!(21.0),
            dec!(23.0),
            dec!(22.0),
            dec!(24.0),
            dec!(25.0),
            dec!(25.0),
            dec!(25.0),
            dec!(25.0),
            dec!(30.0),
            dec!(35.0),
            dec!(50.0),
        ];
        let period = 5;
        let std_dev_multiplier = dec!(2.0);
        let smoothing_method = SmoothingMethod::SMA(period);

        // Manually calculate expected values
        // For simplicity, we'll calculate for the last data point
        // Middle Band (MB): SMA of last 3 data points: (22 + 24 + 25) / 3 = 23.6667
        // Standard Deviation (SD): sqrt(((x - MB)^2)/n)
        // SD = sqrt(((22 - 23.6667)^2 + (24 - 23.6667)^2 + (25 - 23.6667)^2)/3)
        // SD ≈ 1.2472
        // Upper Band (UB): MB + 2 * SD ≈ 23.6667 + 2 * 1.2472 ≈ 26.1611
        // Lower Band (LB): MB - 2 * SD ≈ 23.6667 - 2 * 1.2472 ≈ 21.1722

        let (mb, ub, lb, osc, w) = bollinger_bands(&data, std_dev_multiplier, smoothing_method).unwrap();

        let last_mb = mb.last().unwrap();
        let last_ub = ub.last().unwrap();
        let last_lb = lb.last().unwrap();
        let last_osc = osc.last().unwrap();
        let last_w = w.last().unwrap();

        assert_eq!(last_ub.round_dp(2), dec!(52.66));
        assert_eq!(last_mb.round_dp(2), dec!(33.));
        assert_eq!(last_lb.round_dp(2), dec!(13.34));
        assert_eq!(last_osc.round_dp(2), dec!(0.93));
        assert_eq!(last_w.round_dp(2), dec!(1.19));
    }

    #[test]
    fn test_bollinger_bands_insufficient_data() {
        let data = vec![dec!(20.0), dec!(22.0)];
        let std_dev_multiplier = dec!(2.0);
        let smoothing_method = SmoothingMethod::SMA(3);

        let result = bollinger_bands(&data, std_dev_multiplier, smoothing_method);
        assert!(result.is_none());
    }

    #[test]
    fn test_bollinger_bands_constant_data() {
        let data = vec![dec!(100.0); 20];
        let std_dev_multiplier = dec!(2.0);
        let smoothing_method = SmoothingMethod::SMA(5);

        let (mb, ub, lb, osc, w) = bollinger_bands(&data, std_dev_multiplier, smoothing_method).unwrap();

        // Since the data is constant, SD should be zero, and all bands should be equal
        assert!(mb.iter().all(|&ub| ub == dec!(100.0)));
        assert!(ub.iter().all(|&mb| mb == dec!(100.0)));
        assert!(lb.iter().all(|&lb| lb == dec!(100.0)));
        assert!(osc.iter().all(|&osc| osc == dec!(0.5)));
        assert!(w.iter().all(|&w| w == dec!(0.0)));
    }

    #[test]
    fn test_rsi_known_values() {
        let data = vec![
            dec!(44.3389),
            dec!(44.0902),
            dec!(44.1497),
            dec!(43.6124),
            dec!(44.3278),
            dec!(44.8264),
            dec!(45.0955),
            dec!(45.4245),
            dec!(45.8433),
            dec!(46.0826),
            dec!(45.8931),
            dec!(46.0328),
            dec!(45.6140),
            dec!(46.2820),
            dec!(46.2820),
            dec!(46.0028),
            dec!(46.0328),
            dec!(46.4116),
            dec!(46.2222),
            dec!(45.6439),
            dec!(46.2122),
            dec!(46.2521),
            dec!(45.7137),
            dec!(46.4515),
            dec!(45.7835),
            dec!(45.3548),
            dec!(44.0288),
            dec!(44.1783),
            dec!(44.2181),
            dec!(44.5672),
            dec!(44.6656),
        ];
        let smoothing_method = SmoothingMethod::EMA(14);

        let rsi_values = rsi(&data, smoothing_method).unwrap();

        // The first RSI value corresponds to the 15th data point
        let expected_rsi_first = dec!(52.6066); // This value may vary slightly depending on calculations

        let first_rsi = rsi_values.first().unwrap();
        assert_eq!(first_rsi.round_dp(4), expected_rsi_first);
    }

    #[test]
    fn test_rsi_insufficient_data() {
        let data = vec![dec!(44.3389), dec!(44.0902)];
        let smoothing_method = SmoothingMethod::EMA(14);

        let rsi_values = rsi(&data, smoothing_method);
        assert!(rsi_values.is_none());
    }

    #[test]
    fn test_rsi_constant_data() {
        let data = vec![dec!(50.0); 20];
        let smoothing_method = SmoothingMethod::EMA(14);

        let rsi_values = rsi(&data, smoothing_method).unwrap();

        // RSI should be 50 when there are no gains or losses
        assert!(rsi_values.iter().all(|&rsi| rsi.round_dp(2) == dec!(50.0)));
    }

    #[test]
    fn test_rsi_all_gains() {
        let data: Vec<Decimal> = (1..=20).map(Decimal::from).collect();
        let smoothing_method = SmoothingMethod::EMA(14);

        let rsi_values = rsi(&data, smoothing_method).unwrap();

        // RSI should approach 100 in case of continuous gains
        let last_rsi = rsi_values.last().unwrap();
        assert!(last_rsi.round_dp(2) > dec!(99.0));
    }

    #[test]
    fn test_rsi_all_losses() {
        let data: Vec<Decimal> = (1..=20).rev().map(Decimal::from).collect();
        let smoothing_method = SmoothingMethod::EMA(14);

        let rsi_values = rsi(&data, smoothing_method).unwrap();

        // RSI should approach 0 in case of continuous losses
        let last_rsi = rsi_values.last().unwrap();
        assert!(last_rsi.round_dp(2) < dec!(1.0));
    }

    #[test]
    fn test_macd_known_values() {
        let data = vec![
            dec!(26.0),
            dec!(27.0),
            dec!(28.0),
            dec!(29.0),
            dec!(30.0),
            dec!(31.0),
            dec!(32.0),
            dec!(33.0),
            dec!(34.0),
            dec!(35.0),
            dec!(36.0),
            dec!(37.0),
            dec!(38.0),
            dec!(39.0),
            dec!(40.0),
            dec!(41.0),
            dec!(42.0),
            dec!(43.0),
            dec!(44.0),
            dec!(45.0),
            dec!(46.0),
            dec!(47.0),
            dec!(48.0),
            dec!(49.0),
            dec!(50.0),
        ];

        let fast = SmoothingMethod::EMA(5);
        let slow = SmoothingMethod::EMA(13);
        let signal = SmoothingMethod::EMA(7);

        let (macd_line, signal_line, histogram) = macd(&data, fast, slow, signal).unwrap();

        // Since data is linearly increasing, MACD line should approach zero
        let last_macd = macd_line.last().expect("MACD line is empty");
        let last_signal = signal_line.last().expect("MACD signal line is empty");
        let last_histogram = histogram.last().expect("MACD histogram is empty");

        // We expect MACD and Histogram to be close to zero
        assert_eq!(last_macd.round_dp(2), dec!(3.85));
        assert_eq!(last_signal.round_dp(2), dec!(3.72));
        assert_eq!(last_histogram.round_dp(2), dec!(0.14));
    }

    #[test]
    fn test_macd_insufficient_data() {
        let data = vec![dec!(1.0); 25]; // Less than slow period (26)
        let fast = SmoothingMethod::EMA(12);
        let slow = SmoothingMethod::EMA(26);
        let signal = SmoothingMethod::EMA(9);

        let result = macd(&data, fast, slow, signal);
        assert!(result.is_none());
    }

    #[test]
    fn test_macd_constant_data() {
        let data = vec![dec!(100.0); 100];
        let fast = SmoothingMethod::EMA(12);
        let slow = SmoothingMethod::EMA(26);
        let signal = SmoothingMethod::EMA(9);

        let (macd_line, signal_line, histogram) = macd(&data, fast, slow, signal).unwrap();

        // All MACD and Histogram values should be zero
        assert!(macd_line.iter().all(|&x| x.round_dp(4) == dec!(0.0)));
        assert!(signal_line.iter().all(|&x| x.round_dp(4) == dec!(0.0)));
        assert!(histogram.iter().all(|&x| x.round_dp(4) == dec!(0.0)));
    }

    #[test]
    fn test_macd_with_different_smoothing_methods() {
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

        let fast = SmoothingMethod::SMA(12);
        let slow = SmoothingMethod::SMA(26);
        let signal = SmoothingMethod::SMA(9);

        let (macd_line, signal_line, histogram) = macd(&data, fast, slow, signal).unwrap();

        // Ensure that the lengths of the outputs are consistent
        assert_eq!(macd_line.len(), signal_line.len());
        assert_eq!(histogram.len(), signal_line.len());
    }
}
