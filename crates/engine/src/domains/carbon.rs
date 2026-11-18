//! Carbon dioxide emissions of fuels and electricity.
//!
//! "carbon footprint natural gas", "carbon footprint coal 100W",
//! "23W petroleum carbon footprint", "carbon footprint gas 15000 Btu/h",
//! "carbon footprint driving 536 miles at 32mpg".

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

/// names | display | kg CO₂ per MMBtu burned (EPA) | kg CO₂ per kWh of US electricity generated from it (EIA)
const FUELS: &[(&[&str], &str, f64, f64)] = &[
    (&["natural gas", "gas", "methane"], "natural gas", 53.06, 0.440),
    (&["coal", "bituminous coal"], "coal", 93.28, 1.043),
    (&["petroleum", "oil", "fuel oil", "heating oil", "diesel"], "petroleum (distillate fuel oil)", 73.96, 1.080),
    (&["propane", "lpg"], "propane", 62.87, 0.0),
    (&["gasoline", "petrol"], "gasoline", 70.22, 0.0),
];

const MMBTU_PER_KWH: f64 = 0.003_412_142;

fn kg(v: f64) -> String {
    if v >= 1000.0 {
        format!("{} t", fmt_float((v / 1000.0 * 1000.0).round() / 1000.0, 5))
    } else if v >= 1.0 {
        format!("{} kg", fmt_float((v * 1000.0).round() / 1000.0, 5))
    } else {
        format!("{} g", fmt_float((v * 1e5).round() / 100.0, 5))
    }
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    if !re!(r"carbon footprint|co2 emissions?|carbon emissions?|co₂ emissions?").is_match(&l) {
        return None;
    }
    let rest = re!(r"(?:the )?(?:carbon footprint|co2 emissions?|carbon emissions?|co₂ emissions?)(?: of| for| from)?").replace_all(&l, " ").to_string();
    let rest = re!(r"\s+").replace_all(rest.trim(), " ").to_string();

    // driving: "driving 536 miles at 32mpg"
    if let Some(m) = re!(r"^(?:driving|a car|car|a drive of|drive)?\s*(\d[\d,]*(?:\.\d+)?) (miles|mi|km|kilometers) (?:at|with|getting) (\d+(?:\.\d+)?) ?(mpg|miles per gallon|l/100 ?km)$").captures(&rest) {
        let d: f64 = m[1].replace(',', "").parse().ok()?;
        let e: f64 = m[3].parse().ok()?;
        let miles = if m[2].starts_with('k') { d / 1.609344 } else { d };
        let gallons = if m[4].starts_with('l') { miles * 1.609344 * e / 100.0 / 3.785411784 } else { miles / e };
        let co2 = gallons * 8.887;
        return Some(Answer::new("carbon footprint of driving", vec![
            pod("Input", "Input interpretation", text(format!("driving {} {} at {} {}", &m[1], &m[2], &m[3], &m[4]))),
            primary(pod("Result", "CO₂ emitted", text(kg(co2)))),
            pod("Table", "Details", table(None, vec![
                vec![text("fuel used"), text(format!("{} US gal ({} L)", fmt_float((gallons * 100.0).round() / 100.0, 6), fmt_float((gallons * 3.785411784 * 10.0).round() / 10.0, 6)))],
                vec![text("emission factor"), text("8.887 kg CO₂ per US gallon of gasoline (EPA)")],
            ])),
        ]));
    }

    let has = |n: &str| rest == n || rest.starts_with(&format!("{n} ")) || rest.ends_with(&format!(" {n}")) || rest.contains(&format!(" {n} "));
    let fuel = FUELS.iter().find(|f| f.0.iter().any(|n| has(n)))?;
    let (_, name, per_mmbtu, per_kwh_e) = *fuel;
    // a power: "100W", "23 W", "2 kW"; or a heat rate "15000 Btu/h"
    let power = re!(r"(\d[\d,]*(?:\.\d+)?) ?(w|kw|mw|watts?|kilowatts?|megawatts?|btu/h|btu/hr|btu per hour|btuh)\b").captures(&rest);
    if let Some(p) = power {
        let v: f64 = p[1].replace(',', "").parse().ok()?;
        let unit = &p[2];
        let (kg_per_hour, basis) = if unit.starts_with("btu") {
            (v / 1e6 * per_mmbtu, format!("burning {name} at {} Btu/h ({} kg CO₂ per MMBtu, EPA)", fmt_float(v, 9), fmt_float(per_mmbtu, 5)))
        } else {
            if per_kwh_e == 0.0 {
                return None;
            }
            let kw = match unit { u if u.starts_with("kw") || u.starts_with("kilo") => v, u if u.starts_with("mw") || u.starts_with("mega") => v * 1000.0, _ => v / 1000.0 };
            (kw * per_kwh_e, format!("{} of electricity generated from {name} ({} kg CO₂ per kWh, EIA US average)", if kw < 1.0 { format!("{} W", fmt_float(kw * 1000.0, 6)) } else { format!("{} kW", fmt_float(kw, 6)) }, fmt_float(per_kwh_e, 4)))
        };
        let rows = vec![
            vec![text("per hour"), text(kg(kg_per_hour))],
            vec![text("per day"), text(kg(kg_per_hour * 24.0))],
            vec![text("per year"), text(kg(kg_per_hour * 24.0 * 365.25))],
        ];
        return Some(Answer::new(format!("carbon footprint of {name}"), vec![
            pod("Input", "Input interpretation", text(basis)),
            primary(pod("Result", "CO₂ emissions", text(format!("{} per hour | {} per year", kg(kg_per_hour), kg(kg_per_hour * 24.0 * 365.25))))),
            pod("Table", "Emissions", table(None, rows)),
        ]));
    }
    // the factors alone
    let per_kwh_t = per_mmbtu * MMBTU_PER_KWH;
    let mut rows = vec![
        vec![text("per MMBtu of fuel burned"), text(format!("{} kg CO₂", fmt_float(per_mmbtu, 5)))],
        vec![text("per kWh of heat released"), text(format!("{} kg CO₂", fmt_float((per_kwh_t * 1000.0).round() / 1000.0, 4)))],
    ];
    if per_kwh_e > 0.0 {
        rows.push(vec![text("per kWh of electricity generated (US)"), text(format!("{} kg CO₂", fmt_float(per_kwh_e, 4)))]);
    }
    if name == "gasoline" {
        rows.push(vec![text("per US gallon"), text("8.887 kg CO₂")]);
    }
    if name.starts_with("petroleum") {
        rows.push(vec![text("per US gallon of diesel"), text("10.18 kg CO₂")]);
    }
    Some(Answer::new(format!("carbon footprint of {name}"), vec![
        pod("Input", "Input interpretation", text(format!("{name} | carbon dioxide emissions"))),
        primary(pod("Result", "Emission factor", text(if per_kwh_e > 0.0 { format!("{} kg CO₂ per kWh of electricity | {} kg CO₂ per kWh of heat", fmt_float(per_kwh_e, 4), fmt_float((per_kwh_t * 1000.0).round() / 1000.0, 4)) } else { format!("{} kg CO₂ per kWh of heat", fmt_float((per_kwh_t * 1000.0).round() / 1000.0, 4)) }))),
        pod("Table", "Emission factors", table(None, rows)),
        pod("Source", "Sources", text("US EPA emission factors for greenhouse gas inventories; US EIA average emissions per kWh generated")),
    ]))
}
