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
static CMU_Z: &[u8] = include_bytes!("../../data/cmudict.z");
static REL_Z: &[u8] = include_bytes!("../../data/relations.z");

/// word → (broader terms "a;b", narrower terms "c;d") from WordNet
fn relations() -> &'static HashMap<&'static str, (&'static str, &'static str)> {
    static D: OnceLock<HashMap<&'static str, (&'static str, &'static str)>> = OnceLock::new();
    D.get_or_init(|| {
        inflate(REL_Z)
            .lines()
            .filter_map(|line| {
                let c: Vec<&'static str> = line.splitn(4, '\t').collect();
                (c.len() == 4).then(|| (c[0], (c[2], c[3])))
            })
            .collect()
    })
}

/// "broader terms wine", "narrower terms green", "hypernyms of dog"
fn related(l: &str) -> Option<Answer> {
    let m = re!(r"^(broader|narrower|more general|more specific) (?:terms?|words?)(?: for| of| than)? (?:the word )?([a-z]+)$|^(hypernyms?|hyponyms?) (?:of |for )?(?:the word )?([a-z]+)$|^([a-z]+) (broader|narrower) terms$").captures(l)?;
    let kind = m.get(1).or(m.get(3)).or(m.get(6))?.as_str();
    let w = m.get(2).or(m.get(4)).or(m.get(5))?.as_str();
    let up = kind.starts_with("broader") || kind == "more general" || kind.starts_with("hyper");
    let (base, (b, n)) = stems(w).into_iter().find_map(|s| relations().get(s.as_str()).map(|r| (s.clone(), *r)))?;
    let list = if up { b } else { n };
    if list.is_empty() {
        return None;
    }
    let label = if up { "Broader terms" } else { "Narrower terms" };
    let items: Vec<&str> = list.split(';').collect();
    Some(Answer::new(format!("{} for {base}", label.to_lowercase()), vec![
        pod("Input", "Input interpretation", text(format!("{base} (English word) | {}", label.to_lowercase()))),
        primary(pod("Result", label, text(word_list(&items)))),
        pod("Source", "Source", text("WordNet 3.1, Princeton University (the two most common senses)")),
    ]))
}

/// word → ARPAbet pronunciations (CMU Pronouncing Dictionary)
fn cmudict() -> &'static HashMap<&'static str, Vec<&'static str>> {
    static D: OnceLock<HashMap<&'static str, Vec<&'static str>>> = OnceLock::new();
    D.get_or_init(|| {
        inflate(CMU_Z)
            .lines()
            .filter_map(|line| {
                let (w, p) = line.split_once('\t')?;
                Some((w, p.split('|').collect()))
            })
            .collect()
    })
}

fn is_vowel(ph: &str) -> bool {
    ph.ends_with(|c: char| c.is_ascii_digit())
}

/// ARPAbet → IPA with stress marks placed before the stressed syllable.
pub fn ipa(arpabet: &str) -> String {
    let ph: Vec<&str> = arpabet.split_whitespace().collect();
    let sym = |p: &str| -> &'static str {
        let (base, stress) = p.split_at(p.trim_end_matches(|c: char| c.is_ascii_digit()).len());
        match base {
            "AA" => "ɑ", "AE" => "æ", "AH" => if stress == "0" { "ə" } else { "ʌ" }, "AO" => "ɔ", "AW" => "aʊ", "AY" => "aɪ",
            "EH" => "ɛ", "ER" => if stress == "0" { "ɚ" } else { "ɝ" }, "EY" => "eɪ", "IH" => "ɪ", "IY" => "i", "OW" => "oʊ",
            "OY" => "ɔɪ", "UH" => "ʊ", "UW" => "u", "B" => "b", "CH" => "tʃ", "D" => "d", "DH" => "ð", "F" => "f", "G" => "ɡ",
            "HH" => "h", "JH" => "dʒ", "K" => "k", "L" => "l", "M" => "m", "N" => "n", "NG" => "ŋ", "P" => "p", "R" => "ɹ",
            "S" => "s", "SH" => "ʃ", "T" => "t", "TH" => "θ", "V" => "v", "W" => "w", "Y" => "j", "Z" => "z", "ZH" => "ʒ",
            _ => "",
        }
    };
    const ONSETS: &[&str] = &["P R", "P L", "B R", "B L", "T R", "D R", "K R", "K L", "G R", "G L", "F R", "F L", "TH R", "SH R", "S P", "S T", "S K", "S M", "S N", "S L", "S W", "T W", "K W", "D W", "P Y", "B Y", "K Y", "F Y", "M Y", "HH Y", "V Y"];
    // where each stressed syllable starts (index of its first phone)
    let mut marks: HashMap<usize, &str> = HashMap::new();
    let vowels: Vec<usize> = (0..ph.len()).filter(|&i| is_vowel(ph[i])).collect();
    let multi = vowels.len() > 1;
    for (k, &v) in vowels.iter().enumerate() {
        let mark = if ph[v].ends_with('1') { "ˈ" } else if ph[v].ends_with('2') { "ˌ" } else { continue };
        if !multi {
            continue;
        }
        let prev = if k == 0 { None } else { Some(vowels[k - 1]) };
        let start = match prev {
            None => 0,
            Some(pv) => {
                let cons: Vec<&str> = ph[pv + 1..v].to_vec();
                let n = cons.len();
                if n == 0 {
                    v
                } else if n >= 2 && ONSETS.contains(&cons[n - 2..].join(" ").as_str()) {
                    if n >= 3 && cons[n - 3] == "S" && ["P R", "T R", "K R", "P L", "K L", "K W"].contains(&cons[n - 2..].join(" ").as_str()) { v - 3 } else { v - 2 }
                } else {
                    v - 1
                }
            }
        };
        marks.insert(start, mark);
    }
    let mut out = String::new();
    for (i, p) in ph.iter().enumerate() {
        if let Some(m) = marks.get(&i) {
            out.push_str(m);
        }
        out.push_str(sym(p));
    }
    out
}

fn rhyme_part(arpabet: &str) -> String {
    let ph: Vec<&str> = arpabet.split_whitespace().collect();
    let last = ph.iter().rposition(|p| p.ends_with('1')).or_else(|| ph.iter().rposition(|p| is_vowel(p))).unwrap_or(0);
    ph[last..].iter().map(|p| p.trim_end_matches(|c: char| c.is_ascii_digit())).collect::<Vec<_>>().join(" ")
}

fn rhymes_of(w: &str) -> Vec<&'static str> {
    static R: OnceLock<HashMap<String, Vec<&'static str>>> = OnceLock::new();
    let index = R.get_or_init(|| {
        let mut m: HashMap<String, Vec<&'static str>> = HashMap::new();
        for (word, ps) in cmudict() {
            if word.chars().all(|c| c.is_ascii_lowercase()) && (dictionary().contains_key(word) || words().binary_search(word).is_ok()) {
                m.entry(rhyme_part(ps[0])).or_default().push(word);
            }
        }
        m
    });
    let Some(ps) = cmudict().get(w) else { return vec![] };
    let mut v: Vec<&'static str> = index.get(&rhyme_part(ps[0])).map(|x| x.iter().copied().filter(|x| *x != w && !x.ends_with(w)).collect()).unwrap_or_default();
    // common words (with a dictionary entry) first, then shorter
    v.sort_by_key(|x| (!dictionary().contains_key(x), x.len(), *x));
    v
}

fn syllables(arpabet: &str) -> usize {
    arpabet.split_whitespace().filter(|p| is_vowel(p)).count()
}

fn pron_rows(w: &str) -> Option<Vec<(String, String)>> {
    let ps = cmudict().get(w)?;
    let mut rows = vec![("IPA".to_string(), ps.iter().map(|p| format!("/{}/", ipa(p))).collect::<Vec<_>>().join(", "))];
    rows.push(("ARPAbet".to_string(), ps[0].to_string()));
    let n = syllables(ps[0]);
    rows.push(("syllables".to_string(), n.to_string()));
    Some(rows)
}

/// "pronunciation of colonel", "phonetic hypothesize", "how to pronounce quinoa",
/// "syllables in banana", "rhymes with orange", "words close to house"
fn sounds(l: &str) -> Option<Answer> {
    let src = pod("Source", "Source", text("CMU Pronouncing Dictionary (Carnegie Mellon University); American English"));
    if let Some(m) = re!(r"^(?:pronunciation|pronounciation|phonetics?|phonetic spelling|phonetic transcription|ipa|ipa transcription)(?: of| for)? (?:the word )?([a-z']+)$|^(?:how (?:do (?:you|i|we) |to |should i |is )?(?:pronounce|say)|pronounce) (?:the word )?([a-z']+)(?: pronounced)?$|^([a-z']+) (?:pronunciation|phonetics?|in ipa|ipa)$").captures(l) {
        let w = m.get(1).or(m.get(2)).or(m.get(3))?.as_str();
        let rows = pron_rows(w)?;
        return Some(Answer::new(format!("pronunciation of {w}"), vec![
            pod("Input", "Input interpretation", text(format!("{w} (English word) | pronunciation"))),
            primary(pod("Result", "Pronunciation", text(rows[0].1.clone()))),
            pod("Table", "Details", table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())),
            src,
        ]));
    }
    if let Some(m) = re!(r"^(?:how many )?syllables (?:in|of|does) (?:the word )?([a-z']+)(?: have)?$|^(?:number of )?syllables (?:in|of) (?:the word )?([a-z']+)$|^([a-z']+) syllables$").captures(l) {
        let w = m.get(1).or(m.get(2)).or(m.get(3))?.as_str();
        let rows = pron_rows(w)?;
        return Some(Answer::new(format!("syllables in {w}"), vec![
            pod("Input", "Input interpretation", text(format!("{w} (English word) | syllables"))),
            primary(pod("Result", "Syllables", text(rows[2].1.clone()))),
            pod("Table", "Pronunciation", table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())),
            src,
        ]));
    }
    if let Some(m) = re!(r"^(?:(?:words )?(?:that )?rhym(?:e|es|ing) (?:with|for) |what rhymes with |rhymes (?:of|to) |rhyme )(?:the word )?([a-z']+)$|^([a-z']+) rhymes$").captures(l) {
        let w = m.get(1).or(m.get(2))?.as_str();
        cmudict().get(w)?;
        let r = rhymes_of(w);
        return Some(Answer::new(format!("rhymes with {w}"), vec![
            pod("Input", "Input interpretation", text(format!("{w} | rhymes"))),
            primary(pod("Result", "Rhymes", text(if r.is_empty() { "no perfect rhymes in the dictionary".to_string() } else { word_list(&r) }))),
            pod("Pronunciation", "Pronunciation", text(format!("/{}/", ipa(cmudict()[w][0])))),
            src,
        ]));
    }
    if let Some(m) = re!(r"^(?:words (?:close|similar|near) to|words one letter (?:away )?from|neighbou?rs of the word) ([a-z]+)$").captures(l) {
        let w = &m[1];
        let mut out: Vec<&'static str> = Vec::new();
        let b = w.as_bytes();
        let mut cands: Vec<String> = Vec::new();
        for i in 0..=b.len() {
            for c in b'a'..=b'z' {
                let mut x = b.to_vec();
                x.insert(i, c);
                cands.push(String::from_utf8(x).unwrap());
                if i < b.len() && b[i] != c {
                    let mut y = b.to_vec();
                    y[i] = c;
                    cands.push(String::from_utf8(y).unwrap());
                }
            }
            if i < b.len() {
                let mut x = b.to_vec();
                x.remove(i);
                cands.push(String::from_utf8(x).unwrap());
            }
        }
        for c in cands {
            if let Ok(k) = words().binary_search(&c.as_str()) {
                if !out.contains(&words()[k]) {
                    out.push(words()[k]);
                }
            }
        }
        out.sort_by_key(|x| (!dictionary().contains_key(x), *x));
        if out.is_empty() {
            return None;
        }
        return Some(Answer::new(format!("words close to {w}"), vec![
            pod("Input", "Input interpretation", text(format!("words one letter away from {w}"))),
            primary(pod("Result", "Words", text(word_list(&out)))),
            pod("Note", "Note", text("words formed by adding, removing or changing one letter")),
        ]));
    }
    None
}

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
    let m = re!(r"^(?:define|definition of|definition for|meaning of|the meaning of|what does|(?:the )?word)\s+(?:the word\s+)?([a-z]+)(?:\s+mean)?$|^([a-z]+)\s+(?:definition|meaning)$|^what is the (?:definition|meaning) of\s+(?:the word\s+)?([a-z]+)$").captures(l)?;
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
    if let Some((b, n)) = relations().get(base.as_str()) {
        if !b.is_empty() {
            pods.push(pod("Broader", "Broader terms", text(b.replace(';', "  |  "))));
        }
        if !n.is_empty() {
            pods.push(pod("Narrower", "Narrower terms", text(n.split(';').take(15).collect::<Vec<_>>().join("  |  "))));
        }
    }
    if let Some(rows) = pron_rows(&base) {
        pods.insert(2, pod("Pronunciation", "Pronunciation", table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())));
    }
    pods.push(pod("Scrabble", "Scrabble score", text(format!("{} ({} letters)", letters_score(&base), base.len()))));
    pods.push(pod("Source", "Source", text("WordNet 3.1, Princeton University; pronunciation from the CMU Pronouncing Dictionary")));
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
    let m = re!(r"^(?:anagrams?|word anagrams?)\s+(?:(?:of|for)\s+)?(?:the word\s+)?([a-z]+)$|^([a-z]+)\s+anagrams?$|^(?:unscramble|descramble|rearrange the letters(?: of| in)?)\s+([a-z]+)$").captures(l)?;
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

/// "word subsets recycle" (words spelled from some of the letters, each used
/// at most as often as it appears) and "letter bank tale" (words using every
/// letter of the bank and no others, repeats allowed).
fn subwords(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(word subsets?|subwords?|sub-words?)\s+(?:of\s+)?([a-z]+)|words?\s+(?:that\s+)?(?:can\s+be\s+)?(?:made|formed|spelled)\s+(?:from|with|using)\s+(?:the\s+letters?\s+(?:of|in)\s+)?([a-z]+)|(letter bank)\s+([a-z]+))$").captures(l)?;
    let bank = m.get(4).is_some();
    let w = m.get(2).or(m.get(3)).or(m.get(5))?.as_str();
    if w.len() < 2 || w.len() > 20 {
        return None;
    }
    let count = |x: &str| {
        let mut c = [0u8; 26];
        for b in x.bytes().filter(u8::is_ascii_lowercase) {
            c[(b - b'a') as usize] += 1;
        }
        c
    };
    let have = count(w);
    let mut found: Vec<&str> = words()
        .iter()
        .copied()
        .filter(|x| x.len() >= 3 && x.bytes().all(|b| b.is_ascii_lowercase()))
        .filter(|x| {
            let c = count(x);
            if bank {
                (0..26).all(|i| (c[i] > 0) == (have[i] > 0))
            } else {
                (0..26).all(|i| c[i] <= have[i])
            }
        })
        .collect();
    found.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    let what = if bank { "letter bank" } else { "word subsets" };
    Some(Answer::new(format!("{what} of {w}"), vec![
        pod("Input", "Input interpretation", text(format!("{what} | {w}"))),
        primary(pod("Result", "Result", text(if found.is_empty() { "no words found".to_string() } else { word_list(&found) }))),
        pod("Count", "Number of words", text(found.len().to_string())),
        pod("Note", "Note", text(if bank { "words of 3 or more letters using every letter of the bank and no others; letters may repeat" } else { "words of 3 or more letters using each letter no more often than it appears" })),
    ]))
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

/// American Soundex code of a word ("Wolfram" → "W416").
pub fn soundex(w: &str) -> Option<String> {
    let cs: Vec<char> = w.chars().filter(|c| c.is_ascii_alphabetic()).map(|c| c.to_ascii_lowercase()).collect();
    let first = *cs.first()?;
    let code = |c: char| match c {
        'b' | 'f' | 'p' | 'v' => '1',
        'c' | 'g' | 'j' | 'k' | 'q' | 's' | 'x' | 'z' => '2',
        'd' | 't' => '3',
        'l' => '4',
        'm' | 'n' => '5',
        'r' => '6',
        'h' | 'w' => '-',
        _ => '0',
    };
    let mut out = first.to_ascii_uppercase().to_string();
    let mut last = code(first);
    for &c in &cs[1..] {
        let d = code(c);
        if d == '-' {
            continue;
        }
        if d != '0' && d != last {
            out.push(d);
            if out.len() == 4 {
                break;
            }
        }
        last = d;
    }
    while out.len() < 4 {
        out.push('0');
    }
    Some(out)
}

fn same_soundex(code: &str, except: &str) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = words().iter().copied().filter(|w| w.len() <= 12 && *w != except && w.chars().next().map(|c| c.to_ascii_uppercase()) == code.chars().next() && soundex(w).as_deref() == Some(code)).collect();
    v.sort_by_key(|w| (!dictionary().contains_key(w), w.len(), *w));
    v.truncate(40);
    v
}

fn soundex_query(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^soundex(?: code)?(?: of| for)? ([a-z])(\d{3})$").captures(l) {
        let code = format!("{}{}", m[1].to_uppercase(), &m[2]);
        let ws = same_soundex(&code, "");
        let pods = vec![pod("Input", "Input interpretation", text(format!("words with Soundex code {code}"))), primary(pod("Result", "Words", text(if ws.is_empty() { "none in the word list".into() } else { ws.join(" | ") })))];
        return Some(Answer::new("Soundex", pods));
    }
    if let Some(m) = re!(r"^(?:the )?soundex(?: code| encoding)?(?: of| for)? ([a-z][a-z' -]*)$").captures(l) {
        let w = m[1].trim();
        let code = soundex(w)?;
        let ws = same_soundex(&code, w);
        let mut pods = vec![pod("Input", "Input interpretation", text(format!("Soundex code of \"{w}\""))), primary(pod("Result", "Soundex code", text(code.clone())))];
        if !ws.is_empty() {
            pods.push(pod("Similar", "Words with the same code", text(ws.join(" | "))));
        }
        return Some(Answer::new("Soundex", pods));
    }
    if let Some(m) = re!(r"^(?:words (?:that )?)?sounds? (?:like|similar to) ([a-z]+)$").captures(l) {
        let w = &m[1];
        let code = soundex(w)?;
        let ws = same_soundex(&code, w);
        if ws.is_empty() {
            return None;
        }
        let pods = vec![pod("Input", "Input interpretation", text(format!("words that sound like \"{w}\" (Soundex {code})"))), primary(pod("Result", "Words", text(ws.join(" | "))))];
        return Some(Answer::new("sounds like", pods));
    }
    None
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?').trim();
    soundex_query(l).or_else(|| sounds(l)).or_else(|| related(l)).or_else(|| anagram_query(l)).or_else(|| subwords(l)).or_else(|| define(l)).or_else(|| synonyms(l)).or_else(|| validity(l)).or_else(|| patterns(l))
}
