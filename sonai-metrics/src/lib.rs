use aho_corasick::AhoCorasick;
use linfa_clustering::KMeans;
use linfa_nn::distance::{Distance, L2Dist};
use ndarray::{Array1, Array2, ArrayView1, Axis};
use pulldown_cmark::Event;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use serde::Serialize;
use std::fmt;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Serialize)]
pub struct TextMetrics {
    // higher = more AI-like
    pub emoji_rate: f64, // Emoji * 2 / sentences

    pub buzzword_count: f64,              // Buzzwords
    pub not_just_count: f64,              // It's not just _, it's _
    pub html_escape_count: f64,           // &amp;
    pub devlog_count: f64,                // Devlog #whatever
    pub backstory_count: f64,             // I built this for the people of America.
    pub incorrect_perspective_count: f64, // We, they, you, etc
    pub mr_fancy_pants: f64,              //I am quite profficient in English grammar!

    pub irregular_ellipsis: f64,   // bad ellipses
    pub irregular_quotations: f64, // Fancy quotation marks / total quotation marks
    pub irregular_dashes: f64,     // Em-dashes / total dashes
    pub irregular_markdown: f64,   // bad markdown syntax present

    //pub i_speak_of_the_english: f64, // Bad english
    pub labels: f64,
    pub hashtags: f64,
}

impl fmt::Display for TextMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "emoji_rate\t\tnot_just_count\t\tbuzzword_count\t\thtml_escape_count
          {}\t\t\t{}\t\t\t{}\t\t\t{}
          irregular_ellipsis\tirregular_quotations\t\tirregular_dash\tirregular_markdown
          {}\t\t\t{}\t\t\t\t{}\t\t{}
          mr_fancy_pants\tincorrect_perspective_count\tdevlog_count\tlabels
          {}\t\t\t{}\t\t\t\t{}\t\t{}
          hashtags\t\tbackstory_count
          {}\t\t\t{}",
            self.emoji_rate,
            self.not_just_count,
            self.buzzword_count,
            self.html_escape_count,
            self.irregular_ellipsis,
            self.irregular_quotations,
            self.irregular_dashes,
            self.irregular_markdown,
            self.mr_fancy_pants,
            self.incorrect_perspective_count,
            self.devlog_count,
            self.labels,
            self.hashtags,
            self.backstory_count,
        )
    }
}

#[derive(Debug)]
pub struct TextMetricFactory {
    buzzword_ahocorasick: AhoCorasick,
    negative_buzzword_ahocorasick: AhoCorasick,
    not_just_ahocorasick: AhoCorasick,
    devlog_ahocorasick: AhoCorasick,
    irr_ell_ahocorasick: AhoCorasick,
    backstory_ahocorasick: AhoCorasick,
    incorrect_perspective_ahocorasick: AhoCorasick,
    mr_fancy_pants_ahocorasick: AhoCorasick,
}

impl TextMetricFactory {
    pub fn new() -> Result<Self, aho_corasick::BuildError> {
        Ok(Self {
            buzzword_ahocorasick: AhoCorasick::new([
                "the app",
                "-powered",
                "powered by",
                "-based",
                "based on",
                "-like",
                "todo app",
                "interactive cards",
                "modern",
                "delivers",
                "delivers both",
                "across all devices",
                "style and usability",
                "real-time",
                "calm, reflective space",
                "simulate",
                "self-care",
                "meaningful",
                "user interaction",
                "digital wellness",
                "user-friendly interface",
                "responsive",
                "auto-typing",
                "engagement",
                "community",
                "ambitious goal",
                "world of data",
                "programming toolkit",
                "summer of learning",
                "and a custom",
                "foundational principles",
                "began to wonder",
                "i'm announcing",
                "i’m announcing",
                "fully featured",
                "next.js 13",
                "next.js 14",
                "next.js 13/14",
                "svelte 4",
                "app router",
                "modern",
                "web dashboard",
                "step-by-step",
                "excited",
                "tailwindcss",
                "build this",
                "inner workings",
                "live code editor",
                "new project",
                "kicking off",
                "lightweight",
                "in the browser",
                "morphisim",
                "comprehensive",
                "philosophy",
                "revolutionary",
                "wisdom",
                "leetcode",
                "global accessibility",
                "developers",
                "harmony of tradition and innovation",
                "intuitive",
                "powerful features",
                "cross-platform",
                "inspiration",
                "technical architecture",
                "users can",
                "rewarding feel",
                "progress tracking",
                "understandable",
                "digital co-pilot",
                "significantly improves usability",
                "easier to navigate",
                "react for the frontend",
                "stylish",
                "mobile-",
                "ui/ux",
            ])?,
            negative_buzzword_ahocorasick: AhoCorasick::new(["modern english"])?,
            mr_fancy_pants_ahocorasick: AhoCorasick::new(["(e.g.", "(formerly"])?,
            not_just_ahocorasick: AhoCorasick::new([
                "more than just",
                "isn’t a",
                "isn't a",
                "this isn’t a prototype",
                "isn’t just a",
                "isn't just a",
                "it’s not just",
                "it's not just",
                "i'm not just",
                "i’m not just",
                "it’s just not",
                "it's just not",
                "i'm just not",
                "i’m just not",
                "isn’t just",
                "isn't just",
                "didn't just",
                "didn’t just",
                "more than a",
                "it’s more",
                "it's more",
            ])?,
            devlog_ahocorasick: AhoCorasick::new([
                "dev log",
                "dev-log",
                "day",
                "devlog #",
                "dev log #",
                "dev-log #",
                "day #",
                "today,",
                ", 2025",
                "2025.",
                "june ",
                ".05",
                ".06",
                ".07",
                "july ",
                "key improvements",
                "this week was all about",
                "the project",
                "what’s next",
                "what's next",
                "next steps",
                "why it matters",
            ])?,
            irr_ell_ahocorasick: AhoCorasick::new(["…", "..."])?,
            incorrect_perspective_ahocorasick: AhoCorasick::new([
                "we",
                "they",
                "you",
                "us",
                "our",
                "ours",
                "ourselves",
                "they",
                "them",
                "people",
                "theirs",
                "themselves",
                "oneself",
                "users",
            ])?,
            backstory_ahocorasick: AhoCorasick::new([
                "as a",
                "high school student",
                "middle school student",
                "preparing for",
                "exams",
                "was born from",
                "personal frustration",
                "makes it unique",
                "and eventually",
                "the intention",
                "it’s been a journey",
                "it's been a journey",
                "a journey",
                "it’s all about",
                "it's all about",
                "leverage that knowledge",
            ])?,
        })
    }

    pub fn calculate_iter<I, S>(&self, texts: I) -> impl Iterator<Item = TextMetrics>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        texts.into_iter().map(|s| self.calculate(s.as_ref()))
    }

    pub fn calculate(&self, text: &str) -> TextMetrics {
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
                        | Event::Start(
                            Tag::BlockQuote(_)
                                | Tag::CodeBlock(_)
                                | Tag::FootnoteDefinition(_)
                                | Tag::Emphasis
                                | Tag::Subscript
                                | Tag::Superscript
                                | Tag::Strong
                                | Tag::Strikethrough
                                | Tag::Heading { .. }
                                | Tag::Link { .. }
                                | Tag::Image { .. }
                        )
                )
            })
            .count()
            + text.matches('•').count(); // Lists are OK, this shit is not

        let text = text.to_ascii_lowercase();

        // split sentences
        let sentence_splits: Vec<&str> = text
            .split(|c| ".!?".contains(c))
            .filter(|s| !s.trim().is_empty())
            .collect();

        let sentence_count = sentence_splits.len().max(1);

        let words = text.split_whitespace().filter(|w| !w.is_empty());

        let mut hashtags = 0usize;
        for word in words {
            if word.starts_with('#') && word.len() > 1 {
                hashtags += 1;
            }
        }

        let mut labels = 0usize;

        for line in text.lines() {
            if let Some((label, _)) = line.split_once(':') {
                let label = label.trim();

                if !label.is_empty()
                    && label
                        .chars()
                        .all(|c| c.is_alphabetic() || c.is_whitespace())
                {
                    labels += 1;
                }
            }
        }

        let mut emoji_count = 0;
        let mut irr_dash = 0;
        let mut irr_quote = 0;

        for grapheme in text.graphemes(true) {
            if emojis::get(grapheme).is_some() {
                emoji_count += 1;
                continue;
            }

            let mut iter = grapheme.chars().peekable();

            while let Some(c) = iter.next() {
                match c {
                    '–' | '—' | '‒' | '―' => irr_dash += 1,
                    '“' | '”' | '‘' | '’' => irr_quote += 1,
                    '-' => {
                        if iter.peek().is_some_and(|x| !x.is_whitespace()) {
                            irr_dash += 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        // tradeoff is fine for a match list this small
        let irr_ell = self.irr_ell_ahocorasick.find_iter(&text).count();

        let sc = sentence_count as f64;

        // slow but fine, only one.
        let html_escapes = text.matches("&amp;").count();

        let dev_log = self.devlog_ahocorasick.find_iter(&text).count();

        let buzzwords = self.buzzword_ahocorasick.find_iter(&text).count()
            - self.negative_buzzword_ahocorasick.find_iter(&text).count();

        let not_just = self.not_just_ahocorasick.find_iter(&text).count();

        let backstory = self.backstory_ahocorasick.find_iter(&text).count();
        let incper = self
            .incorrect_perspective_ahocorasick
            .find_iter(&text)
            .count();

        let fancy = self.mr_fancy_pants_ahocorasick.find_iter(&text).count();

        TextMetrics {
            emoji_rate: (emoji_count * 5) as f64 / sc,
            buzzword_count: (buzzwords * 2) as f64 / sc,
            backstory_count: backstory as f64,
            incorrect_perspective_count: incper as f64,
            mr_fancy_pants: fancy as f64,

            devlog_count: dev_log as f64,
            html_escape_count: html_escapes as f64,
            not_just_count: not_just as f64,

            irregular_quotations: irr_quote as f64,
            irregular_dashes: irr_dash as f64,
            irregular_ellipsis: irr_ell as f64,
            irregular_markdown: markdown as f64,

            labels: labels as f64,
            hashtags: hashtags as f64,
        }
    }
}

pub fn features_from_metrics(data: &[&TextMetrics]) -> Array2<f64> {
    let n_features = 14;
    let n_samples = data.len();

    let mut array = Array2::<f64>::zeros((n_samples, n_features));

    for (i, sample) in data.iter().enumerate() {
        array[[i, 0]] = sample.emoji_rate * 2.;
        array[[i, 1]] = sample.buzzword_count * 10.;
        array[[i, 2]] = sample.irregular_dashes * 20.;
        array[[i, 3]] = sample.irregular_quotations * 5.;
        array[[i, 4]] = sample.labels;
        array[[i, 5]] = sample.irregular_ellipsis;
        array[[i, 6]] = sample.html_escape_count * 5.;
        array[[i, 7]] = sample.not_just_count * 5.;
        array[[i, 8]] = sample.devlog_count;
        array[[i, 9]] = sample.irregular_markdown;
        array[[i, 10]] = sample.hashtags;
        array[[i, 11]] = sample.mr_fancy_pants;
        array[[i, 12]] = sample.incorrect_perspective_count;
        array[[i, 13]] = sample.backstory_count;
    }

    array
}

fn linaccel_ultrahypr(value: f64) -> f64 {
    if value > 2. {
        value * 10.
    } else {
        value * 10. * (value - 1.).powi(2)
    }
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
