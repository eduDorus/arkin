#![allow(non_snake_case)]
use clarabel::algebra::*;
use clarabel::solver::*;
use statrs::statistics::Statistics;

fn main() {
    // Let's make tree vectors with returns for three assets (positive and negative returns like stocks)
    let returns = vec![
        vec![0.06, -0.06, -0.03, 0.04, -0.05],
        vec![0.02, -0.03, -0.04, 0.05, -0.06],
        vec![-0.03, -0.04, -0.05, -0.06, -0.07],
    ];

    // Assemble the covariance matrix
    let covariance_matrix = returns
        .iter()
        .enumerate()
        .map(|(i, _)| {
            returns
                .iter()
                .enumerate()
                .map(|(j, _)| returns[i].clone().covariance(returns[j].clone()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<_>>>();

    info!("Covariance Matrix:");
    for row in &covariance_matrix {
        for val in row {
            print!("{:.5} ", val);
        }
        info!();
    }
    info!("Covariance Matrix: {:?}", covariance_matrix);

    // Example covariance matrix (Σ)
    // let sigma = [[0.1, 0.02, 0.01], [0.02, 0.15, 0.03], [0.01, 0.03, 0.15]];
    let mu = [0.08, -0.07, -0.03]; // expected returns
    let w_prev = [-0.05, 0.02, 0.03]; // Example previous holdings
    let lambda = 0.5; // risk aversion
    let gamma = 0.1; // penalty on deviation from previous portfolio
    info!("Expected Returns: {:?}", mu);
    info!("Previous Portfolio Weights: {:?}", w_prev);
    info!("Risk Aversion: {:?}", lambda);
    info!("Penalty on Deviation from Previous Portfolio: {:?}", gamma);

    // Construct P and q:
    // P = 2 * Σ
    // q = -λ * μ
    // let P_data: Vec<Vec<f64>> = covariance_matrix
    //     .iter()
    //     .map(|row| row.iter().map(|val| 2.0 * val).collect())
    //     .collect();
    // let q: Vec<f64> = mu.iter().map(|r| -lambda * r).collect();

    // let P = CscMatrix::from(&P_data);

    // Constraints (Long Only Portfolio):
    // sum(w) = 1  --> single equality constraint
    // w ≥ 0       --> enforce non-negativity using inequality constraints

    // The constraint matrix A includes both equality and inequality:
    // A w = b, with A and b covering both equality and inequality via cones.
    // Rows:
    //   0: w1 + w2 + w3 = 1       (equality)
    //   1: -w1 ≤ 0 → w1 ≥ 0
    //   2: -w2 ≤ 0 → w2 ≥ 0
    //   3: -w3 ≤ 0 → w3 ≥ 0
    // let A = CscMatrix::from(&[
    //     [1.0, 1.0, 1.0],  // sum(w)=1
    //     [-1.0, 0.0, 0.0], // w1 ≥ 0
    //     [0.0, -1.0, 0.0], // w2 ≥ 0
    //     [0.0, 0.0, -1.0], // w3 ≥ 0
    // ]);
    // let b = vec![1.0, 0.0, 0.0, 0.0];

    // First constraint row is equality (ZeroCone) and the rest are nonnegative constraints
    // let cones = [ZeroConeT(1), NonnegativeConeT(n)];

    // Construct P and q with the extra penalty
    // Original: P = 2 * Σ, q = -λ * μ
    // Now:      P = 2 * (Σ + γI)
    //           q = -λμ - 2γ w_prev
    let mut P_data: Vec<Vec<f64>> = covariance_matrix
        .iter()
        .map(|row| row.iter().map(|val| 2.0 * val).collect())
        .collect();

    let n = 3;

    // Add 2*γ to the diagonal of P_data
    for i in 0..n {
        P_data[i][i] += 2. * gamma;
    }
    let P = CscMatrix::from(&P_data);

    let mut q: Vec<f64> = mu.iter().map(|r| -lambda * r).collect();
    for i in 0..n {
        q[i] -= 2.0 * gamma * w_prev[i];
    }

    // Constraints (Long/Short Portfolio):
    // sum(w) = 1  --> single equality constraint (Fully Invested Absence of Cash)
    // -1 ≤ w ≤ 1  --> enforce long/short using inequality constraints

    // The constraint matrix A includes both equality and inequality:
    // A w = b, with A and b covering both equality and inequality via cones.
    // Rows:
    //   0: w1 + w2 + w3 = 0       (equality)
    //
    //   2: -w1 ≤ 1 → w1 ≥ -1
    //   3: -w2 ≤ 1 → w2 ≥ -1
    //   4: -w3 ≤ 1 → w3 ≥ -1

    let (A, b, cones) = create_constraints(n);

    let mut settings = DefaultSettings::default();
    settings.verbose = false;
    let mut solver = DefaultSolver::new(&P, &q, &A, &b, &cones, settings);

    solver.solve();

    info!("Optimisation status: {:?}", solver.solution.status);
    info!("Optimal portfolio weights: {:?}", solver.solution.x);
    info!("Sum of weights: {:?}", solver.solution.x.iter().sum::<f64>());
}

// fn mean(series: &Vec<f64>) -> f64 {
//     let sum: f64 = series.iter().sum();
//     sum / series.len() as f64
// }

// fn standard_deviation(series: &Vec<f64>, mean: f64) -> f64 {
//     let variance = series
//         .iter()
//         .map(|value| {
//             let diff = value - mean;
//             diff * diff
//         })
//         .sum::<f64>()
//         / (series.len() as f64 - 1.0);
//     variance.sqrt()
// }

// fn covariance(series1: &Vec<f64>, mean1: f64, series2: &Vec<f64>, mean2: f64) -> f64 {
//     let cov = series1
//         .iter()
//         .zip(series2.iter())
//         .map(|(x, y)| (x - mean1) * (y - mean2))
//         .sum::<f64>()
//         / (series1.len() as f64 - 1.0);
//     cov
// }

fn create_constraints(n: usize) -> (CscMatrix<f64>, Vec<f64>, [SupportedConeT<f64>; 2]) {
    let m = 1 + 2 * n;

    let mut I = Vec::with_capacity(m);
    let mut J = Vec::with_capacity(n);
    let mut V = Vec::new();

    // Add sum(w) = 1 constraint
    for i in 0..n {
        let column = i;
        let row = 0;
        let val = 1.0;
        I.push(row);
        J.push(column);
        V.push(val);
    }

    // Add w ≥ 0 constraints
    for i in 0..n {
        let column = i;
        let row = i + 1;
        let val = -1.0;
        I.push(row);
        J.push(column);
        V.push(val);
    }

    // Add w ≤ 1 constraints
    for i in 0..n {
        let column = i;
        let row = i + n + 1;
        let val = 1.0;
        I.push(row);
        J.push(column);
        V.push(val);
    }

    let A = CscMatrix::new_from_triplets(m, n, I, J, V);

    let mut b = vec![0.0; 1];
    b.extend(vec![1.0; 2 * n]);

    let cones = [ZeroConeT(1), NonnegativeConeT(m - 1)];

    (A, b, cones)
}
