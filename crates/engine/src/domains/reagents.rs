//! Concentrated laboratory reagents: mass fraction, density, molarity and
//! normality of commercial concentrated acids, bases and peroxide.
//!
//! "concentrated acetic acid", "molarity of concentrated hydrochloric acid",
//! "concentrated sulfuric acid", "concentrated ammonia".

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

/// names | display | formula | mass % | density g/mL | molar mass g/mol | acidic or basic equivalents per mole
const REAGENTS: &[(&[&str], &str, &str, f64, f64, f64, u32)] = &[
    (&["acetic acid", "glacial acetic acid", "ethanoic acid"], "glacial acetic acid", "CH₃COOH", 99.7, 1.049, 60.05, 1),
    (&["hydrochloric acid", "hcl", "muriatic acid"], "concentrated hydrochloric acid", "HCl", 37.0, 1.19, 36.46, 1),
    (&["sulfuric acid", "sulphuric acid", "h2so4"], "concentrated sulfuric acid", "H₂SO₄", 98.0, 1.84, 98.08, 2),
    (&["nitric acid", "hno3"], "concentrated nitric acid", "HNO₃", 70.0, 1.42, 63.01, 1),
    (&["phosphoric acid", "orthophosphoric acid", "h3po4"], "concentrated phosphoric acid", "H₃PO₄", 85.0, 1.69, 98.00, 3),
    (&["ammonia", "ammonium hydroxide", "aqueous ammonia", "nh3", "nh4oh"], "concentrated aqueous ammonia", "NH₃", 28.0, 0.90, 17.03, 1),
    (&["hydrofluoric acid", "hf"], "concentrated hydrofluoric acid", "HF", 48.0, 1.15, 20.01, 1),
    (&["formic acid", "methanoic acid"], "concentrated formic acid", "HCOOH", 90.0, 1.20, 46.03, 1),
    (&["perchloric acid", "hclo4"], "concentrated perchloric acid", "HClO₄", 70.0, 1.67, 100.46, 1),
    (&["hydrobromic acid", "hbr"], "concentrated hydrobromic acid", "HBr", 48.0, 1.49, 80.91, 1),
    (&["hydriodic acid", "hydroiodic acid", "hi"], "concentrated hydriodic acid", "HI", 57.0, 1.70, 127.91, 1),
    (&["hydrogen peroxide", "h2o2"], "30% hydrogen peroxide", "H₂O₂", 30.0, 1.11, 34.01, 0),
    (&["sodium hydroxide", "naoh", "lye"], "50% sodium hydroxide", "NaOH", 50.0, 1.53, 40.00, 1),
    (&["potassium hydroxide", "koh"], "45% potassium hydroxide", "KOH", 45.0, 1.45, 56.11, 1),
];

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let m = re!(r"^(?:(?:what is the )?(molarity|normality|concentration|density|molality) of )?(?:the )?(concentrated|conc\.?|glacial|stock) (.+?)(?: (molarity|normality|concentration|density))?$").captures(&l)?;
    let name = m[3].trim();
    let glacial = &m[2] == "glacial";
    let r = REAGENTS.iter().find(|r| r.0.contains(&name))?;
    if glacial && !r.0.contains(&"acetic acid") {
        return None;
    }
    let (_, disp, formula, pct, dens, mm, eq) = *r;
    let molarity = pct / 100.0 * dens * 1000.0 / mm;
    let molality = (pct / 100.0) / mm / (1.0 - pct / 100.0) * 1000.0;
    let prop = m.get(1).or(m.get(4)).map(|x| x.as_str()).unwrap_or("molarity");
    let (title, val) = match prop {
        "density" => ("Density", format!("{} g/mL", fmt_float(dens, 4))),
        "normality" if eq > 0 => ("Normality", format!("{} N", fmt_float((molarity * eq as f64 * 10.0).round() / 10.0, 4))),
        "molality" => ("Molality", format!("{} mol/kg", fmt_float((molality * 10.0).round() / 10.0, 5))),
        _ => ("Molar concentration", format!("{} mol/L", fmt_float((molarity * 10.0).round() / 10.0, 4))),
    };
    let mut rows = vec![
        vec![text("formula"), text(formula)],
        vec![text("mass fraction"), text(format!("{}%", fmt_float(pct, 4)))],
        vec![text("density"), text(format!("{} g/mL", fmt_float(dens, 4)))],
        vec![text("molar mass"), text(format!("{} g/mol", fmt_float(mm, 5)))],
        vec![text("molarity"), text(format!("{} mol/L", fmt_float((molarity * 10.0).round() / 10.0, 4)))],
        vec![text("molality"), text(format!("{} mol/kg", fmt_float((molality * 10.0).round() / 10.0, 5)))],
    ];
    if eq > 0 {
        rows.push(vec![text("normality"), text(format!("{} N", fmt_float((molarity * eq as f64 * 10.0).round() / 10.0, 4)))]);
    }
    rows.push(vec![text("volume for 1 L of 1 M solution"), text(format!("{} mL", fmt_float((1000.0 / molarity * 10.0).round() / 10.0, 4)))]);
    Some(Answer::new(disp, vec![
        pod("Input", "Input interpretation", text(format!("{disp} (typical commercial reagent)"))),
        primary(pod("Result", title, text(val))),
        pod("Table", "Properties", table(None, rows)),
        pod("Note", "Note", text("typical reagent-grade values at about 20 °C; exact assay varies by supplier and lot")),
    ]))
}
