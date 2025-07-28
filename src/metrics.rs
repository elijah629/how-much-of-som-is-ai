use std::fmt;

use pulldown_cmark::Event;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use serde::Serialize;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Serialize)]
pub struct TextMetrics {
    // higher = more AI-like
    pub emoji_rate: f64, // Emoji * 2.0 / sentences

    pub not_just_count: f64,    // It's not just _, it's _
    pub buzzword_count: f64,    // Buzzwords * 2 / words
    pub html_escape_count: f64, // &amp;
    pub devlog_count: f64,      // Devlog #whatever

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
            "emoji_rate={} not_just_count={} buzzword_count={} html_escape_count={} devlog_count={} irregular_ellipsis={} irregular_quotations={} irregular_dashes={} irregular_markdown={} labels={} hashtags={}",
            self.emoji_rate,
            self.not_just_count,
            self.buzzword_count,
            self.html_escape_count,
            self.devlog_count,
            self.irregular_ellipsis,
            self.irregular_quotations,
            self.irregular_dashes,
            self.irregular_markdown,
            self.labels,
            self.hashtags
        )
    }
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
            for c in grapheme.chars() {
                match c {
                    '–' | '—' | '‒' | '―' => irr_dash += 1,
                    '“' | '”' | '‘' | '’' => irr_quote += 1,
                    _ => {}
                }
            }
        }

        let irr_ell = text.matches('…').count() + text.matches("...").count();

        let sc = sentence_count as f64;
        let html_escapes = text.matches("&amp;").count();

        let dev_log = count_matches(
            &text,
            &[
                "dev log",
                "dev-log",
                "day",
                "devlog #",
                "dev log #",
                "dev-log #",
                "day #",
            ],
        );
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
                "it’s been a journey",
                "it's been a journey",
                "a journey",
                "ambitious goal",
                "world of data",
                "programming toolkit",
                "summer of learning",
                "and a custom",
                "foundational principles",
                "began to wonder",
                "i'm announcing",
                "i’m announcing",
                "it’s all about",
                "it's all about",
                "leverage that knowledge",
                "fully featured",
                "next.js 13",
                "next.js 14",
                "svelte 4",
                "app router",
                "modern",
                "web dashboard",
                "the intention",
                "(formerly",
                "step-by-step",
                "excited",
                "tailwindcss",
                "build this",
                "inner workings",
                "live code editor",
                "new project",
                "kicking off",
                "lightweight",
                "-like",
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
                "this week was all about",
            ],
        ) - count_matches(&text, &["modern english"]);

        let not_just = count_matches(
            &text,
            &[
                "more than just",
                "isn’t a",
                "isn't a",
                "isn’t just a",
                "isn't just a",
                "it’s not just",
                "it's not just",
                "i'm not just",
                "i’m not just",
                "isn’t just",
                "isn't just",
                "didn't just",
                "didn’t just",
            ],
        );

        TextMetrics {
            emoji_rate: (emoji_count * 2) as f64 / sc,

            buzzword_count: buzzwords as f64,
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

fn count_matches(text: &str, patterns: &[&str]) -> usize {
    patterns.iter().map(|pat| text.matches(pat).count()).sum()
}
