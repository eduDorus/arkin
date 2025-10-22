use std::sync::Arc;

use arkin_core::PersistenceReader;

use crate::{
    config::FeatureConfig,
    features::{
        DistributionType, DualRangeFeature, LagFeature, NormalizeFeature, QuantileTransformer, RangeFeature,
        RobustScaler, TwoValueFeature,
    },
    Feature,
};

pub struct FeatureFactory {}

impl FeatureFactory {
    pub async fn from_config(
        persistence: &Arc<dyn PersistenceReader>,
        configs: &[FeatureConfig],
    ) -> Vec<Arc<dyn Feature>> {
        let mut features = Vec::new();

        for config in configs {
            match config {
                FeatureConfig::Lag(c) => {
                    // Validate that input, output, and lag have the same length
                    assert_eq!(c.input.len(), c.output.len(), "input and output must have the same length");
                    assert_eq!(c.input.len(), c.lag.len(), "input and lag must have the same length");

                    for i in 0..c.input.len() {
                        let input_feature = persistence.get_feature_id(&c.input[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            LagFeature::builder()
                                .input(input_feature)
                                .output(output_feature)
                                .lag(c.lag[i])
                                .method(c.method.clone())
                                .fill_strategy(c.fill_strategy)
                                .persist(c.persist)
                                .build(),
                        ) as Arc<dyn Feature>);
                    }
                }
                FeatureConfig::Range(c) => {
                    // Validate that input, output, and data have the same length
                    assert_eq!(c.input.len(), c.output.len(), "input and output must have the same length");
                    assert_eq!(c.input.len(), c.data.len(), "input and data must have the same length");

                    for i in 0..c.input.len() {
                        let input_feature = persistence.get_feature_id(&c.input[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            RangeFeature::builder()
                                .input(input_feature)
                                .output(output_feature)
                                .data(c.data[i].clone())
                                .method(c.method.clone())
                                .fill_strategy(c.fill_strategy)
                                .persist(c.persist)
                                .build(),
                        ) as Arc<dyn Feature>);
                    }
                }
                FeatureConfig::DualRange(c) => {
                    // Validate that all arrays have the same length
                    assert_eq!(
                        c.input_1.len(),
                        c.input_2.len(),
                        "input_1 and input_2 must have the same length"
                    );
                    assert_eq!(c.input_1.len(), c.output.len(), "input_1 and output must have the same length");
                    assert_eq!(c.input_1.len(), c.data.len(), "input_1 and data must have the same length");

                    for i in 0..c.input_1.len() {
                        let input_1_feature = persistence.get_feature_id(&c.input_1[i]).await;
                        let input_2_feature = persistence.get_feature_id(&c.input_2[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            DualRangeFeature::builder()
                                .input_1(input_1_feature)
                                .input_2(input_2_feature)
                                .output(output_feature)
                                .data(c.data[i].clone())
                                .method(c.method.clone())
                                .fill_strategy(c.fill_strategy)
                                .persist(c.persist)
                                .build(),
                        ) as Arc<dyn Feature>);
                    }
                }
                FeatureConfig::TwoValue(c) => {
                    // Validate that all arrays have the same length
                    assert_eq!(
                        c.input_1.len(),
                        c.input_2.len(),
                        "input_1 and input_2 must have the same length"
                    );
                    assert_eq!(c.input_1.len(), c.output.len(), "input_1 and output must have the same length");

                    for i in 0..c.input_1.len() {
                        let input_1_feature = persistence.get_feature_id(&c.input_1[i]).await;
                        let input_2_feature = persistence.get_feature_id(&c.input_2[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            TwoValueFeature::builder()
                                .input_1(input_1_feature)
                                .input_2(input_2_feature)
                                .output(output_feature)
                                .method(c.method.clone())
                                .fill_strategy(c.fill_strategy)
                                .persist(c.persist)
                                .build(),
                        ) as Arc<dyn Feature>);
                    }
                }
                FeatureConfig::Normalize(c) => {
                    let transformer = QuantileTransformer::new(&c.data_location, &c.version, DistributionType::Normal);
                    let scaler = RobustScaler::new(&c.data_location, &c.version);

                    // Register input features
                    let mut input_features = Vec::new();
                    for input in &c.input {
                        input_features.push(persistence.get_feature_id(input).await);
                    }

                    // Register output feature
                    let output_feature = persistence.get_feature_id(&c.output).await;

                    features.push(Arc::new(
                        NormalizeFeature::builder()
                            .input(input_features)
                            .output(output_feature)
                            .transformer(transformer)
                            .scaler(scaler)
                            .method(c.method.clone())
                            .persist(c.persist)
                            .build(),
                    ) as Arc<dyn Feature>);
                }
            }
        }

        features
    }
}
