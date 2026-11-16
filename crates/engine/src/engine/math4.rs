//! Number names (spelling numbers, scale words, German numerals),
//! arithmetic of number types ("integer / integer" is a rational number),
//! Wigner 3j and 6j symbols and Clebsch–Gordan coefficients (exact, by
//! Racah's formulas), and Latin squares.

use super::*;
use crate::expr::Expr::*;
use crate::expr::*;
use crate::print::text as ptext;
use crate::simplify as s;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

// ---------------------------------------------------------------------------
// number names
// ---------------------------------------------------------------------------

const SCALES: &[(&str, u32)] = &[
    ("thousand", 3), ("million", 6), ("billion", 9), ("trillion", 12), ("quadrillion", 15), ("quintillion", 18), ("sextillion", 21), ("septillion", 24), ("octillion", 27), ("nonillion", 30),
    ("decillion", 33), ("undecillion", 36), ("duodecillion", 39), ("tredecillion", 42), ("quattuordecillion", 45), ("quindecillion", 48), ("sexdecillion", 51), ("septendecillion", 54), ("octodecillion", 57),
    ("novemdecillion", 60), ("vigintillion", 63), ("centillion", 303), ("googol", 100),
];

fn parse_integer(t: &str) -> Option<BigInt> {
    let t = t.trim().replace([',', '_'], "").replace(' ', "");
    if let Some((b, e)) = t.split_once('^') {
        let b: BigInt = b.parse().ok()?;
        let e: u32 = e.parse().ok().filter(|e| *e <= 300)?;
        return Some(num_traits::pow(b, e as usize));
    }
    if let Some((m, e)) = t.split_once(['e', 'E']) {
        let m: BigInt = m.parse().ok()?;
        let e: u32 = e.parse().ok().filter(|e| *e <= 300)?;
        return Some(m * num_traits::pow(BigInt::from(10), e as usize));
    }
    t.parse().ok()
}

fn names(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:how (?:do you|to) )?(?:spell(?: out)?|write out|write|say|name|english name (?:of|for))\s+(?:the number )?(.+?)(?: in (?:english )?words| in english)?$").captures(l).or_else(|| re!(r"^(?:the number )?(.+?) (?:in (?:english )?words|spelled out|written out|in english)$").captures(l)) {
        let n = parse_integer(&m[1])?;
        let name = crate::numtheory::number_name(&n)?;
        let mut pods = vec![input(format!("{n} in words")), primary(pod("Result", "English name", text(name.clone())))];
        pods.push(pod("Length", "Number of characters", text(format!("{} letters", name.chars().filter(|c| c.is_alphabetic()).count()))));
        if n.to_string().len() > 6 {
            pods.push(pod("Scale", "Note", text("short scale (American and modern British usage): billion = 10^9")));
        }
        return Some(Answer::new("number name", pods));
    }
    if let Some(m) = re!(r"^(?:(?:one|1|a)\s+)?(thousand|million|billion|trillion|quadrillion|quintillion|sextillion|septillion|octillion|nonillion|decillion|undecillion|duodecillion|tredecillion|quattuordecillion|quindecillion|sexdecillion|septendecillion|octodecillion|novemdecillion|vigintillion|centillion|googol)$").captures(l) {
        let (name, e) = SCALES.iter().find(|(n, _)| *n == &m[1])?;
        let long = match *name {
            "googol" => String::from("same in all scales"),
            "thousand" => String::from("same in all scales"),
            "million" => String::from("same in all scales"),
            _ => {
                // long scale: n-illion = 10^(6n); short: 10^(3n+3)
                let k = (e - 3) / 3;
                format!("10^{} in the long scale (traditional European usage)", 6 * k)
            }
        };
        let rows = vec![vec![text("value"), text(format!("10^{e}"))], vec![text("digits"), text(format!("1 followed by {e} zeros"))], vec![text("long scale"), text(long)]];
        return Some(Answer::new(*name, vec![input(format!("1 {name}")), primary(pod("Result", "Value (short scale)", text(format!("10^{e}")))), pod("Details", "Details", table(None, rows))]));
    }
    if matches!(l, "german numeral names" | "german numbers" | "german number names" | "numbers in german" | "german numerals") {
        let list = [
            ("0", "null"), ("1", "eins"), ("2", "zwei"), ("3", "drei"), ("4", "vier"), ("5", "fünf"), ("6", "sechs"), ("7", "sieben"), ("8", "acht"), ("9", "neun"), ("10", "zehn"), ("11", "elf"), ("12", "zwölf"), ("13", "dreizehn"),
            ("14", "vierzehn"), ("15", "fünfzehn"), ("16", "sechzehn"), ("17", "siebzehn"), ("18", "achtzehn"), ("19", "neunzehn"), ("20", "zwanzig"), ("21", "einundzwanzig"), ("30", "dreißig"), ("40", "vierzig"), ("50", "fünfzig"),
            ("60", "sechzig"), ("70", "siebzig"), ("80", "achtzig"), ("90", "neunzig"), ("100", "hundert"), ("1000", "tausend"), ("10^6", "eine Million"), ("10^9", "eine Milliarde"), ("10^12", "eine Billion"),
        ];
        let rows = list.iter().map(|(a, b)| vec![text(*a), text(*b)]).collect();
        return Some(Answer::new("German numerals", vec![input("German number names"), primary(pod("Result", "German numerals", table(Some(vec!["number", "German"]), rows))), pod("Note", "Note", text("German uses the long scale: Milliarde = 10^9, Billion = 10^12; units come before tens (einundzwanzig = one-and-twenty)"))]));
    }
    None
}

/// "five sixteenths", "three quarters": the fraction.
fn fraction_words(l: &str) -> Option<Answer> {
    if !re!(r"^[a-z -]+$").is_match(l) || l.split_whitespace().count() > 6 {
        return None;
    }
    let t = super::rewrite::number_words(l);
    let m = re!(r"^(\d+)/(\d+)$").captures(t.trim())?;
    let (p, q): (i64, i64) = (m[1].parse().ok()?, m[2].parse().ok()?);
    let v = rat(p, q);
    let dec = p as f64 / q as f64;
    Some(Answer::new("fraction", vec![input(format!("{l} = {p}/{q}")), primary(pod("Result", "Exact value", text(ptext(&v)))), pod("Decimal", "Decimal form", text(crate::print::fmt_float(dec, 12))), pod("Percent", "Percentage", text(format!("{}%", crate::print::fmt_float(dec * 100.0, 10))))]))
}

// ---------------------------------------------------------------------------
// arithmetic of number types
// ---------------------------------------------------------------------------

fn type_samples(t: &str) -> Option<(Vec<Expr>, &'static str)> {
    let ints = |v: &[i64]| v.iter().map(|&k| int(k)).collect::<Vec<_>>();
    let rats = |v: &[(i64, i64)]| v.iter().map(|&(a, b)| rat(a, b)).collect::<Vec<_>>();
    let t = t.trim().trim_start_matches("a ").trim_start_matches("an ").trim_end_matches('s');
    Some(match t {
        "integer" | "whole number" => (ints(&[-7, -3, -2, -1, 0, 1, 2, 3, 5, 12]), "integer"),
        "even number" | "even integer" => (ints(&[-6, -4, -2, 0, 2, 4, 8]), "even integer"),
        "odd number" | "odd integer" => (ints(&[-7, -3, -1, 1, 3, 5, 9]), "odd integer"),
        "positive integer" | "natural number" | "counting number" => (ints(&[1, 2, 3, 4, 5, 10]), "positive integer"),
        "nonnegative integer" | "non-negative integer" => (ints(&[0, 1, 2, 3, 5, 10]), "nonnegative integer"),
        "negative integer" => (ints(&[-1, -2, -3, -5, -10]), "negative integer"),
        "rational" | "rational number" | "fraction" => (rats(&[(-7, 2), (-3, 4), (-1, 1), (0, 1), (1, 2), (2, 1), (5, 3), (3, 1)]), "rational number"),
        "positive rational" | "positive rational number" => (rats(&[(1, 2), (3, 4), (1, 1), (5, 3), (2, 1), (7, 1)]), "positive rational number"),
        "nonnegative rational" | "non-negative rational" | "nonnegative rational number" => (rats(&[(0, 1), (1, 2), (3, 4), (1, 1), (5, 3), (2, 1)]), "nonnegative rational number"),
        "negative rational" | "negative rational number" => (rats(&[(-1, 2), (-3, 4), (-1, 1), (-5, 3), (-2, 1)]), "negative rational number"),
        _ => return None,
    })
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
enum Class {
    Integer,
    Rational,
    RealAlgebraic,
    Algebraic,
}

fn classify(e: &Expr) -> Option<(Class, f64, Option<bool>)> {
    let z = crate::eval::const_c64(e)?;
    if !z.re.is_finite() || !z.im.is_finite() {
        return None;
    }
    if let Num(r) = e {
        let even = if r.is_integer() { Some((r.numer() % BigInt::from(2)).is_zero()) } else { None };
        return Some((if r.is_integer() { Class::Integer } else { Class::Rational }, r.to_f64().unwrap_or(0.0), even));
    }
    if z.im.abs() > 1e-12 * (1.0 + z.re.abs()) {
        return Some((Class::Algebraic, f64::NAN, None));
    }
    Some((Class::RealAlgebraic, z.re, None))
}

fn number_types(l: &str) -> Option<Answer> {
    let (a, op, b, root) = if let Some(m) = re!(r"^(?:the )?square root of (?:an? )?([a-z -]+)$").captures(l) {
        (m[1].to_string(), "sqrt".to_string(), String::new(), true)
    } else {
        let m = re!(r"^([a-z -]+?)\s*([+\-*/^×÷])\s*([a-z -]+)$").captures(l)?;
        (m[1].to_string(), m[2].to_string(), m[3].to_string(), false)
    };
    let (sa, na) = type_samples(&a)?;
    let (sb, nb) = if root { (vec![rat(1, 2)], "") } else { type_samples(&b)? };
    let mut class = Class::Integer;
    let (mut pos, mut nonneg, mut neg, mut nonpos) = (true, true, true, true);
    let (mut all_even, mut all_odd) = (true, true);
    let mut any = false;
    for x in &sa {
        for y in &sb {
            let e = match op.as_str() {
                "+" => add(vec![x.clone(), y.clone()]),
                "-" => sub(x.clone(), y.clone()),
                "*" | "×" => mul(vec![x.clone(), y.clone()]),
                "/" | "÷" => {
                    if y.is_zero() {
                        continue;
                    }
                    div(x.clone(), y.clone())
                }
                _ => {
                    if x.is_zero() && y.to_f64_const().is_some_and(|v| v <= 0.0) {
                        continue;
                    }
                    pow(x.clone(), y.clone())
                }
            };
            let e = s::simplify(&e);
            let Some((c, v, even)) = classify(&e) else { continue };
            any = true;
            if c > class {
                class = c;
            }
            if c == Class::Algebraic {
                pos = false;
                nonneg = false;
                neg = false;
                nonpos = false;
            } else {
                pos &= v > 0.0;
                nonneg &= v >= 0.0;
                neg &= v < 0.0;
                nonpos &= v <= 0.0;
            }
            all_even &= even == Some(true);
            all_odd &= even == Some(false);
        }
    }
    if !any {
        return None;
    }
    let sign = if pos { "positive " } else if nonneg { "nonnegative " } else if neg { "negative " } else if nonpos { "nonpositive " } else { "" };
    let name = match class {
        Class::Integer => format!("{sign}{}integer", if all_even { "even " } else if all_odd { "odd " } else { "" }),
        Class::Rational => format!("{sign}rational number"),
        Class::RealAlgebraic => format!("{sign}real algebraic number"),
        Class::Algebraic => "algebraic number (possibly non-real)".to_string(),
    };
    let what = if root { format!("square root of {na}") } else { format!("{na} {op} {nb}") };
    let mut pods = vec![input(what.clone()), primary(pod("Result", "Result type", text(name)))];
    if class == Class::Algebraic {
        pods.push(pod("Example", "Example", text("(−1)^(1/2) = i")));
    }
    Some(Answer::new("number type arithmetic", pods))
}

// ---------------------------------------------------------------------------
// Wigner symbols
// ---------------------------------------------------------------------------

fn fact(n: i64) -> Option<BigInt> {
    if n < 0 {
        return None;
    }
    Some((1..=n).fold(BigInt::one(), |a, k| a * k))
}

/// Twice a half-integer value given as text ("1/2", "3", "-1.5").
fn twice(t: &str) -> Option<i64> {
    let t = t.trim();
    if let Some((p, q)) = t.split_once('/') {
        let (p, q): (i64, i64) = (p.trim().parse().ok()?, q.trim().parse().ok()?);
        return match q {
            1 => Some(2 * p),
            2 => Some(p),
            _ => None,
        };
    }
    let v: f64 = t.parse().ok()?;
    let d = 2.0 * v;
    (d.fract() == 0.0).then_some(d as i64)
}

fn half(d: i64) -> String {
    if d % 2 == 0 { (d / 2).to_string() } else { format!("{d}/2") }
}

fn triangle(a: i64, b: i64, c: i64) -> bool {
    // doubled values
    (a + b + c) % 2 == 0 && c <= a + b && c >= (a - b).abs()
}

/// Δ(abc) = (a+b−c)!(a−b+c)!(−a+b+c)!/(a+b+c+1)! for doubled a, b, c.
fn delta(a: i64, b: i64, c: i64) -> Option<BigRational> {
    let n = fact((a + b - c) / 2)? * fact((a - b + c) / 2)? * fact((-a + b + c) / 2)?;
    let d = fact((a + b + c) / 2 + 1)?;
    Some(BigRational::new(n, d))
}

/// Value as (sign · rational · sqrt(rational)) → exact expression.
fn signed_sqrt(sum: BigRational, radicand: BigRational) -> Expr {
    if sum.is_zero() || radicand.is_zero() {
        return zero();
    }
    s::simplify(&mul(vec![num(sum), sqrt(num(radicand))]))
}

/// Wigner 3j symbol with doubled arguments.
fn three_j(j1: i64, j2: i64, j3: i64, m1: i64, m2: i64, m3: i64) -> Option<Expr> {
    if m1 + m2 + m3 != 0 || !triangle(j1, j2, j3) || m1.abs() > j1 || m2.abs() > j2 || m3.abs() > j3 || (j1 + m1) % 2 != 0 || (j2 + m2) % 2 != 0 || (j3 + m3) % 2 != 0 {
        return Some(zero());
    }
    let h = |x: i64| x / 2;
    let rad = delta(j1, j2, j3)? * BigRational::from_integer(fact(h(j1 + m1))? * fact(h(j1 - m1))? * fact(h(j2 + m2))? * fact(h(j2 - m2))? * fact(h(j3 + m3))? * fact(h(j3 - m3))?);
    let kmin = 0.max(h(j2 - j3 - m1)).max(h(j1 - j3 + m2));
    let kmax = h(j1 + j2 - j3).min(h(j1 - m1)).min(h(j2 + m2));
    let mut sum = BigRational::zero();
    for k in kmin..=kmax {
        let den = fact(k)? * fact(h(j3 - j2 + m1) + k)? * fact(h(j3 - j1 - m2) + k)? * fact(h(j1 + j2 - j3) - k)? * fact(h(j1 - m1) - k)? * fact(h(j2 + m2) - k)?;
        let term = BigRational::new(BigInt::one(), den);
        if k % 2 == 0 {
            sum += term;
        } else {
            sum -= term;
        }
    }
    let phase = h(j1 - j2 - m3);
    if phase.rem_euclid(2) == 1 {
        sum = -sum;
    }
    Some(signed_sqrt(sum, rad))
}

fn six_j(j: [i64; 6]) -> Option<Expr> {
    let [j1, j2, j3, j4, j5, j6] = j;
    if !(triangle(j1, j2, j3) && triangle(j1, j5, j6) && triangle(j4, j2, j6) && triangle(j4, j5, j3)) {
        return Some(zero());
    }
    let rad = delta(j1, j2, j3)? * delta(j1, j5, j6)? * delta(j4, j2, j6)? * delta(j4, j5, j3)?;
    let a = [(j1 + j2 + j3) / 2, (j1 + j5 + j6) / 2, (j4 + j2 + j6) / 2, (j4 + j5 + j3) / 2];
    let b = [(j1 + j2 + j4 + j5) / 2, (j2 + j3 + j5 + j6) / 2, (j3 + j1 + j6 + j4) / 2];
    let tmin = *a.iter().max()?;
    let tmax = *b.iter().min()?;
    let mut sum = BigRational::zero();
    for t in tmin..=tmax {
        let mut den = BigInt::one();
        for x in a {
            den *= fact(t - x)?;
        }
        for x in b {
            den *= fact(x - t)?;
        }
        let term = BigRational::new(fact(t + 1)?, den);
        if t % 2 == 0 {
            sum += term;
        } else {
            sum -= term;
        }
    }
    Some(signed_sqrt(sum, rad))
}

fn numbers_in(t: &str) -> Vec<String> {
    re!(r"-?\d+(?:\.\d+)?(?:/\d+)?").find_iter(t).map(|m| m.as_str().to_string()).collect()
}

fn wigner(l: &str) -> Option<Answer> {
    let kind = if re!(r"\b(?:wigner )?6-?j\b|sixjsymbol|six-j").is_match(l) {
        6
    } else if re!(r"\b(?:wigner )?3-?j\b|threejsymbol|three-j").is_match(l) {
        3
    } else if re!(r"clebsch|clebschgordan").is_match(l) {
        2
    } else {
        return None;
    };
    let rest = re!(r"wigner|symbols?|coefficients?|calculator|clebsch[- ]?gordan|threejsymbol|sixjsymbol|\b[36]-?j\b").replace_all(l, " ").to_string();
    let nums = numbers_in(&rest);
    let d: Option<Vec<i64>> = nums.iter().map(|x| twice(x)).collect();
    let (d, example) = match d.filter(|v| v.len() == 6) {
        Some(v) => (v, false),
        None if nums.is_empty() => (
            match kind {
                6 => vec![2, 2, 2, 2, 2, 2],
                3 => vec![2, 2, 2, 0, 2, -2],
                _ => vec![1, 1, 1, -1, 2, 0],
            },
            true,
        ),
        None => return None,
    };
    let (label, value, formula) = match kind {
        3 => {
            let v = three_j(d[0], d[1], d[2], d[3], d[4], d[5])?;
            (format!("({} {} {}; {} {} {})", half(d[0]), half(d[1]), half(d[2]), half(d[3]), half(d[4]), half(d[5])), v, "(j₁ j₂ j₃; m₁ m₂ m₃) by Racah's formula; zero unless m₁ + m₂ + m₃ = 0 and the j satisfy the triangle condition")
        }
        6 => {
            let mut a = [0i64; 6];
            a.copy_from_slice(&d);
            (format!("{{{} {} {}; {} {} {}}}", half(d[0]), half(d[1]), half(d[2]), half(d[3]), half(d[4]), half(d[5])), six_j(a)?, "{j₁ j₂ j₃; j₄ j₅ j₆} by Racah's formula; zero unless all four triads satisfy the triangle condition")
        }
        _ => {
            // ⟨j1 m1 j2 m2 | J M⟩ = (−1)^(j1−j2+M) √(2J+1) (j1 j2 J; m1 m2 −M)
            let (j1, m1, j2, m2, jj, mm) = (d[0], d[1], d[2], d[3], d[4], d[5]);
            let tj = three_j(j1, j2, jj, m1, m2, -mm)?;
            let phase = (j1 - j2 + mm) / 2;
            let sign = if phase.rem_euclid(2) == 1 { int(-1) } else { one() };
            let v = s::simplify(&mul(vec![sign, sqrt(int(jj + 1)), tj]));
            (format!("⟨{} {}; {} {} | {} {}⟩", half(j1), half(m1), half(j2), half(m2), half(jj), half(mm)), v, "⟨j₁ m₁ j₂ m₂ | J M⟩ = (−1)^(j₁−j₂+M) √(2J + 1) (j₁ j₂ J; m₁ m₂ −M)")
        }
    };
    let title = match kind {
        3 => "Wigner 3j symbol",
        6 => "Wigner 6j symbol",
        _ => "Clebsch–Gordan coefficient",
    };
    let approx = crate::eval::const_c64(&value).map(|z| crate::print::fmt_float(z.re, 10)).unwrap_or_default();
    let mut pods = vec![input(format!("{title} {label}")), primary(pod("Result", "Exact result", text(ptext(&value)))), pod("Decimal", "Decimal approximation", text(approx)), pod("Definition", "Definition", text(formula))];
    if example {
        pods.push(pod("Note", "Note", text(match kind {
            3 => "example; give six arguments j₁ j₂ j₃ m₁ m₂ m₃, e.g. \"wigner 3j (1, 1, 0, 0, 0, 0)\"",
            6 => "example; give six arguments j₁ … j₆, e.g. \"wigner 6j (1, 1, 1, 1, 1, 1)\"",
            _ => "example; give j₁ m₁ j₂ m₂ J M, e.g. \"clebsch gordan 1/2 1/2 1/2 -1/2 1 0\"",
        })));
    }
    Some(Answer::new(title, pods))
}

// ---------------------------------------------------------------------------
// Latin squares
// ---------------------------------------------------------------------------

fn latin(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:a )?(?:size|order)[- ](\d+) latin square$|^latin square (?:of )?(?:size|order) (\d+)$|^(\d+)\s*(?:x|×|by)\s*\d+ latin square$").captures(l)?;
    let n: usize = m.get(1).or(m.get(2)).or(m.get(3))?.as_str().parse().ok()?;
    if n == 0 || n > 1000 {
        return None;
    }
    // counts of reduced Latin squares (OEIS A000315) and all Latin squares (A002860)
    const REDUCED: &[&str] = &["1", "1", "1", "4", "56", "9408", "16942080", "535281401856", "377597570964258816", "7580721483160132811489280", "5363937773277371298119673540771840"];
    const ALL: &[&str] = &["1", "2", "12", "576", "161280", "812851200", "61479419904000", "108776032459082956800", "5524751496156892842531225600", "9982437658213039871725064756920320000", "776966836171770144107444346734230682311065600000"];
    let show = n.min(12);
    let rows: Vec<Vec<Content>> = (0..show).map(|i| (0..show).map(|j| text(((i + j) % n + 1).to_string())).collect()).collect();
    let mut pods = vec![input(format!("Latin square of order {n}")), primary(pod("Result", if n > 12 { "Cyclic Latin square L(i, j) = (i + j) mod n + 1 (first 12 rows and columns)" } else { "Cyclic Latin square L(i, j) = (i + j) mod n + 1" }, table(None, rows)))];
    if n <= 11 {
        pods.push(pod("Counts", "Number of Latin squares", table(None, vec![vec![text("reduced (first row and column in order)"), text(REDUCED[n - 1])], vec![text("all"), text(ALL[n - 1])]])));
    } else {
        pods.push(pod("Counts", "Number of Latin squares", text(format!("not known for order {n}: exact counts are known only up to order 11"))));
    }
    Some(Answer::new("Latin square", pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    names(&l).or_else(|| wigner(&l)).or_else(|| number_types(&l)).or_else(|| latin(&l)).or_else(|| fraction_words(&l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wigner_symbols() {
        assert_eq!(ptext(&three_j(2, 2, 0, 0, 0, 0).unwrap()), "-sqrt(3)/3");
        // (1/2 1/2 1; 1/2 -1/2 0) = 1/√6
        assert_eq!(ptext(&three_j(1, 1, 2, 1, -1, 0).unwrap()), "sqrt(6)/6");
        // {1 1 1; 1 1 1} = 1/6
        assert_eq!(ptext(&six_j([2; 6]).unwrap()), "1/6");
        // {1/2 1/2 1; 1/2 1/2 0} = 1/2
        assert_eq!(ptext(&six_j([1, 1, 2, 1, 1, 0]).unwrap()), "1/2");
    }
}
