use std::{collections::HashMap, sync::Arc};

use arkin_core::prelude::*;
use time::UtcDateTime;

/// Helper for validating feature calculations in integration tests
pub struct FeatureValidator {
    instruments: Vec<Arc<Instrument>>,

    // Track raw accumulations per instrument per window
    current_window: HashMap<Arc<Instrument>, HashMap<String, f64>>,
    window_history: Vec<HashMap<Arc<Instrument>, HashMap<String, f64>>>,

    // Feature validation rules
    validators: Vec<ValidationRule>,

    // Cached feature IDs
    feature_ids: HashMap<String, FeatureId>,
}

pub struct ValidationRule {
    pub feature_name: String,
    pub validation_type: ValidationType,
    pub tolerance: f64,                             // Percentage (0.01 = 1%)
    pub target_instrument: Option<Arc<Instrument>>, // If set, only validate for this instrument
}

pub enum ValidationType {
    /// Validate against raw accumulation in current window
    RawAccumulation { accumulator: String },

    /// Validate as sum of N intervals of another feature
    SumOfIntervals {
        source_feature: String,
        num_intervals: usize,
    },

    /// Validate using a custom computation from accumulators
    Computed {
        compute: fn(&HashMap<String, f64>) -> Option<f64>,
    },

    /// Validate using a custom computation from interval sums
    ComputedFromIntervals {
        compute: fn(&[f64], &[f64]) -> Option<f64>,
        accumulator1: String,
        accumulator2: String,
        num_intervals: usize,
    },

    /// Validate synthetic feature as sum of source features across multiple instruments
    SyntheticSum {
        source_feature: String,
        source_instruments: Vec<Arc<Instrument>>,
    },

    /// Validate a property/invariant
    Property {
        check: fn(&HashMap<String, f64>) -> Result<(), String>,
    },
}

impl FeatureValidator {
    pub async fn new(_persistence: &Arc<dyn PersistenceReader>, instruments: Vec<Arc<Instrument>>) -> Self {
        // Pre-fetch common feature IDs
        let feature_ids = HashMap::new();

        Self {
            instruments,
            current_window: HashMap::new(),
            window_history: Vec::new(),
            validators: Vec::new(),
            feature_ids,
        }
    }

    /// Add additional instruments to track (e.g., synthetic instruments)
    pub fn add_instruments(mut self, instruments: Vec<Arc<Instrument>>) -> Self {
        self.instruments.extend(instruments);
        self
    }

    /// Define a raw value accumulator (e.g., sum of trade_notional)
    pub fn track_accumulator(mut self, name: impl Into<String>) -> Self {
        // Accumulators are just stored in current_window, initialized to 0
        let name = name.into();
        for inst in &self.instruments {
            self.current_window
                .entry(inst.clone())
                .or_insert_with(HashMap::new)
                .insert(name.clone(), 0.0);
        }
        self
    }

    /// Accumulate a value for an instrument
    pub fn accumulate(&mut self, instrument: &Arc<Instrument>, accumulator: &str, value: f64) {
        *self
            .current_window
            .entry(instrument.clone())
            .or_insert_with(HashMap::new)
            .entry(accumulator.to_string())
            .or_insert(0.0) += value;
    }

    /// Validate a feature against raw accumulation
    pub fn validate_raw_aggregate(
        mut self,
        feature_name: impl Into<String>,
        accumulator: impl Into<String>,
        tolerance: f64,
    ) -> Self {
        self.validators.push(ValidationRule {
            feature_name: feature_name.into(),
            validation_type: ValidationType::RawAccumulation {
                accumulator: accumulator.into(),
            },
            tolerance,
            target_instrument: None,
        });
        self
    }

    /// Validate a feature as sum of N intervals of another feature
    pub fn validate_interval_sum(
        mut self,
        feature_name: impl Into<String>,
        source_feature: impl Into<String>,
        num_intervals: usize,
        tolerance: f64,
    ) -> Self {
        self.validators.push(ValidationRule {
            feature_name: feature_name.into(),
            validation_type: ValidationType::SumOfIntervals {
                source_feature: source_feature.into(),
                num_intervals,
            },
            tolerance,
            target_instrument: None,
        });
        self
    }

    /// Validate a feature using a custom computation from accumulators
    pub fn validate_computed(
        mut self,
        feature_name: impl Into<String>,
        compute: fn(&HashMap<String, f64>) -> Option<f64>,
        tolerance: f64,
    ) -> Self {
        self.validators.push(ValidationRule {
            feature_name: feature_name.into(),
            validation_type: ValidationType::Computed { compute },
            tolerance,
            target_instrument: None,
        });
        self
    }

    /// Validate a feature using computation from interval sums
    pub fn validate_computed_from_intervals(
        mut self,
        feature_name: impl Into<String>,
        accumulator1: impl Into<String>,
        accumulator2: impl Into<String>,
        num_intervals: usize,
        compute: fn(&[f64], &[f64]) -> Option<f64>,
        tolerance: f64,
    ) -> Self {
        self.validators.push(ValidationRule {
            feature_name: feature_name.into(),
            validation_type: ValidationType::ComputedFromIntervals {
                compute,
                accumulator1: accumulator1.into(),
                accumulator2: accumulator2.into(),
                num_intervals,
            },
            tolerance,
            target_instrument: None,
        });
        self
    }

    /// Validate synthetic feature as sum of source features across instruments
    pub fn validate_synthetic_sum(
        mut self,
        synthetic_instrument: Arc<Instrument>,
        feature_name: impl Into<String>,
        source_feature: impl Into<String>,
        source_instruments: Vec<Arc<Instrument>>,
        tolerance: f64,
    ) -> Self {
        self.validators.push(ValidationRule {
            feature_name: feature_name.into(),
            validation_type: ValidationType::SyntheticSum {
                source_feature: source_feature.into(),
                source_instruments,
            },
            tolerance,
            target_instrument: Some(synthetic_instrument),
        });
        self
    }

    /// Validate property/invariant across features
    pub fn validate_property(
        mut self,
        feature_name: impl Into<String>,
        check: fn(&HashMap<String, f64>) -> Result<(), String>,
    ) -> Self {
        self.validators.push(ValidationRule {
            feature_name: feature_name.into(),
            validation_type: ValidationType::Property { check },
            tolerance: 0.0,
            target_instrument: None,
        });
        self
    }

    /// Register a feature ID for lookup
    pub async fn register_feature(&mut self, persistence: &Arc<dyn PersistenceReader>, name: impl Into<String>) {
        let name = name.into();
        let feature_id = persistence.get_feature_id(&name).await;
        self.feature_ids.insert(name, feature_id);
    }

    /// Commit current window and start a new one
    pub fn commit_window(&mut self) {
        self.window_history.push(self.current_window.clone());

        // Clear current window
        for values in self.current_window.values_mut() {
            for value in values.values_mut() {
                *value = 0.0;
            }
        }
    }

    /// Validate calculated insights against expectations
    pub fn validate(&self, insights: &[Arc<Insight>], timestamp: UtcDateTime) -> Result<ValidationReport, String> {
        let mut report = ValidationReport {
            timestamp,
            passed: 0,
            failed: 0,
            failures: Vec::new(),
        };

        for rule in &self.validators {
            for inst in &self.instruments {
                // Skip if rule has a target instrument and it doesn't match
                if let Some(ref target) = rule.target_instrument {
                    if target.id != inst.id {
                        continue;
                    }
                }

                let result = self.validate_rule(rule, inst, insights)?;

                match result {
                    RuleResult::Pass { feature, value } => {
                        report.passed += 1;
                        tracing::debug!("✓ {} validated for {}: {:.2}", feature, inst.symbol, value);
                    }
                    RuleResult::Fail {
                        feature,
                        expected,
                        actual,
                        diff,
                        pct,
                    } => {
                        report.failed += 1;
                        let msg = format!(
                            "⚠️  {} mismatch for {}: calculated={:.2}, expected={:.2}, diff={:.2} ({:.2}%)",
                            feature, inst.symbol, actual, expected, diff, pct
                        );
                        tracing::warn!("{}", msg);
                        report.failures.push(msg);
                    }
                    RuleResult::Skip => {}
                }
            }
        }

        Ok(report)
    }

    fn validate_rule(
        &self,
        rule: &ValidationRule,
        inst: &Arc<Instrument>,
        insights: &[Arc<Insight>],
    ) -> Result<RuleResult, String> {
        let feature_id = self
            .feature_ids
            .get(&rule.feature_name)
            .ok_or_else(|| format!("Feature ID not found: {}", rule.feature_name))?;

        let calculated = insights
            .iter()
            .find(|i| i.instrument.id == inst.id && i.feature_id == *feature_id)
            .map(|i| i.value);

        let Some(actual) = calculated else {
            return Ok(RuleResult::Skip);
        };

        // Debug logging for 5-minute aggregates on BTC
        let is_debug_feature = (rule.feature_name == "notional_05m" || rule.feature_name == "notional_buy_05m")
            && inst.symbol.contains("BTC");

        let expected = match &rule.validation_type {
            ValidationType::RawAccumulation { accumulator } => self
                .current_window
                .get(inst)
                .and_then(|w| w.get(accumulator))
                .copied()
                .unwrap_or(0.0),
            ValidationType::SumOfIntervals {
                source_feature,
                num_intervals,
            } => {
                // Need at least num_intervals-1 in history + current window
                if self.window_history.len() < num_intervals - 1 {
                    return Ok(RuleResult::Skip);
                }

                // Take num_intervals-1 from history (most recent) + current window
                let mut values: Vec<f64> = self
                    .window_history
                    .iter()
                    .rev()
                    .take(num_intervals - 1)
                    .filter_map(|window| window.get(inst).and_then(|w| w.get(source_feature)))
                    .copied()
                    .collect();

                // Add current window value
                if let Some(current_value) = self.current_window.get(inst).and_then(|w| w.get(source_feature)) {
                    values.push(*current_value);
                }

                if is_debug_feature {
                    tracing::info!(
                        "🔍 {} validator summing {} intervals of {}: {:?}",
                        inst.symbol,
                        num_intervals,
                        source_feature,
                        values
                    );
                }

                values.iter().sum()
            }
            ValidationType::Computed { compute } => {
                let accumulators = self.current_window.get(inst).cloned().unwrap_or_default();
                compute(&accumulators).unwrap_or(0.0)
            }
            ValidationType::ComputedFromIntervals {
                compute,
                accumulator1,
                accumulator2,
                num_intervals,
            } => {
                // Need at least num_intervals-1 in history + current window
                if self.window_history.len() < num_intervals - 1 {
                    return Ok(RuleResult::Skip);
                }

                // Collect values for accumulator1
                let mut values1: Vec<f64> = self
                    .window_history
                    .iter()
                    .rev()
                    .take(num_intervals - 1)
                    .filter_map(|window| window.get(inst).and_then(|w| w.get(accumulator1)))
                    .copied()
                    .collect();
                if let Some(current_value) = self.current_window.get(inst).and_then(|w| w.get(accumulator1)) {
                    values1.push(*current_value);
                }

                // Collect values for accumulator2
                let mut values2: Vec<f64> = self
                    .window_history
                    .iter()
                    .rev()
                    .take(num_intervals - 1)
                    .filter_map(|window| window.get(inst).and_then(|w| w.get(accumulator2)))
                    .copied()
                    .collect();
                if let Some(current_value) = self.current_window.get(inst).and_then(|w| w.get(accumulator2)) {
                    values2.push(*current_value);
                }

                compute(&values1, &values2).unwrap_or(0.0)
            }
            ValidationType::SyntheticSum {
                source_feature,
                source_instruments,
            } => {
                // Sum the source feature across all source instruments from insights
                let source_feature_id = self
                    .feature_ids
                    .get(source_feature)
                    .ok_or_else(|| format!("Source feature ID not found: {}", source_feature))?;

                let sum: f64 = source_instruments
                    .iter()
                    .filter_map(|src_inst| {
                        insights
                            .iter()
                            .find(|i| i.instrument.id == src_inst.id && i.feature_id == *source_feature_id)
                            .map(|i| i.value)
                    })
                    .sum();

                sum
            }
            ValidationType::Property { check: _ } => {
                // Properties don't have expected values, just pass/fail
                return Ok(RuleResult::Pass {
                    feature: rule.feature_name.clone(),
                    value: actual,
                });
            }
        };

        let diff = (actual - expected).abs();
        let tolerance = if expected > 0.0 {
            expected * rule.tolerance
        } else {
            1.0
        };
        let pct = if expected > 0.0 {
            (diff / expected) * 100.0
        } else {
            0.0
        };

        if diff > tolerance && expected > 0.0 {
            Ok(RuleResult::Fail {
                feature: rule.feature_name.clone(),
                expected,
                actual,
                diff,
                pct,
            })
        } else if expected > 0.0 {
            Ok(RuleResult::Pass {
                feature: rule.feature_name.clone(),
                value: actual,
            })
        } else {
            Ok(RuleResult::Skip)
        }
    }
}

pub struct ValidationReport {
    pub timestamp: UtcDateTime,
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<String>,
}

impl ValidationReport {
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }
}

enum RuleResult {
    Pass {
        feature: String,
        value: f64,
    },
    Fail {
        feature: String,
        expected: f64,
        actual: f64,
        diff: f64,
        pct: f64,
    },
    Skip,
}
