use once_cell::sync::Lazy;
use pulldown_cmark::Event;
use pulldown_cmark::Parser;
use regex::Regex;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

static STOPWORDS: Lazy<HashMap<&'static str, ()>> = Lazy::new(|| {
    [
        "the", "and", "is", "in", "it", "to", "of", "a", "that", "i", "was", "he", "for", "you",
        "with", "on", "as", "have", "but", "be",
    ]
    .iter()
    .copied()
    .map(|w| (w, ()))
    .collect()
});

#[derive(Debug)]
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

    pub type_token_ratio: f64,       // unique words / total words
    pub hapax_legomena_rate: f64,    // words occurring once / total words
    pub avg_syllables_per_word: f64, // total syllables / total words
    pub flesch_reading_ease: f64,    // 206.835 – 1.015*(words/sentences) – 84.6*(syllables/words)
    pub flesch_kincaid_grade: f64,   // 0.39*(words/sentences) + 11.8*(syllables/words) – 15.59
    pub stopword_rate: f64,          // stopwords / total words
    pub uppercase_word_rate: f64,    // ALL-CAPS words / total words
    pub digit_rate: f64,             // words containing digits / total words
    pub url_email_rate: f64,         // words matching URL/email regex / total words
    pub passive_voice_rate: f64,     // be‑verbs + past participle per sentence
    pub sentence_length_stddev: f64, // stdev of words per sentence
}

impl TextMetrics {
    pub fn calculate(text: &str) -> Self {
        // existing markdown vs non-markdown
        let (markdown, not_md) = Parser::new(text).fold((0usize, 0usize), |(md, non_md), event| {
            if matches!(
                event,
                Event::Code(_)
                    | Event::InlineMath(_)
                    | Event::DisplayMath(_)
                    | Event::Html(_)
                    | Event::FootnoteReference(_)
                    | Event::SoftBreak
                    | Event::TaskListMarker(_)
                    | Event::Rule
                    | Event::HardBreak
                    | Event::InlineHtml(_)
            ) {
                (md + 1, non_md)
            } else {
                (md, non_md + 1)
            }
        });

        // split sentences
        let sentence_splits: Vec<&str> = text
            .split(|c| ".!?".contains(c))
            .filter(|s| !s.trim().is_empty())
            .collect();
        let sentence_count = sentence_splits.len().max(1);
        let sentence_word_counts: Vec<usize> = sentence_splits
            .iter()
            .map(|s| s.split_whitespace().count())
            .collect();

        // word-level stats
        let words: Vec<&str> = text.split_whitespace().filter(|w| !w.is_empty()).collect();
        let word_count = words.len().max(1);
        let total_word_chars: usize = words.iter().map(|w| w.chars().count()).sum();

        // vocabulary distribution
        let mut freqs = HashMap::new();
        for w in &words {
            let w_lower = w
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            *freqs.entry(w_lower).or_insert(0usize) += 1;
        }
        let unique_words = freqs.len();
        let hapaxes = freqs.values().filter(|&&c| c == 1).count();

        // syllable estimate: vowel groups per word
        let vowel_re = Regex::new(r"[aeiouyAEIOUY]+").unwrap();
        let total_syllables: usize = words
            .iter()
            .map(|w| vowel_re.find_iter(w).count().max(1))
            .sum();

        // stopwords, uppercase, digits, URLs/emails
        let url_email_re = Regex::new(r"^(https?://\S+|\S+@\S+\.\S+)$").unwrap();
        let mut stopword_count = 0;
        let mut uppercase_count = 0;
        let mut digit_count = 0;
        let mut url_email_count = 0;
        for w in &words {
            let w_clean = w.trim_matches(|c: char| !c.is_alphanumeric());
            if STOPWORDS.contains_key(&w_clean.to_lowercase().as_str()) {
                stopword_count += 1;
            }
            if w_clean.chars().all(|c| c.is_uppercase()) && w_clean.len() > 1 {
                uppercase_count += 1;
            }
            if w_clean.chars().any(|c| c.is_ascii_digit()) {
                digit_count += 1;
            }
            if url_email_re.is_match(w) {
                url_email_count += 1;
            }
        }

        // passive voice (rough): look for "was|were|is|are|be|been|being" + word ending in "ed"
        let passive_re = Regex::new(r"\b(was|were|is|are|be|been|being)\s+\w+ed\b").unwrap();
        let passive_matches = passive_re.find_iter(text).count();

        // sentence length standard deviation
        let mean_sl = sentence_word_counts.iter().sum::<usize>() as f64 / sentence_count as f64;
        let var_sl = sentence_word_counts
            .iter()
            .map(|&c| {
                let diff = c as f64 - mean_sl;
                diff * diff
            })
            .sum::<f64>()
            / sentence_count as f64;
        let stddev_sl = var_sl.sqrt();

        // existing dash/quote/emoji/punct/ellipsis counts
        let mut emoji_count = 0;
        let mut reg_dash = 0;
        let mut irr_dash = 0;
        let mut reg_quote = 0;
        let mut irr_quote = 0;
        let mut punct_count = 0;
        for grapheme in text.graphemes(true) {
            if emojis::get(grapheme).is_some() {
                emoji_count += 1;
                continue;
            }
            for c in grapheme.chars() {
                match c {
                    '-' => reg_dash += 1,
                    '–' | '—' | '‒' | '―' => irr_dash += 1,
                    '"' | '\'' => reg_quote += 1,
                    '“' | '”' | '‘' | '’' => irr_quote += 1,
                    '.' | '!' | '?' | ',' | ':' | ';' => punct_count += 1,
                    _ => {}
                }
            }
        }
        let unicode_ell = text.matches('…').count();
        let ascii_ell = text.matches("...").count();
        let ell_count = unicode_ell + ascii_ell;

        // finalize calculations
        let wc = word_count as f64;
        let sc = sentence_count as f64;
        let syl = total_syllables as f64;

        let flesch_reading_ease = 206.835 - 1.015 * (wc / sc) - 84.6 * (syl / wc);
        let flesch_kincaid_grade = 0.39 * (wc / sc) + 11.8 * (syl / wc) - 15.59;

        TextMetrics {
            emoji_rate: emoji_count as f64 / sc,
            irregular_quotation_rate: irr_quote as f64 / (reg_quote + irr_quote).max(1) as f64,
            irregular_dash_rate: irr_dash as f64 / (reg_dash + irr_dash).max(1) as f64,
            avg_sentence_length: wc / sc,
            avg_word_length: total_word_chars as f64 / wc,
            punctuation_rate: punct_count as f64 / wc,
            ellipsis_rate: ell_count as f64 / sc,
            markdown_use: markdown as f64 / not_md.max(1) as f64,

            type_token_ratio: unique_words as f64 / wc,
            hapax_legomena_rate: hapaxes as f64 / wc,
            avg_syllables_per_word: syl / wc,
            flesch_reading_ease,
            flesch_kincaid_grade,
            stopword_rate: stopword_count as f64 / wc,
            uppercase_word_rate: uppercase_count as f64 / wc,
            digit_rate: digit_count as f64 / wc,
            url_email_rate: url_email_count as f64 / wc,
            passive_voice_rate: passive_matches as f64 / sc,
            sentence_length_stddev: stddev_sl,
        }
    }
}
