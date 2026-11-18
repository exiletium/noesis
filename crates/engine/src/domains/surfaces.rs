//! Named parametric surfaces drawn as wireframes: "Dini's surface",
//! "Möbius strip", "Klein bottle", "Enneper surface", "helicoid", "catenoid".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names | display | x, y, z in u and v | u range | v range | properties
#[allow(clippy::type_complexity)]
const SURFACES: &[(&[&str], &str, [&str; 3], (f64, f64), (f64, f64), &[(&str, &str)])] = &[
    (
        &["dini's surface", "dini surface", "dinis surface", "dini's helicoid"],
        "Dini's surface",
        ["cos(u) sin(v)", "sin(u) sin(v)", "cos(v) + log(tan(v/2)) + 0.2 u"],
        (0.0, 12.566370614359172),
        (0.01, 1.2),
        &[("parameters", "a = 1, b = 0.2"), ("Gaussian curvature", "−1/(a² + b²) (constant negative curvature)"), ("description", "a twisted pseudosphere")],
    ),
    (
        &["mobius strip", "möbius strip", "moebius strip", "mobius band", "möbius band", "moebius band"],
        "Möbius strip",
        ["(1 + v/2 cos(u/2)) cos(u)", "(1 + v/2 cos(u/2)) sin(u)", "v/2 sin(u/2)"],
        (0.0, 6.283185307179586),
        (-1.0, 1.0),
        &[("orientable", "no"), ("Euler characteristic", "0"), ("boundary components", "1"), ("sides", "1")],
    ),
    (
        &["klein bottle", "klein's bottle", "figure 8 klein bottle", "figure-8 klein bottle"],
        "Klein bottle (figure-8 immersion)",
        ["(2 + cos(u/2) sin(v) - sin(u/2) sin(2 v)) cos(u)", "(2 + cos(u/2) sin(v) - sin(u/2) sin(2 v)) sin(u)", "sin(u/2) sin(v) + cos(u/2) sin(2 v)"],
        (0.0, 6.283185307179586),
        (0.0, 6.283185307179586),
        &[("orientable", "no"), ("Euler characteristic", "0"), ("boundary components", "0 (closed surface)"), ("genus (non-orientable)", "2")],
    ),
    (
        &["enneper surface", "enneper's surface", "enneper minimal surface"],
        "Enneper surface",
        ["u - u^3/3 + u v^2", "v - v^3/3 + v u^2", "u^2 - v^2"],
        (-2.0, 2.0),
        (-2.0, 2.0),
        &[("type", "minimal surface (mean curvature 0)"), ("Gaussian curvature", "−4/(1 + u² + v²)⁴")],
    ),
    (
        &["helicoid"],
        "helicoid",
        ["v cos(u)", "v sin(u)", "u"],
        (0.0, 12.566370614359172),
        (-1.0, 1.0),
        &[("type", "ruled minimal surface (mean curvature 0)"), ("Gaussian curvature", "−1/(1 + v²)²")],
    ),
    (
        &["catenoid"],
        "catenoid",
        ["cosh(v) cos(u)", "cosh(v) sin(u)", "v"],
        (0.0, 6.283185307179586),
        (-1.0, 1.0),
        &[("type", "minimal surface of revolution (mean curvature 0)"), ("Gaussian curvature", "−sech⁴(v)")],
    ),
];

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    if let Some(a) = riemann(&l) {
        return Some(a);
    }
    let name = re!(r"^(?:plot |draw |show |graph |3d plot (?:of )?|plot of |picture of |image of |a |the )*(.+?)(?: plot| surface plot| parametrization| parametric equations?)?$").captures(&l)?.get(1)?.as_str().to_string();
    let s = SURFACES.iter().find(|s| s.0.contains(&name.as_str()))?;
    let f: Option<Vec<crate::expr::Expr>> = s.2.iter().map(|e| crate::parse::parse(e).ok().map(|p| p.expr)).collect();
    let f = f?;
    let (u, v) = (crate::expr::sym("u"), crate::expr::sym("v"));
    let pl = crate::plot::parametric_surface([&f[0], &f[1], &f[2]], &u, &v, s.3, s.4)?;
    let mut rows = vec![
        vec![text("x(u, v)"), text(s.2[0])],
        vec![text("y(u, v)"), text(s.2[1])],
        vec![text("z(u, v)"), text(s.2[2])],
    ];
    let fmt = |x: f64| crate::print::fmt_float((x * 1000.0).round() / 1000.0, 5);
    rows.push(vec![text("range"), text(format!("{} ≤ u ≤ {}, {} ≤ v ≤ {}", fmt(s.3 .0), fmt(s.3 .1), fmt(s.4 .0), fmt(s.4 .1)))]);
    Some(Answer::new(s.1, vec![
        pod("Input", "Input interpretation", text(format!("{} | parametric surface", s.1))),
        primary(pod("Plot", "Surface (wireframe)", plot(pl))),
        pod("Parametrization", "Parametric equations", table(None, rows)),
        pod("Properties", "Properties", table(None, s.5.iter().map(|(k, v)| vec![text(*k), text(*v)]).collect())),
    ]))
}

/// Riemann surfaces of log z and z^(p/q): the imaginary part of the continued
/// function over polar coordinates z = u e^(iv), followed around all its sheets.
fn riemann(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:plot |draw |show )?(?:the )?riemann surfaces? (?:of |for )?(.+)$").captures(l)?;
    let f = m[1].replace(' ', "");
    let (name, z, ur, vr, sheets, branch) = match f.as_str() {
        "logz" | "log(z)" | "lnz" | "ln(z)" => ("log z", "v".to_string(), (0.05, 2.0), (-2.0 * std::f64::consts::PI, 2.0 * std::f64::consts::PI), "infinitely many".to_string(), "z = 0 and z = ∞ (logarithmic)"),
        _ => {
            let (p, q): (u32, u32) = if f == "sqrt(z)" || f == "sqrtz" || f == "√z" {
                (1, 2)
            } else if f == "cbrt(z)" || f == "cbrtz" {
                (1, 3)
            } else {
                let c = re!(r"^z\^\(?(\d+)/(\d+)\)?$").captures(&f)?;
                (c[1].parse().ok()?, c[2].parse().ok()?)
            };
            let g = { let (mut a, mut b) = (p, q); while b != 0 { let t = a % b; a = b; b = t; } a };
            let (p, q) = (p / g, q / g);
            if q < 2 || q > 6 || p == 0 || p > 12 {
                return None;
            }
            let name = if (p, q) == (1, 2) { "√z".to_string() } else { format!("z^({p}/{q})") };
            (
                Box::leak(name.into_boxed_str()) as &str,
                format!("u^({p}/{q}) sin({p} v/{q})"),
                (0.0, 1.5),
                (0.0, 2.0 * std::f64::consts::PI * q as f64),
                q.to_string(),
                "z = 0 and z = ∞ (algebraic)",
            )
        }
    };
    let xs = ["u cos(v)".to_string(), "u sin(v)".to_string(), z.clone()];
    let fx: Option<Vec<crate::expr::Expr>> = xs.iter().map(|e| crate::parse::parse(e).ok().map(|p| p.expr)).collect();
    let fx = fx?;
    let (u, v) = (crate::expr::sym("u"), crate::expr::sym("v"));
    let pl = crate::plot::parametric_surface([&fx[0], &fx[1], &fx[2]], &u, &v, ur, vr)?;
    let fmt = |x: f64| crate::print::fmt_float((x * 1000.0).round() / 1000.0, 5);
    Some(Answer::new(format!("Riemann surface of {name}"), vec![
        pod("Input", "Input interpretation", text(format!("Riemann surface | {name}"))),
        primary(pod("Plot", &format!("Im({name}) over z = u e^(iv)"), plot(pl))),
        pod("Properties", "Properties", table(None, vec![
            vec![text("sheets"), text(sheets)],
            vec![text("branch points"), text(branch)],
            vec![text("height"), text(format!("Im({name}) = {z}"))],
            vec![text("range"), text(format!("{} ≤ |z| ≤ {}, {} ≤ arg z ≤ {}", fmt(ur.0), fmt(ur.1), fmt(vr.0), fmt(vr.1)))],
        ])),
    ]))
}
