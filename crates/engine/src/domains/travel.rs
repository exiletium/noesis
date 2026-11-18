//! Travel between cities: straight-line distance in a chosen unit and
//! estimated travel time by aircraft, car, train, ship, bicycle or on foot.

use super::geo::{distance_km, find_city};
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

/// mode words | label | average speed km/h | route factor over the great-circle distance | overhead hours | note
const MODES: &[(&[&str], &str, f64, f64, f64, &str)] = &[
    (&["aircraft", "airplane", "aeroplane", "plane", "air", "flight", "flying", "jet"], "aircraft", 850.0, 1.05, 0.5, "average airliner speed of 850 km/h (530 mph) along a slightly longer than great-circle route, plus 30 minutes for take-off and landing"),
    (&["car", "driving", "drive", "road", "automobile"], "car", 90.0, 1.25, 0.0, "average road speed of 90 km/h (56 mph) over a road distance about 25% longer than the straight line, without stops"),
    (&["train", "rail", "railway"], "train", 100.0, 1.2, 0.0, "average intercity train speed of 100 km/h (62 mph); high-speed lines are 2–3 times faster"),
    (&["ship", "boat", "sea", "cruise ship", "ferry", "sailing"], "ship", 37.0, 1.2, 0.0, "average ship speed of 20 knots (37 km/h) over a sea route about 20% longer than the straight line"),
    (&["bicycle", "bike", "cycling"], "bicycle", 16.0, 1.25, 0.0, "average cycling speed of 16 km/h (10 mph)"),
    (&["foot", "walking", "walk", "on foot"], "walking", 5.0, 1.25, 0.0, "average walking speed of 5 km/h (3.1 mph) without rest"),
];

fn hours_text(h: f64) -> String {
    if h >= 48.0 {
        let d = h / 24.0;
        format!("{} days ({} hours)", fmt_float((d * 10.0).round() / 10.0, 4), fmt_float(h.round(), 6))
    } else {
        let total = (h * 60.0).round() as i64;
        let (hh, mm) = (total / 60, total % 60);
        if hh == 0 { format!("{mm} minutes") } else if mm == 0 { format!("{hh} hours") } else { format!("{hh} h {mm} min") }
    }
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).to_string();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();

    // "distance from Rome to Naples in kilometers"
    if let Some(m) = re!(r"^(?:the )?(?:distance|how far)(?: is it)? (?:from|between) (.+?) (?:to|and) (.+?) in (kilometers|kilometres|km|miles|mi|nautical miles|nmi|meters|metres)$").captures(&l) {
        let (a, b) = (find_city(&m[1])?, find_city(&m[2])?);
        let km = distance_km(a, b);
        let (v, unit) = match &m[3] {
            "miles" | "mi" => (km / 1.609344, "miles"),
            "nautical miles" | "nmi" => (km / 1.852, "nautical miles"),
            "meters" | "metres" => (km * 1000.0, "meters"),
            _ => (km, "km"),
        };
        return Some(Answer::new("distance", vec![
            input(format!("distance from {} to {} in {unit}", a.name, b.name)),
            primary(pod("Result", "Distance", text(format!("{} {unit}", fmt_float((v * 10.0).round() / 10.0, 7))))),
            pod("Note", "Note", text("great-circle (straight-line) distance between the city centres")),
        ]));
    }

    // "travel time from Tokyo to Sydney by aircraft", "flight time from X to Y",
    // "estimated travel time by car from Toronto to Detroit", "how long to drive from X to Y"
    let m = re!(r"^(?:(?:estimated )?(?:travel|journey|trip) time|time to (?:travel|get|go)|how long (?:does it take )?to (?:travel|get|go)|(flight|driving|drive|sailing|walking|cycling|train) time|how long (?:is the |does it take to |to |would it take to |will it take to )?(fly|flight|drive|walk|sail|cycle|bike))(?: by ([a-z ]+?))? from (.+?) to (.+?)(?: by ([a-z ]+))?$").captures(&l)?;
    let mode_word = m.get(1).or(m.get(2)).or(m.get(3)).or(m.get(6)).map(|x| x.as_str()).unwrap_or("aircraft");
    let mode_word = match mode_word { "fly" => "flight", "sail" | "sailing" => "ship", "cycle" | "cycling" => "bike", "walk" => "walking", "driving" => "drive", x => x };
    let (_, label, speed, factor, overhead, note) = MODES.iter().find(|x| x.0.contains(&mode_word))?;
    let (a, b) = (find_city(&m[4])?, find_city(&m[5])?);
    let km = distance_km(a, b);
    let route = km * factor;
    let h = route / speed + overhead;
    let mut rows = Vec::new();
    for (_, lab, sp, f, o, _) in MODES {
        rows.push(vec![text(*lab), text(hours_text(km * f / sp + o))]);
    }
    Some(Answer::new("travel time", vec![
        input(format!("travel time from {} to {} by {label}", a.name, b.name)),
        primary(pod("Result", "Estimated travel time", text(hours_text(h)))),
        pod("Distance", "Distance", text(format!("{} km straight line ({} miles); about {} km by this route", fmt_float(km.round(), 7), fmt_float((km / 1.609344).round(), 7), fmt_float(route.round(), 7)))),
        pod("Modes", "By other means", table(Some(vec!["mode", "time"]), rows)),
        pod("Note", "Assumptions", text(*note)),
    ]))
}
