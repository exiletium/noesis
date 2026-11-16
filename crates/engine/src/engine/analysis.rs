//! Calculus questions in words: continuity and differentiability
//! ("is tan(x) continuous at pi", "discontinuities of sec(x)tan(x)",
//! "does abs(x) have a derivative"), behaviour at a point or on an interval
//! ("is x^3 increasing at x=0", "concavity of cos x + x^3/48 on 1<x<1.8"),
//! directional derivatives, Jacobians, volumes of revolution, lengths of
//! parametric curves, iterated integrals written with "dx dy, x=a..b, y=c..d",
//! upper and lower limits, and discrete limits.

use super::*;
use crate::calculus::{self, integrate::Definite, limit::Dir};
use crate::eval::Program;
use crate::expr::Expr::*;
use crate::expr::*;
use crate::print::text as ptext;
use crate::simplify as s;
use crate::solve;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// Mathematica-style brackets and names: "cos[x]", "UnitStep[x-2]".
fn normalise(s: &str) -> String {
    let mut t = s.trim().to_string();
    for _ in 0..4 {
        let n = re!(r"\b([A-Za-z]+)\[([^\[\]]*)\]").replace_all(&t, "$1($2)").to_string();
        if n == t {
            break;
        }
        t = n;
    }
    t.replace("UnitStep", "heaviside").replace("Heaviside", "heaviside")
}

fn ex(s: &str) -> Option<Expr> {
    let t = normalise(s);
    let t = re!(r"^(?:the\s+)?(?:curve\s+|function\s+)?(?:[fgh]\s*\(\s*[a-z]\s*\)\s*=\s*|y\s*=\s*)?").replace(&t, "").to_string();
    crate::parse::parse(t.trim()).ok().map(|p| p.expr)
}

fn num(e: &Expr) -> Option<f64> {
    crate::eval::eval_f64(e, &[]).filter(|v| v.is_finite())
}

fn bound(s: &str) -> Option<Expr> {
    match s.trim().to_lowercase().as_str() {
        "inf" | "infinity" | "∞" | "+inf" | "+infinity" | "oo" | "+oo" => Some(inf()),
        "-inf" | "-infinity" | "-∞" | "-oo" => Some(neg(inf())),
        t => ex(t),
    }
}

fn main_var(e: &Expr) -> Expr {
    let fs = e.free_symbols();
    for c in ["x", "t", "y", "z", "n", "u", "s", "r"] {
        if fs.iter().any(|n| &**n == c) {
            return sym(c);
        }
    }
    fs.into_iter().next().map(|n| sym(&n)).unwrap_or_else(|| sym("x"))
}

fn show(e: &Expr) -> String {
    ptext(e)
}

fn ans(title: &str, input: String, result_title: &str, result: Content, extra: Vec<Pod>) -> Answer {
    let mut pods = vec![pod("Input", "Input interpretation", text(input)), primary(pod("Result", result_title, result))];
    pods.extend(extra);
    Answer::new(title, pods)
}

// ---------------------------------------------------------------------------
// numeric probing
// ---------------------------------------------------------------------------

struct F {
    p: Program,
}

impl F {
    fn new(e: &Expr, x: &Expr) -> Option<F> {
        let Sym(n) = x else { return None };
        Some(F { p: Program::compile(e, &[n])? })
    }
    fn at(&self, x: f64) -> f64 {
        self.p.eval1(x)
    }
    /// One-sided limit estimate: Some(value) when the samples settle.
    fn side(&self, a: f64, dir: f64) -> Side {
        let hs = [1e-3, 1e-4, 1e-5, 1e-6, 1e-7];
        let vals: Vec<f64> = hs.iter().map(|h| self.at(a + dir * h * a.abs().max(1.0))).collect();
        if vals.iter().all(|v| v.is_nan()) {
            return Side::Undefined;
        }
        let growing = vals.windows(2).all(|w| w[1].abs() > w[0].abs() && w[0].signum() == w[1].signum());
        if (vals[3..].iter().all(|v| v.abs() > 1e6) && vals[4].abs() > vals[2].abs()) || (growing && vals[4].abs() > 10.0 && vals[4].is_finite()) || vals[4].is_infinite() {
            return Side::Infinite;
        }
        let (a3, a4) = (vals[3], vals[4]);
        if a3.is_finite() && a4.is_finite() && (a3 - a4).abs() <= 1e-4 * (1.0 + a4.abs()) {
            return Side::Value(a4);
        }
        Side::Oscillates
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Side {
    Value(f64),
    Infinite,
    Oscillates,
    Undefined,
}

#[derive(Clone, Debug)]
enum Kind {
    Continuous,
    Removable,
    Jump,
    Infinite,
    Oscillating,
    OutsideDomain,
}

fn classify(f: &F, a: f64) -> Kind {
    let (l, r) = (f.side(a, -1.0), f.side(a, 1.0));
    let v = f.at(a);
    match (l, r) {
        (Side::Value(x), Side::Value(y)) if (x - y).abs() <= 1e-4 * (1.0 + x.abs()) => {
            if v.is_finite() && (v - x).abs() <= 1e-4 * (1.0 + x.abs()) {
                Kind::Continuous
            } else {
                Kind::Removable
            }
        }
        (Side::Value(_), Side::Value(_)) => Kind::Jump,
        (Side::Infinite, _) | (_, Side::Infinite) => Kind::Infinite,
        (Side::Undefined, Side::Undefined) => Kind::OutsideDomain,
        (Side::Undefined, Side::Value(_)) | (Side::Value(_), Side::Undefined) => {
            if v.is_finite() {
                Kind::Continuous // an endpoint of the domain
            } else {
                Kind::OutsideDomain
            }
        }
        _ => Kind::Oscillating,
    }
}

fn kind_text(k: &Kind) -> &'static str {
    match k {
        Kind::Continuous => "continuous",
        Kind::Removable => "removable discontinuity",
        Kind::Jump => "jump discontinuity",
        Kind::Infinite => "infinite discontinuity",
        Kind::Oscillating => "essential (oscillating) discontinuity",
        Kind::OutsideDomain => "not in the domain",
    }
}

/// Points where an expression may fail to be continuous: zeros of
/// denominators, poles of tan/sec/cot/csc, jumps of step functions.
fn candidates(e: &Expr, x: &Expr, out: &mut Vec<(Expr, bool)>) {
    let push_zeros = |g: &Expr, out: &mut Vec<(Expr, bool)>| {
        if !g.has_sym(x) {
            return;
        }
        let rs = solve::solve(g, x);
        for r in rs.real() {
            out.push((r, rs.periodic));
        }
    };
    match e {
        Pow(b, p) => {
            candidates(b, x, out);
            candidates(p, x, out);
            if num(p).is_some_and(|v| v < 0.0) {
                push_zeros(b, out);
            }
        }
        Func(name, args) => {
            for a in args {
                candidates(a, x, out);
            }
            let a = args.first().cloned().unwrap_or_else(zero);
            match &**name {
                "tan" | "sec" => push_zeros(&func("cos", vec![a]), out),
                "cot" | "csc" => push_zeros(&func("sin", vec![a]), out),
                "heaviside" | "sign" => push_zeros(&a, out),
                _ => {}
            }
        }
        Add(v) | Mul(v) | List(v) => v.iter().for_each(|t| candidates(t, x, out)),
        _ => {}
    }
}

/// Points where a continuous expression may have no derivative.
fn kinks(e: &Expr, x: &Expr, out: &mut Vec<(Expr, bool)>) {
    let push_zeros = |g: &Expr, out: &mut Vec<(Expr, bool)>| {
        if !g.has_sym(x) {
            return;
        }
        let rs = solve::solve(g, x);
        for r in rs.real() {
            out.push((r, rs.periodic));
        }
    };
    match e {
        Pow(b, p) => {
            kinks(b, x, out);
            if num(p).is_some_and(|v| v.fract() != 0.0 && v < 1.0 || v < 0.0) {
                push_zeros(b, out);
            }
        }
        Func(name, args) => {
            for a in args {
                kinks(a, x, out);
            }
            if matches!(&**name, "abs" | "sqrt" | "cbrt" | "floor" | "ceil" | "round" | "heaviside" | "sign" | "asin" | "acos") {
                if let Some(a) = args.first() {
                    if matches!(&**name, "asin" | "acos") {
                        push_zeros(&sub(square(a.clone()), one()), out);
                    } else {
                        push_zeros(a, out);
                    }
                }
            }
        }
        Add(v) | Mul(v) | List(v) => v.iter().for_each(|t| kinks(t, x, out)),
        _ => {}
    }
}

fn dedup_points(v: Vec<(Expr, bool)>) -> Vec<(Expr, bool, f64)> {
    let mut out: Vec<(Expr, bool, f64)> = Vec::new();
    for (e, per) in v {
        let e = s::simplify(&e);
        // periodic families: show the family, probe the member nearest 0
        let probe = if per { e.subs(&sym(solve::PARAM), &zero()) } else { e.clone() };
        let Some(val) = num(&probe) else { continue };
        if !out.iter().any(|(o, p2, w)| (*w - val).abs() < 1e-9 && *p2 == per && (per || *o == e || true)) {
            out.push((e, per, val));
        }
    }
    let mut out = merge_families(out);
    out.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
    out
}

/// Merge periodic families c_i + k n whose offsets are evenly spaced by
/// k/m into one family (π/2 + 2πn and -π/2 + 2πn become π/2 + πn).
fn merge_families(v: Vec<(Expr, bool, f64)>) -> Vec<(Expr, bool, f64)> {
    let n = sym(solve::PARAM);
    let step = |e: &Expr| num(&s::simplify(&calculus::diff(e, &n)));
    let mut out: Vec<(Expr, bool, f64)> = Vec::new();
    let mut used = vec![false; v.len()];
    for i in 0..v.len() {
        if used[i] {
            continue;
        }
        let (e, per, val) = &v[i];
        let Some(k) = step(e).filter(|k| *per && k.abs() > 1e-12) else {
            out.push(v[i].clone());
            continue;
        };
        let group: Vec<usize> = (i..v.len()).filter(|&j| !used[j] && v[j].1 && step(&v[j].0).is_some_and(|k2| (k2 - k).abs() < 1e-9)).collect();
        let m = group.len();
        let mut offs: Vec<f64> = group.iter().map(|&j| v[j].2.rem_euclid(k.abs())).collect();
        offs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let d = k.abs() / m as f64;
        let even = m > 1 && offs.windows(2).all(|w| (w[1] - w[0] - d).abs() < 1e-9);
        if !even {
            used[i] = true;
            out.push((e.clone(), *per, *val));
            continue;
        }
        let best = *group.iter().min_by(|&&a, &&b| v[a].2.abs().partial_cmp(&v[b].2.abs()).unwrap().then(v[b].2.partial_cmp(&v[a].2).unwrap())).unwrap();
        let c = s::simplify(&v[best].0.subs(&n, &zero()));
        let ke = s::simplify(&div(calculus::diff(&v[best].0, &n), int(m as i64)));
        for &j in &group {
            used[j] = true;
        }
        out.push((s::simplify(&add(vec![c, mul(vec![ke, n.clone()])])), true, v[best].2));
    }
    out
}

fn point_text(e: &Expr, per: bool, x: &Expr) -> String {
    if per {
        format!("{} = {} (n ∈ ℤ)", show(x), show(e))
    } else {
        format!("{} = {}", show(x), show(e))
    }
}

/// One-sided difference quotients agree?
fn differentiable_at(f: &F, a: f64) -> bool {
    let fa = f.at(a);
    if !fa.is_finite() {
        return false;
    }
    let q = |h: f64| (f.at(a + h) - fa) / h;
    let (r1, r2) = (q(1e-6), q(1e-7));
    let (l1, l2) = (q(-1e-6), q(-1e-7));
    let settled = |u: f64, v: f64| u.is_finite() && v.is_finite() && (u - v).abs() <= 1e-3 * (1.0 + v.abs()) && v.abs() < 1e5;
    settled(r1, r2) && settled(l1, l2) && (r2 - l2).abs() <= 1e-3 * (1.0 + r2.abs())
}

fn continuity(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches('?').trim();
    let l = c.to_lowercase();
    let (body, prop, at) = if let Some(m) = re!(r"(?i)^(?:is|are)\s+(?:the\s+)?(?:function\s+)?(.+?)\s+(?:an?\s+)?(continuous|differentiable)(?:\s+function)?(?:\s+(?:at|for)\s+(?:[a-z]\s*=\s*)?(.+?)|\s+(?:over|on|in)\s+(?:the\s+)?(?:reals|real line|real numbers|r|ℝ)|\s+everywhere)?$").captures(c) {
        (m[1].to_string(), m[2].to_lowercase(), m.get(3).map(|x| x.as_str().to_string()))
    } else if let Some(m) = re!(r"(?i)^(?:does|do)\s+(.+?)\s+have\s+a\s+derivative(?:\s+at\s+(?:[a-z]\s*=\s*)?(.+))?$").captures(c) {
        (m[1].to_string(), "differentiable".into(), m.get(2).map(|x| x.as_str().to_string()))
    } else if let Some(m) = re!(r"(?i)^(.+?)\s*:\s*is\s+it\s+(continuous|differentiable)$").captures(c) {
        (m[1].to_string(), m[2].to_lowercase(), None)
    } else if let Some(m) = re!(r"(?i)^(?:find\s+)?(?:the\s+)?(?:discontinuities|points of discontinuity|discontinuity points)\s+(?:(?:of|for|in)\s+)?(.+)$").captures(c) {
        (m[1].to_string(), "discontinuities".into(), None)
    } else if let Some(m) = re!(r"(?i)^where\s+is\s+(.+?)\s+(discontinuous|not continuous|not differentiable)$").captures(c) {
        (m[1].to_string(), if m[2].contains("differ") { "differentiable".into() } else { "discontinuities".into() }, None)
    } else {
        return None;
    };
    let _ = l;
    let e = ex(&body)?;
    // vectors: every component
    let comps: Vec<Expr> = match &e {
        List(v) => v.clone(),
        _ => vec![e.clone()],
    };
    let fs = e.free_symbols();
    if fs.len() > 1 && at.is_none() {
        // several variables: decide by the building blocks alone
        let mut bad = Vec::new();
        for comp in &comps {
            for v in &fs {
                let mut c1 = Vec::new();
                candidates(comp, &sym(v), &mut c1);
                if prop == "differentiable" {
                    kinks(comp, &sym(v), &mut c1);
                }
                if !c1.is_empty() {
                    bad.push(v.to_string());
                }
            }
        }
        let yes = bad.is_empty();
        let word = if prop == "discontinuities" { "continuous" } else { prop.as_str() };
        let res = if yes { format!("yes: {} is {word} everywhere (built from polynomials and functions {word} everywhere)", show(&e)) } else { "not everywhere: it has singular points".to_string() };
        return Some(ans(&format!("{} {word}", show(&e)), format!("{} | {word}?", show(&e)), "Result", text(res), vec![]));
    }
    let x = main_var(&e);
    let mut cands = Vec::new();
    for comp in &comps {
        candidates(comp, &x, &mut cands);
        if prop == "differentiable" {
            kinks(comp, &x, &mut cands);
        }
    }
    let fns: Vec<F> = comps.iter().map(|c| F::new(c, &x)).collect::<Option<_>>()?;
    let input = format!("{} | {}", show(&e), if prop == "discontinuities" { "discontinuities".to_string() } else { format!("{prop}{}", at.as_ref().map(|a| format!(" at {} = {a}", show(&x))).unwrap_or_default()) });
    // a single point
    if let Some(a) = &at {
        let ae = bound(a)?;
        let av = num(&ae)?;
        let kinds: Vec<Kind> = fns.iter().map(|f| classify(f, av)).collect();
        let cont = kinds.iter().all(|k| matches!(k, Kind::Continuous));
        let result = if prop == "continuous" {
            if cont {
                format!("yes: {} is continuous at {} = {}", show(&e), show(&x), show(&ae))
            } else {
                let k = kinds.iter().find(|k| !matches!(k, Kind::Continuous)).unwrap();
                format!("no: {} at {} = {}", kind_text(k), show(&x), show(&ae))
            }
        } else if !cont {
            format!("no: not even continuous at {} = {}", show(&x), show(&ae))
        } else if fns.iter().all(|f| differentiable_at(f, av)) {
            let d = s::simplify(&calculus::diff(&e, &x).subs(&x, &ae));
            format!("yes: the derivative there is {}", show(&d))
        } else {
            format!("no: the one-sided derivatives differ at {} = {}", show(&x), show(&ae))
        };
        return Some(ans(&format!("{prop} at a point"), input, "Result", text(result), vec![]));
    }
    // everywhere
    let pts = dedup_points(cands);
    let mut rows = Vec::new();
    let mut bad = Vec::new();
    for (pe, per, pv) in &pts {
        let kinds: Vec<Kind> = fns.iter().map(|f| classify(f, *pv)).collect();
        let k = kinds.iter().find(|k| !matches!(k, Kind::Continuous)).cloned().unwrap_or(Kind::Continuous);
        let label = match (&k, prop.as_str()) {
            (Kind::Continuous, "differentiable") => {
                if fns.iter().all(|f| differentiable_at(f, *pv)) {
                    continue;
                }
                "continuous, but not differentiable (corner or vertical tangent)"
            }
            (Kind::Continuous, _) => continue,
            (k, _) => kind_text(k),
        };
        bad.push(point_text(pe, *per, &x));
        rows.push(vec![text(point_text(pe, *per, &x)), text(label.to_string())]);
    }
    let word = if prop == "discontinuities" { "continuous" } else { prop.as_str() };
    let result = if bad.is_empty() {
        if prop == "discontinuities" {
            "none: the function is continuous on its domain".to_string()
        } else {
            format!("yes: {} is {word} on its domain", show(&e))
        }
    } else if prop == "discontinuities" {
        bad.join(", ")
    } else {
        format!("no: not {word} at {}", bad.join(", "))
    };
    let mut extra = Vec::new();
    if !rows.is_empty() {
        extra.push(pod("Points", "Points", table(Some(vec!["point", "behaviour"]), rows)));
    }
    Some(ans(&format!("{} {word}", show(&e)), input, if prop == "discontinuities" { "Discontinuities" } else { "Result" }, text(result), extra))
}

// ---------------------------------------------------------------------------
// increasing / concave at a point or on an interval
// ---------------------------------------------------------------------------

fn interval(s: &str) -> Option<(Expr, Expr)> {
    let s = s.trim();
    if let Some(m) = re!(r"^(.+?)\s*<=?\s*[a-z]\s*<=?\s*(.+)$").captures(s) {
        return Some((bound(&m[1])?, bound(&m[2])?));
    }
    if let Some(m) = re!(r"^[\[(]\s*(.+?)\s*,\s*(.+?)\s*[\])]$").captures(s) {
        return Some((bound(&m[1])?, bound(&m[2])?));
    }
    if let Some(m) = re!(r"^(?:[a-z]\s*=\s*)?(.+?)\s+(?:and|to|\.\.)\s+(.+)$").captures(s) {
        return Some((bound(&m[1])?, bound(&m[2])?));
    }
    None
}

fn behaviour_at(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches('?').trim();
    let m = re!(r"(?i)^is\s+(?:the\s+)?(?:function\s+)?(.+?)\s+(increasing|decreasing|concave up|concave down|convex|concave)\s+at\s+(?:[a-z]\s*=\s*)?(.+)$").captures(c)?;
    let e = ex(&m[1])?;
    let x = main_var(&e);
    let a = bound(&m[3])?;
    let av = num(&a)?;
    let what = m[2].to_lowercase();
    let order = if what.contains("creasing") { 1 } else { 2 };
    let d = s::simplify(&calculus::diff(&if order == 2 { calculus::diff(&e, &x) } else { e.clone() }, &x));
    let f = F::new(&d, &x)?;
    let v = f.at(av);
    let (l, r) = (f.at(av - 1e-3), f.at(av + 1e-3));
    let sign = if v.abs() > 1e-12 { v.signum() } else if l >= 0.0 && r >= 0.0 && (l > 0.0 || r > 0.0) { 1.0 } else if l <= 0.0 && r <= 0.0 && (l < 0.0 || r < 0.0) { -1.0 } else { 0.0 };
    let want = if what == "increasing" || what == "concave up" || what == "convex" { 1.0 } else { -1.0 };
    let label = match (order, sign as i32) {
        (1, 1) => "increasing",
        (1, -1) => "decreasing",
        (2, 1) => "concave up",
        (2, -1) => "concave down",
        _ => "neither (stationary there)",
    };
    let yes = sign == want;
    let dv = s::simplify(&d.subs(&x, &a));
    let name = if order == 1 { "f′" } else { "f″" };
    let why = if v.abs() > 1e-12 { format!("{name}({}) = {}", show(&a), show(&dv)) } else { format!("{name}({}) = 0, and {name} keeps one sign on both sides", show(&a)) };
    Some(ans(
        &format!("{} {what} at {}", show(&e), show(&a)),
        format!("{} | {what} at {} = {}", show(&e), show(&x), show(&a)),
        "Result",
        text(format!("{}: {} is {label} at {} = {} ({why})", if yes { "yes" } else { "no" }, show(&e), show(&x), show(&a))),
        vec![pod("Derivative", if order == 1 { "Derivative" } else { "Second derivative" }, math(&d))],
    ))
}

fn on_interval(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches('?').trim();
    let (what, body, iv) = if let Some(m) = re!(r"(?i)^(?:the\s+)?(concavity|monotonicity|inflection points?|intervals of increase(?: and decrease)?)\s+(?:of\s+)?(?:the\s+)?(?:curve\s+|function\s+)?(.+?)\s+(?:on|between|for|over|with|in|where)\s+(.+)$").captures(c) {
        (m[1].to_lowercase(), m[2].to_string(), m[3].to_string())
    } else if let Some(m) = re!(r"(?i)^(.+?)\s+(concavity|monotonicity)\s+(?:on|between|for|over)\s+(.+)$").captures(c) {
        (m[2].to_lowercase(), m[1].to_string(), m[3].to_string())
    } else {
        return None;
    };
    let e = ex(&body)?;
    let x = main_var(&e);
    let (a, b) = interval(&iv)?;
    let (av, bv) = (num(&a)?, num(&b)?);
    if !(av < bv) {
        return None;
    }
    let second = what.starts_with("concav") || what.starts_with("inflection");
    let d1 = s::simplify(&calculus::diff(&e, &x));
    let d = if second { s::simplify(&calculus::diff(&d1, &x)) } else { d1 };
    let f = F::new(&d, &x)?;
    let fe = F::new(&e, &x)?;
    let roots: Vec<f64> = crate::numeric::find_roots(&|t| f.at(t), av, bv, 4000).into_iter().filter(|r| *r > av + 1e-9 && *r < bv - 1e-9).collect();
    let mut cuts = vec![av];
    cuts.extend(roots.iter().copied());
    cuts.push(bv);
    let mut rows = Vec::new();
    let mut changes = Vec::new();
    let mut prev: Option<f64> = None;
    for w in cuts.windows(2) {
        let mid = (w[0] + w[1]) / 2.0;
        let sg = f.at(mid).signum();
        let label = match (second, sg as i32) {
            (true, 1) => "concave up",
            (true, -1) => "concave down",
            (false, 1) => "increasing",
            (false, -1) => "decreasing",
            _ => "constant",
        };
        if let Some(p) = prev {
            if p != sg {
                changes.push(w[0]);
            }
        }
        prev = Some(sg);
        if let Some(last) = rows.last_mut() {
            let last: &mut (f64, f64, &str) = last;
            if last.2 == label {
                last.1 = w[1];
                continue;
            }
        }
        rows.push((w[0], w[1], label));
    }
    let fmt = |v: f64| crate::print::fmt_float(v, 6);
    let table_rows: Vec<Vec<Content>> = rows.iter().map(|(p, r, l)| vec![text(format!("{} < {} < {}", fmt(*p), show(&x), fmt(*r))), text(l.to_string())]).collect();
    let main = if what.starts_with("inflection") {
        if changes.is_empty() {
            "no inflection points on this interval".to_string()
        } else {
            changes.iter().map(|p| format!("({}, {})", fmt(*p), fmt(fe.at(*p)))).collect::<Vec<_>>().join(", ")
        }
    } else if rows.len() == 1 {
        format!("{} on the whole interval", rows[0].2)
    } else {
        rows.iter().map(|(p, r, l)| format!("{l} on ({}, {})", fmt(*p), fmt(*r))).collect::<Vec<_>>().join("; ")
    };
    Some(ans(
        &format!("{what} of {}", show(&e)),
        format!("{} | {what} | {} ≤ {} ≤ {}", show(&e), show(&a), show(&x), show(&b)),
        if what.starts_with("inflection") { "Inflection points" } else { "Result" },
        text(main),
        vec![pod("Intervals", "Intervals", table(None, table_rows)), pod("Derivative", if second { "Second derivative" } else { "Derivative" }, math(&d))],
    ))
}

// ---------------------------------------------------------------------------
// directional derivatives and Jacobians
// ---------------------------------------------------------------------------

/// Strip one pair of outer brackets.
fn unwrap_brackets(s: &str) -> &str {
    let t = s.trim();
    let (Some(f), Some(l)) = (t.chars().next(), t.chars().last()) else { return t };
    if matches!((f, l), ('(', ')') | ('<', '>') | ('{', '}') | ('[', ']')) && t.len() >= 2 {
        &t[1..t.len() - 1]
    } else {
        t
    }
}

fn vector(s: &str) -> Option<Vec<Expr>> {
    let t = unwrap_brackets(s);
    let parts = super::nl::split_top(t);
    parts.iter().map(|p| ex(p)).collect()
}

fn ordered_vars(e: &Expr, text_src: &str) -> Vec<Expr> {
    let fs = e.free_symbols();
    let mut v: Vec<(usize, String)> = fs.iter().map(|n| (re!(r"[a-z]+").find_iter(text_src).position(|m| m.as_str() == &**n).unwrap_or(usize::MAX), n.to_string())).collect();
    let standard = ["x", "y", "z", "w"];
    if v.iter().all(|(_, n)| standard.contains(&n.as_str())) {
        v.sort_by_key(|(_, n)| standard.iter().position(|s| s == n));
    } else {
        v.sort();
    }
    v.into_iter().map(|(_, n)| sym(&n)).collect()
}

fn directional(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches('?').trim();
    let m = re!(r"(?i)^(?:the\s+)?(?:directional\s+)?derivative\s+of\s+(.+?)\s+in\s+(?:the\s+)?direction\s+(?:of\s+)?([(<{\[].+?[)>}\]])(?:\s+at\s+(?:the\s+point\s+)?([(<{\[].+?[)>}\]]))?$").captures(c)?;
    let f = ex(&m[1])?;
    let dir = vector(&m[2])?;
    let vars = ordered_vars(&f, &m[1]);
    if vars.len() != dir.len() || vars.is_empty() {
        return None;
    }
    let grad: Vec<Expr> = vars.iter().map(|v| s::simplify(&calculus::diff(&f, v))).collect();
    let norm = s::simplify(&func("sqrt", vec![add(dir.iter().map(|d| square(d.clone())).collect())]));
    let dd = s::simplify(&div(add(grad.iter().zip(dir.iter()).map(|(g, d)| mul(vec![g.clone(), d.clone()])).collect()), norm.clone()));
    let mut extra = vec![pod("Gradient", "Gradient", math(&List(grad.clone())))];
    let result = if let Some(p) = m.get(3) {
        let pt = vector(p.as_str())?;
        let mut v = dd.clone();
        for (x, a) in vars.iter().zip(pt.iter()) {
            v = v.subs(x, a);
        }
        extra.push(pod("General", "Directional derivative (general point)", math(&dd)));
        s::simplify(&v)
    } else {
        dd
    };
    extra.push(pod("Unit", "Unit direction", math(&List(dir.iter().map(|d| s::simplify(&div(d.clone(), norm.clone()))).collect()))));
    Some(ans("directional derivative", format!("directional derivative | {} | direction {}", show(&f), show(&List(dir))), "Result", math(&result), extra))
}

fn jacobian(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches('?').trim();
    let m = re!(r"(?i)^(?:the\s+)?jacobian(?:\s+(matrix|determinant))?\s+(?:of\s+)?(.+?)(?:\s+(?:with respect to|w\.?\s?r\.?\s?t\.?)\s+(.+))?$").captures(c).map(|m| (m.get(1).map(|x| x.as_str().to_string()), m[2].to_string(), m.get(3).map(|x| x.as_str().to_string()))).or_else(|| {
        let m = re!(r"(?i)^([(<{\[].+[)>}\]])\s*,?\s*(?:(?:what'?s|what is|find)\s+)?(?:the\s+)?jacobian(?:\s+(matrix|determinant))?$").captures(c)?;
        Some((m.get(2).map(|x| x.as_str().to_string()), m[1].to_string(), None))
    })?;
    let (kind, body, wrt) = m;
    let body = re!(r"(?i)^(?:the\s+)?(?:vector\s+)?(?:field\s+|function\s+|map\s+|transformation\s+)?(?:[fF]\s*=\s*)?").replace(&body, "").to_string();
    let comps = vector(&body)?;
    if comps.len() < 2 {
        return None;
    }
    let all = List(comps.clone());
    let vars: Vec<Expr> = match &wrt {
        Some(w) => w.trim_matches(|c: char| "()[]{}<> ".contains(c)).split([',', ' ']).map(str::trim).filter(|s| !s.is_empty() && *s != "and").map(sym).collect(),
        None => ordered_vars(&all, &body),
    };
    if vars.is_empty() {
        return None;
    }
    let rows: Vec<Expr> = comps.iter().map(|f| List(vars.iter().map(|v| s::simplify(&calculus::diff(f, v))).collect())).collect();
    let mat = List(rows);
    let mut extra = Vec::new();
    let det = if comps.len() == vars.len() {
        let mm = crate::linalg::from_expr(&mat)?;
        Some(s::simplify(&crate::linalg::det(&mm)))
    } else {
        None
    };
    let want_det = kind.as_deref().is_some_and(|k| k.eq_ignore_ascii_case("determinant"));
    if let Some(d) = &det {
        if !want_det {
            extra.push(pod("Determinant", "Jacobian determinant", math(d)));
        }
    }
    let (title, content) = if want_det { ("Jacobian determinant", math(det.as_ref()?)) } else { ("Jacobian matrix", math(&mat)) };
    if want_det {
        extra.push(pod("Matrix", "Jacobian matrix", math(&mat)));
    }
    Some(ans("Jacobian", format!("Jacobian | {} | with respect to {}", show(&all), vars.iter().map(show).collect::<Vec<_>>().join(", ")), title, content, extra))
}

// ---------------------------------------------------------------------------
// integrals: iterated, revolution, arc length
// ---------------------------------------------------------------------------

fn definite_value(e: &Expr, x: &Expr, a: &Expr, b: &Expr) -> Option<Expr> {
    match calculus::integrate::definite(e, x, a, b) {
        Definite::Exact(v) => Some(s::simplify(&v)),
        Definite::Numeric(v, _) => Some(float(v)),
        _ => None,
    }
}

fn result_pods(v: &Expr) -> (Content, Vec<Pod>) {
    let mut extra = Vec::new();
    if let Some(f) = num(v) {
        if !matches!(v, Float(_)) {
            extra.push(pod("Decimal", "Decimal approximation", text(crate::print::fmt_float(f, 10))));
        }
    }
    (math(v), extra)
}

fn iterated(q: &Query) -> Option<Answer> {
    let c = normalise(q.clean.trim().trim_end_matches('?'));
    let m = re!(r"(?i)^(?:int|integrate|integral(?:\s+of)?|∫)\s+(.+?)\s+((?:d[a-z]\s*)+)\s*,\s*(.+)$").captures(&c)?;
    let f = ex(&m[1])?;
    let ds: Vec<String> = re!(r"d([a-z])").captures_iter(&m[2]).map(|x| x[1].to_string()).collect();
    let mut bounds: Vec<(String, Expr, Expr)> = Vec::new();
    for b in re!(r"([a-z])\s*=\s*(.+?)\s*(?:\.\.|\bto\b)\s*([^,]+)").captures_iter(&m[3]) {
        bounds.push((b[1].to_string(), bound(&b[2])?, bound(&b[3].trim())?));
    }
    if bounds.len() != ds.len() || ds.is_empty() {
        return None;
    }
    let mut cur = f.clone();
    let mut exact = true;
    for d in &ds {
        let (_, a, b) = bounds.iter().find(|(v, ..)| v == d)?;
        match calculus::integrate::definite(&cur, &sym(d), a, b) {
            Definite::Exact(v) => cur = s::simplify(&v),
            _ => {
                exact = false;
                break;
            }
        }
    }
    let value = if exact {
        cur
    } else {
        // nested numeric quadrature (finite numeric bounds)
        let names: Vec<&str> = ds.iter().map(String::as_str).collect();
        let p = Program::compile(&f, &names)?;
        let lims: Vec<(f64, f64)> = ds.iter().map(|d| bounds.iter().find(|(v, ..)| v == d).and_then(|(_, a, b)| Some((num(a)?, num(b)?)))).collect::<Option<_>>()?;
        fn nest(p: &Program, lims: &[(f64, f64)], k: usize, vals: &mut Vec<f64>) -> f64 {
            let (a, b) = lims[k];
            let f = |t: f64| {
                let mut v = vals.clone();
                v[k] = t;
                if k + 1 == lims.len() {
                    p.eval(&v)
                } else {
                    nest(p, lims, k + 1, &mut v)
                }
            };
            crate::numeric::quad(&f, a, b, 1e-8).value
        }
        // innermost variable first in `ds`: evaluate outermost last
        let mut order: Vec<usize> = (0..ds.len()).rev().collect();
        let lims_o: Vec<(f64, f64)> = order.iter().map(|&i| lims[i]).collect();
        let po = Program::compile(&f, &order.iter().map(|&i| names[i]).collect::<Vec<_>>())?;
        let _ = p;
        order.clear();
        let mut vals = vec![0.0; ds.len()];
        float(nest(&po, &lims_o, 0, &mut vals))
    };
    let (content, extra) = result_pods(&value);
    let setup = format!("∫ {} {}", show(&f), ds.iter().map(|d| {
        let (_, a, b) = bounds.iter().find(|(v, ..)| v == d).unwrap();
        format!("d{d} ({d} = {} to {})", show(a), show(b))
    }).collect::<Vec<_>>().join(" "));
    Some(ans("iterated integral", setup, "Result", content, extra))
}

fn revolution(q: &Query) -> Option<Answer> {
    let c = normalise(q.clean.trim().trim_end_matches('?'));
    let l = c.to_lowercase();
    if !(l.starts_with("revolve") || l.contains("of revolution") || l.contains("solid of revolution")) {
        return None;
    }
    let axis = re!(r"(?:around|about)\s+(?:the\s+)?([xy])[- ]?axis").captures(&l)?.get(1)?.as_str().to_string();
    let rest = re!(r"(?i),?\s*(?:around|about)\s+(?:the\s+)?[xy][- ]?axis.*$").replace(&c, "").to_string();
    let rest = re!(r"(?i)^(?:find\s+)?(?:the\s+)?(?:volume\s+of\s+(?:the\s+)?(?:solid\s+of\s+)?revolution\s+(?:of|for|generated by)?\s*|revolve\s+)").replace(&rest, "").to_string();
    // the interval
    let (a, b, rest) = if let Some(m) = re!(r"(?i),?\s*(?:from\s+)?x\s*=\s*(\S+)\s+(?:to|\.\.)\s+(\S+)\s*,?").captures(&rest) {
        (bound(&m[1])?, bound(&m[2])?, rest.replace(m.get(0)?.as_str(), " "))
    } else if let Some(m) = re!(r",?\s*(\S+)\s*<=?\s*x\s*<=?\s*(\S+)\s*,?").captures(&rest) {
        (bound(&m[1])?, bound(&m[2])?, rest.replace(m.get(0)?.as_str(), " "))
    } else if let Some(m) = re!(r"(?i),?\s*from\s+(\S+)\s+to\s+(\S+)\s*,?").captures(&rest) {
        (bound(&m[1])?, bound(&m[2])?, rest.replace(m.get(0)?.as_str(), " "))
    } else {
        return None;
    };
    let rest = rest.trim().trim_end_matches(',').trim();
    let rest = re!(r"(?i)^(?:the\s+)?(?:region\s+)?(?:between|bounded by)\s+").replace(rest, "").to_string();
    let parts: Vec<&str> = re!(r"(?i)\s+and\s+").split(&rest).collect();
    let fs: Vec<Expr> = parts.iter().map(|p| ex(p)).collect::<Option<_>>()?;
    let x = sym("x");
    let (f, g) = match fs.len() {
        1 => (fs[0].clone(), zero()),
        2 => (fs[0].clone(), fs[1].clone()),
        _ => return None,
    };
    let integrand = if axis == "x" { mul(vec![pi(), sub(square(f.clone()), square(g.clone()))]) } else { mul(vec![int(2), pi(), x.clone(), sub(f.clone(), g.clone())]) };
    let integrand = s::simplify(&integrand);
    let mut v = definite_value(&integrand, &x, &a, &b)?;
    if num(&v).is_some_and(|t| t < 0.0) {
        v = s::simplify(&neg(v));
    }
    let (content, mut extra) = result_pods(&v);
    extra.insert(0, pod("Setup", if axis == "x" { "Washer method" } else { "Shell method" }, text(format!("V = ∫ from {} to {} of {} dx", show(&a), show(&b), show(&integrand)))));
    Some(ans("volume of revolution", format!("solid of revolution | {} | {} ≤ x ≤ {} | about the {axis}-axis", if fs.len() == 1 { show(&f) } else { format!("between {} and {}", show(&f), show(&g)) }, show(&a), show(&b)), "Volume", content, extra))
}

fn arc_length(q: &Query) -> Option<Answer> {
    let c = normalise(q.clean.trim().trim_end_matches('?'));
    let m = re!(r"(?i)^(?:the\s+)?(?:arc\s*)?length\s+of\s+(?:the\s+)?(?:curve\s+|parametric curve\s+|helix\s+)?([({\[].+[)}\]])\s*,?\s*(?:for\s+|from\s+)?(?:([a-z])\s*=\s*)?(.+?)\s*(?:to|\.\.)\s*(.+)$").captures(&c)?;
    let comps: Vec<Expr> = {
        let t = unwrap_brackets(&m[1]);
        super::nl::split_top(t).iter().map(|p| ex(re!(r"^\s*[a-z]\s*=\s*").replace(p, "").as_ref())).collect::<Option<_>>()?
    };
    if comps.len() < 2 {
        return None;
    }
    let t = m.get(2).map(|v| sym(v.as_str())).unwrap_or_else(|| main_var(&List(comps.clone())));
    let (a, b) = (bound(&m[3])?, bound(&m[4])?);
    let speed2 = s::simplify(&add(comps.iter().map(|f| square(calculus::diff(f, &t))).collect()));
    let integrand = s::simplify(&func("sqrt", vec![speed2]));
    let v = definite_value(&integrand, &t, &a, &b)?;
    let (content, mut extra) = result_pods(&v);
    extra.insert(0, pod("Setup", "Integral", text(format!("∫ from {} to {} of {} d{}", show(&a), show(&b), show(&integrand), show(&t)))));
    Some(ans("arc length", format!("arc length | {} | {} = {} to {}", show(&List(comps)), show(&t), show(&a), show(&b)), "Arc length", content, extra))
}

// ---------------------------------------------------------------------------
// limits: discrete, superior and inferior
// ---------------------------------------------------------------------------

fn limits(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches('?').trim();
    // "discrete limit of (3 n)/(1 + 6 n) as n -> infinity", "discrete lim n/(n+1)"
    if let Some(m) = re!(r"(?i)^discrete\s+(lim\s*inf|lim\s*sup|liminf|limsup|lim(?:it)?)\s+(?:of\s+)?(.+?)(?:\s+as\s+([a-z])\s*->\s*(.+))?$").captures(c) {
        let kind = m[1].to_lowercase().replace(' ', "");
        let body = m[2].to_string();
        let var = m.get(3).map_or("n", |v| v.as_str());
        let to = m.get(4).map_or("infinity", |v| v.as_str());
        if kind.starts_with("liminf") || kind.starts_with("limsup") {
            return sup_inf(&kind, &body, var, to, true);
        }
        let r = crate::engine::query_with(&format!("limit {body} as {var} -> {to}"), q.opts);
        if r.success {
            return Some(Answer { interpretation: r.interpretation.unwrap_or_default(), pods: r.pods, assumptions: vec![] });
        }
        return None;
    }
    let m = re!(r"(?i)^(lim\s*sup|limsup|lim\s*inf|liminf|superior limit|inferior limit|upper limit|lower limit|max(?:imum)?\s+lim(?:it)?|min(?:imum)?\s+lim(?:it)?)\s+(?:of\s+)?(.+?)(?:(?:\s*,\s*|\s+as\s+)([a-z])\s*->\s*(.+))?$").captures(c)?;
    let k = m[1].to_lowercase();
    let kind = if k.contains("sup") || k.starts_with("upper") || k.starts_with("max") { "limsup" } else { "liminf" };
    let body = m[2].to_string();
    let e = ex(&body)?;
    let var = m.get(3).map(|v| v.as_str().to_string()).unwrap_or_else(|| show(&main_var(&e)));
    let to = m.get(4).map_or("infinity", |v| v.as_str()).to_string();
    let discrete = var == "n" || var == "k" || var == "m";
    sup_inf(kind, &body, &var, &to, discrete)
}

fn sup_inf(kind: &str, body: &str, var: &str, to: &str, discrete: bool) -> Option<Answer> {
    let e = ex(body)?;
    let x = sym(var);
    let a = bound(to)?;
    let sup = kind.contains("sup");
    let f = F::new(&e, &x)?;
    // an ordinary limit settles both
    let lim = calculus::limit::limit(&e, &x, &a, Dir::Both).filter(|l| !l.has(&|n| matches!(n, Const(K::Undef | K::CInf))) && num(l).is_some());
    let value: Expr = if let Some(l) = lim {
        l
    } else {
        let samples: Vec<f64> = if a.is_inf() || matches!(&a, Mul(_)) && num(&a).is_none() {
            let positive = !matches!(&a, Mul(_));
            if discrete {
                (1_000_000..1_200_000).step_by(3).map(|n| f.at(if positive { n as f64 } else { -(n as f64) })).collect()
            } else {
                (0..200_000).map(|i| {
                    let t = 1e3 + i as f64 * 0.0137;
                    f.at(if positive { t } else { -t })
                }).collect()
            }
        } else {
            let av = num(&a)?;
            (1..200_000).map(|i| {
                let h = 1e-4 * (1.0 - i as f64 / 200_001.0) + 1e-9;
                f.at(av + if i % 2 == 0 { h } else { -h })
            }).collect()
        };
        let good: Vec<f64> = samples.into_iter().filter(|v| v.is_finite()).collect();
        if good.is_empty() {
            return None;
        }
        let v = if sup { good.iter().cloned().fold(f64::NEG_INFINITY, f64::max) } else { good.iter().cloned().fold(f64::INFINITY, f64::min) };
        if v.abs() > 1e8 {
            if v > 0.0 { inf() } else { neg(inf()) }
        } else {
            let r = (v * 1e4).round() / 1e4;
            super::math::identify_constant(r).filter(|c| num(c).is_some_and(|w| (w - v).abs() < 1e-3)).unwrap_or_else(|| if (r - r.round()).abs() < 1e-3 { int(r.round() as i64) } else { float(r) })
        }
    };
    let name = if sup { "lim sup" } else { "lim inf" };
    Some(ans(
        name,
        format!("{name} {} as {var} → {}{}", show(&e), show(&a), if discrete { " (over integers)" } else { "" }),
        if sup { "Upper limit" } else { "Lower limit" },
        math(&value),
        vec![],
    ))
}

// ---------------------------------------------------------------------------
// coordinate geometry basics
// ---------------------------------------------------------------------------

/// An angle as (exact value, is in degrees).
fn angle(s: &str) -> Option<(Expr, bool)> {
    let t = s.trim().to_lowercase();
    if let Some(m) = re!(r"^(.+?)\s*(?:°|deg|degs|degrees?)$").captures(&t) {
        return Some((ex(&m[1])?, true));
    }
    if let Some(m) = re!(r"^(.+?)\s*(?:rad|rads|radians?)$").captures(&t) {
        return Some((ex(&m[1])?, false));
    }
    let e = ex(&t)?;
    let has_pi = e.has(&|n| matches!(n, Const(K::Pi)));
    Some((e, !has_pi))
}

fn deg_text(e: &Expr, deg: bool) -> String {
    let s = show(&s::simplify(e));
    if deg {
        format!("{s}°")
    } else {
        format!("{s} rad")
    }
}

fn point2(s: &str) -> Option<(Expr, Expr)> {
    let v = vector(s)?;
    (v.len() == 2).then(|| (v[0].clone(), v[1].clone()))
}

fn geometry(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches('?').trim();
    let l = c.to_lowercase();
    // complementary / supplementary / reference angles
    if let Some(m) = re!(r"^(?:the\s+)?(complement(?:ary)?|supplement(?:ary)?|reference)(?:\s+angle)?\s+(?:of|for|to)\s+(?:the\s+angle\s+)?(?:an?\s+)?(.+?)(?:\s+angle)?$").captures(&l) {
        let (a, deg) = angle(&m[2])?;
        let full = |k: i64| if deg { int(90 * k) } else { mul(vec![Num(Rat::new(k.into(), 2.into())), pi()]) };
        let kind = &m[1];
        let (name, v) = if kind.starts_with("complement") {
            ("complementary angle", s::simplify(&sub(full(1), a.clone())))
        } else if kind.starts_with("supplement") {
            ("supplementary angle", s::simplify(&sub(full(2), a.clone())))
        } else {
            let av = num(&a)?;
            let turn = if deg { 360.0 } else { 2.0 * std::f64::consts::PI };
            let k = (av / turn).floor();
            let r = s::simplify(&sub(a.clone(), mul(vec![int(k as i64), full(4)])));
            let rv = num(&r)?;
            let q4 = turn / 4.0;
            let v = if rv <= q4 { r } else if rv <= 2.0 * q4 { sub(full(2), r) } else if rv <= 3.0 * q4 { sub(r, full(2)) } else { sub(full(4), r) };
            ("reference angle", s::simplify(&v))
        };
        return Some(ans(name, format!("{name} | {}", deg_text(&a, deg)), "Result", text(deg_text(&v, deg)), vec![]));
    }
    // quadrants
    let quad_of = |x: f64, y: f64| -> &'static str {
        match (x.partial_cmp(&0.0), y.partial_cmp(&0.0)) {
            (Some(std::cmp::Ordering::Greater), Some(std::cmp::Ordering::Greater)) => "quadrant I",
            (Some(std::cmp::Ordering::Less), Some(std::cmp::Ordering::Greater)) => "quadrant II",
            (Some(std::cmp::Ordering::Less), Some(std::cmp::Ordering::Less)) => "quadrant III",
            (Some(std::cmp::Ordering::Greater), Some(std::cmp::Ordering::Less)) => "quadrant IV",
            _ => "on an axis (no quadrant)",
        }
    };
    if let Some(m) = re!(r"^(?:what|which)\s+quadrant\s+(?:is|does)\s+(?:the\s+point\s+)?(\(.+?\))(?:\s+(?:in|lie in|lie))?$").captures(&l).or_else(|| re!(r"^quadrant\s+(?:of\s+)?(?:the\s+point\s+)?(\(.+?\))$").captures(&l)) {
        let (x, y) = point2(&m[1])?;
        let qd = quad_of(num(&x)?, num(&y)?);
        return Some(ans("quadrant", format!("quadrant of ({}, {})", show(&x), show(&y)), "Result", text(qd), vec![]));
    }
    if let Some(m) = re!(r"^is\s+(?:the\s+point\s+)?(\(.+?\))\s+in\s+(?:the\s+)?(?:quadrant\s+)?(1|2|3|4|i|ii|iii|iv|first|second|third|fourth|1st|2nd|3rd|4th)(?:\s+quadrant)?$").captures(&l) {
        let (x, y) = point2(&m[1])?;
        let qd = quad_of(num(&x)?, num(&y)?);
        let want = match &m[2] {
            "1" | "i" | "first" | "1st" => "quadrant I",
            "2" | "ii" | "second" | "2nd" => "quadrant II",
            "3" | "iii" | "third" | "3rd" => "quadrant III",
            _ => "quadrant IV",
        };
        let yes = qd == want;
        return Some(ans("quadrant", format!("is ({}, {}) in {want}?", show(&x), show(&y)), "Result", text(format!("{}: the point is in {qd}", if yes { "yes" } else { "no" })), vec![]));
    }
    // plane through three points
    if let Some(m) = re!(r"^(?:the\s+)?(?:equation of (?:the\s+)?)?plane\s+(?:through|containing|passing through)\s+(?:the\s+points\s+)?(\(.+?\)),?\s+(\(.+?\)),?\s+(?:and\s+)?(\(.+?\))$").captures(&l) {
        let p: Vec<Vec<Expr>> = (1..=3).map(|i| vector(&m[i])).collect::<Option<_>>()?;
        if p.iter().any(|v| v.len() != 3) {
            return None;
        }
        let u: Vec<Expr> = (0..3).map(|i| sub(p[1][i].clone(), p[0][i].clone())).collect();
        let w: Vec<Expr> = (0..3).map(|i| sub(p[2][i].clone(), p[0][i].clone())).collect();
        let cr = |a: usize, b: usize| s::simplify(&sub(mul(vec![u[a].clone(), w[b].clone()]), mul(vec![u[b].clone(), w[a].clone()])));
        let mut n = [cr(1, 2), cr(2, 0), cr(0, 1)];
        if n.iter().all(|x| x.as_int().is_some()) {
            use num_integer::Integer;
            let mut g = num_bigint::BigInt::from(0);
            for x in &n {
                g = g.gcd(x.as_int().unwrap());
            }
            // make the first non-zero coefficient positive
            if let Some(f) = n.iter().find(|x| !x.is_zero()) {
                if f.as_int().unwrap() < &num_bigint::BigInt::from(0) {
                    g = -g;
                }
            }
            if g != num_bigint::BigInt::from(0) {
                for x in n.iter_mut() {
                    *x = Expr::Num(crate::expr::Rat::from_integer(x.as_int().unwrap() / &g));
                }
            }
        }
        if n.iter().all(|x| x.is_zero()) {
            return Some(ans("plane", "plane through three points".into(), "Result", text("the points are collinear: infinitely many planes contain them"), vec![]));
        }
        let xs = [sym("x"), sym("y"), sym("z")];
        let lhs = s::simplify(&add((0..3).map(|i| mul(vec![n[i].clone(), xs[i].clone()])).collect()));
        let d = s::simplify(&add((0..3).map(|i| mul(vec![n[i].clone(), p[0][i].clone()])).collect()));
        return Some(ans("plane", format!("plane through {}, {}, {}", show(&List(p[0].clone())), show(&List(p[1].clone())), show(&List(p[2].clone()))), "Plane", math(&eq(lhs, d)), vec![pod("Normal", "Normal vector", math(&List(n.to_vec())))]));
    }
    // rotate a point about the origin
    if let Some(m) = re!(r"^rotate\s+(?:the\s+point\s+)?([({].+?[)}])\s+(?:by\s+)?(.+?)(?:\s+(counter-?clockwise|anticlockwise|clockwise|ccw|cw))?(?:\s+(?:about|around)\s+(?:the\s+)?origin)?$").captures(&l) {
        let (x, y) = point2(&m[1])?;
        let (a, deg) = angle(&m[2])?;
        let mut th = if deg { mul(vec![a.clone(), pi(), Num(Rat::new(1.into(), 180.into()))]) } else { a.clone() };
        if m.get(3).is_some_and(|d| d.as_str() == "clockwise" || d.as_str() == "cw") {
            th = neg(th);
        }
        let (cth, sth) = (func("cos", vec![th.clone()]), func("sin", vec![th]));
        let nx = s::simplify(&sub(mul(vec![x.clone(), cth.clone()]), mul(vec![y.clone(), sth.clone()])));
        let ny = s::simplify(&add(vec![mul(vec![x.clone(), sth]), mul(vec![y.clone(), cth])]));
        return Some(ans("rotation", format!("rotate ({}, {}) by {} about the origin", show(&x), show(&y), deg_text(&a, deg)), "Rotated point", math(&List(vec![nx, ny])), vec![]));
    }
    // reflect a point across y = m x + b, or the reflection matrix
    if let Some(m) = re!(r"^reflect\s+(?:the\s+point\s+)?([({].+?[)}])?\s*(?:over|across|in|about|through)\s+(?:the\s+line\s+)?y\s*=\s*(.+)$").captures(&l) {
        let line = ex(&m[2])?;
        let x = sym("x");
        let slope = s::simplify(&calculus::diff(&line, &x));
        let b = s::simplify(&line.subs(&x, &zero()));
        if slope.has_sym(&x) {
            return None;
        }
        let den = add(vec![one(), square(slope.clone())]);
        let Some(pt) = m.get(1) else {
            let mat = List(vec![
                List(vec![s::simplify(&div(sub(one(), square(slope.clone())), den.clone())), s::simplify(&div(mul(vec![int(2), slope.clone()]), den.clone()))]),
                List(vec![s::simplify(&div(mul(vec![int(2), slope.clone()]), den.clone())), s::simplify(&div(sub(square(slope.clone()), one()), den.clone()))]),
            ]);
            let note = if b.is_zero() { String::new() } else { format!(" (after shifting down by {})", show(&b)) };
            return Some(ans("reflection", format!("reflection across y = {}", show(&line)), "Reflection matrix", math(&mat), vec![pod("Notes", "Notes", text(format!("Applies to points relative to the line's y-intercept{note}.")))]));
        };
        let (px, py) = point2(pt.as_str())?;
        let d = div(add(vec![px.clone(), mul(vec![sub(py.clone(), b.clone()), slope.clone()])]), den);
        let nx = s::simplify(&sub(mul(vec![int(2), d.clone()]), px.clone()));
        let ny = s::simplify(&add(vec![mul(vec![int(2), d, slope]), neg(py.clone()), mul(vec![int(2), b])]));
        return Some(ans("reflection", format!("reflect ({}, {}) across y = {}", show(&px), show(&py), show(&line)), "Reflected point", math(&List(vec![nx, ny])), vec![]));
    }
    // a line from slope and intercept
    if let Some(m) = re!(r"^(?:the\s+)?line,?\s+(?:with\s+)?slope\s*(?:=|of)?\s*(.+?),?\s+(?:and\s+)?y-?\s*intercept\s*(?:=|of)?\s*(.+)$").captures(&l) {
        let (sl, b) = (ex(&m[1])?, ex(&m[2])?);
        let x = sym("x");
        let line = s::simplify(&add(vec![mul(vec![sl.clone(), x.clone()]), b.clone()]));
        let xi = s::simplify(&div(neg(b.clone()), sl.clone()));
        return Some(ans("line", format!("line | slope {} | y-intercept {}", show(&sl), show(&b)), "Equation", math(&eq(sym("y"), line)), vec![pod("XIntercept", "x-intercept", math(&xi))]));
    }
    None
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.as_str();
    if l.contains("continu") || l.contains("differentiab") || l.contains("derivative") && l.starts_with("do") || l.contains("discontinu") {
        if let Some(a) = continuity(q) {
            return Some(a);
        }
    }
    behaviour_at(q)
        .or_else(|| geometry(q))
        .or_else(|| on_interval(q))
        .or_else(|| directional(q))
        .or_else(|| jacobian(q))
        .or_else(|| iterated(q))
        .or_else(|| revolution(q))
        .or_else(|| arc_length(q))
        .or_else(|| limits(q))
}

#[cfg(test)]
mod tests {
    fn ask(s: &str) -> String {
        crate::engine::query(s).short_answer().unwrap_or_default()
    }

    #[test]
    fn analysis() {
        assert!(ask("is tan(x) continuous at pi").starts_with("yes"));
        assert!(ask("is tan(x) continuous at pi/2").starts_with("no"));
        assert!(ask("does abs(x) have a derivative").starts_with("no"));
        assert!(ask("Is x^3 increasing at x=0?").starts_with("yes"));
        assert_eq!(ask("int e^-(x^2+y^2) dx dy, x=-oo to oo, y=-oo to oo"), "π");
        assert_eq!(ask("revolve f(x)=sqrt(4-x^2), x = -1 to 1, around the x-axis"), "(22 π)/3");
        assert_eq!(ask("limsup (-1)^n"), "1");
    }
}
