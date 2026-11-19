//! Electronic (collision) stopping power of charged particles in matter:
//! the Bethe formula for heavy particles and the ICRU 37 (Rohrlich–Carlson)
//! formula for electrons, with NIST mean excitation energies.
//!
//! "stopping power air 0.5MeV electron",
//! "stopping power of lead against 5 MeV alpha particles".

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

const ME: f64 = 0.510_998_95; // MeV
const K: f64 = 0.307_075; // 4π N_A r_e² m_e c², MeV cm²/mol

/// names | display | Z/A | I (eV) | density g/cm³ | effective Z (for bremsstrahlung)
const MATERIALS: &[(&[&str], &str, f64, f64, f64, f64)] = &[
    (&["air", "dry air"], "dry air (sea level)", 0.49919, 85.7, 1.20479e-3, 7.3),
    (&["water", "liquid water", "h2o"], "liquid water", 0.55508, 75.0, 1.0, 7.4),
    (&["beryllium", "be"], "beryllium", 4.0 / 9.012182, 63.7, 1.848, 4.0),
    (&["aluminum", "aluminium", "al"], "aluminum", 13.0 / 26.981538, 166.0, 2.699, 13.0),
    (&["silicon", "si"], "silicon", 14.0 / 28.0855, 173.0, 2.33, 14.0),
    (&["titanium", "ti"], "titanium", 22.0 / 47.867, 233.0, 4.54, 22.0),
    (&["iron", "fe", "steel"], "iron", 26.0 / 55.845, 286.0, 7.874, 26.0),
    (&["nickel", "ni"], "nickel", 28.0 / 58.6934, 311.0, 8.902, 28.0),
    (&["copper", "cu"], "copper", 29.0 / 63.546, 322.0, 8.96, 29.0),
    (&["silver", "ag"], "silver", 47.0 / 107.8682, 470.0, 10.5, 47.0),
    (&["tungsten", "w"], "tungsten", 74.0 / 183.84, 727.0, 19.3, 74.0),
    (&["gold", "au"], "gold", 79.0 / 196.96655, 790.0, 19.32, 79.0),
    (&["lead", "pb"], "lead", 82.0 / 207.2, 823.0, 11.35, 82.0),
    (&["uranium", "u"], "uranium", 92.0 / 238.0289, 890.0, 18.95, 92.0),
    (&["polyethylene", "polythene"], "polyethylene", 0.57034, 57.4, 0.94, 5.5),
    (&["polystyrene"], "polystyrene", 0.53768, 68.7, 1.06, 5.7),
    (&["pmma", "lucite", "plexiglass", "plexiglas", "acrylic", "perspex"], "PMMA (acrylic)", 0.53937, 74.0, 1.19, 6.5),
    (&["plate glass", "glass", "window glass"], "plate glass", 0.49831, 145.4, 2.4, 11.0),
    (&["concrete", "ordinary concrete"], "ordinary concrete", 0.50274, 135.2, 2.3, 11.0),
];

/// names | display | rest energy MeV | charge number
const PARTICLES: &[(&[&str], &str, f64, f64)] = &[
    (&["electrons", "electron", "beta particles", "beta particle", "betas", "beta", "e-"], "electron", ME, -1.0),
    (&["protons", "proton", "p+"], "proton", 938.272_088, 1.0),
    (&["alpha particles", "alpha particle", "alphas", "alpha", "helium nuclei", "helium nucleus"], "alpha particle", 3727.379_4, 2.0),
    (&["deuterons", "deuteron"], "deuteron", 1875.612_9, 1.0),
    (&["muons", "muon"], "muon", 105.658_38, -1.0),
    (&["pions", "pion"], "pion", 139.570_4, 1.0),
];

fn sig(x: f64) -> String {
    crate::stats::fmt(if x == 0.0 { 0.0 } else { let p = 10f64.powi(3 - x.abs().log10().floor() as i32); (x * p).round() / p })
}

/// Mass collision stopping power (MeV cm²/g) and the log term of the formula.
pub fn collision(t: f64, mass: f64, z: f64, za: f64, i_ev: f64) -> Option<(f64, f64)> {
    let i = i_ev * 1e-6;
    let gamma = 1.0 + t / mass;
    let b2 = 1.0 - 1.0 / (gamma * gamma);
    if !(b2 > 0.0) {
        return None;
    }
    if (mass - ME).abs() < 1e-6 {
        let tau = t / ME;
        let f = 1.0 - b2 + (tau * tau / 8.0 - (2.0 * tau + 1.0) * std::f64::consts::LN_2) / ((tau + 1.0) * (tau + 1.0));
        let br = (tau * tau * (tau + 2.0) / (2.0 * (i / ME).powi(2))).ln() + f;
        Some((K / 2.0 * za / b2 * br, br))
    } else {
        let r = ME / mass;
        let tmax = 2.0 * ME * b2 * gamma * gamma / (1.0 + 2.0 * gamma * r + r * r);
        let br = 0.5 * (2.0 * ME * b2 * gamma * gamma * tmax / (i * i)).ln() - b2;
        Some((K * z * z * za / b2 * br, br))
    }
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    if !re!(r"\bstopping power\b|\blinear energy transfer\b|\bde/dx\b").is_match(&l) {
        return None;
    }
    let e = re!(r"(\d+(?:\.\d+)?(?:e-?\d+)?) ?(ev|kev|mev|gev)\b").captures(&l)?;
    let t = e[1].parse::<f64>().ok()? * match &e[2] { "ev" => 1e-6, "kev" => 1e-3, "mev" => 1.0, _ => 1e3 };
    let rest = l.replacen(&e[0], " ", 1);
    let has = |names: &[&str]| names.iter().any(|n| Regex::new(&format!(r"(?:^|[ ,(]){}(?:$|[ ,)])", regex::escape(n))).map_or(false, |r| r.is_match(&rest)));
    let p = PARTICLES.iter().find(|p| has(p.0))?;
    let mat = MATERIALS.iter().find(|m| has(m.0))?;
    if !(t > 0.0) {
        return None;
    }
    let (s, br) = collision(t, p.2, p.3.abs(), mat.2, mat.3)?;
    if !(s > 0.0) {
        return None;
    }
    let rho = mat.4;
    let lin = s * rho; // MeV/cm
    let gamma = 1.0 + t / p.2;
    let beta = (1.0 - 1.0 / (gamma * gamma)).sqrt();
    let mut rows = vec![
        vec![text("mass stopping power"), text(format!("{} MeV cm²/g", sig(s)))],
        vec![text("linear stopping power"), text(format!("{} MeV/cm = {} keV/µm", sig(lin), sig(lin * 0.1)))],
        vec![text("density"), text(format!("{} g/cm³", sig(rho)))],
        vec![text("Z/A"), text(fmt_float((mat.2 * 1e5).round() / 1e5, 6))],
        vec![text("mean excitation energy I"), text(format!("{} eV", fmt_float(mat.3, 6)))],
        vec![text("particle speed"), text(format!("β = v/c = {} (γ = {})", sig(beta), sig(gamma)))],
    ];
    let electron = p.1 == "electron";
    if !electron {
        let r = ME / p.2;
        let tmax = 2.0 * ME * beta * beta * gamma * gamma / (1.0 + 2.0 * gamma * r + r * r);
        rows.push(vec![text("maximum energy transfer to an electron"), text(format!("{} keV", sig(tmax * 1e3)))]);
    }
    let formula = if electron {
        "S/ρ = (K/2)(Z/A)(1/β²)[ln(τ²(τ+2)/(2(I/mc²)²)) + F(τ)],  F(τ) = 1 − β² + (τ²/8 − (2τ+1) ln 2)/(τ+1)²"
    } else {
        "S/ρ = K z² (Z/A)(1/β²)[½ ln(2mc²β²γ²T_max/I²) − β²],  K = 0.307075 MeV cm²/mol"
    };
    let mut note = String::from("Electronic (collision) stopping power, without the density-effect correction (it lowers values by a few percent for fast electrons in condensed media) or shell corrections.");
    if electron {
        let z = mat.5;
        let ec = if rho < 0.01 { 710.0 / (z + 0.92) } else { 610.0 / (z + 1.24) };
        note.push_str(&format!(" Radiative (bremsstrahlung) losses come on top and equal the collision losses near the critical energy, about {} MeV here.", sig(ec)));
    }
    if br < 3.0 {
        note.push_str(" The logarithmic term is small here, so shell and other low-energy corrections are significant and this value is only a rough estimate.");
    }
    Some(Answer::new(format!("stopping power of {} for {} MeV {}", mat.1, sig(t), p.1), vec![
        pod("Input", "Input interpretation", text(format!("electronic stopping power | {} | {} | kinetic energy {} MeV", mat.1, p.1, sig(t)))),
        primary(pod("Result", "Collision stopping power", text(format!("{} MeV cm²/g ({} MeV/cm)", sig(s), sig(lin))))),
        pod("Properties", "Details", table(None, rows)),
        pod("Formula", if electron { "Formula (ICRU 37, electrons)" } else { "Formula (Bethe)" }, text(formula)),
        pod("Note", "Note", text(note)),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_electron_and_alpha() {
        // ESTAR water 1 MeV collision 1.849 (with density effect); Bethe without it is ~1% higher
        let (s, _) = collision(1.0, ME, 1.0, 0.55508, 75.0).unwrap();
        assert!((s - 1.87).abs() < 0.02, "{s}");
        // ASTAR water 5 MeV alpha ≈ 889 MeV cm²/g; plain Bethe ≈ 3% higher
        let (s, _) = collision(5.0, 3727.3794, 2.0, 0.55508, 75.0).unwrap();
        assert!((s - 916.0).abs() < 10.0, "{s}");
    }
}
