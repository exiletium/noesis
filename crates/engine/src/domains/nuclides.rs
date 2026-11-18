//! Nuclides and counting particles: "calcium-40 neutron number",
//! "mass defect of Os-184 in kg", "binding energy of Cl-37 in joules",
//! "full symbol U-235", "how many electrons are in MnCO3",
//! "hafnium term symbol".
//!
//! Masses, abundances, half-lives, spins and parities of 3,557 nuclides come
//! from the mendeleev package (MIT), extracted by tools/datagen/isotopes.py.

use super::chem::{self, Element};
use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

static ISOTOPES_Z: &[u8] = include_bytes!("../../data/isotopes.z");

pub struct Nuclide {
    pub z: u32,
    pub a: u32,
    /// Atomic mass in u.
    pub mass: Option<f64>,
    /// Natural abundance in percent.
    pub abundance: Option<f64>,
    /// Half-life in seconds; infinite when stable.
    pub half_life: Option<f64>,
    pub spin: String,
    pub parity: String,
}

pub fn nuclides() -> &'static Vec<Nuclide> {
    static N: OnceLock<Vec<Nuclide>> = OnceLock::new();
    N.get_or_init(|| {
        let bytes = miniz_oxide::inflate::decompress_to_vec(ISOTOPES_Z).unwrap_or_default();
        String::from_utf8_lossy(&bytes)
            .lines()
            .filter_map(|l| {
                let f: Vec<&str> = l.split('\t').collect();
                if f.len() < 7 {
                    return None;
                }
                Some(Nuclide {
                    z: f[0].parse().ok()?,
                    a: f[1].parse().ok()?,
                    mass: f[2].parse().ok(),
                    abundance: f[3].parse().ok(),
                    half_life: if f[4] == "stable" { Some(f64::INFINITY) } else { f[4].parse().ok() },
                    spin: f[5].to_string(),
                    parity: f[6].to_string(),
                })
            })
            .collect()
    })
}

pub fn nuclide(z: u32, a: u32) -> Option<&'static Nuclide> {
    nuclides().iter().find(|n| n.z == z && n.a == a)
}

/// The most abundant isotope (or the longest-lived one).
fn main_isotope(z: u32) -> Option<&'static Nuclide> {
    let all = nuclides().iter().filter(|n| n.z == z);
    all.clone().filter(|n| n.abundance.is_some()).max_by(|a, b| a.abundance.partial_cmp(&b.abundance).unwrap()).or_else(|| all.max_by(|a, b| a.half_life.unwrap_or(0.0).partial_cmp(&b.half_life.unwrap_or(0.0)).unwrap()))
}

const M_H: f64 = 1.007_825_032_07; // hydrogen-1 atom, u
const M_N: f64 = 1.008_664_915_95; // neutron, u
const U_MEV: f64 = 931.494_102_42;
const U_KG: f64 = 1.660_539_066_60e-27;
const MEV_J: f64 = 1.602_176_634e-13;

fn sup(n: u32) -> String {
    n.to_string().chars().map(|c| "⁰¹²³⁴⁵⁶⁷⁸⁹".chars().nth(c as usize - '0' as usize).unwrap()).collect()
}

fn sub(n: u32) -> String {
    n.to_string().chars().map(|c| "₀₁₂₃₄₅₆₇₈₉".chars().nth(c as usize - '0' as usize).unwrap()).collect()
}

fn n(v: f64) -> String {
    fmt_float(v, 7)
}

fn element_word(w: &str) -> Option<&'static Element> {
    chem::find_element(w).or_else(|| chem::elements().iter().find(|e| e.symbol.to_lowercase() == w))
}

/// "oxygen-14", "O-14", "U235", "tritium" → (element, mass number).
fn find_nuclide(l: &str) -> Option<(&'static Element, u32, std::ops::Range<usize>)> {
    for (word, z, a) in [("tritium", 1, 3), ("deuterium", 1, 2), ("protium", 1, 1), ("radiocarbon", 6, 14)] {
        if let Some(i) = l.find(word) {
            return Some((chem::elements().iter().find(|e| e.z == z)?, a, i..i + word.len()));
        }
    }
    for m in re!(r"\b([a-z]{1,13})[- ]?(\d{1,3})\b").captures_iter(l) {
        let w = &m[1];
        if matches!(w, "in" | "of" | "at" | "is" | "for" | "and" | "to" | "be" | "as" | "no") {
            continue;
        }
        if let Some(e) = element_word(w) {
            let a: u32 = m[2].parse().ok()?;
            if a >= e.z && a < 320 {
                return Some((e, a, m.get(0).unwrap().range()));
            }
        }
    }
    // "14C", "235U"
    let m = re!(r"(?:^|\s)(\d{1,3})([a-z]{1,2})(?:$|\s)").captures(l)?;
    let e = chem::elements().iter().find(|e| e.symbol.to_lowercase() == m[2])?;
    let a: u32 = m[1].parse().ok()?;
    (a >= e.z).then_some((e, a, m.get(0).unwrap().range()))
}

fn half_life_text(s: f64) -> String {
    if s.is_infinite() {
        return "stable".into();
    }
    let year = 3.15576e7;
    let (v, u) = if s >= 1e9 * year {
        (s / (1e9 * year), "billion years")
    } else if s >= 1e6 * year {
        (s / (1e6 * year), "million years")
    } else if s >= year {
        (s / year, "years")
    } else if s >= 86400.0 {
        (s / 86400.0, "days")
    } else if s >= 3600.0 {
        (s / 3600.0, "hours")
    } else if s >= 60.0 {
        (s / 60.0, "minutes")
    } else if s >= 1.0 {
        (s, "seconds")
    } else if s >= 1e-3 {
        (s * 1e3, "ms")
    } else if s >= 1e-6 {
        (s * 1e6, "µs")
    } else if s >= 1e-9 {
        (s * 1e9, "ns")
    } else {
        return format!("{} s", fmt_float(s, 4));
    };
    format!("{} {u}", fmt_float(v, 4))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    term_symbol(l).or_else(|| particle_count(q, l)).or_else(|| nuclide_question(l))
}

// ---------------------------------------------------------------------------
// biological and effective half-lives
// ---------------------------------------------------------------------------

/// (Z, A, biological half-life in days, critical organ or compartment), from
/// standard health-physics tables (Cember, Introduction to Health Physics;
/// ICRP); individual values vary widely.
const BIOLOGICAL: &[(u32, u32, f64, &str)] = &[
    (1, 3, 10.0, "whole body (as tritiated water)"),
    (6, 14, 40.0, "whole body"),
    (11, 24, 11.0, "whole body"),
    (15, 32, 1155.0, "bone"),
    (16, 35, 90.0, "skin"),
    (19, 42, 58.0, "whole body"),
    (20, 45, 18000.0, "bone"),
    (26, 59, 600.0, "blood"),
    (27, 60, 9.5, "whole body"),
    (38, 90, 18000.0, "bone"),
    (43, 99, 1.0, "whole body (as pertechnetate)"),
    (53, 123, 138.0, "thyroid"),
    (53, 125, 138.0, "thyroid"),
    (53, 129, 138.0, "thyroid"),
    (53, 131, 138.0, "thyroid"),
    (55, 134, 70.0, "whole body"),
    (55, 137, 70.0, "whole body"),
    (84, 210, 50.0, "whole body (spleen and kidneys)"),
    (88, 226, 16000.0, "bone"),
    (92, 238, 15.0, "kidney"),
    (94, 239, 73000.0, "bone"),
    (95, 241, 73000.0, "bone"),
];

fn days_text(d: f64) -> String {
    half_life_text(d * 86400.0)
}

fn biological(e: &Element, a: u32, rest: &str) -> Option<Answer> {
    let &(_, _, tb, organ) = BIOLOGICAL.iter().find(|b| b.0 == e.z && b.1 == a)?;
    let tp = nuclide(e.z, a)?.half_life.filter(|h| h.is_finite())? / 86400.0;
    let te = tp * tb / (tp + tb);
    let effective = rest.contains("effective");
    let lifetime = rest.contains("lifetime") || rest.contains("mean life");
    let name = format!("{}-{a}", e.name.to_lowercase());
    let ln2 = std::f64::consts::LN_2;
    let (title, res) = if effective && lifetime {
        ("Effective mean lifetime", format!("{} (effective half-life {} ÷ ln 2)", days_text(te / ln2), days_text(te)))
    } else if effective {
        ("Effective half-life", days_text(te))
    } else {
        ("Biological half-life", format!("{} ({organ})", days_text(tb)))
    };
    Some(Answer::new(format!("{name} biological half-life"), vec![
        pod("Input", "Input interpretation", text(format!("{name} | {}", title.to_lowercase()))),
        primary(pod("Result", title, text(res))),
        pod("Table", "Half-lives", table(None, vec![
            vec![text("physical half-life T_p"), text(days_text(tp))],
            vec![text("biological half-life T_b"), text(format!("{} ({organ})", days_text(tb)))],
            vec![text("effective half-life T_e"), text(days_text(te))],
            vec![text("effective mean lifetime"), text(days_text(te / ln2))],
        ])),
        pod("Formula", "Formula", text("1/T_e = 1/T_p + 1/T_b, so T_e = T_p·T_b/(T_p + T_b)")),
        pod("Note", "Note", text("biological half-lives are typical reference values for adults; they depend on chemical form, organ and the individual")),
    ]))
}

// ---------------------------------------------------------------------------
// nuclide questions
// ---------------------------------------------------------------------------

fn nuclide_question(l: &str) -> Option<Answer> {
    let (e, a, range) = find_nuclide(l)?;
    let rest = format!("{} {}", &l[..range.start], &l[range.end..]);
    let rest = rest.trim();
    // a biological or effective half-life is not a property of the nuclide
    if rest.contains("biological") || rest.contains("effective") {
        return biological(e, a, rest);
    }
    // only nuclide words around the nuclide, so "oxygen 2 atoms" is not one
    let prop = if rest.is_empty() || rest == "isotope" || rest == "nucleus" {
        "all"
    } else if rest.contains("mass number") || rest.contains("nucleon") {
        "A"
    } else if rest.contains("neutron") {
        "N"
    } else if rest.contains("proton") || rest.contains("atomic number") {
        "Z"
    } else if rest.contains("electron") {
        "electrons"
    } else if rest.contains("mass defect") || rest.contains("mass deficit") {
        "defect"
    } else if rest.contains("binding energy per nucleon") {
        "be_per"
    } else if rest.contains("binding energy") || rest.contains("nuclear binding") {
        "be"
    } else if rest.contains("symbol") || rest.contains("notation") {
        "symbol"
    } else if rest.contains("spin") || rest.contains("parity") {
        "spin"
    } else if rest.contains("abundance") {
        "abundance"
    } else if rest.contains("half-life") || rest.contains("half life") || rest.contains("halflife") {
        "half"
    } else if rest.contains("mass") || rest.contains("weight") {
        "mass"
    } else {
        return None;
    };
    let iso = nuclide(e.z, a);
    // a nuclide that is not in the table only answers counting questions
    if iso.is_none() && !matches!(prop, "A" | "N" | "Z" | "electrons" | "symbol") {
        return None;
    }
    let nn = a - e.z;
    let name = format!("{}-{a}", e.name.to_lowercase());
    let full = format!("{}{}{}", sup(a), sub(e.z), e.symbol);
    let brief = format!("{}{}", sup(a), e.symbol);
    let joules = rest.contains("joule") || rest.contains(" j") || rest.ends_with("in j");
    let kg = rest.contains(" kg") || rest.contains("kilogram");
    let defect = iso.and_then(|i| i.mass).map(|m| e.z as f64 * M_H + nn as f64 * M_N - m);
    let (title, result) = match prop {
        "A" => ("Mass number", format!("{a}  ({} protons + {nn} neutrons)", e.z)),
        "N" => ("Neutron number", nn.to_string()),
        "Z" => ("Protons", format!("{}  (atomic number)", e.z)),
        "electrons" => ("Electrons", format!("{} (neutral atom)", e.z)),
        "symbol" => {
            let s = if rest.contains("brief") || rest.contains("abbreviated") || rest.contains("short") { brief.clone() } else { full.clone() };
            ("Nuclide symbol", s)
        }
        "defect" => {
            let d = defect?;
            if kg {
                ("Mass defect", format!("{} kg", n(d * U_KG)))
            } else {
                ("Mass defect", format!("{} u", n(d)))
            }
        }
        "be" => {
            let b = defect? * U_MEV;
            if joules {
                ("Binding energy", format!("{} J", n(b * MEV_J)))
            } else {
                ("Binding energy", format!("{} MeV", n(b)))
            }
        }
        "be_per" => ("Binding energy per nucleon", format!("{} MeV", n(defect? * U_MEV / a as f64))),
        "spin" => ("Spin and parity", format!("{}{}", iso?.spin, iso?.parity)),
        "abundance" => ("Natural abundance", iso?.abundance.map_or("none (not found in nature in measurable amounts)".to_string(), |x| format!("{}%", fmt_float(x, 6)))),
        "half" => ("Half-life", half_life_text(iso?.half_life?)),
        "mass" => ("Atomic mass", format!("{} u", iso?.mass.map(n)?)),
        _ => ("Nuclide", format!("{full}  ({} protons, {nn} neutrons)", e.z)),
    };
    let mut rows = vec![
        vec![text("symbol"), text(format!("{full}  ({}-{a})", e.symbol))],
        vec![text("protons"), text(e.z.to_string())],
        vec![text("neutrons"), text(nn.to_string())],
        vec![text("mass number"), text(a.to_string())],
    ];
    if let Some(i) = iso {
        if let Some(m) = i.mass {
            rows.push(vec![text("atomic mass"), text(format!("{} u", n(m)))]);
        }
        if let Some(d) = defect {
            rows.push(vec![text("mass defect"), text(format!("{} u = {} kg", n(d), n(d * U_KG)))]);
            rows.push(vec![text("binding energy"), text(format!("{} MeV = {} J", n(d * U_MEV), n(d * U_MEV * MEV_J)))]);
            if a > 1 {
                rows.push(vec![text("binding energy per nucleon"), text(format!("{} MeV", n(d * U_MEV / a as f64)))]);
            }
        }
        if let Some(h) = i.half_life {
            rows.push(vec![text("half-life"), text(half_life_text(h))]);
        }
        if let Some(ab) = i.abundance {
            rows.push(vec![text("natural abundance"), text(format!("{}%", fmt_float(ab, 6)))]);
        }
        if !i.spin.is_empty() {
            rows.push(vec![text("spin and parity"), text(format!("{}{}", i.spin, i.parity))]);
        }
    }
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{name} | {}", title.to_lowercase())))];
    if prop == "all" {
        pods.push(primary(pod("Result", "Nuclide properties", table(None, rows))));
    } else {
        pods.push(primary(pod("Result", title, text(result))));
        pods.push(pod("Props", "Nuclide properties", table(None, rows)));
    }
    if matches!(prop, "defect" | "be" | "be_per") {
        pods.push(pod("Formula", "Formula", text("Δm = Z m(¹H) + N m(n) − m(atom);  E = Δm c² (931.494 MeV per u)")));
    }
    Some(Answer::new(format!("{name} {}", title.to_lowercase()), pods))
}

// ---------------------------------------------------------------------------
// particle counts in compounds
// ---------------------------------------------------------------------------

fn particle_count(q: &Query, l: &str) -> Option<Answer> {
    let (what, name) = if let Some(m) = re!(r"^(?:how many|(?:the )?(?:total )?number of|count (?:the )?(?:number of )?)?\s*(?:total )?(electrons|protons|neutrons|atoms)\s+(?:are |is )?(?:there )?(?:in|of|does)\s+(?:a |an |one )?(?:molecule of |formula unit of )?(.+?)(?:\s+(?:have|contain))?$").captures(l) {
        (m[1].to_string(), m[2].to_string())
    } else if let Some(m) = re!(r"^(.+?)\s+(electron|proton|neutron|atom)s?\s+(?:count|number)$").captures(l).or_else(|| re!(r"^(electron|proton|neutron|atom) count (?:of |in |for )?(.+)$").captures(l).map(|m| m)) {
        if m[1].ends_with("count") || ["electron", "proton", "neutron", "atom"].contains(&&m[1]) {
            (format!("{}s", &m[1]), m[2].to_string())
        } else {
            (format!("{}s", &m[2]), m[1].to_string())
        }
    } else {
        return None;
    };
    // keep the formula's case: "MnCO3"
    let orig = q.clean.as_str();
    let start = orig.to_lowercase().find(&name)?;
    let raw = orig[start..start + name.len()].trim();
    if find_nuclide(&name).is_some() && chem::compound_formula(&name).is_none() {
        return None;
    }
    let (formula, label) = if let Some(f) = chem::compound_formula(&name) {
        (f.to_string(), format!("{name} ({f})"))
    } else if let Some(e) = chem::find_element(raw) {
        (e.symbol.clone(), e.name.to_lowercase())
    } else {
        (raw.to_string(), raw.to_string())
    };
    let counts: BTreeMap<String, u32> = chem::parse_formula(&formula)?;
    let mut rows = Vec::new();
    let mut total = 0u64;
    for (sym, k) in &counts {
        let e = chem::by_symbol(sym)?;
        let per = match what.as_str() {
            "atoms" => 1,
            "neutrons" => main_isotope(e.z).map(|i| (i.a - e.z) as u64)?,
            _ => e.z as u64,
        };
        total += per * *k as u64;
        rows.push(vec![text(sym.clone()), text(k.to_string()), text(per.to_string()), text((per * *k as u64).to_string())]);
    }
    let unit = &what[..what.len() - 1];
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{label} | number of {what}"))),
        primary(pod("Result", &format!("Number of {what}"), text(total.to_string()))),
        pod("Breakdown", "By element", table(Some(vec!["element", "atoms", &format!("{unit}s per atom"), "total"]), rows)),
    ];
    if what == "neutrons" {
        pods.push(pod("Note", "Note", text("Counted for the most abundant isotope of each element.")));
    }
    if what == "electrons" {
        pods.push(pod("Note", "Note", text("A neutral molecule has as many electrons as protons.")));
    }
    Some(Answer::new(format!("{what} in {label}"), pods))
}

// ---------------------------------------------------------------------------
// ground-state term symbols by Hund's rules
// ---------------------------------------------------------------------------

fn l_of(c: char) -> Option<u32> {
    "spdfg".find(c).map(|i| i as u32)
}

/// (2S+1, L, 2J, parity odd) of the ground state.
pub fn ground_term(e: &Element) -> Option<(u32, u32, u32, bool)> {
    // cerium is the one element where Hund's rules on the configuration
    // miss the observed ground level (¹G°₄)
    if e.z == 58 {
        return Some((1, 4, 8, true));
    }
    let cfg = e.config.as_ref()?;
    let mut two_s = 0i32;
    let mut l_tot = 0i32;
    let mut parity = 0u32;
    let mut open: Vec<(u32, u32)> = Vec::new(); // (l, electrons)
    for t in cfg.split_whitespace() {
        let mut ch = t.chars();
        ch.next()?; // shell number (one digit up to n = 7)
        let l = l_of(ch.next()?)?;
        let k: u32 = ch.as_str().parse().ok()?;
        parity += l * k;
        let orbitals = 2 * l + 1;
        if k == 0 || k == 2 * orbitals {
            continue;
        }
        let up = k.min(orbitals);
        let down = k - up;
        two_s += up as i32 - down as i32;
        // maximum M_L: spin-up electrons from m = l downward, then spin-down
        let ml = |count: u32| -> i32 { (0..count).map(|i| l as i32 - i as i32).sum() };
        l_tot += ml(up) + ml(down);
        open.push((l, k));
    }
    let two_l = 2 * l_tot;
    // J: |L − S| when the deciding subshell is less than half full
    open.sort_by(|a, b| b.0.cmp(&a.0));
    let mut less = true;
    for (l, k) in &open {
        let half = 2 * l + 1;
        if *k != half {
            less = *k < half;
            break;
        }
    }
    let two_j = if less { (two_l - two_s).unsigned_abs() } else { (two_l + two_s) as u32 };
    Some((two_s as u32 + 1, l_tot as u32, two_j, parity % 2 == 1))
}

pub fn term_text(t: (u32, u32, u32, bool)) -> String {
    const LETTERS: &str = "SPDFGHIKLMNOQRTUV";
    let letter = LETTERS.chars().nth(t.1 as usize).unwrap_or('?');
    let j = if t.2 % 2 == 0 { sub(t.2 / 2) } else { format!("{}/{}", sub(t.2), sub(2)) };
    format!("{}{letter}{}{j}", sup(t.0), if t.3 { "°" } else { "" })
}

fn term_symbol(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:what is |what are |compare |show )?(?:the )?(?:ground[- ]state )?(?:term symbols?|spectroscopic terms?|ground[- ]state terms?)\s+(?:for|of)\s+(.+)$").captures(l).map(|m| m[1].to_string()).or_else(|| re!(r"^(.+?)\s+(?:ground[- ]state )?term symbols?$").captures(l).map(|m| m[1].to_string()))?;
    let names: Vec<&str> = m.split([',', '&']).flat_map(|p| p.split(" and ")).map(str::trim).filter(|p| !p.is_empty()).collect();
    let els: Vec<&Element> = names.iter().map(|w| element_word(w)).collect::<Option<_>>()?;
    let rows: Vec<Vec<Content>> = els.iter().map(|e| Some(vec![text(e.name.clone()), text(term_text(ground_term(e)?)), text(chem_short(e))])).collect::<Option<_>>()?;
    let res = if els.len() == 1 { content_text(&rows[0][1]) } else { els.iter().zip(&rows).map(|(e, r)| format!("{}: {}", e.symbol, content_text(&r[1]))).collect::<Vec<_>>().join("  |  ") };
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("{} | ground-state term symbol", els.iter().map(|e| e.name.as_str()).collect::<Vec<_>>().join(", ")))),
        primary(pod("Result", "Term symbol", text(res))),
        pod("Table", "Ground states", table(Some(vec!["element", "term symbol", "configuration"]), rows)),
        pod("Note", "Note", text("From the ground configuration by Hund's rules: maximum S, then maximum L, J = |L − S| for subshells less than half full and L + S otherwise.")),
    ];
    Some(Answer::new(format!("term symbol of {}", els.iter().map(|e| e.name.as_str()).collect::<Vec<_>>().join(", ")), pods))
}

fn chem_short(e: &Element) -> String {
    e.config.clone().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn term(sym: &str) -> String {
        term_text(ground_term(chem::by_symbol(sym).unwrap()).unwrap())
    }

    #[test]
    fn terms() {
        assert_eq!(term("H"), "²S₁/₂");
        assert_eq!(term("C"), "³P₀");
        assert_eq!(term("O"), "³P₂");
        assert_eq!(term("Si"), "³P₀");
        assert_eq!(term("Cr"), "⁷S₃");
        assert_eq!(term("Fe"), "⁵D₄");
        assert_eq!(term("Hf"), "³F₂");
        assert_eq!(term("Nb"), "⁶D₁/₂");
        assert_eq!(term("Gd"), "⁹D°₂");
        assert_eq!(term("U"), "⁵L°₆");
        assert_eq!(term("Pa"), "⁴K₁₁/₂");
        assert_eq!(term("Ne"), "¹S₀");
    }

    #[test]
    fn nuclide_numbers() {
        let o = nuclide(8, 16).unwrap();
        let d = 8.0 * M_H + 8.0 * M_N - o.mass.unwrap();
        assert!((d * U_MEV - 127.62).abs() < 0.05, "{}", d * U_MEV);
        assert!(nuclides().len() > 3000);
    }
}
