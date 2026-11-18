//! Equinoxes and solstices for years 1000–3000 (Meeus, Astronomical
//! Algorithms, ch. 27; accurate to about a minute).
//!
//! "autumnal equinox 1850", "next equinox", "summer solstice 2026",
//! "equinoxes and solstices 2025", "when is the next solstice".

use crate::engine::*;
use chrono::{Datelike, NaiveDateTime, Utc};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

const TERMS: [(f64, f64, f64); 24] = [
    (485.0, 324.96, 1934.136), (203.0, 337.23, 32964.467), (199.0, 342.08, 20.186), (182.0, 27.85, 445267.112),
    (156.0, 73.14, 45036.886), (136.0, 171.52, 22518.443), (77.0, 222.54, 65928.934), (74.0, 296.72, 3034.906),
    (70.0, 243.58, 9037.513), (58.0, 119.81, 33718.147), (52.0, 297.17, 150.678), (50.0, 21.02, 2281.226),
    (45.0, 247.54, 29929.562), (44.0, 325.15, 31555.956), (29.0, 60.93, 4443.417), (18.0, 155.12, 67555.328),
    (17.0, 288.79, 4562.452), (16.0, 198.04, 62894.029), (14.0, 199.76, 31436.921), (12.0, 95.39, 14577.848),
    (12.0, 287.11, 31931.756), (12.0, 320.81, 34777.259), (9.0, 227.73, 1222.114), (8.0, 15.45, 16859.074),
];

/// ΔT = TT − UT in seconds (Espenak and Meeus polynomial fits).
fn delta_t(y: f64) -> f64 {
    let u = (y - 1820.0) / 100.0;
    match y {
        y if y < 1600.0 => -20.0 + 32.0 * u * u,
        y if y < 1700.0 => { let t = y - 1600.0; 120.0 - 0.9808 * t - 0.01532 * t * t + t.powi(3) / 7129.0 }
        y if y < 1800.0 => { let t = y - 1700.0; 8.83 + 0.1603 * t - 0.0059285 * t * t + 0.00013336 * t.powi(3) - t.powi(4) / 1_174_000.0 }
        y if y < 1860.0 => { let t = y - 1800.0; 13.72 - 0.332447 * t + 0.0068612 * t * t + 0.0041116 * t.powi(3) - 0.00037436 * t.powi(4) + 0.0000121272 * t.powi(5) - 0.0000001699 * t.powi(6) + 0.000000000875 * t.powi(7) }
        y if y < 1900.0 => { let t = y - 1860.0; 7.62 + 0.5737 * t - 0.251754 * t * t + 0.01680668 * t.powi(3) - 0.0004473624 * t.powi(4) + t.powi(5) / 233_174.0 }
        y if y < 1920.0 => { let t = y - 1900.0; -2.79 + 1.494119 * t - 0.0598939 * t * t + 0.0061966 * t.powi(3) - 0.000197 * t.powi(4) }
        y if y < 1941.0 => { let t = y - 1920.0; 21.20 + 0.84493 * t - 0.076100 * t * t + 0.0020936 * t.powi(3) }
        y if y < 1961.0 => { let t = y - 1950.0; 29.07 + 0.407 * t - t * t / 233.0 + t.powi(3) / 2547.0 }
        y if y < 1986.0 => { let t = y - 1975.0; 45.45 + 1.067 * t - t * t / 260.0 - t.powi(3) / 718.0 }
        y if y < 2005.0 => { let t = y - 2000.0; 63.86 + 0.3345 * t - 0.060374 * t * t + 0.0017275 * t.powi(3) + 0.000651814 * t.powi(4) + 0.00002373599 * t.powi(5) }
        y if y < 2050.0 => { let t = y - 2000.0; 62.92 + 0.32217 * t + 0.005589 * t * t }
        y if y < 2150.0 => -20.0 + 32.0 * u * u - 0.5628 * (2150.0 - y),
        _ => -20.0 + 32.0 * u * u,
    }
}

/// Julian Ephemeris Day of event k (0 March equinox, 1 June solstice, 2 September equinox, 3 December solstice).
fn event_jde(year: i32, k: usize) -> f64 {
    let y = (year as f64 - 2000.0) / 1000.0;
    let c: [[f64; 5]; 4] = [
        [2451623.80984, 365242.37404, 0.05169, -0.00411, -0.00057],
        [2451716.56767, 365241.62603, 0.00325, 0.00888, -0.00030],
        [2451810.21715, 365242.01767, -0.11575, 0.00337, 0.00078],
        [2451900.05952, 365242.74049, -0.06223, -0.00823, 0.00032],
    ];
    let p = c[k];
    let jde0 = p[0] + p[1] * y + p[2] * y * y + p[3] * y.powi(3) + p[4] * y.powi(4);
    let t = (jde0 - 2451545.0) / 36525.0;
    let w = (35999.373 * t - 2.47).to_radians();
    let dl = 1.0 + 0.0334 * w.cos() + 0.0007 * (2.0 * w).cos();
    let s: f64 = TERMS.iter().map(|(a, b, cc)| a * (b + cc * t).to_radians().cos()).sum();
    jde0 + 0.00001 * s / dl
}

fn event_utc(year: i32, k: usize) -> NaiveDateTime {
    let jd = event_jde(year, k) - delta_t(year as f64) / 86400.0;
    let secs = ((jd - 2_440_587.5) * 86400.0).round() as i64;
    chrono::DateTime::from_timestamp(secs, 0).map(|d| d.naive_utc()).unwrap_or_default()
}

const NAMES: [&str; 4] = ["March equinox", "June solstice", "September equinox", "December solstice"];
const NORTH: [&str; 4] = ["spring (vernal) equinox", "summer solstice", "autumnal equinox", "winter solstice"];

fn relative(t: NaiveDateTime) -> String {
    let now = Utc::now().naive_utc();
    let days = (t - now).num_seconds() as f64 / 86400.0;
    if days.abs() < 1.0 {
        if days >= 0.0 { "within the next day".into() } else { "within the past day".into() }
    } else if days > 0.0 {
        format!("in {} days", super::geo::fmt_int(days.floor() as u64))
    } else {
        format!("{} days ago", super::geo::fmt_int((-days).floor() as u64))
    }
}

fn fmt(t: NaiveDateTime) -> String {
    t.format("%A, %B %-d, %Y, %H:%M UTC").to_string()
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = re!(r"^(?:when is |when was |when will be |date of |time of |what day is )?(?:the )?").replace(&l, "").to_string();
    let now = Utc::now().naive_utc();
    // "equinoxes and solstices 2025", "solstices 2026"
    if let Some(m) = re!(r"^(equinoxes and solstices|solstices and equinoxes|equinoxes|solstices|seasons)(?: (?:in |of |for )?(\d{3,4}))?$").captures(&l) {
        let year: i32 = m.get(2).and_then(|y| y.as_str().parse().ok()).unwrap_or(now.year());
        if !(1000..=3000).contains(&year) {
            return None;
        }
        let ks: Vec<usize> = match &m[1] { "equinoxes" => vec![0, 2], "solstices" => vec![1, 3], _ => vec![0, 1, 2, 3] };
        let rows: Vec<Vec<Content>> = ks.iter().map(|&k| vec![text(NAMES[k]), text(fmt(event_utc(year, k)))]).collect();
        let short = ks.iter().map(|&k| format!("{}: {}", NAMES[k], event_utc(year, k).format("%B %-d, %H:%M UTC"))).collect::<Vec<_>>().join(" | ");
        return Some(Answer::new(format!("{} {year}", &m[1]), vec![
            pod("Input", "Input interpretation", text(format!("{} | {year}", &m[1]))),
            primary(pod("Result", "Dates", text(short))),
            pod("Table", "Events", table(Some(vec!["event", "time"]), rows)),
        ]));
    }
    let m = re!(r"^(?:(next|last|previous|this year's|upcoming) )?(spring|vernal|march|summer|june|autumnal|autumn|fall|september|winter|december)? ?(equinox|solstice)(?: (?:in |of )?(\d{3,4}))?$").captures(&l)?;
    let which = m.get(2).map(|x| x.as_str());
    let kind = &m[3];
    let ks: Vec<usize> = match (which, kind) {
        (Some("spring" | "vernal" | "march"), "equinox") => vec![0],
        (Some("autumnal" | "autumn" | "fall" | "september"), "equinox") => vec![2],
        (Some("summer" | "june"), "solstice") => vec![1],
        (Some("winter" | "december"), "solstice") => vec![3],
        (None, "equinox") => vec![0, 2],
        (None, "solstice") => vec![1, 3],
        _ => return None,
    };
    let year: Option<i32> = m.get(4).and_then(|y| y.as_str().parse().ok());
    let dir = m.get(1).map(|x| x.as_str());
    let (y, k) = if let Some(y) = year {
        if !(1000..=3000).contains(&y) {
            return None;
        }
        if ks.len() > 1 {
            // "equinox 2025": both of that year
            let rows: Vec<Vec<Content>> = ks.iter().map(|&k| vec![text(NAMES[k]), text(fmt(event_utc(y, k)))]).collect();
            let short = ks.iter().map(|&k| format!("{}: {}", NAMES[k], event_utc(y, k).format("%B %-d, %H:%M UTC"))).collect::<Vec<_>>().join(" | ");
            return Some(Answer::new(format!("{kind}es {y}"), vec![
                pod("Input", "Input interpretation", text(format!("{kind} | {y}"))),
                primary(pod("Result", "Dates", text(short))),
                pod("Table", "Events", table(Some(vec!["event", "time"]), rows)),
            ]));
        }
        (y, ks[0])
    } else {
        // the next (or last) one from now
        let mut cands: Vec<(NaiveDateTime, i32, usize)> = Vec::new();
        for y in now.year() - 1..=now.year() + 1 {
            for &k in &ks {
                cands.push((event_utc(y, k), y, k));
            }
        }
        cands.sort();
        let pick = if matches!(dir, Some("last" | "previous")) {
            cands.iter().rev().find(|c| c.0 < now)?
        } else if dir == Some("this year's") {
            cands.iter().find(|c| c.1 == now.year())?
        } else {
            cands.iter().find(|c| c.0 >= now)?
        };
        (pick.1, pick.2)
    };
    let t = event_utc(y, k);
    let southern = NORTH[(k + 2) % 4];
    Some(Answer::new(format!("{} {y}", NAMES[k]), vec![
        pod("Input", "Input interpretation", text(format!("{} {y}", NAMES[k]))),
        primary(pod("Result", "Date and time", text(fmt(t)))),
        pod("Table", "Details", table(None, vec![
            vec![text("Northern Hemisphere"), text(NORTH[k])],
            vec![text("Southern Hemisphere"), text(southern)],
            vec![text("time from now"), text(relative(t))],
            vec![text("Julian Ephemeris Day"), text(format!("{:.5}", event_jde(y, k)))],
        ])),
        pod("Note", "Method", text("Meeus, Astronomical Algorithms, ch. 27 (accurate to about a minute)")),
    ]))
}
