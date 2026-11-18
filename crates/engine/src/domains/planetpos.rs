//! Apparent geocentric right ascension and declination of the Sun and the
//! planets, from the low-precision ephemeris (errors of a few arcminutes).
//!
//! "Sun right ascension and declination", "where is Jupiter in the sky",
//! "right ascension of Mars", "Venus position on 2030-01-01".

use crate::engine::*;
use crate::print::fmt_float;
use chrono::{TimeZone, Utc};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

const BODIES: &[&str] = &["sun", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune"];

/// Zodiac constellations by ecliptic longitude (IAU boundaries, approximate, J2000).
const ZODIAC: &[(f64, &str)] = &[
    (28.7, "Pisces"), (53.4, "Aries"), (90.1, "Taurus"), (118.0, "Gemini"), (138.0, "Cancer"), (173.9, "Leo"), (217.8, "Virgo"),
    (241.0, "Libra"), (247.7, "Scorpius"), (266.3, "Ophiuchus"), (299.7, "Sagittarius"), (327.9, "Capricornus"), (351.6, "Aquarius"), (360.0, "Pisces"),
];

fn hms(deg: f64) -> String {
    let h = deg.rem_euclid(360.0) / 15.0;
    let hh = h.floor();
    let mm = ((h - hh) * 60.0).floor();
    let ss = ((h - hh) * 60.0 - mm) * 60.0;
    format!("{}h {}m {}s", hh as i64, mm as i64, fmt_float((ss * 10.0).round() / 10.0, 3))
}

fn dms(deg: f64) -> String {
    let sign = if deg < 0.0 { "−" } else { "+" };
    let d = deg.abs();
    let dd = d.floor();
    let mm = ((d - dd) * 60.0).floor();
    let ss = ((d - dd) * 60.0 - mm) * 60.0;
    format!("{sign}{}° {}′ {}″", dd as i64, mm as i64, ss.round() as i64)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let m = re!(r"^(?:what is |what's )?(?:the )?(?:(right ascension and declination|ra and dec|right ascension|declination|position|coordinates|celestial coordinates|sky position|location) of (?:the )?(sun|mercury|venus|mars|jupiter|saturn|uranus|neptune)|(sun|mercury|venus|mars|jupiter|saturn|uranus|neptune)(?:'s)? (right ascension and declination|right ascension|declination|ra and dec|position|coordinates|celestial coordinates|sky position|position in the sky)|where is (?:the )?(sun|mercury|venus|mars|jupiter|saturn|uranus|neptune) in the sky(?: now| tonight| right now)?)(?: (?:now|today|right now|on (.+)))?$").captures(&l)?;
    let body = m.get(2).or(m.get(3)).or(m.get(5))?.as_str();
    let what = m.get(1).or(m.get(4)).map(|x| x.as_str()).unwrap_or("position");
    if !BODIES.contains(&body) {
        return None;
    }
    let at = match m.get(6) {
        Some(d) => {
            let date = super::misc::parse_date(d.as_str())?;
            Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0)?)
        }
        None => Utc::now(),
    };
    let t = super::ephem::centuries(at);
    let earth = super::ephem::heliocentric("Earth", t)?;
    let p = if body == "sun" {
        [0.0, 0.0, 0.0]
    } else {
        let mut c = body.chars();
        let name = c.next()?.to_uppercase().collect::<String>() + c.as_str();
        super::ephem::heliocentric(&name, t)?
    };
    let g = [p[0] - earth[0], p[1] - earth[1], p[2] - earth[2]];
    let dist = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
    let lon = g[1].atan2(g[0]).to_degrees().rem_euclid(360.0);
    // the elements are referred to the J2000 ecliptic and equinox
    let eps = 23.439_291_f64.to_radians();
    let (x, y, z) = (g[0], g[1] * eps.cos() - g[2] * eps.sin(), g[1] * eps.sin() + g[2] * eps.cos());
    let ra = y.atan2(x).to_degrees().rem_euclid(360.0);
    let dec = (z / dist).asin().to_degrees();
    let constellation = ZODIAC.iter().find(|(b, _)| lon < *b).map(|z| z.1).unwrap_or("Pisces");
    let elong = if body == "sun" {
        0.0
    } else {
        let s = [-earth[0], -earth[1], -earth[2]];
        let sd = (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
        ((g[0] * s[0] + g[1] * s[1] + g[2] * s[2]) / (dist * sd)).clamp(-1.0, 1.0).acos().to_degrees()
    };
    let disp = {
        let mut c = body.chars();
        c.next()?.to_uppercase().collect::<String>() + c.as_str()
    };
    let result = match what {
        "right ascension" => hms(ra),
        "declination" => dms(dec),
        "right ascension and declination" | "ra and dec" => format!("RA {} | Dec {}", hms(ra), dms(dec)),
        _ => format!("RA {} | Dec {} | in {constellation}", hms(ra), dms(dec)),
    };
    let mut rows = vec![
        vec![text("right ascension"), text(hms(ra))],
        vec![text("declination"), text(dms(dec))],
        vec![text("ecliptic longitude"), text(format!("{}°", fmt_float((lon * 100.0).round() / 100.0, 5)))],
        vec![text("constellation"), text(constellation)],
        vec![text("distance from Earth"), text(format!("{} au", fmt_float((dist * 10000.0).round() / 10000.0, 6)))],
    ];
    if body != "sun" {
        rows.push(vec![text("elongation from the Sun"), text(format!("{}°", fmt_float(elong.round(), 4)))]);
    }
    Some(Answer::new(format!("{disp} position"), vec![
        pod("Input", "Input interpretation", text(format!("{disp} | geocentric equatorial coordinates | {}", at.format("%B %-d, %Y, %H:%M UTC")))),
        primary(pod("Result", "Position", text(result))),
        pod("Table", "Coordinates (J2000, geometric)", table(None, rows)),
        pod("Note", "Accuracy", text("low-precision ephemeris (JPL mean elements); errors of a few arcminutes")),
    ]))
}
