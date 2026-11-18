//! Coulomb wave functions F_L(η, ρ), G_L(η, ρ) and H±_L = G ± iF, with
//! derivatives, by Steed's method (two continued fractions and the Wronskian).
//!
//! "incoming irregular Coulomb wave 2, .2, 3", "plot Coulomb wave of first type 3, 0.12, x",
//! "derivative irregular Coulomb wave 5, 0.46, u", "CoulombF[3, 0.12, 2]".

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

/// (F, F′, G, G′) for real L ≥ 0, real η and ρ > 0.
pub fn coulomb(l: f64, eta: f64, rho: f64) -> Option<(f64, f64, f64, f64)> {
    if !(rho > 0.0) || l < 0.0 {
        return None;
    }
    const TINY: f64 = 1e-300;
    // CF1: f = F′/F, tracking the sign of F
    let s = |k: f64| k / rho + eta / k;
    let r2 = |k: f64| 1.0 + eta * eta / (k * k);
    let mut f = s(l + 1.0);
    if f == 0.0 {
        f = TINY;
    }
    let (mut c, mut d, mut sign) = (f, 0.0f64, 1.0f64);
    let mut j = 1.0;
    loop {
        let a = -r2(l + j);
        let b = s(l + j) + s(l + j + 1.0);
        d = b + a * d;
        if d == 0.0 {
            d = TINY;
        }
        c = b + a / c;
        if c == 0.0 {
            c = TINY;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;
        if d < 0.0 {
            sign = -sign;
        }
        if (delta - 1.0).abs() < 1e-16 {
            break;
        }
        j += 1.0;
        if j > 2e5 {
            return None;
        }
    }
    // CF2: p + iq = H⁺′/H⁺ (a seed of 1e-30: complex division by 1e-300 underflows |b|²)
    const CTINY: f64 = 1e-30;
    let (a, b) = (C::new(1.0 + l, eta), C::new(-l, eta));
    let (mut ft, mut c2, mut d2) = (C::new(CTINY, 0.0), C::new(CTINY, 0.0), C::new(0.0, 0.0));
    let mut n = 1.0;
    loop {
        let an = (a + (n - 1.0)) * (b + (n - 1.0));
        let bn = C::new(2.0 * (rho - eta), 2.0 * n);
        d2 = bn + an * d2;
        if d2.norm() == 0.0 {
            d2 = C::new(CTINY, 0.0);
        }
        c2 = bn + an / c2;
        if c2.norm() == 0.0 {
            c2 = C::new(CTINY, 0.0);
        }
        d2 = C::new(1.0, 0.0) / d2;
        let dl = c2 * d2;
        ft *= dl;
        if (dl - 1.0).norm() < 1e-15 {
            break;
        }
        n += 1.0;
        if n > 2e5 {
            return None;
        }
    }
    let pq = C::new(0.0, 1.0 - eta / rho) + C::new(0.0, 1.0 / rho) * ft;
    let (p, q) = (pq.re, pq.im);
    if q == 0.0 {
        return None;
    }
    let gam = (f - p) / q;
    let ff = sign / (q * (1.0 + gam * gam)).sqrt();
    let g = gam * ff;
    let out = (ff, f * ff, g, p * g - q * ff);
    [out.0, out.1, out.2, out.3].iter().all(|v| v.is_finite()).then_some(out)
}

#[derive(Clone, Copy, PartialEq)]
enum Which {
    F,
    G,
    HPlus,
    HMinus,
}

fn pick(w: Which, deriv: bool, v: (f64, f64, f64, f64)) -> C {
    let (f, g) = if deriv { (v.1, v.3) } else { (v.0, v.2) };
    match w {
        Which::F => C::new(f, 0.0),
        Which::G => C::new(g, 0.0),
        Which::HPlus => C::new(g, f),
        Which::HMinus => C::new(g, -f),
    }
}

fn show(z: C) -> String {
    let r = |v: f64| fmt_float(v, 9);
    if z.im == 0.0 {
        r(z.re)
    } else {
        format!("{} {} {} i", r(z.re), if z.im < 0.0 { "-" } else { "+" }, r(z.im.abs()))
    }
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.raw.trim().trim_end_matches(['?', '.']).trim().to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    if !l.contains("coulomb") {
        return None;
    }
    let m = re!(r"^(?:(plot|graph|evaluate|compute|calculate|find)\s+)?(?:the\s+)?(derivative (?:of (?:the )?)?)?((?:regular|irregular|incoming|outgoing|incoming irregular|outgoing irregular)\s+)?coulomb ?(?:wave)?(?: function)?(?:s)?(?: of (?:the )?(first|second) (?:type|kind))?\s*(f|g|h1|h2|hplus|hminus)?\s*(prime|'|′|derivative)?\s*[\[(]?\s*([^\[\]()]+?)\s*[\])]?$").captures(&l)?;
    let kind_word = m.get(3).map_or("", |x| x.as_str().trim());
    let which = match (m.get(5).map(|x| x.as_str()), kind_word, m.get(4).map(|x| x.as_str())) {
        (Some("f"), ..) => Which::F,
        (Some("g"), ..) => Which::G,
        (Some("h1" | "hplus"), ..) => Which::HPlus,
        (Some("h2" | "hminus"), ..) => Which::HMinus,
        (_, k, _) if k.starts_with("incoming") => Which::HMinus,
        (_, k, _) if k.starts_with("outgoing") => Which::HPlus,
        (_, "irregular", _) => Which::G,
        (_, _, Some("second")) => Which::G,
        _ => Which::F,
    };
    let deriv = m.get(2).is_some() || m.get(6).is_some();
    let args: Vec<&str> = m[7].split(',').map(|s| s.trim()).collect();
    if args.len() != 3 {
        return None;
    }
    let is_var = |s: &str| matches!(s, "x" | "r" | "rho" | "ρ" | "u" | "z");
    let num = |s: &str| s.parse::<f64>().ok();
    let (lv, eta) = (num(args[0])?, num(args[1])?);
    let (name, title) = match which {
        Which::F => ("CoulombF", "regular Coulomb wave function F"),
        Which::G => ("CoulombG", "irregular Coulomb wave function G"),
        Which::HPlus => ("CoulombH1", "outgoing irregular Coulomb wave function H⁺ = G + iF"),
        Which::HMinus => ("CoulombH2", "incoming irregular Coulomb wave function H⁻ = G − iF"),
    };
    let fname = format!("{name}{}", if deriv { "Prime" } else { "" });
    let call = format!("{fname}[{}, {}, {}]", args[0], args[1], args[2]);
    let mut pods = vec![pod("Input", "Input", text(format!("{call} | {}{title}", if deriv { "derivative of the " } else { "" })))];
    if is_var(args[2]) {
        if matches!(which, Which::HPlus | Which::HMinus) && !m.get(1).is_some_and(|p| p.as_str().starts_with('p') || p.as_str().starts_with('g')) {
            return None;
        }
        let (lo, hi) = (0.05, 20.0);
        let n = 400;
        let (mut xs, mut ys, mut ys2) = (Vec::new(), Vec::new(), Vec::new());
        for k in 0..=n {
            let x = lo + (hi - lo) * k as f64 / n as f64;
            if let Some(v) = coulomb(lv, eta, x) {
                let y = pick(which, deriv, v);
                if y.norm() < 50.0 {
                    xs.push(x);
                    ys.push(y.re);
                    ys2.push(y.im);
                }
            }
        }
        if xs.len() < 50 {
            return None;
        }
        let mut series = vec![(if which == Which::F || which == Which::G { fname.as_str() } else { "real part" }, xs.clone(), ys)];
        if matches!(which, Which::HPlus | Which::HMinus) {
            series.push(("imaginary part", xs, ys2));
        }
        let pl = crate::plot::lines(series, args[2], &fname);
        pods.push(primary(pod("Plot", &format!("Plot for {} from {} to {}", args[2], fmt_float(lo, 3), fmt_float(hi, 3)), plot(pl))));
    } else {
        let rho = num(args[2])?;
        let v = coulomb(lv, eta, rho)?;
        pods.push(primary(pod("Result", "Result", text(show(pick(which, deriv, v))))));
        let rows = vec![
            vec![text("F (regular)"), text(fmt_float(v.0, 9)), text(fmt_float(v.1, 9))],
            vec![text("G (irregular)"), text(fmt_float(v.2, 9)), text(fmt_float(v.3, 9))],
        ];
        pods.push(pod("Table", "Values", table(Some(vec!["function", "value", "derivative"]), rows)));
    }
    pods.push(pod("Equation", "Differential equation", text("u″ + (1 − 2η/ρ − L(L + 1)/ρ²) u = 0 (Wronskian F′G − FG′ = 1)")));
    Some(Answer::new(call, pods))
}
