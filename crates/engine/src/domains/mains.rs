//! Mains electricity by country: voltage, frequency and plug types.
//!
//! "mains voltage in Bhutan", "plug type Japan", "electricity in France",
//! "what voltage does Brazil use".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names | voltage | frequency | plug types
const MAINS: &[(&[&str], &str, &str, &str)] = &[
    (&["united states", "us", "usa", "america"], "120 V", "60 Hz", "A, B"),
    (&["canada"], "120 V", "60 Hz", "A, B"),
    (&["mexico"], "127 V", "60 Hz", "A, B"),
    (&["united kingdom", "uk", "britain", "great britain", "england", "scotland", "wales"], "230 V", "50 Hz", "G"),
    (&["ireland"], "230 V", "50 Hz", "G"),
    (&["france"], "230 V", "50 Hz", "C, E"),
    (&["germany"], "230 V", "50 Hz", "C, F"),
    (&["spain"], "230 V", "50 Hz", "C, F"),
    (&["portugal"], "230 V", "50 Hz", "C, F"),
    (&["italy"], "230 V", "50 Hz", "C, F, L"),
    (&["switzerland"], "230 V", "50 Hz", "C, J"),
    (&["netherlands", "holland"], "230 V", "50 Hz", "C, F"),
    (&["belgium"], "230 V", "50 Hz", "C, E"),
    (&["austria"], "230 V", "50 Hz", "C, F"),
    (&["denmark"], "230 V", "50 Hz", "C, E, F, K"),
    (&["sweden"], "230 V", "50 Hz", "C, F"),
    (&["norway"], "230 V", "50 Hz", "C, F"),
    (&["finland"], "230 V", "50 Hz", "C, F"),
    (&["poland"], "230 V", "50 Hz", "C, E"),
    (&["czech republic", "czechia"], "230 V", "50 Hz", "C, E"),
    (&["greece"], "230 V", "50 Hz", "C, F"),
    (&["turkey", "türkiye"], "230 V", "50 Hz", "C, F"),
    (&["russia"], "230 V", "50 Hz", "C, F"),
    (&["ukraine"], "230 V", "50 Hz", "C, F"),
    (&["israel"], "230 V", "50 Hz", "C, H, M"),
    (&["iran"], "230 V", "50 Hz", "C, F"),
    (&["saudi arabia"], "230 V", "60 Hz", "G"),
    (&["united arab emirates", "uae"], "230 V", "50 Hz", "G"),
    (&["egypt"], "220 V", "50 Hz", "C, F"),
    (&["morocco"], "220 V", "50 Hz", "C, E"),
    (&["nigeria"], "230 V", "50 Hz", "D, G"),
    (&["kenya"], "240 V", "50 Hz", "G"),
    (&["south africa"], "230 V", "50 Hz", "C, D, M, N"),
    (&["india"], "230 V", "50 Hz", "C, D, M"),
    (&["pakistan"], "230 V", "50 Hz", "C, D"),
    (&["bangladesh"], "220 V", "50 Hz", "C, D, G, K"),
    (&["sri lanka"], "230 V", "50 Hz", "D, G, M"),
    (&["nepal"], "230 V", "50 Hz", "C, D, M"),
    (&["bhutan"], "230 V", "50 Hz", "C, D, F, G, M"),
    (&["china"], "220 V", "50 Hz", "A, C, I"),
    (&["hong kong"], "220 V", "50 Hz", "G"),
    (&["taiwan"], "110 V", "60 Hz", "A, B"),
    (&["japan"], "100 V", "50 Hz (east) / 60 Hz (west)", "A, B"),
    (&["south korea", "korea"], "220 V", "60 Hz", "C, F"),
    (&["philippines"], "220 V", "60 Hz", "A, B, C"),
    (&["thailand"], "220 V", "50 Hz", "A, B, C, F, O"),
    (&["vietnam"], "220 V", "50 Hz", "A, C, F"),
    (&["indonesia"], "230 V", "50 Hz", "C, F"),
    (&["malaysia"], "240 V", "50 Hz", "G"),
    (&["singapore"], "230 V", "50 Hz", "G"),
    (&["australia"], "230 V", "50 Hz", "I"),
    (&["new zealand"], "230 V", "50 Hz", "I"),
    (&["brazil"], "127 V / 220 V (varies by state)", "60 Hz", "C, N"),
    (&["argentina"], "220 V", "50 Hz", "C, I"),
    (&["chile"], "220 V", "50 Hz", "C, L"),
    (&["colombia"], "120 V", "60 Hz", "A, B"),
    (&["peru"], "220 V", "60 Hz", "A, C"),
    (&["venezuela"], "120 V", "60 Hz", "A, B"),
    (&["cuba"], "110 V / 220 V", "60 Hz", "A, B, C, L"),
    (&["jamaica"], "110 V", "50 Hz", "A, B"),
];

const PLUGS: &[(&str, &str)] = &[
    ("A", "two flat parallel pins (North America, Japan)"),
    ("B", "two flat pins plus a round earth pin (North America)"),
    ("C", "two round pins, 4 mm (Europlug)"),
    ("D", "three large round pins in a triangle (old British, India)"),
    ("E", "two round pins with a socket earth pin (France, Belgium)"),
    ("F", "two round pins with side earth clips (Schuko)"),
    ("G", "three rectangular pins (British BS 1363)"),
    ("H", "three pins in a V (Israel)"),
    ("I", "two flat pins in a V plus earth (Australia, China)"),
    ("J", "three round pins (Switzerland)"),
    ("K", "two round pins plus a half-round earth (Denmark)"),
    ("L", "three round pins in a row (Italy)"),
    ("M", "three large round pins (South Africa, 15 A)"),
    ("N", "three round pins (Brazil, South Africa)"),
    ("O", "three round pins (Thailand)"),
];

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let m = re!(r"^(?:what )?(?:(mains|household|wall|outlet|electrical|electricity|power) )?(voltage|frequency|electricity|plugs?|plug types?|power plugs?|sockets?|outlets?|mains)(?: (?:is used|are used|does .+ use))? (?:in|of|for) (?:the )?(.+?)$|^(?:the )?(.+?) (mains voltage|voltage|electricity|plug types?|plugs?|power plugs?|sockets?|outlets?|mains frequency)$|^what (voltage|plugs?|plug type) does (?:the )?(.+?) use$|^(plug types?|plugs?|mains voltage|power plugs?) ([a-z ]+)$").captures(&l)?;
    let (what, place) = if let Some(p) = m.get(3) {
        let w = m.get(2)?.as_str();
        // plain "voltage in" or "frequency in" needs a mains word or it is a physics question
        if matches!(w, "voltage" | "frequency") && m.get(1).is_none() && !MAINS.iter().any(|c| c.0.contains(&p.as_str().trim())) {
            return None;
        }
        (w, p.as_str())
    } else if let Some(p) = m.get(4) {
        (m.get(5)?.as_str(), p.as_str())
    } else if let Some(p) = m.get(7) {
        (m.get(6)?.as_str(), p.as_str())
    } else {
        (m.get(8)?.as_str(), m.get(9)?.as_str())
    };
    let place = place.trim();
    let c = MAINS.iter().find(|c| c.0.contains(&place))?;
    let name = {
        let n = c.0[0];
        if n.len() <= 3 { n.to_uppercase() } else { n.split(' ').map(|w| { let mut ch = w.chars(); ch.next().map(|f| f.to_uppercase().collect::<String>() + ch.as_str()).unwrap_or_default() }).collect::<Vec<_>>().join(" ") }
    };
    let (title, result) = if what.starts_with("plug") || what.starts_with("socket") || what.starts_with("outlet") || what.starts_with("power plug") {
        ("Plug types", format!("types {}", c.3))
    } else if what.contains("frequency") {
        ("Mains frequency", c.2.to_string())
    } else {
        ("Mains electricity", format!("{}, {}", c.1, c.2))
    };
    let plug_rows: Vec<Vec<Content>> = c.3.split(", ").filter_map(|p| PLUGS.iter().find(|x| x.0 == p)).map(|(t, d)| vec![text(format!("type {t}")), text(*d)]).collect();
    Some(Answer::new(format!("{name} mains electricity"), vec![
        pod("Input", "Input interpretation", text(format!("{name} | mains electricity"))),
        primary(pod("Result", title, text(result))),
        pod("Table", "Mains electricity", table(None, vec![
            vec![text("voltage"), text(c.1)],
            vec![text("frequency"), text(c.2)],
            vec![text("plug types"), text(c.3)],
        ])),
        pod("Plugs", "Plug types", table(None, plug_rows)),
    ]))
}
