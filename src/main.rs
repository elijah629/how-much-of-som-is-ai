use bincode::config::standard;

use bincode::serde::{decode_from_slice, encode_to_vec};
use linfa::Dataset;
use linfa::traits::{Fit, Predict};
use linfa_clustering::KMeans;
use ndarray::{Array1, Array2};
use tokio::fs;

use crate::metrics::TextMetrics;
use crate::model::features_from_metrics;
use crate::summer_of_making::fetch_all;

mod metrics;
mod model;
mod summer_of_making;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = standard();

    println!("Fetching projects + devlogs");

    let data: Vec<String> = if fs::try_exists("som.data").await? {
        let data = fs::read("som.data").await?;
        let result: Vec<String> = decode_from_slice(&data, config)?.0;

        result
    } else {
        let env_map = dotenvy::EnvLoader::new().load()?;
        let logs = fetch_all(&env_map.var("JOURNEY")?).await?;

        fs::write("som.data", encode_to_vec(&logs, config)?).await?;

        logs
    };

    println!("Calculating metrics");
    let metrics: Vec<TextMetrics> = data.iter().map(|x| TextMetrics::calculate(x)).collect();
    let metrics_refs: Vec<&TextMetrics> = metrics.iter().collect();
    let features = features_from_metrics(&metrics_refs);

    println!("Building dataset");
    let dataset = Dataset::new(features.clone(), Array2::<f32>::zeros((metrics.len(), 0)));

    println!("Training");
    let model: KMeans<f64, linfa_nn::distance::L2Dist> = KMeans::params(2)
        .max_n_iterations(1000)
        .tolerance(1e-18)
        .n_runs(10)
        .fit(&dataset)?;

    fs::write("model.kmeans", encode_to_vec(&model, config)?).await?;

    println!("Predicting");
    let predicted: Array1<usize> = model.predict(&features);

    let (emoji_sums, counts) = metrics.iter().zip(predicted.iter()).fold(
        ([0.0f64; 2], [0usize; 2]),
        |(mut current_emoji_sums, mut current_counts), (metric, &label)| {
            current_emoji_sums[label] += metric.emoji_rate;
            current_counts[label] += 1;
            (current_emoji_sums, current_counts)
        },
    );

    let avg_emoji = [
        emoji_sums[0] / (counts[0].max(1) as f64),
        emoji_sums[1] / (counts[1].max(1) as f64),
    ];

    let ai_label = if avg_emoji[0] > avg_emoji[1] { 0 } else { 1 };
    let human_label = if avg_emoji[0] > avg_emoji[1] { 1 } else { 0 };

    fs::write("model.ai.cluster", [ai_label as u8]).await?;

    println!("AI Cluster for model revision = {ai_label}");

    let cluster_counts: [usize; 2] = predicted.iter().fold([0, 0], |mut counts, &label| {
        counts[label] += 1;
        counts
    });

    let ai = cluster_counts[ai_label];
    let human = cluster_counts[human_label];
    let total = ai + human;

    println!(
        "human={human}, ai={ai}, human%={}, ai%={}",
        human * 100 / total,
        ai * 100 / total
    );

    let mut count = 0;
    let mut other = 0;
    for ((label, metrics), devlog) in predicted.into_iter().zip(metrics).zip(data).skip(7000) {
        if label == 0 && count != 2 {
            count += 1;
        } else if label == 1 && other != 2 {
            other += 1;
        } else if (label == 0 && count == 2) || label == 1 && other == 2 {
            continue;
        }
        println!("--{label}--\n {devlog}\n{metrics:?}");
    }

    Ok(())
}
