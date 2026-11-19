//! Standard physical systems and their governing formulas: charge
//! distributions, retarded potentials, potential flows, beams and quantum
//! particles.
//!
//! "charged disk", "electric charge density of a charged ellipsoid",
//! "Lienard-Wiechert potential", "flow around a cylinder", "vortex potential".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

struct Sys {
    names: &'static [&'static str],
    title: &'static str,
    field: &'static str,
    desc: &'static str,
    rows: &'static [(&'static str, &'static str)],
}

const SYSTEMS: &[Sys] = &[
    Sys {
        names: &["charged disk", "charged disc", "uniformly charged disk", "uniformly charged disc", "disk of charge", "charged disk electric field", "electric field of a charged disk"],
        title: "uniformly charged disk",
        field: "electrostatics",
        desc: "A thin disk of radius R carrying uniform surface charge density σ; fields on its axis at height z.",
        rows: &[
            ("total charge", "Q = σ π R²"),
            ("potential on the axis", "V(z) = σ/(2ε₀) (√(z² + R²) − |z|)"),
            ("field on the axis", "E_z(z) = σ/(2ε₀) (1 − |z|/√(z² + R²)) sgn z"),
            ("near the disk (|z| ≪ R)", "E → σ/(2ε₀), the field of an infinite sheet"),
            ("far away (|z| ≫ R)", "E ≈ Q/(4πε₀ z²), a point charge"),
        ],
    },
    Sys {
        names: &["charged sphere", "uniformly charged sphere", "charged ball", "uniformly charged ball", "sphere of charge", "charged solid sphere"],
        title: "uniformly charged ball",
        field: "electrostatics",
        desc: "A ball of radius R with total charge Q spread uniformly through its volume (ρ = 3Q/(4πR³)).",
        rows: &[
            ("field inside (r < R)", "E(r) = Q r/(4πε₀ R³)"),
            ("field outside (r ≥ R)", "E(r) = Q/(4πε₀ r²)"),
            ("potential inside", "V(r) = Q (3R² − r²)/(8πε₀ R³)"),
            ("potential outside", "V(r) = Q/(4πε₀ r)"),
            ("electrostatic self-energy", "U = 3Q²/(20πε₀ R)"),
            ("conducting sphere instead", "charge on the surface, E = 0 inside, V = Q/(4πε₀ R) inside, C = 4πε₀ R"),
        ],
    },
    Sys {
        names: &["charged ellipsoid", "conducting ellipsoid", "charged conducting ellipsoid", "electric charge density of a charged ellipsoid", "charge density of a charged ellipsoid", "surface charge density of a charged ellipsoid", "charged ellipsoid charge density"],
        title: "charged conducting ellipsoid",
        field: "electrostatics",
        desc: "A conductor with surface x²/a² + y²/b² + z²/c² = 1 holding charge Q.",
        rows: &[
            ("surface charge density", "σ(x, y, z) = Q/(4π a b c) · (x²/a⁴ + y²/b⁴ + z²/c⁴)^(−1/2)"),
            ("density at the axis ends", "σ = Q/(4π b c) at x = ±a (largest at the ends of the longest axis)"),
            ("capacitance", "C = 8πε₀ / ∫₀^∞ ds/√((a² + s)(b² + s)(c² + s))"),
            ("sphere (a = b = c = R)", "σ = Q/(4πR²), C = 4πε₀ R"),
            ("field inside", "E = 0 (the surface is an equipotential)"),
        ],
    },
    Sys {
        names: &["infinite line charge", "line charge", "infinite line of charge", "charged wire", "long charged wire"],
        title: "infinite line charge",
        field: "electrostatics",
        desc: "A straight line with uniform charge per length λ; fields at distance r.",
        rows: &[
            ("electric field", "E = λ/(2πε₀ r), pointing radially away from the line"),
            ("potential", "V(r) = −(λ/(2πε₀)) ln(r/r₀)  (r₀ a reference distance)"),
            ("via Gauss's law", "E · 2πrL = λL/ε₀"),
        ],
    },
    Sys {
        names: &["electric dipole", "dipole field", "electric dipole field", "field of an electric dipole", "electric dipole potential"],
        title: "electric dipole",
        field: "electrostatics",
        desc: "Charges ±q separated by d, with dipole moment p = q d; fields far from the dipole (r ≫ d).",
        rows: &[
            ("potential", "V = p cos θ/(4πε₀ r²) = p · r̂/(4πε₀ r²)"),
            ("field", "E = (3(p · r̂) r̂ − p)/(4πε₀ r³)"),
            ("field on the axis", "E = 2p/(4πε₀ r³)"),
            ("field on the perpendicular bisector", "E = −p/(4πε₀ r³)"),
            ("torque in a field E", "τ = p × E"),
            ("energy in a field E", "U = −p · E"),
        ],
    },
    Sys {
        names: &["magnetized ball", "magnetized sphere", "uniformly magnetized sphere", "uniformly magnetized ball", "magnetised sphere", "magnetised ball"],
        title: "uniformly magnetized ball",
        field: "magnetostatics",
        desc: "A ball of radius R with uniform magnetization M.",
        rows: &[
            ("field inside", "B = (2/3) μ₀ M,  H = −M/3 (demagnetizing factor 1/3)"),
            ("magnetic moment", "m = (4/3) π R³ M"),
            ("field outside", "B = μ₀ (3(m · r̂) r̂ − m)/(4π r³), exactly a dipole field"),
            ("bound currents", "surface current K = M × n̂ = M sin θ φ̂, no volume current"),
        ],
    },
    Sys {
        names: &["lienard-wiechert potential", "lienard wiechert potential", "liénard-wiechert potential", "lienard-wiechert potentials", "liénard–wiechert potential", "lienard-wiechert", "lienard wiechert potentials", "lienard-wiechert fields"],
        title: "Liénard–Wiechert potentials",
        field: "electrodynamics",
        desc: "The retarded potentials of a point charge q moving on a trajectory r_s(t).",
        rows: &[
            ("retarded time", "t_r = t − |r − r_s(t_r)|/c"),
            ("geometry", "R = |r − r_s(t_r)|,  n = (r − r_s(t_r))/R,  β = v_s(t_r)/c"),
            ("scalar potential", "φ(r, t) = q/(4πε₀) · [1/((1 − n · β) R)] at t_r"),
            ("vector potential", "A(r, t) = μ₀ q c/(4π) · [β/((1 − n · β) R)] at t_r = β φ/c"),
            ("electric field", "E = q/(4πε₀) [ (n − β)(1 − β²)/((1 − n · β)³ R²) + n × ((n − β) × β̇)/(c (1 − n · β)³ R) ] at t_r"),
            ("magnetic field", "B = n × E / c"),
            ("radiation", "the 1/R term carries energy away; for β ≪ 1 the power is the Larmor formula P = q² a²/(6πε₀ c³)"),
        ],
    },
    Sys {
        names: &["time-dependent charge distribution", "time dependent charge distribution", "time-dependent charge distribution electric potential", "time-dependent charge distribution: electric potential", "retarded potential", "retarded potentials", "jefimenko's equations", "jefimenko equations"],
        title: "time-dependent charge and current distribution (retarded potentials)",
        field: "electrodynamics",
        desc: "Potentials of charge density ρ(r′, t) and current density J(r′, t) in the Lorenz gauge.",
        rows: &[
            ("retarded time", "t_r = t − |r − r′|/c"),
            ("electric potential", "φ(r, t) = (1/(4πε₀)) ∫ ρ(r′, t_r)/|r − r′| d³r′"),
            ("vector potential", "A(r, t) = (μ₀/(4π)) ∫ J(r′, t_r)/|r − r′| d³r′"),
            ("fields", "E = −∇φ − ∂A/∂t,  B = ∇ × A"),
            ("Lorenz gauge", "∇ · A + (1/c²) ∂φ/∂t = 0"),
            ("wave equations", "□φ = ρ/ε₀,  □A = μ₀ J"),
        ],
    },
    Sys {
        names: &["flow around a cylinder", "flow past a cylinder", "potential flow around a cylinder", "flow over a cylinder", "cylinder flow", "potential flow past a cylinder", "flow around cylinder"],
        title: "potential flow around a cylinder",
        field: "fluid mechanics",
        desc: "Ideal (inviscid, irrotational, incompressible) flow of speed U past a circular cylinder of radius a.",
        rows: &[
            ("velocity potential", "φ = U (r + a²/r) cos θ"),
            ("stream function", "ψ = U (r − a²/r) sin θ"),
            ("complex potential", "w(z) = U (z + a²/z)"),
            ("velocity", "u_r = U (1 − a²/r²) cos θ,  u_θ = −U (1 + a²/r²) sin θ"),
            ("speed on the surface", "|u| = 2U |sin θ|, stagnation points at θ = 0 and π"),
            ("pressure coefficient on the surface", "C_p = 1 − 4 sin² θ"),
            ("force", "no drag (d'Alembert's paradox); with circulation Γ the lift per length is ρ U Γ (Kutta–Joukowski)"),
        ],
    },
    Sys {
        names: &["flow around a sphere", "flow past a sphere", "potential flow around a sphere", "stokes flow around a sphere", "flow over a sphere"],
        title: "flow around a sphere",
        field: "fluid mechanics",
        desc: "Uniform flow of speed U past a sphere of radius a: the ideal-fluid solution and the creeping-flow (Stokes) limit.",
        rows: &[
            ("velocity potential (ideal flow)", "φ = U (r + a³/(2r²)) cos θ"),
            ("speed on the surface (ideal flow)", "|u| = (3/2) U sin θ"),
            ("pressure coefficient (ideal flow)", "C_p = 1 − (9/4) sin² θ"),
            ("Stokes stream function (Re ≪ 1)", "ψ = (U/2) r² sin² θ (1 − 3a/(2r) + a³/(2r³))"),
            ("Stokes drag", "F = 6π μ a U, drag coefficient C_D = 24/Re"),
        ],
    },
    Sys {
        names: &["vortex potential", "point vortex", "line vortex", "irrotational vortex", "free vortex", "potential vortex"],
        title: "point (irrotational) vortex",
        field: "fluid mechanics",
        desc: "Two-dimensional flow circulating about the origin with circulation Γ.",
        rows: &[
            ("velocity potential", "φ = Γ θ/(2π)"),
            ("stream function", "ψ = −(Γ/(2π)) ln r"),
            ("complex potential", "w(z) = −(i Γ/(2π)) ln z"),
            ("velocity", "u_θ = Γ/(2π r),  u_r = 0"),
            ("vorticity", "zero everywhere except a delta function at r = 0"),
            ("pressure", "p = p_∞ − ρ Γ²/(8π² r²)"),
        ],
    },
    Sys {
        names: &["beam deformation", "beam deflection", "euler-bernoulli beam", "euler bernoulli beam", "beam bending", "bending of a beam", "deflection of a beam"],
        title: "Euler–Bernoulli beam",
        field: "solid mechanics",
        desc: "Small deflection w(x) of a slender beam with Young's modulus E and second moment of area I under a distributed load q(x).",
        rows: &[
            ("beam equation", "E I d⁴w/dx⁴ = q(x)"),
            ("bending moment and shear", "M = −E I w″,  V = −E I w‴"),
            ("bending stress", "σ = M y / I"),
            ("cantilever, end load P", "w_max = P L³/(3 E I)"),
            ("cantilever, uniform load q", "w_max = q L⁴/(8 E I)"),
            ("simply supported, centre load P", "w_max = P L³/(48 E I)"),
            ("simply supported, uniform load q", "w_max = 5 q L⁴/(384 E I)"),
        ],
    },
    Sys {
        names: &["spherically symmetric potential", "central potential", "particle in a central potential", "particle in a spherically symmetric potential", "quantum particle in a spherically symmetric potential"],
        title: "particle in a spherically symmetric potential",
        field: "quantum mechanics",
        desc: "A particle of mass m in a potential V(r) that depends only on the distance from the centre.",
        rows: &[
            ("separation", "ψ(r, θ, φ) = R(r) Y_l^m(θ, φ)"),
            ("radial equation (u = r R)", "−(ħ²/2m) u″ + [V(r) + ħ² l(l + 1)/(2m r²)] u = E u"),
            ("conserved", "energy, L² = ħ² l(l + 1) and L_z = ħ m"),
            ("degeneracy", "each level is at least (2l + 1)-fold degenerate"),
            ("classical counterpart", "V_eff(r) = V(r) + L²/(2m r²); motion stays in a plane"),
            ("examples", "hydrogen atom (V = −e²/(4πε₀ r)), 3-D harmonic oscillator, spherical well"),
        ],
    },
    Sys {
        names: &["quantum particle in a time-independent 1d potential", "quantum particle in a time independent 1d potential", "particle in a 1d potential", "quantum particle in a 1d potential", "particle in a one-dimensional potential", "time-independent schrodinger equation", "time independent schrodinger equation"],
        title: "quantum particle in a time-independent 1-D potential",
        field: "quantum mechanics",
        desc: "A particle of mass m moving on a line in a potential V(x).",
        rows: &[
            ("Schrödinger equation", "−(ħ²/2m) ψ″(x) + V(x) ψ(x) = E ψ(x)"),
            ("stationary states", "Ψ(x, t) = ψ(x) e^(−iEt/ħ)"),
            ("probability current", "j = (ħ/m) Im(ψ* ψ′)"),
            ("normalization", "∫ |ψ|² dx = 1 for bound states (E below V at ±∞)"),
            ("matching", "ψ and ψ′ continuous where V is finite"),
            ("examples", "infinite square well E_n = n²π²ħ²/(2mL²); harmonic oscillator E_n = ħω(n + 1/2)"),
        ],
    },
    Sys {
        names: &["quantum particle in an external electromagnetic field", "quantum particle in an electromagnetic field", "charged particle in an electromagnetic field", "particle in an electromagnetic field", "pauli equation"],
        title: "quantum particle in an external electromagnetic field",
        field: "quantum mechanics",
        desc: "A particle of charge q and mass m in potentials φ(r, t) and A(r, t).",
        rows: &[
            ("Hamiltonian (minimal coupling)", "H = (p − qA)²/(2m) + qφ"),
            ("with spin ½ (Pauli)", "H = (σ · (p − qA))²/(2m) + qφ = (p − qA)²/(2m) + qφ − (qħ/2m) σ · B"),
            ("gauge transformation", "A → A + ∇χ,  φ → φ − ∂χ/∂t,  ψ → e^(iqχ/ħ) ψ"),
            ("probability current", "j = (1/m) Re(ψ* (−iħ∇ − qA) ψ)"),
            ("uniform field B", "Landau levels E_n = ħ ω_c (n + 1/2), ω_c = |q| B/m"),
        ],
    },
];

fn norm(s: &str) -> String {
    let s = s.replace(['–', '—', '‐', '‑'], "-").replace('é', "e");
    let s = re!(r"^(?:(?:show|describe|give)(?: me)? )?(?:the |an? )?").replace(&s, "").to_string();
    s.trim().trim_end_matches(['?', '.']).trim().to_string()
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = norm(&q.lower);
    let s = SYSTEMS.iter().find(|s| s.names.iter().any(|n| norm(n) == l))?;
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{} | {}", s.title, s.field))),
        pod("Description", "Description", text(s.desc)),
        primary(pod("Formulas", "Formulas", table(None, s.rows.iter().map(|(k, v)| vec![text(*k), text(*v)]).collect()))),
    ];
    if s.title == "uniformly charged disk" {
        // field on the axis in units of σ/(2ε₀), for R = 1
        let xs: Vec<f64> = (0..=200).map(|i| -4.0 + 8.0 * i as f64 / 200.0).collect();
        let ys: Vec<f64> = xs.iter().map(|z| z.signum() * (1.0 - z.abs() / (z * z + 1.0).sqrt())).collect();
        let vs: Vec<f64> = xs.iter().map(|z| (z * z + 1.0).sqrt() - z.abs()).collect();
        pods.push(pod("Plot", "On-axis field and potential (R = 1, units of σ/(2ε₀))", plot(crate::plot::lines(vec![("E_z", xs.clone(), ys), ("V", xs, vs)], "z/R", ""))));
    }
    Some(Answer::new(s.title, pods))
}
