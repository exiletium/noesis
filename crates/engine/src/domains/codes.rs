//! ISBN check digits, ISBN-10 ↔ ISBN-13 conversion and registration groups;
//! ISO 216/269 and North American paper sizes, and the sizes nearest a given
//! sheet or photo.
//!
//! "ISBN 1-5795-5008-8", "isbn 9780306406157", "paper size A4",
//! "nearest paper sizes for a 5\"x7\" photo".

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

/// Registration groups by prefix (after 978, or with 979 written out).
const GROUPS: &[(&str, &str)] = &[
    ("9790", "music (ISMN)"),
    ("97910", "France"),
    ("97911", "South Korea"),
    ("97912", "Italy"),
    ("9798", "United States"),
    ("0", "English-speaking countries"),
    ("1", "English-speaking countries"),
    ("2", "French-speaking countries"),
    ("3", "German-speaking countries"),
    ("4", "Japan"),
    ("5", "former USSR (Russian Federation and others)"),
    ("7", "China"),
    ("65", "Brazil"),
    ("80", "Czech Republic and Slovakia"),
    ("81", "India"),
    ("82", "Norway"),
    ("83", "Poland"),
    ("84", "Spain"),
    ("85", "Brazil"),
    ("86", "former Yugoslavia"),
    ("87", "Denmark"),
    ("88", "Italy"),
    ("89", "South Korea"),
    ("90", "Netherlands and Flemish Belgium"),
    ("91", "Sweden"),
    ("92", "international organisations"),
    ("93", "India"),
    ("94", "Netherlands"),
    ("600", "Iran"),
    ("601", "Kazakhstan"),
    ("602", "Indonesia"),
    ("603", "Saudi Arabia"),
    ("604", "Vietnam"),
    ("605", "Turkey"),
    ("950", "Argentina"),
    ("951", "Finland"),
    ("952", "Finland"),
    ("953", "Croatia"),
    ("954", "Bulgaria"),
    ("955", "Sri Lanka"),
    ("956", "Chile"),
    ("957", "Taiwan"),
    ("958", "Colombia"),
    ("959", "Cuba"),
    ("960", "Greece"),
    ("961", "Slovenia"),
    ("962", "Hong Kong"),
    ("963", "Hungary"),
    ("964", "Iran"),
    ("965", "Israel"),
    ("966", "Ukraine"),
    ("968", "Mexico"),
    ("969", "Pakistan"),
    ("970", "Mexico"),
    ("971", "Philippines"),
    ("972", "Portugal"),
    ("973", "Romania"),
    ("974", "Thailand"),
    ("975", "Turkey"),
    ("977", "Egypt"),
    ("978", "Nigeria"),
    ("979", "Indonesia"),
    ("980", "Venezuela"),
    ("981", "Singapore"),
    ("983", "Malaysia"),
    ("984", "Bangladesh"),
    ("985", "Belarus"),
    ("986", "Taiwan"),
    ("987", "Argentina"),
    ("988", "Hong Kong"),
    ("989", "Portugal"),
];

fn check10(d: &[u32]) -> char {
    let s: u32 = d.iter().take(9).enumerate().map(|(i, v)| (10 - i as u32) * v).sum();
    match (11 - s % 11) % 11 {
        10 => 'X',
        c => char::from_digit(c, 10).unwrap(),
    }
}

fn check13(d: &[u32]) -> char {
    let s: u32 = d.iter().take(12).enumerate().map(|(i, v)| if i % 2 == 0 { *v } else { 3 * v }).sum();
    char::from_digit((10 - s % 10) % 10, 10).unwrap()
}

fn group(isbn13: &str) -> Option<&'static str> {
    let key = if let Some(r) = isbn13.strip_prefix("978") { r.to_string() } else { isbn13.to_string() };
    GROUPS.iter().filter(|(p, _)| key.starts_with(p)).max_by_key(|(p, _)| p.len()).map(|(_, g)| *g)
}

fn isbn(q: &Query) -> Option<Answer> {
    let m = re!(r"(?i)^(?:is\s+)?(?:the\s+)?isbn(?:-1[03])?(?:\s+(?:number|code))?\s*:?\s*([0-9][0-9\- ]{8,20}[0-9xX])(?:\s+(?:valid|a valid isbn))?$").captures(q.raw.trim().trim_end_matches(['?', '.']))?;
    let given = m[1].trim().to_string();
    let compact: String = given.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_uppercase();
    let digits: Vec<u32> = compact.chars().map(|c| if c == 'X' { 10 } else { c.to_digit(10).unwrap_or(99) }).collect();
    if digits.iter().any(|d| *d == 99) || compact[..compact.len() - 1].contains('X') {
        return None;
    }
    let mut rows = Vec::new();
    let (valid, correct, isbn10, isbn13) = match digits.len() {
        10 => {
            let c = check10(&digits);
            let ok = compact.ends_with(c);
            let body = &compact[..9];
            let d13: Vec<u32> = format!("978{body}").chars().map(|c| c.to_digit(10).unwrap()).collect();
            let c13 = check13(&d13);
            // keep the given hyphenation for the ISBN-13
            let hyph = if given.contains('-') { format!("978-{}{c13}", &given[..given.len() - 1]) } else { format!("978{body}{c13}") };
            (ok, c, Some(format!("{body}{c}")), hyph)
        }
        13 => {
            let c = check13(&digits);
            let ok = compact.ends_with(c);
            let ten = compact.strip_prefix("978").map(|r| {
                let d: Vec<u32> = r.chars().take(9).map(|c| c.to_digit(10).unwrap()).collect();
                format!("{}{}", &r[..9], check10(&d))
            });
            (ok, c, ten, format!("{}{c}", &given[..given.len() - 1]))
        }
        _ => return None,
    };
    let kind = if digits.len() == 10 { "ISBN-10" } else { "ISBN-13" };
    let result = if valid { format!("valid {kind}") } else { format!("invalid {kind}: the check digit should be {correct}") };
    rows.push(vec![text("check digit"), text(format!("{} ({})", compact.chars().last()?, if valid { "correct" } else { "incorrect" }))]);
    if valid {
        if let Some(t) = &isbn10 {
            rows.push(vec![text("ISBN-10"), text(t.clone())]);
        }
        rows.push(vec![text("ISBN-13"), text(isbn13.clone())]);
        let c13: String = isbn13.chars().filter(|c| c.is_ascii_digit()).collect();
        if let Some(g) = group(&c13) {
            rows.push(vec![text("registration group"), text(g)]);
        }
    }
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("ISBN {given}"))),
        primary(pod("Result", "Result", text(result))),
        pod("Details", "Details", table(None, rows)),
        pod("Rule", "Check digit rule", text(if digits.len() == 10 { "Σ (11 − i)·dᵢ ≡ 0 (mod 11), with X = 10" } else { "Σ wᵢ·dᵢ ≡ 0 (mod 10), weights 1, 3, 1, 3, …" })),
    ];
    Some(Answer::new(format!("ISBN {given}"), pods))
}

/// (name, width mm, height mm)
fn sizes() -> Vec<(String, f64, f64)> {
    let mut v = Vec::new();
    // ISO 216 A and B, ISO 269 C: halve the long side, rounding down to whole millimetres
    for (series, w0, h0) in [("A", 841.0, 1189.0), ("B", 1000.0, 1414.0), ("C", 917.0, 1297.0)] {
        let (mut w, mut h): (f64, f64) = (w0, h0);
        for n in 0..=10 {
            v.push((format!("{series}{n}"), w, h));
            let nw = (h / 2.0).floor();
            h = w;
            w = nw;
        }
    }
    let inch = 25.4;
    for (n, w, h) in [("US Letter", 8.5, 11.0), ("US Legal", 8.5, 14.0), ("Tabloid (Ledger)", 11.0, 17.0), ("Executive", 7.25, 10.5), ("Half Letter (Statement)", 5.5, 8.5), ("Junior Legal", 5.0, 8.0), ("Index card 3×5", 3.0, 5.0), ("Index card 4×6", 4.0, 6.0), ("Index card 5×8", 5.0, 8.0), ("Photo 4×6", 4.0, 6.0), ("Photo 5×7", 5.0, 7.0), ("Photo 8×10", 8.0, 10.0), ("Photo 11×14", 11.0, 14.0), ("ANSI C", 17.0, 22.0), ("ANSI D", 22.0, 34.0), ("ANSI E", 34.0, 44.0)] {
        v.push((n.to_string(), w * inch, h * inch));
    }
    v
}

fn dims(w: f64, h: f64) -> String {
    format!("{} mm × {} mm ({} in × {} in)", fmt_float(w, 4), fmt_float(h, 4), fmt_float(w / 25.4, 3), fmt_float(h / 25.4, 3))
}

fn paper(q: &Query) -> Option<Answer> {
    let l = q.raw.trim().trim_end_matches(['?', '.']).to_lowercase().replace(['“', '”', '″'], "\"");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    // a named size
    if let Some(m) = re!(r"^(?:what is the |what are the )?(?:(?:paper )?(?:size|dimensions?) (?:of )?(?:an? )?([abc]\d{1,2}|us letter|letter|legal|tabloid|ledger|executive)(?: paper| sheet)?|(?:an? )?([abc]\d{1,2}|us letter|letter|legal|tabloid|ledger|executive) (?:paper|sheet)(?: size| dimensions?)?)$").captures(&l) {
        let name = m.get(1).or(m.get(2))?.as_str();
        let want = match name {
            "letter" | "us letter" => "US Letter".to_string(),
            "legal" => "US Legal".into(),
            "tabloid" | "ledger" => "Tabloid (Ledger)".into(),
            "executive" => "Executive".into(),
            n => n.to_uppercase(),
        };
        let (n, w, h) = sizes().into_iter().find(|(n, ..)| *n == want)?;
        let mut rows = vec![vec![text("width × height"), text(dims(w, h))], vec![text("area"), text(format!("{} m²", fmt_float(w * h / 1e6, 4)))], vec![text("aspect ratio"), text(fmt_float(h / w, 5))]];
        if n.len() <= 3 && n.starts_with('A') {
            rows.push(vec![text("note"), text("A sizes have aspect ratio √2 and A0 has area 1 m²; each size is half the one before")]);
        }
        return Some(Answer::new(format!("{n} paper"), vec![
            pod("Input", "Input interpretation", text(format!("{n} paper size"))),
            primary(pod("Result", "Dimensions", text(dims(w, h)))),
            pod("Details", "Properties", table(None, rows)),
        ]));
    }
    // nearest sizes to a sheet or photo
    let m = re!(r#"^(?:the )?(?:nearest|closest) (?:standard )?paper sizes? (?:to|for) (?:an? )?(\d+(?:\.\d+)?)\s*(?:"|in|inches?|mm|cm)?\s*(?:x|×|by)\s*(\d+(?:\.\d+)?)\s*("|in|inches?|mm|cm)?(?: (?:photo|print|picture|sheet|page|paper))?$"#).captures(&l)?;
    let (a, b): (f64, f64) = (m[1].parse().ok()?, m[2].parse().ok()?);
    let scale = match m.get(3).map(|x| x.as_str()) {
        Some("mm") => 1.0,
        Some("cm") => 10.0,
        _ => 25.4,
    };
    let (w, h) = ((a.min(b)) * scale, (a.max(b)) * scale);
    let mut all: Vec<(String, f64, f64, f64)> = sizes().into_iter().filter(|(n, ..)| !n.starts_with("Photo")).map(|(n, sw, sh)| {
        let d = ((sw - w).powi(2) + (sh - h).powi(2)).sqrt();
        (n, sw, sh, d)
    }).collect();
    all.sort_by(|x, y| x.3.total_cmp(&y.3));
    let rows: Vec<Vec<Content>> = all.iter().take(6).map(|(n, sw, sh, _)| {
        let fits = *sw >= w - 0.5 && *sh >= h - 0.5;
        vec![text(n.clone()), text(dims(*sw, *sh)), text(if fits { "yes" } else { "no" })]
    }).collect();
    let fit = all.iter().filter(|(_, sw, sh, _)| *sw >= w - 0.5 && *sh >= h - 0.5).min_by(|x, y| (x.1 * x.2).total_cmp(&(y.1 * y.2)))?;
    let input = format!("{} × {} {}", fmt_float(a.min(b), 4), fmt_float(a.max(b), 4), match scale as i32 { 1 => "mm", 10 => "cm", _ => "in" });
    Some(Answer::new(format!("paper sizes near {input}"), vec![
        pod("Input", "Input interpretation", text(format!("nearest paper sizes | {input}"))),
        primary(pod("Result", "Nearest sizes", text(all.iter().take(3).map(|x| x.0.clone()).collect::<Vec<_>>().join(", ")))),
        pod("Table", "Closest standard sizes", table(Some(vec!["size", "dimensions", "holds it"]), rows)),
        pod("Fit", "Smallest standard size that holds it", text(format!("{} ({})", fit.0, dims(fit.1, fit.2)))),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    isbn(q).or_else(|| paper(q))
}
