//! Earthquake magnitude and intensity scales.
//!
//! "Richter scale 6", "calculate earthquake characteristics seismic moment
//! magnitude 7", "Mercalli scale", "earthquake intensity scales", "Liedu
//! scale", "define body wave magnitude", "Japan Meteorological Agency
//! magnitude definition", "styles of faulting".

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

/// lower magnitude | class | typical effects | worldwide per year (USGS) | typical maximum Mercalli intensity
const CLASSES: &[(f64, &str, &str, &str, &str)] = &[
    (0.0, "micro", "not felt; recorded only by seismographs", "several million", "I"),
    (2.0, "minor", "felt slightly by some people; no damage", "about 1,300,000 (M2) and 130,000 (M3)", "I–III"),
    (4.0, "light", "noticeable shaking of indoor objects and rattling noises; damage very unlikely", "about 13,000", "IV–V"),
    (5.0, "moderate", "can damage poorly constructed buildings; slight damage to well-designed ones", "about 1,300", "VI–VII"),
    (6.0, "strong", "destructive in populated areas up to about 100 km from the epicentre", "about 130", "VII–IX"),
    (7.0, "major", "serious damage over large areas", "about 15", "VIII or higher"),
    (8.0, "great", "severe destruction near the epicentre, felt over very large regions", "about 1", "IX or higher"),
];

/// Modified Mercalli intensity: numeral | shaking | description
const MERCALLI: &[(&str, &str, &str)] = &[
    ("I", "not felt", "not felt except by a very few under especially favourable conditions"),
    ("II", "weak", "felt only by a few people at rest, especially on upper floors"),
    ("III", "weak", "felt quite noticeably indoors; vibration like a passing truck; standing cars may rock slightly"),
    ("IV", "light", "felt indoors by many, outdoors by few; dishes, windows and doors disturbed; walls creak"),
    ("V", "moderate", "felt by nearly everyone, many awakened; some dishes and windows broken; unstable objects overturned"),
    ("VI", "strong", "felt by all, many frightened; some heavy furniture moved; fallen plaster; slight damage"),
    ("VII", "very strong", "negligible damage to well-built buildings, slight to moderate in ordinary structures, considerable in poorly built ones"),
    ("VIII", "severe", "slight damage to specially designed structures; considerable damage and partial collapse of ordinary buildings; chimneys, columns and walls fall"),
    ("IX", "violent", "considerable damage even to specially designed structures; buildings shifted off their foundations"),
    ("X", "extreme", "some well-built wooden structures destroyed; most masonry and frame structures destroyed with their foundations"),
    ("XI", "extreme", "few if any masonry structures remain standing; bridges destroyed; rails bent greatly"),
    ("XII", "extreme", "damage total; waves seen on the ground surface; objects thrown into the air"),
];

/// name; aliases | levels | used in | notes
const INTENSITY: &[(&str, &str, &str, &str)] = &[
    ("Modified Mercalli intensity scale (MMI);mercalli scale;mercalli intensity scale;modified mercalli scale;modified mercalli intensity scale;mmi", "I–XII", "United States and many other countries", "Wood and Neumann (1931) from Mercalli (1902); describes observed effects, not energy"),
    ("European Macroseismic Scale (EMS-98);ems-98;ems98;european macroseismic scale", "I–XII", "Europe", "1998; accounts for building vulnerability classes"),
    ("Medvedev–Sponheuer–Karnik scale (MSK-64);msk-64;msk scale;medvedev-sponheuer-karnik scale", "I–XII", "former USSR states, India, Israel", "1964; forerunner of EMS-98"),
    ("Japan Meteorological Agency seismic intensity scale (shindo);shindo;jma scale;jma seismic intensity scale;japan meteorological agency seismic intensity scale", "0–7 (10 levels: 0, 1, 2, 3, 4, 5−, 5+, 6−, 6+, 7)", "Japan, Taiwan (a similar scale)", "measured instrumentally from ground acceleration since 1996"),
    ("China Seismic Intensity Scale (Liedu);liedu scale;liedu;china seismic intensity scale;csis", "I–XII", "China", "national standard GB/T 17742; 烈度 (liedu) means 'intensity'; combines observed effects with peak ground acceleration and velocity"),
    ("Rossi–Forel scale;rossi-forel scale;rossi forel scale", "I–X", "historical (19th century)", "Michele de Rossi and François-Alphonse Forel, 1883; the first widely used intensity scale"),
];

/// magnitude scales: name;aliases | symbol | definition | notes
const MAGNITUDES: &[(&str, &str, &str, &str)] = &[
    ("local magnitude (Richter);richter scale;richter magnitude;local magnitude;richter magnitude scale", "M_L", "M_L = log₁₀ A − log₁₀ A₀(δ): A is the maximum trace amplitude on a Wood–Anderson torsion seismograph, A₀(δ) a calibration for epicentral distance δ", "Charles Richter, 1935, for southern California; saturates near magnitude 7"),
    ("surface wave magnitude;surface-wave magnitude", "M_s", "M_s = log₁₀(A/T) + 1.66 log₁₀ Δ + 3.3: A is the Rayleigh-wave ground displacement in μm, T ≈ 20 s its period, Δ the epicentral distance in degrees (20°–160°)", "Prague formula (IASPEI, 1962); saturates near magnitude 8"),
    ("body wave magnitude;body-wave magnitude", "m_b", "m_b = log₁₀(A/T) + Q(Δ, h): A is the P-wave ground displacement in μm, T ≈ 1 s its period, Q(Δ, h) a correction for distance and focal depth", "Gutenberg and Richter (1956); used for deep and distant events and to detect nuclear tests; saturates near magnitude 6–6.5"),
    ("moment magnitude;moment magnitude scale;seismic moment magnitude", "M_w", "M_w = (2/3)(log₁₀ M₀ − 9.1), with M₀ = μ A D the seismic moment in N·m (rigidity × rupture area × average slip)", "Hanks and Kanamori, 1979; does not saturate; the standard for large earthquakes"),
    ("Japan Meteorological Agency magnitude;jma magnitude;japan meteorological agency magnitude;japan meteorological agency magnitude scale", "M_j", "M_j = log₁₀ A + 1.73 log₁₀ Δ − 0.83: A is the maximum horizontal ground displacement in μm, Δ the epicentral distance in km", "Tsuboi, 1954; computed by JMA from displacement and velocity amplitudes for shallow earthquakes in and near Japan"),
    ("energy magnitude", "M_e", "M_e = (2/3) log₁₀ E_s − 2.9, with E_s the radiated seismic energy in joules", "Choy and Boatwright, 1995"),
    ("duration magnitude;coda magnitude", "M_d", "M_d = a + b log₁₀ τ + c Δ, with τ the duration of the coda (signal length) in seconds and constants fitted for each network", "used for small local earthquakes"),
];

fn find<'a, T>(table: &'a [(&'a str, T, &'a str, &'a str)], s: &str) -> Option<&'a (&'a str, T, &'a str, &'a str)> {
    table.iter().find(|x| x.0.split(';').any(|a| a.eq_ignore_ascii_case(s)))
}

fn title(x: &str) -> &str {
    x.split(';').next().unwrap_or(x)
}

fn magnitude_card(m: f64, from_moment: bool) -> Answer {
    let c = CLASSES.iter().rev().find(|c| m >= c.0).unwrap_or(&CLASSES[0]);
    let e = 10f64.powf(1.5 * m + 4.8);
    let m0 = 10f64.powf(1.5 * m + 9.1);
    let kt = e / 4.184e12;
    let tnt = if kt >= 1000.0 { format!("{} megatons of TNT", fmt_float(kt / 1000.0, 3)) } else if kt >= 1.0 { format!("{} kilotons of TNT", fmt_float(kt, 3)) } else { format!("{} tons of TNT", fmt_float(kt * 1000.0, 3)) };
    let sci = |v: f64| {
        let ex = v.log10().floor();
        format!("{}×10^{}", fmt_float(v / 10f64.powf(ex), 3), ex as i64)
    };
    let mm = fmt_float(m, 4);
    Answer::new("earthquake magnitude", vec![
        pod("Input", "Input interpretation", text(format!("{} {mm} earthquake", if from_moment { "moment magnitude" } else { "magnitude" }))),
        primary(pod("Result", "Classification", text(format!("{} earthquake: {}", c.1, c.2)))),
        pod("Table", "Properties", table(None, vec![
            vec![text("radiated energy"), text(format!("{} J ({tnt})", sci(e)))],
            vec![text("seismic moment"), text(format!("{} N·m", sci(m0)))],
            vec![text("typical maximum Mercalli intensity"), text(c.4)],
            vec![text("occurrences worldwide per year"), text(c.3)],
            vec![text("energy vs one magnitude less"), text("about 31.6 times more (10^1.5)")],
        ])),
        pod("Formulas", "Formulas", text("log₁₀ E = 1.5 M + 4.8 (E in J, Gutenberg–Richter); M₀ = 10^(1.5 M_w + 9.1) N·m (Hanks–Kanamori)")),
    ])
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"^(?:what is (?:the |a )?|what are (?:the )?|define (?:the )?|definition of (?:the )?|calculate earthquake characteristics,?\s*|earthquake characteristics,?\s*|the )").replace(l, "").to_string();
    let l = re!(r"\s+(?:definition|defined)$").replace(&l, "").to_string();
    let l = l.as_str();
    // a magnitude: "Richter scale 6", "magnitude 7.5 earthquake", "seismic moment magnitude 7"
    if let Some(m) = re!(r"^(?:(richter(?: scale)?|(?:seismic )?moment magnitude|magnitude|earthquake(?: of)?(?: magnitude)?|m_?w|ml)\s+(\d+(?:\.\d+)?)(?: earthquake)?|(?:a )?(?:magnitude )?(\d+(?:\.\d+)?)(?: magnitude)? earthquake)$").captures(l) {
        let v: f64 = m.get(2).or(m.get(3))?.as_str().parse().ok()?;
        if !(0.0..=10.5).contains(&v) {
            return None;
        }
        let moment = m.get(1).is_some_and(|k| k.as_str().contains("moment") || k.as_str().starts_with("mw") || k.as_str().starts_with("m_w"));
        return Some(magnitude_card(v, moment));
    }
    // "mercalli scale VII", "mercalli intensity 8"
    if let Some(m) = re!(r"^(?:modified )?mercalli(?: intensity)?(?: scale)?(?: intensity)? ([ivx]+|\d+)$").captures(l) {
        let k = match m[1].parse::<usize>() {
            Ok(n) => n,
            Err(_) => MERCALLI.iter().position(|x| x.0.eq_ignore_ascii_case(&m[1]))? + 1,
        };
        let x = MERCALLI.get(k.checked_sub(1)?)?;
        return Some(Answer::new("Mercalli intensity", vec![
            pod("Input", "Input interpretation", text(format!("Modified Mercalli intensity {}", x.0))),
            primary(pod("Result", "Shaking", text(format!("{}: {}", x.1, x.2)))),
        ]));
    }
    if let Some(x) = find(INTENSITY, l) {
        let mut pods = vec![
            pod("Input", "Input interpretation", text(title(x.0))),
            primary(pod("Result", "Levels", text(x.1))),
            pod("Table", "Properties", table(None, vec![vec![text("used in"), text(x.2)], vec![text("notes"), text(x.3)]])),
        ];
        if x.0.starts_with("Modified Mercalli") {
            pods.push(pod("Levels", "Intensity levels", table(Some(vec!["intensity", "shaking", "effects"]), MERCALLI.iter().map(|m| vec![text(m.0), text(m.1), text(m.2)]).collect())));
        }
        return Some(Answer::new("seismic intensity scale", pods));
    }
    if re!(r"^(?:list of )?(?:earthquake|seismic) intensity scales?$|^(?:earthquake )?intensity scales$").is_match(l) {
        return Some(Answer::new("seismic intensity scales", vec![
            pod("Input", "Input interpretation", text("earthquake intensity scales")),
            primary(pod("Result", "Scales", text(INTENSITY.iter().map(|x| title(x.0)).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Scales", table(Some(vec!["scale", "levels", "used in"]), INTENSITY.iter().map(|x| vec![text(title(x.0)), text(x.1), text(x.2)]).collect())),
            pod("Note", "Intensity vs magnitude", text("intensity describes the shaking and damage at a place; magnitude measures the size of the earthquake itself")),
        ]));
    }
    if let Some(x) = find(MAGNITUDES, l).or_else(|| find(MAGNITUDES, l.trim_end_matches(" scale"))) {
        return Some(Answer::new("magnitude scale", vec![
            pod("Input", "Input interpretation", text(format!("{} ({})", title(x.0), x.1))),
            primary(pod("Result", "Definition", text(x.2))),
            pod("Notes", "Notes", text(x.3)),
        ]));
    }
    if re!(r"^(?:list of )?(?:earthquake|seismic) magnitude scales?$|^magnitude scales$|^convert earthquake magnitudes$|^earthquake magnitude conversion$").is_match(l) {
        return Some(Answer::new("magnitude scales", vec![
            pod("Input", "Input interpretation", text("earthquake magnitude scales")),
            primary(pod("Result", "Scales", text(MAGNITUDES.iter().map(|x| format!("{} {}", x.1, title(x.0))).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Definitions", table(Some(vec!["symbol", "scale", "definition"]), MAGNITUDES.iter().map(|x| vec![text(x.1), text(title(x.0)), text(x.2)]).collect())),
            pod("Note", "Conversions", text("the scales agree roughly for moderate earthquakes by design; m_b saturates near 6–6.5, M_L near 7 and M_s near 8, so large events are reported as M_w")),
        ]));
    }
    if re!(r"^(?:styles|types|kinds) of (?:faulting|faults?)$|^fault (?:types|styles)$|^faulting styles$").is_match(l) {
        let rows = vec![
            vec![text("normal"), text("extension"), text("hanging wall moves down relative to the footwall"), text("−90°")],
            vec![text("reverse (thrust if dip < 45°)"), text("compression"), text("hanging wall moves up relative to the footwall"), text("+90°")],
            vec![text("strike-slip, left-lateral (sinistral)"), text("shear"), text("blocks slide horizontally; the far side moves left"), text("0°")],
            vec![text("strike-slip, right-lateral (dextral)"), text("shear"), text("blocks slide horizontally; the far side moves right"), text("180°")],
            vec![text("oblique-slip"), text("mixed"), text("a combination of dip-slip and strike-slip motion"), text("other values")],
        ];
        return Some(Answer::new("styles of faulting", vec![
            pod("Input", "Input interpretation", text("styles of faulting")),
            primary(pod("Result", "Styles", text("normal | reverse (thrust) | strike-slip (left- or right-lateral) | oblique-slip"))),
            pod("Table", "Fault styles", table(Some(vec!["style", "stress regime", "motion", "rake"]), rows)),
        ]));
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn magnitude_energy() {
        let a = super::magnitude_card(6.0, false);
        let s = serde_json::to_string(&a.pods).unwrap();
        assert!(s.contains("strong earthquake"));
        assert!(s.contains("6.31×10^13 J"));
    }
}
