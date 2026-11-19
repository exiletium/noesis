//! Well-known exoplanets: host star, mass, radius, orbit, and how and when
//! they were found.
//!
//! "Kepler-16b", "55 Cancri d, 55 Cancri e, 55 Cancri f", "how was CoRoT-1b
//! discovered?", "mass Iota Draconis b / Jupiter".

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

struct Planet {
    names: &'static [&'static str],
    host: &'static str,
    /// Jupiter masses (minimum mass m sin i when `min`)
    mass: f64,
    min: bool,
    /// Jupiter radii; 0 when unmeasured
    radius: f64,
    /// days
    period: f64,
    /// AU
    a: f64,
    year: u32,
    method: &'static str,
    /// light years to the system
    ly: f64,
    note: &'static str,
}

const fn p(names: &'static [&'static str], host: &'static str, mass: f64, min: bool, radius: f64, period: f64, a: f64, year: u32, method: &'static str, ly: f64, note: &'static str) -> Planet {
    Planet { names, host, mass, min, radius, period, a, year, method, ly, note }
}

const EARTH_MJ: f64 = 317.83;
const EARTH_RJ: f64 = 11.21;

const PLANETS: &[Planet] = &[
    p(&["51 pegasi b", "51 peg b", "dimidium"], "51 Pegasi (G2IV)", 0.46, true, 0.0, 4.2308, 0.0527, 1995, "radial velocity (Michel Mayor and Didier Queloz, Haute-Provence Observatory)", 50.6, "the first exoplanet found orbiting a Sun-like star; the prototype hot Jupiter"),
    p(&["kepler-16b", "kepler-16 b", "kepler-16(ab) b", "kepler 16b"], "Kepler-16 A and B (a K and an M dwarf)", 0.333, false, 0.754, 228.8, 0.7048, 2011, "transit (Kepler space telescope)", 245.0, "the first confirmed circumbinary planet: it orbits both stars of an eclipsing binary, like Tatooine"),
    p(&["55 cancri b", "55 cnc b", "galileo (planet)"], "55 Cancri A (G8V)", 0.83, true, 0.0, 14.65, 0.1134, 1996, "radial velocity", 41.0, ""),
    p(&["55 cancri c", "55 cnc c"], "55 Cancri A (G8V)", 0.17, true, 0.0, 44.4, 0.237, 2004, "radial velocity", 41.0, ""),
    p(&["55 cancri d", "55 cnc d", "lipperhey"], "55 Cancri A (G8V)", 3.9, true, 0.0, 5200.0, 5.7, 2002, "radial velocity", 41.0, "a Jupiter analogue on a 14-year orbit; period and distance are uncertain by several percent"),
    p(&["55 cancri e", "55 cnc e", "janssen"], "55 Cancri A (G8V)", 0.0251, false, 0.167, 0.7365, 0.01544, 2004, "radial velocity; transits found in 2011", 41.0, "a super-Earth of about 8 Earth masses and 1.9 Earth radii with a dayside hotter than 2,000 K"),
    p(&["55 cancri f", "55 cnc f", "harriot"], "55 Cancri A (G8V)", 0.15, true, 0.0, 260.0, 0.77, 2007, "radial velocity", 41.0, "in the star's habitable zone; a gas giant of about 47 Earth masses"),
    p(&["upsilon andromedae b", "ups and b", "υ andromedae b", "saffar"], "Upsilon Andromedae A (F8V)", 0.69, true, 0.0, 4.617, 0.059, 1996, "radial velocity (Geoffrey Marcy and Paul Butler)", 44.0, "the innermost of the first multi-planet system found around a main-sequence star"),
    p(&["iota draconis b", "iota dra b", "ι draconis b", "hypatia"], "Iota Draconis (K2III giant)", 16.4, false, 0.0, 511.0, 1.45, 2002, "radial velocity (Lick Observatory)", 101.0, "the first planet found around a giant star; the true mass comes from Hipparcos and Gaia astrometry (minimum mass about 9 Jupiter masses)"),
    p(&["corot-1b", "corot-1 b", "corot 1b"], "CoRoT-1 (G0V)", 1.03, false, 1.49, 1.509, 0.0254, 2007, "transit, by the CoRoT space telescope (the mission's first planet), confirmed with radial velocities", 1560.0, "a hot Jupiter whose phase curve was measured from space"),
    p(&["wasp-17b", "wasp-17 b", "ditsö̀"], "WASP-17 (F6V)", 0.486, false, 1.99, 3.735, 0.0515, 2009, "transit (SuperWASP survey)", 1300.0, "one of the largest and least dense planets known; its orbit is retrograde"),
    p(&["hd 209458 b", "osiris"], "HD 209458 (G0V)", 0.69, false, 1.38, 3.525, 0.047, 1999, "radial velocity; the first exoplanet seen to transit its star", 157.0, "the first exoplanet with a detected atmosphere (sodium, 2001)"),
    p(&["hd 189733 b"], "HD 189733 A (K1V)", 1.13, false, 1.13, 2.219, 0.031, 2005, "transit", 64.5, "a deep-blue hot Jupiter, the first exoplanet with a temperature map"),
    p(&["wasp-12b", "wasp-12 b"], "WASP-12 (G0)", 1.47, false, 1.9, 1.091, 0.0234, 2008, "transit (SuperWASP survey)", 1400.0, "being tidally stretched and slowly spiralling into its star"),
    p(&["kelt-9b", "kelt-9 b"], "KELT-9 (B9.5/A0)", 2.88, false, 1.89, 1.481, 0.0346, 2016, "transit (KELT survey)", 670.0, "the hottest known exoplanet, with a dayside above 4,000 K"),
    p(&["proxima centauri b", "proxima b"], "Proxima Centauri (M5.5V)", 0.0034, true, 0.0, 11.19, 0.0485, 2016, "radial velocity (Pale Red Dot campaign)", 4.24, "the nearest exoplanet, in its star's habitable zone; about 1.07 Earth masses minimum"),
    p(&["trappist-1e", "trappist-1 e"], "TRAPPIST-1 (M8V)", 0.00217, false, 0.0820, 6.10, 0.029, 2017, "transit (TRAPPIST and Spitzer)", 40.7, "a rocky planet of about 0.69 Earth masses and 0.92 Earth radii in the habitable zone"),
    p(&["kepler-452b", "kepler-452 b"], "Kepler-452 (G2V)", 0.0, false, 0.145, 384.8, 1.046, 2015, "transit (Kepler space telescope)", 1800.0, "about 1.6 Earth radii on a nearly Earth-like year around a Sun-like star"),
    p(&["kepler-22b", "kepler-22 b"], "Kepler-22 (G5V)", 0.0, false, 0.187, 289.9, 0.849, 2011, "transit (Kepler space telescope)", 640.0, "the first Kepler planet found in a habitable zone; about 2.1 Earth radii"),
    p(&["gliese 581 c", "gj 581 c"], "Gliese 581 (M3V)", 0.0173, true, 0.0, 12.92, 0.072, 2007, "radial velocity (HARPS)", 20.5, "about 5.5 Earth masses minimum"),
];

/// Hot Jupiters: giant planets on orbits shorter than 10 days.
fn hot_jupiter(p: &Planet) -> bool {
    p.period < 10.0 && p.mass >= 0.3
}

fn find(s: &str) -> Option<&'static Planet> {
    let s = s.trim().trim_start_matches("the ").trim_start_matches("exoplanet ").trim_start_matches("planet ").trim();
    PLANETS.iter().find(|p| p.names.contains(&s))
}

fn title(p: &Planet) -> String {
    let n = p.names[0];
    // "kepler-16b" → "Kepler-16b", "55 cancri d" → "55 Cancri d"
    let mut out: Vec<String> = n.split(' ').map(|w| {
        if w.len() == 1 { w.to_string() } else {
            let mut c = w.chars();
            c.next().map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str())
        }
    }).collect();
    for w in out.iter_mut() {
        for (a, b) in [("Corot", "CoRoT"), ("Wasp", "WASP"), ("Kelt", "KELT"), ("Hd", "HD"), ("Trappist", "TRAPPIST")] {
            if w.starts_with(a) {
                *w = w.replacen(a, b, 1);
            }
        }
    }
    out.join(" ")
}

fn mass_text(p: &Planet) -> String {
    if p.mass == 0.0 {
        return "not measured".into();
    }
    let me = p.mass * EARTH_MJ;
    let main = if p.mass < 0.1 { format!("{} Earth masses ({} Jupiter masses)", fmt_float((me * 100.0).round() / 100.0, 3), fmt_float(p.mass, 3)) } else { format!("{} Jupiter masses ({} Earth masses)", fmt_float(p.mass, 3), fmt_float(me.round(), 4)) };
    if p.min { format!("{main}, minimum (m sin i)") } else { main }
}

fn radius_text(p: &Planet) -> String {
    if p.radius == 0.0 {
        return "not measured (does not transit)".into();
    }
    let re = p.radius * EARTH_RJ;
    if p.radius < 0.3 { format!("{} Earth radii ({} Jupiter radii)", fmt_float((re * 100.0).round() / 100.0, 3), fmt_float(p.radius, 3)) } else { format!("{} Jupiter radii ({} Earth radii)", fmt_float(p.radius, 3), fmt_float((re * 10.0).round() / 10.0, 3)) }
}

fn period_text(p: &Planet) -> String {
    if p.period > 700.0 { format!("about {} days ({} years)", fmt_float(p.period.round(), 5), fmt_float((p.period / 365.25 * 10.0).round() / 10.0, 3)) } else { format!("{} days", fmt_float(p.period, 5)) }
}

fn rows(p: &Planet) -> Vec<(&'static str, String)> {
    vec![
        ("host star", p.host.to_string()),
        ("mass", mass_text(p)),
        ("radius", radius_text(p)),
        ("orbital period", period_text(p)),
        ("semimajor axis", format!("{} AU", fmt_float(p.a, 4))),
        ("distance from Earth", format!("about {} light years", fmt_float(p.ly, 4))),
        ("discovered", p.year.to_string()),
        ("discovery method", p.method.to_string()),
    ]
}

fn card(p: &Planet) -> Answer {
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{} (exoplanet)", title(p)))),
        primary(pod("Result", "Properties", table(None, rows(p).into_iter().map(|(k, v)| vec![text(k), text(v)]).collect()))),
    ];
    if !p.note.is_empty() {
        pods.push(pod("Notes", "Notes", text(p.note)));
    }
    Answer::new(format!("{} exoplanet", title(p)), pods)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace('–', "-");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    // "how was CoRoT-1b discovered", "when was 51 Pegasi b discovered"
    if let Some(m) = re!(r"^(how|when|who|by whom) (?:was|were) (.+?) (?:discovered|found|detected)$").captures(&l) {
        let p = find(&m[2])?;
        let (label, val) = if &m[1] == "when" { ("Discovered", p.year.to_string()) } else { ("Discovery method", format!("{} ({})", p.method, p.year)) };
        let mut a = card(p);
        a.pods[1] = primary(pod("Result", label, text(val)));
        a.pods.push(pod("Properties", "Properties", table(None, rows(p).into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())));
        return Some(a);
    }
    // "discovery year of all hot Jupiters"
    if re!(r"^(?:the )?(?:discovery (?:years?|dates?) of |when were )?(?:all |the )?hot jupiters(?: discovered)?$").is_match(&l) {
        let mut v: Vec<&Planet> = PLANETS.iter().filter(|p| hot_jupiter(p)).collect();
        v.sort_by_key(|p| p.year);
        return Some(Answer::new("hot Jupiters", vec![
            pod("Input", "Input interpretation", text("hot Jupiters | discovery year")),
            primary(pod("Result", "Discovery years", text(v.iter().map(|p| format!("{}: {}", title(p), p.year)).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Hot Jupiters", table(Some(vec!["planet", "discovered", "period", "mass"]), v.iter().map(|p| vec![text(title(p)), text(p.year.to_string()), text(period_text(p)), text(mass_text(p))]).collect())),
            pod("Note", "Note", text("giant planets (at least 0.3 Jupiter masses) on orbits shorter than 10 days, among the well-known planets in Noesis's list; thousands of exoplanets are catalogued")),
        ]));
    }
    // "largest exoplanets by radius", "most massive exoplanets"
    if let Some(m) = re!(r"^(?:the )?(largest|biggest|smallest|most massive|least massive|closest|nearest|hottest) (?:known )?exoplanets?(?: by (radius|mass))?$").captures(&l) {
        let by_mass = m[1].contains("massive") || m.get(2).is_some_and(|x| x.as_str() == "mass");
        let near = matches!(&m[1], "closest" | "nearest");
        let small = matches!(&m[1], "smallest" | "least massive");
        let mut v: Vec<&Planet> = PLANETS.iter().filter(|p| near || if by_mass { p.mass > 0.0 } else { p.radius > 0.0 }).collect();
        let key = |p: &&Planet| if near { -p.ly } else if by_mass { p.mass } else { p.radius };
        v.sort_by(|a, b| key(b).partial_cmp(&key(a)).unwrap());
        if small {
            v.reverse();
        }
        v.truncate(8);
        let col = if near { "distance" } else if by_mass { "mass" } else { "radius" };
        return Some(Answer::new("exoplanets", vec![
            pod("Input", "Input interpretation", text(format!("{} exoplanets by {col}", &m[1]))),
            primary(pod("Result", "Result", text(v.iter().map(|p| title(p)).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Ranking", table(Some(vec!["planet", col, "host"]), v.iter().map(|p| vec![text(title(p)), text(if near { format!("{} ly", fmt_float(p.ly, 4)) } else if by_mass { mass_text(p) } else { radius_text(p) }), text(p.host)]).collect())),
            pod("Note", "Note", text("ranked among the well-known planets in Noesis's list, not the full catalogue of known exoplanets")),
        ]));
    }
    // "mass Iota Draconis b / Jupiter", "radius of WASP-17b"
    if let Some(m) = re!(r"^(?:the )?(mass|radius|orbital period|period|semimajor axis|distance|discovery year|host star)(?: of)? (.+?)(?: ?/ ?(jupiter|earth|jupiter's mass|earth's mass|jupiter mass|earth mass))?$").captures(&l) {
        let p = find(&m[2])?;
        let rel = m.get(3).map(|x| x.as_str());
        let (label, val) = match (&m[1], rel) {
            ("mass", Some(r)) if p.mass > 0.0 => ("Mass ratio", if r.starts_with("jupiter") { format!("{} (mass relative to Jupiter){}", fmt_float(p.mass, 4), if p.min { "; minimum mass" } else { "" }) } else { format!("{} (mass relative to Earth)", fmt_float((p.mass * EARTH_MJ * 100.0).round() / 100.0, 4)) }),
            ("radius", Some(r)) if p.radius > 0.0 => ("Radius ratio", if r.starts_with("jupiter") { fmt_float(p.radius, 4) } else { fmt_float((p.radius * EARTH_RJ * 100.0).round() / 100.0, 4) }),
            ("mass", None) => ("Mass", mass_text(p)),
            ("radius", None) => ("Radius", radius_text(p)),
            ("orbital period" | "period", None) => ("Orbital period", period_text(p)),
            ("semimajor axis", None) => ("Semimajor axis", format!("{} AU", fmt_float(p.a, 4))),
            ("distance", None) => ("Distance from Earth", format!("about {} light years", fmt_float(p.ly, 4))),
            ("discovery year", None) => ("Discovered", p.year.to_string()),
            ("host star", None) => ("Host star", p.host.to_string()),
            _ => return None,
        };
        let mut a = card(p);
        a.pods[1] = primary(pod("Result", label, text(val)));
        a.pods.push(pod("Properties", "Properties", table(None, rows(p).into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())));
        return Some(a);
    }
    // one planet or a list: "55 Cancri d, 55 Cancri e, 55 Cancri f"
    let parts: Vec<&str> = re!(r"\s*,\s*(?:and\s+)?|\s+(?:vs\.?|versus|and)\s+").split(&l).filter(|p| !p.is_empty()).collect();
    let ps: Option<Vec<&Planet>> = parts.iter().map(|x| find(x)).collect();
    let ps = ps?;
    if ps.len() == 1 {
        return Some(card(ps[0]));
    }
    let head: Vec<String> = std::iter::once(String::new()).chain(ps.iter().map(|p| title(p))).collect();
    let keys: Vec<&str> = rows(ps[0]).iter().map(|r| r.0).collect();
    let body = keys.iter().enumerate().map(|(i, k)| std::iter::once(text(*k)).chain(ps.iter().map(|p| text(rows(p)[i].1.clone()))).collect()).collect();
    Some(Answer::new("exoplanets", vec![
        pod("Input", "Input interpretation", text(ps.iter().map(|p| title(p)).collect::<Vec<_>>().join(" | "))),
        primary(pod("Result", "Comparison", table(Some(head.iter().map(|s| s.as_str()).collect()), body))),
    ]))
}
