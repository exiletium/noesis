//! The everyday sky: eclipses (Meeus, *Astronomical Algorithms*, ch. 54),
//! the Moon's phase on any date, a catalogue of bright and nearby stars and
//! deep-sky objects, meteor showers, when a constellation is best seen,
//! perihelion and aphelion, comparisons between moons and planets, sky
//! coordinates, and dates from the history of spaceflight.

use super::astro::{Body, BODIES};
use crate::engine::*;
use crate::print::fmt_float;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Utc};
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

fn sin(d: f64) -> f64 {
    d.to_radians().sin()
}

fn cos(d: f64) -> f64 {
    d.to_radians().cos()
}

pub(crate) fn jd_now() -> f64 {
    Utc::now().timestamp() as f64 / 86400.0 + 2_440_587.5
}

fn jd_of(d: NaiveDate) -> f64 {
    d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as f64 / 86400.0 + 2_440_587.5
}

fn datetime_of(jd: f64) -> NaiveDateTime {
    let secs = ((jd - 2_440_587.5) * 86400.0).round() as i64;
    chrono::DateTime::from_timestamp(secs, 0).map(|d| d.naive_utc()).unwrap_or_default()
}

fn when(jd: f64) -> String {
    datetime_of(jd).format("%A, %B %-d, %Y, %H:%M UTC").to_string()
}

fn day(jd: f64) -> String {
    datetime_of(jd).format("%B %-d, %Y").to_string()
}

// ---------------------------------------------------------------------------
// eclipses
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Eclipse {
    jd: f64,
    solar: bool,
    kind: &'static str,
    magnitude: f64,
    gamma: f64,
    /// semidurations of the partial, total and penumbral phases (minutes)
    durations: (f64, f64, f64),
}

/// The eclipse, if any, at lunation `k` (an integer for a new moon, an
/// integer plus one half for a full moon).
fn eclipse_at(k: f64) -> Option<Eclipse> {
    let solar = k.fract().abs() < 0.25;
    let t = k / 1236.85;
    let jde = 2_451_550.097_66 + 29.530_588_861 * k + 0.000_154_37 * t * t - 0.000_000_150 * t.powi(3) + 0.000_000_000_73 * t.powi(4);
    let m = 2.5534 + 29.105_356_70 * k - 0.000_001_4 * t * t - 0.000_000_11 * t.powi(3);
    let mp = 201.5643 + 385.816_935_28 * k + 0.010_758_2 * t * t + 0.000_012_38 * t.powi(3) - 0.000_000_058 * t.powi(4);
    let f = 160.7108 + 390.670_502_84 * k - 0.001_611_8 * t * t - 0.000_002_27 * t.powi(3) + 0.000_000_011 * t.powi(4);
    let om = 124.7746 - 1.563_755_88 * k + 0.002_067_2 * t * t + 0.000_002_15 * t.powi(3);
    let e = 1.0 - 0.002_516 * t - 0.000_007_4 * t * t;
    if sin(f).abs() > 0.36 {
        return None;
    }
    let f1 = f - 0.02665 * sin(om);
    let a1 = 299.77 + 0.107_408 * k - 0.009_173 * t * t;
    let mut jd = jde + if solar { -0.4075 * sin(mp) + 0.1721 * e * sin(m) } else { -0.4065 * sin(mp) + 0.1727 * e * sin(m) };
    jd += 0.0161 * sin(2.0 * mp) - 0.0097 * sin(2.0 * f1) + 0.0073 * e * sin(mp - m) - 0.0050 * e * sin(mp + m) - 0.0023 * sin(mp - 2.0 * f1)
        + 0.0021 * e * sin(2.0 * m)
        + 0.0012 * sin(mp + 2.0 * f1)
        + 0.0006 * e * sin(2.0 * mp + m)
        - 0.0004 * sin(3.0 * mp)
        - 0.0003 * e * sin(m + 2.0 * f1)
        + 0.0003 * sin(a1)
        - 0.0002 * e * sin(m - 2.0 * f1)
        - 0.0002 * e * sin(2.0 * mp - m)
        - 0.0002 * sin(om);
    let p = 0.2070 * e * sin(m) + 0.0024 * e * sin(2.0 * m) - 0.0392 * sin(mp) + 0.0116 * sin(2.0 * mp) - 0.0073 * e * sin(mp + m) + 0.0067 * e * sin(mp - m) + 0.0118 * sin(2.0 * f1);
    let q = 5.2207 - 0.0048 * e * cos(m) + 0.0020 * e * cos(2.0 * m) - 0.3299 * cos(mp) - 0.0060 * e * cos(mp + m) + 0.0041 * e * cos(mp - m);
    let w = cos(f1).abs();
    let gamma = (p * cos(f1) + q * sin(f1)) * (1.0 - 0.0048 * w);
    let u = 0.0059 + 0.0046 * e * cos(m) - 0.0182 * cos(mp) + 0.0004 * cos(2.0 * mp) - 0.0005 * cos(m + mp);
    let g = gamma.abs();
    if solar {
        if g > 1.5433 + u {
            return None;
        }
        let (kind, mag) = if g < 0.9972 {
            let kind = if u < 0.0 {
                "total"
            } else if u > 0.0047 || u >= 0.00464 * (1.0 - gamma * gamma).sqrt() {
                "annular"
            } else {
                "hybrid"
            };
            (kind, 1.0)
        } else {
            ("partial", (1.5433 + u - g) / (0.5461 + 2.0 * u))
        };
        return Some(Eclipse { jd: jd - 69.2 / 86400.0, solar, kind, magnitude: mag, gamma, durations: (0.0, 0.0, 0.0) });
    }
    let pen = (1.5573 + u - g) / 0.5450;
    let umb = (1.0128 - u - g) / 0.5450;
    if pen <= 0.0 {
        return None;
    }
    let kind = if umb >= 1.0 { "total" } else if umb > 0.0 { "partial" } else { "penumbral" };
    let nn = 0.5458 + 0.0400 * cos(mp);
    let semi = |r: f64| if r > g { 60.0 / nn * (r * r - gamma * gamma).sqrt() } else { 0.0 };
    Some(Eclipse { jd: jd - 69.2 / 86400.0, solar, kind, magnitude: if umb > 0.0 { umb } else { pen }, gamma, durations: (semi(1.0128 - u), semi(0.4678 - u), semi(1.5573 + u)) })
}

/// Eclipses after Julian date `jd`, in order.
fn eclipses_after(jd: f64, solar: Option<bool>, count: usize) -> Vec<Eclipse> {
    let year = 2000.0 + (jd - 2_451_545.0) / 365.25;
    let mut k = ((year - 2000.0) * 12.3685).floor() - 2.0;
    let mut out = Vec::new();
    while out.len() < count && k < 1e6 {
        for kk in [k, k + 0.5] {
            if let Some(e) = eclipse_at(kk) {
                if e.jd > jd && solar.map_or(true, |s| s == e.solar) {
                    out.push(e);
                }
            }
        }
        k += 1.0;
    }
    out.truncate(count);
    out
}

fn eclipse_row(e: &Eclipse) -> Vec<Content> {
    vec![text(day(e.jd)), text(format!("{} {}", e.kind, if e.solar { "solar" } else { "lunar" })), text(if e.solar && e.kind != "partial" { "central".to_string() } else { format!("magnitude {}", fmt_float(e.magnitude, 3)) })]
}

fn eclipse_answer(e: &Eclipse, heading: &str, table_of: Vec<Eclipse>) -> Answer {
    let what = if e.solar { "solar" } else { "lunar" };
    let mut r = vec![("date and time of greatest eclipse", when(e.jd)), ("type", format!("{} {what} eclipse", e.kind))];
    if e.solar {
        if e.kind == "partial" {
            r.push(("magnitude", fmt_float(e.magnitude, 3)));
        }
        r.push(("gamma (axis distance in Earth radii)", fmt_float(e.gamma, 4)));
        r.push(("visibility", if e.gamma > 0.0 { "path or penumbra mainly in the Northern Hemisphere".into() } else { "path or penumbra mainly in the Southern Hemisphere".into() }));
    } else {
        r.push(("umbral magnitude", fmt_float((1.0128 - (e.gamma.abs()) - 0.0) / 0.5450, 3)));
        let (p, t, pen) = e.durations;
        if pen > 0.0 {
            r.push(("penumbral duration", format!("{} min", n(2.0 * pen))));
        }
        if p > 0.0 {
            r.push(("partial duration", format!("{} min", n(2.0 * p))));
        }
        if t > 0.0 {
            r.push(("totality", format!("{} min", n(2.0 * t))));
        }
        r.push(("visibility", "the whole night side of the Earth".into()));
    }
    // the umbral magnitude row above is approximate; replace with the stored value
    if !e.solar {
        if let Some(row) = r.iter_mut().find(|(k, _)| *k == "umbral magnitude") {
            row.1 = if e.kind == "penumbral" { format!("none (penumbral magnitude {})", fmt_float(e.magnitude, 3)) } else { fmt_float(e.magnitude, 3) };
        }
    }
    let mut pods = vec![input(heading.to_string()), primary(pod("Result", "Result", text(format!("{} {what} eclipse — {}", capitalize(e.kind), when(e.jd))))), pod("Details", "Details", rows(r))];
    if !table_of.is_empty() {
        pods.push(pod("Upcoming", "Upcoming eclipses", table(Some(vec!["date", "type", "magnitude"]), table_of.iter().map(eclipse_row).collect())));
    }
    pods.push(pod("Method", "Method", text("Meeus, Astronomical Algorithms, chapter 54; times good to a few minutes.")));
    Answer::new("eclipse", pods)
}

pub(crate) fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn eclipses(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:when is |when's |when will be )?(?:the\s+)?(next|last|previous|upcoming)\s+(total\s+|partial\s+|annular\s+|penumbral\s+)?(lunar|solar|moon|sun)?\s*eclipses?(?:\s+of the (moon|sun))?$").captures(l) {
        let kind = m.get(2).map(|k| k.as_str().trim().to_string());
        let solar = match (m.get(3).map(|x| x.as_str()), m.get(4).map(|x| x.as_str())) {
            (Some("lunar" | "moon"), _) | (_, Some("moon")) => Some(false),
            (Some("solar" | "sun"), _) | (_, Some("sun")) => Some(true),
            _ => None,
        };
        let now = jd_now();
        if &m[1] == "last" || &m[1] == "previous" {
            let list: Vec<Eclipse> = eclipses_after(now - 800.0, solar, 12).into_iter().filter(|e| e.jd < now && kind.as_deref().map_or(true, |k| e.kind == k)).collect();
            let e = list.last()?.clone();
            return Some(eclipse_answer(&e, &format!("last {}eclipse", kind.map(|k| k + " ").unwrap_or_default()), vec![]));
        }
        let list: Vec<Eclipse> = eclipses_after(now, solar, 40).into_iter().filter(|e| kind.as_deref().map_or(true, |k| e.kind == k)).take(8).collect();
        let e = list.first()?.clone();
        return Some(eclipse_answer(&e, &format!("next {}{}eclipse", kind.map(|k| k + " ").unwrap_or_default(), match solar { Some(true) => "solar ", Some(false) => "lunar ", None => "" }), list));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(lunar|solar)?\s*eclipses\s+(?:in|of|during)\s+(\d{4})$").captures(l) {
        let y: i32 = m[2].parse().ok()?;
        let solar = m.get(1).map(|s| s.as_str() == "solar");
        let start = jd_of(NaiveDate::from_ymd_opt(y, 1, 1)?);
        let end = jd_of(NaiveDate::from_ymd_opt(y + 1, 1, 1)?);
        let list: Vec<Eclipse> = eclipses_after(start, solar, 12).into_iter().filter(|e| e.jd < end).collect();
        let main = list.iter().map(|e| format!("{} ({} {})", day(e.jd), e.kind, if e.solar { "solar" } else { "lunar" })).collect::<Vec<_>>().join("; ");
        return Some(Answer::new("eclipses", vec![input(format!("{}eclipses in {y}", m.get(1).map(|s| format!("{} ", s.as_str())).unwrap_or_default())), primary(pod("Result", "Eclipses", text(main))), pod("Table", "Details", table(Some(vec!["date", "type", "magnitude"]), list.iter().map(eclipse_row).collect()))]));
    }
    None
}

// ---------------------------------------------------------------------------
// the Moon's phase
// ---------------------------------------------------------------------------

/// Geocentric elongation of the Moon from the Sun (degrees, 0–360) at a
/// Julian date, from the leading terms of the lunar and solar theories.
fn elongation(jd: f64) -> f64 {
    let t = (jd - 2_451_545.0) / 36525.0;
    let lp = 218.316_447_7 + 481_267.881_234_21 * t;
    let d = 297.850_192_1 + 445_267.111_403_4 * t;
    let m = 357.529_109_2 + 35_999.050_290_9 * t;
    let mp = 134.963_396_4 + 477_198.867_505_5 * t;
    let f = 93.272_095_0 + 483_202.017_523_3 * t;
    let moon = lp + 6.289 * sin(mp) + 1.274 * sin(2.0 * d - mp) + 0.658 * sin(2.0 * d) + 0.214 * sin(2.0 * mp) - 0.186 * sin(m) - 0.114 * sin(2.0 * f);
    let sun = 280.466_46 + 36_000.769_83 * t + 1.914_602 * sin(m) + 0.019_993 * sin(2.0 * m);
    (moon - sun).rem_euclid(360.0)
}

fn phase_name(e: f64) -> &'static str {
    match e {
        x if !(6.1..=353.9).contains(&x) => "new moon",
        x if x < 83.9 => "waxing crescent",
        x if x <= 96.1 => "first quarter",
        x if x < 173.9 => "waxing gibbous",
        x if x <= 186.1 => "full moon",
        x if x < 263.9 => "waning gibbous",
        x if x <= 276.1 => "last quarter",
        _ => "waning crescent",
    }
}

/// The next time after `jd` that the elongation reaches `target` degrees.
fn next_phase(jd: f64, target: f64) -> f64 {
    let gap = |j: f64| (elongation(j) - target).rem_euclid(360.0);
    // the elongation grows about 12.19° a day
    let mut j = jd + ((target - elongation(jd)).rem_euclid(360.0)) / 12.19 - 2.0;
    if j < jd {
        j = jd;
    }
    // step until the gap wraps, then bisect
    let mut a = j;
    let mut prev = gap(a);
    let mut b = a;
    for _ in 0..80 {
        b = a + 0.5;
        let g = gap(b);
        if g < prev {
            break;
        }
        a = b;
        prev = g;
    }
    for _ in 0..50 {
        let mid = (a + b) / 2.0;
        if gap(mid) > 180.0 {
            a = mid;
        } else {
            b = mid;
        }
    }
    (a + b) / 2.0
}

fn date_phrase(s: &str) -> Option<NaiveDate> {
    let s = s.trim().trim_start_matches("the ");
    if let Some(m) = re!(r"^(first|second|third|fourth|fifth|last|\d+(?:st|nd|rd|th))\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)\s+(?:in|of)\s+([a-z]+)(?:,?\s+(\d{4}))?$").captures(s) {
        let nth = super::clock::ordinal_word(&m[1])?;
        let wd = super::clock::weekday(&m[2])?;
        let mo = super::clock::month_num(&m[3])?;
        let today = super::misc::today();
        let pick = |y: i32| -> Option<NaiveDate> {
            if nth > 0 {
                NaiveDate::from_weekday_of_month_opt(y, mo, wd, nth as u8)
            } else {
                let last = if mo == 12 { NaiveDate::from_ymd_opt(y + 1, 1, 1)? } else { NaiveDate::from_ymd_opt(y, mo + 1, 1)? } - Duration::days(1);
                let back = (last.weekday().num_days_from_monday() + 7 - wd.num_days_from_monday()) % 7;
                Some(last - Duration::days(back as i64))
            }
        };
        return match m.get(4) {
            Some(y) => pick(y.as_str().parse().ok()?),
            None => {
                let d = pick(today.year())?;
                if d < today { pick(today.year() + 1) } else { Some(d) }
            }
        };
    }
    super::misc::parse_date(s)
}

fn moon_phase(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:what is |what's |what was |what will be )?(?:the\s+)?(?:moon(?:'s)? phase|phase of the moon|lunar phase|moon)(?:\s+(?:on|for|at)\s+(.+?)|\s+(today|tonight|tomorrow|now))?$").captures(l)?;
    if m.get(1).is_none() && m.get(2).is_none() && !l.contains("phase") {
        return None;
    }
    let date = match m.get(1).or(m.get(2)).map(|g| g.as_str()) {
        None | Some("today" | "tonight" | "now") => super::misc::today(),
        Some(s) => date_phrase(s)?,
    };
    let jd = jd_of(date) + 0.5; // noon UTC
    let e = elongation(jd);
    let illum = (1.0 - cos(e)) / 2.0;
    let name = phase_name(e);
    let age = e / 360.0 * 29.530_588;
    let mut r = vec![("phase", name.to_string()), ("illuminated fraction", format!("{}%", fmt_float(illum * 100.0, 3))), ("moon age", format!("{} days since new moon", fmt_float(age, 3)))];
    for (label, target) in [("next new moon", 0.0), ("next first quarter", 90.0), ("next full moon", 180.0), ("next last quarter", 270.0)] {
        r.push((label, when(next_phase(jd, target))));
    }
    Some(Answer::new(
        "moon phase",
        vec![
            input(format!("Moon phase on {}", super::misc::date_long(date))),
            primary(pod("Result", "Phase", text(format!("{name} ({}% illuminated)", fmt_float(illum * 100.0, 3))))),
            pod("Details", "Details", rows(r)),
        ],
    ))
}

fn next_moon(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:when is |when's )?(?:the\s+)?(next|last|previous)\s+(full moon|new moon|first quarter(?: moon)?|last quarter(?: moon)?|third quarter(?: moon)?|half moon)$").captures(l)?;
    let target = match &m[2] {
        "full moon" => 180.0,
        "new moon" => 0.0,
        s if s.starts_with("first") || s == "half moon" => 90.0,
        _ => 270.0,
    };
    let now = jd_now();
    let jd = if &m[1] == "next" { next_phase(now, target) } else { next_phase(now - 30.0, target).max(next_phase(now - 60.0, target)).min(now) };
    let jd = if &m[1] != "next" {
        let mut j = next_phase(now - 31.0, target);
        while next_phase(j + 1.0, target) < now {
            j = next_phase(j + 1.0, target);
        }
        j
    } else {
        jd
    };
    let list: Vec<Vec<Content>> = (0..6).scan(jd, |j, _| {
        let cur = *j;
        *j = next_phase(cur + 1.0, target);
        Some(vec![text(when(cur))])
    }).collect();
    Some(Answer::new("moon phase", vec![input(format!("{} {}", &m[1], &m[2])), primary(pod("Result", "Date", text(when(jd)))), pod("Following", "This and following dates", table(None, list))]))
}

// ---------------------------------------------------------------------------
// stars and deep-sky objects
// ---------------------------------------------------------------------------

pub(crate) struct Star {
    pub(crate) names: &'static [&'static str],
    /// right ascension (h, m, s) and declination (sign, d, m, s), J2000
    pub(crate) ra: (f64, f64, f64),
    pub(crate) dec: (f64, f64, f64, f64),
    pub(crate) mag: f64,
    /// distance, light years
    pub(crate) ly: f64,
    pub(crate) spectral: &'static str,
    pub(crate) constellation: &'static str,
    pub(crate) note: &'static str,
}

const fn st(names: &'static [&'static str], ra: (f64, f64, f64), dec: (f64, f64, f64, f64), mag: f64, ly: f64, spectral: &'static str, constellation: &'static str, note: &'static str) -> Star {
    Star { names, ra, dec, mag, ly, spectral, constellation, note }
}

pub(crate) static STARS: &[Star] = &[
    st(&["sirius", "alpha canis majoris", "dog star"], (6.0, 45.0, 8.9), (-1.0, 16.0, 42.0, 58.0), -1.46, 8.60, "A1V + DA2 (white dwarf companion)", "Canis Major", "the brightest star in the night sky"),
    st(&["canopus", "alpha carinae"], (6.0, 23.0, 57.1), (-1.0, 52.0, 41.0, 44.0), -0.74, 310.0, "A9II", "Carina", "second-brightest star"),
    st(&["alpha centauri", "rigil kentaurus", "rigil kent", "alpha centauri a"], (14.0, 39.0, 36.5), (-1.0, 60.0, 50.0, 2.0), -0.27, 4.37, "G2V + K1V", "Centaurus", "the nearest star system to the Sun; combined light of A and B"),
    st(&["arcturus", "alpha bootis"], (14.0, 15.0, 39.7), (1.0, 19.0, 10.0, 57.0), -0.05, 36.7, "K1.5III", "Boötes", "brightest star in the northern celestial hemisphere"),
    st(&["vega", "alpha lyrae"], (18.0, 36.0, 56.3), (1.0, 38.0, 47.0, 1.0), 0.03, 25.0, "A0V", "Lyra", "corner of the Summer Triangle; the zero point of the magnitude scale"),
    st(&["capella", "alpha aurigae"], (5.0, 16.0, 41.4), (1.0, 45.0, 59.0, 53.0), 0.08, 42.9, "G3III + G0III", "Auriga", ""),
    st(&["rigel", "beta orionis"], (5.0, 14.0, 32.3), (-1.0, 8.0, 12.0, 6.0), 0.13, 860.0, "B8Ia", "Orion", "blue supergiant; Orion's left foot"),
    st(&["procyon", "alpha canis minoris"], (7.0, 39.0, 18.1), (1.0, 5.0, 13.0, 30.0), 0.34, 11.46, "F5IV-V", "Canis Minor", ""),
    st(&["achernar", "alpha eridani"], (1.0, 37.0, 42.8), (-1.0, 57.0, 14.0, 12.0), 0.46, 139.0, "B6Vep", "Eridanus", ""),
    st(&["betelgeuse", "alpha orionis"], (5.0, 55.0, 10.3), (1.0, 7.0, 24.0, 25.0), 0.50, 548.0, "M1-2Ia-ab", "Orion", "red supergiant; semiregular variable, magnitude 0.0 to 1.6"),
    st(&["hadar", "beta centauri", "agena"], (14.0, 3.0, 49.4), (-1.0, 60.0, 22.0, 23.0), 0.61, 390.0, "B1III", "Centaurus", "one of the Southern Pointers"),
    st(&["altair", "alpha aquilae"], (19.0, 50.0, 47.0), (1.0, 8.0, 52.0, 6.0), 0.76, 16.7, "A7V", "Aquila", "corner of the Summer Triangle"),
    st(&["acrux", "alpha crucis"], (12.0, 26.0, 35.9), (-1.0, 63.0, 5.0, 57.0), 0.76, 320.0, "B0.5IV", "Crux", "brightest star of the Southern Cross"),
    st(&["aldebaran", "alpha tauri"], (4.0, 35.0, 55.2), (1.0, 16.0, 30.0, 33.0), 0.86, 65.3, "K5III", "Taurus", "the eye of the Bull"),
    st(&["antares", "alpha scorpii"], (16.0, 29.0, 24.5), (-1.0, 26.0, 25.0, 55.0), 0.96, 550.0, "M1.5Iab", "Scorpius", "red supergiant; heart of the Scorpion"),
    st(&["spica", "alpha virginis"], (13.0, 25.0, 11.6), (-1.0, 11.0, 9.0, 41.0), 0.97, 250.0, "B1V", "Virgo", ""),
    st(&["pollux", "beta geminorum"], (7.0, 45.0, 18.9), (1.0, 28.0, 1.0, 34.0), 1.14, 33.8, "K0III", "Gemini", ""),
    st(&["fomalhaut", "alpha piscis austrini"], (22.0, 57.0, 39.0), (-1.0, 29.0, 37.0, 20.0), 1.16, 25.1, "A3V", "Piscis Austrinus", ""),
    st(&["deneb", "alpha cygni"], (20.0, 41.0, 25.9), (1.0, 45.0, 16.0, 49.0), 1.25, 2600.0, "A2Ia", "Cygnus", "corner of the Summer Triangle; one of the most luminous stars known"),
    st(&["mimosa", "beta crucis", "becrux"], (12.0, 47.0, 43.3), (-1.0, 59.0, 41.0, 19.0), 1.25, 280.0, "B0.5III", "Crux", ""),
    st(&["regulus", "alpha leonis"], (10.0, 8.0, 22.3), (1.0, 11.0, 58.0, 2.0), 1.40, 79.3, "B8IVn", "Leo", "the heart of the Lion"),
    st(&["castor", "alpha geminorum"], (7.0, 34.0, 36.0), (1.0, 31.0, 53.0, 18.0), 1.58, 51.0, "A1V (sextuple system)", "Gemini", ""),
    st(&["bellatrix", "gamma orionis"], (5.0, 25.0, 7.9), (1.0, 6.0, 20.0, 59.0), 1.64, 250.0, "B2III", "Orion", ""),
    st(&["polaris", "north star", "pole star", "alpha ursae minoris", "the north star", "the pole star"], (2.0, 31.0, 49.1), (1.0, 89.0, 15.0, 51.0), 1.98, 433.0, "F7Ib (Cepheid variable)", "Ursa Minor", "within 0.7° of the north celestial pole; end of the Little Dipper's handle"),
    st(&["dubhe", "alpha ursae majoris"], (11.0, 3.0, 43.7), (1.0, 61.0, 45.0, 3.0), 1.79, 123.0, "K0III", "Ursa Major", "one of the Pointers to Polaris"),
    st(&["merak", "beta ursae majoris"], (11.0, 1.0, 50.5), (1.0, 56.0, 22.0, 57.0), 2.37, 79.7, "A1V", "Ursa Major", "one of the Pointers to Polaris"),
    st(&["mizar", "zeta ursae majoris"], (13.0, 23.0, 55.5), (1.0, 54.0, 55.0, 31.0), 2.23, 82.9, "A2V (quadruple system)", "Ursa Major", "middle star of the Big Dipper's handle; naked-eye pair with Alcor"),
    st(&["alcor", "80 ursae majoris"], (13.0, 25.0, 13.5), (1.0, 54.0, 59.0, 17.0), 3.99, 81.7, "A5V", "Ursa Major", "naked-eye companion of Mizar"),
    st(&["algol", "beta persei", "demon star"], (3.0, 8.0, 10.1), (1.0, 40.0, 57.0, 20.0), 2.12, 90.0, "B8V", "Perseus", "eclipsing binary"),
    st(&["eta draconis", "athebyne"], (16.0, 23.0, 59.5), (1.0, 61.0, 30.0, 51.0), 2.73, 92.0, "G8III", "Draco", "a yellow giant with a white-dwarf companion"),
    st(&["gamma centauri", "muhlifain"], (12.0, 41.0, 31.0), (-1.0, 48.0, 57.0, 35.0), 2.17, 130.0, "A1IV", "Centaurus", "a close binary of two similar A-type stars"),
    st(&["mira", "omicron ceti"], (2.0, 19.0, 20.8), (-1.0, 2.0, 58.0, 39.0), 3.04, 300.0, "M7IIIe", "Cetus", "the prototype long-period (Mira) variable"),
    st(&["proxima centauri", "proxima", "alpha centauri c"], (14.0, 29.0, 43.0), (-1.0, 62.0, 40.0, 46.0), 11.13, 4.2465, "M5.5Ve", "Centaurus", "the nearest known star to the Sun; flare star with planets"),
    st(&["barnard's star", "barnards star"], (17.0, 57.0, 48.5), (1.0, 4.0, 41.0, 36.0), 9.51, 5.96, "M4V", "Ophiuchus", "largest proper motion of any star"),
    st(&["wolf 359"], (10.0, 56.0, 29.2), (1.0, 7.0, 0.0, 53.0), 13.51, 7.86, "M6V", "Leo", ""),
    st(&["lalande 21185"], (11.0, 3.0, 20.2), (1.0, 35.0, 58.0, 12.0), 7.52, 8.31, "M2V", "Ursa Major", ""),
    st(&["epsilon eridani"], (3.0, 32.0, 55.8), (-1.0, 9.0, 27.0, 30.0), 3.73, 10.5, "K2V", "Eridanus", ""),
    st(&["tau ceti"], (1.0, 44.0, 4.1), (-1.0, 15.0, 56.0, 15.0), 3.50, 11.9, "G8V", "Cetus", ""),
    st(&["crab nebula", "m1", "messier 1"], (5.0, 34.0, 31.94), (1.0, 22.0, 0.0, 52.2), 8.4, 6500.0, "supernova remnant (SN 1054) with a pulsar", "Taurus", ""),
    st(&["orion nebula", "m42", "messier 42"], (5.0, 35.0, 17.3), (-1.0, 5.0, 23.0, 28.0), 4.0, 1344.0, "emission nebula", "Orion", "the nearest region of massive star formation"),
    st(&["andromeda galaxy", "m31", "messier 31", "andromeda"], (0.0, 42.0, 44.3), (1.0, 41.0, 16.0, 9.0), 3.44, 2.5e6, "spiral galaxy (SA(s)b)", "Andromeda", "the nearest large galaxy"),
    st(&["pleiades", "m45", "seven sisters", "messier 45"], (3.0, 47.0, 24.0), (1.0, 24.0, 7.0, 0.0), 1.6, 444.0, "open star cluster", "Taurus", ""),
    st(&["ring nebula", "m57", "messier 57"], (18.0, 53.0, 35.1), (1.0, 33.0, 1.0, 45.0), 8.8, 2570.0, "planetary nebula", "Lyra", ""),
    st(&["whirlpool galaxy", "m51", "messier 51"], (13.0, 29.0, 52.7), (1.0, 47.0, 11.0, 43.0), 8.4, 2.3e7, "spiral galaxy", "Canes Venatici", ""),
    st(&["lagoon nebula", "m8", "messier 8", "ngc 6523"], (18.0, 3.0, 37.0), (-1.0, 24.0, 23.0, 12.0), 6.0, 4100.0, "emission nebula (H II region)", "Sagittarius", "a star-forming cloud visible to the naked eye"),
    st(&["eta carinae nebula", "carina nebula", "ngc 3372", "great nebula in carina"], (10.0, 45.0, 8.5), (-1.0, 59.0, 52.0, 4.0), 1.0, 8500.0, "emission nebula (H II region)", "Carina", "one of the largest diffuse nebulae in the sky; contains the luminous blue variable Eta Carinae"),
    st(&["tarantula nebula", "30 doradus", "ngc 2070"], (5.0, 38.0, 38.0), (-1.0, 69.0, 5.0, 42.0), 8.0, 160000.0, "emission nebula (H II region)", "Dorado", "in the Large Magellanic Cloud; the most active star-forming region in the Local Group"),
    st(&["helix nebula", "ngc 7293", "eye of god"], (22.0, 29.0, 38.55), (-1.0, 20.0, 50.0, 13.6), 7.6, 655.0, "planetary nebula", "Aquarius", "one of the nearest planetary nebulae"),
    st(&["ngc 246", "skull nebula"], (0.0, 47.0, 3.3), (-1.0, 11.0, 52.0, 19.0), 10.9, 1600.0, "planetary nebula", "Cetus", ""),
    st(&["m76", "messier 76", "little dumbbell nebula", "ngc 650"], (1.0, 42.0, 19.9), (1.0, 51.0, 34.0, 31.0), 10.1, 2500.0, "planetary nebula", "Perseus", ""),
    st(&["m4", "messier 4", "ngc 6121"], (16.0, 23.0, 35.22), (-1.0, 26.0, 31.0, 32.7), 5.6, 7200.0, "globular cluster", "Scorpius", "one of the nearest globular clusters"),
    st(&["m3", "messier 3", "ngc 5272"], (13.0, 42.0, 11.62), (1.0, 28.0, 22.0, 38.2), 6.2, 33900.0, "globular cluster", "Canes Venatici", "about 500,000 stars, rich in RR Lyrae variables"),
    st(&["m15", "messier 15", "ngc 7078"], (21.0, 29.0, 58.33), (1.0, 12.0, 10.0, 1.2), 6.2, 35700.0, "globular cluster", "Pegasus", "one of the densest globular clusters, with a collapsed core"),
    st(&["m13", "messier 13", "hercules cluster", "great globular cluster in hercules", "ngc 6205"], (16.0, 41.0, 41.24), (1.0, 36.0, 27.0, 35.5), 5.8, 22200.0, "globular cluster", "Hercules", ""),
    st(&["omega centauri", "ngc 5139"], (13.0, 26.0, 47.28), (-1.0, 47.0, 28.0, 46.1), 3.9, 17090.0, "globular cluster", "Centaurus", "the largest globular cluster of the Milky Way, about 10 million stars"),
    st(&["m37", "messier 37", "ngc 2099"], (5.0, 52.0, 18.0), (1.0, 32.0, 33.0, 2.0), 6.2, 4500.0, "open cluster", "Auriga", ""),
    st(&["m80", "messier 80", "ngc 6093"], (16.0, 17.0, 2.41), (-1.0, 22.0, 58.0, 33.9), 7.87, 32600.0, "globular cluster", "Scorpius", ""),
    st(&["ngc 2808"], (9.0, 12.0, 3.1), (-1.0, 64.0, 51.0, 48.6), 6.2, 31300.0, "globular cluster", "Carina", "hosts several generations of stars"),
    st(&["m53", "messier 53", "ngc 5024"], (13.0, 12.0, 55.25), (1.0, 18.0, 10.0, 5.4), 7.6, 58000.0, "globular cluster", "Coma Berenices", ""),
    st(&["ngc 288"], (0.0, 52.0, 45.24), (-1.0, 26.0, 34.0, 57.4), 8.1, 28700.0, "globular cluster", "Sculptor", ""),
    st(&["beehive cluster", "m44", "messier 44", "praesepe"], (8.0, 40.0, 24.0), (1.0, 19.0, 40.0, 0.0), 3.7, 577.0, "open cluster", "Cancer", ""),
    st(&["double cluster", "ngc 869 and ngc 884", "h and chi persei"], (2.0, 20.0, 0.0), (1.0, 57.0, 8.0, 0.0), 3.7, 7500.0, "pair of open clusters (NGC 869 and NGC 884)", "Perseus", ""),
    st(&["ngc 4603"], (12.0, 40.0, 55.2), (-1.0, 40.0, 58.0, 35.0), 11.9, 1.08e8, "spiral galaxy (SA(s)c)", "Centaurus", "its distance was measured with Cepheid variables by the Hubble Space Telescope"),
    st(&["leo triplet", "m66 group"], (11.0, 18.0, 55.0), (1.0, 13.0, 5.0, 0.0), 8.9, 3.5e7, "group of three spiral galaxies (M65, M66, NGC 3628)", "Leo", ""),
];

pub(crate) fn ra_deg(s: &Star) -> f64 {
    (s.ra.0 + s.ra.1 / 60.0 + s.ra.2 / 3600.0) * 15.0
}

pub(crate) fn dec_deg(s: &Star) -> f64 {
    s.dec.0 * (s.dec.1 + s.dec.2 / 60.0 + s.dec.3 / 3600.0)
}

pub(crate) fn ra_text(s: &Star) -> String {
    format!("{}h {:02}m {:04.1}s", s.ra.0, s.ra.1, s.ra.2)
}

pub(crate) fn dec_text(s: &Star) -> String {
    format!("{}{}° {:02}′ {:02}″", if s.dec.0 < 0.0 { "−" } else { "+" }, s.dec.1, s.dec.2, s.dec.3.round())
}

pub(crate) fn ly_text(ly: f64) -> String {
    if ly >= 1e6 {
        format!("{} million light years", n(ly / 1e6))
    } else {
        format!("{} light years ({} parsecs)", n(ly), n(ly / 3.261_56))
    }
}

pub(crate) fn find_star(w: &str) -> Option<&'static Star> {
    let w = w.trim().trim_start_matches("the ").trim_start_matches("star ").trim();
    let w = re!(r"^(?:m|messier)\s*(\d+)$").replace(w, "m$1");
    let w = re!(r"^(ngc|ic)\s*(\d+)$").replace(&w, "$1 $2").into_owned();
    let w = w.as_str();
    let find = |w: &str| STARS.iter().find(|s| s.names.iter().any(|n| *n == w || n.trim_start_matches("the ") == w));
    find(w).or_else(|| find(re!(r"\s+(?:star cluster|cluster|star|galaxy)$").replace(w, "").trim()))
}

pub(crate) fn star_rows(s: &Star) -> Vec<(&'static str, String)> {
    let mut r = vec![
        ("right ascension", ra_text(s)),
        ("declination", dec_text(s)),
        ("apparent magnitude", fmt_float(s.mag, 3)),
        ("distance", ly_text(s.ly)),
        (if s.spectral.contains("nebula") || s.spectral.contains("galax") || s.spectral.contains("cluster") || s.spectral.contains("remnant") { "type" } else { "spectral type" }, s.spectral.to_string()),
        ("constellation", s.constellation.to_string()),
    ];
    if s.ly < 1e5 && !s.spectral.contains("nebula") && !s.spectral.contains("cluster") && !s.spectral.contains("remnant") {
        let abs = s.mag - 5.0 * (s.ly / 3.261_56 / 10.0).log10();
        r.push(("absolute magnitude", fmt_float(abs, 3)));
        r.push(("luminosity (visual)", format!("{} × Sun", n(10f64.powf((4.83 - abs) / 2.5)))));
    }
    if !s.note.is_empty() {
        r.push(("note", s.note.to_string()));
    }
    r
}

fn stars(l: &str) -> Option<Answer> {
    // "closest stars to Earth"
    if re!(r"^(?:the\s+)?(?:closest|nearest)\s+stars?(?:\s+(?:to|from)\s+(?:the\s+)?(?:earth|us|sun|the sun))?$").is_match(l) {
        let list = [("Proxima Centauri", 4.2465, "M5.5V red dwarf"), ("Alpha Centauri A", 4.37, "G2V"), ("Alpha Centauri B", 4.37, "K1V"), ("Barnard's Star", 5.96, "M4V red dwarf"), ("Luhman 16 A and B", 6.5, "brown dwarfs"), ("WISE 0855−0714", 7.4, "brown dwarf"), ("Wolf 359", 7.86, "M6V red dwarf"), ("Lalande 21185", 8.31, "M2V red dwarf"), ("Sirius A", 8.60, "A1V"), ("Sirius B", 8.60, "white dwarf"), ("Gliese 65 A and B (Luyten 726-8)", 8.79, "M5.5V red dwarfs"), ("Ross 154", 9.70, "M3.5V red dwarf")];
        return Some(Answer::new("nearest stars", vec![input("stars nearest to the Sun"), primary(pod("Result", "Result", text("Proxima Centauri, 4.2465 light years away"))), pod("Table", "Nearest stars and brown dwarfs", table(Some(vec!["object", "distance (ly)", "type"]), list.iter().map(|(a, d, t)| vec![text(*a), text(n(*d)), text(*t)]).collect()))]));
    }
    if re!(r"^(?:the\s+)?brightest stars(?:\s+in the (?:night\s+)?sky)?$").is_match(l) {
        let mut v: Vec<&Star> = STARS.iter().filter(|s| s.spectral.chars().next().map_or(false, |c| "OBAFGKM".contains(c))).collect();
        v.sort_by(|a, b| a.mag.partial_cmp(&b.mag).unwrap());
        let t: Vec<Vec<Content>> = v.iter().take(15).map(|s| vec![text(capitalize(s.names[0])), text(fmt_float(s.mag, 3)), text(n(s.ly)), text(s.constellation)]).collect();
        return Some(Answer::new("brightest stars", vec![input("brightest stars in the night sky"), primary(pod("Result", "Result", text("Sirius (magnitude −1.46)"))), pod("Table", "Brightest stars", table(Some(vec!["star", "magnitude", "distance (ly)", "constellation"]), t))]));
    }
    // angular separation between two objects
    if let Some(m) = re!(r"^(?:the\s+)?(?:angular\s+)?(?:separation|distance)\s+between\s+(.+?)\s+and\s+(.+)$").captures(l) {
        let (a, b) = (find_star(&m[1])?, find_star(&m[2])?);
        let (r1, d1, r2, d2) = (ra_deg(a), dec_deg(a), ra_deg(b), dec_deg(b));
        let c = sin(d1) * sin(d2) + cos(d1) * cos(d2) * cos(r1 - r2);
        // haversine form for small angles
        let hav = (((d2 - d1).to_radians() / 2.0).sin().powi(2) + cos(d1) * cos(d2) * ((r2 - r1).to_radians() / 2.0).sin().powi(2)).sqrt();
        let sep = 2.0 * hav.asin().to_degrees();
        let _ = c;
        let txt = if sep < 1.0 { format!("{}′ ({}°)", n(sep * 60.0), n(sep)) } else { format!("{}°", n(sep)) };
        let mut extra = vec![];
        let d_ly = (a.ly * a.ly + b.ly * b.ly - 2.0 * a.ly * b.ly * cos(sep)).sqrt();
        if a.ly < 1e5 && b.ly < 1e5 {
            extra.push(pod("Space", "Distance between them in space (from the catalogue distances)", text(format!("about {} light years", n(d_ly)))));
        }
        let mut pods = vec![input(format!("angular separation between {} and {}", capitalize(a.names[0]), capitalize(b.names[0]))), primary(pod("Result", "Angular separation", text(txt)))];
        pods.extend(extra);
        return Some(Answer::new("angular separation", pods));
    }
    // variable star properties
    if let Some(m) = re!(r"^(?:the\s+)?variable star (?:properties|data|information) (?:of|for)\s+(.+)$").captures(l) {
        let s = find_star(&m[1])?;
        let r: Vec<(&str, String)> = match s.names[0] {
            "algol" => vec![("variable type", "Algol-type eclipsing binary (EA)".into()), ("magnitude range", "2.12 to 3.39 (V)".into()), ("period", "2.867 days (2 d 20 h 49 m)".into()), ("eclipse duration", "about 10 hours".into()), ("components", "B8V primary eclipsed by a K0IV subgiant".into())],
            "betelgeuse" => vec![("variable type", "semiregular (SRc)".into()), ("magnitude range", "0.0 to 1.6".into()), ("periods", "about 400 days, with a long secondary period near 2100 days".into())],
            "polaris" => vec![("variable type", "classical Cepheid (DCEP-S)".into()), ("magnitude range", "1.86 to 2.13".into()), ("period", "3.97 days".into())],
            "mira" => vec![("variable type", "Mira variable (M), the prototype".into()), ("magnitude range", "2.0 to 10.1 (V)".into()), ("period", "332 days".into())],
            "antares" => vec![("variable type", "semiregular (LC)".into()), ("magnitude range", "0.6 to 1.6".into()), ("period", "about 1650 days".into())],
            _ => return None,
        };
        return Some(Answer::new("variable star", vec![input(format!("{} | variable star properties", capitalize(s.names[0]))), primary(pod("Result", "Variable star properties", rows(r)))]));
    }
    // "<property> of <object>"
    if let Some(m) = re!(r"^(?:what is\s+)?(?:the\s+)?(right ascension and declination|ra and dec|coordinates|position|apparent magnitude|magnitude|brightness|distance(?: to earth| from earth)?|spectral type|spectral class|constellation|absolute magnitude|luminosity)\s+(?:of|for)\s+(?:the\s+)?(.+?)(?:\s+(?:to|from) (?:the )?earth)?(?:\s+in light ?years)?$").captures(l) {
        let s = find_star(&m[2])?;
        let want = &m[1];
        let all = star_rows(s);
        let pick = |k: &str| all.iter().find(|(a, _)| *a == k).map(|(_, v)| v.clone());
        let main = match want {
            w if w.starts_with("right ascension") || w == "ra and dec" || w == "coordinates" || w == "position" => format!("right ascension {} | declination {}", ra_text(s), dec_text(s)),
            "apparent magnitude" | "magnitude" | "brightness" => pick("apparent magnitude")?,
            w if w.starts_with("distance") => pick("distance")?,
            "spectral type" | "spectral class" => pick("spectral type").or_else(|| pick("type"))?,
            "constellation" => pick("constellation")?,
            "absolute magnitude" => pick("absolute magnitude")?,
            _ => pick("luminosity (visual)")?,
        };
        return Some(Answer::new(s.names[0], vec![input(format!("{} | {want}", capitalize(s.names[0]))), primary(pod("Result", &capitalize(want), text(main))), pod("Properties", "Properties", rows(all))]));
    }
    // "distance from the North Star to Earth", "how far is Sirius"
    if let Some(m) = re!(r"^(?:the\s+)?(?:distance (?:from|to)\s+(?:the\s+)?(.+?)\s+(?:to|from)\s+(?:the\s+)?(?:earth|us|the sun)(?:\s+in light ?years)?|how far (?:away )?is\s+(?:the\s+)?(.+?)(?:\s+from (?:the )?earth)?)$").captures(l) {
        let s = find_star(m.get(1).or(m.get(2))?.as_str())?;
        return Some(Answer::new(s.names[0], vec![input(format!("{} | distance from Earth", capitalize(s.names[0]))), primary(pod("Result", "Distance", text(ly_text(s.ly)))), pod("Properties", "Properties", rows(star_rows(s)))]));
    }
    // a bare name
    let s = find_star(l)?;
    if matches!(l, "andromeda" | "proxima" | "m3" | "m4" | "m8" | "m13" | "m15" | "m37" | "m44" | "m53" | "m76" | "m80") {
        return None;
    }
    Some(Answer::new(s.names[0], vec![input(capitalize(s.names[0])), primary(pod("Properties", "Properties", rows(star_rows(s))))]))
}

// ---------------------------------------------------------------------------
// coordinates on the sky
// ---------------------------------------------------------------------------

fn sky_coordinates(l: &str) -> Option<Answer> {
    let m = re!(r#"^(?:ra|right ascension)\s*(\d+)\s*h\s*(\d+)\s*m\s*([\d.]+)\s*s?\s*,?\s*(?:dec|declination)\s*([+-−]?)\s*(\d+)\s*(?:d|°)\s*(\d+)\s*(?:'|′|m)\s*([\d.]+)\s*(?:"|″|s)?$"#).captures(l)?;
    let ra = (m[1].parse::<f64>().ok()? + m[2].parse::<f64>().ok()? / 60.0 + m[3].parse::<f64>().ok()? / 3600.0) * 15.0;
    let sign = if m[4].is_empty() || &m[4] == "+" { 1.0 } else { -1.0 };
    let dec = sign * (m[5].parse::<f64>().ok()? + m[6].parse::<f64>().ok()? / 60.0 + m[7].parse::<f64>().ok()? / 3600.0);
    // galactic coordinates (J2000 pole at RA 192.85948°, Dec 27.12825°; l of the NCP 122.93192°)
    let (ag, dg, lncp) = (192.859_48, 27.128_25, 122.931_92);
    let sb = sin(dec) * sin(dg) + cos(dec) * cos(dg) * cos(ra - ag);
    let b = sb.asin().to_degrees();
    let y = cos(dec) * sin(ra - ag);
    let x = sin(dec) * cos(dg) - cos(dec) * sin(dg) * cos(ra - ag);
    let lg = (lncp - y.atan2(x).to_degrees()).rem_euclid(360.0);
    // ecliptic coordinates (obliquity 23.4393°)
    let eps: f64 = 23.4393;
    let lam = (sin(ra) * cos(eps) + dec.to_radians().tan() * sin(eps)).atan2(cos(ra)).to_degrees().rem_euclid(360.0);
    let bet = (sin(dec) * cos(eps) - cos(dec) * sin(eps) * sin(ra)).asin().to_degrees();
    let nearest = STARS.iter().map(|s| {
        let hav = (((dec_deg(s) - dec).to_radians() / 2.0).sin().powi(2) + cos(dec) * cos(dec_deg(s)) * ((ra_deg(s) - ra).to_radians() / 2.0).sin().powi(2)).sqrt();
        (2.0 * hav.asin().to_degrees(), s)
    }).min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())?;
    let mut r = vec![
        ("decimal", format!("RA {}°, Dec {}°", n(ra), n(dec))),
        ("galactic coordinates", format!("l = {}°, b = {}°", n(lg), n(b))),
        ("ecliptic coordinates", format!("λ = {}°, β = {}°", n(lam), n(bet))),
        ("visible from latitudes", if dec >= 0.0 { format!("north of {}°S", n(90.0 - dec)) } else { format!("south of {}°N", n(90.0 + dec)) }),
    ];
    r.push(("nearest catalogued bright object", format!("{} ({}° away)", capitalize(nearest.1.names[0]), n(nearest.0))));
    Some(Answer::new("sky coordinates", vec![input(format!("right ascension {}h {}m {}s, declination {}{}° {}′ {}″", &m[1], &m[2], &m[3], if sign < 0.0 { "−" } else { "+" }, &m[5], &m[6], &m[7])), primary(pod("Result", "Position", rows(r)))]))
}

// ---------------------------------------------------------------------------
// constellations and finding things in the sky
// ---------------------------------------------------------------------------

/// (name, RA hours, Dec degrees, note)
pub(crate) const CONSTELLATIONS: &[(&str, f64, f64, &str)] = &[
    ("orion", 5.6, 5.0, "the Hunter: the three stars of Orion's Belt, red Betelgeuse and blue-white Rigel"),
    ("ursa major", 11.3, 50.0, "the Great Bear; contains the Big Dipper (the Plough)"),
    ("big dipper", 12.2, 56.0, "seven bright stars of Ursa Major"),
    ("ursa minor", 15.0, 77.0, "the Little Bear; Polaris ends the Little Dipper's handle"),
    ("cassiopeia", 1.0, 62.0, "the W-shaped Queen opposite the Big Dipper across Polaris"),
    ("scorpius", 16.9, -30.0, "the Scorpion, with red Antares at its heart"),
    ("sagittarius", 19.1, -28.0, "the Archer; the Teapot asterism points to the Galactic Centre"),
    ("cygnus", 20.6, 42.0, "the Swan (Northern Cross), with Deneb"),
    ("lyra", 18.9, 36.0, "the Lyre, with Vega"),
    ("leo", 10.7, 15.0, "the Lion, with Regulus at the base of the Sickle"),
    ("taurus", 4.7, 15.0, "the Bull, with Aldebaran, the Hyades and the Pleiades"),
    ("gemini", 7.1, 22.0, "the Twins Castor and Pollux"),
    ("crux", 12.5, -60.0, "the Southern Cross"),
    ("southern cross", 12.5, -60.0, "the smallest constellation, Crux"),
    ("andromeda", 0.8, 38.0, "home of the Andromeda Galaxy"),
    ("pegasus", 22.7, 20.0, "the Winged Horse; the Great Square of Pegasus"),
    ("perseus", 3.2, 45.0, "the Hero, with the eclipsing binary Algol"),
    ("canis major", 6.8, -22.0, "the Great Dog, with Sirius"),
    ("virgo", 13.4, -4.0, "the Maiden, with Spica"),
    ("aquarius", 22.3, -11.0, "the Water Bearer"),
    ("pisces", 0.5, 13.0, "the Fishes"),
    ("aries", 2.6, 21.0, "the Ram"),
    ("cancer", 8.6, 20.0, "the Crab, with the Beehive Cluster"),
    ("libra", 15.2, -15.0, "the Scales"),
    ("capricornus", 21.0, -18.0, "the Sea Goat"),
    ("boötes", 14.7, 31.0, "the Herdsman, with Arcturus"),
    ("bootes", 14.7, 31.0, "the Herdsman, with Arcturus"),
    ("draco", 17.0, 65.0, "the Dragon, winding between the Dippers"),
    ("centaurus", 13.1, -47.0, "the Centaur, with Alpha Centauri and Hadar"),
    ("auriga", 6.0, 42.0, "the Charioteer, with Capella"),
    ("aquila", 19.7, 3.0, "the Eagle, with Altair"),
    ("hercules", 17.4, 27.0, "the Hero, with the globular cluster M13"),
];

pub(crate) const MONTH_NAMES: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// The month in which an object at right ascension `ra` hours is highest
/// in the sky at about 9 pm local time.
pub(crate) fn best_month(ra: f64) -> usize {
    // local sidereal time at midnight is about 0h on September 21 and gains
    // 2 hours a month; at 9 pm it is 3 hours less
    let months_after_sep = ((ra + 3.0) / 2.0).rem_euclid(12.0);
    ((8.0 + months_after_sep + 20.0 / 30.0).floor() as usize) % 12
}

fn visibility(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:when (?:can|could|do|should) (?:i|you|we) (?:see|view|observe|spot)|when is|best time to (?:see|view|observe))\s+(?:the\s+)?(?:constellation\s+)?(.+?)(?:\s+(?:visible|in the sky))?$").captures(l)?;
    let name = m[1].trim();
    let (ra, dec, note, label) = if let Some(c) = CONSTELLATIONS.iter().find(|c| c.0 == name) {
        (c.1, c.2, c.3.to_string(), capitalize(c.0))
    } else {
        let s = find_star(name)?;
        (ra_deg(s) / 15.0, dec_deg(s), format!("{} in {}", s.spectral, s.constellation), capitalize(s.names[0]))
    };
    let best = best_month(ra);
    let range = format!("{} to {}", MONTH_NAMES[(best + 10) % 12], MONTH_NAMES[(best + 2) % 12]);
    let lat = if dec >= 0.0 { format!("everywhere north of latitude {}°S", n(90.0 - dec)) } else { format!("everywhere south of latitude {}°N", n(90.0 + dec)) };
    Some(Answer::new(
        "visibility",
        vec![
            input(format!("{label} | best viewing time")),
            primary(pod("Result", "Best seen", text(format!("{} evenings (highest around 9 pm); well placed {range}", MONTH_NAMES[best])))),
            pod("Details", "Details", rows(vec![("right ascension", format!("about {}h", n(ra))), ("declination", format!("about {}°", n(dec))), ("visible from", lat), ("description", note)])),
        ],
    ))
}

fn finding(l: &str) -> Option<Answer> {
    let t = l.trim_end_matches(" in the sky").trim_end_matches(" in the night sky");
    let (title, main, more): (&str, &str, Vec<(&str, &str)>) = match t {
        "where can i find the north star" | "where is the north star" | "how do i find the north star" | "how to find the north star" | "where is polaris" | "how to find polaris" | "where can i find polaris" | "find the north star" => (
            "finding the North Star",
            "Look due north: Polaris sits at an altitude equal to your latitude. Follow the two Pointer stars at the end of the Big Dipper's bowl (Merak to Dubhe) about five times their separation to reach it.",
            vec![("brightness", "magnitude 2.0 — the 48th-brightest star, not the brightest"), ("from the Southern Hemisphere", "below the horizon; use the Southern Cross and the Southern Pointers to find the south celestial pole instead"), ("constellation", "end of the Little Dipper's handle (Ursa Minor)")],
        ),
        "where are the pointer stars" | "pointer stars" | "the pointer stars" | "what are the pointer stars" | "where are the pointers" => (
            "pointer stars",
            "Dubhe and Merak, the two stars at the front of the Big Dipper's bowl, point north to Polaris; in the southern sky Alpha and Beta Centauri (the Southern Pointers) point to the Southern Cross.",
            vec![("Dubhe", "RA 11h 03.7m, Dec +61° 45′, magnitude 1.8"), ("Merak", "RA 11h 01.8m, Dec +56° 23′, magnitude 2.4"), ("Alpha Centauri", "RA 14h 39.6m, Dec −60° 50′, magnitude −0.3"), ("Beta Centauri (Hadar)", "RA 14h 03.8m, Dec −60° 22′, magnitude 0.6")],
        ),
        _ => return None,
    };
    Some(Answer::new(title, vec![input(title.to_string()), primary(pod("Result", "Result", text(main))), pod("Details", "Details", rows(more.into_iter().map(|(a, b)| (a, b.to_string())).collect()))]))
}

// ---------------------------------------------------------------------------
// meteor showers
// ---------------------------------------------------------------------------

/// (name, peak month, peak day, zenithal hourly rate, parent body, active)
const SHOWERS: &[(&str, u32, u32, u32, &str, &str)] = &[
    ("Quadrantids", 1, 3, 110, "asteroid 2003 EH1", "Dec 28 – Jan 12"),
    ("Lyrids", 4, 22, 18, "comet C/1861 G1 (Thatcher)", "Apr 14 – Apr 30"),
    ("Eta Aquariids", 5, 6, 50, "comet 1P/Halley", "Apr 19 – May 28"),
    ("Southern Delta Aquariids", 7, 30, 25, "comet 96P/Machholz (probably)", "Jul 12 – Aug 23"),
    ("Perseids", 8, 12, 100, "comet 109P/Swift–Tuttle", "Jul 17 – Aug 24"),
    ("Draconids", 10, 8, 10, "comet 21P/Giacobini–Zinner", "Oct 6 – Oct 10"),
    ("Orionids", 10, 21, 20, "comet 1P/Halley", "Oct 2 – Nov 7"),
    ("Leonids", 11, 17, 15, "comet 55P/Tempel–Tuttle", "Nov 6 – Nov 30"),
    ("Geminids", 12, 14, 150, "asteroid 3200 Phaethon", "Dec 4 – Dec 20"),
    ("Ursids", 12, 22, 10, "comet 8P/Tuttle", "Dec 17 – Dec 26"),
];

fn meteor_showers(l: &str) -> Option<Answer> {
    let today = super::misc::today();
    let next_peak = |mo: u32, d: u32| -> NaiveDate {
        let this = NaiveDate::from_ymd_opt(today.year(), mo, d).unwrap();
        if this < today { NaiveDate::from_ymd_opt(today.year() + 1, mo, d).unwrap() } else { this }
    };
    let row = |s: &(&str, u32, u32, u32, &str, &str)| vec![text(s.0), text(super::misc::date_long(next_peak(s.1, s.2))), text(s.3.to_string()), text(s.4), text(s.5)];
    let head = Some(vec!["shower", "next peak", "ZHR", "parent", "active"]);
    if re!(r"^(?:list of |all )?(?:the\s+)?(?:annual |major )?meteor showers(?: this year| in \d{4})?$").is_match(l) || re!(r"^(?:when is |when's )?(?:the\s+)?next meteor shower$").is_match(l) {
        let mut v: Vec<&(&str, u32, u32, u32, &str, &str)> = SHOWERS.iter().collect();
        v.sort_by_key(|s| next_peak(s.1, s.2));
        let first = v[0];
        return Some(Answer::new("meteor showers", vec![input("major annual meteor showers"), primary(pod("Result", "Next major shower", text(format!("{} — peak {}", first.0, super::misc::date_long(next_peak(first.1, first.2)))))), pod("Table", "Major meteor showers", table(head, v.into_iter().map(row).collect()))]));
    }
    if re!(r"zenith(?:al)? hourly rate|\bzhr\b").is_match(l) {
        let picked: Vec<&(&str, u32, u32, u32, &str, &str)> = re!(r"\b(quadrantid|lyrid|eta aquariid|delta aquariid|perseid|draconid|orionid|leonid|geminid|ursid)s?\b").captures_iter(l).filter_map(|c| {
            let key = c[1].replace(' ', "");
            SHOWERS.iter().find(|s| s.0.to_lowercase().replace(' ', "").contains(&key))
        }).collect();
        if picked.is_empty() {
            return None;
        }
        let main = picked.iter().map(|s| format!("{}: {} per hour", s.0, s.3)).collect::<Vec<_>>().join(" | ");
        let mut pods = vec![input(format!("peak zenithal hourly rate | {}", picked.iter().map(|s| s.0).collect::<Vec<_>>().join(", "))), primary(pod("Result", "Peak zenithal hourly rate", text(main)))];
        if picked.len() == 2 {
            pods.push(pod("Ratio", "Ratio", text(fmt_float(picked[0].3 as f64 / picked[1].3 as f64, 4))));
        }
        return Some(Answer::new("meteor showers", pods));
    }
    let name = re!(r"\b(quadrantid|lyrid|eta aquariid|delta aquariid|perseid|draconid|orionid|leonid|geminid|ursid)s?\b").captures(l)?;
    if !re!(r"meteor|shower|peak|when").is_match(l) {
        return None;
    }
    let key = name[1].replace(' ', "");
    let s = SHOWERS.iter().find(|s| s.0.to_lowercase().replace(' ', "").contains(&key))?;
    let peak = next_peak(s.1, s.2);
    Some(Answer::new(
        "meteor shower",
        vec![
            input(format!("{} meteor shower", s.0)),
            primary(pod("Result", "Next peak", text(format!("night of {} (approximate; the exact peak shifts by about a day from year to year)", super::misc::date_long(peak))))),
            pod("Details", "Details", rows(vec![("zenithal hourly rate", format!("about {} meteors per hour under ideal skies", s.3)), ("active period", s.5.to_string()), ("parent body", s.4.to_string())])),
        ],
    ))
}

// ---------------------------------------------------------------------------
// perihelion, aphelion and comparisons of solar-system bodies
// ---------------------------------------------------------------------------

static MOONS: &[Body] = &[
    Body { name: "Phobos", kind: "moon of Mars", mass: 1.0659e16, radius: 1.1267e4, orbit: 9.376e6, period: 0.318_91, rotation: 7.654, temp: 233.0, moons: 0 },
    Body { name: "Deimos", kind: "moon of Mars", mass: 1.4762e15, radius: 6.2e3, orbit: 2.3463e7, period: 1.263, rotation: 30.312, temp: 233.0, moons: 0 },
    Body { name: "Io", kind: "moon of Jupiter", mass: 8.9319e22, radius: 1.8216e6, orbit: 4.217e8, period: 1.769, rotation: 42.46, temp: 110.0, moons: 0 },
    Body { name: "Europa", kind: "moon of Jupiter", mass: 4.7998e22, radius: 1.5608e6, orbit: 6.709e8, period: 3.551, rotation: 85.23, temp: 102.0, moons: 0 },
    Body { name: "Ganymede", kind: "moon of Jupiter", mass: 1.4819e23, radius: 2.6341e6, orbit: 1.0704e9, period: 7.155, rotation: 171.7, temp: 110.0, moons: 0 },
    Body { name: "Callisto", kind: "moon of Jupiter", mass: 1.0759e23, radius: 2.4103e6, orbit: 1.8827e9, period: 16.689, rotation: 400.5, temp: 134.0, moons: 0 },
    Body { name: "Titan", kind: "moon of Saturn", mass: 1.3452e23, radius: 2.5747e6, orbit: 1.221_87e9, period: 15.945, rotation: 382.68, temp: 94.0, moons: 0 },
    Body { name: "Enceladus", kind: "moon of Saturn", mass: 1.0802e20, radius: 2.521e5, orbit: 2.38e8, period: 1.370, rotation: 32.9, temp: 75.0, moons: 0 },
    Body { name: "Triton", kind: "moon of Neptune", mass: 2.139e22, radius: 1.3534e6, orbit: 3.5476e8, period: -5.877, rotation: 141.05, temp: 38.0, moons: 0 },
    Body { name: "Charon", kind: "moon of Pluto", mass: 1.586e21, radius: 6.06e5, orbit: 1.9591e7, period: 6.387, rotation: 153.3, temp: 53.0, moons: 0 },
    Body { name: "Ceres", kind: "dwarf planet", mass: 9.3839e20, radius: 4.697e5, orbit: 4.14e11, period: 1680.0, rotation: 9.074, temp: 168.0, moons: 0 },
];

fn any_body(w: &str) -> Option<&'static Body> {
    let w = w.trim().trim_start_matches("the ").trim();
    let w = match w {
        "our planet" | "our world" | "this planet" | "the earth" | "earth" => "earth",
        "our moon" | "the moon" | "luna" => "moon",
        "our sun" | "the sun" | "sol" => "sun",
        other => other.trim_start_matches("planet "),
    };
    BODIES.iter().chain(MOONS.iter()).find(|b| b.name.eq_ignore_ascii_case(w))
}

fn compare_bodies(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(mass|radius|diameter|circumference|surface gravity|gravity|density|volume|surface area|escape velocity|orbital period)\s+of\s+(.+)$").captures(l)?;
    let names: Vec<&str> = re!(r"\s*(?:,|\bvs\.?|\bversus\b|\band\b|\bcompared to\b)\s*").split(&m[2]).filter(|s| !s.trim().is_empty()).collect();
    if names.len() < 2 {
        // a single moon not in the main table
        let b = any_body(&m[2])?;
        if BODIES.iter().any(|x| x.name == b.name) {
            return None;
        }
        return compare_list(&m[1], vec![b]);
    }
    let bodies: Option<Vec<&'static Body>> = names.iter().map(|s| any_body(s)).collect();
    compare_list(&m[1], bodies?)
}

fn compare_list(prop: &str, bodies: Vec<&'static Body>) -> Option<Answer> {
    let g = 6.674_30e-11;
    let value = |b: &Body| -> (f64, String) {
        let pi = std::f64::consts::PI;
        match prop {
            "mass" => (b.mass, format!("{} kg", fmt_float(b.mass, 5))),
            "radius" => (b.radius, format!("{} km", n(b.radius / 1000.0))),
            "diameter" => (2.0 * b.radius, format!("{} km", n(2.0 * b.radius / 1000.0))),
            "circumference" => (2.0 * pi * b.radius, format!("{} km", n(2.0 * pi * b.radius / 1000.0))),
            "surface gravity" | "gravity" => (g * b.mass / b.radius.powi(2), format!("{} m/s²", n(g * b.mass / b.radius.powi(2)))),
            "density" => {
                let d = b.mass / (4.0 / 3.0 * pi * b.radius.powi(3));
                (d, format!("{} kg/m³", n(d)))
            }
            "volume" => {
                let v = 4.0 / 3.0 * pi * b.radius.powi(3);
                (v, format!("{} km³", fmt_float(v / 1e9, 5)))
            }
            "surface area" => (4.0 * pi * b.radius.powi(2), format!("{} km²", fmt_float(4.0 * pi * b.radius.powi(2) / 1e6, 5))),
            "escape velocity" => {
                let v = (2.0 * g * b.mass / b.radius).sqrt();
                (v, format!("{} km/s", n(v / 1000.0)))
            }
            _ => (b.period.abs(), format!("{} days", n(b.period.abs()))),
        }
    };
    let vals: Vec<(f64, String)> = bodies.iter().map(|b| value(b)).collect();
    let main = bodies.iter().zip(&vals).map(|(b, v)| format!("{}: {}", b.name, v.1)).collect::<Vec<_>>().join(" | ");
    let mut pods = vec![input(format!("{prop} of {}", bodies.iter().map(|b| b.name).collect::<Vec<_>>().join(" vs "))), primary(pod("Result", &capitalize(prop), text(main)))];
    if bodies.len() >= 2 {
        let base = &vals[0];
        let t: Vec<Vec<Content>> = bodies.iter().zip(&vals).skip(1).map(|(b, v)| vec![text(format!("{} / {}", bodies[0].name, b.name)), text(n(base.0 / v.0))]).collect();
        pods.push(pod("Ratios", "Ratios", table(None, t)));
    }
    Some(Answer::new(prop, pods))
}

fn apsides(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(perihelion|aphelion|shortest distance|closest distance|minimum distance|nearest distance|greatest distance|farthest distance|furthest distance|maximum distance|longest distance)\s+(?:of|between|from)\s+(?:the\s+)?(?:planet\s+)?([a-z ]+?)(?:\s+(?:to|from|and)\s+(?:our |the )?sun)?(?:\s+during (?:its |the )?orbit)?$").captures(l)?;
    let near = matches!(&m[1], "perihelion" | "shortest distance" | "closest distance" | "minimum distance" | "nearest distance");
    let b = any_body(&m[2])?;
    let name = if b.name == "Moon" { return None } else { b.name };
    let (a, e) = super::ephem::orbit_shape(name, (jd_now() - 2_451_545.0) / 36525.0)?;
    let au = 1.495_978_707e11;
    let d = if near { a * (1.0 - e) } else { a * (1.0 + e) };
    let other = if near { a * (1.0 + e) } else { a * (1.0 - e) };
    let mut r = vec![("semi-major axis", format!("{} au", n(a))), ("eccentricity", fmt_float(e, 4)), (if near { "aphelion" } else { "perihelion" }, format!("{} au = {} million km", n(other), n(other * au / 1e9)))];
    if name == "Earth" {
        r.push(("dates", "perihelion about January 3, aphelion about July 4".into()));
    }
    Some(Answer::new(
        if near { "perihelion" } else { "aphelion" },
        vec![
            input(format!("{name} | {}", if near { "perihelion distance" } else { "aphelion distance" })),
            primary(pod("Result", if near { "Perihelion" } else { "Aphelion" }, text(format!("{} au = {} million km ({} million miles)", n(d), n(d * au / 1e9), n(d * au / 1.609_344e9))))),
            pod("Orbit", "Orbit", rows(r)),
        ],
    ))
}

// ---------------------------------------------------------------------------
// spaceflight and astronomy history
// ---------------------------------------------------------------------------

/// (names, date, description)
const EVENTS: &[(&[&str], (i32, u32, u32), &str)] = &[
    (&["discovery of pluto", "pluto discovery", "pluto's discovery", "pluto discovered"], (1930, 2, 18), "Pluto discovered by Clyde Tombaugh at Lowell Observatory"),
    (&["discovery of neptune", "neptune discovery"], (1846, 9, 23), "Neptune observed by Johann Galle and Heinrich d'Arrest from Le Verrier's prediction"),
    (&["discovery of uranus", "uranus discovery"], (1781, 3, 13), "Uranus discovered by William Herschel"),
    (&["sputnik launch", "launch of sputnik", "sputnik 1 launch", "sputnik", "sputnik 1"], (1957, 10, 4), "Sputnik 1, the first artificial satellite, launched by the USSR"),
    (&["sputnik decay", "sputnik reentry", "sputnik re-entry", "sputnik 1 decay"], (1958, 1, 4), "Sputnik 1 re-entered the atmosphere and burned up"),
    (&["sputnik 2", "sputnik 2 launch", "laika launch"], (1957, 11, 3), "Sputnik 2 carried the dog Laika, the first animal to orbit the Earth"),
    (&["vostok 1", "first human in space", "first man in space", "gagarin's flight", "yuri gagarin's flight"], (1961, 4, 12), "Yuri Gagarin orbited the Earth once in Vostok 1 (108 minutes)"),
    (&["first american in space", "freedom 7"], (1961, 5, 5), "Alan Shepard's suborbital flight in Freedom 7 (15 minutes)"),
    (&["first woman in space", "vostok 6"], (1963, 6, 16), "Valentina Tereshkova flew Vostok 6 (almost 3 days)"),
    (&["first spacewalk", "voskhod 2"], (1965, 3, 18), "Alexei Leonov made the first spacewalk (12 minutes)"),
    (&["moon landing", "first moon landing", "apollo 11", "apollo 11 landing", "apollo 11 moon landing"], (1969, 7, 20), "Apollo 11: Neil Armstrong and Buzz Aldrin landed in the Sea of Tranquility"),
    (&["challenger disaster", "challenger explosion", "challenger"], (1986, 1, 28), "Space Shuttle Challenger (STS-51-L) broke apart 73 seconds after launch"),
    (&["columbia disaster", "columbia"], (2003, 2, 1), "Space Shuttle Columbia (STS-107) broke apart on re-entry"),
    (&["hubble launch", "hubble telescope launch", "hubble space telescope launch", "launch of hubble", "hubble telescope", "hubble space telescope", "hubble"], (1990, 4, 24), "the Hubble Space Telescope launched aboard Space Shuttle Discovery (STS-31); deployed April 25"),
    (&["james webb space telescope launch", "jwst launch", "webb telescope launch", "james webb space telescope", "jwst", "webb telescope"], (2021, 12, 25), "the James Webb Space Telescope launched on an Ariane 5 from Kourou"),
    (&["iss launch", "international space station launch", "first iss module launch"], (1998, 11, 20), "Zarya, the first module of the International Space Station, launched"),
    (&["voyager 1 launch", "voyager 1"], (1977, 9, 5), "Voyager 1 launched from Cape Canaveral"),
    (&["voyager 2 launch", "voyager 2"], (1977, 8, 20), "Voyager 2 launched from Cape Canaveral"),
    (&["curiosity landing", "mars curiosity rover landing", "curiosity rover landing"], (2012, 8, 6), "the Curiosity rover landed in Gale Crater, Mars (05:17 UTC)"),
    (&["opportunity launch", "opportunity rover launch", "opportunity rover launch date"], (2003, 7, 7), "the Opportunity rover (MER-B) launched from Cape Canaveral"),
    (&["galileo's discovery of jupiter's moons", "discovery of jupiter's moons", "discovery of the galilean moons"], (1610, 1, 7), "Galileo first observed Jupiter's moons"),
];

fn find_event(w: &str) -> Option<&'static (&'static [&'static str], (i32, u32, u32), &'static str)> {
    let w = w.trim().trim_start_matches("the ").trim_end_matches(" date").trim();
    EVENTS.iter().find(|e| e.0.iter().any(|n| *n == w))
}

fn event_date(e: &(&[&str], (i32, u32, u32), &str)) -> NaiveDate {
    NaiveDate::from_ymd_opt(e.1 .0, e.1 .1, e.1 .2).unwrap()
}

fn history(l: &str) -> Option<Answer> {
    // "Sputnik launch date to Sputnik decay date"
    if let Some(m) = re!(r"^(?:(?:time|days)\s+(?:from|between)\s+)?(?:the\s+)?(.+?)(?:\s+date)?\s+(?:to|and|until)\s+(?:the\s+)?(.+?)(?:\s+date)?$").captures(l) {
        if let (Some(a), Some(b)) = (find_event(&m[1]), find_event(&m[2])) {
            let (da, db) = (event_date(a), event_date(b));
            let days = (db - da).num_days();
            return Some(Answer::new("time between events", vec![input(format!("time from {} to {}", a.0[0], b.0[0])), primary(pod("Result", "Time span", text(super::misc::span_text(days)))), pod("Events", "Events", rows(vec![("from", format!("{} — {}", super::misc::date_long(da), a.2)), ("to", format!("{} — {}", super::misc::date_long(db), b.2))]))]));
        }
    }
    // "100 year anniversary of the discovery of Pluto"
    if let Some(m) = re!(r"^(?:the\s+)?(\d+)(?:st|nd|rd|th)?[- ]?(?:year\s+)?anniversary\s+of\s+(?:the\s+)?(.+)$").captures(l) {
        let e = find_event(&m[2])?;
        let k: i32 = m[1].parse().ok()?;
        let d = event_date(e);
        let ann = NaiveDate::from_ymd_opt(d.year() + k, d.month(), d.day()).or_else(|| NaiveDate::from_ymd_opt(d.year() + k, 3, 1))?;
        let diff = (ann - super::misc::today()).num_days();
        return Some(Answer::new("anniversary", vec![input(format!("{k}th anniversary of {}", e.0[0])), primary(pod("Result", "Date", text(super::misc::date_long(ann)))), pod("Event", "Event", text(format!("{} — {}", super::misc::date_long(d), e.2))), pod("Relative", "Time from today", text(if diff >= 0 { format!("in {}", super::misc::span_text(diff)) } else { format!("{} ago", super::misc::span_text(diff)) }))]));
    }
    // "When was the Hubble telescope launched?", "date of the first moon landing"
    if let Some(m) = re!(r"^(?:when (?:was|did|were)\s+(?:the\s+)?(.+?)(?:\s+(?:launched|launch|happen|occur|discovered|land|landed|take place))?|(?:the\s+)?(?:date|launch date|day) of\s+(?:the\s+)?(.+)|(?:the\s+)?(.+?) (?:launch )?date)$").captures(l) {
        let w = m.get(1).or(m.get(2)).or(m.get(3))?.as_str();
        let e = find_event(w).or_else(|| find_event(&format!("{w} launch")))?;
        let d = event_date(e);
        return Some(Answer::new("event date", vec![input(e.0[0].to_string()), primary(pod("Result", "Date", text(super::misc::date_long(d)))), pod("Event", "Event", text(e.2)), pod("Relative", "Time since", text(format!("{} ago", super::misc::span_text((super::misc::today() - d).num_days()))))]));
    }
    let facts: (&str, &str, Vec<Vec<Content>>) = match l {
        "first animal in orbit" | "first animal in space to orbit" | "first animal to orbit the earth" | "first animal to orbit earth" | "laika" | "laika the dog" => (
            "first animal in orbit",
            "Laika, a dog, aboard Sputnik 2 (launched November 3, 1957)",
            vec![vec![text("first animals in space (suborbital)"), text("fruit flies on a US V-2 rocket, February 20, 1947")], vec![text("first mammals to orbit and return alive"), text("dogs Belka and Strelka, Sputnik 5, August 19–20, 1960")], vec![text("first primate in space"), text("Albert II, a rhesus monkey, June 14, 1949 (suborbital)")]],
        ),
        "moon landing dates of the apollo space missions" | "apollo moon landings" | "apollo moon landing dates" | "apollo missions" | "apollo landings" | "moon landings" | "apollo moon missions" => (
            "Apollo Moon landings",
            "six crewed landings, July 20, 1969 – December 11, 1972",
            vec![
                vec![text("Apollo 11"), text("July 20, 1969"), text("Sea of Tranquility"), text("Armstrong, Aldrin")],
                vec![text("Apollo 12"), text("November 19, 1969"), text("Ocean of Storms"), text("Conrad, Bean")],
                vec![text("Apollo 14"), text("February 5, 1971"), text("Fra Mauro"), text("Shepard, Mitchell")],
                vec![text("Apollo 15"), text("July 30, 1971"), text("Hadley–Apennine"), text("Scott, Irwin")],
                vec![text("Apollo 16"), text("April 21, 1972"), text("Descartes Highlands"), text("Young, Duke")],
                vec![text("Apollo 17"), text("December 11, 1972"), text("Taurus–Littrow"), text("Cernan, Schmitt")],
            ],
        ),
        "crew aboard for the challenger disaster" | "challenger crew" | "crew of the challenger" | "challenger disaster crew" | "crew of challenger" | "who died in the challenger disaster" => (
            "Challenger crew",
            "Francis R. Scobee, Michael J. Smith, Ellison S. Onizuka, Judith A. Resnik, Ronald E. McNair, Gregory B. Jarvis, Christa McAuliffe",
            vec![
                vec![text("Francis R. (Dick) Scobee"), text("commander")],
                vec![text("Michael J. Smith"), text("pilot")],
                vec![text("Ellison S. Onizuka"), text("mission specialist")],
                vec![text("Judith A. Resnik"), text("mission specialist")],
                vec![text("Ronald E. McNair"), text("mission specialist")],
                vec![text("Gregory B. Jarvis"), text("payload specialist")],
                vec![text("Christa McAuliffe"), text("payload specialist (Teacher in Space)")],
            ],
        ),
        "how long was the vostok 1 mission" | "vostok 1 mission duration" | "duration of vostok 1" | "length of the vostok 1 mission" | "vostok 1 duration" => (
            "Vostok 1",
            "108 minutes (1 h 48 min), one orbit of the Earth",
            vec![vec![text("date"), text("April 12, 1961")], vec![text("cosmonaut"), text("Yuri Gagarin")], vec![text("orbit"), text("181 × 327 km, inclination 64.95°")], vec![text("landing"), text("Gagarin ejected and parachuted down near Engels, Saratov Oblast")]],
        ),
        "galileo's astronomical discoveries" | "galileo's discoveries" | "what did galileo discover" | "galileo discoveries" | "galileo's contributions to astronomy" => (
            "Galileo's astronomical discoveries",
            "Jupiter's four large moons, the phases of Venus, mountains and craters on the Moon, sunspots, the stars of the Milky Way and Saturn's odd shape",
            vec![
                vec![text("1609"), text("mountains and craters on the Moon")],
                vec![text("1610"), text("Io, Europa, Ganymede and Callisto orbiting Jupiter (January 7–13)")],
                vec![text("1610"), text("the Milky Way resolved into countless stars")],
                vec![text("1610"), text("Saturn's 'ears' (the rings, not yet recognised)")],
                vec![text("1610"), text("the full set of phases of Venus, evidence for the Sun-centred system")],
                vec![text("1612–1613"), text("sunspots and the Sun's rotation")],
            ],
        ),
        "newton's laws" | "newton's laws of motion" | "newtons laws" | "newtons laws of motion" | "three laws of motion" | "laws of motion" => (
            "Newton's laws of motion",
            "1. A body stays at rest or in uniform motion unless a net force acts on it. 2. F = m a (force equals the rate of change of momentum). 3. Every action has an equal and opposite reaction.",
            vec![vec![text("first law (inertia)"), text("Σ F = 0 ⇒ v constant")], vec![text("second law"), text("Σ F = d p/d t = m a")], vec![text("third law"), text("F_AB = −F_BA")], vec![text("law of universal gravitation"), text("F = G m₁ m₂ / r²")]],
        ),
        "what are the colors in a rainbow" | "colors in a rainbow" | "colors of the rainbow" | "rainbow colors" | "colours of the rainbow" | "what are the colors of the rainbow" => (
            "colors of the rainbow",
            "red, orange, yellow, green, blue, indigo, violet",
            vec![
                vec![text("red"), text("620–750 nm")],
                vec![text("orange"), text("590–620 nm")],
                vec![text("yellow"), text("570–590 nm")],
                vec![text("green"), text("495–570 nm")],
                vec![text("blue"), text("450–495 nm")],
                vec![text("indigo"), text("about 425–450 nm")],
                vec![text("violet"), text("380–425 nm")],
            ],
        ),
        _ => return None,
    };
    Some(Answer::new(facts.0, vec![input(facts.0.to_string()), primary(pod("Result", "Result", text(facts.1))), pod("Details", "Details", table(None, facts.2))]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    eclipses(l)
        .or_else(|| next_moon(l))
        .or_else(|| moon_phase(l))
        .or_else(|| meteor_showers(l))
        .or_else(|| sky_coordinates(l))
        .or_else(|| finding(l))
        .or_else(|| visibility(l))
        .or_else(|| apsides(l))
        .or_else(|| compare_bodies(l))
        .or_else(|| history(l))
        .or_else(|| stars(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eclipses_match_published_dates() {
        // NASA: total lunar 2025-09-07, total lunar 2026-03-03, partial lunar
        // 2026-08-28, total solar 2026-08-12, annular solar 2027-02-06
        let start = jd_of(NaiveDate::from_ymd_opt(2025, 9, 1).unwrap());
        let v = eclipses_after(start, None, 6);
        let got: Vec<(String, &str, bool)> = v.iter().map(|e| (datetime_of(e.jd).format("%Y-%m-%d").to_string(), e.kind, e.solar)).collect();
        assert_eq!(got[0], ("2025-09-07".to_string(), "total", false));
        assert_eq!(got[1], ("2025-09-21".to_string(), "partial", true));
        assert_eq!(got[2], ("2026-02-17".to_string(), "annular", true));
        assert_eq!(got[3], ("2026-03-03".to_string(), "total", false));
        assert_eq!(got[4], ("2026-08-12".to_string(), "total", true));
        assert_eq!(got[5], ("2026-08-28".to_string(), "partial", false));
    }

    #[test]
    fn full_moon_dates() {
        // full moons: 2026-01-03 10:03 UTC, 2026-12-24 01:28 UTC
        let j = next_phase(jd_of(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()), 180.0);
        assert_eq!(datetime_of(j).format("%Y-%m-%d").to_string(), "2026-01-03");
        let j = next_phase(jd_of(NaiveDate::from_ymd_opt(2026, 12, 20).unwrap()), 180.0);
        assert_eq!(datetime_of(j).format("%Y-%m-%d").to_string(), "2026-12-24");
    }
}
