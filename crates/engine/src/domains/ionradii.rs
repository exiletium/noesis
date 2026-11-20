//! Ionic radii: Shannon's (1976) effective ionic and crystal radii by charge,
//! coordination and spin state. "chloride ionic radius", "ionic radius of
//! Fe3+", "Na+ ionic radius".

use super::chem::{by_symbol, find_element};
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

static DATA: &str = include_str!("../../data/ionic_radii.txt");

struct Radius {
    symbol: &'static str,
    charge: i32,
    coordination: &'static str,
    spin: &'static str,
    ionic: f64,
    crystal: f64,
    reliable: bool,
}

fn radii() -> &'static Vec<Radius> {
    static R: OnceLock<Vec<Radius>> = OnceLock::new();
    R.get_or_init(|| {
        DATA.lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| {
                let c: Vec<&'static str> = l.split('\t').collect();
                (c.len() >= 7).then(|| Radius { symbol: c[0], charge: c[1].parse().unwrap_or(0), coordination: c[2], spin: c[3], ionic: c[4].parse().unwrap_or(f64::NAN), crystal: c[5].parse().unwrap_or(f64::NAN), reliable: c[6] == "1" })
            })
            .collect()
    })
}

/// Thermochemical radii of polyatomic ions (Marcus, *Ion Properties*, 1997).
const POLYATOMIC: &[(&str, &str, f64)] = &[("acetate", "CH₃COO⁻", 162.0)];

const ANIONS: &[(&str, &str, i32)] = &[("fluoride", "F", -1), ("chloride", "Cl", -1), ("bromide", "Br", -1), ("iodide", "I", -1), ("oxide", "O", -2), ("sulfide", "S", -2), ("sulphide", "S", -2), ("selenide", "Se", -2), ("telluride", "Te", -2), ("nitride", "N", -3), ("hydride", "H", -1)];

fn sup(z: i32) -> String {
    let digits = |n: i32| n.to_string().chars().map(|c| match c {
        '0' => '⁰', '1' => '¹', '2' => '²', '3' => '³', '4' => '⁴', '5' => '⁵', '6' => '⁶', '7' => '⁷', '8' => '⁸', _ => '⁹',
    }).collect::<String>();
    let n = if z.abs() == 1 { String::new() } else { digits(z.abs()) };
    format!("{n}{}", if z > 0 { '⁺' } else { '⁻' })
}

fn roman(r: &str) -> Option<i32> {
    Some(match r {
        "i" => 1, "ii" => 2, "iii" => 3, "iv" => 4, "v" => 5, "vi" => 6, "vii" => 7, "viii" => 8,
        _ => return None,
    })
}

/// (symbol, charge, explicit) for an ion written as a name, symbol or formula.
fn ion(s: &str) -> Option<(String, Option<i32>)> {
    let s = s.trim().trim_start_matches("the ").trim_end_matches(" ion").trim_end_matches(" cation").trim_end_matches(" anion").trim();
    let l = s.to_lowercase();
    if let Some(a) = ANIONS.iter().find(|a| a.0 == l) {
        return Some((a.1.to_string(), Some(a.2)));
    }
    // "Fe3+", "Fe+3", "Fe^3+", "O2-", "Na+"
    if let Some(m) = re!(r"^([A-Z][a-z]?)\s*\^?\s*(?:(\d?)([+-])|([+-])(\d))$").captures(s) {
        let e = by_symbol(&m[1])?;
        let (k, sign) = match (m.get(2), m.get(3), m.get(4), m.get(5)) {
            (Some(k), Some(sg), _, _) => (k.as_str().parse().unwrap_or(1), sg.as_str()),
            (_, _, Some(sg), Some(k)) => (k.as_str().parse().unwrap_or(1), sg.as_str()),
            _ => return None,
        };
        return Some((e.symbol.clone(), Some(if sign == "+" { k } else { -k })));
    }
    // "iron(iii)", "iron 3+", "iron iii", "iron"
    let m = re!(r"^([a-z]+)\s*(?:\(\s*([ivx]+|\d\+?)\s*\)|\s+([ivx]+|\d\+))?$").captures(&l)?;
    let e = find_element(&m[1]).or_else(|| by_symbol(s))?;
    let z = match m.get(2).or(m.get(3)) {
        Some(r) => Some(roman(r.as_str()).or_else(|| r.as_str().trim_end_matches('+').parse().ok())?),
        None => None,
    };
    Some((e.symbol.clone(), z))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches('?').trim();
    let kw = r"(?:effective |shannon |thermochemical )?(?:ionic radius|ionic radii|ion radius|ion size|ionic size|crystal radius|crystal radii)";
    let who = Regex::new(&format!(r"(?i)^(?:what is |what's )?(?:the )?{kw} (?:of|for) (?:an? |the )?(.+)$")).ok()?.captures(l).or_else(|| Regex::new(&format!(r"(?i)^(.+?)(?:'s)? {kw}$")).ok()?.captures(l))?[1].to_string();
    if let Some(p) = POLYATOMIC.iter().find(|p| who.to_lowercase().trim_end_matches(" ion") == p.0) {
        return Some(Answer::new("ionic radius", vec![
            pod("Input", "Input interpretation", text(format!("{} ({}) | ionic radius", p.0, p.1))),
            primary(pod("Result", "Result", text(format!("{} pm (thermochemical radius)", fmt_float(p.2, 4))))),
            pod("Notes", "Notes", text("Polyatomic ions have no Shannon radius; the thermochemical radius is the radius of a sphere that reproduces the ion's lattice energies (Marcus, Ion Properties, 1997).")),
        ]));
    }
    let (sym, z) = ion(&who)?;
    let e = by_symbol(&sym)?;
    let all: Vec<&Radius> = radii().iter().filter(|r| r.symbol == e.symbol).collect();
    if all.is_empty() {
        return None;
    }
    let charge = match z {
        Some(z) => z,
        None => {
            let ox = e.oxidation_states.clone().unwrap_or_default();
            ox.iter().copied().find(|c| all.iter().any(|r| r.charge == *c)).unwrap_or(all[0].charge)
        }
    };
    let list: Vec<&&Radius> = all.iter().filter(|r| r.charge == charge).collect();
    if list.is_empty() {
        return None;
    }
    // the six-coordinate (high-spin) value is the usual quoted radius
    let main = list.iter().find(|r| r.coordination == "VI" && r.spin != "LS").or_else(|| list.iter().find(|r| r.reliable)).unwrap_or(&list[0]);
    let cn = |c: &str| -> String {
        let (num, geo) = c.split_at(c.find(|ch: char| !"IVX".contains(ch)).unwrap_or(c.len()));
        let n = num.chars().rev().fold((0, 0), |(t, prev), ch| {
            let v = match ch { 'I' => 1, 'V' => 5, _ => 10 };
            if v < prev { (t - v, prev) } else { (t + v, v) }
        }).0;
        match geo {
            "SQ" => format!("{n} (square planar)"),
            "PY" => format!("{n} (pyramidal)"),
            "" => n.to_string(),
            g => format!("{n} ({})", g.to_lowercase()),
        }
    };
    let name = format!("{}{}", e.symbol, sup(charge));
    let crystal = l.to_lowercase().contains("crystal radi");
    let rows: Vec<Vec<Content>> = list.iter().map(|r| vec![text(cn(r.coordination)), text(match r.spin { "HS" => "high spin", "LS" => "low spin", _ => "" }), text(format!("{} pm", fmt_float(r.ionic, 4))), text(format!("{} pm", fmt_float(r.crystal, 4)))]).collect();
    let mut charges: Vec<i32> = all.iter().map(|r| r.charge).collect();
    charges.dedup();
    let spin = match main.spin { "HS" => ", high spin", "LS" => ", low spin", _ => "" };
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{name} ({} ion) | ionic radius", e.name.to_lowercase()))),
        primary(pod("Result", "Result", text(format!("{} pm{} (coordination number {}{spin})", fmt_float(if crystal { main.crystal } else { main.ionic }, 4), if crystal { " crystal radius" } else { "" }, cn(main.coordination))))),
        pod("Radii", "Effective ionic radii", table(Some(vec!["coordination", "spin", "ionic radius", "crystal radius"]), rows)),
    ];
    if charges.len() > 1 {
        pods.push(pod("Charges", "Other charge states", text(charges.iter().filter(|c| **c != charge).map(|c| format!("{}{}", e.symbol, sup(*c))).collect::<Vec<_>>().join(", "))));
    }
    pods.push(pod("Notes", "Notes", text("Shannon (1976) effective ionic radii, based on r(O²⁻) = 140 pm in six-fold coordination; crystal radii are 14 pm larger for cations and 14 pm smaller for anions.")));
    Some(Answer::new("ionic radius", pods))
}
