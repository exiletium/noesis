//! The seven continents: area, population, number of countries and the
//! highest and lowest points.
//!
//! "Africa", "area of Asia", "highest point in South America",
//! "highest elevation of South America vs North America", "Europe vs Asia".

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

struct Continent {
    names: &'static [&'static str],
    name: &'static str,
    /// km²
    area: f64,
    population: &'static str,
    countries: u32,
    highest: (&'static str, f64),
    lowest: (&'static str, f64),
}

const CONTINENTS: &[Continent] = &[
    Continent { names: &["asia"], name: "Asia", area: 44_579_000.0, population: "about 4.8 billion (2024)", countries: 48, highest: ("Mount Everest (Nepal–China)", 8849.0), lowest: ("Dead Sea shore (Israel–Jordan–West Bank)", -430.0) },
    Continent { names: &["africa"], name: "Africa", area: 30_370_000.0, population: "about 1.5 billion (2024)", countries: 54, highest: ("Kilimanjaro (Tanzania)", 5895.0), lowest: ("Lake Assal (Djibouti)", -155.0) },
    Continent { names: &["north america"], name: "North America", area: 24_709_000.0, population: "about 610 million (2024, including Central America and the Caribbean)", countries: 23, highest: ("Denali (Alaska, United States)", 6190.0), lowest: ("Badwater Basin, Death Valley (California, United States)", -86.0) },
    Continent { names: &["south america"], name: "South America", area: 17_840_000.0, population: "about 440 million (2024)", countries: 12, highest: ("Aconcagua (Argentina)", 6961.0), lowest: ("Laguna del Carbón (Argentina)", -105.0) },
    Continent { names: &["antarctica"], name: "Antarctica", area: 14_200_000.0, population: "no permanent population (about 1,000–5,000 research staff)", countries: 0, highest: ("Vinson Massif", 4892.0), lowest: ("Deep Lake, Vestfold Hills (exposed surface; the bedrock under the Denman Glacier reaches about −3,500 m)", -50.0) },
    Continent { names: &["europe"], name: "Europe", area: 10_180_000.0, population: "about 745 million (2024)", countries: 44, highest: ("Mount Elbrus (Russia)", 5642.0), lowest: ("Caspian Sea shore (Russia–Kazakhstan)", -28.0) },
    Continent { names: &["oceania", "australia and oceania", "australasia"], name: "Oceania", area: 8_526_000.0, population: "about 46 million (2024)", countries: 14, highest: ("Puncak Jaya (New Guinea, Indonesia); on the Australian mainland Mount Kosciuszko, 2,228 m", 4884.0), lowest: ("Kati Thanda–Lake Eyre (Australia)", -15.0) },
];

fn find(s: &str) -> Option<&'static Continent> {
    let s = s.trim().trim_start_matches("the ").trim_start_matches("continent of ").trim_end_matches(" continent").trim();
    CONTINENTS.iter().find(|c| c.names.contains(&s))
}

fn m_ft(m: f64) -> String {
    format!("{} m ({} ft)", fmt_float(m, 7), fmt_float((m / 0.3048).round(), 7))
}

fn group(v: f64) -> String {
    super::geo::fmt_int(v.round() as u64)
}

fn rows(c: &Continent) -> Vec<(&'static str, String)> {
    vec![
        ("area", format!("{} km² ({} mi²)", group(c.area), group(c.area / 2.589_988))),
        ("share of Earth's land", format!("{}%", fmt_float((c.area / 148_940_000.0 * 1000.0).round() / 10.0, 4))),
        ("population", c.population.to_string()),
        ("countries", if c.countries == 0 { "none (governed under the Antarctic Treaty)".to_string() } else { c.countries.to_string() }),
        ("highest point", format!("{}, {}", c.highest.0, m_ft(c.highest.1))),
        ("lowest point", format!("{}, {}", c.lowest.0, m_ft(c.lowest.1))),
    ]
}

fn prop(p: &str) -> Option<&'static str> {
    Some(match p.trim() {
        "area" | "size" | "land area" | "total area" => "area",
        "population" | "people" => "population",
        "countries" | "number of countries" | "how many countries" => "countries",
        "highest point" | "highest elevation" | "highest mountain" | "tallest mountain" | "highest peak" | "elevation" | "maximum elevation" => "highest point",
        "lowest point" | "lowest elevation" | "minimum elevation" => "lowest point",
        _ => return None,
    })
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.trim_start_matches("what is the ").trim_start_matches("what's the ").trim_start_matches("the ");
    // "P of/in A (vs B)", "A P", "A vs B"
    let (p, body) = if let Some(m) = re!(r"^([a-z ]+?) (?:of|in|on) (.+)$").captures(l) {
        match prop(&m[1]) {
            Some(p) => (Some(p), m[2].to_string()),
            None => (None, l.to_string()),
        }
    } else if let Some(m) = re!(r"^(.+?) (area|population|highest point|highest elevation|highest mountain|lowest point|countries)$").captures(l) {
        (prop(&m[2]), m[1].to_string())
    } else {
        (None, l.to_string())
    };
    let names: Vec<&str> = re!(r"\s*(?:,|\bvs\.?|\bversus\b|\band\b|\bcompared (?:to|with)\b)\s*").split(&body).filter(|s| !s.trim().is_empty()).collect();
    let cs: Option<Vec<&'static Continent>> = names.iter().map(|s| find(s)).collect();
    let cs = cs?;
    if cs.is_empty() {
        return None;
    }
    let src = pod("Source", "Notes", text("areas and populations are rounded; continent boundaries and country counts follow common conventions (UN member states and observers)"));
    if cs.len() == 1 {
        let c = cs[0];
        let r = rows(c);
        let (title, value) = match p {
            Some(p) => {
                let (k, v) = r.iter().find(|(k, _)| *k == p)?;
                (k.to_string(), v.clone())
            }
            None => ("Basic information".to_string(), format!("continent | area {} | population {}", r[0].1, c.population)),
        };
        let title = title[..1].to_uppercase() + &title[1..];
        return Some(Answer::new(c.name, vec![
            pod("Input", "Input interpretation", text(format!("{}{}", c.name, p.map(|p| format!(" | {p}")).unwrap_or_default()))),
            primary(pod("Result", &title, text(value))),
            pod("Table", "Properties", table(None, r.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())),
            src,
        ]));
    }
    let keys: Vec<&str> = match p {
        Some(p) => vec![p],
        None => vec!["area", "population", "countries", "highest point", "lowest point"],
    };
    let mut header = vec!["continent"];
    header.extend(keys.iter().copied());
    let table_rows: Vec<Vec<Content>> = cs
        .iter()
        .map(|c| {
            let r = rows(c);
            let mut row = vec![text(c.name)];
            for k in &keys {
                row.push(text(r.iter().find(|(x, _)| x == k).map(|x| x.1.clone()).unwrap_or_default()));
            }
            row
        })
        .collect();
    let short = match p {
        Some(p) => cs.iter().map(|c| format!("{}: {}", c.name, rows(c).into_iter().find(|(k, _)| *k == p).map(|x| x.1).unwrap_or_default())).collect::<Vec<_>>().join(" | "),
        None => cs.iter().map(|c| c.name).collect::<Vec<_>>().join(" | "),
    };
    Some(Answer::new("continents", vec![
        pod("Input", "Input interpretation", text(format!("{}{}", cs.iter().map(|c| c.name).collect::<Vec<_>>().join(" | "), p.map(|p| format!(" | {p}")).unwrap_or_default()))),
        primary(pod("Result", "Comparison", text(short))),
        pod("Table", "Properties", table(Some(header), table_rows)),
        src,
    ]))
}
