# how much of som is ai?

I trained a 255 byte AI Detection KMeans model on every SoM devlog. Here's how
it went!

## How this version works

### Training

- Get every devlog from SoM api
- Calculate text features
- KMeans training
- Export model
- Meause the model against its own training data to see how much of SoM is AI.

### Inference

- Load the previously exported model
- Compute text features
- Perform a prediction with the features and exported model

## My whole brainsstorming era

First I made an embedder using L12 MiniLM on huggingface. This suprisingly
worked first try, embedding every single devlog in under 30s (credit to my super
computer). Due to the popular "Everything is AI whyyy" complaints from hackclub
members, I then attempted to measure how much of the devlogs were written by AI.
I decided to use the `linfa` ecosystem by the `rust-ml` group. I use a KMeans
algorithim to classify the text into two unlabeled clusters based off of a
feature matrix. I calculate these features:

```rust
pub struct TextMetrics {
    // higher = more AI-like
    pub emoji_rate: f64,               // Emoji / words
    pub irregular_quotation_rate: f64, // Fancy quotation marks / total quotation marks
    pub irregular_dash_rate: f64,      // Em-dashes / total dashes
    pub avg_sentence_length: f64,      // Words / sentences
    pub avg_word_length: f64,          // Characters / words
    pub punctuation_rate: f64,         // Punctuation marks / words
    pub ellipsis_rate: f64,            // Ellipses / sentences
    pub markdown_use: f64,             // markdown syntax present

    pub avg_syllables_per_word: f64, // total syllables / total words
    pub flesch_reading_ease: f64,    // 206.835 – 1.015*(words/sentences) – 84.6*(syllables/words)
    pub flesch_kincaid_grade: f64,   // 0.39*(words/sentences) + 11.8*(syllables/words) – 15.59
    pub uppercase_word_rate: f64,    // ALL-CAPS words / total words
    pub digit_rate: f64,             // words containing digits / total words
    pub sentence_length_stddev: f64, // stdev of words per sentence
}
```

I then pipe them all into a KMeans model, train it a few (10 billion) times.

## WASM

For demo purposes, this crate has been ported to WASM and a static site where
you can run the AI detection model on your own text. Compile the demo wasm it
yourself with:

```sh
wasm-pack build --release -d demo/src/pkg
```

All the opt flags have been preconfigured in `Cargo.toml`
