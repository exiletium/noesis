//! Rising, transit and setting of the Sun and planets for a place, and their
//! local hour angles.
//!
//! "next rise of Venus from Rome", "when does Mars set in Paris",
//! "Jupiter transit time in Tokyo", "local hour angle of Mars".

use super::geo::{self, find_city};
use crate::engine::*;
use crate::print::fmt_float;
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

const BODY: &str = "(sun|mercury|venus|mars|jupiter|saturn|uranus|neptune)";

fn jd(t: DateTime<Utc>) -> f64 {
    t.timestamp() as f64 / 86400.0 + 2440587.5
}

/// Local sidereal time in degrees.
fn lst(t: DateTime<Utc>, lon: f64) -> f64 {
    let d = jd(t) - 2451545.0;
    (280.46061837 + 360.98564736629 * d + lon).rem_euclid(360.0)
}

/// Hour angle (degrees, −180..180), altitude and azimuth (from north, eastwards).
fn horizontal(body: &str, t: DateTime<Utc>, lat: f64, lon: f64) -> Option<(f64, f64, f64)> {
    let (ra, dec, _) = super::planetpos::radec(body, t)?;
    let h = (lst(t, lon) - ra + 540.0).rem_euclid(360.0) - 180.0;
    let (hr, dr, pr) = (h.to_radians(), dec.to_radians(), lat.to_radians());
    let alt = (pr.sin() * dr.sin() + pr.cos() * dr.cos() * hr.cos()).asin().to_degrees();
    let az = (hr.sin().atan2(hr.cos() * pr.sin() - dr.tan() * pr.cos()).to_degrees() + 180.0).rem_euclid(360.0);
    Some((h, alt, az))
}

/// The next time after `from` at which f changes sign from `up`'s side, by scan and bisection.
fn next_crossing(f: &dyn Fn(DateTime<Utc>) -> Option<f64>, from: DateTime<Utc>, up: bool, hours: i64) -> Option<DateTime<Utc>> {
    let step = Duration::minutes(10);
    let mut t0 = from;
    let mut v0 = f(t0)?;
    for _ in 0..(hours * 6) {
        let t1 = t0 + step;
        let v1 = f(t1)?;
        if (up && v0 < 0.0 && v1 >= 0.0) || (!up && v0 >= 0.0 && v1 < 0.0) {
            let (mut a, mut b) = (t0, t1);
            for _ in 0..20 {
                let mid = a + (b - a) / 2;
                let vm = f(mid)?;
                if (vm >= 0.0) == up {
                    b = mid;
                } else {
                    a = mid;
                }
            }
            return Some(b);
        }
        t0 = t1;
        v0 = v1;
    }
    None
}

struct Place {
    name: String,
    lat: f64,
    lon: f64,
    tz: chrono_tz::Tz,
    assumed: bool,
}

fn place(s: Option<&str>) -> Option<Place> {
    let (name, lat, lon, tz, assumed) = match s {
        Some(p) => {
            let c = find_city(p.trim())?;
            (format!("{}, {}", c.name, geo::countries().get(&c.country).map_or(c.country.clone(), |x| x.name.clone())), c.lat, c.lon, c.timezone.clone(), false)
        }
        None => {
            let (n, la, lo, tz) = geo::home();
            (n, la, lo, tz, true)
        }
    };
    Some(Place { name, lat, lon, tz: tz.parse().ok()?, assumed })
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str())
}

fn hms(deg: f64) -> String {
    let h = deg.rem_euclid(360.0) / 15.0;
    let s = (h * 3600.0).round() as i64;
    format!("{}h {}m {}s", s / 3600, s / 60 % 60, s % 60)
}

/// "Islamic prayer times", "prayer times in Cairo": Muslim World League angles
/// (Fajr 18°, Isha 17° below the horizon) and the standard (Shafi'i) Asr shadow.
fn prayer_times(l: &str) -> Option<Answer> {
    use chrono::TimeZone;
    let m = re!(r"^(?:islamic |muslim |salat |salah )?(?:prayer times?|prayer schedule|namaz times?|salat times?)(?: (?:in|for|at) (.+?))?(?: today| now)?$").captures(l)?;
    let p = place(m.get(1).map(|x| x.as_str()))?;
    let today = Utc::now().with_timezone(&p.tz).date_naive();
    let start = p.tz.from_local_datetime(&today.and_hms_opt(0, 0, 0)?).earliest()?.with_timezone(&Utc);
    let sun = |t: DateTime<Utc>| horizontal("sun", t, p.lat, p.lon);
    let below = |deg: f64| move |t: DateTime<Utc>| sun(t).map(|x| x.1 + deg);
    let fajr = next_crossing(&below(18.0), start, true, 24);
    let sunrise = next_crossing(&below(0.8333), start, true, 24);
    let ha = |t: DateTime<Utc>| sun(t).map(|x| if x.0.abs() > 170.0 { -1.0 } else { x.0 });
    let dhuhr = next_crossing(&ha, start, true, 24)?;
    let noon_alt = sun(dhuhr)?.1;
    let zen = (90.0 - noon_alt).to_radians();
    let asr_alt = (1.0 / (1.0 + zen.tan())).atan().to_degrees();
    let asr = next_crossing(&move |t: DateTime<Utc>| sun(t).map(|x| x.1 - asr_alt), dhuhr, false, 12);
    let maghrib = next_crossing(&below(0.8333), dhuhr, false, 12);
    let isha = next_crossing(&below(17.0), dhuhr, false, 14);
    let fmt = |t: Option<DateTime<Utc>>| t.map_or("— (the Sun does not reach this angle today)".to_string(), |t| t.with_timezone(&p.tz).format("%-I:%M %p").to_string());
    let rows: Vec<Vec<Content>> = [("Fajr", fajr), ("Sunrise", sunrise), ("Dhuhr", Some(dhuhr)), ("Asr", asr), ("Maghrib", maghrib), ("Isha", isha)].iter().map(|(n, t)| vec![text(*n), text(fmt(*t))]).collect();
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("Islamic prayer times | {} | {}", p.name, today.format("%A, %B %-d, %Y")))),
        primary(pod("Result", "Prayer times", text(format!("Fajr {} | Dhuhr {} | Asr {} | Maghrib {} | Isha {}", fmt(fajr), fmt(Some(dhuhr)), fmt(asr), fmt(maghrib), fmt(isha))))),
        pod("Table", "Schedule", table(None, rows)),
        pod("Note", "Method", text("Muslim World League convention: Fajr at 18° and Isha at 17° solar depression, Asr when an object's shadow equals its length plus its noon shadow, Maghrib at sunset; local time, flat horizon")),
    ];
    if p.assumed {
        pods.push(pod("Assumption", "Assumption", text(format!("location: {} (no place given; set NOESIS_LOCATION to a city name to change)", p.name))));
    }
    Some(Answer::new("prayer times", pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    if let Some(a) = prayer_times(&l) {
        return Some(a);
    }
    let loc = r"(?: (?:from|in|at|for|over) (.+?))?(?: (?:now|today|tonight|right now))?$";
    if let Some(m) = Regex::new(&format!(r"^(?:what is |what's )?(?:the )?(?:local )?hour angle of (?:the )?{BODY}{loc}")).ok()?.captures(&l) {
        let p = place(m.get(2).map(|x| x.as_str()))?;
        let now = Utc::now();
        let (h, alt, az) = horizontal(&m[1], now, p.lat, p.lon)?;
        let disp = cap(&m[1]);
        let mut pods = vec![
            pod("Input", "Input interpretation", text(format!("{disp} | local hour angle | {} | {}", p.name, now.with_timezone(&p.tz).format("%B %-d, %Y, %-I:%M %p %Z")))),
            primary(pod("Result", "Local hour angle", text(format!("{} ({}°)", hms(h), fmt_float((h.rem_euclid(360.0) * 100.0).round() / 100.0, 5))))),
            pod("Table", "Horizontal coordinates", table(None, vec![
                vec![text("hour angle"), text(format!("{}° ({})", fmt_float((h * 100.0).round() / 100.0, 5), if h < 0.0 { "east of the meridian, rising toward transit" } else { "west of the meridian, past transit" }))],
                vec![text("altitude"), text(format!("{}°", fmt_float((alt * 10.0).round() / 10.0, 4)))],
                vec![text("azimuth"), text(format!("{}°", fmt_float((az * 10.0).round() / 10.0, 4)))],
                vec![text("local sidereal time"), text(hms(lst(now, p.lon)))],
            ])),
        ];
        if p.assumed {
            pods.push(pod("Assumption", "Assumption", text(format!("location: {} (no place given; set NOESIS_LOCATION to a city name to change)", p.name))));
        }
        return Some(Answer::new(format!("{disp} hour angle"), pods));
    }
    let m = Regex::new(&format!(r"^(?:when (?:does|will) (?:the )?{BODY} (rise|set|transit)(?: next)?|(?:the )?(?:next )?(rise|rising|set|setting|transit|rise and set|rise and set times|rise, transit and set)(?: times?)? (?:of|for) (?:the )?{BODY}|(?:the )?{BODY}(?:'s)? (?:next )?(rise|rising|set|setting|transit|rise and set|rise and set times)(?: times?)?){loc}")).ok()?.captures(&l)?;
    let body = m.get(1).or(m.get(4)).or(m.get(5))?.as_str().to_string();
    let what = m.get(2).or(m.get(3)).or(m.get(6))?.as_str().to_string();
    if body == "sun" && !what.contains("transit") && !what.contains(" and ") {
        // the Sun has its own almanac (sunrise/sunset)
        return None;
    }
    let p = place(m.get(7).map(|x| x.as_str()))?;
    let now = Utc::now();
    let h0 = if body == "sun" { -0.8333 } else { -0.5667 };
    let alt = |t: DateTime<Utc>| horizontal(&body, t, p.lat, p.lon).map(|x| x.1 - h0);
    let ha = |t: DateTime<Utc>| horizontal(&body, t, p.lat, p.lon).map(|x| if x.0.abs() > 170.0 { -1.0 } else { x.0 });
    let rise = next_crossing(&alt, now, true, 48);
    let set = next_crossing(&alt, now, false, 48);
    let transit = next_crossing(&ha, now, true, 48);
    let fmt = |t: DateTime<Utc>| t.with_timezone(&p.tz).format("%A, %B %-d, %-I:%M %p %Z").to_string();
    let disp = cap(&body);
    let mut rows = Vec::new();
    let mut events: Vec<(&str, Option<DateTime<Utc>>)> = vec![("rise", rise), ("transit", transit), ("set", set)];
    events.sort_by_key(|e| e.1.map_or(i64::MAX, |t| t.timestamp()));
    for (name, t) in &events {
        let cell = match t {
            Some(t) => {
                let (_, a, az) = horizontal(&body, *t, p.lat, p.lon)?;
                let extra = if *name == "transit" { format!(" (altitude {}°)", fmt_float(a.round(), 3)) } else { format!(" (azimuth {}°)", fmt_float(az.round(), 3)) };
                format!("{}{extra}", fmt(*t))
            }
            None => if *name == "transit" { "—".into() } else { "none in the next two days (always above or below the horizon)".into() },
        };
        rows.push(vec![text(format!("next {name}")), text(cell)]);
    }
    let key = if what.starts_with("rise and") || what.contains(',') { "rise" } else if what.starts_with("rise") { "rise" } else if what.starts_with("set") { "set" } else { "transit" };
    let main = match key {
        "rise" => rise,
        "set" => set,
        _ => transit,
    };
    let result = main.map(fmt).unwrap_or_else(|| format!("no {key} in the next two days"));
    let (_, alt_now, az_now) = horizontal(&body, now, p.lat, p.lon)?;
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{disp} | next {key} | {}", p.name))),
        primary(pod("Result", &format!("Next {key}"), text(result))),
        pod("Table", "Rise, transit and set", table(None, rows)),
        pod("Now", "Current position", text(format!("altitude {}°, azimuth {}° ({})", fmt_float((alt_now * 10.0).round() / 10.0, 4), fmt_float((az_now * 10.0).round() / 10.0, 4), if alt_now > h0 { "above the horizon" } else { "below the horizon" }))),
    ];
    if p.assumed {
        pods.push(pod("Assumption", "Assumption", text(format!("location: {} (no place given; set NOESIS_LOCATION to a city name to change)", p.name))));
    }
    pods.push(pod("Note", "Accuracy", text("Low-precision ephemeris with standard refraction (−0.57° for planets, −0.83° for the Sun's upper limb); times good to a few minutes on a flat horizon.")));
    Some(Answer::new(format!("{disp} {key}"), pods))
}
