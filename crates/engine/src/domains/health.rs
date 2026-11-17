//! Personal health calculators: blood alcohol (Widmark), heart-rate
//! training zones (Karvonen), pregnancy due dates (Naegele's rule) and
//! gestational age, visual acuity notations, blood pressure categories
//! (ACC/AHA 2017), and turkey servings and roasting times (USDA).

use crate::domains::misc::{date_long, parse_date, today};
use crate::engine::*;
use crate::print::fmt_float;
use chrono::{Duration, NaiveDate};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn n(v: f64) -> String {
    fmt_float(v, 4)
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn rows(r: Vec<(&str, String)>) -> Content {
    table(None, r.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())
}

fn get(l: &str, re: &Regex) -> Option<f64> {
    re.captures(l).and_then(|m| m[1].replace(',', "").parse().ok())
}

fn sex(l: &str) -> Option<bool> {
    if re!(r"\b(?:female|woman|women|girl|f)\b").is_match(l) {
        Some(false)
    } else if re!(r"\b(?:male|man|men|boy|m)\b").is_match(l) {
        Some(true)
    } else {
        None
    }
}

fn hours(h: f64) -> String {
    let m = (h * 60.0).round() as i64;
    if m < 60 { format!("{m} minutes") } else { format!("{} h {} min", m / 60, m % 60) }
}

fn bac(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    if !re!(r"^(?:bac|blood alcohol(?: content| concentration| level)?)\b").is_match(l) {
        return None;
    }
    let drinks = get(l, re!(r"([\d.]+)\s*(?:standard )?(?:drinks?|beers?|glasses|shots)"))?;
    let hrs = get(l, re!(r"([\d.]+)\s*(?:hours?|hrs?|h)\b")).unwrap_or(1.0);
    let male = sex(l).unwrap_or(true);
    let kg = get(l, re!(r"([\d.]+)\s*kg\b")).or_else(|| get(l, re!(r"([\d.]+)\s*(?:lbs?|pounds?)\b")).map(|p| p * 0.45359237)).unwrap_or(if male { 80.0 } else { 65.0 });
    let r = if male { 0.68 } else { 0.55 };
    let grams = drinks * 14.0;
    let peak = grams / (kg * 1000.0 * r) * 100.0;
    let now = (peak - 0.015 * hrs).max(0.0);
    let to_zero = now / 0.015;
    let to_legal = ((now - 0.08) / 0.015).max(0.0);
    let state = if now >= 0.08 { "over the US legal driving limit (0.08%)" } else if now >= 0.05 { "under the US limit but over the 0.05% limit used in most of Europe" } else if now > 0.0 { "under common legal driving limits" } else { "no alcohol expected in the blood" };
    Some(Answer::new("blood alcohol content", vec![
        input(format!("blood alcohol content | {} standard drinks over {} hours | {} | {} kg", n(drinks), n(hrs), if male { "male" } else { "female" }, n(kg))),
        primary(pod("Result", "Estimated blood alcohol content", text(format!("{}%  ({state})", fmt_float(now, 3))))),
        pod("Details", "Details", rows(vec![
            ("alcohol consumed", format!("{} g ethanol", n(grams))),
            ("peak BAC without elimination", format!("{}%", fmt_float(peak, 3))),
            ("time until below 0.08%", hours(to_legal)),
            ("time until sober", hours(to_zero)),
        ])),
        pod("Method", "Method", text("Widmark formula: BAC = A/(r W) × 100 − 0.015 t, with A grams of ethanol (14 g per US standard drink), W body mass in grams, r = 0.68 (male) or 0.55 (female), and elimination of 0.015% per hour. Individual results vary widely; never drive after drinking.")),
    ]))
}

fn heart_rate(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    if !re!(r"^(?:target |training |maximum |max )?heart rate(?: zones?)?\b").is_match(l) {
        return None;
    }
    let age = get(l, re!(r"(\d+)\s*(?:yo|y/o|years?(?: old)?|yr)\b")).or_else(|| get(l, re!(r"\bage\s*(?:=|of)?\s*(\d+)")))?;
    let rest = get(l, re!(r"(?:resting (?:hr|heart rate)\s*(?:=|of)?\s*|rhr\s*=?\s*)(\d+)")).unwrap_or(70.0);
    let max = 220.0 - age;
    let tanaka = 208.0 - 0.7 * age;
    let reserve = max - rest;
    let zones: Vec<Vec<Content>> = [(50.0, 60.0, "very light (recovery)"), (60.0, 70.0, "light (fat burning, endurance)"), (70.0, 80.0, "moderate (aerobic)"), (80.0, 90.0, "hard (anaerobic threshold)"), (90.0, 100.0, "maximum")]
        .iter()
        .map(|(a, b, name)| vec![text(format!("{a}–{b}%")), text(*name), text(format!("{}–{} bpm", (rest + reserve * a / 100.0).round(), (rest + reserve * b / 100.0).round()))])
        .collect();
    Some(Answer::new("heart rate zones", vec![
        input(format!("heart rate | age {} | resting heart rate {} bpm{}", n(age), n(rest), match sex(l) { Some(true) => " | male", Some(false) => " | female", None => "" })),
        primary(pod("Result", "Maximum heart rate", text(format!("{} bpm (220 − age); {} bpm (Tanaka: 208 − 0.7 × age)", n(max), n(tanaka))))),
        pod("Zones", "Training zones (Karvonen, percent of heart rate reserve)", table(Some(vec!["intensity", "zone", "heart rate"]), zones)),
        pod("Reserve", "Heart rate reserve", text(format!("{} bpm", n(reserve)))),
    ]))
}

fn date_any(s: &str) -> Option<NaiveDate> {
    if let Some(m) = re!(r"^(\d{1,2})/(\d{1,2})/(\d{2})$").captures(s.trim()) {
        let y: i32 = m[3].parse().ok()?;
        let y = if y < 70 { 2000 + y } else { 1900 + y };
        return NaiveDate::from_ymd_opt(y, m[1].parse().ok()?, m[2].parse().ok()?);
    }
    parse_date(s)
}

fn gestation_text(days: i64) -> String {
    format!("{} weeks {} days", days / 7, days % 7)
}

fn pregnancy(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    if let Some(m) = re!(r"^([\d.]+) (months?|weeks?) pregnant$").captures(l) {
        let v: f64 = m[1].parse().ok()?;
        let weeks = if m[2].starts_with('m') { v * 4.345 } else { v };
        let tri = if weeks < 14.0 { "first" } else if weeks < 28.0 { "second" } else { "third" };
        let due = today() + Duration::days(((40.0 - weeks) * 7.0).round() as i64);
        return Some(Answer::new("pregnancy", vec![
            input(format!("{} {} pregnant", m[1].to_string(), &m[2])),
            primary(pod("Result", "Gestational age", text(format!("about {} weeks (counted from the last menstrual period), {tri} trimester", n(weeks.round()))))),
            pod("Details", "Details", rows(vec![
                ("weeks since conception (fetal age)", format!("about {}", n((weeks - 2.0).round()))),
                ("weeks remaining", format!("about {}", n((40.0 - weeks).max(0.0).round()))),
                ("estimated due date if counted from today", date_long(due)),
            ])),
        ]));
    }
    let lmp_m = re!(r"(?:last (?:menstrual )?period|lmp)\s*(?:=|:|on|of|was)?\s*(.+)$").captures(l);
    let due_m = re!(r"^(?:pregnancy )?due date\s*(?:of|=|:|is|on)?\s*(.+)$").captures(l);
    let (lmp, label) = if let Some(m) = lmp_m {
        (date_any(m[1].trim())?, "last menstrual period")
    } else if let Some(m) = due_m {
        let s = m[1].trim();
        if s.starts_with("calculator") || s.is_empty() {
            return None;
        }
        (date_any(s)? - Duration::days(280), "due date")
    } else {
        return None;
    };
    let due = lmp + Duration::days(280);
    let conception = lmp + Duration::days(14);
    let ga = (today() - lmp).num_days();
    let mut r = vec![
        ("last menstrual period", date_long(lmp)),
        ("estimated conception", date_long(conception)),
        ("end of first trimester (13 weeks 6 days)", date_long(lmp + Duration::days(97))),
        ("end of second trimester (27 weeks 6 days)", date_long(lmp + Duration::days(195))),
        ("due date (40 weeks)", date_long(due)),
    ];
    if (0..=300).contains(&ga) {
        r.push(("gestational age today", gestation_text(ga)));
    }
    Some(Answer::new("pregnancy due date", vec![
        input(format!("pregnancy | given {label}: {}", date_long(if label == "due date" { due } else { lmp }))),
        primary(pod("Result", if label == "due date" { "Estimated conception date" } else { "Estimated due date" }, text(date_long(if label == "due date" { conception } else { due })))),
        pod("Timeline", "Timeline", rows(r)),
        pod("Method", "Method", text("Naegele's rule: due date = first day of the last menstrual period + 280 days; conception about 14 days after it")),
    ]))
}

fn vision(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    let m = re!(r"^(?:visual acuity |vision |snellen )?(\d+(?:\.\d+)?)/(\d+(?:\.\d+)?)(?: vision| visual acuity| eyesight)$").captures(l)?;
    let (a, b): (f64, f64) = (m[1].parse().ok()?, m[2].parse().ok()?);
    if !(a == 20.0 || a == 6.0 || a == 10.0) || b <= 0.0 {
        return None;
    }
    let dec = a / b;
    let logmar = (b / a).log10();
    let cat = if dec >= 0.5 {
        "normal or mild loss (no WHO impairment category)"
    } else if dec >= 0.3 {
        "mild visual impairment (WHO category 0)"
    } else if dec >= 0.1 {
        "moderate visual impairment (WHO category 1)"
    } else if dec >= 0.05 {
        "severe visual impairment (WHO category 2)"
    } else {
        "blindness (WHO categories 3–5)"
    };
    Some(Answer::new("visual acuity", vec![
        input(format!("visual acuity {}/{}", &m[1], &m[2])),
        primary(pod("Result", "Interpretation", text(format!("you see at {} {} what a person with normal vision sees at {} {}", &m[1], if a == 6.0 { "m" } else { "ft" }, &m[2], if a == 6.0 { "m" } else { "ft" })))),
        pod("Equivalents", "Equivalent notations", rows(vec![
            ("decimal acuity", n(dec)),
            ("logMAR", n(logmar)),
            ("Snellen (feet)", format!("20/{}", n(20.0 / dec))),
            ("Snellen (meters)", format!("6/{}", n(6.0 / dec))),
            ("minimum angle of resolution", format!("{} arcminutes", n(1.0 / dec))),
        ])),
        pod("Category", "Category", text(format!("{cat}{}", if dec <= 0.1 { "; legally blind in the US when this is the best corrected acuity (20/200 or worse)" } else if dec < 0.5 { "; below the 20/40 most US states require for an unrestricted driver's license" } else { "" }))),
    ]))
}

fn blood_pressure(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    let m = re!(r"^(?:blood pressure|bp)\s*(?:of|=|:)?\s*(\d{2,3})\s*/\s*(\d{2,3})(?:\s*mm ?hg)?$").captures(l)?;
    let (s, d): (f64, f64) = (m[1].parse().ok()?, m[2].parse().ok()?);
    let cat = if s > 180.0 || d > 120.0 {
        "hypertensive crisis (seek care immediately if symptomatic)"
    } else if s >= 140.0 || d >= 90.0 {
        "stage 2 hypertension"
    } else if s >= 130.0 || d >= 80.0 {
        "stage 1 hypertension"
    } else if s >= 120.0 {
        "elevated"
    } else if s < 90.0 || d < 60.0 {
        "low (hypotension range)"
    } else {
        "normal"
    };
    Some(Answer::new("blood pressure", vec![
        input(format!("blood pressure {}/{} mmHg", n(s), n(d))),
        primary(pod("Result", "Category (ACC/AHA 2017)", text(cat))),
        pod("Derived", "Derived quantities", rows(vec![("pulse pressure", format!("{} mmHg", n(s - d))), ("mean arterial pressure", format!("{} mmHg", n(d + (s - d) / 3.0)))])),
        pod("Categories", "Categories (systolic / diastolic, mmHg)", rows(vec![
            ("normal", "< 120 and < 80".into()),
            ("elevated", "120–129 and < 80".into()),
            ("stage 1 hypertension", "130–139 or 80–89".into()),
            ("stage 2 hypertension", "≥ 140 or ≥ 90".into()),
            ("hypertensive crisis", "> 180 and/or > 120".into()),
        ])),
    ]))
}

/// USDA roasting times at 325 °F: (max lb, unstuffed h lo–hi, stuffed h lo–hi).
const TURKEY: &[(f64, f64, f64, f64, f64)] = &[
    (12.0, 2.75, 3.0, 3.0, 3.5),
    (14.0, 3.0, 3.75, 3.5, 4.0),
    (18.0, 3.75, 4.25, 4.0, 4.25),
    (20.0, 4.25, 4.5, 4.25, 4.75),
    (24.0, 4.5, 5.0, 4.75, 5.25),
];

fn turkey(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    if !l.contains("turkey") {
        return None;
    }
    let lb = get(l, re!(r"([\d.]+)\s*(?:-?\s*)(?:lbs?|pounds?)")).or_else(|| get(l, re!(r"([\d.]+)\s*kg")).map(|k| k / 0.45359237))?;
    let feeds = l.contains("feed") || l.contains("serve") || l.contains("people");
    let cook = l.contains("cook") || l.contains("roast") || l.contains("time");
    if !feeds && !cook {
        return None;
    }
    let (lo_p, hi_p) = ((lb / 1.5).floor(), lb.floor());
    let row = TURKEY.iter().find(|r| lb <= r.0).copied().unwrap_or((lb, 5.0, 5.0 + (lb - 24.0) * 0.1, 5.25, 5.25 + (lb - 24.0) * 0.1));
    let roast = format!("{}–{} hours unstuffed, {}–{} hours stuffed, at 325 °F (163 °C)", n(row.1), n(row.2), n(row.3), n(row.4));
    let serves = format!("{}–{} people (1 to 1½ lb of whole bird per person)", lo_p, hi_p);
    Some(Answer::new("turkey", vec![
        input(format!("{} lb turkey", n(lb))),
        primary(pod("Result", if feeds { "Servings" } else { "Roasting time" }, text(if feeds { serves.clone() } else { roast.clone() }))),
        pod("Details", "Details", rows(vec![
            ("servings", serves),
            ("roasting time", roast),
            ("thawing in the refrigerator", format!("about {} days (24 hours per 4–5 lb)", n((lb / 4.5).ceil()))),
            ("thawing in cold water", format!("about {} hours (30 minutes per lb)", n(lb / 2.0))),
            ("safe internal temperature", "165 °F (74 °C) in the thickest part of the thigh, wing and breast".into()),
        ])),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    bac(q).or_else(|| heart_rate(q)).or_else(|| pregnancy(q)).or_else(|| vision(q)).or_else(|| blood_pressure(q)).or_else(|| turkey(q))
}
