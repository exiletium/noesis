//! More calculus and algebra on demand: ranges and domains on a restricted
//! interval, domains of functions of several variables, periods in each
//! variable, rotation matrices, fractional derivatives of polynomials,
//! cusps and corners, area between a curve and the x-axis, critical points
//! near a given point, arithmetic of number classes ("even number * odd
//! number"), transfer functions, the predator–prey model, and arithmetic
//! with units ("(5 meters - 2 meters)/(18 seconds - 8 seconds)").

use super::*;
use crate::calculus::diff::diff;
use crate::eval::Program;
use crate::expr::Expr::*;
use crate::expr::*;
use crate::print::{fmt_float, text as ptext};
use crate::simplify as s;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn n(v: f64) -> String {
    fmt_float(crate::eval::chop(v), 6)
}

fn ex(s: &str) -> Option<Expr> {
    let t = re!(r"^(?:the\s+)?(?:curve\s+|function\s+)?(?:[a-z]\s*\(\s*[a-z](?:\s*,\s*[a-z])*\s*\)\s*=\s*|[yz]\s*=\s*)?").replace(s.trim(), "").to_string();
    crate::parse::parse(t.trim()).ok().map(|p| p.expr)
}

fn show(e: &Expr) -> String {
    ptext(e)
}

fn ans(title: &str, input: String, result_title: &str, result: Content, extra: Vec<Pod>) -> Answer {
    let mut pods = vec![pod("Input", "Input interpretation", text(input)), primary(pod("Result", result_title, result))];
    pods.extend(extra);
    Answer::new(title, pods)
}

fn main_var(e: &Expr) -> Expr {
    let fs = e.free_symbols();
    for c in ["x", "t", "y", "z", "u", "s", "r"] {
        if fs.iter().any(|n| &**n == c) {
            return sym(c);
        }
    }
    fs.into_iter().next().map(|n| sym(&n)).unwrap_or_else(|| sym("x"))
}

/// Small-denominator rational, rational multiple of π, or square root of a rational.
fn nice(v: f64) -> Option<String> {
    let gcd = |mut a: i64, mut b: i64| {
        a = a.abs();
        while b != 0 {
            let t = a % b;
            a = b;
            b = t;
        }
        a.max(1)
    };
    for d in [1i64, 2, 3, 4, 5, 6, 8, 10, 12] {
        let r = (v * d as f64).round();
        if (v * d as f64 - r).abs() < 1e-9 * (1.0 + v.abs()) {
            let g = gcd(r as i64, d);
            let (p, q) = (r as i64 / g, d / g);
            return Some(if q == 1 { p.to_string() } else { format!("{p}/{q}") });
        }
    }
    let w = v / std::f64::consts::PI;
    for d in [1i64, 2, 3, 4, 6] {
        let r = (w * d as f64).round();
        if r != 0.0 && (w * d as f64 - r).abs() < 1e-9 {
            let g = gcd(r as i64, d);
            let (p, q) = (r as i64 / g, d / g);
            let num = match p { 1 => "π".to_string(), -1 => "-π".to_string(), _ => format!("{p}π") };
            return Some(if q == 1 { num } else { format!("{num}/{q}") });
        }
    }
    let sq = v * v;
    for d in 1..=100i64 {
        let r = (sq * d as f64).round();
        if r > 0.0 && (sq * d as f64 - r).abs() < 1e-8 * (1.0 + sq) {
            let g = gcd(r as i64, d);
            let (p, q) = (r as i64 / g, d / g);
            let sign = if v < 0.0 { "-" } else { "" };
            return Some(match (p, q) {
                (1, q) => format!("{sign}1/√{q}"),
                (p, 1) => format!("{sign}√{p}"),
                (p, q) => format!("{sign}√({p}/{q})"),
            });
        }
    }
    None
}

fn shown(v: f64) -> String {
    nice(v).unwrap_or_else(|| n(v))
}

// ---------------------------------------------------------------------------
// ranges and domains on a restricted interval
// ---------------------------------------------------------------------------

/// "1<x<4", "x>0", "0 <= x <= pi", "x < 2" → (lo, lo strict, hi, hi strict)
fn restriction(s: &str) -> Option<(f64, bool, f64, bool)> {
    let s = s.trim();
    let val = |t: &str| -> Option<f64> { crate::eval::eval_f64(&ex(t)?, &[]).filter(|v| v.is_finite()) };
    if let Some(m) = re!(r"^(.+?)\s*(<=?|≤)\s*[a-z]\s*(<=?|≤)\s*(.+)$").captures(s) {
        return Some((val(&m[1])?, &m[2] == "<", val(&m[4])?, &m[3] == "<"));
    }
    if let Some(m) = re!(r"^[a-z]\s*(>=?|≥)\s*(.+)$").captures(s) {
        return Some((val(&m[2])?, &m[1] == ">", f64::INFINITY, true));
    }
    if let Some(m) = re!(r"^[a-z]\s*(<=?|≤)\s*(.+)$").captures(s) {
        return Some((f64::NEG_INFINITY, true, val(&m[2])?, &m[1] == "<"));
    }
    None
}

fn restricted(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(domain and range|range and domain|range|domain)\s+(?:of\s+)?(.+?)\s+(?:restricted to|on|over|for)\s+(.+)$").captures(l)?;
    let what = m[1].to_string();
    let e = ex(&m[2])?;
    if e.free_symbols().len() != 1 {
        return None;
    }
    let x = main_var(&e);
    let (lo, lo_open, hi, hi_open) = restriction(&m[3])?;
    if !(lo < hi) {
        return None;
    }
    let Sym(name) = &x else { return None };
    let p = Program::compile(&e, &[name])?;
    // sample, denser near the ends; unbounded sides are mapped
    let map = |u: f64| -> f64 {
        match (lo.is_finite(), hi.is_finite()) {
            (true, true) => lo + (hi - lo) * u,
            (true, false) => lo + u / (1.0 - u).max(1e-12),
            (false, true) => hi - (1.0 - u) / u.max(1e-12),
            _ => (u - 0.5) / (u * (1.0 - u)).max(1e-12),
        }
    };
    let nn = 40000;
    let (mut ymin, mut ymax, mut umin, mut umax) = (f64::INFINITY, f64::NEG_INFINITY, 0.0, 0.0);
    let mut undefined = 0usize;
    for k in 1..nn {
        let u = k as f64 / nn as f64;
        let v = p.eval1(map(u));
        if !v.is_finite() {
            undefined += 1;
            continue;
        }
        if v < ymin {
            ymin = v;
            umin = u;
        }
        if v > ymax {
            ymax = v;
            umax = u;
        }
    }
    if !ymin.is_finite() {
        return None;
    }
    let big = 1e7;
    // an interior extreme that keeps growing under refinement is a pole
    let grows = |u: f64, sign: f64, best: f64| -> bool {
        let du = 1.0 / nn as f64;
        let mut m = best * sign;
        for k in -2000..=2000 {
            let v = p.eval1(map(u + du * k as f64 / 1000.0)) * sign;
            if v.is_finite() && v > m {
                m = v;
            }
        }
        m > 1e3 && m > 4.0 * (best * sign).abs().max(1.0)
    };
    if ymax.is_finite() && grows(umax, 1.0, ymax) {
        ymax = f64::INFINITY;
    }
    if ymin.is_finite() && grows(umin, -1.0, ymin) {
        ymin = f64::NEG_INFINITY;
    }
    // describe a bound: at an end of the interval (open or closed) or inside
    let at_end = |u: f64| u < 2.0 / nn as f64 || u > 1.0 - 2.0 / nn as f64;
    let end_value = |u: f64| -> (f64, bool, Option<String>) {
        // value at the nearer end, whether that end is excluded, exact form
        let (t, open) = if u < 0.5 { (lo, lo_open) } else { (hi, hi_open) };
        if !t.is_finite() {
            // the limit at infinity, when the function settles
            let (v1, v2) = (p.eval1(t.signum() * 1e6), p.eval1(t.signum() * 1e9));
            if v1.is_finite() && v2.is_finite() && (v1 - v2).abs() < 1e-5 * (1.0 + v2.abs()) {
                let r = if v2.abs() < 1e-6 { 0.0 } else { v2 };
                return (r, true, nice(r));
            }
            return (f64::NAN, true, None);
        }
        let exact = s::simplify(&e.subs(&x, &ex(&fmt_float(t, 15)).unwrap_or_else(|| int(0))));
        let ev = crate::eval::eval_f64(&exact, &[]).unwrap_or(f64::NAN);
        let txt = nice(ev).or_else(|| {
            let tt = nice(t)?;
            let e2 = s::simplify(&e.subs(&x, &ex(&tt.replace('π', "pi").replace('√', "sqrt"))?));
            Some(show(&e2))
        });
        (ev, open, txt)
    };
    let lower = if !(ymin >= -big) {
        None
    } else if at_end(umin) || (hi.is_infinite() && umin > 0.5 && (p.eval1(map(1.0 - 1.0 / nn as f64)) - ymin).abs() < 1e-12 * (1.0 + ymin.abs())) {
        let (v, open, txt) = end_value(umin);
        if v.is_nan() { Some((shown(ymin), true)) } else { Some((txt.unwrap_or_else(|| n(v)), open)) }
    } else {
        let t = map(umin);
        let (_, v) = crate::numeric::golden_min(&|t| p.eval1(t), t - 1e-3, t + 1e-3, 1e-13);
        if v < -1e3 && v < 4.0 * ymin.min(-1.0) { None } else { Some((shown(v.min(ymin)), false)) }
    };
    let upper = if !(ymax <= big) {
        None
    } else if at_end(umax) || (hi.is_infinite() && umax > 0.5 && (p.eval1(map(1.0 - 1.0 / nn as f64)) - ymax).abs() < 1e-12 * (1.0 + ymax.abs())) {
        let (v, open, txt) = end_value(umax);
        if v.is_nan() { Some((shown(ymax), true)) } else { Some((txt.unwrap_or_else(|| n(v)), open)) }
    } else {
        let t = map(umax);
        let (_, v) = crate::numeric::golden_min(&|t| -p.eval1(t), t - 1e-3, t + 1e-3, 1e-13);
        if -v > 1e3 && -v > 4.0 * ymax.max(1.0) { None } else { Some((shown(-v.min(-ymax)), false)) }
    };
    let range = match (&lower, &upper) {
        (None, None) => "all real numbers".to_string(),
        (Some((a, o)), None) => format!("{{y ∈ ℝ : y {} {a}}}", if *o { ">" } else { "≥" }),
        (None, Some((b, o))) => format!("{{y ∈ ℝ : y {} {b}}}", if *o { "<" } else { "≤" }),
        (Some((a, oa)), Some((b, ob))) => format!("{{y ∈ ℝ : {a} {} y {} {b}}}", if *oa { "<" } else { "≤" }, if *ob { "<" } else { "≤" }),
    };
    let ivl = |v: f64| if v.is_infinite() { if v < 0.0 { "−∞".to_string() } else { "∞".to_string() } } else { shown(v) };
    let mut dom = match (lo.is_finite(), hi.is_finite()) {
        (true, true) => format!("{{x ∈ ℝ : {} {} x {} {}}}", ivl(lo), if lo_open { "<" } else { "≤" }, if hi_open { "<" } else { "≤" }, ivl(hi)),
        (true, false) => format!("{{x ∈ ℝ : x {} {}}}", if lo_open { ">" } else { "≥" }, ivl(lo)),
        (false, true) => format!("{{x ∈ ℝ : x {} {}}}", if hi_open { "<" } else { "≤" }, ivl(hi)),
        _ => "all real numbers".to_string(),
    };
    if undefined > 0 {
        dom.push_str(" (excluding points where the function is undefined)");
    }
    let mut pods = Vec::new();
    if what.contains("domain") {
        pods.push(primary(pod("Domain", "Domain", text(dom.replace('x', &name.to_string()).replace("ℝ :", "ℝ :")))));
    }
    if what.contains("range") {
        let mut pd = pod("Range", "Range", text(range));
        pd.primary = !what.contains("domain");
        pods.push(pd);
    }
    let mut a = Answer::new(&format!("{what} on an interval"), vec![pod("Input", "Input interpretation", text(format!("{what} | {} | restricted to {}", show(&e), m[3].trim())))]);
    a.pods.extend(pods);
    Some(a)
}

// ---------------------------------------------------------------------------
// domains (and ranges) of functions of several variables
// ---------------------------------------------------------------------------

#[derive(Clone)]
enum Cond {
    Pos(Expr),
    NonNeg(Expr),
    NonZero(Expr),
    Unit(Expr),
}

fn conditions(e: &Expr, out: &mut Vec<Cond>) {
    match e {
        Func(f, a) => {
            for t in a {
                conditions(t, out);
            }
            if a.len() == 1 && !a[0].free_symbols().is_empty() {
                match &**f {
                    "log" | "ln" => out.push(Cond::Pos(a[0].clone())),
                    "sqrt" => out.push(Cond::NonNeg(a[0].clone())),
                    "asin" | "acos" | "arcsin" | "arccos" => out.push(Cond::Unit(a[0].clone())),
                    "tan" => {}
                    _ => {}
                }
            }
        }
        Pow(b, p) => {
            conditions(b, out);
            conditions(p, out);
            if !b.free_symbols().is_empty() {
                if let Some(r) = p.as_rat() {
                    if r < &Rat::from_integer(0.into()) {
                        out.push(Cond::NonZero((**b).clone()));
                    }
                    let den = r.denom().clone();
                    if (den.clone() % 2u32) == 0u32.into() {
                        out.push(if r < &Rat::from_integer(0.into()) { Cond::Pos((**b).clone()) } else { Cond::NonNeg((**b).clone()) });
                    }
                }
            }
        }
        Add(v) | Mul(v) => v.iter().for_each(|t| conditions(t, out)),
        _ => {}
    }
}

/// "1 - x^2 - y^2 > 0" → "x^2 + y^2 < 1"
fn cond_text(a: &Expr, rel: &str) -> String {
    let a = s::simplify(a);
    let (c, rest): (Vec<Expr>, Vec<Expr>) = match &a {
        Add(v) => v.iter().cloned().partition(|t| t.free_symbols().is_empty()),
        _ => (vec![], vec![a.clone()]),
    };
    let c = s::simplify(&add(c));
    let rest = s::simplify(&add(rest));
    let negated = s::simplify(&neg(rest.clone()));
    let flip = |r: &str| match r { ">" => "<", ">=" => "≤", "≥" => "≤", "<" => ">", "≠" => "≠", _ => r }.to_string();
    let rel = if rel == ">=" { "≥" } else { rel };
    // prefer the form with fewer minus signs
    if show(&rest).matches('-').count() > show(&negated).matches('-').count() {
        format!("{} {} {}", show(&negated), flip(rel), show(&c))
    } else {
        format!("{} {rel} {}", show(&rest), show(&s::simplify(&neg(c))))
    }
}

fn multi_domain(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(domain and range|range and domain|domain|range)\s+(?:of\s+)?(.+)$").captures(l)?;
    let what = m[1].to_string();
    let e = ex(&m[2])?;
    let mut vars: Vec<String> = e.free_symbols().into_iter().map(|v| v.to_string()).collect();
    vars.sort();
    if !(2..=3).contains(&vars.len()) || vars.iter().any(|v| v.len() > 1) {
        return None;
    }
    let mut cs = Vec::new();
    conditions(&e, &mut cs);
    let mut texts: Vec<String> = Vec::new();
    for c in &cs {
        let t = match c {
            Cond::Pos(a) => cond_text(a, ">"),
            Cond::NonNeg(a) => cond_text(a, "≥"),
            Cond::NonZero(a) => cond_text(a, "≠"),
            Cond::Unit(a) => format!("-1 ≤ {} ≤ 1", show(a)),
        };
        if !texts.contains(&t) {
            texts.push(t);
        }
    }
    let space = if vars.len() == 2 { "ℝ²" } else { "ℝ³" };
    let tuple = format!("({})", vars.join(", "));
    let dom = if texts.is_empty() { format!("all {tuple} ∈ {space}") } else { format!("{{{tuple} ∈ {space} : {}}}", texts.join(" and ")) };
    // range by sampling the domain on growing boxes
    let names: Vec<&str> = vars.iter().map(|s| s.as_str()).collect();
    let p = Program::compile(&e, &names)?;
    let sample = |r: f64| -> (f64, f64) {
        let k = if names.len() == 2 { 160 } else { 36 };
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        let mut idx = vec![0usize; names.len()];
        loop {
            let pt: Vec<f64> = idx.iter().map(|&i| -r + 2.0 * r * i as f64 / k as f64).collect();
            let v = p.eval(&pt);
            if v.is_finite() {
                lo = lo.min(v);
                hi = hi.max(v);
            }
            let mut j = 0;
            loop {
                if j == idx.len() {
                    return (lo, hi);
                }
                idx[j] += 1;
                if idx[j] <= k {
                    break;
                }
                idx[j] = 0;
                j += 1;
            }
        }
    };
    let (l1, h1) = sample(10.0);
    let (l2, h2) = sample(100.0);
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{what} | f{tuple} = {}", show(&e))))];
    if what.contains("domain") {
        pods.push(primary(pod("Domain", "Domain", text(dom))));
    }
    if what.contains("range") {
        if !l1.is_finite() {
            return None;
        }
        // unbounded when the extremes keep growing with the box; near a
        // domain boundary only sampling evidence is available
        let unb_hi = h2 > h1 + 10.0 * (1.0 + h1.abs()) || h2 > 1e6;
        let unb_lo = l2 < l1 - 10.0 * (1.0 + l1.abs()) || l2 < -1e6;
        let approx = if texts.is_empty() { "" } else { " (sampled)" };
        let lo_s = nice(l2).unwrap_or_else(|| n(l2));
        let hi_s = nice(h2).unwrap_or_else(|| n(h2));
        let r = match (unb_lo, unb_hi) {
            (true, true) => "all real numbers".to_string(),
            (false, true) => format!("{{z ∈ ℝ : z ≥ {lo_s}}}{approx}"),
            (true, false) => format!("{{z ∈ ℝ : z ≤ {hi_s}}}{approx}"),
            (false, false) => format!("{{z ∈ ℝ : {lo_s} ≤ z ≤ {hi_s}}}{approx}"),
        };
        let mut pd = pod("Range", "Range", text(r));
        pd.primary = !what.contains("domain");
        pods.push(pd);
    }
    Some(Answer::new(&format!("{what} of a function of several variables"), pods))
}

// ---------------------------------------------------------------------------
// periods in each variable; named trigonometric functions
// ---------------------------------------------------------------------------

fn periods(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:what is\s+)?(?:the\s+)?(?:fundamental\s+)?period\s+of\s+(?:the\s+)?(sine|cosine|tangent|cotangent|secant|cosecant)(?:\s+function)?$").captures(l) {
        let (f, p) = match &m[1] {
            "sine" => ("sin(x)", "2π"),
            "cosine" => ("cos(x)", "2π"),
            "tangent" => ("tan(x)", "π"),
            "cotangent" => ("cot(x)", "π"),
            "secant" => ("sec(x)", "2π"),
            _ => ("csc(x)", "2π"),
        };
        return Some(ans("period", format!("period of {f}"), "Period", text(format!("{p} ≈ {}", n(if p == "π" { std::f64::consts::PI } else { 2.0 * std::f64::consts::PI }))), vec![pod("Meaning", "Meaning", text(format!("{f} repeats after {p}: f(x + {p}) = f(x) for all x")))]));
    }
    let m = re!(r"^(?:the\s+)?periods?\s+(?:of\s+)?(.+)$").captures(l)?;
    let e = ex(&m[1])?;
    let mut vars: Vec<String> = e.free_symbols().into_iter().map(|v| v.to_string()).collect();
    vars.sort();
    if vars.len() < 2 || vars.len() > 4 {
        return None;
    }
    // period with the other variables fixed at two different values; the
    // dependence on them is recognised as K, K/w, K·w or K/(w1 w2…)
    let per_with = |i: usize, vals: &[i64]| -> Option<f64> {
        let mut f = e.clone();
        for (j, w) in vars.iter().enumerate() {
            if j != i {
                f = f.subs(&sym(w), &int(vals[j]));
            }
        }
        super::math::period(&f, &sym(&vars[i]))?.to_f64_const()
    };
    let mut rows = Vec::new();
    let mut any = false;
    for (i, v) in vars.iter().enumerate() {
        let others: Vec<&String> = vars.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, w)| w).collect();
        let (a, b) = (per_with(i, &[3, 3, 3, 3]), per_with(i, &[5, 5, 5, 5]));
        let cell = match (a, b) {
            (Some(p1), Some(p2)) => {
                any = true;
                let prod = |k: f64| k.powi(others.len() as i32);
                let k_over = |p: f64, w: f64| nice(p * prod(w));
                if (p1 - p2).abs() < 1e-9 * p1 {
                    format!("{} ≈ {}", shown(p1), n(p1))
                } else if (p1 * prod(3.0) - p2 * prod(5.0)).abs() < 1e-9 * p1 * prod(3.0) {
                    let k = k_over(p1, 3.0).unwrap_or_else(|| n(p1 * prod(3.0)));
                    let den = others.iter().map(|w| w.as_str()).collect::<Vec<_>>().join(" ");
                    format!("{k}/{} (for {} ≠ 0)", if others.len() > 1 { format!("({den})") } else { den.clone() }, den)
                } else if (p1 / 3.0 - p2 / 5.0).abs() < 1e-9 * p1 && others.len() == 1 {
                    format!("{} {}", shown(p1 / 3.0), others[0])
                } else {
                    "depends on the other variables".to_string()
                }
            }
            _ => "not periodic".to_string(),
        };
        rows.push(vec![text(v.clone()), text(cell)]);
    }
    if !any {
        return None;
    }
    let main = rows.iter().map(|r| format!("{}: {}", content_text(&r[0]), content_text(&r[1]))).collect::<Vec<_>>().join("; ");
    Some(ans("periods", format!("periods of {} in each variable", show(&e)), "Periods", text(main), vec![pod("Table", "By variable", table(Some(vec!["variable", "period"]), rows))]))
}

// ---------------------------------------------------------------------------
// rotation matrices
// ---------------------------------------------------------------------------

fn rotation(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:rotate|rotation(?: matrix)?(?: for)?|2d rotation(?: matrix)?)\s+(?:by\s+)?(-?\d+(?:\.\d+)?)\s*(?:degrees?|°|deg)(?:\s+(counterclockwise|counter-clockwise|anticlockwise|clockwise))?$").captures(l)?;
    let mut deg: f64 = m[1].parse().ok()?;
    if m.get(2).map_or(false, |d| d.as_str() == "clockwise") {
        deg = -deg;
    }
    let th = ex(&format!("({}) * pi / 180", fmt_float(deg, 12)))?;
    let c = s::simplify(&func("cos", vec![th.clone()]));
    let sn = s::simplify(&func("sin", vec![th]));
    let ms = s::simplify(&neg(sn.clone()));
    let mat = format!("( {}  {} )\n( {}  {} )", show(&c), show(&ms), show(&sn), show(&c));
    let cv = crate::eval::eval_f64(&c, &[]).unwrap_or(f64::NAN);
    let sv = crate::eval::eval_f64(&sn, &[]).unwrap_or(f64::NAN);
    Some(ans(
        "rotation matrix",
        format!("rotation by {}° {}", n(deg.abs()), if deg >= 0.0 { "counterclockwise" } else { "clockwise" }),
        "Rotation matrix",
        text(mat),
        vec![
            pod("Decimal", "Decimal form", text(format!("( {}  {} )\n( {}  {} )", n(cv), n(-sv), n(sv), n(cv)))),
            pod("Action", "Action on a point", text(format!("(x, y) ↦ ({} x − {} y, {} x + {} y)", n(cv), n(sv), n(sv), n(cv)))),
            pod("Properties", "Properties", text("orthogonal (Rᵀ R = I), determinant 1; the inverse rotates by the opposite angle")),
        ],
    ))
}

// ---------------------------------------------------------------------------
// fractional derivatives of polynomials (Riemann–Liouville and Caputo)
// ---------------------------------------------------------------------------

/// Γ(r) for rational r as an exact expression where possible.
fn gamma_rat(r: &Rat) -> Option<Expr> {
    let one = Rat::from_integer(1.into());
    let half = Rat::new(1.into(), 2.into());
    if r.is_integer() {
        if r <= &Rat::from_integer(0.into()) {
            return None; // pole
        }
        let k: i64 = r.to_integer().try_into().ok()?;
        let mut f = num_bigint::BigInt::from(1);
        for i in 2..k {
            f *= i;
        }
        return Some(Num(Rat::from_integer(f)));
    }
    if r.denom() == &2.into() {
        // Γ(n + 1/2) by the recurrence from Γ(1/2) = √π
        let mut acc = Rat::from_integer(1.into());
        let mut t = half.clone();
        let target = r.clone();
        while t < target {
            acc *= t.clone();
            t += one.clone();
        }
        while t > target {
            t -= one.clone();
            acc /= t.clone();
        }
        return Some(mul(vec![Num(acc), pow(pi(), rat(1, 2))]));
    }
    Some(func("gamma", vec![Num(r.clone())]))
}

fn fractional(l: &str) -> Option<Answer> {
    let (body, var, order, caputo) = if let Some(m) = re!(r"^(?:the\s+)?(fractional|riemann[- ]liouville|caputo|caputod)\s*(?:derivative|differintegral)?\s+(?:of\s+)?(.+?)\s*\{\s*([a-z])\s*,\s*([^}]+?)\s*\}$").captures(l) {
        (m[2].to_string(), m[3].to_string(), m[4].to_string(), m[1].starts_with("caputo"))
    } else if let Some(m) = re!(r"^(?:the\s+)?(half|1/2|one[- ]half)[- ](?:order\s+)?derivative\s+(?:of\s+)?(.+)$").captures(l) {
        (m[2].to_string(), String::new(), "1/2".to_string(), false)
    } else if let Some(m) = re!(r"^(?:the\s+)?(?:(caputo|riemann[- ]liouville)\s+)?(?:fractional\s+)?derivative\s+of\s+order\s+(\S+)\s+of\s+(.+)$").captures(l) {
        (m[3].to_string(), String::new(), m[2].to_string(), m.get(1).map_or(false, |c| c.as_str() == "caputo"))
    } else {
        return None;
    };
    let e = ex(&body)?;
    let x = if var.is_empty() { main_var(&e) } else { sym(&var) };
    let mu = ex(&order)?.as_rat()?.clone();
    if mu <= Rat::from_integer(0.into()) {
        return None;
    }
    let cs = crate::poly::coeffs(&e, &x)?;
    let mut terms = Vec::new();
    let ceil_mu: i64 = mu.ceil().to_integer().try_into().ok()?;
    for (k, c) in cs.iter().enumerate() {
        if c.is_zero() {
            continue;
        }
        if caputo && (k as i64) < ceil_mu {
            continue; // the Caputo derivative removes low powers
        }
        let a = Rat::from_integer((k as i64).into());
        let num_g = gamma_rat(&(a.clone() + Rat::from_integer(1.into())))?;
        let den_arg = a.clone() + Rat::from_integer(1.into()) - mu.clone();
        let Some(den_g) = gamma_rat(&den_arg) else { continue }; // 1/Γ(pole) = 0
        let coeff = mul(vec![c.clone(), num_g, pow(den_g, int(-1))]);
        terms.push(mul(vec![coeff, pow(x.clone(), Num(a - mu.clone()))]));
    }
    let res = s::simplify(&add(terms));
    let kind = if caputo { "Caputo" } else { "Riemann–Liouville" };
    let mut extra = vec![pod("Rule", "Rule", text(format!("D^μ x^a = Γ(a + 1)/Γ(a + 1 − μ) x^(a − μ){}", if caputo { "; the Caputo form drops powers below ⌈μ⌉" } else { "" })))];
    if let Some(v) = crate::eval::eval_f64(&res.subs(&x, &int(1)), &[]) {
        extra.push(pod("Value", &format!("Value at {} = 1", show(&x)), text(n(v))));
    }
    Some(ans(
        "fractional derivative",
        format!("{kind} derivative of order {} of {} with respect to {}", show(&Num(mu)), show(&e), show(&x)),
        "Result",
        text(show(&res)),
        extra,
    ))
}

// ---------------------------------------------------------------------------
// cusps and corners
// ---------------------------------------------------------------------------

fn kink_candidates(e: &Expr, out: &mut Vec<Expr>) {
    match e {
        Func(f, a) => {
            if a.len() == 1 && matches!(&**f, "abs" | "sqrt" | "cbrt") {
                out.push(a[0].clone());
            }
            a.iter().for_each(|t| kink_candidates(t, out));
        }
        Pow(b, p) => {
            if p.as_rat().map_or(false, |r| !r.is_integer()) {
                out.push((**b).clone());
            }
            kink_candidates(b, out);
        }
        Add(v) | Mul(v) => v.iter().for_each(|t| kink_candidates(t, out)),
        _ => {}
    }
}

fn cusps(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(cusps?|corners?|cusps? and corners?|corners? and cusps?|corner points?|cusp points?|sharp points?)\s+(?:of\s+)?(.+)$").captures(l)?;
    let want = m[1].to_string();
    let e = ex(&m[2])?;
    if e.free_symbols().len() != 1 {
        return None;
    }
    let x = main_var(&e);
    let Sym(name) = &x else { return None };
    let f = Program::compile(&e, &[name])?;
    let mut inner = Vec::new();
    kink_candidates(&e, &mut inner);
    let (lo, hi) = (-10.0, 10.0);
    let mut pts: Vec<f64> = Vec::new();
    for a in &inner {
        let pa = Program::compile(a, &[name])?;
        for r in crate::numeric::find_roots(&|t| pa.eval1(t), lo, hi, 20000) {
            let r = (r * 1e9).round() / 1e9;
            if !pts.iter().any(|p| (p - r).abs() < 1e-7) {
                pts.push(r);
            }
        }
    }
    pts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut found: Vec<(f64, &str)> = Vec::new();
    for &c in &pts {
        let fc = f.eval1(c);
        if !fc.is_finite() {
            continue;
        }
        let slope = |h: f64, dir: f64| (f.eval1(c + dir * h) - fc) / (dir * h);
        let (l1, l2) = (slope(1e-5, -1.0), slope(1e-7, -1.0));
        let (r1, r2) = (slope(1e-5, 1.0), slope(1e-7, 1.0));
        if ![l1, l2, r1, r2].iter().all(|v| v.is_finite()) {
            continue;
        }
        let diverging = |a: f64, b: f64| b.abs() > 3.0 * a.abs() && b.abs() > 100.0;
        let (ld, rd) = (diverging(l1, l2), diverging(r1, r2));
        let kind = if ld && rd {
            if l2.signum() != r2.signum() { "cusp" } else { "vertical tangent" }
        } else if !ld && !rd && (l2 - r2).abs() > 1e-3 * (1.0 + l2.abs()) {
            "corner"
        } else if ld != rd {
            "corner (one infinite one-sided slope)"
        } else {
            continue;
        };
        found.push((c, kind));
    }
    let keep = |k: &str| if want.starts_with("cusp") && !want.contains("corner") { k == "cusp" } else if want.starts_with("corner") && !want.contains("cusp") { k.starts_with("corner") } else { k != "vertical tangent" };
    let sel: Vec<&(f64, &str)> = found.iter().filter(|(_, k)| keep(k)).collect();
    let periodic = super::math::period(&e, &x).is_some() || inner.iter().any(|a| super::math::period(a, &x).is_some());
    let main = if sel.is_empty() {
        format!("no {} found for {} ≤ {} ≤ {}", if want.starts_with("cusp") { "cusps" } else { "corners" }, n(lo), name, n(hi))
    } else {
        sel.iter().map(|(c, _)| format!("({}, {})", shown(*c), shown(f.eval1(*c)))).collect::<Vec<_>>().join(", ")
    };
    let rows: Vec<Vec<Content>> = found.iter().map(|(c, k)| vec![text(format!("{} = {}", name, shown(*c))), text(k.to_string())]).collect();
    let mut extra = Vec::new();
    if !rows.is_empty() {
        extra.push(pod("Points", "Points where the derivative fails to exist", table(Some(vec!["point", "kind"]), rows)));
    }
    if periodic {
        extra.push(pod("Note", "Note", text(format!("the pattern repeats; points are listed for {} ≤ {} ≤ {}", n(lo), name, n(hi)))));
    }
    Some(ans(&format!("{want} of a function"), format!("{want} | {}", show(&e)), if want.starts_with("cusp") { "Cusps" } else { "Corners" }, text(main), extra))
}

// ---------------------------------------------------------------------------
// area between a curve and the x-axis
// ---------------------------------------------------------------------------

fn area_axis(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(?:total\s+)?area\s+between\s+(?:the\s+curve\s+)?(?:y\s*=\s*)?(.+?)\s+and\s+the\s+x[- ]?axis\s+(?:from|between|for)\s+(?:x\s*=\s*)?(.+?)\s+(?:to|and)\s+(?:x\s*=\s*)?(.+)$").captures(l)?;
    let e = ex(&m[1])?;
    if e.free_symbols().len() != 1 {
        return None;
    }
    let x = main_var(&e);
    let Sym(name) = &x else { return None };
    let a = crate::eval::eval_f64(&ex(&m[2])?, &[])?;
    let b = crate::eval::eval_f64(&ex(&m[3])?, &[])?;
    let f = Program::compile(&e, &[name])?;
    let roots: Vec<f64> = crate::numeric::find_roots(&|t| f.eval1(t), a.min(b), a.max(b), 20000);
    let area = crate::numeric::quad_breaks(&|t| f.eval1(t).abs(), a, b, &roots, 1e-10);
    let signed = crate::numeric::quad_breaks(&|t| f.eval1(t), a, b, &roots, 1e-10);
    if !area.value.is_finite() {
        return None;
    }
    Some(ans(
        "area between a curve and the x-axis",
        format!("area between y = {} and the x-axis for {} ≤ {name} ≤ {}", show(&e), m[2].trim(), m[3].trim()),
        "Area",
        text(n(area.value)),
        vec![
            pod("Integral", "As an integral", text(format!("∫ |{}| d{name} from {} to {}", show(&e), m[2].trim(), m[3].trim()))),
            pod("Signed", "Signed integral (area above minus below)", text(n(signed.value))),
            pod("Crossings", "Crossings of the axis inside the interval", text(if roots.is_empty() { "none".into() } else { format!("{} crossings", roots.len()) })),
        ],
    ))
}

// ---------------------------------------------------------------------------
// critical points near a given point
// ---------------------------------------------------------------------------

fn critical_near(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(saddle points?|critical points?|stationary points?|local (?:minimum|maximum|minima|maxima|extremum|extrema)|extremum|minimum|maximum)\s+of\s+(.+?)\s+near\s+\(\s*([^,]+?)\s*,\s*([^)]+?)\s*\)$").captures(l)?;
    let e = ex(&m[2])?;
    let mut vars: Vec<String> = e.free_symbols().into_iter().map(|v| v.to_string()).collect();
    vars.sort();
    if vars.len() != 2 {
        return None;
    }
    let (xs, ys) = (sym(&vars[0]), sym(&vars[1]));
    let names = [vars[0].as_str(), vars[1].as_str()];
    let gx = s::simplify(&diff(&e, &xs));
    let gy = s::simplify(&diff(&e, &ys));
    let pgx = Program::compile(&gx, &names)?;
    let pgy = Program::compile(&gy, &names)?;
    let hxx = Program::compile(&s::simplify(&diff(&gx, &xs)), &names)?;
    let hxy = Program::compile(&s::simplify(&diff(&gx, &ys)), &names)?;
    let hyy = Program::compile(&s::simplify(&diff(&gy, &ys)), &names)?;
    let x0 = crate::eval::eval_f64(&ex(&m[3])?, &[])?;
    let y0 = crate::eval::eval_f64(&ex(&m[4])?, &[])?;
    // Newton on the gradient with the exact Hessian; when it does not settle
    // from the given point, start from a widening grid around it and keep
    // the converged point nearest to it
    let newton = |start: [f64; 2]| -> Option<Vec<f64>> {
        let mut sol = start.to_vec();
        for _ in 0..100 {
            let (g1, g2) = (pgx.eval(&sol), pgy.eval(&sol));
            if !(g1.is_finite() && g2.is_finite()) {
                return None;
            }
            if g1.hypot(g2) < 1e-13 {
                break;
            }
            let (a, b, c) = (hxx.eval(&sol), hxy.eval(&sol), hyy.eval(&sol));
            let det = a * c - b * b;
            if !det.is_finite() || det.abs() < 1e-300 {
                return None;
            }
            let dx = (c * g1 - b * g2) / det;
            let dy = (a * g2 - b * g1) / det;
            let step = dx.hypot(dy);
            let k = if step > 0.5 { 0.5 / step } else { 1.0 };
            sol[0] -= k * dx;
            sol[1] -= k * dy;
        }
        (pgx.eval(&sol).hypot(pgy.eval(&sol)) < 1e-9).then_some(sol)
    };
    let dist = |v: &[f64]| (v[0] - x0).hypot(v[1] - y0);
    let mut sol = newton([x0, y0]).filter(|v| dist(v) < 1.0);
    let mut radius = 1.0;
    while sol.is_none() && radius <= 16.0 {
        let mut best: Option<Vec<f64>> = None;
        let k = 12;
        for i in 0..=k {
            for j in 0..=k {
                let st = [x0 - radius + 2.0 * radius * i as f64 / k as f64, y0 - radius + 2.0 * radius * j as f64 / k as f64];
                if let Some(v) = newton(st) {
                    if dist(&v) <= radius * 1.5 && best.as_ref().map_or(true, |b| dist(&v) < dist(b)) {
                        best = Some(v);
                    }
                }
            }
        }
        sol = best;
        radius *= 2.0;
    }
    let sol = sol?;
    let (px, py) = (sol[0], sol[1]);
    let pe = Program::compile(&e, &names)?;
    let (a, b, c) = (hxx.eval(&sol), hxy.eval(&sol), hyy.eval(&sol));
    let det = a * c - b * b;
    let kind = if det < -1e-12 { "saddle point" } else if det > 1e-12 { if a > 0.0 { "local minimum" } else { "local maximum" } } else { "degenerate critical point (second-derivative test inconclusive)" };
    Some(ans(
        "critical point",
        format!("critical point of {} near ({}, {})", show(&e), m[3].trim(), m[4].trim()),
        "Result",
        text(format!("{kind} at ({}, {}) ≈ ({}, {}), value {}", vars[0], vars[1], n(px), n(py), n(pe.eval(&sol)))),
        vec![
            pod("Hessian", "Hessian there", text(format!("( {}  {} )\n( {}  {} ),  determinant {}", n(a), n(b), n(b), n(c), n(det)))),
            pod("Gradient", "Gradient", text(format!("({}, {})", show(&gx), show(&gy)))),
        ],
    ))
}

// ---------------------------------------------------------------------------
// arithmetic of number classes
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tri {
    Yes,
    No,
    Unknown,
}

#[derive(Clone, Copy, Debug)]
struct Class {
    integer: Tri,
    even: Tri, // meaningful when integer
    rational: Tri,
    sign: i8, // 1 positive, -1 negative, 0 zero, 2 unknown, 3 non-negative
}

fn class_of(word: &str) -> Option<Class> {
    let w = word.trim();
    let mut c = Class { integer: Tri::Unknown, even: Tri::Unknown, rational: Tri::Unknown, sign: 2 };
    let mut any = false;
    for part in w.split_whitespace() {
        match part {
            "even" => { c.integer = Tri::Yes; c.even = Tri::Yes; c.rational = Tri::Yes; any = true; }
            "odd" => { c.integer = Tri::Yes; c.even = Tri::No; c.rational = Tri::Yes; c.sign = if c.sign == 2 { 2 } else { c.sign }; any = true; }
            "integer" | "integers" | "whole" => { c.integer = Tri::Yes; c.rational = Tri::Yes; any = true; }
            "natural" => { c.integer = Tri::Yes; c.rational = Tri::Yes; c.sign = 1; any = true; }
            "rational" => { c.rational = Tri::Yes; any = true; }
            "irrational" => { c.rational = Tri::No; c.integer = Tri::No; any = true; }
            "positive" => { c.sign = 1; any = true; }
            "negative" => { c.sign = -1; any = true; }
            "nonnegative" | "non-negative" => { c.sign = 3; any = true; }
            "real" | "number" | "numbers" => { any = true; }
            _ => return None,
        }
    }
    if any { Some(c) } else { None }
}

fn lit(v: i64) -> Class {
    Class { integer: Tri::Yes, even: if v % 2 == 0 { Tri::Yes } else { Tri::No }, rational: Tri::Yes, sign: v.signum() as i8 }
}

fn both(a: Tri, b: Tri) -> Tri {
    match (a, b) {
        (Tri::Yes, Tri::Yes) => Tri::Yes,
        _ => Tri::Unknown,
    }
}

fn add_c(a: Class, b: Class, minus: bool) -> Class {
    let integer = both(a.integer, b.integer);
    let even = if integer == Tri::Yes {
        match (a.even, b.even) {
            (Tri::Yes, Tri::Yes) | (Tri::No, Tri::No) => Tri::Yes,
            (Tri::Yes, Tri::No) | (Tri::No, Tri::Yes) => Tri::No,
            _ => Tri::Unknown,
        }
    } else {
        Tri::Unknown
    };
    let rational = match (a.rational, b.rational) {
        (Tri::Yes, Tri::Yes) => Tri::Yes,
        (Tri::Yes, Tri::No) | (Tri::No, Tri::Yes) => Tri::No,
        _ => Tri::Unknown,
    };
    let bs = if minus { match b.sign { 1 => -1, -1 => 1, 0 => 0, 3 => 4, s => s } } else { b.sign };
    let sign = match (a.sign, bs) {
        (x, 0) => x,
        (0, y) if y != 4 => y,
        (1, 1) | (1, 3) | (3, 1) => 1,
        (-1, -1) => -1,
        (3, 3) => 3,
        _ => 2,
    };
    Class { integer: if rational == Tri::No { Tri::No } else { integer }, even, rational, sign }
}

fn mul_c(a: Class, b: Class) -> Class {
    let integer = both(a.integer, b.integer);
    let even = if integer == Tri::Yes {
        if a.even == Tri::Yes || b.even == Tri::Yes { Tri::Yes } else if a.even == Tri::No && b.even == Tri::No { Tri::No } else { Tri::Unknown }
    } else {
        Tri::Unknown
    };
    let rational = both(a.rational, b.rational);
    let sign = match (a.sign, b.sign) {
        (0, _) | (_, 0) => 0,
        (1, 1) | (-1, -1) => 1,
        (1, -1) | (-1, 1) => -1,
        (3, 1) | (1, 3) | (3, 3) => 3,
        _ => 2,
    };
    Class { integer, even, rational, sign }
}

fn pow_c(a: Class, k: i64) -> Option<Class> {
    if k < 1 {
        return None;
    }
    let sign = match a.sign {
        0 => 0,
        1 => 1,
        -1 => if k % 2 == 0 { 1 } else { -1 },
        3 => 3,
        _ => if k % 2 == 0 { 3 } else { 2 },
    };
    Some(Class { integer: a.integer, even: a.even, rational: if a.rational == Tri::Yes { Tri::Yes } else { Tri::Unknown }, sign })
}

fn describe(c: Class) -> String {
    let sign = match c.sign {
        1 => "positive ",
        -1 => "negative ",
        0 => "zero: ",
        3 => "non-negative ",
        _ => "",
    };
    let kind = if c.integer == Tri::Yes {
        match c.even {
            Tri::Yes => "even integer",
            Tri::No => "odd integer",
            _ => "integer",
        }
    } else if c.rational == Tri::Yes {
        "rational number"
    } else if c.rational == Tri::No {
        "irrational number"
    } else {
        "real number"
    };
    format!("{sign}{kind}")
}

fn number_classes(l: &str) -> Option<Answer> {
    if !re!(r"\b(even|odd|integer|rational|irrational|natural|positive|negative)\b").is_match(l) {
        return None;
    }
    // tokens: class phrases, integers, operators, parentheses
    let words = r"(?:(?:positive|negative|non-?negative|even|odd|rational|irrational|natural|whole|real)\s+)*(?:number|numbers|integer|integers|even|odd|rational|irrational)";
    let tok_re = Regex::new(&format!(r"^\s*(?:({words})|(\d+)|([-+*/^()×]))")).ok()?;
    let mut toks: Vec<(Option<Class>, Option<i64>, Option<char>)> = Vec::new();
    let mut rest = l;
    while !rest.trim().is_empty() {
        let m = tok_re.captures(rest)?;
        if let Some(w) = m.get(1) {
            toks.push((Some(class_of(w.as_str())?), None, None));
        } else if let Some(d) = m.get(2) {
            toks.push((None, Some(d.as_str().parse().ok()?), None));
        } else {
            let ch = m.get(3)?.as_str().chars().next()?;
            toks.push((None, None, Some(if ch == '×' { '*' } else { ch })));
        }
        rest = &rest[m.get(0)?.end()..];
    }
    if !toks.iter().any(|t| t.0.is_some()) || toks.len() < 2 {
        return None;
    }
    struct P<'a> {
        t: &'a [(Option<Class>, Option<i64>, Option<char>)],
        i: usize,
    }
    impl P<'_> {
        fn op(&self) -> Option<char> {
            self.t.get(self.i).and_then(|t| t.2)
        }
        fn expr(&mut self) -> Option<Class> {
            let mut a = self.term()?;
            while let Some(o @ ('+' | '-')) = self.op() {
                self.i += 1;
                let b = self.term()?;
                a = add_c(a, b, o == '-');
            }
            Some(a)
        }
        fn term(&mut self) -> Option<Class> {
            let mut a = self.power()?;
            while let Some('*') = self.op() {
                self.i += 1;
                let b = self.power()?;
                a = mul_c(a, b);
            }
            if self.op() == Some('/') {
                return None;
            }
            Some(a)
        }
        fn power(&mut self) -> Option<Class> {
            let a = self.atom()?;
            if self.op() == Some('^') {
                self.i += 1;
                let k = self.t.get(self.i)?.1?;
                self.i += 1;
                return pow_c(a, k);
            }
            Some(a)
        }
        fn atom(&mut self) -> Option<Class> {
            let t = self.t.get(self.i)?;
            self.i += 1;
            if let Some(c) = t.0 {
                return Some(c);
            }
            if let Some(v) = t.1 {
                return Some(lit(v));
            }
            if t.2 == Some('(') {
                let c = self.expr()?;
                if self.op() != Some(')') {
                    return None;
                }
                self.i += 1;
                return Some(c);
            }
            if t.2 == Some('-') {
                let c = self.atom()?;
                return Some(mul_c(lit(-1), c));
            }
            None
        }
    }
    let mut p = P { t: &toks, i: 0 };
    let c = p.expr()?;
    if p.i != toks.len() {
        return None;
    }
    let mut d = describe(c);
    if d == "real number" {
        d = "a real number (nothing more can be said in general)".into();
    }
    Some(ans("number classes", l.to_string(), "Result", text(d), vec![pod("Rules", "Rules used", text("even ± even = even, odd ± odd = even, even ± odd = odd; even × integer = even, odd × odd = odd; rational ± irrational = irrational; a negative number to an even power is positive"))]))
}

// ---------------------------------------------------------------------------
// transfer functions
// ---------------------------------------------------------------------------

fn transfer(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(?:transfer function|poles and zeros of(?: the transfer function)?|stability of(?: the transfer function)?)\s+(?:h\s*\(\s*s\s*\)\s*=\s*|g\s*\(\s*s\s*\)\s*=\s*)?(.+)$").captures(l)?;
    let e = ex(&m[1])?;
    let s_ = sym("s");
    if !e.free_symbols().iter().all(|v| &**v == "s") {
        return None;
    }
    let (num_e, den_e) = s::numer_denom(&s::simplify(&e));
    let np = crate::poly::to_qpoly(&num_e, &s_)?;
    let dp = crate::poly::to_qpoly(&den_e, &s_)?;
    if dp.degree() == 0 {
        return None;
    }
    let fmtc = |z: num_complex::Complex64| {
        let (re, im) = (crate::eval::chop(z.re), crate::eval::chop(z.im));
        if im.abs() < 1e-9 { n(re) } else { format!("{} {} {}i", n(re), if im < 0.0 { "−" } else { "+" }, n(im.abs())) }
    };
    let poles = dp.roots_c();
    let zeros = np.roots_c();
    let stable = poles.iter().all(|p| p.re < -1e-12);
    let marginal = !stable && poles.iter().all(|p| p.re <= 1e-12);
    let dc = {
        let d0 = crate::eval::eval_f64(&den_e.subs(&s_, &int(0)), &[]).unwrap_or(f64::NAN);
        let n0 = crate::eval::eval_f64(&num_e.subs(&s_, &int(0)), &[]).unwrap_or(f64::NAN);
        if d0.abs() < 1e-14 { "infinite (pole at s = 0)".to_string() } else { shown(n0 / d0) }
    };
    let rows = vec![
        vec![text("poles"), text(poles.iter().map(|z| fmtc(*z)).collect::<Vec<_>>().join(", "))],
        vec![text("zeros"), text(if zeros.is_empty() { "none".into() } else { zeros.iter().map(|z| fmtc(*z)).collect::<Vec<_>>().join(", ") })],
        vec![text("order"), text(dp.degree().to_string())],
        vec![text("relative degree"), text(format!("{}{}", dp.degree() as i64 - np.degree() as i64, if np.degree() > dp.degree() { " (improper)" } else if np.degree() == dp.degree() { " (biproper)" } else { " (strictly proper)" }))],
        vec![text("DC gain H(0)"), text(dc)],
    ];
    Some(ans(
        "transfer function",
        format!("transfer function H(s) = {}", show(&e)),
        "Stability",
        text(if stable { "stable: every pole has negative real part".to_string() } else if marginal { "marginally stable: poles on the imaginary axis".to_string() } else { "unstable: a pole lies in the right half-plane".to_string() }),
        vec![pod("Properties", "Poles, zeros and gain", table(None, rows))],
    ))
}

// ---------------------------------------------------------------------------
// the predator–prey model
// ---------------------------------------------------------------------------

fn predator_prey(l: &str) -> Option<Answer> {
    if !matches!(l, "predator-prey model" | "predator prey model" | "predator-prey equations" | "predator prey equations" | "lotka-volterra" | "lotka-volterra equations" | "lotka volterra equations" | "lotka-volterra model" | "predator-prey") {
        return None;
    }
    Some(ans(
        "predator–prey model",
        "Lotka–Volterra predator–prey model".into(),
        "Equations",
        text("x′ = α x − β x y   (prey)\ny′ = δ x y − γ y   (predators)"),
        vec![
            pod("Equilibria", "Equilibria", text("(0, 0) (a saddle) and (γ/δ, α/β) (a centre)")),
            pod("Conserved", "Conserved quantity", text("V = δ x − γ ln x + β y − α ln y is constant along orbits, so populations cycle")),
            pod("Period", "Small oscillations", text("period ≈ 2π/√(α γ) near the coexistence equilibrium")),
            pod("Parameters", "Parameters", text("α prey growth rate, β predation rate, δ predator growth per prey eaten, γ predator death rate")),
        ],
    ))
}

// ---------------------------------------------------------------------------
// arithmetic with units
// ---------------------------------------------------------------------------

fn unit_arith(l: &str) -> Option<Answer> {
    use crate::domains::units;
    if !re!(r"[-+*/×÷]").is_match(l) {
        return None;
    }
    // replace each "number unit" with a placeholder
    let qre = re!(r"(\d+(?:\.\d+)?(?:e-?\d+)?)\s*([a-zA-Zµμ°][a-zA-Z²³^0-9]*(?:\s+(?:per|squared|cubed)\s+[a-zA-Z]+)?)");
    let mut qs: Vec<(f64, units::Dim, String)> = Vec::new();
    let mut out = String::new();
    let mut last = 0;
    for m in qre.captures_iter(l) {
        let whole = m.get(0)?;
        let (v, u, _) = units::parse_quantity(whole.as_str())?;
        out.push_str(&l[last..whole.start()]);
        out.push_str(&format!(" Q{} ", qs.len()));
        qs.push((units::to_si(v, &u), u.dim, u.name.clone()));
        last = whole.end();
    }
    out.push_str(&l[last..]);
    if qs.len() < 2 {
        return None;
    }
    // tokens
    let toks: Vec<String> = re!(r"Q\d+|\d+(?:\.\d+)?|[-+*/×÷()]").find_iter(&out).map(|m| m.as_str().to_string()).collect();
    let rebuilt: String = toks.concat();
    if rebuilt.len() != out.chars().filter(|c| !c.is_whitespace()).collect::<String>().len() {
        return None;
    }
    type V = (f64, units::Dim);
    fn dim_add(a: units::Dim, b: units::Dim, sgn: i8) -> units::Dim {
        let mut d = a;
        for i in 0..d.len() {
            d[i] += sgn * b[i];
        }
        d
    }
    struct P<'a> {
        t: &'a [String],
        i: usize,
        q: &'a [(f64, units::Dim, String)],
    }
    impl P<'_> {
        fn peek(&self) -> Option<&str> {
            self.t.get(self.i).map(|s| s.as_str())
        }
        fn expr(&mut self) -> Option<V> {
            let mut a = self.term()?;
            while let Some(o @ ("+" | "-")) = self.peek() {
                let minus = o == "-";
                self.i += 1;
                let b = self.term()?;
                if a.1 != b.1 {
                    return None;
                }
                a = (if minus { a.0 - b.0 } else { a.0 + b.0 }, a.1);
            }
            Some(a)
        }
        fn term(&mut self) -> Option<V> {
            let mut a = self.atom()?;
            while let Some(o @ ("*" | "/" | "×" | "÷")) = self.peek() {
                let div = o == "/" || o == "÷";
                self.i += 1;
                let b = self.atom()?;
                a = if div { (a.0 / b.0, dim_add(a.1, b.1, -1)) } else { (a.0 * b.0, dim_add(a.1, b.1, 1)) };
            }
            Some(a)
        }
        fn atom(&mut self) -> Option<V> {
            let t = self.t.get(self.i)?.clone();
            self.i += 1;
            if t == "(" {
                let v = self.expr()?;
                if self.peek() != Some(")") {
                    return None;
                }
                self.i += 1;
                return Some(v);
            }
            if let Some(k) = t.strip_prefix('Q') {
                let q = self.q.get(k.parse::<usize>().ok()?)?;
                return Some((q.0, q.1));
            }
            Some((t.parse().ok()?, [0; 8]))
        }
    }
    let mut p = P { t: &toks, i: 0, q: &qs };
    let (v, d) = p.expr()?;
    if p.i != toks.len() || !v.is_finite() {
        return None;
    }
    let si = units::si_name(&d);
    let unit = if si.is_empty() { String::new() } else { format!(" {si}") };
    let mut extra = Vec::new();
    if d == [1, 0, -1, 0, 0, 0, 0, 0] {
        extra.push(pod("Other", "Unit conversions", text(format!("{} km/h\n{} mph\n{} ft/s", n(v * 3.6), n(v / 0.44704), n(v / 0.3048)))));
    }
    Some(ans("unit arithmetic", l.to_string(), "Result", text(format!("{}{unit}", n(v))), extra))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    restricted(l)
        .or_else(|| periods(l))
        .or_else(|| rotation(l))
        .or_else(|| fractional(l))
        .or_else(|| cusps(l))
        .or_else(|| area_axis(l))
        .or_else(|| critical_near(l))
        .or_else(|| number_classes(l))
        .or_else(|| transfer(l))
        .or_else(|| predator_prey(l))
        .or_else(|| unit_arith(l))
        .or_else(|| multi_domain(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_class_rules() {
        let q = |s: &str| number_classes(s).map(|a| content_text(&a.pods[1].subpods[0].content));
        assert_eq!(q("1 + (even number * odd number)").as_deref(), Some("odd integer"));
        assert_eq!(q("negative number ^ 40").as_deref(), Some("positive real number"));
        assert_eq!(q("rational number + positive even").as_deref(), Some("rational number"));
    }

    #[test]
    fn half_integer_gamma() {
        // Γ(5/2) = 3√π/4
        let g = gamma_rat(&Rat::new(5.into(), 2.into())).unwrap();
        let v = crate::eval::eval_f64(&g, &[]).unwrap();
        assert!((v - 1.329_340_388).abs() < 1e-8);
    }
}
