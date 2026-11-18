//! Geodesy, the ocean and a little chemistry: positions given as latitude
//! and longitude, distances along routes through several cities, seawater
//! (UNESCO EOS-80 density at the surface, Mackenzie sound speed, freezing
//! point), pressure at altitude, error-correcting codes, chemical bonds,
//! functional groups and the cosmic origins of the elements.

use super::geo::{cities, distance_km, find_city, City};
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

fn rows(r: Vec<(&str, String)>) -> Content {
    table(None, r.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())
}

// ---------------------------------------------------------------------------
// coordinates
// ---------------------------------------------------------------------------

/// One coordinate: "55 deg 45' N", "23.5S", "55.75 N", "37°37′E", "-46.4".
fn coord(s: &str) -> Option<(f64, char)> {
    let s = s.trim().replace("degrees", "°").replace("degree", "°").replace(" deg", "°").replace("deg", "°");
    let c = re!(r#"^(-?\d+(?:\.\d+)?)\s*°?\s*(?:(\d+(?:\.\d+)?)\s*(?:'|′|m|min)\s*)?(?:(\d+(?:\.\d+)?)\s*(?:"|″|s|sec)\s*)?\s*([nsewNSEW])?$"#).captures(s.trim())?;
    let mut v: f64 = c[1].parse().ok()?;
    let sign = if v < 0.0 { -1.0 } else { 1.0 };
    v = v.abs();
    if let Some(m) = c.get(2) {
        v += m.as_str().parse::<f64>().ok()? / 60.0;
    }
    if let Some(sec) = c.get(3) {
        v += sec.as_str().parse::<f64>().ok()? / 3600.0;
    }
    let h = c.get(4).map(|h| h.as_str().to_ascii_uppercase().chars().next().unwrap()).unwrap_or('?');
    let v = match h {
        'S' | 'W' => -v,
        _ => v * sign,
    };
    Some((v, h))
}

/// "55 deg 45' N, 37 deg 37' E", "23.5S 46.4W"
fn position(s: &str) -> Option<(f64, f64)> {
    let s = s.trim();
    // split after the first hemisphere letter, or at a comma
    let (a, b) = if let Some(m) = re!(r"^(.+?[nsNS])\s*,?\s+(.+?[ewEW])$").captures(s) {
        (m[1].to_string(), m[2].to_string())
    } else {
        let (a, b) = s.split_once(',')?;
        (a.to_string(), b.to_string())
    };
    let (lat, h1) = coord(&a)?;
    let (lon, h2) = coord(&b)?;
    if matches!(h1, 'E' | 'W') || matches!(h2, 'N' | 'S') || lat.abs() > 90.0 || lon.abs() > 180.0 {
        return None;
    }
    // bare numbers need a comma and are taken as (latitude, longitude)
    if h1 == '?' && h2 == '?' && !s.contains(',') {
        return None;
    }
    Some((lat, lon))
}

fn dms(v: f64, pos: char, negc: char) -> String {
    let h = if v < 0.0 { negc } else { pos };
    let a = v.abs();
    let d = a.floor();
    let m = ((a - d) * 60.0).floor();
    let s = ((a - d) * 60.0 - m) * 60.0;
    format!("{}° {}′ {}″ {h}", d, m, fmt_float(s.round(), 3))
}

fn pseudo_city(lat: f64, lon: f64) -> City {
    City { name: String::new(), country: String::new(), lat, lon, population: 0, timezone: String::new(), capital: false, population_year: String::new(), admin1: String::new() }
}

fn nearest_city(lat: f64, lon: f64) -> Option<(&'static City, f64)> {
    let p = pseudo_city(lat, lon);
    cities().iter().filter(|c| c.population >= 100_000).map(|c| (c, distance_km(&p, c))).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
}

fn bearing(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (p1, p2) = (a.0.to_radians(), b.0.to_radians());
    let dl = (b.1 - a.1).to_radians();
    let y = dl.sin() * p2.cos();
    let x = p1.cos() * p2.sin() - p1.sin() * p2.cos() * dl.cos();
    (y.atan2(x).to_degrees() + 360.0) % 360.0
}

fn compass(deg: f64) -> &'static str {
    const D: [&str; 16] = ["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"];
    D[(((deg + 11.25) % 360.0) / 22.5) as usize]
}

fn coordinates(l: &str) -> Option<Answer> {
    // two positions: "23.5S 46.4W to 56.8N 60.6E"
    if let Some(m) = re!(r"^(?:distance\s+(?:from|between)\s+)?(.+?)\s+(?:to|and)\s+(.+)$").captures(l) {
        if let (Some(a), Some(b)) = (position(&m[1]), position(&m[2])) {
            let d = distance_km(&pseudo_city(a.0, a.1), &pseudo_city(b.0, b.1));
            let br = bearing(a, b);
            return Some(Answer::new("distance", vec![
                input(format!("distance from {}, {} to {}, {}", dms(a.0, 'N', 'S'), dms(a.1, 'E', 'W'), dms(b.0, 'N', 'S'), dms(b.1, 'E', 'W'))),
                primary(pod("Result", "Great-circle distance", text(format!("{} km  ({} miles, {} nautical miles)", n(d), n(d / 1.609_344), n(d / 1.852))))),
                pod("Bearing", "Initial bearing", text(format!("{}° ({})", n(br), compass(br)))),
                pod("Flight", "Flight time at 900 km/h", text(format!("about {} hours", fmt_float(d / 900.0, 3)))),
            ]));
        }
    }
    let (lat, lon) = position(l)?;
    let mut r = vec![("decimal", format!("{}, {}", n(lat), n(lon))), ("degrees, minutes, seconds", format!("{}, {}", dms(lat, 'N', 'S'), dms(lon, 'E', 'W')))];
    if let Some((c, d)) = nearest_city(lat, lon) {
        r.push(("nearest large city", format!("{} ({}), {} km away", c.name, c.country, n(d))));
    }
    let deg_lat = 111.132_954 - 0.559_822 * (2.0 * lat.to_radians()).cos() + 0.001_175 * (4.0 * lat.to_radians()).cos();
    let deg_lon = 111.412_84 * lat.to_radians().cos() - 0.093_5 * (3.0 * lat.to_radians()).cos();
    r.push(("one degree of latitude here", format!("{} km", n(deg_lat))));
    r.push(("one degree of longitude here", format!("{} km", n(deg_lon))));
    r.push(("solar time offset", format!("UTC{}{} h (mean solar time)", if lon >= 0.0 { "+" } else { "−" }, n(lon.abs() / 15.0))));
    Some(Answer::new("position", vec![input(format!("{}, {}", dms(lat, 'N', 'S'), dms(lon, 'E', 'W'))), primary(pod("Result", "Position", rows(r)))]))
}

fn latitude_line(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:latitude\s+)?(\d+(?:\.\d+)?)\s*(?:°|degrees?)?\s*(north|south)(?:\s+latitude)?$").captures(l)?;
    let lat: f64 = m[1].parse().ok()?;
    if lat > 90.0 {
        return None;
    }
    let circ = 2.0 * std::f64::consts::PI * 6_378.137 * lat.to_radians().cos() / (1.0 - 0.006_694_38 * lat.to_radians().sin().powi(2)).sqrt();
    let sun = 90.0 - (lat - 23.44).abs();
    let named = match lat.round() as i64 {
        0 => "the equator",
        23 if (lat - 23.44).abs() < 0.3 => if &m[2] == "north" { "the Tropic of Cancer" } else { "the Tropic of Capricorn" },
        66 | 67 if (lat - 66.56).abs() < 0.3 => if &m[2] == "north" { "the Arctic Circle" } else { "the Antarctic Circle" },
        _ => "",
    };
    let mut r = vec![("length of the parallel", format!("{} km  ({} miles)", n(circ), n(circ / 1.609_344))), ("distance from the equator", format!("{} km along a meridian", n(lat * 111.0)))];
    r.push(("noon Sun altitude at the June solstice", format!("{}°", n(if &m[2] == "north" { sun } else { 90.0 - (lat + 23.44) }))));
    if !named.is_empty() {
        r.push(("name", named.to_string()));
    }
    Some(Answer::new("latitude", vec![input(format!("latitude {}° {}", n(lat), if &m[2] == "north" { "N" } else { "S" })), primary(pod("Result", "Parallel", rows(r)))]))
}

fn routes(l: &str) -> Option<Answer> {
    let l = l.trim_start_matches("distance from ").trim_start_matches("route from ").trim_start_matches("flight from ");
    let parts: Vec<&str> = l.split(" to ").map(|s| s.trim()).collect();
    if parts.len() < 2 || parts.len() > 8 {
        return None;
    }
    // bare words like "salt" or "nice" are towns too: insist on sizeable places
    let cs: Option<Vec<&City>> = parts.iter().map(|p| find_city(p).filter(|c| c.population >= 200_000 || p.contains(','))).collect();
    let cs = cs?;
    let mut legs = Vec::new();
    let mut total = 0.0;
    for w in cs.windows(2) {
        let d = distance_km(w[0], w[1]);
        total += d;
        legs.push(vec![text(format!("{} → {}", w[0].name, w[1].name)), text(format!("{} km", n(d))), text(format!("{} mi", n(d / 1.609_344)))]);
    }
    let names = cs.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(" → ");
    let mut pods = vec![input(names), primary(pod("Result", if cs.len() == 2 { "Distance" } else { "Total distance" }, text(format!("{} km  ({} miles)", n(total), n(total / 1.609_344)))))];
    if cs.len() > 2 {
        pods.push(pod("Legs", "Legs", table(Some(vec!["leg", "km", "miles"]), legs)));
    } else {
        let br = bearing((cs[0].lat, cs[0].lon), (cs[1].lat, cs[1].lon));
        pods.push(pod("Bearing", "Initial bearing", text(format!("{}° ({})", n(br), compass(br)))));
    }
    pods.push(pod("Flight", "Flight time at 900 km/h (direct)", text(format!("about {} hours", fmt_float(total / 900.0, 3)))));
    Some(Answer::new("route", pods))
}

// ---------------------------------------------------------------------------
// seawater
// ---------------------------------------------------------------------------

/// UNESCO (1981) one-atmosphere density of seawater, kg/m³.
fn seawater_density(s: f64, t: f64) -> f64 {
    let rw = 999.842_594 + 6.793_952e-2 * t - 9.095_290e-3 * t * t + 1.001_685e-4 * t.powi(3) - 1.120_083e-6 * t.powi(4) + 6.536_332e-9 * t.powi(5);
    rw + s * (0.824_493 - 4.0899e-3 * t + 7.6438e-5 * t * t - 8.2467e-7 * t.powi(3) + 5.3875e-9 * t.powi(4)) + s.powf(1.5) * (-5.724_66e-3 + 1.0227e-4 * t - 1.6546e-6 * t * t) + 4.8314e-4 * s * s
}

/// Mackenzie (1981) speed of sound in seawater, m/s (depth in metres).
fn sound_speed(s: f64, t: f64, d: f64) -> f64 {
    1448.96 + 4.591 * t - 5.304e-2 * t * t + 2.374e-4 * t.powi(3) + 1.340 * (s - 35.0) + 1.630e-2 * d + 1.675e-7 * d * d - 1.025e-2 * t * (s - 35.0) - 7.139e-13 * t * d.powi(3)
}

fn seawater(l: &str) -> Option<Answer> {
    if !re!(r"\b(?:sea ?water|ocean|seawater|sea)\b").is_match(l) || !re!(r"psu|salinity|‰|ppt|per mil").is_match(l) {
        return None;
    }
    let s: f64 = re!(r"(\d+(?:\.\d+)?)\s*(?:psu|‰|ppt|per mil)").captures(l).or_else(|| re!(r"salinity\s+(?:of\s+)?(\d+(?:\.\d+)?)").captures(l))?[1].parse().ok()?;
    let t = if let Some(c) = re!(r"(-?\d+(?:\.\d+)?)\s*(?:°\s*)?(c|f|degrees? c|degrees? f|celsius|fahrenheit)\b").captures(l) {
        let v: f64 = c[1].parse().ok()?;
        if c[2].starts_with('f') || c[2].contains(" f") { (v - 32.0) / 1.8 } else { v }
    } else {
        15.0
    };
    let depth = if let Some(c) = re!(r"(\d+(?:\.\d+)?)\s*(ft|feet|m|meters|metres|km)\b").captures(l) {
        let v: f64 = c[1].parse().ok()?;
        match &c[2] {
            "ft" | "feet" => v * 0.3048,
            "km" => v * 1000.0,
            _ => v,
        }
    } else {
        0.0
    };
    if !(0.0..=42.0).contains(&s) || !(-2.5..=40.0).contains(&t) || depth > 11_000.0 {
        return None;
    }
    let rho = seawater_density(s, t);
    let c = sound_speed(s, t, depth.min(8000.0));
    let p_dbar = depth * 1.0094; // close to depth in metres
    let tf = -0.0575 * s + 1.710_523e-3 * s.powf(1.5) - 2.154_996e-4 * s * s - 7.53e-4 * p_dbar;
    let pressure = 101_325.0 + rho * 9.80665 * depth;
    let mut r = vec![("density at the surface", format!("{} kg/m³ (σₜ = {})", fmt_float(rho, 7), fmt_float(rho - 1000.0, 5))), ("speed of sound", format!("{} m/s", fmt_float(c, 6))), ("freezing point", format!("{} °C", fmt_float(tf, 4)))];
    if depth > 0.0 {
        r.insert(0, ("pressure at depth", format!("{} MPa ({} atm)", fmt_float(pressure / 1e6, 5), fmt_float(pressure / 101_325.0, 5))));
        r.push(("in-situ density", format!("about {} kg/m³ (compressibility 4.4 × 10⁻¹⁰ /Pa)", fmt_float(rho * (1.0 + 4.4e-10 * (pressure - 101_325.0)), 6))));
    }
    let head = if depth > 0.0 { format!("seawater at {} m, {} °C, salinity {} psu", n(depth), n(t), n(s)) } else { format!("seawater at {} °C, salinity {} psu", n(t), n(s)) };
    Some(Answer::new("seawater", vec![input(head), primary(pod("Result", "Properties", rows(r))), pod("Method", "Method", text("density: UNESCO EOS-80 one-atmosphere equation; sound speed: Mackenzie (1981); freezing point: UNESCO (1978)"))]))
}

// ---------------------------------------------------------------------------
// pressure at altitude
// ---------------------------------------------------------------------------

fn isa_pressure(h: f64) -> f64 {
    if h < 11_000.0 {
        101_325.0 * (1.0 - 2.255_77e-5 * h).powf(5.255_88)
    } else if h < 20_000.0 {
        22_632.06 * (-(h - 11_000.0) / 6_341.62).exp()
    } else {
        5_474.89 * (1.0 + (h - 20_000.0) / 216_650.0).powf(-34.1632)
    }
}

fn altitude_table(l: &str) -> Option<Answer> {
    if !matches!(l, "barometric pressure at extreme altitudes" | "air pressure at high altitude" | "atmospheric pressure at altitude" | "pressure at extreme altitudes" | "air pressure at altitude" | "barometric pressure at high altitude" | "atmospheric pressure vs altitude") {
        return None;
    }
    let places: [(&str, f64); 8] = [("sea level", 0.0), ("Denver", 1609.0), ("La Paz", 3640.0), ("Mont Blanc", 4806.0), ("Everest Base Camp", 5364.0), ("Denali", 6190.0), ("K2", 8611.0), ("Mount Everest", 8849.0)];
    let mut t: Vec<Vec<Content>> = places.iter().map(|(p, h)| {
        let pr = isa_pressure(*h);
        vec![text(*p), text(format!("{} m", n(*h))), text(format!("{} kPa", fmt_float(pr / 1000.0, 4))), text(format!("{}%", fmt_float(pr / 1013.25, 3)))]
    }).collect();
    for h in [10_000.0, 15_000.0, 20_000.0, 30_000.0] {
        let pr = isa_pressure(h);
        t.push(vec![text(if h == 10_000.0 { "airliner cruise" } else { "" }), text(format!("{} m", n(h))), text(format!("{} kPa", fmt_float(pr / 1000.0, 4))), text(format!("{}%", fmt_float(pr / 1013.25, 3)))]);
    }
    Some(Answer::new("pressure at altitude", vec![input("barometric pressure at high altitudes (International Standard Atmosphere)"), primary(pod("Result", "Mount Everest summit", text(format!("{} kPa — {}% of sea-level pressure", fmt_float(isa_pressure(8849.0) / 1000.0, 4), fmt_float(isa_pressure(8849.0) / 1013.25, 3))))), pod("Table", "Standard pressure", table(Some(vec!["place", "altitude", "pressure", "of sea level"]), t))]))
}

// ---------------------------------------------------------------------------
// error-correcting codes
// ---------------------------------------------------------------------------

fn codes(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^codes? (?:that can |which can |able to )?(correct|detect) (\d+) errors?$").captures(l) {
        let k: u64 = m[2].parse().ok()?;
        let d = if &m[1] == "correct" { 2 * k + 1 } else { k + 1 };
        let mut ex = vec![("minimum distance needed", d.to_string()), ("rule", if &m[1] == "correct" { "a code with minimum distance d corrects ⌊(d − 1)/2⌋ errors".into() } else { "a code with minimum distance d detects d − 1 errors".into() })];
        ex.push(("repetition code", format!("[{d}, 1, {d}] — each bit sent {d} times")));
        if &m[1] == "correct" {
            let bch = match k {
                1 => Some("[7, 4, 3] Hamming code; [15, 11, 3] Hamming code"),
                2 => Some("[15, 7, 5] BCH code; [31, 21, 5] BCH code"),
                3 => Some("[23, 12, 7] binary Golay code; [15, 5, 7] BCH code; [31, 16, 7] BCH code"),
                4 => Some("[63, 45, 9] BCH code"),
                5 => Some("[31, 11, 11] BCH code"),
                7 => Some("[31, 6, 15] BCH code; [63, 24, 15] BCH code; [127, 78, 15] BCH code"),
                _ => None,
            };
            if let Some(b) = bch {
                ex.push(("examples", b.to_string()));
            }
            ex.push(("Reed–Solomon", format!("an RS code with {} parity symbols corrects {k} symbol errors", 2 * k)));
        } else {
            ex.push(("parity-style codes", format!("any linear code with minimum distance {d} or more, such as an RS code with {k} parity symbols")));
        }
        return Some(Answer::new("error-correcting codes", vec![input(format!("codes that {} {k} errors", &m[1])), primary(pod("Result", "Requirement", text(format!("minimum Hamming distance d ≥ {d}")))), pod("Details", "Details", rows(ex))]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:binary\s+)?golay code(?:\s+(23|24))?$").captures(l) {
        let ext = m.get(1).map_or(false, |x| x.as_str() == "24");
        let (nn, k, d) = if ext { (24, 12, 8) } else { (23, 12, 7) };
        return Some(Answer::new("Golay code", vec![
            input(format!("{}binary Golay code", if ext { "extended " } else { "" })),
            primary(pod("Result", "Parameters", text(format!("[{nn}, {k}, {d}]")))),
            pod("Details", "Details", rows(vec![
                ("length", nn.to_string()),
                ("dimension", format!("{k} (4096 codewords)")),
                ("minimum distance", d.to_string()),
                ("corrects", format!("{} errors{}", (d - 1) / 2, if ext { ", detects 4" } else { "" })),
                ("information rate", format!("{k}/{nn} ≈ {}", fmt_float(k as f64 / nn as f64, 4))),
                ("perfect", if ext { "no (but self-dual)".into() } else { "yes: Hamming spheres of radius 3 tile the space".to_string() }),
                ("generator polynomial", "x¹¹ + x¹⁰ + x⁶ + x⁵ + x⁴ + x² + 1".into()),
                ("automorphism group", if ext { "Mathieu group M₂₄".into() } else { "Mathieu group M₂₃".to_string() }),
            ])),
        ]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:information rate|code rate|rate|parameters)\s+of\s+(?:the\s+)?(?:a\s+)?(?:binary\s+)?bch code\s+(\d+)\s*,\s*(\d+)$").captures(l).or_else(|| re!(r"^(?:binary\s+)?bch code\s+(\d+)\s*,\s*(\d+)$").captures(l)) {
        let nn: u64 = m[1].parse().ok()?;
        let t: u64 = m[2].parse().ok()?;
        // primitive narrow-sense binary BCH: n = 2^m − 1, k from the cyclotomic cosets
        let mm = (nn + 1).trailing_zeros() as u64;
        if (1u64 << mm) != nn + 1 || !(3..=10).contains(&mm) || t == 0 {
            return None;
        }
        let mut roots = std::collections::BTreeSet::new();
        for i in 1..=2 * t {
            let mut j = i % nn;
            for _ in 0..mm {
                roots.insert(j);
                j = (j * 2) % nn;
            }
        }
        if roots.len() as u64 >= nn {
            return None;
        }
        let k = nn - roots.len() as u64;
        return Some(Answer::new("BCH code", vec![input(format!("binary BCH code of length {nn} correcting {t} errors")), primary(pod("Result", "Information rate", text(format!("{k}/{nn} ≈ {}", fmt_float(k as f64 / nn as f64, 4))))), pod("Details", "Parameters", rows(vec![("length n", nn.to_string()), ("dimension k", k.to_string()), ("designed distance", (2 * t + 1).to_string()), ("corrects", format!("{t} errors")), ("parity bits", (nn - k).to_string())]))]));
    }
    None
}

// ---------------------------------------------------------------------------
// chemical bonds
// ---------------------------------------------------------------------------

/// (atoms, order, energy kJ/mol, length pm)
const BONDS: &[(&str, u8, f64, f64)] = &[
    ("H-H", 1, 436.0, 74.0), ("C-H", 1, 413.0, 109.0), ("N-H", 1, 391.0, 101.0), ("O-H", 1, 463.0, 96.0), ("H-F", 1, 567.0, 92.0), ("H-Cl", 1, 431.0, 127.0), ("H-Br", 1, 366.0, 141.0), ("H-I", 1, 299.0, 161.0),
    ("C-C", 1, 348.0, 154.0), ("C-C", 2, 614.0, 134.0), ("C-C", 3, 839.0, 120.0), ("C-N", 1, 293.0, 147.0), ("C-N", 2, 615.0, 129.0), ("C-N", 3, 891.0, 116.0),
    ("C-O", 1, 358.0, 143.0), ("C-O", 2, 799.0, 120.0), ("C-O", 3, 1072.0, 113.0), ("C-F", 1, 485.0, 135.0), ("C-Cl", 1, 328.0, 177.0), ("C-Br", 1, 276.0, 194.0), ("C-I", 1, 240.0, 214.0), ("C-S", 1, 259.0, 182.0),
    ("N-N", 1, 163.0, 145.0), ("N-N", 2, 418.0, 125.0), ("N-N", 3, 945.0, 110.0), ("N-O", 1, 201.0, 140.0), ("N-O", 2, 607.0, 121.0),
    ("O-O", 1, 146.0, 148.0), ("O-O", 2, 495.0, 121.0), ("F-F", 1, 155.0, 142.0), ("Cl-Cl", 1, 242.0, 199.0), ("Br-Br", 1, 193.0, 228.0), ("I-I", 1, 151.0, 267.0),
    ("Si-H", 1, 323.0, 148.0), ("Si-O", 1, 452.0, 163.0), ("Si-Si", 1, 226.0, 235.0), ("S-H", 1, 339.0, 134.0), ("S-S", 1, 266.0, 205.0), ("P-H", 1, 322.0, 144.0), ("P-P", 1, 201.0, 221.0),
];

fn bonds(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(?:(single|double|triple)\s+)?([a-z]{1,2})\s*(?:-|–|=|≡|\s)\s*([a-z]{1,2})\s+(?:(single|double|triple)\s+)?bonds?(?:\s+(energy|energies|length|lengths|strength|enthalpy))?$").captures(l)
        .or_else(|| re!(r"^(?:the\s+)?(?:bond\s+)?(energy|length|strength|enthalpy)\s+of\s+(?:an?\s+|the\s+)?(?:(single|double|triple)\s+)?([a-z]{1,2})\s*(?:-|–|=|≡|\s)\s*([a-z]{1,2})(?:\s+bond)?()$").captures(l))?;
    let (order_w, a, b) = if m.get(5).map_or(false, |g| g.as_str().is_empty()) && m.get(3).is_some() && !["single", "double", "triple"].contains(&&m[1]) && m.get(4).is_some() && m[4].len() <= 2 {
        // second pattern: groups (what, order, a, b, "")
        (m.get(2).map(|g| g.as_str().to_string()), m[3].to_string(), m[4].to_string())
    } else {
        (m.get(1).or(m.get(4)).map(|g| g.as_str().to_string()), m[2].to_string(), m[3].to_string())
    };
    let cap = |s: &str| {
        let mut c = s.chars();
        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
    };
    let (a, b) = (cap(&a), cap(&b));
    let key1 = format!("{a}-{b}");
    let key2 = format!("{b}-{a}");
    let order = order_w.as_deref().map(|o| match o { "double" => 2u8, "triple" => 3, _ => 1 });
    let order = order.or_else(|| if l.contains('≡') { Some(3) } else if l.contains('=') { Some(2) } else { None });
    let hits: Vec<&(&str, u8, f64, f64)> = BONDS.iter().filter(|x| (x.0 == key1 || x.0 == key2) && order.map_or(true, |o| o == x.1)).collect();
    if hits.is_empty() {
        return None;
    }
    let sym = |o: u8| match o { 1 => "–", 2 => "=", _ => "≡" };
    let t: Vec<Vec<Content>> = hits.iter().map(|x| {
        let (p, q) = x.0.split_once('-').unwrap();
        vec![text(format!("{p}{}{q}", sym(x.1))), text(match x.1 { 1 => "single", 2 => "double", _ => "triple" }), text(format!("{} kJ/mol", n(x.2))), text(format!("{} pm", n(x.3)))]
    }).collect();
    let main = hits.iter().map(|x| {
        let (p, q) = x.0.split_once('-').unwrap();
        format!("{p}{}{q}: {} kJ/mol, {} pm", sym(x.1), n(x.2), n(x.3))
    }).collect::<Vec<_>>().join(" | ");
    Some(Answer::new("chemical bonds", vec![input(format!("{a}–{b} bonds")), primary(pod("Result", "Average bond energy and length", text(main))), pod("Table", "Bonds", table(Some(vec!["bond", "order", "energy", "length"]), t)), pod("Note", "Note", text("average bond enthalpies in gaseous molecules; values in a particular molecule differ by a few percent"))]))
}

// ---------------------------------------------------------------------------
// functional groups
// ---------------------------------------------------------------------------

/// (names, structure, suffix or prefix, example)
const GROUPS: &[(&[&str], &str, &str, &str)] = &[
    (&["alkane"], "R–H (C–C single bonds only)", "-ane", "ethane CH₃CH₃"),
    (&["alkene"], "C=C", "-ene", "ethene CH₂=CH₂"),
    (&["alkyne"], "C≡C", "-yne", "ethyne HC≡CH"),
    (&["arene", "aromatic ring", "phenyl"], "aromatic ring (C₆H₅–)", "phenyl-, -benzene", "toluene C₆H₅CH₃"),
    (&["alcohol", "hydroxyl"], "R–OH", "-ol, hydroxy-", "ethanol CH₃CH₂OH"),
    (&["phenol"], "Ar–OH", "-phenol", "phenol C₆H₅OH"),
    (&["ether"], "R–O–R′", "alkoxy-, … ether", "diethyl ether (C₂H₅)₂O"),
    (&["mom ether", "methoxymethyl ether", "mom"], "R–O–CH₂–O–CH₃", "methoxymethyl (MOM) ether", "MOM-protected alcohols; removed by acid"),
    (&["aldehyde", "formyl"], "R–CHO", "-al, formyl-", "ethanal CH₃CHO"),
    (&["ketone", "carbonyl"], "R–CO–R′", "-one, oxo-", "propanone (acetone) CH₃COCH₃"),
    (&["carboxylic acid", "carboxyl"], "R–COOH", "-oic acid, carboxy-", "ethanoic (acetic) acid CH₃COOH"),
    (&["ester"], "R–COO–R′", "-oate", "ethyl acetate CH₃COOC₂H₅"),
    (&["amide"], "R–CO–NR′R″", "-amide", "acetamide CH₃CONH₂"),
    (&["primary amine"], "R–NH₂", "-amine, amino-", "methylamine CH₃NH₂"),
    (&["secondary amine"], "R₂NH", "N-alkyl…amine", "dimethylamine (CH₃)₂NH"),
    (&["tertiary amine"], "R₃N", "N,N-dialkyl…amine", "trimethylamine (CH₃)₃N"),
    (&["amine", "amino"], "R–NH₂, R₂NH or R₃N", "-amine, amino-", "methylamine CH₃NH₂"),
    (&["nitrile", "cyano"], "R–C≡N", "-nitrile, cyano-", "acetonitrile CH₃CN"),
    (&["nitro"], "R–NO₂", "nitro-", "nitromethane CH₃NO₂"),
    (&["imine"], "R₂C=NR′", "-imine, imino-", "methanimine CH₂=NH"),
    (&["thiol", "sulfhydryl", "mercaptan"], "R–SH", "-thiol, sulfanyl-", "ethanethiol C₂H₅SH"),
    (&["sulfide", "thioether"], "R–S–R′", "alkylthio-", "dimethyl sulfide (CH₃)₂S"),
    (&["haloalkane", "alkyl halide", "halide"], "R–X (X = F, Cl, Br, I)", "fluoro-, chloro-, bromo-, iodo-", "chloromethane CH₃Cl"),
    (&["acyl chloride", "acid chloride"], "R–COCl", "-oyl chloride", "acetyl chloride CH₃COCl"),
    (&["anhydride", "acid anhydride"], "R–CO–O–CO–R′", "-oic anhydride", "acetic anhydride (CH₃CO)₂O"),
    (&["epoxide"], "three-membered C–O–C ring", "epoxy-, oxirane", "ethylene oxide C₂H₄O"),
    (&["azide"], "R–N₃", "azido-", "methyl azide CH₃N₃"),
    (&["isocyanate"], "R–N=C=O", "isocyanato-", "methyl isocyanate CH₃NCO"),
];

fn functional_groups(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:the\s+)?lightest\s+(.+?)s?(?:\s+by\s+molar\s+mass)?$").captures(l) {
        let (name, mass, f) = match m[1].trim_end_matches('s') {
            "nitrile" => ("acetonitrile, CH₃CN (hydrogen cyanide, HCN, 27.03 g/mol, if counted as formonitrile)", 41.05, "C₂H₃N"),
            "alcohol" => ("methanol, CH₃OH", 32.04, "CH₄O"),
            "aldehyde" => ("formaldehyde, HCHO", 30.03, "CH₂O"),
            "ketone" => ("acetone, CH₃COCH₃", 58.08, "C₃H₆O"),
            "carboxylic acid" => ("formic acid, HCOOH", 46.03, "CH₂O₂"),
            "amine" => ("methylamine, CH₃NH₂", 31.06, "CH₅N"),
            "ester" => ("methyl formate, HCOOCH₃", 60.05, "C₂H₄O₂"),
            "ether" => ("dimethyl ether, CH₃OCH₃", 46.07, "C₂H₆O"),
            "alkene" => ("ethylene, CH₂=CH₂", 28.05, "C₂H₄"),
            "alkyne" => ("acetylene, HC≡CH", 26.04, "C₂H₂"),
            "alkane" => ("methane, CH₄", 16.04, "CH₄"),
            _ => return None,
        };
        return Some(Answer::new("lightest compound", vec![input(format!("lightest {} by molar mass", m[1].trim_end_matches('s'))), primary(pod("Result", "Result", text(name))), pod("Mass", "Molar mass", text(format!("{} g/mol ({f})", n(mass))))]));
    }
    let m = re!(r"^(?:the\s+)?(.+?)(?:\s+functional\s+group|\s+group)s?$").captures(l)?;
    let w = m[1].trim();
    let g = GROUPS.iter().find(|g| g.0.iter().any(|x| *x == w))?;
    Some(Answer::new("functional group", vec![input(format!("{} functional group", g.0[0])), primary(pod("Result", "Structure", text(g.1))), pod("Details", "Details", rows(vec![("naming", g.2.to_string()), ("example", g.3.to_string())]))]))
}

// ---------------------------------------------------------------------------
// cosmic origins of the elements
// ---------------------------------------------------------------------------

/// Main production sites by atomic number (after J. A. Johnson's periodic table of origins).
fn origins(z: u32) -> &'static str {
    match z {
        1 => "Big Bang nucleosynthesis",
        2 => "Big Bang nucleosynthesis (with some from stars)",
        3 => "Big Bang, cosmic-ray fission and dying low-mass stars",
        4 | 5 => "cosmic-ray fission (spallation)",
        6 => "dying low-mass stars (AGB stars) and exploding massive stars",
        7 => "dying low-mass stars and exploding massive stars",
        8..=12 | 14 | 16 | 18 | 20 => "exploding massive stars (core-collapse supernovae)",
        13 | 15 | 17 | 19 | 21 | 22 => "exploding massive stars",
        23..=28 => "exploding white dwarfs (type Ia supernovae) and exploding massive stars",
        29..=32 => "exploding massive stars and dying low-mass stars (s-process)",
        38..=40 | 56..=60 => "dying low-mass stars (s-process), with some from merging neutron stars",
        41 | 42 | 82 => "dying low-mass stars (s-process)",
        43 | 61 => "none in nature today (radioactive; made in reactors)",
        63 | 64 | 66..=70 | 75..=79 | 52..=54 => "merging neutron stars (r-process), with some from the s-process",
        90 | 92 | 94 => "merging neutron stars (r-process)",
        33..=37 | 44..=51 | 55 | 62 | 65 | 71..=74 | 80 | 81 | 83 => "dying low-mass stars and merging neutron stars",
        84..=89 | 91 | 93 => "radioactive decay of heavier elements (short-lived)",
        _ => "",
    }
}

fn cosmic_origins(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:what are |what is )?(?:the\s+)?(?:cosmic|nucleosynthetic|stellar)\s+origins?\s+of\s+(.+)$").captures(l)
        .or_else(|| re!(r"^(.+?)\s+(?:cosmic|nucleosynthetic)\s+origins?$").captures(l))
        .or_else(|| re!(r"^where (?:did|does|do)\s+(.+?)\s+come from$").captures(l))?;
    let names: Vec<&str> = re!(r"\s*(?:,|&|\band\b)\s*").split(&m[1]).filter(|s| !s.trim().is_empty()).collect();
    // symbols arrive lowercased: "c & au" → C, Au
    let sym = |s: &str| {
        let mut c = s.chars();
        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
    };
    let els: Option<Vec<&super::chem::Element>> = names.iter().map(|s| super::chem::find_element(s.trim()).or_else(|| (s.trim().len() <= 2).then(|| super::chem::find_element(&sym(s.trim()))).flatten())).collect();
    let els = els?;
    let rowsv: Vec<Vec<Content>> = els.iter().map(|e| vec![text(format!("{} ({})", e.name, e.symbol)), text(origins(e.z))]).collect();
    if els.iter().any(|e| origins(e.z).is_empty()) {
        return None;
    }
    let main = els.iter().map(|e| format!("{}: {}", e.name, origins(e.z))).collect::<Vec<_>>().join(" | ");
    Some(Answer::new("cosmic origins", vec![input(format!("cosmic origins of {}", els.iter().map(|e| e.name.as_str()).collect::<Vec<_>>().join(", "))), primary(pod("Result", "Main sources", text(main))), pod("Table", "Origins", table(Some(vec!["element", "made mainly by"]), rowsv))]))
}

/// Layers of the Earth by depth below the surface (km) and of the Sun by
/// distance from its centre (km): names, range, state, composition, temperature.
const EARTH_LAYERS: &[(&str, &str, f64, f64, &str, &str, &str)] = &[
    ("crust", "crust", 0.0, 35.0, "solid", "silicate rocks: granitic continental crust (average 35 km thick, up to 70 km under mountains) and basaltic oceanic crust (5–10 km)", "up to about 500–1,000 °C at its base"),
    ("lithosphere", "lithosphere", 0.0, 100.0, "solid, rigid", "the crust and the uppermost mantle, broken into tectonic plates; 50–200 km thick", "up to about 1,300 °C at its base"),
    ("asthenosphere", "asthenosphere", 100.0, 350.0, "solid but ductile, slightly molten in places", "upper-mantle peridotite that flows slowly, letting the plates above move", "about 1,300–1,600 °C"),
    ("upper mantle", "upper mantle", 35.0, 410.0, "solid (ductile below the lithosphere)", "peridotite: olivine, pyroxene and garnet", "about 500–1,900 °C"),
    ("transition zone;mantle transition zone", "mantle transition zone", 410.0, 660.0, "solid", "high-pressure forms of olivine (wadsleyite, ringwoodite) and garnet; holds much water", "about 1,500–1,900 °C"),
    ("lower mantle", "lower mantle", 660.0, 2891.0, "solid", "bridgmanite and ferropericlase", "about 1,900–3,700 °C"),
    ("mantle;earth's mantle", "mantle", 35.0, 2891.0, "solid, flowing slowly over geological time", "silicate rocks rich in magnesium and iron; about 84% of Earth's volume and 67% of its mass", "about 500 °C near the crust to about 3,700 °C at the core boundary"),
    ("outer core", "outer core", 2891.0, 5150.0, "liquid", "iron and nickel with lighter elements; its convection generates Earth's magnetic field", "about 4,000–5,400 °C"),
    ("inner core", "inner core", 5150.0, 6371.0, "solid", "iron–nickel alloy, solid because of the enormous pressure (about 330–360 GPa)", "about 5,200–5,400 °C"),
    ("core;earth's core", "core", 2891.0, 6371.0, "liquid outer core and solid inner core", "iron and nickel; about 32% of Earth's mass", "about 4,000–5,400 °C"),
];

const SUN_LAYERS: &[(&str, &str, f64, f64, &str, &str)] = &[
    ("core;solar core;sun's core", "core", 0.0, 174_000.0, "nuclear fusion of hydrogen into helium releases the Sun's energy", "about 15.7 million K at the centre"),
    ("radiative zone;radiation zone", "radiative zone", 174_000.0, 487_000.0, "energy moves outward as radiation, absorbed and re-emitted countless times (taking 100,000 years or more)", "about 7 million K falling to about 2 million K"),
    ("tachocline", "tachocline", 480_000.0, 495_000.0, "thin shear layer between the rigidly rotating radiative zone and the convection zone; thought to drive the solar dynamo", "about 2 million K"),
    ("convection zone;convective zone", "convection zone", 487_000.0, 695_700.0, "hot plasma rises and cooler plasma sinks, carrying energy to the surface", "about 2 million K falling to about 5,800 K"),
    ("photosphere", "photosphere", 695_700.0, 696_200.0, "the visible surface, about 500 km thick, where light escapes into space; shows granulation and sunspots", "about 5,772 K (effective temperature)"),
    ("chromosphere", "chromosphere", 696_200.0, 698_200.0, "thin reddish layer about 2,000 km thick above the photosphere, seen during total eclipses", "about 4,000 K rising to about 20,000 K"),
    ("corona;solar corona", "corona", 698_200.0, 0.0, "the outer atmosphere, extending millions of kilometres and flowing out as the solar wind", "about 1–3 million K"),
];

/// "asthenosphere", "radiative zone of the sun", "50,000 km beneath the surface of the sun",
/// "100 km below the surface of the Earth"
fn layers(l: &str) -> Option<Answer> {
    let fmt_km = |v: f64| super::geo::fmt_int(v.round() as u64);
    // depth below the surface
    if let Some(m) = re!(r"^([\d,.]+) ?(km|kilometers|kilometres|miles|mi) (?:beneath|below|under|deep (?:in|into)|into) (?:the )?(?:surface (?:of|on) )?(?:the )?(earth|sun)(?:'s surface)?$").captures(l) {
        let mut d: f64 = m[1].replace(',', "").parse().ok()?;
        if m[2].starts_with('m') {
            d *= 1.609344;
        }
        if &m[3] == "earth" {
            let fine = ["crust", "asthenosphere", "upper mantle", "mantle transition zone", "lower mantle", "outer core", "inner core"];
            let layer = if d < 35.0 { "crust" } else if d < 100.0 { "upper mantle" } else { fine.iter().copied().find(|n| EARTH_LAYERS.iter().any(|x| x.1 == *n && d >= x.2 && d < x.3)).unwrap_or("inner core") };
            let x = EARTH_LAYERS.iter().find(|x| x.1 == layer)?;
            if d > 6371.0 {
                return None;
            }
            return Some(Answer::new("Earth layer", vec![
                pod("Input", "Input interpretation", text(format!("{} km below Earth's surface", fmt_km(d)))),
                primary(pod("Result", "Layer", text(format!("{} ({}–{} km deep)", x.1, fmt_km(x.2), fmt_km(x.3))))),
                pod("Table", "Properties", table(None, vec![vec![text("state"), text(x.4)], vec![text("composition"), text(x.5)], vec![text("temperature"), text(x.6)], vec![text("depth"), text(format!("{} km of 6,371 km to the centre ({}%)", fmt_km(d), fmt_float((d / 6371.0 * 1000.0).round() / 10.0, 4)))]])),
            ]));
        }
        let r = 695_700.0 - d;
        if r < 0.0 {
            return None;
        }
        let x = SUN_LAYERS.iter().filter(|x| x.1 != "tachocline").find(|x| r >= x.2 && (x.3 == 0.0 || r < x.3))?;
        return Some(Answer::new("solar layer", vec![
            pod("Input", "Input interpretation", text(format!("{} km below the Sun's surface", fmt_km(d)))),
            primary(pod("Result", "Layer", text(format!("{} ({} solar radii from the centre)", x.1, fmt_float((r / 695_700.0 * 1000.0).round() / 1000.0, 4))))),
            pod("Table", "Properties", table(None, vec![vec![text("description"), text(x.4)], vec![text("temperature"), text(x.5)]])),
        ]));
    }
    // the layers listed or compared: "layers of the earth", "crust, mantle", "inner core vs outer core"
    let list: Option<Vec<&(&str, &str, f64, f64, &str, &str, &str)>> = if re!(r"^(?:the )?(?:layers|structure|interior|internal structure) of (?:the )?earth$|^earth(?:'s)? (?:layers|interior|internal structure)$").is_match(l) {
        Some(["crust", "mantle", "outer core", "inner core"].iter().filter_map(|n| EARTH_LAYERS.iter().find(|x| x.1 == *n)).collect())
    } else {
        let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\.?|\bversus\b|\band\b)\s*").split(l).map(|p| p.trim().trim_start_matches("the ").trim_start_matches("earth's ")).filter(|p| !p.is_empty()).collect();
        if parts.len() >= 2 { parts.iter().map(|p| EARTH_LAYERS.iter().find(|x| x.0.split(';').any(|a| a == *p))).collect() } else { None }
    };
    if let Some(xs) = list {
        let rows = xs.iter().map(|x| vec![text(x.1), text(format!("{}–{} km", fmt_km(x.2), fmt_km(x.3))), text(x.4), text(x.5), text(x.6)]).collect();
        return Some(Answer::new("Earth layers", vec![
            pod("Input", "Input interpretation", text(format!("Earth | {}", xs.iter().map(|x| x.1).collect::<Vec<_>>().join(" | ")))),
            primary(pod("Result", "Depths", text(xs.iter().map(|x| format!("{}: {}–{} km", x.1, fmt_km(x.2), fmt_km(x.3))).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Properties", table(Some(vec!["layer", "depth", "state", "composition", "temperature"]), rows)),
        ]));
    }
    // a named layer: "asthenosphere", "radiative zone of the sun", "earth's outer core"
    let sun = l.contains("sun") || l.contains("solar");
    let name = re!(r"^(?:the )?(?:earth'?s |sun'?s |solar )?(.+?)(?: (?:of|in) (?:the )?(?:earth|sun))?$").captures(l)?;
    let n = name[1].trim().trim_start_matches("the ");
    if sun || ["photosphere", "chromosphere", "corona", "radiative zone", "convection zone", "tachocline"].contains(&n) {
        let x = SUN_LAYERS.iter().find(|x| x.0.split(';').any(|a| a == n))?;
        if !sun && x.1 == "corona" {
            return None;
        }
        let range = if x.3 == 0.0 { format!("from about {} km from the centre outward", fmt_km(x.2)) } else { format!("{}–{} km from the centre ({}–{} solar radii)", fmt_km(x.2), fmt_km(x.3), fmt_float((x.2 / 695_700.0 * 100.0).round() / 100.0, 3), fmt_float((x.3 / 695_700.0 * 100.0).round() / 100.0, 3)) };
        return Some(Answer::new("solar layer", vec![
            pod("Input", "Input interpretation", text(format!("Sun | {}", x.1))),
            primary(pod("Result", "Description", text(x.4))),
            pod("Table", "Properties", table(None, vec![vec![text("extent"), text(range)], vec![text("temperature"), text(x.5)]])),
        ]));
    }
    let x = EARTH_LAYERS.iter().find(|x| x.0.split(';').any(|a| a == n))?;
    if ["core", "crust", "mantle"].contains(&x.1) && !l.contains("earth") {
        return None;
    }
    Some(Answer::new("Earth layer", vec![
        pod("Input", "Input interpretation", text(format!("Earth | {}", x.1))),
        primary(pod("Result", "Depth", text(format!("{}–{} km below the surface", fmt_km(x.2), fmt_km(x.3))))),
        pod("Table", "Properties", table(None, vec![vec![text("state"), text(x.4)], vec![text("composition"), text(x.5)], vec![text("temperature"), text(x.6)], vec![text("thickness"), text(format!("about {} km", fmt_km(x.3 - x.2)))]])),
    ]))
}

/// Composition of dry air near the surface, by volume.
const AIR: &[(&str, &str, &str)] = &[
    ("nitrogen", "N₂", "78.084%"),
    ("oxygen", "O₂", "20.946%"),
    ("argon", "Ar", "0.934%"),
    ("carbon dioxide", "CO₂", "about 0.042% (420 ppm, 2023–2024; rising about 2.5 ppm a year)"),
    ("neon", "Ne", "18.18 ppm"),
    ("helium", "He", "5.24 ppm"),
    ("methane", "CH₄", "about 1.9 ppm"),
    ("krypton", "Kr", "1.14 ppm"),
    ("hydrogen", "H₂", "0.55 ppm"),
    ("nitrous oxide", "N₂O", "about 0.34 ppm"),
    ("carbon monoxide", "CO", "about 0.1 ppm"),
    ("xenon", "Xe", "0.087 ppm"),
    ("ozone", "O₃", "0–0.07 ppm near the ground (up to about 10 ppm in the ozone layer)"),
];

/// "atmosphere of Earth", "composition of air", "what is air made of"
fn atmosphere(l: &str) -> Option<Answer> {
    if !re!(r"^(?:the )?(?:(?:composition|gases) of (?:the )?(?:air|atmosphere|earth'?s atmosphere|atmosphere of (?:the )?earth)|(?:the )?atmosphere of (?:the )?earth|earth'?s atmosphere|air composition|atmospheric composition|composition of the earth'?s atmosphere|(?:what is )?(?:the )?air made of|what gases (?:are|make up) (?:in )?(?:the )?air)$").is_match(l) {
        return None;
    }
    let rows = AIR.iter().map(|(n, f, v)| vec![text(*n), text(*f), text(*v)]).collect();
    Some(Answer::new("Earth's atmosphere", vec![
        pod("Input", "Input interpretation", text("Earth | atmosphere")),
        primary(pod("Result", "Composition (dry air, by volume)", text("nitrogen 78.08% | oxygen 20.95% | argon 0.93% | carbon dioxide 0.042% | trace gases"))),
        pod("Table", "Constituents", table(Some(vec!["gas", "formula", "fraction by volume"]), rows)),
        pod("Properties", "Properties", table(None, vec![
            vec![text("water vapour"), text("0–4% by volume, about 0.25% averaged over the whole atmosphere")],
            vec![text("mean molar mass of dry air"), text("28.96 g/mol")],
            vec![text("total mass"), text("5.15×10¹⁸ kg")],
            vec![text("mean surface pressure"), text("101.325 kPa (1 atm) at sea level")],
            vec![text("density at sea level, 15 °C"), text("1.225 kg/m³")],
            vec![text("scale height"), text("about 8.5 km")],
            vec![text("layers"), text("troposphere, stratosphere, mesosphere, thermosphere, exosphere")],
        ])),
    ]))
}

/// "SSW", "south-southwest", "ENE": the 32-wind compass (16 named here)
fn compass_point(raw: &str, l: &str) -> Option<Answer> {
    const P: [(&str, &str); 16] = [
        ("N", "north"), ("NNE", "north-northeast"), ("NE", "northeast"), ("ENE", "east-northeast"), ("E", "east"), ("ESE", "east-southeast"), ("SE", "southeast"), ("SSE", "south-southeast"),
        ("S", "south"), ("SSW", "south-southwest"), ("SW", "southwest"), ("WSW", "west-southwest"), ("W", "west"), ("WNW", "west-northwest"), ("NW", "northwest"), ("NNW", "north-northwest"),
    ];
    let r = raw.trim().trim_end_matches(['?', '.']);
    let name = l.trim_start_matches("compass direction ").trim_start_matches("compass point ").replace(' ', "-");
    let k = P.iter().position(|(a, n)| (a.len() == 3 && *a == r) || (n.contains('-') && *n == name) || (raw.to_lowercase().starts_with("compass") && a.eq_ignore_ascii_case(&name)))?;
    let (abbr, full) = P[k];
    let deg = k as f64 * 22.5;
    let opp = P[(k + 8) % 16];
    Some(Answer::new("compass point", vec![
        pod("Input", "Input interpretation", text(format!("{full} ({abbr})"))),
        primary(pod("Result", "Bearing", text(format!("{}° (range {}° to {}°)", fmt_float(deg, 6), fmt_float((deg - 11.25 + 360.0) % 360.0, 6), fmt_float(deg + 11.25, 6))))),
        pod("Table", "Basic information", table(None, vec![
            vec![text("name"), text(full)],
            vec![text("abbreviation"), text(abbr)],
            vec![text("bearing"), text(format!("{}° clockwise from north", fmt_float(deg, 6)))],
            vec![text("opposite direction"), text(format!("{} ({})", opp.1, opp.0))],
        ])),
    ]))
}

/// The ten WMO cloud genera: name, abbreviation, étage, typical base in
/// temperate regions (km), appearance, composition, precipitation.
const CLOUDS: &[(&str, &str, &str, &str, &str, &str, &str)] = &[
    ("cirrus", "Ci", "high", "5–13 km (16,500–45,000 ft)", "detached white filaments, patches or narrow bands with a fibrous or silky look", "ice crystals", "none reaching the ground"),
    ("cirrocumulus", "Cc", "high", "5–13 km (16,500–45,000 ft)", "thin white patches or layers of very small grains or ripples, without shading", "ice crystals (supercooled droplets at times)", "none reaching the ground (virga at times)"),
    ("cirrostratus", "Cs", "high", "5–13 km (16,500–45,000 ft)", "transparent whitish veil, smooth or fibrous, often producing a halo around the Sun or Moon", "ice crystals", "none"),
    ("altocumulus", "Ac", "middle", "2–7 km (6,500–23,000 ft)", "white or grey patches, sheets or layers of rounded masses or rolls, partly shaded", "mostly water droplets, ice when very cold", "virga, rarely light rain or snow"),
    ("altostratus", "As", "middle", "2–7 km (6,500–23,000 ft), often thicker upward", "greyish or bluish sheet that may hide the Sun or show it as through ground glass, no halo", "water droplets and ice crystals", "light continuous rain or snow"),
    ("nimbostratus", "Ns", "middle (base usually low)", "base usually 0.5–3 km (1,500–10,000 ft); several kilometres thick", "dark grey layer, diffuse by continuous rain or snow, thick enough to blot out the Sun", "water droplets, raindrops, ice crystals and snowflakes", "continuous moderate or heavy rain or snow"),
    ("stratocumulus", "Sc", "low", "0.5–2 km (1,500–6,500 ft)", "grey or whitish patches, sheets or layers with dark parts, rounded masses or rolls", "water droplets", "drizzle, light rain or snow"),
    ("stratus", "St", "low", "surface–0.5 km (0–1,500 ft)", "uniform grey layer with a fairly even base; fog when it touches the ground", "water droplets (ice when very cold)", "drizzle, snow grains"),
    ("cumulus", "Cu", "low (vertical development)", "base 0.5–2 km (1,500–6,500 ft); tops up to about 6 km", "detached dense clouds with sharp outlines rising as mounds or towers, bright white tops and flat dark bases", "water droplets", "none for fair-weather cumulus; showers from cumulus congestus"),
    ("cumulonimbus", "Cb", "low (spanning all étages)", "base 0.5–2 km (1,500–6,500 ft); tops at 10–16 km (33,000–52,000 ft)", "heavy, dense towering cloud, often with a flattened anvil-shaped top", "water droplets below, ice crystals in the upper part", "heavy showers, hail, thunder and lightning"),
];

/// "cumulus", "nimbostratus, cumulus", "altocumulus, cumulus cloud height", "cirrus clouds"
fn clouds(l: &str) -> Option<Answer> {
    let height = re!(r"\s+(?:cloud )?(?:height|altitude|base|base height|heights|altitudes)$").is_match(l) || re!(r"^(?:height|altitude) of ").is_match(l);
    let body = re!(r"\s+(?:cloud )?(?:height|altitude|base|base height|heights|altitudes)$").replace(l, "");
    let body = re!(r"^(?:(?:height|altitude) of |what (?:is|are) )?(?:an? |the )?").replace(&body, "");
    let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\.?|\bversus\b|\band\b)\s*").split(body.trim()).filter(|x| !x.is_empty()).collect();
    let found: Option<Vec<_>> = parts
        .iter()
        .map(|p| {
            let w = re!(r"\s+clouds?$").replace(p.trim(), "");
            let w = w.trim_end_matches('s');
            CLOUDS.iter().find(|c| c.0 == w || c.0 == format!("{w}s") || c.1.eq_ignore_ascii_case(w) && p.trim().ends_with("cloud"))
        })
        .collect();
    let found = found?;
    if found.is_empty() {
        return None;
    }
    let cap = |s: &str| s[..1].to_uppercase() + &s[1..];
    let src = pod("Source", "Source", text("WMO International Cloud Atlas; heights are typical of temperate regions (lower near the poles, higher in the tropics)"));
    if found.len() == 1 {
        let c = found[0];
        let (title, value) = if height { ("Typical altitude", c.3.to_string()) } else { ("Description", format!("{} ({}): {} cloud — {}", cap(c.0), c.1, c.2, c.4)) };
        return Some(Answer::new("cloud type", vec![
            pod("Input", "Input interpretation", text(format!("{} cloud{}", c.0, if height { " | altitude" } else { "" }))),
            primary(pod("Result", title, text(value))),
            pod("Table", "Properties", table(None, vec![
                vec![text("abbreviation"), text(c.1)],
                vec![text("étage"), text(c.2)],
                vec![text("typical altitude"), text(c.3)],
                vec![text("appearance"), text(c.4)],
                vec![text("composition"), text(c.5)],
                vec![text("precipitation"), text(c.6)],
            ])),
            src,
        ]));
    }
    let rows = found.iter().map(|c| vec![text(c.0), text(c.1), text(c.2), text(c.3), text(c.6)]).collect();
    Some(Answer::new("cloud types", vec![
        pod("Input", "Input interpretation", text(found.iter().map(|c| c.0).collect::<Vec<_>>().join(" | "))),
        primary(pod("Result", if height { "Typical altitudes" } else { "Cloud types" }, text(found.iter().map(|c| format!("{}: {}", c.0, if height { c.3 } else { c.2 })).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Properties", table(Some(vec!["genus", "abbr.", "étage", "typical altitude", "precipitation"]), rows)),
        src,
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    compass_point(&q.raw, l)
        .or_else(|| layers(l))
        .or_else(|| atmosphere(l))
        .or_else(|| clouds(l))
        .or_else(|| seawater(l))
        .or_else(|| altitude_table(l))
        .or_else(|| codes(l))
        .or_else(|| bonds(l))
        .or_else(|| functional_groups(l))
        .or_else(|| cosmic_origins(l))
        .or_else(|| latitude_line(l))
        .or_else(|| coordinates(l))
        .or_else(|| routes(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seawater_reference_values() {
        // UNESCO check value: S = 35, T = 25 °C → 1023.343 kg/m³ at one atmosphere
        assert!((seawater_density(35.0, 25.0) - 1023.343).abs() < 0.002);
        // pure water at 4 °C ≈ 999.975
        assert!((seawater_density(0.0, 4.0) - 999.975).abs() < 0.002);
    }

    #[test]
    fn bch_dimensions() {
        let a = codes("information rate of bch code 31, 5").unwrap();
        assert!(content_text(&a.pods[1].subpods[0].content).starts_with("11/31"));
    }
}
