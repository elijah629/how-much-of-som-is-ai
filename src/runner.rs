use linfa::traits::{Predict, Transformer};
use linfa_clustering::KMeans;
use linfa_preprocessing::norm_scaling::NormScaler;
use ndarray::Array2;
use tokio::fs;

use crate::metrics::TextMetrics;

const TEST_DEVLOG: &str = r#"I built the Biology Learning Hub to give students a complete, engaging, and visually appealing space to explore the world of biology. Biology can feel overwhelming with all its terminology, diagrams, and systems — so I wanted to create a website that simplifies complex topics while keeping the learning experience enjoyable. This hub organizes everything in one place: notes, diagrams, quizzes, study tips, and more — all designed to support biology learners at any level. What makes this website stand out is its clean, card-based design, colorful gradient sections, and smooth animations. The homepage showcases six interactive cards that guide users to key biology resources: Notes, Diagrams, Quizzes, Study Tips, Famous Biologists, and a custom Unit Converter. Whether you're reviewing cell structure, practicing with labeled diagrams, or learning about figures like Darwin and Mendel, every section is easy to navigate and mobile-friendly. A built-in dark/light mode toggle also allows users to personalize their viewing experience. Students benefit from an interactive quiz system that offers instant feedback, scoring, and the option to retry or move to the next set of questions. Clear diagrams, study strategies, and topic-based summaries help learners absorb and retain information more effectively. With a biology-themed color palette and a fully responsive layout, the Biology Learning Hub is more than just a study site — it's a companion for mastering biology in a modern, intuitive way"#;

pub async fn predict() -> anyhow::Result<()> {
    let config = bincode::config::standard();
    let file = fs::read("model.kmeans").await?;
    let model: KMeans<f64, linfa_nn::distance::L2Dist> =
        bincode::serde::decode_from_slice(&file, config)?.0;

    let sample = TextMetrics::calculate(TEST_DEVLOG);

    let n_features = 14;

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

    println!("{sample:?}");

    let scaler = NormScaler::l1();
    let array = scaler.transform(array);

    println!("Combining datasets, building feature matricies");
    let prediction = model.predict(&array);

    println!("{prediction}");

    Ok(())
}
