//! Reactions and compounds in solution: total and net ionic equations and
//! spectator ions, equilibrium-constant and reaction-quotient expressions,
//! combustion reactions, empirical formulas from masses, ionic compound
//! names ("calcium hydrogen phosphate", "silver(I) selenide"), elemental
//! composition, compound comparisons and pH indicators.

use super::chem::{by_symbol, compound_pods, find_element, parse_formula};
use super::chemcalc::{coefficients, find_substance, pretty, substance};
use super::solutions::IONS;
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

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a.abs()
    } else {
        gcd(b, a % b)
    }
}

fn sup(z: i32) -> String {
    let digits = |s: String| s.chars().map(|c| match c {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        _ => c,
    }).collect::<String>();
    let m = z.unsigned_abs();
    let sign = if z > 0 { "⁺" } else { "⁻" };
    if z == 0 {
        String::new()
    } else if m == 1 {
        sign.to_string()
    } else {
        format!("{}{sign}", digits(m.to_string()))
    }
}

fn ion_text(f: &str, z: i32) -> String {
    format!("{}{}", pretty(f), sup(z))
}

// ---------------------------------------------------------------------------
// ions of a compound
// ---------------------------------------------------------------------------

/// Monatomic anion charges by symbol.
fn mono_anion(sym: &str) -> Option<i32> {
    Some(match sym {
        "F" | "Cl" | "Br" | "I" | "H" => -1,
        "O" | "S" | "Se" | "Te" => -2,
        "N" | "P" | "As" => -3,
        "C" => -4,
        _ => return None,
    })
}

fn anion_charge(f: &str) -> Option<i32> {
    if let Some(z) = mono_anion(f) {
        return Some(z);
    }
    let alt = match f {
        "C2H3O2" | "OOCCH3" => "CH3COO",
        "HCO2" => "HCOO",
        "NCS" => "SCN",
        "CNO" => "OCN",
        x => x,
    };
    IONS.iter().find(|i| i.1 == alt && i.2 < 0).map(|i| i.2)
}

fn is_metal(sym: &str) -> bool {
    by_symbol(sym).and_then(|e| e.series.as_deref()).is_some_and(|s| s.contains("metal") || s == "Lanthanides" || s == "Actinides")
}

/// An ionic compound split into ions: (cation, count, charge, anion, count, charge).
#[derive(Clone, Debug)]
struct Ionic {
    cat: String,
    nc: u32,
    zc: i32,
    an: String,
    na: u32,
    za: i32,
}

fn decompose(f: &str) -> Option<Ionic> {
    let (cat, nc, rest): (String, u32, &str) = if let Some(r) = f.strip_prefix("(NH4)") {
        let d: String = r.chars().take_while(|c| c.is_ascii_digit()).collect();
        ("NH4".into(), d.parse().unwrap_or(1), &r[d.len()..])
    } else if let Some(r) = f.strip_prefix("NH4") {
        ("NH4".into(), 1, r)
    } else if f.starts_with("Hg2") && f[3..].starts_with(|c: char| c.is_ascii_uppercase() || c == '(') {
        ("Hg2".into(), 1, &f[3..])
    } else {
        let sym_len = if f.len() > 1 && f.as_bytes()[1].is_ascii_lowercase() { 2 } else { 1 };
        let sym = f.get(..sym_len)?;
        if !sym.starts_with(|c: char| c.is_ascii_uppercase()) {
            return None;
        }
        let r = &f[sym_len..];
        let d: String = r.chars().take_while(|c| c.is_ascii_digit()).collect();
        (sym.to_string(), d.parse().unwrap_or(1), &r[d.len()..])
    };
    if rest.is_empty() {
        return None;
    }
    let (an, na) = if let Some(inner) = rest.strip_prefix('(') {
        let (x, after) = inner.split_once(')')?;
        (x.to_string(), if after.is_empty() { 1 } else { after.parse().ok()? })
    } else if let Some(m) = re!(r"^([A-Z][a-z]?)(\d*)$").captures(rest) {
        (m[1].to_string(), if m[2].is_empty() { 1 } else { m[2].parse().ok()? })
    } else {
        (rest.to_string(), 1)
    };
    let za = anion_charge(&an)?;
    let total = -za * na as i32;
    if total <= 0 || total % nc as i32 != 0 {
        return None;
    }
    let zc = total / nc as i32;
    let ok_cat = cat == "NH4" || cat == "Hg2" || cat == "H" || is_metal(&cat);
    if !ok_cat || zc > 4 || (cat == "H" && zc != 1) || (cat == "NH4" && zc != 1) || (cat == "Hg2" && zc != 2) {
        return None;
    }
    if cat == "H" && an == "O" {
        return None; // water
    }
    Some(Ionic { cat, nc, zc, an, na, za })
}

const STRONG_ACID_ANIONS: &[&str] = &["Cl", "Br", "I", "NO3", "ClO4", "ClO3", "SO4", "BrO3", "IO3", "MnO4"];

fn strong_electrolyte(i: &Ionic) -> bool {
    if i.cat == "H" {
        return STRONG_ACID_ANIONS.contains(&i.an.as_str());
    }
    true
}

/// Textbook solubility rules for an ionic compound in water.
fn soluble(i: &Ionic) -> bool {
    let group1 = by_symbol(&i.cat).and_then(|e| e.group) == Some(1) && i.cat != "H";
    if group1 || i.cat == "NH4" || i.cat == "H" {
        return true;
    }
    match i.an.as_str() {
        "NO3" | "CH3COO" | "C2H3O2" | "ClO4" | "ClO3" | "HCO3" => true,
        "Cl" | "Br" | "I" => !matches!(i.cat.as_str(), "Ag" | "Pb" | "Hg2" | "Cu" if i.zc == 1 || i.cat != "Cu"),
        "SO4" => !matches!(i.cat.as_str(), "Ba" | "Sr" | "Pb" | "Ca" | "Hg2" | "Ag"),
        "OH" => matches!(i.cat.as_str(), "Ba" | "Sr" | "Ca"),
        "F" => !matches!(i.cat.as_str(), "Mg" | "Ca" | "Sr" | "Ba" | "Pb"),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// species in an equation
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Species {
    coef: Option<i64>,
    formula: String,
    charge: i32,
    state: Option<String>,
}

const ROMAN: &[(&str, i32)] = &[("i", 1), ("ii", 2), ("iii", 3), ("iv", 4), ("v", 5), ("vi", 6), ("vii", 7), ("1", 1), ("2", 2), ("3", 3), ("4", 4), ("5", 5), ("6", 6), ("7", 7)];

/// "lead(2)", "chromium(III)", "iron ii" → (symbol, charge)
fn named_cation(s: &str) -> Option<(String, i32)> {
    let l = s.trim().to_lowercase();
    let m = re!(r"^([a-z]+)\s*(?:\(\s*([ivx]+|\d)\s*\+?\s*\)|\s([ivx]+))(?:\s+(?:ion|cation))?$").captures(&l)?;
    let e = find_element(&m[1])?;
    let r = m.get(2).or(m.get(3))?.as_str();
    let z = ROMAN.iter().find(|(k, _)| *k == r)?.1;
    (is_metal(&e.symbol) || e.symbol == "H").then(|| (e.symbol.clone(), z))
}

fn split_plus(side: &str) -> Vec<String> {
    // a '+' separates species unless it is a charge: "Pb2+ + Cl-", "Na+(aq)"
    let chars: Vec<char> = side.chars().collect();
    let mut parts = Vec::new();
    let mut cur = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c == '+' {
            let prev_ws = i > 0 && chars[i - 1].is_whitespace();
            let rest: String = chars[i + 1..].iter().collect();
            let rest_t = rest.trim_start();
            let state_next = re!(r"^\((?:aq|s|l|g)\)").is_match(rest_t);
            let next = rest_t.chars().next();
            let next_ok = next.is_some_and(|n| n.is_alphanumeric() || n == '[' || (n == '(' && !state_next));
            let split = if prev_ws {
                true
            } else if rest.starts_with(char::is_whitespace) {
                next_ok
            } else {
                next.is_some_and(|n| n.is_ascii_uppercase() || n.is_ascii_digit() || n == '[' || (n == '(' && !state_next))
            };
            if split && !cur.trim().is_empty() {
                parts.push(std::mem::take(&mut cur));
                continue;
            }
        }
        cur.push(c);
    }
    parts.push(cur);
    parts.into_iter().map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
}

fn parse_species(p: &str) -> Option<Species> {
    let mut p = p.trim().to_string();
    let mut state = None;
    if let Some(m) = re!(r"\s*\((aq|s|l|g)\)\s*$").captures(&p) {
        state = Some(m[1].to_string());
        p = p[..m.get(0)?.start()].trim().to_string();
    }
    let mut coef = None;
    if let Some(m) = re!(r"^(\d+(?:\.\d+)?)\s*(?:mols?|moles?)?\s*").captures(&p) {
        let rest = &p[m.get(0)?.end()..];
        if rest.starts_with(|c: char| c.is_ascii_uppercase() || c.is_ascii_lowercase() || c == '(') && !m[1].contains('.') {
            coef = m[1].parse().ok();
            p = rest.to_string();
        }
    }
    if let Some((sym, z)) = named_cation(&p) {
        return Some(Species { coef, formula: sym, charge: z, state });
    }
    // written charges: "Cl-", "Pb2+", "SO4^2-", "PO4 3-"
    if let Some(m) = re!(r"^([A-Z][A-Za-z0-9()]*?)\s*\^?\s*(\d?)([+-])$").captures(&p) {
        let k: i32 = if m[2].is_empty() { 1 } else { m[2].parse().ok()? };
        let f = m[1].to_string();
        parse_formula(&f)?;
        return Some(Species { coef, formula: f, charge: if &m[3] == "+" { k } else { -k }, state });
    }
    let lower = p.to_lowercase();
    let lower = lower.trim_end_matches(" ion");
    if let Some(i) = IONS.iter().find(|i| i.0 == lower) {
        return Some(Species { coef, formula: i.1.to_string(), charge: i.2, state });
    }
    if lower == "hydrogen ion" || lower == "proton" {
        return Some(Species { coef, formula: "H".into(), charge: 1, state });
    }
    if lower == "nitronium" {
        return Some(Species { coef, formula: "NO2".into(), charge: 1, state });
    }
    // a formula as typed ("CuCl2", "BaSO4"), not its database spelling
    if p.starts_with(|c: char| c.is_ascii_uppercase()) && !p.contains(' ') && parse_formula(&p).is_some() && (p.len() > 2 || p.chars().any(|c| c.is_ascii_digit()) || by_symbol(&p).is_some()) {
        return Some(Species { coef, formula: p.clone(), charge: 0, state });
    }
    let (_, f, _) = substance(&p).or_else(|| {
        let c = super::chem::find_db_compound(&p)?;
        Some((String::new(), c.formula.to_string(), parse_formula(c.formula)?))
    }).or_else(|| {
        let f = ionic_formula(&p)?;
        Some((String::new(), f.clone(), parse_formula(&f)?))
    })?;
    // the common diatomic elements as gases
    let f = if matches!(f.as_str(), "H" | "N" | "O" | "F" | "Cl" | "Br" | "I") && !p.chars().next()?.is_ascii_uppercase() { format!("{f}2") } else { f };
    Some(Species { coef, formula: f, charge: 0, state })
}

fn parse_equation(body: &str) -> Option<(Vec<Species>, Vec<Species>)> {
    let arrow = ["-->", "->", "=>", "→", "⟶", "⇌", "<=>", " yields ", " = "].iter().find(|a| body.contains(**a))?;
    let (l, r) = body.split_once(arrow)?;
    let left: Vec<Species> = split_plus(l).iter().map(|p| parse_species(p)).collect::<Option<_>>()?;
    let right: Vec<Species> = split_plus(r).iter().map(|p| parse_species(p)).collect::<Option<_>>()?;
    (!left.is_empty() && !right.is_empty()).then_some((left, right))
}

/// Balance with the written coefficients when they already balance.
fn balanced(left: &mut [Species], right: &mut [Species]) -> Option<()> {
    let counts = |s: &Species| parse_formula(&s.formula);
    let given_ok = left.iter().chain(right.iter()).all(|s| s.coef.is_some()) && {
        let mut bal: BTreeMap<String, i64> = BTreeMap::new();
        for (k, s) in left.iter().enumerate().map(|(i, s)| (i, s)).chain(right.iter().enumerate().map(|(i, s)| (i + 1000, s))) {
            let sign = if k < 1000 { 1 } else { -1 };
            if let Some(c) = parse_formula(&s.formula) {
                for (e, n) in c {
                    *bal.entry(e).or_default() += sign * s.coef.unwrap() * n as i64;
                }
            }
            *bal.entry("charge".into()).or_default() += sign * s.coef.unwrap() * s.charge as i64;
        }
        bal.values().all(|v| *v == 0)
    };
    if given_ok {
        return Some(());
    }
    let lc: Vec<_> = left.iter().map(counts).collect::<Option<_>>()?;
    let rc: Vec<_> = right.iter().map(counts).collect::<Option<_>>()?;
    // charges must balance as net charge, so fold "+" and "-" into one row
    let has_charge = left.iter().chain(right.iter()).any(|s| s.charge != 0);
    let coef = if has_charge {
        // solve with the net charge as an extra conservation law by brute force
        // over small coefficients (ionic equations are tiny)
        let n = left.len() + right.len();
        if n > 6 {
            return None;
        }
        let all: Vec<&Species> = left.iter().chain(right.iter()).collect();
        let (lf, rf) = (&lc, &rc);
        let mut best = None;
        let mut c = vec![1i64; n];
        'search: loop {
            let mut bal: BTreeMap<String, i64> = BTreeMap::new();
            for (k, cnt) in lf.iter().chain(rf.iter()).enumerate() {
                let sign = if k < left.len() { 1 } else { -1 };
                for (e, v) in cnt {
                    *bal.entry(e.clone()).or_default() += sign * c[k] * *v as i64;
                }
                *bal.entry("charge".into()).or_default() += sign * c[k] * all[k].charge as i64;
            }
            if bal.values().all(|v| *v == 0) {
                best = Some(c.iter().map(|x| *x as i128).collect::<Vec<_>>());
                break;
            }
            for i in 0..n {
                c[i] += 1;
                if c[i] <= 8 {
                    continue 'search;
                }
                c[i] = 1;
            }
            break;
        }
        best?
    } else {
        coefficients(&lc, &rc)?
    };
    for (s, k) in left.iter_mut().chain(right.iter_mut()).zip(coef) {
        s.coef = Some(k as i64);
    }
    Some(())
}

fn show_side(v: &[Species], states: bool) -> String {
    v.iter()
        .map(|s| {
            let k = s.coef.unwrap_or(1);
            let st = if states { s.state.as_ref().map(|x| format!("({x})")).unwrap_or_default() } else { String::new() };
            format!("{}{}{st}", if k == 1 { String::new() } else { format!("{k} ") }, ion_text(&s.formula, s.charge))
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Dissociate strong electrolytes in solution.
fn ionize(side: &[Species]) -> Vec<Species> {
    let mut out: Vec<Species> = Vec::new();
    let mut push = |f: String, z: i32, k: i64, state: Option<String>| {
        if let Some(e) = out.iter_mut().find(|s| s.formula == f && s.charge == z && s.state == state) {
            e.coef = Some(e.coef.unwrap_or(0) + k);
        } else {
            out.push(Species { coef: Some(k), formula: f, charge: z, state });
        }
    };
    for s in side {
        let k = s.coef.unwrap_or(1);
        let ionic = if s.charge == 0 { decompose(&s.formula) } else { None };
        let dissolved = match (&ionic, s.state.as_deref()) {
            (Some(i), Some("aq")) => strong_electrolyte(i),
            (Some(i), None) => strong_electrolyte(i) && soluble(i),
            _ => false,
        };
        if let (true, Some(i)) = (dissolved, ionic) {
            push(i.cat.clone(), i.zc, k * i.nc as i64, Some("aq".into()));
            push(i.an.clone(), i.za, k * i.na as i64, Some("aq".into()));
        } else {
            let state = s.state.clone().or_else(|| if decompose(&s.formula).is_some() { Some("s".into()) } else { None });
            push(s.formula.clone(), s.charge, k, state);
        }
    }
    out
}

fn ionic_equation(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches('?').trim();
    let m = re!(r"(?i)^(?:what\s+(?:is|are)\s+|show\s+me\s+|find\s+|give\s+me\s+|write\s+)?(?:the\s+)?(total ionic|complete ionic|full ionic|net ionic|ionic|spectator ions?)\s*(?:equation|reaction)?s?\s*(?:for|of|in)?\s*:?\s*(.+)$").captures(c).or_else(|| re!(r"(?i)^(.+?)\s+(total ionic|complete ionic|net ionic|spectator ions?)\s*(?:equation|reaction)?$").captures(c).map(|m| m))?;
    let (kind, body) = if m[1].to_lowercase().contains("ionic") || m[1].to_lowercase().starts_with("spectator") { (m[1].to_lowercase(), m[2].to_string()) } else { (m[2].to_lowercase(), m[1].to_string()) };
    let (mut left, mut right) = parse_equation(&body)?;
    balanced(&mut left, &mut right)?;
    let molecular = format!("{} → {}", show_side(&left, true), show_side(&right, true));
    let (tl, tr) = (ionize(&left), ionize(&right));
    let total = format!("{} → {}", show_side(&tl, true), show_side(&tr, true));
    // cancel spectators
    let mut nl = tl.clone();
    let mut nr = tr.clone();
    let mut spectators = Vec::new();
    for a in nl.iter_mut() {
        if a.charge == 0 {
            continue;
        }
        if let Some(b) = nr.iter_mut().find(|b| b.formula == a.formula && b.charge == a.charge) {
            let k = a.coef.unwrap().min(b.coef.unwrap());
            if k > 0 {
                spectators.push(ion_text(&a.formula, a.charge));
            }
            a.coef = Some(a.coef.unwrap() - k);
            b.coef = Some(b.coef.unwrap() - k);
        }
    }
    nl.retain(|s| s.coef.unwrap() > 0);
    nr.retain(|s| s.coef.unwrap() > 0);
    let g = nl.iter().chain(nr.iter()).fold(0, |g, s| gcd(g, s.coef.unwrap()));
    if g > 1 {
        for s in nl.iter_mut().chain(nr.iter_mut()) {
            s.coef = Some(s.coef.unwrap() / g);
        }
    }
    let net = if nl.is_empty() || nr.is_empty() { "no net reaction: every species is a spectator".to_string() } else { format!("{} → {}", show_side(&nl, true), show_side(&nr, true)) };
    let spect = if spectators.is_empty() { "none".to_string() } else { spectators.join(", ") };
    let (title, main) = if kind.starts_with("net") {
        ("Net ionic equation", net.clone())
    } else if kind.starts_with("spectator") {
        ("Spectator ions", spect.clone())
    } else {
        ("Total ionic equation", total.clone())
    };
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{molecular} | {}", title.to_lowercase()))), primary(pod("Result", title, text(main)))];
    pods.push(pod("Molecular", "Balanced molecular equation", text(molecular)));
    if !title.starts_with("Total") {
        pods.push(pod("Total", "Total ionic equation", text(total)));
    }
    if !title.starts_with("Net") {
        pods.push(pod("Net", "Net ionic equation", text(net)));
    }
    if !title.starts_with("Spectator") {
        pods.push(pod("Spectators", "Spectator ions", text(spect)));
    }
    pods.push(pod("Notes", "Notes", text("Strong acids, strong bases and soluble salts in solution are written as ions; weak electrolytes, solids, liquids and gases stay as written. States not given are filled in from the usual solubility rules.")));
    Some(Answer::new(title.to_lowercase(), pods))
}

// ---------------------------------------------------------------------------
// equilibrium constants and reaction quotients
// ---------------------------------------------------------------------------

fn equilibrium(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches('?').trim();
    let m = re!(r"(?i)^(?:(?:what\s+is\s+)?(?:the\s+)?(equilibrium constant|equilibrium expression|reaction quotient)\s*(?:expression)?\s*(?:for|of)?\s*:?\s*(.+)|(.+?)\s+(equilibrium constant|equilibrium expression|reaction quotient)(?:\s+expression)?)$").captures(c)?;
    let (kind, body) = if let Some(k) = m.get(1) { (k.as_str().to_lowercase(), m[2].to_string()) } else { (m[4].to_lowercase(), m[3].to_string()) };
    let (mut left, mut right) = parse_equation(&body)?;
    balanced(&mut left, &mut right)?;
    let term = |s: &Species| -> Option<String> {
        if matches!(s.state.as_deref(), Some("s") | Some("l")) {
            return None;
        }
        let k = s.coef.unwrap_or(1);
        let base = format!("[{}]", ion_text(&s.formula, s.charge));
        Some(if k == 1 { base } else { format!("{base}^{k}") })
    };
    let num: Vec<String> = right.iter().filter_map(term).collect();
    let den: Vec<String> = left.iter().filter_map(term).collect();
    let sym = if kind.contains("quotient") || kind == "q" { "Q_c" } else { "K_c" };
    let wrap = |v: &[String]| if v.is_empty() { "1".to_string() } else { v.join(" ") };
    let paren = |v: &[String]| if v.len() == 1 { v[0].clone() } else { format!("({})", v.join(" ")) };
    let expr = if den.is_empty() { format!("{sym} = {}", wrap(&num)) } else { format!("{sym} = {} / {}", wrap(&num), paren(&den)) };
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{} → {} | {}", show_side(&left, true), show_side(&right, true), if sym == "Q_c" { "reaction quotient" } else { "equilibrium constant" }))),
        primary(pod("Result", "Expression", text(expr))),
        pod("Balanced", "Balanced equation", text(format!("{} → {}", show_side(&left, true), show_side(&right, true)))),
    ];
    let gases_l: Vec<&Species> = left.iter().filter(|s| s.state.as_deref() == Some("g")).collect();
    let gases_r: Vec<&Species> = right.iter().filter(|s| s.state.as_deref() == Some("g")).collect();
    if !gases_l.is_empty() || !gases_r.is_empty() {
        let p = |s: &&Species| {
            let k = s.coef.unwrap_or(1);
            let base = format!("p({})", pretty(&s.formula));
            if k == 1 { base } else { format!("{base}^{k}") }
        };
        let num: Vec<String> = gases_r.iter().map(p).collect();
        let den: Vec<String> = gases_l.iter().map(p).collect();
        let dn: i64 = gases_r.iter().map(|s| s.coef.unwrap_or(1)).sum::<i64>() - gases_l.iter().map(|s| s.coef.unwrap_or(1)).sum::<i64>();
        let kp = if sym == "Q_c" { "Q_p" } else { "K_p" };
        let e = if den.is_empty() { format!("{kp} = {}", wrap(&num)) } else { format!("{kp} = {} / {}", wrap(&num), paren(&den)) };
        pods.push(pod("Kp", "Pressure form", text(format!("{e}\n{kp} = {} (RT)^{dn}", if sym == "Q_c" { "Q_c" } else { "K_c" }))));
    }
    pods.push(pod("Notes", "Notes", text("Pure solids and liquids are left out of the expression (their activity is 1).")));
    Some(Answer::new(format!("{sym} expression"), pods))
}

// ---------------------------------------------------------------------------
// combustion
// ---------------------------------------------------------------------------

fn combustion(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?');
    let m = re!(r"^(?:balanced\s+)?(?:equation\s+for\s+(?:the\s+)?)?(?:complete\s+)?(?:combustion|burning|oxidation)\s+(?:reaction\s+)?(?:of|for)\s+(.+)$").captures(l).or_else(|| re!(r"^(.+?)\s+(?:complete\s+)?(?:combustion|burning)(?:\s+(?:reaction|equation))?$").captures(l))?;
    let name = m[1].trim();
    let raw = &q.clean[q.clean.len() - name.len().min(q.clean.len())..];
    let (label, f, counts) = substance(raw.trim()).or_else(|| substance(name)).or_else(|| {
        let c = super::chem::find_db_compound(name)?;
        Some((format!("{name} ({})", pretty(c.formula)), c.formula.to_string(), parse_formula(c.formula)?))
    })?;
    if !counts.contains_key("C") && !counts.contains_key("H") && counts.len() > 1 {
        return None;
    }
    // metals and other elements burn to their usual oxides
    let mut products: Vec<String> = Vec::new();
    for e in counts.keys() {
        let p = match e.as_str() {
            "C" => "CO2",
            "H" => "H2O",
            "N" => "N2",
            "S" => "SO2",
            "O" => continue,
            "Mg" => "MgO",
            "Ca" => "CaO",
            "Fe" => "Fe2O3",
            "Al" => "Al2O3",
            "Cu" => "CuO",
            "Zn" => "ZnO",
            "Na" => "Na2O",
            "K" => "K2O",
            "Li" => "Li2O",
            "P" => "P4O10",
            "Si" => "SiO2",
            _ => return None,
        };
        products.push(p.to_string());
    }
    if products.is_empty() {
        return None;
    }
    let lc = vec![counts.clone(), parse_formula("O2")?];
    let rc: Vec<_> = products.iter().map(|p| parse_formula(p)).collect::<Option<_>>()?;
    let k = coefficients(&lc, &rc)?;
    let fmt = |c: i128, f: &str| if c == 1 { pretty(f) } else { format!("{c} {}", pretty(f)) };
    let eq = format!("{} + {} → {}", fmt(k[0], &f), fmt(k[1], "O2"), products.iter().enumerate().map(|(i, p)| fmt(k[2 + i], p)).collect::<Vec<_>>().join(" + "));
    let mm = super::chem::molar_mass(&counts);
    let o2 = k[1] as f64 / k[0] as f64;
    let rows = vec![
        vec![text("O₂ per mole of fuel"), text(format!("{} mol", fmt_float(o2, 6)))],
        vec![text("O₂ per gram of fuel"), text(format!("{} g", fmt_float(o2 * 31.998 / mm, 6)))],
    ];
    Some(Answer::new(
        format!("combustion of {label}"),
        vec![pod("Input", "Input interpretation", text(format!("complete combustion | {label}"))), primary(pod("Result", "Balanced equation", text(eq))), pod("Oxygen", "Oxygen needed", table(None, rows))],
    ))
}

// ---------------------------------------------------------------------------
// empirical formula from masses or percentages
// ---------------------------------------------------------------------------

fn empirical(q: &Query) -> Option<Answer> {
    let c = q.clean.trim();
    let l = c.to_lowercase();
    let pairs: Vec<(f64, &str, String)> = re!(r"(\d+(?:\.\d+)?)\s*(%|percent|grams?|g|mg|kg)?\s*(?:of\s+)?([A-Z][a-z]?\b|[a-z]{3,})").captures_iter(c).filter_map(|m| {
        let v: f64 = m[1].parse().ok()?;
        let e = find_element(&m[3]).or_else(|| if m[3].len() <= 2 { by_symbol(&m[3]) } else { None })?;
        Some((v, m.get(2).map_or("", |u| u.as_str()), e.symbol.clone()))
    }).collect();
    if pairs.len() < 2 || pairs.iter().map(|p| &p.2).collect::<std::collections::BTreeSet<_>>().len() != pairs.len() {
        return None;
    }
    let explicit = l.contains("empirical") || l.contains("simplest formula");
    // without the word, accept "7.31 grams C + 0.737 grams H + 1.95 grams O"
    if !explicit && !(pairs.iter().all(|p| !p.1.is_empty()) && re!(r"^[\d.\s%a-zA-Z+,]+$").is_match(c) && (c.contains('+') || c.contains(','))) {
        return None;
    }
    let moles: Vec<(String, f64)> = pairs
        .iter()
        .map(|(v, u, s)| {
            let scale = match *u {
                "mg" => 1e-3,
                "kg" => 1e3,
                _ => 1.0,
            };
            (s.clone(), v * scale / by_symbol(s).map_or(1.0, |e| e.mass))
        })
        .collect();
    let min = moles.iter().map(|m| m.1).fold(f64::INFINITY, f64::min);
    let ratios: Vec<f64> = moles.iter().map(|m| m.1 / min).collect();
    let mult = (1..=12).find(|k| ratios.iter().all(|r| (r * *k as f64 - (r * *k as f64).round()).abs() < 0.12))?;
    let ints: Vec<u32> = ratios.iter().map(|r| (r * mult as f64).round() as u32).collect();
    // Hill order: C, H, then alphabetical
    let mut items: Vec<(String, u32)> = moles.iter().map(|m| m.0.clone()).zip(ints.iter().copied()).collect();
    let has_c = items.iter().any(|i| i.0 == "C");
    items.sort_by_key(|(s, _)| if has_c && s == "C" { (0, s.clone()) } else if has_c && s == "H" { (1, s.clone()) } else { (2, s.clone()) });
    let formula: String = items.iter().map(|(s, n)| if *n == 1 { s.clone() } else { format!("{s}{n}") }).collect();
    let counts = parse_formula(&formula)?;
    let rows = moles.iter().zip(ratios.iter()).map(|((s, mol), r)| vec![text(s.clone()), text(format!("{} mol", fmt_float(*mol, 5))), text(fmt_float(*r, 4)), text(fmt_float(r * mult as f64, 4))]).collect();
    Some(Answer::new(
        format!("empirical formula {formula}"),
        vec![
            pod("Input", "Input interpretation", text(format!("empirical formula | {}", pairs.iter().map(|(v, u, s)| format!("{v} {u} {s}")).collect::<Vec<_>>().join(", ")))),
            primary(pod("Result", "Empirical formula", text(pretty(&formula)))),
            pod("Ratios", "Mole ratios", table(Some(vec!["element", "amount", "ratio", "scaled"]), rows)),
            pod("Mass", "Empirical formula mass", text(format!("{} g/mol", fmt_float(super::chem::molar_mass(&counts), 6)))),
        ],
    ))
}

// ---------------------------------------------------------------------------
// ionic compound names
// ---------------------------------------------------------------------------

const MONO_ANIONS: &[(&str, &str)] = &[("hydride", "H"), ("carbide", "C"), ("nitride", "N"), ("oxide", "O"), ("fluoride", "F"), ("phosphide", "P"), ("sulfide", "S"), ("sulphide", "S"), ("chloride", "Cl"), ("selenide", "Se"), ("bromide", "Br"), ("telluride", "Te"), ("iodide", "I"), ("arsenide", "As")];
const ORGANIC_ANIONS: &[(&str, &str, i32)] = &[("ethoxide", "C2H5O", -1), ("ethanolate", "C2H5O", -1), ("methoxide", "CH3O", -1), ("methanolate", "CH3O", -1), ("benzoate", "C6H5COO", -1), ("propanoate", "C2H5COO", -1), ("propionate", "C2H5COO", -1), ("butanoate", "C3H7COO", -1), ("butyrate", "C3H7COO", -1), ("stearate", "C17H35COO", -1), ("lactate", "C3H5O3", -1), ("tartrate", "C4H4O6", -2), ("succinate", "C4H4O4", -2), ("ferricyanide", "Fe(CN)6", -3), ("ferrocyanide", "Fe(CN)6", -4)];

/// "calcium hydrogen phosphate" → CaHPO4, "silver I selenide" → Ag2Se.
pub(super) fn ionic_formula(name: &str) -> Option<String> {
    let l = name.trim().to_lowercase().replace("  ", " ");
    let l = l.trim_end_matches(" hydrate").trim();
    let words: Vec<&str> = l.split_whitespace().collect();
    for k in 1..words.len() {
        let (cat_s, an_s) = (words[..k].join(" "), words[k..].join(" "));
        let (cat, zc): (String, i32) = if cat_s == "ammonium" {
            ("NH4".into(), 1)
        } else if let Some(c) = named_cation(&cat_s) {
            c
        } else if let Some(e) = find_element(&cat_s).filter(|e| is_metal(&e.symbol)) {
            let z = match e.group {
                Some(1) => 1,
                Some(2) => 2,
                _ => match e.symbol.as_str() {
                    "Al" | "Ga" | "Sc" | "Y" | "La" => 3,
                    "Zn" | "Cd" => 2,
                    "Ag" => 1,
                    _ => {
                        let ox: Vec<i32> = e.oxidation_states.clone().unwrap_or_default().into_iter().filter(|z| *z > 0).collect();
                        if ox.len() == 1 { ox[0] } else { continue }
                    }
                },
            };
            (e.symbol.clone(), z)
        } else {
            continue;
        };
        let (an, za): (String, i32) = if let Some(i) = IONS.iter().find(|i| i.0 == an_s && i.2 < 0) {
            (i.1.to_string(), i.2)
        } else if let Some((_, s)) = MONO_ANIONS.iter().find(|(n, _)| *n == an_s) {
            (s.to_string(), mono_anion(s)?)
        } else if let Some(o) = ORGANIC_ANIONS.iter().find(|o| o.0 == an_s) {
            (o.1.to_string(), o.2)
        } else {
            continue;
        };
        let g = gcd(zc as i64, -za as i64) as i32;
        let (nc, na) = ((-za) / g, zc / g);
        let poly = |f: &str| f.chars().filter(|c| c.is_ascii_uppercase()).count() > 1 || f.chars().any(|c| c.is_ascii_digit());
        let part = |f: &str, n: i32| if n == 1 { f.to_string() } else if poly(f) { format!("({f}){n}") } else { format!("{f}{n}") };
        return Some(format!("{}{}", part(&cat, nc), part(&an, na)));
    }
    None
}

fn ionic_name(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?');
    let l = l.trim_start_matches("formula of ").trim_start_matches("formula for ").trim_start_matches("chemical formula of ").trim_start_matches("chemical formula for ").trim_end_matches(" formula").trim_end_matches(" chemical formula");
    if super::chem::compound_formula(l).is_some() || super::chem::find_db_compound(l).is_some() {
        return None;
    }
    let f = ionic_formula(l)?;
    let counts = parse_formula(&f)?;
    let label = format!("{l} ({})", pretty(&f));
    let mut pods = compound_pods(&label, &f, &counts);
    for p in pods.iter_mut() {
        p.primary = false;
    }
    pods.retain(|p| p.id != "Formula");
    pods.insert(1, primary(pod("Formula", "Chemical formula", text(pretty(&f)))));
    let i = decompose(&f);
    if let Some(i) = i {
        pods.push(pod("Ions", "Ions", text(format!("{} {} + {} {}", i.nc, ion_text(&i.cat, i.zc), i.na, ion_text(&i.an, i.za)))));
        pods.push(pod("Solubility", "Solubility in water (rules of thumb)", text(if soluble(&i) { "soluble" } else { "insoluble or sparingly soluble" })));
    }
    Some(Answer::new(label, pods))
}

/// "chromium(III)", "lead(2) ion"
fn named_ion(q: &Query) -> Option<Answer> {
    let (s, z) = named_cation(q.lower.trim())?;
    let e = by_symbol(&s)?;
    let electrons = e.z as i32 - z;
    let rows = vec![vec![text("formula"), text(ion_text(&s, z))], vec![text("charge"), text(format!("+{z}"))], vec![text("electrons"), text(electrons.to_string())], vec![text("molar mass"), text(format!("{} g/mol", fmt_float(e.mass, 6)))]];
    Some(Answer::new(
        format!("{}({z}+) ion", e.name),
        vec![pod("Input", "Input interpretation", text(format!("{} ion, charge +{z}", e.name.to_lowercase()))), primary(pod("Result", "Ion", text(ion_text(&s, z)))), pod("Properties", "Properties", table(None, rows))],
    ))
}

// ---------------------------------------------------------------------------
// composition and comparisons
// ---------------------------------------------------------------------------

fn resolve(name: &str) -> Option<(String, String, BTreeMap<String, u32>)> {
    substance(name).or_else(|| {
        let c = super::chem::find_db_compound(name)?;
        Some((format!("{} ({})", c.name, pretty(c.formula)), c.formula.to_string(), parse_formula(c.formula)?))
    }).or_else(|| {
        let f = ionic_formula(name)?;
        Some((format!("{} ({})", name.trim(), pretty(&f)), f.clone(), parse_formula(&f)?))
    })
}

fn composition(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?');
    let m = re!(r"^(?:(?:mass|elemental|element|percent|percentage|chemical|atomic)\s+)?composition\s+(?:of\s+)?(.+)$").captures(l).or_else(|| re!(r"^(.+?)\s+(?:(?:mass|elemental|element|percent|percentage|atomic)\s+)?composition$").captures(l))?;
    let name = m[1].trim();
    let raw = q.clean.trim().trim_end_matches('?');
    let raw_name = raw.get(raw.to_lowercase().find(name)?..)?.get(..name.len())?;
    let (label, f, counts) = resolve(raw_name).or_else(|| resolve(name))?;
    if counts.len() < 2 {
        return None;
    }
    let mut pods = compound_pods(&label, &f, &counts);
    for p in pods.iter_mut() {
        p.primary = p.id == "Composition";
    }
    pods.retain(|p| p.id != "Result");
    Some(Answer::new(format!("composition of {label}"), pods))
}

fn compare_compounds(q: &Query) -> Option<Answer> {
    let raw = q.clean.trim().trim_end_matches('?');
    let parts: Vec<&str> = if raw.to_lowercase().contains(" vs ") || raw.to_lowercase().contains(" versus ") {
        re!(r"(?i)\s+(?:vs\.?|versus)\s+").split(raw).collect()
    } else if raw.contains(", ") {
        raw.split(", ").flat_map(|p| p.split(" and ")).collect()
    } else {
        return None;
    };
    if parts.len() < 2 || parts.len() > 6 {
        return None;
    }
    // "CBS, NBC, ABC" are acronyms, not formulas
    if parts.iter().any(|p| p.trim().chars().all(|c| c.is_ascii_uppercase())) {
        return None;
    }
    let items: Vec<_> = parts.iter().map(|p| resolve(p.trim()).filter(|r| r.2.len() >= 2).map(|r| (p.trim(), r))).collect::<Option<_>>()?;
    let n = |v: Option<f64>, u: &str| v.map_or("—".to_string(), |x| format!("{} {u}", fmt_float(x, 5)));
    let mut rows: Vec<Vec<Content>> = Vec::new();
    rows.push(std::iter::once(text("formula")).chain(items.iter().map(|(_, r)| text(pretty(&r.1)))).collect());
    rows.push(std::iter::once(text("molar mass")).chain(items.iter().map(|(_, r)| text(format!("{} g/mol", fmt_float(super::chem::molar_mass(&r.2), 6))))).collect());
    let subs: Vec<_> = items.iter().map(|(p, _)| find_substance(p)).collect();
    rows.push(std::iter::once(text("density")).chain(subs.iter().map(|s| text(n(s.and_then(|s| s.density), "g/cm³")))).collect());
    rows.push(std::iter::once(text("boiling point")).chain(subs.iter().map(|s| text(n(s.and_then(|s| s.bp), "°C")))).collect());
    let names: Vec<String> = items.iter().map(|(p, _)| p.to_string()).collect();
    let header = std::iter::once("").chain(names.iter().map(String::as_str)).collect();
    Some(Answer::new(names.join(" | "), vec![pod("Input", "Input interpretation", text(names.join("  |  "))), primary(pod("Result", "Comparison", table(Some(header), rows)))]))
}

// ---------------------------------------------------------------------------
// pH indicators
// ---------------------------------------------------------------------------

/// (name, low pH, high pH, acid colour, base colour)
const INDICATORS: &[(&str, f64, f64, &str, &str)] = &[
    ("methyl violet", 0.0, 1.6, "yellow", "blue-violet"),
    ("thymol blue (acid range)", 1.2, 2.8, "red", "yellow"),
    ("bromophenol blue", 3.0, 4.6, "yellow", "blue"),
    ("congo red", 3.0, 5.0, "blue-violet", "red"),
    ("methyl orange", 3.1, 4.4, "red", "yellow"),
    ("bromocresol green", 3.8, 5.4, "yellow", "blue"),
    ("methyl red", 4.4, 6.2, "red", "yellow"),
    ("litmus", 4.5, 8.3, "red", "blue"),
    ("chlorophenol red", 4.8, 6.4, "yellow", "red"),
    ("bromocresol purple", 5.2, 6.8, "yellow", "purple"),
    ("bromothymol blue", 6.0, 7.6, "yellow", "blue"),
    ("phenol red", 6.4, 8.0, "yellow", "red"),
    ("neutral red", 6.8, 8.0, "red", "yellow"),
    ("cresol red", 7.2, 8.8, "yellow", "reddish purple"),
    ("thymol blue (base range)", 8.0, 9.6, "yellow", "blue"),
    ("phenolphthalein", 8.2, 10.0, "colourless", "pink"),
    ("thymolphthalein", 9.3, 10.5, "colourless", "blue"),
    ("alizarin yellow R", 10.1, 12.0, "yellow", "red"),
    ("indigo carmine", 11.4, 13.0, "blue", "yellow"),
];

fn indicators(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?');
    let rows_for = |list: Vec<&(&str, f64, f64, &str, &str)>| -> Vec<Vec<Content>> { list.iter().map(|(n, a, b, ca, cb)| vec![text(n.to_string()), text(format!("{a} – {b}")), text(format!("{ca} → {cb}"))]).collect() };
    let header = || Some(vec!["indicator", "transition pH", "colour change"]);
    if let Some(m) = re!(r"indicators?\b.*\bph\s*(?:of\s*|=\s*)?(\d+(?:\.\d+)?)").captures(l).or_else(|| re!(r"\bph\s*(\d+(?:\.\d+)?)\b.*\bindicators?\b").captures(l)) {
        let x: f64 = m[1].parse().ok()?;
        let list: Vec<_> = INDICATORS.iter().filter(|i| i.1 - 0.3 <= x && x <= i.2 + 0.3).collect();
        if list.is_empty() {
            return None;
        }
        return Some(Answer::new(format!("indicators for pH {x}"), vec![pod("Input", "Input interpretation", text(format!("acid–base indicators | transition near pH {x}"))), primary(pod("Result", "Indicators", table(header(), rows_for(list))))]));
    }
    if re!(r"^(?:list of |table of |common )?(?:acid[- ]base |ph )indicators?$").is_match(l) {
        return Some(Answer::new("pH indicators", vec![pod("Input", "Input interpretation", text("acid–base indicators")), primary(pod("Result", "Indicators", table(header(), rows_for(INDICATORS.iter().collect()))))]));
    }
    let m = re!(r"^(?:(?:ph|transition|colou?r change)\s+(?:range|interval)\s+(?:of\s+)?(.+)|(.+?)\s+(?:ph\s+range|transition range|colou?r change|colou?r range|indicator range|ph indicator|indicator))$").captures(l)?;
    let name = m.get(1).or(m.get(2))?.as_str().trim();
    let list: Vec<_> = INDICATORS.iter().filter(|i| i.0 == name || i.0.starts_with(&format!("{name} ("))).collect();
    if list.is_empty() {
        return None;
    }
    Some(Answer::new(format!("{name} indicator"), vec![pod("Input", "Input interpretation", text(format!("{name} | acid–base indicator"))), primary(pod("Result", "Transition", table(header(), rows_for(list))))]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    ionic_equation(q)
        .or_else(|| equilibrium(q))
        .or_else(|| combustion(q))
        .or_else(|| empirical(q))
        .or_else(|| indicators(q))
        .or_else(|| composition(q))
        .or_else(|| named_ion(q))
        .or_else(|| ionic_name(q))
        .or_else(|| compare_compounds(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(s: &str) -> String {
        crate::engine::query(s).short_answer().unwrap_or_default()
    }

    #[test]
    fn reactions() {
        assert_eq!(ionic_formula("calcium hydrogen phosphate").as_deref(), Some("CaHPO4"));
        assert_eq!(ionic_formula("silver I selenide").as_deref(), Some("Ag2Se"));
        assert_eq!(ionic_formula("aluminum sulfate").as_deref(), Some("Al2(SO4)3"));
        assert!(ask("net ionic reaction Hg2(NO3)2(aq) + KI(aq) -> Hg2I2(s) + KNO3(aq)").contains("Hg₂²⁺(aq) + 2 I⁻(aq) → Hg₂I₂(s)"));
        assert_eq!(ask("spectator ions Zn(s) + CuSO4(aq) -> ZnSO4(aq) + Cu(s)"), "SO₄²⁻");
        assert!(ask("Show me the net ionic reaction for Ba(OH)2(aq) + 2HNO2(aq) -> Ba(NO2)2(aq) + H2O(l)").contains("OH⁻(aq) + HNO₂(aq) → NO₂⁻(aq) + H₂O(l)"));
        assert_eq!(ask("equilibrium constant CoO(s) + CO(g) -> Co(s) + CO2(g)"), "K_c = [CO₂] / [CO]");
        assert_eq!(ask("7.31 grams C + 0.737 grams H + 1.95 grams O"), "C₅H₆O");
        assert_eq!(ask("combustion of propane"), "C₃H₈ + 5 O₂ → 3 CO₂ + 4 H₂O");
    }
}
