use bincode::config::standard;

use bincode::serde::{decode_from_slice, encode_to_vec};
use linfa::Dataset;
use linfa::traits::{Fit, Predict, Transformer};
use linfa_clustering::KMeans;
use linfa_preprocessing::norm_scaling::NormScaler;
use ndarray::{Array1, Array2};
use tokio::fs;

use crate::metrics::TextMetrics;
use crate::runner::predict;

mod devlogs;
mod metrics;
mod runner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    predict().await?;

    return Ok(());

    let config = standard();

    println!("Fetching devlogs");
    let devlogs: Vec<String> = if fs::try_exists("devlogs.data").await? {
        let data = fs::read("devlogs.data").await?;
        let result: Vec<String> = decode_from_slice(&data, config)?.0;

        result
    } else {
        let env_map = dotenvy::EnvLoader::new().load()?;

        let logs = devlogs::get_all_devlogs(env_map.var("JOURNEY")?).await?;

        fs::write("devlogs.data", encode_to_vec(&logs, config)?).await?;

        logs
    };

    println!("Calculating embeddings");
    /*let embeddings = if fs::try_exists("embeddings.data").await? {
        let data = fs::read("embeddings.data").await?;
        let result: Vec<Vec<f32>> = decode_from_slice(&data, config)?.0;

        result
    } else {
        let embedder = TextEmbedder::from_pretrained_hf(
            "Bert",
            "sentence-transformers/all-MiniLM-L12-v2",
            None,
            None,
            None,
        )?;

        let mut embeddings = Vec::with_capacity(devlogs.len());

        let chunks = devlogs.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
        let chunks = chunks.as_slice();

        /*let chunks = chunks.chunks(4);

        let total_chunks = chunks.len();

        for (i, devlogs) in chunks.enumerate() {
            println!(
                "Embedding {}/{} {}%",
                i + 1,
                total_chunks,
                (i + 1) * 100 / total_chunks
            );

            let results = embedder.embed(devlogs, Some(devlogs.len()), None).await?;

            let vectors = results
                .iter()
                .map(|e| e.to_dense())
                .collect::<Result<Vec<_>, _>>()?;

            embeddings.extend(vectors);
        }*/

        println!("BEGIN > CUDA COMPUTE");
        let results = embedder.embed(chunks, Some(4), None).await?;
        println!("END > CUDA COMPUTE (processed all!)");

        let vectors = results
            .iter()
            .map(|e| e.to_dense())
            .collect::<Result<Vec<_>, _>>()?;

        embeddings.extend(vectors);

        fs::write(
            "embeddings.data",
            bincode::encode_to_vec(&embeddings, config)?,
        )
        .await?;

        embeddings
    };*/

    println!("Zipping metrics and embeddings");
    let data: Vec<TextMetrics> = devlogs
        .iter()
        //.zip(embeddings)
        //.map(|(text, embeddings)| TextMetrics::calculate(text, embeddings))
        .map(|x| TextMetrics::calculate(x))
        .collect();

    println!("KMeans calculation");
    let n_samples = data.len();
    let n_features = 14; // + 384;

    let mut array = Array2::<f64>::zeros((n_samples, n_features));

    println!("Combining datasets, building feature matricies");
    for (i, sample) in data.iter().enumerate() {
        array[[i, 0]] = sample.emoji_rate;
        array[[i, 1]] = sample.irregular_quotation_rate;
        array[[i, 2]] = sample.irregular_dash_rate;
        array[[i, 3]] = sample.avg_sentence_length;
        array[[i, 4]] = sample.avg_word_length;
        array[[i, 5]] = sample.punctuation_rate;
        array[[i, 6]] = sample.ellipsis_rate;
        array[[i, 7]] = sample.markdown_use;
        array[[i, 8]] = sample.avg_syllables_per_word;
        array[[i, 9]] = sample.flesch_reading_ease;
        array[[i, 10]] = sample.flesch_kincaid_grade;
        array[[i, 11]] = sample.uppercase_word_rate;
        array[[i, 12]] = sample.digit_rate;
        array[[i, 13]] = sample.sentence_length_stddev;

        //for (j, embedding) in sample.embeddings.iter().enumerate() {
        //    array[[i, j + 14]] = *embedding as f64;
        //}
    }
    println!("Normalizing dataset");
    let scaler = NormScaler::l1();
    let array = scaler.transform(array);

    println!("Building dataset");
    let dataset = Dataset::new(array.clone(), Array2::<f32>::zeros((n_samples, 0)));

    println!("train");
    let model: KMeans<f64, linfa_nn::distance::L2Dist> =
        KMeans::params(2).max_n_iterations(1000).fit(&dataset)?;

    fs::write("model.kmeans", encode_to_vec(&model, config)?).await?;

    println!("Predicting");
    let predicted: Array1<usize> = model.predict(&array);

    let mut emoji_sums = [0.0f64; 2];
    let mut counts = [0usize; 2];

    for (datum, label) in data.iter().zip(predicted.iter()) {
        emoji_sums[*label] += datum.emoji_rate;
        counts[*label] += 1;
    }

    let avg_emoji = [
        emoji_sums[0] / counts[0].max(1) as f64,
        emoji_sums[1] / counts[1].max(1) as f64,
    ];

    // Step 2: Determine which label is AI (higher avg emoji_rate)
    let ai_label = if avg_emoji[0] > avg_emoji[1] { 0 } else { 1 };

    println!("{ai_label}");

    let human = data
        .iter()
        .zip(predicted.iter())
        .filter(|(_, label)| **label != ai_label)
        .count();

    let ai = data
        .iter()
        .zip(predicted.iter())
        .filter(|(_, label)| **label == ai_label)
        .count();

    let total = ai + human;

    println!("Done!");
    println!(
        "human={human}, ai={ai}, human%={}, ai%={}",
        human * 100 / total,
        ai * 100 / total
    ); /*let mut rng = rng();
    

    // Collect filtered items into a Vec
    let mut examples: Vec<_> = data
    .iter()
    .zip(predicted.iter())
    .filter(|(_, label)| **label == cluster_id)
    .collect();

    // Shuffle in place
    examples.shuffle(&mut rng);

    // Take up to 10 examples and print them
    for ((text, sample), _) in examples.into_iter().take(10) {
    println!("  - {text}\n  {sample:?}");
    }
    println!();*/

    //println!("{cm:?}");
    //println!("accuracy {}, MCC {}", cm.accuracy(), cm.mcc());

    /*.map(|x| {
        let metrics = TextMetrics::calculate(x.as_str());

        (x, metrics)
    });*/

    /*for (devlog, metric) in metrics {
        if metric.markdown_use {
            println!("{devlog:?}");
        }
    }*/

    /*let (train, valid) = linfa_datasets::winequality()
        .map_targets(|x| if *x > 6 { "good" } else { "bad" })
        .split_with_ratio(0.9);

    let model = GaussianNb::params().fit(&train)?;

    // Predict the validation dataset
    let pred = model.predict(&valid);

    // Construct confusion matrix
    let cm = pred.confusion_matrix(&valid)?;

    println!("{cm:?}");
    println!("accuracy {}, MCC {}", cm.accuracy(), cm.mcc());*/

    /*)let config = standard();

    let devlogs: Vec<String> = if fs::try_exists("devlogs.data").await? {
        let data = fs::read("devlogs.data").await?;
        let result: Vec<String> = bincode::decode_from_slice(&data, config)?.0;

        result
    } else {
        let logs = devlogs::get_all_devlogs().await?;

        fs::write("devlogs.data", bincode::encode_to_vec(&logs, config)?).await?;

        logs
    };

    let devlog_count = devlogs.len();
    println!("Total devlogs: {devlog_count}");

    let embedder = TextEmbedder::from_pretrained_hf(
        "Bert",
        "sentence-transformers/all-MiniLM-L6-v2",
        None,
        None,
        None,
    )?;

    let embeddings = if fs::try_exists("embeddings.data").await? {
        let data = fs::read("embeddings.data").await?;
        let result: Vec<Vec<f32>> = bincode::decode_from_slice(&data, config)?.0;

        result
    } else {
        let mut embeddings = Vec::with_capacity(devlogs.len());

        let chunks = devlogs.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

        let chunks = chunks.as_slice();

        /*let chunks = chunks.chunks(4);

        let total_chunks = chunks.len();

        for (i, devlogs) in chunks.enumerate() {
            println!(
                "Embedding {}/{} {}%",
                i + 1,
                total_chunks,
                (i + 1) * 100 / total_chunks
            );

            let results = embedder.embed(devlogs, Some(devlogs.len()), None).await?;

            let vectors = results
                .iter()
                .map(|e| e.to_dense())
                .collect::<Result<Vec<_>, _>>()?;

            embeddings.extend(vectors);
        }*/

        let results = embedder.embed(chunks, Some(4), None).await?;

        let vectors = results
            .iter()
            .map(|e| e.to_dense())
            .collect::<Result<Vec<_>, _>>()?;

        embeddings.extend(vectors);

        fs::write(
            "embeddings.data",
            bincode::encode_to_vec(&embeddings, config)?,
        )
        .await?;

        embeddings
    };

    let query = "Added MD5-based diffing to improve file sync accuracy";
    let query_embed = embedder.embed(&[query], None, None).await?[0].to_dense()?;

    let mut sims: Vec<(usize, f32)> = embeddings
        .iter()
        .enumerate()
        .map(|(i, v)| (i, cosine_similarity(&query_embed, v)))
        .collect();

    sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Top matches for query '{query}':");
    for (idx, score) in sims.iter().take(10) {
        println!("[Score: {:.4}] {}", score, devlogs[*idx]);
    }*/

    Ok(())
}
