//! Oxidation numbers of the atoms in a compound or polyatomic ion, from the
//! usual assignment rules with the remaining element taking up the charge.
//!
//! "oxidation numbers of bicarbonate anion", "oxidation states of KMnO4",
//! "oxidation number of S in H2SO4", "oxidation numbers of SO4^2-".

use crate::engine::*;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

const ALKALI: &[&str] = &["Li", "Na", "K", "Rb", "Cs", "Fr"];
const ALKALINE: &[&str] = &["Be", "Mg", "Ca", "Sr", "Ba", "Ra"];
const METALS: &[&str] = &["Li", "Na", "K", "Rb", "Cs", "Be", "Mg", "Ca", "Sr", "Ba", "Al", "Zn", "Ag"];

/// The species: its formula, charge and a display name.
fn species(s: &str) -> Option<(String, i32, String)> {
    let s = s.trim().trim_start_matches("the ").trim();
    let s = re!(r"\s+(?:anion|cation|ion|molecule|compound)$").replace(s, "").to_string();
    let lower = s.to_lowercase();
    if let Some((name, f, c)) = super::solutions::IONS.iter().find(|(n, _, _)| *n == lower.as_str()) {
        return Some((f.to_string(), *c, name.to_string()));
    }
    // a formula with an optional charge: "HCO3-", "SO4^2-", "SO4 2-", "NH4+"
    // a charge number needs "^" or a space before it, so "NH4+" is NH4 with charge +1
    if let Some(m) = re!(r"^([A-Z][A-Za-z0-9()\[\]]*?)(?:(?:\^|\s+)\(?(\d+)?([+-])\)?|([+-]))?$").captures(&s) {
        let f = m[1].to_string();
        if super::chem::parse_formula(&f).is_some() {
            let charge = match (m.get(3).or(m.get(4)), m.get(2)) {
                (Some(sg), n) => {
                    let n: i32 = n.map_or(Some(1), |x| x.as_str().parse().ok())?;
                    if sg.as_str() == "-" { -n } else { n }
                }
                (None, _) => 0,
            };
            return Some((f.clone(), charge, s.clone()));
        }
    }
    let f = super::chem::compound_formula(&lower)?;
    Some((f.to_string(), 0, lower))
}

/// Oxidation number of each element (a fraction when an average).
fn assign(counts: &BTreeMap<String, u32>, charge: i32) -> Option<Vec<(String, i32, i32)>> {
    // one element: O₂ is 0, O₂²⁻ (peroxide) is −1 each
    if counts.len() == 1 {
        let (e, n) = counts.iter().next()?;
        let g = gcd(charge.abs(), *n as i32).max(1);
        return Some(vec![(e.clone(), charge / g, *n as i32 / g)]);
    }
    let has_o = counts.contains_key("O");
    let nonmetal_besides_h = counts.keys().any(|k| k != "H" && !METALS.contains(&k.as_str()));
    let mut known: BTreeMap<&str, i32> = BTreeMap::new();
    for el in counts.keys() {
        let e = el.as_str();
        let v = match e {
            "F" => Some(-1),
            _ if ALKALI.contains(&e) => Some(1),
            _ if ALKALINE.contains(&e) => Some(2),
            "Al" => Some(3),
            "Zn" => Some(2),
            "Ag" => Some(1),
            "H" => Some(if nonmetal_besides_h { 1 } else { -1 }),
            "O" if !counts.contains_key("F") => Some(-2),
            "Cl" | "Br" | "I" if !has_o => Some(-1),
            _ => None,
        };
        if let Some(v) = v {
            known.insert(e, v);
        }
    }
    let unknown: Vec<&String> = counts.keys().filter(|k| !known.contains_key(k.as_str())).collect();
    let fixed: i32 = known.iter().map(|(e, v)| v * counts[*e] as i32).sum();
    let mut out: Vec<(String, i32, i32)> = Vec::new();
    match unknown.len() {
        0 => {
            if fixed != charge {
                return None;
            }
        }
        1 => {
            let n = counts[unknown[0]] as i32;
            let rest = charge - fixed;
            let g = gcd(rest.abs(), n).max(1);
            out.push((unknown[0].clone(), rest / g, n / g));
        }
        _ => return None,
    }
    for (e, v) in known {
        out.push((e.to_string(), v, 1));
    }
    // formula order: as the elements appear by electronegativity is overkill; keep metals, the central atom, H, O
    out.sort_by_key(|(e, ..)| match e.as_str() {
        e if METALS.contains(&e) => 0,
        "H" => 2,
        "O" | "F" | "Cl" | "Br" | "I" => 3,
        _ => 1,
    });
    Some(out)
}

fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn sign(n: i32, d: i32) -> String {
    let v = if d == 1 { n.abs().to_string() } else { format!("{}/{}", n.abs(), d) };
    match n.signum() {
        1 => format!("+{v}"),
        -1 => format!("−{v}"),
        _ => "0".into(),
    }
}

pub fn handle(q: &Query) -> Option<Answer> {
    let raw = q.raw.trim().trim_end_matches(['?', '.']).trim();
    let raw = re!(r"\s+").replace_all(raw, " ").to_string();
    let m = re!(r"(?i)^(?:what (?:is|are) )?(?:the )?oxidation (?:numbers?|states?)(?: of (?:the )?([A-Z][a-z]?) in| (?:of|in|for))? (.+)$").captures(&raw)?;
    let (formula, charge, name) = species(&m[2])?;
    let counts = super::chem::parse_formula(&formula)?;
    // a lone element is the element handler's ("oxidation states of iron")
    if charge == 0 && counts.len() == 1 && counts.values().all(|n| *n == 1) {
        return None;
    }
    let ox = assign(&counts, charge)?;
    let charge_s = match charge {
        0 => String::new(),
        c => format!(" (charge {})", if c > 0 { format!("+{c}") } else { format!("−{}", -c) }),
    };
    let interp = format!("{name}{} | oxidation numbers", if name.contains(&formula) { charge_s.clone() } else { format!(" ({formula}{charge_s})") });
    let averaged = ox.iter().any(|(e, _, d)| *d != 1 || (counts[e] > 1 && !matches!(e.as_str(), "H" | "O" | "F" | "Cl" | "Br" | "I") && !METALS.contains(&e.as_str())));
    let result = if let Some(el) = m.get(1) {
        let (_, n, d) = ox.iter().find(|(e, ..)| e == el.as_str())?;
        format!("{}: {}", el.as_str(), sign(*n, *d))
    } else {
        ox.iter().map(|(e, n, d)| format!("{e}: {}", sign(*n, *d))).collect::<Vec<_>>().join(" | ")
    };
    let rows = ox.iter().map(|(e, n, d)| vec![text(e.clone()), text(counts[e].to_string()), text(sign(*n, *d))]).collect();
    let mut pods = vec![
        pod("Input", "Input interpretation", text(interp.clone())),
        primary(pod("Result", "Oxidation numbers", text(result))),
        pod("Table", "Atoms", table(Some(vec!["element", "atoms", "oxidation number"]), rows)),
    ];
    if averaged {
        pods.push(pod("Note", "Note", text("an element with several atoms gets the average oxidation number; individual atoms may differ")));
    }
    Some(Answer::new(interp, pods))
}
