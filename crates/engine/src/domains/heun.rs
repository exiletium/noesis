//! Heun functions from their local power series about z = 0: the general
//! (HeunG), confluent (HeunC), biconfluent (HeunB) and triconfluent (HeunT)
//! functions, normalised to 1 at z = 0, and their derivatives.
//!
//! "HeunG[4.3 + 0.1 I, -0.2, 1.3, 0.12, -0.14, 4.32, 0.12]",
//! "compute HeunG derivative [...]", "series confluent heun function (4, -0.6, -0.7, -0.18, 0.3, x)",
//! "plot HeunB(1, 2, 0.5, 0.3, 0.1, x)", "HeunTPrime[x, -0.6, -0.7, -0.18, 0.3, 0.12]".

use crate::engine::*;
use crate::print::fmt_float;
use num_complex::Complex64 as C;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    G,
    C,
    B,
    T,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::G => "HeunG",
            Kind::C => "HeunC",
            Kind::B => "HeunB",
            Kind::T => "HeunT",
        }
    }
    fn title(self) -> &'static str {
        match self {
            Kind::G => "general Heun function",
            Kind::C => "confluent Heun function",
            Kind::B => "biconfluent Heun function",
            Kind::T => "triconfluent Heun function",
        }
    }
    fn params(self) -> &'static [&'static str] {
        match self {
            Kind::G => &["a", "q", "α", "β", "γ", "δ", "z"],
            _ => &["q", "α", "γ", "δ", "ε", "z"],
        }
    }
    fn equation(self) -> &'static str {
        match self {
            Kind::G => "z(z − 1)(z − a) y″ + [γ(z − 1)(z − a) + δ z(z − a) + ε z(z − 1)] y′ + (αβ z − q) y = 0, ε = α + β + 1 − γ − δ",
            Kind::C => "z(z − 1) y″ + [γ(z − 1) + δ z + ε z(z − 1)] y′ + (α z − q) y = 0",
            Kind::B => "z y″ + (γ + δ z + ε z²) y′ + (α z − q) y = 0",
            Kind::T => "y″ + (γ + δ z + ε z²) y′ + (α z − q) y = 0",
        }
    }
}

/// Series coefficients c₀ … c_{n−1}, with c₀ = 1 (and c₁ = 0 for HeunT).
fn coeffs(kind: Kind, p: &[C], n: usize) -> Option<Vec<C>> {
    let one = C::new(1.0, 0.0);
    let mut c = vec![C::new(0.0, 0.0); n + 2];
    c[0] = one;
    let f = |k: usize| C::new(k as f64, 0.0);
    match kind {
        Kind::G => {
            let (a, q, al, be, ga, de) = (p[0], p[1], p[2], p[3], p[4], p[5]);
            let ep = al + be + one - ga - de;
            if (a * ga).norm() == 0.0 {
                return None;
            }
            c[1] = q / (a * ga);
            for k in 1..n {
                let kk = f(k);
                let den = a * (kk + one) * (kk + ga);
                if den.norm() == 0.0 {
                    return None;
                }
                c[k + 1] = ((kk * ((kk - one + ga) * (one + a) + a * de + ep) + q) * c[k] - (kk - one + al) * (kk - one + be) * c[k - 1]) / den;
            }
        }
        Kind::C | Kind::B => {
            let (q, al, ga, de, ep) = (p[0], p[1], p[2], p[3], p[4]);
            if ga.norm() == 0.0 {
                return None;
            }
            c[1] = if kind == Kind::C { -q / ga } else { q / ga };
            for k in 1..n {
                let kk = f(k);
                let den = (kk + one) * (kk + ga);
                if den.norm() == 0.0 {
                    return None;
                }
                c[k + 1] = if kind == Kind::C {
                    (c[k] * (kk * (kk - one + ga + de - ep) - q) + c[k - 1] * (ep * (kk - one) + al)) / den
                } else {
                    -(c[k] * (de * kk - q) + c[k - 1] * (ep * (kk - one) + al)) / den
                };
            }
        }
        Kind::T => {
            let (q, al, ga, de, ep) = (p[0], p[1], p[2], p[3], p[4]);
            for k in 0..n {
                let kk = f(k);
                let cm1 = if k >= 1 { c[k - 1] } else { C::new(0.0, 0.0) };
                c[k + 2] = -(ga * (kk + one) * c[k + 1] + (de * kk - q) * c[k] + (ep * (kk - one) + al) * cm1) / ((kk + f(2)) * (kk + one));
            }
        }
    }
    c.truncate(n);
    Some(c)
}

/// The radius of convergence of the series about 0.
fn radius(kind: Kind, p: &[C]) -> f64 {
    match kind {
        Kind::G => p[0].norm().min(1.0),
        Kind::C => 1.0,
        _ => f64::INFINITY,
    }
}

/// The function (or its derivative) at z, summing the series.
fn value(kind: Kind, p: &[C], z: C, deriv: bool) -> Option<C> {
    if z.norm() >= radius(kind, p) * 0.98 {
        return None;
    }
    let n = 4000;
    let c = coeffs(kind, p, n)?;
    let mut sum = C::new(0.0, 0.0);
    let mut zp = C::new(1.0, 0.0);
    let mut small = 0;
    for (k, ck) in c.iter().enumerate() {
        let term = if deriv {
            if k == 0 {
                continue;
            }
            let t = ck * f64::from(k as u32) * zp;
            zp *= z;
            t
        } else {
            let t = ck * zp;
            zp *= z;
            t
        };
        if !term.re.is_finite() || !term.im.is_finite() {
            return None;
        }
        sum += term;
        if term.norm() <= 1e-17 * sum.norm().max(1e-300) {
            small += 1;
            if small > 4 {
                return Some(sum);
            }
        } else {
            small = 0;
        }
    }
    None
}

fn num(s: &str) -> String {
    fmt_float(s.parse::<f64>().unwrap_or(0.0), 6)
}

fn show(z: C) -> String {
    let r = |v: f64| fmt_float(if v.abs() < 1e-14 { 0.0 } else { v }, 9);
    if z.im.abs() < 1e-12 * z.re.abs().max(1.0) {
        r(z.re)
    } else if z.re.abs() < 1e-12 * z.im.abs() {
        format!("{} i", r(z.im))
    } else {
        format!("{} {} {} i", r(z.re), if z.im < 0.0 { "-" } else { "+" }, r(z.im.abs()))
    }
}

/// "4.3 + 0.1 I", "-.2", "2i", "x".
enum Arg {
    Num(C),
    Var,
}

fn parse_arg(s: &str) -> Option<Arg> {
    let t: String = s.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_lowercase().replace('*', "");
    if matches!(t.as_str(), "x" | "z" | "u" | "t") {
        return Some(Arg::Var);
    }
    let real = r"[+-]?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?";
    if let Ok(v) = t.parse::<f64>() {
        return Some(Arg::Num(C::new(v, 0.0)));
    }
    let im_re = Regex::new(&format!(r"^({real})?([+-]?(?:\d+\.?\d*|\.\d+)?)i$")).ok()?;
    let m = im_re.captures(&t)?;
    let re_part: f64 = m.get(1).map_or(Some(0.0), |x| x.as_str().parse().ok())?;
    let im_s = m.get(2).map_or("", |x| x.as_str());
    let im_part: f64 = match im_s {
        "" | "+" => 1.0,
        "-" => -1.0,
        s => s.parse().ok()?,
    };
    Some(Arg::Num(C::new(re_part, im_part)))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.raw.trim().trim_end_matches(['?', '.']).trim().to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    if !l.contains("heun") {
        return None;
    }
    let m = re!(r"^(?:(evaluate|compute|calculate|find|plot|graph|series|series expansion of|taylor series of|power series of)\s+)?(?:the\s+)?(general heun function|confluent heun function|biconfluent heun function|triconfluent heun function|heun ?function|heun([gcbt]))\s*(prime|'|derivative|′)?\s*(?:of\s+)?[\[(]\s*([^\[\]()]+?)\s*[\])]\s*(?:(series|plot|derivative))?$").captures(&l)?;
    let mode = m.get(1).map(|x| x.as_str()).or(m.get(6).map(|x| x.as_str())).unwrap_or("");
    let kind = match (m.get(3).map(|x| x.as_str()), m.get(2).map_or("", |x| x.as_str())) {
        (Some("g"), _) => Kind::G,
        (Some("c"), _) => Kind::C,
        (Some("b"), _) => Kind::B,
        (Some("t"), _) => Kind::T,
        (_, n) if n.starts_with("confluent") => Kind::C,
        (_, n) if n.starts_with("biconfluent") => Kind::B,
        (_, n) if n.starts_with("triconfluent") => Kind::T,
        _ => Kind::G,
    };
    let deriv = m.get(4).is_some() || mode == "derivative";
    let raw_args: Vec<&str> = m[5].split(',').map(|s| s.trim()).collect();
    let names = kind.params();
    if raw_args.len() != names.len() {
        return None;
    }
    let args: Vec<Arg> = raw_args.iter().map(|a| parse_arg(a)).collect::<Option<_>>()?;
    let vars: Vec<usize> = args.iter().enumerate().filter(|(_, a)| matches!(a, Arg::Var)).map(|(i, _)| i).collect();
    let fname = format!("{}{}", kind.name(), if deriv { "Prime" } else { "" });
    let call = format!("{fname}[{}]", raw_args.join(", "));
    let mut pods = vec![pod("Input", "Input", text(format!("{call} | {}{}", if deriv { "derivative of the " } else { "" }, kind.title())))];
    let nums = |vals: &[Arg], at: Option<C>| -> Vec<C> { vals.iter().map(|a| match a { Arg::Num(c) => *c, Arg::Var => at.unwrap_or_default() }).collect() };
    match vars.len() {
        0 => {
            let v = nums(&args, None);
            let n = v.len();
            let val = value(kind, &v[..n - 1], v[n - 1], deriv)?;
            pods.push(primary(pod("Result", "Result", text(show(val)))));
            let other = value(kind, &v[..n - 1], v[n - 1], !deriv);
            if let Some(o) = other {
                pods.push(pod("Other", if deriv { "Function value" } else { "Derivative" }, text(show(o))));
            }
        }
        1 if mode.contains("series") || mode == "taylor series of" || mode == "power series of" => {
            if vars[0] != names.len() - 1 {
                return None;
            }
            let v = nums(&args, None);
            let n = v.len();
            let c = coeffs(kind, &v[..n - 1], 8)?;
            let mut terms = Vec::new();
            for (k, ck) in c.iter().enumerate().take(6) {
                let (coef, k) = if deriv { (*ck * (k as f64), k as i64 - 1) } else { (*ck, k as i64) };
                if k < 0 || coef.norm() < 1e-15 {
                    continue;
                }
                let cs = show(coef);
                let cs = if cs.contains(' ') { format!("({cs})") } else { cs };
                terms.push(match k {
                    0 => cs,
                    1 => format!("{cs} x"),
                    _ => format!("{cs} x^{k}"),
                });
            }
            let s = format!("{} + O(x^{})", terms.join(" + ").replace("+ -", "- "), if deriv { 5 } else { 6 });
            pods.push(primary(pod("Series", "Series expansion at x = 0", text(s))));
        }
        1 => {
            let i = vars[0];
            let (lo, hi) = if i == names.len() - 1 {
                let r = radius(kind, &nums(&args, None)[..names.len() - 1]);
                if r.is_finite() { (-0.95 * r, 0.95 * r) } else { (-3.0, 3.0) }
            } else {
                (-5.0, 5.0)
            };
            let n = 300;
            let (mut xs, mut ys) = (Vec::new(), Vec::new());
            for k in 0..=n {
                let x = lo + (hi - lo) * k as f64 / n as f64;
                let v = nums(&args, Some(C::new(x, 0.0)));
                let m = v.len();
                if let Some(y) = value(kind, &v[..m - 1], v[m - 1], deriv) {
                    if y.im.abs() <= 1e-9 * y.re.abs().max(1.0) && y.re.abs() < 1e6 {
                        xs.push(x);
                        ys.push(y.re);
                    }
                }
            }
            if xs.len() < 20 {
                return None;
            }
            let pl = crate::plot::lines(vec![(fname.as_str(), xs, ys)], "x", &fname);
            pods.push(primary(pod("Plot", &format!("Plot for x from {} to {}", num(&lo.to_string()), num(&hi.to_string())), plot(pl))));
        }
        _ => return None,
    }
    pods.push(pod("Equation", "Differential equation", text(format!("{} (y(0) = 1{})", kind.equation(), if kind == Kind::T { ", y′(0) = 0" } else { "" }))));
    Some(Answer::new(call, pods))
}
