//! Set algebra (Venn regions, identities, subsets, finite sets, Cartesian
//! products), cardinal arithmetic with alephs and beths, numbered Boolean
//! functions, minterms and maxterms, normal forms and minimisation,
//! injectivity and surjectivity of real functions, and divide-and-conquer
//! recurrences (master theorem / Akra–Bazzi).

use super::nl::ex;
use super::*;
use crate::expr::*;
use crate::expr::Expr::*;
use crate::print::text as ptext;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn input(s: String) -> Pod {
    pod("Input", "Input interpretation", text(s))
}

// ---------------------------------------------------------------------------
// Boolean minimisation (shared by sets and logic)
// ---------------------------------------------------------------------------

/// A cube: for each variable, Some(true/false) or None (don't care).
type Cube = Vec<Option<bool>>;

fn cube_covers(c: &Cube, m: usize, n: usize) -> bool {
    c.iter().enumerate().all(|(j, v)| match v {
        None => true,
        Some(b) => ((m >> (n - 1 - j)) & 1 == 1) == *b,
    })
}

/// Minimal sum of products for the on-set `on` over n ≤ 6 variables
/// (all prime implicants, then exact cover by increasing size).
fn minimise(on: &[bool], n: usize) -> Vec<Cube> {
    if on.iter().all(|&b| !b) {
        return vec![];
    }
    let total = 3usize.pow(n as u32);
    let mut implicants: Vec<Cube> = Vec::new();
    for code in 0..total {
        let mut c = Vec::with_capacity(n);
        let mut t = code;
        for _ in 0..n {
            c.push(match t % 3 {
                0 => Some(false),
                1 => Some(true),
                _ => None,
            });
            t /= 3;
        }
        if (0..on.len()).filter(|&m| cube_covers(&c, m, n)).all(|m| on[m]) {
            implicants.push(c);
        }
    }
    // keep the prime ones (not contained in a larger implicant)
    let contains = |big: &Cube, small: &Cube| big.iter().zip(small).all(|(b, s)| b.is_none() || b == s);
    let primes: Vec<Cube> = implicants.iter().filter(|c| !implicants.iter().any(|d| d != *c && contains(d, c))).cloned().collect();
    let minterms: Vec<usize> = (0..on.len()).filter(|&m| on[m]).collect();
    // exact cover by brute force over subsets of increasing size (primes are few for n ≤ 4)
    if primes.len() <= 16 {
        for size in 1..=primes.len() {
            let mut best: Option<Vec<Cube>> = None;
            let mut idx: Vec<usize> = (0..size).collect();
            loop {
                if minterms.iter().all(|&m| idx.iter().any(|&i| cube_covers(&primes[i], m, n))) {
                    let pick: Vec<Cube> = idx.iter().map(|&i| primes[i].clone()).collect();
                    let lits = |v: &Vec<Cube>| v.iter().map(|c| c.iter().filter(|x| x.is_some()).count()).sum::<usize>();
                    if best.as_ref().map(|b| lits(&pick) < lits(b)).unwrap_or(true) {
                        best = Some(pick);
                    }
                }
                // next combination
                let mut i = size;
                while i > 0 && idx[i - 1] == primes.len() - size + i - 1 {
                    i -= 1;
                }
                if i == 0 {
                    break;
                }
                idx[i - 1] += 1;
                for k in i..size {
                    idx[k] = idx[k - 1] + 1;
                }
            }
            if let Some(b) = best {
                return b;
            }
        }
    }
    // greedy fallback
    let mut left: Vec<usize> = minterms;
    let mut out = Vec::new();
    while !left.is_empty() {
        let best = primes.iter().max_by_key(|c| left.iter().filter(|&&m| cube_covers(c, m, n)).count()).unwrap().clone();
        left.retain(|&m| !cube_covers(&best, m, n));
        out.push(best);
    }
    out
}

fn show_sop(cubes: &[Cube], names: &[String], and: &str, or: &str, not: &dyn Fn(&str) -> String, tru: &str, fal: &str) -> String {
    if cubes.is_empty() {
        return fal.into();
    }
    if cubes.iter().any(|c| c.iter().all(|x| x.is_none())) {
        return tru.into();
    }
    let terms: Vec<String> = cubes.iter().map(|c| {
        let lits: Vec<String> = c.iter().enumerate().filter_map(|(j, v)| v.map(|b| if b { names[j].clone() } else { not(&names[j]) })).collect();
        if lits.len() > 1 && cubes.len() > 1 { format!("({})", lits.join(and)) } else { lits.join(and) }
    }).collect();
    terms.join(or)
}

// ---------------------------------------------------------------------------
// set algebra
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum SetE {
    Var(usize),
    Universe,
    Empty,
    Not(Box<SetE>),
    And(Box<SetE>, Box<SetE>),
    Or(Box<SetE>, Box<SetE>),
    Diff(Box<SetE>, Box<SetE>),
    Xor(Box<SetE>, Box<SetE>),
}

impl SetE {
    fn eval(&self, m: usize, n: usize) -> bool {
        match self {
            SetE::Var(j) => (m >> (n - 1 - j)) & 1 == 1,
            SetE::Universe => true,
            SetE::Empty => false,
            SetE::Not(a) => !a.eval(m, n),
            SetE::And(a, b) => a.eval(m, n) && b.eval(m, n),
            SetE::Or(a, b) => a.eval(m, n) || b.eval(m, n),
            SetE::Diff(a, b) => a.eval(m, n) && !b.eval(m, n),
            SetE::Xor(a, b) => a.eval(m, n) != b.eval(m, n),
        }
    }
}

struct SetParser {
    toks: Vec<String>,
    pos: usize,
    names: Vec<String>,
}

impl SetParser {
    fn new(s: &str) -> Option<SetParser> {
        let s = s.replace('∪', " union ").replace('∩', " intersect ").replace('\\', " minus ").replace('Δ', " symdiff ").replace('∅', " emptyset ");
        let mut toks = Vec::new();
        let mut cur = String::new();
        for ch in s.chars() {
            if ch.is_alphanumeric() || ch == '_' {
                cur.push(ch);
            } else {
                if !cur.is_empty() {
                    toks.push(std::mem::take(&mut cur));
                }
                if "()'~".contains(ch) {
                    toks.push(ch.to_string());
                } else if !ch.is_whitespace() {
                    return None;
                }
            }
        }
        if !cur.is_empty() {
            toks.push(cur);
        }
        const KW: &[&str] = &["union", "cup", "or", "symdiff", "xor", "minus", "without", "intersect", "intersection", "cap", "and", "complement", "not", "emptyset", "empty", "universe"];
        for t in toks.iter_mut() {
            let lw = t.to_lowercase();
            if KW.contains(&lw.as_str()) {
                *t = lw;
            }
        }
        Some(SetParser { toks, pos: 0, names: Vec::new() })
    }
    fn peek(&self) -> Option<&str> {
        self.toks.get(self.pos).map(|s| s.as_str())
    }
    fn union(&mut self) -> Option<SetE> {
        let mut a = self.diff()?;
        loop {
            match self.peek() {
                Some("union" | "cup" | "or") => {
                    self.pos += 1;
                    a = SetE::Or(Box::new(a), Box::new(self.diff()?));
                }
                Some("symdiff" | "xor") => {
                    self.pos += 1;
                    a = SetE::Xor(Box::new(a), Box::new(self.diff()?));
                }
                _ => return Some(a),
            }
        }
    }
    fn diff(&mut self) -> Option<SetE> {
        let mut a = self.inter()?;
        while matches!(self.peek(), Some("minus" | "without")) {
            self.pos += 1;
            a = SetE::Diff(Box::new(a), Box::new(self.inter()?));
        }
        Some(a)
    }
    fn inter(&mut self) -> Option<SetE> {
        let mut a = self.unary()?;
        while matches!(self.peek(), Some("intersect" | "intersection" | "cap" | "and")) {
            self.pos += 1;
            a = SetE::And(Box::new(a), Box::new(self.unary()?));
        }
        Some(a)
    }
    fn unary(&mut self) -> Option<SetE> {
        let t = self.peek()?.to_string();
        self.pos += 1;
        let mut e = match t.as_str() {
            "complement" | "not" | "~" => SetE::Not(Box::new(self.unary()?)),
            "(" => {
                let e = self.union()?;
                if self.peek() != Some(")") {
                    return None;
                }
                self.pos += 1;
                e
            }
            "emptyset" | "empty" => SetE::Empty,
            "universe" => SetE::Universe,
            w if w.len() <= 2 && w.chars().next()?.is_alphabetic() => {
                let j = match self.names.iter().position(|n| n == w) {
                    Some(j) => j,
                    None => {
                        self.names.push(w.to_string());
                        self.names.len() - 1
                    }
                };
                SetE::Var(j)
            }
            _ => return None,
        };
        while self.peek() == Some("'") {
            self.pos += 1;
            e = SetE::Not(Box::new(e));
        }
        Some(e)
    }
}

fn parse_set(s: &str, names: &mut Vec<String>) -> Option<SetE> {
    let mut p = SetParser::new(s)?;
    p.names = std::mem::take(names);
    let e = p.union()?;
    if p.pos != p.toks.len() {
        return None;
    }
    *names = p.names;
    Some(e)
}

fn has_set_op(s: &str) -> bool {
    re!(r"(?i)\b(union|intersect|intersection|complement|minus|symdiff)\b|[∪∩']").is_match(s)
}

fn region_name(m: usize, names: &[String]) -> String {
    let n = names.len();
    names.iter().enumerate().map(|(j, v)| if (m >> (n - 1 - j)) & 1 == 1 { v.clone() } else { format!("{v}'") }).collect::<Vec<_>>().join(" ∩ ")
}

fn sets(raw: &str) -> Option<Answer> {
    let l = raw.trim();
    // literal finite sets
    if let Some(m) = re!(r"(?i)^(?:is )?(?:the set )?\{([^{}]*)\}\s*(=|==|subset(?: of)?|⊆|⊂)\s*\{([^{}]*)\}$").captures(l) {
        let parse = |t: &str| -> Vec<String> {
            let mut v: Vec<String> = t.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
            v.sort();
            v.dedup();
            v
        };
        let (a, b) = (parse(&m[1]), parse(&m[3]));
        let eq_ = m[2].starts_with('=');
        let res = if eq_ { a == b } else { a.iter().all(|x| b.contains(x)) };
        let why = if eq_ {
            format!("as sets, order and repetition do not matter: both are {{{}}}", a.join(", "))
        } else {
            format!("every element of {{{}}} {} in {{{}}}", a.join(", "), if res { "lies" } else { "does not lie" }, b.join(", "))
        };
        return Some(Answer::new("sets", vec![input(format!("{{{}}} {} {{{}}}", &m[1], if eq_ { "=" } else { "⊆" }, &m[3])), primary(pod("Result", "Result", text(if res { "True" } else { "False" }))), pod("Reason", "Reason", text(why))]));
    }
    if let Some(m) = re!(r"(?i)^(?:the )?cartesian product (?:of )?\{([^{}]*)\}\s*(?:,|and|x|×)\s*\{([^{}]*)\}$").captures(l) {
        let a: Vec<&str> = m[1].split(',').map(str::trim).filter(|x| !x.is_empty()).collect();
        let b: Vec<&str> = m[2].split(',').map(str::trim).filter(|x| !x.is_empty()).collect();
        let pairs: Vec<String> = a.iter().flat_map(|x| b.iter().map(move |y| format!("({x}, {y})"))).collect();
        return Some(Answer::new("cartesian product", vec![input(format!("{{{}}} × {{{}}}", a.join(", "), b.join(", "))), primary(pod("Result", "Result", text(format!("{{{}}}", pairs.join(", "))))), pod("Size", "Number of elements", text(format!("{} × {} = {}", a.len(), b.len(), pairs.len())))]));
    }
    // symbolic set algebra
    let l = re!(r"(?i)^symmetric difference (?:of )?(\w+) and (\w+)$").replace(l, "$1 symdiff $2").to_string();
    let l = re!(r"(?i)\bcomplement of\b").replace_all(&l, "complement").to_string();
    let l = re!(r"\s*(==|=)\s*").replace_all(&l, " = ").to_string();
    let l = re!(r"(?i)^is\s+").replace(&l, "").to_string();
    let l = re!(r"\b(\w)\^c\b").replace_all(&l, "$1'").to_string();
    if !has_set_op(&l) {
        return None;
    }
    // identities and subsets
    if let Some(m) = re!(r"(?i)^(.+?)\s+(=|subset(?: of)?|is a subset of|⊆|⊂|a subset of)\s+(.+?)\??$").captures(&l) {
        let mut names = Vec::new();
        let a = parse_set(&m[1], &mut names)?;
        let b = parse_set(&m[3], &mut names)?;
        let n = names.len();
        if n == 0 || n > 6 {
            return None;
        }
        let eq_ = m[2].starts_with('=');
        let bad: Vec<usize> = (0..1 << n).filter(|&k| if eq_ { a.eval(k, n) != b.eval(k, n) } else { a.eval(k, n) && !b.eval(k, n) }).collect();
        let res = bad.is_empty();
        let why = if res {
            if eq_ { "both sides contain exactly the same Venn regions".to_string() } else { "every Venn region of the left side lies in the right side".to_string() }
        } else {
            format!("counterexample region: {}", region_name(bad[0], &names))
        };
        let shown = format!("{} {} {}", &m[1], if eq_ { "=" } else { "⊆" }, &m[3]);
        return Some(Answer::new("set identity", vec![input(shown), primary(pod("Result", "Result", text(if res { "True" } else { "False" }))), pod("Reason", "Reason", text(why))]));
    }
    let mut names = Vec::new();
    let e = parse_set(&l, &mut names)?;
    let n = names.len();
    if n == 0 || n > 5 {
        return None;
    }
    let on: Vec<bool> = (0..1 << n).map(|k| e.eval(k, n)).collect();
    let regions: Vec<String> = (0..1 << n).filter(|&k| on[k]).map(|k| region_name(k, &names)).collect();
    let cubes = minimise(&on, n);
    let simple = show_sop(&cubes, &names, " ∩ ", " ∪ ", &|v| format!("{v}'"), "universal set", "∅");
    let mut pods = vec![input(l.clone()), primary(pod("Result", "Simplest form", text(simple)))];
    pods.push(pod("Regions", &format!("Venn diagram regions ({} of {})", regions.len(), 1 << n), text(if regions.is_empty() { "none (empty set)".into() } else { regions.join("; ") })));
    if n <= 3 {
        pods.push(pod("Venn", "Venn diagram", plot(venn_plot(&on, &names))));
    }
    Some(Answer::new("set expression", pods))
}

fn venn_plot(on: &[bool], names: &[String]) -> crate::plot::Plot {
    use crate::plot::*;
    let n = names.len();
    let centres: Vec<(f64, f64)> = match n {
        1 => vec![(0.0, 0.0)],
        2 => vec![(-0.6, 0.0), (0.6, 0.0)],
        _ => vec![(-0.6, 0.35), (0.6, 0.35), (0.0, -0.65)],
    };
    let r = 1.0;
    // shade the selected regions with a grid of points
    let mut pts = Vec::new();
    let step = 0.06;
    let mut y = -2.0;
    while y <= 2.0 {
        let mut x = -2.2;
        while x <= 2.2 {
            let mut m = 0;
            for (j, (cx, cy)) in centres.iter().enumerate() {
                if (x - cx).powi(2) + (y - cy).powi(2) <= r * r {
                    m |= 1 << (n - 1 - j);
                }
            }
            if on[m] {
                pts.push([x, y]);
            }
            x += step;
        }
        y += step;
    }
    let mut p = scatter(pts, "selected region", None);
    for (j, (cx, cy)) in centres.iter().enumerate() {
        let circ: Vec<[f64; 2]> = (0..=120).map(|k| {
            let t = k as f64 / 120.0 * std::f64::consts::TAU;
            [cx + r * t.cos(), cy + r * t.sin()]
        }).collect();
        p.series.push(Series { label: names[j].clone(), segments: vec![circ], points: false });
    }
    p.x_range = (-2.2, 2.2);
    p.y_range = (-2.0, 2.0);
    p.equal_aspect = true;
    p
}

// ---------------------------------------------------------------------------
// cardinal arithmetic
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Card {
    Fin(u64),
    Aleph(u32),
    Beth(u32),
}

fn norm(c: Card) -> Card {
    match c {
        Card::Beth(0) => Card::Aleph(0),
        x => x,
    }
}

fn show_card(c: Card) -> String {
    let sub = |k: u32| k.to_string().chars().map(|d| char::from_u32(0x2080 + d.to_digit(10).unwrap()).unwrap()).collect::<String>();
    match norm(c) {
        Card::Fin(n) => n.to_string(),
        Card::Aleph(k) => format!("ℵ{}", sub(k)),
        Card::Beth(k) => format!("ℶ{}", sub(k)),
    }
}

/// Provably x ≤ y in ZFC: Some(true), provably not: Some(false), else None.
fn le(x: Card, y: Card) -> Option<bool> {
    use Card::*;
    Some(match (norm(x), norm(y)) {
        (Fin(a), Fin(b)) => a <= b,
        (Fin(_), _) => true,
        (_, Fin(_)) => false,
        (Aleph(a), Aleph(b)) | (Beth(a), Beth(b)) => a <= b,
        (Aleph(a), Beth(b)) => {
            if a <= b {
                true
            } else {
                return None;
            }
        }
        (Beth(b), Aleph(a)) => {
            if a < b {
                false
            } else {
                return None;
            }
        }
    })
}

fn cmax(x: Card, y: Card) -> Option<Card> {
    if le(x, y)? { Some(y) } else if le(y, x)? { Some(x) } else { None }
}

fn two_pow(y: Card) -> Option<Card> {
    match norm(y) {
        Card::Fin(n) if n < 64 => Some(Card::Fin(1u64 << n)),
        Card::Aleph(0) => Some(Card::Beth(1)),
        Card::Beth(k) => Some(Card::Beth(k + 1)),
        _ => None,
    }
}

fn card_op(op: char, x: Card, y: Card, gch: bool) -> Option<Card> {
    use Card::*;
    let (x, y) = if gch {
        let f = |c: Card| match c {
            Beth(k) => Aleph(k),
            o => o,
        };
        (f(norm(x)), f(norm(y)))
    } else {
        (norm(x), norm(y))
    };
    match op {
        '+' => match (x, y) {
            (Fin(a), Fin(b)) => a.checked_add(b).map(Fin),
            _ => cmax(x, y),
        },
        '*' => match (x, y) {
            (Fin(a), Fin(b)) => a.checked_mul(b).map(Fin),
            (Fin(0), _) | (_, Fin(0)) => Some(Fin(0)),
            _ => cmax(x, y),
        },
        '^' => match (x, y) {
            (_, Fin(0)) => Some(Fin(1)),
            (Fin(a), Fin(b)) => a.checked_pow(b.min(64) as u32).map(Fin),
            (_, Fin(_)) => Some(x),
            (Fin(0), _) => Some(Fin(0)),
            (Fin(1), _) => Some(Fin(1)),
            _ if gch => {
                // GCH: κ^λ = κ if λ < cf κ = κ (alephs with finite index are regular), else λ⁺
                let (Aleph(a), Aleph(b)) = (x, y) else {
                    return match (x, y) {
                        (Fin(_), Aleph(b)) => Some(Aleph(b + 1)),
                        _ => None,
                    };
                };
                Some(if b < a { Aleph(a) } else { Aleph(b + 1) })
            }
            (Fin(_), _) => two_pow(y),
            _ => {
                // 2 ≤ x ≤ 2^y ⇒ x^y = 2^y
                if let Some(t) = two_pow(y) {
                    if le(x, t) == Some(true) {
                        return Some(t);
                    }
                }
                // ℶ_j^y = ℶ_j when y ≤ ℶ_(j−1)
                if let Beth(j) = x {
                    if j >= 1 && le(y, Beth(j - 1)) == Some(true) {
                        return Some(x);
                    }
                }
                None
            }
        },
        _ => None,
    }
}

fn card_eval(s: &str, gch: bool) -> Option<Card> {
    // tokens: numbers, alephN, bethN, operators, parentheses
    let toks: Vec<String> = re!(r"aleph\s*_?\s*\d+|ℵ\s*_?\d+|beth?\s*_?\s*\d+|ℶ\s*_?\d+|\d+|[-+*^()×·]").find_iter(s).map(|m| m.as_str().replace(' ', "")).collect();
    if toks.concat().replace('_', "") != s.replace([' ', '_'], "") {
        return None;
    }
    fn atom(t: &str) -> Option<Card> {
        let digits: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
        let k: u64 = digits.parse().ok()?;
        if t.starts_with("aleph") || t.starts_with('ℵ') {
            Some(Card::Aleph(k as u32))
        } else if t.starts_with("bet") || t.starts_with('ℶ') {
            Some(Card::Beth(k as u32))
        } else {
            Some(Card::Fin(k))
        }
    }
    struct P<'a> {
        t: &'a [String],
        i: usize,
        gch: bool,
    }
    impl P<'_> {
        fn sum(&mut self) -> Option<Card> {
            let mut a = self.prod()?;
            while self.t.get(self.i).map(|s| s.as_str()) == Some("+") {
                self.i += 1;
                let b = self.prod()?;
                a = card_op('+', a, b, self.gch)?;
            }
            Some(a)
        }
        fn prod(&mut self) -> Option<Card> {
            let mut a = self.pw()?;
            while matches!(self.t.get(self.i).map(|s| s.as_str()), Some("*" | "×" | "·")) {
                self.i += 1;
                let b = self.pw()?;
                a = card_op('*', a, b, self.gch)?;
            }
            Some(a)
        }
        fn pw(&mut self) -> Option<Card> {
            let a = self.prim()?;
            if self.t.get(self.i).map(|s| s.as_str()) == Some("^") {
                self.i += 1;
                let b = self.pw()?;
                return card_op('^', a, b, self.gch);
            }
            Some(a)
        }
        fn prim(&mut self) -> Option<Card> {
            let t = self.t.get(self.i)?.clone();
            self.i += 1;
            if t == "(" {
                let v = self.sum()?;
                if self.t.get(self.i).map(|s| s.as_str()) != Some(")") {
                    return None;
                }
                self.i += 1;
                return Some(v);
            }
            atom(&t)
        }
    }
    let mut p = P { t: &toks, i: 0, gch };
    let v = p.sum()?;
    if p.i != toks.len() {
        return None;
    }
    Some(v)
}

fn card_facts(c: Card) -> &'static str {
    match norm(c) {
        Card::Aleph(0) => "the cardinality of the natural numbers: the smallest infinite cardinal (countably infinite)",
        Card::Aleph(1) => "the smallest uncountable cardinal: the cardinality of the set of countable ordinals; ℵ₁ = 2^ℵ₀ is the continuum hypothesis",
        Card::Beth(1) => "2^ℵ₀, the cardinality of the continuum (ℝ, the power set of ℕ)",
        Card::Beth(2) => "2^(2^ℵ₀), the cardinality of the set of all functions ℝ → ℝ and of the power set of ℝ",
        Card::Aleph(_) => "an uncountable aleph number (an infinite well-orderable cardinal)",
        Card::Beth(_) => "a beth number: ℶ_(k+1) = 2^ℶ_k",
        Card::Fin(_) => "a finite cardinal",
    }
}

fn cardinals(l: &str) -> Option<Answer> {
    if !re!(r"aleph|ℵ|\bbeth?\s*_?\s*\d|ℶ").is_match(l) {
        return None;
    }
    // comparison
    if let Some(m) = re!(r"^(?:is )?(.+?)\s*(<=|>=|=|==|<|>|≤|≥)\s*(.+?)\??$").captures(l) {
        let a = card_eval(&m[1], false);
        let b = card_eval(&m[3], false);
        let (ag, bg) = (card_eval(&m[1], true)?, card_eval(&m[3], true)?);
        let rel = &m[2];
        let decide = |a: Card, b: Card| -> Option<bool> {
            let (ab, ba) = (le(a, b), le(b, a));
            match rel {
                "=" | "==" => match (ab, ba) {
                    (Some(true), Some(true)) => Some(true),
                    (Some(false), _) | (_, Some(false)) => Some(false),
                    _ => None,
                },
                "<=" | "≤" => ab,
                ">=" | "≥" => ba,
                "<" => match (ab, ba) {
                    (Some(true), Some(false)) => Some(true),
                    (_, Some(true)) => Some(false),
                    _ => None,
                },
                _ => match (ba, ab) {
                    (Some(true), Some(false)) => Some(true),
                    (_, Some(true)) => Some(false),
                    _ => None,
                },
            }
        };
        let zfc = match (a, b) {
            (Some(a), Some(b)) => decide(a, b),
            _ => None,
        };
        let gch = decide(ag, bg);
        let main = match zfc {
            Some(true) => "True".to_string(),
            Some(false) => "False".to_string(),
            None => format!("independent of ZFC; {} under the generalized continuum hypothesis", match gch { Some(true) => "true", Some(false) => "false", None => "undetermined" }),
        };
        let mut pods = vec![input(format!("{} {} {}", m[1].trim(), rel, m[3].trim())), primary(pod("Result", "Result", text(main)))];
        if let (Some(a), Some(b)) = (a, b) {
            pods.push(pod("Values", "Values", text(format!("{} {} {}", show_card(a), rel, show_card(b)))));
        }
        pods.push(pod("Note", "Note", text("ℵ_k ≤ ℶ_k for every k, with equality for all k under GCH; ℵ₀ = ℶ₀")));
        return Some(Answer::new("cardinal comparison", pods));
    }
    let gch = card_eval(l, true)?;
    let zfc = card_eval(l, false);
    let main = match zfc {
        Some(v) => show_card(v),
        None => format!("{} (assuming GCH; not determined by ZFC alone)", show_card(gch)),
    };
    let val = zfc.unwrap_or(gch);
    Some(Answer::new("cardinal", vec![input(l.to_string()), primary(pod("Result", "Result", text(main))), pod("Description", "Description", text(card_facts(val)))]))
}

// ---------------------------------------------------------------------------
// Boolean functions, minterms, normal forms
// ---------------------------------------------------------------------------

fn var_names(spec: Option<&str>, n: usize) -> Vec<String> {
    if let Some(s) = spec {
        let v: Vec<String> = s.split([',', ' ']).map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
        if v.len() == n || n == 0 {
            return v;
        }
    }
    let base = ["a", "b", "c", "d", "e", "f"];
    if n <= 3 {
        return ["x", "y", "z"][..n].iter().map(|s| s.to_string()).collect();
    }
    base[..n.min(6)].iter().map(|s| s.to_string()).collect()
}

fn logic_answer(title: String, on: &[bool], names: &[String], primary_kind: &str) -> Answer {
    let n = names.len();
    let rows: Vec<Vec<Content>> = (0..1usize << n).rev().map(|m| {
        let mut r: Vec<Content> = (0..n).map(|j| text(if (m >> (n - 1 - j)) & 1 == 1 { "T" } else { "F" })).collect();
        r.push(text(if on[m] { "T" } else { "F" }));
        r
    }).collect();
    let dnf = show_sop(&minimise(on, n), names, " ∧ ", " ∨ ", &|v| format!("¬{v}"), "True", "False");
    let off: Vec<bool> = on.iter().map(|b| !b).collect();
    let cnf_cubes = minimise(&off, n);
    let cnf = if cnf_cubes.is_empty() {
        "True".to_string()
    } else if cnf_cubes.iter().any(|c| c.iter().all(|x| x.is_none())) {
        "False".to_string()
    } else {
        cnf_cubes.iter().map(|c| {
            let lits: Vec<String> = c.iter().enumerate().filter_map(|(j, v)| v.map(|b| if b { format!("¬{}", names[j]) } else { names[j].clone() })).collect();
            if lits.len() > 1 && cnf_cubes.len() > 1 { format!("({})", lits.join(" ∨ ")) } else { lits.join(" ∨ ") }
        }).collect::<Vec<_>>().join(" ∧ ")
    };
    let mut header: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    header.push("value");
    let (pt, pv) = match primary_kind {
        "cnf" => ("Conjunctive normal form", cnf.clone()),
        _ => ("Minimal disjunctive normal form", dnf.clone()),
    };
    let mut pods = vec![input(title), primary(pod("Result", pt, text(pv)))];
    if primary_kind == "cnf" {
        pods.push(pod("DNF", "Minimal disjunctive normal form", text(dnf)));
    } else {
        pods.push(pod("CNF", "Minimal conjunctive normal form", text(cnf)));
    }
    let count = on.iter().filter(|b| **b).count();
    pods.push(pod("TruthTable", "Truth table", table(Some(header), rows)));
    pods.push(pod("Properties", "Properties", text(format!("true for {count} of {} inputs{}", on.len(), if count == 0 { " (contradiction)" } else if count == on.len() { " (tautology)" } else { "" }))));
    Answer::new("boolean function", pods)
}

fn boolean(l: &str, raw: &str) -> Option<Answer> {
    // the k-th Boolean function of n variables (Wolfram numbering: bit 2^n − 1 − m of k is the value at input m read as binary, all-true first)
    if let Some(m) = re!(r"^(?:the )?(\d+)(?:st|nd|rd|th)? boolean function (?:of|in) (?:(\d+) variables|((?:[a-z]\s*,\s*)+[a-z]))$").captures(l) {
        let k: u64 = m[1].parse().ok()?;
        let names = match m.get(2) {
            Some(n) => var_names(None, n.as_str().parse().ok()?),
            None => var_names(Some(&m[3]), 0),
        };
        let n = names.len();
        if n == 0 || n > 6 || (n < 6 && k >= 1u64 << (1u64 << n)) {
            return None;
        }
        let on: Vec<bool> = (0..1usize << n).map(|m| (k >> m) & 1 == 1).collect();
        return Some(logic_answer(format!("Boolean function number {k} of {}", names.join(", ")), &on, &names, "dnf"));
    }
    if let Some(m) = re!(r"^(?:the )?(\d+)(?:st|nd|rd|th)? (minterm|maxterm) (?:in|of) (\d+) variables$").captures(l) {
        let k: usize = m[1].parse().ok()?;
        let n: usize = m[3].parse().ok()?;
        if n == 0 || n > 8 || k >= 1 << n {
            return None;
        }
        let names = var_names(None, n);
        let bits: Vec<bool> = (0..n).map(|j| (k >> (n - 1 - j)) & 1 == 1).collect();
        let (res, note) = if &m[2] == "minterm" {
            (bits.iter().zip(&names).map(|(b, v)| if *b { v.clone() } else { format!("¬{v}") }).collect::<Vec<_>>().join(" ∧ "), format!("true only when ({}) = ({})", names.join(", "), bits.iter().map(|b| if *b { "T" } else { "F" }).collect::<Vec<_>>().join(", ")))
        } else {
            (bits.iter().zip(&names).map(|(b, v)| if *b { format!("¬{v}") } else { v.clone() }).collect::<Vec<_>>().join(" ∨ "), format!("false only when ({}) = ({})", names.join(", "), bits.iter().map(|b| if *b { "T" } else { "F" }).collect::<Vec<_>>().join(", ")))
        };
        return Some(Answer::new("minterm", vec![input(format!("{} {} in {} variables", k, &m[2], n)), primary(pod("Result", "Result", text(res))), pod("Meaning", "Meaning", text(note)), pod("Binary", "Index in binary", text(format!("{k} = {}₂", bits.iter().map(|b| if *b { '1' } else { '0' }).collect::<String>())))]));
    }
    // normal forms and minimisation of a formula
    let m = re!(r"(?i)^(cnf|dnf|conjunctive normal form(?: of)?|disjunctive normal form(?: of)?|minimi[sz]e|simplify|boolean minimi[sz]ation|logic circuit|anf)\s+(.+)$").captures(raw.trim())?;
    let kind = m[1].to_lowercase();
    let body = m[2].replace("&&", " and ").replace("||", " or ").replace('&', " and ").replace('|', " or ").replace('~', " not ").replace('!', " not ");
    let e = ex(&body)?;
    let is_logic = |e: &Expr| matches!(e, Func(n, _) if matches!(&**n, "and" | "or" | "not" | "xor" | "implies" | "iff" | "nand" | "nor"));
    if !is_logic(&e) && !matches!(kind.as_str(), "cnf" | "dnf" | "logic circuit") {
        return None;
    }
    let vars: Vec<Name> = e.free_symbols().into_iter().collect();
    let n = vars.len();
    if n == 0 || n > 6 {
        return None;
    }
    let on: Vec<bool> = (0..1usize << n).map(|k| {
        let assign: Vec<(Expr, Expr)> = vars.iter().enumerate().map(|(j, v)| (sym(v), boolean_c((k >> (n - 1 - j)) & 1 == 1))).collect();
        matches!(crate::simplify::simplify(&e.subs_many(&assign)), Const(K::True))
    }).collect();
    let names: Vec<String> = vars.iter().map(|v| v.to_string()).collect();
    let pk = if kind.starts_with('c') { "cnf" } else { "dnf" };
    let mut a = logic_answer(ptext(&e), &on, &names, pk);
    if kind == "logic circuit" {
        let gates = ptext(&e).matches(['∧', '∨', '¬', '⊻']).count();
        a.pods.push(pod("Circuit", "Circuit", text(format!("{gates} gates in the given form; inputs {}", names.join(", ")))));
    }
    Some(a)
}

fn boolean_c(b: bool) -> Expr {
    crate::expr::boolean(b)
}

// ---------------------------------------------------------------------------
// injectivity, surjectivity
// ---------------------------------------------------------------------------

fn mapping(l: &str) -> Option<Answer> {
    let m = re!(r"^is (?:the function )?(?:[a-z]\s*\(x\)\s*=\s*|y\s*=\s*)?(.+?)(?: an?)? (one-to-one and onto|injective and surjective|one-to-one|injective|surjective|onto|bijective|bijection)(?: function)?(?: (?:on|for|over) (.+?))?$").captures(l)?;
    let (fs, prop, dom) = (m[1].trim().trim_end_matches(" a").to_string(), m[2].to_string(), m.get(3).map(|x| x.as_str().to_string()));
    // "for x<2" inside the function text
    let (fs, dom) = match re!(r"^(.+?) for (.+)$").captures(&fs) {
        Some(c) => (c[1].to_string(), Some(c[2].to_string())),
        None => (fs, dom),
    };
    let f = ex(&fs)?;
    let x = sym("x");
    let fv = f.free_symbols();
    if fv.len() != 1 || !f.has_sym(&x) {
        return None;
    }
    // domain interval
    let (mut a, mut b, mut desc) = (f64::NEG_INFINITY, f64::INFINITY, "ℝ".to_string());
    if let Some(d) = &dom {
        let d = d.trim();
        if let Some(c) = re!(r"^\(\s*(.+?)\s*,\s*(.+?)\s*\)$").captures(d) {
            a = crate::eval::eval_f64(&ex(&c[1])?, &[])?;
            b = crate::eval::eval_f64(&ex(&c[2])?, &[])?;
            desc = format!("({}, {})", &c[1], &c[2]);
        } else if let Some(c) = re!(r"^x\s*(<|>|<=|>=)\s*(.+)$").captures(d) {
            let v = crate::eval::eval_f64(&ex(&c[2])?, &[])?;
            if c[1].starts_with('<') { b = v } else { a = v }
            desc = format!("x {} {}", &c[1], &c[2]);
        } else {
            return None;
        }
    }
    let Sym(xn) = &x else { return None };
    let prog = crate::eval::Program::compile(&f, &[xn])?.with_real_roots();
    // sample the domain through a tangent map
    let map = |u: f64| -> f64 {
        match (a.is_finite(), b.is_finite()) {
            (true, true) => a + (b - a) * u,
            (true, false) => a + u / (1.0 - u),
            (false, true) => b - (1.0 - u) / u,
            (false, false) => (std::f64::consts::PI * (u - 0.5)).tan(),
        }
    };
    let n = 20000;
    let pts: Vec<(f64, f64)> = (1..n).map(|k| {
        let t = map(k as f64 / n as f64);
        (t, prog.eval1(t))
    }).filter(|(_, v)| v.is_finite()).collect();
    if pts.len() < n / 2 {
        return None;
    }
    // split the samples into continuous branches at poles: a sign change that
    // stays large when the bracket is narrowed (a root would shrink to zero)
    let is_pole = |w: &[(f64, f64)]| -> bool {
        if w[0].1.signum() == w[1].1.signum() || (w[1].1 - w[0].1).abs() < 1e3 {
            return false;
        }
        let (mut x0, mut x1, f0) = (w[0].0, w[1].0, w[0].1);
        for _ in 0..60 {
            let xm = 0.5 * (x0 + x1);
            let fm = prog.eval1(xm);
            if !fm.is_finite() {
                return true;
            }
            if fm.signum() == f0.signum() { x0 = xm } else { x1 = xm }
        }
        prog.eval1(x0).abs() + prog.eval1(x1).abs() > 1e-3 * (w[0].1.abs() + w[1].1.abs())
    };
    let mut branches: Vec<&[(f64, f64)]> = Vec::new();
    let mut startb = 0;
    for i in 0..pts.len() - 1 {
        if is_pole(&pts[i..i + 2]) {
            branches.push(&pts[startb..=i]);
            startb = i + 1;
        }
    }
    branches.push(&pts[startb..]);
    let poles = branches.len() > 1;
    // behaviour at each end of the domain: Some(±1) when f → ±∞
    let end = |left: bool| -> Option<i32> {
        let vs: Vec<f64> = (1..=5).map(|j| {
            let u = 10f64.powi(-3 * j);
            prog.eval1(map(if left { u } else { 1.0 - u }))
        }).collect();
        if vs.iter().any(|v| !v.is_finite()) {
            return None;
        }
        let ab: Vec<f64> = vs.iter().map(|v| v.abs()).collect();
        let growing = ab.windows(2).all(|w| w[1] > w[0]) && ab[4] > ab[3] + 1.0 && (ab[4] - ab[3]) >= 0.5 * (ab[3] - ab[2]);
        let sign = vs[4].signum();
        (growing && vs[3].signum() == sign).then_some(sign as i32)
    };
    let (el, er) = (end(true), end(false));
    let inner: Vec<f64> = pts.iter().filter(|(t, _)| t.abs() <= 10.0).map(|p| p.1).collect();
    let scale = {
        let mut v: Vec<f64> = inner.iter().map(|v| v.abs()).collect();
        v.sort_by(|x, y| x.partial_cmp(y).unwrap());
        v.get(v.len() / 2).cloned().unwrap_or(1.0).max(1.0)
    };
    struct Br {
        lo: f64,
        hi: f64,
        dir: i32,
        turn: Option<f64>,
    }
    let nb = branches.len();
    let info: Vec<Br> = branches.iter().enumerate().map(|(bi, br)| {
        let (mut ups, mut downs, mut last, mut turn) = (0, 0, 0i32, None);
        for w in br.windows(2) {
            let d = w[1].1 - w[0].1;
            let tol = 1e-12 * (1.0 + w[0].1.abs());
            let dir = if d > tol { 1 } else if d < -tol { -1 } else { 0 };
            if dir == 1 { ups += 1 }
            if dir == -1 { downs += 1 }
            if dir != 0 {
                if last != 0 && dir != last && turn.is_none() {
                    turn = Some(w[0].0);
                }
                last = dir;
            }
        }
        let mut lo = br.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let mut hi = br.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        // values next to a pole run off to infinity
        let (first, lastv) = (br[0].1, br[br.len() - 1].1);
        if bi > 0 && first.abs() > 1e3 * scale {
            if first < 0.0 { lo = f64::NEG_INFINITY } else { hi = f64::INFINITY }
        }
        if bi + 1 < nb && lastv.abs() > 1e3 * scale {
            if lastv < 0.0 { lo = f64::NEG_INFINITY } else { hi = f64::INFINITY }
        }
        // the ends of the domain
        let ends = [(bi == 0).then_some(el).flatten(), (bi + 1 == nb).then_some(er).flatten()];
        for e in ends.into_iter().flatten() {
            if e < 0 { lo = f64::NEG_INFINITY } else { hi = f64::INFINITY }
        }
        // oscillating but unbounded on an unbounded domain (x² sin x)
        if !a.is_finite() && !b.is_finite() && nb == 1 {
            if hi > 1e3 * scale { hi = f64::INFINITY }
            if lo < -1e3 * scale { lo = f64::NEG_INFINITY }
        }
        let dir = if ups > 0 && downs > 0 { 0 } else if downs == 0 { 1 } else { -1 };
        Br { lo, hi, dir, turn }
    }).collect();
    let monotone = info.iter().all(|b| b.dir != 0);
    let overlap = (0..nb).any(|i| (i + 1..nb).any(|j| info[i].hi > info[j].lo && info[j].hi > info[i].lo));
    let inj_ok = monotone && !overlap;
    let mut iv: Vec<(f64, f64)> = info.iter().map(|b| (b.lo, b.hi)).collect();
    iv.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
    let mut reach = f64::NEG_INFINITY;
    let mut covered = iv.first().map(|x| x.0 == f64::NEG_INFINITY).unwrap_or(false);
    for (l0, h0) in &iv {
        if *l0 > reach && reach != f64::NEG_INFINITY {
            covered = false;
        }
        reach = reach.max(*h0);
    }
    let surjective = covered && reach == f64::INFINITY;
    let lo = iv.iter().map(|x| x.0).fold(f64::INFINITY, f64::min);
    let hi = iv.iter().map(|x| x.1).fold(f64::NEG_INFINITY, f64::max);
    let fmt = |v: f64| crate::print::fmt_float(crate::eval::chop(v), 5);
    let inj_text = if inj_ok {
        if poles {
            format!("injective: f is strictly monotonic on each of its {nb} branches and their ranges do not overlap")
        } else {
            format!("injective: f is strictly {} on {desc}", if info[0].dir > 0 { "increasing" } else { "decreasing" })
        }
    } else if let Some(t) = info.iter().find_map(|b| b.turn) {
        format!("not injective: f changes direction near x ≈ {}, so values near f({}) ≈ {} are taken twice", fmt(t), fmt(t), fmt(prog.eval1(t)))
    } else {
        "not injective: different branches take the same values".to_string()
    };
    let range = if surjective { "ℝ".to_string() } else { format!("approximately [{}, {}]{}", if lo == f64::NEG_INFINITY { "−∞".into() } else { fmt(lo) }, if hi == f64::INFINITY { "∞".into() } else { fmt(hi) }, if poles { " with gaps" } else { "" }) };
    let sur_text = if surjective { "surjective onto ℝ: every real value is attained".to_string() } else { format!("not surjective onto ℝ: the range is {range}") };
    let (title, res) = match prop.as_str() {
        "one-to-one" | "injective" => ("Injective", inj_ok),
        "surjective" | "onto" => ("Surjective", surjective),
        _ => ("Bijective", inj_ok && surjective),
    };
    Some(Answer::new("function properties", vec![
        input(format!("is f(x) = {} {} on {desc}?", ptext(&f), prop)),
        primary(pod("Result", title, text(if res { "yes" } else { "no" }))),
        pod("Injectivity", "Injectivity", text(inj_text)),
        pod("Surjectivity", "Surjectivity (codomain ℝ)", text(sur_text)),
    ]))
}

// ---------------------------------------------------------------------------
// divide-and-conquer recurrences
// ---------------------------------------------------------------------------

fn divide_conquer(l: &str) -> Option<Answer> {
    // normalise T[n], f_n, f_(n/5), ==
    let s = l.replace("==", "=").replace('[', "(").replace(']', ")");
    let s = re!(r"(?i)\b([a-z])_\(([^()]*)\)").replace_all(&s, "$1($2)").to_string();
    let s = re!(r"(?i)\b([a-z])_n\b").replace_all(&s, "$1(n)").to_string();
    let m = re!(r"(?i)^(?:solve )?([a-z])\(n\)\s*=\s*(.+)$").captures(&s)?;
    let f = &m[1];
    let rhs = m[2].to_string();
    let pat = Regex::new(&format!(r"(?:(\d+(?:\.\d+)?)\s*\*?\s*)?{f}\(\s*(?:n\s*/\s*(\d+(?:\.\d+)?)|(\d+(?:\.\d+)?)\s*\*?\s*n|n\s*\*\s*(\d+(?:\.\d+)?))\s*\)")).ok()?;
    let mut terms: Vec<(f64, f64)> = Vec::new(); // (a, b): a T(n/b)
    for c in pat.captures_iter(&rhs) {
        let a: f64 = c.get(1).map(|x| x.as_str().parse().unwrap_or(1.0)).unwrap_or(1.0);
        let b: f64 = if let Some(d) = c.get(2) { d.as_str().parse().ok()? } else { 1.0 / c.get(3).or(c.get(4))?.as_str().parse::<f64>().ok()? };
        if b <= 1.0 {
            return None;
        }
        terms.push((a, b));
    }
    if terms.is_empty() {
        return None;
    }
    let g_text = pat.replace_all(&rhs, "0").to_string();
    let g_text = g_text.replace("logn", "log(n)").replace("Log", "log");
    let g = crate::simplify::simplify(&ex(&g_text)?);
    let n = sym("n");
    if g.free_symbols().iter().any(|v| &**v != "n") {
        return None;
    }
    // p with Σ a b^(−p) = 1
    let h = |p: f64| terms.iter().map(|(a, b)| a * b.powf(-p)).sum::<f64>() - 1.0;
    let (mut lo, mut hi) = (-20.0, 40.0);
    if h(lo) < 0.0 || h(hi) > 0.0 {
        return None;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if h(mid) > 0.0 { lo = mid } else { hi = mid }
    }
    let p = 0.5 * (lo + hi);
    // growth of g: n^k (log n)^j
    let (k, j) = if crate::simplify::simplify(&g).is_zero() {
        (f64::NEG_INFINITY, 0.0)
    } else {
        let lg = |t: f64| -> Option<f64> {
            // ln g(e^t), computed in log space for large t
            let v = crate::eval::eval_f64(&g, &[("n", t.exp())])?;
            (v > 0.0).then(|| v.ln())
        };
        let (t1, t2, t3) = (20.0, 40.0, 80.0);
        let (l1, l2, l3) = (lg(t1)?, lg(t2)?, lg(t3)?);
        let k = (l3 - l2) / (t3 - t2);
        let k = if (k - k.round()).abs() < 0.02 { k.round() } else { (k * 12.0).round() / 12.0 };
        let j = ((l3 - k * t3) - (l2 - k * t2)) / (t3 / t2).ln();
        let _ = l1;
        (k, j.round())
    };
    let pr = crate::print::fmt_float(p, 6);
    let p_exact = if terms.len() == 1 {
        let (a, b) = terms[0];
        if (p - p.round()).abs() < 1e-9 { pr.clone() } else { format!("log_{}({}) ≈ {pr}", crate::print::fmt_float(b, 6), crate::print::fmt_float(a, 6)) }
    } else {
        pr.clone()
    };
    let np = |_: &str| if (p - p.round()).abs() < 1e-9 { if p.round() == 0.0 { String::new() } else if p.round() == 1.0 { "n".to_string() } else { format!("n^{}", p.round()) } } else { format!("n^{}", crate::print::fmt_float(p, 5)) };
    let logs = |j: f64| if j == 0.0 { String::new() } else if j == 1.0 { "log n".into() } else { format!("(log n)^{}", crate::print::fmt_float(j, 4)) };
    let joinf = |a: String, b: String| match (a.is_empty(), b.is_empty()) {
        (true, true) => "1".to_string(),
        (true, false) => b,
        (false, true) => a,
        _ => format!("{a} {b}"),
    };
    let (theta, case) = if k < p - 1e-9 {
        (joinf(np("p"), String::new()), format!("g(n) = O(n^(p−ε)): the recursion dominates"))
    } else if (k - p).abs() < 1e-9 {
        (joinf(np("p"), logs(j + 1.0)), "g(n) = Θ(n^p (log n)^j): balanced, one extra log factor".to_string())
    } else {
        let gk = if k == 0.0 { String::new() } else if k == 1.0 { "n".into() } else { format!("n^{}", crate::print::fmt_float(k, 4)) };
        (joinf(gk, logs(j)), "g(n) grows faster than n^p: the top level dominates".to_string())
    };
    let eqs = terms.iter().map(|(a, b)| format!("{}·{}^(−p)", crate::print::fmt_float(*a, 6), crate::print::fmt_float(*b, 6))).collect::<Vec<_>>().join(" + ");
    let mut pods = vec![
        input(format!("{f}(n) = {}", rhs.trim())),
        primary(pod("Result", "Asymptotic solution", text(format!("{f}(n) = Θ({theta})")))),
        pod("Exponent", "Critical exponent p", text(format!("{eqs} = 1 ⇒ p = {p_exact}"))),
        pod("Case", "Reasoning (Akra–Bazzi)", text(format!("non-recursive part g(n) = {}; {case}", ptext(&g)))),
    ];
    // exact solution for T(n) = T(n/b) + c with n a power of b
    if terms.len() == 1 && terms[0].0 == 1.0 && g.free_symbols().is_empty() {
        let b = terms[0].1;
        pods.push(pod("Exact", "Exact solution for n = bᵏ", text(format!("{f}(n) = {f}(1) + {} log_{}(n)", ptext(&g), crate::print::fmt_float(b, 6)))));
    }
    let _ = n;
    Some(Answer::new("divide and conquer recurrence", pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let raw = q.clean.trim().trim_end_matches(['?', '.']);
    let l = raw.to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let rawn = re!(r"\s+").replace_all(raw, " ").to_string();
    cardinals(&l).or_else(|| sets(&rawn)).or_else(|| boolean(&l, raw)).or_else(|| mapping(&l)).or_else(|| divide_conquer(&rawn))
}

#[cfg(test)]
mod tests {
    fn ans(s: &str) -> String {
        serde_json::to_string(&crate::engine::query(s).pods).unwrap_or_default()
    }

    #[test]
    fn sets_logic_cardinals() {
        assert!(ans("is (A union B)'=(A') intersect (B')").contains("True"));
        assert!(ans("is complement B a subset of B union A").contains("False"));
        assert!(ans("11th Boolean function of 2 variables").contains("¬x ∨ y"));
        assert!(ans("5*aleph0^aleph0").contains("ℶ₁"));
        assert!(ans("bet3 < aleph3").contains("False"));
        assert!(ans("T[n]==T[n/2]+1").contains("Θ(log n)"));
        assert!(ans("is y=x^3+x a one-to-one function?").contains("strictly increasing"));
    }
}
