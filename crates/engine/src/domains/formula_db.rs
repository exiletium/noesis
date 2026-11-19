//! A table of named formulas (data/formulas.txt) and a generic solver:
//! "ideal gas law 2.2mol, 2.0atm, 500K", "lens, f=20ft, o=400ft",
//! "Rydberg formula ni=2, nf=5", "Snell's law ... angle 45".
//!
//! Given quantities are matched to the formula's variables by explicit
//! name ("f=20ft"), by a descriptive phrase ("focal length of 2m"), by a
//! particle name ("electron") or by physical dimension, in that order.
//! Equations are then solved numerically, one unknown at a time, until
//! nothing more can be computed.  With no values the formula itself and
//! its variables are shown.

use super::units::{self, Dim};
use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

// ---------------------------------------------------------------------------
// a small expression evaluator (ASCII identifiers, + - * / ^, functions)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Node {
    Num(f64),
    Var(String),
    Neg(Box<Node>),
    Bin(char, Box<Node>, Box<Node>),
    Call(String, Vec<Node>),
}

struct P<'a> {
    t: Vec<&'a str>,
    i: usize,
}

fn tokens(s: &str) -> Vec<&str> {
    re!(r"\d+\.?\d*(?:[eE][-+]?\d+)?|\.\d+(?:[eE][-+]?\d+)?|[A-Za-z_][A-Za-z0-9_]*|[-+*/^(),]").find_iter(s).map(|m| m.as_str()).collect()
}

impl<'a> P<'a> {
    fn peek(&self) -> Option<&'a str> {
        self.t.get(self.i).copied()
    }
    fn next(&mut self) -> Option<&'a str> {
        self.i += 1;
        self.t.get(self.i - 1).copied()
    }
    fn expr(&mut self) -> Option<Node> {
        let mut a = self.term()?;
        while let Some(op @ ("+" | "-")) = self.peek() {
            self.next();
            a = Node::Bin(op.chars().next()?, Box::new(a), Box::new(self.term()?));
        }
        Some(a)
    }
    fn term(&mut self) -> Option<Node> {
        let mut a = self.unary()?;
        while let Some(op @ ("*" | "/")) = self.peek() {
            self.next();
            a = Node::Bin(op.chars().next()?, Box::new(a), Box::new(self.unary()?));
        }
        Some(a)
    }
    fn unary(&mut self) -> Option<Node> {
        if self.peek() == Some("-") {
            self.next();
            return Some(Node::Neg(Box::new(self.unary()?)));
        }
        if self.peek() == Some("+") {
            self.next();
        }
        self.power()
    }
    fn power(&mut self) -> Option<Node> {
        let a = self.atom()?;
        if self.peek() == Some("^") {
            self.next();
            return Some(Node::Bin('^', Box::new(a), Box::new(self.unary()?)));
        }
        Some(a)
    }
    fn atom(&mut self) -> Option<Node> {
        let t = self.next()?;
        if t == "(" {
            let e = self.expr()?;
            (self.next()? == ")").then_some(e)
        } else if t.chars().next()?.is_ascii_digit() || t.starts_with('.') {
            t.parse().ok().map(Node::Num)
        } else if t.chars().next()?.is_ascii_alphabetic() || t.starts_with('_') {
            if self.peek() == Some("(") {
                self.next();
                let mut args = vec![self.expr()?];
                while self.peek() == Some(",") {
                    self.next();
                    args.push(self.expr()?);
                }
                (self.next()? == ")").then_some(Node::Call(t.to_string(), args))
            } else {
                Some(Node::Var(t.to_string()))
            }
        } else {
            None
        }
    }
}

fn parse_node(s: &str) -> Option<Node> {
    let mut p = P { t: tokens(s), i: 0 };
    let e = p.expr()?;
    (p.i == p.t.len()).then_some(e)
}

/// Physical constants available to every formula (SI units).
const CONSTANTS: &[(&str, f64)] = &[
    ("pi", std::f64::consts::PI),
    ("e", std::f64::consts::E),
    ("c", 299_792_458.0),
    ("h", 6.626_070_15e-34),
    ("hbar", 1.054_571_817e-34),
    ("G", 6.674_30e-11),
    ("kB", 1.380_649e-23),
    ("qe", 1.602_176_634e-19),
    ("me", 9.109_383_701_5e-31),
    ("mp", 1.672_621_923_69e-27),
    ("eps0", 8.854_187_812_8e-12),
    ("mu0", 1.256_637_062_12e-6),
    ("NA", 6.022_140_76e23),
    ("R", 8.314_462_618),
    ("Faraday", 96_485.332_12),
    ("sigma_SB", 5.670_374_419e-8),
    ("b_Wien", 2.897_771_955e-3),
    ("g0", 9.806_65),
    ("Msun", 1.988_47e30),
    ("Lsun", 3.828e26),
    ("Rsun", 6.957e8),
    ("Mearth", 5.9722e24),
    ("Rearth", 6.371e6),
    ("AU", 1.495_978_707e11),
    ("pc", 3.085_677_581_49e16),
    ("RH", 1.096_775_834e7),
    ("Rinf", 10_973_731.568_160),
    ("a0", 5.291_772_109_03e-11),
    ("alpha_fs", 7.297_352_569_3e-3),
    ("ke", 8.987_551_792_3e9),
    ("Ry_eV", 13.605_693_122_994),
    ("eV", 1.602_176_634e-19),
];

fn call(f: &str, a: &[f64]) -> f64 {
    let x = a.first().copied().unwrap_or(f64::NAN);
    match (f, a.len()) {
        ("sqrt", 1) => x.sqrt(),
        ("cbrt", 1) => x.cbrt(),
        ("exp", 1) => x.exp(),
        ("ln", 1) => x.ln(),
        ("log10", 1) | ("log", 1) => x.log10(),
        ("log2", 1) => x.log2(),
        ("sin", 1) => x.sin(),
        ("cos", 1) => x.cos(),
        ("tan", 1) => x.tan(),
        ("asin", 1) => x.asin(),
        ("acos", 1) => x.acos(),
        ("atan", 1) => x.atan(),
        ("sinh", 1) => x.sinh(),
        ("cosh", 1) => x.cosh(),
        ("tanh", 1) => x.tanh(),
        ("abs", 1) => x.abs(),
        ("atan2", 2) => x.atan2(a[1]),
        ("besselzero", 2) | ("besselzerod", 2) => {
            // n-th positive zero of J_m (or of J_m′), m = 0..5, n = 1..5
            const J: [[f64; 5]; 6] = [
                [2.404826, 5.520078, 8.653728, 11.791534, 14.930918],
                [3.831706, 7.015587, 10.173468, 13.323692, 16.470630],
                [5.135622, 8.417244, 11.619841, 14.795952, 17.959819],
                [6.380162, 9.761023, 13.015201, 16.223466, 19.409415],
                [7.588342, 11.064709, 14.372537, 17.615966, 20.826933],
                [8.771484, 12.338604, 15.700174, 18.980134, 22.217800],
            ];
            const JD: [[f64; 5]; 6] = [
                [3.831706, 7.015587, 10.173468, 13.323692, 16.470630],
                [1.841184, 5.331443, 8.536316, 11.706005, 14.863589],
                [3.054237, 6.706133, 9.969468, 13.170371, 16.347522],
                [4.201189, 8.015237, 11.345924, 14.585848, 17.788748],
                [5.317553, 9.282396, 12.681908, 15.964107, 19.196029],
                [6.415616, 10.519861, 13.987189, 17.312842, 20.575515],
            ];
            let (m, k) = (x.round(), a[1].round());
            if !(0.0..=5.0).contains(&m) || !(1.0..=5.0).contains(&k) {
                return f64::NAN;
            }
            (if f == "besselzero" { J } else { JD })[m as usize][k as usize - 1]
        }
        ("min", 2) => x.min(a[1]),
        ("max", 2) => x.max(a[1]),
        _ => f64::NAN,
    }
}

fn eval(n: &Node, vars: &HashMap<String, f64>) -> f64 {
    match n {
        Node::Num(v) => *v,
        Node::Var(s) => vars.get(s).copied().or_else(|| CONSTANTS.iter().find(|(k, _)| k == s).map(|(_, v)| *v)).unwrap_or(f64::NAN),
        Node::Neg(a) => -eval(a, vars),
        Node::Bin(op, a, b) => {
            let (x, y) = (eval(a, vars), eval(b, vars));
            match op {
                '+' => x + y,
                '-' => x - y,
                '*' => x * y,
                '/' => x / y,
                _ => x.powf(y),
            }
        }
        Node::Call(f, args) => call(f, &args.iter().map(|a| eval(a, vars)).collect::<Vec<_>>()),
    }
}

fn names(n: &Node, out: &mut Vec<String>) {
    match n {
        Node::Var(s) => {
            if !out.contains(s) {
                out.push(s.clone())
            }
        }
        Node::Neg(a) => names(a, out),
        Node::Bin(_, a, b) => {
            names(a, out);
            names(b, out);
        }
        Node::Call(_, args) => args.iter().for_each(|a| names(a, out)),
        Node::Num(_) => {}
    }
}

// ---------------------------------------------------------------------------
// the formula table
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Var {
    pub sym: String,
    pub desc: String,
    /// Display unit text ("" for a pure number).
    pub unit: String,
    factor: f64,
    offset: f64,
    dim: Dim,
    pub aliases: Vec<String>,
    /// Default value in the display unit.
    pub default: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct Formula {
    pub title: String,
    pub names: Vec<String>,
    eqs: Vec<(String, Node, Node)>,
    pub vars: Vec<Var>,
    pub note: String,
}

fn unit_info(u: &str) -> Option<(f64, f64, Dim)> {
    let u = u.trim();
    if u.is_empty() || u == "1" {
        return Some((1.0, 0.0, [0; 8]));
    }
    if matches!(u, "deg" | "degrees" | "°") {
        return Some((std::f64::consts::PI / 180.0, 0.0, [0; 8]));
    }
    if u == "rad" {
        return Some((1.0, 0.0, [0; 8]));
    }
    if matches!(u, "°C" | "degC") {
        return Some((1.0, 273.15, [0, 0, 0, 0, 1, 0, 0, 0]));
    }
    if matches!(u, "°F" | "degF") {
        return Some((5.0 / 9.0, 459.67, [0, 0, 0, 0, 1, 0, 0, 0]));
    }
    if let Some((a, b)) = u.split_once("/(") {
        let (fa, _, da) = unit_info(a)?;
        let (fb, _, db) = unit_info(b.strip_suffix(')')?)?;
        let mut d = da;
        for k in 0..8 {
            d[k] -= db[k];
        }
        return Some((fa / fb, 0.0, d));
    }
    if matches!(u, "molal" | "m molal") {
        return Some((1.0, 0.0, [0, -1, 0, 0, 0, 1, 0, 0]));
    }
    if matches!(u, "molar" | "M") {
        return Some((1000.0, 0.0, [-3, 0, 0, 0, 0, 1, 0, 0]));
    }
    if let Some(rest) = u.strip_prefix("1/").or_else(|| u.strip_prefix("per ")) {
        let (f, _, d) = unit_info(rest)?;
        return Some((1.0 / f, 0.0, d.map(|x| -x)));
    }
    let p = units::parse_unit(u)?;
    Some((p.factor, p.offset, p.dim))
}

/// Problems found while reading the table (checked by a test).
static PROBLEMS: OnceLock<std::sync::Mutex<Vec<String>>> = OnceLock::new();

fn problem(s: String) {
    PROBLEMS.get_or_init(Default::default).lock().unwrap().push(s);
}

pub fn formulas() -> &'static [Formula] {
    static T: OnceLock<Vec<Formula>> = OnceLock::new();
    T.get_or_init(|| parse_table(include_str!("../../data/formulas.txt")))
}

fn parse_table(src: &str) -> Vec<Formula> {
    let mut out: Vec<Formula> = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("==") {
            let parts: Vec<String> = rest.split('|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            let mut names: Vec<String> = parts.iter().map(|s| s.to_lowercase()).collect();
            names.dedup();
            out.push(Formula { title: parts.first().cloned().unwrap_or_default(), names, eqs: vec![], vars: vec![], note: String::new() });
            continue;
        }
        let Some(f) = out.last_mut() else { continue };
        if let Some(rest) = line.strip_prefix("eq ") {
            let (a, b) = rest.split_once('=').expect("equation needs =");
            let (Some(l), Some(r)) = (parse_node(a), parse_node(b)) else {
                problem(format!("bad equation: {rest}"));
                continue;
            };
            f.eqs.push((rest.trim().to_string(), l, r));
        } else if let Some(rest) = line.strip_prefix("var ") {
            let p: Vec<&str> = rest.split(';').map(|s| s.trim()).collect();
            let unit = p.get(2).copied().unwrap_or("");
            let Some((factor, offset, dim)) = unit_info(unit) else {
                problem(format!("unknown unit {unit:?} in {rest}"));
                continue;
            };
            let mut aliases: Vec<String> = p.get(3).map(|s| s.split(',').map(|a| a.trim().to_lowercase()).filter(|a| !a.is_empty()).collect()).unwrap_or_default();
            aliases.push(p[1].to_lowercase());
            f.vars.push(Var { sym: p[0].to_string(), desc: p[1].to_string(), unit: unit.to_string(), factor, offset, dim, aliases, default: p.get(4).and_then(|s| s.parse().ok()) });
        } else if let Some(rest) = line.strip_prefix("note ") {
            f.note = rest.to_string();
        }
    }
    out
}

// ---------------------------------------------------------------------------
// reading the question
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Given {
    /// SI value, or the bare number when there was no unit.
    value: f64,
    unit: Option<(f64, f64, Dim, String)>,
    /// Text before the number, lower case.
    before: String,
    shown: String,
}

fn parse_number(s: &str) -> Option<f64> {
    let s = s.replace([' ', ','], "").replace('×', "*").replace('x', "*");
    if let Some((a, b)) = s.split_once("*10^") {
        return Some(a.parse::<f64>().ok()? * 10f64.powf(b.parse().ok()?));
    }
    if let Some((a, b)) = s.split_once('^') {
        return Some(a.parse::<f64>().ok()?.powf(b.parse().ok()?));
    }
    if let Some((a, b)) = s.split_once('/') {
        return Some(a.parse::<f64>().ok()? / b.parse::<f64>().ok()?);
    }
    s.parse().ok()
}

fn givens(text: &str) -> Vec<Given> {
    let num = re!(r"(?:^|[^A-Za-z0-9_.])([-+]?(?:\d[\d,]*(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+|\s*(?:x|×|\*)\s*10\^[-+]?\d+|\^[-+]?\d+)?(?:/\d+(?:\.\d+)?)?)");
    let mut out = Vec::new();
    let mut last_end = 0;
    for m in num.captures_iter(text) {
        let g = m.get(1).unwrap();
        if g.start() < last_end {
            continue;
        }
        let Some(mut value) = parse_number(g.as_str()) else { continue };
        // the unit: words after the number, up to a delimiter
        let rest = &text[g.end()..];
        let stop = re!(r"(?i)[,;?]|\s+(?:and|for|at|with|from|to|in|on|of|is|by|after|over|using|v[a-z]*\s*=|[a-z_]\w*\s*=)\s|\s+[a-z_]\w*\s*=|$").find(rest).map_or(rest.len(), |s| s.start());
        let words: Vec<&str> = rest[..stop].split_whitespace().collect();
        let mut unit = None;
        let mut used = 0;
        for k in (1..=words.len().min(4)).rev() {
            let cand = words[..k].join(" ");
            let cand = if cand.contains('(') { cand.trim_end_matches('.') } else { cand.trim_end_matches(['.', ')']) };
            if let Some(u) = unit_info(cand).filter(|_| !cand.is_empty()) {
                unit = Some((u.0, u.1, u.2, cand.to_string()));
                used = k;
                break;
            }
        }
        if unit.is_none() {
            // glued: "650nm", "0.8c" are already split by the regex, so only
            // a unit starting right after the number is left to check
            let glued: String = rest.chars().take_while(|c| c.is_alphanumeric() || "/^*°µμΩ".contains(*c)).collect();
            if !glued.is_empty() {
                if let Some(u) = unit_info(&glued) {
                    unit = Some((u.0, u.1, u.2, glued.clone()));
                }
            }
        }
        let unit_len = if used > 0 { rest.find(words[used - 1]).map_or(0, |p| p + words[used - 1].len()) } else { unit.as_ref().map_or(0, |u| u.3.len()) };
        if let Some((f, o, _, _)) = &unit {
            value = (value + o) * f;
        }
        let before = text[last_end..g.start()].to_lowercase();
        last_end = g.end() + unit_len;
        out.push(Given { value, unit, before, shown: text[g.start()..last_end].trim().to_string() });
    }
    out
}

fn word_at_end(hay: &str, needle: &str) -> bool {
    hay.ends_with(needle) && hay[..hay.len() - needle.len()].chars().last().map_or(true, |c| !c.is_alphanumeric())
}

fn contains_word(hay: &str, needle: &str) -> bool {
    let mut from = 0;
    while let Some(p) = hay[from..].find(needle) {
        let s = from + p;
        let e = s + needle.len();
        let ok_l = hay[..s].chars().last().map_or(true, |c| !c.is_alphanumeric());
        let ok_r = hay[e..].chars().next().map_or(true, |c| !c.is_alphanumeric());
        if ok_l && ok_r {
            return true;
        }
        from = s + needle.len().max(1);
    }
    false
}

/// Strip connecting words from the text before a number.
fn lead(before: &str) -> String {
    let mut s = before.trim().to_string();
    loop {
        let t = s.trim_end_matches([' ', '=', ':', '~', '(']).to_string();
        let t2 = ["of", "is", "at", "equal to", "equals", "with", "a", "an", "the", "by", "about"].iter().find_map(|w| (word_at_end(&t, w) && t.len() > w.len()).then(|| t[..t.len() - w.len()].to_string())).unwrap_or(t.clone());
        if t2 == s {
            return s;
        }
        s = t2;
    }
}

const PARTICLES: &[(&str, f64)] = &[
    ("electron", 9.109_383_701_5e-31),
    ("positron", 9.109_383_701_5e-31),
    ("proton", 1.672_621_923_69e-27),
    ("neutron", 1.674_927_498_04e-27),
    ("muon", 1.883_531_627e-28),
    ("alpha particle", 6.644_657_335_7e-27),
    ("deuteron", 3.343_583_772_4e-27),
];

struct Solved {
    /// Units the question used, per dimension: results are shown in them.
    pref: Vec<(Dim, f64, f64, String)>,
    values: HashMap<String, f64>,
    given: Vec<(usize, String)>,
    computed: Vec<usize>,
}

fn same_dim(a: &Dim, b: &Dim) -> bool {
    a == b
}

fn solve_formula(f: &Formula, text: &str) -> Solved {
    // "2 in by 6 in by 10 ft": a list of dimensions
    let listed = re!(r"(\d(?:\.\d+)?\s*[A-Za-z\x22']*)\s+(?:by|x|×)\s+(\d)").replace_all(text, "$1, $2").to_string();
    let listed = re!(r"(\d(?:\.\d+)?\s*[A-Za-z\x22']*)\s+(?:by|x|×)\s+(\d)").replace_all(&listed, "$1, $2").to_string();
    let text = listed.as_str();
    let lower = text.to_lowercase();
    let mut values: HashMap<String, f64> = HashMap::new();
    let mut given: Vec<(usize, String)> = Vec::new();
    let gs = givens(text);
    let mut used = vec![false; gs.len()];
    let to_si = |v: &Var, g: &Given| -> Option<f64> {
        match &g.unit {
            Some((_, _, d, _)) if same_dim(d, &v.dim) => Some(g.value),
            // "100 dB" reads as decibytes; a dimensionless level takes the bare number
            Some((_, _, _, u)) if u.eq_ignore_ascii_case("db") && v.dim == [0; 8] => parse_number(g.shown[..g.shown.len() - u.len()].trim()),
            Some((_, _, _, u)) => {
                // single letters with a second reading: C and F as temperatures, H as henries
                let alt = match u.as_str() {
                    "C" => "°C",
                    "F" => "°F",
                    "H" => "henry",
                    "T" => "tesla",
                    _ => return None,
                };
                let (fa, off, d) = unit_info(alt)?;
                let raw = parse_number(g.shown[..g.shown.len() - u.len()].trim())?;
                same_dim(&d, &v.dim).then(|| (raw + off) * fa)
            }
            None => Some((g.value + v.offset) * v.factor),
        }
    };
    // 1. explicit "sym = value" and descriptive phrases
    for (k, g) in gs.iter().enumerate() {
        let b = lead(&g.before);
        let sym = re!(r"([A-Za-z_][A-Za-z0-9_]*)\s*$").captures(g.before.trim_end().trim_end_matches(['=', ':']).trim_end()).map(|m| m[1].to_string());
        let explicit = g.before.trim_end().ends_with('=') || g.before.trim_end().ends_with(':');
        let mut pick = None;
        if explicit {
            // the whole name before "=": "atmospheric pressure=1 bar"
            let phrase = g.before.rsplit([',', ';', '(']).next().unwrap_or("").trim().trim_end_matches(['=', ':']).trim().to_lowercase();
            pick = f.vars.iter().position(|v| !values.contains_key(&v.sym) && (v.aliases.contains(&phrase) || v.sym.to_lowercase() == phrase));
        }
        if explicit && pick.is_none() {
            if let Some(s) = &sym {
                pick = f.vars.iter().position(|v| !values.contains_key(&v.sym) && (v.sym == *s || v.sym.eq_ignore_ascii_case(s) || v.aliases.iter().any(|a| a == &s.to_lowercase())));
            }
        }
        if pick.is_none() {
            let best = f.vars.iter().enumerate().rev().filter(|(_, v)| !values.contains_key(&v.sym)).filter_map(|(i, v)| v.aliases.iter().filter(|a| a.len() > 1 && word_at_end(&b, a)).map(|a| a.len()).max().map(|n| (i, n))).max_by_key(|x| x.1);
            pick = best.map(|x| x.0);
        }
        if let Some(i) = pick {
            if let Some(si) = to_si(&f.vars[i], g) {
                values.insert(f.vars[i].sym.clone(), si);
                given.push((i, g.shown.clone()));
                used[k] = true;
            }
        }
    }
    // 2. particle names
    for (name, m) in PARTICLES {
        if contains_word(&lower, name) {
            if let Some(i) = f.vars.iter().position(|v| v.dim == [0, 1, 0, 0, 0, 0, 0, 0] && !values.contains_key(&v.sym)) {
                values.insert(f.vars[i].sym.clone(), *m);
                given.push((i, name.to_string()));
            }
        }
    }
    // 3. remaining quantities by dimension: inputs without defaults, inputs
    //    with defaults, then outputs (variables an equation computes)
    let outputs: Vec<&str> = f.eqs.iter().filter_map(|(_, l, _)| if let Node::Var(v) = l { Some(v.as_str()) } else { None }).collect();
    for pass in 0..3 {
        for (k, g) in gs.iter().enumerate() {
            if used[k] {
                continue;
            }
            let fits = |v: &Var| -> bool {
                match &g.unit {
                    None => v.dim == [0; 8],
                    Some(_) => to_si(v, g).is_some(),
                }
            };
            let ok_pass = |v: &Var| {
                let out = outputs.contains(&v.sym.as_str());
                match pass {
                    0 => !out && v.default.is_none(),
                    1 => !out,
                    _ => true,
                }
            };
            if let Some(i) = f.vars.iter().position(|v| !values.contains_key(&v.sym) && ok_pass(v) && fits(v)) {
                if let Some(si) = to_si(&f.vars[i], g) {
                    values.insert(f.vars[i].sym.clone(), si);
                    given.push((i, g.shown.clone()));
                    used[k] = true;
                }
            }
        }
    }
    // the target, when named: "p=?", "torque?", "what is the stress"
    let mut target: Option<usize> = None;
    if let Some(m) = re!(r"([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\?").captures(text) {
        target = f.vars.iter().position(|v| v.sym == m[1] || v.aliases.contains(&m[1].to_lowercase()));
    }
    if target.is_none() {
        target = f.vars.iter().enumerate().rev().filter(|(_, v)| !values.contains_key(&v.sym)).filter_map(|(i, v)| v.aliases.iter().filter(|a| a.len() > 2 && contains_word(&lower, a)).map(|a| a.len()).max().map(|n| (i, n))).max_by_key(|x| x.1).map(|x| x.0);
    }
    // defaults for variables that are still unknown, except the target
    let mut defaults_used = Vec::new();
    // solve, one unknown at a time
    let mut computed: Vec<usize> = Vec::new();
    // which equation gave each computed value
    let mut from_eq: std::collections::HashMap<usize, usize> = Default::default();
    let mut progress = true;
    let mut used_defaults = false;
    while progress {
        progress = false;
        for (ei, (_, l, r)) in f.eqs.iter().enumerate() {
            let mut ns = Vec::new();
            names(l, &mut ns);
            names(r, &mut ns);
            let unknown: Vec<&String> = ns.iter().filter(|n| f.vars.iter().any(|v| &v.sym == *n) && !values.contains_key(*n)).collect();
            if unknown.len() != 1 {
                continue;
            }
            let x = unknown[0].clone();
            let vi = f.vars.iter().position(|v| v.sym == x).unwrap();
            if let Some(v) = solve_one(l, r, &x, &values, &f.vars[vi]) {
                from_eq.insert(vi, ei);
                values.insert(x, v);
                computed.push(vi);
                progress = true;
            }
        }
        if !progress && !used_defaults {
            used_defaults = true;
            for (i, v) in f.vars.iter().enumerate() {
                if !values.contains_key(&v.sym) && Some(i) != target {
                    if let Some(d) = v.default {
                        values.insert(v.sym.clone(), (d + v.offset) * v.factor);
                        defaults_used.push(i);
                        progress = true;
                    }
                }
            }
        }
    }
    for i in defaults_used {
        if !computed.contains(&i) {
            given.push((i, format!("{} (assumed)", show_value(&f.vars[i], values[&f.vars[i].sym]))));
        }
    }
    // with no target named, the formula's main (first) equation leads: a
    // radius worked out from a diameter follows the flow rate
    if target.is_none() {
        computed.sort_by_key(|c| from_eq.get(c).copied().unwrap_or(usize::MAX));
    }
    if let Some(t) = target {
        if let Some(p) = computed.iter().position(|&c| c == t) {
            let v = computed.remove(p);
            computed.insert(0, v);
        }
    }
    let pref = gs.iter().filter_map(|g| g.unit.clone()).filter(|u| u.2 != [0; 8]).map(|(f, o, d, t)| (d, f, o, t)).collect();
    Solved { pref, values, given, computed }
}

fn residual(l: &Node, r: &Node, x: &str, v: f64, vals: &HashMap<String, f64>) -> f64 {
    let mut m = vals.clone();
    m.insert(x.to_string(), v);
    eval(l, &m) - eval(r, &m)
}

fn solve_one(l: &Node, r: &Node, x: &str, vals: &HashMap<String, f64>, var: &Var) -> Option<f64> {
    // direct: x = expression
    if let Node::Var(s) = l {
        if s == x {
            let mut ns = Vec::new();
            names(r, &mut ns);
            if !ns.iter().any(|n| n == x) {
                let v = eval(r, vals);
                return v.is_finite().then_some(v);
            }
        }
    }
    let f = |v: f64| residual(l, r, x, v, vals);
    let is_angle = var.unit == "deg" || var.unit == "rad";
    let mut grid: Vec<f64> = Vec::new();
    if is_angle {
        grid.extend((0..=2000).map(|k| k as f64 * std::f64::consts::PI / 2000.0));
    } else {
        let mut v = 1e-60;
        while v < 1e60 {
            grid.push(v);
            v *= 1.25;
        }
        let neg: Vec<f64> = grid.iter().map(|v| -v).collect();
        grid.push(0.0);
        grid.extend(neg);
    }
    let mut prev: Option<(f64, f64)> = None;
    for &g in &grid {
        let y = f(g);
        if y == 0.0 {
            return Some(g);
        }
        if !y.is_finite() {
            prev = None;
            continue;
        }
        if let Some((pg, py)) = prev {
            if py.signum() != y.signum() && (pg >= 0.0) == (g >= 0.0) {
                // bisect
                let (mut a, mut b, mut fa) = (pg, g, py);
                for _ in 0..200 {
                    let m = if !is_angle && a > 0.0 && b > 0.0 { (a * b).sqrt() } else if !is_angle && a < 0.0 && b < 0.0 { -((a * b).sqrt()) } else { 0.5 * (a + b) };
                    let fm = f(m);
                    if !fm.is_finite() {
                        break;
                    }
                    if fm.signum() == fa.signum() {
                        a = m;
                        fa = fm;
                    } else {
                        b = m;
                    }
                }
                let root = 0.5 * (a + b);
                // reject poles: the residual must be small near the root
                let scale = f(root * 1.000001 + 1e-300).abs().max(fa.abs()).max(1e-300);
                if f(root).abs() <= 1e-6 * scale.max(py.abs()) || f(root).abs() < 1e-9 * (1.0 + py.abs()) {
                    return Some(root);
                }
            }
        }
        prev = Some((g, y));
    }
    None
}

fn show_value(v: &Var, si: f64) -> String {
    show_in(v, si, &[])
}

/// A value in the unit the question used for this dimension, if any, else
/// in the variable's own unit.
fn show_in(v: &Var, si: f64, pref: &[(Dim, f64, f64, String)]) -> String {
    let (factor, offset, unit) = pref.iter().find(|p| p.0 == v.dim && !v.unit.is_empty() && v.unit != "deg").map(|p| (p.1, p.2, p.3.as_str())).unwrap_or((v.factor, v.offset, v.unit.as_str()));
    let x = si / factor - offset;
    let n = fmt_float(x, 6);
    if unit.is_empty() || unit == "1" {
        if v.desc.starts_with("percent") { format!("{n}%") } else { n }
    } else if unit == "deg" {
        format!("{n}°")
    } else {
        format!("{n} {unit}")
    }
}

fn pretty_eq(s: &str) -> String {
    let mut t = s.replace(" * ", " ").replace('*', " ").replace("sqrt", "√").replace("pi", "π").replace("lam", "λ").replace("theta", "θ").replace("sigma_SB", "σ").replace("eps0", "ε₀").replace("mu0", "μ₀").replace("hbar", "ħ");
    for (a, b) in [("^2", "²"), ("^3", "³"), ("^4", "⁴")] {
        t = t.replace(a, b);
    }
    t
}

// ---------------------------------------------------------------------------
// choosing a formula
// ---------------------------------------------------------------------------

fn normalise(l: &str) -> String {
    let l = l.to_lowercase().replace(['’', '‘'], "'").replace('-', " ");
    let l = re!(r"\b(calculator|calculation|formula|equation|law|relation|principle|effect)s?\b").replace_all(&l, "$1");
    l.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn find(l: &str) -> Option<&'static Formula> {
    let n = normalise(l);
    let mut best: Option<(&Formula, usize)> = None;
    for f in formulas() {
        for name in &f.names {
            let nn = normalise(name);
            if contains_word(&n, &nn) && best.map_or(true, |b| nn.len() > b.1) {
                best = Some((f, nn.len()));
            }
        }
    }
    best.map(|b| b.0)
}

/// No formula named: "rho=1g/cm^3, h=3m, atmospheric pressure=1 bar, p=?"
fn find_by_symbols(text: &str) -> Option<&'static Formula> {
    let syms: Vec<String> = re!(r"([A-Za-z_][A-Za-z0-9_ ]*?)\s*=").captures_iter(text).map(|m| m[1].trim().to_lowercase()).collect();
    // without a "?", at least one name must be a word ("height = 3 km")
    if syms.len() < 2 || !text.contains('?') && !syms.iter().any(|s| s.len() >= 4 && s.chars().all(|c| c.is_ascii_alphabetic() || c == ' ')) {
        return None;
    }
    let score = |f: &Formula| syms.iter().filter(|s| f.vars.iter().any(|v| v.sym.to_lowercase() == **s || v.aliases.contains(s))).count();
    let best = formulas().iter().max_by_key(|f| (score(f), usize::MAX - f.vars.len()))?;
    (score(best) == syms.len()).then_some(best)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let f = find(&q.lower).or_else(|| find_by_symbols(q.raw))?;
    let s = solve_formula(f, if q.raw.contains('?') { q.raw.trim() } else { &q.clean });
    if s.computed.is_empty() && s.given.is_empty() && !only_names(f, &q.lower) {
        // "apparent magnitude of Rigel" asks about Rigel, not for the formula
        return None;
    }
    let mut pods = Vec::new();
    let given_text = s.given.iter().map(|(i, shown)| format!("{} = {shown}", f.vars[*i].desc)).collect::<Vec<_>>().join("  |  ");
    pods.push(pod("Input", "Input interpretation", text(if given_text.is_empty() { f.title.clone() } else { format!("{}  |  {given_text}", f.title) })));
    if let Some(&first) = s.computed.first() {
        let v = &f.vars[first];
        pods.push(primary(pod("Result", &cap(&v.desc), text(show_in(v, s.values[&v.sym], &s.pref)))));
        if v.unit != units::si_name(&v.dim) && !v.unit.is_empty() && v.unit != "deg" {
            let si = units::si_name(&v.dim);
            if !si.is_empty() {
                pods.push(pod("SI", "In SI units", text(format!("{} {si}", fmt_float(s.values[&v.sym], 6)))));
            }
        }
        if s.computed.len() > 1 {
            let rows = s.computed[1..].iter().map(|&i| vec![text(f.vars[i].desc.clone()), text(show_in(&f.vars[i], s.values[&f.vars[i].sym], &s.pref))]).collect();
            pods.push(pod("More", "Other results", table(None, rows)));
        }
    }
    let eqs = f.eqs.iter().map(|(t, ..)| pretty_eq(t)).collect::<Vec<_>>().join("\n");
    let eq_pod = pod("Equation", "Equation", text(eqs));
    pods.push(if s.computed.is_empty() { primary(eq_pod) } else { eq_pod });
    let rows = f.vars.iter().map(|v| vec![text(pretty_eq(&v.sym)), text(v.desc.clone()), text(if v.unit.is_empty() { "—".to_string() } else { v.unit.clone() })]).collect();
    pods.push(pod("Variables", "Variables", table(Some(vec!["symbol".into(), "meaning".into(), "unit".into()]), rows)));
    if !f.note.is_empty() {
        pods.push(pod("Notes", "Notes", text(f.note.clone())));
    }
    Some(Answer::new(f.title.clone(), pods))
}

/// Whether the text is just a name of the formula plus generic words.
fn only_names(f: &Formula, l: &str) -> bool {
    let n = normalise(l);
    let mut names: Vec<String> = f.names.iter().map(|x| normalise(x)).filter(|x| contains_word(&n, x)).collect();
    if names.is_empty() {
        return false;
    }
    names.sort_by_key(|x| std::cmp::Reverse(x.len()));
    let mut rest = n.clone();
    for name in names {
        if contains_word(&rest, &name) {
            rest = rest.replacen(&name, " ", 1);
        }
    }
    const GENERIC: &[&str] = &[
        "formula", "formulas", "equation", "equations", "law", "calculator", "calculation", "calculations", "principle", "effect", "relation", "what", "is", "the", "a", "an", "show", "me", "of", "for", "give", "explain", "define", "definition", "derive", "derivation", "use", "using",
        "how", "to", "calculate", "compute", "find", "solve", "example", "examples", "problem", "problems", "physics", "chemistry", "general", "simple", "basic", "and", "in", "?",
    ];
    rest.split(|c: char| c.is_whitespace() || c == ',' || c == '?').filter(|w| !w.is_empty()).all(|w| GENERIC.contains(&w))
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_parses_and_units_resolve() {
        let t = formulas();
        let probs = PROBLEMS.get_or_init(Default::default).lock().unwrap().clone();
        assert!(probs.is_empty(), "{}", probs.join("\n"));
        assert!(t.len() > 50, "{} formulas", t.len());
        for f in t {
            for (src, l, r) in &f.eqs {
                let mut ns = Vec::new();
                names(l, &mut ns);
                names(r, &mut ns);
                for n in ns {
                    assert!(f.vars.iter().any(|v| v.sym == n) || CONSTANTS.iter().any(|c| c.0 == n), "{}: unknown name {n} in {src}", f.title);
                }
            }
        }
    }

    fn run(q: &str) -> (String, f64) {
        let f = find(&q.to_lowercase()).or_else(|| find_by_symbols(q)).unwrap_or_else(|| panic!("no formula for {q}"));
        let s = solve_formula(f, q);
        let i = *s.computed.first().unwrap_or_else(|| panic!("nothing computed for {q}: given {:?}", s.given));
        let v = &f.vars[i];
        let (factor, offset) = s.pref.iter().find(|p| p.0 == v.dim && !v.unit.is_empty() && v.unit != "deg").map(|p| (p.1, p.2)).unwrap_or((v.factor, v.offset));
        (v.sym.clone(), s.values[&v.sym] / factor - offset)
    }

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol * b.abs()
    }

    #[test]
    fn solves_examples() {
        let (s, v) = run("ideal gas law 2.2mol, 2.0atm, 500K");
        assert_eq!(s, "V");
        assert!(close(v, 45.13, 1e-3), "{v}");
        let (_, v) = run("lens, f=20ft, o=400ft");
        assert!(close(v, 21.0526, 1e-4), "{v}");
        let (_, v) = run("Rydberg formula ni=2, nf=5");
        assert!(close(v, 434.17, 1e-3), "{v}");
        let (_, v) = run("Snell's law index of refraction 1, index of refraction 1.3, angle 45");
        assert!(close(v, 32.955, 1e-3), "{v}");
        let (_, v) = run("electric power 1.5V, 3A");
        assert!(close(v, 4.5, 1e-9), "{v}");
        let (_, v) = run("rho=1g/cm^3, h=3m, atmospheric pressure=1 bar, p=?");
        assert!(close(v, 1.2941995, 1e-4), "{v}"); // in bar, the unit the question used
        let (_, v) = run("adiabatic process Vi=10L, Vf=2L, pi=1atm");
        assert!(close(v, 9.518, 1e-3), "{v}");
        let (_, v) = run("Compton wavelength proton");
        assert!(close(v, 1.32141e-15, 1e-4), "{v}");
        let (_, v) = run("Doppler shift 300Hz, 75mph");
        assert!(close(v, 330.0, 2e-2), "{v}");
    }
}
