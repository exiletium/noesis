//! Rope by fibre and diameter: approximate breaking strength, weight,
//! specific gravity and water absorption.
//!
//! "20mm manila rope", "21mm nylon rope", "water absorption of 28mm sisal
//! rope", "specific gravity of 28mm polyester rope", "1/2 inch nylon rope".

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

/// fibre | breaking strength coefficient (kN per mm² of diameter², 3-strand
/// laid rope) | fibre specific gravity | water absorption | stretch at break |
/// notes
const FIBRES: &[(&str, f64, f64, &str, &str, &str)] = &[
    ("nylon", 0.19, 1.14, "moderate: absorbs about 4–9% of its weight and loses 10–15% of its strength when wet", "about 20–30%", "strong and elastic; absorbs shock loads; sinks; good abrasion and UV resistance"),
    ("polyester", 0.17, 1.38, "very low (under 1%); strength is unchanged when wet", "about 12–15%", "low stretch; good UV and abrasion resistance; sinks; common for sailing lines"),
    ("polypropylene", 0.13, 0.91, "none (hydrophobic)", "about 15–20%", "lightweight and floats; weaker UV resistance; low melting point"),
    ("polyethylene", 0.11, 0.95, "none (hydrophobic)", "about 15–20%", "floats; inexpensive; slippery, so knots hold poorly"),
    ("manila", 0.075, 1.38, "high: the natural fibre absorbs water, swells and shrinks, and rots if stored wet", "about 10–12%", "natural abaca fibre; the traditional standard for natural rope; grips well"),
    ("sisal", 0.06, 1.38, "high: the natural fibre absorbs water readily, swells, and rots if stored wet", "about 10–12%", "natural agave fibre; about 80% of the strength of manila; rough and inexpensive"),
    ("hemp", 0.07, 1.48, "high: natural fibre that absorbs water and swells", "about 5–10%", "natural fibre; strong for its type; low stretch"),
    ("cotton", 0.04, 1.54, "high: absorbs water readily (up to about a quarter of its weight)", "about 10–15%", "soft and easy on the hands; weak; used for crafts and light lines"),
];

/// diameter in mm from "20mm", "2 cm", "1/2 inch", "0.75 in"
fn diameter(num: &str, unit: &str) -> Option<f64> {
    let v = if let Some((a, b)) = num.split_once('/') {
        a.trim().parse::<f64>().ok()? / b.trim().parse::<f64>().ok()?
    } else {
        num.parse::<f64>().ok()?
    };
    let mm = match unit {
        "mm" | "millimeter" | "millimeters" | "millimetre" | "millimetres" => v,
        "cm" => v * 10.0,
        "in" | "inch" | "inches" | "\"" | "″" => v * 25.4,
        _ => return None,
    };
    (mm > 0.5 && mm < 200.0).then_some(mm)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let m = re!(r#"^(?:(?:the )?(breaking strength|tensile strength|strength|working load|safe working load|weight|mass|specific gravity|density|water absorption|stretch|elongation)\s+(?:of\s+)?(?:an?\s+)?)?(?:(\d+(?:\.\d+)?|\d+\s*/\s*\d+)[- ]?(mm|cm|millimet(?:er|re)s?|in|inch|inches|"|″)\s+)?(nylon|polyester|polypropylene|polyethylene|manila|sisal|hemp|cotton)\s+(?:rope|line|cord)(?:\s+(breaking strength|strength|weight|specific gravity|water absorption))?$"#).captures(l)?;
    let fibre = &m[4];
    let f = FIBRES.iter().find(|f| f.0 == fibre)?;
    let d = match (m.get(2), m.get(3)) {
        (Some(n), Some(u)) => Some(diameter(&n.as_str().replace(' ', ""), u.as_str())?),
        _ => None,
    };
    let prop = m.get(1).or(m.get(5)).map(|x| x.as_str());
    let mut rows = vec![
        vec![text("fibre"), text(fibre)],
        vec![text("specific gravity of the fibre"), text(format!("{} ({})", fmt_float(f.2, 3), if f.2 < 1.0 { "floats" } else { "sinks" }))],
        vec![text("water absorption"), text(f.3)],
        vec![text("stretch at break"), text(f.4)],
    ];
    let mut result = format!("specific gravity {} | {}", fmt_float(f.2, 3), if f.2 < 1.0 { "floats" } else { "sinks" });
    if let Some(d) = d {
        let mbs = f.1 * d * d; // kN
        let kg_per_m = f.2 * 1000.0 * std::f64::consts::PI * (d / 1000.0).powi(2) / 4.0 * 0.75;
        rows.insert(0, vec![text("diameter"), text(format!("{} mm ({} in)", fmt_float((d * 10.0).round() / 10.0, 4), fmt_float((d / 25.4 * 100.0).round() / 100.0, 3)))]);
        rows.insert(2, vec![text("approximate breaking strength"), text(format!("{} kN ({} kgf | {} lbf)", fmt_float((mbs * 10.0).round() / 10.0, 4), super::geo::fmt_int((mbs * 101.97).round() as u64), super::geo::fmt_int((mbs * 224.81).round() as u64)))]);
        rows.insert(3, vec![text("safe working load (5:1)"), text(format!("{} kN ({} kgf)", fmt_float((mbs / 5.0 * 10.0).round() / 10.0, 4), super::geo::fmt_int((mbs / 5.0 * 101.97).round() as u64)))]);
        rows.insert(4, vec![text("approximate weight"), text(format!("{} kg per 100 m ({} lb per 100 ft)", fmt_float((kg_per_m * 1000.0).round() / 10.0, 4), fmt_float((kg_per_m * 0.67197 * 1000.0).round() / 10.0, 4)))]);
        result = format!("breaking strength about {} kN ({} lbf) | {} kg per 100 m", fmt_float((mbs * 10.0).round() / 10.0, 4), super::geo::fmt_int((mbs * 224.81).round() as u64), fmt_float((kg_per_m * 1000.0).round() / 10.0, 4));
    }
    let (title, res) = match prop {
        Some(p) if p.contains("gravity") || p == "density" => ("Specific gravity", format!("{} ({}; fibre density {} g/cm³)", fmt_float(f.2, 3), if f.2 < 1.0 { "floats in water" } else { "sinks in water" }, fmt_float(f.2, 3))),
        Some(p) if p.contains("absorption") => ("Water absorption", f.3.to_string()),
        Some(p) if p == "stretch" || p == "elongation" => ("Stretch at break", f.4.to_string()),
        _ => ("Result", result),
    };
    let input = format!("{}{fibre} rope", d.map(|d| format!("{} mm ", fmt_float((d * 10.0).round() / 10.0, 4))).unwrap_or_default());
    Some(Answer::new("rope", vec![
        pod("Input", "Input interpretation", text(match prop { Some(p) => format!("{input} | {p}"), None => input })),
        primary(pod("Result", title, text(res))),
        pod("Table", "Properties", table(None, rows)),
        pod("Notes", "Notes", text(f.5)),
        pod("Assumptions", "Assumptions", text("typical new 3-strand laid rope; breaking strength varies by maker, construction and condition, so use the manufacturer's rating for load-bearing work")),
    ]))
}
