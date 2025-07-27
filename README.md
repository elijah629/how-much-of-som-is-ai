# how much of som is ai?

I trained a ~~271~~ ~~250~~ 175 ✔️ byte AI Detection KMeans model on every SoM devlog. Here's how
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
#[derive(Debug, Serialize)]
pub struct TextMetrics {
    // higher = more AI-like

    // Rates
    pub emoji_rate: f64,     // Emoji * 2 / words
    pub buzzword_ratio: f64, // Buzzwords * 2 / words
    pub markdown_use: f64,   // ai-like markdown syntax present

    // Counts
    pub irregular_ellipsis: f64,   // bad ellipses (unicode ...)
    pub rule_of_threes: f64,       // It's not just _, it's _ (i know this rule is not 3, but two)
    pub devlog_day_count: f64,     // /(dev(-)?log|day)( \D+)?/gi
    pub html_escapes: f64,         // &amp; etc
    pub irregular_quotations: f64, // Fancy quotation marks / total quotation marks
    pub irregular_dashes: f64,     // Em-dashes / total dashes
}
```

I then pipe them all into a KMeans model, train it a few (10 billion) times.

## DIY

Place `JOURNEY=` in `.env` to fetch devlogs & projects, or use the provided `som.data`
file.

## WASM

For demo purposes, this crate has been ported to WASM and a static site where
you can run the AI detection model on your own text. Compile the wasm demo
yourself with:

```sh
wasm-pack build --release -d demo/src/pkg
```

All the opt flags have been preconfigured in `Cargo.toml`
