use hdbscan::{DistanceMetric, Hdbscan, HdbscanHyperParams, NnAlgorithm};
use std::collections::HashMap;

/// Struct to hold general clustering results for any 2D numerical data.
#[derive(Debug)]
pub struct ClusterResults {
    /// Cluster labels for each data point (-1 indicates noise).
    pub labels: Vec<i32>,
    /// Cluster centroids (mean of x and y per cluster).
    pub centroids: HashMap<i32, (f64, f64)>,
    /// Ranges for x and y per cluster (min_x, max_x, min_y, max_y).
    pub ranges: HashMap<i32, (f64, f64, f64, f64)>,
}

/// Clusters 2D numerical data using HDBSCAN to identify clusters.
///
/// # Arguments
/// * `data` - Vector of (x, y) pairs as `Vec<(f64, f64)>`, representing any two numerical features.
/// * `min_cluster_size` - Minimum number of points required to form a cluster.
/// * `min_samples` - Number of samples in a neighborhood for a point to be a core point.
/// * `dist_metric` - Distance metric to use (e.g., Euclidean, Manhattan).
///
/// # Returns
/// A `ClusterResults` struct containing labels, centroids, and ranges for both dimensions.
///
/// # Panics
/// Panics if clustering fails (e.g., invalid hyperparameters).
pub fn compute_clusters(
    data: Vec<(f64, f64)>,
    min_cluster_size: usize,
    min_samples: usize,
    dist_metric: DistanceMetric,
) -> ClusterResults {
    // Convert data to Vec<Vec<f32>> for hdbscan
    let data_f32: Vec<Vec<f32>> = data.iter().map(|(x, y)| vec![*x as f32, *y as f32]).collect();

    // Configure HDBSCAN hyperparameters
    let hyper_params = HdbscanHyperParams::builder()
        .min_cluster_size(min_cluster_size)
        .min_samples(min_samples)
        .dist_metric(dist_metric)
        .nn_algorithm(NnAlgorithm::BruteForce) // Use KDTree for larger datasets
        .build();

    // Run HDBSCAN clustering
    let clusterer = Hdbscan::new(&data_f32, hyper_params);
    let labels = clusterer.cluster().expect("HDBSCAN clustering failed");

    // Compute centroids and ranges for each cluster
    let mut centroids = HashMap::new();
    let mut ranges = HashMap::new();

    // Identify unique cluster IDs (excluding noise, -1)
    let cluster_ids: Vec<i32> = labels.iter().filter(|&&label| label != -1).cloned().collect::<Vec<_>>();
    let unique_clusters: Vec<i32> = cluster_ids
        .into_iter()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for cluster_id in unique_clusters {
        let cluster_points: Vec<(f64, f64)> = data
            .iter()
            .zip(&labels)
            .filter(|(_, label)| *label == &cluster_id)
            .map(|(&(x, y), _)| (x, y))
            .collect();

        // Calculate centroid (mean x and y)
        let count = cluster_points.len() as f64;
        let sum_x: f64 = cluster_points.iter().map(|&(x, _)| x).sum();
        let sum_y: f64 = cluster_points.iter().map(|&(_, y)| y).sum();
        let centroid = (sum_x / count, sum_y / count);

        // Calculate ranges for x and y
        let min_x = cluster_points.iter().map(|&(x, _)| x).fold(f64::INFINITY, f64::min);
        let max_x = cluster_points.iter().map(|&(x, _)| x).fold(f64::NEG_INFINITY, f64::max);
        let min_y = cluster_points.iter().map(|&(_, y)| y).fold(f64::INFINITY, f64::min);
        let max_y = cluster_points.iter().map(|&(_, y)| y).fold(f64::NEG_INFINITY, f64::max);

        centroids.insert(cluster_id, centroid);
        ranges.insert(cluster_id, (min_x, max_x, min_y, max_y));
    }

    ClusterResults {
        labels,
        centroids,
        ranges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_clusters_with_provided_data() {
        // Test data provided by the user (price-volume pairs)
        let data = vec![
            (3150.70000000, 9.77600000),
            (3150.51000000, 0.62000000),
            (3150.25000000, 0.41000000),
            (3150.25000000, 0.10000000),
            (3150.24000000, 0.00300000),
            (3150.25000000, 7.08100000),
            (3150.24000000, 0.15800000),
            (3150.25000000, 0.41500000),
            (3150.26000000, 1.16000000),
            (3150.34000000, 1.08100000),
            (3150.35000000, 1.00000000),
            (3150.38000000, 0.08900000),
            (3150.39000000, 0.14500000),
            (3150.40000000, 1.58700000),
            (3150.43000000, 4.03400000),
            (3150.35000000, 2.00000000),
            (3150.43000000, 1.44400000),
            (3150.24000000, 0.30500000),
            (3150.43000000, 0.00200000),
            (3150.25000000, 0.27000000),
            (3150.43000000, 1.01000000),
            (3150.35000000, 0.10100000),
            (3150.26000000, 0.84600000),
            (3150.25000000, 1.53300000),
            (3150.35000000, 0.56200000),
            (3150.25000000, 0.51700000),
            (3150.24000000, 0.48300000),
            (3150.25000000, 0.19000000),
            (3150.34000000, 1.55000000),
            (3150.35000000, 0.51100000),
            (3150.24000000, 9.76900000),
            (3150.12000000, 1.18500000),
            (3150.09000000, 1.83600000),
            (3150.08000000, 1.45600000),
            (3150.04000000, 1.44300000),
            (3150.24000000, 0.00400000),
            (3150.00000000, 0.21200000),
            (3149.95000000, 0.26700000),
            (3149.57000000, 1.59000000),
            (3149.50000000, 0.08100000),
            (3149.47000000, 0.12700000),
            (3149.42000000, 0.04700000),
            (3149.35000000, 0.53400000),
            (3149.26000000, 4.65300000),
            (3149.20000000, 2.50000000),
            (3149.13000000, 4.67900000),
            (3149.10000000, 0.70200000),
            (3149.09000000, 0.08800000),
            (3149.08000000, 1.86400000),
            (3149.02000000, 1.80000000),
            (3149.01000000, 1.49800000),
            (3149.00000000, 0.51500000),
            (3149.95000000, 0.11700000),
            (3150.25000000, 0.22900000),
            (3149.60000000, 0.28000000),
            (3150.17000000, 0.03700000),
            (3150.17000000, 0.25000000),
            (3150.10000000, 0.10600000),
            (3150.17000000, 0.12700000),
            (3150.09000000, 0.03100000),
            (3150.17000000, 0.11700000),
            (3150.09000000, 0.73100000),
            (3149.94000000, 0.28900000),
            (3149.92000000, 0.38200000),
            (3150.17000000, 1.34100000),
            (3150.25000000, 0.28800000),
            (3150.25000000, 0.18800000),
            (3150.24000000, 0.04000000),
            (3149.92000000, 1.41800000),
            (3149.60000000, 1.14400000),
            (3149.60000000, 0.08000000),
            (3149.60000000, 0.29600000),
            (3149.54000000, 0.20000000),
        ];

        // Run clustering with reasonable hyperparameters
        let min_cluster_size = 10; // At least 5 points per cluster
        let min_samples = 1; // Sensitivity to noise
        let dist_metric = DistanceMetric::Euclidean;
        let clusters = compute_clusters(data.clone(), min_cluster_size, min_samples, dist_metric);

        // Assertions to verify clustering behavior
        // 1. Ensure some clusters are formed
        let num_clusters = clusters.centroids.len();
        assert!(num_clusters > 0, "No clusters were formed");

        // 2. Check that all points have a label
        assert_eq!(clusters.labels.len(), data.len(), "Number of labels does not match data points");

        // 3. Verify ranges are valid (min_x <= max_x and min_y <= max_y)
        for (&cluster_id, &(min_x, max_x, min_y, max_y)) in &clusters.ranges {
            assert!(min_x <= max_x, "Invalid x range for cluster {}", cluster_id);
            assert!(min_y <= max_y, "Invalid y range for cluster {}", cluster_id);
        }

        // 4. Check that points with frequent x-value (e.g., 3150.25) are mostly in the same cluster
        // let label_3150_25 = clusters.labels[data.iter().position(|&(x, _)| x == 3150.25).unwrap()];
        // let same_cluster_count = data
        //     .iter()
        //     .zip(&clusters.labels)
        //     .filter(|&((x, _), &label)| *x == 3150.25 && label == label_3150_25)
        //     .count();
        // let total_3150_25 = data.iter().filter(|&&(x, _)| x == 3150.25).count();
        // assert!(
        //     same_cluster_count >= total_3150_25 - 1, // Allow one outlier
        //     "Points at x=3150.25 not consistently clustered: {}/{} in same cluster",
        //     same_cluster_count,
        //     total_3150_25
        // );

        // Print results for manual inspection
        info!("Number of clusters: {}", num_clusters);
        info!("Cluster Labels: {:?}", clusters.labels);
        info!("Centroids: {:?}", clusters.centroids);
        info!("Ranges: {:?}", clusters.ranges);
    }
}
