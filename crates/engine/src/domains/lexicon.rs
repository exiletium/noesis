//! Words: a bundled English word list and dictionary.
//!
//! The word list (274,000 words) answers anagram, pattern and "is it a
//! word" questions; the dictionary (77,000 entries from WordNet 3.1) gives
//! definitions and synonyms.  Both are stored deflate-compressed and
//! unpacked on first use.

use crate::engine::{pod, primary, table, text, Answer, Content, Query};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

static WORDS_Z: &[u8] = include_bytes!("../../data/words.z");
static DICT_Z: &[u8] = include_bytes!("../../data/dictionary.z");

fn inflate(data: &[u8]) -> &'static str {
    let bytes = miniz_oxide::inflate::decompress_to_vec(data).unwrap_or_default();
    Box::leak(String::from_utf8(bytes).unwrap_or_default().into_boxed_str())
}

fn words() -> &'static Vec<&'static str> {
    static W: OnceLock<Vec<&'static str>> = OnceLock::new();
    W.get_or_init(|| inflate(WORDS_Z).lines().collect())
}

/// word → (glosses "pos:text|…", synonyms "a,b,…")
fn dictionary() -> &'static HashMap<&'static str, (&'static str, &'static str)> {
    static D: OnceLock<HashMap<&'static str, (&'static str, &'static str)>> = OnceLock::new();
    D.get_or_init(|| {
        inflate(DICT_Z)
            .lines()
            .filter_map(|line| {
                let mut it = line.splitn(3, '\t');
                Some((it.next()?, (it.next()?, it.next().unwrap_or(""))))
            })
            .collect()
    })
}

pub fn is_word(w: &str) -> bool {
    words().binary_search(&w).is_ok() || dictionary().contains_key(w)
}

fn signature(w: &str) -> Vec<u8> {
    let mut v: Vec<u8> = w.bytes().collect();
    v.sort_unstable();
    v
}

pub fn anagrams(w: &str) -> Vec<&'static str> {
    let sig = signature(w);
    words().iter().copied().filter(|x| x.len() == w.len() && *x != w && signature(x) == sig).collect()
}

/// Base forms to try for an inflected word.
fn stems(w: &str) -> Vec<String> {
    let mut v = vec![w.to_string()];
    for (suffix, add) in [("ies", "y"), ("es", ""), ("s", ""), ("ed", ""), ("ed", "e"), ("ing", ""), ("ing", "e"), ("ly", ""), ("er", ""), ("est", "")] {
        if let Some(stem) = w.strip_suffix(suffix) {
            if stem.len() >= 2 {
                v.push(format!("{stem}{add}"));
            }
        }
    }
    v
}

fn part_of_speech(tag: &str) -> &'static str {
    match tag {
        "n" => "noun",
        "v" => "verb",
        "adj" => "adjective",
        _ => "adverb",
    }
}

fn definition_rows(entry: &str) -> Vec<Vec<Content>> {
    entry.split('|').filter_map(|g| g.split_once(':')).map(|(tag, gloss)| vec![text(part_of_speech(tag)), text(gloss.to_string())]).collect()
}

fn lookup(w: &str) -> Option<(String, &'static str, &'static str)> {
    stems(w).into_iter().find_map(|s| dictionary().get(s.as_str()).map(|(g, syn)| (s.clone(), *g, *syn)))
}

fn word_list(found: &[&str]) -> String {
    const SHOWN: usize = 120;
    let mut s = found.iter().take(SHOWN).copied().collect::<Vec<_>>().join("  |  ");
    if found.len() > SHOWN {
        s.push_str(&format!("  |  … ({} in total)", found.len()));
    }
    s
}

fn letters_score(w: &str) -> u32 {
    w.chars()
        .map(|c| match c {
            'a' | 'e' | 'i' | 'o' | 'u' | 'l' | 'n' | 's' | 't' | 'r' => 1,
            'd' | 'g' => 2,
            'b' | 'c' | 'm' | 'p' => 3,
            'f' | 'h' | 'v' | 'w' | 'y' => 4,
            'k' => 5,
            'j' | 'x' => 8,
            'q' | 'z' => 10,
            _ => 0,
        })
        .sum()
}

fn define(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:define|definition of|definition for|meaning of|the meaning of|what does)\s+(?:the word\s+)?([a-z]+)(?:\s+mean)?$|^([a-z]+)\s+(?:definition|meaning)$|^what is the (?:definition|meaning) of\s+(?:the word\s+)?([a-z]+)$").captures(l)?;
    let w = m.get(1).or(m.get(2)).or(m.get(3))?.as_str();
    let (base, glosses, syns) = lookup(w)?;
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{base} (English word) | definition")))];
    pods.push(primary(pod("Result", "Definitions", table(None, definition_rows(glosses)))));
    if !syns.is_empty() {
        pods.push(pod("Synonyms", "Synonyms", text(syns.replace(',', "  |  "))));
    }
    let others = anagrams(&base);
    if !others.is_empty() {
        pods.push(pod("Anagrams", "Anagrams", text(word_list(&others))));
    }
    pods.push(pod("Scrabble", "Scrabble score", text(format!("{} ({} letters)", letters_score(&base), base.len()))));
    pods.push(pod("Source", "Source", text("WordNet 3.1, Princeton University")));
    Some(Answer::new(format!("definition of {base}"), pods))
}

fn synonyms(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:synonyms?|another word|other words?|thesaurus)\s+(?:of|for)\s+(?:the word\s+)?([a-z]+)$|^([a-z]+)\s+synonyms?$").captures(l)?;
    let w = m.get(1).or(m.get(2))?.as_str();
    let (base, glosses, syns) = lookup(w)?;
    if syns.is_empty() {
        return None;
    }
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("{base} (English word) | synonyms"))),
        primary(pod("Result", "Synonyms", text(syns.replace(',', "  |  ")))),
        pod("Definitions", "Definitions", table(None, definition_rows(glosses))),
        pod("Source", "Source", text("WordNet 3.1, Princeton University")),
    ];
    Some(Answer::new(format!("synonyms of {base}"), pods))
}

fn anagram_query(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:anagrams?|word anagrams?)\s+(?:of|for)\s+(?:the word\s+)?([a-z]+)$|^([a-z]+)\s+anagrams?$|^(?:unscramble|descramble|rearrange the letters(?: of| in)?)\s+([a-z]+)$").captures(l)?;
    let w = m.get(1).or(m.get(2)).or(m.get(3))?.as_str();
    if w.len() < 2 || w.len() > 30 {
        return None;
    }
    let mut found = anagrams(w);
    let unscramble = m.get(3).is_some();
    if unscramble && is_word(w) {
        found.insert(0, words().iter().copied().find(|x| *x == w)?);
    }
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("anagrams | {w}")))];
    if found.is_empty() {
        pods.push(primary(pod("Result", "Result", text(format!("no English words can be formed from all the letters of \"{w}\"")))));
    } else {
        pods.push(primary(pod("Result", "Result", text(word_list(&found)))));
    }
    let mut counts = std::collections::BTreeMap::new();
    for ch in w.chars() {
        *counts.entry(ch).or_insert(0u32) += 1;
    }
    let mut total = crate::numtheory::factorial(w.len() as u64);
    for k in counts.values() {
        total /= crate::numtheory::factorial(*k as u64);
    }
    pods.push(pod("Arrangements", "Distinct arrangements of the letters", text(total.to_string())));
    Some(Answer::new(format!("anagrams of {w}"), pods))
}

fn patterns(l: &str) -> Option<Answer> {
    // crossword patterns: letters with _ or ? for unknowns
    if let Some(m) = re!(r"^(?:words?\s+(?:matching|like|of the form|that match(?:es)?|fitting)\s+|crossword\s+)?([a-z]*[_?][a-z_?]*)$").captures(l) {
        let pat = &m[1];
        if pat.len() < 2 || pat.len() > 30 || !pat.contains(|c: char| c.is_ascii_lowercase()) {
            return None;
        }
        let found: Vec<&str> = words().iter().copied().filter(|w| w.len() == pat.len() && w.bytes().zip(pat.bytes()).all(|(a, b)| b == b'_' || b == b'?' || a == b)).collect();
        let shown = pat.replace('?', "_");
        let mut pods = vec![pod("Input", "Input interpretation", text(format!("words matching {shown}")))];
        pods.push(primary(pod("Result", "Result", text(if found.is_empty() { "no words match".to_string() } else { word_list(&found) }))));
        pods.push(pod("Count", "Number of words", text(found.len().to_string())));
        return Some(Answer::new(format!("words matching {shown}"), pods));
    }
    let m = re!(r"^(?:(\d+)[- ]letter\s+)?words?\s+(?:that\s+|which\s+)?(start(?:s|ing)?|begin(?:s|ning)?|end(?:s|ing)?|contain(?:s|ing)?)\s+(?:with|in)?\s*(?:the letters?\s+)?([a-z]+)(?:\s+(?:and|that)\s+(end(?:s|ing)?|start(?:s|ing)?|begin(?:s|ning)?|contain(?:s|ing)?)\s+(?:with|in)?\s*([a-z]+))?(?:\s+with\s+(\d+)\s+letters)?$").captures(l)?;
    let len: Option<usize> = m.get(1).or(m.get(6)).and_then(|x| x.as_str().parse().ok());
    let test = |kind: &str, part: &str, w: &str| {
        if kind.starts_with("start") || kind.starts_with("begin") {
            w.starts_with(part)
        } else if kind.starts_with("end") {
            w.ends_with(part)
        } else {
            w.contains(part)
        }
    };
    let found: Vec<&str> = words()
        .iter()
        .copied()
        .filter(|w| len.is_none_or(|n| w.len() == n) && test(&m[2], &m[3], w) && m.get(4).is_none_or(|k| test(k.as_str(), &m[5], w)))
        .collect();
    let label = l.to_string();
    let mut pods = vec![pod("Input", "Input interpretation", text(label.clone()))];
    pods.push(primary(pod("Result", "Result", text(if found.is_empty() { "no words found".to_string() } else { word_list(&found) }))));
    pods.push(pod("Count", "Number of words", text(found.len().to_string())));
    Some(Answer::new(label, pods))
}

fn validity(l: &str) -> Option<Answer> {
    let m = re!(r#"^is\s+"?([a-z]+)"?\s+(?:a|an)\s+(?:real\s+|valid\s+|english\s+|scrabble\s+)*word(?:\s+in\s+(?:english|scrabble))?$"#).captures(l)?;
    let w = &m[1];
    let yes = is_word(w);
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("is \"{w}\" an English word?")))];
    pods.push(primary(pod("Result", "Result", text(if yes { format!("yes, \"{w}\" is in the word list") } else { format!("no, \"{w}\" is not in the word list") }))));
    if let Some((_, glosses, _)) = lookup(w) {
        pods.push(pod("Definitions", "Definitions", table(None, definition_rows(glosses))));
    }
    Some(Answer::new(format!("is {w} a word"), pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?').trim();
    anagram_query(l).or_else(|| define(l)).or_else(|| synonyms(l)).or_else(|| validity(l)).or_else(|| patterns(l))
}
