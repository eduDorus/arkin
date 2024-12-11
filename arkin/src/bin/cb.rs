// Bring catboost module into the scope
use catboost_rs as catboost;

fn main() {
    // Log our current directory
    println!("Current directory: {:?}", std::env::current_dir().unwrap());
    // Load the trained model
    let model = catboost::Model::load("./models/btc_log_return_10min.cbm").unwrap();

    println!("Number of cat features {}", model.get_cat_features_count());
    println!("Number of float features {}", model.get_float_features_count());

    // Data
    let data = vec![
        vec![
            0.446507, 0.391301, -0.406777, -0.169099, -0.080779, 0.25762, -27.394094, 0.0004, 0.000369, 0.000367,
            0.430656, 0.644107, -0.000447, 0.000357, -0.000303, -0.000144, 0.000803, -0.598385, 0.000273, -0.213115,
            -0.17201, 0.000285, -0.143569,
        ],
        vec![
            0.440472, 0.390638, -0.282312, -0.190224, -0.078039, 0.251508, -29.910406, 0.000393, 0.00037, 0.000367,
            0.402756, 0.640727, -0.000345, 0.000388, -0.000194, -0.000151, 0.000733, 0.520981, 0.000471, 0.377049,
            -0.073011, 0.000311, -0.069195,
        ],
        vec![
            0.432029, 0.389944, -0.325336, -0.208501, -0.080441, 0.245433, -32.960624, 0.000394, 0.00037, 0.000367,
            0.368795, 0.636559, -0.000505, 0.000139, -0.000329, -0.000177, 0.000644, 0.304617, 0.000222, -0.3,
            -0.019064, 0.000298, -0.102167,
        ],
        vec![
            0.421373, 0.389221, -0.451995, -0.226899, -0.125458, 0.239351, -37.344436, 0.000395, 0.000371, 0.000367,
            0.337428, 0.63219, -0.000502, 0.000057, -0.000306, -0.000195, 0.000558, -0.812292, 0.000148, -0.266667,
            -0.132382, 0.000277, -0.125667,
        ],
        vec![
            0.40892, 0.388471, -0.471863, -0.245415, -0.138888, 0.233255, -42.863765, 0.000396, 0.000372, 0.000367,
            0.309268, 0.627727, -0.000491, -0.000014, -0.000284, -0.000208, 0.000477, -0.593788, 0.00024, -0.180328,
            -0.198298, 0.000272, -0.133476,
        ],
        vec![
            0.395219, 0.387697, -0.53185, -0.264865, -0.142161, 0.227149, -49.178442, 0.000401, 0.000373, 0.000367,
            0.275719, 0.622307, -0.000646, -0.000267, -0.000409, -0.000237, 0.000379, -0.606843, 0.000325, -0.2,
            -0.256661, 0.000279, -0.142979,
        ],
        vec![
            0.380877, 0.386898, -0.395648, -0.281823, -0.129165, 0.221054, -54.59889, 0.0004, 0.000374, 0.000368,
            0.251827, 0.617274, -0.000536, -0.000239, -0.000292, -0.000245, 0.000297, 0.079374, 0.000327, 0.016393,
            -0.208656, 0.000286, -0.120212,
        ],
        vec![
            0.366497, 0.386078, -0.382744, -0.296342, -0.134301, 0.214969, -59.048451, 0.000399, 0.000375, 0.000368,
            0.231654, 0.612351, -0.000543, -0.000325, -0.000292, -0.000251, 0.000218, -0.148899, 0.000372, 0.275862,
            -0.20012, 0.000298, -0.06363,
        ],
        vec![
            0.352636, 0.385238, -0.381711, -0.308751, -0.149427, 0.208886, -62.825641, 0.000401, 0.000376, 0.000368,
            0.216647, 0.607755, -0.00033, -0.000162, -0.0001, -0.00023, 0.000168, 0.693899, 0.000198, 0.310345,
            -0.072403, 0.000284, -0.010205,
        ],
        vec![
            0.339781, 0.384378, -0.370297, -0.319124, -0.145309, 0.202814, -65.758372, 0.000402, 0.000377, 0.000368,
            0.203707, 0.603328, -0.000381, -0.000267, -0.000161, -0.00022, 0.000114, 0.100065, 0.000275, 0.034483,
            -0.047764, 0.000283, -0.003821,
        ],
    ];

    // Print the shape of the data
    println!("Data shape: {:?}", data.len());
    println!("Data columns: {:?}", data[0].len());

    let cat_data = vec![
        vec![1, 2, 120, 0],
        vec![1, 2, 121, 1],
        vec![1, 2, 122, 2],
        vec![1, 2, 123, 3],
        vec![1, 2, 124, 4],
        vec![1, 2, 125, 5],
        vec![1, 2, 126, 6],
        vec![1, 2, 127, 7],
        vec![1, 2, 128, 8],
        vec![1, 2, 129, 9],
    ];

    // Convert to strings
    let cat_data: Vec<Vec<String>> = cat_data.iter().map(|x| x.iter().map(|y| y.to_string()).collect()).collect();
    // Apply the model
    let prediction = model.calc_model_prediction(data, cat_data).unwrap();
    println!("Prediction {:?}", prediction);
}