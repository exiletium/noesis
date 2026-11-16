//! Music theory: notes and their frequencies (twelve-tone equal temperament,
//! A4 = 440 Hz), chords, scales and modes, key signatures and intervals.

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

const SHARP_NAMES: [&str; 12] = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"];
const FLAT_NAMES: [&str; 12] = ["C", "D♭", "D", "E♭", "E", "F", "G♭", "G", "A♭", "A", "B♭", "B"];

/// Pitch class (0 = C) and whether the spelling used flats.
fn pitch(s: &str) -> Option<(i32, bool)> {
    let s = s.trim();
    let mut c = s.chars();
    let letter = c.next()?.to_ascii_uppercase();
    let base = match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let rest = c.as_str().to_lowercase();
    let rest = rest.trim();
    let (acc, flat) = match rest {
        "" | "natural" | "♮" => (0, false),
        "#" | "♯" | "sharp" | " sharp" | "s" => (1, false),
        "##" | "x" | "double sharp" => (2, false),
        "b" | "♭" | "flat" | " flat" => (-1, true),
        "bb" | "double flat" => (-2, true),
        _ => return None,
    };
    Some(((base + acc as i32).rem_euclid(12), flat))
}

fn name(pc: i32, flats: bool) -> &'static str {
    let k = pc.rem_euclid(12) as usize;
    if flats {
        FLAT_NAMES[k]
    } else {
        SHARP_NAMES[k]
    }
}

fn freq(pc: i32, octave: i32) -> f64 {
    let midi = 12 * (octave + 1) + pc;
    440.0 * 2f64.powf((midi as f64 - 69.0) / 12.0)
}


const CHORDS: &[(&[&str], &str, &[i32])] = &[
    (&["", "major", "maj", "M"], "major triad", &[0, 4, 7]),
    (&["minor", "min", "m", "-"], "minor triad", &[0, 3, 7]),
    (&["diminished", "dim", "°", "o"], "diminished triad", &[0, 3, 6]),
    (&["augmented", "aug", "+"], "augmented triad", &[0, 4, 8]),
    (&["7", "dominant 7th", "dominant seventh", "dom7", "seventh", "7th"], "dominant seventh", &[0, 4, 7, 10]),
    (&["maj7", "major 7th", "major seventh", "M7", "Δ7", "major 7"], "major seventh", &[0, 4, 7, 11]),
    (&["m7", "min7", "minor 7th", "minor seventh", "-7", "minor 7"], "minor seventh", &[0, 3, 7, 10]),
    (&["dim7", "diminished 7th", "diminished seventh", "°7"], "diminished seventh", &[0, 3, 6, 9]),
    (&["m7b5", "half diminished", "half-diminished", "ø", "ø7", "min7b5"], "half-diminished seventh", &[0, 3, 6, 10]),
    (&["mmaj7", "minor major 7th", "minmaj7", "m(maj7)"], "minor-major seventh", &[0, 3, 7, 11]),
    (&["6", "major 6th", "maj6", "sixth", "6th"], "major sixth", &[0, 4, 7, 9]),
    (&["m6", "min6", "minor 6th", "minor sixth"], "minor sixth", &[0, 3, 7, 9]),
    (&["9", "dominant 9th", "9th", "ninth"], "dominant ninth", &[0, 4, 7, 10, 14]),
    (&["maj9", "major 9th", "major ninth"], "major ninth", &[0, 4, 7, 11, 14]),
    (&["m9", "min9", "minor 9th", "minor ninth"], "minor ninth", &[0, 3, 7, 10, 14]),
    (&["sus2", "suspended 2nd", "suspended second"], "suspended second", &[0, 2, 7]),
    (&["sus4", "sus", "suspended 4th", "suspended fourth"], "suspended fourth", &[0, 5, 7]),
    (&["add9", "add 9"], "added ninth", &[0, 4, 7, 14]),
    (&["5", "power"], "power chord", &[0, 7]),
];

const SCALES: &[(&[&str], &str, &[i32])] = &[
    (&["major", "ionian", ""], "major (Ionian)", &[0, 2, 4, 5, 7, 9, 11]),
    (&["minor", "natural minor", "aeolian"], "natural minor (Aeolian)", &[0, 2, 3, 5, 7, 8, 10]),
    (&["harmonic minor"], "harmonic minor", &[0, 2, 3, 5, 7, 8, 11]),
    (&["melodic minor"], "melodic minor (ascending)", &[0, 2, 3, 5, 7, 9, 11]),
    (&["dorian"], "Dorian", &[0, 2, 3, 5, 7, 9, 10]),
    (&["phrygian"], "Phrygian", &[0, 1, 3, 5, 7, 8, 10]),
    (&["lydian"], "Lydian", &[0, 2, 4, 6, 7, 9, 11]),
    (&["mixolydian"], "Mixolydian", &[0, 2, 4, 5, 7, 9, 10]),
    (&["locrian"], "Locrian", &[0, 1, 3, 5, 6, 8, 10]),
    (&["major pentatonic", "pentatonic"], "major pentatonic", &[0, 2, 4, 7, 9]),
    (&["minor pentatonic"], "minor pentatonic", &[0, 3, 5, 7, 10]),
    (&["blues", "minor blues"], "blues", &[0, 3, 5, 6, 7, 10]),
    (&["major blues"], "major blues", &[0, 2, 3, 4, 7, 9]),
    (&["chromatic"], "chromatic", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
    (&["whole tone", "whole-tone"], "whole tone", &[0, 2, 4, 6, 8, 10]),
    (&["phrygian dominant", "spanish", "blues phrygian"], "Phrygian dominant", &[0, 1, 4, 5, 7, 8, 10]),
];

const INTERVALS: [&str; 13] = ["unison", "minor second", "major second", "minor third", "major third", "perfect fourth", "tritone", "perfect fifth", "minor sixth", "major sixth", "minor seventh", "major seventh", "octave"];

/// Major-key signatures by tonic pitch class: positive = sharps, negative = flats.
fn major_key_accidentals(pc: i32, flat_spelling: bool) -> i32 {
    // circle of fifths position
    let k = (pc * 7).rem_euclid(12);
    let sharps = if k > 6 || (k == 6 && flat_spelling) { k - 12 } else { k };
    if flat_spelling && sharps > 0 && pc != 0 {
        sharps - 12
    } else {
        sharps
    }
}

/// Spell a scale on letter names, one letter per degree for seven-note scales.
fn spell(root: i32, flats: bool, steps: &[i32]) -> Vec<String> {
    if steps.len() == 7 {
        let letters = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
        let natural = [0, 2, 4, 5, 7, 9, 11];
        let root_name = name(root, flats);
        let li = letters.iter().position(|c| root_name.starts_with(*c)).unwrap_or(0);
        return steps
            .iter()
            .enumerate()
            .map(|(k, s)| {
                let letter_idx = (li + k) % 7;
                let target = (root + s).rem_euclid(12);
                let mut diff = (target - natural[letter_idx]).rem_euclid(12);
                if diff > 6 {
                    diff -= 12;
                }
                let acc = match diff {
                    0 => "",
                    1 => "♯",
                    2 => "𝄪",
                    -1 => "♭",
                    -2 => "𝄫",
                    _ => "?",
                };
                format!("{}{acc}", letters[letter_idx])
            })
            .collect();
    }
    steps.iter().map(|s| name(root + s, flats).to_string()).collect()
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    key_signature(l).or_else(|| scale(l)).or_else(|| chord(q)).or_else(|| interval(l)).or_else(|| note(q))
}

fn note(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    let body = l.trim_start_matches("note ").trim_start_matches("the note ").trim_start_matches("musical note ").trim_start_matches("frequency of ").trim_start_matches("frequency of the note ").trim();
    let m = re!(r"^([a-g])\s*(#|♯|b|♭|sharp|flat)?\s*(-?\d)?(?:\s+note)?$").captures(body)?;
    // a bare letter is too ambiguous unless the question said "note"
    if m.get(2).is_none() && m.get(3).is_none() && !l.contains("note") {
        return None;
    }
    // "B" alone, "Bb", "F#" are fine; "b" lower-case alone only with "note"
    let (pc, flats) = pitch(&format!("{}{}", &m[1], m.get(2).map_or("", |x| x.as_str())))?;
    let octave: i32 = m.get(3).and_then(|x| x.as_str().parse().ok()).unwrap_or(4);
    let f = freq(pc, octave);
    let midi = 12 * (octave + 1) + pc;
    let enh = if name(pc, true) != name(pc, false) { format!("  (enharmonic: {})", name(pc, !flats)) } else { String::new() };
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("{}{octave}{enh}", name(pc, flats)))),
        primary(pod("Result", "Frequency", text(format!("{} Hz", fmt_float(f, 6))))),
        pod(
            "Info",
            "Properties",
            table(
                None,
                vec![
                    vec![text("MIDI note number"), text(midi.to_string())],
                    vec![text("wavelength in air (343 m/s)"), text(format!("{} m", fmt_float(343.0 / f, 5)))],
                    vec![text("period"), text(format!("{} ms", fmt_float(1000.0 / f, 5)))],
                    vec![text("octave neighbors"), text(format!("{} Hz, {} Hz", fmt_float(f / 2.0, 6), fmt_float(f * 2.0, 6)))],
                ],
            ),
        ),
        pod("Tuning", "Tuning", text("twelve-tone equal temperament, A4 = 440 Hz")),
    ];
    Some(Answer::new(format!("note {}{octave}", name(pc, flats)), pods))
}

fn chord(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    let raw = q.clean.trim();
    let body_l = l.trim_start_matches("chord ").trim_start_matches("the ").trim_end_matches(" chord").trim();
    let m = re!(r"^([a-g])\s*(#|♯|b|♭|sharp|flat)?\s*(.*?)$").captures(body_l)?;
    let qual_l = m[3].trim();
    // keep the case of the quality: "M7" (major) vs "m7" (minor)
    let qual_raw = raw.to_lowercase().rfind(qual_l).filter(|_| !qual_l.is_empty()).map(|p| raw[p..p + qual_l.len()].to_string()).unwrap_or_default();
    // without the word "chord", only unmistakable chord names: "Dbmin6", "Cmaj7"
    if !l.contains("chord") && (qual_l.len() < 2 || qual_l.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    let (names, title, steps) = CHORDS.iter().find(|(names, ..)| names.iter().any(|n| *n == qual_raw.as_str())).or_else(|| CHORDS.iter().find(|(names, ..)| names.iter().any(|n| n.to_lowercase() == qual_l && n.len() > 1)))?;
    let _ = names;
    let (root, flats) = pitch(&format!("{}{}", &m[1], m.get(2).map_or("", |x| x.as_str())))?;
    let notes: Vec<String> = steps.iter().map(|s| name(root + s, flats || (*s == 3 || *s == 10 || *s == 6 || *s == 8) && !matches!(root, 4 | 11 | 9 | 7 | 2 | 6 | 1)).to_string()).collect();
    let ivals: Vec<String> = steps.iter().map(|s| INTERVALS.get(*s as usize % 12).map(|x| if *s >= 12 { format!("{x} + octave") } else { x.to_string() }).unwrap_or_default()).collect();
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("{} {title} chord", name(root, flats)))),
        primary(pod("Result", "Notes", text(notes.join(", ")))),
        pod("Intervals", "Intervals from the root", text(ivals.join(", "))),
        pod("Frequencies", "Frequencies (root in octave 4)", text(steps.iter().map(|s| format!("{} Hz", fmt_float(freq(root, 4) * 2f64.powf(*s as f64 / 12.0), 5))).collect::<Vec<_>>().join(", "))),
    ];
    Some(Answer::new(format!("{} {title}", name(root, flats)), pods))
}

fn scale(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?([a-g])\s*(#|♯|b|♭|sharp|flat)?\s+(.+?)\s+scale$|^(?:the\s+)?([a-g])\s*(#|♯|b|♭|sharp|flat)?\s+(dorian|phrygian|lydian|mixolydian|locrian|aeolian|ionian|blues phrygian|blues|pentatonic|harmonic minor|melodic minor|whole tone|chromatic)(?:\s+mode)?$|^(?:the\s+)?([a-g])\s*(#|♯|b|♭|sharp|flat)?\s+scale$").captures(l)?;
    let (letter, acc, kind) = if m.get(1).is_some() {
        (m[1].to_string(), m.get(2).map_or("", |x| x.as_str()).to_string(), m[3].to_string())
    } else if m.get(4).is_some() {
        (m[4].to_string(), m.get(5).map_or("", |x| x.as_str()).to_string(), m[6].to_string())
    } else {
        (m[7].to_string(), m.get(8).map_or("", |x| x.as_str()).to_string(), "major".to_string())
    };
    let (_, title, steps) = SCALES.iter().find(|(names, ..)| names.iter().any(|n| *n == kind.trim()))?;
    let (root, flats) = pitch(&format!("{letter}{acc}"))?;
    // minor-flavoured scales on C, D, F, G use flats
    let flats = flats || (steps.contains(&3) && matches!(root, 0 | 2 | 5 | 7 | 10 | 3 | 8));
    let notes = spell(root, flats, steps);
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("{} {title} scale", name(root, flats)))),
        primary(pod("Result", "Notes", text(notes.join(", ")))),
        pod("Steps", "Interval pattern (semitones)", text(steps.windows(2).map(|w| (w[1] - w[0]).to_string()).chain(std::iter::once((12 - steps.last()? + steps[0]).to_string())).collect::<Vec<_>>().join("-"))),
    ];
    Some(Answer::new(format!("{} {title} scale", name(root, flats)), pods))
}

fn key_signature(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(?:how many\s+)?(?:sharps|flats|sharps or flats|accidentals)\s+(?:are\s+)?in\s+(?:the\s+)?(?:key\s+of\s+)?|key signature (?:of|for)\s+|key of\s+)([a-g])\s*(#|♯|b|♭|sharp|flat)?\s*(major|minor)?(?:\s+key)?$").captures(l)?;
    let (pc, flats) = pitch(&format!("{}{}", &m[1], m.get(2).map_or("", |x| x.as_str())))?;
    let minor = m.get(3).is_some_and(|x| x.as_str() == "minor");
    let major_pc = if minor { pc + 3 } else { pc };
    let k = major_key_accidentals(major_pc.rem_euclid(12), flats || (minor && matches!(pc, 0 | 2 | 5 | 7 | 10 | 3 | 8)));
    const SHARP_ORDER: [&str; 7] = ["F♯", "C♯", "G♯", "D♯", "A♯", "E♯", "B♯"];
    const FLAT_ORDER: [&str; 7] = ["B♭", "E♭", "A♭", "D♭", "G♭", "C♭", "F♭"];
    let (count, list) = if k >= 0 { (format!("{k} sharp{}", if k == 1 { "" } else { "s" }), SHARP_ORDER[..k as usize].join(", ")) } else { (format!("{} flat{}", -k, if k == -1 { "" } else { "s" }), FLAT_ORDER[..(-k) as usize].join(", ")) };
    let relative = if minor { format!("{} major", name(major_pc, k < 0)) } else { format!("{} minor", name(pc - 3, k < 0)) };
    let key_name = format!("{} {}", name(pc, flats || k < 0), if minor { "minor" } else { "major" });
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("key signature of {key_name}"))),
        primary(pod("Result", "Key signature", text(if list.is_empty() { "no sharps or flats".to_string() } else { format!("{count}: {list}") }))),
        pod("Relative", "Relative key", text(relative)),
        pod("Scale", "Scale", text(spell(pc, k < 0 || flats, if minor { &[0, 2, 3, 5, 7, 8, 10] } else { &[0, 2, 4, 5, 7, 9, 11] }).join(", "))),
    ];
    Some(Answer::new(format!("key of {key_name}"), pods))
}

fn interval(l: &str) -> Option<Answer> {
    let body = l.trim_start_matches("intervals ").trim_start_matches("interval ").trim_start_matches("between ").trim();
    if !l.starts_with("interval") {
        return None;
    }
    // tokens "c4", "g", "bb3", "f#"; a note without an octave lies above the previous one
    let mut notes: Vec<(i32, bool, i32)> = Vec::new();
    for tok in body.split(|c: char| c.is_whitespace() || c == ',').filter(|t| !t.is_empty() && !matches!(*t, "to" | "and" | "-" | "from")) {
        let m = re!(r"^([a-g])(#|♯|b|♭)?(-?\d)?$").captures(tok)?;
        let (p, f) = pitch(&format!("{}{}", &m[1], m.get(2).map_or("", |x| x.as_str())))?;
        let oct = match m.get(3) {
            Some(o) => o.as_str().parse().ok()?,
            None => match notes.last() {
                Some(&(lp, _, lo)) => if p > lp { lo } else { lo + 1 },
                None => 4,
            },
        };
        notes.push((p, f, oct));
    }
    if notes.len() < 2 {
        return None;
    }
    let rows: Vec<Vec<Content>> = notes
        .windows(2)
        .map(|w| {
            let a = 12 * w[0].2 + w[0].0;
            let b = 12 * w[1].2 + w[1].0;
            let st = b - a;
            let nm = INTERVALS.get((st.abs() % 12) as usize).copied().unwrap_or("");
            let oct = st.abs() / 12;
            let label = if st.abs() == 12 { "octave".to_string() } else if oct > 0 { format!("{nm} + {oct} octave{}", if oct > 1 { "s" } else { "" }) } else { nm.to_string() };
            vec![text(format!("{}{} → {}{}", name(w[0].0, w[0].1), w[0].2, name(w[1].0, w[1].1), w[1].2)), text(format!("{st} semitones")), text(label)]
        })
        .collect();
    let first = content_text(&rows[0][2]);
    let pods = vec![pod("Input", "Input interpretation", text(format!("intervals {body}"))), primary(pod("Result", "Intervals", table(Some(vec!["notes", "size", "interval"]), rows))), pod("First", "First interval", text(first))];
    Some(Answer::new("musical intervals", pods))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theory() {
        assert!((freq(0, 5) - 523.251).abs() < 1e-3);
        assert_eq!(spell(9, false, &[0, 2, 3, 5, 7, 8, 10]).join(" "), "A B C D E F G");
        assert_eq!(spell(10, true, &[0, 2, 4, 5, 7, 9, 11]).join(" "), "B♭ C D E♭ F G A");
        assert_eq!(major_key_accidentals(10, true), -2);
        assert_eq!(major_key_accidentals(7, false), 1);
        assert_eq!(major_key_accidentals(6, true), -6);
    }
}
