//! Miscibility of common laboratory solvents.
//!
//! "is ethanol soluble in benzene?", "can I dissolve acetone in benzene?",
//! "What solvents are miscible in acetone?", "hexane water miscibility".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names (';'-separated, first is the display name)
const SOLVENTS: &[&str] = &[
    "water;h2o",
    "methanol;methyl alcohol;meoh",
    "ethanol;ethyl alcohol;etoh;alcohol",
    "isopropanol;isopropyl alcohol;2-propanol;ipa",
    "acetone;propanone",
    "acetonitrile;mecn;methyl cyanide",
    "dimethyl sulfoxide;dmso",
    "dimethylformamide;dmf;n,n-dimethylformamide",
    "tetrahydrofuran;thf",
    "1,4-dioxane;dioxane",
    "dichloromethane;methylene chloride;dcm",
    "chloroform;trichloromethane",
    "carbon tetrachloride;tetrachloromethane",
    "diethyl ether;ether;ethyl ether",
    "ethyl acetate;etoac",
    "toluene;methylbenzene",
    "benzene",
    "hexane;n-hexane;hexanes",
    "heptane;n-heptane",
    "pentane;n-pentane",
    "cyclohexane",
];

/// (a, b, relation): pairs that are not fully miscible; every other pair
/// in the list mixes in all proportions at room temperature
const NOT_MISCIBLE: &[(&str, &str, &str)] = &[
    ("water", "dichloromethane", "immiscible (water dissolves about 1.3% dichloromethane)"),
    ("water", "chloroform", "immiscible (water dissolves about 0.8% chloroform)"),
    ("water", "carbon tetrachloride", "immiscible"),
    ("water", "diethyl ether", "partially miscible (water dissolves about 6.9% ether)"),
    ("water", "ethyl acetate", "partially miscible (water dissolves about 8.3% ethyl acetate)"),
    ("water", "toluene", "immiscible"),
    ("water", "benzene", "immiscible (water dissolves about 0.18% benzene)"),
    ("water", "hexane", "immiscible"),
    ("water", "heptane", "immiscible"),
    ("water", "pentane", "immiscible"),
    ("water", "cyclohexane", "immiscible"),
    ("methanol", "hexane", "immiscible (two layers)"),
    ("methanol", "heptane", "immiscible (two layers)"),
    ("methanol", "pentane", "immiscible (two layers)"),
    ("methanol", "cyclohexane", "immiscible (two layers)"),
    ("acetonitrile", "hexane", "immiscible (two layers)"),
    ("acetonitrile", "heptane", "immiscible (two layers)"),
    ("acetonitrile", "pentane", "immiscible (two layers)"),
    ("acetonitrile", "cyclohexane", "immiscible (two layers)"),
    ("dimethyl sulfoxide", "hexane", "immiscible (two layers)"),
    ("dimethyl sulfoxide", "heptane", "immiscible (two layers)"),
    ("dimethyl sulfoxide", "pentane", "immiscible (two layers)"),
    ("dimethyl sulfoxide", "cyclohexane", "immiscible (two layers)"),
    ("dimethylformamide", "hexane", "immiscible (two layers)"),
    ("dimethylformamide", "heptane", "immiscible (two layers)"),
    ("dimethylformamide", "pentane", "immiscible (two layers)"),
    ("dimethylformamide", "cyclohexane", "immiscible (two layers)"),
];

fn find(s: &str) -> Option<&'static str> {
    let s = s.trim().trim_start_matches("the ").trim();
    SOLVENTS.iter().find(|n| n.split(';').any(|x| x == s)).map(|n| n.split(';').next().unwrap())
}

fn relation(a: &str, b: &str) -> (&'static str, bool) {
    if a == b {
        return ("the same solvent", true);
    }
    match NOT_MISCIBLE.iter().find(|p| (p.0 == a && p.1 == b) || (p.0 == b && p.1 == a)) {
        Some(p) => (p.2, false),
        None => ("miscible in all proportions", true),
    }
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let note = pod("Note", "Note", text("at room temperature; based on standard solvent miscibility charts. Partially miscible pairs form two layers once the solubility limit is passed"));
    // "what solvents are miscible in acetone"
    if let Some(m) = re!(r"^(?:what |which )?solvents? (?:are |is )?(miscible|immiscible|soluble|insoluble) (?:in|with) (.+)$|^solvents? (?:miscible|compatible) with (.+)$").captures(l) {
        let s = find(m.get(2).or(m.get(3))?.as_str())?;
        let want_mix = m.get(1).map_or(true, |x| !x.as_str().starts_with('i'));
        let list: Vec<&str> = SOLVENTS.iter().map(|n| n.split(';').next().unwrap()).filter(|o| *o != s && relation(s, o).1 == want_mix).collect();
        let others: Vec<Vec<Content>> = SOLVENTS.iter().map(|n| n.split(';').next().unwrap()).filter(|o| *o != s).map(|o| vec![text(o), text(relation(s, o).0)]).collect();
        let head = format!("with {s}");
        return Some(Answer::new(format!("solvents miscible with {s}"), vec![
            pod("Input", "Input interpretation", text(format!("solvents {} with {s}", if want_mix { "miscible" } else { "not miscible" }))),
            primary(pod("Result", "Result", text(if list.is_empty() { "none of the common solvents".to_string() } else { list.join(" | ") }))),
            pod("Table", "Miscibility with common solvents", table(Some(vec!["solvent", &head]), others)),
            note,
        ]));
    }
    let m = re!(r"^(?:is |are )?(.+?) (?:soluble|miscible|dissolvable) (?:in|with) (.+)$|^(?:can (?:i|you) )?(?:dissolve|mix) (.+?) (?:in|with|and) (.+)$|^(?:does |do )?(.+?) (?:mix|dissolve) (?:with|in) (.+)$|^(.+?) (?:and |, ?|/ ?)?(.+?) miscibility$").captures(l)?;
    let a = find(m.get(1).or(m.get(3)).or(m.get(5)).or(m.get(7))?.as_str())?;
    let b = find(m.get(2).or(m.get(4)).or(m.get(6)).or(m.get(8))?.as_str())?;
    let (rel, ok) = relation(a, b);
    Some(Answer::new(format!("{a} and {b} miscibility"), vec![
        pod("Input", "Input interpretation", text(format!("{a} | {b} | miscibility"))),
        primary(pod("Result", "Result", text(format!("{}: {rel}", if ok { "yes" } else if rel.starts_with("partially") { "partly" } else { "no" })))),
        note,
    ]))
}
