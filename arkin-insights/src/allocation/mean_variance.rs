#![allow(non_snake_case)]
use std::{fmt, sync::Arc};

use anyhow::Result;
use clarabel::algebra::*;
use clarabel::solver::*;
use rust_decimal::prelude::*;
use statrs::statistics::*;
use time::OffsetDateTime;
use tracing::debug;
use tracing::warn;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use uuid::Uuid;

use crate::{state::InsightsState, Computation};

#[derive(TypedBuilder)]
pub struct MeanVarianceFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input_expected_returns: FeatureId,
    input_returns: FeatureId,
    output: FeatureId,
    periods_returns: usize,
    risk_aversion: f64,
    risk_free_rate: f64,
    max_exposure_long: f64,
    max_exposure_short: f64,
    max_exposure_long_per_asset: f64,
    max_exposure_short_per_asset: f64,
}

impl fmt::Debug for MeanVarianceFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MeanVarianceFeature")
            .field("pipeline", &self.pipeline)
            .field("input_expected_returns", &self.input_expected_returns)
            .field("input_returns", &self.input_returns)
            .field("output", &self.output)
            .field("periods_returns", &self.periods_returns)
            .field("risk_aversion", &self.risk_aversion)
            .field("tracking_aversion", &self.risk_free_rate)
            .finish()
    }
}

impl Computation for MeanVarianceFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_expected_returns.clone(), self.input_returns.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instruments: &[Arc<Instrument>], event_time: OffsetDateTime) -> Result<Vec<Arc<Insight>>> {
        debug!("Calculating Mean Variance Portfolio...");

        // let mut w_prev = instruments
        //     .iter()
        //     .filter_map(|i| self.insight_state.last(Some(i.clone()), self.output.clone(), event_time))
        //     .map(|v| v.to_f64().expect("Failed to convert to f64"))
        //     .collect::<Vec<_>>();

        // // If w_prev is empty return all zeros for the length of instruments
        // // If w_prev is not empty, check if the length of w_prev is equal to the length of instruments
        // if w_prev.is_empty() {
        //     w_prev = vec![0.0; instruments.len()];
        // } else if w_prev.len() != instruments.len() {
        //     warn!("Length of w_prev is not equal to the length of instruments");
        //     return Ok(vec![]);
        // }

        let expected_returns = instruments
            .iter()
            .filter_map(|i| {
                self.insight_state
                    .last(Some(i.clone()), self.input_expected_returns.clone(), event_time)
            })
            .map(|v| v.to_f64().expect("Failed to convert to f64"))
            .collect::<Vec<_>>();

        // Check if expected returns is empty and has the same length as instruments
        if expected_returns.is_empty() || expected_returns.len() != instruments.len() {
            warn!("Expected returns is empty or has different length than instruments");
            return Ok(vec![]);
        }

        let returns = instruments
            .iter()
            .map(|i| {
                self.insight_state.periods(
                    Some(i.clone()),
                    self.input_returns.clone(),
                    event_time,
                    self.periods_returns,
                )
            })
            .map(|v| {
                v.into_iter()
                    .map(|v| v.to_f64().expect("Failed to convert to f64"))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // Check if returns is empty and has the same length as instruments
        if returns.is_empty() || returns.len() != instruments.len() {
            warn!("Returns is empty or has different length than instruments");
            return Ok(vec![]);
        }

        // Check that all returns have the same length
        if returns.iter().any(|r| r.len() != self.periods_returns) {
            warn!("Returns have different lengths");
            return Ok(vec![]);
        }

        // Create our optimization problem
        let mean_variance_optimization = MeanVarianceOptimization::builder()
            .lambda(self.risk_aversion)
            .max_exposure_long(self.max_exposure_long)
            .max_exposure_short(self.max_exposure_short)
            .max_long_per_asset(self.max_exposure_long_per_asset)
            .max_short_per_asset(self.max_exposure_short_per_asset)
            .risk_free_rate(self.risk_free_rate)
            .build();

        let weights = mean_variance_optimization.solve(&returns, &expected_returns);

        let insights = instruments
            .iter()
            .zip(weights.iter())
            .map(|(i, w)| {
                Insight::builder()
                    .id(Uuid::new_v4())
                    .event_time(event_time)
                    .pipeline(self.pipeline.clone())
                    .instrument(Some(i.clone()))
                    .feature_id(self.output.clone())
                    .value(Decimal::from_f64(*w).expect("Failed to convert to Decimal"))
                    .build()
                    .into()
            })
            .collect::<Vec<_>>();
        self.insight_state.insert_batch(&insights);
        Ok(insights)
    }
}

#[derive(Debug, TypedBuilder)]
pub struct MeanVarianceOptimization {
    lambda: f64,
    max_exposure_long: f64,
    max_exposure_short: f64,
    max_long_per_asset: f64,
    max_short_per_asset: f64,
    risk_free_rate: f64,
}

impl MeanVarianceOptimization {
    pub fn solve(&self, u: &[Vec<f64>], mu: &[f64]) -> Vec<f64> {
        let lambda = self.lambda;

        let n = mu.len();

        // Assemble the covariance matrix
        let covariance_matrix = compute_covariance_matrix(u);

        // Problem definition
        let P_data = covariance_matrix
            .iter()
            .map(|row| row.iter().map(|val| 2.0 * val).collect())
            .collect::<Vec<Vec<f64>>>();
        let P = CscMatrix::from(&P_data);

        // Linear coefficients
        let q: Vec<f64> = mu.iter().map(|r| -lambda * r).collect();

        // Constraints
        let (A, b, cones) = self.create_constraints(n);

        // Solve the problem
        let mut settings = DefaultSettings::default();
        settings.verbose = false;

        let mut solver = DefaultSolver::new(&P, &q, &A, &b, &cones, settings);
        solver.solve();

        let weights = solver.solution.x;
        for (i, w) in weights.iter().enumerate() {
            debug!("Asset {}: {}%", i, (w * 100.).round());
        }
        debug!("Sum of weights: {:.5}", weights.iter().sum::<f64>());
        debug!(
            "Solution Status: {:?} with {:?} iterations",
            solver.solution.status, solver.solution.iterations
        );

        let expected_return = compute_portfolio_return(&weights, mu);
        let portfolio_variance = compute_portfolio_variance(&weights, &covariance_matrix);
        let risk = portfolio_variance.sqrt();
        let sharpe = compute_sharpe_ratio(expected_return, portfolio_variance, self.risk_free_rate);
        debug!("Expected Return: {:.5}", expected_return);
        debug!("Portfolio Variance: {:.5}", portfolio_variance);
        debug!("Risk: {:.5}", risk);
        debug!("Sharpe Ratio: {:.5}", sharpe);
        weights
    }

    fn create_constraints(&self, n: usize) -> (CscMatrix<f64>, Vec<f64>, [SupportedConeT<f64>; 1]) {
        let m = 2 + 2 * n;
        let max_exposure_long = self.max_exposure_long;
        let max_exposure_short = self.max_exposure_short;
        let max_long_per_asset = self.max_long_per_asset;
        let max_short_per_asset = self.max_short_per_asset;

        let mut I = Vec::with_capacity(m);
        let mut J = Vec::with_capacity(n);
        let mut V = Vec::new();

        // Max total long allocation
        for i in 0..n {
            let column = i;
            let row = 0;
            let val = 1.0;
            I.push(row);
            J.push(column);
            V.push(val);
        }

        // max total_short allocation
        for i in 0..n {
            let column = i;
            let row = 1;
            let val = -1.0;
            I.push(row);
            J.push(column);
            V.push(val);
        }

        // Max long constraint
        for i in 0..n {
            let column = i;
            let row = i + 2;
            let val = 1.0;
            I.push(row);
            J.push(column);
            V.push(val);
        }

        // Max short constraint
        for i in 0..n {
            let column = i;
            let row = i + n + 2;
            let val = -1.0;
            I.push(row);
            J.push(column);
            V.push(val);
        }

        let A = CscMatrix::new_from_triplets(m, n, I, J, V);

        let mut b = vec![max_exposure_long, max_exposure_short];
        b.extend(vec![max_long_per_asset; n]);
        b.extend(vec![max_short_per_asset; n]);

        let cones = [NonnegativeConeT(m)];

        (A, b, cones)
    }
}

/// Function to compute the covariance matrix from data
fn compute_covariance_matrix(u: &[Vec<f64>]) -> Vec<Vec<f64>> {
    u.iter()
        .enumerate()
        .map(|(i, _)| {
            u.iter()
                .enumerate()
                .map(|(j, _)| u[i].clone().covariance(u[j].clone()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<_>>>()
}

fn compute_portfolio_return(weights: &[f64], expected_return: &[f64]) -> f64 {
    weights.iter().zip(expected_return.iter()).map(|(w, r)| w * r).sum()
}

/// Function to compute portfolio variance: w^T * Sigma * w
fn compute_portfolio_variance(weights: &[f64], covariance_matrix: &Vec<Vec<f64>>) -> f64 {
    let mut variance = 0.0;
    let n = weights.len();
    for i in 0..n {
        for j in 0..n {
            variance += weights[i] * covariance_matrix[i][j] * weights[j];
        }
    }
    variance
}

fn compute_sharpe_ratio(expected_return: f64, portfolio_variance: f64, risk_free_rate: f64) -> f64 {
    (expected_return - risk_free_rate) / portfolio_variance.sqrt()
}
