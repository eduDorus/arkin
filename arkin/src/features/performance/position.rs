use crate::config::PositionConfig;
use crate::features::{Feature, FeatureDataRequest, FeatureDataResponse, FeatureId, Latest, NodeId, Window};
use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Debug)]
pub struct PositionFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    data: Vec<FeatureDataRequest>,
    input_position_price: Latest,
    input_position_quantity: Latest,
    input_fill_price: Window,
    input_fill_quantity: Window,
    output_price: FeatureId,
    output_quantity: FeatureId,
}

impl PositionFeature {
    pub fn from_config(config: &PositionConfig) -> Self {
        let data = vec![
            config.input_position_price.to_owned().into(),
            config.input_position_quantity.to_owned().into(),
            config.input_fill_price.to_owned().into(),
            config.input_fill_quantity.to_owned().into(),
        ];

        let sources = vec![
            config.input_position_price.from.clone(),
            config.input_position_quantity.from.clone(),
            config.input_fill_price.from.clone(),
            config.input_fill_quantity.from.clone(),
        ];
        PositionFeature {
            id: config.id.to_owned(),
            sources,
            data,
            input_fill_price: config.input_fill_price.to_owned(),
            input_fill_quantity: config.input_fill_quantity.to_owned(),
            input_position_price: config.input_position_price.to_owned(),
            input_position_quantity: config.input_position_quantity.to_owned(),
            output_price: config.output_price.to_owned(),
            output_quantity: config.output_quantity.to_owned(),
        }
    }
}

impl Feature for PositionFeature {
    fn id(&self) -> &NodeId {
        &self.id
    }

    fn sources(&self) -> &[NodeId] {
        &self.sources
    }

    fn data(&self) -> &[FeatureDataRequest] {
        &self.data
    }

    fn calculate(&self, data: FeatureDataResponse) -> Result<HashMap<FeatureId, f64>> {
        debug!("Calculating Position");
        let position_price = data.last(&self.input_position_price.feature_id).unwrap_or(0.);
        let position_quantity = data.last(&self.input_position_quantity.feature_id).unwrap_or(0.);

        let fill_price = data.get(&self.input_fill_price.feature_id);
        let fill_quantity = data.get(&self.input_fill_quantity.feature_id);
        info!(
            "CALCULATION: Position price: {} Position quantity: {} Fill price: {:?} Fill quantity: {:?}",
            position_price, position_quantity, fill_price, fill_quantity
        );
        let mut position_price_sum = position_price * position_quantity;
        let mut position_quantity_sum = position_quantity;

        fill_price.into_iter().zip(fill_quantity).for_each(|(price, quantity)| {
            position_price_sum += price * quantity;
            position_quantity_sum += quantity;
        });

        let position_price = if position_quantity_sum == 0.0 {
            0.0
        } else {
            position_price_sum / position_quantity_sum
        };
        info!(
            "Calculated: Position price: {} Position quantity: {}",
            position_price, position_quantity_sum
        );

        let mut res = HashMap::new();
        res.insert(self.output_price.clone(), position_price);
        res.insert(self.output_quantity.clone(), position_quantity_sum);
        Ok(res)
    }
}
