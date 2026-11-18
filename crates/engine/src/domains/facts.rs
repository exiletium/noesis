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

struct Character {
    name: &'static str,
    about: &'static str,
    works: &'static str,
    creators: &'static str,
    links: u32,
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
    characters: HashMap<String, Character>,
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
                "F" => {
                    let c = Character { name: at(1), about: at(2), works: at(3), creators: dedupe(at(4)), links: at(5).parse().unwrap_or(0) };
                    let key = norm(at(1));
                    if f.characters.get(&key).is_none_or(|old| old.links < c.links) {
                        f.characters.insert(key, c);
                    }
                }
                "A" => {
                    f.awards.insert((norm(at(1)), at(2).to_string()), at(3));
                }
                _ => {}
            }
        }
        for &(office, aliases, terms) in EXTRA_OFFICES {
            let id = match f.offices.get(&norm(office)) {
                Some(id) => *id,
                None => {
                    f.office_list.push(Office { name: office, terms: Vec::new() });
                    f.office_list.len() - 1
                }
            };
            let replaced: Vec<&str> = terms.iter().map(|t| t.0).collect();
            f.office_list[id].terms.retain(|t| !replaced.contains(&t.holder));
            for &(holder, start, end) in terms {
                f.office_list[id].terms.push(Term { holder, start, end, ordinal: None });
            }
            for a in std::iter::once(office).chain(aliases.iter().copied()) {
                f.offices.insert(norm(a), id);
            }
        }
        for o in &mut f.office_list {
            o.terms.sort_by_key(|t| sort_key(t.start));
        }
        for &(name, about, works, creators) in EXTRA_CHARACTERS {
            f.characters.entry(norm(name)).or_insert(Character { name, about, works, creators, links: 0 });
        }
        f
    })
}

/// Office terms missing from the generated data (which starts the British
/// monarchy at the 1927 change of title): office | aliases | (holder, start, end).
#[allow(clippy::type_complexity)]
const EXTRA_OFFICES: &[(&str, &[&str], &[(&str, &str, &str)])] = &[
    (
        "Monarch of the United Kingdom",
        &["king of england", "queen of england", "monarch of england", "english monarch", "king of britain", "queen of britain", "king of great britain", "queen of great britain", "british monarch", "king of the uk", "queen of the uk", "king of the united kingdom", "queen of the united kingdom", "monarch of the uk", "british king", "british queen"],
        &[
            ("Henry VII", "1485-08-22", "1509-04-21"), ("Henry VIII", "1509-04-22", "1547-01-28"), ("Edward VI", "1547-01-28", "1553-07-06"),
            ("Lady Jane Grey", "1553-07-10", "1553-07-19"), ("Mary I", "1553-07-19", "1558-11-17"), ("Elizabeth I", "1558-11-17", "1603-03-24"),
            ("James I", "1603-03-24", "1625-03-27"), ("Charles I", "1625-03-27", "1649-01-30"), ("Charles II", "1660-05-29", "1685-02-06"),
            ("James II", "1685-02-06", "1688-12-11"), ("William III and Mary II", "1689-02-13", "1694-12-28"), ("William III", "1694-12-28", "1702-03-08"),
            ("Anne", "1702-03-08", "1714-08-01"), ("George I", "1714-08-01", "1727-06-11"), ("George II", "1727-06-11", "1760-10-25"),
            ("George III", "1760-10-25", "1820-01-29"), ("George IV", "1820-01-29", "1830-06-26"), ("William IV", "1830-06-26", "1837-06-20"),
            ("Victoria", "1837-06-20", "1901-01-22"), ("Edward VII", "1901-01-22", "1910-05-06"), ("George V", "1910-05-06", "1936-01-20"),
        ],
    ),
    (
        "President of Sudan",
        &["president of sudan", "leader of sudan", "head of state of sudan"],
        &[
            ("Ismail al-Azhari", "1965-07-10", "1969-05-25"), ("Gaafar Nimeiry", "1969-05-25", "1985-04-06"), ("Abdel Rahman Swar al-Dahab", "1985-04-06", "1986-05-06"),
            ("Ahmed al-Mirghani", "1986-05-06", "1989-06-30"), ("Omar al-Bashir", "1989-06-30", "2019-04-11"), ("Ahmed Awad Ibn Auf", "2019-04-11", "2019-04-12"),
            ("Abdel Fattah al-Burhan", "2019-04-12", ""),
        ],
    ),
];

/// Well-known characters missing from the generated data.
const EXTRA_CHARACTERS: &[(&str, &str, &str, &str)] = &[
    ("Captain America", "superhero appearing in Marvel Comics (Steve Rogers)", "Captain America Comics (1941); Avengers; Captain America: The First Avenger; Captain America: The Winter Soldier; Captain America: Civil War", "Joe Simon; Jack Kirby"),
    ("Count Dracula", "vampire and title character of Bram Stoker's novel", "Dracula (1897); Nosferatu; Dracula (1931); Dracula (1958); Bram Stoker's Dracula", "Bram Stoker"),
    ("Garfield", "lasagna-loving orange cat of the comic strip Garfield (1978)", "Garfield (comic strip); Garfield and Friends; Garfield: The Movie; The Garfield Movie", "Jim Davis"),
    ("Pikachu", "Electric-type Pokémon and the franchise mascot", "Pokémon Red and Green (1996); Pokémon (anime); Pokémon Detective Pikachu", "Atsuko Nishida; Ken Sugimori"),
    ("Winnie-the-Pooh", "teddy bear of A. A. Milne's stories", "Winnie-the-Pooh (1926); The House at Pooh Corner (1928); The Many Adventures of Winnie the Pooh", "A. A. Milne; E. H. Shepard"),
];

/// Other names for characters in the data.
const CHARACTER_ALIASES: &[(&str, &str)] = &[
    ("isabella cullen", "bella swan"), ("bella cullen", "bella swan"), ("isabella swan", "bella swan"),
    ("dracula", "count dracula"), ("pooh", "winnie the pooh"), ("pooh bear", "winnie the pooh"),
    ("spongebob", "spongebob squarepants"), ("dumbledore", "albus dumbledore"), ("steve rogers", "captain america"),
    ("tony stark", "iron man"), ("peter parker", "spider man"), ("clark kent", "superman"), ("bruce wayne", "batman"),
];

/// Creator lists in the data repeat names ("Tex Avery; Tex Avery").
fn dedupe(s: &'static str) -> &'static str {
    let mut seen: Vec<&str> = Vec::new();
    for n in s.split("; ").filter(|n| !n.is_empty()) {
        if !seen.contains(&n) {
            seen.push(n);
        }
    }
    if seen.join("; ") == s {
        return s;
    }
    Box::leak(seen.join("; ").into_boxed_str())
}

/// A character by name, ignoring spaces and hyphens ("Spiderman") and allowing aliases.
fn character(name: &str) -> Option<&'static Character> {
    let f = facts();
    let k = norm(name);
    if let Some(c) = f.characters.get(&k) {
        return Some(c);
    }
    if let Some((_, to)) = CHARACTER_ALIASES.iter().find(|(a, _)| *a == k) {
        return f.characters.get(*to);
    }
    static COMPACT: OnceLock<HashMap<String, &'static Character>> = OnceLock::new();
    let compact = COMPACT.get_or_init(|| facts().characters.iter().map(|(k, c)| (k.replace(' ', ""), c)).collect());
    compact.get(&k.replace(' ', "")).copied()
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

/// Wikidata years are astronomical: 0 is 1 BC and −30 is 31 BC.
fn year_text(y: i32) -> String {
    if y <= 0 {
        format!("{} BC", 1 - y)
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

/// An event's dates as stored (astronomical years): (name, start, end, point in time).
pub(crate) fn event(name: &str) -> Option<(&'static str, &'static str, &'static str, &'static str)> {
    let f = facts();
    let e = &f.event_list[*f.events.get(&norm(name))?];
    Some((e.name, e.start, e.end, e.point))
}

/// A Wikidata person card: (display name, description, card pod).
pub(crate) fn person_card(name: &str) -> Option<(&'static str, &'static str, crate::engine::Pod)> {
    let p = facts().people.get(&norm(name))?;
    Some((p.name, p.about, person_pod(name)?))
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

/// Dates of events named in a clause: "Osama Bin Laden was killed", "WWI ended".
const EVENT_PHRASES: &[(&str, &str)] = &[
    ("osama bin laden was killed", "2011-05-02"), ("bin laden was killed", "2011-05-02"), ("osama bin laden died", "2011-05-02"), ("bin laden died", "2011-05-02"),
    ("gettysburg address", "1863-11-19"), ("south sudan's independence date", "2011-07-09"), ("south sudan's independence", "2011-07-09"),
    ("south sudan became independent", "2011-07-09"), ("moon landing", "1969-07-20"), ("apollo 11 landed", "1969-07-20"), ("berlin wall fell", "1989-11-09"),
    ("fall of the berlin wall", "1989-11-09"), ("titanic sank", "1912-04-15"), ("pearl harbor was attacked", "1941-12-07"), ("attack on pearl harbor", "1941-12-07"),
    ("jfk was assassinated", "1963-11-22"), ("kennedy was assassinated", "1963-11-22"), ("declaration of independence was signed", "1776-07-04"),
    ("9/11", "2001-09-11"), ("september 11 attacks", "2001-09-11"), ("lincoln was assassinated", "1865-04-14"),
];

fn event_date(phrase: &str) -> Option<(i32, u32, u32)> {
    let p = phrase.trim().trim_end_matches('?').trim().replace('\u{2019}', "'");
    let p = p.strip_prefix("the ").unwrap_or(&p);
    if let Some((_, d)) = EVENT_PHRASES.iter().find(|(k, _)| *k == p) {
        return parts(d);
    }
    if let Some(d) = super::misc::parse_date(p) {
        return Some((d.year(), d.month(), d.day()));
    }
    let f = facts();
    let (name, end) = if let Some(m) = re!(r"^(?:the\s+)?(end|start|beginning|outbreak)\s+of\s+(?:the\s+)?(.+)$").captures(p) {
        (m[2].to_string(), &m[1] == "end")
    } else {
        let m = re!(r"^(.+?)(?:\s+(ended|began|started|broke out|happened|took place|occurred|was signed))?$").captures(p)?;
        (m[1].to_string(), m.get(2).is_some_and(|x| x.as_str() == "ended"))
    };
    let e = &f.event_list[*f.events.get(&norm(&name))?];
    let d = if end && !e.end.is_empty() { e.end } else if !e.start.is_empty() { e.start } else { e.point };
    parts(d).filter(|d| d.1 > 0)
}

/// The holder of an office on a date (the later one on a handover day).
fn holder_on(office: &'static Office, d: (i32, u32, u32)) -> Option<&'static Term> {
    office.terms.iter().rfind(|t| sort_key(t.start) <= d && (t.end.is_empty() || parts(t.end).is_some_and(|e| e >= d)))
}

/// "leader of X", "ruler of X": the head of state, then of government.
fn office_or_leader(name: &str) -> Option<&'static Office> {
    find_office(name).or_else(|| {
        let rest = re!(r"^(?:leader|ruler|head)\s+of\s+(.+)$").captures(name)?.get(1)?.as_str().to_string();
        ["president of", "monarch of", "head of state of", "prime minister of"].iter().find_map(|p| find_office(&format!("{p} {rest}")))
    })
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
                    // the English crown long predates the holders listed here
                    None if office.name == "Monarch of the United Kingdom" => return None,
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
    // "Prime Minister of Pakistan when Osama Bin Laden was killed", "President of the US 87 years after the Gettysburg Address"
    let when = re!(r"^(?:who\s+(?:is|was)\s+(?:the\s+)?)?(?:the\s+)?(.+?)\s+(?:when|at the time of|on the day of|on|during)\s+(.+)$").captures(l).map(|m| (m[1].to_string(), m[2].to_string(), 0i32))
        .or_else(|| re!(r"^(?:who\s+(?:is|was)\s+(?:the\s+)?)?(?:the\s+)?(.+?)\s+(\d+)\s+years?\s+(after|before)\s+(.+)$").captures(l).map(|m| {
            let n: i32 = m[2].parse().unwrap_or(0);
            (m[1].to_string(), m[4].to_string(), if &m[3] == "after" { n } else { -n })
        }));
    if let Some((off, ev, shift)) = when {
        if let (Some(office), Some(d)) = (office_or_leader(&off), event_date(&ev)) {
            let d = (d.0 + shift, d.1, d.2);
            let term = holder_on(office, d)?;
            let date = NaiveDate::from_ymd_opt(d.0, d.1.max(1), d.2.max(1)).map(|x| x.format("%B %-d, %Y").to_string()).unwrap_or_else(|| d.0.to_string());
            return Some(simple(format!("{} | {}", office.name, date), "Result", term_text(term), person_pod(term.holder).into_iter().collect()));
        }
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
    // "leaders of New Zealand": the current heads of state and of government
    if let Some(m) = re!(r"^(?:who\s+are\s+)?(?:the\s+)?(?:current\s+)?(?:leaders|heads of state and government)\s+of\s+(?:the\s+)?(.+?)(?:\s+(?:today|now))?$").captures(l) {
        let place = m[1].to_string();
        let mut rows: Vec<(&str, &'static Office, &'static Term)> = Vec::new();
        let roles: [(&str, &[&str]); 2] = [
            ("head of state", &["president of", "monarch of", "king of", "emperor of", "head of state of"]),
            ("head of government", &["prime minister of", "chancellor of", "premier of", "head of government of"]),
        ];
        for (role, prefixes) in roles {
            let found = prefixes.iter().filter_map(|p| find_office(&format!("{p} {place}"))).find_map(|o| {
                let t = o.terms.iter().rfind(|t| sort_key(t.start) <= now && (t.end.is_empty() || parts(t.end).is_some_and(|e| e >= now)))?;
                Some((o, t))
            });
            if let Some((o, t)) = found {
                match rows.iter_mut().find(|r| std::ptr::eq(r.1, o)) {
                    Some(r) => r.0 = "head of state and government",
                    None => rows.push((role, o, t)),
                }
            }
        }
        if !rows.is_empty() {
            let value = rows.iter().map(|r| format!("{} ({})", r.2.holder, r.1.name)).collect::<Vec<_>>().join(" | ");
            let table_rows = rows.iter().map(|r| vec![text(r.0), text(r.1.name), text(r.2.holder), text(format!("since {}", short_date(r.2.start)))]).collect();
            let interp = format!("leaders of {}", place);
            return Some(Answer::new(interp.clone(), vec![
                pod("Input", "Input interpretation", text(interp)),
                primary(pod("Result", "Current leaders", text(value))),
                pod("Table", "Offices", table(Some(vec!["role", "office", "holder", "term"]), table_rows)),
                pod("Source", "Source", text("Wikidata (CC0)")),
            ]));
        }
    }
    // "who is the president of france", "list of presidents of the united states",
    // and a bare office "President of Argentina" (the current holder)
    let l = l.trim_end_matches(" today").trim_end_matches(" right now").trim_end_matches(" now");
    let bare = re!(r"^(?:the\s+)?(?:president|prime minister|premier|chancellor|king|queen|monarch|emperor|governor|head of state|head of government)\s+of\s+.+$").is_match(l);
    let m = re!(r"^(?:who\s+is\s+(?:the\s+)?|(?:list of\s+|all\s+)(?:the\s+)?|(?:the\s+)?)(.+)$").captures(l)?;
    if !bare && !l.starts_with("who is") && !l.starts_with("list of") && !l.starts_with("all ") {
        return None;
    }
    let office = find_office(m.get(1)?.as_str())?;
    if l.starts_with("who is") || bare {
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
    // the curated people table has checked dates and places
    if super::people::known(name) {
        return None;
    }
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

fn characters(l: &str) -> Option<Answer> {
    let (name, want) = if let Some(m) = re!(r"^who\s+(?:created|invented|wrote)\s+(?:the\s+character\s+)?(.+)$").captures(l) {
        (m.get(1)?.as_str(), "creator")
    } else if let Some(m) = re!(r"^(?:who\s+(?:is|was)\s+)?(?:the\s+)?creators?\s+of\s+(.+)$").captures(l) {
        (m.get(1)?.as_str(), "creator")
    } else if let Some(m) = re!(r"^(?:what|which)\s+(?:show|series|book|film|movie|work|game)s?\s+(?:is|was|does)\s+(.+?)\s+(?:in|from|appear in)$").captures(l) {
        (m.get(1)?.as_str(), "works")
    } else if let Some(m) = re!(r"^who\s+(?:is|was)\s+(.+)$").captures(l) {
        (m.get(1)?.as_str(), "about")
    } else {
        return None;
    };
    // "who created Bugs Bunny and Daffy Duck"
    if want == "creator" {
        let parts: Vec<&str> = re!(r"\s+and\s+|,\s*(?:and\s+)?").split(name).map(str::trim).filter(|p| !p.is_empty()).collect();
        if parts.len() > 1 {
            let cs: Option<Vec<&'static Character>> = parts.iter().map(|p| character(p)).collect();
            let cs = cs?;
            let rows = cs.iter().map(|c| vec![text(c.name), text(c.creators.replace("; ", ", "))]).collect();
            let short = cs.iter().map(|c| format!("{}: {}", c.name, c.creators.replace("; ", ", "))).collect::<Vec<_>>().join(" | ");
            return Some(simple(format!("{} (fictional characters) | creators", cs.iter().map(|c| c.name).collect::<Vec<_>>().join(", ")), "Creators", short, vec![pod("Table", "Creators", table(Some(vec!["character", "created by"]), rows))]));
        }
    }
    let c = character(name)?;
    let mut rows = vec![vec![text("type"), text("fictional character")]];
    if !c.about.is_empty() {
        rows.push(vec![text("description"), text(c.about)]);
    }
    if !c.works.is_empty() {
        rows.push(vec![text("appears in"), text(c.works.replace("; ", ", "))]);
    }
    if !c.creators.is_empty() {
        rows.push(vec![text("created by"), text(c.creators.replace("; ", ", "))]);
    }
    let card = pod("Character", c.name, table(None, rows));
    let (label, value) = match want {
        "creator" if !c.creators.is_empty() => ("creator", c.creators.replace("; ", "  |  ")),
        "works" if !c.works.is_empty() => ("appears in", c.works.replace("; ", "  |  ")),
        _ if !c.about.is_empty() => ("description", c.about.to_string()),
        _ => return None,
    };
    Some(simple(format!("{} (fictional character) | {label}", c.name), &format!("{}{}", label[..1].to_uppercase(), &label[1..]), value, vec![card]))
}

fn awards(l: &str) -> Option<Answer> {
    // "Nobel Prize 1930", "Nobel prizes in 1930", "1930 Nobel Prize winners"
    if let Some(m) = re!(r"^(?:(\d{4}) )?nobel (?:prizes?|laureates)(?: winners)?(?: (?:in |of )?(\d{4}))?$").captures(l) {
        let year = m.get(1).or(m.get(2))?.as_str();
        let order = ["Nobel Prize in Physics", "Nobel Prize in Chemistry", "Nobel Prize in Physiology or Medicine", "Nobel Prize in Literature", "Nobel Peace Prize", "Prize in Economic Sciences in Memory of Alfred Nobel"];
        let f = facts();
        let rows: Vec<Vec<Content>> = order.iter().filter_map(|a| {
            let w = f.awards.get(&(norm(a), year.to_string()))?;
            Some(vec![text(a.trim_start_matches("Nobel Prize in ").trim_start_matches("Prize in ").trim_end_matches(" in Memory of Alfred Nobel").replace("Nobel Peace Prize", "Peace")), text(w.replace("; ", ", "))])
        }).collect();
        if rows.is_empty() {
            return None;
        }
        let short = rows.iter().map(|r| format!("{}: {}", crate::engine::content_text(&r[0]), crate::engine::content_text(&r[1]))).collect::<Vec<_>>().join(" | ");
        return Some(simple(format!("Nobel Prizes | {year}"), "Laureates", short, vec![pod("Table", "Nobel Prizes", table(Some(vec!["prize", "laureates"]), rows))]));
    }
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


/// A work's title, allowing a possessive creator in front: "Edward Hopper's Nighthawks".
fn work_named(s: &str) -> Option<&'static Vec<Work>> {
    let f = facts();
    let s = s.trim().trim_start_matches("the painting ").trim_start_matches("the film ").trim_start_matches("the movie ").trim_start_matches("the novel ").trim_start_matches("the book ");
    f.works.get(&norm(s)).or_else(|| {
        let (who, title) = s.split_once("'s ").or_else(|| s.split_once("’s "))?;
        let all = f.works.get(&norm(title))?;
        let who = who.to_lowercase();
        all.iter().any(|w| w.names.to_lowercase().contains(who.split_whitespace().last().unwrap_or(&who))).then_some(all)
    })
}

fn work_card(all: &'static [Work]) -> Answer {
    let w = &all[0];
    let mut rows = vec![vec![text("type"), text(w.kind)], vec![text(w.role), text(w.names.replace("; ", ", "))]];
    if !w.year.is_empty() {
        rows.push(vec![text("date"), text(w.year)]);
    }
    let mut extra = vec![pod("Work", w.title, table(None, rows))];
    for name in w.names.split("; ").take(2) {
        extra.extend(person_pod(name));
    }
    let others: Vec<Vec<Content>> = all.iter().skip(1).take(6).map(|o| vec![text(format!("{} ({}{})", o.title, o.kind, if o.year.is_empty() { String::new() } else { format!(", {}", o.year) })), text(format!("{}: {}", o.role, o.names))]).collect();
    if !others.is_empty() {
        extra.push(pod("Others", "Other works with this title", table(None, others)));
    }
    let value = format!("{} by {}{}", w.kind, w.names.replace("; ", ", "), if w.year.is_empty() { String::new() } else { format!(" ({})", w.year) });
    simple(format!("{} ({})", w.title, w.kind), "Basic information", value, extra)
}

/// "When was Edward Hopper's Nighthawks completed?", "what year was Hamlet written"
fn when_made(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:when|what year|in what year|in which year)\s+(?:was|were|did)\s+(.+?)\s+(?:completed|painted|made|created|written|published|released|composed|finished|come out|came out|premiered?|first performed|sculpted|directed|recorded)$").captures(l)?;
    let all = work_named(&m[1])?;
    let w = all.iter().find(|w| !w.year.is_empty())?;
    Some(simple(format!("{} ({}) | date", w.title, w.kind), "Date", w.year.to_string(), vec![pod("Work", w.title, table(None, vec![vec![text("type"), text(w.kind)], vec![text(w.role), text(w.names.replace("; ", ", "))]]))]))
}

/// "sculptures by Picasso", "paintings by Frida Kahlo", "films directed by Spike Lee"
fn works_by(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:list of |all |famous |notable )?(paintings|sculptures|works of art|artworks|works|novels|books|plays|poems|films|movies|operas|symphonies|albums|songs|video games|games|musicals|tv shows|television series)\s+(?:by|of|from|directed by|written by|painted by|composed by|created by)\s+(.+)$").captures(l)?;
    let kinds: &[&str] = match &m[1] {
        "paintings" => &["painting"],
        "sculptures" => &["sculpture"],
        "works of art" | "artworks" => &["painting", "sculpture", "visual artwork"],
        "novels" => &["novel", "novella", "novel series"],
        "books" => &["book", "novel", "literary work", "written work", "novella", "book series", "novel series"],
        "plays" => &["play", "tragedy", "comedy"],
        "poems" => &["poem", "epic poem"],
        "films" | "movies" => &["film", "animated film", "short film"],
        "operas" => &["opera"],
        "symphonies" => &["symphony"],
        "albums" => &["album"],
        "songs" => &["song", "single"],
        "video games" | "games" => &["video game"],
        "musicals" => &["musical"],
        "tv shows" | "television series" => &["television series"],
        _ => &[],
    };
    let who = m[2].trim().to_lowercase();
    let who_words: Vec<&str> = who.split_whitespace().collect();
    if who_words.is_empty() {
        return None;
    }
    let matches = |names: &str| {
        names.split("; ").any(|n| {
            let n = n.to_lowercase();
            n == who || (who_words.len() == 1 && n.split_whitespace().last() == Some(who.as_str())) || (who_words.len() > 1 && n.contains(&who))
        })
    };
    let mut seen = std::collections::HashSet::new();
    let mut found: Vec<&'static Work> = facts().works.values().flatten().filter(|w| (kinds.is_empty() || kinds.contains(&w.kind)) && matches(w.names) && seen.insert(norm(w.title))).collect();
    if found.is_empty() {
        return None;
    }
    found.sort_by_key(|w| (sort_key(w.year), w.title));
    let creator = found[0].names.split("; ").find(|n| matches(n)).unwrap_or(found[0].names);
    let short = found.iter().take(40).map(|w| w.title).collect::<Vec<_>>().join(" | ");
    let rows = found.iter().take(60).map(|w| vec![text(w.title), text(w.kind), text(w.year)]).collect();
    let mut extra = vec![pod("Table", "Works", table(Some(vec!["title", "type", "date"]), rows))];
    extra.extend(person_pod(creator));
    Some(simple(format!("{} | {}", creator, &m[1]), "Works", short, extra))
}

/// Last resort, after every other handler: a bare title or character name,
/// typed with capitals ("Girl with a Pearl Earring", "Rear Window vs Vertigo", "Pikachu").
pub fn bare(q: &Query) -> Option<Answer> {
    let raw = q.clean.trim().trim_end_matches(['?', '.', '!']).trim();
    if raw.len() < 4 || raw.len() > 100 || !raw.chars().next().is_some_and(|c| c.is_uppercase() || c.is_ascii_digit()) {
        return None;
    }
    let parts: Vec<&str> = re!(r"\s+(?:vs\.?|versus|and)\s+|,\s+").split(raw).map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if parts.len() > 1 {
        let ws: Option<Vec<&'static Vec<Work>>> = parts.iter().map(|p| work_named(p)).collect();
        if let Some(ws) = ws {
            let rows = ws.iter().map(|all| {
                let w = &all[0];
                vec![text(w.title), text(w.kind), text(format!("{}: {}", w.role, w.names.replace("; ", ", "))), text(w.year)]
            });
            let short = ws.iter().map(|all| format!("{}: {} by {}{}", all[0].title, all[0].kind, all[0].names.replace("; ", ", "), if all[0].year.is_empty() { String::new() } else { format!(" ({})", all[0].year) })).collect::<Vec<_>>().join(" | ");
            return Some(simple(parts.join(" | "), "Comparison", short, vec![pod("Table", "Works", table(Some(vec!["title", "type", "creator", "date"]), rows.collect()))]));
        }
        let cs: Option<Vec<&'static Character>> = parts.iter().map(|p| character(p)).collect();
        let cs = cs?;
        let rows = cs.iter().map(|c| vec![text(c.name), text(c.about), text(c.works.replace("; ", ", ")), text(c.creators.replace("; ", ", "))]);
        let short = cs.iter().map(|c| format!("{}: {}", c.name, c.about)).collect::<Vec<_>>().join(" | ");
        return Some(simple(parts.join(" | "), "Comparison", short, vec![pod("Table", "Fictional characters", table(Some(vec!["character", "description", "appears in", "created by"]), rows.collect()))]));
    }
    if let Some(all) = work_named(raw) {
        return Some(work_card(all));
    }
    let c = character(raw)?;
    characters(&format!("who is {}", c.name.to_lowercase()))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?').trim();
    if l.len() < 6 || l.len() > 120 {
        return None;
    }
    // "who created superman" means the character, not the 1978 film
    if re!(r"^who\s+(?:created|invented)\s").is_match(l) {
        if let Some(a) = characters(l) {
            return Some(a);
        }
    }
    creators(l).or_else(|| when_made(l)).or_else(|| works_by(l)).or_else(|| awards(l)).or_else(|| offices(l)).or_else(|| events(l)).or_else(|| people(l)).or_else(|| characters(l))
}
