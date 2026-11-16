//! Kinetic theory and cosmology calculators: the Maxwell–Boltzmann speed
//! distribution of a gas at a temperature, the Bose–Einstein condensation
//! temperature of an atomic gas, and the state of a flat ΛCDM universe
//! (Planck 2018 parameters) at a cosmic time or a redshift.

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

const KB: f64 = 1.380649e-23;
const HBAR: f64 = 1.054571817e-34;
const AMU: f64 = 1.66053906660e-27;
const C: f64 = 299_792_458.0;
const ZETA_3_2: f64 = 2.612375348685488;

fn n(v: f64) -> String {
    fmt_float(v, 6)
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

/// Temperature in kelvin from "800 K", "25 °C", "70 F".
fn parse_temperature(s: &str) -> Option<f64> {
    let m = re!(r"(-?[\d.]+(?:e-?\d+)?)\s*(k|kelvin|°c|degrees? c(?:elsius)?|c|celsius|°f|degrees? f(?:ahrenheit)?|f|fahrenheit|mk|µk|uk|nk)\b").captures(s)?;
    let v: f64 = m[1].parse().ok()?;
    let u = &m[2];
    Some(if u.contains('f') {
        (v - 32.0) * 5.0 / 9.0 + 273.15
    } else if u.contains('c') {
        v + 273.15
    } else if u == "mk" {
        v * 1e-3
    } else if u == "µk" || u == "uk" {
        v * 1e-6
    } else if u == "nk" {
        v * 1e-9
    } else {
        v
    })
}

const DIATOMIC: &[&str] = &["H", "N", "O", "F", "Cl", "Br", "I"];

/// Molar mass (g/mol) and a display name for a gas named by element,
/// compound name or formula. Elemental H, N, O and the halogens are diatomic.
fn gas(name: &str) -> Option<(f64, String)> {
    let name = name.trim().trim_start_matches("the ").trim_end_matches(" gas").trim();
    if let Some(e) = crate::domains::chem::find_element(name) {
        return Some(if DIATOMIC.contains(&e.symbol.as_str()) {
            (2.0 * e.mass, format!("{} ({}₂)", e.name.to_lowercase(), e.symbol))
        } else {
            (e.mass, e.name.to_lowercase())
        });
    }
    let original = name;
    if let Some(c) = crate::domains::chem::find_db_compound(name) {
        let counts = crate::domains::chem::parse_formula(c.formula)?;
        return Some((crate::domains::chem::molar_mass(&counts), c.name.to_string()));
    }
    if original.chars().any(|c| c.is_ascii_uppercase()) {
        let counts = crate::domains::chem::parse_formula(original)?;
        return Some((crate::domains::chem::molar_mass(&counts), original.to_string()));
    }
    None
}

fn maxwell(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let m = re!(r"^(?:msd|maxwell(?:[- ]boltzmann)? (?:speed |velocity )?distribution|speed distribution|molecular speeds?|rms speed|average speed|mean speed|most probable speed)\s+(?:of |for )?(?:an? )?(.+?)(?:\s+(?:at|t\s*=)\s*(.+))?$").captures(l)?;
    // gas names keep their case (formulas such as CO2)
    let off = m.get(1)?.start();
    let gas_src = q.raw.trim().get(off..).unwrap_or("");
    let gas_src = gas_src.split(" at ").next().unwrap_or(gas_src).split(" T=").next().unwrap_or(gas_src);
    let t = m.get(2).and_then(|t| parse_temperature(t.as_str())).unwrap_or(298.15);
    let (mm, name) = gas(gas_src).or_else(|| gas(&m[1]))?;
    let mass = mm * 1e-3 / 6.02214076e23;
    let vp = (2.0 * KB * t / mass).sqrt();
    let mean = (8.0 * KB * t / (std::f64::consts::PI * mass)).sqrt();
    let rms = (3.0 * KB * t / mass).sqrt();
    let f = |v: f64| 4.0 * std::f64::consts::PI * (mass / (2.0 * std::f64::consts::PI * KB * t)).powf(1.5) * v * v * (-mass * v * v / (2.0 * KB * t)).exp();
    let xs: Vec<f64> = (0..=200).map(|i| i as f64 * 4.0 * vp / 200.0).collect();
    let ys: Vec<f64> = xs.iter().map(|&v| f(v)).collect();
    let rows = vec![
        vec![text("most probable speed"), text(format!("{} m/s", n(vp)))],
        vec![text("mean speed"), text(format!("{} m/s", n(mean)))],
        vec![text("root-mean-square speed"), text(format!("{} m/s", n(rms)))],
        vec![text("mean kinetic energy per molecule"), text(format!("{} J = {} eV", n(1.5 * KB * t), n(1.5 * KB * t / 1.602176634e-19)))],
        vec![text("peak probability density"), text(format!("{} s/m", n(f(vp))))],
    ];
    Some(Answer::new(&format!("Maxwell speed distribution of {name}"), vec![
        input(format!("Maxwell–Boltzmann speed distribution | gas: {name} (M = {} g/mol) | temperature: {} K", n(mm), n(t))),
        pod("Formula", "Distribution", text("f(v) = 4π (m/(2π k_B T))^(3/2) v² exp(−m v²/(2 k_B T))")),
        primary(pod("Result", "Characteristic speeds", table(None, rows))),
        pod("Plot", "Probability density (s/m) against speed (m/s)", plot(crate::plot::lines(vec![("f(v)", xs, ys)], "speed (m/s)", "f(v) (s/m)"))),
    ]))
}

/// Isotope masses (u) of the atoms usually condensed.
const BEC_ATOMS: &[(&str, u32, f64)] = &[
    ("H", 1, 1.00782503), ("He", 4, 4.00260325), ("Li", 7, 7.01600344), ("Na", 23, 22.98976928), ("K", 39, 38.96370649),
    ("K", 41, 40.96182526), ("Ca", 40, 39.96259086), ("Cr", 52, 51.94050623), ("Rb", 87, 86.90918053), ("Rb", 85, 84.91178974),
    ("Sr", 84, 83.9134191), ("Cs", 133, 132.90545196), ("Dy", 164, 163.9291819), ("Er", 168, 167.9323767), ("Yb", 174, 173.9388664),
];

fn bec(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let m = re!(r"^(?:bose[- ]einstein )?(?:condensation|bec|critical) temperature (?:of |for )?(?:an? )?([a-z]+)(?:[- ]?(\d+))?(?:\s+(?:gas|atoms?))?(?:,?\s+(?:at |with )?(?:a )?(?:density|n)\s*(?:of|=)?\s*([\d.]+(?:\s*[x×*]\s*10\^-?\d+|e-?\d+)?)\s*(cm\^-3|/cm\^3|per cm\^3|m\^-3|/m\^3|per m\^3)?)?$").captures(l)?;
    let el = crate::domains::chem::find_element(&m[1]).or_else(|| {
        let s = m[1].to_string();
        crate::domains::chem::by_symbol(&(s.get(..1)?.to_uppercase() + s.get(1..)?))
    })?;
    let want: Option<u32> = m.get(2).and_then(|s| s.as_str().parse().ok());
    let iso = BEC_ATOMS.iter().find(|(s, a, _)| *s == el.symbol && want.map_or(true, |w| w == *a));
    let (a, mass_u) = match (iso, want) {
        (Some((_, a, mu)), _) => (*a, *mu),
        (None, Some(w)) => (w, w as f64),
        (None, None) => (el.mass.round() as u32, el.mass),
    };
    let mut dens = 1e20; // atoms per m³ (10¹⁴ cm⁻³, typical of trapped-atom experiments)
    let mut assumed = true;
    if let Some(d) = m.get(3) {
        let s = d.as_str().replace(' ', "").replace(['×', '*'], "x");
        let v = if let Some((b, e)) = s.split_once("x10^") { b.parse::<f64>().ok().zip(e.parse::<i32>().ok()).map(|(b, e)| b * 10f64.powi(e)) } else { s.parse().ok() };
        if let Some(v) = v {
            dens = if m.get(4).map_or(true, |u| u.as_str().contains("cm")) { v * 1e6 } else { v };
            assumed = false;
        }
    }
    let mass = mass_u * AMU;
    let tc = 2.0 * std::f64::consts::PI * HBAR * HBAR / (mass * KB) * (dens / ZETA_3_2).powf(2.0 / 3.0);
    let lambda = (2.0 * std::f64::consts::PI * HBAR * HBAR / (mass * KB * tc)).sqrt();
    let name = format!("{}-{a}", el.name.to_lowercase());
    let rows = vec![
        vec![text("critical temperature"), text(format!("{} nK = {} K", n(tc * 1e9), n(tc)))],
        vec![text("number density"), text(format!("{} cm⁻³{}", n(dens * 1e-6), if assumed { " (assumed, typical of atom traps)" } else { "" }))],
        vec![text("atomic mass"), text(format!("{} u", n(mass_u)))],
        vec![text("thermal de Broglie wavelength at T_c"), text(format!("{} µm", n(lambda * 1e6)))],
        vec![text("mean interparticle spacing"), text(format!("{} µm", n(dens.powf(-1.0 / 3.0) * 1e6)))],
    ];
    Some(Answer::new(&format!("Bose–Einstein condensation temperature of {name}"), vec![
        input(format!("Bose–Einstein condensation temperature | atom: {name} | density: {} cm⁻³", n(dens * 1e-6))),
        pod("Formula", "Formula", text("T_c = (2π ħ²/(m k_B)) (n/ζ(3/2))^(2/3),  ζ(3/2) ≈ 2.612375")),
        primary(pod("Result", "Result", table(None, rows))),
    ]))
}

// ---- cosmology: flat ΛCDM, Planck 2018 (TT,TE,EE+lowE+lensing+BAO) ----

const H0: f64 = 67.66; // km/s/Mpc
const OM: f64 = 0.3111;
const OR: f64 = 9.14e-5; // photons + three massless neutrino species
const TCMB: f64 = 2.7255;
const MPC_M: f64 = 3.085677581491367e22;
const GYR_S: f64 = 3.15576e16;
const LY_M: f64 = 9.4607304725808e15;

fn ol() -> f64 {
    1.0 - OM - OR
}

fn h0_si() -> f64 {
    H0 * 1e3 / MPC_M
}

/// H(a)/H0
fn e(a: f64) -> f64 {
    (OR / a.powi(4) + OM / a.powi(3) + ol()).sqrt()
}

fn simpson(f: impl Fn(f64) -> f64, a: f64, b: f64, k: usize) -> f64 {
    let h = (b - a) / k as f64;
    let mut s = f(a) + f(b);
    for i in 1..k {
        s += f(a + i as f64 * h) * if i % 2 == 1 { 4.0 } else { 2.0 };
    }
    s * h / 3.0
}

/// Cosmic time (s) at scale factor a; integrated in ln a for early times.
fn age(a: f64) -> f64 {
    // t = ∫ da/(a H) = ∫ d(ln a)/H ; below a = 1e-12 the radiation era gives a²/(2 H0 √Ωr)
    let lo = 1e-12f64.min(a);
    let early = lo * lo / (2.0 * h0_si() * OR.sqrt());
    if a <= lo {
        return a * a / (2.0 * h0_si() * OR.sqrt());
    }
    early + simpson(|x: f64| 1.0 / (h0_si() * e(x.exp())), lo.ln(), a.ln(), 4000)
}

/// Comoving distance (m) to scale factor a.
fn comoving(a: f64) -> f64 {
    simpson(|x: f64| {
        let a = x.exp();
        C / (a * h0_si() * e(a))
    }, a.ln(), 0.0, 4000)
}

fn scale_at_time(t: f64) -> f64 {
    let (mut lo, mut hi) = (-60.0f64, 2.0f64);
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if age(mid.exp()) < t { lo = mid } else { hi = mid }
    }
    ((lo + hi) / 2.0).exp()
}

fn fmt_time(s: f64) -> String {
    let yr = 3.15576e7;
    if s < 60.0 {
        format!("{} seconds", n(s))
    } else if s < 3600.0 * 24.0 {
        format!("{} minutes", n(s / 60.0))
    } else if s < yr * 1e3 {
        format!("{} years", n(s / yr))
    } else if s < yr * 1e6 {
        format!("{} thousand years", n(s / yr / 1e3))
    } else if s < yr * 1e9 {
        format!("{} million years", n(s / yr / 1e6))
    } else {
        format!("{} billion years", n(s / GYR_S))
    }
}

fn era(a: f64) -> &'static str {
    let t = TCMB / a;
    if t > 1e10 {
        "radiation era: before big bang nucleosynthesis completes"
    } else if t > 1e8 {
        "radiation era: big bang nucleosynthesis"
    } else if OR / a > OM {
        "radiation-dominated era"
    } else if a < 1.0 / 1090.0 {
        "matter-dominated era, before recombination"
    } else if a < 1.0 / 30.0 {
        "matter-dominated era: cosmic dark ages / first stars"
    } else if OM / a.powi(3) > ol() {
        "matter-dominated era: galaxy formation"
    } else {
        "dark-energy-dominated era"
    }
}

fn universe_at(a: f64, label: String) -> Answer {
    let t = age(a);
    let now = age(1.0);
    let z = 1.0 / a - 1.0;
    let h = H0 * e(a);
    let dc = comoving(a);
    let mut rows = vec![
        vec![text("time since the big bang"), text(fmt_time(t))],
        vec![text("redshift z"), text(n(z))],
        vec![text("scale factor a"), text(n(a))],
        vec![text("CMB temperature"), text(format!("{} K", n(TCMB / a)))],
        vec![text("Hubble parameter"), text(format!("{} km/s/Mpc", n(h)))],
        vec![text("lookback time"), text(fmt_time(now - t))],
    ];
    if z > 1e-9 {
        rows.push(vec![text("comoving distance (light emitted then)"), text(format!("{} billion light years", n(dc / LY_M / 1e9)))]);
        rows.push(vec![text("proper distance then"), text(format!("{} billion light years", n(dc * a / LY_M / 1e9)))]);
    }
    let w = |x: f64| x / (OR / a.powi(4) + OM / a.powi(3) + ol());
    rows.push(vec![text("density fractions (radiation, matter, dark energy)"), text(format!("{}, {}, {}", n(w(OR / a.powi(4))), n(w(OM / a.powi(3))), n(w(ol()))))]);
    rows.push(vec![text("era"), text(era(a))]);
    Answer::new(&label, vec![
        input(label.clone()),
        primary(pod("Result", "Universe at that time", table(None, rows))),
        pod("Assumptions", "Cosmological model", text(format!("flat ΛCDM, H₀ = {H0} km/s/Mpc, Ω_m = {OM}, Ω_Λ = {:.4}, Ω_r = {OR:e}, T_CMB = {TCMB} K (Planck 2018); present age {}", ol(), fmt_time(now)))),
    ])
}

fn cosmology(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    if let Some(m) = re!(r"^(?:the )?(?:universe |cosmos )?(?:at )?([\d.]+(?:e-?\d+)?)\s*(thousand |million |billion |trillion )?(seconds?|s|minutes?|hours?|days?|years?|yrs?|myr|gyr) after the big bang$").captures(l) {
        let v: f64 = m[1].parse().ok()?;
        let scale = match m.get(2).map(|s| s.as_str().trim()) {
            Some("thousand") => 1e3,
            Some("million") => 1e6,
            Some("billion") => 1e9,
            Some("trillion") => 1e12,
            _ => 1.0,
        };
        let unit = match &m[3] {
            u if u.starts_with('s') => 1.0,
            u if u.starts_with("min") => 60.0,
            u if u.starts_with('h') => 3600.0,
            u if u.starts_with('d') => 86400.0,
            "myr" => 3.15576e13,
            "gyr" => GYR_S,
            _ => 3.15576e7,
        };
        let t = v * scale * unit;
        if t >= age(1.0) * 1.0000001 {
            return None;
        }
        let a = scale_at_time(t);
        return Some(universe_at(a, format!("universe {} after the big bang", fmt_time(t))));
    }
    if let Some(m) = re!(r"^(?:(?:age of )?(?:the )?universe|cosmic time|age) at (?:a )?(?:redshift )?(?:z\s*=\s*)?([\d.]+)$").captures(l) {
        let z: f64 = m[1].parse().ok()?;
        return Some(universe_at(1.0 / (1.0 + z), format!("universe at redshift z = {}", n(z))));
    }
    if re!(r"^(?:what is )?(?:the )?(?:current )?age of the universe$").is_match(l) {
        return Some(universe_at(1.0, "universe today".into()));
    }
    None
}

pub fn handle(q: &Query) -> Option<Answer> {
    maxwell(q).or_else(|| bec(q)).or_else(|| cosmology(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosmic_ages() {
        let now = age(1.0) / GYR_S;
        assert!((now - 13.79).abs() < 0.03, "{now}");
        // recombination z ≈ 1090 at ≈ 370 thousand years
        let t = age(1.0 / 1091.0) / 3.15576e7;
        assert!((t - 3.7e5).abs() < 2e4, "{t}");
        let a = scale_at_time(1e9 * 3.15576e7);
        assert!((1.0 / a - 1.0 - 5.7).abs() < 0.2, "{}", 1.0 / a - 1.0);
    }
}
