//! The four nations of the United Kingdom.
//!
//! "Scotland", "population of Wales", "capital of Northern Ireland",
//! "Scotland vs Wales".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// name | capital | largest city | population (census) | area km² | languages | legislature | patron saint and day
const NATIONS: &[(&str, &str, &str, (u64, &str), f64, &str, &str, &str)] = &[
    ("England", "London", "London", (56_490_048, "2021 census"), 130_279.0, "English", "none of its own (the UK Parliament)", "Saint George, 23 April"),
    ("Scotland", "Edinburgh", "Glasgow", (5_439_842, "2022 census"), 77_910.0, "English, Scots, Scottish Gaelic", "Scottish Parliament (Holyrood), since 1999", "Saint Andrew, 30 November"),
    ("Wales", "Cardiff", "Cardiff", (3_107_494, "2021 census"), 20_779.0, "Welsh, English", "Senedd (Welsh Parliament), since 1999", "Saint David, 1 March"),
    ("Northern Ireland", "Belfast", "Belfast", (1_903_175, "2021 census"), 14_130.0, "English, Irish, Ulster Scots", "Northern Ireland Assembly (Stormont), since 1998", "Saint Patrick, 17 March"),
];

type Nation = (&'static str, &'static str, &'static str, (u64, &'static str), f64, &'static str, &'static str, &'static str);

fn rows(n: &Nation) -> Vec<(&'static str, String)> {
    let g = super::geo::fmt_int;
    vec![
        ("sovereign state", "United Kingdom".to_string()),
        ("capital", n.1.to_string()),
        ("largest city", n.2.to_string()),
        ("population", format!("{} people ({})", g(n.3 .0), n.3 .1)),
        ("area", format!("{} km² ({} mi²)", g(n.4 as u64), g((n.4 / 2.589_988).round() as u64))),
        ("density", format!("{:.0} people/km²", n.3 .0 as f64 / n.4)),
        ("languages", n.5.to_string()),
        ("legislature", n.6.to_string()),
        ("patron saint", n.7.to_string()),
    ]
}

fn prop(p: &str) -> Option<&'static str> {
    Some(match p {
        "capital" | "capital city" => "capital",
        "largest city" | "biggest city" => "largest city",
        "population" | "people" => "population",
        "area" | "size" => "area",
        "languages" | "language" | "official languages" => "languages",
        "density" | "population density" => "density",
        "patron saint" => "patron saint",
        _ => return None,
    })
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = l.trim_start_matches("what is the ").trim_start_matches("the ");
    let (p, body) = match re!(r"^([a-z ]+?) of (.+)$").captures(l) {
        Some(m) if prop(&m[1]).is_some() => (prop(&m[1]), m[2].to_string()),
        _ => match re!(r"^(.+?) (capital|population|area|languages|largest city|patron saint)$").captures(l) {
            Some(m) => (prop(&m[2]), m[1].to_string()),
            None => (None, l.to_string()),
        },
    };
    let names: Vec<&str> = re!(r"\s*(?:,|\bvs\.?|\bversus\b|\band\b)\s*").split(&body).filter(|s| !s.is_empty()).collect();
    let ns: Option<Vec<&'static Nation>> = names.iter().map(|s| NATIONS.iter().find(|n| n.0.to_lowercase() == s.trim())).collect();
    let ns = ns?;
    // England alone stays with the country lookup unless a property is asked
    if ns.is_empty() || (ns.len() == 1 && ns[0].0 == "England" && p.is_none()) {
        return None;
    }
    let note = pod("Source", "Notes", text("populations from the latest census in each nation; area includes inland water"));
    if ns.len() == 1 {
        let n = ns[0];
        let r = rows(n);
        let (title, value) = match p {
            Some(p) => (p.to_string(), r.iter().find(|(k, _)| *k == p)?.1.clone()),
            None => ("basic information".to_string(), format!("constituent country of the United Kingdom | capital {} | population {}", n.1, super::geo::fmt_int(n.3 .0))),
        };
        return Some(Answer::new(n.0, vec![
            pod("Input", "Input interpretation", text(format!("{}{}", n.0, p.map(|p| format!(" | {p}")).unwrap_or_default()))),
            primary(pod("Result", &(title[..1].to_uppercase() + &title[1..]), text(value))),
            pod("Table", "Properties", table(None, r.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())),
            note,
        ]));
    }
    let keys: Vec<&str> = p.map(|p| vec![p]).unwrap_or_else(|| vec!["capital", "population", "area"]);
    let mut header = vec!["nation"];
    header.extend(keys.iter().copied());
    let trs = ns.iter().map(|n| {
        let r = rows(n);
        let mut row = vec![text(n.0)];
        for k in &keys {
            row.push(text(r.iter().find(|(x, _)| x == k).map(|x| x.1.clone()).unwrap_or_default()));
        }
        row
    });
    let short = ns.iter().map(|n| format!("{}: {}", n.0, rows(n).into_iter().find(|(k, _)| *k == keys[0]).map(|x| x.1).unwrap_or_default())).collect::<Vec<_>>().join(" | ");
    Some(Answer::new("UK nations", vec![
        pod("Input", "Input interpretation", text(ns.iter().map(|n| n.0).collect::<Vec<_>>().join(" | "))),
        primary(pod("Result", "Comparison", text(short))),
        pod("Table", "Properties", table(Some(header), trs.collect())),
        note,
    ]))
}
