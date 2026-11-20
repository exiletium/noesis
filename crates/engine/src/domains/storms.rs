//! Notable tropical cyclones (hurricanes, typhoons and cyclones): dates,
//! peak intensity, deaths and the places they struck.
//!
//! "Hurricane Katrina, Hurricane Rita", "Cyclone Tracy", "When did Typhoon
//! Angela start?", "Hurricane Andrew, hurricane hugo duration".

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

struct Storm {
    name: &'static str,
    kind: &'static str,
    /// other local names
    aliases: &'static [&'static str],
    year: i32,
    basin: &'static str,
    /// (month, day) formed and dissipated; (0, 0) when not given
    formed: (u32, u32),
    dissipated: (u32, u32),
    /// peak 1-minute sustained wind, mph; NaN when not given
    wind: f64,
    /// minimum central pressure, mbar; NaN when not given
    pressure: f64,
    category: &'static str,
    deaths: &'static str,
    damage: &'static str,
    areas: &'static str,
    note: &'static str,
}

const N: f64 = f64::NAN;

const STORMS: &[Storm] = &[
    Storm { name: "Katrina", kind: "hurricane", aliases: &[], year: 2005, basin: "North Atlantic", formed: (8, 23), dissipated: (8, 31), wind: 175.0, pressure: 902.0, category: "Category 5 (Category 3 at Louisiana landfall)", deaths: "1,392 (earlier counts about 1,833)", damage: "about $125 billion (2005 USD)", areas: "Bahamas, South Florida, Louisiana, Mississippi, Alabama", note: "levee failures flooded about 80 % of New Orleans; one of the costliest US hurricanes" },
    Storm { name: "Rita", kind: "hurricane", aliases: &[], year: 2005, basin: "North Atlantic", formed: (9, 18), dissipated: (9, 26), wind: 180.0, pressure: 895.0, category: "Category 5 (Category 3 at landfall)", deaths: "97–125", damage: "about $18.5 billion (2005 USD)", areas: "Florida Keys, southwestern Louisiana, southeastern Texas", note: "the most intense tropical cyclone recorded in the Gulf of Mexico; prompted one of the largest evacuations in US history" },
    Storm { name: "Andrew", kind: "hurricane", aliases: &[], year: 1992, basin: "North Atlantic", formed: (8, 16), dissipated: (8, 28), wind: 175.0, pressure: 922.0, category: "Category 5", deaths: "65", damage: "about $27.3 billion (1992 USD)", areas: "Bahamas, South Florida (Homestead), Louisiana", note: "upgraded to Category 5 in a 2002 reanalysis" },
    Storm { name: "Hugo", kind: "hurricane", aliases: &[], year: 1989, basin: "North Atlantic", formed: (9, 10), dissipated: (9, 25), wind: 160.0, pressure: 918.0, category: "Category 5 (Category 4 at South Carolina landfall)", deaths: "67", damage: "about $11 billion (1989 USD)", areas: "Leeward Islands (Guadeloupe, Montserrat), US Virgin Islands, Puerto Rico, South Carolina", note: "made landfall near Charleston, South Carolina, on September 22, 1989" },
    Storm { name: "Isidore", kind: "hurricane", aliases: &[], year: 2002, basin: "North Atlantic", formed: (9, 14), dissipated: (0, 0), wind: 125.0, pressure: 934.0, category: "Category 3", deaths: "22", damage: "about $1.3 billion (2002 USD)", areas: "Jamaica, Cuba, Yucatán Peninsula, Louisiana", note: "stalled over the Yucatán Peninsula, then crossed the Gulf of Mexico as a tropical storm to Louisiana" },
    Storm { name: "Pauline", kind: "hurricane", aliases: &[], year: 1997, basin: "eastern North Pacific", formed: (10, 5), dissipated: (10, 10), wind: 135.0, pressure: 948.0, category: "Category 4", deaths: "230–400", damage: "about $448 million (1997 USD)", areas: "Oaxaca and Guerrero, Mexico (Acapulco)", note: "floods and mudslides in Acapulco caused most of the deaths" },
    Storm { name: "Tracy", kind: "cyclone", aliases: &[], year: 1974, basin: "Australian region", formed: (12, 20), dissipated: (12, 26), wind: N, pressure: 950.0, category: "Category 4 (Australian scale)", deaths: "71", damage: "about A$837 million (1974)", areas: "Darwin, Northern Territory, Australia", note: "struck Darwin on Christmas Day 1974 and destroyed about 70 % of its buildings; a gust of 217 km/h was recorded before the anemometer failed" },
    Storm { name: "Wanda", kind: "cyclone", aliases: &[], year: 1974, basin: "Australian region", formed: (1, 24), dissipated: (0, 0), wind: N, pressure: N, category: "weak tropical cyclone", deaths: "16 (in the Brisbane flood)", damage: "about A$980 million (1974) for the flood", areas: "southeastern Queensland, Brisbane", note: "crossed the coast near Maryborough on January 25, 1974; its rain caused the January 1974 Brisbane flood" },
    Storm { name: "Angela", kind: "typhoon", aliases: &["rosing"], year: 1995, basin: "western North Pacific", formed: (10, 25), dissipated: (11, 7), wind: 180.0, pressure: 910.0, category: "super typhoon (Category 5 equivalent)", deaths: "about 936", damage: "about $315 million (1995 USD)", areas: "Philippines (Bicol, Luzon including Manila)", note: "known in the Philippines as Super Typhoon Rosing; one of the strongest typhoons to hit the Philippines" },
    Storm { name: "Durian", kind: "typhoon", aliases: &["reming"], year: 2006, basin: "western North Pacific", formed: (11, 25), dissipated: (12, 6), wind: 155.0, pressure: 915.0, category: "super typhoon (Category 4 equivalent)", deaths: "about 1,500", damage: "about $580 million (2006 USD)", areas: "Philippines (Albay, Bicol), Vietnam", note: "known in the Philippines as Typhoon Reming; heavy rain sent lahars down the slopes of Mayon volcano" },
    Storm { name: "Songda", kind: "typhoon", aliases: &[], year: 2004, basin: "western North Pacific", formed: (8, 26), dissipated: (9, 8), wind: N, pressure: 925.0, category: "super typhoon", deaths: "about 45", damage: "about $9 billion (2004 USD)", areas: "Okinawa, Kyushu and Hokkaido (Japan), South Korea", note: "made landfall near Nagasaki, Japan, on September 7, 2004; the name Songda was used again in 2011 and 2016" },
];

static CYCLONES_Z: &[u8] = include_bytes!("../../data/cyclones.z");

/// Every tropical cyclone of tropical-storm strength in IBTrACS (NOAA NCEI).
fn best_track() -> &'static Vec<Storm> {
    static B: OnceLock<Vec<Storm>> = OnceLock::new();
    B.get_or_init(|| {
        let bytes = miniz_oxide::inflate::decompress_to_vec(CYCLONES_Z).unwrap_or_default();
        let text: &'static str = Box::leak(String::from_utf8(bytes).unwrap_or_default().into_boxed_str());
        let md = |t: &str| -> (u32, u32) { (t.get(5..7).and_then(|x| x.parse().ok()).unwrap_or(0), t.get(8..10).and_then(|x| x.parse().ok()).unwrap_or(0)) };
        text.lines()
            .filter_map(|l| {
                let c: Vec<&'static str> = l.split('\t').collect();
                if c.len() < 10 {
                    return None;
                }
                let kt: f64 = c[5].parse().ok()?;
                let cat: i32 = c[7].parse().unwrap_or(0);
                let (kind, word) = match (c[1], cat >= 1) {
                    (_, false) => ("storm", "tropical storm"),
                    ("NA" | "EP", true) => ("hurricane", "hurricane"),
                    ("WP", true) => ("typhoon", if kt >= 130.0 { "super typhoon" } else { "typhoon" }),
                    _ => ("cyclone", "tropical cyclone"),
                };
                let basin = match c[1] {
                    "NA" => "North Atlantic",
                    "EP" => "eastern North Pacific",
                    "WP" => "western North Pacific",
                    "NI" => "North Indian Ocean",
                    "SI" => "South Indian Ocean",
                    "SP" => "South Pacific",
                    _ => "South Atlantic",
                };
                let category: &'static str = Box::leak(if cat >= 1 { format!("Category {cat} {word} (Saffir–Simpson scale)") } else { word.to_string() }.into_boxed_str());
                let note: &'static str = Box::leak(format!("peak intensity near {}° {}, {}° {} (IBTrACS best track)", c[8].trim_start_matches('-'), if c[8].starts_with('-') { "S" } else { "N" }, c[9].trim_start_matches('-'), if c[9].starts_with('-') { "W" } else { "E" }).into_boxed_str());
                Some(Storm {
                    name: c[2],
                    kind,
                    aliases: &[],
                    year: c[3].get(..4)?.parse().ok()?,
                    basin,
                    formed: md(c[3]),
                    dissipated: md(c[4]),
                    wind: (kt * 1.150779 / 5.0).round() * 5.0,
                    pressure: c[6].parse().unwrap_or(N),
                    category,
                    deaths: "",
                    damage: "",
                    areas: "",
                    note,
                })
            })
            .collect()
    })
}

/// "hurricanes in 1991", "typhoons in 2004", "cyclones in 1974"
fn season_list(l: &str) -> Option<Answer> {
    let (sea, what, year) = if let Some(m) = re!(r"^(?:list of |all |the )?(?:(atlantic|pacific|eastern pacific|indian ocean) )?(hurricanes|typhoons|cyclones|tropical cyclones|tropical storms)(?: in| of| during)?(?: the)? (\d{4})(?: season)?$").captures(l) {
        (m.get(1).map(|x| x.as_str().to_string()), m[2].to_string(), m[3].parse::<i32>().ok()?)
    } else {
        let m = re!(r"^(?:the )?(\d{4}) (?:(atlantic|pacific|eastern pacific|indian ocean) )?(hurricanes|typhoons|cyclones|tropical cyclones|hurricane season|typhoon season|cyclone season)$").captures(l)?;
        (m.get(2).map(|x| x.as_str().to_string()), m[3].to_string(), m[1].parse::<i32>().ok()?)
    };
    let basins: &[&str] = match (what.as_str(), sea.as_deref()) {
        (_, Some("atlantic")) => &["North Atlantic"],
        (_, Some("pacific" | "eastern pacific")) if what.starts_with("hurricane") => &["eastern North Pacific"],
        (_, Some("pacific")) => &["western North Pacific"],
        (_, Some("indian ocean")) => &["North Indian Ocean", "South Indian Ocean"],
        (w, _) if w.starts_with("hurricane") => &["North Atlantic", "eastern North Pacific"],
        (w, _) if w.starts_with("typhoon") => &["western North Pacific"],
        (w, _) if w.starts_with("cyclone") => &["North Indian Ocean", "South Indian Ocean", "South Pacific"],
        _ => &["North Atlantic", "eastern North Pacific", "western North Pacific", "North Indian Ocean", "South Indian Ocean", "South Pacific", "South Atlantic"],
    };
    let strong = !what.contains("storm") && !what.contains("tropical") && !what.contains("season");
    let mut list: Vec<&Storm> = best_track().iter().filter(|s| s.year == year && basins.contains(&s.basin) && (!strong || s.kind != "storm")).collect();
    if list.is_empty() {
        return None;
    }
    list.sort_by_key(|s| (basins.iter().position(|b| *b == s.basin), s.formed, s.name));
    let n = list.len();
    let rows: Vec<Vec<Content>> = list
        .iter()
        .take(60)
        .map(|s| vec![text(s.name), text(s.basin), text(format!("{} – {}", date(s, s.formed).unwrap_or_default().trim_end_matches(&format!(", {}", s.year)), date(s, s.dissipated).unwrap_or_default())), text(s.category.trim_end_matches(" (Saffir–Simpson scale)")), text(format!("{} mph", fmt_float(s.wind, 4))), text(if s.pressure.is_nan() { String::new() } else { format!("{} mbar", fmt_float(s.pressure, 5)) })])
        .collect();
    let strongest = list.iter().max_by(|a, b| a.wind.total_cmp(&b.wind))?;
    Some(Answer::new("tropical cyclones", vec![
        pod("Input", "Input interpretation", text(format!("{what} | {year} | {}", basins.join(", ")))),
        primary(pod("Result", "Storms", table(Some(vec!["name", "basin", "dates", "peak intensity", "max wind", "min pressure"]), rows))),
        pod("Summary", "Summary", text(format!("{n} {} in {year}; the strongest was {} ({} mph)", if strong { "storms of hurricane strength" } else { "tropical cyclones" }, title(strongest), fmt_float(strongest.wind, 4)))),
        pod("Source", "Source", text("IBTrACS v04r01 best-track data (NOAA NCEI); winds are 1-minute sustained from US agencies where available, else 10-minute from the regional centre.")),
    ]))
}

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

fn date(s: &Storm, md: (u32, u32)) -> Option<String> {
    (md.0 > 0).then(|| format!("{} {}, {}", MONTHS[md.0 as usize - 1], md.1, s.year))
}

fn days(s: &Storm) -> Option<i64> {
    let (a, b) = (s.formed, s.dissipated);
    if a.0 == 0 || b.0 == 0 {
        return None;
    }
    let doy = |m: u32, d: u32| -> i64 {
        let ml = [31, if s.year % 4 == 0 { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        ml[..m as usize - 1].iter().sum::<i64>() + d as i64
    };
    Some(doy(b.0, b.1) - doy(a.0, a.1))
}

fn title(s: &Storm) -> String {
    let k = match s.kind {
        "hurricane" => "Hurricane",
        "typhoon" => "Typhoon",
        "storm" => "Tropical Storm",
        _ => "Cyclone",
    };
    format!("{k} {} ({})", s.name, s.year)
}

fn wind_text(s: &Storm) -> String {
    if s.wind.is_nan() {
        return "(not given)".into();
    }
    format!("{} mph ({} km/h), 1-minute sustained", fmt_float(s.wind, 6), fmt_float((s.wind * 1.609344).round(), 6))
}

/// "hurricane katrina", "typhoon angela 1995", "cyclone tracy"
fn find(t: &str) -> Option<&'static Storm> {
    let m = re!(r"^(?:the )?(?:(?:super |severe tropical |tropical |major )?(hurricane|typhoon|cyclone|storm) )(\w+)(?: \(?(\d{4})\)?)?$").captures(t.trim())?;
    let name = &m[2];
    let year: Option<i32> = m.get(3).and_then(|y| y.as_str().parse().ok());
    STORMS.iter().find(|s| {
        (s.name.eq_ignore_ascii_case(name) || s.aliases.contains(&name))
            && year.is_none_or(|y| y == s.year)
            && (&m[1] == "storm" || &m[1] == s.kind || (s.kind == "cyclone" && &m[1] == "hurricane"))
    })
    .or_else(|| {
        // the best-track archive: the strongest storm of that name in a basin that uses the word
        let basins: &[&str] = match &m[1] {
            "hurricane" => &["North Atlantic", "eastern North Pacific"],
            "typhoon" => &["western North Pacific"],
            _ => &[],
        };
        best_track()
            .iter()
            .filter(|s| s.name.eq_ignore_ascii_case(name) && s.name != "Unnamed" && year.is_none_or(|y| y == s.year) && (basins.is_empty() || basins.contains(&s.basin)))
            .max_by(|a, b| a.wind.total_cmp(&b.wind))
    })
}

fn rows(s: &Storm) -> Vec<Vec<Content>> {
    let mut r = vec![vec![text("basin"), text(s.basin)]];
    if let Some(f) = date(s, s.formed) {
        r.push(vec![text("formed"), text(f)]);
    }
    if let Some(d) = date(s, s.dissipated) {
        r.push(vec![text("dissipated"), text(d)]);
    }
    if let Some(n) = days(s) {
        r.push(vec![text("duration"), text(format!("{n} days"))]);
    }
    r.push(vec![text("peak intensity"), text(s.category)]);
    if !s.wind.is_nan() {
        r.push(vec![text("maximum wind speed"), text(wind_text(s))]);
    }
    if !s.pressure.is_nan() {
        r.push(vec![text("minimum pressure"), text(format!("{} mbar", fmt_float(s.pressure, 6)))]);
    }
    for (k, v) in [("deaths", s.deaths), ("damage", s.damage), ("areas affected", s.areas)] {
        if !v.is_empty() {
            r.push(vec![text(k), text(v)]);
        }
    }
    r
}

fn card(s: &Storm) -> Answer {
    Answer::new("tropical cyclone", vec![
        pod("Input", "Input interpretation", text(title(s))),
        primary(pod("Result", "Summary", text(format!("{}; {}", s.category, s.note)))),
        pod("Properties", "Properties", table(None, rows(s))),
    ])
}

/// One property of a storm, as (label, value).
fn prop(s: &Storm, p: &str) -> Option<(&'static str, String)> {
    Some(match p {
        "start" | "start date" | "formation date" | "form" | "begin" | "formed" => ("Formed", date(s, s.formed)?),
        "end" | "end date" | "dissipate" | "dissipation date" | "dissipated" => ("Dissipated", date(s, s.dissipated)?),
        "duration" | "length" | "how long" => ("Duration", format!("{} days ({} – {})", days(s)?, date(s, s.formed)?, date(s, s.dissipated)?)),
        "max wind speed" | "maximum wind speed" | "wind speed" | "peak wind speed" | "maximum sustained winds" | "max winds" | "top wind speed" => ("Maximum wind speed", (!s.wind.is_nan()).then(|| wind_text(s))?),
        "deaths" | "death toll" | "fatalities" if !s.deaths.is_empty() => ("Deaths", s.deaths.to_string()),
        "damage" | "cost" if !s.damage.is_empty() => ("Damage", s.damage.to_string()),
        "pressure" | "minimum pressure" | "central pressure" => ("Minimum pressure", format!("{} mbar", fmt_float(s.pressure, 6))),
        "category" | "intensity" | "strength" => ("Peak intensity", s.category.to_string()),
        _ => return None,
    })
}

const PROPS: &str = r"start|start date|formation date|end|end date|dissipation date|duration|length|max wind speed|maximum wind speed|wind speed|peak wind speed|maximum sustained winds|max winds|top wind speed|deaths|death toll|fatalities|damage|cost|pressure|minimum pressure|central pressure|category|intensity|strength";

fn split(t: &str) -> Option<Vec<&'static Storm>> {
    let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\b\.?|\bversus\b|\band\b)\s*").split(t).filter(|x| !x.is_empty()).collect();
    if parts.len() < 2 {
        return None;
    }
    parts.iter().map(|p| find(p)).collect()
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    if let Some(a) = season_list(l) {
        return Some(a);
    }
    // "when did Typhoon Angela start", "when did hurricane katrina end"
    if let Some(m) = re!(r"^when (?:did|was) (.+?) (start|begin|form|formed|end|dissipate|dissipated)$").captures(l) {
        let s = find(&m[1])?;
        let (label, v) = prop(s, &m[2])?;
        return Some(answer_prop(&[s], label, vec![v]));
    }
    // "how long did hurricane andrew last"
    if let Some(m) = re!(r"^how long did (.+?) last$").captures(l) {
        let s = find(&m[1])?;
        let (label, v) = prop(s, "duration")?;
        return Some(answer_prop(&[s], label, vec![v]));
    }
    // "Hurricane Beth max wind speed", "Hurricane Andrew, hurricane hugo duration", "deaths from Cyclone Tracy"
    let pm = Regex::new(&format!(r"^(.+?) ({PROPS})$")).ok()?;
    let pm2 = Regex::new(&format!(r"^(?:the )?({PROPS}) (?:of|for|from|in) (.+)$")).ok()?;
    let hit = pm.captures(l).map(|m| (m[1].to_string(), m[2].to_string())).or_else(|| pm2.captures(l).map(|m| (m[2].to_string(), m[1].to_string())));
    if let Some((who, p)) = hit {
        let list = split(&who).or_else(|| find(&who).map(|s| vec![s]));
        if let Some(list) = list {
            let vals: Option<Vec<(&str, String)>> = list.iter().map(|s| prop(s, &p)).collect();
            let vals = vals?;
            let label = vals[0].0;
            return Some(answer_prop(&list, label, vals.into_iter().map(|x| x.1).collect()));
        }
    }
    // "Hurricane Katrina, Hurricane Rita", "Typhoon Durian 2006, Typhoon Angela 1995"
    if let Some(list) = split(l) {
        let head = vec!["", "formed", "duration", "peak intensity", "max wind", "deaths"];
        let rows = list
            .iter()
            .map(|s| vec![text(title(s)), text(date(s, s.formed).unwrap_or_default()), text(days(s).map(|d| format!("{d} days")).unwrap_or_default()), text(s.category), text(if s.wind.is_nan() { String::new() } else { format!("{} mph", fmt_float(s.wind, 6)) }), text(s.deaths)])
            .collect();
        return Some(Answer::new("tropical cyclones", vec![
            pod("Input", "Input interpretation", text(list.iter().map(|s| title(s)).collect::<Vec<_>>().join(" | "))),
            primary(pod("Result", "Comparison", table(Some(head), rows))),
        ]));
    }
    find(l).map(card)
}

fn answer_prop(list: &[&Storm], label: &str, vals: Vec<String>) -> Answer {
    let main = if list.len() == 1 { vals[0].clone() } else { list.iter().zip(&vals).map(|(s, v)| format!("{}: {v}", title(s))).collect::<Vec<_>>().join(" | ") };
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{} | {}", list.iter().map(|s| title(s)).collect::<Vec<_>>().join(" | "), label.to_lowercase()))),
        primary(pod("Result", label, text(main))),
    ];
    if list.len() == 1 {
        pods.push(pod("Properties", "Properties", table(None, rows(list[0]))));
    }
    Answer::new("tropical cyclone", pods)
}
