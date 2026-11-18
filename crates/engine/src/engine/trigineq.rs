//! Inequalities in periodic functions ("sin(x) > 1/2", "csc(x) <= 2/sqrt(3)"):
//! the solution over one period found numerically, with the end points
//! recognised as rational multiples of π, then repeated with the period.

use super::nl::{ex, main_var};
use crate::eval::Program;
use crate::expr::Expr::*;
use crate::expr::*;
use crate::plot as pl;
use std::f64::consts::PI;

/// An end point as text: a rational multiple of π, a small fraction, or a decimal.
pub fn nice(v: f64) -> String {
    if v.abs() < 1e-12 {
        return "0".into();
    }
    for (scale, sym) in [(PI, "π"), (1.0, "")] {
        for q in 1..=12i64 {
            let p = v / scale * q as f64;
            if (p - p.round()).abs() < 1e-8 * q as f64 {
                let p = p.round() as i64;
                let g = gcd(p.abs(), q);
                let (p, q) = (p / g, q / g);
                let sign = if p < 0 { "−" } else { "" };
                let pa = p.abs();
                let num = match (pa, sym) {
                    (1, "π") => "π".to_string(),
                    (n, "π") => format!("{n}π"),
                    (n, _) => n.to_string(),
                };
                return if q == 1 { format!("{sign}{num}") } else { format!("{sign}{num}/{q}") };
            }
        }
    }
    crate::print::fmt_float(v, 6)
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a.max(1) } else { gcd(b, a % b) }
}

/// An end point snapped to a nearby rational multiple of π (or rational), if any.
fn snap(v: f64) -> f64 {
    for scale in [PI, 1.0] {
        for q in 1..=12i64 {
            let p = v / scale * q as f64;
            if (p - p.round()).abs() < 1e-7 {
                return p.round() / q as f64 * scale;
            }
        }
    }
    v
}

pub struct Solution {
    pub text: String,
    pub spans: Vec<pl::Span>,
    pub period: f64,
}

/// Solve a op b for a periodic a − b.
pub fn solve(op: RelOp, a: &Expr, b: &Expr, x: &Expr) -> Option<Solution> {
    if matches!(op, RelOp::Eq | RelOp::Ne) {
        return None;
    }
    let f = crate::simplify::simplify(&(a.clone() - b.clone()));
    let per = super::math::period(&f, x).or_else(|| super::math::period(a, x).filter(|_| !b.has_sym(x)))?;
    let p = per.to_f64_const()?;
    if !(p > 0.0) || p > 1e4 {
        return None;
    }
    let Sym(name) = x else { return None };
    let prog = Program::compile(&f, &[name])?;
    let holds = |t: f64| {
        let g = prog.eval1(t);
        g.is_finite()
            && match op {
                RelOp::Lt => g < 0.0,
                RelOp::Le => g <= 1e-12,
                RelOp::Gt => g > 0.0,
                _ => g >= -1e-12,
            }
    };
    let n = 4000;
    let xs: Vec<f64> = (0..n).map(|i| p * i as f64 / n as f64).collect();
    let truth: Vec<bool> = xs.iter().map(|t| holds(*t)).collect();
    if truth.iter().all(|t| !t) {
        return None;
    }
    let strict = matches!(op, RelOp::Lt | RelOp::Gt);
    // a boundary between samples i and i+1 (cyclically)
    let boundary = |i: usize| -> (f64, bool) {
        let (mut lo, mut hi) = (xs[i], if i + 1 < n { xs[i + 1] } else { p });
        let t0 = truth[i];
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if holds(mid) == t0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let v = snap(0.5 * (lo + hi));
        let g = prog.eval1(v);
        let closed = !strict && g.is_finite() && g.abs() < 1e-9;
        (v, closed)
    };
    // intervals over [0, p)
    let mut iv: Vec<(f64, bool, f64, bool)> = Vec::new();
    let all_true = truth.iter().all(|t| *t);
    if all_true {
        return Some(Solution { text: format!("all real {}", crate::print::text(x)), spans: vec![pl::Span { lo: f64::NEG_INFINITY, hi: f64::INFINITY, lo_closed: false, hi_closed: false }], period: p });
    }
    // start at a false sample so every interval is complete
    let start = truth.iter().position(|t| !t)?;
    let mut cur: Option<(f64, bool)> = None;
    for k in 0..n {
        let i = (start + k) % n;
        let j = (i + 1) % n;
        if truth[i] != truth[j] {
            // positions past the end of the period continue into the next one
            let (v, closed) = boundary(i);
            let v = v + if start + k >= n { p } else { 0.0 };
            if truth[j] {
                cur = Some((v, closed));
            } else if let Some((lo, lc)) = cur.take() {
                iv.push((lo, lc, v, closed));
            }
        }
    }
    if iv.is_empty() {
        return None;
    }
    // keep each interval within one period, starting in (−p, p)
    for t in iv.iter_mut() {
        while t.0 >= p - 1e-12 {
            t.0 -= p;
            t.2 -= p;
        }
    }
    // prefer the representation starting in [0, p) unless it straddles zero
    for t in iv.iter_mut() {
        if t.2 > p + 1e-12 && t.0 > 0.0 {
            t.0 -= p;
            t.2 -= p;
        }
    }
    iv.sort_by(|a, b| a.0.total_cmp(&b.0));
    let xv = crate::print::text(x);
    let per_text = match nice(p).as_str() {
        "π" => "πn".to_string(),
        s if !s.contains('/') => format!("{s}n"),
        s => format!("({s})n"),
    };
    let add = |v: f64| if v.abs() < 1e-12 { per_text.clone() } else { format!("{} + {per_text}", nice(v)) };
    let parts: Vec<String> = iv.iter().map(|(lo, lc, hi, hc)| {
        if (hi - lo).abs() < 1e-12 {
            format!("{xv} = {}", add(*lo))
        } else {
            format!("{} {} {xv} {} {}", add(*lo), if *lc { "≤" } else { "<" }, if *hc { "≤" } else { "<" }, add(*hi))
        }
    }).collect();
    let text = format!("{} for integer n", parts.join(" or "));
    let mut spans = Vec::new();
    for k in -1..=1 {
        for (lo, lc, hi, hc) in &iv {
            spans.push(pl::Span { lo: lo + k as f64 * p, hi: hi + k as f64 * p, lo_closed: *lc, hi_closed: *hc });
        }
    }
    Some(Solution { text, spans, period: p })
}

/// Parse "sin(x) > 1/2" into a relation in one variable with a trigonometric part.
pub fn parse(body: &str) -> Option<(RelOp, Expr, Expr, Expr)> {
    if !body.contains(['<', '>', '≤', '≥']) || !regex::Regex::new(r"\b(?:sin|cos|tan|sec|csc|cot)\b").ok()?.is_match(body) {
        return None;
    }
    let e = ex(body)?;
    let Rel(op, a, b) = &e else { return None };
    let x = main_var(&e);
    if e.free_symbols().len() != 1 {
        return None;
    }
    Some((*op, (**a).clone(), (**b).clone(), x))
}
