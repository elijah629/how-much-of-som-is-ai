mod metrics;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
static MODEL_BYTES: &[u8] = include_bytes!("../model.kmeans");

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn predict(devlog: &str) -> bool {
    use crate::metrics::TextMetrics;
    use linfa::traits::{Predict, Transformer};
    use linfa_clustering::KMeans;
    use linfa_preprocessing::norm_scaling::NormScaler;
    use ndarray::Array2;

    let config = bincode::config::standard();
    let model: KMeans<f64, linfa_nn::distance::L2Dist> =
        bincode::serde::decode_from_slice(MODEL_BYTES, config)
            .unwrap()
            .0;

    let sample = TextMetrics::calculate(devlog);

    let n_features = 15;

    let mut array = Array2::<f64>::zeros((1, n_features));

    array[[0, 0]] = sample.emoji_rate;
    array[[0, 1]] = sample.irregular_quotation_rate;
    array[[0, 2]] = sample.irregular_dash_rate;
    array[[0, 3]] = sample.avg_sentence_length;
    array[[0, 4]] = sample.avg_word_length;
    array[[0, 5]] = sample.punctuation_rate;
    array[[0, 6]] = sample.ellipsis_rate;
    array[[0, 7]] = sample.markdown_use;
    array[[0, 8]] = sample.avg_syllables_per_word;
    array[[0, 9]] = sample.flesch_reading_ease;
    array[[0, 10]] = sample.flesch_kincaid_grade;
    array[[0, 11]] = sample.uppercase_word_rate;
    array[[0, 12]] = sample.digit_rate;
    array[[0, 13]] = sample.sentence_length_stddev;
    array[[0, 14]] = sample.rule_of_threes;

    let scaler = NormScaler::l2();
    let array = scaler.transform(array);

    let prediction = model.predict(&array);

    prediction[0] == 0 // 0 means ai, 1 means human
}
