//! Numerical methods and vector questions in free phrasing: ODE steppers
//! ("use Euler method y' = -2xy, y(1) = 2, from 1 to 5", backward Euler,
//! midpoint, Heun, Runge–Kutta, implicit midpoint, Bogacki–Shampine,
//! Runge–Kutta–Fehlberg), root finding ("using Newton's method solve
//! x cos x = 0", secant, bisection), quadrature rules ("5 interval
//! trapezoidal rule integrate sin x cos x on [0,4]", Boole's rule),
//! vectors in i, j, k notation, linear independence, orthogonality, row and
//! column spaces, projections, spherical coordinates, forms of complex
//! numbers and harmonic conjugates.

use super::*;
use crate::calculus::diff::diff;
use crate::eval::Program;
use crate::expr::Expr::*;
use crate::expr::*;
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

fn ex(s: &str) -> Option<Expr> {
    let t = re!(r"\b([A-Za-z]+)\[([^\[\]]*)\]").replace_all(s.trim(), "$1($2)").to_string();
    crate::parse::parse(t.trim()).ok().map(|p| p.expr)
}

fn implicit(t: &str) -> Option<Expr> {
    Some(match ex(t)? {
        Rel(RelOp::Eq, a, b) => s::simplify(&sub(*a, *b)),
        Rel(..) => return None,
        e => e,
    })
}

fn num(t: &str) -> Option<f64> {
    let t = t.trim();
    let t = if t.starts_with('.') { format!("0{t}") } else if t.starts_with("-.") { format!("-0{}", &t[1..]) } else { t.to_string() };
    ex(&t)?.to_f64_const().filter(|v| v.is_finite())
}

fn fmt(v: f64) -> String {
    crate::stats::fmt(v)
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
    for c in ["x", "t", "y", "z"] {
        if fs.iter().any(|n| &**n == c) {
            return sym(c);
        }
    }
    fs.iter().next().map(|n| sym(n)).unwrap_or_else(|| sym("x"))
}

// ---------------------------------------------------------------------------
// ODE steppers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Method {
    Euler,
    BackwardEuler,
    Midpoint,
    ImplicitMidpoint,
    Heun,
    Rk4,
    Bs23,
    Rkf45,
}

impl Method {
    fn name(self) -> &'static str {
        match self {
            Method::Euler => "forward Euler",
            Method::BackwardEuler => "backward Euler",
            Method::Midpoint => "midpoint (RK2)",
            Method::ImplicitMidpoint => "implicit midpoint",
            Method::Heun => "Heun",
            Method::Rk4 => "Runge–Kutta (RK4)",
            Method::Bs23 => "Bogacki–Shampine (RK23)",
            Method::Rkf45 => "Runge–Kutta–Fehlberg (RKF45)",
        }
    }
    fn formula(self) -> &'static str {
        match self {
            Method::Euler => "y_(k+1) = y_k + h f(x_k, y_k)",
            Method::BackwardEuler => "y_(k+1) = y_k + h f(x_(k+1), y_(k+1)) (solved by Newton iteration)",
            Method::Midpoint => "y_(k+1) = y_k + h f(x_k + h/2, y_k + (h/2) f(x_k, y_k))",
            Method::ImplicitMidpoint => "y_(k+1) = y_k + h f(x_k + h/2, (y_k + y_(k+1))/2)",
            Method::Heun => "y_(k+1) = y_k + (h/2) [f(x_k, y_k) + f(x_(k+1), y_k + h f(x_k, y_k))]",
            Method::Rk4 => "y_(k+1) = y_k + (h/6)(k₁ + 2k₂ + 2k₃ + k₄)",
            Method::Bs23 => "third-order Bogacki–Shampine stages with fixed step",
            Method::Rkf45 => "fifth-order Runge–Kutta–Fehlberg solution with fixed step",
        }
    }
}

fn find_method(l: &str) -> Option<Method> {
    let l = l.replace('-', " ");
    Some(if re!(r"backward euler|implicit euler").is_match(&l) {
        Method::BackwardEuler
    } else if re!(r"implicit midpoint").is_match(&l) {
        Method::ImplicitMidpoint
    } else if re!(r"midpoint method|modified euler|rk2\b").is_match(&l) {
        Method::Midpoint
    } else if re!(r"\bheun|improved euler").is_match(&l) {
        Method::Heun
    } else if re!(r"bogacki|shampine|\brk23\b|ode23").is_match(&l) {
        Method::Bs23
    } else if re!(r"\br\s*k\s*f\b|fehlberg|rkf45|\brkf\b").is_match(&l) {
        Method::Rkf45
    } else if re!(r"runge\s*kutta|\brk4\b|\brk\b").is_match(&l) {
        Method::Rk4
    } else if re!(r"euler").is_match(&l) {
        Method::Euler
    } else {
        return None;
    })
}

struct Ivp {
    f: Program,
    fy: Program,
    rhs: Expr,
    x0: f64,
    y0: f64,
    x1: f64,
    h: f64,
}

fn parse_ivp(c: &str) -> Option<Ivp> {
    let t = c.replace("y'(x)", "y'").replace("y′", "y'").replace("dy/dx", "y'").replace("dy/dt", "y'");
    let rhs_m = re!(r"y'\s*=\s*([^,{}]+?)(?:\s*,|\s*\}|\s+from\b|\s+with\b|\s+apply\b|\s+using\b|\s+by\b|$)").captures(&t)?;
    let rhs = ex(&rhs_m[1])?;
    let ic = re!(r"y\s*\(\s*(-?[\d.]+)\s*\)\s*=\s*(-?[\d.]+)").captures(&t)?;
    let (x0, y0) = (num(&ic[1])?, num(&ic[2])?);
    let range = re!(r"from\s+(-?[\d.]+)\s+to\s+(-?[\d.]+)").captures(&t);
    let x1 = range.as_ref().and_then(|r| num(&r[2])).or_else(|| re!(r"(?:to|until)\s+x\s*=\s*(-?[\d.]+)").captures(&t).and_then(|m| num(&m[1])));
    let h = re!(r"(?:\bh|step(?:\s+size)?)\s*=\s*(-?\.?[\d.]+)").captures(&t).and_then(|m| num(&m[1]));
    let (xs, x1, h) = match (x1, h) {
        (Some(b), Some(h)) => (range.as_ref().and_then(|r| num(&r[1])).unwrap_or(x0), b, h),
        (Some(b), None) => {
            let a = range.as_ref().and_then(|r| num(&r[1])).unwrap_or(x0);
            (a, b, (b - a) / 10.0)
        }
        (None, Some(h)) => (x0, x0 + 10.0 * h, h),
        (None, None) => (x0, x0 + 1.0, 0.1),
    };
    let _ = xs;
    if h <= 0.0 || x1 <= x0 || (x1 - x0) / h > 100_000.0 {
        return None;
    }
    let ys = sym("y");
    let xv = if rhs.free_symbols().iter().any(|n| &**n == "t") { "t" } else { "x" };
    let f = Program::compile(&rhs, &[xv, "y"])?;
    let fy = Program::compile(&s::simplify(&diff(&rhs, &ys)), &[xv, "y"])?;
    Some(Ivp { f, fy, rhs, x0, y0, x1, h })
}

fn step(ivp: &Ivp, m: Method, x: f64, y: f64, h: f64) -> f64 {
    let f = |a: f64, b: f64| ivp.f.eval(&[a, b]);
    match m {
        Method::Euler => y + h * f(x, y),
        Method::BackwardEuler => {
            let mut z = y + h * f(x, y);
            for _ in 0..50 {
                let g = z - y - h * f(x + h, z);
                let dg = 1.0 - h * ivp.fy.eval(&[x + h, z]);
                let nz = z - g / dg;
                if (nz - z).abs() < 1e-15 * nz.abs().max(1.0) {
                    z = nz;
                    break;
                }
                z = nz;
            }
            z
        }
        Method::ImplicitMidpoint => {
            let mut z = y + h * f(x, y);
            for _ in 0..50 {
                let mid = (y + z) / 2.0;
                let g = z - y - h * f(x + h / 2.0, mid);
                let dg = 1.0 - h * ivp.fy.eval(&[x + h / 2.0, mid]) / 2.0;
                let nz = z - g / dg;
                if (nz - z).abs() < 1e-15 * nz.abs().max(1.0) {
                    z = nz;
                    break;
                }
                z = nz;
            }
            z
        }
        Method::Midpoint => y + h * f(x + h / 2.0, y + h / 2.0 * f(x, y)),
        Method::Heun => {
            let k1 = f(x, y);
            let k2 = f(x + h, y + h * k1);
            y + h / 2.0 * (k1 + k2)
        }
        Method::Rk4 => {
            let k1 = f(x, y);
            let k2 = f(x + h / 2.0, y + h / 2.0 * k1);
            let k3 = f(x + h / 2.0, y + h / 2.0 * k2);
            let k4 = f(x + h, y + h * k3);
            y + h / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4)
        }
        Method::Bs23 => {
            let k1 = f(x, y);
            let k2 = f(x + h / 2.0, y + h / 2.0 * k1);
            let k3 = f(x + 0.75 * h, y + 0.75 * h * k2);
            y + h * (2.0 / 9.0 * k1 + 1.0 / 3.0 * k2 + 4.0 / 9.0 * k3)
        }
        Method::Rkf45 => {
            let k1 = h * f(x, y);
            let k2 = h * f(x + h / 4.0, y + k1 / 4.0);
            let k3 = h * f(x + 3.0 * h / 8.0, y + 3.0 * k1 / 32.0 + 9.0 * k2 / 32.0);
            let k4 = h * f(x + 12.0 * h / 13.0, y + 1932.0 * k1 / 2197.0 - 7200.0 * k2 / 2197.0 + 7296.0 * k3 / 2197.0);
            let k5 = h * f(x + h, y + 439.0 * k1 / 216.0 - 8.0 * k2 + 3680.0 * k3 / 513.0 - 845.0 * k4 / 4104.0);
            let k6 = h * f(x + h / 2.0, y - 8.0 * k1 / 27.0 + 2.0 * k2 - 3544.0 * k3 / 2565.0 + 1859.0 * k4 / 4104.0 - 11.0 * k5 / 40.0);
            y + 16.0 * k1 / 135.0 + 6656.0 * k3 / 12825.0 + 28561.0 * k4 / 56430.0 - 9.0 * k5 / 50.0 + 2.0 * k6 / 55.0
        }
    }
}

fn integrate_ivp(ivp: &Ivp, m: Method) -> Vec<(f64, f64)> {
    let n = ((ivp.x1 - ivp.x0) / ivp.h).round().max(1.0) as usize;
    let h = (ivp.x1 - ivp.x0) / n as f64;
    let mut out = vec![(ivp.x0, ivp.y0)];
    let (mut x, mut y) = (ivp.x0, ivp.y0);
    for k in 1..=n {
        y = step(ivp, m, x, y, h);
        x = ivp.x0 + k as f64 * h;
        out.push((x, y));
    }
    out
}

fn ode_methods(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches(['?', '.']).to_string();
    let l = c.to_lowercase();
    if !l.contains("y'") && !l.contains("dy/d") && !l.contains("y′") {
        return None;
    }
    let ms: Vec<Method> = {
        let parts: Vec<&str> = re!(r"\s+(?:vs\.?|versus|and|compared? (?:to|with))\s+").split(&l).collect();
        let mut v: Vec<Method> = parts.iter().filter_map(|p| find_method(p)).collect();
        v.dedup();
        v
    };
    if ms.is_empty() {
        return None;
    }
    let ivp = parse_ivp(&l)?;
    let n = ((ivp.x1 - ivp.x0) / ivp.h).round() as usize;
    let sols: Vec<Vec<(f64, f64)>> = ms.iter().map(|m| integrate_ivp(&ivp, *m)).collect();
    if sols.iter().any(|s| s.iter().any(|(_, y)| !y.is_finite())) {
        return None;
    }
    let label = format!("{} for y′ = {}, y({}) = {}, from {} to {}, h = {}", ms.iter().map(|m| m.name()).collect::<Vec<_>>().join(" vs "), show(&ivp.rhs), fmt(ivp.x0), fmt(ivp.y0), fmt(ivp.x0), fmt(ivp.x1), fmt(ivp.h));
    let stride = (n / 20).max(1);
    let mut header = vec!["k", "x_k"];
    header.extend(ms.iter().map(|m| m.name()));
    let rowsv: Vec<Vec<Content>> = (0..=n).filter(|k| k % stride == 0 || *k == n).map(|k| {
        let mut r = vec![text(k.to_string()), text(fmt(sols[0][k].0))];
        r.extend(sols.iter().map(|s| text(fmt(s[k].1))));
        r
    }).collect();
    let mut pods = vec![pod("Input", "Input interpretation", text(label))];
    let res = if ms.len() == 1 { fmt(sols[0][n].1) } else { ms.iter().zip(&sols).map(|(m, s)| format!("{}: {}", m.name(), fmt(s[n].1))).collect::<Vec<_>>().join("\n") };
    pods.push(primary(pod("Result", &format!("y({}) ≈", fmt(ivp.x1)), text(res))));
    pods.push(pod("Steps", "Steps", table(Some(header), rowsv)));
    // exact solution for comparison, when the ODE solver finds one
    let eq_text = format!("y' = {}, y({}) = {}", show(&ivp.rhs), fmt(ivp.x0), fmt(ivp.y0));
    let exact = super::query_with(&eq_text, q.opts);
    if exact.success {
        if let Some(p) = exact.pods.iter().find(|p| p.title.to_lowercase().contains("solution") && p.id != "Input") {
            let mut p = p.clone();
            p.primary = false;
            p.title = "Exact solution".into();
            pods.push(p);
        }
    }
    pods.push(pod("Formula", "Iteration", text(ms.iter().map(|m| format!("{}: {}", m.name(), m.formula())).collect::<Vec<_>>().join("\n"))));
    Some(Answer::new("numerical ODE solution", pods))
}

// ---------------------------------------------------------------------------
// root finding
// ---------------------------------------------------------------------------

fn roots(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches(['?', '.']).to_string();
    let l = c.to_lowercase();
    let kind = if l.contains("newton") { "newton" } else if l.contains("secant") { "secant" } else if l.contains("bisection") { "bisection" } else { return None };
    let body = re!(r"(?i)\b(?:using|use|with|by|apply)?\s*(?:the\s+)?(?:newton(?:'s|-raphson)?|secant|bisection)(?:\s+method)?\b").replace_all(&c, " ").to_string();
    let body = re!(r"(?i)^\s*(?:to\s+)?(?:solve|find|for|on)?\s*").replace(body.trim(), "").to_string();
    // "root of 2" → x² − 2
    let (f, guesses): (Expr, Vec<f64>) = if let Some(m) = re!(r"(?i)^(?:the\s+)?(?:square\s+)?root\s+of\s+(\d+(?:\.\d+)?)\s*$").captures(body.trim()) {
        let a = num(&m[1])?;
        (sub(square(sym("x")), float(a)), vec![0.0, a.max(1.0)])
    } else {
        let gm = re!(r"(?i)\s*(?:,|\bat\b|\bwith\b|\bstarting\b|\bfrom\b|\bon\b|\bin\b)\s*(.*)$").captures(body.trim());
        let (expr_part, rest) = match &gm {
            Some(m) => (&body.trim()[..m.get(0)?.start()], m[1].to_string()),
            None => (body.trim(), String::new()),
        };
        let f = implicit(expr_part.trim())?;
        let gs: Vec<f64> = re!(r"-?\d+(?:\.\d+)?").find_iter(&re!(r"x_?\d\s*=").replace_all(&rest, "")).filter_map(|m| m.as_str().parse().ok()).collect();
        (f, gs)
    };
    let x = main_var(&f);
    let Sym(xn) = &x else { return None };
    let p = Program::compile(&f, &[xn])?;
    let dp = Program::compile(&s::simplify(&diff(&f, &x)), &[xn])?;
    let ev = |v: f64| p.eval1(v);
    let mut rowsv = Vec::new();
    let root = match kind {
        "newton" => {
            let mut xk = guesses.first().copied().unwrap_or(1.0);
            rowsv.push(vec![text("0"), text(fmt(xk)), text(fmt(ev(xk)))]);
            for k in 1..=30 {
                let d = dp.eval1(xk);
                if d == 0.0 || !d.is_finite() {
                    break;
                }
                let nx = xk - ev(xk) / d;
                let done = (nx - xk).abs() < 1e-14 * nx.abs().max(1.0);
                xk = nx;
                if k <= 12 {
                    rowsv.push(vec![text(k.to_string()), text(crate::print::fmt_float(xk, 15)), text(fmt(ev(xk)))]);
                }
                if done {
                    break;
                }
            }
            xk
        }
        "secant" => {
            let (mut a, mut b) = (guesses.first().copied().unwrap_or(0.0), guesses.get(1).copied().unwrap_or(1.0));
            rowsv.push(vec![text("0"), text(fmt(a)), text(fmt(ev(a)))]);
            rowsv.push(vec![text("1"), text(fmt(b)), text(fmt(ev(b)))]);
            for k in 2..=60 {
                let (fa, fb) = (ev(a), ev(b));
                if fb == fa {
                    break;
                }
                let c2 = b - fb * (b - a) / (fb - fa);
                if !c2.is_finite() {
                    return None;
                }
                a = b;
                b = c2;
                if k <= 14 {
                    rowsv.push(vec![text(k.to_string()), text(crate::print::fmt_float(b, 15)), text(fmt(ev(b)))]);
                }
                if (b - a).abs() < 1e-14 * b.abs().max(1.0) {
                    break;
                }
            }
            b
        }
        _ => {
            let (mut a, mut b) = match guesses.as_slice() {
                [a, b, ..] => (*a, *b),
                _ => {
                    // search an interval with a sign change around the origin
                    let mut found = None;
                    'outer: for w in [1.0, 2.0, 5.0, 10.0, 100.0] {
                        let n = 200;
                        for i in 0..n {
                            let (u, v) = (-w + 2.0 * w * i as f64 / n as f64, -w + 2.0 * w * (i + 1) as f64 / n as f64);
                            if ev(u) * ev(v) <= 0.0 && ev(u).is_finite() && ev(v).is_finite() {
                                found = Some((u, v));
                                break 'outer;
                            }
                        }
                    }
                    let (u, v) = found?;
                    // prefer the positive root for "root of a"
                    if guesses.len() == 2 { (guesses[0], guesses[1]) } else { (u, v) }
                }
            };
            if ev(a) * ev(b) > 0.0 {
                return None;
            }
            for k in 1..=60 {
                let m = (a + b) / 2.0;
                if k <= 12 {
                    rowsv.push(vec![text(k.to_string()), text(fmt(a)), text(fmt(b)), text(fmt(m))]);
                }
                if ev(a) * ev(m) <= 0.0 {
                    b = m;
                } else {
                    a = m;
                }
            }
            (a + b) / 2.0
        }
    };
    if !root.is_finite() || ev(root).abs() > 1e-6 * (1.0 + root.abs()) {
        return None;
    }
    let header = if kind == "bisection" { vec!["k", "a", "b", "midpoint"] } else { vec!["k", "x_k", "f(x_k)"] };
    let name = match kind {
        "newton" => "Newton's method",
        "secant" => "secant method",
        _ => "bisection method",
    };
    Some(ans("root finding", format!("{name} for {} = 0", show(&f)), "Root", text(crate::print::fmt_float(root, 15)), vec![pod("Iterations", "Iterations", table(Some(header), rowsv))]))
}

// ---------------------------------------------------------------------------
// quadrature rules
// ---------------------------------------------------------------------------

fn quadrature(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches(['?', '.']).to_string();
    let l = c.to_lowercase();
    let kind = if l.contains("boole") { "boole" } else if l.contains("simpson") { "simpson" } else if re!(r"trapez").is_match(&l) { "trapezoid" } else if l.contains("midpoint rule") { "midpoint" } else { return None };
    let n = re!(r"(\d+)[- ]?(?:intervals?|subintervals?|panels?|strips?|steps?)\b|\bn\s*=\s*(\d+)").captures(&l).and_then(|m| m.get(1).or(m.get(2)).and_then(|x| x.as_str().parse::<usize>().ok()));
    let bounds = re!(r"(?:from\s+(\S+)\s+to\s+(\S+?)(?:\s|$|,)|on\s+\[\s*([^,\]]+)\s*,\s*([^\]]+)\])").captures(&l)?;
    let (a, b) = (num(bounds.get(1).or(bounds.get(3))?.as_str())?, num(bounds.get(2).or(bounds.get(4))?.as_str())?);
    // the integrand: strip the rule words, bounds and n
    let mut t = l.clone();
    for r in [re!(r"\d+[- ]?(?:intervals?|subintervals?|panels?|strips?|steps?)\b"), re!(r"\bn\s*=\s*\d+"), re!(r"(?:from\s+\S+\s+to\s+\S+|on\s+\[[^\]]*\])"), re!(r"(?:using|use|with|by|apply)?\s*(?:the\s+)?(?:composite\s+)?(?:boole'?s|simpson'?s|trapezoidal|trapezoid|trapezium|midpoint)\s+rule"), re!(r"\b(?:integrate|integral(?:\s+of)?|approximate|estimate)\b"), re!(r"\bd[xt]\b")] {
        t = r.replace_all(&t, " ").to_string();
    }
    let t = t.replace(',', " ");
    let f = ex(t.trim())?;
    let x = main_var(&f);
    let Sym(xn) = &x else { return None };
    let p = Program::compile(&f, &[xn])?;
    let n = n.unwrap_or(match kind {
        "boole" => 4,
        "simpson" => 2,
        _ => 4,
    });
    if n == 0 || n > 1_000_000 {
        return None;
    }
    let h = (b - a) / n as f64;
    let fv = |i: f64| p.eval1(a + i * h);
    let (name, val) = match kind {
        "boole" => {
            if n % 4 != 0 {
                return None;
            }
            let s: f64 = (0..n / 4).map(|j| {
                let i = (4 * j) as f64;
                7.0 * fv(i) + 32.0 * fv(i + 1.0) + 12.0 * fv(i + 2.0) + 32.0 * fv(i + 3.0) + 7.0 * fv(i + 4.0)
            }).sum();
            ("Boole's rule", 2.0 * h / 45.0 * s)
        }
        "simpson" => {
            if n % 2 == 1 {
                return None;
            }
            let s: f64 = (0..=n).map(|i| fv(i as f64) * if i == 0 || i == n { 1.0 } else if i % 2 == 1 { 4.0 } else { 2.0 }).sum();
            ("Simpson's rule", h / 3.0 * s)
        }
        "midpoint" => ("midpoint rule", h * (0..n).map(|i| fv(i as f64 + 0.5)).sum::<f64>()),
        _ => ("trapezoidal rule", h * ((fv(0.0) + fv(n as f64)) / 2.0 + (1..n).map(|i| fv(i as f64)).sum::<f64>())),
    };
    if !val.is_finite() {
        return None;
    }
    let mut extra = Vec::new();
    if let Some(qd) = crate::calculus::integrate::nintegrate(&f, &x, a, b) {
        extra.push(pod("Exact", "Value of the integral", text(format!("{} (error of the rule {})", fmt(qd.value), fmt(val - qd.value)))));
    }
    extra.push(pod("Width", "Subinterval width", text(format!("h = {}", fmt(h)))));
    Some(ans("quadrature", format!("{name} for ∫ {} d{} from {} to {} with {} subintervals", show(&f), xn, fmt(a), fmt(b), n), name, text(fmt(val)), extra))
}

// ---------------------------------------------------------------------------
// vectors
// ---------------------------------------------------------------------------

/// "2i - 4j + 3k" → {2, -4, 3}
fn ijk(t: &str) -> Option<Vec<Expr>> {
    let t = t.trim().trim_matches(|c| c == '(' || c == ')').replace(' ', "");
    if !re!(r"[ijk]").is_match(&t) || !re!(r"^[-+\d./*ijk^()]+$").is_match(&t) {
        return None;
    }
    let mut v = vec![zero(), zero(), zero()];
    let mut seen_k = false;
    for m in re!(r"([+-]?)([\d./^]*)\*?([ijk])").captures_iter(&t) {
        let c = if m[2].is_empty() { one() } else { ex(&m[2])? };
        let c = if &m[1] == "-" { neg(c) } else { c };
        let slot = match &m[3] {
            "i" => 0,
            "j" => 1,
            _ => {
                seen_k = true;
                2
            }
        };
        v[slot] = s::simplify(&add(vec![v[slot].clone(), c]));
    }
    // reject leftovers (a scalar part)
    let rest = re!(r"([+-]?)([\d./^]*)\*?([ijk])").replace_all(&t, "");
    if !rest.is_empty() {
        return None;
    }
    if !seen_k {
        v.pop();
    }
    Some(v)
}

fn vec_of(t: &str) -> Option<Vec<Expr>> {
    let t = t.trim();
    if let Some(v) = ijk(t) {
        return Some(v);
    }
    let inner = t.strip_prefix('(').and_then(|x| x.strip_suffix(')')).or_else(|| t.strip_prefix('{').and_then(|x| x.strip_suffix('}'))).or_else(|| t.strip_prefix('<').and_then(|x| x.strip_suffix('>')))?;
    let parts = super::nl::split_top(inner);
    if parts.len() < 2 {
        return None;
    }
    parts.iter().map(|p| ex(p).map(|e| s::simplify(&e))).collect()
}

fn vec_list(t: &str) -> Option<Vec<Vec<Expr>>> {
    let t = t.trim().trim_start_matches('{').trim_end_matches('}');
    let t = if t.starts_with('{') || t.starts_with('(') { t.to_string() } else { format!("{{{t}}}") };
    let items: Vec<String> = re!(r"[({<][^(){}<>]*[)}>]").find_iter(&t).map(|m| m.as_str().to_string()).collect();
    if items.is_empty() {
        return None;
    }
    let vs: Vec<Vec<Expr>> = items.iter().map(|i| vec_of(i)).collect::<Option<_>>()?;
    let n = vs[0].len();
    (vs.iter().all(|v| v.len() == n)).then_some(vs)
}

fn dot(a: &[Expr], b: &[Expr]) -> Expr {
    s::simplify(&add(a.iter().zip(b).map(|(x, y)| mul(vec![x.clone(), y.clone()])).collect()))
}

fn norm(a: &[Expr]) -> Expr {
    s::simplify(&func("sqrt", vec![add(a.iter().map(|x| square(x.clone())).collect())]))
}

fn vectors(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches(['?', '.']).to_string();
    let l = c.to_lowercase();
    // a single vector
    if let Some(m) = re!(r"^vector\s+(.+)$").captures(&l) {
        let body = m[1].trim();
        if let Some(v) = vec_of(body) {
            let n = norm(&v);
            let unit: Vec<Expr> = v.iter().map(|x| s::simplify(&div(x.clone(), n.clone()))).collect();
            let mut rowsv = vec![("components", show(&List(v.clone()))), ("magnitude", format!("{}{}", show(&n), n.to_f64_const().filter(|_| !n.is_integer()).map(|x| format!(" ≈ {}", fmt(x))).unwrap_or_default())), ("unit vector", show(&List(unit)))];
            if v.len() == 2 {
                if let (Some(a), Some(b)) = (v[0].to_f64_const(), v[1].to_f64_const()) {
                    rowsv.push(("direction angle", format!("{}°", fmt(b.atan2(a).to_degrees()))));
                }
            } else if v.len() == 3 {
                if let Some(nv) = n.to_f64_const() {
                    let ang: Vec<String> = v.iter().filter_map(|x| x.to_f64_const()).map(|x| format!("{}°", fmt((x / nv).acos().to_degrees()))).collect();
                    rowsv.push(("direction angles (x, y, z)", ang.join(", ")));
                }
            }
            return Some(ans("vector", format!("vector {}", show(&List(v))), "Properties", table(None, rowsv.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect()), vec![]));
        }
        // "vector (1,3,-1) + (-2,1,6)": sums and differences of tuples
        let mut acc: Option<Vec<Expr>> = None;
        let mut rest = body.trim().to_string();
        let mut sign = 1;
        while !rest.is_empty() {
            let m = re!(r"^(\([^()]*\)|\{[^{}]*\}|<[^<>]*>)\s*([+-])?\s*").captures(&rest)?;
            let v = vec_of(&m[1])?;
            acc = Some(match acc {
                None => v,
                Some(a) if a.len() == v.len() => a.into_iter().zip(v).map(|(x, y)| s::simplify(&if sign > 0 { add(vec![x, y]) } else { sub(x, y) })).collect(),
                _ => return None,
            });
            sign = if m.get(2).map(|x| x.as_str()) == Some("-") { -1 } else { 1 };
            let done = m.get(2).is_none();
            rest = rest[m.get(0)?.end()..].to_string();
            if done && !rest.is_empty() {
                return None;
            }
        }
        let v = acc?;
        return Some(ans("vector sum", format!("vector {}", body.trim()), "Result", math(&List(v.clone())), vec![pod("Norm", "Vector length", math(&norm(&v)))]));
    }
    // dot product in i, j, k notation: "(7i-j+3k).(4i-2k)"
    if let Some(m) = re!(r"^\(([^()]+)\)\s*(\.|·|x|×|cross|dot)\s*\(([^()]+)\)$").captures(&l) {
        let (a, b) = (ijk(&m[1])?, ijk(&m[3])?);
        let (mut a, mut b) = (a, b);
        while a.len() < 3 {
            a.push(zero());
        }
        while b.len() < 3 {
            b.push(zero());
        }
        if matches!(&m[2], "." | "·" | "dot") {
            return Some(ans("dot product", format!("({}) · ({})", show(&List(a.clone())), show(&List(b.clone()))), "Dot product", math(&dot(&a, &b)), vec![]));
        }
        let cr = vec![s::simplify(&sub(mul(vec![a[1].clone(), b[2].clone()]), mul(vec![a[2].clone(), b[1].clone()]))), s::simplify(&sub(mul(vec![a[2].clone(), b[0].clone()]), mul(vec![a[0].clone(), b[2].clone()]))), s::simplify(&sub(mul(vec![a[0].clone(), b[1].clone()]), mul(vec![a[1].clone(), b[0].clone()])))];
        return Some(ans("cross product", format!("({}) × ({})", show(&List(a)), show(&List(b))), "Cross product", math(&List(cr)), vec![]));
    }
    // linear independence
    if let Some(m) = re!(r"^(?:are\s+|is\s+(?:the\s+set\s+)?)?(?:the\s+vectors\s+)?(?:linear(?:ly)?\s+(?:in)?dependen(?:ce|t)\s+(?:of\s+)?)?(.+?)(?:\s+linearly\s+(?:in)?dependent)?$").captures(&l) {
        if l.contains("linear") && (l.contains("independ") || l.contains("dependen")) {
            let vs = vec_list(&m[1])?;
            if vs.len() < 2 {
                return None;
            }
            let mat: crate::linalg::Mat = vs.clone();
            let rank = crate::linalg::rank(&mat);
            let syms: Vec<_> = vs.iter().flatten().flat_map(|e| e.free_symbols()).collect();
            let mut extra = Vec::new();
            let verdict = if vs.len() == vs[0].len() {
                let d = s::simplify(&crate::linalg::det(&mat));
                extra.push(pod("Det", "Determinant", math(&d)));
                if !syms.is_empty() && !d.is_zero() {
                    let free: Vec<_> = d.free_symbols().into_iter().collect();
                    if free.len() == 1 {
                        let v = sym(&free[0]);
                        let rs = crate::solve::solve(&d, &v);
                        let roots: Vec<String> = rs.real().iter().map(|r| format!("{} = {}", show(&v), show(r))).collect();
                        if !roots.is_empty() {
                            format!("linearly independent unless {}", roots.join(" or "))
                        } else {
                            "linearly independent".into()
                        }
                    } else {
                        "linearly independent when the determinant is non-zero".into()
                    }
                } else if d.is_zero() {
                    "linearly dependent".into()
                } else {
                    "linearly independent".into()
                }
            } else if rank == vs.len() {
                "linearly independent".into()
            } else {
                "linearly dependent".into()
            };
            extra.push(pod("Rank", "Rank of the matrix of the vectors", text(rank.to_string())));
            if verdict == "linearly dependent" {
                // a dependency: null space of the transposed matrix
                let ns = crate::linalg::nullspace(&crate::linalg::transpose(&mat));
                if let Some(c) = ns.first() {
                    extra.push(pod("Relation", "Linear relation (coefficients)", math(&List(c.clone()))));
                }
            }
            return Some(ans("linear independence", format!("linear independence of {}", vs.iter().map(|v| show(&List(v.clone()))).collect::<Vec<_>>().join(", ")), "Result", text(verdict), extra));
        }
    }
    // orthogonality
    if l.contains("orthogonal") {
        let body = re!(r"^(?:are\s+)?(?:the\s+vectors\s+)?(?:orthogonality\s+(?:of\s+)?)?(.+?)(?:\s+(?:mutually\s+)?orthogonal(?:ity)?)?$").captures(&l)?;
        let body = re!(r"\s+and\s+").replace_all(&body[1], ", ").to_string();
        let items: Vec<String> = re!(r"\(([^()]*)\)").find_iter(&body).map(|m| m.as_str().to_string()).collect();
        let vs: Vec<Vec<Expr>> = items.iter().map(|i| vec_of(i)).collect::<Option<_>>()?;
        if vs.len() < 2 || vs.iter().any(|v| v.len() != vs[0].len()) {
            return None;
        }
        let mut rowsv = Vec::new();
        let mut all_zero = true;
        let mut undecided = false;
        for i in 0..vs.len() {
            for j in i + 1..vs.len() {
                let d = dot(&vs[i], &vs[j]);
                if !d.free_symbols().is_empty() {
                    undecided = true;
                } else if !d.is_zero() && d.to_f64_const().is_none_or(|v| v.abs() > 1e-12) {
                    all_zero = false;
                }
                rowsv.push(vec![text(format!("v{} · v{}", i + 1, j + 1)), math(&d)]);
            }
        }
        let verdict = if undecided && all_zero { "orthogonal exactly when every dot product below is zero".to_string() } else if all_zero { "yes: every pair is orthogonal".into() } else { "no: some dot products are non-zero".into() };
        return Some(ans("orthogonality", format!("orthogonality of {}", vs.iter().map(|v| show(&List(v.clone()))).collect::<Vec<_>>().join(", ")), "Result", text(verdict), vec![pod("Dots", "Dot products", table(None, rowsv))]));
    }
    // row space / column space
    if let Some(m) = re!(r"^(?:the\s+)?(row|column)\s+space\s+(?:of\s+)?(\{.+\})$|^(\{.+\})\s+(row|column)\s+space$").captures(&c.to_lowercase()) {
        let (kind, body) = if let Some(k) = m.get(1) { (k.as_str().to_string(), m[2].to_string()) } else { (m[4].to_string(), m[3].to_string()) };
        let mat = crate::linalg::from_expr(&ex(&body)?)?;
        let basis: Vec<Vec<Expr>> = if kind == "row" {
            let (r, piv) = crate::linalg::rref(&mat);
            r.into_iter().take(piv.len()).collect()
        } else {
            let (_, piv) = crate::linalg::rref(&mat);
            piv.iter().map(|&j| mat.iter().map(|row| row[j].clone()).collect()).collect()
        };
        let dim = basis.len();
        return Some(ans(&format!("{kind} space"), format!("{kind} space of {}", show(&crate::linalg::to_expr(&mat))), "Basis", math(&List(basis.into_iter().map(List).collect())), vec![pod("Dim", "Dimension", text(dim.to_string()))]));
    }
    // projections
    if let Some(m) = re!(r"^(?:what is\s+)?(?:the\s+)?(?:vector\s+)?(?:projection|project)\s+(?:of\s+)?(?:the\s+)?(?:point\s+|vector\s+)?(.+?)\s+(?:onto|on)\s+(?:the\s+)?(.+)$").captures(&l) {
        let u = vec_of(m[1].trim())?;
        let target = m[2].trim().trim_end_matches(" axis").trim();
        let axes = ["x", "y", "z", "w"];
        let keep: Option<Vec<usize>> = if let Some(a) = re!(r"^([xyz])[- ]?(?:axis)?$").captures(target) {
            Some(vec![axes.iter().position(|x| *x == &a[1])?])
        } else if let Some(p) = re!(r"^([xyz])([xyz])[- ]?plane$").captures(target) {
            Some(vec![axes.iter().position(|x| *x == &p[1])?, axes.iter().position(|x| *x == &p[2])?])
        } else {
            None
        };
        if let Some(keep) = keep {
            if keep.iter().any(|&k| k >= u.len()) {
                return None;
            }
            let pr: Vec<Expr> = (0..u.len()).map(|i| if keep.contains(&i) { u[i].clone() } else { zero() }).collect();
            return Some(ans("projection", format!("projection of {} onto the {}", show(&List(u.clone())), m[2].trim()), "Projection", math(&List(pr)), vec![]));
        }
        if let Some(sp) = re!(r"^span\s+(?:of\s+)?(.+)$").captures(target) {
            let basis = vec_list(&sp[1])?;
            // least squares: A (AᵀA)⁻¹ Aᵀ u with the basis as columns
            let a: crate::linalg::Mat = (0..u.len()).map(|i| basis.iter().map(|b| b[i].clone()).collect()).collect();
            let at = crate::linalg::transpose(&a);
            let ata = crate::linalg::matmul(&at, &a)?;
            let inv = crate::linalg::inverse(&ata)?;
            let atu: Vec<Expr> = at.iter().map(|r| dot(r, &u)).collect();
            let coef: Vec<Expr> = inv.iter().map(|r| dot(r, &atu)).collect();
            let pr: Vec<Expr> = a.iter().map(|r| dot(r, &coef)).collect();
            return Some(ans("projection", format!("projection of {} onto the span of {}", show(&List(u.clone())), basis.iter().map(|b| show(&List(b.clone()))).collect::<Vec<_>>().join(", ")), "Projection", math(&List(pr)), vec![pod("Coef", "Coefficients in the spanning vectors", math(&List(coef)))]));
        }
        let v = vec_of(target)?;
        if v.len() != u.len() {
            return None;
        }
        let k = s::simplify(&div(dot(&u, &v), dot(&v, &v)));
        let pr: Vec<Expr> = v.iter().map(|x| s::simplify(&mul(vec![k.clone(), x.clone()]))).collect();
        let scalar = s::simplify(&div(dot(&u, &v), norm(&v)));
        return Some(ans("projection", format!("projection of {} onto {}", show(&List(u.clone())), show(&List(v.clone()))), "Vector projection", math(&List(pr)), vec![pod("Scalar", "Scalar projection", math(&scalar))]));
    }
    // spherical / cylindrical coordinates of a point
    if let Some(m) = re!(r"^(.+?)\s+(?:in|to)\s+(spherical|cylindrical|polar)\s+coordinates$").captures(&l) {
        let v = vec_of(m[1].trim())?;
        let r2 = add(v.iter().map(|x| square(x.clone())).collect());
        let r = s::simplify(&func("sqrt", vec![r2]));
        let (x, y) = (v[0].clone(), v[1].clone());
        let theta = {
            let (xf, yf) = (x.to_f64_const()?, y.to_f64_const()?);
            let base = s::simplify(&func("atan", vec![div(y.clone(), x.clone())]));
            if xf > 0.0 { base } else if xf < 0.0 { s::simplify(&add(vec![base, if yf >= 0.0 { pi() } else { neg(pi()) }])) } else { s::simplify(&mul(vec![Num(Rat::new(if yf >= 0.0 { 1.into() } else { (-1).into() }, 2.into())), pi()])) }
        };
        let deg = |e: &Expr| e.to_f64_const().map(|v| format!(" ≈ {}°", fmt(v.to_degrees()))).unwrap_or_default();
        let rows_v: Vec<(&str, String)> = match (&m[2], v.len()) {
            ("spherical", 3) => {
                let phi = s::simplify(&func("acos", vec![div(v[2].clone(), r.clone())]));
                vec![("r (radius)", show(&r)), ("θ (polar angle from +z)", format!("{}{}", show(&phi), deg(&phi))), ("φ (azimuth in the xy-plane)", format!("{}{}", show(&theta), deg(&theta)))]
            }
            ("cylindrical", 3) => {
                let rho = s::simplify(&func("sqrt", vec![add(vec![square(x.clone()), square(y.clone())])]));
                vec![("ρ", show(&rho)), ("φ", format!("{}{}", show(&theta), deg(&theta))), ("z", show(&v[2]))]
            }
            ("polar", 2) => vec![("r", show(&r)), ("θ", format!("{}{}", show(&theta), deg(&theta)))],
            _ => return None,
        };
        return Some(ans("coordinates", format!("{} in {} coordinates", show(&List(v.clone())), &m[2]), "Coordinates", table(None, rows_v.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect()), vec![]));
    }
    None
}

// ---------------------------------------------------------------------------
// forms of complex numbers, harmonic conjugates
// ---------------------------------------------------------------------------

fn complex_forms(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches(['?', '.']).to_string();
    let l = c.to_lowercase();
    if let Some(m) = re!(r"^(.+?)\s+(?:from\s+[a-z ]+?\s+)?(?:to|in(?:to)?)\s+(?:the\s+)?(cartesian|rectangular|algebraic|polar(?:\s+trigonometric|\s+exponential)?|trigonometric|exponential)\s+form$").captures(&l) {
        let z = ex(&m[1].replace('I', "i"))?;
        if !z.free_symbols().is_empty() {
            return None;
        }
        let w = Program::compile(&z, &[])?.eval_c(&[]);
        if !w.re.is_finite() || !w.im.is_finite() {
            return None;
        }
        let re_ = s::simplify(&func("re", vec![z.clone()]));
        let im_ = s::simplify(&func("im", vec![z.clone()]));
        let r = s::simplify(&func("abs", vec![z.clone()]));
        let th = s::simplify(&func("arg", vec![z.clone()]));
        let exact_ok = !r.has_float() && r.to_f64_const().is_some() && th.to_f64_const().is_some() && !show(&th).contains("arg");
        let (rs, ts) = if exact_ok { (show(&r), show(&th)) } else { (fmt(w.norm()), fmt(w.arg())) };
        let target = &m[2];
        let (title, res) = if target.contains("exponential") {
            ("Polar exponential form", format!("{rs} e^(i {ts})"))
        } else if target.contains("trig") || target == "polar" {
            ("Polar trigonometric form", format!("{rs} (cos({ts}) + i sin({ts}))"))
        } else {
            let (a, b) = if re_.to_f64_const().is_some() && !show(&re_).contains("re(") { (show(&re_), show(&im_)) } else { (fmt(w.re), fmt(w.im)) };
            ("Cartesian form", format!("{a} + ({b}) i"))
        };
        return Some(ans("complex number", format!("{} in {} form", show(&z), target), title, text(res), vec![pod("Parts", "Modulus and argument", text(format!("|z| = {rs}, arg z = {ts}")))]));
    }
    // harmonic conjugate / analytic function from its real or imaginary part
    if let Some(m) = re!(r"^(?:the\s+)?(?:(harmonic conjugate)\s+(?:of\s+)?(.+)|(?:analytic|holomorphic)\s+function\s+(?:with|whose)\s+(real|imaginary)\s+part\s+(?:is\s+)?(.+))$").captures(&c.to_lowercase()) {
        let (given_real, body) = if m.get(1).is_some() { (true, m[2].to_string()) } else { (&m[3] == "real", m[4].to_string()) };
        let given = ex(&body.replace("sinh[x]", "sinh(x)").replace("sin[y]", "sin(y)"))?;
        let (x, y) = (sym("x"), sym("y"));
        // u_xx + u_yy must vanish
        let lap = s::simplify(&add(vec![diff(&diff(&given, &x), &x), diff(&diff(&given, &y), &y)]));
        if !lap.is_zero() {
            return Some(ans("harmonic", format!("{} is not harmonic", show(&given)), "Result", text(format!("no analytic function exists: the Laplacian is {} ≠ 0", show(&lap))), vec![]));
        }
        // real part u: v_y = u_x, v_x = −u_y. imaginary part v: u_x = v_y, u_y = −v_x.
        let (px, py) = if given_real { (s::simplify(&neg(diff(&given, &y))), s::simplify(&diff(&given, &x))) } else { (s::simplify(&diff(&given, &y)), s::simplify(&neg(diff(&given, &x)))) };
        // other = ∫ py dy + g(x), g′ = px − ∂/∂x ∫ py dy
        let part = crate::calculus::integrate::integrate(&py, &y)?;
        let gp = s::simplify(&sub(px, diff(&part, &x)));
        if gp.has_sym(&y) {
            return None;
        }
        let g = crate::calculus::integrate::integrate(&gp, &x)?;
        let other = s::simplify(&add(vec![part, g]));
        let (u, v) = if given_real { (given.clone(), other.clone()) } else { (other.clone(), given.clone()) };
        // Milne-Thomson: f(z) = u(z, 0) + i v(z, 0)
        let z = sym("z");
        let fz = s::simplify(&add(vec![u.subs_many(&[(x.clone(), z.clone()), (y.clone(), zero())]), mul(vec![crate::expr::i(), v.subs_many(&[(x.clone(), z.clone()), (y.clone(), zero())])])]));
        let title = if given_real { "Harmonic conjugate v" } else { "Real part u" };
        return Some(ans("analytic function", format!("{} {}", if given_real { "harmonic conjugate of" } else { "analytic function with imaginary part" }, show(&given)), title, math(&other), vec![pod("F", "Analytic function (up to an additive constant)", math(&eq(func("f", vec![z.clone()]), fz))), pod("Note", "Cauchy–Riemann", text("u_x = v_y and u_y = −v_x"))]));
    }
    None
}

pub fn handle(q: &Query) -> Option<Answer> {
    ode_methods(q).or_else(|| roots(q)).or_else(|| quadrature(q)).or_else(|| vectors(q)).or_else(|| complex_forms(q))
}

#[cfg(test)]
mod tests {
    fn ask(s: &str) -> String {
        let r = crate::engine::query(s);
        serde_json::to_string(&r.pods).unwrap_or_default()
    }

    #[test]
    fn methods_answers() {
        assert!(ask("Runge-Kutta method, dy/dx = -2xy, y(0) = 2, from 0 to 1, h = .25").contains("0.73"));
        assert!(ask("using Newton's method solve x cos x = 0").contains("Root"));
        assert!(ask("find root of 2 with bisection method").contains("1.41421356"));
        assert!(ask("integral (x^2-2)/x dx from 1 to 2 using Boole's rule").contains("0.11"));
        assert!(ask("(7i-j+3k).(4i-2k)").contains("22"));
        assert!(ask("linear independence (1, 3, -2), (2, 1, -3), (-3, 6, 3)").contains("dependent"));
        assert!(ask("harmonic conjugate x^3 - 3 x y^2").contains("3 x^2 y - y^3"));
    }
}
