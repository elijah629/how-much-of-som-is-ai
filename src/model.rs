use crate::metrics::TextMetrics;
use linfa_clustering::KMeans;
use linfa_nn::distance::{Distance, L2Dist};
use ndarray::{Array1, Array2, ArrayView1, Axis};

pub fn features_from_metrics(data: &[&TextMetrics]) -> Array2<f64> {
    let n_features = 9;
    let n_samples = data.len();

    let mut array = Array2::<f64>::zeros((n_samples, n_features));

    for (i, sample) in data.iter().enumerate() {
        array[[i, 0]] = sample.emoji_rate;
        array[[i, 1]] = sample.irregular_quotations;
        array[[i, 2]] = sample.irregular_dashes;
        array[[i, 3]] = sample.irregular_ellipsis;

        array[[i, 4]] = sample.markdown_use;

        // array[[i, 8]] = sample.avg_syllables_per_word;
        // array[[i, 9]] = sample.flesch_reading_ease;
        // array[[i, 10]] = sample.flesch_kincaid_grade;
        // array[[i, 11]] = sample.uppercase_word_rate;
        // array[[i, 12]] = sample.digit_rate;
        // array[[i, 13]] = sample.sentence_length_stddev;

        array[[i, 5]] = sample.rule_of_threes;
        array[[i, 6]] = sample.devlog_day_count;
        array[[i, 7]] = sample.html_escapes;
        array[[i, 8]] = sample.buzzword_ratio;
        // array[[i, 9]] = sample.avg_sentence_length;
        // array[[i, 9]] = sample.avg_word_length;

        // array[[i, 8]] = sample.perplexity;
        // array[[i, 9]] = sample.burstiness;

        //for (j, embedding) in sample.embeddings.iter().enumerate() {
        //    array[[i, j + 14]] = *embedding as f64;
        //}
    }

    array
}

pub fn point_confidence(
    model: &KMeans<f64, L2Dist>,
    observation: ArrayView1<f64>,
) -> (Array1<f64>, Array1<f64>) {
    let centroids = model.centroids();
    let dist_fn = L2Dist;

    let distances = centroids
        .axis_iter(Axis(0))
        .map(|centroid_row| dist_fn.distance(observation, centroid_row))
        .collect::<Array1<_>>();

    let mut sims = distances.mapv(|d| 1.0 / (1.0 + d));
    let sum: f64 = sims.sum();
    if sum > 0.0 {
        sims /= sum;
    }
    (distances, sims)
}
