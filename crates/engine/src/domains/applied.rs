//! Applied earth science and materials calculators: the 1976 US Standard
//! Atmosphere at an altitude or on a mountain top, the layers of the
//! atmosphere, horizon and radio horizon distances, raindrop terminal
//! speeds and Marshall–Palmer drop sizes, the Earth's interior at a depth,
//! ocean pressure at a depth, global warming potentials, cloud base
//! heights, indentation hardness formulas, and log-scale board feet.

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
    fmt_float(v, 6)
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn rows(r: Vec<(&str, String)>) -> Content {
    table(None, r.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())
}

fn get(l: &str, re: &Regex) -> Option<f64> {
    re.captures(l).and_then(|m| m[1].replace(',', "").parse().ok())
}

/// A length in metres from "40000ft", "30,000 feet", "8.6 km", "2 miles".
fn length_m(s: &str) -> Option<f64> {
    let m = re!(r"([\d,]*\.?\d+)\s*(feet|foot|ft|'|meters?|metres?|m|kilometers?|kilometres?|km|miles?|mi)\b").captures(s)?;
    let v: f64 = m[1].replace(',', "").parse().ok()?;
    Some(v * match &m[2] {
        "feet" | "foot" | "ft" | "'" => 0.3048,
        u if u.starts_with('k') => 1000.0,
        u if u.starts_with("mi") => 1609.344,
        _ => 1.0,
    })
}

const PEAKS: &[(&str, f64)] = &[
    ("mount everest", 8848.86), ("everest", 8848.86), ("k2", 8611.0), ("kangchenjunga", 8586.0), ("lhotse", 8516.0), ("makalu", 8485.0),
    ("aconcagua", 6961.0), ("denali", 6190.0), ("mount mckinley", 6190.0), ("kilimanjaro", 5895.0), ("mount kilimanjaro", 5895.0),
    ("mount elbrus", 5642.0), ("elbrus", 5642.0), ("mont blanc", 4806.0), ("matterhorn", 4478.0), ("mount whitney", 4421.0),
    ("mount rainier", 4392.0), ("pikes peak", 4302.0), ("mount fuji", 3776.0), ("mount kosciuszko", 2228.0), ("mount washington", 1917.0),
    ("ben nevis", 1345.0), ("denver", 1609.0), ("la paz", 3640.0), ("mexico city", 2240.0),
];

fn peak(s: &str) -> Option<(&'static str, f64)> {
    let s = s.trim().trim_start_matches("the ").trim_start_matches("top of ").trim_start_matches("the top of ").trim_start_matches("summit of ").trim();
    PEAKS.iter().find(|(name, _)| *name == s).map(|(a, b)| (*a, *b))
}

/// US Standard Atmosphere 1976 below 86 km: (T K, P Pa, ρ kg/m³).
pub fn atmosphere(z: f64) -> Option<(f64, f64, f64)> {
    if !(-5000.0..=86000.0).contains(&z) {
        return None;
    }
    const R0: f64 = 6_356_766.0;
    const LAYERS: [(f64, f64, f64, f64); 7] = [
        (0.0, 288.15, -0.0065, 101_325.0),
        (11_000.0, 216.65, 0.0, 22_632.06),
        (20_000.0, 216.65, 0.001, 5_474.889),
        (32_000.0, 228.65, 0.0028, 868.0187),
        (47_000.0, 270.65, 0.0, 110.9063),
        (51_000.0, 270.65, -0.0028, 66.93887),
        (71_000.0, 214.65, -0.002, 3.956420),
    ];
    let h = R0 * z / (R0 + z);
    let (hb, tb, lr, pb) = *LAYERS.iter().rev().find(|l| h >= l.0).unwrap_or(&LAYERS[0]);
    let g0m_r = 9.80665 * 0.0289644 / 8.3144598;
    let t = tb + lr * (h - hb);
    let p = if lr == 0.0 { pb * (-g0m_r * (h - hb) / tb).exp() } else { pb * (tb / t).powf(g0m_r / lr) };
    Some((t, p, p / (287.053 * t)))
}

fn layer_name(z: f64) -> &'static str {
    if z < 12_000.0 {
        "troposphere"
    } else if z < 50_000.0 {
        "stratosphere"
    } else if z < 85_000.0 {
        "mesosphere"
    } else if z < 600_000.0 {
        "thermosphere"
    } else {
        "exosphere"
    }
}

fn standard_atmosphere(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    if l.contains("depth") || l.contains("below") || l.contains("underwater") || l.contains("ocean") || l.contains("water") {
        return None;
    }
    let m = re!(r"^(?:what is )?(?:the )?(standard atmosphere|atmosphere|air pressure|atmospheric pressure|barometric pressure|air temperature|temperature|air density|density of air|sound speed|speed of sound|pressure|boiling point of water|air|conditions)\s+(?:at|on top of|on the top of|on|of|for|in)\s+(?:an altitude of |altitude |the top of |the summit of )?(.+?)(?: altitude| elevation| above (?:the earth|sea level|ground)| high)?$")
        .captures(l)
        .map(|c| (c.get(1).map(|x| x.as_str().to_string()), c[2].to_string()))
        .or_else(|| re!(r"^(?:atmosphere|altitude)\s+([\d,.]+\s*(?:ft|feet|m|km|miles?))$").captures(l).map(|c| (Some("atmosphere".into()), c[1].to_string())))
        .or_else(|| re!(r"^([\d,.]+\s*(?:ft|feet|m|km|meters|miles?)) above (?:the earth|sea level|ground)$").captures(l).map(|c| (Some("atmosphere".into()), c[1].to_string())))?;
    let (what, place) = (m.0.unwrap_or_default(), m.1);
    if let Some(c) = re!(r"^(-?[\d.]+)\s*(?:°\s*|degrees? )?(c|f|k|celsius|fahrenheit|kelvin)$").captures(place.trim()) {
        if !matches!(what.as_str(), "sound speed" | "speed of sound" | "air density" | "density of air") {
            return None;
        }
        let v: f64 = c[1].parse().ok()?;
        let t = match &c[2][..1] { "f" => (v - 32.0) / 1.8 + 273.15, "k" => v, _ => v + 273.15 };
        if t <= 0.0 {
            return None;
        }
        let a = (1.4 * 287.053 * t).sqrt();
        let rho = 101_325.0 / (287.053 * t);
        let sound = what.contains("sound");
        return Some(Answer::new(if sound { "speed of sound" } else { "air density" }, vec![
            input(format!("{} in dry air at {} K ({} °C), 1 atm", if sound { "speed of sound" } else { "density" }, n(t), n(t - 273.15))),
            primary(pod("Result", if sound { "Speed of sound" } else { "Air density" }, text(if sound { format!("{} m/s = {} km/h = {} mph", n(a), n(a * 3.6), n(a / 0.44704)) } else { format!("{} kg/m³", n(rho)) }))),
            pod("Formula", "Formula", text(if sound { "c = √(γ R T), γ = 1.4, R = 287.05 J/(kg K)" } else { "ρ = p/(R T), p = 101.325 kPa, R = 287.05 J/(kg K)" })),
        ]));
    }
    if what == "boiling point of water" || what == "temperature" && peak(&place).is_none() && !place.contains("ft") && !place.contains("feet") && !place.contains("km") {
        return None;
    }
    let (z, label) = if let Some((name, h)) = peak(&place) {
        (h, format!("top of {name} ({} m)", n(h)))
    } else {
        let z = length_m(&place)?;
        if !re!(r"^[\d,.]+\s*\S+$").is_match(place.trim()) {
            return None;
        }
        (z, format!("altitude {} m ({} ft)", n(z), n(z / 0.3048)))
    };
    let (t, p, rho) = atmosphere(z)?;
    let a = (1.4 * 287.053 * t).sqrt();
    let mu = 1.458e-6 * t.powf(1.5) / (t + 110.4);
    let g = 9.80665 * (6_356_766.0f64 / (6_356_766.0 + z)).powi(2);
    let (title, head) = match what.as_str() {
        "sound speed" | "speed of sound" => ("Speed of sound", format!("{} m/s ({} km/h, {} mph)", n(a), n(a * 3.6), n(a / 0.44704))),
        "air density" | "density of air" => ("Air density", format!("{} kg/m³", n(rho))),
        "air temperature" | "temperature" => ("Temperature", format!("{} K = {} °C = {} °F", n(t), n(t - 273.15), n((t - 273.15) * 1.8 + 32.0))),
        _ => ("Pressure", format!("{} kPa = {} atm = {} inHg = {}% of sea level", n(p / 1000.0), n(p / 101_325.0), n(p / 3386.389), n(p / 1013.25))),
    };
    Some(Answer::new("standard atmosphere", vec![
        input(format!("US standard atmosphere 1976 | {label}")),
        primary(pod("Result", title, text(head))),
        pod("Properties", "Atmospheric properties", rows(vec![
            ("temperature", format!("{} K = {} °C = {} °F", n(t), n(t - 273.15), n((t - 273.15) * 1.8 + 32.0))),
            ("pressure", format!("{} kPa ({} mbar, {} atm)", n(p / 1000.0), n(p / 100.0), n(p / 101_325.0))),
            ("density", format!("{} kg/m³ ({}% of sea level)", n(rho), n(rho / 1.225 * 100.0))),
            ("speed of sound", format!("{} m/s", n(a))),
            ("dynamic viscosity", format!("{} Pa s", n(mu))),
            ("gravitational acceleration", format!("{} m/s²", n(g))),
            ("oxygen partial pressure", format!("{} kPa (sea level 21.2 kPa)", n(p * 0.2095 / 1000.0))),
            ("layer", layer_name(z).to_string()),
        ])),
    ]))
}

fn atmosphere_layers(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    const LAYERS: &[(&str, &str, &str)] = &[
        ("troposphere", "0 – 8 to 18 km (about 12 km on average)", "weather, clouds, 75% of the air mass; temperature falls about 6.5 °C per km"),
        ("stratosphere", "tropopause – about 50 km", "ozone layer; temperature rises with height; jet aircraft cruise in its lowest part"),
        ("mesosphere", "about 50 – 85 km", "coldest layer (down to about −90 °C); meteors burn up; noctilucent clouds"),
        ("thermosphere", "about 85 – 600 km", "aurorae, ionosphere, the International Space Station; temperatures over 1000 °C in thin gas"),
        ("exosphere", "above about 600 km to roughly 10,000 km", "atoms escape to space; merges into the solar wind"),
    ];
    if l == "atmospheric layers" || l == "layers of the atmosphere" || l == "atmosphere layers" || l == "layers of earth's atmosphere" {
        let r: Vec<Vec<Content>> = LAYERS.iter().map(|(a, b, c)| vec![text(*a), text(*b), text(*c)]).collect();
        return Some(Answer::new("atmospheric layers", vec![
            input("layers of the Earth's atmosphere"),
            primary(pod("Result", "Layers", table(Some(vec!["layer", "altitude", "features"]), r))),
        ]));
    }
    let (name, alt, feat) = LAYERS.iter().find(|x| x.0 == l)?;
    Some(Answer::new(*name, vec![
        input(format!("{name} (atmospheric layer)")),
        primary(pod("Result", "Altitude range", text(*alt))),
        pod("Features", "Features", text(*feat)),
    ]))
}

fn horizon(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let radio = l.contains("radio");
    if !(l.contains("horizon") || l.contains("how far can i see") || l.contains("line of sight")) {
        return None;
    }
    let h = re!(r"\bfrom\s+(?:an? )?(?:altitude of |height of )?(.+?)(?:\s+(?:altitude|high|tall|elevation))?$")
        .captures(l)
        .and_then(|c| peak(&c[1]).map(|p| p.1).or_else(|| length_m(&c[1])))
        .or_else(|| length_m(l))?;
    const R: f64 = 6_371_000.0;
    let geo = (2.0 * R * h + h * h).sqrt();
    let optical = (2.0 * R * 7.0 / 6.0 * h).sqrt();
    let radio_d = (2.0 * R * 4.0 / 3.0 * h).sqrt();
    let head = if radio { radio_d } else { geo };
    Some(Answer::new("horizon distance", vec![
        input(format!("{} from a height of {} m ({} ft)", if radio { "radio horizon" } else { "distance to the horizon" }, n(h), n(h / 0.3048))),
        primary(pod("Result", if radio { "Radio horizon (4/3 Earth radius)" } else { "Distance to the horizon (geometric)" }, text(format!("{} km = {} miles", n(head / 1000.0), n(head / 1609.344))))),
        pod("Details", "Details", rows(vec![
            ("geometric", format!("{} km", n(geo / 1000.0))),
            ("with typical optical refraction (k = 7/6)", format!("{} km", n(optical / 1000.0))),
            ("radio horizon (k = 4/3)", format!("{} km", n(radio_d / 1000.0))),
            ("dip of the horizon", format!("{}°", n((R / (R + h)).acos().to_degrees()))),
        ])),
        pod("Formula", "Formula", text("d = √(2 k R h + h²), R = 6371 km")),
    ]))
}

fn raindrops(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    if let Some(r) = get(l, re!(r"(?:drop size|drop diameter|raindrop size|rain drop size)\s+in\s+([\d.]+)\s*mm\s*/\s*(?:hr|hour|h)")) {
        let lam = 4.1 * r.powf(-0.21);
        return Some(Answer::new("raindrop size", vec![
            input(format!("raindrop size distribution | rain rate {} mm/h", n(r))),
            primary(pod("Result", "Median volume diameter", text(format!("{} mm", n(3.67 / lam))))),
            pod("Details", "Marshall–Palmer distribution N(D) = N₀ e^(−Λ D)", rows(vec![("Λ", format!("{} mm⁻¹", n(lam))), ("mean diameter", format!("{} mm", n(1.0 / lam))), ("N₀", "8000 m⁻³ mm⁻¹".into()), ("drops per cubic meter", n(8000.0 / lam))])),
        ]));
    }
    let d = get(l, re!(r"([\d.]+)\s*mm\b(?: diameter)?\s*(?:rain\s?drops?|drops? of rain)")).or_else(|| get(l, re!(r"(?:rain\s?drop|drop of rain)\D*([\d.]+)\s*mm")))?;
    if !(0.05..=8.0).contains(&d) {
        return None;
    }
    let v = (9.65 - 10.3 * (-0.6 * d).exp()).max(0.0);
    let vol = std::f64::consts::PI / 6.0 * d.powi(3);
    Some(Answer::new("raindrop", vec![
        input(format!("raindrop of diameter {} mm", n(d))),
        primary(pod("Result", "Terminal speed", text(format!("{} m/s = {} km/h = {} mph", n(v), n(v * 3.6), n(v / 0.44704))))),
        pod("Properties", "Properties", rows(vec![("volume", format!("{} mm³ = {} µL", n(vol), n(vol))), ("mass", format!("{} mg", n(vol))), ("shape", (if d < 1.0 { "nearly spherical" } else if d < 4.5 { "flattened at the bottom (hamburger bun)" } else { "unstable; tends to break up" }).to_string())])),
        pod("Method", "Method", text("Atlas, Srivastava and Sekhon (1973): v = 9.65 − 10.3 e^(−0.6 D) m/s with D in mm, sea-level air")),
    ]))
}

fn earth_interior(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^(.+?)\s+(?:below the (?:surface of the )?earth|below the earth's surface|deep in the earth|underground|into the earth)$").captures(l)?;
    let d = length_m(&m[1])? / 1000.0;
    if !(0.0..=6371.0).contains(&d) {
        return None;
    }
    // PREM boundaries and densities (kg/m³) at the top and bottom of each layer
    const L: &[(f64, f64, &str, f64, f64)] = &[
        (0.0, 35.0, "crust (continental average)", 2700.0, 2900.0),
        (35.0, 410.0, "upper mantle", 3380.0, 3540.0),
        (410.0, 660.0, "mantle transition zone", 3720.0, 3990.0),
        (660.0, 2891.0, "lower mantle", 4380.0, 5570.0),
        (2891.0, 5150.0, "outer core (liquid iron–nickel)", 9900.0, 12170.0),
        (5150.0, 6371.0, "inner core (solid iron–nickel)", 12760.0, 13090.0),
    ];
    let (a, b, name, ra, rb) = *L.iter().find(|x| d <= x.1)?;
    let rho = ra + (rb - ra) * (d - a) / (b - a);
    let r: Vec<Vec<Content>> = L.iter().map(|x| vec![text(x.2), text(format!("{} – {} km", x.0, x.1))]).collect();
    Some(Answer::new("Earth's interior", vec![
        input(format!("{} km below the Earth's surface", n(d))),
        primary(pod("Result", "Layer", text(name))),
        pod("Properties", "Properties (PREM)", rows(vec![("distance from the center", format!("{} km", n(6371.0 - d))), ("fraction of the radius", n((6371.0 - d) / 6371.0)), ("approximate density", format!("{} kg/m³", n(rho)))])),
        pod("Layers", "Layers", table(Some(vec!["layer", "depth"]), r)),
    ]))
}

fn ocean_depth(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^(?:ocean|sea|seawater|water pressure|pressure)\s+(?:at\s+)?(?:a depth of\s+)?([\d,.]+\s*(?:m|meters?|metres?|ft|feet|km))(?:\s+(?:depth|deep|below sea level|underwater))?$").captures(l)?;
    if !(l.starts_with("ocean") || l.starts_with("sea") || l.contains("depth") || l.contains("deep") || l.contains("underwater")) {
        return None;
    }
    let d = length_m(&m[1])?;
    let gauge = 1025.0 * 9.80665 * d;
    let abs = gauge + 101_325.0;
    Some(Answer::new("ocean pressure", vec![
        input(format!("ocean at a depth of {} m ({} ft)", n(d), n(d / 0.3048))),
        primary(pod("Result", "Absolute pressure", text(format!("{} MPa = {} atm = {} psi", n(abs / 1e6), n(abs / 101_325.0), n(abs / 6894.757))))),
        pod("Details", "Details", rows(vec![
            ("gauge pressure", format!("{} MPa ({} atm)", n(gauge / 1e6), n(gauge / 101_325.0))),
            ("zone", (if d < 200.0 { "epipelagic (sunlight zone)" } else if d < 1000.0 { "mesopelagic (twilight zone)" } else if d < 4000.0 { "bathypelagic (midnight zone)" } else if d < 6000.0 { "abyssopelagic" } else { "hadal zone (trenches)" }).to_string()),
            ("compression of air volume (Boyle)", format!("{} of surface volume", n(101_325.0 / abs))),
        ])),
        pod("Assumptions", "Assumptions", text("seawater density 1025 kg/m³, g = 9.80665 m/s²")),
    ]))
}

/// IPCC AR6 global warming potentials (20 and 100 years).
const GWP: &[(&[&str], &str, f64, f64, &str)] = &[
    (&["carbon dioxide", "co2"], "carbon dioxide", 1.0, 1.0, "lifetime: multiple processes (centuries to millennia)"),
    (&["methane", "ch4"], "methane", 81.2, 27.9, "lifetime 11.8 years; fossil methane: 82.5 (20 yr), 29.8 (100 yr)"),
    (&["nitrous oxide", "n2o"], "nitrous oxide", 273.0, 273.0, "lifetime 109 years"),
    (&["sulfur hexafluoride", "sf6"], "sulfur hexafluoride", 18300.0, 25200.0, "lifetime 3200 years"),
    (&["nitrogen trifluoride", "nf3"], "nitrogen trifluoride", 13400.0, 17400.0, "lifetime 569 years"),
    (&["hfc-134a", "r-134a", "tetrafluoroethane"], "HFC-134a", 4140.0, 1530.0, "lifetime 14 years"),
    (&["hfc-23", "fluoroform", "trifluoromethane"], "HFC-23", 12400.0, 14600.0, "lifetime 228 years"),
    (&["cf4", "tetrafluoromethane", "carbon tetrafluoride"], "tetrafluoromethane (PFC-14)", 5300.0, 7380.0, "lifetime 50,000 years"),
];

fn gwp(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^(?:global warming potential|gwp)(?: of)? (.+)$").captures(l).or_else(|| re!(r"^(.+?) (?:global warming potential|gwp)$").captures(l))?;
    let (_, name, g20, g100, note) = GWP.iter().find(|x| x.0.contains(&m[1].trim()))?;
    Some(Answer::new("global warming potential", vec![
        input(format!("global warming potential of {name}")),
        primary(pod("Result", "Global warming potential (100-year)", text(n(*g100)))),
        pod("Details", "Details (IPCC AR6, relative to CO₂ by mass)", rows(vec![("20-year GWP", n(*g20)), ("100-year GWP", n(*g100)), ("note", note.to_string())])),
    ]))
}

fn cloud_base(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    if !l.starts_with("cloud base") {
        return None;
    }
    let t = get(l, re!(r"(?:temperature|t)\s*(?:=|of)?\s*(-?[\d.]+)"));
    let td = get(l, re!(r"(?:dew ?point|td)\s*(?:=|of)?\s*(-?[\d.]+)"));
    let (Some(t), Some(td)) = (t, td) else {
        if l == "cloud base height" || l == "cloud base" {
            return Some(Answer::new("cloud base height", vec![input("cloud base height formula"), primary(pod("Formula", "Formula", text("h ≈ 125 m/°C × (T − T_d), with T the surface temperature and T_d the dew point"))), pod("Note", "Note", text("From the dry adiabatic lapse rate (9.8 °C/km) minus the dew-point lapse rate (about 1.8 °C/km)."))]));
        }
        return None;
    };
    let fahrenheit = l.contains('f') && !l.contains("°c");
    let spread_c = if fahrenheit { (t - td) / 1.8 } else { t - td };
    let h = 125.0 * spread_c;
    Some(Answer::new("cloud base height", vec![
        input(format!("cloud base | temperature {} | dew point {}", n(t), n(td))),
        primary(pod("Result", "Cloud base height", text(format!("{} m = {} ft above the ground", n(h), n(h / 0.3048))))),
        pod("Formula", "Formula", text("h ≈ 125 m/°C × (T − T_d)")),
    ]))
}

fn hardness(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    let force = |l: &str| -> Option<f64> {
        let m = re!(r"([\d.]+)\s*(kgf|gf|n|kn)\b").captures(l)?;
        let v: f64 = m[1].parse().ok()?;
        Some(match &m[2] { "gf" => v / 1000.0, "n" => v / 9.80665, "kn" => v * 1000.0 / 9.80665, _ => v })
    };
    let mms: Vec<f64> = re!(r"([\d.]+)\s*mm\b").captures_iter(l).filter_map(|c| c[1].parse().ok()).collect();
    if l.starts_with("brinell") {
        let f = force(l)?;
        let (dd, d) = (*mms.first()?, *mms.get(1)?);
        let hb = 2.0 * f / (std::f64::consts::PI * dd * (dd - (dd * dd - d * d).sqrt()));
        return Some(Answer::new("Brinell hardness", vec![
            input(format!("Brinell hardness | load {} kgf | ball diameter {} mm | indentation diameter {} mm", n(f), n(dd), n(d))),
            primary(pod("Result", "Brinell hardness number", text(format!("{} HBW", n(hb))))),
            pod("Formula", "Formula", text("HB = 2F/(π D (D − √(D² − d²)))")),
        ]));
    }
    if l.starts_with("vickers") {
        let f = force(l)?;
        let d = *mms.first()?;
        let hv = 1.854_4 * f / (d * d);
        return Some(Answer::new("Vickers hardness", vec![
            input(format!("Vickers hardness | load {} kgf | mean indentation diagonal {} mm", n(f), n(d))),
            primary(pod("Result", "Vickers hardness number", text(format!("{} HV", n(hv))))),
            pod("Formula", "Formula", text("HV = 2F sin(136°/2)/d² ≈ 1.8544 F/d²")),
        ]));
    }
    if l.starts_with("knoop") {
        let f = force(l)?;
        let len = *mms.first()?;
        let cp = get(l, re!(r"cp\s*=\s*([\d.]+)")).unwrap_or(0.070_279);
        let hk = f / (cp * len * len);
        return Some(Answer::new("Knoop hardness", vec![
            input(format!("Knoop hardness | load {} kgf | long diagonal {} mm | Cp = {}", n(f), n(len), n(cp))),
            primary(pod("Result", "Knoop hardness number", text(format!("{} HK", n(hk))))),
            pod("Formula", "Formula", text("HK = F/(C_p L²), C_p = 0.070279 for the standard indenter")),
        ]));
    }
    None
}

fn board_feet(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    if !re!(r"^(?:lumber|log|board feet|board foot|timber)(?: volume| scale)?").is_match(l) || !l.contains("diameter") {
        return None;
    }
    let to_in = |c: &regex::Captures| -> Option<f64> {
        let v: f64 = c[1].parse().ok()?;
        Some(match &c[2] { "ft" | "feet" | "foot" => v * 12.0, "cm" => v / 2.54, "m" => v / 0.0254, _ => v })
    };
    let dia = re!(r"([\d.]+)\s*(ft|feet|foot|in|inches|inch|cm|m)\b\s*(?:in )?diameter").captures(l).and_then(|c| to_in(&c))?;
    let len_ft = re!(r"([\d.]+)\s*(ft|feet|foot|in|inches|inch|cm|m)\b\s*(?:in )?length|length\s*(?:of )?([\d.]+)").captures(l).and_then(|c| if c.get(3).is_some() { c[3].parse().ok() } else { to_in(&c).map(|x| x / 12.0) })
        .or_else(|| re!(r"([\d.]+)\s*(ft|feet|foot|m)\b\s*(?:long|length)").captures(l).and_then(|c| to_in(&c).map(|x| x / 12.0)))?;
    let doyle = (dia - 4.0).powi(2) * len_ft / 16.0;
    let scribner = (0.79 * dia * dia - 2.0 * dia - 4.0) * len_ft / 16.0;
    // International ¼-inch rule, half-inch taper per 4-foot section
    let mut intl = 0.0;
    let mut d = dia;
    let mut rem = len_ft;
    while rem >= 4.0 - 1e-9 {
        intl += 0.22 * d * d - 0.71 * d;
        d += 0.5;
        rem -= 4.0;
    }
    if rem > 0.0 {
        intl += (0.22 * d * d - 0.71 * d) * rem / 4.0;
    }
    intl *= 0.904_762;
    let cyl = std::f64::consts::PI / 4.0 * (dia / 12.0).powi(2) * len_ft;
    Some(Answer::new("log volume", vec![
        input(format!("log | small-end diameter {} in | length {} ft", n(dia), n(len_ft))),
        primary(pod("Result", "Doyle log rule", text(format!("{} board feet", n(doyle.round()))))),
        pod("Rules", "Other log rules", rows(vec![("Scribner (formula)", format!("{} board feet", n(scribner.round()))), ("International ¼-inch", format!("{} board feet", n(intl.round()))), ("solid cylinder volume", format!("{} ft³ = {} board feet (no sawing loss)", n(cyl), n((cyl * 12.0).round())))])),
        pod("Formula", "Doyle rule", text("BF = (D − 4)² L/16, D small-end diameter in inches, L length in feet")),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    standard_atmosphere(q)
        .or_else(|| atmosphere_layers(q))
        .or_else(|| horizon(q))
        .or_else(|| raindrops(q))
        .or_else(|| earth_interior(q))
        .or_else(|| ocean_depth(q))
        .or_else(|| gwp(q))
        .or_else(|| cloud_base(q))
        .or_else(|| hardness(q))
        .or_else(|| board_feet(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_atmosphere_values() {
        let (t, p, rho) = atmosphere(11_000.0).unwrap();
        assert!((t - 216.77).abs() < 0.1 && (p - 22_700.0).abs() < 100.0 && (rho - 0.3648).abs() < 0.002, "{t} {p} {rho}");
        let (t, p, _) = atmosphere(0.0).unwrap();
        assert!((t - 288.15).abs() < 1e-9 && (p - 101_325.0).abs() < 1e-6);
    }
}
