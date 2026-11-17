//! Clock and calendar questions: a time in one place shown in another
//! ("3 pm in California is what time in Maine?"), the nth weekday of a month
//! or year, spans between dates, Unix time stamps, Julian ↔ Gregorian dates,
//! "is Easter on March 31st?", a year's holidays, Jewish and Islamic
//! holidays, "now" and facts about a year.

use super::calendars;
use super::geo::{cities, find_city, find_country};
use super::misc::{date_long, parse_date, today};
use crate::engine::*;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Weekday};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn rows(r: Vec<(&str, String)>) -> Content {
    table(None, r.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())
}

const US_STATES: &[(&str, &str)] = &[
    ("alabama", "America/Chicago"), ("alaska", "America/Anchorage"), ("arizona", "America/Phoenix"), ("arkansas", "America/Chicago"), ("california", "America/Los_Angeles"),
    ("colorado", "America/Denver"), ("connecticut", "America/New_York"), ("delaware", "America/New_York"), ("florida", "America/New_York"), ("georgia", "America/New_York"),
    ("hawaii", "Pacific/Honolulu"), ("idaho", "America/Boise"), ("illinois", "America/Chicago"), ("indiana", "America/Indiana/Indianapolis"), ("iowa", "America/Chicago"),
    ("kansas", "America/Chicago"), ("kentucky", "America/New_York"), ("louisiana", "America/Chicago"), ("maine", "America/New_York"), ("maryland", "America/New_York"),
    ("massachusetts", "America/New_York"), ("michigan", "America/Detroit"), ("minnesota", "America/Chicago"), ("mississippi", "America/Chicago"), ("missouri", "America/Chicago"),
    ("montana", "America/Denver"), ("nebraska", "America/Chicago"), ("nevada", "America/Los_Angeles"), ("new hampshire", "America/New_York"), ("new jersey", "America/New_York"),
    ("new mexico", "America/Denver"), ("new york state", "America/New_York"), ("north carolina", "America/New_York"), ("north dakota", "America/Chicago"), ("ohio", "America/New_York"),
    ("oklahoma", "America/Chicago"), ("oregon", "America/Los_Angeles"), ("pennsylvania", "America/New_York"), ("rhode island", "America/New_York"), ("south carolina", "America/New_York"),
    ("south dakota", "America/Chicago"), ("tennessee", "America/Chicago"), ("texas", "America/Chicago"), ("utah", "America/Denver"), ("vermont", "America/New_York"),
    ("virginia", "America/New_York"), ("washington state", "America/Los_Angeles"), ("west virginia", "America/New_York"), ("wisconsin", "America/Chicago"), ("wyoming", "America/Denver"),
];

const DEMONYMS: &[(&str, &str)] = &[
    ("japanese", "japan"), ("chinese", "china"), ("korean", "korea"), ("indian", "india"), ("british", "uk"), ("french", "france"), ("german", "germany"),
    ("italian", "italy"), ("spanish", "spain"), ("russian", "russia"), ("brazilian", "brazil"), ("mexican", "mexico"), ("canadian", "canada"),
    ("australian", "australia"), ("egyptian", "egypt"), ("turkish", "turkey"), ("thai", "thailand"), ("vietnamese", "vietnam"), ("nepali", "nepal"),
    ("pakistani", "pakistan"), ("israeli", "israel"), ("greek", "greece"), ("uruguayan", "uruguay"), ("argentine", "argentina"), ("argentinian", "argentina"),
    ("chilean", "chile"), ("peruvian", "peru"), ("colombian", "colombia"), ("kenyan", "kenya"), ("nigerian", "nigeria"), ("south african", "south africa"),
];

/// A place or zone name → (display name, IANA zone).
fn place(s: &str) -> Option<(String, String)> {
    let s = s.trim().trim_end_matches(['?', '.', ',']).trim();
    let s = re!(r"\s+(?:standard\s+)?(?:time(?:\s+zone)?|timezone)$").replace(s, "").to_string();
    let s = s.trim_start_matches("the ").trim();
    if s.is_empty() {
        return None;
    }
    if let Some((tz, name)) = crate::engine::daily::zone(s) {
        return Some((name.to_string(), tz.to_string()));
    }
    if let Some((st, tz)) = US_STATES.iter().find(|(n, _)| *n == s || n.trim_end_matches(" state") == s) {
        let mut c = st.chars();
        let name = c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default();
        return Some((name, tz.to_string()));
    }
    let s2 = DEMONYMS.iter().find(|(d, _)| *d == s).map(|(_, c)| c.to_string()).unwrap_or_else(|| s.to_string());
    if let Some((tz, name)) = crate::engine::daily::zone(&s2) {
        return Some((name.to_string(), tz.to_string()));
    }
    if let Some(c) = find_city(&s2) {
        return Some((c.name.clone(), c.timezone.clone()));
    }
    let c = find_country(&s2)?;
    let big = cities().iter().filter(|x| x.country == c.iso2).max_by_key(|x| x.population)?;
    Some((c.name.clone(), big.timezone.clone()))
}

/// "noon", "7pm", "12:35pm", "3 pm", "15:00", "2:34:06pm"
fn clock_time(s: &str) -> Option<NaiveTime> {
    let s = s.trim();
    match s {
        "noon" | "midday" => return NaiveTime::from_hms_opt(12, 0, 0),
        "midnight" => return NaiveTime::from_hms_opt(0, 0, 0),
        _ => {}
    }
    let c = re!(r"^(\d{1,2})(?::(\d\d))?(?::(\d\d))?\s*(am|pm|a\.m\.|p\.m\.)?$").captures(s)?;
    let mut h: u32 = c[1].parse().ok()?;
    let mi: u32 = c.get(2).map_or(0, |x| x.as_str().parse().unwrap_or(0));
    let se: u32 = c.get(3).map_or(0, |x| x.as_str().parse().unwrap_or(0));
    match c.get(4).map(|x| x.as_str().chars().next().unwrap_or(' ')) {
        Some('p') if h < 12 => h += 12,
        Some('a') if h == 12 => h = 0,
        None if c.get(2).is_none() => return None, // a bare number is not a time
        _ => {}
    }
    NaiveTime::from_hms_opt(h, mi, se)
}

const TIME: &str = r"(noon|midnight|midday|\d{1,2}(?::\d\d){0,2}\s*(?:am|pm|a\.m\.|p\.m\.)?)";

fn convert_time(l: &str) -> Option<Answer> {
    let pats = [
        format!(r"^(?:if|when) it(?: is|'s| was)\s+{TIME}\s+in\s+(.+?),?\s+what time (?:is it|will it be|would it be|was it) in\s+(.+)$"),
        format!(r"^{TIME}\s+in\s+(.+?)\s+(?:is\s+)?what time\s+(?:is it\s+)?in\s+(.+)$"),
        format!(r"^(?:convert\s+)?{TIME}\s+(?:in\s+)?(.+?)\s+(?:to|in|into)\s+(.+)$"),
        format!(r"^(?:what time is it in\s+|what time in\s+)(.+?)\s+when it(?: is|'s)\s+{TIME}\s+in\s+(.+)$"),
    ];
    for (k, p) in pats.iter().enumerate() {
        let r = Regex::new(p).ok()?;
        let Some(m) = r.captures(l) else { continue };
        let (t, a, b) = if k == 3 { (&m[2], &m[3], &m[1]) } else { (&m[1], &m[2], &m[3]) };
        let Some(time) = clock_time(t) else { continue };
        let (Some((na, za)), Some((nb, zb))) = (place(a), place(b)) else { continue };
        let tza: chrono_tz::Tz = za.parse().ok()?;
        let tzb: chrono_tz::Tz = zb.parse().ok()?;
        let day = chrono::Utc::now().with_timezone(&tza).date_naive();
        let src = tza.from_local_datetime(&day.and_time(time)).earliest()?;
        let dst = src.with_timezone(&tzb);
        let note = match (dst.date_naive() - day).num_days() {
            1 => " (the next day)",
            -1 => " (the previous day)",
            _ => "",
        };
        let diff = dst.offset().fix().local_minus_utc() - src.offset().fix().local_minus_utc();
        let h = diff as f64 / 3600.0;
        return Some(Answer::new(
            "time conversion",
            vec![
                input(format!("{} in {na} → {nb}", src.format("%-I:%M %p"))),
                primary(pod("Result", "Result", text(format!("{}{note}", dst.format("%-I:%M %p %Z"))))),
                pod("Details", "Details", rows(vec![
                    (na.as_str(), format!("{} ({za})", src.format("%-I:%M %p %Z, %A, %B %-d"))),
                    (nb.as_str(), format!("{} ({zb})", dst.format("%-I:%M %p %Z, %A, %B %-d"))),
                    ("difference", format!("{nb} is {} h {} {na}", crate::print::fmt_float(h.abs(), 4), if h >= 0.0 { "ahead of" } else { "behind" })),
                    ("UTC", src.with_timezone(&chrono::Utc).format("%H:%M UTC").to_string()),
                ])),
            ],
        ));
    }
    None
}

/// "12:35pm in Kathmandu": that time today in the place, in UTC and nearby zones.
fn time_in_place(l: &str) -> Option<Answer> {
    let m = Regex::new(&format!(r"^{TIME}\s+in\s+(.+)$")).ok()?.captures(l)?;
    let time = clock_time(&m[1])?;
    let (name, zone) = place(&m[2])?;
    let tz: chrono_tz::Tz = zone.parse().ok()?;
    let day = chrono::Utc::now().with_timezone(&tz).date_naive();
    let t = tz.from_local_datetime(&day.and_time(time)).earliest()?;
    let mut r = vec![("UTC", t.with_timezone(&chrono::Utc).format("%-I:%M %p, %A").to_string())];
    let others = [("US Eastern", "America/New_York"), ("US Pacific", "America/Los_Angeles"), ("London", "Europe/London"), ("Central Europe", "Europe/Paris"), ("India", "Asia/Kolkata"), ("Tokyo", "Asia/Tokyo")];
    for (n, z) in others {
        if z != zone {
            let tz2: chrono_tz::Tz = z.parse().ok()?;
            r.push((n, t.with_timezone(&tz2).format("%-I:%M %p %Z, %A").to_string()));
        }
    }
    Some(Answer::new(
        "time in a place",
        vec![
            input(format!("{} in {name}", t.format("%-I:%M %p"))),
            primary(pod("Result", "Result", text(format!("{} ({}, UTC{})", t.format("%-I:%M %p %Z, %A, %B %-d, %Y"), zone, t.format("%:z"))))),
            pod("Elsewhere", "The same moment elsewhere", rows(r)),
        ],
    ))
}

pub(crate) fn weekday(s: &str) -> Option<Weekday> {
    s.parse().ok()
}

pub(crate) fn ordinal_word(s: &str) -> Option<i32> {
    Some(match s {
        "first" | "1st" => 1,
        "second" | "2nd" => 2,
        "third" | "3rd" => 3,
        "fourth" | "4th" => 4,
        "fifth" | "5th" => 5,
        "last" => -1,
        _ => s.trim_end_matches(|c: char| c.is_ascii_alphabetic()).parse().ok()?,
    })
}

const MONTHS: [&str; 12] = ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];

pub(crate) fn month_num(s: &str) -> Option<u32> {
    let s = s.trim();
    MONTHS.iter().position(|m| *m == s || (s.len() >= 3 && m.starts_with(s))).map(|k| k as u32 + 1)
}

fn nth_weekday(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(first|second|third|fourth|fifth|last|\d+(?:st|nd|rd|th))\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)\s+(?:in|of)\s+(?:([a-z]+)(?:,?\s+(\d{4}))?|(\d{4}))$").captures(l)?;
    let n = ordinal_word(&m[1])?;
    let wd = weekday(&m[2])?;
    let t = today();
    let d = if let Some(y) = m.get(5) {
        // nth weekday of a year
        let y: i32 = y.as_str().parse().ok()?;
        if n > 0 {
            let jan1 = NaiveDate::from_ymd_opt(y, 1, 1)?;
            let first = jan1 + Duration::days(((wd.num_days_from_monday() as i64) - (jan1.weekday().num_days_from_monday() as i64)).rem_euclid(7));
            let d = first + Duration::weeks(n as i64 - 1);
            if d.year() != y {
                return None;
            }
            d
        } else {
            let dec31 = NaiveDate::from_ymd_opt(y, 12, 31)?;
            dec31 - Duration::days(((dec31.weekday().num_days_from_monday() as i64) - (wd.num_days_from_monday() as i64)).rem_euclid(7))
        }
    } else {
        let mo = month_num(m.get(3)?.as_str())?;
        let explicit = m.get(4).and_then(|y| y.as_str().parse::<i32>().ok());
        let find = |y: i32| -> Option<NaiveDate> {
            if n > 0 {
                NaiveDate::from_weekday_of_month_opt(y, mo, wd, n as u8)
            } else {
                let next = if mo == 12 { NaiveDate::from_ymd_opt(y + 1, 1, 1)? } else { NaiveDate::from_ymd_opt(y, mo + 1, 1)? };
                let last = next - Duration::days(1);
                Some(last - Duration::days(((last.weekday().num_days_from_monday() as i64) - (wd.num_days_from_monday() as i64)).rem_euclid(7)))
            }
        };
        match explicit {
            Some(y) => find(y)?,
            None => match find(t.year()) {
                Some(d) if d >= t => d,
                _ => find(t.year() + 1)?,
            },
        }
    };
    let days = (d - t).num_days();
    Some(Answer::new(
        "nth weekday",
        vec![
            input(m[0].to_string()),
            primary(pod("Result", "Result", text(d.format("%A, %B %-d, %Y").to_string()))),
            pod("Relative", "Time from today", text(if days >= 0 { format!("in {days} days") } else { format!("{} days ago", -days) })),
            pod("DayOfYear", "Day of the year", text(format!("day {} of {}", d.ordinal(), if d.leap_year() { 366 } else { 365 }))),
        ],
    ))
}

fn span_text(days: i64) -> String {
    let (w, d) = (days.abs() / 7, days.abs() % 7);
    format!("{} days  ({w} weeks {d} days)", days)
}

fn date_span(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:from\s+)?(.+?\d{4})\s+(?:to|until|till|through|-|–)\s+(.+?\d{4})$").captures(l)?;
    let (a, b) = (parse_date(&m[1])?, parse_date(&m[2])?);
    let days = (b - a).num_days();
    // calendar difference in years, months, days
    let (mut y, mut mo, mut d) = (b.year() - a.year(), b.month() as i32 - a.month() as i32, b.day() as i32 - a.day() as i32);
    if d < 0 {
        mo -= 1;
        let prev = if b.month() == 1 { NaiveDate::from_ymd_opt(b.year() - 1, 12, 1)? } else { NaiveDate::from_ymd_opt(b.year(), b.month() - 1, 1)? };
        d += (NaiveDate::from_ymd_opt(b.year(), b.month(), 1)? - prev).num_days() as i32;
    }
    if mo < 0 {
        y -= 1;
        mo += 12;
    }
    let plural = |n: i32, w: &str| format!("{n} {w}{}", if n == 1 { "" } else { "s" });
    Some(Answer::new(
        "date span",
        vec![
            input(format!("from {} to {}", date_long(a), date_long(b))),
            primary(pod("Result", "Result", text(format!("{}, {}, {}", plural(y, "year"), plural(mo, "month"), plural(d, "day"))))),
            pod("Days", "Total", rows(vec![("days", span_text(days)), ("hours", (days * 24).to_string()), ("years (365.2425 d)", crate::print::fmt_float(days as f64 / 365.2425, 6))])),
        ],
    ))
}

fn unix_time(l: &str) -> Option<Answer> {
    let m = re!(r"^(.+?)\s+(?:in|to|as)\s+unix(?:\s+time(?:stamp)?)?$").captures(l)?;
    let s = m[1].trim();
    let (time, date) = if let Some(c) = Regex::new(&format!(r"^{TIME},?\s+(?:on\s+)?(.+)$")).ok()?.captures(s) {
        (clock_time(&c[1])?, parse_date(&c[2])?)
    } else if let Some(c) = Regex::new(&format!(r"^(.+?),?\s+(?:at\s+)?{TIME}$")).ok()?.captures(s) {
        (clock_time(&c[2])?, parse_date(&c[1])?)
    } else {
        (NaiveTime::from_hms_opt(0, 0, 0)?, parse_date(s)?)
    };
    let dt = NaiveDateTime::new(date, time);
    let ts = dt.and_utc().timestamp();
    Some(Answer::new(
        "unix time",
        vec![
            input(format!("{} UTC → Unix time", dt.format("%-I:%M:%S %p, %A, %B %-d, %Y"))),
            primary(pod("Result", "Unix time", text(ts.to_string()))),
            pod("Details", "Details", rows(vec![("milliseconds", (ts * 1000).to_string()), ("assumption", "the time is taken as UTC (Unix time counts seconds since 1970-01-01 00:00 UTC)".into())])),
        ],
    ))
}

fn julian_gregorian(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:convert\s+)?(julian|gregorian)(?:\s+calendar)?(?:\s+date)?\s+(.+?)\s+(?:to|in|into)\s+(?:the\s+)?(julian|gregorian)(?:\s+calendar)?$").captures(l)?;
    if m[1] == m[3] {
        return None;
    }
    let d = parse_date(&m[2])?;
    let from = &m[1];
    let (out, outname) = if from == "julian" {
        let f = calendars::fixed_from_julian(d.year() as i64, d.month() as i64, d.day() as i64);
        (calendars::from_fixed(f)?.format("%A, %B %-d, %Y").to_string(), "Gregorian")
    } else {
        let (y, mo, da) = calendars::julian_from_fixed(calendars::fixed(d));
        (format!("{} {da}, {y}", ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"][(mo - 1) as usize]), "Julian")
    };
    Some(Answer::new(
        "calendar conversion",
        vec![
            input(format!("{} {} ({} calendar) → {outname} calendar", d.format("%B %-d,"), d.year(), from)),
            primary(pod("Result", &format!("{outname} calendar date"), text(out))),
            pod("Note", "Note", text("between 1900 and 2099 the Julian calendar runs 13 days behind the Gregorian")),
        ],
    ))
}

/// Jewish and Islamic holidays in a Gregorian year.
pub(crate) fn lunar_holiday(name: &str, y: i32) -> Option<NaiveDate> {
    let n = name.trim().trim_start_matches("the ");
    let heb = |hy: i64, m: i64, d: i64| calendars::from_fixed(calendars::fixed_from_hebrew(hy, m, d));
    let hy = y as i64 + 3760;
    let isl = |m: i64, d: i64| -> Option<NaiveDate> {
        let h = ((y as f64 - 622.0) * 33.0 / 32.0) as i64;
        (h - 1..=h + 2).filter_map(|hh| calendars::from_fixed(calendars::fixed_from_islamic(hh, m, d))).find(|g| g.year() == y)
    };
    match n {
        "rosh hashanah" | "rosh hashana" | "jewish new year" => heb(hy + 1, 7, 1),
        "yom kippur" => heb(hy + 1, 7, 10),
        "sukkot" => heb(hy + 1, 7, 15),
        "hanukkah" | "chanukah" | "hanukah" => heb(hy + 1, 9, 25),
        "passover" | "pesach" => heb(hy, 1, 15),
        "shavuot" => heb(hy, 3, 6),
        "ramadan" | "start of ramadan" | "beginning of ramadan" => isl(9, 1),
        "eid al-fitr" | "eid al fitr" | "eid ul-fitr" | "eid" => isl(10, 1),
        "eid al-adha" | "eid al adha" | "eid ul-adha" => isl(12, 10),
        "islamic new year" | "hijri new year" | "muharram" => isl(1, 1),
        "mawlid" | "mawlid an-nabi" => isl(3, 12),
        "ashura" => isl(1, 10),
        _ => None,
    }
}

fn lunar_holidays(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:when is |when was |date of )?(?:the )?([a-z' -]+?)(?: in)?(?: (\d{4}))?$").captures(l)?;
    let name = m[1].trim();
    let t = today();
    let (d, year_given) = match m.get(2) {
        Some(y) => (lunar_holiday(name, y.as_str().parse().ok()?)?, true),
        None => {
            let d = lunar_holiday(name, t.year())?;
            (if d < t { lunar_holiday(name, t.year() + 1)? } else { d }, false)
        }
    };
    let islamic = matches!(name, "ramadan" | "start of ramadan" | "beginning of ramadan" | "eid al-fitr" | "eid al fitr" | "eid ul-fitr" | "eid" | "eid al-adha" | "eid al adha" | "eid ul-adha" | "islamic new year" | "hijri new year" | "muharram" | "mawlid" | "mawlid an-nabi" | "ashura");
    let f = calendars::fixed(d);
    let mut pods = vec![
        input(format!("{name}{}", if year_given { format!(" {}", d.year()) } else { String::new() })),
        primary(pod("Result", "Date", text(d.format("%A, %B %-d, %Y").to_string()))),
        pod("Calendar", if islamic { "Islamic date" } else { "Hebrew date" }, text(if islamic { calendars::islamic_text(f) } else { calendars::hebrew_text(f) })),
    ];
    if islamic {
        pods.push(pod("Note", "Note", text("from the arithmetic Islamic calendar; dates set by moon sighting can differ by a day or two")));
    } else {
        pods.push(pod("Note", "Note", text("Jewish holidays begin at sundown on the evening before")));
    }
    let days = (d - t).num_days();
    pods.push(pod("Relative", "Time from today", text(if days >= 0 { format!("in {days} days") } else { format!("{} days ago", -days) })));
    Some(Answer::new("holiday", pods))
}

/// A date with or without a year ("March 31st" → this year).
fn date_any_year(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    match s {
        "yesterday" => return Some(today() - Duration::days(1)),
        "today" => return Some(today()),
        "tomorrow" => return Some(today() + Duration::days(1)),
        _ => {}
    }
    if let Some(m) = re!(r"^([a-z]+)\s+(\d{1,2})(?:st|nd|rd|th)?$").captures(s) {
        return NaiveDate::from_ymd_opt(today().year(), month_num(&m[1])?, m[2].parse().ok()?);
    }
    if let Some(m) = re!(r"^(?:the\s+)?(\d{1,2})(?:st|nd|rd|th)?\s+(?:of\s+)?([a-z]+)$").captures(s) {
        return NaiveDate::from_ymd_opt(today().year(), month_num(&m[2])?, m[1].parse().ok()?);
    }
    parse_date(s)
}

fn holiday_named(name: &str, y: i32) -> Option<NaiveDate> {
    let n = name.trim().trim_start_matches("the ").trim_end_matches(" day").to_string();
    super::knowledge::holiday_date(name.trim().trim_start_matches("the "), y)
        .map(|h| h.0)
        .or_else(|| super::knowledge::holiday_date(&format!("{n} day"), y).map(|h| h.0))
        .or_else(|| lunar_holiday(name, y))
}

fn holiday_check(l: &str) -> Option<Answer> {
    let (date, name) = if let Some(m) = re!(r"^(?:was|is|will)\s+(yesterday|today|tomorrow)\s+(?:be\s+)?(.+)$").captures(l) {
        (date_any_year(&m[1])?, m[2].to_string())
    } else if let Some(m) = re!(r"^(?:is|was|will)\s+(.+?)\s+(?:be\s+)?on\s+(.+)$").captures(l) {
        (date_any_year(&m[2])?, m[1].to_string())
    } else {
        return None;
    };
    let name = name.trim().trim_end_matches(" day").to_string();
    let actual = holiday_named(&name, date.year()).or_else(|| holiday_named(&format!("{name} day"), date.year()))?;
    let yes = actual == date;
    Some(Answer::new(
        "holiday check",
        vec![
            input(format!("is {} on {}?", name, date_long(date))),
            primary(pod("Result", "Result", text(if yes { "yes".to_string() } else { format!("no: in {} it is on {}", date.year(), actual.format("%A, %B %-d")) }))),
        ],
    ))
}

fn holidays_in_year(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:us |u\.s\. |american |federal )?holidays(?: in)? (\d{4})$").captures(l)?;
    let y: i32 = m[1].parse().ok()?;
    let names = [
        "new year's day", "martin luther king day", "presidents day", "easter", "memorial day", "juneteenth", "independence day", "labor day", "columbus day", "halloween", "veterans day", "thanksgiving", "christmas",
    ];
    let mut r: Vec<(NaiveDate, &str)> = names.iter().filter_map(|n| super::knowledge::holiday_date(n, y).map(|h| (h.0, *n))).collect();
    if y < 2021 {
        r.retain(|(_, n)| *n != "juneteenth");
    }
    r.sort();
    let title = |s: &str| s.split(' ').map(|w| { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() }).collect::<Vec<_>>().join(" ");
    Some(Answer::new(
        "holidays",
        vec![
            input(format!("United States holidays in {y}")),
            primary(pod("Result", "Holidays", table(Some(vec!["holiday", "date"]), r.iter().map(|(d, n)| vec![text(title(n)), text(d.format("%A, %B %-d").to_string())]).collect()))),
        ],
    ))
}

fn now(l: &str) -> Option<Answer> {
    if !matches!(l, "now" | "the time" | "current time" | "time now" | "right now" | "what time is it" | "current date and time" | "date and time") {
        return None;
    }
    let t = chrono::Local::now();
    let u = chrono::Utc::now();
    Some(Answer::new(
        "now",
        vec![
            input("current date and time"),
            primary(pod("Result", "Result", text(t.format("%-I:%M:%S %p %Z, %A, %B %-d, %Y").to_string()))),
            pod("Details", "Details", rows(vec![("UTC", u.format("%H:%M:%S, %Y-%m-%d").to_string()), ("Unix time", u.timestamp().to_string()), ("ISO week", format!("week {} of {}", t.iso_week().week(), t.iso_week().year())), ("day of the year", t.ordinal().to_string())])),
        ],
    ))
}

const ZODIAC: [&str; 12] = ["Rat", "Ox", "Tiger", "Rabbit", "Dragon", "Snake", "Horse", "Goat", "Monkey", "Rooster", "Dog", "Pig"];
const ELEMENTS: [&str; 5] = ["Wood", "Fire", "Earth", "Metal", "Water"];

fn year_facts(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?year\s+(\d{3,4})(?:\s+(?:ad|ce))?$").captures(l)?;
    let y: i32 = m[1].parse().ok()?;
    let jan1 = NaiveDate::from_ymd_opt(y, 1, 1)?;
    let leap = jan1.leap_year();
    let k = (y - 4).rem_euclid(60);
    let animal = ZODIAC[(k % 12) as usize];
    let element = ELEMENTS[((k % 10) / 2) as usize];
    let f = calendars::fixed(jan1);
    let fe = calendars::fixed(NaiveDate::from_ymd_opt(y, 12, 31)?);
    let roman = |mut n: i32| {
        let t = [(1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"), (50, "L"), (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")];
        let mut s = String::new();
        for (v, r) in t {
            while n >= v {
                s.push_str(r);
                n -= v;
            }
        }
        s
    };
    let (h1, _, _) = calendars::hebrew_from_fixed(f);
    let (h2, _, _) = calendars::hebrew_from_fixed(fe);
    let (i1, _, _) = calendars::islamic_from_fixed(f);
    let (i2, _, _) = calendars::islamic_from_fixed(fe);
    let ago = today().year() - y;
    Some(Answer::new(
        "year",
        vec![
            input(format!("the year {y}")),
            primary(pod("Result", "Calendar facts", rows(vec![
                ("leap year", if leap { "yes (366 days)".into() } else { "no (365 days)".into() }),
                ("January 1", jan1.format("%A").to_string()),
                ("December 31", NaiveDate::from_ymd_opt(y, 12, 31)?.format("%A").to_string()),
                ("Roman numerals", roman(y)),
                ("Chinese zodiac", format!("{element} {animal} (from Chinese New Year)")),
                ("Hebrew years", if h1 == h2 { h1.to_string() } else { format!("{h1}–{h2}") }),
                ("Islamic (AH) years", if i1 == i2 { i1.to_string() } else { format!("{i1}–{i2}") }),
                ("time since", if ago >= 0 { format!("{ago} years ago") } else { format!("in {} years", -ago) }),
            ]))),
        ],
    ))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    now(l)
        .or_else(|| convert_time(l))
        .or_else(|| time_in_place(l))
        .or_else(|| nth_weekday(l))
        .or_else(|| unix_time(l))
        .or_else(|| julian_gregorian(l))
        .or_else(|| holidays_in_year(l))
        .or_else(|| holiday_check(l))
        .or_else(|| lunar_holidays(l))
        .or_else(|| year_facts(l))
        .or_else(|| date_span(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lunar_holidays_known_dates() {
        // Rosh Hashanah 2015 began at sundown on September 13: 1 Tishri = September 14
        assert_eq!(lunar_holiday("rosh hashanah", 2015), NaiveDate::from_ymd_opt(2015, 9, 14));
        assert_eq!(lunar_holiday("passover", 2024), NaiveDate::from_ymd_opt(2024, 4, 23));
        assert_eq!(lunar_holiday("hanukkah", 2023), NaiveDate::from_ymd_opt(2023, 12, 8));
        let e = lunar_holiday("eid al-fitr", 2024).unwrap();
        assert!((e - NaiveDate::from_ymd_opt(2024, 4, 10).unwrap()).num_days().abs() <= 1, "{e}");
    }
}
