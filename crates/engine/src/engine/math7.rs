//! Still more mathematics on request: fractional derivatives of
//! exponentials, holomorphic functions from a real or imaginary part,
//! iterated limits, the differential equation a function satisfies,
//! quadric surfaces, state-space systems, Julia sets, random permutations,
//! solids solved for a dimension, rotated rectangles and a few reference
//! tables.

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

/// Accept Mathematica-style brackets: "Sinh[x] Sin[y]" → "sinh(x) sin(y)".
fn ex(s: &str) -> Option<Expr> {
    let t = re!(r"([A-Za-z][A-Za-z0-9]*)\[").replace_all(s.trim(), "$1(").replace(']', ")");
    crate::parse::parse(t.to_lowercase().trim()).ok().map(|p| p.expr)
}

fn show(e: &Expr) -> String {
    ptext(e)
}

fn ans(title: &str, input: String, result_title: &str, result: Content, extra: Vec<Pod>) -> Answer {
    let mut pods = vec![pod("Input", "Input interpretation", text(input)), primary(pod("Result", result_title, result))];
    pods.extend(extra);
    Answer::new(title, pods)
}

fn val(e: &Expr) -> Option<f64> {
    crate::eval::eval_f64(e, &[]).filter(|v| v.is_finite())
}

// ---------------------------------------------------------------------------
// fractional derivatives of exponentials
// ---------------------------------------------------------------------------

/// Regularised lower incomplete gamma P(a, z) by its series.
fn gamma_p(a: f64, z: f64) -> f64 {
    if z <= 0.0 {
        return 0.0;
    }
    let mut term = 1.0 / a;
    let mut sum = term;
    for k in 1..500 {
        term *= z / (a + k as f64);
        sum += term;
        if term.abs() < 1e-17 * sum.abs() {
            break;
        }
    }
    (a * z.ln() - z - crate::special::lgamma(a)).exp() * sum
}

fn fractional_exp(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(caputod|caputo|fractional|riemann[- ]liouville)\s*(?:derivative\s+)?(?:of\s+)?(.+?)\s*\{\s*([a-z])\s*,\s*([^}]+?)\s*\}$").captures(l)?;
    let caputo = m[1].starts_with("caputo");
    let e = ex(&m[2])?;
    let t = sym(&m[3]);
    let mu = ex(&m[4])?.as_rat()?.clone();
    let muf = val(&Num(mu.clone()))?;
    if !(muf > 0.0 && muf < 1.0) {
        return None;
    }
    // e^(k t) with a positive constant k
    let arg = match &e {
        Func(f, a) if &**f == "exp" && a.len() == 1 => a[0].clone(),
        Pow(b, p) if matches!(&**b, Const(K::E)) => (**p).clone(),
        _ => return None,
    };
    let k = s::simplify(&diff(&arg, &t));
    if !k.free_symbols().is_empty() || !s::simplify(&arg.subs(&t, &int(0))).is_zero() {
        return None;
    }
    let kf = val(&k)?;
    if kf <= 0.0 {
        return None;
    }
    let a = Num(Rat::from_integer(1.into()) - mu.clone());
    let a_s = show(&a);
    let kt = show(&s::simplify(&mul(vec![k.clone(), t.clone()])));
    let pre = if kf == 1.0 { String::new() } else { format!("{} ", show(&s::simplify(&pow(k.clone(), Num(mu.clone()))))) };
    let mu_s = show(&Num(mu.clone()));
    let caputo_form = format!("{pre}e^({kt}) (1 − Γ({a_s}, {kt})/Γ({a_s}))");
    let tname = show(&t);
    let main = if caputo { caputo_form.clone() } else { format!("{caputo_form} + {tname}^(−{})/Γ({a_s})", show(&Num(mu.clone()))) };
    let at1 = {
        let c = kf.powf(muf) * kf.exp() * gamma_p(1.0 - muf, kf);
        if caputo { c } else { c + 1.0 / crate::special::gamma(1.0 - muf) }
    };
    Some(ans(
        "fractional derivative",
        format!("{} derivative of order {} of {} with respect to {tname}", if caputo { "Caputo" } else { "Riemann–Liouville" }, show(&Num(mu.clone())), show(&e)),
        "Result",
        text(main),
        vec![
            pod("Alternate", "Alternate form", text(if kf == 1.0 && caputo { format!("{tname}^(1 − {mu_s}) E_(1, 2 − {mu_s})({tname}), with the Mittag-Leffler function E") } else { format!("e^({kt}) P({a_s}, {kt}){}, P the regularised lower incomplete gamma function", if pre.is_empty() { String::new() } else { format!(" × {}", pre.trim()) }) })),
            pod("Value", &format!("Value at {tname} = 1"), text(n(at1))),
            pod("Definition", "Definition", text(format!("Caputo: D^μ f(t) = 1/Γ(1 − μ) ∫₀ᵗ f′(τ) (t − τ)^(−μ) dτ, 0 < μ < 1{}", if caputo { "" } else { "; Riemann–Liouville adds f(0) t^(−μ)/Γ(1 − μ)" }))),
        ],
    ))
}

// ---------------------------------------------------------------------------
// holomorphic functions from a real or imaginary part (Milne-Thomson)
// ---------------------------------------------------------------------------

fn holomorphic(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(?:holomorphic|analytic|entire)\s+function\s+(?:with\s+|whose\s+)?(?:the\s+)?(real|imaginary)\s+part\s+(?:is\s+)?(.+)$").captures(l)?;
    let imag = &m[1] == "imaginary";
    let e = ex(&m[2])?;
    let (x, y) = (sym("x"), sym("y"));
    if !e.free_symbols().iter().all(|v| &**v == "x" || &**v == "y") {
        return None;
    }
    // harmonic check
    let lap = s::simplify(&add(vec![diff(&diff(&e, &x), &x), diff(&diff(&e, &y), &y)]));
    let p = Program::compile(&lap, &["x", "y"])?;
    for pt in [[0.31, 0.77], [1.13, -0.42], [-0.6, 0.25]] {
        let v = p.eval(&pt);
        if v.is_finite() && v.abs() > 1e-8 {
            return Some(ans("holomorphic function", format!("holomorphic function with {} part {}", &m[1], show(&e)), "Result", text(format!("none: {} is not harmonic (its Laplacian is {})", show(&e), show(&lap))), vec![]));
        }
    }
    let z = sym("z");
    let at = |g: Expr| s::simplify(&g.subs(&x, &z).subs(&y, &int(0)));
    // f'(z) = A(z) + i B(z)
    let (a, b) = if imag { (at(diff(&e, &y)), at(diff(&e, &x))) } else { (at(diff(&e, &x)), at(neg(diff(&e, &y)))) };
    let ia = crate::calculus::integrate::integrate(&a, &z).map(|r| s::simplify(&r))?;
    let ib = if b.is_zero() { int(0) } else { crate::calculus::integrate::integrate(&b, &z).map(|r| s::simplify(&r))? };
    let f = match (ia.is_zero(), ib.is_zero()) {
        (false, true) => show(&ia),
        (true, false) => format!("i ({})", show(&ib)),
        _ => format!("{} + i ({})", show(&ia), show(&ib)),
    };
    let constant = if imag { "c, with c real" } else { "i c, with c real" };
    let other = if imag { "real part" } else { "imaginary part" };
    Some(ans(
        "holomorphic function",
        format!("holomorphic function with {} part {}", &m[1], show(&e)),
        "Result",
        text(format!("f(z) = {f} + {constant}")),
        vec![
            pod("Derivative", "Derivative from the Cauchy–Riemann equations", text(format!("f′(z) = {}{}", show(&a), if b.is_zero() { String::new() } else { format!(" + i ({})", show(&b)) }))),
            pod("Method", "Method", text(format!("Milne-Thomson: put x = z, y = 0 in f′ = {}, then integrate; the {other} is fixed up to a constant", if imag { "v_y + i v_x" } else { "u_x − i u_y" }))),
        ],
    ))
}

// ---------------------------------------------------------------------------
// iterated limits
// ---------------------------------------------------------------------------

fn iterated_limit(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(?:iterated\s+)?limit\s+(?:of\s+)?(.+?)\s+as\s+([a-z])\s*(?:->|→|approaches|goes to)\s*(\S+?)\s*,?\s+(?:and\s+)?(?:then\s+)?as\s+([a-z])\s*(?:->|→|approaches|goes to)\s*(\S+)$").captures(l)?;
    let e = ex(&m[1])?;
    let (x, y) = (sym(&m[2]), sym(&m[4]));
    let (a, b) = (ex(&m[3].replace("infinity", "oo"))?, ex(&m[5].replace("infinity", "oo"))?);
    use crate::calculus::limit::{limit, Dir};
    let inner = limit(&e, &x, &a, Dir::Both)?;
    let outer = limit(&inner, &y, &b, Dir::Both);
    let swapped = limit(&e, &y, &b, Dir::Both).and_then(|g| limit(&g, &x, &a, Dir::Both));
    let res = outer.as_ref().map(show).unwrap_or_else(|| "does not exist".into());
    let mut extra = vec![pod("Inner", &format!("Inner limit ({} → {})", show(&x), show(&a)), text(show(&inner)))];
    if let Some(sw) = &swapped {
        let same = outer.as_ref().map_or(false, |o| show(o) == show(sw));
        extra.push(pod("Other", "Limits taken in the other order", text(format!("{}{}", show(sw), if same { "" } else { " — the iterated limits differ, so the double limit does not exist" }))));
    }
    Some(ans("iterated limit", format!("lim_({}→{}) lim_({}→{}) {}", show(&y), show(&b), show(&x), show(&a), show(&e)), "Result", text(res), extra))
}

// ---------------------------------------------------------------------------
// the differential equation a function satisfies
// ---------------------------------------------------------------------------

fn ode_for(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(?:linear\s+)?(?:differential equations?|odes?)\s+(?:for|of|satisfied by|obeyed by)?\s*(.+)$").captures(l)?;
    let body = m[1].trim();
    // Bessel functions J_n(x)
    if let Some(b) = re!(r"^(?:j_?|besselj\s*\(?\s*)(\d+)\s*(?:\(\s*x\s*\)|,\s*x\s*\))$").captures(body) {
        let k: i64 = b[1].parse().ok()?;
        return Some(ans("differential equation", format!("differential equation satisfied by J_{k}(x)"), "Result", text(format!("x² y″ + x y′ + (x² − {}) y = 0", k * k)), vec![pod("Name", "Name", text(format!("Bessel's equation of order {k}; the second solution is Y_{k}(x)")))]));
    }
    let e = ex(body)?;
    let fs = e.free_symbols();
    if fs.len() != 1 {
        return None;
    }
    let x = sym(&fs.into_iter().next()?);
    let Sym(name) = &x else { return None };
    let mut ders = vec![e.clone()];
    for _ in 0..5 {
        let d = s::simplify(&diff(ders.last()?, &x));
        ders.push(d);
    }
    let progs: Option<Vec<Program>> = ders.iter().map(|d| Program::compile(d, &[name])).collect();
    let progs = progs?;
    let pts = [0.37, 0.83, 1.21, -0.54, 0.11, 1.67, -1.09];
    for k in 1..=4usize {
        // f^(k) = Σ c_j f^(j), j < k: solve with k points, check the rest
        let mut a = vec![vec![0.0; k]; k];
        let mut rhs = vec![0.0; k];
        for (r, &p) in pts.iter().take(k).enumerate() {
            for j in 0..k {
                a[r][j] = progs[j].eval1(p);
            }
            rhs[r] = progs[k].eval1(p);
        }
        let Some(c) = crate::numeric::solve_linear(a, rhs) else { continue };
        let ok = pts.iter().skip(k).all(|&p| {
            let lhs = progs[k].eval1(p);
            let r: f64 = (0..k).map(|j| c[j] * progs[j].eval1(p)).sum();
            (lhs - r).abs() < 1e-7 * (1.0 + lhs.abs())
        });
        if !ok {
            continue;
        }
        // rational coefficients
        let rc: Option<Vec<Rat>> = c.iter().map(|v| {
            for d in 1..=12i64 {
                let r = (v * d as f64).round();
                if (v * d as f64 - r).abs() < 1e-7 {
                    return Some(Rat::new((r as i64).into(), d.into()));
                }
            }
            None
        }).collect();
        let rc = rc?;
        let mark = |j: usize| match j {
            0 => "y".to_string(),
            1 => "y′".to_string(),
            2 => "y″".to_string(),
            3 => "y‴".to_string(),
            j => format!("y^({j})"),
        };
        let mut terms = vec![mark(k)];
        for j in (0..k).rev() {
            let c = -rc[j].clone();
            if c == Rat::from_integer(0.into()) {
                continue;
            }
            let neg_ = c < Rat::from_integer(0.into());
            let mag = if neg_ { -c } else { c };
            let coef = if mag == Rat::from_integer(1.into()) { String::new() } else { format!("{} ", show(&Num(mag))) };
            terms.push(format!("{} {coef}{}", if neg_ { "−" } else { "+" }, mark(j)));
        }
        let eq = format!("{} = 0", terms.join(" "));
        // characteristic polynomial
        let mut charp = format!("r^{k}");
        for j in (0..k).rev() {
            let c = -rc[j].clone();
            if c != Rat::from_integer(0.into()) {
                let neg_ = c < Rat::from_integer(0.into());
                let mag = if neg_ { -c } else { c };
                let coef = if mag == Rat::from_integer(1.into()) && j > 0 { String::new() } else { show(&Num(mag)) };
                charp.push_str(&format!(" {} {coef}{}", if neg_ { "−" } else { "+" }, match j { 0 => String::new(), 1 => "r".into(), _ => format!("r^{j}") }));
            }
        }
        return Some(ans(
            "differential equation",
            format!("linear differential equation with constant coefficients satisfied by {}", show(&e)),
            "Result",
            text(eq.replace("y", "y").replace("  ", " ")),
            vec![pod("Characteristic", "Characteristic polynomial", text(format!("{charp} = 0"))), pod("Order", "Order", text(format!("{k} (the least order possible with constant coefficients)")))],
        ));
    }
    None
}

// ---------------------------------------------------------------------------
// quadric surfaces
// ---------------------------------------------------------------------------

fn quadric(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:plot|graph|3d plot|classify|identify|what (?:kind of )?surface is|quadric surface|surface)\s+(?:the\s+)?(?:surface\s+)?(.+?)\s*=\s*(.+)$").captures(l)?;
    let lhs = ex(&m[1])?;
    let rhs = ex(&m[2])?;
    let f = s::simplify(&s::expand(&sub(lhs, rhs)));
    let (x, y, z) = (sym("x"), sym("y"), sym("z"));
    let fs = f.free_symbols();
    if !fs.iter().all(|v| matches!(&**v, "x" | "y" | "z")) || fs.len() != 3 {
        return None;
    }
    let p = Program::compile(&f, &["x", "y", "z"])?;
    // quadratic form coefficients from finite differences; reject non-quadrics
    let at = |a: f64, b: f64, c: f64| p.eval(&[a, b, c]);
    let c0 = at(0.0, 0.0, 0.0);
    let coef2 = |i: usize| {
        let mut e1 = [0.0; 3];
        e1[i] = 1.0;
        let mut e2 = [0.0; 3];
        e2[i] = -1.0;
        ((at(e1[0], e1[1], e1[2]) + at(e2[0], e2[1], e2[2])) / 2.0 - c0, (at(e1[0], e1[1], e1[2]) - at(e2[0], e2[1], e2[2])) / 2.0)
    };
    let (q, lin): (Vec<f64>, Vec<f64>) = (0..3).map(coef2).unzip();
    let cross = |i: usize, j: usize| {
        let mut v = [0.0; 3];
        v[i] = 1.0;
        v[j] = 1.0;
        at(v[0], v[1], v[2]) - c0 - q[i] - q[j] - lin[i] - lin[j]
    };
    if cross(0, 1).abs() > 1e-9 || cross(0, 2).abs() > 1e-9 || cross(1, 2).abs() > 1e-9 {
        return None;
    }
    // verify it is exactly quadratic
    let test = [1.7, -0.6, 2.3];
    let model = c0 + (0..3).map(|i| q[i] * test[i] * test[i] + lin[i] * test[i]).sum::<f64>();
    if (model - at(test[0], test[1], test[2])).abs() > 1e-8 * (1.0 + model.abs()) {
        return None;
    }
    let _ = (&x, &y, &z);
    // complete squares: q_i (x_i + l_i/(2 q_i))² ...; a variable with q = 0 but l ≠ 0 is linear
    let mut k = -c0;
    let mut sq = Vec::new();
    let mut linear = Vec::new();
    for i in 0..3 {
        if q[i].abs() > 1e-12 {
            k += lin[i] * lin[i] / (4.0 * q[i]);
            sq.push(q[i]);
        } else if lin[i].abs() > 1e-12 {
            linear.push(i);
        }
    }
    let pos = sq.iter().filter(|v| **v > 0.0).count();
    let negc = sq.len() - pos;
    let kind = match (sq.len(), linear.len()) {
        (3, 0) => {
            if k.abs() < 1e-12 {
                if pos == 3 || negc == 3 { "a single point" } else { "an elliptic cone" }
            } else {
                let (p2, n2) = if k > 0.0 { (pos, negc) } else { (negc, pos) };
                match (p2, n2) {
                    (3, 0) => {
                        if (sq[0] - sq[1]).abs() < 1e-12 && (sq[1] - sq[2]).abs() < 1e-12 { "a sphere" } else { "an ellipsoid" }
                    }
                    (2, 1) => "a hyperboloid of one sheet",
                    (1, 2) => "a hyperboloid of two sheets",
                    _ => "the empty set (no real points)",
                }
            }
        }
        (2, 1) => if pos == 2 || negc == 2 { "an elliptic paraboloid" } else { "a hyperbolic paraboloid (saddle)" },
        (2, 0) => {
            if k.abs() < 1e-12 {
                if pos == 2 || negc == 2 { "a line" } else { "a pair of intersecting planes" }
            } else if (k > 0.0 && pos == 2) || (k < 0.0 && negc == 2) {
                "an elliptic cylinder"
            } else if pos == 1 {
                "a hyperbolic cylinder"
            } else {
                "the empty set (no real points)"
            }
        }
        (1, 1) | (1, 2) => "a parabolic cylinder",
        _ => return None,
    };
    let eq = format!("{} = {}", show(&s::simplify(&ex(&m[1])?)), show(&s::simplify(&ex(&m[2])?)));
    Some(ans(
        "quadric surface",
        format!("surface {eq}"),
        "Surface",
        text(format!("{eq} is {kind}")),
        vec![pod("Classification", "Classification", text(format!("{} squared term{}{}; constant side {}", sq.len(), if sq.len() == 1 { "" } else { "s" }, if linear.is_empty() { String::new() } else { format!(" and {} linear term{}", linear.len(), if linear.len() == 1 { "" } else { "s" }) }, n(k))))],
    ))
}

// ---------------------------------------------------------------------------
// state-space systems
// ---------------------------------------------------------------------------

fn parse_matrix(s: &str) -> Option<Vec<Vec<Expr>>> {
    let s = s.trim();
    let inner = s.strip_prefix('{')?.strip_suffix('}')?;
    let rows: Vec<&str> = re!(r"\}\s*,\s*\{").split(inner.trim().trim_start_matches('{').trim_end_matches('}')).collect();
    let m: Option<Vec<Vec<Expr>>> = rows.iter().map(|r| r.split(',').map(|c| ex(c)).collect()).collect();
    let m = m?;
    let w = m.first()?.len();
    (m.iter().all(|r| r.len() == w)).then_some(m)
}

fn det(m: &[Vec<Expr>]) -> Expr {
    let k = m.len();
    if k == 1 {
        return m[0][0].clone();
    }
    let mut terms = Vec::new();
    for j in 0..k {
        if m[0][j].is_zero() {
            continue;
        }
        let minor: Vec<Vec<Expr>> = m[1..].iter().map(|r| r.iter().enumerate().filter(|(c, _)| *c != j).map(|(_, v)| v.clone()).collect()).collect();
        let t = mul(vec![m[0][j].clone(), det(&minor)]);
        terms.push(if j % 2 == 0 { t } else { neg(t) });
    }
    s::expand(&add(terms))
}

fn state_space(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:state(?:\s+matrix)?\s+)(\{.+?\}\})\s*,\s*input(?:\s+matrix)?\s+(\{.+?\}\})\s*,\s*output(?:\s+matrix)?\s+(\{.+?\}\})$").captures(l)?;
    let a = parse_matrix(&m[1])?;
    let b = parse_matrix(&m[2])?;
    let c = parse_matrix(&m[3])?;
    let k = a.len();
    if k == 0 || k > 5 || a.iter().any(|r| r.len() != k) || b.len() != k || b[0].len() != 1 || c.len() != 1 || c[0].len() != k {
        return None;
    }
    let sv = sym("s");
    let si: Vec<Vec<Expr>> = (0..k).map(|i| (0..k).map(|j| s::simplify(&sub(if i == j { sv.clone() } else { int(0) }, a[i][j].clone()))).collect()).collect();
    let den = s::simplify(&det(&si));
    // numerator C adj(sI − A) B: adj_ij = (−1)^(i+j) M_ji
    let mut num_terms = Vec::new();
    for i in 0..k {
        for j in 0..k {
            if c[0][i].is_zero() || b[j][0].is_zero() {
                continue;
            }
            let minor: Vec<Vec<Expr>> = si.iter().enumerate().filter(|(r, _)| *r != j).map(|(_, row)| row.iter().enumerate().filter(|(cc, _)| *cc != i).map(|(_, v)| v.clone()).collect()).collect();
            let cof = if k == 1 { int(1) } else { det(&minor) };
            let t = mul(vec![c[0][i].clone(), cof, b[j][0].clone()]);
            num_terms.push(if (i + j) % 2 == 0 { t } else { neg(t) });
        }
    }
    let num = s::simplify(&s::expand(&add(num_terms)));
    let h = s::simplify(&mul(vec![num.clone(), pow(den.clone(), int(-1))]));
    // numeric controllability and observability
    let af: Option<Vec<Vec<f64>>> = a.iter().map(|r| r.iter().map(val).collect()).collect();
    let bf: Option<Vec<f64>> = b.iter().map(|r| val(&r[0])).collect();
    let cf: Option<Vec<f64>> = c[0].iter().map(val).collect();
    let mut extra = vec![pod("Characteristic", "Characteristic polynomial", text(format!("det(s I − A) = {}", show(&den))))];
    if let (Some(af), Some(bf), Some(cf)) = (af, bf, cf) {
        let matvec = |m: &Vec<Vec<f64>>, v: &Vec<f64>| -> Vec<f64> { m.iter().map(|r| r.iter().zip(v).map(|(x, y)| x * y).sum()).collect() };
        let vecmat = |v: &Vec<f64>, m: &Vec<Vec<f64>>| -> Vec<f64> { (0..k).map(|j| (0..k).map(|i| v[i] * m[i][j]).sum()).collect() };
        let mut ctrl = vec![bf.clone()];
        let mut obs = vec![cf.clone()];
        for _ in 1..k {
            let nb = matvec(&af, ctrl.last()?);
            ctrl.push(nb);
            let nc = vecmat(obs.last()?, &af);
            obs.push(nc);
        }
        let rank = |rows: &Vec<Vec<f64>>| -> usize {
            let mut m = rows.clone();
            let mut r = 0;
            for col in 0..k {
                let Some(piv) = (r..m.len()).max_by(|&i, &j| m[i][col].abs().partial_cmp(&m[j][col].abs()).unwrap()) else { break };
                if m[piv][col].abs() < 1e-10 {
                    continue;
                }
                m.swap(r, piv);
                for i in 0..m.len() {
                    if i != r {
                        let f = m[i][col] / m[r][col];
                        for j in 0..k {
                            let t = m[r][j];
                            m[i][j] -= f * t;
                        }
                    }
                }
                r += 1;
            }
            r
        };
        let (rc, ro) = (rank(&ctrl), rank(&obs));
        extra.push(pod("Properties", "Properties", table(None, vec![
            vec![text("controllable"), text(if rc == k { "yes".to_string() } else { format!("no (rank {rc} of {k})") })],
            vec![text("observable"), text(if ro == k { "yes".to_string() } else { format!("no (rank {ro} of {k})") })],
        ])));
        if let Some(dp) = crate::poly::to_qpoly(&den, &sv) {
            let poles = dp.roots_c();
            let stable = poles.iter().all(|p| p.re < -1e-12);
            let list = poles.iter().map(|z| if z.im.abs() < 1e-9 { n(z.re) } else { format!("{} {} {}i", n(z.re), if z.im < 0.0 { "−" } else { "+" }, n(z.im.abs())) }).collect::<Vec<_>>().join(", ");
            extra.push(pod("Poles", "Poles (eigenvalues of A)", text(format!("{list} — {}", if stable { "asymptotically stable" } else { "not asymptotically stable" }))));
        }
    }
    Some(ans("state-space model", format!("state-space model with A = {}, B = {}, C = {}", &m[1], &m[2], &m[3]), "Transfer function", text(format!("H(s) = {}", show(&h))), extra))
}

// ---------------------------------------------------------------------------
// Julia sets
// ---------------------------------------------------------------------------

fn parse_complex(s: &str) -> Option<(f64, f64)> {
    let s = s.replace(' ', "").replace('−', "-");
    if let Ok(r) = s.parse::<f64>() {
        return Some((r, 0.0));
    }
    if let Some(m) = re!(r"^([+-]?\d*\.?\d*)\*?i$").captures(&s) {
        let im = match &m[1] {
            "" | "+" => 1.0,
            "-" => -1.0,
            t => t.parse().ok()?,
        };
        return Some((0.0, im));
    }
    let m = re!(r"^([+-]?\d*\.?\d+)([+-]\d*\.?\d*)\*?i$").captures(&s)?;
    let im = match &m[2] {
        "+" => 1.0,
        "-" => -1.0,
        t => t.parse().ok()?,
    };
    Some((m[1].parse().ok()?, im))
}

fn julia(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(?:filled\s+)?julia set\s+(?:for\s+|of\s+)?(?:c\s*=\s*)?(.+)$").captures(l)?;
    let (cr, ci) = parse_complex(&m[1])?;
    // the critical orbit decides connectedness
    let (mut zr, mut zi) = (0.0f64, 0.0f64);
    let mut escaped = None;
    let mut orbit = Vec::new();
    for k in 0..5000 {
        let t = zr * zr - zi * zi + cr;
        zi = 2.0 * zr * zi + ci;
        zr = t;
        if zr * zr + zi * zi > 4.0 {
            escaped = Some(k + 1);
            break;
        }
        if k >= 4000 {
            orbit.push((zr, zi));
        }
    }
    let connected = escaped.is_none();
    // period of the attracting cycle, if the orbit has settled
    let period = if connected {
        let last = *orbit.last()?;
        (1..=64).find(|&p| {
            let o = orbit[orbit.len() - 1 - p];
            (o.0 - last.0).hypot(o.1 - last.1) < 1e-9
        })
    } else {
        None
    };
    let disc = num_complex::Complex64::new(1.0 - 4.0 * cr, -4.0 * ci).sqrt();
    let f1 = (num_complex::Complex64::new(1.0, 0.0) + disc) / 2.0;
    let f2 = (num_complex::Complex64::new(1.0, 0.0) - disc) / 2.0;
    let cz = |z: num_complex::Complex64| {
        if z.im.abs() < 1e-12 {
            n(z.re)
        } else if z.re.abs() < 1e-12 {
            format!("{}i", n(z.im))
        } else {
            format!("{} {} {}i", n(z.re), if z.im < 0.0 { "−" } else { "+" }, n(z.im.abs()))
        }
    };
    let mut rows = vec![
        vec![text("map"), text(format!("z ↦ z² + c, c = {}", cz(num_complex::Complex64::new(cr, ci))))],
        vec![text("c in the Mandelbrot set"), text(if connected { "yes" } else { "no" })],
        vec![text("fixed points"), text(format!("{}  (|f′| = {}),  {}  (|f′| = {})", cz(f1), n(2.0 * f1.norm()), cz(f2), n(2.0 * f2.norm())))],
    ];
    if let Some(p) = period {
        rows.push(vec![text("attracting cycle"), text(format!("period {p}"))]);
    }
    if let Some(k) = escaped {
        rows.push(vec![text("critical orbit"), text(format!("escapes after {k} iterations"))]);
    }
    Some(ans(
        "Julia set",
        format!("Julia set of z² + c, c = {}", cz(num_complex::Complex64::new(cr, ci))),
        "Topology",
        text(if connected { "connected (c lies in the Mandelbrot set)".to_string() } else { "totally disconnected — a Cantor set (\"Fatou dust\"), since the critical orbit escapes".to_string() }),
        vec![pod("Properties", "Properties", table(None, rows))],
    ))
}

// ---------------------------------------------------------------------------
// random permutations
// ---------------------------------------------------------------------------

fn random_permutation(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:a\s+)?random permutation\s+(?:on|of)\s+(\d+)(?:\s+(?:elements|objects|items|letters|symbols|points))?$").captures(l)?;
    let k: usize = m[1].parse().ok()?;
    if !(1..=1000).contains(&k) {
        return None;
    }
    let mut seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x9e37_79b9) | 1;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut p: Vec<usize> = (1..=k).collect();
    for i in (1..k).rev() {
        let j = (next() % (i as u64 + 1)) as usize;
        p.swap(i, j);
    }
    // cycles
    let mut seen = vec![false; k];
    let mut cycles: Vec<Vec<usize>> = Vec::new();
    for i in 0..k {
        if !seen[i] {
            let mut c = vec![];
            let mut j = i;
            while !seen[j] {
                seen[j] = true;
                c.push(j + 1);
                j = p[j] - 1;
            }
            cycles.push(c);
        }
    }
    let gcd = |mut a: usize, mut b: usize| {
        while b != 0 {
            let t = a % b;
            a = b;
            b = t;
        }
        a
    };
    let order = cycles.iter().fold(1usize, |acc, c| acc / gcd(acc, c.len()) * c.len());
    let parity = cycles.iter().map(|c| c.len() - 1).sum::<usize>() % 2;
    let cyc = cycles.iter().filter(|c| c.len() > 1).map(|c| format!("({})", c.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" "))).collect::<Vec<_>>().join("");
    Some(ans(
        "random permutation",
        format!("random permutation of {{1, …, {k}}}"),
        "Result",
        text(format!("[{}]", p.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "))),
        vec![pod("Properties", "Properties", table(None, vec![
            vec![text("cycle notation"), text(if cyc.is_empty() { "identity".into() } else { cyc })],
            vec![text("cycle lengths"), text(cycles.iter().map(|c| c.len().to_string()).collect::<Vec<_>>().join(", "))],
            vec![text("order"), text(order.to_string())],
            vec![text("parity"), text(if parity == 0 { "even" } else { "odd" })],
            vec![text("fixed points"), text(cycles.iter().filter(|c| c.len() == 1).count().to_string())],
        ]))],
    ))
}

// ---------------------------------------------------------------------------
// solids solved for a dimension; rotated rectangles
// ---------------------------------------------------------------------------

fn solid_dimension(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:what is\s+)?(?:the\s+)?(radius|height|diameter)\s+of\s+(?:a|an|the)\s+(cone|cylinder|sphere|ball)\s+with\s+(?:a\s+)?volume\s+(?:of\s+)?(\S+?)(?:\s+and\s+(?:a\s+)?(height|radius)\s+(?:of\s+)?(\S+))?$").captures(l)?;
    let want = &m[1];
    let solid = &m[2];
    let v = ex(&m[3])?;
    let pi_ = pi();
    let res = match (solid, want, m.get(4).map(|g| g.as_str())) {
        ("cone", "radius", Some("height")) => pow(mul(vec![int(3), v.clone(), pow(mul(vec![pi_.clone(), ex(&m[5])?]), int(-1))]), rat(1, 2)),
        ("cylinder", "radius", Some("height")) => pow(mul(vec![v.clone(), pow(mul(vec![pi_.clone(), ex(&m[5])?]), int(-1))]), rat(1, 2)),
        ("cone", "height", Some("radius")) => mul(vec![int(3), v.clone(), pow(mul(vec![pi_.clone(), pow(ex(&m[5])?, int(2))]), int(-1))]),
        ("cylinder", "height", Some("radius")) => mul(vec![v.clone(), pow(mul(vec![pi_.clone(), pow(ex(&m[5])?, int(2))]), int(-1))]),
        ("sphere" | "ball", "radius", None) => pow(mul(vec![int(3), v.clone(), pow(mul(vec![int(4), pi_.clone()]), int(-1))]), rat(1, 3)),
        ("sphere" | "ball", "diameter", None) => mul(vec![int(2), pow(mul(vec![int(3), v.clone(), pow(mul(vec![int(4), pi_.clone()]), int(-1))]), rat(1, 3))]),
        _ => return None,
    };
    let r = s::simplify(&res);
    let formula = match solid {
        "cone" => "V = π r² h / 3",
        "cylinder" => "V = π r² h",
        _ => "V = 4 π r³ / 3",
    };
    let mut extra = vec![pod("Formula", "Formula", text(formula))];
    if let Some(x) = val(&r) {
        extra.push(pod("Decimal", "Decimal approximation", text(n(x))));
    }
    Some(ans("solid dimension", format!("{want} of a {solid} with volume {}{}", show(&v), m.get(5).map(|h| format!(" and {} {}", &m[4], h.as_str())).unwrap_or_default()), &format!("{}{}", want[..1].to_uppercase(), &want[1..]), text(show(&r)), extra))
}

fn rotated_rectangle(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:a\s+)?(\d+(?:\.\d+)?)\s*(?:by|x|×)\s*(\d+(?:\.\d+)?)\s+rectangle\s+rotated\s+(?:by\s+)?(-?\d+(?:\.\d+)?)\s*(?:degrees|°|deg)(?:\s+(counterclockwise|clockwise))?$").captures(l)?;
    let (w, h) = (ex(&m[1])?, ex(&m[2])?);
    let mut deg = m[3].to_string();
    if m.get(4).map_or(false, |d| d.as_str() == "clockwise") {
        deg = format!("-{deg}");
    }
    let th = ex(&format!("({deg})*pi/180"))?;
    let (c, sn) = (s::simplify(&func("cos", vec![th.clone()])), s::simplify(&func("sin", vec![th])));
    // corners of [0,w]×[0,h] about the origin
    let corners = [(int(0), int(0)), (w.clone(), int(0)), (w.clone(), h.clone()), (int(0), h.clone())];
    let rot: Vec<String> = corners.iter().map(|(x, y)| {
        let xr = s::simplify(&s::expand(&sub(mul(vec![x.clone(), c.clone()]), mul(vec![y.clone(), sn.clone()]))));
        let yr = s::simplify(&s::expand(&add(vec![mul(vec![x.clone(), sn.clone()]), mul(vec![y.clone(), c.clone()])])));
        format!("({}, {})", show(&xr), show(&yr))
    }).collect();
    let abs_c = s::simplify(&func("abs", vec![c.clone()]));
    let abs_s = s::simplify(&func("abs", vec![sn.clone()]));
    let bw = s::simplify(&s::expand(&add(vec![mul(vec![w.clone(), abs_c.clone()]), mul(vec![h.clone(), abs_s.clone()])])));
    let bh = s::simplify(&s::expand(&add(vec![mul(vec![w.clone(), abs_s]), mul(vec![h.clone(), abs_c])])));
    Some(ans(
        "rotated rectangle",
        format!("{} × {} rectangle rotated {}° {}", show(&w), show(&h), m[3].trim_start_matches('-'), if deg.starts_with('-') { "clockwise" } else { "counterclockwise" }),
        "Vertices (rotated about the corner at the origin)",
        text(rot.join(", ")),
        vec![
            pod("Bounding", "Bounding box", text(format!("{} wide × {} tall (≈ {} × {})", show(&bw), show(&bh), n(val(&bw).unwrap_or(f64::NAN)), n(val(&bh).unwrap_or(f64::NAN))))),
            pod("Invariants", "Unchanged by the rotation", text(format!("area {}, perimeter {}, diagonal {}", show(&s::simplify(&mul(vec![w.clone(), h.clone()]))), show(&s::simplify(&mul(vec![int(2), add(vec![w.clone(), h.clone()])]))), show(&s::simplify(&pow(add(vec![pow(w.clone(), int(2)), pow(h.clone(), int(2))]), rat(1, 2))))))),
        ],
    ))
}

// ---------------------------------------------------------------------------
// reference tables and small facts
// ---------------------------------------------------------------------------

fn reference(l: &str) -> Option<Answer> {
    let (title, main, rows): (&str, &str, Vec<(&str, &str)>) = match l {
        "euler phi" | "euler's phi function" | "euler phi function" | "euler totient" | "euler's totient function" | "totient function" | "euler totient function" | "eulerphi" => (
            "Euler's totient function",
            "φ(n) counts the integers 1 ≤ k ≤ n with gcd(k, n) = 1",
            vec![("formula", "φ(n) = n ∏_(p | n) (1 − 1/p)"), ("multiplicative", "φ(m n) = φ(m) φ(n) when gcd(m, n) = 1"), ("prime powers", "φ(pᵏ) = pᵏ − pᵏ⁻¹"), ("Euler's theorem", "a^φ(n) ≡ 1 (mod n) when gcd(a, n) = 1"), ("sum over divisors", "Σ_(d | n) φ(d) = n"), ("first values", "1, 1, 2, 2, 4, 2, 6, 4, 6, 4, 10, 4, 12, 6, 8, 8, 16, 6, 18, 8 (OEIS A000010)")],
        ),
        "law of haversines" | "haversine formula" | "haversine law" | "spherical law of haversines" => (
            "law of haversines",
            "hav(c) = hav(a − b) + sin a sin b hav(C)",
            vec![("haversine", "hav θ = sin²(θ/2) = (1 − cos θ)/2"), ("meaning", "a, b, c are the sides (arcs) of a spherical triangle and C the angle opposite c"), ("great-circle distance", "hav(d/R) = hav(φ₂ − φ₁) + cos φ₁ cos φ₂ hav(λ₂ − λ₁)"), ("planar limit", "for small triangles it reduces to the law of cosines")],
        ),
        "closed-form values of tan(x)" | "closed form values of tan(x)" | "exact values of tan" | "exact values of tan(x)" | "special values of tan" | "tan special values" | "closed-form values of tan" => (
            "exact values of tan x",
            "tan 0 = 0, tan π/6 = 1/√3, tan π/4 = 1, tan π/3 = √3, tan π/2 undefined",
            vec![("π/12 (15°)", "2 − √3"), ("π/10 (18°)", "√(1 − 2/√5)"), ("π/8 (22.5°)", "√2 − 1"), ("π/6 (30°)", "1/√3 = √3/3"), ("π/5 (36°)", "√(5 − 2√5)"), ("π/4 (45°)", "1"), ("3π/10 (54°)", "√(1 + 2/√5)"), ("π/3 (60°)", "√3"), ("3π/8 (67.5°)", "√2 + 1"), ("5π/12 (75°)", "2 + √3"), ("π/2 (90°)", "undefined (pole)")],
        ),
        "parallelogram with all right angles" | "a parallelogram with all right angles" | "parallelogram with four right angles" => (
            "parallelogram with all right angles",
            "a rectangle",
            vec![("also equal sides", "a square"), ("properties", "opposite sides equal and parallel, diagonals equal and bisect each other")],
        ),
        "basketball shape" | "shape of a basketball" | "what shape is a basketball" => (
            "basketball",
            "a sphere",
            vec![("size 7 (men's) circumference", "29.5 in (74.9 cm); diameter about 9.4 in (23.9 cm)"), ("size 6 (women's) circumference", "28.5 in (72.4 cm)"), ("surface area (size 7)", "about 277 in² (1790 cm²)"), ("volume (size 7)", "about 434 in³ (7.1 L)")],
        ),
        _ => return None,
    };
    Some(Answer::new(title, vec![pod("Input", "Input interpretation", text(title)), primary(pod("Result", "Result", text(main))), pod("Details", "Details", table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect()))]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    fractional_exp(l)
        .or_else(|| holomorphic(l))
        .or_else(|| iterated_limit(l))
        .or_else(|| ode_for(l))
        .or_else(|| quadric(l))
        .or_else(|| state_space(l))
        .or_else(|| julia(l))
        .or_else(|| random_permutation(l))
        .or_else(|| solid_dimension(l))
        .or_else(|| rotated_rectangle(l))
        .or_else(|| reference(l))
}
