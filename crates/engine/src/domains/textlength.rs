//! Document length estimates: words ↔ pages ↔ characters, reading and
//! speaking time, and screenplay running time.
//!
//! "70,000 words", "500 page book", "1,000,000 characters",
//! "120 page screenplay", "35,000 words in Finnish".

use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// Typical rates (English prose)
const WORDS_PER_PAGE_SINGLE: f64 = 500.0;
const WORDS_PER_PAGE_DOUBLE: f64 = 250.0;
const WORDS_PER_BOOK_PAGE: f64 = 275.0;
const CHARS_PER_WORD: f64 = 6.0; // about 5 letters plus a space or punctuation
const READ_WPM: f64 = 238.0; // silent reading of non-fiction (Brysbaert 2019)
const SPEAK_WPM: f64 = 150.0;
const TYPE_WPM: f64 = 40.0;

fn duration(min: f64) -> String {
    if min < 1.0 {
        return format!("{} seconds", fmt_float((min * 60.0).round(), 4));
    }
    if min < 60.0 {
        return format!("{} minutes", fmt_float(min.round(), 4));
    }
    let h = (min / 60.0).floor();
    let m = (min - h * 60.0).round();
    if h >= 48.0 {
        return format!("{} hours ({} days of continuous reading)", fmt_float(h, 6), fmt_float((min / 1440.0 * 10.0).round() / 10.0, 4));
    }
    format!("{} h {} min", fmt_float(h, 6), fmt_float(m, 3))
}

fn n(v: f64) -> String {
    super::geo::fmt_int(v.round().max(0.0) as u64)
}

fn category(words: f64) -> &'static str {
    if words < 1000.0 {
        "flash fiction (under 1,000 words)"
    } else if words < 7500.0 {
        "short story (under 7,500 words)"
    } else if words < 17_500.0 {
        "novelette (7,500–17,500 words)"
    } else if words < 40_000.0 {
        "novella (17,500–40,000 words)"
    } else if words < 110_000.0 {
        "novel (40,000 words or more; most published novels are 70,000–110,000 words)"
    } else {
        "long novel (over 110,000 words)"
    }
}

const RESOLUTIONS: &[(u32, u32, &str)] = &[
    (320, 240, "QVGA"), (640, 480, "VGA"), (800, 600, "SVGA"), (1024, 768, "XGA"), (1280, 720, "HD (720p)"), (1280, 800, "WXGA"),
    (1280, 1024, "SXGA"), (1366, 768, "WXGA (HD)"), (1440, 900, "WXGA+"), (1600, 900, "HD+"), (1600, 1200, "UXGA"), (1680, 1050, "WSXGA+"),
    (1920, 1080, "Full HD (1080p)"), (1920, 1200, "WUXGA"), (2048, 1080, "DCI 2K"), (2560, 1440, "QHD / WQHD (1440p)"), (2560, 1600, "WQXGA"),
    (3440, 1440, "UWQHD"), (3840, 2160, "4K UHD (2160p)"), (4096, 2160, "DCI 4K"), (5120, 2880, "5K"), (7680, 4320, "8K UHD (4320p)"),
];

/// "1024 x 768 pixels", "1920x1080 resolution"
fn resolution(l: &str) -> Option<Answer> {
    let m = re!(r"^(\d{2,5})\s*(?:x|×|by)\s*(\d{2,5})\s*(?:pixels?|px|resolution|screen|display|monitor)$|^(?:resolution|screen resolution|display resolution)\s+(\d{2,5})\s*(?:x|×|by)\s*(\d{2,5})$").captures(l)?;
    let w: u32 = m.get(1).or(m.get(3))?.as_str().parse().ok()?;
    let h: u32 = m.get(2).or(m.get(4))?.as_str().parse().ok()?;
    if w == 0 || h == 0 {
        return None;
    }
    let g = {
        let (mut a, mut b) = (w, h);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    };
    let px = w as u64 * h as u64;
    let name = RESOLUTIONS.iter().find(|r| r.0 == w && r.1 == h).map(|r| r.2);
    let mut rows = vec![
        vec![text("total pixels"), text(format!("{} ({} megapixels)", n(px as f64), fmt_float((px as f64 / 1e4).round() / 100.0, 4)))],
        vec![text("aspect ratio"), text(format!("{}:{} ({})", w / g, h / g, fmt_float((w as f64 / h as f64 * 1000.0).round() / 1000.0, 4)))],
        vec![text("uncompressed 24-bit frame"), text(format!("{} MB", fmt_float((px as f64 * 3.0 / 1e6 * 100.0).round() / 100.0, 4)))],
        vec![text("print size at 300 ppi"), text(format!("{} × {} in", fmt_float((w as f64 / 300.0 * 100.0).round() / 100.0, 4), fmt_float((h as f64 / 300.0 * 100.0).round() / 100.0, 4)))],
    ];
    if let Some(nm) = name {
        rows.insert(0, vec![text("standard name"), text(nm)]);
    }
    Some(Answer::new("display resolution", vec![
        pod("Input", "Input interpretation", text(format!("{w} × {h} pixels"))),
        primary(pod("Result", "Total pixels", text(format!("{} pixels{}", n(px as f64), name.map(|x| format!(" ({x})")).unwrap_or_default())))),
        pod("Table", "Properties", table(None, rows)),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    if let Some(a) = resolution(l) {
        return Some(a);
    }
    let l = re!(r"(\d),(\d{3})").replace_all(l, "$1$2").to_string();
    let l = re!(r"(\d),(\d{3})").replace_all(&l, "$1$2").to_string();
    let m = re!(r"^(?:how long (?:is|to read) )?(\d+(?:\.\d+)?)[- ]?(words?|pages?|characters?|chars)(?:[- ](book|novel|essay|document|paper|manuscript|screenplay|script|text))?(?: (?:in|written in) ([a-z]+))?$").captures(&l)?;
    let v: f64 = m[1].parse().ok()?;
    if v <= 0.0 || v > 1e9 {
        return None;
    }
    let unit = &m[2];
    let kind = m.get(3).map(|x| x.as_str());
    let lang = m.get(4).map(|x| x.as_str());
    if let Some(lg) = lang {
        // only languages, not "500 pages in mm" or similar
        if !matches!(lg, "english" | "spanish" | "french" | "german" | "italian" | "portuguese" | "dutch" | "finnish" | "swedish" | "norwegian" | "danish" | "polish" | "russian" | "turkish" | "greek" | "hungarian" | "czech" | "latin") {
            return None;
        }
    }
    let screenplay = matches!(kind, Some("screenplay" | "script"));
    let book = matches!(kind, Some("book" | "novel"));
    let words = if unit.starts_with("word") {
        v
    } else if unit.starts_with("page") {
        if screenplay {
            v * 180.0
        } else if book {
            v * WORDS_PER_BOOK_PAGE
        } else {
            v * WORDS_PER_PAGE_SINGLE
        }
    } else {
        v / CHARS_PER_WORD
    };
    if words < 50.0 {
        return None;
    }
    let mut rows = vec![
        vec![text("words"), text(n(words))],
        vec![text("characters (with spaces)"), text(n(words * CHARS_PER_WORD))],
        vec![text("pages, single-spaced (12 pt, 1-inch margins)"), text(n(words / WORDS_PER_PAGE_SINGLE))],
        vec![text("pages, double-spaced"), text(n(words / WORDS_PER_PAGE_DOUBLE))],
        vec![text("printed book pages"), text(n(words / WORDS_PER_BOOK_PAGE))],
        vec![text("reading time (238 words/min)"), text(duration(words / READ_WPM))],
        vec![text("speaking time (150 words/min)"), text(duration(words / SPEAK_WPM))],
        vec![text("typing time (40 words/min)"), text(duration(words / TYPE_WPM))],
    ];
    let (title, result) = if screenplay {
        rows.insert(0, vec![text("screen time"), text(format!("about {} (one page ≈ one minute)", duration(if unit.starts_with("page") { v } else { words / 180.0 })))]);
        ("Estimated screen time", format!("about {}", duration(if unit.starts_with("page") { v } else { words / 180.0 })))
    } else if unit.starts_with("word") {
        ("Length", format!("{} single-spaced pages | {} double-spaced pages | reading time {}", n(words / WORDS_PER_PAGE_SINGLE), n(words / WORDS_PER_PAGE_DOUBLE), duration(words / READ_WPM)))
    } else if unit.starts_with("page") {
        ("Length", format!("about {} words | reading time {}", n(words), duration(words / READ_WPM)))
    } else {
        ("Length", format!("about {} words | {} single-spaced pages", n(words), n(words / WORDS_PER_PAGE_SINGLE)))
    };
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{} {}{}{}", fmt_float(v, 9), unit, kind.map(|k| format!(" ({k})")).unwrap_or_default(), lang.map(|g| format!(" in {}", g[..1].to_uppercase() + &g[1..])).unwrap_or_default()))),
        primary(pod("Result", title, text(result))),
        pod("Table", "Estimates", table(None, rows)),
    ];
    if !screenplay {
        pods.push(pod("Category", "Fiction category (by word count)", text(category(words))));
    }
    let mut note = "estimates for English prose: about 500 words per single-spaced page, 250 double-spaced, 275 per printed book page and 6 characters per word including spaces".to_string();
    if let Some(lg) = lang.filter(|g| *g != "english") {
        note.push_str(&format!("; {lg} words are on average longer or shorter than English ones, so character and page counts differ"));
    }
    pods.push(pod("Note", "Assumptions", text(note)));
    Some(Answer::new("document length", pods))
}
