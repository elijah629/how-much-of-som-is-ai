use linfa_clustering::KMeans;
use model::{TextMetrics, features_from_metrics, point_confidence};
use once_cell::sync::Lazy;

const AI_CLUSTER: usize = include_bytes!("../../training-bin/model.ai.cluster")[0] as usize;

static MODEL: Lazy<KMeans<f64, linfa_nn::distance::L2Dist>> = Lazy::new(|| {
    let config = bincode::config::standard();
    bincode::serde::decode_from_slice(include_bytes!("../../training-bin/model.kmeans"), config)
        .unwrap()
        .0
});

#[derive(Debug, serde::Serialize)]
pub struct Prediction {
    pub chance_ai: f64,
    pub chance_human: f64,
    pub metrics: TextMetrics,
}

pub fn predict(devlog: &str) -> Prediction {
    let sample = TextMetrics::calculate(devlog);
    let features = features_from_metrics(&[&sample]);
    let features = features.row(0);
    let model = &*MODEL;

    let (_, sims) = point_confidence(model, features);

    let chance_human = sims.get(AI_CLUSTER).cloned().unwrap_or(0.0) * 100.0;
    let chance_ai = 100.0 - chance_human;

    Prediction {
        metrics: sample,
        chance_ai,
        chance_human,
    }
}
