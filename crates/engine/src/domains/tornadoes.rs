//! US tornadoes from the NOAA Storm Prediction Center's database
//! (1950–2024): "tornadoes in 2010", "F4 tornadoes in Indiana",
//! "tornadoes within 40 miles of St. Louis since 2010".

use super::geo::{find_city, home, us_state, US_STATES};
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

static TORNADOES_Z: &[u8] = include_bytes!("../../data/tornadoes.z");

struct Tornado {
    date: u32,
    state: &'static str,
    mag: i32,
    inj: u32,
    fat: u32,
    lat: f64,
    lon: f64,
    len: f64,
}

fn data() -> &'static Vec<Tornado> {
    static D: OnceLock<Vec<Tornado>> = OnceLock::new();
    D.get_or_init(|| {
        let bytes = miniz_oxide::inflate::decompress_to_vec(TORNADOES_Z).unwrap_or_default();
        let text: &'static str = Box::leak(String::from_utf8(bytes).unwrap_or_default().into_boxed_str());
        text.lines()
            .filter_map(|l| {
                let c: Vec<&'static str> = l.split('\t').collect();
                (c.len() >= 9).then(|| Tornado { date: c[0].parse().unwrap_or(0), state: c[1], mag: c[2].parse().unwrap_or(-9), inj: c[3].parse().unwrap_or(0), fat: c[4].parse().unwrap_or(0), lat: c[5].parse().unwrap_or(0.0), lon: c[6].parse().unwrap_or(0.0), len: c[7].parse().unwrap_or(0.0) })
            })
            .collect()
    })
}

fn miles(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let a = ((p2 - p1) / 2.0).sin().powi(2) + p1.cos() * p2.cos() * ((lon2 - lon1).to_radians() / 2.0).sin().powi(2);
    3958.8 * 2.0 * a.sqrt().asin()
}

fn scale(t: &Tornado) -> String {
    if t.mag < 0 {
        return "unrated".into();
    }
    format!("{}{}", if t.date >= 20070201 { "EF" } else { "F" }, t.mag)
}

fn date(d: u32) -> String {
    const M: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    format!("{} {}, {}", M[((d / 100 % 100).max(1) - 1) as usize % 12], d % 100, d / 10000)
}

fn int(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?').trim();
    let m = re!(r"^(?:how many |list of |all |show (?:me )?)?(?:(e?f)\s?(\d)(\+)? |(violent|strong|significant|killer|deadly) )?tornado(?:es|s)?(?: (?:were there|occurred|have there been|happened))?(.*)$").captures(l)?;
    let mut rest = m[5].trim().to_string();
    let mut mag_min: Option<i32> = None;
    let mut mag_exact: Option<i32> = m.get(2).and_then(|x| x.as_str().parse().ok());
    if m.get(3).is_some() {
        mag_min = mag_exact.take();
    }
    match m.get(4).map(|x| x.as_str()) {
        Some("violent") => mag_min = Some(4),
        Some("strong" | "significant") => mag_min = Some(2),
        _ => {}
    }
    let killer = matches!(m.get(4).map(|x| x.as_str()), Some("killer" | "deadly"));
    let (mut y0, mut y1) = (1950u32, 2024u32);
    let mut state: Option<&'static str> = None;
    let mut near: Option<(String, f64, f64, f64)> = None;
    // peel off the qualifiers one at a time
    for _ in 0..6 {
        rest = rest.trim().trim_start_matches(',').trim().to_string();
        if rest.is_empty() {
            break;
        }
        if let Some(c) = re!(r"^(?:in|during) (\d{4})\b(.*)$").captures(&rest) {
            y0 = c[1].parse().ok()?;
            y1 = y0;
            rest = c[2].to_string();
        } else if let Some(c) = re!(r"^(?:since|after|from) (\d{4})\b(.*)$").captures(&rest) {
            y0 = c[1].parse::<u32>().ok()? + if rest.starts_with("after") { 1 } else { 0 };
            rest = c[2].to_string();
            if let Some(c2) = re!(r"^\s*(?:to|through|until|-) (\d{4})\b(.*)$").captures(&rest) {
                y1 = c2[1].parse().ok()?;
                rest = c2[2].to_string();
            }
        } else if let Some(c) = re!(r"^before (\d{4})\b(.*)$").captures(&rest) {
            y1 = c[1].parse::<u32>().ok()? - 1;
            rest = c[2].to_string();
        } else if let Some(c) = re!(r"^(?:within|in a radius of) (\d+(?:\.\d+)?) ?(miles?|mi|km|kilometers?) (?:of|from|around) (.+?)((?: (?:since|after|before|in|during|from) \d{4}).*)?$").captures(&rest) {
            let r: f64 = c[1].parse().ok()?;
            let r = if c[2].starts_with('k') { r / 1.609344 } else { r };
            let (name, lat, lon) = place(&c[3])?;
            near = Some((name, lat, lon, r));
            rest = c.get(4).map_or(String::new(), |x| x.as_str().to_string());
        } else if let Some(c) = re!(r"^near (.+?)((?: (?:since|after|before|in|during|from) \d{4}).*)?$").captures(&rest) {
            let (name, lat, lon) = place(&c[1])?;
            near = Some((name, lat, lon, 25.0));
            rest = c.get(2).map_or(String::new(), |x| x.as_str().to_string());
        } else if let Some(c) = re!(r"^in (?:the )?(us|u\.s\.|usa|united states)\b(.*)$").captures(&rest) {
            rest = c[2].to_string();
        } else if let Some(c) = re!(r"^in ([a-z .]+?)((?: (?:since|after|before|in|during|from) \d{4}).*)?$").captures(&rest) {
            state = Some(us_state(&c[1])?);
            rest = c.get(2).map_or(String::new(), |x| x.as_str().to_string());
        } else {
            return None;
        }
    }
    if !rest.trim().is_empty() || y0 > y1 || y0 < 1950 {
        return None;
    }
    let list: Vec<&Tornado> = data()
        .iter()
        .filter(|t| {
            let y = t.date / 10000;
            y >= y0 && y <= y1
                && state.is_none_or(|s| t.state == s)
                && mag_exact.is_none_or(|k| t.mag == k)
                && mag_min.is_none_or(|k| t.mag >= k)
                && (!killer || t.fat > 0)
                && near.as_ref().is_none_or(|(_, la, lo, r)| miles(*la, *lo, t.lat, t.lon) <= *r)
        })
        .collect();
    let mut what = String::new();
    if let Some(k) = mag_exact {
        what.push_str(&format!("(E)F{k} "));
    } else if let Some(k) = mag_min {
        what.push_str(&format!("(E)F{k}+ "));
    } else if killer {
        what.push_str("killer ");
    }
    what.push_str("tornadoes");
    let mut wher = match (&near, state) {
        (Some((n, _, _, r)), _) => format!("within {} miles of {n}", fmt_float(*r, 4)),
        (None, Some(s)) => US_STATES.iter().find(|x| x.0 == s).map_or(s.to_string(), |x| x.1.to_string()),
        _ => "United States".into(),
    };
    wher.push_str(&if y0 == y1 { format!(" | {y0}") } else { format!(" | {y0}–{y1}") });
    let n = list.len();
    let fat: u64 = list.iter().map(|t| t.fat as u64).sum();
    let inj: u64 = list.iter().map(|t| t.inj as u64).sum();
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{what} | {wher}"))),
        primary(pod("Result", "Number of tornadoes", text(format!("{} tornadoes", int(n as u64))))),
    ];
    if n == 0 {
        pods.push(pod("Source", "Source", text("NOAA Storm Prediction Center tornado database, 1950–2024.")));
        return Some(Answer::new("tornadoes", pods));
    }
    let by_mag: Vec<Vec<Content>> = (-1..=5)
        .filter_map(|k| {
            let c = list.iter().filter(|t| if k < 0 { t.mag < 0 } else { t.mag == k }).count();
            (c > 0).then(|| vec![text(if k < 0 { "unrated".to_string() } else { format!("(E)F{k}") }), text(int(c as u64))])
        })
        .collect();
    pods.push(pod("Magnitude", "By magnitude", table(Some(vec!["rating", "tornadoes"]), by_mag)));
    pods.push(pod("Casualties", "Casualties", text(format!("{} deaths, {} injuries", int(fat), int(inj)))));
    let years = (y1 - y0 + 1) as f64;
    if y1 > y0 {
        pods.push(pod("Rate", "Average per year", text(format!("{} tornadoes per year", fmt_float((n as f64 / years * 10.0).round() / 10.0, 5)))));
    }
    if state.is_none() && near.is_none() {
        let mut st: Vec<(&str, usize)> = Vec::new();
        for t in &list {
            match st.iter_mut().find(|x| x.0 == t.state) {
                Some(x) => x.1 += 1,
                None => st.push((t.state, 1)),
            }
        }
        st.sort_by(|a, b| b.1.cmp(&a.1));
        pods.push(pod("States", "Most tornadoes by state", text(st.iter().take(5).map(|(s, c)| format!("{s}: {}", int(*c as u64))).collect::<Vec<_>>().join(" | "))));
    }
    let mut notable: Vec<&&Tornado> = list.iter().collect();
    notable.sort_by(|a, b| (b.fat, b.mag, b.len as u64).cmp(&(a.fat, a.mag, a.len as u64)));
    let rows: Vec<Vec<Content>> = notable
        .iter()
        .take(12)
        .map(|t| vec![text(date(t.date)), text(t.state), text(scale(t)), text(format!("{:.2}° N, {:.2}° W", t.lat, -t.lon)), text(format!("{} mi", fmt_float(t.len, 4))), text(t.fat.to_string()), text(t.inj.to_string())])
        .collect();
    pods.push(pod("Notable", if n > 12 { "Deadliest and strongest" } else { "Tornadoes" }, table(Some(vec!["date", "state", "rating", "start", "path length", "deaths", "injuries"]), rows)));
    pods.push(pod("Source", "Source", text("NOAA Storm Prediction Center tornado database, 1950–2024 (F scale before February 2007, Enhanced Fujita scale after); a tornado that crossed a state line is counted in the state where it began.")));
    Some(Answer::new("tornadoes", pods))
}

fn place(s: &str) -> Option<(String, f64, f64)> {
    let s = s.trim();
    if matches!(s, "me" | "here" | "my location") {
        let (n, lat, lon, _) = home();
        return Some((n, lat, lon));
    }
    let c = find_city(s)?;
    Some((c.name.clone(), c.lat, c.lon))
}
