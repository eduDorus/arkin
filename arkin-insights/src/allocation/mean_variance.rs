#![allow(non_snake_case)]
use std::{fmt, sync::Arc};

use anyhow::Result;
use clarabel::algebra::*;
use clarabel::solver::*;
use rust_decimal::prelude::*;
use statrs::statistics::*;
use time::UtcDateTime;
use tracing::debug;
use tracing::info;
use tracing::warn;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{state::InsightsState, Computation};

#[derive(TypedBuilder)]
pub struct MeanVarianceFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input_expected_returns: FeatureId,
    input_returns: FeatureId,
    periods_returns: usize,
    output: FeatureId,
    persist: bool,
    risk_aversion: f64,
    risk_free_rate: f64,
    max_exposure_long: f64,
    max_exposure_short: f64,
    max_exposure_long_per_asset: f64,
    max_exposure_short_per_asset: f64,
    transaction_cost: f64,
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
            .field("risk_free_rate", &self.risk_free_rate)
            .field("max_exposure_long", &self.max_exposure_long)
            .field("max_exposure_short", &self.max_exposure_short)
            .field("max_exposure_long_per_asset", &self.max_exposure_long_per_asset)
            .field("max_exposure_short_per_asset", &self.max_exposure_short_per_asset)
            .field("transaction_cost", &self.transaction_cost)
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

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: UtcDateTime) -> Option<Vec<Arc<Insight>>> {
        info!("Calculating Mean Variance Portfolio at {}", event_time);

        let mut w_prev = instruments
            .iter()
            .filter_map(|i| self.insight_state.last(Some(i.clone()), self.output.clone(), event_time))
            .map(|v| v.to_f64().expect("Failed to convert to f64"))
            .collect::<Vec<_>>();

        // If w_prev is empty return all zeros for the length of instruments
        // If w_prev is not empty, check if the length of w_prev is equal to the length of instruments
        if w_prev.is_empty() {
            w_prev = vec![0.0; instruments.len()];
        } else if w_prev.len() != instruments.len() {
            warn!("Length of w_prev is not equal to the length of instruments, returning all zeros");
            w_prev = vec![0.0; instruments.len()];
        }

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

        info!("Fetching return input feature-id: {}", self.input_returns);
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
        if returns.is_empty() || returns.len() != instruments.len() || returns.iter().any(|r| r.is_empty()) {
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
            .transaction_cost(self.transaction_cost)
            .max_exposure_long(self.max_exposure_long)
            .max_exposure_short(self.max_exposure_short)
            .max_long_per_asset(self.max_exposure_long_per_asset)
            .max_short_per_asset(self.max_exposure_short_per_asset)
            .risk_free_rate(self.risk_free_rate)
            .build();

        let weights = mean_variance_optimization.solve(&returns, &expected_returns, &w_prev);

        let insights = instruments
            .iter()
            .zip(weights.iter())
            .map(|(i, w)| {
                Insight::builder()
                    .event_time(event_time)
                    .pipeline(Some(self.pipeline.clone()))
                    .instrument(Some(i.clone()))
                    .feature_id(self.output.clone())
                    .value(Decimal::from_f64(*w).expect("Failed to convert to Decimal"))
                    .persist(self.persist)
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
    transaction_cost: f64,
    risk_free_rate: f64,
}

impl MeanVarianceOptimization {
    pub fn solve(&self, u: &[Vec<f64>], mu: &[f64], w0: &[f64]) -> Vec<f64> {
        // Scale lambda to be in the range of 0.0 to 1.0
        let lambda = self.lambda * 1000.0;

        let n = mu.len();

        // Assemble the covariance matrix
        let covariance_matrix = compute_covariance_matrix(u);

        info!("Predicted returns: {:.5?}", mu);
        info!("Covariance Matrix: {:.9?}", covariance_matrix);
        // Problem definition
        let mut col = Vec::new();
        let mut row = Vec::new();
        let mut val = Vec::new();

        for row_ptr in 0..n {
            for col_ptr in 0..n {
                col.push(col_ptr);
                row.push(row_ptr);
                val.push(2. * lambda * covariance_matrix[row_ptr][col_ptr]);
            }
        }
        let P = CscMatrix::new_from_triplets(2 * n, 2 * n, col, row, val);
        debug!("P: {:?}", P);

        // Linear coefficients
        let mut q = mu.iter().map(|r| -r).collect::<Vec<_>>();
        q.extend(vec![self.transaction_cost; n]);

        // Constraints
        let (A, b, cones) = self.create_constraints(n, w0);

        // Solve the problem
        let mut settings = DefaultSettings::default();
        settings.verbose = false;

        let mut solver = DefaultSolver::new(&P, &q, &A, &b, &cones, settings);
        solver.solve();

        let weights_z = solver.solution.x;

        // Step 6: Retrieve and Process the Solution
        let w = &weights_z[0..n];
        let z = &weights_z[n..2 * n];

        debug!("Transaction Costs (z_i)");
        for (i, z_i) in z.iter().enumerate() {
            debug!("Absolute difference {}: {:.2}", i, z_i);
        }
        debug!("Sum of weights: {:.5}", w.iter().sum::<f64>());
        debug!(
            "Solution Status: {:?} with {:?} iterations",
            solver.solution.status, solver.solution.iterations
        );

        let expected_return = compute_portfolio_return(&w, mu);
        let portfolio_variance = compute_portfolio_variance(&w, &covariance_matrix);
        let risk = portfolio_variance.sqrt();
        let sharp = compute_sharpe_ratio(expected_return, portfolio_variance, self.risk_free_rate);
        debug!("Expected Return: {:.5}", expected_return);
        debug!("Portfolio Variance: {:.5}", portfolio_variance);
        debug!("Risk: {:.5}", risk);
        debug!("Sharp Ratio: {:.5}", sharp);

        info!(
            "Optimal Allocation with lambda: {} ({:.5?}) turnover: {:.5?} return {:.5?} risk: {:.5?} sharpe: {:.5?}",
            self.lambda,
            w,
            z.sum(),
            expected_return,
            risk,
            sharp,
        );
        w.to_vec()
    }

    fn create_constraints(&self, n: usize, w0: &[f64]) -> (CscMatrix<f64>, Vec<f64>, [SupportedConeT<f64>; 1]) {
        // Total constraints:
        // 1. Sum of weights <= max_exposure_long
        // 2. Sum of -weights <= max_exposure_short
        // 3. Each w_i <= max_long_per_asset (n constraints)
        // 4. Each -w_i <= max_short_per_asset (n constraints)
        // 5. For each asset, two constraints for |w_i - w0_i| <= z_i (2n constraints)
        let m = 2 + 4 * n; // 2 + 4n

        let max_exposure_long = self.max_exposure_long;
        let max_exposure_short = self.max_exposure_short;
        let max_long_per_asset = self.max_long_per_asset;
        let max_short_per_asset = self.max_short_per_asset;

        let mut I = Vec::new(); // Estimating non-zero entries
        let mut J = Vec::new();
        let mut V = Vec::new();

        let mut b = Vec::new();

        // Constraint 1: Sum of weights <= max_exposure_long
        for i in 0..n {
            I.push(0); // Row for this constraint
            J.push(i); // Column for w_i
            V.push(1.0); // Coefficient for w_i
        }
        b.push(max_exposure_long);

        // Constraint 2: Sum of -weights <= max_exposure_short
        for i in 0..n {
            I.push(1); // Row for this constraint
            J.push(i); // Column for w_i
            V.push(-1.0); // Coefficient for w_i
        }
        b.push(max_exposure_short);

        // Constraint 3: w_i <= max_long_per_asset
        for i in 0..n {
            I.push(2 + i);
            J.push(i);
            V.push(1.0);
            b.push(max_long_per_asset);
        }

        // Constraint 4: -w_i <= max_short_per_asset
        for i in 0..n {
            I.push(2 + n + i);
            J.push(i);
            V.push(-1.0);
            b.push(max_short_per_asset);
        }

        // Constraint 5: For each asset, two constraints for |w_i - w0_i| <= z_i
        for i in 0..n {
            I.push(2 + 2 * n + i); // Row for this constraint
            J.push(i); // Column for w_i
            V.push(1.0); // Coefficient for w_i

            I.push(2 + 2 * n + i); // Row for this constraint
            J.push(n + i); // Column for z_i
            V.push(-1.0); // Coefficient for z_i
            b.push(w0[i]); // RHS
        }

        // Constraint 6: For each asset, two constraints for |-w_i - w0_i| <= z_i
        for i in 0..n {
            // Constraint 2: -w_i - z_i <= -w0_i
            I.push(2 + n * 3 + i); // Row for this constraint
            J.push(i); // Column for w_i
            V.push(-1.0); // Coefficient for w_i

            I.push(2 + n * 3 + i); // Row for this constraint
            J.push(n + i); // Column for z_i
            V.push(-1.0); // Coefficient for z_i
            b.push(-w0[i]); // RHS
        }

        debug!("A Shape: {} {}", I.len(), J.len());

        let A = CscMatrix::new_from_triplets(m, 2 * n, I, J, V);

        let cones = [NonnegativeConeT(m)];

        (A, b, cones)
    }
}

/// Function to compute the covariance matrix from data
fn compute_covariance_matrix(u: &[Vec<f64>]) -> Vec<Vec<f64>> {
    // info!("u: {:.6?}", u);
    let cov = u
        .iter()
        .enumerate()
        .map(|(i, _)| {
            u.iter()
                .enumerate()
                .map(|(j, _)| u[i].clone().covariance(u[j].clone()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<_>>>();

    // Print the covariance matrix
    // for i in 0..cov.len() {
    //     for j in 0..cov[i].len() {
    //         info!("{:.5} ", cov[i][j]);
    //     }
    // }
    cov
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
