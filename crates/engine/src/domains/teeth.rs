//! Human teeth: primary and permanent sets, eruption and shedding ages.
//!
//! "primary incisors", "wisdom teeth", "when do molars come in",
//! "baby teeth", "permanent canines".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// kind | tooth | upper eruption | lower eruption | shed (primary only)
const PRIMARY: &[(&str, &str, &str, &str, &str)] = &[
    ("incisor", "central incisor", "8–12 months", "6–10 months", "6–7 years"),
    ("incisor", "lateral incisor", "9–13 months", "10–16 months", "7–8 years"),
    ("canine", "canine (cuspid)", "16–22 months", "17–23 months", "9–12 years"),
    ("molar", "first molar", "13–19 months", "14–18 months", "9–11 years"),
    ("molar", "second molar", "25–33 months", "23–31 months", "10–12 years"),
];

const PERMANENT: &[(&str, &str, &str, &str)] = &[
    ("incisor", "central incisor", "7–8 years", "6–7 years"),
    ("incisor", "lateral incisor", "8–9 years", "7–8 years"),
    ("canine", "canine (cuspid)", "11–12 years", "9–10 years"),
    ("premolar", "first premolar (bicuspid)", "10–11 years", "10–12 years"),
    ("premolar", "second premolar (bicuspid)", "10–12 years", "11–12 years"),
    ("molar", "first molar", "6–7 years", "6–7 years"),
    ("molar", "second molar", "12–13 years", "11–13 years"),
    ("molar", "third molar (wisdom tooth)", "17–21 years", "17–21 years"),
];

fn kind(s: &str) -> Option<&'static str> {
    Some(match s.trim_end_matches('s').trim_end_matches("e") {
        "incisor" | "front tooth" | "front teeth" => "incisor",
        "canine" | "cuspid" | "eye tooth" | "eye teeth" => "canine",
        "premolar" | "bicuspid" => "premolar",
        "molar" => "molar",
        "tooth" | "teeth" | "" => "",
        _ => return None,
    })
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = l.trim_start_matches("what are ").trim_start_matches("the ");
    let src = pod("Source", "Notes", text("typical ages from the American Dental Association; individual timing varies by several months or more"));
    if re!(r"^(?:wisdom teeth|wisdom tooth|third molars?)(?: (?:age|eruption|eruption age))?$|^when do (?:you get )?wisdom teeth (?:come in|erupt|grow)$").is_match(l) {
        return Some(Answer::new("wisdom teeth", vec![
            pod("Input", "Input interpretation", text("wisdom teeth (third molars)")),
            primary(pod("Result", "Typical eruption", text("17–21 years (often later, or never)"))),
            pod("Table", "Properties", table(None, vec![
                vec![text("number"), text("up to 4 (one at the back of each quadrant)")],
                vec![text("tooth numbers (Universal)"), text("1, 16, 17, 32")],
                vec![text("FDI notation"), text("18, 28, 38, 48")],
                vec![text("set"), text("permanent dentition only; there are no primary third molars")],
                vec![text("notes"), text("often impacted for lack of space and commonly removed; about a quarter of people lack at least one")],
            ])),
            src,
        ]));
    }
    let m = re!(r"^(?:(primary|baby|deciduous|milk|permanent|adult|secondary)\s+)?(incisors?|canines?|cuspids?|premolars?|bicuspids?|molars?|teeth|tooth)(?:\s+(?:eruption(?: chart| ages?)?|chart))?$|^when do (?:(baby|permanent|adult) )?(incisors|canines|premolars|molars|teeth) (?:come in|erupt)$").captures(l)?;
    let set = m.get(1).or(m.get(3)).map(|x| x.as_str());
    let k = kind(m.get(2).or(m.get(4))?.as_str())?;
    // a bare "teeth" or "molar" without a set is too vague unless it is plural tooth types
    if set.is_none() && k.is_empty() {
        return None;
    }
    let primary_set = matches!(set, Some("primary" | "baby" | "deciduous" | "milk"));
    if primary_set {
        let rows: Vec<Vec<Content>> = PRIMARY.iter().filter(|t| k.is_empty() || t.0 == k).map(|t| vec![text(t.1), text(t.2), text(t.3), text(t.4)]).collect();
        if rows.is_empty() {
            return Some(Answer::new("primary teeth", vec![pod("Input", "Input interpretation", text(format!("primary {k}s"))), primary(pod("Result", "Result", text(format!("there are no primary {k}s: the 20 primary teeth are incisors, canines and molars"))))]));
        }
        let count = match k { "incisor" => 8, "canine" => 4, "molar" => 8, _ => 20 };
        return Some(Answer::new("primary teeth", vec![
            pod("Input", "Input interpretation", text(format!("primary (baby) {}", if k.is_empty() { "teeth".to_string() } else { format!("{k}s") }))),
            primary(pod("Result", "Number", text(format!("{count} (of 20 primary teeth)")))),
            pod("Table", "Eruption and shedding", table(Some(vec!["tooth", "upper erupts", "lower erupts", "shed"]), rows)),
            src,
        ]));
    }
    let rows: Vec<Vec<Content>> = PERMANENT.iter().filter(|t| k.is_empty() || t.0 == k).map(|t| vec![text(t.1), text(t.2), text(t.3)]).collect();
    let count = match k { "incisor" => 8, "canine" => 4, "premolar" => 8, "molar" => 12, _ => 32 };
    Some(Answer::new("permanent teeth", vec![
        pod("Input", "Input interpretation", text(format!("permanent {}", if k.is_empty() { "teeth".to_string() } else { format!("{k}s") }))),
        primary(pod("Result", "Number", text(format!("{count} (of 32 permanent teeth)")))),
        pod("Table", "Eruption", table(Some(vec!["tooth", "upper erupts", "lower erupts"]), rows)),
        src,
    ]))
}
