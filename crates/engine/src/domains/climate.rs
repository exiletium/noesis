//! Climate normals: 1991–2020 monthly mean temperatures at the GHCN-Monthly
//! station nearest a place, with degree days estimated from them.
//! "climate Hannover", "average temperature in Paris", "degree days Chicago".

use super::geo::{countries, find_city, home};
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

static CLIMATE_Z: &[u8] = include_bytes!("../../data/climate.z");

struct Station {
    name: &'static str,
    lat: f64,
    lon: f64,
    elev: f64,
    t: [f64; 12],
}

fn stations() -> &'static Vec<Station> {
    static S: OnceLock<Vec<Station>> = OnceLock::new();
    S.get_or_init(|| {
        let bytes = miniz_oxide::inflate::decompress_to_vec(CLIMATE_Z).unwrap_or_default();
        let text: &'static str = Box::leak(String::from_utf8(bytes).unwrap_or_default().into_boxed_str());
        text.lines()
            .filter_map(|l| {
                let c: Vec<&'static str> = l.split('\t').collect();
                if c.len() < 16 {
                    return None;
                }
                let mut t = [0.0; 12];
                for (k, v) in c[4..16].iter().enumerate() {
                    t[k] = v.parse().ok()?;
                }
                Some(Station { name: c[0], lat: c[1].parse().ok()?, lon: c[2].parse().ok()?, elev: c[3].parse().unwrap_or(0.0), t })
            })
            .collect()
    })
}

fn km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let a = ((p2 - p1) / 2.0).sin().powi(2) + p1.cos() * p2.cos() * ((lon2 - lon1).to_radians() / 2.0).sin().powi(2);
    6371.0 * 2.0 * a.sqrt().asin()
}

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const DAYS: [f64; 12] = [31.0, 28.25, 31.0, 30.0, 31.0, 30.0, 31.0, 31.0, 30.0, 31.0, 30.0, 31.0];

fn c_f(c: f64) -> String {
    format!("{} °C ({} °F)", fmt_float((c * 10.0).round() / 10.0, 4), fmt_float((c * 1.8 + 32.0).round(), 4))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?').trim();
    let (kind, month, place) = if let Some(m) = re!(r"^(?:what is |what's )?(?:the )?(climate|weather averages|average weather|climate normals|temperature normals|average temperatures?|mean temperatures?|average annual temperature|mean annual temperature|average (january|february|march|april|may|june|july|august|september|october|november|december) temperature|(?:annual )?(?:heating |cooling )?degree days)(?: (?:of|in|for|at))? ([a-z][a-z .,'-]*?)(?: in (january|february|march|april|may|june|july|august|september|october|november|december))?$").captures(l) {
        (m[1].to_string(), m.get(2).or(m.get(4)).map(|x| x.as_str().to_string()), m[3].to_string())
    } else if let Some(m) = re!(r"^([a-z][a-z .,'-]*?) (climate|average temperatures?|mean temperature|weather averages|(?:heating |cooling )?degree days)$").captures(l) {
        (m[2].to_string(), None, m[1].to_string())
    } else {
        return None;
    };
    let place = place.trim();
    let (pname, lat, lon) = if matches!(place, "here" | "my location" | "me") {
        let (n, lat, lon, _) = home();
        (n, lat, lon)
    } else {
        let c = find_city(place)?;
        (format!("{}, {}", c.name, countries().get(&c.country).map_or(c.country.clone(), |x| x.name.clone())), c.lat, c.lon)
    };
    let (st, d) = stations().iter().map(|s| (s, km(lat, lon, s.lat, s.lon))).min_by(|a, b| a.1.total_cmp(&b.1))?;
    if d > 100.0 {
        return None;
    }
    let annual: f64 = st.t.iter().zip(DAYS).map(|(t, d)| t * d).sum::<f64>() / 365.25;
    let (wi, warm) = st.t.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1))?;
    let (ci, cold) = st.t.iter().enumerate().min_by(|a, b| a.1.total_cmp(b.1))?;
    let station = format!("{} ({:.2}° {}, {:.2}° {}, {} m), {} km from {}", st.name, st.lat.abs(), if st.lat >= 0.0 { "N" } else { "S" }, st.lon.abs(), if st.lon >= 0.0 { "E" } else { "W" }, fmt_float(st.elev, 5), fmt_float(d.round(), 4), pname.split(',').next().unwrap_or(&pname));
    let rows: Vec<Vec<Content>> = (0..12).map(|k| vec![text(MONTHS[k]), text(c_f(st.t[k]))]).collect();
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{pname} | {}", if kind.contains("degree") { kind.clone() } else { "climate".into() })))];
    if kind.contains("degree") {
        // monthly-mean estimate: HDD = Σ days × max(0, 65 °F − T), CDD likewise
        let hdd: f64 = st.t.iter().zip(DAYS).map(|(t, d)| d * (65.0 - (t * 1.8 + 32.0)).max(0.0)).sum();
        let cdd: f64 = st.t.iter().zip(DAYS).map(|(t, d)| d * ((t * 1.8 + 32.0) - 65.0).max(0.0)).sum();
        let main = if kind.contains("heating") {
            format!("about {} heating degree days (°F) per year", fmt_float((hdd / 10.0).round() * 10.0, 6))
        } else if kind.contains("cooling") {
            format!("about {} cooling degree days (°F) per year", fmt_float((cdd / 10.0).round() * 10.0, 6))
        } else {
            format!("about {} heating and {} cooling degree days (°F) per year", fmt_float((hdd / 10.0).round() * 10.0, 6), fmt_float((cdd / 10.0).round() * 10.0, 6))
        };
        pods.push(primary(pod("Result", "Result", text(main))));
        let drows: Vec<Vec<Content>> = (0..12).map(|k| {
            let tf = st.t[k] * 1.8 + 32.0;
            vec![text(MONTHS[k]), text(fmt_float((DAYS[k] * (65.0 - tf).max(0.0)).round(), 5)), text(fmt_float((DAYS[k] * (tf - 65.0).max(0.0)).round(), 5))]
        }).collect();
        pods.push(pod("Monthly", "Degree days by month", table(Some(vec!["month", "heating", "cooling"]), drows)));
        pods.push(pod("Notes", "Notes", text(format!("Estimated from 1991–2020 monthly mean temperatures with a 65 °F (18.3 °C) base; day-to-day swings add degree days in spring and autumn, so true totals are somewhat higher. Station: {station}.")))) ;
        return Some(Answer::new("degree days", pods));
    }
    let main = match &month {
        Some(mn) => {
            let k = MONTHS.iter().position(|m| m.to_lowercase() == *mn)?;
            format!("{} average temperature: {}", MONTHS[k], c_f(st.t[k]))
        }
        None => format!("average temperature {} | warmest month {} ({}) | coldest month {} ({})", c_f(annual), MONTHS[wi], c_f(*warm), MONTHS[ci], c_f(*cold)),
    };
    pods.push(primary(pod("Result", "Temperature normals", text(main))));
    pods.push(pod("Monthly", "Monthly mean temperatures", table(Some(vec!["month", "mean temperature"]), rows)));
    pods.push(pod("Range", "Annual range", text(format!("{} °C ({} °F) between the coldest and warmest months", fmt_float(((warm - cold) * 10.0).round() / 10.0, 4), fmt_float(((warm - cold) * 1.8).round(), 4)))));
    pods.push(pod("Station", "Weather station", text(format!("{station}; 1991–2020 normals from GHCN-Monthly v4 (NOAA NCEI)"))));
    Some(Answer::new("climate", pods))
}
