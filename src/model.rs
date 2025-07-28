use crate::metrics::TextMetrics;
use linfa_clustering::KMeans;
use linfa_nn::distance::{Distance, L2Dist};
use ndarray::{Array1, Array2, ArrayView1, Axis};

pub fn features_from_metrics(data: &[&TextMetrics]) -> Array2<f64> {
    let n_features = 11;
    let n_samples = data.len();

    let mut array = Array2::<f64>::zeros((n_samples, n_features));

    for (i, sample) in data.iter().enumerate() {
        array[[i, 0]] = sample.emoji_rate;

        array[[i, 1]] = sample.irregular_quotations * 5.;
        array[[i, 2]] = sample.irregular_dashes * 5.;
        array[[i, 3]] = sample.irregular_ellipsis;
        array[[i, 4]] = sample.irregular_markdown;

        array[[i, 5]] = sample.not_just_count * 5.;
        array[[i, 6]] = sample.devlog_count;
        array[[i, 7]] = sample.html_escape_count * 5.;
        array[[i, 8]] = sample.buzzword_count * 10.;

        array[[i, 9]] = sample.hashtags;
        array[[i, 10]] = sample.labels;
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
