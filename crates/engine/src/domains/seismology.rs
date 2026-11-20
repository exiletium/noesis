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

/// Notable historical earthquakes: date | name | ISO 3166 alpha-2 | latitude | longitude | moment magnitude | deaths.
/// Magnitudes and death tolls are the commonly cited values (USGS and standard references); tolls are approximate.
const QUAKES: &str = "\
1556-01-23|Shaanxi earthquake|CN|34.5|109.7|8.0|about 830,000 (estimated)
1693-01-11|Sicily earthquake|IT|37.2|15.0|7.4|about 60,000
1755-11-01|Lisbon earthquake|PT|36.0|-11.0|8.5|about 30,000–50,000
1811-12-16|New Madrid earthquake|US|36.6|-89.6|7.5|few
1886-08-31|Charleston earthquake|US|32.9|-80.0|7.0|about 60
1906-04-18|San Francisco earthquake|US|37.75|-122.55|7.9|about 3,000
1908-12-28|Messina earthquake|IT|38.15|15.68|7.1|about 75,000–82,000
1915-01-13|Avezzano earthquake|IT|42.0|13.5|6.7|about 30,000
1920-12-16|Haiyuan earthquake|CN|36.6|105.3|7.9|about 200,000–270,000
1923-09-01|Great Kantō earthquake|JP|35.3|139.1|7.9|about 105,000
1933-03-11|Long Beach earthquake|US|33.6|-118.0|6.4|about 120
1940-05-24|Lima earthquake|PE|-11.2|-77.8|8.2|about 180
1960-05-22|Valdivia earthquake|CL|-38.2|-73.0|9.5|about 1,000–6,000
1964-03-28|Great Alaska earthquake|US|61.0|-147.7|9.2|131
1965-02-04|Rat Islands earthquake|US|51.2|178.5|8.7|0
1968-01-15|Belice earthquake|IT|37.8|12.9|6.4|about 230–400
1970-05-31|Ancash earthquake|PE|-9.2|-78.8|7.9|about 67,000–70,000
1971-02-09|San Fernando earthquake|US|34.4|-118.4|6.6|65
1974-10-03|Lima earthquake|PE|-12.3|-77.8|8.1|78
1976-02-04|Guatemala earthquake|GT|15.3|-89.1|7.5|about 23,000
1976-05-06|Friuli earthquake|IT|46.3|13.2|6.5|about 990
1976-07-28|Tangshan earthquake|CN|39.6|118.2|7.6|about 242,000 (official)
1980-11-23|Irpinia earthquake|IT|40.8|15.3|6.9|about 2,900
1985-09-19|Mexico City earthquake (Michoacán)|MX|18.2|-102.5|8.0|about 5,000–10,000
1985-09-21|Mexico City aftershock|MX|17.8|-101.6|7.5|included above
1988-12-07|Spitak earthquake|AM|40.9|44.2|6.8|about 25,000
1989-10-17|Loma Prieta earthquake|US|37.04|-121.88|6.9|63
1990-06-21|Manjil–Rudbar earthquake|IR|36.96|49.41|7.4|about 35,000–40,000
1990-07-16|Luzon earthquake|PH|15.7|121.2|7.7|about 1,600
1994-01-17|Northridge earthquake|US|34.21|-118.54|6.7|57
1995-01-17|Great Hanshin (Kobe) earthquake|JP|34.6|135.0|6.9|6,434
1997-09-26|Umbria and Marche earthquake|IT|43.1|12.8|6.0|11
1999-08-17|İzmit earthquake|TR|40.75|29.86|7.6|about 17,000
1999-09-21|Chi-Chi earthquake|TW|23.8|120.8|7.7|2,415
2001-01-26|Gujarat earthquake|IN|23.4|70.2|7.7|about 20,000
2002-11-03|Denali earthquake|US|63.5|-147.4|7.9|0
2003-12-26|Bam earthquake|IR|29.0|58.3|6.6|about 26,000–34,000
2004-12-23|Macquarie Island earthquake|AU|-50.1|160.4|8.1|0
2004-12-26|Indian Ocean earthquake and tsunami (Sumatra–Andaman)|ID|3.3|95.9|9.1|about 228,000
2005-03-28|Nias–Simeulue earthquake|ID|2.1|97.1|8.6|about 1,300
2005-10-08|Kashmir earthquake|PK|34.5|73.6|7.6|about 86,000
2006-05-27|Yogyakarta earthquake|ID|-7.96|110.4|6.4|about 5,700
2007-08-15|Pisco earthquake|PE|-13.4|-76.6|8.0|519
2008-05-12|Wenchuan (Sichuan) earthquake|CN|31.0|103.4|7.9|about 87,600 (dead or missing)
2009-04-06|L'Aquila earthquake|IT|42.35|13.38|6.3|309
2009-09-30|Padang earthquake|ID|-0.7|99.9|7.6|about 1,100
2010-01-12|Haiti earthquake|HT|18.45|-72.5|7.0|about 100,000–300,000 (estimates vary)
2010-02-27|Maule earthquake|CL|-36.1|-72.9|8.8|525
2011-02-22|Christchurch earthquake|NZ|-43.6|172.7|6.2|185
2011-03-11|Tōhoku earthquake and tsunami|JP|38.3|142.4|9.1|about 18,500 (dead or missing)
2012-05-20|Emilia earthquake|IT|44.9|11.2|6.1|27
2013-10-15|Bohol earthquake|PH|9.9|124.1|7.2|222
2015-04-25|Gorkha (Nepal) earthquake|NP|28.2|84.7|7.8|about 9,000
2016-08-24|Amatrice earthquake|IT|42.7|13.2|6.2|299
2016-10-30|Norcia earthquake|IT|42.8|13.1|6.5|0 (direct)
2017-09-08|Chiapas earthquake|MX|15.0|-93.9|8.2|about 98
2017-09-19|Puebla earthquake|MX|18.5|-98.4|7.1|370
2018-09-28|Sulawesi (Palu) earthquake and tsunami|ID|-0.2|119.8|7.5|about 4,300
2018-11-30|Anchorage earthquake|US|61.3|-149.9|7.1|0
2019-07-06|Ridgecrest earthquake|US|35.8|-117.6|7.1|1
2021-07-29|Chignik earthquake|US|55.4|-157.9|8.2|0
2023-02-06|Turkey–Syria (Kahramanmaraş) earthquake|TR|37.2|37.0|7.8|about 59,000
2023-09-08|Al Haouz (Morocco) earthquake|MA|31.1|-8.4|6.8|about 2,960
2024-01-01|Noto Peninsula earthquake|JP|37.5|137.2|7.5|about 500 (including related deaths)
2024-04-03|Hualien earthquake|TW|23.8|121.6|7.4|18";

const MONTHS: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];

/// "earthquakes in Italy", "earthquakes Mexico 1985", "earthquakes Dec 2004", "earthquakes near Lima", "earthquakes Asia"
fn quakes(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:notable |historical |major |largest |biggest |significant |recent )?earthquakes?(?: (?:in|near|around|of|from|for|at))? ?(.*)$").captures(l)?;
    let mut rest = m[1].trim().to_string();
    // a date filter: a year, optionally preceded by a month
    let mut year: Option<i32> = None;
    let mut month: Option<usize> = None;
    if let Some(d) = re!(r"(?:^|\s)(?:in )?(?:(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]*\.? )?(\d{4})$").captures(&rest.clone()) {
        year = d[2].parse().ok();
        if let Some(mo) = d.get(1) {
            month = Some(MONTHS.iter().position(|x| *x == mo.as_str())? + 1);
        }
        rest = rest[..d.get(0)?.start()].trim().to_string();
    }
    let rest = rest.trim_start_matches("in ").trim_start_matches("near ").trim_start_matches("the ").trim();
    if re!(r"\b(?:last|this|past|today|week|month|now|map|focal|mechanism|slip|travel|ground|prediction)\b").is_match(rest) {
        return None;
    }
    let rows: Vec<Vec<&str>> = QUAKES.lines().map(|x| x.split('|').collect()).collect();
    let (filtered, scope): (Vec<(&Vec<&str>, Option<f64>)>, String) = if rest.is_empty() {
        (rows.iter().map(|r| (r, None)).collect(), "worldwide".to_string())
    } else if let Some(cont) = match rest {
        "asia" => Some("Asia"),
        "europe" => Some("Europe"),
        "africa" => Some("Africa"),
        "north america" => Some("North America"),
        "south america" => Some("South America"),
        "oceania" => Some("Oceania"),
        _ => None,
    } {
        let cs = super::geo::countries();
        (rows.iter().filter(|r| cs.values().any(|c| c.iso2 == r[2] && c.continent == cont)).map(|r| (r, None)).collect(), format!("in {cont}"))
    } else if let Some(st) = super::geo::us_state(rest).filter(|_| rest.len() > 2) {
        // US states: within about 600 km of the state's largest city
        let c = super::geo::cities().iter().filter(|c| c.country == "US" && c.admin1 == st).max_by_key(|c| c.population)?;
        let near: Vec<_> = rows.iter().filter(|r| r[2] == "US").filter_map(|r| {
            let d = dist(c.lat, c.lon, r[3].parse().ok()?, r[4].parse().ok()?);
            (d < if st == "AK" { 1500.0 } else { 600.0 }).then_some((r, Some(d)))
        }).collect();
        (near, format!("in {}", cap(rest)))
    } else if let Some(c) = super::geo::find_country(rest).filter(|_| super::geo::find_city(rest).is_none_or(|x| !x.capital || rest.split_whitespace().count() > 1 || rest == "mexico")) {
        (rows.iter().filter(|r| r[2] == c.iso2).map(|r| (r, None)).collect(), format!("in {}", c.name))
    } else {
        let c = super::geo::find_city(rest.trim_end_matches(" 101").trim_end_matches(" tower"))?;
        let near: Vec<_> = rows.iter().filter_map(|r| {
            let d = dist(c.lat, c.lon, r[3].parse().ok()?, r[4].parse().ok()?);
            (d < 500.0).then_some((r, Some(d)))
        }).collect();
        (near, format!("within 500 km of {}", c.name))
    };
    let picked: Vec<_> = filtered.into_iter().filter(|(r, _)| {
        let y: i32 = r[0][..4].parse().unwrap_or(0);
        let mo: usize = r[0][5..7].parse().unwrap_or(0);
        year.is_none_or(|k| k == y) && month.is_none_or(|k| k == mo)
    }).collect();
    if picked.is_empty() {
        return None;
    }
    let mut picked = picked;
    if rest.is_empty() && year.is_none() {
        picked.sort_by(|a, b| b.0[5].parse::<f64>().unwrap_or(0.0).total_cmp(&a.0[5].parse::<f64>().unwrap_or(0.0)));
        picked.truncate(12);
    } else {
        picked.sort_by(|a, b| b.0[0].cmp(a.0[0]));
    }
    let when = match (month, year) {
        (Some(mo), Some(y)) => format!(" | {} {y}", cap(MONTHS[mo - 1])),
        (None, Some(y)) => format!(" | {y}"),
        _ => String::new(),
    };
    let mut head = vec!["date", "earthquake", "magnitude", "deaths"];
    if picked.iter().any(|p| p.1.is_some()) {
        head.push("distance");
    }
    let rows: Vec<Vec<Content>> = picked.iter().map(|(r, d)| {
        let mut v = vec![text(r[0]), text(r[1]), text(format!("M_w {}", r[5])), text(r[6])];
        if let Some(d) = d {
            v.push(text(format!("{} km", super::geo::fmt_int(((d / 10.0).round() * 10.0) as u64))));
        }
        v
    }).collect();
    let short = picked.iter().take(6).map(|(r, _)| format!("{} {} (M_w {})", r[0], r[1], r[5])).collect::<Vec<_>>().join(" | ");
    Some(Answer::new("earthquakes", vec![
        pod("Input", "Input interpretation", text(format!("notable earthquakes | {scope}{when}"))),
        primary(pod("Result", "Notable earthquakes", text(short))),
        pod("Table", "Details", table(Some(head), rows)),
        pod("Note", "Note", text("offline catalog of notable historical earthquakes (moment magnitudes and approximate death tolls); recent and minor events need a live seismic feed such as the USGS")),
    ]))
}

fn dist(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let (p1, p2) = (a.to_radians(), c.to_radians());
    let h = ((p2 - p1) / 2.0).sin().powi(2) + p1.cos() * p2.cos() * ((d - b).to_radians() / 2.0).sin().powi(2);
    2.0 * 6371.0 * h.sqrt().asin()
}

fn cap(s: &str) -> String {
    s.split(' ').map(|w| {
        let mut c = w.chars();
        c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
    }).collect::<Vec<_>>().join(" ")
}

pub fn handle(q: &Query) -> Option<Answer> {
    if let Some(a) = quakes(q.lower.trim().trim_end_matches(['?', '.']).trim()) {
        return Some(a);
    }
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
