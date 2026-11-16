//! Amounts of substance and solutions: "500 mg of silver nitrate",
//! "10^30 atoms of helium to liters", "molality of 0.385 g cholesterol in
//! 40.0 g chloroform", "dilute 30 ml of 3 molar NaOH with 30 ml of water",
//! "prepare 1.25 L of 3 M NaOH from 10 M", "normality of 12 molar
//! phosphoric acid", reactions with given amounts ("22g NaOH + 16g MgCl2 ->
//! Mg(OH)2 + NaCl") and polyatomic ions.

use super::chem::{by_symbol, molar_mass, parse_formula};
use super::chemcalc::{coefficients, find_substance, pretty, substance};
use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

const AVOGADRO: f64 = 6.022_140_76e23;
/// Molar volume of an ideal gas at 0 °C and 1 atm, L/mol.
const STP_L: f64 = 22.413_969_54;

fn n(v: f64) -> String {
    fmt_float(v, 6)
}

/// A number as written: "500", "1.5e3", "10^30", "2.5×10^-3".
const NUM: &str = r"(\d[\d,]*(?:\.\d+)?(?:\s*(?:x|×|\*)\s*10\^-?\d+|e-?\d+)?|\d+\^-?\d+|\.\d+)";
const UNIT: &str = r"(mg|milligrams?|µg|ug|micrograms?|kg|kilograms?|g|grams?|lbs?|pounds?|oz|ounces?|mmol|mmoles?|millimoles?|µmol|umol|micromoles?|kmol|mol|moles?|µl|ul|microliters?|microlitres?|ml|milliliters?|millilitres?|l|liters?|litres?|gallons?|gal|cups?|atoms|molecules|particles|formula units)";

fn number(s: &str) -> Option<f64> {
    let t = s.replace(',', "").replace(['×', 'x'], "*").replace(' ', "");
    if let Some((a, b)) = t.split_once('^') {
        let (base, mant) = match a.split_once('*') {
            Some((m, b)) => (b.to_string(), m.parse::<f64>().ok()?),
            None => (a.to_string(), 1.0),
        };
        return Some(mant * base.parse::<f64>().ok()?.powf(b.parse().ok()?));
    }
    t.parse().ok()
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Mass,
    Moles,
    Volume,
    Count,
}

/// (kind, factor to g / mol / L / particles)
fn unit(u: &str) -> Option<(Kind, f64)> {
    let u = u.to_lowercase();
    Some(match u.as_str() {
        "mg" | "milligram" | "milligrams" => (Kind::Mass, 1e-3),
        "µg" | "ug" | "microgram" | "micrograms" => (Kind::Mass, 1e-6),
        "kg" | "kilogram" | "kilograms" => (Kind::Mass, 1e3),
        "g" | "gram" | "grams" => (Kind::Mass, 1.0),
        "lb" | "lbs" | "pound" | "pounds" => (Kind::Mass, 453.592_37),
        "oz" | "ounce" | "ounces" => (Kind::Mass, 28.349_523_125),
        "mmol" | "mmole" | "mmoles" | "millimole" | "millimoles" => (Kind::Moles, 1e-3),
        "µmol" | "umol" | "micromole" | "micromoles" => (Kind::Moles, 1e-6),
        "kmol" => (Kind::Moles, 1e3),
        "mol" | "mole" | "moles" => (Kind::Moles, 1.0),
        "µl" | "ul" | "microliter" | "microliters" | "microlitre" | "microlitres" => (Kind::Volume, 1e-6),
        "ml" | "milliliter" | "milliliters" | "millilitre" | "millilitres" => (Kind::Volume, 1e-3),
        "l" | "liter" | "liters" | "litre" | "litres" => (Kind::Volume, 1.0),
        "gallon" | "gallons" | "gal" => (Kind::Volume, 3.785_411_784),
        "cup" | "cups" => (Kind::Volume, 0.236_588_236_5),
        "atoms" | "molecules" | "particles" | "formula units" => (Kind::Count, 1.0),
        _ => return None,
    })
}

/// Density in g/mL: the substance table, water for aqueous solvents.
fn density(name: &str) -> Option<f64> {
    let l = name.to_lowercase();
    if matches!(l.as_str(), "water" | "h2o") {
        return Some(1.0);
    }
    find_substance(&l).and_then(|s| s.density)
}

fn is_gas(name: &str, counts: &BTreeMap<String, u32>) -> bool {
    if let Some(s) = find_substance(&name.to_lowercase()) {
        if let Some(bp) = s.bp {
            return bp < 0.0;
        }
    }
    // noble gases and the diatomic gases
    let f: Vec<(&String, &u32)> = counts.iter().collect();
    f.len() == 1 && matches!((f[0].0.as_str(), *f[0].1), ("He" | "Ne" | "Ar" | "Kr" | "Xe" | "Rn", 1) | ("H" | "N" | "O" | "F" | "Cl", 2))
}

struct Thing {
    label: String,
    name: String,
    counts: BTreeMap<String, u32>,
    mm: f64,
}

fn thing(s: &str) -> Option<Thing> {
    let s = s.trim().trim_end_matches(" solution").trim_end_matches(" gas").trim();
    let (label, _, counts) = substance(s).or_else(|| {
        // a formula that starts with a bracket: "(NH4)2SeO4"
        let c = parse_formula(s).filter(|_| s.starts_with('('))?;
        Some((pretty(s), s.to_string(), c))
    })?;
    let mm = molar_mass(&counts);
    Some(Thing { label, name: s.to_string(), counts, mm })
}

/// Moles in an amount of a substance.
fn moles_of(v: f64, kind: Kind, f: f64, t: &Thing) -> Option<f64> {
    Some(match kind {
        Kind::Mass => v * f / t.mm,
        Kind::Moles => v * f,
        Kind::Count => v / AVOGADRO,
        Kind::Volume => v * f * 1000.0 * density(&t.name)? / t.mm,
    })
}

pub fn handle(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches(['?', '.']).trim();
    let l = c.to_lowercase();
    ions(c, &l)
        .or_else(|| stoichiometry(c))
        .or_else(|| dilute(c, &l))
        .or_else(|| prepare(c, &l))
        .or_else(|| normality(c, &l))
        .or_else(|| concentration(c, &l))
        .or_else(|| solution_amount(c, &l))
        .or_else(|| quantity(c, &l))
        .or_else(|| bare_solution(q, c, &l))
}

/// The original-case text matching a span of the lower-case query.
fn orig<'a>(c: &'a str, m: &regex::Captures, k: usize) -> &'a str {
    let g = m.get(k).unwrap();
    &c[g.start()..g.end()]
}

// ---------------------------------------------------------------------------
// "500 mg of silver nitrate", "10^30 atoms of helium to liters"
// ---------------------------------------------------------------------------

fn quantity(c: &str, l: &str) -> Option<Answer> {
    let r = re!(r"^(?:convert\s+)?(\d[\d,]*(?:\.\d+)?(?:\s*(?:x|×|\*)\s*10\^-?\d+|e-?\d+)?|\d+\^-?\d+|\.\d+)\s*(mg|milligrams?|µg|ug|micrograms?|kg|kilograms?|g|grams?|lbs?|pounds?|oz|ounces?|mmol|mmoles?|millimoles?|µmol|umol|micromoles?|kmol|mol|moles?|µl|ul|microliters?|microlitres?|ml|milliliters?|millilitres?|l|liters?|litres?|gallons?|gal|cups?|atoms|molecules|particles|formula units)\s+(?:of\s+)?(.+?)(?:\s+(?:to|in|into|as)\s+(moles?|mol|mmol|grams?|g|kg|mg|liters?|litres?|l|ml|milliliters?|molecules|atoms|particles|mass|volume))?$");
    let m = r.captures(l)?;
    let v = number(&m[1])?;
    let (kind, f) = unit(&m[2])?;
    let t = thing(orig(c, &m, 3))?;
    let gas = is_gas(&t.name, &t.counts);
    let moles = if kind == Kind::Volume && gas && density(&t.name).is_none() { v * f / STP_L } else { moles_of(v, kind, f, &t)? };
    let want = m.get(4).map_or("", |x| x.as_str());
    let mass = moles * t.mm;
    let mut rows: Vec<(&str, &str, String)> = vec![
        ("Moles", "Amount of substance", format!("{} mol", n(moles))),
        ("Mass", "Mass", format!("{} g", n(mass))),
        ("Particles", if t.counts.values().sum::<u32>() == 1 { "Number of atoms" } else { "Number of molecules" }, n(moles * AVOGADRO)),
    ];
    if gas {
        rows.push(("Volume", "Volume as a gas at 0 °C and 1 atm", format!("{} L", n(moles * STP_L))));
    } else if let Some(d) = density(&t.name) {
        rows.push(("Volume", "Volume", format!("{} mL  (density {} g/mL)", n(mass / d), n(d))));
    }
    let lead = match want {
        w if w.starts_with("mol") && !w.starts_with("molec") || w == "mmol" => "Moles",
        "g" | "kg" | "mg" | "mass" | "gram" | "grams" => "Mass",
        w if w.starts_with('l') || w.starts_with("ml") || w.starts_with("milli") || w == "volume" => "Volume",
        "molecules" | "atoms" | "particles" => "Particles",
        _ => {
            if kind == Kind::Moles {
                "Mass"
            } else {
                "Moles"
            }
        }
    };
    if !rows.iter().any(|r| r.0 == lead) {
        return None;
    }
    let label = format!("{} {} of {}", &m[1], &m[2], t.label);
    let mut pods = vec![pod("Input", "Input interpretation", text(label.clone()))];
    for (id, title, val) in rows {
        let p = pod(id, title, text(val));
        if id == lead {
            pods.insert(1, primary(p));
        } else {
            pods.push(p);
        }
    }
    pods.push(pod("MolarMass", "Molar mass", text(format!("{} g/mol", n(t.mm)))));
    Some(Answer::new(label, pods))
}

// ---------------------------------------------------------------------------
// concentrations: molarity, molality, mass and mole fraction
// ---------------------------------------------------------------------------

fn concentration(c: &str, l: &str) -> Option<Answer> {
    let r = Regex::new(&format!(r"^(?:the )?(molarity|molality|mass fraction|mass percent|percent by mass|weight percent|mole fraction|concentration|mass concentration) of {NUM}\s*{UNIT}\s+(?:of\s+)?(?:(?:a\s+)?{NUM}\s*(m|molar|mm|millimolar)\s+)?(.+?) (?:dissolved )?(?:in|into|with) {NUM}\s*{UNIT}\s+(?:of\s+)?(.+)$")).ok()?;
    let m = r.captures(l)?;
    let what = m[1].to_string();
    let (v1, (k1, f1)) = (number(&m[2])?, unit(&m[3])?);
    let solute = thing(orig(c, &m, 6))?;
    let (v2, (k2, f2)) = (number(&m[7])?, unit(&m[8])?);
    let solvent_name = orig(c, &m, 9).trim().trim_start_matches("of ").to_string();
    let solvent = thing(&solvent_name);
    // moles of solute: from a stock solution when a molarity is given
    let n1 = match (m.get(4), k1) {
        (Some(cc), Kind::Volume) => {
            let conc = number(cc.as_str())? * if m[5].starts_with("mm") || m[5].starts_with("milli") { 1e-3 } else { 1.0 };
            v1 * f1 * conc
        }
        _ => moles_of(v1, k1, f1, &solute)?,
    };
    let mass1 = n1 * solute.mm;
    let d2 = density(&solvent_name);
    let (solvent_g, solvent_l) = match k2 {
        Kind::Mass => (v2 * f2, d2.map(|d| v2 * f2 / d / 1000.0)),
        Kind::Volume => (d2.map(|d| v2 * f2 * 1000.0 * d)?, Some(v2 * f2)),
        _ => return None,
    };
    // the stock volume adds to the total volume
    let total_l = solvent_l.map(|s| s + if k1 == Kind::Volume { v1 * f1 } else { 0.0 });
    let molarity = total_l.map(|vl| n1 / vl);
    let molality = n1 / (solvent_g / 1000.0);
    let mass_fraction = mass1 / (mass1 + solvent_g);
    let mole_fraction = solvent.as_ref().map(|s| n1 / (n1 + solvent_g / s.mm));
    let mut rows = Vec::new();
    if let Some(mv) = molarity {
        rows.push(("molarity", format!("{} mol/L", n(mv))));
    }
    rows.push(("molality", format!("{} mol/kg", n(molality))));
    rows.push(("mass fraction", format!("{}  ({}%)", n(mass_fraction), n(100.0 * mass_fraction))));
    if let Some(x) = mole_fraction {
        rows.push(("mole fraction", n(x)));
    }
    let key = match what.as_str() {
        "molality" => "molality",
        "mass fraction" | "mass percent" | "percent by mass" | "weight percent" => "mass fraction",
        "mole fraction" => "mole fraction",
        _ => "molarity",
    };
    let (_, val) = rows.iter().find(|r| r.0 == key)?.clone();
    let label = format!("{} {} of {} in {} {} of {}", &m[2], &m[3], solute.label, &m[7], &m[8], solvent.as_ref().map_or(solvent_name.clone(), |s| s.label.clone()));
    let method = match key {
        "molality" => "molality = moles of solute ÷ kilograms of solvent",
        "mass fraction" => "mass fraction = mass of solute ÷ total mass",
        "mole fraction" => "mole fraction = moles of solute ÷ total moles",
        _ => "molarity = moles of solute ÷ litres of solution (taken as the solvent volume)",
    };
    let pods = vec![
        pod("Input", "Input interpretation", text(label.clone())),
        primary(pod("Result", &key[..1].to_uppercase().chars().chain(key[1..].chars()).collect::<String>(), text(val))),
        pod("All", "Concentration measures", table(None, rows.iter().map(|(k, v)| vec![text(*k), text(v.clone())]).collect())),
        pod("Amounts", "Amounts", text(format!("solute: {} mol = {} g;  solvent: {} g", n(n1), n(mass1), n(solvent_g)))),
        pod("Method", "Method", text(method)),
    ];
    Some(Answer::new(label, pods))
}

// ---------------------------------------------------------------------------
// dilution and preparation
// ---------------------------------------------------------------------------

fn conc_unit(u: &str) -> Option<(f64, &'static str)> {
    let u = u.trim().to_lowercase();
    Some(match u.as_str() {
        "m" | "molar" | "mol/l" => (1.0, "M"),
        "mm" | "millimolar" | "mmol/l" => (1e-3, "M"),
        "µm" | "um" | "micromolar" => (1e-6, "M"),
        "dm" | "decimolar" => (0.1, "M"),
        "n" | "normal" => (1.0, "N"),
        _ if u.starts_with('%') || u.starts_with("percent") => (1.0, "%"),
        _ => return None,
    })
}

fn dilute(c: &str, l: &str) -> Option<Answer> {
    let r = Regex::new(&format!(r"^dilute {NUM}\s*{UNIT} (?:of )?(?:a )?{NUM}\s*(m|molar|mm|millimolar|%\s*(?:\(?w/v\)?|\(?w/w\)?|\(?v/v\)?)?|percent) (.+?) (with|in|to) {NUM}\s*{UNIT}(?: (?:of )?(?:water|solvent|total))?$")).ok()?;
    let m = r.captures(l)?;
    let (v1, (k1, f1)) = (number(&m[1])?, unit(&m[2])?);
    let (c1, (cf, cu)) = (number(&m[3])?, conc_unit(&m[4])?);
    let (v2, (k2, f2)) = (number(&m[7])?, unit(&m[8])?);
    if k1 != Kind::Volume || k2 != Kind::Volume {
        return None;
    }
    let total = if &m[6] == "to" { v2 * f2 } else { v1 * f1 + v2 * f2 };
    if total <= v1 * f1 * 0.999 {
        return None;
    }
    let c2 = c1 * v1 * f1 / total;
    let what = orig(c, &m, 5);
    let shown = |x: f64| if cu == "%" { format!("{}%{}", n(x), m[4].trim_start_matches('%').trim().to_string().replace(['(', ')'], " ").trim_end()) } else if cf < 1.0 { format!("{} mM", n(x)) } else { format!("{} M", n(x)) };
    let _ = cf;
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("dilute {} {} of {} {what} to a total of {} L", &m[1], &m[2], shown(c1), n(total)))),
        primary(pod("Result", "Final concentration", text(shown(c2)))),
        pod("Volume", "Final volume", text(format!("{} mL", n(total * 1000.0)))),
        pod("Factor", "Dilution factor", text(n(total / (v1 * f1)))),
        pod("Method", "Method", text("C₁ V₁ = C₂ V₂")),
    ];
    Some(Answer::new(format!("dilution of {what}"), pods))
}

fn prepare(c: &str, l: &str) -> Option<Answer> {
    let from_stock = Regex::new(&format!(r"^(?:prepare|preparation of|prepare a|make|making|how to (?:prepare|make))\s+(?:a\s+)?{NUM}\s*{UNIT}\s+(?:of\s+)?(?:a\s+)?{NUM}\s*(m|molar|mm|millimolar|dm|µm|um|%)\s+(.+?)(?:\s+solution)?\s+from\s+(?:a\s+)?{NUM}\s*(m|molar|mm|millimolar|%)(?:\s+stock)?(?:\s+solution)?$")).ok()?;
    if let Some(m) = from_stock.captures(l) {
        let (v, (k, f)) = (number(&m[1])?, unit(&m[2])?);
        let (c2, (cf2, _)) = (number(&m[3])?, conc_unit(&m[4])?);
        let (c1, (cf1, _)) = (number(&m[6])?, conc_unit(&m[7])?);
        if k != Kind::Volume || c1 * cf1 <= c2 * cf2 {
            return None;
        }
        let total = v * f;
        let stock = total * c2 * cf2 / (c1 * cf1);
        let what = orig(c, &m, 5);
        let pods = vec![
            pod("Input", "Input interpretation", text(format!("prepare {} {} of {} {} {what} from {} {} stock", &m[1], &m[2], &m[3], &m[4], &m[6], &m[7]))),
            primary(pod("Result", "Recipe", text(format!("take {} mL of the stock and add water to a total volume of {} mL", n(stock * 1000.0), n(total * 1000.0))))),
            pod("Water", "Water added (approximately)", text(format!("{} mL", n((total - stock) * 1000.0)))),
            pod("Method", "Method", text("C₁ V₁ = C₂ V₂")),
        ];
        return Some(Answer::new(format!("prepare {what}"), pods));
    }
    let r = Regex::new(&format!(r"^(?:prepare|preparation of|prepare a|make|making|how to (?:prepare|make))\s+(?:a\s+)?(?:{NUM}\s*{UNIT}\s+(?:of\s+)?(?:a\s+)?)?{NUM}\s*(m|molar|mm|millimolar|dm|µm|um|decimolar)\s+(.+?)(?:\s+solution)?$|^{NUM}\s*(m|molar|mm|millimolar|dm|µm|um|decimolar)\s+(.+?)(?:\s+solution)?\s+preparation$")).ok()?;
    let m = r.captures(l)?;
    let (vol, conc, cu, name_k) = if m.get(5).is_some() {
        let v = match (m.get(1), m.get(2)) {
            (Some(a), Some(u)) => {
                let (k, f) = unit(u.as_str())?;
                if k != Kind::Volume {
                    return None;
                }
                number(a.as_str())? * f
            }
            _ => 1.0,
        };
        (v, number(&m[3])?, m[4].to_string(), 5)
    } else {
        (1.0, number(&m[6])?, m[7].to_string(), 8)
    };
    let (cf, _) = conc_unit(&cu)?;
    let t = thing(orig(c, &m, name_k))?;
    let moles = vol * conc * cf;
    let grams = moles * t.mm;
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("prepare {} L of {} {} {}", n(vol), n(conc), cu.to_uppercase().replace("MOLAR", "M"), t.label))),
        primary(pod("Result", "Amount of solute", text(format!("{} g  ({} mol)", n(grams), n(moles))))),
        pod("Recipe", "Recipe", text(format!("dissolve {} g of {} in water and make up to {} mL", n(grams), t.name, n(vol * 1000.0)))),
    ];
    if let Some(d) = density(&t.name) {
        pods.push(pod("Liquid", "As a volume of the pure liquid", text(format!("{} mL  (density {} g/mL)", n(grams / d), n(d)))));
    }
    pods.push(pod("Method", "Method", text("mass = molarity × volume × molar mass")));
    Some(Answer::new(format!("prepare {}", t.name), pods))
}

/// Acidic hydrogens or hydroxides per formula unit.
fn equivalents(formula: &str, counts: &BTreeMap<String, u32>) -> Option<u32> {
    let known: &[(&str, u32)] = &[("H2SO4", 2), ("H3PO4", 3), ("H3PO3", 2), ("H2CO3", 2), ("H2C2O4", 2), ("H2SO3", 2), ("HCl", 1), ("HNO3", 1), ("HBr", 1), ("HI", 1), ("HF", 1), ("HClO4", 1), ("HCN", 1), ("CH3COOH", 1), ("C2H4O2", 1), ("HCOOH", 1), ("C6H8O7", 3)];
    if let Some((_, k)) = known.iter().find(|(f, _)| *f == formula) {
        return Some(*k);
    }
    if let Some(i) = formula.find("(OH)") {
        return formula[i + 4..].chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().ok().or(Some(1));
    }
    if formula.ends_with("OH") && counts.len() == 3 && !formula.starts_with('C') {
        return Some(1);
    }
    if formula.starts_with('H') && !formula.starts_with("He") && !formula.starts_with("Hg") && !formula.starts_with("Hf") && !formula.starts_with("Ho") {
        let k: String = formula[1..].chars().take_while(|c| c.is_ascii_digit()).collect();
        return Some(k.parse().unwrap_or(1));
    }
    None
}

fn normality(c: &str, l: &str) -> Option<Answer> {
    let r = Regex::new(&format!(r"^normality of (?:a )?{NUM}\s*(m|molar|mm|millimolar)\s+(.+?)(?:\s+solution)?$")).ok()?;
    let m = r.captures(l)?;
    let (conc, (cf, _)) = (number(&m[1])?, conc_unit(&m[2])?);
    let name = orig(c, &m, 3);
    let (label, formula, counts) = substance(name)?;
    let eq = equivalents(&formula, &counts)?;
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("normality of {} M {label}", n(conc * cf)))),
        primary(pod("Result", "Normality", text(format!("{} N", n(conc * cf * eq as f64))))),
        pod("Eq", "Equivalents per mole", text(format!("{eq} ({})", if formula.contains("OH") && !formula.starts_with('H') { "hydroxide ions" } else { "acidic hydrogens" }))),
        pod("Method", "Method", text("normality = molarity × equivalents per mole")),
    ];
    Some(Answer::new(format!("normality of {label}"), pods))
}

/// "100 mL 0.4 M sulfuric acid", "100 mL of 1.5 molar K2CO3 in THF"
fn solution_amount(c: &str, l: &str) -> Option<Answer> {
    let r = Regex::new(&format!(r"^{NUM}\s*{UNIT}\s+(?:of\s+)?(?:a\s+)?{NUM}\s*(m|molar|mm|millimolar)\s+(.+?)(?:\s+solution)?(?:\s+in\s+(.+))?$")).ok()?;
    let m = r.captures(l)?;
    let (v, (k, f)) = (number(&m[1])?, unit(&m[2])?);
    if k != Kind::Volume {
        return None;
    }
    let (conc, (cf, _)) = (number(&m[3])?, conc_unit(&m[4])?);
    let t = thing(orig(c, &m, 5))?;
    let moles = v * f * conc * cf;
    let solvent = m.get(6).map_or("water".to_string(), |_| orig(c, &m, 6).to_string());
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("{} {} of {} M {} in {solvent}", &m[1], &m[2], n(conc * cf), t.label))),
        primary(pod("Result", "Amount of solute", text(format!("{} mol", n(moles))))),
        pod("Mass", "Mass of solute", text(format!("{} g", n(moles * t.mm)))),
        pod("Particles", "Number of solute particles", text(n(moles * AVOGADRO))),
        pod("MolarMass", "Molar mass", text(format!("{} g/mol", n(t.mm)))),
    ];
    Some(Answer::new(format!("{} of {}", &m[1], t.name), pods))
}

/// "0.4 molar potassium hydroxide", "5 mass% HCl"
fn bare_solution(q: &Query, c: &str, l: &str) -> Option<Answer> {
    let r = Regex::new(&format!(r"^{NUM}\s*(m|molar|mm|millimolar|mass\s*%|% by mass|wt\s*%|%\s*w/w)\s+(.+?)(?:\s+solution)?$")).ok()?;
    let m = r.captures(l)?;
    let conc = number(&m[1])?;
    let t = thing(orig(c, &m, 3))?;
    let unit_l = m[2].to_lowercase();
    let mut pods = Vec::new();
    if unit_l.contains('%') {
        // grams of solute per 100 g of solution
        let per_kg_solvent = 10.0 * conc / (1.0 - conc / 100.0) / t.mm;
        pods.push(pod("Input", "Input interpretation", text(format!("{}% by mass {}", n(conc), t.label))));
        pods.push(primary(pod("Result", "Mass fraction", text(format!("{} g per 100 g of solution", n(conc))))));
        pods.push(pod("Molality", "Molality", text(format!("{} mol/kg", n(per_kg_solvent * 100.0 / 100.0)))));
    } else {
        let molar = conc * conc_unit(&unit_l)?.0;
        pods.push(pod("Input", "Input interpretation", text(format!("{} M {}", n(molar), t.label))));
        pods.push(primary(pod("Result", "Mass concentration", text(format!("{} g/L", n(molar * t.mm))))));
        if let Some(r) = crate::engine::query_with(&format!("pH of {} M {}", n(molar), t.name), q.opts).pods.into_iter().find(|p| p.primary) {
            pods.push(Pod { id: "pH".into(), title: "pH".into(), subpods: r.subpods, primary: false });
        }
    }
    pods.push(pod("MolarMass", "Molar mass of solute", text(format!("{} g/mol", n(t.mm)))));
    Some(Answer::new(format!("{} solution", t.name), pods))
}

// ---------------------------------------------------------------------------
// reactions with amounts
// ---------------------------------------------------------------------------

struct Species {
    formula: String,
    counts: BTreeMap<String, u32>,
    moles: Option<f64>,
    shown: Option<String>,
}

fn parse_side(c: &str, side: &str) -> Option<Vec<Species>> {
    let amt = Regex::new(&format!(r"(?i)^{NUM}\s*{UNIT}\s+(?:of\s+)?(.+)$")).ok()?;
    let _ = c;
    let mut out = Vec::new();
    for part in side.split(" + ").flat_map(|p| p.split("+ ")).flat_map(|p| p.split(" +")) {
        let mut p = part.trim();
        if p.is_empty() {
            continue;
        }
        let (mut moles_raw, mut shown) = (None, None);
        if let Some(m) = amt.captures(p) {
            let (v, (k, f)) = (number(&m[1])?, unit(&m[2])?);
            moles_raw = Some((v, k, f));
            shown = Some(format!("{} {}", &m[1], &m[2]));
            p = m.get(3).unwrap().as_str().trim();
        }
        for st in ["(g)", "(l)", "(s)", "(aq)"] {
            p = p.trim_end_matches(st).trim();
        }
        let p = p.trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == ' ');
        let (_, formula, counts) = substance(p)?;
        let mm = molar_mass(&counts);
        let moles = match moles_raw {
            Some((v, k, f)) => {
                let t = Thing { label: String::new(), name: p.to_string(), counts: counts.clone(), mm };
                Some(moles_of(v, k, f, &t)?)
            }
            None => None,
        };
        out.push(Species { formula, counts, moles, shown });
    }
    (!out.is_empty()).then_some(out)
}

fn stoichiometry(c: &str) -> Option<Answer> {
    let arrow = ["-->", "->", "=>", "→", "⟶"].iter().find(|a| c.contains(**a))?;
    let (lhs, rhs) = c.split_once(arrow)?;
    let has_amount = Regex::new(&format!(r"(?i)(?:^|\+)\s*{NUM}\s*{UNIT}\s")).ok()?;
    if !has_amount.is_match(lhs) && !has_amount.is_match(rhs) {
        return None;
    }
    let (left, right) = (parse_side(c, lhs)?, parse_side(c, rhs)?);
    let coef = coefficients(&left.iter().map(|s| s.counts.clone()).collect::<Vec<_>>(), &right.iter().map(|s| s.counts.clone()).collect::<Vec<_>>())?;
    let all: Vec<(&Species, i128, bool)> = left.iter().zip(&coef).map(|(s, k)| (s, *k, true)).chain(right.iter().zip(&coef[left.len()..]).map(|(s, k)| (s, *k, false))).collect();
    // extent of reaction: the limiting reactant, else the given product
    let given_r: Vec<(usize, f64)> = all.iter().enumerate().filter(|(_, (s, _, r))| *r && s.moles.is_some()).map(|(i, (s, k, _))| (i, s.moles.unwrap() / *k as f64)).collect();
    let (xi, limiting) = if !given_r.is_empty() {
        let (i, x) = given_r.iter().cloned().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())?;
        (x, Some(i))
    } else {
        let (i, (s, k, _)) = all.iter().enumerate().find(|(_, (s, _, _))| s.moles.is_some())?;
        let _ = i;
        (s.moles? / *k as f64, None)
    };
    let side = |r: bool| all.iter().filter(|x| x.2 == r).map(|(s, k, _)| if *k == 1 { pretty(&s.formula) } else { format!("{k} {}", pretty(&s.formula)) }).collect::<Vec<_>>().join(" + ");
    let balanced = format!("{} → {}", side(true), side(false));
    let rows: Vec<Vec<Content>> = all
        .iter()
        .enumerate()
        .map(|(i, (s, k, r))| {
            let mm = molar_mass(&s.counts);
            let used = xi * *k as f64;
            let left_over = s.moles.filter(|_| *r && Some(i) != limiting).map(|m0| format!("{} g left over", n((m0 - used) * mm))).unwrap_or_default();
            vec![
                text(pretty(&s.formula)),
                text(if *r { "reactant" } else { "product" }),
                text(s.shown.clone().unwrap_or_default()),
                text(format!("{} mol", n(used))),
                text(format!("{} g", n(used * mm))),
                text(left_over),
            ]
        })
        .collect();
    let result = if limiting.is_some() {
        all.iter().filter(|x| !x.2).map(|(s, k, _)| format!("{} g {}", n(xi * *k as f64 * molar_mass(&s.counts)), pretty(&s.formula))).collect::<Vec<_>>().join(", ")
    } else {
        all.iter().filter(|x| x.2).map(|(s, k, _)| format!("{} g {}", n(xi * *k as f64 * molar_mass(&s.counts)), pretty(&s.formula))).collect::<Vec<_>>().join(", ")
    };
    let mut pods = vec![
        pod("Input", "Input interpretation", text(c.to_string())),
        pod("Balanced", "Balanced equation", text(balanced.clone())),
        primary(pod("Result", if limiting.is_some() { "Theoretical yield" } else { "Reactants needed" }, text(result))),
    ];
    if let (Some(i), true) = (limiting, given_r.len() > 1) {
        pods.push(pod("Limiting", "Limiting reactant", text(pretty(&all[i].0.formula))));
    }
    pods.push(pod("Table", "Stoichiometry", table(Some(vec!["species", "role", "given", "moles", "mass", ""]), rows)));
    Some(Answer::new(format!("stoichiometry of {balanced}"), pods))
}

// ---------------------------------------------------------------------------
// polyatomic ions
// ---------------------------------------------------------------------------

pub(super) const IONS: &[(&str, &str, i32)] = &[
    ("ammonium", "NH4", 1),
    ("hydronium", "H3O", 1),
    ("mercury(I)", "Hg2", 2),
    ("acetate", "CH3COO", -1),
    ("bicarbonate", "HCO3", -1),
    ("hydrogen carbonate", "HCO3", -1),
    ("bisulfate", "HSO4", -1),
    ("hydrogen sulfate", "HSO4", -1),
    ("bisulfite", "HSO3", -1),
    ("hydrogen sulfite", "HSO3", -1),
    ("dihydrogen phosphate", "H2PO4", -1),
    ("hydrogen phosphate", "HPO4", -2),
    ("hydroxide", "OH", -1),
    ("cyanide", "CN", -1),
    ("cyanate", "OCN", -1),
    ("thiocyanate", "SCN", -1),
    ("nitrate", "NO3", -1),
    ("nitrite", "NO2", -1),
    ("perchlorate", "ClO4", -1),
    ("chlorate", "ClO3", -1),
    ("chlorite", "ClO2", -1),
    ("hypochlorite", "ClO", -1),
    ("bromate", "BrO3", -1),
    ("iodate", "IO3", -1),
    ("periodate", "IO4", -1),
    ("permanganate", "MnO4", -1),
    ("azide", "N3", -1),
    ("formate", "HCOO", -1),
    ("carbonate", "CO3", -2),
    ("sulfate", "SO4", -2),
    ("sulfite", "SO3", -2),
    ("thiosulfate", "S2O3", -2),
    ("oxalate", "C2O4", -2),
    ("chromate", "CrO4", -2),
    ("dichromate", "Cr2O7", -2),
    ("peroxide", "O2", -2),
    ("silicate", "SiO3", -2),
    ("manganate", "MnO4", -2),
    ("phosphate", "PO4", -3),
    ("phosphite", "PO3", -3),
    ("arsenate", "AsO4", -3),
    ("borate", "BO3", -3),
    ("citrate", "C6H5O7", -3),
];

fn charge_text(z: i32) -> String {
    let mag = z.unsigned_abs();
    let sign = if z > 0 { "+" } else { "−" };
    if mag == 1 { sign.to_string() } else { format!("{mag}{sign}") }
}

fn ion_rows(list: &[&(&str, &str, i32)]) -> Vec<Vec<Content>> {
    list.iter().map(|(name, f, z)| vec![text(name.to_string()), text(format!("{}{}", pretty(f), charge_text(*z))), text(z.to_string()), text(format!("{} g/mol", n(parse_formula(f).map(|c| molar_mass(&c)).unwrap_or(0.0))))]).collect()
}

fn ions(c: &str, l: &str) -> Option<Answer> {
    if re!(r"^(?:list of |table of |common )?(?:charges of |names of )?(?:common )?polyatomic ions$").is_match(l) {
        let all: Vec<&(&str, &str, i32)> = IONS.iter().collect();
        let pods = vec![pod("Input", "Input", text("polyatomic ions")), primary(pod("Result", "Polyatomic ions", table(Some(vec!["name", "formula", "charge", "mass"]), ion_rows(&all))))];
        return Some(Answer::new("polyatomic ions", pods));
    }
    // by name, or written with its charge: "NH4+", "PO4-3", "SO4^2-"
    let parts: Vec<&str> = c.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    let mut found: Vec<(&str, String, i32)> = Vec::new();
    for p in &parts {
        let pl = p.to_lowercase();
        let name = pl.trim_end_matches(" ion").trim_end_matches(" anion").trim_end_matches(" cation").trim();
        if let Some(i) = IONS.iter().find(|i| i.0 == name) {
            found.push((i.0, i.1.to_string(), i.2));
            continue;
        }
        // "NH4+", "PO4-3", "SO4^2-", "SO42-": try each reading, known ions first
        let mut cands: Vec<(String, i32)> = Vec::new();
        if let Some(m) = re!(r"^([A-Z][A-Za-z0-9()]*)\s*\^?\s*([+-])(\d?)$").captures(p) {
            let k: i32 = if m[3].is_empty() { 1 } else { m[3].parse().ok()? };
            cands.push((m[1].to_string(), if &m[2] == "+" { k } else { -k }));
        }
        if let Some(m) = re!(r"^([A-Z][A-Za-z0-9()]*?)\s*\^?\s*(\d)([+-])$").captures(p) {
            let k: i32 = m[2].parse().ok()?;
            cands.push((m[1].to_string(), if &m[3] == "+" { k } else { -k }));
        }
        let pick = cands.iter().find(|(f, z)| IONS.iter().any(|i| i.1 == f && i.2 == *z)).or_else(|| cands.iter().find(|(f, _)| by_symbol(f).is_some())).or_else(|| cands.iter().find(|(f, _)| parse_formula(f).is_some()))?;
        let (f, z) = pick.clone();
        let name = IONS.iter().find(|i| i.1 == f && i.2 == z).map_or("", |i| i.0);
        found.push((name, f, z));
    }
    if found.is_empty() || (parts.len() == 1 && found[0].0.is_empty() && !c.contains(['+', '-'])) {
        return None;
    }
    let rows: Vec<Vec<Content>> = found
        .iter()
        .map(|(name, f, z)| {
            const ANIONS: &[(&str, &str)] = &[("H", "hydride"), ("C", "carbide"), ("N", "nitride"), ("O", "oxide"), ("F", "fluoride"), ("P", "phosphide"), ("S", "sulfide"), ("Cl", "chloride"), ("Se", "selenide"), ("Br", "bromide"), ("Te", "telluride"), ("I", "iodide"), ("As", "arsenide")];
            let label = if !name.is_empty() {
                name.to_string()
            } else if let Some((_, a)) = ANIONS.iter().find(|(sy, _)| sy == f).filter(|_| *z < 0) {
                format!("{a} ion")
            } else {
                by_symbol(f).map_or(f.clone(), |e| format!("{} ion", e.name.to_lowercase()))
            };
            let electrons: i64 = parse_formula(f).map(|cs| cs.iter().map(|(s, k)| by_symbol(s).map_or(0, |e| e.z as i64) * *k as i64).sum::<i64>()).unwrap_or(0) - *z as i64;
            vec![text(label), text(format!("{}{}", pretty(f), charge_text(*z))), text(z.to_string()), text(format!("{} g/mol", n(parse_formula(f).map(|c| molar_mass(&c)).unwrap_or(0.0)))), text(electrons.to_string())]
        })
        .collect();
    let res = if rows.len() == 1 { format!("{}, charge {}", content_text(&rows[0][1]), content_text(&rows[0][2])) } else { rows.iter().map(|r| format!("{} ({})", content_text(&r[1]), content_text(&r[0]))).collect::<Vec<_>>().join(", ") };
    let pods = vec![
        pod("Input", "Input interpretation", text(rows.iter().map(|r| content_text(&r[0])).collect::<Vec<_>>().join(", "))),
        primary(pod("Result", if rows.len() == 1 { "Ion" } else { "Ions" }, text(res))),
        pod("Table", "Properties", table(Some(vec!["name", "formula", "charge", "molar mass", "electrons"]), rows)),
    ];
    Some(Answer::new("ions", pods))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first(a: Answer) -> String {
        content_text(&a.pods.iter().find(|p| p.primary).unwrap().subpods[0].content)
    }

    #[test]
    fn solutions() {
        let a = quantity("500mg of silver nitrate", "500mg of silver nitrate").unwrap();
        assert!(first(a).starts_with("0.00294"));
        let a = concentration("molality of 0.385 g cholesterol in 40.0 g chloroform", "molality of 0.385 g cholesterol in 40.0 g chloroform").unwrap();
        assert!(first(a).starts_with("0.0248"), "molality");
        let a = dilute("dilute 30 ml of 3 molar NaOH with 30 ml of water", "dilute 30 ml of 3 molar naoh with 30 ml of water").unwrap();
        assert_eq!(first(a), "1.5 M");
        let a = prepare("Prepare 1.25 L of 3 M NaOH from 10 M", "prepare 1.25 l of 3 m naoh from 10 m").unwrap();
        assert!(first(a).starts_with("take 375 mL"));
        let a = normality("normality of 12 molar phosphoric acid", "normality of 12 molar phosphoric acid").unwrap();
        assert_eq!(first(a), "36 N");
        let a = stoichiometry("22g NaOH + 16g MgCl2 -> Mg(OH)2 + NaCl").unwrap();
        assert!(first(a).starts_with("9.8"), "yield");
        let a = ions("NH4+", "nh4+").unwrap();
        assert!(first(a).starts_with("NH₄+"));
    }
}
