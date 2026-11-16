//! Simple computational systems: cellular automata ("rule 30",
//! "CA 3-color, range 1, rule 4594122302107", "3-color code 1086"),
//! Turing machines in Wolfram's numbering ("Turing machine rule 2506",
//! "3,2 TM rule 2139050", "Wolfram 2,3 machine", explicit rule lists,
//! "evolve Wolfram 2,3 for 100 steps", "TM 3434 on random tape"), busy
//! beavers and substitution systems ("substitution system 0->10, 1->01").
//!
//! Numbering follows the standard conventions of A New Kind of Science: a
//! cellular-automaton rule number lists, in base k, the new colour for each
//! neighbourhood read as a base-k number; a Turing-machine rule number lists,
//! in base 2sk, the outcome {s', a', move} for the cases {1, k-1}, …, {1, 0},
//! {2, k-1}, …, {s, 0}, with digit 2((s'-1)k + a') + (move = right).

use super::*;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

const SHADES: [char; 5] = ['·', '░', '▒', '▓', '█'];

fn shade(v: u8, k: u8) -> char {
    if v == 0 {
        return SHADES[0];
    }
    if k <= 2 {
        return '█';
    }
    let idx = 1 + ((v as usize - 1) * 3) / (k as usize - 1).max(1);
    SHADES[idx.min(4)]
}

fn picture(rows: &[Vec<u8>], k: u8) -> String {
    rows.iter().map(|r| r.iter().map(|&v| shade(v, k)).collect::<String>()).collect::<Vec<_>>().join("\n")
}

/// A small deterministic generator for "random" tapes and rules.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn time_seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1)
}

fn hash_pos(p: i64, seed: u64) -> u64 {
    let mut r = Rng::new(seed ^ (p as u64).wrapping_mul(0xD1B54A32D192ED03));
    r.next();
    r.next()
}

// ---------------------------------------------------------------------------
// cellular automata
// ---------------------------------------------------------------------------

struct Ca {
    k: u8,
    r: usize,
    /// new colour for each neighbourhood value (or each total, when totalistic)
    table: Vec<u8>,
    totalistic: bool,
}

impl Ca {
    fn from_number(n: &BigUint, k: u8, r: usize, totalistic: bool) -> Option<Ca> {
        let size = if totalistic { (2 * r + 1) * (k as usize - 1) + 1 } else { (k as usize).checked_pow(2 * r as u32 + 1)? };
        if size > 1 << 16 {
            return None;
        }
        let kk = BigUint::from(k);
        let mut table = Vec::with_capacity(size);
        let mut m = n.clone();
        for _ in 0..size {
            table.push((&m % &kk).to_u8()?);
            m /= &kk;
        }
        if !m.is_zero() {
            return None;
        }
        Some(Ca { k, r, table, totalistic })
    }

    /// The colour the uniform background takes after one step.
    fn next_bg(&self, bg: u8) -> u8 {
        let w = 2 * self.r + 1;
        let mut v = 0usize;
        for _ in 0..w {
            v = if self.totalistic { v + bg as usize } else { v * self.k as usize + bg as usize };
        }
        self.table[v]
    }

    fn step(&self, row: &[u8], bg: u8) -> Vec<u8> {
        let n = row.len() as i64;
        let w = 2 * self.r as i64 + 1;
        (0..n).map(|i| {
            let mut v = 0usize;
            for d in 0..w {
                let j = i - self.r as i64 + d;
                let c = if j < 0 || j >= n { bg } else { row[j as usize] } as usize;
                v = if self.totalistic { v + c } else { v * self.k as usize + c };
            }
            self.table[v]
        }).collect()
    }

    fn evolve(&self, steps: usize, init: Vec<u8>) -> Vec<Vec<u8>> {
        let mut rows = vec![init];
        let mut bg = 0u8;
        for _ in 0..steps {
            let next = self.step(rows.last().unwrap(), bg);
            bg = self.next_bg(bg);
            rows.push(next);
        }
        rows
    }
}

fn single_cell(width: usize) -> Vec<u8> {
    let mut v = vec![0u8; width];
    v[width / 2] = 1;
    v
}

fn elementary_props(n: u32) -> Vec<Vec<Content>> {
    let out = |x: u32| (n >> x) & 1;
    let refl: u32 = (0..8).map(|x| {
        let (l, c, r) = ((x >> 2) & 1, (x >> 1) & 1, x & 1);
        out((r << 2) | (c << 1) | l) << x
    }).sum();
    let comp: u32 = (0..8).map(|x| (1 - out(7 - x)) << x).sum();
    let both: u32 = (0..8).map(|x| {
        let (l, c, r) = ((x >> 2) & 1, (x >> 1) & 1, x & 1);
        let y = 7 - ((r << 2) | (c << 1) | l);
        (1 - out(y)) << x
    }).sum();
    let mut rowsv = vec![
        vec![text("left-right reflection"), text(format!("rule {refl}"))],
        vec![text("colour complement"), text(format!("rule {comp}"))],
        vec![text("reflection and complement"), text(format!("rule {both}"))],
    ];
    let class = match n.min(refl).min(comp).min(both) {
        0 | 8 | 32 | 40 | 128 | 136 | 160 | 168 => Some("1 (evolves to a uniform state)"),
        18 | 22 | 30 | 45 | 60 | 90 | 105 | 122 | 126 | 146 | 150 => Some("3 (apparently random, chaotic)"),
        54 | 106 | 110 => Some("4 (localized structures interact in complex ways)"),
        _ => Some("2 (simple stable or periodic structures)"),
    };
    if let Some(c) = class {
        rowsv.push(vec![text("Wolfram class"), text(c)]);
    }
    let additive = [0u32, 60, 90, 102, 150, 170, 204, 240, 15, 51, 85, 105, 153, 165, 195, 255].contains(&n);
    rowsv.push(vec![text("additive"), text(if additive { "yes" } else { "no" })]);
    rowsv
}

fn cellular(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let steps = re!(r"(?:,\s*|\s+)(?:for\s+)?(\d+)\s+(?:steps|generations)$").captures(&l).and_then(|m| m[1].parse::<usize>().ok());
    let l = re!(r"(?:,\s*|\s+)(?:for\s+)?\d+\s+(?:steps|generations)$").replace(&l, "").to_string();
    // elementary
    if let Some(m) = re!(r"^(?:(?:elementary|1-?d)\s+)?(?:(?:cellular automaton|ca)\s+)?rule\s+(\d{1,3})(?:\s+(?:elementary\s+)?(?:cellular automaton|ca|automaton))?$").captures(&l) {
        let explicit = l.contains("automat") || l.contains("ca ") || l.ends_with(" ca") || l.contains("elementary");
        let n: u32 = m[1].parse().ok()?;
        if n > 255 || (!explicit && l != format!("rule {n}")) {
            return None;
        }
        let steps = steps.unwrap_or(30).clamp(1, 300);
        let ca = Ca::from_number(&BigUint::from(n), 2, 1, false)?;
        let rows = ca.evolve(steps, single_cell(2 * steps + 1));
        let rule_rows: Vec<Vec<Content>> = (0..8u32).rev().map(|x| {
            let pat: String = (0..3).rev().map(|b| shade(((x >> b) & 1) as u8, 2)).collect();
            vec![text(format!("{pat} ({:03b})", x)), text(format!("{} ({})", shade(((n >> x) & 1) as u8, 2), (n >> x) & 1))]
        }).collect();
        return Some(Answer::new("cellular automaton", vec![
            pod("Input", "Input interpretation", text(format!("rule {n} elementary cellular automaton, {steps} steps from a single black cell"))),
            primary(pod("Evolution", "Evolution", text(picture(&rows, 2)))),
            pod("Rule", "Rule (neighbourhood → new cell)", table(Some(vec!["neighbourhood", "new cell"]), rule_rows)),
            pod("Properties", "Properties", table(None, elementary_props(n))),
            pod("Binary", "Rule number in binary", text(format!("{n:08b}"))),
        ]));
    }
    // general k-colour, range-r, or totalistic codes
    let m = re!(r"^(?:ca|cellular automaton)?\s*,?\s*(?:(\d+)-?\s*colou?rs?)?\s*,?\s*(?:range\s*(\d+(?:\.5|/2)?))?\s*,?\s*(?:(?:totalistic\s+)?(?:rule|code)\s+(\d+))(?:\s+(?:totalistic\s+)?(?:cellular automaton|ca))?$").captures(&l)?;
    if !(l.contains("color") || l.contains("colour") || l.contains("range") || l.contains("code") || l.contains("ca") || l.contains("automaton")) {
        return None;
    }
    let k: u8 = m.get(1).and_then(|x| x.as_str().parse().ok()).unwrap_or(2);
    let r: usize = m.get(2).and_then(|x| x.as_str().parse().ok()).unwrap_or(1);
    if !(2..=5).contains(&k) || r == 0 || r > 4 {
        return None;
    }
    let totalistic = l.contains("code") || l.contains("totalistic");
    let n: BigUint = m[3].parse().ok()?;
    let ca = Ca::from_number(&n, k, r, totalistic)?;
    let steps = steps.unwrap_or(40).clamp(1, 300);
    let rows = ca.evolve(steps, single_cell(2 * r * steps + 1));
    let kind = if totalistic { "totalistic code" } else { "rule" };
    let rule_rows: Vec<Vec<Content>> = if totalistic {
        (0..ca.table.len()).rev().map(|i| vec![text(format!("total {i}")), text(format!("{} ({})", shade(ca.table[i], k), ca.table[i]))]).collect()
    } else if ca.table.len() <= 64 {
        (0..ca.table.len()).rev().map(|i| {
            let mut digits = Vec::new();
            let mut x = i;
            for _ in 0..2 * r + 1 {
                digits.push((x % k as usize) as u8);
                x /= k as usize;
            }
            digits.reverse();
            vec![text(digits.iter().map(|d| d.to_string()).collect::<String>()), text(format!("{} ({})", shade(ca.table[i], k), ca.table[i]))]
        }).collect()
    } else {
        vec![]
    };
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{k}-colour, range-{r} cellular automaton, {kind} {n}, {steps} steps from a single cell of colour 1"))),
        primary(pod("Evolution", "Evolution", text(picture(&rows, k)))),
    ];
    if !rule_rows.is_empty() {
        pods.push(pod("Rule", "Rule", table(Some(vec![if totalistic { "neighbourhood total" } else { "neighbourhood" }, "new cell"]), rule_rows)));
    }
    pods.push(pod("Colors", "Colours", text((0..k).map(|c| format!("{c} = {}", shade(c, k))).collect::<Vec<_>>().join(", "))));
    Some(Answer::new("cellular automaton", pods))
}

// ---------------------------------------------------------------------------
// Turing machines
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Tm {
    s: u8,
    k: u8,
    /// (state, colour) → (new state, new colour, move: +1 right / −1 left); state 0 halts
    rules: HashMap<(u8, u8), (u8, u8, i8)>,
}

impl Tm {
    fn from_number(n: &BigUint, s: u8, k: u8) -> Option<Tm> {
        let base = BigUint::from(2u32 * s as u32 * k as u32);
        let cases = s as usize * k as usize;
        let mut digits = Vec::with_capacity(cases);
        let mut m = n.clone();
        for _ in 0..cases {
            digits.push((&m % &base).to_u32()?);
            m /= &base;
        }
        if !m.is_zero() {
            return None;
        }
        digits.reverse();
        let mut rules = HashMap::new();
        let mut i = 0;
        for st in 1..=s {
            for col in (0..k).rev() {
                let d = digits[i];
                i += 1;
                let mv = if d % 2 == 1 { 1 } else { -1 };
                let rest = d / 2;
                let (ns, nc) = ((rest / k as u32) as u8 + 1, (rest % k as u32) as u8);
                rules.insert((st, col), (ns, nc, mv));
            }
        }
        Some(Tm { s, k, rules })
    }

    fn rule_text(&self) -> Vec<Vec<Content>> {
        let mut v = Vec::new();
        for st in 1..=self.s {
            for col in (0..self.k).rev() {
                if let Some((ns, nc, mv)) = self.rules.get(&(st, col)) {
                    let to = if *ns == 0 { "halt".to_string() } else { format!("state {ns}") };
                    v.push(vec![text(format!("state {st}, colour {col}")), text(format!("write {nc}, move {}, {to}", if *mv > 0 { "right" } else { "left" }))]);
                }
            }
        }
        v
    }
}

enum Tape {
    Blank(u8),
    Pattern(Vec<u8>),
    Random(u64, u8),
}

struct Run {
    rows: Vec<(i64, u8, HashMap<i64, u8>)>,
    halted: Option<usize>,
}

struct Machine<'a> {
    tm: &'a Tm,
    bg: Tape,
    cyclic: Option<i64>,
    cells: HashMap<i64, u8>,
    head: i64,
    state: u8,
}

impl Machine<'_> {
    fn read(&self, p: i64) -> u8 {
        let p = self.cyclic.map(|w| p.rem_euclid(w)).unwrap_or(p);
        if let Some(c) = self.cells.get(&p) {
            return *c;
        }
        match &self.bg {
            Tape::Blank(c) => *c,
            Tape::Pattern(v) => v[p.rem_euclid(v.len() as i64) as usize],
            Tape::Random(seed, k) => (hash_pos(p, *seed) % *k as u64) as u8,
        }
    }
    fn step(&mut self) -> bool {
        let c = self.read(self.head);
        let Some(&(ns, nc, mv)) = self.tm.rules.get(&(self.state, c)) else { return false };
        let p = self.cyclic.map(|w| self.head.rem_euclid(w)).unwrap_or(self.head);
        self.cells.insert(p, nc);
        self.head += mv as i64;
        if let Some(w) = self.cyclic {
            self.head = self.head.rem_euclid(w);
        }
        self.state = ns;
        ns != 0
    }
}

fn run_tm(m: &mut Machine, steps: usize, keep: &dyn Fn(usize) -> bool) -> Run {
    let mut rows = Vec::new();
    let mut halted = None;
    if keep(0) {
        rows.push((m.head, m.state, m.cells.clone()));
    }
    for t in 1..=steps {
        if !m.step() {
            halted = Some(t);
            if !keep(t) {
                rows.push((m.head, m.state, m.cells.clone()));
            }
            break;
        }
        if keep(t) {
            rows.push((m.head, m.state, m.cells.clone()));
        }
        if m.state == 0 {
            halted = Some(t);
            break;
        }
    }
    Run { rows, halted }
}

fn tm_picture(m: &Machine, run: &Run) -> String {
    let (mut lo, mut hi) = (i64::MAX, i64::MIN);
    for (h, _, cells) in &run.rows {
        lo = lo.min(*h);
        hi = hi.max(*h);
        for p in cells.keys() {
            lo = lo.min(*p);
            hi = hi.max(*p);
        }
    }
    if let Some(w) = m.cyclic {
        lo = 0;
        hi = w - 1;
    }
    lo -= 1;
    hi += 1;
    if hi - lo > 120 {
        let h = run.rows.last().map(|r| r.0).unwrap_or(0);
        lo = h - 60;
        hi = h + 60;
    }
    let k = m.tm.k;
    run.rows.iter().map(|(h, st, cells)| {
        (lo..=hi).map(|p| {
            if p == *h {
                return if *st == 0 { 'H' } else { char::from_digit(*st as u32, 36).unwrap_or('?').to_ascii_uppercase() };
            }
            let c = cells.get(&p).copied().unwrap_or_else(|| match &m.bg {
                Tape::Blank(c) => *c,
                Tape::Pattern(v) => v[p.rem_euclid(v.len() as i64) as usize],
                Tape::Random(seed, kk) => (hash_pos(p, *seed) % *kk as u64) as u8,
            });
            shade(c, k)
        }).collect::<String>()
    }).collect::<Vec<_>>().join("\n")
}

fn parse_explicit(t: &str) -> Option<Tm> {
    let mut rules = HashMap::new();
    let mut s = 1u8;
    let mut k = 2u8;
    let caps: Vec<_> = re!(r"\{\s*(\d+)\s*,\s*(\d+|_)\s*\}\s*->\s*\{\s*(\d+)\s*,\s*(\d+)\s*,\s*(r|l|1|-1)\s*\}").captures_iter(t).collect();
    if caps.is_empty() {
        return None;
    }
    let mut parsed = Vec::new();
    for m in &caps {
        let st: u8 = m[1].parse().ok()?;
        let col: Option<u8> = m[2].parse().ok();
        let (ns, nc): (u8, u8) = (m[3].parse().ok()?, m[4].parse().ok()?);
        let mv: i8 = if matches!(&m[5], "r" | "1") { 1 } else { -1 };
        s = s.max(st).max(ns);
        k = k.max(col.unwrap_or(0) + 1).max(nc + 1);
        parsed.push((st, col, ns, nc, mv));
    }
    // specific rules take precedence over wildcards
    for (st, col, ns, nc, mv) in parsed.iter().filter(|p| p.1.is_none()) {
        for c in 0..k {
            rules.insert((*st, c), (*ns, *nc, *mv));
        }
        let _ = col;
    }
    for (st, col, ns, nc, mv) in parsed.iter().filter(|p| p.1.is_some()) {
        rules.insert((*st, col.unwrap()), (*ns, *nc, *mv));
    }
    Some(Tm { s, k, rules })
}

/// Busy-beaver champions (most marks written before halting), state 0 = halt.
fn busy_beaver(s: u8, k: u8) -> Option<(Tm, &'static str)> {
    // rows: (state, colour, new state, write, move)
    let spec: &[(u8, u8, u8, u8, i8)] = match (s, k) {
        (1, 2) => &[(1, 0, 0, 1, 1), (1, 1, 0, 1, 1)],
        (2, 2) => &[(1, 0, 2, 1, 1), (1, 1, 2, 1, -1), (2, 0, 1, 1, -1), (2, 1, 0, 1, 1)],
        (3, 2) => &[(1, 0, 2, 1, 1), (1, 1, 3, 1, -1), (2, 0, 1, 1, -1), (2, 1, 2, 1, 1), (3, 0, 2, 1, -1), (3, 1, 0, 1, 1)],
        (4, 2) => &[(1, 0, 2, 1, 1), (1, 1, 2, 1, -1), (2, 0, 1, 1, -1), (2, 1, 3, 0, -1), (3, 0, 0, 1, 1), (3, 1, 4, 1, -1), (4, 0, 4, 1, 1), (4, 1, 1, 0, 1)],
        (5, 2) => &[(1, 0, 2, 1, 1), (1, 1, 3, 1, -1), (2, 0, 3, 1, 1), (2, 1, 2, 1, 1), (3, 0, 4, 1, 1), (3, 1, 5, 0, -1), (4, 0, 1, 1, -1), (4, 1, 4, 1, -1), (5, 0, 0, 1, 1), (5, 1, 1, 0, -1)],
        (2, 3) => &[(1, 0, 2, 1, 1), (1, 1, 2, 2, -1), (1, 2, 0, 1, 1), (2, 0, 1, 2, -1), (2, 1, 2, 2, 1), (2, 2, 2, 1, -1)],
        _ => return None,
    };
    let note = match (s, k) {
        (1, 2) => "Σ(1) = 1 mark, S(1) = 1 step",
        (2, 2) => "Σ(2) = 4 marks, S(2) = 6 steps",
        (3, 2) => "Σ(3) = 6 marks, S(3) = 21 steps (the step record is held by a different machine)",
        (4, 2) => "Σ(4) = 13 marks, S(4) = 107 steps (Brady, 1983)",
        (5, 2) => "Σ(5) = 4098 marks, S(5) = 47,176,870 steps (Marxen and Buntrock, 1989; proved optimal by the bbchallenge project, 2024)",
        (2, 3) => "Σ(2,3) = 9 marks, S(2,3) = 38 steps",
        _ => "",
    };
    let rules = spec.iter().map(|&(a, b, c, d, e)| ((a, b), (c, d, e))).collect();
    Some((Tm { s, k, rules }, note))
}

fn turing(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    if !re!(r"\btm\b|turing|wolfram 2,\s*3|busy beaver").is_match(&l) {
        return None;
    }
    if l.contains("compress") {
        return None;
    }
    let mut seed_used = None;
    // the machine
    let (tm, name, note): (Tm, String, Option<String>) = if let Some(m) = re!(r"busy beaver,?\s*(\d)[- ]states?(?:,?\s*(\d)[- ]colou?rs?(?:\s*\(?\s*symbols?\)?)?)?").captures(&l) {
        let s: u8 = m[1].parse().ok()?;
        let k: u8 = m.get(2).and_then(|x| x.as_str().parse().ok()).unwrap_or(2);
        match busy_beaver(s, k) {
            Some((tm, note)) => (tm, format!("{s}-state {k}-colour busy beaver"), Some(note.to_string())),
            None => {
                let known = match (s, k) {
                    (6, 2) => "Σ(6) > 2↑↑↑5 marks (Kropitz and others, 2022); the exact value is unknown",
                    (3, 3) => "Σ(3,3) ≥ 374,676,383 marks; the exact value is unknown",
                    (2, 4) => "Σ(2,4) ≥ 2050 marks, S(2,4) ≥ 3,932,964 steps; the exact values are unknown",
                    _ => return None,
                };
                return Some(Answer::new("busy beaver", vec![pod("Input", "Input interpretation", text(format!("{s}-state {k}-colour busy beaver"))), primary(pod("Result", "Known bounds", text(known)))]));
            }
        }
    } else if l.contains("wolfram 2,3") || l.contains("wolfram 2, 3") {
        (Tm::from_number(&BigUint::from(596440u32), 2, 3)?, "Wolfram 2,3 Turing machine (rule 596440)".into(), Some("the smallest universal Turing machine (universality proved by Alex Smith, 2007)".into()))
    } else if l.contains("->") {
        let tm = parse_explicit(&l)?;
        (tm.clone(), format!("{}-state {}-colour Turing machine with the given rules", tm.s, tm.k), None)
    } else if let Some(m) = re!(r"(?:(\d+)[- ]?states?,?\s*(\d+)[- ]?colou?rs?|(\d),\s*(\d))?\s*(?:turing machine|tm)(?:\s+rule)?\s+(\d+)|random\s+(?:(\d+)[- ]?states?\s+(\d+)[- ]?colou?rs?\s+)?turing machine").captures(&l) {
        if let Some(n) = m.get(5) {
            let n: BigUint = n.as_str().parse().ok()?;
            let given = m.get(1).or(m.get(3)).zip(m.get(2).or(m.get(4))).and_then(|(a, b)| Some((a.as_str().parse::<u8>().ok()?, b.as_str().parse::<u8>().ok()?)));
            let (s, k) = match given {
                Some(sk) => sk,
                None => {
                    // the smallest machine whose numbering reaches n
                    let need_colors = re!(r"(?:all|single|=)\s*(\d)s?\b").captures(&l).and_then(|c| c[1].parse::<u8>().ok()).map(|c| c + 1).unwrap_or(2);
                    let cands = [(2u8, 2u8), (2, 3), (3, 2), (2, 4), (3, 3), (2, 5), (4, 2), (3, 4), (4, 3)];
                    *cands.iter().find(|(s, k)| *k >= need_colors && n < BigUint::from(2u32 * *s as u32 * *k as u32).pow(*s as u32 * *k as u32))?
                }
            };
            if !(1..=6).contains(&s) || !(2..=6).contains(&k) {
                return None;
            }
            (Tm::from_number(&n, s, k)?, format!("{s}-state {k}-colour Turing machine rule {n}"), None)
        } else {
            let s: u8 = m.get(6).and_then(|x| x.as_str().parse().ok()).unwrap_or(2);
            let k: u8 = m.get(7).and_then(|x| x.as_str().parse().ok()).unwrap_or(2);
            if !(1..=6).contains(&s) || !(2..=6).contains(&k) {
                return None;
            }
            let seed = time_seed();
            let mut rng = Rng::new(seed);
            let total = BigUint::from(2u32 * s as u32 * k as u32).pow(s as u32 * k as u32);
            let mut n = BigUint::zero();
            for _ in 0..4 {
                n = (n << 64u32) + BigUint::from(rng.next());
            }
            n %= total;
            seed_used = Some(seed);
            (Tm::from_number(&n, s, k)?, format!("random {s}-state {k}-colour Turing machine: rule {n}"), None)
        }
    } else {
        return None;
    };
    // the tape
    let mut cyclic = None;
    let mut cells = HashMap::new();
    let mut head = 0i64;
    let mut state = 1u8;
    let mut bg = Tape::Blank(0);
    let mut tape_desc = "blank tape (all 0)".to_string();
    if l.contains("random tape") {
        let seed = seed_used.unwrap_or_else(time_seed);
        bg = Tape::Random(seed, tm.k);
        tape_desc = "random tape".into();
    }
    if let Some(m) = re!(r"tape\s*=\s*all\s*(\d)s?").captures(&l) {
        bg = Tape::Blank(m[1].parse().ok()?);
        tape_desc = format!("tape of all {}s", &m[1]);
    }
    if let Some(m) = re!(r"repeating\s+(\d+)").captures(&l) {
        bg = Tape::Pattern(m[1].bytes().map(|b| b - b'0').collect());
        tape_desc = format!("tape repeating {}", &m[1]);
    }
    if let Some(m) = re!(r"central\s+(\d+)").captures(&l) {
        let block: Vec<u8> = m[1].bytes().map(|b| b - b'0').collect();
        let off = block.len() as i64 / 2;
        for (i, c) in block.iter().enumerate() {
            cells.insert(i as i64 - off, *c);
        }
        tape_desc = format!("{tape_desc}, central block {}", &m[1]);
    }
    if let Some(m) = re!(r"tape\s*=\s*single\s*(\d)").captures(&l) {
        cells.insert(0, m[1].parse().ok()?);
        tape_desc = format!("blank tape with a single {}", &m[1]);
    }
    if let Some(m) = re!(r"width\s*=?\s*(\d+)\s*(?:tape)?|on\s+width\s+(\d+)\s+tape").captures(&l) {
        let w: i64 = m.get(1).or(m.get(2))?.as_str().parse().ok()?;
        if !(1..=200).contains(&w) {
            return None;
        }
        cyclic = Some(w);
        tape_desc = format!("{tape_desc}, cyclic tape of width {w}");
    }
    if let Some(m) = re!(r"head\s+position\s*=\s*(-?\d+)").captures(&l) {
        head = m[1].parse().ok()?;
    }
    if let Some(m) = re!(r"head\s+state\s*=\s*(\d+)").captures(&l) {
        state = m[1].parse().ok()?;
    }
    if tm.k > 6 || tm.rules.values().any(|(_, c, _)| *c >= tm.k) {
        return None;
    }
    // which steps
    let is_bb = l.contains("busy beaver");
    let mut steps = if is_bb { 100_000_000 } else { 50 };
    let mut keep: Box<dyn Fn(usize) -> bool> = Box::new(|_| true);
    let mut window = String::new();
    if let Some(m) = re!(r"(?:for\s+)?(\d+)\s+steps").captures(&l) {
        steps = m[1].parse().ok()?;
    }
    if let Some(m) = re!(r"steps?\s+(\d+)\s+(?:thru|through|to|-)\s+(\d+)").captures(&l) {
        let (a, b): (usize, usize) = (m[1].parse().ok()?, m[2].parse().ok()?);
        steps = b;
        keep = Box::new(move |t| t >= a && t <= b);
        window = format!(", steps {a} to {b}");
    } else if let Some(m) = re!(r"last\s+(\d+)\s+of\s+(\d+)\s+steps").captures(&l) {
        let (a, b): (usize, usize) = (m[1].parse().ok()?, m[2].parse().ok()?);
        steps = b;
        keep = Box::new(move |t| t + a > b);
        window = format!(", last {a} of {b} steps");
    } else if let Some(m) = re!(r"every\s+(\d+)(?:st|nd|rd|th)?\s+step").captures(&l) {
        let e: usize = m[1].parse().ok()?;
        steps = steps.max(e * 50);
        keep = Box::new(move |t| t % e.max(1) == 0);
        window = format!(", every {e}th step");
    }
    if steps > 200_000_000 {
        return None;
    }
    let mut mach = Machine { tm: &tm, bg, cyclic, cells, head, state };
    let display_bb = is_bb && tm.s >= 5;
    let keep_fn: &dyn Fn(usize) -> bool = if display_bb { &|t| t < 60 } else { &*keep };
    let mut run = run_tm(&mut mach, steps, keep_fn);
    if display_bb && run.rows.len() > 60 {
        run.rows.truncate(60);
    }
    let shown: Vec<_> = if run.rows.len() > 120 && !display_bb { run.rows.len() - 120..run.rows.len() } else { 0..run.rows.len().min(120) }.collect();
    let run_shown = Run { rows: shown.iter().map(|&i| run.rows[i].clone()).collect(), halted: run.halted };
    let pic = tm_picture(&mach, &run_shown);
    let marks = mach.cells.values().filter(|&&c| c != 0).count();
    let status = match run.halted {
        Some(t) => format!("halts after {t} steps with {marks} non-zero cells on the tape"),
        None => format!("still running after {steps} steps; head at position {}, state {}", mach.head, mach.state),
    };
    let pic_title = if display_bb { "First 60 steps (head shown as its state; H = halted)" } else { "Evolution (head shown as its state; H = halted)" };
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{name}, {tape_desc}{window}")))];
    if is_bb {
        pods.push(primary(pod("Status", "Result", text(status))));
        pods.push(pod("Evolution", pic_title, text(pic)));
    } else {
        pods.push(primary(pod("Evolution", pic_title, text(pic))));
        pods.push(pod("Status", "Status", text(status)));
    }
    pods.push(pod("Rule", "Rule", table(Some(vec!["case", "action"]), tm.rule_text())));
    if let Some(n) = note {
        pods.push(pod("Note", "Notes", text(n)));
    }
    Some(Answer::new("Turing machine", pods))
}

// ---------------------------------------------------------------------------
// substitution systems
// ---------------------------------------------------------------------------

fn substitution(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    let m = re!(r"^(?:string\s+)?substitution system\s+(.+?)(?:,?\s*(\d+)\s+steps)?$").captures(&l)?;
    let rules: Vec<(String, String)> = re!(r"([0-9a-z]+)\s*(?:->|→)\s*([0-9a-z]*)").captures_iter(&m[1]).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    if rules.is_empty() {
        return None;
    }
    let steps: usize = m.get(2).and_then(|x| x.as_str().parse().ok()).unwrap_or(8).min(40);
    let init = rules[0].0.clone();
    let apply = |s: &str| -> String {
        let b = s.as_bytes();
        let mut out = String::new();
        let mut i = 0;
        while i < b.len() {
            if let Some((lhs, rhs)) = rules.iter().find(|(lhs, _)| s[i..].starts_with(lhs.as_str())) {
                out.push_str(rhs);
                i += lhs.len();
            } else {
                out.push(b[i] as char);
                i += 1;
            }
        }
        out
    };
    let mut states = vec![init];
    for _ in 0..steps {
        let next = apply(states.last().unwrap());
        if next.len() > 4000 {
            break;
        }
        states.push(next);
    }
    let rowsv: Vec<Vec<Content>> = states.iter().enumerate().map(|(i, s)| {
        let shown = if s.len() > 80 { format!("{}… ({} symbols)", &s[..80], s.len()) } else { s.clone() };
        vec![text(i.to_string()), text(shown), text(s.len().to_string())]
    }).collect();
    let lens: Vec<String> = states.iter().map(|s| s.len().to_string()).collect();
    Some(Answer::new("substitution system", vec![
        pod("Input", "Input interpretation", text(format!("substitution system {} starting from {}", rules.iter().map(|(a, b)| format!("{a} → {b}")).collect::<Vec<_>>().join(", "), states[0]))),
        primary(pod("Evolution", "Evolution", table(Some(vec!["step", "string", "length"]), rowsv))),
        pod("Lengths", "Lengths", text(lens.join(", "))),
    ]))
}

// ---------------------------------------------------------------------------
// fractals drawn as text
// ---------------------------------------------------------------------------

/// (names, title, Hausdorff dimension (exact, decimal), facts)
const FRACTALS: &[(&[&str], &str, &str, f64, &str)] = &[
    (&["koch snowflake", "koch curve", "von koch snowflake"], "Koch snowflake", "log 4 / log 3", 1.2618595071429148, "each step replaces every segment by 4 segments of 1/3 the length; perimeter after n steps 3s(4/3)ⁿ → ∞; enclosed area → (8/5)·(√3/4)s² = 2√3s²/5"),
    (&["minkowski sausage", "minkowski island", "quadratic koch curve"], "Minkowski sausage", "log 8 / log 4 = 3/2", 1.5, "each segment is replaced by 8 segments of 1/4 the length"),
    (&["cesaro fractal", "cesaro curve", "cesàro fractal"], "Cesàro fractal (85° variant of the Koch curve)", "log 4 / log(2(1 + cos 85°))", 1.7848, "a Koch-type curve whose spikes meet at 85° instead of 60°"),
    (&["sierpinski tetrahedron", "tetrix", "sierpinski pyramid"], "Sierpiński tetrahedron", "log 4 / log 2 = 2", 2.0, "4 copies at scale 1/2; the surface area stays constant while the volume tends to 0"),
    (&["menger sponge"], "Menger sponge", "log 20 / log 3", 2.7268330278608417, "20 copies at scale 1/3; after n steps volume (20/27)ⁿ → 0 and surface area → ∞"),
    (&["sierpinski carpet"], "Sierpiński carpet", "log 8 / log 3", 1.892789260714372, "8 copies at scale 1/3; area (8/9)ⁿ → 0"),
    (&["sierpinski triangle", "sierpinski gasket", "sierpinski sieve"], "Sierpiński triangle", "log 3 / log 2", 1.5849625007211562, "3 copies at scale 1/2; also Pascal's triangle mod 2 and rule 90"),
    (&["cantor set", "cantor dust"], "Cantor set", "log 2 / log 3", 0.6309297535714574, "uncountable, measure zero, totally disconnected"),
    (&["dragon curve", "heighway dragon"], "Heighway dragon curve", "2 (the boundary has dimension ≈ 1.52363)", 2.0, "fold a strip in half repeatedly and open each fold to 90°; copies tile the plane"),
    (&["levy c curve", "lévy c curve", "levy curve"], "Lévy C curve", "2", 2.0, "each segment is replaced by two segments at 45°, each 1/√2 as long"),
    (&["peano curve"], "Peano curve", "2", 2.0, "the first space-filling curve (Peano, 1890): 9 copies at scale 1/3"),
    (&["hilbert curve", "hilbert-2 curve", "hilbert 2 curve"], "Hilbert curve", "2", 2.0, "space-filling curve (Hilbert, 1891); step n visits all 4ⁿ cells of a 2ⁿ × 2ⁿ grid, length 2ⁿ − 2⁻ⁿ"),
    (&["pythagoras tree"], "Pythagoras tree", "2 (for the 45° tree)", 2.0, "squares on the legs of right isosceles triangles, repeated; the 45° tree fits in a 6 × 4 box"),
    (&["mandelbrot set"], "Mandelbrot set", "2 (boundary, Shishikura 1998)", 2.0, "c with bounded orbit of z ↦ z² + c from 0; area ≈ 1.5065918849"),
    (&["multibrot set"], "Multibrot set", "2 (boundary)", 2.0, "c with bounded orbit of z ↦ z^d + c; it has d − 1 fold rotational symmetry"),
    (&["apollonian gasket"], "Apollonian gasket", "≈ 1.305688", 1.305688, "circles packed into the gaps between three mutually tangent circles"),
    (&["curlicue fractal"], "curlicue fractal", "depends on the irrational s", f64::NAN, "the path of unit steps turning by 2πs·n at step n; with s = golden ratio it stays bounded and self-similar"),
];

fn takagi(x: &num_rational::BigRational, terms: Option<usize>) -> num_rational::BigRational {
    use num_rational::BigRational;
    use num_traits::One;
    // s(t) = distance to the nearest integer; x_n = 2ⁿ x mod 1 is eventually periodic
    let frac = |t: &BigRational| t - t.floor();
    let s = |t: &BigRational| {
        let f = frac(t);
        let h = BigRational::new(1.into(), 2.into());
        if f <= h { f } else { BigRational::one() - f }
    };
    let two = BigRational::from_integer(2.into());
    let mut seen: Vec<BigRational> = Vec::new();
    let mut xn = frac(x);
    let limit = terms.unwrap_or(10_000);
    loop {
        if terms.is_none() {
            if let Some(p) = seen.iter().position(|v| *v == xn) {
                // T = Σ_{n<p} s_n/2ⁿ + (Σ_{p≤n<len} s_n/2ⁿ) / (1 − 2^{−L})
                let len = seen.len();
                let mut head = BigRational::from_integer(0.into());
                let mut cyc = BigRational::from_integer(0.into());
                let mut w = BigRational::one();
                for (i, v) in seen.iter().enumerate() {
                    let t = s(v) * &w;
                    if i < p { head += t } else { cyc += t }
                    w /= &two;
                }
                let l = (len - p) as i32;
                let denom = BigRational::one() - BigRational::new(1.into(), num_bigint::BigInt::from(2).pow(l as u32));
                return head + cyc / denom;
            }
        }
        if seen.len() >= limit {
            break;
        }
        seen.push(xn.clone());
        xn = frac(&(&xn * &two));
    }
    let mut total = BigRational::from_integer(0.into());
    let mut w = BigRational::one();
    for v in &seen {
        total += s(v) * &w;
        w /= &two;
    }
    total
}

fn fractals(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    // Takagi (blancmange) function
    if let Some(m) = re!(r"^(?:takagi|blancmange)(?:\s+function)?\s*(?:\(|\[|of\s+|at\s+)\s*(\d+)\s*/\s*(\d+)\s*[)\]]?$").captures(&l) {
        let (a, b): (num_bigint::BigInt, num_bigint::BigInt) = (m[1].parse().ok()?, m[2].parse().ok()?);
        if b == num_bigint::BigInt::from(0) || b.bits() > 40 {
            return None;
        }
        let x = num_rational::BigRational::new(a, b);
        let t = takagi(&x, None);
        let dec = num_traits::ToPrimitive::to_f64(&t).unwrap_or(f64::NAN);
        return Some(Answer::new("Takagi function", vec![
            pod("Input", "Input interpretation", text(format!("Takagi (blancmange) function at {x}"))),
            primary(pod("Result", "Exact value", text(t.to_string()))),
            pod("Decimal", "Decimal approximation", text(crate::print::fmt_float(dec, 15))),
            pod("Def", "Definition", text("T(x) = Σ_{n≥0} s(2ⁿx)/2ⁿ, where s(x) is the distance from x to the nearest integer")),
        ]));
    }
    if let Some(m) = re!(r"^(?:takagi|blancmange)(?:\s+function)?(?:\s*,?\s*n\s*=\s*(\d+))?$").captures(&l) {
        let n: usize = m.get(1).and_then(|x| x.as_str().parse().ok()).unwrap_or(10).min(30);
        let pts: Vec<Vec<Content>> = [1u32, 2, 3, 4, 5, 6, 7, 8].iter().map(|&k| {
            let x = num_rational::BigRational::new(k.into(), 16.into());
            let v = takagi(&x, Some(n + 1));
            vec![text(format!("{k}/16")), text(v.to_string())]
        }).collect();
        return Some(Answer::new("blancmange function", vec![
            pod("Input", "Input interpretation", text(format!("blancmange function, partial sum to n = {n}"))),
            primary(pod("Result", "Definition", text(format!("T_{n}(x) = Σ_{{k=0}}^{{{n}}} s(2ᵏx)/2ᵏ, s(x) = distance from x to the nearest integer")))),
            pod("Values", "Values", table(Some(vec!["x", &format!("T_{n}(x)")]), pts)),
            pod("Props", "Properties", text("the limit is continuous everywhere and differentiable nowhere (Takagi, 1901); its maximum is 2/3")),
        ]));
    }
    let key = re!(r"(?:,\s*)?(?:n|d|order|level|steps?)\s*=\s*\d+$").replace(&l, "").trim().to_string();
    if let Some((_, title, dim, dec, facts)) = FRACTALS.iter().find(|f| f.0.contains(&key.as_str())) {
        if !matches!(key.as_str(), "sierpinski carpet" | "sierpinski triangle" | "sierpinski gasket" | "hafermann carpet") {
            let mut rows = vec![vec![text("Hausdorff dimension"), text(if dec.is_nan() { dim.to_string() } else { format!("{dim} ≈ {}", crate::stats::fmt(*dec)) })], vec![text("construction"), text(*facts)]];
            if let Some(m) = re!(r"d\s*=\s*(\d+)").captures(&l) {
                rows.push(vec![text("exponent"), text(format!("z ↦ z^{} + c, with {}-fold symmetry", &m[1], m[1].parse::<u32>().unwrap_or(2).saturating_sub(1)))]);
            }
            return Some(Answer::new("fractal", vec![pod("Input", "Input interpretation", text(*title)), primary(pod("Result", "Properties", table(None, rows)))]));
        }
    }
    let (name, grid): (&str, Vec<Vec<u8>>) = match l.as_str() {
        "hafermann carpet" | "haferman carpet" => {
            // black → the 3×3 pattern 1 0 1 / 0 1 0 / 1 0 1, white → all black
            let mut g = vec![vec![1u8]];
            for _ in 0..3 {
                let n = g.len();
                let mut h = vec![vec![0u8; n * 3]; n * 3];
                for i in 0..n {
                    for j in 0..n {
                        for a in 0..3 {
                            for b in 0..3 {
                                h[i * 3 + a][j * 3 + b] = if g[i][j] == 1 { ((a + b) % 2 == 0) as u8 } else { 1 };
                            }
                        }
                    }
                }
                g = h;
            }
            ("Hafermann carpet", g)
        }
        "sierpinski carpet" => {
            let n = 27;
            ("Sierpiński carpet", (0..n).map(|i| (0..n).map(|j| {
                let (mut a, mut b) = (i, j);
                while a > 0 || b > 0 {
                    if a % 3 == 1 && b % 3 == 1 {
                        return 0;
                    }
                    a /= 3;
                    b /= 3;
                }
                1
            }).collect()).collect())
        }
        "sierpinski triangle" | "sierpinski gasket" | "sierpinski sieve" => ("Sierpiński triangle", (0..32).map(|i| (0..32).map(|j| (j <= i && (i & j) == j) as u8).collect()).collect()),
        _ => return None,
    };
    let mut pods = vec![pod("Input", "Input interpretation", text(name)), primary(pod("Image", "Image (after a few iterations)", text(picture(&grid, 2))))];
    if let Some((_, _, dim, dec, facts)) = FRACTALS.iter().find(|f| f.0.contains(&l.as_str())) {
        pods.push(pod("Props", "Properties", table(None, vec![vec![text("Hausdorff dimension"), text(format!("{dim} ≈ {}", crate::stats::fmt(*dec)))], vec![text("construction"), text(*facts)]])));
    }
    Some(Answer::new("fractal", pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    cellular(q).or_else(|| turing(q)).or_else(|| substitution(q)).or_else(|| fractals(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wolfram_two_three_decodes() {
        let tm = Tm::from_number(&BigUint::from(596440u32), 2, 3).unwrap();
        assert_eq!(tm.rules[&(1, 2)], (1, 1, -1));
        assert_eq!(tm.rules[&(1, 0)], (2, 1, 1));
        assert_eq!(tm.rules[&(2, 1)], (2, 2, 1));
        assert_eq!(tm.rules[&(2, 0)], (1, 2, -1));
    }

    #[test]
    fn busy_beavers_score() {
        for (s, k, marks, steps) in [(2u8, 2u8, 4usize, 6usize), (3, 2, 6, 13), (4, 2, 13, 107), (2, 3, 9, 38)] {
            let (tm, _) = busy_beaver(s, k).unwrap();
            let mut m = Machine { tm: &tm, bg: Tape::Blank(0), cyclic: None, cells: HashMap::new(), head: 0, state: 1 };
            let run = run_tm(&mut m, 1000, &|_| false);
            let n = m.cells.values().filter(|&&c| c != 0).count();
            assert_eq!((n, run.halted), (marks, Some(steps)), "{s},{k}");
        }
    }

    #[test]
    fn rule_30_center_column() {
        let ca = Ca::from_number(&BigUint::from(30u32), 2, 1, false).unwrap();
        let rows = ca.evolve(10, single_cell(21));
        let centre: String = rows.iter().map(|r| (b'0' + r[10]) as char).collect();
        assert_eq!(centre, "11011100110");
    }
}
