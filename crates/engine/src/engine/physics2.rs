//! Physics by name and by formula: laws, principles and effects (statement,
//! people, year, equation); equations of motion from a Lagrangian or a
//! Hamiltonian; standard systems (pendulum, driven oscillator, symmetric
//! top, quantum oscillator, Kerr black holes, quantum statistics, random
//! walks); black holes from their mass; thermal de Broglie wavelengths;
//! charged spheres; impact energies; heat-engine cycles; the Balmer series.

use super::nl::ex;
use super::*;
use crate::calculus::diff::diff;
use crate::expr::*;
use crate::print::text as ptext;
use crate::simplify as s;
use crate::stats::fmt;
use regex::Regex;
use std::f64::consts::PI;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

const H: f64 = 6.62607015e-34;
const HBAR: f64 = H / (2.0 * PI);
const KB: f64 = 1.380649e-23;
const C: f64 = 299_792_458.0;
const G: f64 = 6.67430e-11;
const MSUN: f64 = 1.988_41e30;
const AMU: f64 = 1.660_539_066_60e-27;
const KE: f64 = 8.987_551_792_3e9;
const EPS0: f64 = 8.854_187_812_8e-12;

fn input(s: String) -> Pod {
    pod("Input", "Input interpretation", text(s))
}

fn kv(rows: Vec<(&str, String)>) -> Content {
    table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())
}

fn sci(v: f64, unit: &str) -> String {
    let s = if v != 0.0 && (v.abs() >= 1e6 || v.abs() < 1e-3) {
        let e = v.abs().log10().floor() as i32;
        let m = v / 10f64.powi(e);
        format!("{}×10^{e}", crate::print::fmt_float(m, 4))
    } else {
        crate::print::fmt_float(v, 6)
    };
    if unit.is_empty() { s } else { format!("{s} {unit}") }
}

/// A quantity such as "12cm", "5.5 mC", "17 km/s", "400K" → (value in SI, base unit).
fn quantity(t: &str) -> Option<(f64, String)> {
    let m = re!(r"^\s*(-?[\d.]+(?:e-?\d+)?)\s*([a-zμ°/]*)\s*$").captures(t.trim())?;
    let v: f64 = m[1].parse().ok()?;
    let u = m[2].to_string();
    let prefix = |p: &str| -> Option<f64> {
        Some(match p {
            "" => 1.0,
            "k" => 1e3,
            "m" => 1e-3,
            "c" => 1e-2,
            "μ" | "u" => 1e-6,
            "n" => 1e-9,
            "p" => 1e-12,
            "M" => 1e6,
            "g" => 1e9,
            _ => return None,
        })
    };
    let (mult, base) = match u.as_str() {
        "" => (1.0, ""),
        "km/s" => (1e3, "m/s"),
        "m/s" => (1.0, "m/s"),
        "mph" => (0.44704, "m/s"),
        "k" | "kelvin" => (1.0, "K"),
        "c" if false => (1.0, ""),
        "cm" | "mm" | "km" | "m" | "nm" | "μm" | "um" => (prefix(&u[..u.len() - 1])?, "m"),
        "ft" | "feet" => (0.3048, "m"),
        "in" => (0.0254, "m"),
        "mi" | "miles" => (1609.344, "m"),
        "c" | "mc" | "μc" | "uc" | "nc" | "pc" => (prefix(&u[..u.len() - 1])?, "C"),
        "s" | "sec" | "seconds" => (1.0, "s"),
        "min" | "minutes" => (60.0, "s"),
        "h" | "hr" | "hours" => (3600.0, "s"),
        "days" | "day" | "d" => (86400.0, "s"),
        "years" | "yr" | "y" => (3.15576e7, "s"),
        "kg" => (1.0, "kg"),
        "g" => (1e-3, "kg"),
        "t" | "tonnes" => (1e3, "kg"),
        _ => return None,
    };
    Some((v * mult, base.to_string()))
}

// ---------------------------------------------------------------------------
// laws, principles and effects
// ---------------------------------------------------------------------------

struct Law {
    names: &'static [&'static str],
    title: &'static str,
    statement: &'static str,
    equation: &'static str,
    people: &'static str,
    year: &'static str,
}

const LAWS: &[Law] = &[
    Law { names: &["kepler's first law", "keplers first law", "kepler first law", "law of ellipses"], title: "Kepler's first law", statement: "every planet moves on an ellipse with the Sun at one focus", equation: "r = a(1 − e²)/(1 + e cos θ)", people: "Johannes Kepler", year: "1609 (Astronomia nova)" },
    Law { names: &["kepler's second law", "keplers second law", "law of equal areas"], title: "Kepler's second law", statement: "the line from the Sun to a planet sweeps out equal areas in equal times", equation: "dA/dt = L/(2m) = constant", people: "Johannes Kepler", year: "1609 (Astronomia nova)" },
    Law { names: &["kepler's third law", "keplers third law", "law of harmonies"], title: "Kepler's third law", statement: "the square of the orbital period is proportional to the cube of the semi-major axis", equation: "T² = 4π² a³/(G (M + m))", people: "Johannes Kepler", year: "1619 (Harmonices Mundi)" },
    Law { names: &["zeroth law of thermodynamics"], title: "zeroth law of thermodynamics", statement: "two systems each in thermal equilibrium with a third are in thermal equilibrium with each other", equation: "", people: "Ralph H. Fowler (name)", year: "1930s" },
    Law { names: &["first law of thermodynamics"], title: "first law of thermodynamics", statement: "energy is conserved: the change in internal energy equals heat added minus work done by the system", equation: "ΔU = Q − W", people: "Rudolf Clausius, William Thomson, Julius von Mayer, James Joule", year: "1850" },
    Law { names: &["second law of thermodynamics"], title: "second law of thermodynamics", statement: "the entropy of an isolated system never decreases; heat does not flow spontaneously from cold to hot", equation: "dS ≥ δQ/T", people: "Rudolf Clausius, William Thomson (Lord Kelvin), Sadi Carnot", year: "1850–1851" },
    Law { names: &["third law of thermodynamics", "nernst heat theorem", "nernst theorem"], title: "third law of thermodynamics", statement: "the entropy of a perfect crystal approaches a constant (zero) as the temperature approaches absolute zero; absolute zero cannot be reached in finitely many steps", equation: "lim_(T→0) S = 0", people: "Walther Nernst", year: "1906–1912" },
    Law { names: &["newton's first law", "law of inertia"], title: "Newton's first law", statement: "a body stays at rest or in uniform straight motion unless acted on by a net force", equation: "ΣF = 0 ⇒ v = constant", people: "Isaac Newton", year: "1687 (Principia)" },
    Law { names: &["newton's second law"], title: "Newton's second law", statement: "the net force on a body equals the rate of change of its momentum", equation: "F = dp/dt = m a", people: "Isaac Newton", year: "1687 (Principia)" },
    Law { names: &["newton's third law"], title: "Newton's third law", statement: "to every action there is an equal and opposite reaction", equation: "F₁₂ = −F₂₁", people: "Isaac Newton", year: "1687 (Principia)" },
    Law { names: &["newton's law of gravitation", "law of universal gravitation", "gravitation calculation", "newtonian gravitation"], title: "Newton's law of universal gravitation", statement: "two masses attract with a force proportional to the product of the masses and inversely proportional to the square of their distance", equation: "F = G m₁ m₂/r², G = 6.67430×10⁻¹¹ N m²/kg²", people: "Isaac Newton", year: "1687" },
    Law { names: &["twin paradox"], title: "twin paradox", statement: "a twin who travels at high speed and returns is younger than the twin who stayed home; the situation is not symmetric because the traveller changes inertial frames", equation: "Δτ = ∫ √(1 − v²/c²) dt", people: "Paul Langevin", year: "1911" },
    Law { names: &["buckingham pi theorem", "buckingham π theorem", "pi theorem"], title: "Buckingham π theorem", statement: "a physically meaningful relation among n variables involving k independent physical dimensions can be rewritten as a relation among n − k dimensionless groups", equation: "f(q₁, …, q_n) = 0 ⇔ F(π₁, …, π_(n−k)) = 0", people: "Edgar Buckingham (also Aimé Vaschy, Lord Rayleigh)", year: "1914" },
    Law { names: &["abc effect", "abashian-booth-crowe effect", "abashian booth crowe effect"], title: "ABC effect", statement: "an unexpected enhancement in the two-pion invariant-mass spectrum just above threshold in double-pionic fusion reactions such as p d → ³He π π and p n → d π⁰ π⁰; WASA-at-COSY linked it to the d*(2380) dibaryon resonance", equation: "", people: "Norman E. Booth, Alexander Abashian, Kenneth M. Crowe", year: "1960" },
    Law { names: &["mpemba effect"], title: "Mpemba effect", statement: "under some conditions hot water appears to freeze faster than cold water; whether and when the effect is real remains disputed", equation: "", people: "Erasto Mpemba, Denis Osborne", year: "1969" },
    Law { names: &["leidenfrost effect"], title: "Leidenfrost effect", statement: "a liquid touching a surface much hotter than its boiling point floats on an insulating layer of its own vapour and evaporates slowly", equation: "", people: "Johann Gottlob Leidenfrost", year: "1756" },
    Law { names: &["arrhenius law", "arrhenius equation"], title: "Arrhenius law", statement: "reaction rate constants grow exponentially with temperature", equation: "k = A e^(−E_a/(R T))", people: "Svante Arrhenius", year: "1889" },
    Law { names: &["matthew effect"], title: "Matthew effect", statement: "accumulated advantage: recognition and resources flow disproportionately to those who already have them", equation: "", people: "Robert K. Merton", year: "1968" },
    Law { names: &["epr paradox", "einstein-podolsky-rosen paradox"], title: "EPR paradox", statement: "entangled particles show correlations that, if quantum mechanics were complete and local, would imply simultaneous definite values for incompatible observables; Bell's theorem later showed local hidden variables cannot reproduce the predictions", equation: "", people: "Albert Einstein, Boris Podolsky, Nathan Rosen", year: "1935" },
    Law { names: &["schroedinger's cat", "schrodinger's cat", "schrödinger's cat"], title: "Schrödinger's cat", statement: "a thought experiment in which a cat's fate is tied to a quantum event, making a superposition of 'alive' and 'dead' to question when measurement settles outcomes", equation: "|ψ⟩ = (|alive⟩ + |dead⟩)/√2", people: "Erwin Schrödinger", year: "1935" },
    Law { names: &["black hole information paradox", "information paradox"], title: "black hole information paradox", statement: "Hawking radiation appears thermal, so a black hole formed from a pure state would evaporate into a mixed state, in conflict with unitary quantum evolution", equation: "", people: "Stephen Hawking", year: "1976" },
    Law { names: &["church-turing-deutsch principle", "church turing deutsch principle"], title: "Church–Turing–Deutsch principle", statement: "every finitely realizable physical system can be perfectly simulated by a universal model computing machine operating by finite means", equation: "", people: "David Deutsch (after Alonzo Church and Alan Turing)", year: "1985" },
    Law { names: &["brownian motion"], title: "Brownian motion", statement: "random motion of particles suspended in a fluid, driven by molecular collisions; its mean square displacement grows linearly in time", equation: "⟨x²⟩ = 2 D t, D = k_B T/(6π η r)", people: "Robert Brown (observation), Albert Einstein, Marian Smoluchowski (theory)", year: "1827; 1905" },
    Law { names: &["special relativity"], title: "special relativity", statement: "the laws of physics are the same in all inertial frames and the speed of light in vacuum is the same for all observers", equation: "E² = (p c)² + (m c²)², γ = 1/√(1 − v²/c²)", people: "Albert Einstein", year: "1905" },
    Law { names: &["general relativity"], title: "general relativity", statement: "gravity is the curvature of spacetime produced by energy and momentum", equation: "G_μν + Λ g_μν = (8π G/c⁴) T_μν", people: "Albert Einstein", year: "1915" },
    Law { names: &["archimedes principle", "archimedes' principle"], title: "Archimedes' principle", statement: "a body immersed in a fluid feels an upward buoyant force equal to the weight of the fluid it displaces", equation: "F_b = ρ_fluid g V_displaced", people: "Archimedes", year: "c. 250 BC" },
    Law { names: &["bernoulli's principle", "bernoulli principle", "bernoulli equation"], title: "Bernoulli's principle", statement: "along a streamline of steady inviscid incompressible flow, pressure plus kinetic and potential energy densities is constant", equation: "p + ½ρv² + ρ g h = constant", people: "Daniel Bernoulli", year: "1738" },
    Law { names: &["pauli exclusion principle"], title: "Pauli exclusion principle", statement: "no two identical fermions can occupy the same quantum state simultaneously", equation: "ψ(…, x_i, …, x_j, …) = −ψ(…, x_j, …, x_i, …)", people: "Wolfgang Pauli", year: "1925" },
    Law { names: &["heisenberg uncertainty principle", "uncertainty principle"], title: "Heisenberg uncertainty principle", statement: "position and momentum cannot both be sharply defined", equation: "Δx Δp ≥ ħ/2", people: "Werner Heisenberg (Earle Kennard for the inequality)", year: "1927" },
    Law { names: &["doppler effect"], title: "Doppler effect", statement: "the observed frequency of a wave changes when source and observer move relative to each other", equation: "f' = f (c + v_r)/(c − v_s)", people: "Christian Doppler", year: "1842" },
    Law { names: &["photoelectric effect"], title: "photoelectric effect", statement: "light above a threshold frequency ejects electrons from a metal, with kinetic energy depending on frequency, not intensity", equation: "K_max = h f − φ", people: "Heinrich Hertz (discovery), Albert Einstein (explanation)", year: "1887; 1905" },
    Law { names: &["compton effect", "compton scattering"], title: "Compton effect", statement: "X-rays scattered by electrons have longer wavelengths, shifted by an amount depending on the scattering angle", equation: "Δλ = (h/(m_e c))(1 − cos θ)", people: "Arthur Compton", year: "1923" },
    Law { names: &["hall effect"], title: "Hall effect", statement: "a current-carrying conductor in a perpendicular magnetic field develops a transverse voltage", equation: "V_H = I B/(n q t)", people: "Edwin Hall", year: "1879" },
    Law { names: &["zeeman effect"], title: "Zeeman effect", statement: "spectral lines split in a magnetic field", equation: "ΔE = μ_B g m_j B", people: "Pieter Zeeman", year: "1896" },
    Law { names: &["stark effect"], title: "Stark effect", statement: "spectral lines shift and split in an electric field", equation: "", people: "Johannes Stark", year: "1913" },
    Law { names: &["meissner effect"], title: "Meissner effect", statement: "a superconductor expels magnetic fields from its interior on cooling through its transition", equation: "B = 0 inside", people: "Walther Meissner, Robert Ochsenfeld", year: "1933" },
    Law { names: &["casimir effect"], title: "Casimir effect", statement: "two uncharged parallel conducting plates in vacuum attract because of quantum vacuum fluctuations", equation: "F/A = −π² ħ c/(240 a⁴)", people: "Hendrik Casimir", year: "1948" },
    Law { names: &["cherenkov radiation", "cherenkov effect"], title: "Cherenkov radiation", statement: "a charged particle moving faster than light's phase velocity in a medium emits a cone of light", equation: "cos θ = 1/(n β)", people: "Pavel Cherenkov", year: "1934" },
    Law { names: &["aharonov-bohm effect", "aharonov bohm effect"], title: "Aharonov–Bohm effect", statement: "a charged particle's wavefunction acquires a phase from an electromagnetic potential even where the fields vanish", equation: "Δφ = (q/ħ) ∮ A · dl", people: "Yakir Aharonov, David Bohm (Werner Ehrenberg, Raymond Siday earlier)", year: "1959" },
    Law { names: &["coriolis effect"], title: "Coriolis effect", statement: "in a rotating frame, moving bodies appear deflected perpendicular to their velocity", equation: "F = −2 m Ω × v", people: "Gaspard-Gustave de Coriolis", year: "1835" },
    Law { names: &["seebeck effect"], title: "Seebeck effect", statement: "a temperature difference across a junction of two conductors produces a voltage", equation: "V = S ΔT", people: "Thomas Johann Seebeck", year: "1821" },
    Law { names: &["peltier effect"], title: "Peltier effect", statement: "a current through a junction of two conductors heats or cools it", equation: "Q̇ = Π I", people: "Jean Charles Athanase Peltier", year: "1834" },
    Law { names: &["joule-thomson effect", "joule thomson effect"], title: "Joule–Thomson effect", statement: "a real gas changes temperature when it expands through a throttle at constant enthalpy", equation: "μ_JT = (∂T/∂p)_H", people: "James Joule, William Thomson", year: "1852" },
    Law { names: &["tyndall effect"], title: "Tyndall effect", statement: "light is scattered by particles in a colloid, making the beam visible", equation: "", people: "John Tyndall", year: "1859" },
    Law { names: &["hooke's law", "hookes law"], title: "Hooke's law", statement: "the restoring force of a spring is proportional to its extension", equation: "F = −k x", people: "Robert Hooke", year: "1676" },
    Law { names: &["ohm's law", "ohms law"], title: "Ohm's law", statement: "the current through a conductor is proportional to the voltage across it", equation: "V = I R", people: "Georg Ohm", year: "1827" },
    Law { names: &["coulomb's law", "coulombs law"], title: "Coulomb's law", statement: "two point charges exert a force proportional to the product of the charges and inversely proportional to the square of the distance", equation: "F = q₁ q₂/(4π ε₀ r²)", people: "Charles-Augustin de Coulomb", year: "1785" },
];

fn laws(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:what is\s+)?(?:(discovery date|date|year|people|associated people|eponymous people|discoverers?|equation|iconic equation|formula|statement)\s+(?:for|of)\s+)?(?:the\s+)?(.+?)$").captures(l)?;
    let name = m[2].trim();
    let law = LAWS.iter().find(|x| x.names.contains(&name))?;
    let mut rows = vec![("statement", law.statement.to_string())];
    if !law.equation.is_empty() {
        rows.push(("equation", law.equation.to_string()));
    }
    rows.push(("people", law.people.to_string()));
    rows.push(("date", law.year.to_string()));
    let (title, main) = match m.get(1).map(|x| x.as_str()) {
        Some("discovery date" | "date" | "year") => ("Date", law.year.to_string()),
        Some("people" | "associated people" | "eponymous people" | "discoverer" | "discoverers") => ("People", law.people.to_string()),
        Some("equation" | "iconic equation" | "formula") if !law.equation.is_empty() => ("Equation", law.equation.to_string()),
        _ => ("Statement", law.statement.to_string()),
    };
    Some(Answer::new(law.title, vec![input(law.title.to_string()), primary(pod("Result", title, text(main))), pod("Details", "Details", kv(rows))]))
}

// ---------------------------------------------------------------------------
// Lagrangians and Hamiltonians
// ---------------------------------------------------------------------------

fn mechanics(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:the\s+)?lagrangian\s+(?:l\s*=\s*)?(.+)$").captures(l) {
        // q' → an unused one-letter symbol (the parser splits longer names)
        let raw = m[1].to_string();
        let c = re!(r"([a-zθφ])'").captures(&raw)?[1].to_string();
        let free: Vec<char> = "vwuzyshgfb".chars().filter(|ch| !raw.contains(*ch)).collect();
        if free.len() < 2 || re!(r"([a-zθφ])'").captures_iter(&raw).any(|x| x[1] != c) {
            return None;
        }
        let (vn, an) = (free[0].to_string(), free[1].to_string());
        let body = raw.replace(&format!("{c}'"), &vn);
        let lag = ex(&body)?;
        let q = sym(&c);
        let qd = sym(&vn);
        let qdd = sym(&an);
        let p = diff(&lag, &qd);
        // d/dt ∂L/∂q' = ∂²L/∂q'∂q q' + ∂²L/∂q'² q''
        let dt = add(vec![mul(vec![diff(&p, &q), qd.clone()]), mul(vec![diff(&p, &qd), qdd.clone()])]);
        let el = s::simplify(&sub(dt, diff(&lag, &q)));
        let roots = crate::solve::solve(&eq(el.clone(), zero()), &qdd).roots;
        let back = [(qd.clone(), sym(&format!("{c}'"))), (qdd.clone(), sym(&format!("{c}''")))];
        let shown = |e: &Expr| -> String { ptext(&e.subs_many(&back)) };
        let coords = [c.clone()];
        let eom = match roots.first() {
            Some(r) => format!("{}'' = {}", coords[0], shown(&s::simplify(r))),
            None => format!("{} = 0", shown(&el)),
        };
        let energy = s::simplify(&sub(mul(vec![p.clone(), qd.clone()]), lag.clone()));
        return Some(Answer::new("Lagrangian", vec![input(format!("Lagrangian L = {}", shown(&lag))), primary(pod("Result", "Euler–Lagrange equation", text(eom))), pod("Momentum", "Canonical momentum ∂L/∂q'", text(shown(&s::simplify(&p)))), pod("Energy", "Energy function q' ∂L/∂q' − L", text(shown(&energy)))]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?hamiltonian\s+(?:h\s*=\s*)?(.+)$").captures(l) {
        let ham = ex(&m[1])?;
        let fs = ham.free_symbols();
        let p = sym("p");
        let qn = fs.iter().find(|n| &***n != "p")?.to_string();
        if fs.len() != 2 || !fs.iter().any(|n| &**n == "p") {
            return None;
        }
        let q = sym(&qn);
        let qdot = s::simplify(&diff(&ham, &p));
        let pdot = s::simplify(&neg(diff(&ham, &q)));
        return Some(Answer::new("Hamiltonian", vec![input(format!("Hamiltonian H = {}", ptext(&ham))), primary(pod("Result", "Hamilton's equations", text(format!("{qn}' = ∂H/∂p = {};  p' = −∂H/∂{qn} = {}", ptext(&qdot), ptext(&pdot))))), pod("Conserved", "Conserved quantity", text("H itself (it has no explicit time dependence)"))]));
    }
    None
}

// ---------------------------------------------------------------------------
// central-force systems in curvilinear coordinates
// ---------------------------------------------------------------------------

fn coordinate_systems(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(?:lagrangian (?:of|for) (?:an? |the )?)?(?:classical\s+|isotropic\s+)*(harmonic oscillator|inverse[- ]distance potential|inverse[- ]square (?:force|law)|kepler problem|coulomb potential|free particle|particle)\s+in\s+(\d)\s*d(?:imensions)?\s+in\s+(polar|cylindrical|spherical|hyperspherical|cartesian)\s+coordinates$").captures(l)?;
    let kind = &m[1];
    let dim: usize = m[2].parse().ok()?;
    let sys = &m[3];
    // coordinates and the diagonal metric factor of each (T = m/2 Σ g q'²)
    let r = sym("r");
    let names: Vec<String> = match (sys, dim) {
        ("polar", 2) => vec!["r".into(), "θ".into()],
        ("cylindrical", 3) => vec!["r".into(), "φ".into(), "z".into()],
        ("spherical", 3) => vec!["r".into(), "θ".into(), "φ".into()],
        ("hyperspherical", d) if (2..=6).contains(&d) => {
            let angles = ["ψ", "χ", "θ", "φ", "ω"];
            let mut v = vec!["r".to_string()];
            v.extend(angles[angles.len() + 1 - d..].iter().map(|a| a.to_string()));
            v
        }
        ("cartesian", d) if (1..=4).contains(&d) => ["x", "y", "z", "w"][..d].iter().map(|s| s.to_string()).collect(),
        _ => return None,
    };
    let q: Vec<Expr> = names.iter().map(|n| sym(n)).collect();
    let qd: Vec<Expr> = names.iter().map(|n| sym(&format!("{n}'"))).collect();
    let qdd: Vec<Expr> = names.iter().map(|n| sym(&format!("{n}''"))).collect();
    let mut g: Vec<Expr> = Vec::new();
    match sys {
        "cartesian" => g = vec![int(1); dim],
        "cylindrical" => g = vec![int(1), pow(r.clone(), int(2)), int(1)],
        _ => {
            // r, then angles: r², r² sin²(a₁), r² sin²(a₁) sin²(a₂), …
            g.push(int(1));
            let mut f = pow(r.clone(), int(2));
            for k in 1..q.len() {
                g.push(f.clone());
                f = mul(vec![f, pow(func("sin", vec![q[k].clone()]), int(2))]);
            }
        }
    }
    let radius2 = if sys == "cartesian" { add(q.iter().map(|x| pow(x.clone(), int(2))).collect()) } else { pow(r.clone(), int(2)) };
    let radius = if sys == "cartesian" { pow(radius2.clone(), rat(1, 2)) } else { r.clone() };
    let (mass, k) = (sym("m"), sym("k"));
    let (v, vname) = if kind.contains("harmonic") {
        (mul(vec![rat(1, 2), k.clone(), radius2.clone()]), "V = ½ k r²")
    } else if kind.contains("inverse") || kind.contains("kepler") || kind.contains("coulomb") {
        (neg(mul(vec![k.clone(), pow(radius.clone(), int(-1))])), "V = −k/r")
    } else {
        (int(0), "V = 0")
    };
    let t = mul(vec![rat(1, 2), mass.clone(), add(g.iter().zip(&qd).map(|(gi, v)| mul(vec![gi.clone(), pow(v.clone(), int(2))])).collect())]);
    let lag = s::simplify(&sub(t, v.clone()));
    let mut eoms = Vec::new();
    let mut conserved = Vec::new();
    for i in 0..q.len() {
        let p = s::simplify(&diff(&lag, &qd[i]));
        let mut terms = Vec::new();
        for j in 0..q.len() {
            terms.push(mul(vec![diff(&p, &q[j]), qd[j].clone()]));
            terms.push(mul(vec![diff(&p, &qd[j]), qdd[j].clone()]));
        }
        let dl = s::simplify(&diff(&lag, &q[i]));
        if dl.is_zero() {
            conserved.push(format!("p_{} = {}", names[i], ptext(&p)));
        }
        let el = s::simplify(&sub(add(terms), dl));
        let sol = crate::solve::solve(&eq(el.clone(), zero()), &qdd[i]).roots;
        eoms.push(match sol.first() {
            Some(rt) => format!("{}'' = {}", names[i], ptext(&s::simplify(rt))),
            None => format!("{} = 0", ptext(&el)),
        });
    }
    // Hamiltonian H = Σ p²/(2 m g) + V
    let ham = s::simplify(&add(vec![
        add(names.iter().zip(&g).map(|(n, gi)| mul(vec![pow(sym(&format!("p_{n}")), int(2)), pow(mul(vec![int(2), mass.clone(), gi.clone()]), int(-1))])).collect()),
        v.clone(),
    ]));
    let title = format!("{} in {dim}D in {sys} coordinates", if kind.contains("harmonic") { "isotropic harmonic oscillator" } else if kind.contains("free") || kind == "particle" { "free particle" } else { "inverse-distance potential" });
    let mut pods = vec![
        input(title.clone()),
        primary(pod("Lagrangian", "Lagrangian", text(format!("L = {}", ptext(&lag))))),
        pod("EOM", "Euler–Lagrange equations", text(eoms.join("\n"))),
        pod("Hamiltonian", "Hamiltonian", text(format!("H = {}", ptext(&ham)))),
        pod("Coordinates", "Coordinates and potential", text(format!("generalized coordinates {}; {vname}; m mass, k constant", names.join(", ")))),
    ];
    if !conserved.is_empty() {
        pods.push(pod("Conserved", "Conserved momenta (cyclic coordinates)", text(conserved.join("\n"))));
    }
    Some(Answer::new("Lagrangian system", pods))
}

// ---------------------------------------------------------------------------
// named physical systems
// ---------------------------------------------------------------------------

fn systems(l: &str) -> Option<Answer> {
    let l = l.trim_end_matches(" (physical system)").trim_start_matches("the ").trim_start_matches("a ");
    let l = re!(r"^(?:lagrangian|equations of motion|hamiltonian) (?:of|for) (?:an? |the )?").replace(l, "").to_string();
    let l = l.trim_end_matches(" system").trim_end_matches(" lagrangian");
    let (name, main, rows): (&str, &str, Vec<(&str, &str)>) = match l {
        "coupled pendulum" | "coupled pendulums" | "coupled pendula" | "two coupled pendulums" => ("two pendulums coupled by a spring", "L = ½ m L² (θ₁'² + θ₂'²) + m g L (cos θ₁ + cos θ₂) − ½ k L² (sin θ₁ − sin θ₂)²", vec![("small-angle equations", "θ₁'' = −(g/L) θ₁ − (k/m)(θ₁ − θ₂);  θ₂'' = −(g/L) θ₂ + (k/m)(θ₁ − θ₂)"), ("in-phase normal mode", "ω₁ = √(g/L)"), ("out-of-phase normal mode", "ω₂ = √(g/L + 2k/m)"), ("beats", "energy passes between the pendulums at the beat frequency ω₂ − ω₁")]),
        "double pendulum" => ("double pendulum", "L = ½ (m₁ + m₂) l₁² θ₁'² + ½ m₂ l₂² θ₂'² + m₂ l₁ l₂ θ₁' θ₂' cos(θ₁ − θ₂) + (m₁ + m₂) g l₁ cos θ₁ + m₂ g l₂ cos θ₂", vec![("equation for θ₁", "(m₁ + m₂) l₁ θ₁'' + m₂ l₂ θ₂'' cos(θ₁ − θ₂) + m₂ l₂ θ₂'² sin(θ₁ − θ₂) + (m₁ + m₂) g sin θ₁ = 0"), ("equation for θ₂", "l₂ θ₂'' + l₁ θ₁'' cos(θ₁ − θ₂) − l₁ θ₁'² sin(θ₁ − θ₂) + g sin θ₂ = 0"), ("small-angle modes (m₁ = m₂, l₁ = l₂ = l)", "ω² = (2 ∓ √2) g/l"), ("behaviour", "chaotic for large amplitudes")]),
        "spherical pendulum" => ("spherical pendulum", "L = ½ m L² (θ'² + sin²θ φ'²) + m g L cos θ", vec![("θ equation", "θ'' = sin θ cos θ φ'² − (g/L) sin θ"), ("conserved", "p_φ = m L² sin²θ φ' (angular momentum about the vertical) and the energy"), ("conical pendulum", "steady motion at θ₀ with φ'² = g/(L cos θ₀)")]),
        "pendulum" | "simple pendulum" => ("simple pendulum", "θ'' + (g/L) sin θ = 0", vec![("small-angle period", "T = 2π √(L/g)"), ("exact period", "T = 4 √(L/g) K(sin²(θ₀/2)), K the complete elliptic integral"), ("energy", "E = ½ m L² θ'² − m g L cos θ")]),
        "damped harmonic oscillator with forcing" | "damped driven harmonic oscillator" | "driven damped harmonic oscillator" | "damped sinusoidal driven harmonic oscillator" | "forced damped oscillator" => ("driven damped harmonic oscillator", "m x'' + c x' + k x = F₀ cos(ω t)", vec![("natural frequency", "ω₀ = √(k/m)"), ("damping ratio", "ζ = c/(2√(m k))"), ("steady-state amplitude", "A = (F₀/m)/√((ω₀² − ω²)² + (c ω/m)²)"), ("phase lag", "φ = atan2(c ω/m, ω₀² − ω²)"), ("resonance (amplitude)", "ω_r = ω₀ √(1 − 2ζ²) for ζ < 1/√2")]),
        "damped harmonic oscillator" => ("damped harmonic oscillator", "m x'' + c x' + k x = 0", vec![("underdamped (ζ < 1)", "x = A e^(−ζω₀t) cos(ω₀√(1 − ζ²) t + φ)"), ("critically damped (ζ = 1)", "x = (A + B t) e^(−ω₀ t)"), ("overdamped (ζ > 1)", "x = A e^(r₁t) + B e^(r₂t), r = −ω₀(ζ ± √(ζ² − 1))")]),
        "harmonic oscillator" | "simple harmonic oscillator" | "classical harmonic oscillator" => ("harmonic oscillator", "m x'' + k x = 0", vec![("solution", "x = A cos(ω t + φ), ω = √(k/m)"), ("period", "T = 2π √(m/k)"), ("energy", "E = ½ m x'² + ½ k x² = ½ k A²")]),
        "quantum harmonic oscillator" => ("quantum harmonic oscillator", "Ĥ = p̂²/(2m) + ½ m ω² x̂²", vec![("energy levels", "E_n = ħω (n + ½), n = 0, 1, 2, …"), ("ground state", "ψ₀(x) = (mω/(πħ))^(1/4) e^(−mωx²/(2ħ))"), ("eigenfunctions", "ψ_n ∝ H_n(√(mω/ħ) x) e^(−mωx²/(2ħ)), H_n Hermite polynomials"), ("ladder operators", "a = √(mω/(2ħ)) (x̂ + i p̂/(mω)), [a, a†] = 1")]),
        "free particle" | "quantum free particle" => ("free particle", "iħ ∂ψ/∂t = −(ħ²/(2m)) ∂²ψ/∂x²", vec![("plane waves", "ψ = e^(i(kx − ωt)), E = ħω = ħ²k²/(2m)"), ("classical motion", "x(t) = x₀ + v t"), ("Gaussian packet width", "σ(t) = σ₀ √(1 + (ħt/(2mσ₀²))²)")]),
        "particle in a box" | "infinite square well" => ("particle in a box", "−(ħ²/(2m)) ψ'' = E ψ, ψ(0) = ψ(L) = 0", vec![("energy levels", "E_n = n² π² ħ²/(2 m L²), n = 1, 2, …"), ("eigenfunctions", "ψ_n(x) = √(2/L) sin(nπx/L)")]),
        "poeschl-teller potential" | "pöschl-teller potential" | "poschl-teller potential" => ("Pöschl–Teller potential", "V(x) = −(ħ²/(2m)) λ(λ + 1) sech²(x)", vec![("bound states", "E_n = −(ħ²/(2m)) (λ − n)², n = 0, 1, …, n < λ"), ("reflectionless", "for integer λ the potential reflects no incident wave")]),
        "symmetric top" | "motion of a symmetric top" | "equations of motion for a symmetric top" | "heavy symmetric top" => ("heavy symmetric top", "L = ½ I₁ (θ'² + φ'² sin²θ) + ½ I₃ (ψ' + φ' cos θ)² − M g ℓ cos θ", vec![("conserved", "p_ψ = I₃(ψ' + φ' cos θ) = I₃ ω₃, p_φ = I₁ φ' sin²θ + p_ψ cos θ, and the energy"), ("θ equation", "I₁ θ'' = I₁ φ'² sin θ cos θ − p_ψ φ' sin θ + M g ℓ sin θ"), ("precession", "slow precession rate φ' ≈ M g ℓ/(I₃ ω₃) for a fast top"), ("motion", "θ oscillates between two turning angles (nutation) while φ advances (precession)")]),
        "kerr black hole" | "kerr metric" | "rotating black hole" => ("Kerr black hole", "ds² from the Kerr metric with mass M and spin a = J/(Mc)", vec![("horizons", "r± = r_g ± √(r_g² − a²), r_g = GM/c²"), ("ergosphere", "r_E = r_g + √(r_g² − a² cos²θ)"), ("extremal limit", "a = r_g (J = G M²/c)"), ("area", "A = 4π (r₊² + a²)")]),
        "kerr-newman black hole" | "black hole formula with charge and angular momentum" | "charged rotating black hole" => ("Kerr–Newman black hole", "horizons r± = r_g ± √(r_g² − a² − r_Q²)", vec![("parameters", "r_g = GM/c², a = J/(Mc), r_Q² = G Q²/(4π ε₀ c⁴)"), ("no horizon (naked singularity)", "when a² + r_Q² > r_g²"), ("special cases", "Kerr (Q = 0), Reissner–Nordström (J = 0), Schwarzschild (both zero)")]),
        "fermi-dirac distribution" | "fermi dirac distribution" => ("Fermi–Dirac distribution", "⟨n_i⟩ = 1/(e^((ε_i − μ)/(k_B T)) + 1)", vec![("applies to", "fermions (half-integer spin)"), ("at T = 0", "a step: 1 below the Fermi energy, 0 above")]),
        "bose-einstein distribution" | "bose einstein distribution" => ("Bose–Einstein distribution", "⟨n_i⟩ = 1/(e^((ε_i − μ)/(k_B T)) − 1)", vec![("applies to", "bosons (integer spin); μ ≤ lowest energy"), ("condensation (ideal gas)", "T_c = (2π ħ²/(m k_B)) (n/ζ(3/2))^(2/3)")]),
        "maxwell-boltzmann distribution" | "maxwell boltzmann distribution" => ("Maxwell–Boltzmann distribution", "f(v) = 4π v² (m/(2π k_B T))^(3/2) e^(−m v²/(2 k_B T))", vec![("most probable speed", "√(2 k_B T/m)"), ("mean speed", "√(8 k_B T/(π m))"), ("rms speed", "√(3 k_B T/m)")]),
        "levy distribution" | "lévy distribution" => ("Lévy distribution", "f(x) = √(c/(2π)) e^(−c/(2(x − μ)))/(x − μ)^(3/2), x > μ", vec![("mean and variance", "infinite"), ("median", "μ + c/(2 (erfc⁻¹(1/2))²)"), ("stable", "a stable distribution with α = 1/2, β = 1")]),
        "random walk in 1d" | "1d random walk" | "random walk" => ("one-dimensional random walk", "x_(n+1) = x_n ± 1 with equal probability", vec![("mean position", "0"), ("rms distance after N steps", "√N"), ("return to the origin", "certain (probability 1, Pólya)")]),
        "2d random walk" | "random walk in 2d" => ("two-dimensional random walk", "unit steps in one of 4 lattice directions", vec![("rms distance after N steps", "√N"), ("return to the origin", "certain (probability 1, Pólya)")]),
        "3d random walk" | "random walk in 3d" => ("three-dimensional random walk", "unit steps in one of 6 lattice directions", vec![("rms distance after N steps", "√N"), ("return to the origin", "probability ≈ 0.3405 (Pólya: the walk is transient)")]),
        "isobaric process" => ("isobaric process", "constant pressure", vec![("work", "W = p ΔV"), ("heat (ideal gas)", "Q = n C_p ΔT"), ("relation", "V/T = constant")]),
        "isochoric process" | "isovolumetric process" => ("isochoric process", "constant volume", vec![("work", "W = 0"), ("heat (ideal gas)", "Q = n C_v ΔT"), ("relation", "p/T = constant")]),
        "isothermal process" => ("isothermal process", "constant temperature", vec![("work (ideal gas)", "W = n R T ln(V₂/V₁)"), ("heat", "Q = W (ΔU = 0)"), ("relation", "p V = constant")]),
        "adiabatic process" => ("adiabatic process", "no heat exchange", vec![("relation", "p V^γ = constant, T V^(γ−1) = constant"), ("work (ideal gas)", "W = (p₁V₁ − p₂V₂)/(γ − 1)")]),
        "diesel cycle" | "diesel engine cycle" => ("Diesel cycle", "η = 1 − (1/r^(γ−1)) (ρ^γ − 1)/(γ (ρ − 1))", vec![("strokes", "adiabatic compression, isobaric heat addition, adiabatic expansion, isochoric heat rejection"), ("r", "compression ratio V₁/V₂"), ("ρ", "cutoff ratio V₃/V₂")]),
        "otto cycle" | "otto engine cycle" => ("Otto cycle", "η = 1 − 1/r^(γ−1)", vec![("strokes", "adiabatic compression, isochoric heat addition, adiabatic expansion, isochoric heat rejection"), ("r", "compression ratio")]),
        "carnot cycle" => ("Carnot cycle", "η = 1 − T_c/T_h", vec![("strokes", "isothermal expansion, adiabatic expansion, isothermal compression, adiabatic compression"), ("note", "the highest efficiency of any engine between the two temperatures")]),
        "brayton cycle" => ("Brayton cycle", "η = 1 − 1/r_p^((γ−1)/γ)", vec![("strokes", "adiabatic compression, isobaric heating, adiabatic expansion, isobaric cooling"), ("r_p", "pressure ratio")]),
        "rankine cycle" => ("Rankine cycle", "η = (W_turbine − W_pump)/Q_in = ((h₁ − h₂) − (h₄ − h₃))/(h₁ − h₄)", vec![("stages", "pump (liquid compression), boiler (isobaric heating), turbine (adiabatic expansion), condenser (isobaric cooling)"), ("note", "the working fluid changes phase; enthalpies come from steam tables")]),
        _ => return None,
    };
    Some(Answer::new(name, vec![input(name.to_string()), primary(pod("Result", "Equation", text(main))), pod("Properties", "Properties", table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect()))]))
}

// ---------------------------------------------------------------------------
// black holes
// ---------------------------------------------------------------------------

/// Masses in solar masses (published estimates), distances in light years
/// (NaN when cosmological), kind ("stellar" or "supermassive") and a note.
type Bh = (&'static [&'static str], &'static str, f64, f64, &'static str, &'static str);
const BLACK_HOLES: &[Bh] = &[
    (&["sagittarius a*", "sgr a*", "sagittarius a", "milky way black hole", "black hole at the center of the milky way", "black hole in the center of the milky way"], "Sagittarius A*", 4.3e6, 26_670.0, "supermassive", "Milky Way centre; GRAVITY and EHT estimates ≈ 4.0–4.3 million M☉"),
    (&["m87*", "m87 black hole", "black hole in the center of m87", "black hole at the center of m87", "m87", "powehi"], "M87*", 6.5e9, 53.5e6, "supermassive", "first black hole imaged (EHT, 2019)"),
    (&["m31*", "andromeda black hole", "m31 black hole", "andromeda galaxy black hole", "black hole in the center of andromeda"], "M31* (Andromeda)", 1.4e8, 2.5e6, "supermassive", "centre of the Andromeda Galaxy"),
    (&["ton 618", "ton618"], "TON 618", 4.07e10, f64::NAN, "supermassive", "quasar at redshift 2.22; earlier estimates up to 6.6×10¹⁰ M☉"),
    (&["holmberg 15a", "holm 15a"], "Holmberg 15A", 4.0e10, 700e6, "supermassive", "central galaxy of the cluster Abell 85; mass from stellar dynamics (2019)"),
    (&["ngc 4889"], "NGC 4889", 2.1e10, 308e6, "supermassive", "brightest galaxy of the Coma Cluster"),
    (&["oj 287"], "OJ 287", 1.8e10, 3.5e9, "supermassive", "blazar; primary of a binary black hole"),
    (&["3c 273"], "3C 273", 8.9e8, 2.4e9, "supermassive", "the first quasar identified (1963)"),
    (&["sombrero galaxy black hole", "m104 black hole", "m104*"], "M104 (Sombrero)", 1.0e9, 31.1e6, "supermassive", "centre of the Sombrero Galaxy"),
    (&["centaurus a black hole", "centaurus a*", "ngc 5128 black hole"], "Centaurus A", 5.5e7, 12e6, "supermassive", "nearest active radio galaxy"),
    (&["m81 black hole", "m81*", "bode's galaxy black hole"], "M81*", 7.0e7, 11.8e6, "supermassive", "centre of Bode's Galaxy"),
    (&["ngc 4258 black hole", "m106 black hole", "m106*"], "NGC 4258 (M106)", 3.9e7, 23.7e6, "supermassive", "mass from water masers orbiting the nucleus"),
    (&["gaia bh1"], "Gaia BH1", 9.62, 1_560.0, "stellar", "nearest known black hole, found by Gaia astrometry, 2022"),
    (&["gaia bh2"], "Gaia BH2", 8.94, 3_800.0, "stellar", "Gaia astrometric binary, 2023"),
    (&["gaia bh3"], "Gaia BH3", 32.7, 1_926.0, "stellar", "most massive known stellar black hole in the Milky Way, 2024"),
    (&["cygnus x-1", "cyg x-1"], "Cygnus X-1", 21.2, 7_240.0, "stellar", "first widely accepted black hole (X-ray binary, 1964)"),
    (&["v404 cygni", "v404 cyg"], "V404 Cygni", 9.0, 7_800.0, "stellar", "X-ray nova (outbursts 1989 and 2015)"),
    (&["v616 monocerotis", "v616 mon", "a0620-00", "a0620"], "V616 Monocerotis (A0620-00)", 6.6, 3_300.0, "stellar", "X-ray nova of 1975"),
    (&["xte j1118+480", "kv ursae majoris", "kv uma"], "XTE J1118+480", 7.5, 5_700.0, "stellar", "X-ray transient in Ursa Major"),
    (&["gro j1655-40", "v1033 scorpii", "v1033 sco"], "GRO J1655-40", 5.4, 11_000.0, "stellar", "microquasar in Scorpius"),
    (&["gu muscae", "gu mus", "grs 1124-683", "nova muscae 1991"], "GU Muscae (GRS 1124-683)", 11.0, 16_300.0, "stellar", "X-ray nova of 1991; mass estimate of 2016"),
    (&["v381 normae", "v0381 normae", "v381 nor", "xte j1550-564"], "V381 Normae (XTE J1550-564)", 9.1, 14_300.0, "stellar", "microquasar in Norma"),
    (&["grs 1915+105", "v1487 aquilae", "v1487 aql"], "GRS 1915+105", 12.4, 28_000.0, "stellar", "microquasar with superluminal jets"),
    (&["v4641 sagittarii", "v4641 sgr"], "V4641 Sagittarii", 6.4, 20_000.0, "stellar", "X-ray binary"),
    (&["m33 x-7", "m33 black hole"], "M33 X-7", 15.65, 2.73e6, "stellar", "eclipsing X-ray binary in the Triangulum Galaxy (M33 has no detected central supermassive black hole)"),
    (&["lmc x-1"], "LMC X-1", 10.9, 165_000.0, "stellar", "X-ray binary in the Large Magellanic Cloud"),
    (&["gw150914", "gw150914 remnant"], "GW150914 remnant", 62.0, 1.3e9, "stellar", "product of the first observed black hole merger (LIGO, 14 September 2015)"),
];

fn find_bh(name: &str) -> Option<&'static Bh> {
    let full = name.trim().trim_start_matches("the ").trim();
    let n = full.trim_end_matches(" black hole").trim_end_matches(" system").trim();
    BLACK_HOLES.iter().find(|b| b.0.contains(&full)).or_else(|| BLACK_HOLES.iter().find(|b| b.0.contains(&n)))
}

fn ly_text(d: f64) -> String {
    if d.is_nan() {
        "cosmological (see note)".into()
    } else if d >= 1e6 {
        format!("{} million light years", fmt(d / 1e6))
    } else {
        format!("{} light years", fmt(d))
    }
}

fn bh_props(name: &str, msun: f64, dist: f64, note: &str, want: Option<&str>) -> Answer {
    let m = msun * MSUN;
    let rs = 2.0 * G * m / (C * C);
    let t = HBAR * C.powi(3) / (8.0 * PI * G * m * KB);
    let s_over_k = 4.0 * PI * G * m * m / (HBAR * C);
    let tev = 5120.0 * PI * G * G * m.powi(3) / (HBAR * C.powi(4)) / 3.15576e7;
    let mut rows = vec![("mass", format!("{} M☉ = {}", sci(msun, ""), sci(m, "kg"))), ("Schwarzschild radius", if rs > 1e9 { format!("{} = {} AU", sci(rs, "m"), sci(rs / 1.495978707e11, "")) } else { sci(rs / 1e3, "km") }), ("Hawking temperature", sci(t, "K")), ("Bekenstein–Hawking entropy", format!("{} k_B", sci(s_over_k, ""))), ("evaporation time", sci(tev, "years"))];
    if dist != 0.0 {
        rows.push(("distance", ly_text(dist)));
    }
    rows.push(("note", note.to_string()));
    let (title, main) = match want {
        Some("radius") => ("Schwarzschild radius", rows[1].1.clone()),
        Some("entropy") => ("Entropy", rows[3].1.clone()),
        Some("temperature") => ("Hawking temperature", rows[2].1.clone()),
        Some("distance") if dist != 0.0 => ("Distance", ly_text(dist)),
        _ => ("Mass", rows[0].1.clone()),
    };
    Answer::new("black hole", vec![input(format!("{name} black hole")), primary(pod("Result", title, text(main))), pod("Properties", "Properties (non-rotating estimate)", kv(rows))])
}

fn bh_table(title: &str, list: &[&Bh]) -> Answer {
    let rows: Vec<Vec<Content>> = list.iter().map(|b| vec![text(b.1), text(format!("{} M☉", sci(b.2, ""))), text(ly_text(b.3)), text(b.4)]).collect();
    let main = list.iter().map(|b| b.1).collect::<Vec<_>>().join(" | ");
    Answer::new(title, vec![input(title.to_string()), primary(pod("Result", "Result", text(main))), pod("Table", "Black holes", table(Some(vec!["name", "mass", "distance", "kind"]), rows)), pod("Note", "Note", text("published estimates; masses of distant black holes are uncertain by tens of percent"))])
}

fn black_holes(l: &str) -> Option<Answer> {
    // lists: "stellar mass black holes", "7 nearest black holes", "supermassive black holes"
    if let Some(m) = re!(r"^(?:the\s+|list of\s+)?(?:(\d+)\s+)?(stellar[- ]mass|supermassive|nearest|closest|most massive|largest|biggest|known|famous)\s+black holes?(?:\s+to earth)?$").captures(l) {
        let k: Option<usize> = m.get(1).and_then(|x| x.as_str().parse().ok());
        let kind = m[2].replace('-', " ");
        let mut list: Vec<&Bh> = BLACK_HOLES.iter().collect();
        let title = match kind.as_str() {
            "stellar mass" => {
                list.retain(|b| b.4 == "stellar");
                list.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
                "stellar-mass black holes (nearest first)"
            }
            "supermassive" => {
                list.retain(|b| b.4 == "supermassive");
                list.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
                "supermassive black holes (most massive first)"
            }
            "nearest" | "closest" => {
                list.retain(|b| !b.3.is_nan());
                list.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
                if m.get(1).is_none() && l.ends_with("hole") {
                    let b = list[0];
                    return Some(bh_props(b.1, b.2, b.3, b.5, Some("distance")));
                }
                "nearest known black holes"
            }
            "most massive" | "largest" | "biggest" => {
                list.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
                "most massive known black holes"
            }
            _ => "known black holes",
        };
        if let Some(k) = k {
            list.truncate(k.max(1));
        }
        return Some(bh_table(title, &list));
    }
    let (want, rest) = match re!(r"^(?:the\s+)?(mass|masses|radius|schwarzschild radius|event horizon radius|entropy|temperature|hawking temperature|distance)\s+(?:of|to)\s+(?:the\s+)?(.+)$").captures(l) {
        Some(m) => (Some(match &m[1] {
            "schwarzschild radius" | "radius" | "event horizon radius" => "radius",
            "hawking temperature" | "temperature" => "temperature",
            "entropy" => "entropy",
            "distance" => "distance",
            _ => "mass",
        }), m[2].to_string()),
        None => (None, l.to_string()),
    };
    let items: Vec<&str> = re!(r"\s*(?:,\s*(?:and\s+)?|/|\s+vs\.?\s+|\s+versus\s+|\s+and\s+)\s*").split(&rest).filter(|s| !s.trim().is_empty()).collect();
    if !items.is_empty() && items.len() <= 8 {
        let found: Option<Vec<&Bh>> = items.iter().map(|s| find_bh(s)).collect();
        if let Some(found) = found {
            if found.len() == 1 {
                let b = found[0];
                return Some(bh_props(b.1, b.2, b.3, b.5, want));
            }
            let val = |b: &Bh| -> String {
                let m = b.2 * MSUN;
                match want {
                    Some("radius") => sci(2.0 * G * m / (C * C) / 1e3, "km"),
                    Some("entropy") => format!("{} k_B", sci(4.0 * PI * G * m * m / (HBAR * C), "")),
                    Some("temperature") => sci(HBAR * C.powi(3) / (8.0 * PI * G * m * KB), "K"),
                    Some("distance") => ly_text(b.3),
                    _ => format!("{} M☉", sci(b.2, "")),
                }
            };
            let label = match want { Some("radius") => "Schwarzschild radius", Some("entropy") => "Bekenstein–Hawking entropy", Some("temperature") => "Hawking temperature", Some("distance") => "distance", _ => "mass" };
            let main = found.iter().map(|b| format!("{}: {}", b.1, val(b))).collect::<Vec<_>>().join(" | ");
            let rows: Vec<Vec<Content>> = found.iter().map(|b| vec![text(b.1), text(format!("{} M☉", sci(b.2, ""))), text(ly_text(b.3)), text(sci(2.0 * G * b.2 * MSUN / (C * C) / 1e3, "km"))]).collect();
            return Some(Answer::new("black holes", vec![input(format!("{} | {label}", found.iter().map(|b| b.1).collect::<Vec<_>>().join(", "))), primary(pod("Result", label, text(main))), pod("Table", "Comparison", table(Some(vec!["name", "mass", "distance", "Schwarzschild radius"]), rows))]));
        }
    }
    let m = re!(r"^(?:a\s+)?(?:black hole\s+(?:of\s+|with\s+mass\s+)?([\d.e+]+)\s*(?:solar masses|msun|m☉|suns)|([\d.e+]+)\s*(?:solar mass(?:es)?|msun|m☉)\s+black hole)$").captures(l)?;
    let ms: f64 = m.get(1).or(m.get(2))?.as_str().parse().ok()?;
    Some(bh_props(&format!("{} M☉", fmt(ms)), ms, 0.0, "computed from the mass alone", None))
}

// ---------------------------------------------------------------------------
// small calculations
// ---------------------------------------------------------------------------

fn calcs(l: &str) -> Option<Answer> {
    // thermal de Broglie wavelength
    if let Some(m) = re!(r"^(?:the\s+)?thermal (?:de broglie )?wavelength (?:of\s+)?([a-z]+)\s+(?:at\s+)?([\d.]+)\s*k$").captures(l) {
        let el = crate::domains::chem::find_element(&m[1])?;
        let t: f64 = m[2].parse().ok()?;
        let mass = el.mass * AMU;
        let lam = H / (2.0 * PI * mass * KB * t).sqrt();
        return Some(Answer::new("thermal wavelength", vec![input(format!("thermal de Broglie wavelength of {} at {} K", el.name, fmt(t))), primary(pod("Result", "Result", text(format!("{} = {} Å", sci(lam, "m"), crate::print::fmt_float(lam * 1e10, 5))))), pod("Formula", "Formula", text(format!("λ = h/√(2π m k_B T), m = {} u", fmt(el.mass))))]));
    }
    // Balmer and other hydrogen series
    if let Some(m) = re!(r"^(?:the\s+)?(lyman|balmer|paschen|brackett|pfund)\s+series(?:\s+spectrum|\s+lines|\s+wavelengths)?$").captures(l) {
        let n1 = match &m[1] { "lyman" => 1, "balmer" => 2, "paschen" => 3, "brackett" => 4, _ => 5 };
        let rh = 10_973_731.568_16 / (1.0 + 9.109_383_7015e-31 / 1.672_621_923_69e-27);
        let names = ["α", "β", "γ", "δ", "ε", "ζ"];
        let prefix = match n1 { 1 => "Ly", 2 => "H", 3 => "Pa", 4 => "Br", _ => "Pf" };
        let rows: Vec<Vec<Content>> = (0..6).map(|i| {
            let n2 = n1 + 1 + i;
            let lam = 1.0 / (rh * (1.0 / (n1 * n1) as f64 - 1.0 / (n2 * n2) as f64));
            vec![text(format!("{}-{}", prefix, names[i])), text(format!("{n2} → {n1}")), text(format!("{} nm", crate::print::fmt_float(lam * 1e9, 6)))]
        }).collect();
        let limit = 1.0 / (rh / (n1 * n1) as f64) * 1e9;
        return Some(Answer::new("hydrogen series", vec![input(format!("{} series of hydrogen", &m[1])), primary(pod("Result", "Lines (vacuum wavelengths)", table(Some(vec!["line", "transition", "wavelength"]), rows))), pod("Limit", "Series limit", text(format!("{} nm", crate::print::fmt_float(limit, 6)))), pod("Formula", "Rydberg formula", text(format!("1/λ = R_H (1/{n1}² − 1/n²), R_H = {} m⁻¹", crate::print::fmt_float(rh, 9))))]));
    }
    // charged sphere
    if let Some(m) = re!(r"^(?:a\s+)?(?:charged|conducting) sphere,?\s+(?:radius\s*=?\s*|r\s*=\s*)([\d.]+\s*[a-z]*),?\s+(?:and\s+)?(?:charge\s*=?\s*|q\s*=\s*)([\d.]+\s*[a-zμ]*)$").captures(l) {
        let (r, ru) = quantity(&m[1])?;
        let (q, qu) = quantity(&m[2])?;
        if ru != "m" || qu != "C" || r <= 0.0 {
            return None;
        }
        let rows = vec![("surface charge density", sci(q / (4.0 * PI * r * r), "C/m²")), ("field at the surface", sci(KE * q / (r * r), "V/m")), ("potential at the surface", sci(KE * q / r, "V")), ("capacitance", sci(4.0 * PI * EPS0 * r, "F")), ("stored energy", sci(KE * q * q / (2.0 * r), "J"))];
        return Some(Answer::new("charged sphere", vec![input(format!("conducting sphere, radius {}, charge {}", sci(r, "m"), sci(q, "C"))), primary(pod("Result", "Properties", kv(rows))), pod("Formula", "Formulas", text("σ = Q/(4πR²), E = kQ/R², V = kQ/R, C = 4πε₀R, U = kQ²/(2R)"))]));
    }
    // asteroid impact
    if let Some(m) = re!(r"^([\d.]+\s*[a-z]+)\s+(?:diameter\s+)?(?:asteroid|meteor|meteorite|comet)\s+(?:impact\s+)?(?:at\s+)?([\d.]+\s*[a-z/]+)$").captures(l) {
        let (d, du) = quantity(&m[1])?;
        let (v, vu) = quantity(&m[2])?;
        if du != "m" || vu != "m/s" {
            return None;
        }
        let rho = 3000.0;
        let mass = rho * PI / 6.0 * d.powi(3);
        let e = 0.5 * mass * v * v;
        let rows = vec![("assumed density", "3000 kg/m³ (stony)".to_string()), ("mass", sci(mass, "kg")), ("kinetic energy", sci(e, "J")), ("TNT equivalent", format!("{} megatons", sci(e / 4.184e15, ""))), ("compared with Hiroshima (15 kt)", format!("{}×", sci(e / 6.276e13, "")))];
        return Some(Answer::new("impact", vec![input(format!("impact of a {} m object at {} km/s", fmt(d), fmt(v / 1e3))), primary(pod("Result", "Impact energy", text(format!("{} ≈ {} Mt TNT", sci(e, "J"), sci(e / 4.184e15, ""))))), pod("Details", "Details", kv(rows))]));
    }
    // nuclear explosion yield
    if let Some(m) = re!(r"^([\d.]+)\s*(kiloton|megaton|kt|mt)s?\s+(?:nuclear\s+)?(?:explosion|bomb|blast|detonation)$").captures(l) {
        let y: f64 = m[1].parse().ok()?;
        let kt = if m[2].starts_with('m') { y * 1000.0 } else { y };
        let e = kt * 4.184e12;
        let rows = vec![("energy", sci(e, "J")), ("mass converted (E = mc²)", sci(e / (C * C), "kg")), ("Hiroshima equivalents (15 kt)", fmt((kt / 15.0 * 10.0).round() / 10.0))];
        return Some(Answer::new("explosion", vec![input(format!("{} kt TNT-equivalent explosion", fmt(kt))), primary(pod("Result", "Energy", text(sci(e, "J")))), pod("Details", "Details", kv(rows))]));
    }
    // random walk with a number of steps
    if let Some(m) = re!(r"^(1|2|3)d random walk,?\s+(\d+)\s+steps$").captures(l) {
        let n: f64 = m[2].parse().ok()?;
        let ret = match &m[1] { "3" => "≈ 0.3405 (eventually)", _ => "1 (eventually)" };
        let rows = vec![("rms distance", format!("√{} ≈ {}", fmt(n), fmt((n.sqrt() * 1e4).round() / 1e4))), ("probability of returning to the start", ret.to_string())];
        return Some(Answer::new("random walk", vec![input(format!("{}-D lattice random walk, {} steps", &m[1], fmt(n))), primary(pod("Result", "Expected behaviour", kv(rows)))]));
    }
    None
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let one = |l: &str| laws(l).or_else(|| mechanics(l)).or_else(|| coordinate_systems(l)).or_else(|| systems(l)).or_else(|| black_holes(l)).or_else(|| calcs(l));
    if let Some(a) = one(&l) {
        return Some(a);
    }
    // "A vs B", "A, B": every part must be a law, system or black hole
    let parts: Vec<&str> = re!(r"\s+(?:vs\.?|versus|and)\s+|,\s*").split(&l).map(str::trim).filter(|x| !x.is_empty()).collect();
    if parts.len() < 2 || parts.len() > 5 {
        return None;
    }
    let answers: Vec<Answer> = parts.iter().map(|p| laws(p).or_else(|| systems(p)).or_else(|| black_holes(p))).collect::<Option<_>>()?;
    let mut pods = vec![input(parts.join(" | "))];
    for (part, a) in parts.iter().zip(answers) {
        for mut p in a.pods.into_iter().filter(|p| p.id != "Input") {
            p.id = format!("{}{}", p.id, part);
            p.title = format!("{} — {}", part, p.title);
            p.primary = false;
            pods.push(p);
        }
    }
    if let Some(p) = pods.get_mut(1) {
        p.primary = true;
    }
    Some(Answer::new("comparison", pods))
}

#[cfg(test)]
mod tests {
    fn ans(s: &str) -> String {
        serde_json::to_string(&crate::engine::query(s).pods).unwrap_or_default()
    }

    #[test]
    fn physics_answers() {
        assert!(ans("Lagrangian (q')^2 - q^2").contains("q'' = -q"));
        assert!(ans("Hamiltonian p^2/2 - cos(q)").contains("-sin(q)"));
        assert!(ans("Balmer series spectrum").contains("656.4"));
        assert!(ans("radius of Sagittarius A*").contains("10^10"));
        assert!(ans("third law of thermodynamics").contains("Nernst"));
    }
}
