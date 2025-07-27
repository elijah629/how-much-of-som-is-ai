mod metrics;
mod model;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use once_cell::sync::Lazy;

#[cfg(target_arch = "wasm32")]
static MODEL: Lazy<linfa_clustering::KMeans<f64, linfa_nn::distance::L2Dist>> = Lazy::new(|| {
    let config = bincode::config::standard();
    bincode::serde::decode_from_slice(include_bytes!("../model.kmeans"), config)
        .unwrap()
        .0
});

#[cfg(target_arch = "wasm32")]
static AI_CLUSTER: usize = include_bytes!("../model.ai.cluster")[0] as usize;

#[cfg(target_arch = "wasm32")]
#[derive(Debug, serde::Serialize)]
struct Output {
    percent_ai: f64,
    percent_human: f64,
    ai: bool,
    metrics: crate::metrics::TextMetrics,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn predict(devlog: &str) -> JsValue {
    use crate::{
        metrics::TextMetrics,
        model::{features_from_metrics, point_confidence},
    };
    use linfa::traits::{Predict, Transformer};
    use linfa_preprocessing::norm_scaling::NormScaler;
    use ndarray::Array2;

    let model = &*MODEL;

    let sample = TextMetrics::calculate(devlog);
    let features = features_from_metrics(&[&sample]);
    let features = features.row(0);

    let (distances, sims) = point_confidence(model, features);

    let predicted = sims
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    // normalize sims → percentages
    let percent_human = sims.get(AI_CLUSTER).cloned().unwrap_or(0.0) * 100.0;
    let percent_ai = 100.0 - percent_human;

    let prediction = model.predict(&features);

    serde_wasm_bindgen::to_value(&Output {
        metrics: sample,
        ai: prediction == AI_CLUSTER,
        percent_ai,
        percent_human,
    })
    .unwrap()
}
