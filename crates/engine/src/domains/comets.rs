//! Notable comets: designation, orbit, perihelion dates and discovery.
//!
//! "Halley's comet", "last perihelion of Comet Holmes", "Hale-Bopp",
//! "when will Halley's comet return", "period of comet Encke".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

struct Comet {
    /// names separated by ';' (the first is the display name)
    names: &'static str,
    designation: &'static str,
    period: &'static str,
    perihelion_au: &'static str,
    eccentricity: &'static str,
    inclination: &'static str,
    last: &'static str,
    next: &'static str,
    discovered: &'static str,
    nucleus: &'static str,
    note: &'static str,
}

macro_rules! c {
    ($n:expr, $d:expr, $p:expr, $q:expr, $e:expr, $i:expr, $l:expr, $nx:expr, $disc:expr, $nuc:expr, $note:expr) => {
        Comet { names: $n, designation: $d, period: $p, perihelion_au: $q, eccentricity: $e, inclination: $i, last: $l, next: $nx, discovered: $disc, nucleus: $nuc, note: $note }
    };
}

const COMETS: &[Comet] = &[
    c!("Halley's Comet;halley;halley's;halleys;comet halley;halley comet;1p", "1P/Halley", "about 76 years (75–79)", "0.586 AU", "0.967", "162.3° (retrograde)", "February 9, 1986", "July 28, 2061", "periodicity recognised by Edmond Halley in 1705; recorded since at least 240 BC", "about 15 × 8 km", "the only short-period comet regularly visible to the naked eye; parent of the Eta Aquariid and Orionid meteor showers; visited by Giotto and the Vega probes in 1986"),
    c!("Comet Encke;encke;encke's comet;2p", "2P/Encke", "3.30 years (shortest of any bright comet)", "0.34 AU", "0.848", "11.8°", "October 22, 2023", "February 2027", "Pierre Méchain, 1786; orbit computed by Johann Franz Encke in 1819", "about 4.8 km", "parent of the Taurid meteor showers"),
    c!("Comet Swift–Tuttle;swift-tuttle;swift tuttle;109p", "109P/Swift–Tuttle", "about 133 years", "0.96 AU", "0.963", "113.5°", "December 11, 1992", "July 12, 2126", "Lewis Swift and Horace Tuttle, 1862", "about 26 km", "parent of the Perseid meteor shower; the largest object that repeatedly passes close to Earth"),
    c!("Comet Tempel–Tuttle;tempel-tuttle;tempel tuttle;55p", "55P/Tempel–Tuttle", "about 33.2 years", "0.976 AU", "0.906", "162.5° (retrograde)", "February 28, 1998", "May 2031", "Wilhelm Tempel (December 1865) and Horace Tuttle (January 1866)", "about 3.6 km", "parent of the Leonid meteor shower"),
    c!("Comet Holmes;holmes;holmes' comet;holmes's comet;17p", "17P/Holmes", "about 6.9 years", "2.05 AU", "0.43", "19.1°", "February 19, 2021", "about January 2028", "Edwin Holmes, November 6, 1892, during an outburst", "about 3.4 km", "in October 2007 it brightened about a million-fold in under two days, the largest known comet outburst, and its coma briefly grew larger than the Sun"),
    c!("Comet Churyumov–Gerasimenko;churyumov-gerasimenko;churyumov gerasimenko;67p", "67P/Churyumov–Gerasimenko", "about 6.4 years", "1.24 AU", "0.64", "7.0°", "November 2, 2021", "April 2028", "Klim Churyumov and Svetlana Gerasimenko, 1969", "two lobes, about 4.1 × 3.3 × 1.8 km and 2.6 × 2.3 × 1.8 km", "orbited by ESA's Rosetta from 2014 to 2016; the Philae lander touched down on November 12, 2014"),
    c!("Comet Hale–Bopp;hale-bopp;hale bopp", "C/1995 O1 (Hale–Bopp)", "about 2,400 years", "0.914 AU", "0.995", "89.4°", "April 1, 1997", "about the year 4380", "Alan Hale and Thomas Bopp, July 23, 1995", "about 60 km (unusually large)", "visible to the naked eye for a record 18 months in 1996–1997"),
    c!("Comet Hyakutake;hyakutake", "C/1996 B2 (Hyakutake)", "about 70,000 years after its 1996 passage", "0.230 AU", "0.9998", "124.9°", "May 1, 1996", "not for tens of thousands of years", "Yuji Hyakutake, January 30, 1996", "about 4 km", "passed 0.10 AU from Earth on March 25, 1996; first comet found to emit X-rays; its tail was traced to 570 million km"),
    c!("Comet NEOWISE;neowise", "C/2020 F3 (NEOWISE)", "about 6,800 years", "0.295 AU", "0.999", "128.9°", "July 3, 2020", "about the year 8800", "the NEOWISE space telescope, March 27, 2020", "about 5 km", "the brightest comet in the Northern Hemisphere since Hale–Bopp"),
    c!("Comet Shoemaker–Levy 9;shoemaker-levy 9;shoemaker levy 9;shoemaker-levy;sl9", "D/1993 F2 (Shoemaker–Levy 9)", "orbited Jupiter (about 2 years) until destroyed", "—", "—", "—", "—", "none: destroyed", "Carolyn and Eugene Shoemaker and David Levy, March 24, 1993", "broken into 21 fragments up to about 2 km", "torn apart by Jupiter's tides in 1992; the fragments struck Jupiter on July 16–22, 1994, the first observed collision of Solar System bodies"),
    c!("Comet Tempel 1;tempel 1;9p", "9P/Tempel", "about 5.6 years", "1.54 AU", "0.51", "10.5°", "March 2022", "about October 2027", "Wilhelm Tempel, 1867", "about 7.6 × 4.9 km", "struck by NASA's Deep Impact impactor on July 4, 2005"),
    c!("Comet Wild 2;wild 2;81p", "81P/Wild", "about 6.4 years", "1.59 AU", "0.54", "3.2°", "December 2022", "about May 2029", "Paul Wild, 1978", "about 5.5 km", "NASA's Stardust collected dust from its coma in 2004 and returned it to Earth in 2006"),
    c!("Comet Pons–Brooks;pons-brooks;pons brooks;12p", "12P/Pons–Brooks", "about 71 years", "0.78 AU", "0.955", "74.2°", "April 21, 2024", "about 2095", "Jean-Louis Pons, 1812; rediscovered by William Brooks, 1883", "about 30 km", "known for outbursts that gave its coma a horned shape in 2023"),
    c!("Comet Tsuchinshan–ATLAS;tsuchinshan-atlas;tsuchinshan atlas", "C/2023 A3 (Tsuchinshan–ATLAS)", "about 80,000 years (inbound); likely ejected", "0.39 AU", "about 1.0", "139.1°", "September 27, 2024", "none expected", "Purple Mountain Observatory (January 2023) and ATLAS (February 2023)", "a few km", "a bright naked-eye comet in October 2024"),
    c!("Comet McNaught;mcnaught;great comet of 2007", "C/2006 P1 (McNaught)", "about 90,000 years before its passage; now hyperbolic", "0.17 AU", "1.00002", "77.8°", "January 12, 2007", "none: leaving the Solar System", "Robert McNaught, August 7, 2006", "about 25 km", "the brightest comet in over 40 years, with a spectacular fanned tail seen from the Southern Hemisphere"),
    c!("Comet Ikeya–Seki;ikeya-seki;ikeya seki", "C/1965 S1 (Ikeya–Seki)", "about 880 years (main fragment)", "0.0078 AU", "0.9999", "141.9°", "October 21, 1965", "about the 2840s", "Kaoru Ikeya and Tsutomu Seki, September 18, 1965", "split into at least three pieces", "a Kreutz sungrazer; one of the brightest comets of the last thousand years"),
    c!("2I/Borisov;borisov;comet borisov", "2I/Borisov", "none: interstellar, hyperbolic orbit", "2.01 AU", "3.36", "44.1°", "December 8, 2019", "none: leaving the Solar System", "Gennadiy Borisov, August 30, 2019", "under 1 km", "the first known interstellar comet"),
    c!("3I/ATLAS;3i atlas;comet 3i/atlas", "3I/ATLAS", "none: interstellar, hyperbolic orbit", "1.36 AU", "about 6.1", "175° (retrograde)", "October 29, 2025", "none: leaving the Solar System", "the ATLAS survey, July 1, 2025", "under a few km", "the third known interstellar object"),
];

fn find(s: &str) -> Option<&'static Comet> {
    let s = s.trim().trim_start_matches("the ").replace('’', "'");
    let s = re!(r"^(?:great )?comets?\s+|\s+comets?$").replace_all(&s, "").to_string();
    let s = s.replace('–', "-").replace(" - ", "-");
    let s = s.trim();
    COMETS.iter().find(|c| {
        c.names.split(';').any(|n| {
            let n = n.to_lowercase().replace('–', "-");
            let n = n.trim_start_matches("comet ");
            n == s
        }) || c.designation.to_lowercase() == s
    })
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let (prop, name) = if let Some(m) = re!(r"^(?:when (?:was|is|will) (?:the )?(last|next) (?:perihelion|return|appearance|visit) (?:of|for) |(?:the )?(last|next|previous) (?:perihelion|return|appearance|passage)(?: date)? (?:of|for) |(?:the )?(orbital period|period|perihelion|eccentricity|inclination|discovery|discoverer|size|nucleus) (?:of|for) )(.+)$").captures(l) {
        (m.get(1).or(m.get(2)).or(m.get(3)).map(|x| x.as_str().to_string()), m[4].to_string())
    } else if let Some(m) = re!(r"^when (?:will|does) (.+?) (?:return|come back|be visible again|next appear)$").captures(l) {
        (Some("next".to_string()), m[1].to_string())
    } else {
        (None, l.to_string())
    };
    // a bare name needs the word "comet" unless it is a distinctive one
    let c = find(&name)?;
    if prop.is_none() && !name.contains("comet") && !matches!(name.as_str(), "hale-bopp" | "hale bopp" | "hyakutake" | "shoemaker-levy 9" | "shoemaker levy 9" | "swift-tuttle" | "tempel-tuttle" | "churyumov-gerasimenko" | "ikeya-seki" | "pons-brooks" | "tsuchinshan-atlas") && !name.contains('/') {
        return None;
    }
    let disp = c.names.split(';').next().unwrap();
    let (title, res) = match prop.as_deref() {
        Some("last" | "previous") => ("Last perihelion", c.last.to_string()),
        Some("next") => ("Next perihelion", c.next.to_string()),
        Some("period" | "orbital period") => ("Orbital period", c.period.to_string()),
        Some("perihelion") => ("Perihelion distance", c.perihelion_au.to_string()),
        Some("eccentricity") => ("Eccentricity", c.eccentricity.to_string()),
        Some("inclination") => ("Inclination", c.inclination.to_string()),
        Some("discovery" | "discoverer") => ("Discovery", c.discovered.to_string()),
        Some("size" | "nucleus") => ("Nucleus size", c.nucleus.to_string()),
        _ => ("Basic information", format!("{} | period {} | last perihelion {} | next perihelion {}", c.designation, c.period, c.last, c.next)),
    };
    Some(Answer::new(format!("{disp} (comet)"), vec![
        pod("Input", "Input interpretation", text(format!("{disp} ({}){}", c.designation, prop.as_deref().map(|p| format!(" | {p}")).unwrap_or_default()))),
        primary(pod("Result", title, text(res))),
        pod("Table", "Orbital properties", table(None, vec![
            vec![text("designation"), text(c.designation)],
            vec![text("orbital period"), text(c.period)],
            vec![text("perihelion distance"), text(c.perihelion_au)],
            vec![text("eccentricity"), text(c.eccentricity)],
            vec![text("inclination"), text(c.inclination)],
            vec![text("last perihelion"), text(c.last)],
            vec![text("next perihelion"), text(c.next)],
        ])),
        pod("Physical", "Physical and historical properties", table(None, vec![
            vec![text("nucleus size"), text(c.nucleus)],
            vec![text("discovery"), text(c.discovered)],
            vec![text("notes"), text(c.note)],
        ])),
    ]))
}
