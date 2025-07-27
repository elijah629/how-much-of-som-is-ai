use pulldown_cmark::Event;
use pulldown_cmark::Parser;
use serde::Serialize;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Serialize)]
pub struct TextMetrics {
    // higher = more AI-like

    // Rates
    pub emoji_rate: f64,     // Emoji / words
    pub buzzword_ratio: f64, // Buzzwords * 2 / words
    // pub avg_sentence_length: f64, // Words / sentences
    // pub avg_word_length: f64,     // Characters / words
    // pub punctuation_rate: f64, // Punctuation marks / words
    pub markdown_use: f64, // markdown syntax present

    // pub avg_syllables_per_word: f64, // total syllables / total words
    // pub flesch_reading_ease: f64,    // 206.835 – 1.015*(words/sentences) – 84.6*(syllables/words)
    // pub flesch_kincaid_grade: f64,   // 0.39*(words/sentences) + 11.8*(syllables/words) – 15.59
    // pub uppercase_word_rate: f64,    // ALL-CAPS words / total words
    // pub digit_rate: f64,             // words containing digits / total words
    // pub sentence_length_stddev: f64, // stdev of words per sentence

    // Counts
    pub irregular_ellipsis: f64, // bad ellipses
    pub rule_of_threes: f64,     // It's not just _, it's _
    pub devlog_day_count: f64,
    pub html_escapes: f64,
    pub irregular_quotations: f64, // Fancy quotation marks / total quotation marks
    pub irregular_dashes: f64,     // Em-dashes / total dashes
                                   // Standard metrics
                                   // pub perplexity: f64,
                                   // pub burstiness: f64,
}

impl TextMetrics {
    pub fn calculate(text: &str) -> Self {
        // existing markdown vs non-markdown
        let markdown = Parser::new(text)
            .filter(|event| {
                matches!(
                    event,
                    Event::Code(_)
                        | Event::InlineMath(_)
                        | Event::DisplayMath(_)
                        | Event::Html(_)
                        | Event::FootnoteReference(_)
                        | Event::TaskListMarker(_)
                        | Event::Rule
                        | Event::InlineHtml(_)
                )
            })
            .count();

        let text = text.to_ascii_lowercase();

        // split sentences
        let sentence_splits: Vec<&str> = text
            .split(|c| ".!?".contains(c))
            .filter(|s| !s.trim().is_empty())
            .collect();

        let sentence_count = sentence_splits.len().max(1);

        /*        let sentence_lengths: Vec<usize> = sentence_splits
            .iter()
            .map(|s| s.split_whitespace().count())
            .collect();

        // let burstiness = Self::burstiness(&sentence_lengths);

        // word-level stats

        // let perplexity = Self::compute_perplexity(&words, 2);

        let word_count = words.len().max(1);*/
        // let total_word_chars: usize = words.iter().map(|w| w.chars().count()).sum();

        // vocabulary distribution
        /*let mut freqs = HashMap::new();
        for w in &words {
            let w_lower = w
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            *freqs.entry(w_lower).or_insert(0usize) += 1;
        }*/

        // syllable estimate: vowel groups per word
        /*let total_syllables: usize = words.iter().map(|w| count_syllables(w)).sum();

        // stopwords, uppercase, digits
        let mut uppercase_count = 0;
        let mut digit_count = 0;
        for w in &words {
            let w_clean = w.trim_matches(|c: char| !c.is_alphanumeric());
            if w_clean.chars().all(|c| c.is_uppercase()) && w_clean.len() > 1 {
                uppercase_count += 1;
            }
            if w_clean.chars().any(|c| c.is_ascii_digit()) {
                digit_count += 1;
            }
        }*/
        let words = text.split_whitespace().filter(|w| !w.is_empty()).count();

        // sentence length standard deviation
        /*let mean_sl = sentence_word_counts.iter().sum::<usize>() as f64 / sentence_count as f64;
        let var_sl = sentence_word_counts
            .iter()
            .map(|&c| {
                let diff = c as f64 - mean_sl;
                diff * diff
            })
            .sum::<f64>()
            / sentence_count as f64;
         let stddev_sl = var_sl.sqrt();*/

        let mut emoji_count = 0;
        // let mut reg_dash = 0;
        let mut irr_dash = 0;
        // let mut reg_quote = 0;
        let mut irr_quote = 0;
        // let mut punct_count = 0;
        for grapheme in text.graphemes(true) {
            if emojis::get(grapheme).is_some() {
                emoji_count += 1;
                continue;
            }
            for c in grapheme.chars() {
                match c {
                    // '-' => reg_dash += 1,
                    '–' | '—' | '‒' | '―' => irr_dash += 1,
                    // '"' | '\'' => reg_quote += 1,
                    '“' | '”' | '‘' | '’' => irr_quote += 1,
                    // '.' | '!' | '?' | ',' | ':' | ';' => punct_count += 1,
                    _ => {}
                }
            }
        }

        let irr_ell = text.matches('…').count();
        // let ascii_ell = text.matches("...").count();

        // let ell_count = unicode_ell + ascii_ell;

        // finalize calculations
        // let wc = word_count as f64;
        let sc = sentence_count as f64;
        // let syl = total_syllables as f64;

        //let flesch_reading_ease = 206.835 - 1.015 * (wc / sc) - 84.6 * (syl / wc);
        //let flesch_kincaid_grade = 0.39 * (wc / sc) + 11.8 * (syl / wc) - 15.59;

        let html_escapes = text.matches("&amp;").count();

        // devlog #_ and Day _-_
        let dev_log = count_matches(&text, &["dev log", "dev-log", "day "]);
        let buzzwords = count_matches(
            &text,
            &[
                "the app",
                "-powered",
                "todo app",
                "interactive cards",
                "modern",
                "delivers",
                "across all devices",
                "style and usability",
                "real-time",
                "this isn’t a prototype",
            ],
        );

        let rule_of_threes = count_matches(
            &text,
            &[
                "more than just",
                "it’s a",
                "it's a",
                "isn’t a",
                "isn't a",
                "it’s not just",
                "it's not just",
                "isn’t just",
                "isn't just",
                "and a custom",
            ],
        ) as f64;

        TextMetrics {
            // perplexity,
            // burstiness,
            buzzword_ratio: buzzwords as f64 * 2. / words as f64,
            devlog_day_count: dev_log as f64 * 2. / words as f64,
            html_escapes: html_escapes as f64,
            emoji_rate: emoji_count as f64 / sc,
            irregular_quotations: irr_quote as f64,
            irregular_dashes: irr_dash as f64,
            //avg_sentence_length: wc / sc,
            //avg_word_length: total_word_chars as f64 / wc,
            // punctuation_rate: punct_count as f64 / wc,
            irregular_ellipsis: irr_ell as f64,
            markdown_use: markdown as f64,

            //avg_syllables_per_word: syl / wc,
            //flesch_reading_ease,
            //flesch_kincaid_grade,
            //uppercase_word_rate: uppercase_count as f64 / wc,
            //digit_rate: digit_count as f64 / wc,
            //sentence_length_stddev: stddev_sl,
            rule_of_threes,
            // embeddings,
        }
    }

    /*fn burstiness(sentence_lengths: &[usize]) -> f64 {
        if sentence_lengths.is_empty() {
            return 0.0; // or another safe default like -1.0
        }

        let mean =
            sentence_lengths.iter().copied().sum::<usize>() as f64 / sentence_lengths.len() as f64;
        let variance = sentence_lengths
            .iter()
            .map(|&l| {
                let diff = l as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / sentence_lengths.len() as f64;

        variance.sqrt()
    }

    fn build_ngram_counts<'a>(tokens: &'a [&'a str], n: usize) -> HashMap<Vec<&'a str>, usize> {
        let mut counts = HashMap::new();
        for window in tokens.windows(n) {
            *counts.entry(window.to_vec()).or_insert(0) += 1;
        }
        counts
    }

    fn compute_perplexity(tokens: &[&str], n: usize) -> f64 {
        if n < 1 || tokens.len() < n {
            return 0.0; // not enough tokens to form an n-gram
        }

        let ngram_counts = Self::build_ngram_counts(tokens, n);
        let n_minus1_counts = Self::build_ngram_counts(tokens, n - 1);

        let mut log_prob_sum = 0.0;
        let mut count = 0;

        for window in tokens.windows(n) {
            let ngram = window.to_vec();
            let prefix = ngram[..n - 1].to_vec();
            let ngram_count = *ngram_counts.get(&ngram).unwrap_or(&1);
            let prefix_count = *n_minus1_counts.get(&prefix).unwrap_or(&1);
            let prob = ngram_count as f64 / prefix_count as f64;
            log_prob_sum += prob.ln();
            count += 1;
        }

        if count == 0 {
            0.0
        } else {
            (-log_prob_sum / count as f64).exp()
        }
    }
    /// if you know anything about me, wasm, and syllables. NOT THIS SHIT AGAIN :heavysob:
    fn count_syllables(word: &str) -> usize {
        let vowels = "aeiouyAEIOUY";
        let mut prev_was_vowel = false;
        let mut syllables = 0;

        for ch in word.chars() {
            let is_vowel = vowels.contains(ch);
            if is_vowel && !prev_was_vowel {
                syllables += 1;
            }
            prev_was_vowel = is_vowel;
        }

        syllables.max(1)
    }*/
}

fn count_matches(text: &str, patterns: &[&str]) -> usize {
    patterns.iter().map(|pat| text.matches(pat).count()).sum()
}
