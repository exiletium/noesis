//! Airports from OurAirports (public domain): IATA/ICAO codes, location,
//! elevation and runways, distances and flight times between airports, and
//! airports near a city.
//!
//! "LAX", "JFK, LGA, EWR", "Heathrow airport", "elevation of Denver airport",
//! "distance from JFK to LAX", "flight time from SFO to NYC",
//! "airports near Chicago".
//!
//! `tools/datagen/airports.py` builds `data/airports.z`.

use super::geo::{countries, find_city, City};
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

static AIRPORTS_Z: &[u8] = include_bytes!("../../data/airports.z");

pub struct Airport {
    pub iata: &'static str,
    pub icao: &'static str,
    /// 'L' large, 'M' medium, 'S' small with scheduled service
    pub size: char,
    pub name: &'static str,
    pub city: &'static str,
    pub country: &'static str,
    pub region: &'static str,
    pub lat: f64,
    pub lon: f64,
    pub elevation_ft: Option<f64>,
    pub runways: u32,
    pub longest_ft: Option<f64>,
    pub surface: &'static str,
}

fn data() -> &'static Vec<Airport> {
    static D: OnceLock<Vec<Airport>> = OnceLock::new();
    D.get_or_init(|| {
        let bytes = miniz_oxide::inflate::decompress_to_vec(AIRPORTS_Z).unwrap_or_default();
        let text: &'static str = Box::leak(String::from_utf8(bytes).unwrap_or_default().into_boxed_str());
        text.lines()
            .filter_map(|line| {
                let c: Vec<&'static str> = line.split('\t').collect();
                if c.len() < 13 {
                    return None;
                }
                Some(Airport {
                    iata: c[0],
                    icao: c[1],
                    size: c[2].chars().next().unwrap_or('S'),
                    name: c[3],
                    city: c[4],
                    country: c[5],
                    region: c[6],
                    lat: c[7].parse().ok()?,
                    lon: c[8].parse().ok()?,
                    elevation_ft: c[9].parse().ok(),
                    runways: c[10].parse().unwrap_or(0),
                    longest_ft: c[11].parse().ok(),
                    surface: c[12],
                })
            })
            .collect()
    })
}

/// Codes of large airports that read first as something else when typed alone.
const NOT_BARE: &[&str] = &[
    "AES", "BCD", "CIA", "CNN", "COS", "COV", "DNA", "ERF", "GSM", "HKD", "IST", "LED", "MAD", "NAN", "NAS", "PIE", "RHO", "ROC", "SIN",
    "STI", "TBS", "TLS", "TSA", "UTP", "VAR",
];

fn by_code(code: &str) -> Option<&'static Airport> {
    let u = code.trim().to_uppercase();
    match u.len() {
        3 => data().binary_search_by(|a| a.iata.cmp(&u)).ok().map(|i| &data()[i]),
        4 => data().iter().find(|a| a.icao == u),
        _ => None,
    }
}

fn rank(a: &Airport) -> u8 {
    match a.size {
        'L' => 0,
        'M' => 1,
        _ => 2,
    }
}

/// "Heathrow", "O'Hare", "Denver", "Kennedy" (the text before "airport").
fn by_name(s: &str) -> Option<&'static Airport> {
    let l = s.trim().to_lowercase().replace('’', "'");
    let l = l.trim_start_matches("the ").trim();
    let l = re!(r"\s+(?:international|intl\.?|regional|municipal)$").replace(l, "").to_string();
    if l.len() < 3 {
        return None;
    }
    if let Some(a) = by_code(&l) {
        return Some(a);
    }
    let word = Regex::new(&format!(r"\b{}\b", regex::escape(&l))).ok()?;
    let in_name = data().iter().filter(|a| a.size != 'S' && word.is_match(&a.name.to_lowercase())).min_by_key(|a| rank(a));
    let in_city = data().iter().filter(|a| a.size != 'S' && a.city.to_lowercase() == l).min_by_key(|a| rank(a));
    match (in_name, in_city) {
        (Some(n), Some(c)) => Some(if rank(c) < rank(n) { c } else { n }),
        (n, c) => n.or(c),
    }
}

fn is_code(raw: &str) -> bool {
    re!(r"^[A-Z]{3,4}$").is_match(raw)
}

/// An airport named in a query: an upper-case code, "X airport" or "airport X".
fn named(raw: &str) -> Option<&'static Airport> {
    let r = raw.trim().trim_end_matches(['?', '.']).trim();
    if is_code(r) {
        return by_code(r).filter(|a| a.size != 'S');
    }
    let l = r.to_lowercase();
    if let Some(m) = re!(r"^(?:the )?(.+?) (?:international )?airport$").captures(&l).or_else(|| re!(r"^airport (?:code )?(.+)$").captures(&l)) {
        let inner = m[1].trim();
        // keep the original case to read codes ("JFK airport")
        let orig = &r[r.to_lowercase().find(inner).unwrap_or(0)..][..inner.len()];
        return if is_code(orig) || inner.len() == 3 { by_code(inner).or_else(|| by_name(inner)) } else { by_name(inner) };
    }
    None
}

fn km_between(a: (f64, f64), b: (f64, f64)) -> f64 {
    let r = 6371.0088;
    let (p1, p2) = (a.0.to_radians(), b.0.to_radians());
    let dl = (b.1 - a.1).to_radians();
    let h = ((p2 - p1) / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
    2.0 * r * h.sqrt().asin()
}

fn country_name(code: &str) -> String {
    countries().get(code).map(|c| c.name.clone()).unwrap_or_else(|| code.to_string())
}

fn place(a: &Airport) -> String {
    let region = a.region.split_once('-').map(|x| x.1).unwrap_or("");
    let region = if a.country == "US" || a.country == "CA" || a.country == "AU" { format!(", {region}") } else { String::new() };
    let city = if a.city.is_empty() { String::new() } else { format!("{}{region}, ", a.city) };
    format!("{city}{}", country_name(a.country))
}

fn ft_m(ft: f64) -> String {
    format!("{} ft ({} m)", fmt_float(ft.round(), 7), fmt_float((ft * 0.3048).round(), 7))
}

fn coords(lat: f64, lon: f64) -> String {
    format!("{:.2}° {}, {:.2}° {}", lat.abs(), if lat >= 0.0 { "N" } else { "S" }, lon.abs(), if lon >= 0.0 { "E" } else { "W" })
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn source() -> Pod {
    pod("Source", "Source", text("OurAirports (public domain)"))
}

fn facts(a: &Airport) -> Vec<(String, String)> {
    let mut r = vec![
        ("name".to_string(), a.name.to_string()),
        ("IATA code".to_string(), a.iata.to_string()),
    ];
    if !a.icao.is_empty() {
        r.push(("ICAO code".into(), a.icao.to_string()));
    }
    r.push(("location".into(), place(a)));
    r.push(("coordinates".into(), coords(a.lat, a.lon)));
    if let Some(e) = a.elevation_ft {
        r.push(("elevation".into(), ft_m(e)));
    }
    if a.runways > 0 {
        r.push(("runways".into(), a.runways.to_string()));
    }
    if let Some(len) = a.longest_ft {
        let surf = if a.surface.is_empty() { String::new() } else { format!(", {}", a.surface) };
        r.push(("longest runway".into(), format!("{}{surf}", ft_m(len))));
    }
    r.push(("type".into(), match a.size { 'L' => "large airport", 'M' => "medium airport", _ => "small airport with scheduled service" }.to_string()));
    r
}

fn card(a: &Airport, prop: Option<&str>) -> Answer {
    let f = facts(a);
    let (title, value) = prop
        .and_then(|p| f.iter().find(|(k, _)| k.as_str() == p).map(|(k, v)| (k.clone(), v.clone())))
        .unwrap_or_else(|| ("Airport".to_string(), format!("{} ({}) — {}", a.name, a.iata, place(a))));
    let title = title[..1].to_uppercase() + &title[1..];
    Answer::new("airport", vec![
        input(format!("{} ({}){}", a.name, a.iata, prop.map(|p| format!(" | {p}")).unwrap_or_default())),
        primary(pod("Result", &title, text(value))),
        pod("Table", "Basic information", table(None, f.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())),
        source(),
    ])
}

fn list(aps: &[&'static Airport]) -> Answer {
    let rows = aps
        .iter()
        .map(|a| vec![text(a.iata), text(a.name), text(place(a)), text(a.elevation_ft.map(ft_m).unwrap_or_default()), text(a.longest_ft.map(ft_m).unwrap_or_default())])
        .collect();
    let mut pods = vec![
        input(aps.iter().map(|a| a.iata).collect::<Vec<_>>().join(" | ")),
        primary(pod("Result", "Airports", text(aps.iter().map(|a| format!("{} ({})", a.name, a.iata)).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Basic information", table(Some(vec!["code", "name", "location", "elevation", "longest runway"]), rows)),
    ];
    if aps.len() <= 6 {
        let mut d = vec![];
        for i in 0..aps.len() {
            for j in i + 1..aps.len() {
                let km = km_between((aps[i].lat, aps[i].lon), (aps[j].lat, aps[j].lon));
                d.push(vec![text(format!("{} – {}", aps[i].iata, aps[j].iata)), text(format!("{} km ({} mi)", fmt_float(km.round(), 7), fmt_float((km / 1.609344).round(), 7)))]);
            }
        }
        pods.push(pod("Distances", "Distances", table(Some(vec!["pair", "great-circle distance"]), d)));
    }
    pods.push(source());
    Answer::new("airports", pods)
}

/// An endpoint of a trip: an airport, or else a city.
enum End {
    Port(&'static Airport),
    City(&'static City),
}

impl End {
    fn pos(&self) -> (f64, f64) {
        match self {
            End::Port(a) => (a.lat, a.lon),
            End::City(c) => (c.lat, c.lon),
        }
    }
    fn label(&self) -> String {
        match self {
            End::Port(a) => format!("{} ({})", a.name, a.iata),
            End::City(c) => c.name.clone(),
        }
    }
}

fn endpoint(raw: &str) -> Option<End> {
    named(raw).map(End::Port).or_else(|| find_city(raw).map(End::City))
}

fn hours_text(h: f64) -> String {
    let total = (h * 60.0).round() as i64;
    let (hh, mm) = (total / 60, total % 60);
    if hh == 0 { format!("{mm} minutes") } else if mm == 0 { format!("{hh} hours") } else { format!("{hh} h {mm} min") }
}

fn trip(raw: &str) -> Option<Answer> {
    let r = raw.trim().trim_end_matches(['?', '.']).trim();
    let r = re!(r"\s+").replace_all(r, " ").to_string();
    let m = re!(r"(?i)^(?:the )?(flight (?:time|distance|duration)|distance|how far(?: is it)?|flights?|how long (?:is the |does it take to |to )?fly(?:ing)?|fly)(?: from| between)? (.+?) (?:to|and|-) (.+?)(?: in (km|kilometers|kilometres|miles|mi|nautical miles|nmi))?$").captures(&r)?;
    let (a, b) = (endpoint(&m[2])?, endpoint(&m[3])?);
    if !matches!(a, End::Port(_)) && !matches!(b, End::Port(_)) {
        return None;
    }
    let km = km_between(a.pos(), b.pos());
    let dist = match m.get(4).map(|x| x.as_str().to_lowercase()).as_deref() {
        Some("miles" | "mi") => format!("{} miles", fmt_float((km / 1.609344).round(), 7)),
        Some("nautical miles" | "nmi") => format!("{} nmi", fmt_float((km / 1.852).round(), 7)),
        _ => format!("{} km ({} miles, {} nmi)", fmt_float(km.round(), 7), fmt_float((km / 1.609344).round(), 7), fmt_float((km / 1.852).round(), 7)),
    };
    let hours = km * 1.05 / 850.0 + 0.5;
    let kind = m[1].to_lowercase();
    let time_first = kind.starts_with("how long") || kind.contains("time") || kind.contains("duration") || kind.starts_with("flight") || kind == "fly";
    let (dp, tp) = (pod("Distance", "Great-circle distance", text(dist)), pod("Time", "Typical flight time", text(hours_text(hours))));
    let (first, second) = if time_first { (tp, dp) } else { (dp, tp) };
    Some(Answer::new("flight", vec![
        input(format!("{} → {}", a.label(), b.label())),
        primary(first),
        second,
        pod("Note", "Assumptions", text("great-circle route; flight time at an average airliner speed of 850 km/h (530 mph) over a route 5% longer, plus 30 minutes for take-off and landing")),
        source(),
    ]))
}

fn near(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(?:the )?(?:main |major |nearest |closest )?airports?|airport codes?) (?:near|in|around|for|serving|close to|nearest to|closest to) (.+)$").captures(l)?;
    let c = find_city(&m[1])?;
    let mut v: Vec<(&'static Airport, f64)> = data()
        .iter()
        .filter(|a| a.size != 'S')
        .map(|a| (a, km_between((a.lat, a.lon), (c.lat, c.lon))))
        .filter(|(a, d)| *d < if a.size == 'L' { 90.0 } else { 50.0 })
        .collect();
    v.sort_by(|x, y| rank(x.0).cmp(&rank(y.0)).then(x.1.total_cmp(&y.1)));
    v.truncate(8);
    if v.is_empty() {
        return None;
    }
    let rows = v
        .iter()
        .map(|(a, d)| vec![text(a.iata), text(a.name), text(format!("{} km ({} mi)", fmt_float(d.round(), 5), fmt_float((d / 1.609344).round(), 5))), text(if a.size == 'L' { "large" } else { "medium" })])
        .collect();
    Some(Answer::new("airports", vec![
        input(format!("airports near {}", c.name)),
        primary(pod("Result", "Airports", text(v.iter().map(|(a, _)| format!("{} ({})", a.name, a.iata)).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Airports", table(Some(vec!["code", "name", "distance from the centre", "size"]), rows)),
        source(),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let raw = q.raw.trim().trim_end_matches(['?', '.', '!']).trim();
    // a bare upper-case code: "LAX", "EGLL"
    if is_code(raw) {
        let a = by_code(raw).filter(|a| a.size == 'L' && !NOT_BARE.contains(&a.iata) && (raw.len() == 3 || a.icao == raw))?;
        return Some(card(a, None));
    }
    // "JFK, LGA, EWR"
    if re!(r"^[A-Z]{3}(?:\s*,\s*[A-Z]{3}|\s+and\s+[A-Z]{3})+$").is_match(raw) {
        let aps: Option<Vec<_>> = re!(r"[A-Z]{3}").find_iter(raw).map(|m| by_code(m.as_str()).filter(|a| a.size != 'S')).collect();
        return aps.map(|v| list(&v));
    }
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    if !l.contains("airport") && !l.contains("fly") && !l.contains("flight") && !l.contains("distance") && !l.contains("how far") && !l.contains("runway") && !l.contains("icao") && !l.contains("iata") {
        return None;
    }
    if let Some(a) = near(&l) {
        return Some(a);
    }
    if let Some(a) = trip(raw) {
        return Some(a);
    }
    // "elevation of Denver airport", "JFK airport", "what airport is ORD"
    let mut s = raw.to_string();
    for p in [r"(?i)^(?:what|which) airport is ", r"(?i)^(?:what is |what's )?(?:the )?"] {
        s = Regex::new(p).unwrap().replace(&s, "").to_string();
    }
    if let Some(a) = by_code(&s).filter(|_| is_code(&s) && l.starts_with("what")) {
        return Some(card(a, None));
    }
    let props = [
        ("elevation", "elevation"), ("altitude", "elevation"), ("height", "elevation"), ("location", "location"), ("coordinates", "coordinates"),
        ("longest runway", "longest runway"), ("runway length", "longest runway"), ("number of runways", "runways"), ("runways", "runways"),
        ("icao code", "ICAO code"), ("icao", "ICAO code"), ("iata code", "IATA code"), ("airport code", "IATA code"), ("code", "IATA code"),
    ];
    for (w, key) in props {
        if let Some(m) = Regex::new(&format!(r"(?i)^(?:how many )?{w} (?:of|at|for|does) (.+?)(?: have)?$")).unwrap().captures(&s) {
            if let Some(a) = named(&m[1]).or_else(|| (key == "IATA code" || key == "ICAO code").then(|| by_name(&m[1])).flatten()) {
                return Some(card(a, Some(key)));
            }
        }
    }
    named(&s).map(|a| card(a, None))
}
