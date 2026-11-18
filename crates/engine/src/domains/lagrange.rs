//! Lagrange points of two-body systems, from the circular restricted
//! three-body problem.
//!
//! "Lagrange points Earth and Moon", "Sun-Earth L2", "lagrange points of
//! the sun and jupiter".

use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// primary | secondary | μ = m₂/(m₁+m₂) | mean separation (km)
const SYSTEMS: &[(&str, &str, f64, f64)] = &[
    ("Sun", "Earth", 3.0404e-6, 149_597_870.7),
    ("Earth", "Moon", 0.012150585, 384_400.0),
    ("Sun", "Mercury", 1.6601e-7, 57_909_050.0),
    ("Sun", "Venus", 2.4478e-6, 108_208_000.0),
    ("Sun", "Mars", 3.2271e-7, 227_939_200.0),
    ("Sun", "Jupiter", 9.5388e-4, 778_570_000.0),
    ("Sun", "Saturn", 2.8559e-4, 1_433_530_000.0),
    ("Sun", "Uranus", 4.3662e-5, 2_872_460_000.0),
    ("Sun", "Neptune", 5.1514e-5, 4_495_060_000.0),
];

/// Root of the collinear-point equation on (lo, hi) by bisection.
fn collinear(mu: f64, lo: f64, hi: f64) -> f64 {
    let f = |x: f64| {
        let r1 = x + mu;
        let r2 = x - 1.0 + mu;
        x - (1.0 - mu) * r1 / r1.abs().powi(3) - mu * r2 / r2.abs().powi(3)
    };
    let (mut a, mut b) = (lo, hi);
    for _ in 0..200 {
        let m = 0.5 * (a + b);
        if f(a) * f(m) <= 0.0 {
            b = m;
        } else {
            a = m;
        }
    }
    0.5 * (a + b)
}

fn km(v: f64) -> String {
    if v >= 1e6 {
        format!("{} million km", fmt_float((v / 1e4).round() / 100.0, 6))
    } else {
        format!("{} km", super::geo::fmt_int((v / 10.0).round() as u64 * 10))
    }
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    if !re!(r"\blagrang(?:e|ian)\b|\bl[1-5]\b").is_match(&l) {
        return None;
    }
    let point: Option<usize> = re!(r"\bl([1-5])\b").captures(&l).and_then(|m| m[1].parse().ok());
    let bodies: Vec<&str> = ["sun", "earth", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune"].into_iter().filter(|b| re_word(&l, b)).collect();
    // "L2" alone is too short to mean anything without a system or "lagrange"
    if !l.contains("lagrang") && bodies.len() < 2 {
        return None;
    }
    let sys = match bodies.len() {
        0 => SYSTEMS[0],
        1 => *SYSTEMS.iter().find(|s| s.1.eq_ignore_ascii_case(bodies[0]))?,
        _ => *SYSTEMS.iter().find(|s| bodies.contains(&s.0.to_lowercase().as_str()) && bodies.contains(&s.1.to_lowercase().as_str()))?,
    };
    let (p, s, mu, a) = sys;
    let x1 = collinear(mu, -mu + 1e-9, 1.0 - mu - 1e-9);
    let x2 = collinear(mu, 1.0 - mu + 1e-9, 2.0);
    let x3 = collinear(mu, -2.0, -mu - 1e-9);
    let d1 = (1.0 - mu - x1) * a;
    let d2 = (x2 - 1.0 + mu) * a;
    let d3 = (-mu - x3) * a;
    let rows = vec![
        vec![text("L1"), text(format!("{} from {s}, toward {p}", km(d1))), text(format!("{} from {p}", km(a - d1)))],
        vec![text("L2"), text(format!("{} from {s}, beyond it", km(d2))), text(format!("{} from {p}", km(a + d2)))],
        vec![text("L3"), text(format!("{} from {p}, opposite {s}", km(d3))), text(format!("{} from {s}", km(a + d3)))],
        vec![text("L4"), text(format!("60° ahead of {s} in its orbit")), text(format!("{} from both", km(a)))],
        vec![text("L5"), text(format!("60° behind {s} in its orbit")), text(format!("{} from both", km(a)))],
    ];
    let result = match point {
        Some(1) => format!("{} from {s} (toward {p})", km(d1)),
        Some(2) => format!("{} from {s} (away from {p})", km(d2)),
        Some(3) => format!("{} from {p} (opposite {s})", km(d3)),
        Some(4) => format!("60° ahead of {s}, {} from both bodies", km(a)),
        Some(5) => format!("60° behind {s}, {} from both bodies", km(a)),
        _ => format!("L1: {} from {s} | L2: {} from {s} | L3: {} from {p} | L4, L5: 60° ahead of and behind {s}", km(d1), km(d2), km(d3)),
    };
    Some(Answer::new(format!("{p}–{s} Lagrange points"), vec![
        pod("Input", "Input interpretation", text(format!("{p}–{s} system | Lagrange points{}", point.map(|n| format!(" | L{n}")).unwrap_or_default()))),
        primary(pod("Result", if point.is_some() { "Location" } else { "Lagrange points" }, text(result))),
        pod("Table", "Positions", table(Some(vec!["point", "location", "distance"]), rows)),
        pod("Parameters", "Parameters", text(format!("mass ratio μ = m₂/(m₁+m₂) = {} | mean separation {}", fmt_float(mu, 6), km(a)))),
    ]))
}

fn re_word(l: &str, w: &str) -> bool {
    l.split(|c: char| !c.is_alphanumeric()).any(|x| x == w)
}
