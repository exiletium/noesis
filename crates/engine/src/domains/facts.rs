//! General knowledge from a bundled Wikidata extract (CC0): who created
//! which work, when events began and ended, who held the top offices of
//! each country and when, Nobel prizes, and the people involved.
//!
//! `tools/datagen/wikidata.py` builds `data/facts.z`.

use super::misc::date_long;
use crate::engine::{pod, primary, table, text, Answer, Content, Query};
use chrono::{Datelike, NaiveDate};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

static FACTS_Z: &[u8] = include_bytes!("../../data/facts.z");

struct Work {
    title: &'static str,
    kind: &'static str,
    role: &'static str,
    names: &'static str,
    year: &'static str,
}

struct Event {
    name: &'static str,
    start: &'static str,
    end: &'static str,
    point: &'static str,
}

struct Term {
    holder: &'static str,
    start: &'static str,
    end: &'static str,
    ordinal: Option<u32>,
}

struct Office {
    name: &'static str,
    terms: Vec<Term>,
}

struct Person {
    name: &'static str,
    about: &'static str,
    born: &'static str,
    died: &'static str,
    place: &'static str,
}

#[derive(Default)]
struct Facts {
    works: HashMap<String, Vec<Work>>,
    events: HashMap<String, usize>,
    event_list: Vec<Event>,
    offices: HashMap<String, usize>,
    office_list: Vec<Office>,
    people: HashMap<String, Person>,
    /// (award, year) → winners
    awards: HashMap<(String, String), &'static str>,
}

/// Lower-case, drop punctuation and a leading article, and write Roman
/// numerals and number words as digits, so "World War II" = "world war 2".
fn norm(s: &str) -> String {
    let lower = s.to_lowercase().replace(['’', '\''], "");
    let cleaned: String = lower.chars().map(|c| if c.is_alphanumeric() { c } else { ' ' }).collect();
    let mut words: Vec<&str> = cleaned.split_whitespace().collect();
    if words.len() > 1 && matches!(words[0], "the" | "a" | "an") {
        words.remove(0);
    }
    words
        .into_iter()
        .map(|w| match w {
            "i" | "one" => "1",
            "ii" | "two" => "2",
            "iii" | "three" => "3",
            "iv" | "four" => "4",
            "v" | "five" => "5",
            "vi" | "six" => "6",
            "vii" | "seven" => "7",
            "viii" | "eight" => "8",
            "ix" | "nine" => "9",
            "x" | "ten" => "10",
            "ww1" | "wwi" => "world war 1",
            "ww2" | "wwii" => "world war 2",
            "us" | "usa" | "america" => "united states",
            "uk" | "britain" => "united kingdom",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn facts() -> &'static Facts {
    static F: OnceLock<Facts> = OnceLock::new();
    F.get_or_init(|| {
        let bytes = miniz_oxide::inflate::decompress_to_vec(FACTS_Z).unwrap_or_default();
        let data: &'static str = Box::leak(String::from_utf8(bytes).unwrap_or_default().into_boxed_str());
        let mut f = Facts::default();
        for line in data.lines() {
            let c: Vec<&'static str> = line.split('\t').collect();
            let at = |k: usize| c.get(k).copied().unwrap_or("");
            match at(0) {
                "W" => f.works.entry(norm(at(1))).or_default().push(Work { title: at(1), kind: at(2), role: at(3), names: at(4), year: at(5) }),
                "E" => {
                    let id = f.event_list.len();
                    f.event_list.push(Event { name: at(1), start: at(3), end: at(4), point: at(5) });
                    for name in std::iter::once(at(1)).chain(at(2).split('|')).filter(|n| !n.is_empty()) {
                        f.events.entry(norm(name)).or_insert(id);
                    }
                }
                "P" => {
                    let key = norm(at(1));
                    let id = match f.offices.get(&key) {
                        Some(id) => *id,
                        None => {
                            let id = f.office_list.len();
                            f.office_list.push(Office { name: at(1), terms: Vec::new() });
                            for name in std::iter::once(at(1)).chain(at(2).split('|')).filter(|n| !n.is_empty()) {
                                f.offices.entry(norm(name)).or_insert(id);
                            }
                            id
                        }
                    };
                    f.office_list[id].terms.push(Term { holder: at(3), start: at(4), end: at(5), ordinal: at(6).parse().ok() });
                }
                "H" => {
                    f.people.entry(norm(at(1))).or_insert(Person { name: at(1), about: at(2), born: at(3), died: at(4), place: at(5) });
                }
                "A" => {
                    f.awards.insert((norm(at(1)), at(2).to_string()), at(3));
                }
                _ => {}
            }
        }
        for o in &mut f.office_list {
            o.terms.sort_by_key(|t| sort_key(t.start));
        }
        f
    })
}

/// (year, month, day) of an ISO date or bare year; BCE years are negative.
fn parts(d: &str) -> Option<(i32, u32, u32)> {
    let (neg, body) = match d.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, d),
    };
    let mut it = body.split('-');
    let y: i32 = it.next()?.parse().ok()?;
    let y = if neg { -y } else { y };
    Some((y, it.next().and_then(|m| m.parse().ok()).unwrap_or(0), it.next().and_then(|d| d.parse().ok()).unwrap_or(0)))
}

fn sort_key(d: &str) -> (i32, u32, u32) {
    parts(d).unwrap_or((i32::MAX, 0, 0))
}

fn year_text(y: i32) -> String {
    if y <= 0 {
        format!("{} BC", -y)
    } else {
        y.to_string()
    }
}

/// "Sunday, September 2, 1945", or just the year when that is all that is known.
fn show_date(d: &str) -> String {
    match parts(d) {
        Some((y, m, day)) if m > 0 && day > 0 && y > 0 => NaiveDate::from_ymd_opt(y, m, day).map(date_long).unwrap_or_else(|| year_text(y)),
        Some((y, _, _)) => year_text(y),
        None => String::new(),
    }
}

/// "September 2, 1945" (no weekday).
fn short_date(d: &str) -> String {
    match parts(d) {
        Some((y, m, day)) if m > 0 && day > 0 && y > 0 => NaiveDate::from_ymd_opt(y, m, day).map(|x| x.format("%B %-d, %Y").to_string()).unwrap_or_else(|| year_text(y)),
        Some((y, _, _)) => year_text(y),
        None => String::new(),
    }
}

fn duration(a: &str, b: &str) -> Option<String> {
    let (p, q) = (parts(a)?, parts(b)?);
    if p.1 > 0 && q.1 > 0 && p.0 > 0 {
        let (d1, d2) = (NaiveDate::from_ymd_opt(p.0, p.1, p.2.max(1))?, NaiveDate::from_ymd_opt(q.0, q.1, q.2.max(1))?);
        let days = (d2 - d1).num_days();
        let mut years = d2.year() - d1.year();
        if (d2.month(), d2.day()) < (d1.month(), d1.day()) {
            years -= 1;
        }
        Some(if years >= 1 { format!("{years} years ({days} days)") } else { format!("{days} days") })
    } else {
        Some(format!("about {} years", q.0 - p.0))
    }
}

fn simple(interp: String, title: &str, value: String, extra: Vec<crate::engine::Pod>) -> Answer {
    let mut pods = vec![pod("Input", "Input interpretation", text(interp.clone())), primary(pod("Result", title, text(value)))];
    pods.extend(extra);
    pods.push(pod("Source", "Source", text("Wikidata (CC0)")));
    Answer::new(interp, pods)
}

fn person_pod(name: &str) -> Option<crate::engine::Pod> {
    let p = facts().people.get(&norm(name))?;
    let mut rows = Vec::new();
    if !p.about.is_empty() {
        rows.push(vec![text("description"), text(p.about)]);
    }
    if !p.born.is_empty() {
        rows.push(vec![text("born"), text(format!("{}{}", short_date(p.born), if p.place.is_empty() { String::new() } else { format!(", {}", p.place) }))]);
    }
    if !p.died.is_empty() {
        rows.push(vec![text("died"), text(short_date(p.died))]);
    }
    (!rows.is_empty()).then(|| pod("Person", p.name, table(None, rows)))
}

fn creators(l: &str) -> Option<Answer> {
    const ROLES: &[(&str, &[&str])] = &[
        ("wrote", &["author"]),
        ("directed", &["director"]),
        ("painted", &["painter", "artist"]),
        ("sculpted", &["sculptor", "artist"]),
        ("composed", &["composer"]),
        ("sang", &["performer"]),
        ("performed", &["performer"]),
        ("recorded", &["performer"]),
        ("developed", &["developer"]),
        ("created", &["creator", "author", "painter", "sculptor", "artist", "composer", "developer", "director"]),
        ("made", &["director", "creator", "developer", "painter", "sculptor", "artist", "performer", "composer", "author"]),
    ];
    let (verb, title): (&str, &str) = if let Some(m) = re!(r"^who\s+(wrote|directed|painted|sculpted|composed|sang|performed|recorded|developed|created|made)\s+(?:the\s+(?:book|novel|play|poem|film|movie|painting|song|opera|symphony|game|album|series|show)\s+)?(.+)$").captures(l) {
        (m.get(1)?.as_str(), m.get(2)?.as_str())
    } else if let Some(m) = re!(r"^(?:who\s+(?:is|was)\s+)?(?:the\s+)?(author|writer|director|painter|sculptor|composer|singer|performer|developer|creator|artist)\s+of\s+(.+)$").captures(l) {
        let verb = match m.get(1)?.as_str() {
            "author" | "writer" => "wrote",
            "director" => "directed",
            "painter" | "artist" => "painted",
            "sculptor" => "sculpted",
            "composer" => "composed",
            "singer" | "performer" => "sang",
            "developer" => "developed",
            _ => "created",
        };
        (verb, m.get(2)?.as_str())
    } else if let Some(m) = re!(r"^(.+?)\s+(author|writer|director|painter|composer)$").captures(l) {
        let verb = match m.get(2)?.as_str() {
            "author" | "writer" => "wrote",
            "director" => "directed",
            "painter" => "painted",
            _ => "composed",
        };
        (verb, m.get(1)?.as_str())
    } else if let Some(m) = re!(r"^(.+?)\s+(?:was\s+)?(written|directed|painted|composed)\s+by(?:\s+whom)?$").captures(l) {
        let verb = match m.get(2)?.as_str() {
            "written" => "wrote",
            "directed" => "directed",
            "painted" => "painted",
            _ => "composed",
        };
        (verb, m.get(1)?.as_str().trim_start_matches("who was ").trim_start_matches("who is "))
    } else {
        return None;
    };
    let all = facts().works.get(&norm(title))?;
    let wanted = ROLES.iter().find(|(v, _)| *v == verb)?.1;
    let w = wanted.iter().find_map(|role| all.iter().find(|w| w.role == *role)).or_else(|| (all.len() == 1 || verb == "made" || verb == "created").then(|| &all[0]))?;
    let mut extra = Vec::new();
    let mut rows = vec![vec![text("type"), text(w.kind)]];
    if !w.year.is_empty() {
        rows.push(vec![text("first published or released"), text(w.year)]);
    }
    extra.push(pod("Work", w.title, table(None, rows)));
    for name in w.names.split("; ").take(3) {
        extra.extend(person_pod(name));
    }
    let others: Vec<Vec<Content>> = all.iter().filter(|o| !std::ptr::eq(*o, w)).take(6).map(|o| vec![text(format!("{} ({}{})", o.title, o.kind, if o.year.is_empty() { String::new() } else { format!(", {}", o.year) })), text(format!("{}: {}", o.role, o.names))]).collect();
    if !others.is_empty() {
        extra.push(pod("Others", "Other works with this title", table(None, others)));
    }
    Some(simple(format!("{} | {}", w.title, w.role), &format!("{}{}", w.role[..1].to_uppercase(), &w.role[1..]), w.names.replace("; ", "  |  "), extra))
}

fn events(l: &str) -> Option<Answer> {
    let (name, want): (&str, &str) = if let Some(m) = re!(r"^when\s+(?:did|does|was|were)\s+(.+?)\s+(start(?:ed)?|begin|began|break out|end(?:ed)?|finish(?:ed)?|happen(?:ed)?|take place|occur(?:red)?|fought|signed|over)$").captures(l) {
        (m.get(1)?.as_str(), m.get(2)?.as_str())
    } else if let Some(m) = re!(r"^(?:the\s+)?(start|beginning|end|date|dates|year|duration|length)(?:\s+date)?\s+of\s+(.+)$").captures(l) {
        (m.get(2)?.as_str(), m.get(1)?.as_str())
    } else if let Some(m) = re!(r"^how long\s+(?:did|was)\s+(.+?)(?:\s+last)?$").captures(l) {
        (m.get(1)?.as_str(), "duration")
    } else if let Some(m) = re!(r"^(?:when|what year)\s+(?:was|were)\s+(.+)$").captures(l) {
        (m.get(1)?.as_str(), "date")
    } else if let Some(m) = re!(r"^(.+?)\s+(start|end|begin)\s+date$").captures(l) {
        (m.get(1)?.as_str(), m.get(2)?.as_str())
    } else {
        return None;
    };
    let f = facts();
    let e = &f.event_list[*f.events.get(&norm(name))?];
    let start = if e.start.is_empty() { e.point } else { e.start };
    let end = if e.end.is_empty() { e.point } else { e.end };
    let mut rows = Vec::new();
    if !e.start.is_empty() {
        rows.push(vec![text("start"), text(show_date(e.start))]);
    }
    if !e.end.is_empty() {
        rows.push(vec![text("end"), text(show_date(e.end))]);
    }
    if !e.point.is_empty() && e.start.is_empty() && e.end.is_empty() {
        rows.push(vec![text("date"), text(show_date(e.point))]);
    }
    let span = if e.start.is_empty() || e.end.is_empty() { None } else { duration(e.start, e.end) };
    if let Some(s) = &span {
        rows.push(vec![text("duration"), text(s.clone())]);
    }
    let dates = pod("Dates", e.name, table(None, rows));
    let (label, value) = if want.starts_with("start") || want.starts_with("beg") || want == "break out" {
        ("start date", show_date(start))
    } else if want.starts_with("end") || want.starts_with("finish") || want == "over" {
        ("end date", show_date(end))
    } else if want == "duration" || want == "length" {
        ("duration", span.clone()?)
    } else if !e.start.is_empty() && !e.end.is_empty() && e.start != e.end {
        ("dates", format!("{} to {}", short_date(e.start), short_date(e.end)))
    } else {
        ("date", show_date(start))
    };
    if value.is_empty() {
        return None;
    }
    Some(simple(format!("{} | {label}", e.name), &format!("{}{}", label[..1].to_uppercase(), &label[1..]), value, vec![dates]))
}

fn ordinal_word(w: &str) -> Option<u32> {
    const WORDS: [&str; 20] = ["first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth", "eleventh", "twelfth", "thirteenth", "fourteenth", "fifteenth", "sixteenth", "seventeenth", "eighteenth", "nineteenth", "twentieth"];
    if let Some(k) = WORDS.iter().position(|x| *x == w) {
        return Some(k as u32 + 1);
    }
    w.trim_end_matches(|c: char| c.is_alphabetic()).parse().ok().filter(|_| w.ends_with("st") || w.ends_with("nd") || w.ends_with("rd") || w.ends_with("th"))
}

fn term_text(t: &Term) -> String {
    let until = if t.end.is_empty() { "the present".to_string() } else { short_date(t.end) };
    format!("{} (from {} to {})", t.holder, short_date(t.start), until)
}

fn find_office(name: &str) -> Option<&'static Office> {
    let f = facts();
    let key = norm(name);
    let singular = key.strip_suffix('s').unwrap_or(&key);
    f.offices.get(&key).or_else(|| f.offices.get(singular)).map(|id| &f.office_list[*id])
}

fn offices(l: &str) -> Option<Answer> {
    let today = super::misc::today();
    let now = (today.year(), today.month(), today.day());
    // "the 16th president of the united states", "who was the first …", "last …", "current …"
    if let Some(m) = re!(r"^(?:who\s+(?:is|was)\s+)?(?:the\s+)?(\d+(?:st|nd|rd|th)|first|second|third|fourth|fifth|sixth|seventh|eighth|ninth|tenth|eleventh|twelfth|thirteenth|fourteenth|fifteenth|sixteenth|seventeenth|eighteenth|nineteenth|twentieth|last|latest|current|present)\s+(.+)$").captures(l) {
        let which = m.get(1)?.as_str();
        let office = find_office(m.get(2)?.as_str())?;
        let started: Vec<&Term> = office.terms.iter().filter(|t| sort_key(t.start) <= now).collect();
        let term: &Term = match which {
            "last" | "latest" | "current" | "present" => *started.last()?,
            _ => {
                let n = ordinal_word(which)?;
                match office.terms.iter().find(|t| t.ordinal == Some(n)) {
                    Some(t) => t,
                    None if office.terms.iter().any(|t| t.ordinal.is_some()) => return None,
                    None => {
                        // number the holders in order of their first term
                        let mut seen: Vec<&str> = Vec::new();
                        for t in &started {
                            if !seen.contains(&t.holder) {
                                seen.push(t.holder);
                            }
                        }
                        let holder = *seen.get(n as usize - 1)?;
                        *started.iter().find(|t| t.holder == holder)?
                    }
                }
            }
        };
        let mut extra = Vec::new();
        extra.extend(person_pod(term.holder));
        let at = office.terms.iter().position(|t| std::ptr::eq(t, term))?;
        let near: Vec<Vec<Content>> = office.terms.iter().enumerate().skip(at.saturating_sub(2)).take(5).map(|(_, t)| vec![text(t.holder), text(format!("{} to {}", short_date(t.start), if t.end.is_empty() { "present".to_string() } else { short_date(t.end) }))]).collect();
        extra.push(pod("Sequence", "Predecessors and successors", table(None, near)));
        return Some(simple(format!("{} | {which}", office.name), "Result", term_text(term), extra));
    }
    // "who was president of the united states in 1975", "president of france 1960"
    if let Some(m) = re!(r"^(?:who\s+(?:is|was)\s+)?(?:the\s+)?(.+?)\s+(?:in|during)?\s*(\d{4})$").captures(l) {
        let office = find_office(m.get(1)?.as_str())?;
        let y: i32 = m.get(2)?.as_str().parse().ok()?;
        let held: Vec<&Term> = office.terms.iter().filter(|t| sort_key(t.start).0 <= y && (t.end.is_empty() && y <= now.0 || parts(t.end).is_some_and(|e| e.0 >= y))).collect();
        if held.is_empty() {
            return None;
        }
        let mut extra = Vec::new();
        for t in &held {
            extra.extend(person_pod(t.holder));
        }
        return Some(simple(format!("{} | {y}", office.name), "Result", held.iter().map(|t| term_text(t)).collect::<Vec<_>>().join("\n"), extra));
    }
    // "who is the president of france", "list of presidents of the united states"
    let m = re!(r"^(?:who\s+is\s+(?:the\s+)?|(?:list of\s+|all\s+)(?:the\s+)?)(.+)$").captures(l)?;
    let office = find_office(m.get(1)?.as_str())?;
    if l.starts_with("who is") {
        let term = office.terms.iter().filter(|t| sort_key(t.start) <= now).next_back()?;
        let mut extra = Vec::new();
        extra.extend(person_pod(term.holder));
        return Some(simple(format!("{} | current", office.name), "Result", term_text(term), extra));
    }
    let rows: Vec<Vec<Content>> = office.terms.iter().map(|t| vec![text(t.ordinal.map(|o| o.to_string()).unwrap_or_default()), text(t.holder), text(short_date(t.start)), text(if t.end.is_empty() { "present".to_string() } else { short_date(t.end) })]).collect();
    let interp = format!("{} | holders", office.name);
    let pods = vec![pod("Input", "Input interpretation", text(interp.clone())), primary(pod("Result", "Holders", table(Some(vec!["", "name", "from", "to"]), rows))), pod("Source", "Source", text("Wikidata (CC0)"))];
    Some(Answer::new(interp, pods))
}

fn people(l: &str) -> Option<Answer> {
    let f = facts();
    let (name, want): (&str, &str) = if let Some(m) = re!(r"^(?:when|what year|where)\s+(?:was|did)\s+(.+?)\s+(born|die|died)$").captures(l) {
        (m.get(1)?.as_str(), if l.starts_with("where") { "birthplace" } else if m.get(2)?.as_str() == "born" { "born" } else { "died" })
    } else if let Some(m) = re!(r"^(?:the\s+)?(birthday|birth date|date of birth|birthdate|death date|date of death|birthplace|birth place|place of birth|age)\s+of\s+(.+)$").captures(l) {
        let w = m.get(1)?.as_str();
        (m.get(2)?.as_str(), if w.contains("place") { "birthplace" } else if w.contains("death") { "died" } else if w == "age" { "age" } else { "born" })
    } else if let Some(m) = re!(r"^(.+?)(?:'s)?\s+(birthday|birth date|date of birth|death date|date of death|birthplace|age)$").captures(l) {
        let w = m.get(2)?.as_str();
        (m.get(1)?.as_str(), if w.contains("place") { "birthplace" } else if w.contains("death") { "died" } else if w == "age" { "age" } else { "born" })
    } else if let Some(m) = re!(r"^how old\s+(?:is|was)\s+(.+?)(?:\s+when\s+(?:he|she|they)\s+died)?$").captures(l) {
        (m.get(1)?.as_str(), "age")
    } else if let Some(m) = re!(r"^who\s+(?:is|was)\s+(.+)$").captures(l) {
        (m.get(1)?.as_str(), "about")
    } else {
        return None;
    };
    let p = f.people.get(&norm(name))?;
    let card: Vec<crate::engine::Pod> = person_pod(p.name).into_iter().collect();
    let age = || -> Option<String> {
        let (b, today) = (parts(p.born)?, super::misc::today());
        let end = if p.died.is_empty() { (today.year(), today.month(), today.day()) } else { parts(p.died)? };
        let mut years = end.0 - b.0;
        if b.1 > 0 && end.1 > 0 && (end.1, end.2) < (b.1, b.2) {
            years -= 1;
        }
        Some(if p.died.is_empty() { format!("{years} years") } else { format!("{years} years (at death)") })
    };
    let (label, value) = match want {
        "born" => ("date of birth", show_date(p.born)),
        "died" => ("date of death", if p.died.is_empty() { return None } else { show_date(p.died) }),
        "birthplace" => ("place of birth", if p.place.is_empty() { return None } else { p.place.to_string() }),
        "age" => ("age", age()?),
        _ => ("description", if p.about.is_empty() { return None } else { p.about.to_string() }),
    };
    Some(simple(format!("{} | {label}", p.name), &format!("{}{}", label[..1].to_uppercase(), &label[1..]), value, card))
}

fn awards(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:who\s+won\s+)?(?:the\s+)?(?:winners?\s+of\s+(?:the\s+)?)?(\d{4}\s+)?nobel\s+(?:prize\s+)?(?:in|for)?\s*(physics|chemistry|literature|peace|medicine|physiology or medicine|economics|economic sciences)(?:\s+prize)?(?:\s+(?:winners?|laureates?))?(?:\s+(?:in\s+|of\s+)?(\d{4}))?$").captures(l)?;
    let year = m.get(1).or(m.get(3))?.as_str().trim();
    let field = m.get(2)?.as_str();
    let f = facts();
    let ((award, _), winners) = f.awards.iter().find(|((a, y), _)| y == year && a.contains("nobel") && (a.contains(field) || (field.starts_with("econ") && a.contains("econ")) || (field.contains("medicine") && a.contains("medicine"))))?;
    let mut extra = Vec::new();
    for name in winners.split("; ").take(4) {
        extra.extend(person_pod(name));
    }
    let _ = award;
    Some(simple(format!("Nobel Prize | {field} | {year}"), "Laureates", winners.replace("; ", "  |  "), extra))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?').trim();
    if l.len() < 6 || l.len() > 120 {
        return None;
    }
    creators(l).or_else(|| awards(l)).or_else(|| offices(l)).or_else(|| events(l)).or_else(|| people(l))
}
