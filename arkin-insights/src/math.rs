fn sum_f64_par(data: &[f64]) -> f64 {
    data.par_iter().sum()
}

fn mean_f64_par(data: &[f64]) -> f64 {
    // if data.is_empty() {
    //     return None;
    // }
    let sum: f64 = sum_f64_par(data);
    let n = data.len() as f64;
    sum / n
}

fn variance_f64_par(data: &[f64]) -> f64 {
    // if data.len() < 2 {
    //     return None;
    // }
    let mean = mean_f64_par(data);
    let sum_sq_diff: f64 = data.par_iter().map(|x| (x - mean).powi(2)).sum();
    let n = data.len() as f64 - 1.0;
    sum_sq_diff / n
}

fn std_dev_f64_par(data: &[f64]) -> f64 {
    variance_f64_par(data).sqrt()
}
