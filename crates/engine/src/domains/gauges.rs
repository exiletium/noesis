//! Industrial size standards: wire gauges (AWG, SWG) with the mass and
//! resistance of a length of wire, sheet steel gauges, letter drills,
//! hypodermic needle gauges, sieve mesh sizes, nominal pipe sizes with the
//! mass of a run of pipe, oven gas marks, shipping containers, batteries
//! and ring sizes.

use super::chem::find_element;
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

fn n(v: f64) -> String {
    fmt_float(v, 5)
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn inch_mm(inch: f64) -> String {
    format!("{} in ({} mm)", n(inch), n(inch * 25.4))
}

/// Imperial Standard Wire Gauge, diameters in inches, from 7/0 (index 0) to 50.
const SWG: &[(&str, f64)] = &[
    ("7/0", 0.500), ("6/0", 0.464), ("5/0", 0.432), ("4/0", 0.400), ("3/0", 0.372), ("2/0", 0.348), ("0", 0.324), ("1", 0.300), ("2", 0.276), ("3", 0.252), ("4", 0.232), ("5", 0.212), ("6", 0.192), ("7", 0.176),
    ("8", 0.160), ("9", 0.144), ("10", 0.128), ("11", 0.116), ("12", 0.104), ("13", 0.092), ("14", 0.080), ("15", 0.072), ("16", 0.064), ("17", 0.056), ("18", 0.048), ("19", 0.040), ("20", 0.036), ("21", 0.032),
    ("22", 0.028), ("23", 0.024), ("24", 0.022), ("25", 0.020), ("26", 0.018), ("27", 0.0164), ("28", 0.0148), ("29", 0.0136), ("30", 0.0124), ("31", 0.0116), ("32", 0.0108), ("33", 0.0100), ("34", 0.0092), ("35", 0.0084),
    ("36", 0.0076), ("37", 0.0068), ("38", 0.0060), ("39", 0.0052), ("40", 0.0048), ("41", 0.0044), ("42", 0.0040), ("43", 0.0036), ("44", 0.0032), ("45", 0.0028), ("46", 0.0024), ("47", 0.0020), ("48", 0.0016), ("49", 0.0012), ("50", 0.0010),
];

/// American Wire Gauge diameter in inches (n = −3 for 0000 … 40).
fn awg(gauge: i32) -> f64 {
    0.005 * 92f64.powf((36 - gauge) as f64 / 39.0)
}

fn awg_index(s: &str) -> Option<i32> {
    Some(match s {
        "0000" | "4/0" => -3,
        "000" | "3/0" => -2,
        "00" | "2/0" => -1,
        _ => s.parse::<i32>().ok().filter(|g| (0..=40).contains(g))?,
    })
}

/// Manufacturers' Standard Gauge for sheet steel, thickness in inches.
const SHEET_STEEL: &[(u32, f64)] = &[
    (3, 0.2391), (4, 0.2242), (5, 0.2092), (6, 0.1943), (7, 0.1793), (8, 0.1644), (9, 0.1495), (10, 0.1345), (11, 0.1196), (12, 0.1046), (13, 0.0897), (14, 0.0747), (15, 0.0673), (16, 0.0598),
    (17, 0.0538), (18, 0.0478), (19, 0.0418), (20, 0.0359), (21, 0.0329), (22, 0.0299), (23, 0.0269), (24, 0.0239), (25, 0.0209), (26, 0.0179), (27, 0.0164), (28, 0.0149), (29, 0.0135), (30, 0.0120),
];

/// Letter drill sizes, diameters in inches.
const LETTER_DRILLS: &[(char, f64)] = &[
    ('A', 0.234), ('B', 0.238), ('C', 0.242), ('D', 0.246), ('E', 0.250), ('F', 0.257), ('G', 0.261), ('H', 0.266), ('I', 0.272), ('J', 0.277), ('K', 0.281), ('L', 0.290), ('M', 0.295),
    ('N', 0.302), ('O', 0.316), ('P', 0.323), ('Q', 0.332), ('R', 0.339), ('S', 0.348), ('T', 0.358), ('U', 0.368), ('V', 0.377), ('W', 0.386), ('X', 0.397), ('Y', 0.404), ('Z', 0.413),
];

/// Hypodermic needle gauges (Birmingham): outer diameter mm, nominal inner diameter mm, typical uses.
const NEEDLES: &[(u32, f64, f64, &str)] = &[
    (10, 3.404, 2.693, "large-bore veterinary and industrial dispensing"),
    (12, 2.769, 2.159, "veterinary use, dispensing viscous fluids"),
    (14, 2.108, 1.600, "rapid infusion and trauma IV lines, large-volume transfusion"),
    (16, 1.651, 1.194, "rapid IV infusion, blood donation and transfusion, surgery"),
    (18, 1.270, 0.838, "drawing up medication, blood transfusion, rapid IV infusion"),
    (20, 0.9081, 0.603, "IV infusion in adults, blood draws"),
    (21, 0.8192, 0.514, "standard venipuncture (blood draws), intramuscular injection"),
    (22, 0.7176, 0.413, "intramuscular injection, IV in children and older adults, venipuncture of small veins"),
    (23, 0.6414, 0.337, "intramuscular and subcutaneous injection, blood draws from small or fragile veins (butterfly needles)"),
    (24, 0.5652, 0.311, "IV in infants, subcutaneous injection"),
    (25, 0.5144, 0.260, "subcutaneous and intradermal injection, local anesthesia, vaccinations"),
    (26, 0.4636, 0.260, "intradermal injection (tuberculin tests), subcutaneous injection"),
    (27, 0.4128, 0.210, "intradermal and subcutaneous injection, dental anesthesia, ophthalmic injection"),
    (28, 0.3620, 0.184, "insulin injection"),
    (29, 0.3366, 0.184, "insulin injection"),
    (30, 0.3112, 0.159, "insulin pens, cosmetic injections, intravitreal injection"),
    (31, 0.2604, 0.133, "insulin pens"),
    (32, 0.2350, 0.108, "insulin pens, acupuncture"),
    (33, 0.2096, 0.108, "dental and cosmetic microinjection"),
    (34, 0.1842, 0.0826, "microinjection, laboratory use"),
];

/// US standard sieve (ASTM E11) mesh numbers and openings in micrometres.
const SIEVES: &[(u32, f64)] = &[
    (4, 4750.0), (5, 4000.0), (6, 3350.0), (7, 2800.0), (8, 2360.0), (10, 2000.0), (12, 1700.0), (14, 1400.0), (16, 1180.0), (18, 1000.0), (20, 850.0), (25, 710.0), (30, 600.0), (35, 500.0), (40, 425.0), (45, 355.0),
    (50, 300.0), (60, 250.0), (70, 212.0), (80, 180.0), (100, 150.0), (120, 125.0), (140, 106.0), (170, 90.0), (200, 75.0), (230, 63.0), (270, 53.0), (325, 45.0), (400, 38.0), (450, 32.0), (500, 25.0), (635, 20.0),
];

/// Nominal pipe sizes: (NPS, outer diameter in, schedule 40 wall in, schedule 80 wall in).
const PIPES: &[(&str, f64, f64, f64, f64)] = &[
    ("1/8", 0.125, 0.405, 0.068, 0.095), ("1/4", 0.25, 0.540, 0.088, 0.119), ("3/8", 0.375, 0.675, 0.091, 0.126), ("1/2", 0.5, 0.840, 0.109, 0.147), ("3/4", 0.75, 1.050, 0.113, 0.154),
    ("1", 1.0, 1.315, 0.133, 0.179), ("1 1/4", 1.25, 1.660, 0.140, 0.191), ("1 1/2", 1.5, 1.900, 0.145, 0.200), ("2", 2.0, 2.375, 0.154, 0.218), ("2 1/2", 2.5, 2.875, 0.203, 0.276),
    ("3", 3.0, 3.500, 0.216, 0.300), ("3 1/2", 3.5, 4.000, 0.226, 0.318), ("4", 4.0, 4.500, 0.237, 0.337), ("5", 5.0, 5.563, 0.258, 0.375), ("6", 6.0, 6.625, 0.280, 0.432),
    ("8", 8.0, 8.625, 0.322, 0.500), ("10", 10.0, 10.750, 0.365, 0.594), ("12", 12.0, 12.750, 0.406, 0.688),
];

/// Resistivity at 20 °C (Ω m) and density (g/cm³) of wire and pipe materials.
fn material(name: &str) -> Option<(&'static str, Option<f64>, f64)> {
    let l = name.trim().to_lowercase();
    let table: &[(&[&str], &str, Option<f64>, f64)] = &[
        (&["copper", "cu"], "copper", Some(1.68e-8), 8.96),
        (&["silver", "ag"], "silver", Some(1.59e-8), 10.49),
        (&["gold", "au"], "gold", Some(2.44e-8), 19.30),
        (&["aluminum", "aluminium", "al"], "aluminium", Some(2.65e-8), 2.70),
        (&["tungsten", "w"], "tungsten", Some(5.60e-8), 19.25),
        (&["zinc", "zn"], "zinc", Some(5.90e-8), 7.14),
        (&["nickel", "ni"], "nickel", Some(6.99e-8), 8.908),
        (&["iron", "fe"], "iron", Some(9.71e-8), 7.874),
        (&["platinum", "pt"], "platinum", Some(1.06e-7), 21.45),
        (&["tin", "sn"], "tin", Some(1.09e-7), 7.31),
        (&["lead", "pb"], "lead", Some(2.20e-7), 11.34),
        (&["titanium", "ti"], "titanium", Some(4.20e-7), 4.506),
        (&["brass"], "brass", Some(6.0e-8), 8.5),
        (&["steel", "carbon steel", "mild steel"], "carbon steel", Some(1.43e-7), 7.85),
        (&["stainless steel", "stainless"], "stainless steel", Some(6.9e-7), 8.0),
        (&["nichrome"], "nichrome", Some(1.10e-6), 8.4),
        (&["constantan"], "constantan", Some(4.9e-7), 8.9),
        (&["pvc"], "PVC", None, 1.38),
    ];
    if let Some(r) = table.iter().find(|r| r.0.contains(&l.as_str())) {
        return Some((r.1, r.2, r.3));
    }
    let e = find_element(&l)?;
    Some((Box::leak(e.name.to_lowercase().into_boxed_str()), None, e.density_g_cm3?))
}

fn length_m(v: f64, unit: &str) -> Option<f64> {
    Some(v * match unit.trim().to_lowercase().as_str() {
        "ft" | "feet" | "foot" | "'" => 0.3048,
        "m" | "meter" | "meters" | "metre" | "metres" => 1.0,
        "in" | "inch" | "inches" => 0.0254,
        "yd" | "yard" | "yards" => 0.9144,
        "cm" => 0.01,
        "mm" => 0.001,
        "km" => 1000.0,
        "mi" | "mile" | "miles" => 1609.344,
        _ => return None,
    })
}

fn wire(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(?:the )?diameter of )?(awg|swg|american wire gauge|standard wire gauge|gauge)\s*#?\s*(\d+/0|0+|\d+)(?: wire)?(?:\s+([a-z ]+?))?(?:\s+(\d+(?:\.\d+)?)\s*(ft|feet|foot|m|meters?|metres?|in|inches|yd|yards?|cm|mm|km|mi|miles?))?$").captures(l).or_else(|| re!(r"^(\d+/0|0+|\d+)\s*-?\s*(awg|swg|gauge)(?: wire)?(?:\s+([a-z ]+?))?(?:\s+(\d+(?:\.\d+)?)\s*(ft|feet|foot|m|meters?|metres?|in|inches|yd|yards?|cm|mm|km|mi|miles?))?$").captures(l).map(|c| c))?;
    let (sys, g) = if m[1].chars().next()?.is_ascii_digit() { (m[2].to_string(), m[1].to_string()) } else { (m[1].to_string(), m[2].to_string()) };
    let swg = sys.contains("swg") || sys.contains("standard");
    let (name, d_in) = if swg {
        let g2 = match g.as_str() {
            "00" => "2/0".to_string(),
            "000" => "3/0".to_string(),
            "0000" => "4/0".to_string(),
            x => x.to_string(),
        };
        ("SWG", SWG.iter().find(|(k, _)| *k == g2)?.1)
    } else {
        if sys == "gauge" && m.get(3).is_none() {
            return None;
        }
        ("AWG", awg(awg_index(&g)?))
    };
    let d_m = d_in * 0.0254;
    let area = std::f64::consts::PI * d_m * d_m / 4.0;
    let mut rows = vec![
        vec![text("diameter"), text(inch_mm(d_in))],
        vec![text("cross-sectional area"), text(format!("{} mm² ({} kcmil)", n(area * 1e6), n(d_in * d_in * 1000.0)))],
    ];
    let mut pods = vec![input(format!("{name} {g} wire"))];
    let mat = m.get(3).and_then(|x| material(x.as_str().trim_end_matches(" wire").trim()));
    if sys == "gauge" && mat.is_none() {
        return None;
    }
    if let Some((mname, rho, dens)) = mat {
        rows.push(vec![text("material"), text(mname)]);
        let per_m_mass = area * dens * 1000.0; // kg per metre
        if let Some(r) = rho {
            rows.push(vec![text("resistance per length"), text(format!("{} Ω/km ({} Ω/1000 ft)", n(r / area * 1000.0), n(r / area * 304.8)))]);
        }
        rows.push(vec![text("mass per length"), text(format!("{} kg/km ({} lb/1000 ft)", n(per_m_mass * 1000.0), n(per_m_mass * 304.8 / 0.45359237)))]);
        if let (Some(v), Some(u)) = (m.get(4), m.get(5)) {
            let len = length_m(v.as_str().parse().ok()?, u.as_str())?;
            let mass = per_m_mass * len;
            let mut main = format!("mass {} kg ({} lb)", n(mass), n(mass / 0.45359237));
            if let Some(r) = rho {
                main.push_str(&format!(", resistance {} Ω", n(r * len / area)));
            }
            pods.push(primary(pod("Result", format!("{} {} of {name} {g} {mname} wire", v.as_str(), u.as_str()).as_str(), text(main))));
        }
    }
    let has_primary = pods.iter().any(|p| p.primary);
    pods.push(pod("Properties", "Wire properties", table(None, rows)));
    if !has_primary {
        let last = pods.len() - 1;
        pods[last].primary = true;
    }
    if !swg {
        pods.push(pod("Formula", "AWG formula", text("d = 0.005 in × 92^((36 − n)/39)")));
    }
    Some(Answer::new(format!("{name} {g}"), pods))
}

fn sheet(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(?:the )?thickness of )?(?:(?:sheet )?steel )?(?:sheet metal )?gauge\s*#?\s*(\d+)(?: (?:sheet )?steel)?(?: thickness)?(?: in (mm|inches|in|cm))?$").captures(l)?;
    if !l.contains("steel") && !l.contains("sheet") && !l.contains("thickness") {
        return None;
    }
    let g: u32 = m[1].parse().ok()?;
    let t = SHEET_STEEL.iter().find(|(k, _)| *k == g)?.1;
    let main = match m.get(2).map(|x| x.as_str()) {
        Some("mm") => format!("{} mm", n(t * 25.4)),
        Some("cm") => format!("{} cm", n(t * 2.54)),
        _ => inch_mm(t),
    };
    let rows = vec![vec![text("thickness"), text(inch_mm(t))], vec![text("mass per area (steel)"), text(format!("{} kg/m² ({} lb/ft²)", n(t * 0.0254 * 7850.0), n(t * 0.0254 * 7850.0 / 4.882428)))]];
    Some(Answer::new("sheet steel gauge", vec![input(format!("{g} gauge sheet steel (Manufacturers' Standard Gauge)")), primary(pod("Result", "Thickness", text(main))), pod("Properties", "Properties", table(None, rows))]))
}

fn drill(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(?:the )?diameter of )?(?:an? )?(?:letter )?([a-z])(?: size)? drill(?: bit)?(?: size)?$|^(?:letter )?drill (?:size |bit )?([a-z])$").captures(l)?;
    let c = m.get(1).or(m.get(2))?.as_str().chars().next()?.to_ascii_uppercase();
    let d = LETTER_DRILLS.iter().find(|(k, _)| *k == c)?.1;
    Some(Answer::new("letter drill", vec![input(format!("letter drill size {c}")), primary(pod("Result", "Diameter", text(inch_mm(d))))]))
}

fn needle(l: &str) -> Option<Answer> {
    let m = re!(r"(?:#|number )?(\d+)(?:\s*-?\s*(?:gauge|g|ga))?\s+(?:hypodermic |syringe |medical )?needle|needle (?:gauge|size)\s*#?\s*(\d+)").captures(l)?;
    let g: u32 = m.get(1).or(m.get(2))?.as_str().parse().ok()?;
    let (_, od, id, uses) = *NEEDLES.iter().find(|r| r.0 == g)?;
    let rows = vec![vec![text("outer diameter"), text(format!("{} mm ({} in)", n(od), n(od / 25.4)))], vec![text("nominal inner diameter"), text(format!("{} mm", n(id)))], vec![text("typical uses"), text(uses)]];
    let main = if l.contains("use") { uses.to_string() } else { format!("outer diameter {} mm", n(od)) };
    Some(Answer::new("needle gauge", vec![input(format!("{g} gauge hypodermic needle")), primary(pod("Result", if l.contains("use") { "Typical uses" } else { "Size" }, text(main))), pod("Properties", "Properties", table(None, rows))]))
}

fn sieve(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:us |astm )?(?:sieve|mesh)(?: size| number| no\.?)?\s*#?\s*(\d+)(?: (?:sieve|mesh))?$|^(\d+) mesh(?: sieve)?$").captures(l)?;
    let s: u32 = m.get(1).or(m.get(2))?.as_str().parse().ok()?;
    let um = SIEVES.iter().find(|(k, _)| *k == s)?.1;
    let rows = vec![vec![text("opening"), text(format!("{} μm ({} mm, {} in)", n(um), n(um / 1000.0), n(um / 25400.0)))], vec![text("standard"), text("ASTM E11 (US standard sieve series)")]];
    Some(Answer::new("sieve size", vec![input(format!("US standard sieve number {s}")), primary(pod("Result", "Opening", text(format!("{} μm", n(um))))), pod("Properties", "Properties", table(None, rows))]))
}

fn parse_nps(s: &str) -> Option<&'static (&'static str, f64, f64, f64, f64)> {
    let s = s.trim().trim_end_matches('"').trim_end_matches(" inch").trim_end_matches(" in").trim();
    let s = s.replace('-', " ");
    let v: f64 = if let Some((a, b)) = s.split_once(' ') {
        let (p, q) = b.split_once('/')?;
        a.parse::<f64>().ok()? + p.parse::<f64>().ok()? / q.parse::<f64>().ok()?
    } else if let Some((p, q)) = s.split_once('/') {
        p.parse::<f64>().ok()? / q.parse::<f64>().ok()?
    } else {
        s.parse().ok()?
    };
    PIPES.iter().find(|r| (r.1 - v).abs() < 1e-9)
}

fn pipe(l: &str) -> Option<Answer> {
    let m = re!(r#"^(?:nps|nominal pipe size)\s*([\d/ .-]+?)(?:\s*(?:inch|in|"))?(?:\s+(?:schedule|sch)\s*(40|80))?(?:\s+([a-z ]+?))?(?:\s+(\d+(?:\.\d+)?)\s*(ft|feet|foot|m|meters?|metres?|yd|yards?))?$|^([\d/ .-]+?)\s*(?:inch|in|")\s+(?:schedule (40|80) )?(?:([a-z]+) )?pipe(?:\s+(\d+(?:\.\d+)?)\s*(ft|feet|foot|m|meters?|metres?))?$"#).captures(l)?;
    let size = m.get(1).or(m.get(6))?.as_str();
    let p = parse_nps(size)?;
    let sched80 = m.get(2).or(m.get(7)).map(|x| x.as_str() == "80").unwrap_or(false);
    let wall = if sched80 { p.4 } else { p.3 };
    let id = p.2 - 2.0 * wall;
    let sched = if sched80 { 80 } else { 40 };
    let mut rows = vec![
        vec![text("outer diameter"), text(inch_mm(p.2))],
        vec![text(format!("wall thickness (schedule {sched})")), text(inch_mm(wall))],
        vec![text("inner diameter"), text(inch_mm(id))],
        vec![text("flow area"), text(format!("{} in² ({} cm²)", n(std::f64::consts::PI * id * id / 4.0), n(std::f64::consts::PI * id * id / 4.0 * 6.4516)))],
    ];
    let mut pods = vec![input(format!("NPS {} schedule {sched} pipe", p.0))];
    let mat_name = m.get(3).or(m.get(8)).map(|x| x.as_str()).unwrap_or("steel");
    if let Some((mname, _, dens)) = material(mat_name) {
        let area_in2 = std::f64::consts::PI / 4.0 * (p.2 * p.2 - id * id);
        let kg_per_m = area_in2 * 6.4516e-4 * dens * 1000.0;
        rows.push(vec![text(format!("mass per length ({mname})")), text(format!("{} kg/m ({} lb/ft)", n(kg_per_m), n(kg_per_m * 0.3048 / 0.45359237)))]);
        if let (Some(v), Some(u)) = (m.get(4).or(m.get(9)), m.get(5).or(m.get(10))) {
            let len = length_m(v.as_str().parse().ok()?, u.as_str())?;
            let mass = kg_per_m * len;
            pods.push(primary(pod("Result", format!("Mass of {} {} of pipe", v.as_str(), u.as_str()).as_str(), text(format!("{} kg ({} lb)", n(mass), n(mass / 0.45359237))))));
        }
    }
    let has_primary = pods.iter().any(|p| p.primary);
    pods.push(pod("Properties", "Pipe dimensions", table(None, rows)));
    if !has_primary {
        let k = pods.len() - 1;
        pods[k].primary = true;
    }
    Some(Answer::new("pipe size", pods))
}

fn gas_mark(l: &str) -> Option<Answer> {
    let m = re!(r"gas mark\s*(\d+(?:/\d+|\.\d+)?)").captures(l)?;
    let g: f64 = match m[1].split_once('/') {
        Some((a, b)) => a.parse::<f64>().ok()? / b.parse::<f64>().ok()?,
        None => m[1].parse().ok()?,
    };
    // gas mark 1 = 275 °F, +25 °F per mark; ¼ = 225 °F, ½ = 250 °F
    let f = if g >= 1.0 { 250.0 + 25.0 * g } else if g >= 0.5 { 250.0 } else if g >= 0.25 { 225.0 } else { return None };
    let c = (f - 32.0) * 5.0 / 9.0;
    let k = c + 273.15;
    let main = if l.contains("kelvin") { format!("{} K", n(k)) } else { format!("{} °F ({} °C)", n(f), n(c)) };
    let rows = vec![vec![text("Fahrenheit"), text(format!("{} °F", n(f)))], vec![text("Celsius"), text(format!("{} °C (recipes round to {} °C)", n(c), (c / 10.0).round() * 10.0))], vec![text("kelvin"), text(format!("{} K", n(k)))]];
    Some(Answer::new("gas mark", vec![input(format!("gas mark {}", &m[1])), primary(pod("Result", "Oven temperature", text(main))), pod("Conversions", "Conversions", table(None, rows)), pod("Rule", "Rule", text("gas mark g ≥ 1: T = 250 °F + 25 °F × g"))]))
}

fn misc(l: &str) -> Option<Answer> {
    let t = l.trim_start_matches("a ").trim_start_matches("an ").trim_start_matches("the ");
    if let Some(m) = re!(r"^(20|40)[- ](?:foot|ft|')(?: (high cube|hc))? (?:shipping )?container$").captures(t) {
        let forty = &m[1] == "40";
        let hc = m.get(2).is_some();
        let (ext, int, vol, tare, gross) = match (forty, hc) {
            (false, _) => ("6.058 m × 2.438 m × 2.591 m (20 ft × 8 ft × 8 ft 6 in)", "5.898 m × 2.352 m × 2.393 m", 33.2, 2300.0, 30480.0),
            (true, false) => ("12.192 m × 2.438 m × 2.591 m (40 ft × 8 ft × 8 ft 6 in)", "12.032 m × 2.352 m × 2.393 m", 67.7, 3750.0, 30480.0),
            (true, true) => ("12.192 m × 2.438 m × 2.896 m (40 ft × 8 ft × 9 ft 6 in)", "12.032 m × 2.352 m × 2.698 m", 76.3, 3900.0, 30480.0),
        };
        let rows = vec![vec![text("external dimensions"), text(ext)], vec![text("internal dimensions (typical)"), text(int)], vec![text("capacity"), text(format!("{} m³ ({} ft³)", n(vol), n(vol * 35.3147)))], vec![text("tare mass (typical)"), text(format!("{} kg", n(tare)))], vec![text("maximum gross mass"), text(format!("{} kg", n(gross)))], vec![text("maximum payload"), text(format!("≈ {} kg", n(gross - tare)))], vec![text("equivalent units"), text(if forty { "2 TEU" } else { "1 TEU" })]];
        return Some(Answer::new("shipping container", vec![input(format!("{} ft {}ISO shipping container", &m[1], if hc { "high-cube " } else { "" })), primary(pod("Result", "Capacity", text(format!("{} m³", n(vol))))), pod("Properties", "Typical properties (ISO 668)", table(None, rows))]));
    }
    // watch cells: "voltage Energizer 341"
    if let Some(m) = re!(r"^(?:(voltage|nominal voltage) (?:of )?)?(?:an? )?energizer (341|357|377|371|364|392)(?: battery| cell)?(?: (voltage))?$").captures(t) {
        let (iec, dims) = match &m[2] {
            "341" => ("SR714SW", "7.9 mm × 1.4 mm"),
            "357" => ("SR44", "11.6 mm × 5.4 mm"),
            "377" => ("SR626SW", "6.8 mm × 2.6 mm"),
            "371" => ("SR920SW", "9.5 mm × 2.1 mm"),
            "364" => ("SR621SW", "6.8 mm × 2.1 mm"),
            _ => ("SR41", "7.9 mm × 3.6 mm"),
        };
        let rows = vec![vec![text("chemistry"), text("silver oxide")], vec![text("nominal voltage"), text("1.55 V")], vec![text("IEC designation"), text(iec)], vec![text("dimensions (diameter × height)"), text(dims)]];
        return Some(Answer::new("battery", vec![input(format!("Energizer {} | voltage", &m[2])), primary(pod("Result", "Nominal voltage", text("1.55 V"))), pod("Properties", "Properties", table(None, rows))]));
    }
    if let Some(m) = re!(r"^(?:alkaline |lithium |nimh )?(aaa|aa|c|d|9v|9 volt|cr2032|lr44) (?:battery|batteries|cell)$|^(?:alkaline |nimh )?battery (aaa|aa|c|d|9v)$").captures(t) {
        let k = m.get(1).or(m.get(2))?.as_str();
        let (size, v, cap, dims, mass) = match k {
            "aaa" => ("AAA (LR03)", "1.5 V", "≈ 1000–1200 mAh", "10.5 mm × 44.5 mm", "≈ 11.5 g"),
            "aa" => ("AA (LR6)", "1.5 V", "≈ 2500–3000 mAh", "14.5 mm × 50.5 mm", "≈ 23 g"),
            "c" => ("C (LR14)", "1.5 V", "≈ 6000–8000 mAh", "26.2 mm × 50 mm", "≈ 66 g"),
            "d" => ("D (LR20)", "1.5 V", "≈ 12000–18000 mAh", "34.2 mm × 61.5 mm", "≈ 140 g"),
            "9v" | "9 volt" => ("9-volt (6LR61)", "9 V", "≈ 550 mAh", "48.5 mm × 26.5 mm × 17.5 mm", "≈ 45 g"),
            "cr2032" => ("CR2032 (lithium coin cell)", "3 V", "≈ 225 mAh", "20 mm × 3.2 mm", "≈ 3 g"),
            _ => ("LR44 (alkaline button cell)", "1.5 V", "≈ 110–150 mAh", "11.6 mm × 5.4 mm", "≈ 2 g"),
        };
        let rows = vec![vec![text("nominal voltage"), text(v)], vec![text("capacity (alkaline, low drain)"), text(cap)], vec![text("dimensions (diameter × height)"), text(dims)], vec![text("mass"), text(mass)]];
        return Some(Answer::new("battery", vec![input(format!("{size} battery")), primary(pod("Result", "Nominal voltage", text(v))), pod("Properties", "Typical properties", table(None, rows))]));
    }
    // ring sizes: US size s → inner diameter 11.63 + 0.8128 s mm
    if let Some(m) = re!(r"^(?:(?:what is )?(?:the )?(french|uk|british|european|eu|iso|japanese|swiss) (?:equivalent |)(?:ring size )?(?:to|for|of) )?(?:us |american )?ring size (\d+(?:\.\d+)?)(?: in (french|uk|british|european|eu|iso|japanese|swiss|mm|millimeters))?$").captures(t) {
        let s: f64 = m[2].parse().ok()?;
        let d = 11.63 + 0.8128 * s;
        let circ = std::f64::consts::PI * d;
        let french = circ - 40.0;
        let rows = vec![vec![text("inner diameter"), text(format!("{} mm", n(d)))], vec![text("inner circumference"), text(format!("{} mm", n(circ)))], vec![text("ISO / European size"), text(n(circ))], vec![text("French size"), text(n(french))], vec![text("Swiss size"), text(n(circ - 40.0))]];
        let target = m.get(1).or(m.get(3)).map(|x| x.as_str()).unwrap_or("");
        let main = match target {
            "french" | "swiss" => format!("≈ {}", n((french * 4.0).round() / 4.0)),
            "european" | "eu" | "iso" => format!("≈ {}", n((circ * 2.0).round() / 2.0)),
            "mm" | "millimeters" => format!("{} mm inner diameter", n(d)),
            _ => format!("inner diameter {} mm", n(d)),
        };
        return Some(Answer::new("ring size", vec![input(format!("US ring size {}", &m[2])), primary(pod("Result", "Result", text(main))), pod("Sizes", "Equivalent sizes", table(None, rows)), pod("Note", "Note", text("US size s: inner diameter = 11.63 mm + 0.8128 mm × s; French size = circumference in mm − 40"))]));
    }
    None
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).replace('’', "'");
    let l = re!(r"^(?:what is |what are |find )?(?:the )?(?:typical uses of |uses of )?").replace(&l, "").to_string();
    let typical = q.lower.contains("use");
    let l = if typical && !l.contains("use") { format!("{l} uses") } else { l };
    let l = re!(r"\s+").replace_all(l.trim(), " ").to_string();
    wire(&l).or_else(|| sheet(&l)).or_else(|| drill(&l)).or_else(|| needle(&l)).or_else(|| sieve(&l)).or_else(|| pipe(&l)).or_else(|| gas_mark(&l)).or_else(|| misc(&l))
}
