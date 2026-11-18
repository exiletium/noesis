//! Counting word problems: people have things, gain or lose some, and the
//! question asks for one person's count, the total or a difference.
//!
//! "Jordan has 6 basketballs and Shaquille has 4. How many do they have together?"
//! "Tom has 3 apples and buys 5 more. He eats 2. How many apples does Tom have now?"
//! "Ann has 12 pencils and Bob has 7. How many more pencils does Ann have than Bob?"

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

fn number(w: &str) -> Option<f64> {
    const WORDS: [&str; 21] = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen", "twenty"];
    let w = w.trim().to_lowercase();
    match w.as_str() {
        "a" | "an" => return Some(1.0),
        "a dozen" | "dozen" => return Some(12.0),
        _ => {}
    }
    WORDS.iter().position(|x| *x == w).map(|i| i as f64).or_else(|| w.replace(',', "").parse().ok())
}

const NUM: &str = r"(\d[\d,]*(?:\.\d+)?|a dozen|zero|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|an?)";

pub fn handle(q: &Query) -> Option<Answer> {
    let raw = q.clean.trim();
    let l = raw.to_lowercase();
    if !l.contains("how many") || !re!(r"\b(?:has|have|had)\b").is_match(&l) {
        return None;
    }
    // people in order of appearance, with their counts
    let mut people: Vec<(String, f64)> = Vec::new();
    let mut item = String::new();
    let mut last: Option<usize> = None;
    let mut steps: Vec<String> = Vec::new();
    let mut to_whom: Option<String>;
    let clauses: Vec<&str> = re!(r"(?:[.;!?]\s+|,?\s+and\s+|,\s+then\s+|,\s+)").split(raw.trim_end_matches(['?', '.'])).collect();
    let question = clauses.iter().position(|c| c.to_lowercase().starts_with("how many"))?;
    let has = Regex::new(&format!(r"(?i)^(?:([A-Z][a-z]+|he|she|they|i|we)\s+)?(?:has|have|had|owns|collected|picked|bought)\s+{NUM}(?:\s+([a-z ]+?))?$")).ok()?;
    let gain = Regex::new(&format!(r"(?i)^(?:([A-Z][a-z]+|he|she|they)\s+)?(?:gets|got|buys|bought|finds|found|receives|received|picks|picked|is given|was given|earns|earned|collects|collected|wins|won|adds|added)\s+{NUM}(?:\s+more)?(?:\s+([a-z ]+?))?(?:\s+more)?$")).ok()?;
    let lose = Regex::new(&format!(r"(?i)^(?:([A-Z][a-z]+|he|she|they)\s+)?(?:gives away|gave away|loses|lost|eats|ate|sells|sold|uses|used|breaks|broke|spends|spent|drops|dropped|gives|gave)(?:\s+([A-Z][a-z]+|him|her|them|me))?\s+{NUM}(?:\s+(?:of\s+(?:them|his|her|their)\s+)?([a-z ]+?))?(?:\s+away\b.*)?$")).ok()?;
    let who = |name: Option<regex::Match>, people: &mut Vec<(String, f64)>, last: Option<usize>| -> Option<usize> {
        match name.map(|m| m.as_str()) {
            Some(n) if n.chars().next().is_some_and(|c| c.is_uppercase()) && !["He", "She", "They", "I", "We"].contains(&n) => {
                Some(people.iter().position(|p| p.0 == n).unwrap_or_else(|| {
                    people.push((n.to_string(), 0.0));
                    people.len() - 1
                }))
            }
            _ => last,
        }
    };
    for c in &clauses[..question] {
        let c = c.trim();
        if c.is_empty() {
            continue;
        }
        if let Some(m) = has.captures(c) {
            let i = who(m.get(1), &mut people, last)?;
            let v = number(&m[2])?;
            people[i].1 = v;
            if let Some(it) = m.get(3) {
                if item.is_empty() {
                    item = it.as_str().trim().to_string();
                }
            }
            steps.push(format!("{} starts with {}", people[i].0, fmt_float(v, 10)));
            last = Some(i);
        } else if let Some(m) = gain.captures(c) {
            let i = who(m.get(1), &mut people, last)?;
            let v = number(&m[2])?;
            people[i].1 += v;
            steps.push(format!("{} gains {} → {}", people[i].0, fmt_float(v, 10), fmt_float(people[i].1, 10)));
            last = Some(i);
        } else if let Some(m) = {
            // "gives 3 to Bob": take the recipient off before matching
            let t = re!(r"\s+to\s+([A-Z][a-z]+|him|her|them|me)$").captures(c);
            to_whom = t.as_ref().map(|t| t[1].to_string());
            lose.captures(t.map_or(c, |t| &c[..t.get(0).unwrap().start()]))
        } {
            let i = who(m.get(1), &mut people, last)?;
            let v = number(&m[3])?;
            people[i].1 -= v;
            steps.push(format!("{} loses {} → {}", people[i].0, fmt_float(v, 10), fmt_float(people[i].1, 10)));
            // a transfer to someone we track: "gives 3 to Bob", "gives Bob 3"
            if let Some(to) = m.get(2).map(|x| x.as_str().to_string()).or(to_whom.clone()) {
                if let Some(j) = people.iter().position(|p| p.0 == to) {
                    people[j].1 += v;
                    steps.push(format!("{} gains {} → {}", people[j].0, fmt_float(v, 10), fmt_float(people[j].1, 10)));
                }
            }
            last = Some(i);
        } else {
            // a clause we do not understand: better no answer than a wrong one
            return None;
        }
    }
    if people.is_empty() {
        return None;
    }
    let qtext = clauses[question..].join(" ").to_lowercase();
    let (title, v) = if let Some(m) = re!(r"how many (?:more|fewer|less)(?: [a-z ]+?)? (?:does|did) ([a-z]+) have than ([a-z]+)").captures(&qtext) {
        let a = people.iter().find(|p| p.0.to_lowercase() == m[1])?;
        let b = people.iter().find(|p| p.0.to_lowercase() == m[2])?;
        ("Difference", (a.1 - b.1).abs())
    } else if re!(r"\b(?:together|altogether|in all|in total|total|combined|both|they)\b").is_match(&qtext) && people.len() > 1 {
        ("Total", people.iter().map(|p| p.1).sum())
    } else if let Some(m) = re!(r"how many(?: [a-z ]+?)? (?:does|did|will) ([a-z]+) have").captures(&qtext) {
        let name = &m[1];
        let p = if ["he", "she"].contains(&name) { &people[last?] } else { people.iter().find(|p| p.0.to_lowercase() == *name)? };
        ("Result", p.1)
    } else if people.len() == 1 {
        ("Result", people[0].1)
    } else {
        return None;
    };
    if v < 0.0 {
        return None;
    }
    let item = if item.is_empty() { String::new() } else { format!(" {item}") };
    let mut pods = vec![
        pod("Input", "Input interpretation", text(raw.to_string())),
        primary(pod("Result", title, text(format!("{}{item}", fmt_float(v, 10))))),
    ];
    if people.len() > 1 {
        pods.push(pod("Counts", "Counts", table(None, people.iter().map(|p| vec![text(p.0.clone()), text(fmt_float(p.1, 10))]).collect())));
    }
    pods.push(pod("Steps", "Steps", text(steps.join("; "))));
    Some(Answer::new("word problem", pods))
}
