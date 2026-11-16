//! Topics from calculus and statistics courses: directional derivatives,
//! tangent planes, Lagrange multipliers, power-series convergence, the
//! mean value theorem, numerical integration and root finding, Euler's
//! method, applications of integrals, and common distributions and tests.

use super::nl::{answer, caps, decimal_pod, ex, main_var, split_top};
use super::nl_math::{definite_expr, fx, g, plot_pod, point, value_pods};
use super::*;
use crate::calculus::diff::diff;
use crate::calculus::limit::{limit, Dir};
use crate::calculus::sum::{sum, SumResult};
use crate::expr::Expr::*;
use crate::expr::*;
use crate::print::{fmt_float, text as ptext};
use crate::simplify as s;
use crate::solve;
use regex::Regex;
use statrs::distribution::{ChiSquared, ContinuousCDF, Normal, StudentsT};
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn n(v: f64) -> String {
    fmt_float(v, 8)
}

fn f64_of(t: &str) -> Option<f64> {
    ex(t)?.to_f64_const()
}

fn eval_at(f: &Expr, x: &Expr, v: f64) -> Option<f64> {
    f.subs(x, &float(v)).to_f64_const()
}

fn pair(t: &str) -> Option<(Expr, Expr)> {
    let m = re!(r"^\s*[(\[]\s*([^,]+?)\s*,\s*([^,]+?)\s*[)\]]\s*$").captures(t)?;
    Some((ex(&m[1])?, ex(&m[2])?))
}

fn xy() -> (Expr, Expr) {
    (sym("x"), sym("y"))
}

// ---------------------------------------------------------------------------
// several variables
// ---------------------------------------------------------------------------

fn multivariable(l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    let (x, y) = xy();
    if let Some(m) = cap(re!(r"^(?:the\s+)?directional derivative\s+(?:of\s+)?(.+?)\s+at\s+(\([^)]*\))\s+(?:in\s+the\s+direction\s+(?:of\s+)?|along\s+|in direction\s+)(?:the vector\s+)?(<[^>]*>|\([^)]*\))$")) {
        let f = fx(&g(&m, 1))?;
        let (a, b) = pair(&g(&m, 2))?;
        let (u, v) = pair(&g(&m, 3).replace('<', "(").replace('>', ")"))?;
        let (fxv, fyv) = (diff(&f, &x), diff(&f, &y));
        let at = |e: &Expr| s::simplify(&e.subs_many(&[(x.clone(), a.clone()), (y.clone(), b.clone())]));
        let len = s::simplify(&sqrt(add(vec![square(u.clone()), square(v.clone())])));
        if len.is_zero() {
            return None;
        }
        let dd = s::simplify(&div(add(vec![mul(vec![at(&fxv), u.clone()]), mul(vec![at(&fyv), v.clone()])]), len.clone()));
        let label = format!("directional derivative of {} at ({}, {}) along ({}, {})", ptext(&f), ptext(&a), ptext(&b), ptext(&u), ptext(&v));
        let mut pods = value_pods("Directional derivative", None, &dd);
        pods.push(pod("Gradient", "Gradient there", math(&point(&at(&fxv), &at(&fyv)))));
        pods.push(pod("Unit", "Unit direction", math(&point(&s::simplify(&div(u, len.clone())), &s::simplify(&div(v, len))))));
        pods.push(pod("Max", "Largest rate of increase (|∇f|)", math(&s::simplify(&sqrt(add(vec![square(at(&fxv)), square(at(&fyv))]))))));
        return answer(label.clone(), label, pods);
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:equation of\s+(?:the\s+)?)?tangent plane\s+(?:to|of)\s+(?:the surface\s+)?(?:z\s*=\s*|f\s*\(\s*x\s*,\s*y\s*\)\s*=\s*)?(.+?)\s+at\s+(?:the point\s+)?(\([^)]*\))$")) {
        let f = ex(&g(&m, 1))?;
        let coords: Vec<Expr> = g(&m, 2).trim_matches(|ch| ch == '(' || ch == ')').split(',').filter_map(ex).collect();
        if coords.len() < 2 {
            return None;
        }
        let (a, b) = (coords[0].clone(), coords[1].clone());
        let at = |e: &Expr| s::simplify(&e.subs_many(&[(x.clone(), a.clone()), (y.clone(), b.clone())]));
        let (z0, p, q) = (at(&f), at(&diff(&f, &x)), at(&diff(&f, &y)));
        let plane = s::expand(&add(vec![z0.clone(), mul(vec![p.clone(), sub(x.clone(), a.clone())]), mul(vec![q.clone(), sub(y.clone(), b.clone())])]));
        let label = format!("tangent plane to z = {} at ({}, {})", ptext(&f), ptext(&a), ptext(&b));
        let pods = vec![
            primary(pod("Result", "Tangent plane", math(&eq(sym("z"), plane)))),
            pod("Point", "Point of tangency", text(format!("({}, {}, {})", ptext(&a), ptext(&b), ptext(&z0)))),
            pod("Normal", "Normal vector", text(format!("({}, {}, −1)", ptext(&p), ptext(&q)))),
        ];
        return answer(label.clone(), label, pods);
    }
    // Lagrange multipliers
    if let Some(m) = cap(re!(r"^(?:use\s+)?(?:lagrange multipliers?\s*:?\s*(?:to\s+)?)?(maximi[sz]e|minimi[sz]e|optimi[sz]e|extrema of|extreme values of|max and min of|maximum of|minimum of)\s+(.+?)\s+(?:subject to|with the constraint|with constraint|given|constrained by|on|in|such that)\s+(.+)$")).filter(|_| l.contains("subject to") || l.contains("constraint") || l.contains("lagrange") || l.contains("such that") || l.contains(" on ") || l.contains(" given ") || (l.contains(" in ") && l.contains(['<', '>', '='])) ) {
        let f = ex(&g(&m, 2))?;
        let cons = ex(&g(&m, 3))?;
        // a region g ≤ c: extrema on the boundary g = c, plus interior critical points
        let region = matches!(&cons, Rel(RelOp::Le | RelOp::Lt | RelOp::Ge | RelOp::Gt, _, _));
        let gexpr = match &cons {
            Rel(_, a, b) => sub((**a).clone(), (**b).clone()),
            _ => return None,
        };
        let mut vars: Vec<Expr> = f.free_symbols().union(&gexpr.free_symbols()).map(|nm| sym(nm)).collect();
        vars.sort_by_key(|v| ptext(v));
        if !(2..=3).contains(&vars.len()) {
            return None;
        }
        let lam = sym("λ");
        let mut eqs: Vec<Expr> = vars.iter().map(|v| sub(diff(&f, v), mul(vec![lam.clone(), diff(&gexpr, v)]))).collect();
        eqs.push(gexpr.clone());
        let mut all = vars.clone();
        all.push(lam.clone());
        let sol = solve::solve_system(&eqs, &all);
        let mut rows = Vec::new();
        let mut vals: Vec<(f64, String)> = Vec::new();
        for sset in &sol.sols {
            let pt: Vec<Expr> = vars.iter().map(|v| sset.iter().find(|(k, _)| k == v).map(|(_, e)| e.clone())).collect::<Option<_>>()?;
            if pt.iter().any(|e| e.has_const(K::I)) {
                continue;
            }
            let fv = s::simplify(&f.subs_many(&vars.iter().cloned().zip(pt.iter().cloned()).collect::<Vec<_>>()));
            let shown = format!("({})", pt.iter().map(ptext).collect::<Vec<_>>().join(", "));
            if let Some(v) = fv.to_f64_const() {
                vals.push((v, shown.clone()));
            }
            rows.push(vec![text(shown), math(&fv)]);
        }
        if region {
            let sign = if matches!(&cons, Rel(RelOp::Le | RelOp::Lt, _, _)) { 1.0 } else { -1.0 };
            let grads: Vec<Expr> = vars.iter().map(|v| diff(&f, v)).collect();
            let inner = solve::solve_system(&grads, &vars);
            for sset in &inner.sols {
                let Some(pt) = vars.iter().map(|v| sset.iter().find(|(k, _)| k == v).map(|(_, e)| e.clone())).collect::<Option<Vec<Expr>>>() else { continue };
                let subs: Vec<(Expr, Expr)> = vars.iter().cloned().zip(pt.iter().cloned()).collect();
                let Some(gv) = s::simplify(&gexpr.subs_many(&subs)).to_f64_const() else { continue };
                if sign * gv > 1e-12 {
                    continue;
                }
                let fv = s::simplify(&f.subs_many(&subs));
                let shown = format!("({})", pt.iter().map(ptext).collect::<Vec<_>>().join(", "));
                if let Some(v) = fv.to_f64_const() {
                    vals.push((v, shown.clone()));
                }
                rows.push(vec![text(format!("{shown} (interior)")), math(&fv)]);
            }
        }
        if rows.is_empty() {
            return None;
        }
        let want = g(&m, 1).to_lowercase();
        let (mx, mn) = (vals.iter().cloned().fold(None, |b: Option<(f64, String)>, v| if b.as_ref().map_or(true, |bb| v.0 > bb.0) { Some(v) } else { b }), vals.iter().cloned().fold(None, |b: Option<(f64, String)>, v| if b.as_ref().map_or(true, |bb| v.0 < bb.0) { Some(v) } else { b }));
        let summary = if want.starts_with("maxim") || want.starts_with("maximum") {
            mx.map(|(v, p)| format!("maximum {} at {p}", n(v)))
        } else if want.starts_with("minim") || want.starts_with("minimum") {
            mn.map(|(v, p)| format!("minimum {} at {p}", n(v)))
        } else {
            Some(format!("{}{}", mx.map(|(v, p)| format!("largest value {} at {p}", n(v))).unwrap_or_default(), mn.map(|(v, p)| format!(";  smallest value {} at {p}", n(v))).unwrap_or_default()))
        };
        let label = format!("{want} {} subject to {}", ptext(&f), ptext(&cons));
        let mut pods = vec![primary(pod("Result", "Result", text(summary?)))];
        pods.push(pod("Candidates", "Critical points of the Lagrangian", table(Some(vec!["point", "f"]), rows)));
        pods.push(pod("System", "Equations", text(format!("∇f = λ ∇g and g = 0, with g = {}", ptext(&gexpr)))));
        pods.push(pod("Note", "Note", text("On an unbounded constraint curve a candidate may be only a local extremum; compare the values.")));
        return answer(label.clone(), label, pods);
    }
    None
}

// ---------------------------------------------------------------------------
// series
// ---------------------------------------------------------------------------

fn series_q(l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    if let Some(m) = cap(re!(r"^(?:the\s+)?(radius|interval) of convergence\s+(?:of|for)\s+(?:the\s+)?(?:power\s+)?(?:series\s+)?(?:sum\s+)?(.+?)(?:\s*,?\s*(?:from\s+|for\s+)?([a-z])\s*=\s*(\S+)\s+to\s+(?:infinity|inf|∞))?$")) {
        let body = g(&m, 2);
        let a_n = ex(&body)?;
        let x = sym("x");
        if !a_n.has_sym(&x) {
            return None;
        }
        let k = if g(&m, 3).is_empty() { a_n.free_symbols().iter().map(|nm| sym(nm)).find(|v| *v != x)? } else { sym(&g(&m, 3)) };
        let start = if g(&m, 4).is_empty() { if s::simplify(&a_n.subs(&k, &zero())).to_f64_const().is_some_and(|v| v.is_finite()) { zero() } else { one() } } else { ex(&g(&m, 4))? };
        // ratio test in |x − c|: lim |a_(n+1)/a_n|
        let ratio = s::simplify(&div(a_n.subs(&k, &add(vec![k.clone(), one()])), a_n.clone()));
        let lim = limit(&ratio, &k, &inf(), Dir::Both)?;
        // lim = L · |x − c|^p form: evaluate at two points to find L and the centre
        let lim = s::simplify(&lim);
        let centre = solve::solve(&eq(lim.clone(), zero()), &x).real().into_iter().next().unwrap_or_else(zero);
        let at = |v: f64| lim.subs(&x, &add(vec![centre.clone(), float(v)])).to_f64_const().map(f64::abs);
        let (r1, r2) = (at(1.0)?, at(2.0)?);
        if r1 == 0.0 {
            let label = format!("{} of convergence of Σ {}", g(&m, 1), ptext(&a_n));
            return answer(label.clone(), label, vec![primary(pod("Result", "Result", text("The series converges for every x (radius of convergence ∞).")))]);
        }
        let p = (r2 / r1).log2().round();
        if !(1.0..=4.0).contains(&p) {
            return None;
        }
        let radius_f = (1.0 / r1).powf(1.0 / p);
        let radius = super::math::identify_constant(radius_f).unwrap_or_else(|| float(radius_f));
        let label = format!("{} of convergence of Σ {}", g(&m, 1), ptext(&a_n));
        let mut pods = Vec::new();
        let rpod = pod("Radius", "Radius of convergence", math(&radius));
        if g(&m, 1) == "radius" {
            pods.push(primary(rpod.clone()));
        }
        // endpoints
        let mut ends = Vec::new();
        for (sign, side) in [(-1, "left"), (1, "right")] {
            let xe = s::simplify(&add(vec![centre.clone(), mul(vec![int(sign), radius.clone()])]));
            let term = s::simplify(&a_n.subs(&x, &xe));
            let verdict = match sum(&term, &k, &start, &inf()) {
                SumResult::Exact(v) if !v.is_inf() && !v.is_neg_inf() && v.to_f64_const().is_some_and(|f| f.is_finite()) => Some(true),
                SumResult::Numeric(_) => Some(true),
                SumResult::Exact(_) | SumResult::Divergent(_) => Some(false),
                SumResult::Unknown => limit(&term, &k, &inf(), Dir::Both).filter(|t| !t.is_zero()).map(|_| false),
            };
            ends.push((side, xe, verdict));
        }
        let (lo, hi) = (&ends[0], &ends[1]);
        let br = |v: &Option<bool>, open: char, closed: char| match v {
            Some(true) => closed,
            _ => open,
        };
        let interval = format!("{}{}, {}{}", br(&lo.2, '(', '['), ptext(&lo.1), ptext(&hi.1), br(&hi.2, ')', ']'));
        let ipod = pod("Interval", "Interval of convergence", text(interval));
        if g(&m, 1) == "interval" {
            pods.push(primary(ipod));
            pods.push(rpod);
        } else {
            pods.push(ipod);
        }
        let rows: Vec<Vec<Content>> = ends.iter().map(|(side, xe, v)| vec![text(format!("{side} endpoint x = {}", ptext(xe))), text(match v {
            Some(true) => "converges",
            Some(false) => "diverges",
            None => "undetermined",
        })]).collect();
        pods.push(pod("Endpoints", "Endpoints", table(None, rows)));
        pods.push(pod("Method", "Method", text(format!("ratio test: lim |a(n+1)/a(n)| = {} < 1", ptext(&lim)))));
        return answer(label.clone(), label, pods);
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:nth|n-th|divergence) term test\s+(?:for|on|of)\s+(?:the\s+)?(?:series\s+)?(?:sum\s+)?(.+)$")) {
        let f = ex(&g(&m, 1))?;
        let k = main_var(&f);
        let lim = limit(&f, &k, &inf(), Dir::Both)?;
        let label = format!("divergence test for Σ {}", ptext(&f));
        let verdict = if lim.is_zero() { "The terms tend to 0, so the test is inconclusive: the series may converge or diverge." } else { "The terms do not tend to 0, so the series diverges." };
        return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(verdict))), pod("Limit", "Limit of the terms", math(&eq(raw_fn("Limit", vec![f.clone(), k.clone(), inf()]), lim)))]);
    }
    None
}

// ---------------------------------------------------------------------------
// numerical methods and the mean value theorem
// ---------------------------------------------------------------------------

fn numerics(l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    if let Some(m) = cap(re!(r"^(?:the\s+)?mean value theorem\s+(?:for|on|applied to)\s+(.+?)\s+(?:on|over)\s+[\[(]\s*([^,]+?)\s*,\s*([^\])]+?)\s*[\])]$")) {
        let f = fx(&g(&m, 1))?;
        let x = main_var(&f);
        let (a, b) = (ex(&g(&m, 2))?, ex(&g(&m, 3))?);
        let slope = s::simplify(&div(sub(f.subs(&x, &b), f.subs(&x, &a)), sub(b.clone(), a.clone())));
        let (af, bf) = (a.to_f64_const()?, b.to_f64_const()?);
        let roots = solve::solve(&eq(diff(&f, &x), slope.clone()), &x);
        let cs: Vec<Expr> = roots.real().into_iter().filter(|r| r.to_f64_const().is_some_and(|v| v > af && v < bf)).collect();
        let label = format!("mean value theorem for {} on [{}, {}]", ptext(&f), ptext(&a), ptext(&b));
        let mut pods = vec![primary(pod("Result", "Points c with f′(c) = (f(b) − f(a))/(b − a)", if cs.is_empty() { text("none found") } else { table(None, cs.iter().map(|c| vec![math(&eq(sym("c"), c.clone()))]).collect()) }))];
        pods.push(pod("Slope", "Average rate of change", math(&slope)));
        if let Some(p) = plot_pod(&[f.clone()], &x, Some((af, bf))) {
            pods.push(p);
        }
        return answer(label.clone(), label, pods);
    }
    // Riemann sums, trapezoid and Simpson
    if let Some(m) = cap(re!(r"^(?:the\s+)?(left |right |midpoint |left-hand |right-hand |lower |upper )?(riemann sums?|trapezoidal rule|trapezoid rule|trapezium rule|simpson'?s rule|midpoint rule)\s+(?:for|of)\s+(.+?)\s+from\s+(\S+)\s+to\s+(\S+)\s+(?:with|using)\s+(\d+)\s+(?:rectangles|subintervals|intervals|strips|panels|partitions|n)$")) {
        let f = fx(&g(&m, 3))?;
        let x = main_var(&f);
        let (a, b) = (f64_of(&g(&m, 4))?, f64_of(&g(&m, 5))?);
        let nn: usize = g(&m, 6).parse().ok()?;
        if nn == 0 || nn > 100_000 {
            return None;
        }
        let h = (b - a) / nn as f64;
        let fv = |t: f64| eval_at(&f, &x, t);
        let mut table_rows = Vec::new();
        let left: f64 = (0..nn).map(|i| fv(a + i as f64 * h)).sum::<Option<f64>>()? * h;
        let right: f64 = (1..=nn).map(|i| fv(a + i as f64 * h)).sum::<Option<f64>>()? * h;
        let mid: f64 = (0..nn).map(|i| fv(a + (i as f64 + 0.5) * h)).sum::<Option<f64>>()? * h;
        let trap = (left + right) / 2.0;
        let simp = if nn % 2 == 0 { Some((0..=nn).map(|i| fv(a + i as f64 * h).map(|v| v * if i == 0 || i == nn { 1.0 } else if i % 2 == 1 { 4.0 } else { 2.0 })).sum::<Option<f64>>()? * h / 3.0) } else { None };
        let exact = definite_expr(&f, &x, &float(a), &float(b)).and_then(|e| e.to_f64_const());
        let kind = format!("{}{}", g(&m, 1), g(&m, 2)).to_lowercase();
        let (title, value) = if kind.contains("simpson") {
            ("Simpson's rule", simp.ok_or(()).ok()?)
        } else if kind.contains("trapez") {
            ("Trapezoidal rule", trap)
        } else if kind.contains("midpoint") {
            ("Midpoint sum", mid)
        } else if kind.contains("right") || kind.contains("upper") {
            ("Right Riemann sum", right)
        } else {
            ("Left Riemann sum", left)
        };
        table_rows.push(vec![text("left Riemann sum"), text(n(left))]);
        table_rows.push(vec![text("right Riemann sum"), text(n(right))]);
        table_rows.push(vec![text("midpoint sum"), text(n(mid))]);
        table_rows.push(vec![text("trapezoidal rule"), text(n(trap))]);
        if let Some(sv) = simp {
            table_rows.push(vec![text("Simpson's rule"), text(n(sv))]);
        }
        if let Some(ev) = exact {
            table_rows.push(vec![text("exact integral"), text(n(ev))]);
        }
        let label = format!("{title} for {} on [{}, {}] with n = {nn}", ptext(&f), n(a), n(b));
        let mut pods = vec![primary(pod("Result", title, text(n(value))))];
        if let Some(ev) = exact {
            pods.push(pod("Error", "Error", text(format!("{} (exact value {})", n(value - ev), n(ev)))));
        }
        pods.push(pod("Compare", "All approximations", table(None, table_rows)));
        pods.push(pod("Width", "Subinterval width", text(format!("Δx = {}", n(h)))));
        return answer(label.clone(), label, pods);
    }
    // Newton's method
    if let Some(m) = cap(re!(r"^(?:use\s+)?newton(?:'s|-raphson)?(?:\s+method)?\s+(?:for|on|to solve)\s+(.+?)\s+(?:starting at|starting from|with|from|at)\s+(?:x0\s*=\s*|x_0\s*=\s*|x\s*=\s*|an initial guess of\s+|initial guess\s+)?(-?[\d.]+)$")) {
        let body = g(&m, 1);
        let f = match ex(&body)? {
            Rel(RelOp::Eq, a, b) => sub((*a).clone(), (*b).clone()),
            e => e,
        };
        let x = main_var(&f);
        let df = diff(&f, &x);
        let mut xk: f64 = g(&m, 2).parse().ok()?;
        let mut rows = vec![vec![text("0"), text(n(xk)), text(n(eval_at(&f, &x, xk)?))]];
        for k in 1..=12 {
            let d = eval_at(&df, &x, xk)?;
            if d == 0.0 {
                break;
            }
            let next = xk - eval_at(&f, &x, xk)? / d;
            let done = (next - xk).abs() < 1e-14 * next.abs().max(1.0);
            xk = next;
            rows.push(vec![text(k.to_string()), text(fmt_float(xk, 15)), text(n(eval_at(&f, &x, xk)?))]);
            if done {
                break;
            }
        }
        let label = format!("Newton's method for {} = 0", ptext(&f));
        return answer(label.clone(), label, vec![primary(pod("Result", "Root", text(fmt_float(xk, 15)))), pod("Iterations", "Iterations", table(Some(vec!["k", "x_k", "f(x_k)"]), rows)), pod("Formula", "Iteration", text(format!("x_(k+1) = x_k − f(x_k)/f′(x_k),  f′ = {}", ptext(&df))))]);
    }
    // bisection
    if let Some(m) = cap(re!(r"^(?:use\s+)?(?:the\s+)?bisection(?:\s+method)?\s+(?:for|on|to solve)\s+(.+?)\s+(?:on|over|in)\s+[\[(]\s*([^,]+?)\s*,\s*([^\])]+?)\s*[\])]$")) {
        let f = match ex(&g(&m, 1))? {
            Rel(RelOp::Eq, a, b) => sub((*a).clone(), (*b).clone()),
            e => e,
        };
        let x = main_var(&f);
        let (mut a, mut b) = (f64_of(&g(&m, 2))?, f64_of(&g(&m, 3))?);
        let (mut fa, fb) = (eval_at(&f, &x, a)?, eval_at(&f, &x, b)?);
        let label = format!("bisection method for {} = 0 on [{}, {}]", ptext(&f), n(a), n(b));
        if fa * fb > 0.0 {
            return answer(label.clone(), label, vec![primary(pod("Result", "Result", text("f has the same sign at both ends, so bisection cannot start on this interval.")))]);
        }
        let mut rows = Vec::new();
        for k in 1..=40 {
            let mid = (a + b) / 2.0;
            let fm = eval_at(&f, &x, mid)?;
            if k <= 12 {
                rows.push(vec![text(k.to_string()), text(n(a)), text(n(b)), text(n(mid)), text(n(fm))]);
            }
            if fa * fm <= 0.0 {
                b = mid;
            } else {
                a = mid;
                fa = fm;
            }
        }
        return answer(label.clone(), label, vec![primary(pod("Result", "Root", text(fmt_float((a + b) / 2.0, 12)))), pod("Iterations", "First iterations", table(Some(vec!["k", "a", "b", "midpoint", "f(midpoint)"]), rows))]);
    }
    // Euler's method
    if let Some(m) = cap(re!(r"^(?:use\s+)?euler'?s method\s*:?\s*(?:for\s+)?(.+)$")) {
        let parts = split_top(&g(&m, 1));
        let mut rhs = None;
        let (mut x0, mut y0, mut h, mut xend) = (None, None, None, None);
        for p in &parts {
            let p = p.trim();
            if let Some(mm) = re!(r"^y'\s*=\s*(.+)$").captures(p) {
                rhs = ex(&mm[1]);
            } else if let Some(mm) = re!(r"^y\s*\(\s*([^)]+)\)\s*=\s*(.+)$").captures(p) {
                x0 = f64_of(&mm[1]);
                y0 = f64_of(&mm[2]);
            } else if let Some(mm) = re!(r"^(?:h|step(?: size)?|dx)\s*=\s*(.+)$").captures(p) {
                h = f64_of(&mm[1]);
            } else if let Some(mm) = re!(r"^(?:to|until|up to)\s+x\s*=\s*(.+)$").captures(p) {
                xend = f64_of(&mm[1]);
            }
        }
        let (f, x0, y0, h) = (rhs?, x0?, y0?, h?);
        let xend = xend.unwrap_or(x0 + 10.0 * h);
        let steps = ((xend - x0) / h).round() as usize;
        if steps == 0 || steps > 10_000 {
            return None;
        }
        let (xs, ys) = (sym("x"), sym("y"));
        let mut rows = Vec::new();
        let (mut xv, mut yv) = (x0, y0);
        rows.push(vec![text("0"), text(n(xv)), text(n(yv))]);
        for k in 1..=steps {
            let slope = f.subs_many(&[(xs.clone(), float(xv)), (ys.clone(), float(yv))]).to_f64_const()?;
            yv += h * slope;
            xv = x0 + k as f64 * h;
            if k <= 20 || k == steps {
                rows.push(vec![text(k.to_string()), text(n(xv)), text(n(yv))]);
            }
        }
        let label = format!("Euler's method for y′ = {}, y({}) = {}, h = {}", ptext(&f), n(x0), n(y0), n(h));
        return answer(label.clone(), label, vec![primary(pod("Result", &format!("y({}) ≈", n(xend)), text(n(yv)))), pod("Steps", "Steps", table(Some(vec!["k", "x_k", "y_k"]), rows)), pod("Formula", "Iteration", text("y_(k+1) = y_k + h · f(x_k, y_k)"))]);
    }
    None
}

// ---------------------------------------------------------------------------
// applications of integrals
// ---------------------------------------------------------------------------

fn applications(l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    if let Some(m) = cap(re!(r"^(?:the\s+)?volume of (?:the\s+)?solid (?:of revolution\s+)?(?:obtained\s+|formed\s+|generated\s+)?(?:by\s+)?(?:rotating|revolving)?\s*(?:the region under\s+)?(.+?)\s+(?:about|around)\s+(?:the\s+)?(x|y)[- ]axis\s+from\s+(\S+)\s+to\s+(\S+)$"))
        .or_else(|| cap(re!(r"^(?:the\s+)?volume of (?:the\s+)?solid of revolution\s+(.+?)\s+(?:about|around)\s+(?:the\s+)?(x|y)[- ]axis\s+from\s+(\S+)\s+to\s+(\S+)$")))
    {
        let f = fx(&g(&m, 1))?;
        let x = main_var(&f);
        let (a, b) = (ex(&g(&m, 3))?, ex(&g(&m, 4))?);
        let about_x = g(&m, 2) == "x";
        let integrand = if about_x { mul(vec![pi(), square(f.clone())]) } else { mul(vec![int(2), pi(), x.clone(), f.clone()]) };
        let v = definite_expr(&integrand, &x, &a, &b)?;
        let label = format!("volume of revolution of y = {} about the {}-axis, {} from {} to {}", ptext(&f), g(&m, 2), ptext(&x), ptext(&a), ptext(&b));
        let mut pods = value_pods("Volume", None, &s::simplify(&v));
        pods.push(pod("Method", "Method", text(if about_x { "disk method: V = π ∫ f(x)² dx" } else { "shell method: V = 2π ∫ x f(x) dx" })));
        return answer(label.clone(), label, pods);
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:lateral\s+)?surface area of (?:the\s+)?(?:solid\s+|surface\s+)?(?:of\s+)?revolution\s+(?:of\s+)?(.+?)(?:\s+(?:about|around)\s+(?:the\s+)?(x|y)[- ]axis)?\s+from\s+(\S+)\s+to\s+(\S+)$")) {
        let f = fx(&g(&m, 1))?;
        let x = main_var(&f);
        let (a, b) = (ex(&g(&m, 3))?, ex(&g(&m, 4))?);
        let about_y = g(&m, 2) == "y";
        let ds = sqrt(add(vec![one(), square(diff(&f, &x))]));
        let integrand = s::simplify(&mul(vec![int(2), pi(), if about_y { x.clone() } else { f.clone() }, ds]));
        let v = definite_expr(&integrand, &x, &a, &b)?;
        let label = format!("surface area of revolution of y = {} about the {}-axis from {} to {}", ptext(&f), if about_y { "y" } else { "x" }, ptext(&a), ptext(&b));
        let mut pods = value_pods("Surface area", None, &s::simplify(&v));
        pods.push(pod("Integral", "Integral", math(&raw_fn("Integral", vec![integrand, x.clone(), a, b]))));
        return answer(label.clone(), label, pods);
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:centroid|center of mass|centre of mass) of (?:the\s+)?(?:region|area)\s+(?:under|below|bounded by)\s+(.+?)\s+from\s+(\S+)\s+to\s+(\S+)$")) {
        let f = fx(&g(&m, 1))?;
        let x = main_var(&f);
        let (a, b) = (ex(&g(&m, 2))?, ex(&g(&m, 3))?);
        let area = s::simplify(&definite_expr(&f, &x, &a, &b)?);
        if area.is_zero() {
            return None;
        }
        let mx = definite_expr(&mul(vec![x.clone(), f.clone()]), &x, &a, &b)?;
        let my = definite_expr(&div(square(f.clone()), int(2)), &x, &a, &b)?;
        let (xb, yb) = (s::simplify(&div(mx, area.clone())), s::simplify(&div(my, area.clone())));
        let label = format!("centroid of the region under y = {} from {} to {}", ptext(&f), ptext(&a), ptext(&b));
        return answer(label.clone(), label, vec![primary(pod("Result", "Centroid", math(&point(&xb, &yb)))), pod("Area", "Area", math(&area)), pod("Formula", "Formulas", text("x̄ = (1/A) ∫ x f dx,   ȳ = (1/A) ∫ f²/2 dx"))]);
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?area (?:enclosed by|inside|of|bounded by)\s+(?:the\s+)?(?:polar curve\s+|curve\s+|cardioid\s+|rose\s+)?r\s*=\s*(.+?)(?:\s+from\s+(\S+)\s+to\s+(\S+))?$")).or_else(|| cap(re!(r"^(?:the\s+)?area of (?:a|the)\s+polar curve\s+r\s*=\s*(.+?)(?:\s+from\s+(\S+)\s+to\s+(\S+))?$"))) {
        let r = ex(&g(&m, 1).replace("theta", "t").replace('θ', "t"))?;
        let t = sym("t");
        let (lo, hi) = if g(&m, 2).is_empty() {
            // a curve with r(θ + π) = −r(θ) is traced once on [0, π]
            let shifted = s::simplify(&add(vec![r.subs(&t, &add(vec![t.clone(), pi()])), r.clone()]));
            if shifted.is_zero() { (zero(), pi()) } else { (zero(), mul(vec![int(2), pi()])) }
        } else {
            (ex(&g(&m, 2).replace("theta", "t"))?, ex(&g(&m, 3).replace("theta", "t"))?)
        };
        let v = definite_expr(&div(square(r.clone()), int(2)), &t, &lo, &hi)?;
        let label = format!("area enclosed by r = {}", ptext(&r).replace('t', "θ"));
        let mut pods = value_pods("Area", None, &s::simplify(&v));
        pods.push(pod("Formula", "Formula", text(format!("A = ½ ∫ r² dθ over θ from {} to {}", ptext(&lo), ptext(&hi)))));
        return answer(label.clone(), label, pods);
    }
    if l.starts_with("work") && l.contains("pump") && l.contains("tank") {
        let radius = re!(r"radius\s*(?:of\s*)?([\d.]+)").captures(l).and_then(|m| m[1].parse::<f64>().ok());
        let height = re!(r"(?:height|deep|depth|tall)\s*(?:of\s*)?([\d.]+)").captures(l).and_then(|m| m[1].parse::<f64>().ok());
        let mut pods = vec![primary(pod("Formula", "Work to pump a full cylindrical tank out over the top", text("W = ρ g ∫₀^h π r² (h − y) dy = ρ g π r² h² / 2   (ρ = 1000 kg/m³, g = 9.8 m/s²)")))];
        if let (Some(r), Some(h)) = (radius, height) {
            let w = 1000.0 * 9.80665 * std::f64::consts::PI * r * r * h * h / 2.0;
            pods.insert(0, primary(pod("Result", "Work", text(format!("{} J  (radius {} m, height {} m, water)", fmt_float(w, 6), n(r), n(h))))));
            pods[1].primary = false;
        } else {
            pods.push(pod("Hint", "For a number", text("give the size, e.g. \"work to pump water out of a cylindrical tank of radius 2 m and height 5 m\"")));
        }
        return answer("work to pump out a tank", "work to pump water out of a cylindrical tank", pods);
    }
    None
}

// ---------------------------------------------------------------------------
// statistics
// ---------------------------------------------------------------------------

fn kv(l: &str, keys: &[&str]) -> Option<f64> {
    for k in keys {
        let r = Regex::new(&format!(r"(?:^|[\s,(]){}\s*(?:=|of|is)?\s*(-?[\d.]+)", regex::escape(k))).ok()?;
        if let Some(m) = r.captures(l) {
            return m[1].parse().ok();
        }
    }
    None
}

fn binom(nn: u64, k: u64) -> f64 {
    if k > nn {
        return 0.0;
    }
    (0..k).fold(1.0, |acc, i| acc * (nn - i) as f64 / (i + 1) as f64)
}

fn numbers(t: &str) -> Vec<f64> {
    re!(r"-?\d+(?:\.\d+)?").find_iter(t).filter_map(|m| m.as_str().parse().ok()).collect()
}

fn stats(q: &Query, l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    let std = Normal::new(0.0, 1.0).ok()?;
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:probability|percentage|fraction|proportion)\s+(?:that\s+)?(?:a\s+)?(?:normal(?:ly distributed)?\s+)?(?:variable|value|data|observation)?\s*(?:is\s+|lies\s+|falls\s+)?within\s+([\d.]+)\s+standard deviations?(?:\s+of the mean)?$")) {
        let k: f64 = g(&m, 1).parse().ok()?;
        let p = std.cdf(k) - std.cdf(-k);
        let label = format!("probability that a normal variable is within {k} standard deviations of its mean");
        return answer(label.clone(), label, vec![primary(pod("Result", "Probability", text(format!("{}  ({}%)", fmt_float(p, 8), fmt_float(p * 100.0, 6))))), pod("Rule", "68–95–99.7 rule", text("about 68.27% within 1σ, 95.45% within 2σ, 99.73% within 3σ"))]);
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?(expected value|expectation|mean|variance)\s+of\s+(.+?)\s+(?:for|if|when|where)\s+(?:x\s+(?:is\s+)?)?(?:a\s+)?uniform(?:ly distributed)?\s+(?:on|over|in)\s+[\[(]\s*([^,]+?)\s*,\s*([^\])]+?)\s*[\])]$")) {
        let h = ex(&g(&m, 2))?;
        let x = sym("x");
        let (a, b) = (ex(&g(&m, 3))?, ex(&g(&m, 4))?);
        let width = sub(b.clone(), a.clone());
        let e1 = s::simplify(&div(definite_expr(&h, &x, &a, &b)?, width.clone()));
        let label = format!("{} of {} for X uniform on [{}, {}]", g(&m, 1), ptext(&h), ptext(&a), ptext(&b));
        let v = if g(&m, 1) == "variance" {
            let e2 = s::simplify(&div(definite_expr(&square(h.clone()), &x, &a, &b)?, width));
            s::simplify(&sub(e2, square(e1)))
        } else {
            e1
        };
        let mut pods = value_pods("Result", None, &v);
        pods.push(pod("Formula", "Formula", text("E[h(X)] = (1/(b − a)) ∫ₐᵇ h(x) dx")));
        return answer(label.clone(), label, pods);
    }
    // moments of named distributions
    if let Some(m) = cap(re!(r"^(?:the\s+)?(mean|expected value|variance|standard deviation)\s+of\s+(?:a\s+|the\s+)?(binomial|poisson|geometric|exponential|uniform|normal)(?:\s+distribution)?\s*(.*)$")) {
        let rest = g(&m, 3);
        let (mean, var, name) = match g(&m, 2).as_str() {
            "binomial" => {
                let (nn, p) = (kv(&rest, &["n"])?, kv(&rest, &["p"])?);
                (nn * p, nn * p * (1.0 - p), format!("binomial (n = {}, p = {})", n(nn), n(p)))
            }
            "poisson" => {
                let lam = kv(&rest, &["lambda", "λ", "mean", "rate"])?;
                (lam, lam, format!("Poisson (λ = {})", n(lam)))
            }
            "geometric" => {
                let p = kv(&rest, &["p"])?;
                (1.0 / p, (1.0 - p) / (p * p), format!("geometric (p = {}, number of trials)", n(p)))
            }
            "exponential" => {
                let lam = kv(&rest, &["lambda", "λ", "rate"])?;
                (1.0 / lam, 1.0 / (lam * lam), format!("exponential (λ = {})", n(lam)))
            }
            "uniform" => {
                let v = numbers(&rest);
                let (a, b) = (*v.first()?, *v.get(1)?);
                ((a + b) / 2.0, (b - a).powi(2) / 12.0, format!("uniform on [{}, {}]", n(a), n(b)))
            }
            _ => {
                let mu = kv(&rest, &["mean", "mu", "μ"]).unwrap_or(0.0);
                let sd = kv(&rest, &["sd", "sigma", "σ", "standard deviation"]).unwrap_or(1.0);
                (mu, sd * sd, format!("normal (μ = {}, σ = {})", n(mu), n(sd)))
            }
        };
        let what = g(&m, 1);
        let v = match what.as_str() {
            "variance" => var,
            "standard deviation" => var.sqrt(),
            _ => mean,
        };
        let label = format!("{what} of the {name} distribution");
        return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(n(v)))), pod("Moments", "Moments", table(None, vec![vec![text("mean"), text(n(mean))], vec![text("variance"), text(n(var))], vec![text("standard deviation"), text(n(var.sqrt()))]]))]);
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?geometric distribution\s+(.*)$")) {
        let rest = g(&m, 1);
        let p = kv(&rest, &["p"])?;
        let k = re!(r"p\s*\(\s*x\s*=\s*(\d+)\s*\)").captures(&rest.to_lowercase()).and_then(|mm| mm[1].parse::<i32>().ok());
        let mut pods = Vec::new();
        if let Some(k) = k {
            let trials = (1.0 - p).powi(k - 1) * p;
            let failures = (1.0 - p).powi(k) * p;
            pods.push(primary(pod("Result", &format!("P(X = {k}), X = number of trials until the first success"), text(fmt_float(trials, 10)))));
            pods.push(pod("Failures", &format!("P(X = {k}), X = number of failures before the first success"), text(fmt_float(failures, 10))));
        }
        pods.push(pod("Moments", "Moments (number of trials)", table(None, vec![vec![text("mean"), text(n(1.0 / p))], vec![text("variance"), text(n((1.0 - p) / (p * p)))]])));
        if pods.iter().all(|pd| !pd.primary) {
            pods[0].primary = true;
        }
        let label = format!("geometric distribution with p = {}", n(p));
        return answer(label.clone(), label, pods);
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?hypergeometric distribution\s+(.*)$")) {
        let rest = g(&m, 1);
        let (big_n, big_k, nn) = (kv(&rest, &["N"]).or_else(|| kv(&rest, &["population"]))?, kv(&rest, &["K", "successes"])?, kv(&rest, &["n", "draws", "sample"])?);
        let k = kv(&rest, &["k"]).or_else(|| re!(r"p\s*\(\s*x\s*=\s*(\d+)\s*\)").captures(&rest.to_lowercase()).and_then(|mm| mm[1].parse().ok()));
        let (bn, bk, sn) = (big_n as u64, big_k as u64, nn as u64);
        let mean = nn * big_k / big_n;
        let var = nn * (big_k / big_n) * (1.0 - big_k / big_n) * (big_n - nn) / (big_n - 1.0);
        let mut pods = Vec::new();
        if let Some(k) = k {
            let kk = k as u64;
            let p = binom(bk, kk) * binom(bn - bk, sn.saturating_sub(kk)) / binom(bn, sn);
            pods.push(primary(pod("Result", &format!("P(X = {kk})"), text(fmt_float(p, 10)))));
        }
        pods.push(pod("Moments", "Moments", table(None, vec![vec![text("mean"), text(n(mean))], vec![text("variance"), text(n(var))]])));
        if pods.iter().all(|pd| !pd.primary) {
            pods[0].primary = true;
        }
        let label = format!("hypergeometric distribution, N = {bn}, K = {bk}, n = {sn}");
        return answer(label.clone(), label, pods);
    }
    // chi-square goodness of fit
    if let Some(m) = cap(re!(r"^(?:the\s+)?chi[- ]?squared?\s+(?:goodness[- ]of[- ]fit\s+)?test\s+(?:for\s+|of\s+)?(?:observed\s+)?(.+?)\s+(?:vs\.?|versus|against|expected)\s+(?:expected\s+)?(.+)$")) {
        let (o, e) = (numbers(&g(&m, 1)), numbers(&g(&m, 2)));
        if o.len() != e.len() || o.len() < 2 || e.iter().any(|v| *v <= 0.0) {
            return None;
        }
        let chi: f64 = o.iter().zip(&e).map(|(a, b)| (a - b).powi(2) / b).sum();
        let df = (o.len() - 1) as f64;
        let p = 1.0 - ChiSquared::new(df).ok()?.cdf(chi);
        let label = "chi-square goodness-of-fit test".to_string();
        return answer(label.clone(), label, vec![
            primary(pod("Result", "Test statistic", text(format!("χ² = {}  (df = {}, p = {})", n(chi), df, fmt_float(p, 6))))),
            pod("Decision", "Decision at α = 0.05", text(if p < 0.05 { "reject the null hypothesis: the observed counts differ significantly from the expected ones" } else { "do not reject the null hypothesis" })),
            pod("Formula", "Formula", text("χ² = Σ (O − E)² / E")),
        ]);
    }
    // one-sample t test
    if let Some(m) = cap(re!(r"^(?:a\s+)?(?:one[- ]sample\s+)?t[- ]test\s+(?:for\s+|of\s+|on\s+)?(?:the data\s+)?([-\d.,\s{}]+?)\s*,?\s*(?:with\s+)?(?:hypothesi[sz]ed\s+)?(?:mean|mu|μ)\s*(?:=|of)?\s*(-?[\d.]+)$")) {
        let xs = numbers(&g(&m, 1));
        let mu0: f64 = g(&m, 2).parse().ok()?;
        if xs.len() < 2 {
            return None;
        }
        let nn = xs.len() as f64;
        let mean = xs.iter().sum::<f64>() / nn;
        let sd = (xs.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (nn - 1.0)).sqrt();
        let t = (mean - mu0) / (sd / nn.sqrt());
        let dist = StudentsT::new(0.0, 1.0, nn - 1.0).ok()?;
        let p = 2.0 * (1.0 - dist.cdf(t.abs()));
        let label = format!("one-sample t test against μ = {}", n(mu0));
        return answer(label.clone(), label, vec![
            primary(pod("Result", "Test statistic", text(format!("t = {}  (df = {}, two-tailed p = {})", fmt_float(t, 6), nn - 1.0, fmt_float(p, 6))))),
            pod("Decision", "Decision at α = 0.05", text(if p < 0.05 { "reject H₀: the mean differs significantly" } else { "do not reject H₀" })),
            pod("Summary", "Sample", table(None, vec![vec![text("n"), text(n(nn))], vec![text("mean"), text(n(mean))], vec![text("standard deviation"), text(n(sd))], vec![text("standard error"), text(n(sd / nn.sqrt()))]])),
        ]);
    }
    // correlation of two plain lists
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:pearson\s+)?(correlation(?: coefficient)?|covariance)\s+(?:of|between)\s+([-\d.,\s]+?)\s+and\s+([-\d.,\s]+)$")) {
        let (a, b) = (numbers(&g(&m, 2)), numbers(&g(&m, 3)));
        if a.len() != b.len() || a.len() < 3 {
            return None;
        }
        let join = |v: &[f64]| v.iter().map(|x| n(*x)).collect::<Vec<_>>().join(", ");
        return super::nl::again(q, &format!("{} of {{{}}} and {{{}}}", g(&m, 1), join(&a), join(&b)));
    }
    // sample size for a proportion
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:required\s+|minimum\s+)?sample size\s+(?:for|needed for|with)\s+(?:a\s+)?margin of error\s+(?:of\s+)?([\d.]+)\s*(%)?\s+(?:at|with)\s+(\d+(?:\.\d+)?)\s*%\s+confidence$")) {
        let mut e: f64 = g(&m, 1).parse().ok()?;
        if !g(&m, 2).is_empty() || e >= 1.0 {
            e /= 100.0;
        }
        let level: f64 = g(&m, 3).parse::<f64>().ok()? / 100.0;
        let z = std.inverse_cdf(1.0 - (1.0 - level) / 2.0);
        let nn = (z * z * 0.25 / (e * e)).ceil();
        let label = format!("sample size for a margin of error of {} at {}% confidence", n(e), n(level * 100.0));
        return answer(label.clone(), label, vec![primary(pod("Result", "Sample size", text(format!("{} (worst case p = 0.5)", nn)))), pod("Formula", "Formula", text(format!("n = z² p(1 − p) / E²,  z = {}", fmt_float(z, 6))))]);
    }
    let _ = decimal_pod;
    None
}

pub fn handle(q: &Query, l: &str, c: &str) -> Option<Answer> {
    multivariable(l, c).or_else(|| series_q(l, c)).or_else(|| numerics(l, c)).or_else(|| applications(l, c)).or_else(|| stats(q, l, c))
}
