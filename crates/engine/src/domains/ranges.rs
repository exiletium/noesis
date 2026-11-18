//! Playing ranges of musical instruments and voice types.
//!
//! "clarinet", "What frequencies can a violin sound?", "range of a cello",
//! "lowest note on a double bass", "tenor voice range".

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

/// names (';'-separated, first is the display name) | sounding low | high |
/// family | answers a bare query | notes
const RANGES: &[(&str, &str, &str, &str, bool, &str)] = &[
    ("violin;fiddle", "G3", "A7", "bowed string", true, "strings tuned G3 D4 A4 E5"),
    ("viola", "C3", "E6", "bowed string", true, "strings tuned C3 G3 D4 A4; written in alto clef"),
    ("cello;violoncello", "C2", "A5", "bowed string", true, "strings tuned C2 G2 D3 A3"),
    ("double bass;contrabass;upright bass;string bass", "E1", "G4", "bowed string", true, "strings tuned E1 A1 D2 G2; written an octave higher than it sounds; a C extension reaches C1"),
    ("guitar;acoustic guitar;classical guitar;electric guitar", "E2", "E6", "plucked string", true, "standard tuning E2 A2 D3 G3 B3 E4; written an octave higher than it sounds; the top note depends on the number of frets"),
    ("bass guitar;electric bass", "E1", "G4", "plucked string", true, "4-string tuning E1 A1 D2 G2 (24 frets); a 5-string adds B0"),
    ("ukulele;soprano ukulele", "C4", "A5", "plucked string", true, "re-entrant tuning G4 C4 E4 A4"),
    ("mandolin", "G3", "E6", "plucked string", true, "paired strings tuned like a violin, G3 D4 A4 E5"),
    ("harp;concert harp;pedal harp", "C♭1", "G♯7", "plucked string", true, "47 strings with seven pedals"),
    ("piano;pianoforte;grand piano", "A0", "C8", "keyboard", true, "88 keys, 27.5 Hz to 4,186 Hz"),
    ("flute;concert flute", "C4", "D7", "woodwind", true, "non-transposing; a B foot joint extends the range down to B3"),
    ("piccolo", "D5", "C8", "woodwind", true, "written an octave lower than it sounds"),
    ("oboe", "B♭3", "A6", "woodwind", true, "non-transposing double-reed instrument"),
    ("clarinet;b-flat clarinet;bb clarinet", "D3", "B♭6", "woodwind", true, "B♭ clarinet: written E3–C7, sounding a major second lower"),
    ("bass clarinet", "B♭1", "F5", "woodwind", true, "in B♭ with a low C extension; sounds a major ninth below written"),
    ("bassoon", "B♭1", "E5", "woodwind", true, "non-transposing double-reed instrument"),
    ("contrabassoon", "B♭0", "B♭3", "woodwind", true, "sounds an octave lower than written"),
    ("soprano saxophone;soprano sax", "A♭3", "E♭6", "woodwind", true, "in B♭; sounds a major second lower than written"),
    ("alto saxophone;alto sax", "D♭3", "A♭5", "woodwind", true, "in E♭; written B♭3–F6, sounding a major sixth lower"),
    ("tenor saxophone;tenor sax", "A♭2", "E♭5", "woodwind", true, "in B♭; sounds a major ninth lower than written"),
    ("baritone saxophone;baritone sax;bari sax", "D♭2", "A♭4", "woodwind", true, "in E♭; sounds an octave and a major sixth lower than written"),
    ("trumpet;b-flat trumpet;bb trumpet", "E3", "B♭5", "brass", true, "B♭ trumpet: written F♯3–C6, sounding a major second lower; professionals play higher"),
    ("trombone;tenor trombone", "E2", "F5", "brass", true, "seven slide positions; pedal tones reach below E2"),
    ("french horn;horn in f", "B1", "F5", "brass", true, "in F; sounds a perfect fifth lower than written"),
    ("tuba", "D1", "F4", "brass", true, "lowest common brass instrument"),
    ("euphonium", "B♭1", "B♭4", "brass", true, "tenor-voiced conical brass"),
    ("marimba", "C2", "C7", "percussion", true, "5-octave concert instrument"),
    ("xylophone", "F4", "C8", "percussion", true, "written an octave lower than it sounds"),
    ("glockenspiel", "G5", "C8", "percussion", true, "written two octaves lower than it sounds"),
    ("vibraphone", "F3", "F6", "percussion", true, "3-octave instrument with motor-driven vibrato"),
    ("timpani", "D2", "C4", "percussion", true, "a set of four drums covers about D2–A3"),
    ("harmonica;diatonic harmonica", "C4", "C7", "free reed", true, "10-hole diatonic in C"),
    ("accordion", "F3", "A6", "free reed", true, "41-key piano accordion right hand"),
    ("soprano voice;soprano", "C4", "C6", "voice", false, "highest female voice type"),
    ("mezzo-soprano voice;mezzo-soprano;mezzo soprano", "A3", "A5", "voice", false, "middle female voice type"),
    ("contralto voice;contralto;alto voice;alto", "F3", "F5", "voice", false, "lowest female voice type"),
    ("tenor voice;tenor", "C3", "C5", "voice", false, "highest common male voice type"),
    ("baritone voice;baritone", "A2", "A4", "voice", false, "middle male voice type"),
    ("bass voice;bass singer", "E2", "E4", "voice", false, "lowest male voice type"),
];

/// MIDI number of "C♭1", "G♯7", "B♭3", "E2"
fn midi(note: &str) -> i32 {
    let mut ch = note.chars();
    let letter = ch.next().unwrap();
    let rest: String = ch.collect();
    let (acc, oct) = if let Some(r) = rest.strip_prefix('♭') { (-1, r) } else if let Some(r) = rest.strip_prefix('♯') { (1, r) } else { (0, rest.as_str()) };
    let base = match letter { 'C' => 0, 'D' => 2, 'E' => 4, 'F' => 5, 'G' => 7, 'A' => 9, _ => 11 };
    let o: i32 = oct.parse().unwrap_or(4);
    (o + 1) * 12 + base + acc
}

fn freq(note: &str) -> f64 {
    440.0 * 2f64.powf((midi(note) - 69) as f64 / 12.0)
}

fn hz(f: f64) -> String {
    format!("{} Hz", fmt_float(if f < 100.0 { (f * 10.0).round() / 10.0 } else { f.round() }, 6))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let (want, name) = if let Some(m) = re!(r"^(?:what (?:frequencies|notes|pitches) (?:can|does) (?:a |an |the )?(.+?) (?:sound|play|produce|make|have)|(?:the )?(?:(?:playing |pitch |frequency |vocal |note |sounding )?range|frequenc(?:y|ies)|frequency range|tessitura) (?:of|for) (?:a |an |the )?(.+)|(.+?) (?:playing |pitch |frequency |vocal |note |sounding )?range|(?:the )?(lowest|highest) (?:note|pitch|frequency) (?:on|of|for) (?:a |an |the )?(.+)|(.+?) (lowest|highest) (?:note|pitch))$").captures(l) {
        let want = m.get(4).or(m.get(7)).map(|x| x.as_str().to_string());
        let name = m.get(1).or(m.get(2)).or(m.get(3)).or(m.get(5)).or(m.get(6))?.as_str().to_string();
        (want, name)
    } else {
        (None, l.to_string())
    };
    let explicit = name != l;
    let name = name.trim_start_matches("the ").trim().to_string();
    let r = RANGES.iter().find(|r| r.0.split(';').any(|n| n == name))?;
    if !explicit && !r.4 {
        return None;
    }
    let disp = r.0.split(';').next().unwrap();
    let (lo, hi) = (r.1, r.2);
    let (flo, fhi) = (freq(lo), freq(hi));
    let span = (midi(hi) - midi(lo)) as f64 / 12.0;
    let (title, result) = match want.as_deref() {
        Some("lowest") => ("Lowest note", format!("{lo} ({})", hz(flo))),
        Some("highest") => ("Highest note", format!("{hi} ({})", hz(fhi))),
        _ => ("Range", format!("{lo} to {hi} | {} to {}", hz(flo), hz(fhi))),
    };
    Some(Answer::new(format!("{disp} range"), vec![
        pod("Input", "Input interpretation", text(format!("{disp} | {}", if r.3 == "voice" { "vocal range" } else { "playing range" }))),
        primary(pod("Result", title, text(result))),
        pod("Table", "Range", table(Some(vec!["", "note", "frequency", "MIDI"]), vec![
            vec![text("lowest"), text(lo), text(hz(flo)), text(midi(lo).to_string())],
            vec![text("highest"), text(hi), text(hz(fhi)), text(midi(hi).to_string())],
        ])),
        pod("Properties", "Properties", table(None, vec![
            vec![text("family"), text(r.3)],
            vec![text("span"), text(format!("{} semitones ({} octaves)", midi(hi) - midi(lo), fmt_float((span * 10.0).round() / 10.0, 3)))],
            vec![text("notes"), text(r.5)],
        ])),
        pod("Note", "Note", text("sounding (concert) pitches in scientific pitch notation with A4 = 440 Hz; fundamentals only, overtones reach much higher; professional players and singers often exceed these ranges")),
    ]))
}
