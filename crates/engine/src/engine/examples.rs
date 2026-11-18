//! A tool named without input ("linear fit", "Z-transform calculator",
//! "row reduction calculator") is answered, as WolframAlpha does, by
//! working a labelled example.

use super::*;
use std::cell::Cell;

thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// tool names (lower case, without "calculator") | example query
const EXAMPLES: &[(&[&str], &str)] = &[
    (&["row reduction", "row reduce", "rref", "reduced row echelon form", "gaussian elimination", "row echelon form"], "row reduce {{1,2,3},{4,5,6}}"),
    (&["interpolating polynomial", "polynomial interpolation", "lagrange interpolation"], "interpolating polynomial {(1,1),(2,4),(3,9)}"),
    (&["fourier transform"], "Fourier transform of exp(-x^2)"),
    (&["inverse fourier transform"], "inverse Fourier transform of exp(-w^2)"),
    (&["z-transform", "z transform"], "Z transform of 2^n"),
    (&["inverse z-transform", "inverse z transform"], "inverse Z transform of z/(z-2)"),
    (&["laplace transform"], "Laplace transform of sin(t)"),
    (&["inverse laplace transform"], "inverse Laplace transform of 1/(s^2+1)"),
    (&["linear fit", "linear regression", "least squares fit", "line of best fit", "best fit line"], "linear fit {1,2},{2,3.9},{3,6.1},{4,8.2}"),
    (&["quadratic fit", "quadratic regression"], "quadratic fit {1,1},{2,4.1},{3,9.2},{4,15.8}"),
    (&["cubic fit", "cubic regression"], "cubic fit {0,1},{1,2},{2,9},{3,28},{4,65}"),
    (&["normalize vector", "vector normalization", "unit vector"], "normalize (1,2,2)"),
    (&["matrix inverse", "inverse matrix", "matrix inversion"], "inverse {{2,1},{1,1}}"),
    (&["determinant"], "determinant {{1,2},{3,4}}"),
    (&["eigenvalues", "eigenvalue"], "eigenvalues {{2,1},{1,2}}"),
    (&["derivative", "differentiation"], "derivative of x^2 sin(x)"),
    (&["integral", "integration", "antiderivative"], "integrate x^2 sin(x)"),
    (&["definite integral"], "integrate x^2 from 0 to 1"),
    (&["limit"], "limit sin(x)/x as x->0"),
    (&["taylor series", "series expansion"], "Taylor series of e^x at x=0"),
    (&["partial fractions", "partial fraction decomposition"], "partial fractions (x+3)/(x^2-1)"),
    (&["quadratic formula", "quadratic equation solver"], "solve x^2-5x+6=0"),
];

/// "log plot e^x - x": a logarithmic y axis, drawn as log₁₀ of the function.
fn log_plot(q: &Query) -> Option<Answer> {
    let m = regex::Regex::new(r"(?i)^(?:log|log-linear|semilog|semi-log|logarithmic)\s+plot\s+(?:of\s+)?(.+?)(\s*,\s*[a-z]\s*=.+|\s+from\s+.+)?$").ok()?.captures(q.clean.trim())?;
    let f = m[1].trim();
    let rewritten = format!("plot log10({f}){}", m.get(2).map_or("", |x| x.as_str()));
    DEPTH.with(|d| d.set(1));
    let r = run_handlers(&rewritten, q.opts);
    DEPTH.with(|d| d.set(0));
    if !r.success {
        return None;
    }
    let mut pods = vec![pod("Scale", "Scale", text(format!("logarithmic y axis: the curve shown is log₁₀({f}), so each unit up is a factor of 10")))];
    pods.extend(r.pods);
    Some(Answer::new(format!("log plot of {f}"), pods))
}

/// reactant name → formula
const SPECIES: &[(&str, &str)] = &[
    ("hydrogen", "H2"), ("hydrogen gas", "H2"), ("oxygen", "O2"), ("oxygen gas", "O2"), ("nitrogen", "N2"), ("chlorine", "Cl2"), ("chlorine gas", "Cl2"),
    ("sodium", "Na"), ("potassium", "K"), ("lithium", "Li"), ("magnesium", "Mg"), ("calcium", "Ca"), ("iron", "Fe"), ("zinc", "Zn"), ("copper", "Cu"), ("aluminum", "Al"), ("aluminium", "Al"),
    ("carbon", "C"), ("sulfur", "S"), ("sulphur", "S"), ("water", "H2O"), ("methane", "CH4"), ("propane", "C3H8"), ("butane", "C4H10"), ("octane", "C8H18"), ("ethanol", "C2H5OH"), ("glucose", "C6H12O6"),
    ("carbon dioxide", "CO2"), ("sulfur dioxide", "SO2"), ("hydrochloric acid", "HCl"), ("sodium hydroxide", "NaOH"), ("calcium carbonate", "CaCO3"),
    ("sulfuric acid", "H2SO4"), ("baking soda", "NaHCO3"), ("sodium bicarbonate", "NaHCO3"), ("vinegar", "CH3COOH"), ("acetic acid", "CH3COOH"), ("ammonia", "NH3"), ("calcium oxide", "CaO"),
];

/// sorted reactant formulas → balanced-equation input
const REACTIONS: &[(&[&str], &str, &str)] = &[
    (&["H2", "O2"], "H2 + O2 -> H2O", "synthesis of water (combustion of hydrogen)"),
    (&["Cl2", "Na"], "Na + Cl2 -> NaCl", "synthesis of sodium chloride"),
    (&["CH4", "O2"], "CH4 + O2 -> CO2 + H2O", "complete combustion of methane"),
    (&["C3H8", "O2"], "C3H8 + O2 -> CO2 + H2O", "complete combustion of propane"),
    (&["C4H10", "O2"], "C4H10 + O2 -> CO2 + H2O", "complete combustion of butane"),
    (&["C8H18", "O2"], "C8H18 + O2 -> CO2 + H2O", "complete combustion of octane"),
    (&["C2H5OH", "O2"], "C2H5OH + O2 -> CO2 + H2O", "complete combustion of ethanol"),
    (&["C6H12O6", "O2"], "C6H12O6 + O2 -> CO2 + H2O", "cellular respiration (oxidation of glucose)"),
    (&["C", "O2"], "C + O2 -> CO2", "complete combustion of carbon"),
    (&["O2", "S"], "S + O2 -> SO2", "combustion of sulfur"),
    (&["H2", "N2"], "N2 + H2 -> NH3", "ammonia synthesis (Haber–Bosch process)"),
    (&["Fe", "O2"], "Fe + O2 -> Fe2O3", "oxidation (rusting) of iron"),
    (&["Mg", "O2"], "Mg + O2 -> MgO", "combustion of magnesium"),
    (&["Al", "O2"], "Al + O2 -> Al2O3", "oxidation of aluminum"),
    (&["Cu", "O2"], "Cu + O2 -> CuO", "oxidation of copper"),
    (&["Ca", "O2"], "Ca + O2 -> CaO", "oxidation of calcium"),
    (&["Li", "O2"], "Li + O2 -> Li2O", "oxidation of lithium"),
    (&["O2", "SO2"], "SO2 + O2 -> SO3", "oxidation of sulfur dioxide (contact process)"),
    (&["Cl2", "H2"], "H2 + Cl2 -> HCl", "synthesis of hydrogen chloride"),
    (&["Cl2", "Fe"], "Fe + Cl2 -> FeCl3", "synthesis of iron(III) chloride"),
    (&["HCl", "NaOH"], "HCl + NaOH -> NaCl + H2O", "neutralization"),
    (&["H2SO4", "NaOH"], "H2SO4 + NaOH -> Na2SO4 + H2O", "neutralization"),
    (&["HCl", "NH3"], "NH3 + HCl -> NH4Cl", "acid–base reaction forming ammonium chloride"),
    (&["HCl", "Zn"], "Zn + HCl -> ZnCl2 + H2", "single displacement of hydrogen by zinc"),
    (&["HCl", "Mg"], "Mg + HCl -> MgCl2 + H2", "single displacement of hydrogen by magnesium"),
    (&["H2O", "Na"], "Na + H2O -> NaOH + H2", "reaction of sodium with water"),
    (&["H2O", "K"], "K + H2O -> KOH + H2", "reaction of potassium with water"),
    (&["H2O", "Li"], "Li + H2O -> LiOH + H2", "reaction of lithium with water"),
    (&["Ca", "H2O"], "Ca + H2O -> Ca(OH)2 + H2", "reaction of calcium with water"),
    (&["CaO", "H2O"], "CaO + H2O -> Ca(OH)2", "slaking of lime"),
    (&["CO2", "H2O"], "CO2 + H2O -> H2CO3", "formation of carbonic acid"),
    (&["CaCO3", "HCl"], "CaCO3 + HCl -> CaCl2 + H2O + CO2", "acid attack on calcium carbonate"),
    (&["CH3COOH", "NaHCO3"], "NaHCO3 + CH3COOH -> NaCH3COO + H2O + CO2", "baking soda and vinegar"),
];

fn species(s: &str) -> Option<String> {
    let s = s.trim().trim_start_matches("the ").trim();
    if let Some((_, f)) = SPECIES.iter().find(|(n, _)| *n == s.to_lowercase()) {
        return Some(f.to_string());
    }
    // a formula as typed: "H2", "NaOH"
    regex::Regex::new(r"^(?:[A-Z][a-z]?\d*|\(|\)\d*)+$").ok()?.is_match(s).then(|| s.to_string())
}

/// "hydrogen + oxygen ->", "sodium reacts with water": the products of a
/// common reaction, balanced by the equation handler
fn complete_reaction(q: &Query) -> Option<Answer> {
    let t = q.clean.trim().trim_end_matches(['?', '.']).trim();
    let m = regex::Regex::new(r"(?i)^(?:(?:what happens when |what is the product of |products? of |reaction of |reaction between )?(.+?)\s*(?:\+|\breacts? with\b|\band\b)\s*(.+?))\s*(?:->|→|=>|=|\byields?\b|\bproduces?\b|\breact(?:s|ion)?\b)?\s*$").ok()?.captures(t)?;
    let mut fs = vec![species(&m[1])?, species(&m[2])?];
    fs.sort();
    let (_, eq, what) = REACTIONS.iter().find(|r| r.0.len() == 2 && r.0[0] == fs[0] && r.0[1] == fs[1])?;
    // a bare "x + y" needs an arrow or reaction words unless both are named substances
    let worded = t.contains("->") || t.contains('→') || t.to_lowercase().contains("react") || t.contains('=') || t.to_lowercase().contains("product");
    let named = SPECIES.iter().any(|(n, _)| *n == m[1].trim().to_lowercase()) && SPECIES.iter().any(|(n, _)| *n == m[2].trim().to_lowercase());
    if !worded && !(named && t.contains('+')) {
        return None;
    }
    DEPTH.with(|d| d.set(1));
    let r = run_handlers(eq, q.opts);
    DEPTH.with(|d| d.set(0));
    if !r.success {
        return None;
    }
    let mut pods = vec![pod("Reaction", "Reaction type", text(*what))];
    pods.extend(r.pods);
    Some(Answer::new(format!("{what}"), pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    if DEPTH.with(|d| d.get()) > 0 {
        return None;
    }
    if let Some(a) = log_plot(q).or_else(|| complete_reaction(q)) {
        return Some(a);
    }
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = l.trim_end_matches(" calculator").trim_end_matches(" solver").trim_end_matches(" tool").trim();
    let (_, example) = EXAMPLES.iter().find(|(names, _)| names.contains(&l))?;
    DEPTH.with(|d| d.set(1));
    let r = run_handlers(example, q.opts);
    DEPTH.with(|d| d.set(0));
    if !r.success {
        return None;
    }
    let mut pods = vec![pod("Example", "Example", text(format!("no input was given, so here is an example: {example}")))];
    pods.extend(r.pods);
    Some(Answer::new(format!("{l} (example)"), pods))
}
