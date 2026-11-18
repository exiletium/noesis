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
