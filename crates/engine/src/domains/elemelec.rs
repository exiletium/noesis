//! Electrical conductivity and magnetic type of the chemical elements.
//!
//! "conductivity sulfur, silicon, copper", "elements sorted by
//! conductivity", "iron magnetism", "is gold magnetic".

use super::chem;
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

/// symbol | electrical resistivity near 20 °C in Ω m | note
const RESISTIVITY: &[(&str, f64, &str)] = &[
    ("Ag", 1.59e-8, ""), ("Cu", 1.68e-8, ""), ("Au", 2.44e-8, ""), ("Al", 2.65e-8, ""), ("Ca", 3.36e-8, ""), ("Be", 3.6e-8, ""),
    ("Mg", 4.39e-8, ""), ("Na", 4.77e-8, ""), ("Mo", 5.34e-8, ""), ("W", 5.6e-8, ""), ("Zn", 5.9e-8, ""), ("Co", 6.24e-8, ""),
    ("Ni", 6.99e-8, ""), ("K", 7.2e-8, ""), ("Cd", 6.8e-8, ""), ("Li", 9.28e-8, ""), ("Fe", 9.71e-8, ""), ("Pt", 1.06e-7, ""),
    ("Pd", 1.05e-7, ""), ("Sn", 1.09e-7, ""), ("Cr", 1.25e-7, ""), ("Pb", 2.2e-7, ""), ("Ti", 4.2e-7, ""), ("Hg", 9.8e-7, "liquid"),
    ("Mn", 1.44e-6, ""), ("Bi", 1.29e-6, ""), ("C", 3.5e-6, "graphite, in the basal plane; diamond is an insulator (about 10¹² Ω m)"),
    ("Ge", 0.46, "intrinsic semiconductor; doping lowers it by many orders of magnitude"),
    ("Si", 2.3e3, "intrinsic semiconductor; doping lowers it by many orders of magnitude"),
    ("S", 1e15, "insulator"), ("P", 1e9, "white phosphorus; an insulator"), ("Se", 1e4, "grey selenium; a photoconductor"),
];

/// symbol | magnetic type | note
const MAGNETIC: &[(&str, &str, &str)] = &[
    ("Fe", "ferromagnetic", "Curie temperature 1043 K (770 °C)"),
    ("Co", "ferromagnetic", "Curie temperature 1388 K (1115 °C), the highest of any element"),
    ("Ni", "ferromagnetic", "Curie temperature 627 K (354 °C)"),
    ("Gd", "ferromagnetic", "Curie temperature 292 K (19 °C): ferromagnetic only on a cool day"),
    ("Dy", "paramagnetic", "ferromagnetic below 88 K"),
    ("Cr", "antiferromagnetic", "Néel temperature 311 K"),
    ("Mn", "paramagnetic", "α-manganese is antiferromagnetic below about 95 K"),
    ("O", "paramagnetic", "O₂ has two unpaired electrons; liquid oxygen clings to a magnet"),
    ("Li", "paramagnetic", ""), ("Na", "paramagnetic", ""), ("K", "paramagnetic", ""), ("Mg", "paramagnetic", ""), ("Al", "paramagnetic", ""),
    ("Ca", "paramagnetic", ""), ("Ti", "paramagnetic", ""), ("V", "paramagnetic", ""), ("Mo", "paramagnetic", ""), ("W", "paramagnetic", ""),
    ("Pt", "paramagnetic", ""), ("Pd", "paramagnetic", ""), ("U", "paramagnetic", ""),
    ("H", "diamagnetic", ""), ("He", "diamagnetic", ""), ("Be", "diamagnetic", ""), ("C", "diamagnetic", "graphite is strongly diamagnetic; pyrolytic graphite can levitate over magnets"),
    ("N", "diamagnetic", ""), ("Ne", "diamagnetic", ""), ("Si", "diamagnetic", ""), ("P", "diamagnetic", ""), ("S", "diamagnetic", ""), ("Cl", "diamagnetic", ""),
    ("Ar", "diamagnetic", ""), ("Cu", "diamagnetic", ""), ("Zn", "diamagnetic", ""), ("Ga", "diamagnetic", ""), ("Ge", "diamagnetic", ""), ("Se", "diamagnetic", ""),
    ("Br", "diamagnetic", ""), ("Kr", "diamagnetic", ""), ("Ag", "diamagnetic", ""), ("Cd", "diamagnetic", ""), ("Sb", "diamagnetic", ""), ("I", "diamagnetic", ""),
    ("Xe", "diamagnetic", ""), ("Au", "diamagnetic", ""), ("Hg", "diamagnetic", ""), ("Pb", "diamagnetic", ""), ("Bi", "diamagnetic", "the most diamagnetic metal"),
];

fn sci(v: f64) -> String {
    if (1e-3..1e4).contains(&v.abs()) {
        fmt_float(v, 4)
    } else {
        let e = v.abs().log10().floor() as i32;
        let m = v / 10f64.powi(e);
        format!("{}×10^{}", fmt_float((m * 100.0).round() / 100.0, 3), e)
    }
}

fn split(s: &str) -> Vec<String> {
    re!(r"\s*(?:,\s*(?:and\s+)?|\s+and\s+|\s+vs\.?\s+)\s*").split(s.trim()).map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

fn conductivity(l: &str) -> Option<Answer> {
    if l.contains("thermal") || l.contains("heat") {
        return None;
    }
    if re!(r"^(?:chemical )?elements (?:sorted|ranked|ordered|by) (?:by )?(?:electrical )?conductivity$|^(?:most|best) conductive (?:elements|metals)$").is_match(l) {
        let mut v: Vec<(&chem::Element, f64, &str)> = RESISTIVITY.iter().filter_map(|r| Some((chem::by_symbol(r.0)?, r.1, r.2))).collect();
        v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let rows: Vec<Vec<Content>> = v.iter().map(|(e, r, _)| vec![text(e.name.clone()), text(format!("{} S/m", sci(1.0 / r))), text(format!("{} Ω m", sci(*r)))]).collect();
        return Some(Answer::new("elements by electrical conductivity", vec![
            pod("Input", "Input interpretation", text("chemical elements | sorted by electrical conductivity")),
            primary(pod("Result", "Most conductive", text(v.iter().take(6).map(|(e, r, _)| format!("{} ({} S/m)", e.name, sci(1.0 / r))).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Elements by conductivity (near 20 °C)", table(Some(vec!["element", "conductivity", "resistivity"]), rows)),
        ]));
    }
    let m = re!(r"^(?:the )?(?:electrical |electric )?(conductivity|resistivity)(?: of)? (.+)$|^(.+?) (?:electrical |electric )?(conductivity|resistivity)$").captures(l)?;
    let want_res = m.get(1).or(m.get(4))?.as_str() == "resistivity";
    let names = split(m.get(2).or(m.get(3))?.as_str());
    let found: Option<Vec<(&chem::Element, f64, &str)>> = names.iter().map(|n| {
        let e = chem::find_element(n)?;
        let r = RESISTIVITY.iter().find(|r| r.0 == e.symbol)?;
        Some((e, r.1, r.2))
    }).collect();
    let found = found?;
    // single-element resistivity stays with the materials tables
    if found.is_empty() || (want_res && found.len() == 1) {
        return None;
    }
    let val = |r: f64| if want_res { format!("{} Ω m", sci(r)) } else { format!("{} S/m", sci(1.0 / r)) };
    let main = if found.len() == 1 { val(found[0].1) } else { found.iter().map(|(e, r, _)| format!("{}: {}", e.name, val(*r))).collect::<Vec<_>>().join(" | ") };
    let rows: Vec<Vec<Content>> = found.iter().map(|(e, r, note)| vec![text(e.name.clone()), text(format!("{} S/m", sci(1.0 / r))), text(format!("{} Ω m", sci(*r))), text(*note)]).collect();
    Some(Answer::new("electrical conductivity", vec![
        pod("Input", "Input interpretation", text(format!("{} | electrical {}", found.iter().map(|f| f.0.name.as_str()).collect::<Vec<_>>().join(", "), if want_res { "resistivity" } else { "conductivity" }))),
        primary(pod("Result", if want_res { "Electrical resistivity" } else { "Electrical conductivity" }, text(main))),
        pod("Table", "Values near 20 °C", table(Some(vec!["element", "conductivity", "resistivity", "note"]), rows)),
    ]))
}

fn magnetism(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:is |are )?(.+?) (?:magnetism|magnetic properties|magnetic type|magnetic ordering|magnetic)$|^(?:the )?(?:magnetism|magnetic properties|magnetic type|magnetic ordering) of (.+)$").captures(l)?;
    let e = chem::find_element(m.get(1).or(m.get(2))?.as_str())?;
    let (_, kind, note) = MAGNETIC.iter().find(|x| x.0 == e.symbol)?;
    let attracted = match *kind { "ferromagnetic" => "strongly attracted to magnets and can be magnetized", "paramagnetic" => "weakly attracted to magnetic fields", "antiferromagnetic" => "no net magnetization: neighbouring spins point in opposite directions", _ => "weakly repelled by magnetic fields" };
    let question = l.starts_with("is ") || l.starts_with("are ");
    let res = if question { format!("{} ({kind})", if *kind == "ferromagnetic" { "yes" } else { "no, not in the everyday sense" }) } else { kind.to_string() };
    let mut rows = vec![vec![text("magnetic type"), text(*kind)], vec![text("behaviour"), text(attracted)]];
    if !note.is_empty() {
        rows.push(vec![text("note"), text(*note)]);
    }
    Some(Answer::new(format!("{} magnetism", e.name), vec![
        pod("Input", "Input interpretation", text(format!("{} | magnetic type", e.name))),
        primary(pod("Result", "Magnetic type", text(res))),
        pod("Table", "Magnetic properties", table(None, rows)),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    conductivity(l).or_else(|| magnetism(l))
}
