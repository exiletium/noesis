//! Physics lookups: elementary and composite particles ("proton",
//! "tau mass", "spin K+", "top antiquark", "muon decay modes",
//! "electron D*0 neutron"), ratios of masses and constants
//! ("mass proton / electron", "Planck mass / mass of electron"), resistor
//! colour codes, laser lines and work functions of the elements.
//!
//! Particle data are the Particle Data Group's 2024 review values; work
//! functions are the CRC Handbook's polycrystalline values.

use super::{chem, units};
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

const INF: f64 = f64::INFINITY;
const NAN: f64 = f64::NAN;
/// MeV/c² in kilograms.
const MEV_KG: f64 = 1.782_661_921_627e-30;

pub struct Particle {
    pub names: &'static [&'static str],
    pub sym: &'static str,
    pub anti_names: &'static [&'static str],
    pub anti_sym: &'static str,
    /// Rest mass in MeV/c²; NaN when only a bound is known.
    pub mass: f64,
    pub mass_note: &'static str,
    pub charge: &'static str,
    pub spin: &'static str,
    /// Mean lifetime in seconds; infinite when stable.
    pub life: f64,
    pub class: &'static str,
    /// Valence quarks, space separated; a trailing macron marks an antiquark.
    pub content: &'static str,
    pub decays: &'static [(&'static str, &'static str)],
}

macro_rules! p {
    ($n:expr, $s:expr, $an:expr, $as:expr, $m:expr, $mn:expr, $q:expr, $j:expr, $t:expr, $c:expr, $qc:expr, $d:expr) => {
        Particle { names: $n, sym: $s, anti_names: $an, anti_sym: $as, mass: $m, mass_note: $mn, charge: $q, spin: $j, life: $t, class: $c, content: $qc, decays: $d }
    };
}

pub static PARTICLES: &[Particle] = &[
    // leptons
    p!(&["electron", "e-"], "e⁻", &["positron", "antielectron", "e+"], "e⁺", 0.510_998_95, "", "-1", "1/2", INF, "lepton", "", &[]),
    p!(&["muon", "mu-", "mu"], "μ⁻", &["antimuon", "mu+"], "μ⁺", 105.658_375_5, "", "-1", "1/2", 2.196_981_1e-6, "lepton", "", &[("e⁻ ν̄ₑ ν_μ", "≈100%"), ("e⁻ ν̄ₑ ν_μ γ", "1.4%"), ("e⁻ ν̄ₑ ν_μ e⁺ e⁻", "3.4×10⁻⁵")]),
    p!(&["tau", "tauon", "tau-", "tau lepton"], "τ⁻", &["antitau", "tau+"], "τ⁺", 1776.86, "", "-1", "1/2", 2.903e-13, "lepton", "", &[("π⁻ π⁰ ν_τ", "25.49%"), ("e⁻ ν̄ₑ ν_τ", "17.82%"), ("μ⁻ ν̄_μ ν_τ", "17.39%"), ("π⁻ ν_τ", "10.82%"), ("π⁻ 2π⁰ ν_τ", "9.26%"), ("π⁻ π⁺ π⁻ ν_τ", "8.99%")]),
    p!(&["electron neutrino", "nu_e"], "νₑ", &["electron antineutrino"], "ν̄ₑ", NAN, "< 0.8 eV/c² (KATRIN 90% CL)", "0", "1/2", INF, "lepton", "", &[]),
    p!(&["muon neutrino", "nu_mu"], "ν_μ", &["muon antineutrino"], "ν̄_μ", NAN, "< 0.19 MeV/c²", "0", "1/2", INF, "lepton", "", &[]),
    p!(&["tau neutrino", "nu_tau"], "ν_τ", &["tau antineutrino"], "ν̄_τ", NAN, "< 18.2 MeV/c²", "0", "1/2", INF, "lepton", "", &[]),
    // quarks (MS-bar masses; top from direct measurements)
    p!(&["up quark", "u quark"], "u", &["up antiquark", "anti-up quark"], "ū", 2.16, "", "+2/3", "1/2", NAN, "quark", "", &[]),
    p!(&["down quark", "d quark"], "d", &["down antiquark"], "d̄", 4.70, "", "-1/3", "1/2", NAN, "quark", "", &[]),
    p!(&["strange quark", "s quark"], "s", &["strange antiquark"], "s̄", 93.5, "", "-1/3", "1/2", NAN, "quark", "", &[]),
    p!(&["charm quark", "c quark"], "c", &["charm antiquark"], "c̄", 1273.0, "", "+2/3", "1/2", NAN, "quark", "", &[]),
    p!(&["bottom quark", "b quark", "beauty quark"], "b", &["bottom antiquark"], "b̄", 4183.0, "", "-1/3", "1/2", NAN, "quark", "", &[]),
    p!(&["top quark", "t quark", "truth quark"], "t", &["top antiquark"], "t̄", 172_570.0, "", "+2/3", "1/2", 4.7e-25, "quark", "", &[("W⁺ b", "≈96%")]),
    // gauge and scalar bosons
    p!(&["photon", "gamma ray photon"], "γ", &[], "", 0.0, "", "0", "1", INF, "gauge boson", "", &[]),
    p!(&["gluon"], "g", &[], "", 0.0, "", "0", "1", NAN, "gauge boson", "", &[]),
    p!(&["w boson", "w+ boson", "w+", "w particle"], "W⁺", &["w- boson", "w-"], "W⁻", 80_369.2, "", "+1", "1", 3.16e-25, "gauge boson", "", &[("hadrons", "67.41%"), ("e⁺ νₑ", "10.71%"), ("μ⁺ ν_μ", "10.63%"), ("τ⁺ ν_τ", "11.38%")]),
    p!(&["z boson", "z0", "z particle", "z0 boson"], "Z⁰", &[], "", 91_187.6, "", "0", "1", 2.64e-25, "gauge boson", "", &[("hadrons", "69.91%"), ("invisible (νν̄)", "20.00%"), ("e⁺ e⁻", "3.363%"), ("μ⁺ μ⁻", "3.366%"), ("τ⁺ τ⁻", "3.370%")]),
    p!(&["higgs boson", "higgs", "higgs particle"], "H⁰", &[], "", 125_200.0, "", "0", "0", 1.6e-22, "scalar boson", "", &[("b b̄", "58.2%"), ("W W*", "21.4%"), ("g g", "8.2%"), ("τ⁺ τ⁻", "6.3%"), ("c c̄", "2.9%"), ("Z Z*", "2.6%"), ("γ γ", "0.23%")]),
    p!(&["graviton"], "G", &[], "", 0.0, "hypothetical; < 1.76×10⁻²³ eV/c²", "0", "2", INF, "hypothetical gauge boson", "", &[]),
    // baryons
    p!(&["proton", "p", "p+"], "p", &["antiproton", "p-", "pbar"], "p̄", 938.272_088_16, "", "+1", "1/2", INF, "baryon", "u u d", &[]),
    p!(&["neutron", "n", "n0"], "n", &["antineutron"], "n̄", 939.565_420_52, "", "0", "1/2", 878.4, "baryon", "u d d", &[("p e⁻ ν̄ₑ", "100%")]),
    p!(&["lambda", "lambda baryon", "lambda0", "lambda hyperon"], "Λ⁰", &["antilambda"], "Λ̄⁰", 1115.683, "", "0", "1/2", 2.617e-10, "baryon", "u d s", &[("p π⁻", "64.1%"), ("n π⁰", "35.9%")]),
    p!(&["sigma+", "sigma plus", "sigma+ baryon"], "Σ⁺", &[], "Σ̄⁻", 1189.37, "", "+1", "1/2", 8.018e-11, "baryon", "u u s", &[("p π⁰", "51.57%"), ("n π⁺", "48.31%")]),
    p!(&["sigma0", "sigma zero", "sigma0 baryon"], "Σ⁰", &[], "Σ̄⁰", 1192.642, "", "0", "1/2", 7.4e-20, "baryon", "u d s", &[("Λ γ", "100%")]),
    p!(&["sigma-", "sigma minus", "sigma- baryon"], "Σ⁻", &[], "Σ̄⁺", 1197.449, "", "-1", "1/2", 1.479e-10, "baryon", "d d s", &[("n π⁻", "99.848%")]),
    p!(&["xi0", "xi zero", "cascade0"], "Ξ⁰", &[], "Ξ̄⁰", 1314.86, "", "0", "1/2", 2.90e-10, "baryon", "u s s", &[("Λ π⁰", "99.524%")]),
    p!(&["xi-", "xi minus", "cascade-"], "Ξ⁻", &[], "Ξ̄⁺", 1321.71, "", "-1", "1/2", 1.639e-10, "baryon", "d s s", &[("Λ π⁻", "99.887%")]),
    p!(&["omega-", "omega minus", "omega baryon"], "Ω⁻", &[], "Ω̄⁺", 1672.45, "", "-1", "3/2", 8.21e-11, "baryon", "s s s", &[("Λ K⁻", "67.8%"), ("Ξ⁰ π⁻", "23.6%"), ("Ξ⁻ π⁰", "8.6%")]),
    p!(&["delta++", "delta plus plus", "delta(1232)++"], "Δ⁺⁺", &[], "Δ̄⁻⁻", 1232.0, "", "+2", "3/2", 5.6e-24, "baryon", "u u u", &[("p π⁺", "≈100%")]),
    p!(&["lambda_c+", "lambda c", "charmed lambda"], "Λc⁺", &[], "Λ̄c⁻", 2286.46, "", "+1", "1/2", 2.026e-13, "baryon", "u d c", &[]),
    // mesons
    p!(&["pi+", "pion", "positive pion", "pi plus", "charged pion"], "π⁺", &["pi-", "negative pion", "pi minus"], "π⁻", 139.570_39, "", "+1", "0", 2.6033e-8, "meson", "u d̄", &[("μ⁺ ν_μ", "99.98770%"), ("e⁺ νₑ", "0.01230%")]),
    p!(&["pi0", "neutral pion", "pi zero"], "π⁰", &[], "", 134.9768, "", "0", "0", 8.43e-17, "meson", "(u ū − d d̄)/√2", &[("γ γ", "98.823%"), ("e⁺ e⁻ γ", "1.174%")]),
    p!(&["k+", "kaon", "positive kaon", "k plus", "charged kaon"], "K⁺", &["k-", "negative kaon", "k minus"], "K⁻", 493.677, "", "+1", "0", 1.2380e-8, "meson", "u s̄", &[("μ⁺ ν_μ", "63.56%"), ("π⁺ π⁰", "20.67%"), ("π⁺ π⁺ π⁻", "5.583%"), ("π⁰ e⁺ νₑ", "5.07%"), ("π⁰ μ⁺ ν_μ", "3.352%"), ("π⁺ π⁰ π⁰", "1.760%")]),
    p!(&["k0", "neutral kaon", "k zero"], "K⁰", &["anti-k0"], "K̄⁰", 497.611, "", "0", "0", NAN, "meson", "d s̄", &[]),
    p!(&["k-short", "k_s", "ks", "k short"], "K⁰_S", &[], "", 497.611, "", "0", "0", 8.954e-11, "meson", "(d s̄ − s d̄)/√2", &[("π⁺ π⁻", "69.20%"), ("π⁰ π⁰", "30.69%")]),
    p!(&["k-long", "k_l", "kl", "k long"], "K⁰_L", &[], "", 497.611, "", "0", "0", 5.116e-8, "meson", "(d s̄ + s d̄)/√2", &[("π± e∓ ν", "40.55%"), ("π± μ∓ ν", "27.04%"), ("3π⁰", "19.52%"), ("π⁺ π⁻ π⁰", "12.54%")]),
    p!(&["eta", "eta meson"], "η", &[], "", 547.862, "", "0", "0", 5.0e-19, "meson", "(u ū + d d̄ − 2 s s̄)/√6", &[("γ γ", "39.36%"), ("3π⁰", "32.57%"), ("π⁺ π⁻ π⁰", "22.92%")]),
    p!(&["rho", "rho meson", "rho0", "rho(770)"], "ρ⁰", &[], "", 775.26, "", "0", "1", 4.5e-24, "meson", "(u ū − d d̄)/√2", &[("π⁺ π⁻", "≈100%")]),
    p!(&["omega meson", "omega(782)"], "ω", &[], "", 782.66, "", "0", "1", 7.75e-23, "meson", "(u ū + d d̄)/√2", &[("π⁺ π⁻ π⁰", "89.2%"), ("π⁰ γ", "8.35%")]),
    p!(&["phi meson", "phi(1020)"], "φ", &[], "", 1019.461, "", "0", "1", 1.55e-22, "meson", "s s̄", &[("K⁺ K⁻", "49.1%"), ("K⁰_L K⁰_S", "33.9%")]),
    p!(&["j/psi", "jpsi", "j/psi meson", "psi(1s)"], "J/ψ", &[], "", 3096.900, "", "0", "1", 7.1e-21, "meson", "c c̄", &[("hadrons", "87.7%"), ("e⁺ e⁻", "5.971%"), ("μ⁺ μ⁻", "5.961%")]),
    p!(&["upsilon", "upsilon(1s)", "upsilon meson"], "Υ(1S)", &[], "", 9460.40, "", "0", "1", 1.22e-20, "meson", "b b̄", &[("τ⁺ τ⁻", "2.60%"), ("μ⁺ μ⁻", "2.48%"), ("e⁺ e⁻", "2.38%")]),
    p!(&["d0", "d0 meson", "d zero"], "D⁰", &["anti-d0"], "D̄⁰", 1864.84, "", "0", "0", 4.103e-13, "meson", "c ū", &[("K⁻ π⁺", "3.947%"), ("K⁻ π⁺ π⁰", "14.4%")]),
    p!(&["d+", "d+ meson", "d plus"], "D⁺", &["d-", "d- meson"], "D⁻", 1869.66, "", "+1", "0", 1.033e-12, "meson", "c d̄", &[("K⁻ π⁺ π⁺", "9.38%")]),
    p!(&["d*0", "d*(2007)0", "d star 0"], "D*⁰", &[], "D̄*⁰", 2006.85, "", "0", "1", NAN, "meson", "c ū", &[("D⁰ π⁰", "64.7%"), ("D⁰ γ", "35.3%")]),
    p!(&["d*+", "d*(2010)+", "d star plus"], "D*⁺", &["d*-"], "D*⁻", 2010.26, "", "+1", "1", 7.9e-21, "meson", "c d̄", &[("D⁰ π⁺", "67.7%"), ("D⁺ π⁰", "30.7%")]),
    p!(&["d_s+", "ds+", "ds meson", "d sub s"], "D_s⁺", &["d_s-"], "D_s⁻", 1968.35, "", "+1", "0", 5.01e-13, "meson", "c s̄", &[]),
    p!(&["b+", "b+ meson", "b plus"], "B⁺", &["b-", "b- meson"], "B⁻", 5279.41, "", "+1", "0", 1.638e-12, "meson", "u b̄", &[]),
    p!(&["b0", "b0 meson", "b zero"], "B⁰", &["anti-b0"], "B̄⁰", 5279.72, "", "0", "0", 1.517e-12, "meson", "d b̄", &[]),
    p!(&["b_s0", "bs", "bs meson", "b sub s"], "B⁰_s", &[], "B̄⁰_s", 5366.93, "", "0", "0", 1.520e-12, "meson", "s b̄", &[]),
    // nuclei
    p!(&["deuteron", "deuterium nucleus"], "d", &["antideuteron"], "d̄", 1875.612_942_57, "", "+1", "1", INF, "nucleus", "", &[]),
    p!(&["triton", "tritium nucleus"], "t", &[], "", 2808.921_132_98, "", "+1", "1/2", 5.6e8, "nucleus", "", &[("³He e⁻ ν̄ₑ", "100%")]),
    p!(&["helion", "helium-3 nucleus"], "h", &[], "", 2808.391_607_43, "", "+2", "1/2", INF, "nucleus", "", &[]),
    p!(&["alpha particle", "alpha", "helium nucleus", "helium-4 nucleus"], "α", &[], "", 3727.379_409_7, "", "+2", "0", INF, "nucleus", "", &[]),
];

/// Bare names too ambiguous to answer on their own.
const AMBIGUOUS: &[&str] = &["tau", "eta", "rho", "lambda", "alpha", "mu", "p", "n", "upsilon", "e-", "b+", "b-", "b0", "d+", "d-", "d0", "k0", "w+", "w-", "z0", "kl", "ks", "bs", "pion", "kaon"];

fn norm(s: &str) -> String {
    let mut s = s.to_lowercase();
    for (a, b) in [("⁺", "+"), ("⁻", "-"), ("⁰", "0"), ("π", "pi"), ("μ", "mu"), ("τ", "tau"), ("λ", "lambda"), ("σ", "sigma"), ("ξ", "xi"), ("ω", "omega"), ("δ", "delta"), ("η", "eta"), ("ρ", "rho"), ("ψ", "psi"), ("υ", "upsilon"), ("ν", "nu"), ("γ", "gamma"), ("φ", "phi"), ("^", ""), ("−", "-")] {
        s = s.replace(a, b);
    }
    let s = re!(r"\s+([+-]|0\b)").replace_all(&s, "$1");
    let s = s.trim().trim_start_matches("the ").trim_start_matches("an ").trim_start_matches("a ").trim();
    let s = s.strip_suffix(" particle").filter(|r| !r.ends_with("alpha") && !r.ends_with(" w") && !r.ends_with(" z") && *r != "w" && *r != "z" && *r != "higgs").unwrap_or(s);
    s.trim().to_string()
}

/// A particle by name: (particle, is the antiparticle).
pub fn find_particle(name: &str) -> Option<(&'static Particle, bool)> {
    let n = norm(name);
    let look = |n: &str| -> Option<(&'static Particle, bool)> {
        for p in PARTICLES {
            if p.names.contains(&n) {
                return Some((p, false));
            }
            if p.anti_names.contains(&n) {
                return Some((p, true));
            }
        }
        None
    };
    if let Some(r) = look(&n) {
        return Some(r);
    }
    // plurals, "anti-X", "X antiquark", "antiX"
    if let Some(r) = n.strip_suffix('s').and_then(|s| look(s)).filter(|(p, _)| !p.names.contains(&"b") && n.len() > 4) {
        return Some(r);
    }
    if let Some(base) = n.strip_suffix(" antiquark") {
        return look(&format!("{base} quark")).map(|(p, a)| (p, !a));
    }
    for pre in ["anti-", "anti ", "anti"] {
        if let Some(base) = n.strip_prefix(pre) {
            if let Some((p, a)) = look(base) {
                if p.anti_sym.is_empty() {
                    return Some((p, a));
                }
                return Some((p, !a));
            }
        }
    }
    None
}

fn flip_charge(q: &str) -> String {
    if let Some(r) = q.strip_prefix('+') {
        format!("-{r}")
    } else if let Some(r) = q.strip_prefix('-') {
        format!("+{r}")
    } else {
        q.to_string()
    }
}

fn conj_content(c: &str) -> String {
    if c.contains('(') {
        return c.to_string();
    }
    c.split_whitespace()
        .map(|t| if let Some(b) = t.strip_suffix('\u{0304}') { b.to_string() } else { format!("{t}\u{0304}") })
        .collect::<Vec<_>>()
        .join(" ")
}

fn pname(p: &Particle, anti: bool) -> String {
    if anti {
        if let Some(n) = p.anti_names.first() {
            return n.to_string();
        }
        if p.anti_sym.is_empty() {
            return p.names[0].to_string();
        }
        if p.class == "quark" {
            return p.names[0].replace(" quark", " antiquark");
        }
        return format!("anti-{}", p.names[0]);
    }
    p.names[0].to_string()
}

fn psym(p: &Particle, anti: bool) -> &'static str {
    if anti && !p.anti_sym.is_empty() {
        p.anti_sym
    } else {
        p.sym
    }
}

fn charge(p: &Particle, anti: bool) -> String {
    let q = if anti { flip_charge(p.charge) } else { p.charge.to_string() };
    format!("{} e", q.replace('-', "−"))
}

fn mass_text(p: &Particle) -> String {
    if p.mass.is_nan() {
        return p.mass_note.to_string();
    }
    if p.mass == 0.0 {
        return if p.mass_note.is_empty() { "0 (massless)".into() } else { format!("0 ({})", p.mass_note) };
    }
    if p.mass >= 10_000.0 {
        format!("{} GeV/c²", fmt_float(p.mass / 1000.0, 7))
    } else {
        format!("{} MeV/c²", fmt_float(p.mass, 12))
    }
}

fn life_text(t: f64) -> String {
    if t.is_infinite() {
        "stable".into()
    } else if t.is_nan() {
        "—".into()
    } else if t >= 60.0 {
        format!("{} s ({} min)", fmt_float(t, 5), fmt_float(t / 60.0, 4))
    } else {
        format!("{} s", fmt_float(t, 5))
    }
}

fn decays(p: &Particle, anti: bool) -> Vec<Vec<Content>> {
    p.decays
        .iter()
        .map(|(m, f)| {
            let m = if anti { conj_decay(m) } else { m.to_string() };
            vec![text(m), text(f.to_string())]
        })
        .collect()
}

/// Charge-conjugate a decay mode for display.
fn conj_decay(m: &str) -> String {
    m.chars()
        .map(|c| match c {
            '⁺' => '⁻',
            '⁻' => '⁺',
            _ => c,
        })
        .collect()
}

fn particle_answer(p: &'static Particle, anti: bool) -> Answer {
    let name = pname(p, anti);
    let mut rows = vec![
        vec![text("symbol"), text(psym(p, anti).to_string())],
        vec![text("class"), text(p.class.to_string())],
        vec![text("mass"), text(mass_text(p))],
    ];
    if p.mass > 0.0 {
        rows.push(vec![text("mass in kg"), text(format!("{} kg", fmt_float(p.mass * MEV_KG, 6)))]);
    }
    rows.push(vec![text("charge"), text(charge(p, anti))]);
    rows.push(vec![text("spin"), text(p.spin.to_string())]);
    rows.push(vec![text("mean lifetime"), text(life_text(p.life))]);
    if !p.content.is_empty() {
        rows.push(vec![text("quark content"), text(if anti { conj_content(p.content) } else { p.content.to_string() })]);
    }
    let anti_of = if p.anti_sym.is_empty() { "itself".to_string() } else { format!("{} ({})", pname(p, !anti), psym(p, !anti)) };
    rows.push(vec![text("antiparticle"), text(anti_of)]);
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{name} (particle)"))), primary(pod("Properties", "Basic properties", table(None, rows)))];
    if !p.decays.is_empty() {
        pods.push(pod("Decays", "Main decay modes", table(Some(vec!["mode".into(), "fraction".into()]), decays(p, anti))));
    }
    Answer::new(name, pods)
}

const CLASSES: &[(&str, &[&str])] = &[
    ("quark", &["quark"]),
    ("lepton", &["lepton"]),
    ("neutrino", &["lepton"]),
    ("boson", &["gauge boson", "scalar boson"]),
    ("gauge boson", &["gauge boson"]),
    ("meson", &["meson"]),
    ("baryon", &["baryon"]),
    ("hadron", &["meson", "baryon"]),
    ("fermion", &["quark", "lepton"]),
    ("elementary particle", &["quark", "lepton", "gauge boson", "scalar boson"]),
];

fn class_members(word: &str) -> Option<(&'static str, Vec<&'static Particle>)> {
    let w = word.trim().trim_end_matches('s');
    let w = w.strip_suffix(" mass").unwrap_or(w);
    let (name, classes) = CLASSES.iter().find(|(n, _)| *n == w)?;
    let mut v: Vec<_> = PARTICLES.iter().filter(|p| classes.contains(&p.class)).collect();
    if *name == "neutrino" {
        v.retain(|p| p.names[0].contains("neutrino"));
    }
    if *name == "boson" || *name == "gauge boson" {
        v.retain(|p| p.names[0] != "graviton");
    }
    if matches!(*name, "meson" | "baryon" | "hadron") {
        v.truncate(12);
    }
    Some((name, v))
}

#[derive(Clone, Copy, PartialEq)]
enum Prop {
    Mass,
    Charge,
    Spin,
    Life,
    Content,
    Decays,
    Anti,
}

fn prop_word(w: &str) -> Option<Prop> {
    Some(match w {
        "mass" | "rest mass" | "masses" | "weight" => Prop::Mass,
        "charge" | "electric charge" | "charges" => Prop::Charge,
        "spin" | "spins" => Prop::Spin,
        "lifetime" | "mean lifetime" | "mean life" | "half-life" | "half life" | "lifetimes" => Prop::Life,
        "quark content" | "composition" | "quark composition" | "quarks" => Prop::Content,
        "decay modes" | "decays" | "decay channels" | "decay products" | "decay" | "branching ratios" => Prop::Decays,
        "antiparticle" => Prop::Anti,
        _ => return None,
    })
}

fn prop_value(p: &Particle, anti: bool, prop: Prop) -> String {
    match prop {
        Prop::Mass => mass_text(p),
        Prop::Charge => charge(p, anti),
        Prop::Spin => p.spin.to_string(),
        Prop::Life => life_text(p.life),
        Prop::Content => {
            if p.content.is_empty() {
                format!("{} (not made of quarks)", p.class)
            } else if anti {
                conj_content(p.content)
            } else {
                p.content.to_string()
            }
        }
        Prop::Anti => {
            if p.anti_sym.is_empty() {
                "itself".into()
            } else {
                format!("{} ({})", pname(p, !anti), psym(p, !anti))
            }
        }
        Prop::Decays => String::new(),
    }
}

fn prop_name(prop: Prop) -> &'static str {
    match prop {
        Prop::Mass => "mass",
        Prop::Charge => "electric charge",
        Prop::Spin => "spin",
        Prop::Life => "mean lifetime",
        Prop::Content => "quark content",
        Prop::Decays => "decay modes",
        Prop::Anti => "antiparticle",
    }
}

fn property_answer(p: &'static Particle, anti: bool, prop: Prop) -> Answer {
    let name = pname(p, anti);
    let input = pod("Input", "Input interpretation", text(format!("{name} | {}", prop_name(prop))));
    if prop == Prop::Decays {
        if p.decays.is_empty() {
            let r = if p.life.is_infinite() { "stable: no decays".to_string() } else { "no tabulated decay modes".to_string() };
            return Answer::new(format!("{name} decay modes"), vec![input, primary(pod("Result", "Result", text(r)))]);
        }
        return Answer::new(format!("{name} decay modes"), vec![input, primary(pod("Decays", "Main decay modes", table(Some(vec!["mode".into(), "fraction".into()]), decays(p, anti))))]);
    }
    let mut pods = vec![input, primary(pod("Result", "Result", text(prop_value(p, anti, prop))))];
    if prop == Prop::Mass && p.mass > 0.0 {
        let rows = vec![
            vec![text(format!("{} kg", fmt_float(p.mass * MEV_KG, 8)))],
            vec![text(format!("{} u", fmt_float(p.mass / 931.494_102_42, 9)))],
            vec![text(format!("{} electron masses", fmt_float(p.mass / 0.510_998_95, 8)))],
        ];
        pods.push(pod("Conversions", "Unit conversions", table(None, rows)));
    }
    Answer::new(format!("{name} {}", prop_name(prop)), pods)
}

fn class_answer(name: &str, members: &[&'static Particle], prop: Option<Prop>) -> Answer {
    let prop = prop.unwrap_or(Prop::Mass);
    let rows = members.iter().map(|p| vec![text(p.names[0].to_string()), text(p.sym.to_string()), text(prop_value(p, false, prop))]).collect();
    Answer::new(
        format!("{name}s"),
        vec![
            pod("Input", "Input interpretation", text(format!("{name}s | {}", prop_name(prop)))),
            primary(pod("Result", "Result", table(Some(vec!["particle".into(), "symbol".into(), prop_name(prop).into()]), rows))),
        ],
    )
}

fn compare_answer(ps: &[(&'static Particle, bool)]) -> Answer {
    let names: Vec<String> = ps.iter().map(|(p, a)| pname(p, *a)).collect();
    let header = std::iter::once("").chain(names.iter().map(String::as_str)).collect();
    let mut rows = Vec::new();
    for prop in [Prop::Mass, Prop::Charge, Prop::Spin, Prop::Life, Prop::Content] {
        let mut r = vec![text(prop_name(prop))];
        r.extend(ps.iter().map(|(p, a)| text(prop_value(p, *a, prop))));
        rows.push(r);
    }
    Answer::new(names.join(" | "), vec![pod("Input", "Input interpretation", text(names.join("  |  "))), primary(pod("Comparison", "Basic properties", table(Some(header), rows)))])
}

/// Split "electron D*0 neutron" or "electron, proton" into particles.
fn particle_list(l: &str) -> Option<Vec<(&'static Particle, bool)>> {
    let parts: Vec<String> = if l.contains(',') || l.contains(" and ") || l.contains(" vs ") {
        l.split([',']).flat_map(|s| s.split(" and ")).flat_map(|s| s.split(" vs ")).map(|s| s.trim().trim_end_matches('.').to_string()).filter(|s| !s.is_empty()).collect()
    } else {
        let w: Vec<&str> = l.split_whitespace().collect();
        let mut out = Vec::new();
        let mut i = 0;
        'outer: while i < w.len() {
            for k in (1..=3.min(w.len() - i)).rev() {
                let cand = w[i..i + k].join(" ");
                if find_particle(&cand).is_some() {
                    out.push(cand);
                    i += k;
                    continue 'outer;
                }
            }
            return None;
        }
        out
    };
    if parts.len() < 2 {
        return None;
    }
    parts.iter().map(|s| find_particle(s)).collect()
}

fn particles(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?').trim();
    let l = l.trim_start_matches("what is ").trim_start_matches("what's ");
    // "mass of the proton", "proton mass", "spin K+", "tau mass"
    if let Some(m) = re!(r"^(rest mass|mass|masses|electric charge|charges?|spins?|mean lifetime|mean life|lifetimes?|half-life|half life|quark content|quark composition|composition|quarks|decay modes|decay channels|decay products|decays|branching ratios|antiparticle)\s+(?:of\s+)?(?:an?\s+|the\s+)?(.+)$").captures(l).or_else(|| re!(r"^(?:an?\s+|the\s+)?(.+?)(?:'s)?\s+(rest mass|mass|masses|electric charge|charges?|spins?|mean lifetime|mean life|lifetimes?|half-life|half life|quark content|quark composition|composition|decay modes|decay channels|decay products|decays|branching ratios|antiparticle)$").captures(l)) {
        let (pw, who) = if prop_word(&m[1]).is_some() { (&m[1], &m[2]) } else { (&m[2], &m[1]) };
        let prop = prop_word(pw)?;
        if let Some((p, a)) = find_particle(who) {
            if prop == Prop::Mass && p.names[0] == "electron" && !a {
                return None; // the constants table answers with CODATA's kg value
            }
            return Some(property_answer(p, a, prop));
        }
        if let Some((name, members)) = class_members(who) {
            return Some(class_answer(name, &members, Some(prop)));
        }
        return None;
    }
    // bare particle
    if let Some((p, a)) = find_particle(l) {
        if AMBIGUOUS.contains(&norm(l).as_str()) {
            return None;
        }
        return Some(particle_answer(p, a));
    }
    if let Some((name, members)) = class_members(l) {
        if l.ends_with('s') || l.contains(" mass") {
            return Some(class_answer(name, &members, None));
        }
    }
    let list = particle_list(l)?;
    Some(compare_answer(&list))
}

// ---------------------------------------------------------------------------
// ratios: "mass proton / electron", "Planck mass / mass of electron"
// ---------------------------------------------------------------------------

/// A named quantity as (SI value, unit text, display name).
fn named_quantity(s: &str, mass_hint: bool) -> Option<(f64, String, String)> {
    let s = s.trim().trim_start_matches("the ");
    let s = re!(r"\s+").replace_all(s, " ");
    let s = s.as_ref();
    let bare = s.strip_prefix("mass of ").or_else(|| s.strip_prefix("mass ")).or_else(|| s.strip_suffix(" mass"));
    if let Some(c) = units::find_constant(s) {
        return Some((c.value, c.unit.to_string(), c.names[0].to_string()));
    }
    if let Some(b) = bare {
        if let Some(c) = units::find_constant(&format!("{b} mass")) {
            return Some((c.value, c.unit.to_string(), c.names[0].to_string()));
        }
    }
    if mass_hint || bare.is_some() {
        if let Some((p, a)) = find_particle(bare.unwrap_or(s)) {
            if p.mass > 0.0 {
                return Some((p.mass * MEV_KG, "kg".into(), format!("{} mass", pname(p, a))));
            }
        }
    }
    None
}

fn ratio(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    let (a, b) = l.split_once(" / ").or_else(|| l.split_once('/')).or_else(|| l.split_once(" divided by "))?;
    let (a, b) = (a.trim(), b.trim());
    if a.is_empty() || b.is_empty() || a.chars().any(|c| c.is_ascii_digit()) && b.chars().any(|c| c.is_ascii_digit()) && !a.contains(' ') {
        return None;
    }
    let mass_hint = a.starts_with("mass") || a.ends_with(" mass") || b.ends_with(" mass");
    let (av, au, an) = named_quantity(a, mass_hint)?;
    let (bv, bu, bn) = named_quantity(b, mass_hint || au == "kg")?;
    if au.replace(' ', "") != bu.replace(' ', "") {
        return None;
    }
    let r = av / bv;
    Some(Answer::new(
        format!("{an} / {bn}"),
        vec![pod("Input", "Input interpretation", text(format!("{an} / {bn}"))), primary(pod("Result", "Result", text(fmt_float(r, 10))))],
    ))
}

// ---------------------------------------------------------------------------
// resistor colour codes
// ---------------------------------------------------------------------------

const COLOURS: &[(&str, i32)] = &[("black", 0), ("brown", 1), ("red", 2), ("orange", 3), ("yellow", 4), ("green", 5), ("blue", 6), ("violet", 7), ("purple", 7), ("grey", 8), ("gray", 8), ("white", 9), ("gold", -1), ("silver", -2), ("pink", -3)];

fn tolerance(c: &str) -> Option<&'static str> {
    Some(match c {
        "brown" => "±1%",
        "red" => "±2%",
        "green" => "±0.5%",
        "blue" => "±0.25%",
        "violet" | "purple" => "±0.1%",
        "grey" | "gray" => "±0.05%",
        "gold" => "±5%",
        "silver" => "±10%",
        "none" => "±20%",
        _ => return None,
    })
}

fn tempco(c: &str) -> Option<&'static str> {
    Some(match c {
        "black" => "250 ppm/K",
        "brown" => "100 ppm/K",
        "red" => "50 ppm/K",
        "orange" => "15 ppm/K",
        "yellow" => "25 ppm/K",
        "green" => "20 ppm/K",
        "blue" => "10 ppm/K",
        "violet" | "purple" => "5 ppm/K",
        "grey" | "gray" => "1 ppm/K",
        _ => return None,
    })
}

fn ohms(v: f64) -> String {
    let (d, u) = if v >= 1e9 {
        (v / 1e9, "GΩ")
    } else if v >= 1e6 {
        (v / 1e6, "MΩ")
    } else if v >= 1e3 {
        (v / 1e3, "kΩ")
    } else {
        (v, "Ω")
    };
    format!("{} {u}", fmt_float(d, 6))
}

fn resistor(q: &Query) -> Option<Answer> {
    let l = q.lower.as_str();
    if !re!(r"\b(resistors?|colou?r code|bands?)\b").is_match(l) {
        return None;
    }
    let words: Vec<&str> = re!(r"[a-z]+").find_iter(l).map(|m| m.as_str()).filter(|w| COLOURS.iter().any(|(c, _)| c == w)).collect();
    if words.len() >= 3 {
        let n = words.len().min(6);
        let words = &words[..n];
        let digits_n = if n >= 5 { 3 } else { 2 };
        let digit = |w: &str| COLOURS.iter().find(|(c, _)| *c == w).map(|c| c.1).filter(|d| *d >= 0);
        let mut v = 0.0;
        for w in &words[..digits_n] {
            v = v * 10.0 + digit(w)? as f64;
        }
        let mult = COLOURS.iter().find(|(c, _)| *c == words[digits_n])?.1;
        if mult == -3 {
            return None;
        }
        let r = v * 10f64.powi(mult);
        let tol = words.get(digits_n + 1).and_then(|w| tolerance(w)).unwrap_or("±20%");
        let mut rows = vec![vec![text("resistance"), text(ohms(r))], vec![text("tolerance"), text(tol.to_string())]];
        let pct: f64 = tol.trim_start_matches('±').trim_end_matches('%').parse().unwrap_or(20.0) / 100.0;
        rows.push(vec![text("range"), text(format!("{} to {}", ohms(r * (1.0 - pct)), ohms(r * (1.0 + pct))))]);
        if n == 6 {
            if let Some(t) = tempco(words[5]) {
                rows.push(vec![text("temperature coefficient"), text(t.to_string())]);
            }
        }
        return Some(Answer::new(
            format!("resistor {}", words.join(" ")),
            vec![pod("Input", "Input interpretation", text(format!("{}-band resistor | {}", n, words.join(", ")))), primary(pod("Result", "Result", text(format!("{} {tol}", ohms(r))))), pod("Details", "Details", table(None, rows))],
        ));
    }
    // value → bands: "4.7k ohm resistor color code"
    let m = re!(r"(\d+(?:\.\d+)?)\s*(k|m|g|meg)?\s*(?:ohms?|ω)?(?:\s|$)").captures(l)?;
    let mut v: f64 = m[1].parse().ok()?;
    v *= match m.get(2).map(|x| x.as_str()) {
        Some("k") => 1e3,
        Some("m") | Some("meg") => 1e6,
        Some("g") => 1e9,
        _ => 1.0,
    };
    if v <= 0.0 || v >= 1e11 {
        return None;
    }
    let names = ["black", "brown", "red", "orange", "yellow", "green", "blue", "violet", "grey", "white"];
    let bands = |digits: i32| -> Option<Vec<String>> {
        let e = v.log10().floor() as i32 - (digits - 1);
        let sig = (v / 10f64.powi(e)).round() as i64;
        if (sig as f64 * 10f64.powi(e) - v).abs() > v * 1e-9 {
            return None;
        }
        let mut out: Vec<String> = format!("{sig:0width$}", width = digits as usize).chars().map(|c| names[c.to_digit(10).unwrap() as usize].to_string()).collect();
        out.push(match e {
            -1 => "gold".into(),
            -2 => "silver".into(),
            0..=9 => names[e as usize].into(),
            _ => return None,
        });
        Some(out)
    };
    let four = bands(2);
    let five = bands(3);
    if four.is_none() && five.is_none() {
        return None;
    }
    let mut rows = Vec::new();
    if let Some(b) = &four {
        rows.push(vec![text("4-band (±5%)"), text(format!("{}, gold", b.join(", ")))]);
    }
    if let Some(b) = &five {
        rows.push(vec![text("5-band (±1%)"), text(format!("{}, brown", b.join(", ")))]);
    }
    Some(Answer::new(format!("{} resistor colour code", ohms(v)), vec![pod("Input", "Input interpretation", text(format!("{} resistor | colour code", ohms(v)))), primary(pod("Result", "Colour bands", table(None, rows)))]))
}

// ---------------------------------------------------------------------------
// lasers
// ---------------------------------------------------------------------------

/// (names, medium type, wavelengths in nm, note)
const LASERS: &[(&[&str], &str, &[f64], &str)] = &[
    (&["hene", "he-ne", "helium-neon", "helium neon"], "gas", &[632.8, 543.5, 594.1, 611.8, 1152.3, 3391.3], "632.8 nm red line is the usual one"),
    (&["argon ion", "argon", "ar+", "ar ion"], "ion gas", &[488.0, 514.5, 457.9, 476.5, 496.5, 501.7], ""),
    (&["krypton ion", "krypton", "kr+"], "ion gas", &[647.1, 676.4, 530.9, 568.2], ""),
    (&["hecd", "he-cd", "helium-cadmium", "helium cadmium"], "metal vapour", &[441.6, 325.0], ""),
    (&["co2", "carbon dioxide"], "molecular gas", &[10_600.0, 9_400.0], "mid-infrared"),
    (&["co", "carbon monoxide"], "molecular gas", &[5_300.0], "5–6 μm band"),
    (&["nitrogen", "n2"], "molecular gas", &[337.1], "pulsed ultraviolet"),
    (&["arf", "argon fluoride", "arf excimer"], "excimer", &[193.0], "deep-ultraviolet photolithography and eye surgery"),
    (&["krf", "krypton fluoride", "krf excimer"], "excimer", &[248.0], ""),
    (&["xecl", "xenon chloride", "xecl excimer"], "excimer", &[308.0], ""),
    (&["xef", "xenon fluoride", "xef excimer"], "excimer", &[351.0], ""),
    (&["krcl", "krypton chloride"], "excimer", &[222.0], ""),
    (&["f2", "fluorine"], "excimer", &[157.0], ""),
    (&["nd:yag", "nd yag", "ndyag", "neodymium yag", "yag"], "solid state", &[1064.0, 532.0, 355.0, 266.0], "harmonics 532, 355 and 266 nm"),
    (&["nd:ylf", "nd ylf"], "solid state", &[1047.0, 1053.0], ""),
    (&["nd:glass", "neodymium glass"], "solid state", &[1062.0], ""),
    (&["yb:yag", "ytterbium yag"], "solid state", &[1030.0], ""),
    (&["er:yag", "erbium yag"], "solid state", &[2940.0], ""),
    (&["ho:yag", "holmium yag"], "solid state", &[2100.0], ""),
    (&["tm:yag", "thulium yag"], "solid state", &[2010.0], ""),
    (&["er:glass", "erbium glass", "erbium fiber", "erbium-doped fiber"], "solid state", &[1540.0], ""),
    (&["ruby"], "solid state", &[694.3], "the first laser (Maiman, 1960)"),
    (&["ti:sapphire", "ti sapphire", "titanium sapphire", "ti:sa"], "solid state, tunable", &[800.0], "tunable 650–1100 nm"),
    (&["alexandrite"], "solid state, tunable", &[755.0], "tunable 700–820 nm"),
    (&["copper vapor", "copper vapour"], "metal vapour", &[510.6, 578.2], ""),
    (&["gold vapor", "gold vapour"], "metal vapour", &[627.8], ""),
    (&["gan", "blu-ray", "violet diode", "gan diode"], "semiconductor", &[405.0], ""),
    (&["gaas", "gaas diode"], "semiconductor", &[840.0], ""),
    (&["rhodamine 6g", "dye", "rhodamine"], "dye, tunable", &[590.0], "tunable 570–640 nm"),
];

fn colour_of(nm: f64) -> &'static str {
    match nm {
        x if x < 10.0 => "X-ray",
        x if x < 380.0 => "ultraviolet",
        x if x < 450.0 => "violet",
        x if x < 485.0 => "blue",
        x if x < 500.0 => "cyan",
        x if x < 565.0 => "green",
        x if x < 590.0 => "yellow",
        x if x < 625.0 => "orange",
        x if x < 750.0 => "red",
        x if x < 1_000_000.0 => "infrared",
        _ => "microwave",
    }
}

fn find_laser(s: &str) -> Option<&'static (&'static [&'static str], &'static str, &'static [f64], &'static str)> {
    let s = s.trim().trim_end_matches(" lasers").trim_end_matches(" laser").trim_end_matches(" wavelength").trim_end_matches(" laser").trim();
    let s = s.trim_start_matches("wavelength of ").trim_start_matches("the ").trim_start_matches("a ").trim_start_matches("an ");
    LASERS.iter().find(|(n, ..)| n.contains(&s))
}

fn lasers(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    if !l.contains("laser") {
        return None;
    }
    let parts: Vec<&str> = l.split([',', ';']).flat_map(|s| s.split(" and ")).flat_map(|s| s.split(" vs ")).map(str::trim).filter(|s| !s.is_empty()).collect();
    let found: Vec<_> = parts.iter().map(|p| find_laser(p)).collect::<Option<Vec<_>>>()?;
    let mut rows = Vec::new();
    for (names, kind, wl, note) in &found {
        let main = wl[0];
        let ev = 1_239.841_98 / main;
        let freq = 299_792_458.0 / (main * 1e-9);
        let lname = laser_name(names[0]);
        rows.push(vec![
            text(lname),
            text(kind.to_string()),
            text(format!("{} nm ({})", fmt_float(main, 6), colour_of(main))),
            text(format!("{} eV", fmt_float(ev, 4))),
            text(format!("{} THz", fmt_float(freq / 1e12, 4))),
            text(if wl.len() > 1 { wl[1..].iter().map(|w| format!("{} nm", fmt_float(*w, 6))).collect::<Vec<_>>().join(", ") } else { "—".into() }),
            text(if note.is_empty() { "—".to_string() } else { note.to_string() }),
        ]);
    }
    let names: Vec<String> = found.iter().map(|(n, ..)| laser_name(n[0])).collect();
    Some(Answer::new(
        names.join(" | "),
        vec![
            pod("Input", "Input interpretation", text(names.join("  |  "))),
            primary(pod("Result", "Laser properties", table(Some(vec!["laser".into(), "type".into(), "main wavelength".into(), "photon energy".into(), "frequency".into(), "other lines".into(), "notes".into()]), rows))),
        ],
    ))
}

fn laser_name(n: &str) -> String {
    let fixed = match n {
        "hene" => "helium-neon (HeNe)",
        "hecd" => "helium-cadmium (HeCd)",
        "co2" => "carbon dioxide (CO₂)",
        "co" => "carbon monoxide (CO)",
        "arf" => "argon fluoride (ArF) excimer",
        "krf" => "krypton fluoride (KrF) excimer",
        "xecl" => "xenon chloride (XeCl) excimer",
        "xef" => "xenon fluoride (XeF) excimer",
        "krcl" => "krypton chloride (KrCl) excimer",
        "f2" => "fluorine (F₂) excimer",
        "nd:yag" => "Nd:YAG",
        "nd:ylf" => "Nd:YLF",
        "nd:glass" => "Nd:glass",
        "yb:yag" => "Yb:YAG",
        "er:yag" => "Er:YAG",
        "ho:yag" => "Ho:YAG",
        "tm:yag" => "Tm:YAG",
        "er:glass" => "Er:glass",
        "ti:sapphire" => "Ti:sapphire",
        "gan" => "GaN diode",
        "gaas" => "GaAs diode",
        _ => "",
    };
    if fixed.is_empty() {
        format!("{n} laser")
    } else {
        format!("{fixed} laser")
    }
}

// ---------------------------------------------------------------------------
// work functions (CRC Handbook, polycrystalline, eV)
// ---------------------------------------------------------------------------

const WORK_FUNCTIONS: &[(&str, f64)] = &[
    ("Ag", 4.26), ("Al", 4.28), ("As", 3.75), ("Au", 5.1), ("B", 4.45), ("Ba", 2.7), ("Be", 4.98), ("Bi", 4.22), ("C", 5.0), ("Ca", 2.87),
    ("Cd", 4.22), ("Ce", 2.9), ("Co", 5.0), ("Cr", 4.5), ("Cs", 2.14), ("Cu", 4.65), ("Eu", 2.5), ("Fe", 4.5), ("Ga", 4.2), ("Gd", 3.1),
    ("Hf", 3.9), ("Hg", 4.49), ("In", 4.12), ("Ir", 5.27), ("K", 2.30), ("La", 3.5), ("Li", 2.9), ("Lu", 3.3), ("Mg", 3.66), ("Mn", 4.1),
    ("Mo", 4.6), ("Na", 2.75), ("Nb", 4.3), ("Nd", 3.2), ("Ni", 5.15), ("Os", 4.83), ("Pb", 4.25), ("Pd", 5.12), ("Pt", 5.65), ("Rb", 2.16),
    ("Re", 4.96), ("Rh", 4.98), ("Ru", 4.71), ("Sb", 4.55), ("Sc", 3.5), ("Se", 5.9), ("Si", 4.85), ("Sm", 2.7), ("Sn", 4.42), ("Sr", 2.59),
    ("Ta", 4.25), ("Tb", 3.0), ("Te", 4.95), ("Th", 3.4), ("Ti", 4.33), ("Tl", 3.84), ("U", 3.63), ("V", 4.3), ("W", 4.55), ("Y", 3.1),
    ("Zn", 4.33), ("Zr", 4.05), ("Ge", 5.0), ("Yb", 2.6), ("Pr", 2.96), ("Dy", 3.25), ("Ho", 3.22), ("Er", 3.25), ("Tm", 3.1),
];

fn work_function(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    let m = re!(r"^(?:photoelectric\s+)?work\s+functions?\s+(?:of\s+|for\s+)?(.+)$").captures(l).or_else(|| re!(r"^(.+?)(?:'s)?\s+(?:photoelectric\s+)?work\s+functions?$").captures(l))?;
    let who = m[1].trim();
    let raw = &q.clean[q.clean.len() - who.len().min(q.clean.len())..];
    let el = chem::find_element(who).or_else(|| chem::find_element(raw.trim())).or_else(|| chem::find_element(&who.split_whitespace().last().map(|w| {
        let mut c = w.chars();
        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
    })?))?;
    let &(_, w) = WORK_FUNCTIONS.iter().find(|(s, _)| *s == el.symbol)?;
    let nm = 1_239.841_98 / w;
    let rows = vec![
        vec![text("in joules"), text(format!("{} J", fmt_float(w * 1.602_176_634e-19, 5)))],
        vec![text("threshold wavelength"), text(format!("{} nm ({})", fmt_float(nm, 4), colour_of(nm)))],
        vec![text("threshold frequency"), text(format!("{} Hz", fmt_float(w * 1.602_176_634e-19 / 6.626_070_15e-34, 4)))],
    ];
    Some(Answer::new(
        format!("{} work function", el.name),
        vec![
            pod("Input", "Input interpretation", text(format!("{} | work function", el.name))),
            primary(pod("Result", "Result", text(format!("{w} eV")))),
            pod("Derived", "Photoelectric threshold", table(None, rows)),
            pod("Notes", "Notes", text("Polycrystalline value (CRC Handbook); single-crystal faces differ by a few tenths of an eV.")),
        ],
    ))
}

// ---------------------------------------------------------------------------
// light: "700 THz light", "500 nm photon", "2 eV photon"
// ---------------------------------------------------------------------------

fn light(q: &Query) -> Option<Answer> {
    let l = q.clean.trim();
    let m = re!(r"(?i)^(?:an?\s+)?(\d+(?:\.\d+)?(?:e-?\d+)?)\s*([a-zμ]+)\s+(light|photons?|radiation|electromagnetic waves?|em waves?|em radiation|laser light|wave)$").captures(l)?;
    let v: f64 = m[1].parse().ok()?;
    let unit = units::parse_unit(&m[2]).or_else(|| units::parse_unit(&m[2].to_lowercase()))?;
    let si = units::to_si(v, &unit);
    let (h, c, ev) = (6.626_070_15e-34, 299_792_458.0, 1.602_176_634e-19);
    let d = &unit.dim;
    let lambda = if d[0] == 1 && d[1..].iter().all(|x| *x == 0) {
        si
    } else if d[2] == -1 && d.iter().enumerate().all(|(i, x)| i == 2 || *x == 0) {
        c / si
    } else if d[0] == 2 && d[1] == 1 && d[2] == -2 && d[3..].iter().all(|x| *x == 0) {
        h * c / si
    } else {
        return None;
    };
    let f = c / lambda;
    let e = h * f;
    let nm = lambda * 1e9;
    let rows = vec![
        vec![text("wavelength"), text(if nm >= 1e6 { format!("{} mm", fmt_float(nm / 1e6, 6)) } else if nm >= 1e3 { format!("{} μm", fmt_float(nm / 1e3, 6)) } else { format!("{} nm", fmt_float(nm, 6)) })],
        vec![text("frequency"), text(format!("{} THz", fmt_float(f / 1e12, 6)))],
        vec![text("photon energy"), text(format!("{} eV", fmt_float(e / ev, 6)))],
        vec![text("photon energy (J)"), text(format!("{} J", fmt_float(e, 6)))],
        vec![text("wavenumber"), text(format!("{} cm⁻¹", fmt_float(0.01 / lambda, 6)))],
        vec![text("photon momentum"), text(format!("{} kg m/s", fmt_float(h / lambda, 6)))],
        vec![text("band"), text(colour_of(nm).to_string())],
    ];
    Some(Answer::new(
        format!("{} {} light", &m[1], &m[2]),
        vec![
            pod("Input", "Input interpretation", text(format!("electromagnetic radiation | {} {}", &m[1], unit.name))),
            primary(pod("Result", "Properties", table(None, rows))),
        ],
    ))
}

pub fn handle(q: &Query) -> Option<Answer> {
    light(q).or_else(|| resistor(q)).or_else(|| lasers(q)).or_else(|| work_function(q)).or_else(|| ratio(q)).or_else(|| particles(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(s: &str) -> String {
        crate::engine::query(s).short_answer().unwrap_or_default()
    }

    #[test]
    fn particle_lookups() {
        assert!(ask("tau mass").contains("1776.86"));
        assert!(ask("spin K+").contains('0'));
        assert!(ask("top antiquark").contains("−2/3"));
        assert!(ask("mass proton / electron").starts_with("1836.15"));
        assert!(ask("resistor black red green silver").contains("200 kΩ ±10%"));
        assert!(ask("HeNe laser").contains("632.8"));
        assert!(ask("work function Nb").contains("4.3 eV"));
    }
}
