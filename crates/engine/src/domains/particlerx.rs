//! Particle reactions: conservation of charge, baryon number, lepton family
//! numbers, strangeness, charm and bottomness, the Q-value, and with a beam
//! energy the centre-of-mass energy for a fixed target.
//!
//! "pi^- + p -> p + pi^- 3 GeV", "K^- + p -> p + K^-", "p + p -> p + p + pi0".

use super::physics::{find_particle, Particle};
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

type P = (&'static Particle, bool);

fn sym(p: P) -> &'static str {
    if p.1 && !p.0.anti_sym.is_empty() {
        p.0.anti_sym
    } else {
        p.0.sym
    }
}

fn sgn(p: P) -> i32 {
    if p.1 && !p.0.anti_sym.is_empty() { -1 } else { 1 }
}

/// Charge in units of e (hadrons, leptons and nuclei are whole numbers).
fn charge(p: P) -> Option<i32> {
    let q: i32 = p.0.charge.trim_start_matches('+').parse().ok()?;
    Some(q * sgn(p))
}

fn baryon(p: P) -> i32 {
    let b = match (p.0.class, p.0.names[0]) {
        ("baryon", _) => 1,
        ("nucleus", "deuteron") => 2,
        ("nucleus", "alpha particle") => 4,
        ("nucleus", _) => 3,
        _ => 0,
    };
    b * sgn(p)
}

/// Lepton family numbers (Lₑ, L_μ, L_τ).
fn leptons(p: P) -> [i32; 3] {
    if p.0.class != "lepton" {
        return [0; 3];
    }
    let n = p.0.names[0];
    let i = if n.contains("tau") { 2 } else if n.contains("mu") { 1 } else { 0 };
    let mut l = [0; 3];
    l[i] = sgn(p);
    l
}

/// Net flavour (S, C, B′) from the valence quarks; None for a flavour mixture (K⁰_S, K⁰_L).
fn flavour(p: P) -> Option<[i32; 3]> {
    let c = p.0.content;
    if c.contains("s̄ − s d̄") || c.contains("s̄ + s d̄") {
        return None;
    }
    if c.contains('(') {
        return Some([0; 3]);
    }
    let mut f = [0; 3];
    for t in c.split_whitespace() {
        let anti = t.ends_with('\u{0304}');
        let q = t.trim_end_matches('\u{0304}');
        // strangeness and bottomness are −1 for the quark, charm +1
        let (i, v) = match q {
            "s" => (0, -1),
            "c" => (1, 1),
            "b" => (2, -1),
            _ => continue,
        };
        f[i] += if anti { -v } else { v };
    }
    Some(f.map(|v| v * sgn(p)))
}

fn side(s: &str) -> Option<Vec<P>> {
    let mut out = Vec::new();
    for t in re!(r"\s+\+\s+").split(s.trim()) {
        let t = t.trim();
        if t.is_empty() {
            return None;
        }
        // "2 pi0", "2pi0"
        let (n, name) = match re!(r"^(\d)\s*([a-zA-Zπμτνγ].*)$").captures(t) {
            Some(m) => (m[1].parse::<usize>().ok()?, m[2].to_string()),
            None => (1, t.to_string()),
        };
        let p = match name.as_str() {
            "p" | "p+" | "p^+" => find_particle("proton")?,
            "n" | "n0" | "n^0" => find_particle("neutron")?,
            "e" | "e-" | "e^-" => find_particle("electron")?,
            "gamma" | "γ" => find_particle("photon")?,
            _ => find_particle(&name)?,
        };
        if p.0.class == "quark" || p.0.class.contains("hypothetical") {
            return None;
        }
        for _ in 0..n {
            out.push(p);
        }
    }
    (!out.is_empty()).then_some(out)
}

fn total<const N: usize>(ps: &[P], f: impl Fn(P) -> Option<[i32; N]>) -> Option<[i32; N]> {
    let mut t = [0; N];
    for p in ps {
        let v = f(*p)?;
        for i in 0..N {
            t[i] += v[i];
        }
    }
    Some(t)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.raw.trim().trim_end_matches(['?', '.']).trim();
    let m = re!(r"(?i)^(?:(?:is|check)\s+)?(?:the\s+)?(?:reaction\s+)?(.+?)\s*(?:->|→|-->|=>)\s*(.+?)(?:\s*(?:,|at|with)?\s+(\d+(?:\.\d+)?)\s*(gev|mev|tev)(?:\s+beam)?)?(?:\s+(?:possible|allowed))?$").captures(l)?;
    let (ini, fin) = (side(&m[1])?, side(&m[2])?);
    let show = |ps: &[P]| ps.iter().map(|p| sym(*p)).collect::<Vec<_>>().join(" + ");
    let reaction = format!("{} → {}", show(&ini), show(&fin));
    let mut rows = Vec::new();
    let mut ok = true;
    let check = |rows: &mut Vec<Vec<Content>>, ok: &mut bool, name: &str, a: Option<String>, b: Option<String>| {
        let (cell, good) = match (&a, &b) {
            (Some(a), Some(b)) => (if a == b { "conserved" } else { "violated" }, a == b),
            _ => ("not defined (flavour mixture)", true),
        };
        *ok &= good;
        rows.push(vec![text(name), text(a.unwrap_or("—".into())), text(b.unwrap_or("—".into())), text(cell)]);
    };
    let fmt1 = |v: Option<[i32; 1]>| v.map(|v| v[0].to_string());
    let c1 = |ps: &[P]| total(ps, |p| charge(p).map(|c| [c]));
    check(&mut rows, &mut ok, "electric charge", fmt1(c1(&ini)), fmt1(c1(&fin)));
    let b1 = |ps: &[P]| total(ps, |p| Some([baryon(p)]));
    check(&mut rows, &mut ok, "baryon number", fmt1(b1(&ini)), fmt1(b1(&fin)));
    let (li, lf) = (total(&ini, |p| Some(leptons(p)))?, total(&fin, |p| Some(leptons(p)))?);
    for (i, n) in ["electron lepton number", "muon lepton number", "tau lepton number"].iter().enumerate() {
        check(&mut rows, &mut ok, n, Some(li[i].to_string()), Some(lf[i].to_string()));
    }
    let (fi, ff) = (total(&ini, flavour), total(&fin, flavour));
    let mut weak_only = false;
    for (i, n) in ["strangeness", "charm", "bottomness"].iter().enumerate() {
        let (a, b) = (fi.map(|v| v[i]), ff.map(|v| v[i]));
        if let (Some(a), Some(b)) = (a, b) {
            if a != b {
                // flavour can change by one unit in a weak interaction
                weak_only = true;
                rows.push(vec![text(*n), text(a.to_string()), text(b.to_string()), text(if (a - b).abs() == 1 { "violated (allowed only in weak interactions)" } else { "violated" })]);
                if (a - b).abs() != 1 {
                    ok = false;
                }
                continue;
            }
        }
        check(&mut rows, &mut ok, n, a.map(|v| v.to_string()), b.map(|v| v.to_string()));
    }
    // neutrino masses are only bounded: take them as zero
    let mass = |ps: &[P]| ps.iter().map(|p| if p.0.mass.is_nan() && p.0.class == "lepton" { 0.0 } else { p.0.mass }).sum::<f64>();
    let (mi, mf) = (mass(&ini), mass(&fin));
    let qv = mi - mf;
    let has_lepton = ini.iter().chain(&fin).any(|p| p.0.class == "lepton");
    let verdict = if !ok {
        "forbidden: a conservation law is violated".to_string()
    } else if weak_only {
        "allowed only as a weak interaction (flavour changes)".to_string()
    } else if has_lepton {
        "allowed by the conservation laws (weak or electromagnetic)".to_string()
    } else {
        "allowed by the conservation laws (strong interaction)".to_string()
    };
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{reaction}{}", m.get(3).map_or(String::new(), |e| format!(" | beam energy {} {}", e.as_str(), m[4].to_uppercase().replace("EV", "eV")))))),
        primary(pod("Result", "Conservation laws", text(verdict))),
        pod("Laws", "Quantum numbers", table(Some(vec!["quantity", "initial", "final", "status"]), rows)),
    ];
    let mut kin = Vec::new();
    if mi.is_finite() && mf.is_finite() {
        kin.push(vec![text("initial rest mass"), text(format!("{} MeV/c²", fmt_float(mi, 7)))]);
        kin.push(vec![text("final rest mass"), text(format!("{} MeV/c²", fmt_float(mf, 7)))]);
        kin.push(vec![text("Q-value"), text(format!("{} MeV", fmt_float(qv, 6)))]);
        if ini.len() == 2 && qv < 0.0 {
            // fixed-target threshold: E_beam,kinetic = ((Σm_f)² − (m₁ + m₂)²) / (2 m₂)
            let (m1, m2) = (ini[0].0.mass, ini[1].0.mass);
            let t = (mf * mf - (m1 + m2).powi(2)) / (2.0 * m2);
            kin.push(vec![text("threshold beam kinetic energy (fixed target)"), text(format!("{} MeV", fmt_float(t, 6)))]);
        }
    }
    if let (Some(e), 2) = (m.get(3), ini.len()) {
        let unit = match m[4].to_lowercase().as_str() {
            "mev" => 1.0,
            "tev" => 1e6,
            _ => 1e3,
        };
        let eb = e.as_str().parse::<f64>().ok()? * unit;
        let (m1, m2) = (ini[0].0.mass, ini[1].0.mass);
        if eb > m1 && m1.is_finite() && m2.is_finite() {
            // the beam energy is the total lab energy of the first particle; the second is at rest
            let s = m1 * m1 + m2 * m2 + 2.0 * eb * m2;
            let p = (eb * eb - m1 * m1).sqrt();
            let gev = |v: f64| format!("{} GeV", fmt_float(v / 1e3, 6));
            kin.push(vec![text("beam momentum"), text(format!("{}/c", gev(p)))]);
            kin.push(vec![text("beam kinetic energy"), text(gev(eb - m1))]);
            kin.push(vec![text("centre-of-mass energy √s"), text(gev(s.sqrt()))]);
            let gamma = (eb + m2) / s.sqrt();
            kin.push(vec![text("centre-of-mass Lorentz factor γ"), text(fmt_float(gamma, 6))]);
            kin.push(vec![text("centre-of-mass rapidity"), text(fmt_float(((eb + m2 + p) / (eb + m2 - p)).ln() / 2.0, 6))]);
            if s.sqrt() < mf {
                pods.push(pod("Energy", "Energy", text("below threshold: the centre-of-mass energy is less than the final rest mass")));
            }
        }
    }
    if !kin.is_empty() {
        pods.push(pod("Kinematics", "Energetics (beam on a target at rest)", table(None, kin)));
    }
    Some(Answer::new(reaction, pods))
}
