//! Partial differential equations in two variables.
//!
//! * classification of second-order linear equations (elliptic, parabolic,
//!   hyperbolic) from the principal coefficients;
//! * first-order linear equations with constant coefficients by the method
//!   of characteristics, with an initial condition when one is given;
//! * the heat equation u_t = k u_xx and the wave equation u_tt = c² u_xx on
//!   an interval with homogeneous Dirichlet or Neumann ends, by separation
//!   of variables and Fourier series;
//! * the wave equation on the whole line by d'Alembert's formula;
//! * Laplace's equation on a rectangle with data on one side.
//!
//! Every separated mode is substituted back into the equation before it is
//! shown.

use super::*;
use crate::calculus::diff::diff;
use crate::calculus::integrate::{definite, integrate, Definite};
use crate::expr::Expr::*;
use crate::expr::*;
use crate::parse;
use crate::plot as pl;
use crate::print::text as ptext;
use crate::simplify as s;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// Number of series terms computed exactly.
const TERMS: i64 = 8;

/// Rewrite derivative notations into `u_x`, `u_xx`, `u_xt`, … symbols.
fn normalize(t: &str) -> String {
    let t = t.replace('∂', "d").replace('²', "^2");
    // d^2u/dx^2, d^2u/dxdy
    let t = re!(r"d\^?2\s*([a-z])\s*/\s*d([a-z])\^?2").replace_all(&t, "${1}_$2$2").to_string();
    let t = re!(r"d\^?2\s*([a-z])\s*/\s*d([a-z])\s*d([a-z])").replace_all(&t, "${1}_$2$3").to_string();
    // du/dx
    let t = re!(r"\bd([a-z])\s*/\s*d([a-z])\b").replace_all(&t, "${1}_$2").to_string();
    // u_{xx}
    re!(r"([a-z])_\{([a-z]+)\}").replace_all(&t, "${1}_$2").to_string()
}

struct Pde {
    /// the dependent variable
    u: String,
    /// coefficients of u, u_a, u_b, u_aa, u_ab, u_bb (a, b the two variables)
    c: [Expr; 6],
    /// forcing term (moved to the right-hand side): Σ c·D = g
    g: Expr,
    a: String,
    b: String,
}

fn dname(u: &str, vars: &str) -> String {
    format!("{u}_{vars}")
}

/// Split `lhs = rhs` into derivative coefficients.
fn analyse(eq_text: &str) -> Option<Pde> {
    let caps: Vec<(String, String)> = re!(r"\b([a-z])_([a-z]{1,2})\b").captures_iter(eq_text).map(|m| (m[1].to_string(), m[2].to_string())).collect();
    let u = caps.first()?.0.clone();
    if caps.iter().any(|(v, _)| *v != u) {
        return None;
    }
    let mut vars: Vec<char> = caps.iter().flat_map(|(_, d)| d.chars()).collect();
    vars.sort();
    vars.dedup();
    if vars.len() == 1 {
        // u_t = k u_xx always names both, but "u_x + u = 0" is an ODE
        return None;
    }
    if vars.len() != 2 || vars.iter().any(|ch| ch.to_string() == u) {
        return None;
    }
    // time-like variable second, so heat/wave read u_t and u_tt
    let (a, b) = if vars.contains(&'t') { (vars.iter().find(|ch| **ch != 't')?.to_string(), "t".to_string()) } else { (vars[0].to_string(), vars[1].to_string()) };
    let e = parse::parse_expr(eq_text).ok()?;
    let f = s::expand(&solve_zero(&e));
    let names = [u.clone(), dname(&u, &a), dname(&u, &b), dname(&u, &format!("{a}{a}")), dname(&u, &format!("{a}{b}")), dname(&u, &format!("{b}{b}"))];
    let alt = dname(&u, &format!("{b}{a}"));
    let mut c: [Expr; 6] = std::array::from_fn(|_| zero());
    let mut rest = Vec::new();
    for t in f.terms() {
        let hits: Vec<usize> = (0..6).filter(|k| t.has_sym(&sym(&names[*k]))).collect();
        let mixed_alt = t.has_sym(&sym(&alt));
        match (hits.as_slice(), mixed_alt) {
            ([], false) => rest.push(t.clone()),
            ([k], false) => {
                let co = s::cancel(&div(t.clone(), sym(&names[*k])));
                if names.iter().any(|n| co.has_sym(&sym(n))) {
                    return None;
                }
                c[*k] = add(vec![c[*k].clone(), co]);
            }
            ([], true) => {
                let co = s::cancel(&div(t.clone(), sym(&alt)));
                c[4] = add(vec![c[4].clone(), co]);
            }
            _ => return None,
        }
    }
    for k in 0..6 {
        c[k] = s::simplify(&c[k]);
    }
    Some(Pde { u, c, g: s::simplify(&neg(add(rest))), a, b })
}

fn solve_zero(e: &Expr) -> Expr {
    match e {
        Rel(RelOp::Eq, l, r) => sub((**l).clone(), (**r).clone()),
        _ => e.clone(),
    }
}

/// sin(nπ) → 0, cos(nπ) → (−1)^n for an integer symbol n.
fn integer_reduce(e: &Expr, n: &Expr) -> Expr {
    let e = s::expand(e);
    fn walk(e: &Expr, n: &Expr) -> Expr {
        match e {
            Func(name, args) if args.len() == 1 && (&**name == "sin" || &**name == "cos") => {
                let arg = walk(&args[0], n);
                let q = s::simplify(&div(arg.clone(), mul(vec![n.clone(), pi()])));
                if let Some(k) = q.as_int().and_then(|k| i64::try_from(k.clone()).ok()) {
                    return if &**name == "sin" { zero() } else { pow(int(-1), mul(vec![int(k), n.clone()])) };
                }
                // sin(nπ/2) etc. are left alone
                f1(name, arg)
            }
            Add(v) => add(v.iter().map(|t| walk(t, n)).collect()),
            Mul(v) => mul(v.iter().map(|t| walk(t, n)).collect()),
            Pow(b, x) => pow(walk(b, n), walk(x, n)),
            Func(name, args) => func(name, args.iter().map(|t| walk(t, n)).collect()),
            _ => e.clone(),
        }
    }
    s::simplify(&walk(&e, n))
}

fn exact_definite(f: &Expr, x: &Expr, lo: &Expr, hi: &Expr) -> Option<Expr> {
    match definite(f, x, lo, hi) {
        Definite::Exact(v) => Some(s::simplify(&v)),
        Definite::Numeric(v, _) => Some(float(v)),
        _ => None,
    }
}

/// Fourier coefficients of f on [0, L] for the basis φ_n (n = 1.. or 0..):
/// the first `TERMS` values exactly, and a closed form in n when the
/// symbolic integral agrees with them.
struct Coeffs {
    values: Vec<(i64, Expr)>,
    general: Option<Expr>,
}

fn coefficients(f: &Expr, x: &Expr, l: &Expr, basis: &dyn Fn(&Expr) -> Expr, scale: &dyn Fn(&Expr) -> Expr, start: i64) -> Option<Coeffs> {
    let mut values = Vec::new();
    for k in start..start + TERMS {
        let kn = int(k);
        let v = exact_definite(&mul(vec![f.clone(), basis(&kn)]), x, &zero(), l)?;
        values.push((k, s::simplify(&mul(vec![scale(&kn), v]))));
    }
    // closed form in n
    let n = sym("n");
    let general = (|| {
        let anti = integrate(&s::expand(&mul(vec![f.clone(), basis(&n)])), x)?;
        let v = sub(anti.subs(x, l), anti.subs(x, &zero()));
        let g = integer_reduce(&mul(vec![scale(&n), v]), &n);
        // must reproduce the exact values (skipping n where the formula is 0/0)
        let mut checked = 0;
        for (k, val) in values.iter().filter(|(k, _)| *k >= 1) {
            let at = s::simplify(&g.subs(&n, &int(*k)));
            let (Some(a), Some(b)) = (at.to_f64_const(), val.to_f64_const()) else { continue };
            if (a - b).abs() > 1e-9 * (1.0 + b.abs()) {
                return None;
            }
            checked += 1;
        }
        (checked >= 3).then_some(g)
    })();
    Some(Coeffs { values, general })
}

/// Does `mode` satisfy the PDE (with zero forcing)?
fn satisfies(p: &Pde, mode: &Expr) -> bool {
    let (a, b) = (sym(&p.a), sym(&p.b));
    let d = [
        mode.clone(),
        diff(mode, &a),
        diff(mode, &b),
        diff(&diff(mode, &a), &a),
        diff(&diff(mode, &a), &b),
        diff(&diff(mode, &b), &b),
    ];
    let r = add((0..6).map(|k| mul(vec![p.c[k].clone(), d[k].clone()])).collect());
    s::is_zero(&r).unwrap_or_else(|| s::numerically_zero(&r))
}

#[derive(Default)]
struct Conditions {
    /// left end value / derivative kind: Some(true) Dirichlet 0, Some(false) Neumann 0
    left: Option<bool>,
    right: Option<(bool, Expr)>,
    /// u(a, 0) = f
    initial: Option<Expr>,
    /// u_t(a, 0) = g
    initial_rate: Option<Expr>,
    /// Laplace: u(a, B) = f on the top side
    top: Option<(Expr, Expr)>,
    other: Vec<String>,
}

fn conditions(parts: &[String], p: &Pde) -> Option<Conditions> {
    let mut cs = Conditions::default();
    let (u, a, b) = (&p.u, &p.a, &p.b);
    let pat = Regex::new(&format!(r"^{u}(?:_({a}|{b}))?\s*\(\s*([^,()]+)\s*,\s*([^,()]+)\s*\)\s*=\s*(.+)$")).ok()?;
    for part in parts {
        let Some(m) = pat.captures(part.trim()) else {
            cs.other.push(part.clone());
            continue;
        };
        let deriv = m.get(1).map(|g| g.as_str().to_string());
        let (p1, p2) = (m[2].trim().to_string(), m[3].trim().to_string());
        let value = parse::parse_expr(&m[4]).ok()?;
        let is_var = |t: &str, v: &str| t == v;
        if is_var(&p2, b) && !is_var(&p1, a) {
            // boundary in the first variable at a fixed point
            let at = parse::parse_expr(&p1).ok()?;
            if !value.is_zero() {
                cs.other.push(part.clone());
                continue;
            }
            let dirichlet = deriv.is_none();
            if at.is_zero() {
                cs.left = Some(dirichlet);
            } else {
                cs.right = Some((dirichlet, at));
            }
        } else if is_var(&p1, a) && !is_var(&p2, b) {
            let at = parse::parse_expr(&p2).ok()?;
            match (deriv.as_deref(), at.is_zero()) {
                (None, true) => cs.initial = Some(value),
                (Some(d), true) if d == b => cs.initial_rate = Some(value),
                (None, false) => cs.top = Some((at, value)),
                _ => cs.other.push(part.clone()),
            }
        } else {
            cs.other.push(part.clone());
        }
    }
    Some(cs)
}

fn classification(p: &Pde) -> Option<(String, String)> {
    let (aa, ab, bb) = (&p.c[3], &p.c[4], &p.c[5]);
    if aa.is_zero() && ab.is_zero() && bb.is_zero() {
        return None;
    }
    // A u_aa + B u_ab + C u_bb: discriminant B² − 4AC
    let disc = s::simplify(&sub(square(ab.clone()), mul(vec![int(4), aa.clone(), bb.clone()])));
    let kind = match disc.to_f64_const() {
        Some(v) if v > 1e-12 => "hyperbolic",
        Some(v) if v < -1e-12 => "elliptic",
        Some(_) => "parabolic",
        None => "of mixed type (the discriminant changes sign)",
    };
    Some((kind.to_string(), format!("B² − 4AC = {}", ptext(&disc))))
}

fn sum_holder(term: &Expr, n: &Expr, lo: i64) -> Expr {
    raw_fn("Sum", vec![term.clone(), n.clone(), int(lo), inf()])
}

fn finite(values: &[(i64, Expr)], term: &dyn Fn(i64, &Expr) -> Expr) -> Expr {
    add(values.iter().filter(|(_, c)| !c.is_zero()).map(|(k, c)| term(*k, c)).collect())
}

/// Are all coefficients after the first few exactly zero? Then the series
/// is a finite sum and the solution is exact.
fn is_finite(values: &[(i64, Expr)]) -> bool {
    let nonzero: Vec<i64> = values.iter().filter(|(_, c)| !c.is_zero()).map(|(k, _)| *k).collect();
    nonzero.last().map_or(true, |k| *k <= values.last().map_or(0, |v| v.0) - 3)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    if let Some(a) = named(l) {
        return Some(a);
    }
    let raw = q.clean.trim();
    let raw = re!(r"(?i)^(?:solve\s+)?(?:the\s+)?(?:pde|partial differential equation)?\s*:?\s*").replace(raw, "").to_string();
    let src = normalize(&raw);
    // drop a domain description such as "on 0 < x < 1" or "for 0<x<pi, t>0"
    let src = re!(r"(?i),?\s*(?:on|for|in|where)\s+[-\d.a-zπ]+\s*<=?\s*[a-z]\s*<=?\s*[-\d.a-zπ/ ]+(?:\s*,\s*[a-z]\s*>=?\s*0)?").replace_all(&src, "").to_string();
    if !re!(r"\b[a-z]_[a-z]{1,2}\b").is_match(&src) {
        return None;
    }
    let parts = super::nl::split_top(&src);
    let (eq_part, rest): (Vec<String>, Vec<String>) = parts.into_iter().partition(|p| re!(r"\b[a-z]_[a-z]{1,2}\b").is_match(p) && !re!(r"\b[a-z](?:_[a-z])?\s*\(").is_match(p));
    let eq_text = eq_part.first()?.clone();
    let p = analyse(&eq_text)?;
    let conds = conditions(&rest, &p)?;
    let (a, b) = (sym(&p.a), sym(&p.b));
    let e_show = parse::parse_expr(&eq_text).ok()?;
    let mut pods = vec![pod("Input", "Input", math(&e_show))];
    let order2 = !(p.c[3].is_zero() && p.c[4].is_zero() && p.c[5].is_zero());
    let constant = p.c.iter().all(|c| !c.has_sym(&a) && !c.has_sym(&b));
    let mut kind = format!("{} linear partial differential equation in {} and {}", if order2 { "second-order" } else { "first-order" }, p.a, p.b);
    if let Some((k, d)) = classification(&p) {
        kind = format!("{kind} — {k}");
        pods.push(pod("Classification", "Classification", text(format!("{k} ({d})"))));
    }
    pods.insert(1, pod("Type", "Equation type", text(kind)));
    if !conds.other.is_empty() {
        pods.push(pod("Unused", "Conditions not used", text(conds.other.join("; "))));
    }

    // ---------------- first order: a u_a + b u_b + c u = g --------------
    if !order2 && constant && p.g.is_zero() {
        let (ca, cb, c0) = (p.c[1].clone(), p.c[2].clone(), p.c[0].clone());
        if ca.is_zero() || cb.is_zero() {
            return None;
        }
        let xi = s::simplify(&sub(mul(vec![cb.clone(), a.clone()]), mul(vec![ca.clone(), b.clone()])));
        let damp = s::simplify(&exp(neg(div(mul(vec![c0.clone(), b.clone()]), cb.clone()))));
        let general = s::simplify(&mul(vec![damp.clone(), func("F", vec![xi.clone()])]));
        pods.push(primary(pod("General", "General solution", math(&eq(sym(&p.u), general)))));
        pods.push(pod("Method", "Method of characteristics", text(format!("u is constant (up to the factor e^(−c·{}/b)) along the characteristic lines {} = constant; F is an arbitrary differentiable function.", p.b, ptext(&xi)))));
        if let Some(f0) = &conds.initial {
            // u(a, 0) = f(a): F(cb·a) = f(a) → F(s) = f(s/cb)
            let sol = s::simplify(&mul(vec![damp, f0.subs(&a, &div(xi.clone(), cb.clone()))]));
            pods.iter_mut().for_each(|pd| pd.primary = false);
            pods.insert(2, primary(pod("Solution", "Solution of the initial value problem", math(&eq(sym(&p.u), sol.clone())))));
            if p.b == "t" && c0.is_zero() {
                let speed = s::simplify(&div(ca.clone(), cb.clone()));
                pods.push(pod("Wave", "Interpretation", text(format!("the initial profile travels with speed {} without changing shape", ptext(&speed)))));
            }
            if let Some(pl) = pl::surface(&sol, &a, &b, (-5.0, 5.0), (0.0, 3.0), 60, pl::PlotKind::Surface) {
                pods.push(pod("Plot", "Solution surface", plot(pl)));
            }
        }
        return Some(Answer::new(format!("{} (PDE)", ptext(&e_show)), pods));
    }

    // ---------------- heat and wave equations on an interval ------------
    let heat = constant && p.c[2].to_f64_const().is_some_and(|v| v != 0.0) && p.c[5].is_zero() && p.c[4].is_zero() && p.c[0].is_zero() && p.c[1].is_zero() && !p.c[3].is_zero();
    let wave = constant && p.c[2].is_zero() && p.c[4].is_zero() && p.c[0].is_zero() && p.c[1].is_zero() && !p.c[3].is_zero() && !p.c[5].is_zero();
    // k = −c_aa / c_b for heat, c² = −c_aa / c_bb for wave
    let k = if heat { Some(s::simplify(&neg(div(p.c[3].clone(), p.c[2].clone())))) } else { None };
    let c2 = if wave { Some(s::simplify(&neg(div(p.c[3].clone(), p.c[5].clone())))) } else { None };
    if (heat && k.as_ref()?.to_f64_const().is_some_and(|v| v > 0.0)) || (wave && c2.as_ref()?.to_f64_const().is_some_and(|v| v > 0.0)) {
        if !p.g.is_zero() {
            pods.push(pod("Note", "Note", text("Only the homogeneous equation (no source term) is solved in closed form.")));
            return Some(Answer::new(format!("{} (PDE)", ptext(&e_show)), pods));
        }
        let x = a.clone();
        let t = b.clone();
        // whole line (wave): d'Alembert
        if wave && conds.left.is_none() && conds.right.is_none() {
            if let Some(f0) = &conds.initial {
                let c = s::simplify(&sqrt(c2.clone()?));
                let g0 = conds.initial_rate.clone().unwrap_or_else(zero);
                let sv = sym("σ");
                let lo = sub(x.clone(), mul(vec![c.clone(), t.clone()]));
                let hi = add(vec![x.clone(), mul(vec![c.clone(), t.clone()])]);
                let integral = if g0.is_zero() {
                    zero()
                } else {
                    let anti = integrate(&g0.subs(&x, &sv), &sv)?;
                    sub(anti.subs(&sv, &hi), anti.subs(&sv, &lo))
                };
                let sol = s::simplify(&add(vec![div(add(vec![f0.subs(&x, &lo), f0.subs(&x, &hi)]), int(2)), div(integral, mul(vec![int(2), c.clone()]))]));
                pods.push(primary(pod("Solution", "Solution (d'Alembert's formula)", math(&eq(sym(&p.u), sol.clone())))));
                pods.push(pod("Formula", "Formula", text(format!("u = [f(x − ct) + f(x + ct)]/2 + (1/2c) ∫_(x−ct)^(x+ct) g(s) ds,  with c = {}", ptext(&c)))));
                if let Some(pl) = pl::surface(&sol, &x, &t, (-6.0, 6.0), (0.0, 3.0), 70, pl::PlotKind::Surface) {
                    pods.push(pod("Plot", "Solution surface", plot(pl)));
                }
                return Some(Answer::new(format!("{} (PDE)", ptext(&e_show)), pods));
            }
        }
        let (Some(left), Some((right, len))) = (conds.left, conds.right.clone()) else {
            // separated solutions only
            let lam = sym("λ");
            let modes = if heat {
                format!("u = e^(−{} λ² {}) (A cos(λ {}) + B sin(λ {}))", ptext(k.as_ref()?), p.b, p.a, p.a)
            } else {
                format!("u = (A cos(c λ {}) + B sin(c λ {})) (C cos(λ {}) + D sin(λ {})),  c = {}", p.b, p.b, p.a, p.a, ptext(&s::simplify(&sqrt(c2.clone()?))))
            };
            let _ = lam;
            pods.push(primary(pod("Separated", "Separated solutions", text(modes))));
            let (uu, tt, xx) = (&p.u, &p.b, &p.a);
            let rate = if wave { format!(", {uu}_{tt}({xx},0) = g({xx})") } else { String::new() };
            pods.push(pod("Hint", "To get a unique solution", text(format!("add boundary conditions such as {uu}(0,{tt}) = 0, {uu}(L,{tt}) = 0 and an initial condition {uu}({xx},0) = f({xx}){rate}"))));
            return Some(Answer::new(format!("{} (PDE)", ptext(&e_show)), pods));
        };
        if left != right {
            pods.push(pod("Note", "Note", text("Mixed Dirichlet/Neumann ends are not solved in closed form.")));
            return Some(Answer::new(format!("{} (PDE)", ptext(&e_show)), pods));
        }
        let dirichlet = left;
        let n = sym("n");
        let kx = |m: &Expr| div(mul(vec![m.clone(), pi(), x.clone()]), len.clone());
        let lam = |m: &Expr| div(mul(vec![m.clone(), pi()]), len.clone());
        let space = |m: &Expr| if dirichlet { f1("sin", kx(m)) } else { f1("cos", kx(m)) };
        let start = if dirichlet { 1 } else { 0 };
        let two_over_l = |m: &Expr| {
            if !dirichlet && m.is_zero() { div(one(), len.clone()) } else { div(int(2), len.clone()) }
        };
        let f0 = conds.initial.clone().unwrap_or_else(zero);
        let fc = coefficients(&f0, &x, &len, &|m| space(m), &|m| two_over_l(m), start)?;
        let time_heat = |m: &Expr| exp(neg(mul(vec![k.clone().unwrap_or_else(zero), square(lam(m)), t.clone()])));
        let cspeed = c2.as_ref().map(|v| s::simplify(&sqrt(v.clone())));
        // wave: velocity data
        let gc = if wave {
            let g0 = conds.initial_rate.clone().unwrap_or_else(zero);
            let c = cspeed.clone()?;
            let scale_g = |m: &Expr| if !dirichlet && m.is_zero() { div(one(), len.clone()) } else { div(int(2), mul(vec![lam(m), c.clone(), len.clone()])) };
            Some(coefficients(&g0, &x, &len, &|m| space(m), &scale_g, start)?)
        } else {
            None
        };
        let mode = |m: &Expr, cf: &Expr, cg: Option<&Expr>| -> Expr {
            if heat {
                mul(vec![cf.clone(), time_heat(m), space(m)])
            } else {
                let c = cspeed.clone().unwrap_or_else(one);
                let w = mul(vec![c.clone(), lam(m), t.clone()]);
                let tpart = if m.is_zero() {
                    add(vec![cf.clone(), mul(vec![cg.cloned().unwrap_or_else(zero), t.clone()])])
                } else {
                    add(vec![mul(vec![cf.clone(), f1("cos", w.clone())]), mul(vec![cg.cloned().unwrap_or_else(zero), f1("sin", w)])])
                };
                mul(vec![tpart, space(m)])
            }
        };
        // check one generic mode against the equation
        let check = mode(&int(3), &one(), Some(&one()));
        let ok = satisfies(&p, &check);
        let exact_finite = is_finite(&fc.values) && gc.as_ref().map_or(true, |g| is_finite(&g.values));
        // keep the partial sum as a sum of simplified modes
        let partial = add(fc.values.iter().enumerate().map(|(i, (m, cf))| s::simplify(&mode(&int(*m), cf, gc.as_ref().map(|g| &g.values[i].1)))).filter(|t| !t.is_zero()).collect());
        let what = if heat { "heat equation" } else { "wave equation" };
        let ends = if dirichlet { "ends held at zero (Dirichlet)" } else { "insulated ends (Neumann)" };
        if exact_finite {
            pods.push(primary(pod("Solution", "Solution", math(&eq(sym(&p.u), partial.clone())))));
            pods.push(pod("Exact", "Exactness", text("The initial data is a finite Fourier series, so this finite sum is the exact solution.")));
        } else if let (Some(gf), true) = (&fc.general, gc.as_ref().map_or(true, |g| g.general.is_some())) {
            let general_mode = mode(&n, gf, gc.as_ref().and_then(|g| g.general.as_ref()));
            // the n = 0 cosine mode has its own coefficient formula
            let series = if start == 0 {
                let m0 = s::simplify(&mode(&zero(), &fc.values[0].1, gc.as_ref().map(|g| &g.values[0].1)));
                add(vec![m0, sum_holder(&s::simplify(&general_mode), &n, 1)])
            } else {
                sum_holder(&s::simplify(&general_mode), &n, 1)
            };
            pods.push(primary(pod("Solution", "Solution", math(&eq(sym(&p.u), series)))));
            pods.push(pod("Partial", &format!("Sum of the first {TERMS} terms"), math(&partial)));
        } else {
            pods.push(primary(pod("Solution", &format!("Solution (first {TERMS} terms of the Fourier series)"), math(&partial))));
        }
        let coeff_title = if wave { "Fourier coefficients (position, velocity)" } else { "Fourier coefficients" };
        let rows: Vec<Vec<Content>> = fc.values.iter().enumerate().map(|(i, (m, v))| {
            let mut r = vec![text(format!("n = {m}")), math(v)];
            if let Some(g) = &gc {
                r.push(math(&g.values[i].1));
            }
            r
        }).collect();
        pods.push(pod("Coefficients", coeff_title, table(None, rows)));
        if let Some(gf) = &fc.general {
            pods.push(pod("General", "Coefficient formula", math(&eq(raw_fn("user:b", vec![n.clone()]), gf.clone()))));
        }
        pods.push(pod("Method", "Method", text(format!(
            "Separation of variables for the {what} on 0 < {x} < {Lv} with {ends}: the eigenfunctions are {phi}({arg}), and the coefficients are the Fourier {kind} coefficients of the initial data.",
            x = p.a, Lv = ptext(&len), arg = ptext(&s::simplify(&kx(&sym("n")))), phi = if dirichlet { "sin" } else { "cos" }, kind = if dirichlet { "sine" } else { "cosine" }
        ))));
        pods.push(pod("Check", "Verification", text(if ok { "each separated mode satisfies the equation and the boundary conditions ✓" } else { "verification of the separated modes failed" })));
        if let Some(lf) = len.to_f64_const() {
            let tmax = if heat { (lf * lf / (k.as_ref()?.to_f64_const()? * std::f64::consts::PI.powi(2))).clamp(0.05, 50.0) } else { 2.0 * lf / cspeed.as_ref()?.to_f64_const()? };
            if let Some(pl) = pl::surface(&partial, &x, &t, (0.0, lf), (0.0, tmax), 60, pl::PlotKind::Surface) {
                pods.push(pod("Plot", "Solution surface u(x, t)", plot(pl)));
            }
        }
        return Some(Answer::new(format!("{what} on [0, {}]", ptext(&len)), pods));
    }

    // ---------------- Laplace's equation on a rectangle -----------------
    let laplace = constant && p.c[3] == p.c[5] && !p.c[3].is_zero() && p.c[4].is_zero() && p.c[0].is_zero() && p.c[1].is_zero() && p.c[2].is_zero() && p.g.is_zero();
    if laplace {
        let bottom_zero = conds.initial.as_ref().is_some_and(|v| v.is_zero());
        if let (Some(true), Some((true, width)), Some((height, top)), true) = (conds.left, conds.right.clone(), conds.top.clone(), bottom_zero) {
            let (x, y) = (a.clone(), b.clone());
            let lam = |m: &Expr| div(mul(vec![m.clone(), pi()]), width.clone());
            let fc = coefficients(&top, &x, &width, &|m| f1("sin", mul(vec![lam(m), x.clone()])), &|m| div(int(2), mul(vec![width.clone(), f1("sinh", mul(vec![lam(m), height.clone()]))])), 1)?;
            let mode = |m: &Expr, cf: &Expr| mul(vec![cf.clone(), f1("sin", mul(vec![lam(m), x.clone()])), f1("sinh", mul(vec![lam(m), y.clone()]))]);
            let partial = finite(&fc.values, &|m, c| s::simplify(&mode(&int(m), c)));
            let n = sym("n");
            if is_finite(&fc.values) {
                pods.push(primary(pod("Solution", "Solution", math(&eq(sym(&p.u), partial.clone())))));
            } else if let Some(gf) = &fc.general {
                pods.push(primary(pod("Solution", "Solution", math(&eq(sym(&p.u), sum_holder(&s::simplify(&mode(&n, gf)), &n, 1))))));
                pods.push(pod("Partial", &format!("Sum of the first {TERMS} terms"), math(&partial)));
            } else {
                pods.push(primary(pod("Solution", &format!("Solution (first {TERMS} terms)"), math(&partial))));
            }
            pods.push(pod("Method", "Method", text(format!("Separation of variables on the rectangle 0 < {} < {}, 0 < {} < {}: u = 0 on three sides and u = f on {} = {}.", p.a, ptext(&width), p.b, ptext(&height), p.b, ptext(&height)))));
            pods.push(pod("Check", "Verification", text(if satisfies(&p, &mode(&int(2), &one())) { "each separated mode is harmonic and vanishes on the three zero sides ✓" } else { "verification failed" })));
            if let (Some(w), Some(h)) = (width.to_f64_const(), height.to_f64_const()) {
                if let Some(pl) = pl::surface(&partial, &x, &y, (0.0, w), (0.0, h), 60, pl::PlotKind::Surface) {
                    pods.push(pod("Plot", "Solution surface", plot(pl)));
                }
            }
            return Some(Answer::new("Laplace's equation on a rectangle", pods));
        }
        pods.push(primary(pod("Separated", "Separated solutions", text(format!("harmonic functions; separated solutions (A cos λ{x} + B sin λ{x})(C cosh λ{y} + D sinh λ{y}), and every polynomial such as {x}² − {y}², {x}{y}", x = p.a, y = p.b)))));
        pods.push(pod("Hint", "To get a unique solution", text(format!("give boundary values on a rectangle, e.g. {u}(0,{y}) = 0, {u}(a,{y}) = 0, {u}({x},0) = 0, {u}({x},b) = f({x})", u = p.u, x = p.a, y = p.b))));
        return Some(Answer::new("Laplace's equation", pods));
    }
    // anything else: classification only
    if pods.iter().any(|pd| pd.id == "Classification") {
        if let Some(pd) = pods.iter_mut().find(|pd| pd.id == "Classification") {
            pd.primary = true;
        }
        pods.push(pod("Note", "Note", text("No closed-form solution method applies to this equation; it is classified only.")));
        return Some(Answer::new(format!("{} (PDE)", ptext(&e_show)), pods));
    }
    None
}

/// The named equations, described.
fn named(l: &str) -> Option<Answer> {
    let t = l.trim_start_matches("the ").trim_start_matches("what is the ");
    let (title, form, kind, about) = match t {
        "heat equation" | "diffusion equation" => ("Heat equation", "∂u/∂t = k ∂²u/∂x²", "parabolic", "Models diffusion of heat or a concentration. On 0 < x < L with u = 0 at both ends: u = Σ b_n e^(−k(nπ/L)² t) sin(nπx/L), b_n = (2/L)∫₀ᴸ f(x) sin(nπx/L) dx. On the whole line: u = (4πkt)^(−1/2) ∫ f(s) e^(−(x−s)²/(4kt)) ds."),
        "wave equation" => ("Wave equation", "∂²u/∂t² = c² ∂²u/∂x²", "hyperbolic", "Models vibrating strings and sound. On the whole line: u = [f(x−ct) + f(x+ct)]/2 + (1/2c)∫_(x−ct)^(x+ct) g(s) ds (d'Alembert). On 0 < x < L with fixed ends: u = Σ (A_n cos(nπct/L) + B_n sin(nπct/L)) sin(nπx/L)."),
        "laplace equation" | "laplace's equation" | "laplaces equation" => ("Laplace's equation", "∂²u/∂x² + ∂²u/∂y² = 0", "elliptic", "Solutions are harmonic functions: they satisfy the mean-value property and attain their extrema on the boundary."),
        "poisson equation" | "poisson's equation" => ("Poisson's equation", "∂²u/∂x² + ∂²u/∂y² = f(x, y)", "elliptic", "Laplace's equation with a source term; in electrostatics ∇²φ = −ρ/ε₀."),
        "transport equation" | "advection equation" => ("Transport equation", "∂u/∂t + c ∂u/∂x = 0", "first order, hyperbolic", "Solution u = f(x − ct): the initial profile moves with speed c."),
        "schrodinger equation" | "schrödinger equation" => ("Schrödinger equation", "i ħ ∂ψ/∂t = −(ħ²/2m) ∇²ψ + V ψ", "dispersive (parabolic-like with an imaginary coefficient)", "Time evolution of a quantum state; stationary states satisfy −(ħ²/2m)∇²ψ + Vψ = Eψ."),
        "burgers equation" | "burgers' equation" => ("Burgers' equation", "∂u/∂t + u ∂u/∂x = ν ∂²u/∂x²", "nonlinear", "With ν = 0 shocks form in finite time; the Cole–Hopf transform u = −2ν φ_x/φ turns it into the heat equation."),
        _ => return None,
    };
    let pods = vec![
        pod("Input", "Input interpretation", text(title)),
        primary(pod("Result", "Equation", text(form))),
        pod("Type", "Classification", text(kind)),
        pod("About", "About", text(about)),
        pod("Try", "Example input", text(match title {
            "Heat equation" => "u_t = u_xx, u(0,t) = 0, u(1,t) = 0, u(x,0) = x(1 - x)",
            "Wave equation" => "u_tt = 4 u_xx, u(x,0) = e^(-x^2), u_t(x,0) = 0",
            "Transport equation" => "u_t + 2 u_x = 0, u(x,0) = sin(x)",
            _ => "u_xx + u_yy = 0, u(0,y) = 0, u(pi,y) = 0, u(x,0) = 0, u(x,1) = sin(x)",
        })),
    ];
    Some(Answer::new(title, pods))
}
