use std::collections::HashMap;

use bincode::config::standard;

use bincode::serde::{decode_from_slice, encode_to_vec};
use colored::Colorize;
use linfa::Dataset;
use linfa::traits::{Fit, Predict};
use linfa_clustering::KMeans;
use linfa_nn::distance::LInfDist;
use ndarray::{Array1, Array2};
use rand::seq::IndexedRandom;
use rand_xoshiro::Xoshiro256PlusPlus;
use rand_xoshiro::rand_core::SeedableRng;
use tokio::fs;

mod summer_of_making;

use crate::summer_of_making::fetch_all;
use sonai_metrics::features_from_metrics;
use sonai_metrics::{TextMetricFactory, TextMetrics};

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
    let metrics: Vec<TextMetrics> = TextMetricFactory::new()?.calculate_iter(&data).collect();
    let metrics_refs: Vec<&TextMetrics> = metrics.iter().collect();
    let features = features_from_metrics(&metrics_refs);

    println!("Building dataset");
    let dataset = Dataset::new(features.clone(), Array2::<f32>::zeros((metrics.len(), 0)));

    let rng = Xoshiro256PlusPlus::seed_from_u64(69420);

    println!("Training");
    let model: KMeans<f64, _> = KMeans::params_with(2, rng, LInfDist)
        .max_n_iterations(1000)
        .n_runs(10)
        .fit(&dataset)?;

    fs::write("../sonai/model.kmeans", encode_to_vec(&model, config)?).await?;

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

    fs::write("../sonai/model.ai.cluster", [ai_label as u8]).await?;

    let cluster_counts: [usize; 2] = predicted.iter().fold([0, 0], |mut counts, &label| {
        counts[label] += 1;
        counts
    });

    let ai = cluster_counts[ai_label];
    let human = cluster_counts[human_label];
    let total = ai + human;

    let mut clusters: HashMap<usize, Vec<(TextMetrics, String)>> = HashMap::new();

    for ((label, metrics), devlog) in predicted.into_iter().zip(metrics).zip(data) {
        clusters.entry(label).or_default().push((metrics, devlog));
    }

    let mut rng = rand::rng();

    for (label, items) in clusters {
        println!(
            "\n{}",
            format!("==================== Cluster {label} ====================")
                .bold()
                .cyan()
        );

        let sample = items.choose_multiple(&mut rng, 5);

        for (i, (metrics, devlog)) in sample.into_iter().enumerate() {
            println!("{}", format!("--- Sample {i} ---").bold().yellow());
            println!("{} {}", "Features:".green(), metrics);
            println!("{}\n{}", "Text:".blue(), devlog);
            println!("{}", "-------------------------------\n".dimmed());
        }
    }

    println!(
        "ai_cluster={ai_label} human=({}%, {human}) ai=({}%, {ai})",
        human * 100 / total,
        ai * 100 / total
    );
    Ok(())
}
