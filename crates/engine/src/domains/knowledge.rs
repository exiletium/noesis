//! Assorted reference knowledge and small calculators: element groups and
//! isotopes, sunrise and sunset, movable holidays, world superlatives and
//! country details, annuities and returns, colours, text encodings and
//! hashes, circuits, polygons and special right triangles.

use super::chem::{by_symbol, elements, find_element, molar_mass, parse_formula};
use super::geo::{self, cities, countries, find_city, find_country};
use super::misc::{date_long, money, num, today};
use crate::engine::*;
use crate::print::fmt_float;
use chrono::{Datelike, Duration, NaiveDate, TimeZone, Weekday};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn n(v: f64) -> String {
    fmt_float(v, 6)
}

fn group(v: u64) -> String {
    let s = v.to_string();
    let mut out = String::new();
    for (k, ch) in s.chars().enumerate() {
        if k > 0 && (s.len() - k) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn simple(label: &str, title: &str, value: String, extra: Vec<Pod>) -> Answer {
    let mut pods = vec![pod("Input", "Input interpretation", text(label)), primary(pod("Result", title, text(value)))];
    pods.extend(extra);
    Answer::new(label, pods)
}

// ---------------------------------------------------------------------------
// chemistry: element groups, oxidation states, isotopes, composition
// ---------------------------------------------------------------------------

const GROUPS: &[(&[&str], &str)] = &[
    (&["noble gases", "noble gas", "inert gases", "group 18", "group 18 elements"], "Noble gases"),
    (&["alkali metals", "alkali metal", "group 1 metals"], "Alkali metals"),
    (&["alkaline earth metals", "alkaline earth metal", "group 2", "group 2 elements"], "Alkaline earth metals"),
    (&["halogens", "halogen", "group 17", "group 17 elements"], "Halogens"),
    (&["transition metals", "transition metal", "transition elements"], "Transition metals"),
    (&["lanthanides", "lanthanoids", "rare earth elements", "rare earths"], "Lanthanides"),
    (&["actinides", "actinoids"], "Actinides"),
    (&["metalloids", "semimetals"], "Metalloids"),
    (&["nonmetals", "non-metals", "reactive nonmetals"], "Nonmetals"),
    (&["post-transition metals", "poor metals", "post transition metals"], "Poor metals"),
];

fn chemistry(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    let bare = l.trim_start_matches("list of ").trim_start_matches("list the ").trim_start_matches("all ").trim_start_matches("the ").trim_end_matches(" elements").trim_end_matches(" in the periodic table");
    if let Some((_, series)) = GROUPS.iter().find(|(names, _)| names.contains(&bare) || names.iter().any(|n| bare == format!("{n} elements"))) {
        let els: Vec<_> = elements().iter().filter(|e| e.series.as_deref() == Some(*series)).collect();
        if els.is_empty() {
            return None;
        }
        let rows: Vec<Vec<Content>> = els.iter().map(|e| vec![text(e.z.to_string()), text(e.symbol.clone()), text(e.name.clone()), text(format!("{} u", fmt_float(e.mass, 6)))]).collect();
        return Some(simple(
            &format!("{} (chemical elements)", series.to_lowercase()),
            "Elements",
            els.iter().map(|e| e.name.clone()).collect::<Vec<_>>().join(", "),
            vec![pod("Table", "Members", table(Some(vec!["Z", "symbol", "name", "atomic mass"]), rows)), pod("Count", "Count", text(els.len().to_string()))],
        ));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:common\s+|possible\s+)?(oxidation states?|oxidation numbers?|valences?|valency)\s+(?:of|for)\s+(.+)$").captures(l) {
        let e = find_element(&m[2])?;
        let fmt = |v: &[i32]| v.iter().map(|s| if *s > 0 { format!("+{s}") } else { s.to_string() }).collect::<Vec<_>>().join(", ");
        let common = e.oxidation_states.clone().unwrap_or_default();
        let wants_common = l.contains("common");
        let known = e.oxidation_states_all.clone().filter(|v| !v.is_empty() && !wants_common).unwrap_or_else(|| common.clone());
        let shown = if known.is_empty() { "none (chemically inert)".to_string() } else { fmt(&known) };
        let mut extra = Vec::new();
        if !common.is_empty() && common != known {
            extra.push(pod("Common", "Most common oxidation states", text(fmt(&common))));
        }
        extra.push(pod("Config", "Electron configuration", text(e.config.clone().unwrap_or_default())));
        return Some(simple(&format!("{} | known oxidation states", e.name), "Oxidation states", shown, extra));
    }
    // isotopes
    if let Some(m) = re!(r"^(?:the\s+)?(?:known\s+|stable\s+|common\s+)?isotopes\s+of\s+(.+)$").captures(l) {
        let e = find_element(&m[1])?;
        let list: Vec<&Isotope> = ISOTOPES.iter().filter(|i| i.z == e.z).collect();
        if list.is_empty() {
            return None;
        }
        let rows: Vec<Vec<Content>> = list.iter().map(|i| vec![text(format!("{}-{}", e.name.to_lowercase(), i.a)), text(i.alias.to_string()), text(format!("{} p, {} n", i.z, i.a - i.z)), text(half_life_text(i.half_life_s)), text(i.abundance.to_string())]).collect();
        return Some(simple(
            &format!("isotopes of {}", e.name.to_lowercase()),
            "Isotopes",
            list.iter().map(|i| format!("{}-{}", e.symbol, i.a)).collect::<Vec<_>>().join(", "),
            vec![pod("Table", "Isotope data", table(Some(vec!["isotope", "name", "nucleons", "half-life", "natural abundance"]), rows))],
        ));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:(half[- ]?life|decay constant|mean lifetime)\s+of\s+)?([a-z]+)[- ]?(\d{1,3})(?:\s+isotope)?$").captures(l) {
        let e = find_element(&m[2])?;
        let a: u32 = m[3].parse().ok()?;
        let iso = ISOTOPES.iter().find(|i| i.z == e.z && i.a == a)?;
        let what = m.get(1).map(|g| g.as_str()).unwrap_or("half-life");
        let mut extra = vec![pod("Nucleus", "Nucleus", text(format!("{} protons, {} neutrons", iso.z, a - iso.z)))];
        if !iso.alias.is_empty() {
            extra.push(pod("Name", "Also called", text(iso.alias)));
        }
        extra.push(pod("Abundance", "Natural abundance", text(iso.abundance)));
        if iso.half_life_s.is_finite() {
            let lambda = std::f64::consts::LN_2 / iso.half_life_s;
            extra.push(pod("Decay", "Decay", table(None, vec![vec![text("decay mode"), text(iso.decay)], vec![text("decay constant λ"), text(format!("{} s⁻¹", n(lambda)))], vec![text("mean lifetime τ"), text(half_life_text(1.0 / lambda))]])));
        }
        let (title, value) = match what {
            "decay constant" if iso.half_life_s.is_finite() => ("Decay constant", format!("{} s⁻¹", n(std::f64::consts::LN_2 / iso.half_life_s))),
            "mean lifetime" if iso.half_life_s.is_finite() => ("Mean lifetime", half_life_text(iso.half_life_s / std::f64::consts::LN_2)),
            _ => ("Half-life", half_life_text(iso.half_life_s)),
        };
        return Some(simple(&format!("{}-{a}", e.name.to_lowercase()), title, value, extra));
    }
    // composition
    let c = q.clean.trim();
    if let Some(m) = re!(r"(?i)^(?:the\s+)?(percent(?:age)? composition|mass percent(?:age)?|mass fractions?|composition by mass|empirical formula|simplest formula)\s+(?:of|for)\s+(.+)$").captures(c) {
        let what = m[2].trim();
        let formula = super::chem::compound_formula(what).map(str::to_string).unwrap_or_else(|| what.to_string());
        let counts = parse_formula(&formula)?;
        if m[1].to_lowercase().starts_with("empirical") || m[1].to_lowercase().starts_with("simplest") {
            let g = counts.values().fold(0u32, |a, b| gcd(a, *b));
            let reduced: BTreeMap<String, u32> = counts.iter().map(|(k, v)| (k.clone(), v / g.max(1))).collect();
            let f = hill(&reduced);
            return Some(simple(
                &format!("empirical formula of {formula}"),
                "Empirical formula",
                f.clone(),
                vec![pod("Molecular", "Molecular formula", text(format!("{} = ({f})×{g}", hill(&counts)))), pod("Masses", "Molar masses", text(format!("empirical unit {} g/mol, molecule {} g/mol", fmt_float(molar_mass(&reduced), 6), fmt_float(molar_mass(&counts), 6))))],
            ));
        }
        let mm = molar_mass(&counts);
        let rows: Vec<Vec<Content>> = counts
            .iter()
            .map(|(e, k)| {
                let el = by_symbol(e);
                let mass = el.map_or(0.0, |x| x.mass) * *k as f64;
                vec![text(el.map_or(e.clone(), |x| x.name.clone())), text(k.to_string()), text(format!("{} g/mol", fmt_float(mass, 6))), text(format!("{}%", fmt_float(100.0 * mass / mm, 5)))]
            })
            .collect();
        let summary = counts.iter().map(|(e, k)| format!("{e} {}%", fmt_float(100.0 * by_symbol(e).map_or(0.0, |x| x.mass) * *k as f64 / mm, 4))).collect::<Vec<_>>().join(", ");
        return Some(simple(&format!("mass composition of {formula}"), "Percent composition by mass", summary, vec![pod("Table", "Composition", table(Some(vec!["element", "atoms", "mass", "mass percent"]), rows)), pod("MolarMass", "Molar mass", text(format!("{} g/mol", fmt_float(mm, 7))))]));
    }
    None
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn hill(c: &BTreeMap<String, u32>) -> String {
    let mut keys: Vec<&String> = c.keys().collect();
    if c.contains_key("C") {
        keys.sort_by_key(|k| match k.as_str() {
            "C" => (0, String::new()),
            "H" => (1, String::new()),
            o => (2, o.to_string()),
        });
    }
    keys.iter().map(|k| if c[*k] == 1 { k.to_string() } else { format!("{k}{}", c[*k]) }).collect()
}

struct Isotope {
    z: u32,
    a: u32,
    alias: &'static str,
    /// seconds; infinite for stable isotopes
    half_life_s: f64,
    decay: &'static str,
    abundance: &'static str,
}

const YEAR: f64 = 365.25 * 86400.0;
const DAY: f64 = 86400.0;
const INF: f64 = f64::INFINITY;

const fn iso(z: u32, a: u32, alias: &'static str, half_life_s: f64, decay: &'static str, abundance: &'static str) -> Isotope {
    Isotope { z, a, alias, half_life_s, decay, abundance }
}

static ISOTOPES: &[Isotope] = &[
    iso(1, 1, "protium", INF, "stable", "99.9885%"),
    iso(1, 2, "deuterium", INF, "stable", "0.0115%"),
    iso(1, 3, "tritium", 12.32 * YEAR, "β⁻ to helium-3", "trace"),
    iso(2, 3, "", INF, "stable", "0.000134%"),
    iso(2, 4, "", INF, "stable", "99.999866%"),
    iso(6, 11, "", 20.36 * 60.0, "β⁺ to boron-11", "synthetic"),
    iso(6, 12, "", INF, "stable", "98.93%"),
    iso(6, 13, "", INF, "stable", "1.07%"),
    iso(6, 14, "radiocarbon", 5730.0 * YEAR, "β⁻ to nitrogen-14", "trace (about 1 part per trillion)"),
    iso(7, 14, "", INF, "stable", "99.636%"),
    iso(7, 15, "", INF, "stable", "0.364%"),
    iso(8, 16, "", INF, "stable", "99.757%"),
    iso(8, 17, "", INF, "stable", "0.038%"),
    iso(8, 18, "", INF, "stable", "0.205%"),
    iso(15, 32, "", 14.268 * DAY, "β⁻ to sulfur-32", "synthetic"),
    iso(19, 39, "", INF, "stable", "93.2581%"),
    iso(19, 40, "", 1.248e9 * YEAR, "β⁻ to calcium-40 (89%), electron capture to argon-40 (11%)", "0.0117%"),
    iso(19, 41, "", INF, "stable", "6.7302%"),
    iso(27, 60, "", 5.2714 * YEAR, "β⁻ to nickel-60", "synthetic"),
    iso(38, 90, "", 28.79 * YEAR, "β⁻ to yttrium-90", "fission product"),
    iso(43, 99, "", 211_100.0 * YEAR, "β⁻ to ruthenium-99", "fission product (the medical isomer Tc-99m has a 6.01-hour half-life)"),
    iso(53, 127, "", INF, "stable", "100%"),
    iso(53, 131, "", 8.0252 * DAY, "β⁻ to xenon-131", "fission product"),
    iso(55, 133, "", INF, "stable", "100%"),
    iso(55, 137, "", 30.08 * YEAR, "β⁻ to barium-137", "fission product"),
    iso(84, 210, "", 138.376 * DAY, "α to lead-206", "trace"),
    iso(86, 222, "radon", 3.8235 * DAY, "α to polonium-218", "trace"),
    iso(88, 226, "", 1600.0 * YEAR, "α to radon-222", "trace"),
    iso(90, 232, "", 1.405e10 * YEAR, "α to radium-228", "100%"),
    iso(92, 234, "", 245_500.0 * YEAR, "α to thorium-230", "0.0054%"),
    iso(92, 235, "", 7.04e8 * YEAR, "α to thorium-231", "0.7204%"),
    iso(92, 238, "", 4.468e9 * YEAR, "α to thorium-234", "99.2742%"),
    iso(94, 239, "", 24_110.0 * YEAR, "α to uranium-235", "synthetic"),
    iso(95, 241, "", 432.2 * YEAR, "α to neptunium-237", "synthetic"),
];

fn half_life_text(s: f64) -> String {
    if !s.is_finite() {
        "stable".into()
    } else if s >= YEAR {
        let y = s / YEAR;
        if y >= 1e6 || y < 1000.0 { format!("{} years", n(y)) } else { format!("{} years", group(y.round() as u64)) }
    } else if s >= DAY {
        format!("{} days", n(s / DAY))
    } else if s >= 3600.0 {
        format!("{} hours", n(s / 3600.0))
    } else if s >= 60.0 {
        format!("{} minutes", n(s / 60.0))
    } else {
        format!("{} seconds", n(s))
    }
}

// ---------------------------------------------------------------------------
// sunrise and sunset
// ---------------------------------------------------------------------------

/// UTC hour of sunrise or sunset (NOAA almanac algorithm); `Err(true)` when
/// the sun never rises that day, `Err(false)` when it never sets.
fn sun_event(date: NaiveDate, lat: f64, lon: f64, rising: bool) -> Result<f64, bool> {
    let (rad, deg) = (std::f64::consts::PI / 180.0, 180.0 / std::f64::consts::PI);
    let day = date.ordinal() as f64;
    let lng_hour = lon / 15.0;
    let t = day + ((if rising { 6.0 } else { 18.0 }) - lng_hour) / 24.0;
    let m = 0.9856 * t - 3.289;
    let l = (m + 1.916 * (m * rad).sin() + 0.020 * (2.0 * m * rad).sin() + 282.634).rem_euclid(360.0);
    let mut ra = (0.91764 * (l * rad).tan()).atan() * deg;
    ra = ra.rem_euclid(360.0);
    ra += (l / 90.0).floor() * 90.0 - (ra / 90.0).floor() * 90.0;
    ra /= 15.0;
    let sin_dec = 0.39782 * (l * rad).sin();
    let cos_dec = sin_dec.asin().cos();
    let cos_h = ((90.833 * rad).cos() - sin_dec * (lat * rad).sin()) / (cos_dec * (lat * rad).cos());
    if cos_h > 1.0 {
        return Err(true);
    }
    if cos_h < -1.0 {
        return Err(false);
    }
    let h = if rising { 360.0 - cos_h.acos() * deg } else { cos_h.acos() * deg } / 15.0;
    let local_mean = h + ra - 0.06571 * t - 6.622;
    Ok((local_mean - lng_hour).rem_euclid(24.0))
}

fn sun(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(?:what time is|when is)\s+)?(?:the\s+)?(sunrise|sunset|sun ?rise|sun ?set|daylight hours|day length|length of (?:the )?day|hours of daylight)\s+(?:time\s+)?(?:in|at|for)\s+(.+?)(?:\s+(today|tomorrow|on .+))?$").captures(l)?;
    let place = m[2].trim();
    let (name, lat, lon, tz) = if let Some(c) = find_city(place) {
        (format!("{}, {}", c.name, countries().get(&c.country).map_or(c.country.clone(), |x| x.name.clone())), c.lat, c.lon, c.timezone.clone())
    } else {
        let c = find_country(place)?;
        let big = cities().iter().filter(|x| x.country == c.iso2 && x.capital).max_by_key(|x| x.population).or_else(|| cities().iter().filter(|x| x.country == c.iso2).max_by_key(|x| x.population))?;
        (format!("{}, {}", big.name, c.name), big.lat, big.lon, big.timezone.clone())
    };
    let tzv: chrono_tz::Tz = tz.parse().ok()?;
    let local_today = chrono::Utc::now().with_timezone(&tzv).date_naive();
    let date = match m.get(3).map(|g| g.as_str()) {
        Some("tomorrow") => local_today + Duration::days(1),
        Some(s) if s.starts_with("on ") => super::misc::parse_date(&s[3..])?,
        _ => local_today,
    };
    let at = |utc_hours: f64| -> Option<chrono::DateTime<chrono_tz::Tz>> {
        // the event belongs to the local date; try the UTC day before, of and after
        let secs = (utc_hours * 3600.0).round() as i64;
        [-1i64, 0, 1].iter().filter_map(|off| {
            let base = chrono::Utc.from_utc_datetime(&(date + Duration::days(*off)).and_hms_opt(0, 0, 0)?);
            Some((base + Duration::seconds(secs)).with_timezone(&tzv))
        }).find(|t| t.date_naive() == date)
    };
    let fmt = |t: chrono::DateTime<chrono_tz::Tz>| t.format("%-I:%M %p %Z").to_string();
    let rise = sun_event(date, lat, lon, true);
    let set = sun_event(date, lat, lon, false);
    let what = m[1].replace(' ', "");
    let mut rows = Vec::new();
    let (rise_t, set_t) = (rise.ok().and_then(at), set.ok().and_then(at));
    let polar = match (rise, set) {
        (Err(true), _) | (_, Err(true)) => Some("The sun does not rise on this day (polar night)."),
        (Err(false), _) | (_, Err(false)) => Some("The sun does not set on this day (midnight sun)."),
        _ => None,
    };
    if let (Some(r), Some(s)) = (rise_t, set_t) {
        rows.push(vec![text("sunrise"), text(fmt(r))]);
        rows.push(vec![text("sunset"), text(fmt(s))]);
        let len = (s - r).num_minutes();
        let len = if len < 0 { len + 1440 } else { len };
        rows.push(vec![text("day length"), text(format!("{} h {} min", len / 60, len % 60))]);
        let noon = r + Duration::minutes(len / 2);
        rows.push(vec![text("solar noon"), text(fmt(noon))]);
    }
    let value = match polar {
        Some(p) => p.to_string(),
        None => match what.as_str() {
            "sunrise" => fmt(rise_t?),
            "sunset" => fmt(set_t?),
            _ => content_text(&rows.get(2)?[1]),
        },
    };
    let label = format!("{} in {name} on {}", if what.starts_with("sun") { what.clone() } else { "day length".into() }, date_long(date));
    let mut extra = Vec::new();
    if !rows.is_empty() {
        extra.push(pod("Sun", "Sun times", table(None, rows)));
    }
    extra.push(pod("Location", "Location", text(format!("{:.2}° {}, {:.2}° {}", lat.abs(), if lat >= 0.0 { "N" } else { "S" }, lon.abs(), if lon >= 0.0 { "E" } else { "W" }))));
    extra.push(pod("Note", "Note", text("Computed with the NOAA sunrise equation (accurate to about a minute; ignores terrain and local refraction).")));
    Some(simple(&label, if what == "sunrise" { "Sunrise" } else if what == "sunset" { "Sunset" } else { "Day length" }, value, extra))
}

// ---------------------------------------------------------------------------
// holidays
// ---------------------------------------------------------------------------

fn easter(y: i32) -> Option<NaiveDate> {
    let a = y % 19;
    let b = y / 100;
    let c = y % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    NaiveDate::from_ymd_opt(y, month as u32, day as u32)
}

fn nth_weekday(y: i32, m: u32, wd: Weekday, nth: i32) -> Option<NaiveDate> {
    if nth > 0 {
        NaiveDate::from_weekday_of_month_opt(y, m, wd, nth as u8)
    } else {
        // last
        let next = if m == 12 { NaiveDate::from_ymd_opt(y + 1, 1, 1)? } else { NaiveDate::from_ymd_opt(y, m + 1, 1)? };
        let mut d = next - Duration::days(1);
        while d.weekday() != wd {
            d -= Duration::days(1);
        }
        Some(d)
    }
}

fn holiday_date(name: &str, y: i32) -> Option<(NaiveDate, &'static str)> {
    let fixed = |m: u32, d: u32| NaiveDate::from_ymd_opt(y, m, d);
    Some(match name {
        "easter" | "easter sunday" => (easter(y)?, "Western (Gregorian) Easter: the first Sunday after the ecclesiastical full moon on or after March 21"),
        "good friday" => (easter(y)? - Duration::days(2), "two days before Easter Sunday"),
        "easter monday" => (easter(y)? + Duration::days(1), "the day after Easter Sunday"),
        "ash wednesday" => (easter(y)? - Duration::days(46), "46 days before Easter"),
        "palm sunday" => (easter(y)? - Duration::days(7), "one week before Easter"),
        "pentecost" | "whit sunday" => (easter(y)? + Duration::days(49), "49 days after Easter"),
        "mardi gras" | "shrove tuesday" | "pancake day" => (easter(y)? - Duration::days(47), "the day before Ash Wednesday"),
        "thanksgiving" | "us thanksgiving" | "american thanksgiving" => (nth_weekday(y, 11, Weekday::Thu, 4)?, "fourth Thursday of November (United States)"),
        "canadian thanksgiving" => (nth_weekday(y, 10, Weekday::Mon, 2)?, "second Monday of October (Canada)"),
        "mother's day" | "mothers day" => (nth_weekday(y, 5, Weekday::Sun, 2)?, "second Sunday of May (United States and many other countries)"),
        "father's day" | "fathers day" => (nth_weekday(y, 6, Weekday::Sun, 3)?, "third Sunday of June (United States and many other countries)"),
        "memorial day" => (nth_weekday(y, 5, Weekday::Mon, -1)?, "last Monday of May (United States)"),
        "labor day" | "labour day" => (nth_weekday(y, 9, Weekday::Mon, 1)?, "first Monday of September (United States and Canada)"),
        "martin luther king day" | "mlk day" | "martin luther king jr. day" => (nth_weekday(y, 1, Weekday::Mon, 3)?, "third Monday of January (United States)"),
        "presidents day" | "presidents' day" | "washington's birthday" => (nth_weekday(y, 2, Weekday::Mon, 3)?, "third Monday of February (United States)"),
        "columbus day" | "indigenous peoples' day" => (nth_weekday(y, 10, Weekday::Mon, 2)?, "second Monday of October (United States)"),
        "black friday" => (nth_weekday(y, 11, Weekday::Thu, 4)? + Duration::days(1), "the day after US Thanksgiving"),
        "election day" => (nth_weekday(y, 11, Weekday::Mon, 1)? + Duration::days(1), "the Tuesday after the first Monday of November (United States)"),
        "christmas" | "christmas day" | "xmas" => (fixed(12, 25)?, "December 25"),
        "christmas eve" => (fixed(12, 24)?, "December 24"),
        "boxing day" => (fixed(12, 26)?, "December 26"),
        "new year" | "new year's day" | "new years day" | "new years" => (fixed(1, 1)?, "January 1"),
        "new year's eve" | "new years eve" => (fixed(12, 31)?, "December 31"),
        "halloween" => (fixed(10, 31)?, "October 31"),
        "valentine's day" | "valentines day" => (fixed(2, 14)?, "February 14"),
        "independence day" | "fourth of july" | "july 4th" => (fixed(7, 4)?, "July 4 (United States)"),
        "st patrick's day" | "st. patrick's day" | "saint patrick's day" | "st patricks day" => (fixed(3, 17)?, "March 17"),
        "pi day" => (fixed(3, 14)?, "March 14 (3.14)"),
        "earth day" => (fixed(4, 22)?, "April 22"),
        "juneteenth" => (fixed(6, 19)?, "June 19 (United States)"),
        "veterans day" | "remembrance day" | "armistice day" => (fixed(11, 11)?, "November 11"),
        "bastille day" => (fixed(7, 14)?, "July 14 (France)"),
        "canada day" => (fixed(7, 1)?, "July 1 (Canada)"),
        "summer solstice" => (fixed(6, 21)?, "about June 20–21 (Northern Hemisphere; the exact date varies by year)"),
        "winter solstice" => (fixed(12, 21)?, "about December 21–22 (Northern Hemisphere; the exact date varies by year)"),
        _ => return None,
    })
}

fn holidays(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:when is|when was|when will be|what day is|what date is|date of)?\s*(?:the\s+)?(.+?)(?:\s+(?:in\s+)?(\d{4}))?(?:\s+this year|\s+next year)?$").captures(l)?;
    let name = m[1].trim().trim_end_matches(" day of the week").trim();
    let explicit_year: Option<i32> = m.get(2).and_then(|g| g.as_str().parse().ok());
    let now = today();
    let mut year = explicit_year.unwrap_or(now.year());
    if l.ends_with("next year") {
        year += 1;
    }
    let (mut d, rule) = holiday_date(name, year)?;
    if explicit_year.is_none() && !l.ends_with("next year") && d < now {
        year += 1;
        d = holiday_date(name, year)?.0;
    }
    let days = (d - now).num_days();
    let rel = match days {
        0 => "today".to_string(),
        1 => "tomorrow".to_string(),
        k if k > 0 => format!("in {k} days"),
        k => format!("{} days ago", -k),
    };
    let rows: Vec<Vec<Content>> = (year - 2..=year + 4).filter_map(|y| holiday_date(name, y).map(|(dd, _)| vec![text(y.to_string()), text(date_long(dd))])).collect();
    Some(simple(
        &format!("{name} {year}"),
        "Date",
        format!("{}  ({rel})", date_long(d)),
        vec![pod("Rule", "Rule", text(rule)), pod("Years", "Other years", table(Some(vec!["year", "date"]), rows))],
    ))
}

// ---------------------------------------------------------------------------
// world facts and country details
// ---------------------------------------------------------------------------

/// (questions, answer, details)
const SUPERLATIVES: &[(&[&str], &str, &str)] = &[
    (&["tallest mountain", "highest mountain", "highest point on earth", "tallest mountain on earth", "highest peak"], "Mount Everest — 8,848.86 m (29,032 ft) above sea level", "Nepal / China. Measured from base to peak, Mauna Kea (Hawaii) is taller at about 10,210 m; measured from Earth's centre, Chimborazo (Ecuador) is farthest out."),
    (&["longest river", "longest river on earth"], "the Nile — about 6,650 km (4,130 mi)", "Africa. Some measurements make the Amazon (about 6,400–6,992 km) longer; the answer depends on where the source is taken to be."),
    (&["largest river", "biggest river", "largest river by volume", "river with the most water"], "the Amazon — average discharge about 209,000 m³/s", "South America; more than the next seven largest rivers combined."),
    (&["largest ocean", "biggest ocean", "deepest ocean"], "the Pacific Ocean — about 165 million km² (63 million mi²)", "It covers about 30% of Earth's surface and contains the deepest point, the Challenger Deep (about 10,935 m)."),
    (&["smallest ocean"], "the Arctic Ocean — about 14 million km²", "It is also the shallowest ocean."),
    (&["deepest point", "deepest point in the ocean", "deepest place on earth", "deepest point on earth"], "the Challenger Deep, Mariana Trench — about 10,935 m below sea level", "Western Pacific Ocean."),
    (&["largest desert", "biggest desert"], "Antarctica — about 14.2 million km² (a polar desert)", "The largest hot desert is the Sahara, about 9.2 million km²."),
    (&["largest hot desert"], "the Sahara — about 9.2 million km²", "North Africa."),
    (&["largest lake", "biggest lake"], "the Caspian Sea — about 371,000 km²", "Between Europe and Asia. The largest freshwater lake by area is Lake Superior (82,100 km²); by volume, Lake Baikal."),
    (&["deepest lake"], "Lake Baikal — 1,642 m", "Siberia, Russia; it holds about 22% of the world's fresh surface water."),
    (&["largest continent", "biggest continent"], "Asia — about 44.6 million km² and 4.8 billion people", "About 30% of Earth's land area."),
    (&["smallest continent"], "Australia (Oceania) — about 8.6 million km²", ""),
    (&["largest island", "biggest island"], "Greenland — about 2.13 million km²", "Australia is larger but is counted as a continent."),
    (&["highest waterfall", "tallest waterfall"], "Angel Falls — 979 m", "Venezuela."),
    (&["largest waterfall", "biggest waterfall"], "Inga Falls by flow (about 25,000 m³/s); Victoria Falls is the largest single sheet of falling water", ""),
    (&["tallest building", "tallest building in the world", "tallest skyscraper"], "the Burj Khalifa — 828 m (2,717 ft)", "Dubai, United Arab Emirates; completed in 2010."),
    (&["largest animal", "biggest animal", "largest animal ever"], "the blue whale — up to about 30 m long and 150–190 tonnes", ""),
    (&["fastest animal", "fastest bird"], "the peregrine falcon — over 320 km/h (200 mph) in a hunting dive", "The fastest land animal is the cheetah (about 100–120 km/h)."),
    (&["fastest land animal"], "the cheetah — about 100–120 km/h (60–75 mph) in short bursts", ""),
    (&["tallest animal"], "the giraffe — up to about 5.5–6 m", ""),
    (&["largest planet", "biggest planet"], "Jupiter — mean radius 69,911 km, 318 Earth masses", ""),
    (&["smallest planet"], "Mercury — mean radius 2,440 km", ""),
    (&["hottest planet"], "Venus — mean surface temperature about 464 °C", "Its thick CO₂ atmosphere traps heat, making it hotter than Mercury."),
    (&["closest star", "nearest star"], "the Sun; after it, Proxima Centauri — 4.2465 light years", ""),
    (&["brightest star", "brightest star in the sky"], "Sirius — apparent magnitude −1.46", "8.6 light years away, in Canis Major."),
    (&["largest country"], "Russia — about 17.1 million km²", ""),
    (&["smallest country", "smallest country in the world"], "Vatican City — 0.49 km², population about 800", ""),
    (&["hottest temperature", "highest temperature recorded", "hottest temperature ever recorded", "highest temperature ever recorded"], "56.7 °C (134 °F) — Death Valley, California, July 10, 1913", "The reliability of this record is debated; the highest well-verified reading is 54.4 °C, also in Death Valley."),
    (&["coldest temperature", "lowest temperature recorded", "coldest temperature ever recorded", "lowest temperature ever recorded"], "−89.2 °C (−128.6 °F) — Vostok Station, Antarctica, July 21, 1983", "Satellites have measured surface temperatures near −98 °C on the East Antarctic Plateau."),
];

const FACTS: &[(&[&str], &str, &str)] = &[
    (&["age of the universe", "how old is the universe", "universe age"], "13.787 ± 0.020 billion years", "Planck 2018 cosmological parameters (ΛCDM model)."),
    (&["age of the earth", "how old is the earth", "earth age"], "4.54 ± 0.05 billion years", "From radiometric dating of meteorites and the oldest minerals."),
    (&["age of the sun", "how old is the sun"], "4.57 billion years", "From the oldest meteorite inclusions (4.567 billion years); the Sun is about halfway through its main-sequence life."),
    (&["number of countries in the world", "how many countries are there", "how many countries are there in the world", "how many countries in the world", "number of countries"], "195 — 193 UN member states and 2 observer states (Vatican City and Palestine)", "Some lists also count Taiwan, Kosovo and other partially recognised states."),
    (&["population of the world", "world population", "population of earth", "how many people are on earth", "how many people live on earth", "global population"], "about 8.2 billion (2025)", "UN World Population Prospects estimate; growing by about 70 million people a year."),
    (&["number of bones in the human body", "how many bones are in the human body", "how many bones in the human body", "bones in the human body"], "206 (adult)", "Newborns have about 270, many of which fuse during growth."),
    (&["number of elements", "how many elements are there", "number of chemical elements", "how many elements are on the periodic table"], "118 known elements", "Elements 1–94 occur naturally; the rest have been synthesised."),
    (&["speed of earth", "speed of the earth around the sun", "orbital speed of earth"], "about 29.78 km/s (107,200 km/h)", ""),
    (&["number of stars in the milky way", "how many stars in the milky way", "stars in the milky way"], "about 100–400 billion", ""),
    (&["size of the milky way", "diameter of the milky way"], "about 87,400 light years across (stellar disk)", ""),
    (&["number of galaxies", "how many galaxies are there", "number of galaxies in the universe"], "roughly 200 billion to 2 trillion in the observable universe", "Estimates depend on how faint galaxies are counted."),
    (&["number of human chromosomes", "how many chromosomes do humans have", "human chromosomes"], "46 (23 pairs)", ""),
    (&["normal body temperature", "human body temperature", "body temperature"], "about 37 °C (98.6 °F)", "Normal ranges are roughly 36.1–37.2 °C and vary through the day."),
];

fn facts(l: &str) -> Option<Answer> {
    let t = l.trim_start_matches("what is ").trim_start_matches("what's ").trim_start_matches("the ").trim_end_matches(" in the world").trim_end_matches(" on earth").trim_end_matches(" of the world");
    for (keys, ans, note) in SUPERLATIVES.iter().chain(FACTS.iter()) {
        let hit = keys.iter().any(|k| *k == t || *k == l || format!("the {k}") == l || *k == t.trim_start_matches("which is the ").trim_start_matches("what is the "));
        if hit {
            let mut extra = Vec::new();
            if !note.is_empty() {
                extra.push(pod("Details", "Details", text(*note)));
            }
            return Some(simple(keys[0], "Result", ans.to_string(), extra));
        }
    }
    None
}

const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"), ("fr", "French"), ("de", "German"), ("it", "Italian"), ("es", "Spanish"), ("pt", "Portuguese"), ("nl", "Dutch"), ("rm", "Romansh"), ("ru", "Russian"), ("uk", "Ukrainian"),
    ("pl", "Polish"), ("cs", "Czech"), ("sk", "Slovak"), ("hu", "Hungarian"), ("ro", "Romanian"), ("bg", "Bulgarian"), ("el", "Greek"), ("tr", "Turkish"), ("ar", "Arabic"), ("he", "Hebrew"),
    ("fa", "Persian"), ("hi", "Hindi"), ("bn", "Bengali"), ("ur", "Urdu"), ("ta", "Tamil"), ("te", "Telugu"), ("mr", "Marathi"), ("gu", "Gujarati"), ("kn", "Kannada"), ("ml", "Malayalam"),
    ("pa", "Punjabi"), ("zh", "Chinese"), ("ja", "Japanese"), ("ko", "Korean"), ("vi", "Vietnamese"), ("th", "Thai"), ("ms", "Malay"), ("id", "Indonesian"), ("tl", "Tagalog"), ("fil", "Filipino"),
    ("sv", "Swedish"), ("no", "Norwegian"), ("nb", "Norwegian Bokmål"), ("nn", "Norwegian Nynorsk"), ("da", "Danish"), ("fi", "Finnish"), ("is", "Icelandic"), ("ga", "Irish"), ("cy", "Welsh"), ("gd", "Scottish Gaelic"),
    ("ca", "Catalan"), ("eu", "Basque"), ("gl", "Galician"), ("lb", "Luxembourgish"), ("mt", "Maltese"), ("sw", "Swahili"), ("am", "Amharic"), ("ha", "Hausa"), ("yo", "Yoruba"), ("ig", "Igbo"),
    ("zu", "Zulu"), ("xh", "Xhosa"), ("af", "Afrikaans"), ("so", "Somali"), ("ne", "Nepali"), ("si", "Sinhala"), ("my", "Burmese"), ("km", "Khmer"), ("lo", "Lao"), ("mn", "Mongolian"),
    ("kk", "Kazakh"), ("uz", "Uzbek"), ("hy", "Armenian"), ("ka", "Georgian"), ("az", "Azerbaijani"), ("sq", "Albanian"), ("sr", "Serbian"), ("hr", "Croatian"), ("bs", "Bosnian"), ("sl", "Slovenian"),
    ("mk", "Macedonian"), ("lt", "Lithuanian"), ("lv", "Latvian"), ("et", "Estonian"), ("be", "Belarusian"), ("qu", "Quechua"), ("gn", "Guaraní"), ("ay", "Aymara"), ("mi", "Māori"), ("sm", "Samoan"),
];

fn language_name(code: &str) -> Option<String> {
    let base = code.split('-').next()?.to_lowercase();
    let name = LANGUAGES.iter().find(|(c, _)| *c == base)?.1;
    Some(match code.split_once('-') {
        Some((_, region)) => format!("{name} ({region})"),
        None => name.to_string(),
    })
}

fn countries_q(l: &str) -> Option<Answer> {
    // countries in a continent
    if let Some(m) = re!(r"^(?:list (?:of\s+)?(?:all\s+)?(?:the\s+)?|all\s+(?:the\s+)?|the\s+|how many\s+|number of\s+)?countries\s+(?:are\s+)?(?:there\s+)?(?:in|of)\s+(europe|asia|africa|north america|south america|oceania|antarctica|the world)$").captures(l) {
        if &m[1] == "the world" {
            return facts("number of countries in the world");
        }
        let continent = &m[1];
        let mut cs: Vec<&geo::Country> = countries().values().filter(|c| c.continent.eq_ignore_ascii_case(continent)).collect();
        cs.sort_by(|a, b| a.name.cmp(&b.name));
        let rows: Vec<Vec<Content>> = cs.iter().map(|c| vec![text(c.name.clone()), text(c.capital.clone()), text(group(c.population))]).collect();
        let names = cs.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(", ");
        return Some(simple(
            &format!("countries and territories in {continent}"),
            &format!("{} countries and territories", cs.len()),
            names,
            vec![pod("Table", "Details", table(Some(vec!["country", "capital", "population"]), rows)), pod("Note", "Note", text("Includes dependent territories listed in the built-in country table."))],
        ));
    }
    if let Some(m) = re!(r"^(?:how many\s+)?people (?:live|are living|reside|are there) in\s+(.+)$").captures(l) {
        let place = m[1].trim();
        if let Some(c) = find_country(place) {
            return Some(simple(&format!("{} | population", c.name), "Population", format!("{} people", group(c.population)), vec![]));
        }
        let c = find_city(place)?;
        return Some(simple(&format!("{} | population", c.name), "Population", format!("{} people (city proper)", group(c.population)), vec![]));
    }
    let m = re!(r"^(?:the\s+)?(population density|density|languages?(?: spoken)?|official languages?|time ?zones?|latitude|longitude|coordinates|location|neighbou?ring countries|bordering countries|neighbou?rs|borders|internet domain|top level domain|tld|iso code|country code)\s+(?:of|in|for)\s+(.+)$").captures(l)?;
    let what = m[1].to_string();
    let place = m[2].trim();
    if let Some(c) = find_country(place) {
        let (title, value) = match what.as_str() {
            "population density" | "density" => ("Population density", format!("{} people/km²  ({} people/mi²)", fmt_float(c.population as f64 / c.density_area(), 4), fmt_float(c.population as f64 / (c.density_area() / 2.589988), 4))),
            w if w.starts_with("language") || w.starts_with("official") => ("Languages", {
                let mut names: Vec<String> = Vec::new();
                for code in c.languages.split(',') {
                    if let Some(n) = language_name(code.trim()) {
                        let base = n.split(" (").next().unwrap_or(&n).to_string();
                        if !names.iter().any(|x| x.starts_with(&base)) {
                            names.push(base);
                        }
                    }
                }
                if names.is_empty() { c.languages.clone() } else { names.join(", ") }
            }),
            w if w.starts_with("time") => ("Time zones", {
                let mut tz: Vec<String> = cities().iter().filter(|x| x.country == c.iso2).map(|x| x.timezone.clone()).collect();
                tz.sort();
                tz.dedup();
                let offs: Vec<String> = tz.iter().take(12).filter_map(|t| geo::local_time(t).map(|now| format!("{t} (UTC{})", now.format("%:z")))).collect();
                offs.join(", ")
            }),
            w if w.contains("neighbo") || w.contains("border") => ("Bordering countries", {
                let names: Vec<String> = c.neighbours.iter().filter_map(|k| countries().get(k)).map(|x| x.name.clone()).collect();
                if names.is_empty() { "none (no land borders)".to_string() } else { names.join(", ") }
            }),
            "internet domain" | "top level domain" | "tld" => ("Internet country code", c.tld.clone()),
            "iso code" | "country code" => ("ISO codes", format!("{} / {}  (calling code +{})", c.iso2, c.iso3, c.calling_code)),
            _ => {
                let cap = cities().iter().find(|x| x.country == c.iso2 && x.capital)?;
                ("Coordinates of the capital", format!("{}: {:.4}° {}, {:.4}° {}", cap.name, cap.lat.abs(), if cap.lat >= 0.0 { "N" } else { "S" }, cap.lon.abs(), if cap.lon >= 0.0 { "E" } else { "W" }))
            }
        };
        return Some(simple(&format!("{} | {what}", c.name), title, value, vec![]));
    }
    let city = find_city(place)?;
    let ns = |v: f64, p: &str, q: &str| format!("{:.4}° {}", v.abs(), if v >= 0.0 { p } else { q });
    let (title, value) = match what.as_str() {
        "latitude" => ("Latitude", ns(city.lat, "N", "S")),
        "longitude" => ("Longitude", ns(city.lon, "E", "W")),
        "coordinates" | "location" => ("Coordinates", format!("{}, {}", ns(city.lat, "N", "S"), ns(city.lon, "E", "W"))),
        w if w.starts_with("time") => ("Time zone", {
            let now = geo::local_time(&city.timezone)?;
            format!("{} ({}, UTC{})", city.timezone, now.format("%Z"), now.format("%:z"))
        }),
        _ => return None,
    };
    let country = countries().get(&city.country).map_or(city.country.clone(), |c| c.name.clone());
    Some(simple(&format!("{}, {country} | {what}", city.name), title, value, vec![]))
}

// ---------------------------------------------------------------------------
// finance
// ---------------------------------------------------------------------------

fn finance(l: &str) -> Option<Answer> {
    const N: &str = r"\$?([\d,]*\.?\d+k?)";
    let rx = |s: &str| Regex::new(&s.replace("{N}", N)).ok();
    // annuities
    if let Some(m) = rx(r"^(?:the\s+)?(future value|present value|value)\s+(?:of\s+)?(?:an annuity\s+(?:of\s+)?)?{N}\s+(?:per|a|each|every)\s+(month|year|quarter|week)\s+(?:at|with|@)\s+(\d+(?:\.\d+)?)\s*%(?:\s*(?:interest|per year|annually|apr|a year))?\s+(?:for|over)\s+(\d+(?:\.\d+)?)\s+years?$")?.captures(l) {
        let pmt = num(&m[2])?;
        let per_year = match &m[3] { "month" => 12.0, "quarter" => 4.0, "week" => 52.0, _ => 1.0 };
        let r = num(&m[4])? / 100.0 / per_year;
        let periods = num(&m[5])? * per_year;
        let fv = if r == 0.0 { pmt * periods } else { pmt * ((1.0 + r).powf(periods) - 1.0) / r };
        let pv = if r == 0.0 { pmt * periods } else { pmt * (1.0 - (1.0 + r).powf(-periods)) / r };
        let present = m[1].starts_with("present");
        let contributed = pmt * periods;
        let (title, v) = if present { ("Present value", pv) } else { ("Future value", fv) };
        return Some(simple(
            &format!("{} of {} per {} at {}% for {} years", title.to_lowercase(), money(pmt), &m[3], &m[4], &m[5]),
            title,
            money(v),
            vec![
                pod("Totals", "Totals", table(None, vec![vec![text("total contributions"), text(money(contributed))], vec![text(if present { "future value" } else { "interest earned" }), text(if present { money(fv) } else { money(fv - contributed) })], vec![text("periods"), text(n(periods))]])),
                pod("Formula", "Formula", text(if present { "PV = P · (1 − (1 + r)^(−n)) / r" } else { "FV = P · ((1 + r)^n − 1) / r   (payments at the end of each period)" })),
            ],
        ));
    }
    // return on investment
    if let Some(m) = rx(r"^(?:roi|return on investment|return|gain)\s+(?:of\s+|from\s+|on\s+)?(?:investing\s+)?{N}\s+(?:to|->|becomes|grows to|returning|that became)\s+{N}(?:\s+(?:in|over|after)\s+(\d+(?:\.\d+)?)\s+years?)?$")?.captures(l) {
        let (a, b) = (num(&m[1])?, num(&m[2])?);
        if a == 0.0 {
            return None;
        }
        let roi = (b - a) / a * 100.0;
        let mut extra = vec![pod("Gain", "Net gain", text(money(b - a))), pod("Multiple", "Multiple", text(format!("{}×", n(b / a))))];
        if let Some(y) = m.get(3).and_then(|g| g.as_str().parse::<f64>().ok()).filter(|y| *y > 0.0) {
            extra.push(pod("CAGR", "Compound annual growth rate", text(format!("{}% per year", n(((b / a).powf(1.0 / y) - 1.0) * 100.0)))));
        }
        return Some(simple(&format!("return on {} growing to {}", money(a), money(b)), "Return on investment", format!("{}{}%", if roi > 0.0 { "+" } else { "" }, n(roi)), extra));
    }
    // CAGR
    if let Some(m) = rx(r"^(?:cagr|compound annual growth rate|annual growth rate)\s+(?:from\s+|of\s+)?{N}\s+(?:to)\s+{N}\s+(?:in|over)\s+(\d+(?:\.\d+)?)\s+years?$")?.captures(l) {
        let (a, b, y) = (num(&m[1])?, num(&m[2])?, num(&m[3])?);
        return Some(simple(&format!("CAGR from {} to {} over {y} years", n(a), n(b)), "Compound annual growth rate", format!("{}% per year", n(((b / a).powf(1.0 / y) - 1.0) * 100.0)), vec![pod("Formula", "Formula", text("CAGR = (end / start)^(1/years) − 1"))]));
    }
    // APR to APY
    if let Some(m) = re!(r"^(?:convert\s+)?(?:apr\s+(?:to|into|in)\s+apy|apy|effective annual rate|ear|effective rate)\s+(?:of\s+|for\s+)?(\d+(?:\.\d+)?)\s*%(?:\s+apr)?(?:\s+(?:compounded\s+)?(daily|weekly|monthly|quarterly|semiannually|annually|continuously))?$").captures(l) {
        let apr = m[1].parse::<f64>().ok()? / 100.0;
        let how = m.get(2).map_or("monthly", |g| g.as_str());
        let apy = match how {
            "continuously" => apr.exp() - 1.0,
            other => {
                let k = match other { "daily" => 365.0, "weekly" => 52.0, "quarterly" => 4.0, "semiannually" => 2.0, "annually" => 1.0, _ => 12.0 };
                (1.0 + apr / k).powf(k) - 1.0
            }
        };
        let rows: Vec<Vec<Content>> = [("annually", 1.0), ("quarterly", 4.0), ("monthly", 12.0), ("daily", 365.0)].iter().map(|(name, k)| vec![text(*name), text(format!("{}%", n(((1.0 + apr / k).powf(*k) - 1.0) * 100.0)))]).chain(std::iter::once(vec![text("continuously"), text(format!("{}%", n((apr.exp() - 1.0) * 100.0)))])).collect();
        return Some(simple(&format!("APY for {}% APR compounded {how}", n(apr * 100.0)), "Annual percentage yield", format!("{}%", n(apy * 100.0)), vec![pod("Table", "By compounding frequency", table(Some(vec!["compounding", "APY"]), rows)), pod("Formula", "Formula", text("APY = (1 + APR/n)^n − 1"))]));
    }
    // break-even
    if let Some(m) = re!(r"^break[- ]?even(?:\s+point|\s+analysis)?\b(.*)$").captures(l) {
        let rest = m[1].to_string();
        // "fixed 1000" or "1000 fixed": when the text ends with a keyword the
        // numbers come before their labels
        let trailing = re!(r"(?:fixed|variable|cost|costs|price|unit)\s*$").is_match(rest.trim());
        let grab = |keys: &str| -> Option<f64> {
            let pat = if trailing { format!(r"\$?([\d,]*\.?\d+)\s*(?:{keys})") } else { format!(r"(?:{keys})\s*(?:costs?\s*)?(?:=|of|is|:)?\s*\$?([\d,]*\.?\d+)") };
            let c = Regex::new(&pat).ok()?.captures(&rest)?;
            num(c.get(1)?.as_str())
        };
        let fixed = grab("fixed")?;
        let var = grab("variable|unit cost|cost per unit|per unit cost")?;
        let price = grab("price|selling price|sale price")?;
        if price <= var {
            return Some(simple("break-even point", "Result", "No break-even point: the price does not exceed the variable cost per unit.".into(), vec![]));
        }
        let units = fixed / (price - var);
        return Some(simple(
            &format!("break-even with fixed costs {}, variable cost {} per unit, price {}", money(fixed), money(var), money(price)),
            "Break-even quantity",
            format!("{} units", n(units)),
            vec![pod("Revenue", "Break-even revenue", text(money(units * price))), pod("Margin", "Contribution margin", text(format!("{} per unit ({}%)", money(price - var), n((price - var) / price * 100.0)))), pod("Formula", "Formula", text("units = fixed costs ÷ (price − variable cost per unit)"))],
        ));
    }
    None
}

// ---------------------------------------------------------------------------
// colours, encodings and hashes
// ---------------------------------------------------------------------------

const COLORS: &[(&str, u32)] = &[
    ("red", 0xFF0000), ("green", 0x008000), ("lime", 0x00FF00), ("blue", 0x0000FF), ("yellow", 0xFFFF00), ("cyan", 0x00FFFF), ("magenta", 0xFF00FF), ("black", 0x000000), ("white", 0xFFFFFF),
    ("gray", 0x808080), ("grey", 0x808080), ("silver", 0xC0C0C0), ("maroon", 0x800000), ("olive", 0x808000), ("purple", 0x800080), ("teal", 0x008080), ("navy", 0x000080), ("orange", 0xFFA500),
    ("pink", 0xFFC0CB), ("brown", 0xA52A2A), ("gold", 0xFFD700), ("indigo", 0x4B0082), ("violet", 0xEE82EE), ("turquoise", 0x40E0D0), ("coral", 0xFF7F50), ("salmon", 0xFA8072), ("crimson", 0xDC143C),
    ("lavender", 0xE6E6FA), ("beige", 0xF5F5DC), ("khaki", 0xF0E68C), ("chocolate", 0xD2691E), ("tomato", 0xFF6347), ("orchid", 0xDA70D6), ("plum", 0xDDA0DD), ("tan", 0xD2B48C), ("skyblue", 0x87CEEB),
    ("sky blue", 0x87CEEB), ("forestgreen", 0x228B22), ("forest green", 0x228B22), ("hotpink", 0xFF69B4), ("hot pink", 0xFF69B4), ("steelblue", 0x4682B4), ("steel blue", 0x4682B4), ("aqua", 0x00FFFF), ("fuchsia", 0xFF00FF),
];

fn hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let l = (mx + mn) / 2.0;
    if mx == mn {
        return (0.0, 0.0, l);
    }
    let d = mx - mn;
    let s = if l > 0.5 { d / (2.0 - mx - mn) } else { d / (mx + mn) };
    let h = if mx == r { (g - b) / d + if g < b { 6.0 } else { 0.0 } } else if mx == g { (b - r) / d + 2.0 } else { (r - g) / d + 4.0 };
    (h * 60.0, s, l)
}

fn color(l: &str) -> Option<Answer> {
    let rgb: u32 = if let Some(m) = re!(r"^(?:colou?r\s+|hex\s+(?:colou?r\s+)?)?#([0-9a-f]{6}|[0-9a-f]{3})(?:\s+(?:to|in)\s+(?:rgb|hsl|cmyk))?$").captures(l) {
        let h = &m[1];
        let full = if h.len() == 3 { h.chars().flat_map(|c| [c, c]).collect::<String>() } else { h.to_string() };
        u32::from_str_radix(&full, 16).ok()?
    } else if let Some(m) = re!(r"^(?:colou?r\s+)?rgb\s*\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*\)(?:\s+(?:to|in)\s+(?:hex|hsl|cmyk))?$").captures(l) {
        let c: Vec<u32> = (1..=3).filter_map(|k| m[k].parse().ok()).collect();
        if c.iter().any(|v| *v > 255) {
            return None;
        }
        (c[0] << 16) | (c[1] << 8) | c[2]
    } else if let Some(m) = re!(r"^(?:the\s+)?(?:colou?r\s+|hex (?:code|value) (?:of|for)\s+|rgb (?:of|for)\s+)([a-z ]+?)(?:\s+(?:colou?r|in hex|in rgb))?$").captures(l) {
        COLORS.iter().find(|(name, _)| *name == m[1].trim())?.1
    } else {
        return None;
    };
    let (r, g, b) = ((rgb >> 16) & 255, (rgb >> 8) & 255, rgb & 255);
    let (rf, gf, bf) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    let (h, s, li) = hsl(rf, gf, bf);
    let k = 1.0 - rf.max(gf).max(bf);
    let cmyk = if k >= 1.0 { (0.0, 0.0, 0.0, 1.0) } else { ((1.0 - rf - k) / (1.0 - k), (1.0 - gf - k) / (1.0 - k), (1.0 - bf - k) / (1.0 - k), k) };
    let lin = |c: f64| if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
    let lum = 0.2126 * lin(rf) + 0.7152 * lin(gf) + 0.0722 * lin(bf);
    let contrast_white = 1.05 / (lum + 0.05);
    let contrast_black = (lum + 0.05) / 0.05;
    let name = COLORS.iter().min_by_key(|(_, v)| {
        let (a, bb, c) = (((v >> 16) & 255) as i64 - r as i64, ((v >> 8) & 255) as i64 - g as i64, (v & 255) as i64 - b as i64);
        a * a + bb * bb + c * c
    }).map(|(nme, _)| *nme).unwrap_or("");
    let hex = format!("#{rgb:06X}");
    let comp = 0xFFFFFF ^ rgb;
    Some(simple(
        &format!("colour {hex}"),
        "Colour",
        format!("{hex}  ·  rgb({r}, {g}, {b})"),
        vec![
            pod("Forms", "Representations", table(None, vec![
                vec![text("hex"), text(hex.clone())],
                vec![text("RGB"), text(format!("({r}, {g}, {b})"))],
                vec![text("HSL"), text(format!("({}°, {}%, {}%)", n(h.round()), n((s * 100.0).round()), n((li * 100.0).round())))],
                vec![text("CMYK"), text(format!("({}%, {}%, {}%, {}%)", n((cmyk.0 * 100.0).round()), n((cmyk.1 * 100.0).round()), n((cmyk.2 * 100.0).round()), n((cmyk.3 * 100.0).round())))],
                vec![text("decimal"), text(rgb.to_string())],
            ])),
            pod("Nearest", "Nearest named colour", text(name)),
            pod("Complement", "Complementary colour", text(format!("#{comp:06X}"))),
            pod("Contrast", "Contrast ratio", text(format!("against white {}:1, against black {}:1 (relative luminance {})", fmt_float(contrast_white, 3), fmt_float(contrast_black, 3), fmt_float(lum, 4)))),
        ],
    ))
}

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let v = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            if k <= chunk.len() {
                out.push(B64[((v >> (18 - 6 * k)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let s: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if s.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::new();
    for chunk in s.chunks(4) {
        let mut v = 0u32;
        let mut pad = 0;
        for &c in chunk {
            v <<= 6;
            if c == b'=' {
                pad += 1;
            } else {
                v |= B64.iter().position(|x| *x == c)? as u32;
            }
        }
        let bytes = [(v >> 16) as u8, (v >> 8) as u8, v as u8];
        out.extend_from_slice(&bytes[..3 - pad]);
    }
    Some(out)
}

fn md5(data: &[u8]) -> String {
    let s: [u32; 64] = [7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21];
    let k: Vec<u32> = (0..64).map(|i| ((i as f64 + 1.0).sin().abs() * 4294967296.0).floor() as u32).collect();
    let (mut a0, mut b0, mut c0, mut d0) = (0x67452301u32, 0xefcdab89u32, 0x98badcfeu32, 0x10325476u32);
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());
    for chunk in msg.chunks(64) {
        let w: Vec<u32> = chunk.chunks(4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let f = f.wrapping_add(a).wrapping_add(k[i]).wrapping_add(w[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(s[i]));
        }
        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }
    [a0, b0, c0, d0].iter().flat_map(|v| v.to_le_bytes()).map(|b| format!("{b:02x}")).collect()
}

fn pad_be(data: &[u8]) -> Vec<u8> {
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    msg
}

fn sha1(data: &[u8]) -> String {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    for chunk in pad_be(data).chunks(64) {
        let mut w: Vec<u32> = chunk.chunks(4).map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]])).collect();
        for i in 16..80 {
            let v = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
            w.push(v);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i / 20 {
                0 => ((b & c) | (!b & d), 0x5A827999),
                1 => (b ^ c ^ d, 0x6ED9EBA1),
                2 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6u32),
            };
            let t = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(*wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = t;
        }
        for (x, v) in h.iter_mut().zip([a, b, c, d, e]) {
            *x = x.wrapping_add(v);
        }
    }
    h.iter().map(|v| format!("{v:08x}")).collect()
}

fn first_primes(k: usize) -> Vec<u64> {
    crate::numtheory::sieve(400).into_iter().take(k).collect()
}

fn sha256(data: &[u8]) -> String {
    let frac32 = |x: f64| ((x - x.floor()) * 4294967296.0).floor() as u32;
    let k: Vec<u32> = first_primes(64).iter().map(|p| frac32((*p as f64).cbrt())).collect();
    let mut h: Vec<u32> = first_primes(8).iter().map(|p| frac32((*p as f64).sqrt())).collect();
    for chunk in pad_be(data).chunks(64) {
        let mut w: Vec<u32> = chunk.chunks(4).map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]])).collect();
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            let v = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
            w.push(v);
        }
        let mut v: Vec<u32> = h.clone();
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = vec![t1.wrapping_add(t2), v[0], v[1], v[2], v[3].wrapping_add(t1), v[4], v[5], v[6]];
        }
        for (x, y) in h.iter_mut().zip(v) {
            *x = x.wrapping_add(y);
        }
    }
    h.iter().map(|v| format!("{v:08x}")).collect()
}

fn original<'a>(q: &'a Query, lowered: &'a str) -> &'a str {
    let c = q.clean.trim();
    match q.lower.trim().rfind(lowered) {
        Some(at) if c.is_char_boundary(at) && at + lowered.len() <= c.len() && c.is_char_boundary(at + lowered.len()) => &c[at..at + lowered.len()],
        _ => lowered,
    }
}

fn unquote(s: &str) -> &str {
    s.trim().trim_matches(|c| c == '"' || c == '\'' || c == '“' || c == '”')
}

fn encodings(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    if let Some(m) = re!(r"^(md5|sha-?1|sha-?256|sha256|hash)\s*(?:hash\s+)?(?:of|for)?\s+(.+)$").captures(l) {
        let t = unquote(original(q, &m[2]));
        let d = t.as_bytes();
        let (title, v) = match m[1].replace('-', "").as_str() {
            "md5" => ("MD5", md5(d)),
            "sha1" => ("SHA-1", sha1(d)),
            _ => ("SHA-256", sha256(d)),
        };
        return Some(simple(
            &format!("{title} hash of \"{t}\""),
            title,
            v,
            vec![pod("All", "Other hashes", table(None, vec![vec![text("MD5"), text(md5(d))], vec![text("SHA-1"), text(sha1(d))], vec![text("SHA-256"), text(sha256(d))]])), pod("Note", "Note", text("Hashes of the UTF-8 bytes of the text, without a trailing newline."))],
        ));
    }
    if let Some(m) = re!(r"^(?:base ?64\s+(encode|decode)|(encode|decode)\s+(?:in\s+|as\s+|from\s+)?base ?64)\s*:?\s+(.+)$").captures(l) {
        let mode = m.get(1).or(m.get(2))?.as_str();
        let t = unquote(original(q, &m[3]));
        let (title, v) = if mode == "encode" {
            ("Base64", base64_encode(t.as_bytes()))
        } else {
            ("Decoded text", String::from_utf8(base64_decode(t)?).ok()?)
        };
        return Some(simple(&format!("base64 {mode} \"{t}\""), title, v, vec![]));
    }
    if let Some(m) = re!(r#"^(?:convert\s+)?(?:the\s+(?:text|word|string)\s+)?["“']?([a-z][a-z !?.,'-]*?)["”']?\s+(?:in|to|into|as)\s+(binary|hex|hexadecimal|ascii codes?|ascii|octal)$"#).captures(l) {
        let t = unquote(original(q, &m[1]));
        if t.chars().all(|c| c.is_ascii_hexdigit()) && m[2].starts_with("hex") {
            return None;
        }
        let bytes = t.as_bytes();
        let (title, v) = match &m[2] {
            "binary" => ("Binary (8-bit ASCII)", bytes.iter().map(|b| format!("{b:08b}")).collect::<Vec<_>>().join(" ")),
            "octal" => ("Octal", bytes.iter().map(|b| format!("{b:03o}")).collect::<Vec<_>>().join(" ")),
            w if w.starts_with("hex") => ("Hexadecimal", bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")),
            _ => ("ASCII codes", bytes.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(" ")),
        };
        return Some(simple(&format!("\"{t}\" in {}", &m[2]), title, v, vec![pod("Bytes", "Length", text(format!("{} bytes", bytes.len())))]));
    }
    if let Some(m) = re!(r"^([01]{8}(?:\s+[01]{8})+|[01]{16,})\s+(?:to|in|as)\s+(?:text|ascii|string)$").captures(l) {
        let bits: String = m[1].chars().filter(|c| !c.is_whitespace()).collect();
        if bits.len() % 8 != 0 {
            return None;
        }
        let bytes: Option<Vec<u8>> = (0..bits.len()).step_by(8).map(|k| u8::from_str_radix(&bits[k..k + 8], 2).ok()).collect();
        let t = String::from_utf8(bytes?).ok()?;
        return Some(simple("binary to text", "Text", t, vec![]));
    }
    None
}

// ---------------------------------------------------------------------------
// physics and electronics
// ---------------------------------------------------------------------------

/// (colour, wavelength range nm, frequency range THz)
const SPECTRUM: &[(&str, f64, f64)] = &[("violet", 380.0, 450.0), ("blue", 450.0, 495.0), ("green", 495.0, 570.0), ("yellow", 570.0, 590.0), ("orange", 590.0, 620.0), ("red", 620.0, 750.0)];

fn si_prefix(v: f64, unit: &str) -> String {
    let a = v.abs();
    let (f, p) = if a == 0.0 {
        (1.0, "")
    } else if a >= 1e9 {
        (1e9, "G")
    } else if a >= 1e6 {
        (1e6, "M")
    } else if a >= 1e3 {
        (1e3, "k")
    } else if a >= 1.0 {
        (1.0, "")
    } else if a >= 1e-3 {
        (1e-3, "m")
    } else if a >= 1e-6 {
        (1e-6, "µ")
    } else if a >= 1e-9 {
        (1e-9, "n")
    } else if a >= 1e-12 {
        (1e-12, "p")
    } else if a >= 1e-15 {
        (1e-15, "f")
    } else {
        (1e-18, "a")
    };
    format!("{} {p}{unit}", n(v / f))
}

/// A charge such as "1 C", "5 µC", "2 nC" or "3 coulombs" in coulombs.
fn charge(t: &str) -> Option<f64> {
    let m = re!(r"^\s*(-?\d+(?:\.\d+)?(?:e-?\d+)?)\s*(m|u|µ|μ|n|p|milli|micro|nano|pico)?\s*(?:c|coulombs?)\s*$").captures(t)?;
    let v: f64 = m[1].parse().ok()?;
    let f = match m.get(2).map(|g| g.as_str()) {
        Some("m" | "milli") => 1e-3,
        Some("u" | "µ" | "μ" | "micro") => 1e-6,
        Some("n" | "nano") => 1e-9,
        Some("p" | "pico") => 1e-12,
        _ => 1.0,
    };
    Some(v * f)
}

fn quantity(t: &str) -> Option<f64> {
    let (v, u, _) = super::units::parse_quantity(t)?;
    Some(super::units::to_si(v, &u))
}

fn physics(l: &str, c: &str) -> Option<Answer> {
    const C: f64 = 299_792_458.0;
    const H: f64 = 6.626_070_15e-34;
    // visible light
    if let Some(m) = re!(r"^(?:the\s+)?(wavelength|frequency|color|colour|energy)\s+(?:range\s+)?of\s+(violet|blue|green|yellow|orange|red|visible)\s+light$").captures(l) {
        if &m[2] == "visible" {
            let rows: Vec<Vec<Content>> = SPECTRUM.iter().map(|(name, a, b)| vec![text(*name), text(format!("{a}–{b} nm")), text(format!("{}–{} THz", n(C / b / 1e3), n(C / a / 1e3)))]).collect();
            return Some(simple("visible light", "Wavelength range", "about 380–750 nm".into(), vec![pod("Table", "Colours", table(Some(vec!["colour", "wavelength", "frequency"]), rows))]));
        }
        let (_, a, b) = SPECTRUM.iter().find(|(name, ..)| *name == &m[2])?;
        let mid = (a + b) / 2.0;
        let rows = vec![
            vec![text("wavelength"), text(format!("{a}–{b} nm"))],
            vec![text("frequency"), text(format!("{}–{} THz", n(C / b / 1e3), n(C / a / 1e3)))],
            vec![text("photon energy"), text(format!("{}–{} eV", n(H * C / (b * 1e-9) / 1.602_176_634e-19), n(H * C / (a * 1e-9) / 1.602_176_634e-19)))],
        ];
        let value = match &m[1] {
            "frequency" => format!("about {}–{} THz", n(C / b / 1e3), n(C / a / 1e3)),
            "energy" => format!("about {}–{} eV per photon", n(H * C / (b * 1e-9) / 1.602_176_634e-19), n(H * C / (a * 1e-9) / 1.602_176_634e-19)),
            _ => format!("about {a}–{b} nm (typical {} nm)", mid.round()),
        };
        return Some(simple(&format!("{} light", &m[2]), "Result", value, vec![pod("Table", "Properties", table(None, rows))]));
    }
    // mass-energy
    if let Some(m) = re!(r"^(?:the\s+)?(?:rest\s+)?energy\s+(?:equivalent\s+)?(?:of|in|contained in)\s+(.+?)(?:\s+of\s+(?:mass|matter))?$").captures(l) {
        let m_kg = quantity(&m[1]).filter(|_| super::units::parse_quantity(&m[1]).is_some_and(|(_, u, _)| u.dim == [0, 1, 0, 0, 0, 0, 0, 0]))?;
        let e = m_kg * C * C;
        return Some(simple(
            &format!("rest energy of {}", m[1].trim()),
            "Energy (E = m c²)",
            format!("{} J", n(e)),
            vec![pod("Other", "In other units", table(None, vec![vec![text("kilowatt hours"), text(n(e / 3.6e6))], vec![text("megatons of TNT"), text(n(e / 4.184e15))], vec![text("electronvolts"), text(n(e / 1.602_176_634e-19))]]))],
        ));
    }
    // boiling point of water at an altitude: standard atmosphere pressure, then
    // the IAPWS-IF97 saturation curve
    if let Some(m) = re!(r"^(?:the\s+)?boiling (?:point|temperature) of water\s+at\s+(?:an?\s+)?(?:altitude|elevation|height)?\s*(?:of\s+)?(.+?)(?:\s+(?:altitude|elevation|above sea level))?$").captures(l) {
        let h = quantity(m[1].trim())?;
        if !(-500.0..=11_000.0).contains(&h) {
            return None;
        }
        let p = 101_325.0 * (1.0 - 2.255_77e-5 * h).powf(5.255_88);
        let t = saturation_temperature(p) - 273.15;
        return Some(simple(
            &format!("boiling point of water at {} m", n(h)),
            "Boiling point",
            format!("{} °C  ({} °F)", fmt_float(t, 4), fmt_float(t * 9.0 / 5.0 + 32.0, 4)),
            vec![pod("Pressure", "Air pressure (standard atmosphere)", text(format!("{} kPa  ({} atm)", fmt_float(p / 1000.0, 4), fmt_float(p / 101_325.0, 4)))), pod("Method", "Method", text("International Standard Atmosphere for the pressure; IAPWS-IF97 saturation temperature of water."))],
        ));
    }
    // two equal charges
    if let Some(m) = re!(r"^(?:the\s+)?(?:electric\s+|electrostatic\s+|coulomb\s+)?force\s+between\s+two\s+(.+?)\s+charges\s+(.+?)\s+apart$").captures(l) {
        let q = charge(&m[1])?;
        let r = quantity(&m[2])?;
        let f = 8.987_551_792_3e9 * q * q / (r * r);
        return Some(simple(&format!("Coulomb force between two {} charges {} apart", m[1].trim(), m[2].trim()), "Force", format!("{} N (repulsive)", n(f)), vec![pod("Formula", "Formula", text("F = k q₁ q₂ / r²,  k = 8.9875517923×10⁹ N m²/C²"))]));
    }
    // resistors, capacitors and inductors in series or parallel
    if let Some(m) = re!(r"^(?:(?:total|equivalent)\s+)?(resistance|capacitance|inductance)?\s*(?:of\s+)?(.+?)\s+in\s+(series|parallel)$").captures(l) {
        let parts: Vec<&str> = re!(r"\s*(?:,|\band\b|\+|\|\|)\s*").split(m[2].trim()).filter(|p| !p.is_empty()).collect();
        if parts.len() < 2 {
            return None;
        }
        let vals: Option<Vec<(f64, [i8; 8])>> = parts.iter().map(|p| {
            let p = p.replace("ohms", "Ω").replace("ohm", "Ω");
            let (v, u, _) = super::units::parse_quantity(&p)?;
            Some((super::units::to_si(v, &u), u.dim))
        }).collect();
        let vals = vals?;
        let dim = vals[0].1;
        if vals.iter().any(|v| v.1 != dim) {
            return None;
        }
        let (kind, unit) = match dim {
            [2, 1, -3, -2, 0, 0, 0, 0] => ("resistance", "Ω"),
            [-2, -1, 4, 2, 0, 0, 0, 0] => ("capacitance", "F"),
            [2, 1, -2, -2, 0, 0, 0, 0] => ("inductance", "H"),
            _ => return None,
        };
        let series_sum = vals.iter().map(|v| v.0).sum::<f64>();
        let recip_sum = 1.0 / vals.iter().map(|v| 1.0 / v.0).sum::<f64>();
        let parallel = &m[3] == "parallel";
        // capacitors add in parallel and combine reciprocally in series
        let total = if (kind == "capacitance") != parallel { recip_sum } else { series_sum };
        let rule = if (kind == "capacitance") != parallel { "reciprocals add: 1/X = 1/X₁ + 1/X₂ + …" } else { "values add: X = X₁ + X₂ + …" };
        return Some(simple(
            &format!("equivalent {kind} of {} in {}", parts.join(", "), &m[3]),
            &format!("Equivalent {kind}"),
            si_prefix(total, unit),
            vec![pod("Rule", "Rule", text(rule)), pod("Other", if parallel { "In series instead" } else { "In parallel instead" }, text(si_prefix(if total == series_sum { recip_sum } else { series_sum }, unit)))],
        ));
    }
    // energy stored in a capacitor or inductor
    if let Some(m) = re!(r"^(?:(?:the\s+)?energy\s+(?:stored\s+)?(?:in\s+)?(?:a\s+)?(capacitor|inductor)|(capacitor|inductor|capacitance|inductance)\s+energy)\s*(?:of|with|at|:)?\s*(.+)$").captures(l) {
        let kind = m.get(1).or(m.get(2))?.as_str();
        let rest = c.get(c.len().saturating_sub(m[3].len())..).unwrap_or(&m[3]).to_string();
        let qs: Vec<(f64, [i8; 8])> = re!(r"(\d+(?:\.\d+)?(?:e-?\d+)?)\s*([a-zA-Zµμ]+)").captures_iter(&rest).filter_map(|t| {
            let u = super::units::parse_unit(&t[2].replace('u', "µ").replace("µF", "µF"))?;
            Some((super::units::to_si(t[1].parse().ok()?, &u), u.dim))
        }).collect();
        let find = |d: [i8; 8]| qs.iter().find(|q| q.1 == d).map(|q| q.0);
        if kind.starts_with("cap") {
            let (cap, v) = (find([-2, -1, 4, 2, 0, 0, 0, 0])?, find([2, 1, -3, -1, 0, 0, 0, 0])?);
            let e = 0.5 * cap * v * v;
            return Some(simple("energy stored in a capacitor", "Energy", si_prefix(e, "J"), vec![pod("Charge", "Charge", text(si_prefix(cap * v, "C"))), pod("Formula", "Formula", text("E = ½ C V²"))]));
        }
        let (ind, i) = (find([2, 1, -2, -2, 0, 0, 0, 0])?, find([0, 0, 0, 1, 0, 0, 0, 0])?);
        return Some(simple("energy stored in an inductor", "Energy", si_prefix(0.5 * ind * i * i, "J"), vec![pod("Formula", "Formula", text("E = ½ L I²"))]));
    }
    // de Broglie wavelength of a named particle
    // matched on the original text: unit symbols are case-sensitive (MeV vs meV)
    if let Some(m) = re!(r"(?i)^(?:the\s+)?de ?broglie wavelength\s+(?:of\s+)?(?:an?\s+)?(electron|proton|neutron|alpha particle)\s+(?:at|moving at|with (?:a )?(?:speed|energy|kinetic energy)(?: of)?|traveling at|travelling at)\s+(.+)$").captures(c) {
        let mass = match m[1].to_lowercase().as_str() { "electron" => 9.109_383_7015e-31, "proton" => 1.672_621_923_69e-27, "neutron" => 1.674_927_498_04e-27, _ => 6.644_657_3357e-27 };
        let (qv, qu, _) = super::units::parse_quantity(&m[2])?;
        let si = super::units::to_si(qv, &qu);
        const SPEED: [i8; 8] = [1, 0, -1, 0, 0, 0, 0, 0];
        const ENERGY: [i8; 8] = [2, 1, -2, 0, 0, 0, 0, 0];
        let (p, formula, extra) = if qu.dim == SPEED {
            let gamma = 1.0 / (1.0 - (si / C).powi(2)).sqrt();
            if !gamma.is_finite() {
                return None;
            }
            (gamma * mass * si, if gamma > 1.001 { "λ = h / (γ m v)  (relativistic momentum)" } else { "λ = h / (m v)" }, format!("speed {} m/s", n(si)))
        } else if qu.dim == ENERGY {
            // kinetic energy K: p c = √(K² + 2 K m c²)
            let mc2 = mass * C * C;
            ((si * si + 2.0 * si * mc2).sqrt() / C, "λ = h / p,  p c = √(K² + 2 K m c²)  (K = kinetic energy)", format!("kinetic energy {} J", n(si)))
        } else {
            return None;
        };
        let lam = H / p;
        return Some(simple(&format!("de Broglie wavelength of an {} at {}", &m[1], m[2].trim()), "Wavelength", si_prefix(lam, "m"), vec![pod("Momentum", "Momentum", text(format!("{} kg m/s", n(p)))), pod("Given", "Given", text(extra)), pod("Formula", "Formula", text(formula))]));
    }
    None
}

// ---------------------------------------------------------------------------
// polygons and special right triangles
// ---------------------------------------------------------------------------

const POLY_NAMES: &[(&str, u32)] = &[("triangle", 3), ("quadrilateral", 4), ("square", 4), ("rectangle", 4), ("pentagon", 5), ("hexagon", 6), ("heptagon", 7), ("octagon", 8), ("nonagon", 9), ("decagon", 10), ("hendecagon", 11), ("dodecagon", 12)];

fn polygons(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:the\s+)?(?:angle sum|sum of (?:the\s+)?(?:interior\s+)?angles|interior angle sum|sum of interior angles|sum of exterior angles|exterior angle sum|number of diagonals|diagonals)\s+(?:of|in)\s+(?:a|an|the)?\s*(?:regular\s+)?(?:polygon with\s+(\d+)\s+sides|(\d+)[- ]sided polygon|(\d+)[- ]gon|([a-z]+))$").captures(l) {
        let sides: u32 = match (m.get(1).or(m.get(2)).or(m.get(3)), m.get(4)) {
            (Some(k), _) => k.as_str().parse().ok()?,
            (_, Some(name)) => POLY_NAMES.iter().find(|(nm, _)| *nm == name.as_str())?.1,
            _ => return None,
        };
        if sides < 3 {
            return None;
        }
        let sum = 180 * (sides - 2);
        let diag = sides * (sides - 3) / 2;
        let what = if l.contains("diagonal") { "Diagonals" } else if l.contains("exterior") { "Exterior angle sum" } else { "Interior angle sum" };
        let value = match what {
            "Diagonals" => diag.to_string(),
            "Exterior angle sum" => "360°".to_string(),
            _ => format!("{sum}°"),
        };
        return Some(simple(
            &format!("{} of a {sides}-sided polygon", what.to_lowercase()),
            what,
            value,
            vec![pod("Facts", "Polygon facts", table(None, vec![vec![text("interior angle sum"), text(format!("(n − 2)·180° = {sum}°"))], vec![text("each interior angle (regular)"), text(format!("{}°", n(sum as f64 / sides as f64)))], vec![text("each exterior angle (regular)"), text(format!("{}°", n(360.0 / sides as f64)))], vec![text("number of diagonals"), text(format!("n(n − 3)/2 = {diag}"))]]))],
        ));
    }
    if let Some(m) = re!(r"^(30-60-90|45-45-90)\s+(?:right\s+)?triangle\s+(?:with\s+)?(?:a\s+|the\s+)?(hypotenuse|short leg|shorter leg|long leg|longer leg|leg|side)\s*(?:of|=|is)?\s*([\d.]+)$").captures(l) {
        let v: f64 = m[3].parse().ok()?;
        let s3 = 3f64.sqrt();
        let (rows, area): (Vec<(String, f64)>, f64) = if &m[1] == "30-60-90" {
            let short = match &m[2] {
                "hypotenuse" => v / 2.0,
                "long leg" | "longer leg" => v / s3,
                _ => v,
            };
            (vec![("short leg (opposite 30°)".into(), short), ("long leg (opposite 60°)".into(), short * s3), ("hypotenuse".into(), 2.0 * short)], short * short * s3 / 2.0)
        } else {
            let leg = if &m[2] == "hypotenuse" { v / 2f64.sqrt() } else { v };
            (vec![("each leg".into(), leg), ("hypotenuse".into(), leg * 2f64.sqrt())], leg * leg / 2.0)
        };
        let ratio = if &m[1] == "30-60-90" { "sides in the ratio 1 : √3 : 2" } else { "sides in the ratio 1 : 1 : √2" };
        let summary = rows.iter().map(|(k, x)| format!("{k} {}", n(*x))).collect::<Vec<_>>().join(", ");
        let mut table_rows: Vec<Vec<Content>> = rows.iter().map(|(k, x)| vec![text(k.clone()), text(n(*x))]).collect();
        table_rows.push(vec![text("area"), text(n(area))]);
        table_rows.push(vec![text("perimeter"), text(n(rows.iter().map(|r| r.1).sum::<f64>() + if rows.len() == 2 { rows[0].1 } else { 0.0 }))]);
        return Some(simple(&format!("{} triangle with {} {v}", &m[1], &m[2]), "Sides", summary, vec![pod("Table", "Measurements", table(None, table_rows)), pod("Ratio", "Rule", text(ratio))]));
    }
    None
}

// ---------------------------------------------------------------------------
// nutrition
// ---------------------------------------------------------------------------

/// (names, serving, grams, kcal, protein g, carbohydrate g, fat g, fibre g, sugars g)
type Food = (&'static [&'static str], &'static str, f64, f64, f64, f64, f64, f64, f64);

/// Approximate values per typical serving, after USDA FoodData Central.
const FOODS: &[Food] = &[
    (&["banana", "bananas"], "1 medium banana", 118.0, 105.0, 1.3, 27.0, 0.4, 3.1, 14.4),
    (&["apple", "apples"], "1 medium apple", 182.0, 95.0, 0.5, 25.0, 0.3, 4.4, 19.0),
    (&["orange", "oranges"], "1 medium orange", 131.0, 62.0, 1.2, 15.4, 0.2, 3.1, 12.2),
    (&["egg", "eggs", "large egg"], "1 large egg", 50.0, 72.0, 6.3, 0.4, 4.8, 0.0, 0.2),
    (&["slice of bread", "bread", "white bread"], "1 slice white bread", 25.0, 67.0, 2.0, 12.7, 0.8, 0.6, 1.4),
    (&["milk", "cup of milk", "glass of milk", "whole milk"], "1 cup whole milk", 244.0, 149.0, 7.7, 11.7, 7.9, 0.0, 12.3),
    (&["rice", "cup of rice", "white rice", "cooked rice"], "1 cup cooked white rice", 158.0, 205.0, 4.3, 44.5, 0.4, 0.6, 0.1),
    (&["avocado", "avocados"], "1 avocado", 201.0, 322.0, 4.0, 17.1, 29.5, 13.5, 1.3),
    (&["chicken breast", "chicken"], "100 g cooked chicken breast", 100.0, 165.0, 31.0, 0.0, 3.6, 0.0, 0.0),
    (&["salmon"], "100 g cooked salmon", 100.0, 206.0, 22.0, 0.0, 12.4, 0.0, 0.0),
    (&["slice of pizza", "pizza"], "1 slice cheese pizza", 107.0, 285.0, 12.2, 35.7, 10.4, 2.5, 3.8),
    (&["can of coke", "coke", "cola", "can of soda", "soda"], "1 can (355 mL) cola", 368.0, 140.0, 0.0, 39.0, 0.0, 0.0, 39.0),
    (&["coffee", "cup of coffee", "black coffee"], "1 cup brewed black coffee", 237.0, 2.0, 0.3, 0.0, 0.0, 0.0, 0.0),
    (&["olive oil", "tablespoon of olive oil"], "1 tablespoon olive oil", 13.5, 119.0, 0.0, 0.0, 13.5, 0.0, 0.0),
    (&["butter", "tablespoon of butter"], "1 tablespoon butter", 14.2, 102.0, 0.1, 0.0, 11.5, 0.0, 0.0),
    (&["sugar", "tablespoon of sugar"], "1 tablespoon sugar", 12.5, 48.0, 0.0, 12.5, 0.0, 0.0, 12.5),
    (&["potato", "baked potato", "potatoes"], "1 medium baked potato", 173.0, 161.0, 4.3, 36.6, 0.2, 3.8, 2.0),
    (&["carrot", "carrots"], "1 medium carrot", 61.0, 25.0, 0.6, 5.8, 0.1, 1.7, 2.9),
    (&["broccoli"], "1 cup chopped raw broccoli", 91.0, 31.0, 2.5, 6.0, 0.3, 2.4, 1.5),
    (&["strawberries", "strawberry"], "1 cup strawberries", 152.0, 49.0, 1.0, 11.7, 0.5, 3.0, 7.4),
    (&["grapes"], "1 cup grapes", 151.0, 104.0, 1.1, 27.3, 0.2, 1.4, 23.4),
    (&["almonds"], "1 ounce (28 g) almonds", 28.0, 164.0, 6.0, 6.1, 14.2, 3.5, 1.2),
    (&["peanut butter"], "2 tablespoons peanut butter", 32.0, 188.0, 8.0, 6.3, 16.0, 1.9, 3.0),
    (&["oatmeal", "oats", "porridge"], "1 cup cooked oatmeal", 234.0, 166.0, 5.9, 28.1, 3.6, 4.0, 0.6),
    (&["pasta", "spaghetti"], "1 cup cooked pasta", 140.0, 221.0, 8.1, 43.2, 1.3, 2.5, 0.8),
    (&["cheddar", "cheddar cheese", "cheese", "slice of cheese"], "1 ounce (28 g) cheddar", 28.0, 113.0, 7.0, 0.4, 9.3, 0.0, 0.1),
    (&["yogurt", "yoghurt", "plain yogurt"], "1 cup plain whole-milk yogurt", 245.0, 149.0, 8.5, 11.4, 8.0, 0.0, 11.4),
    (&["glass of wine", "wine", "red wine"], "1 glass (148 mL) red wine", 148.0, 125.0, 0.1, 3.8, 0.0, 0.0, 0.9),
    (&["beer", "can of beer"], "1 can (355 mL) beer", 356.0, 153.0, 1.6, 12.6, 0.0, 0.0, 0.0),
    (&["chocolate", "chocolate bar", "milk chocolate"], "1 bar (44 g) milk chocolate", 44.0, 235.0, 3.4, 26.0, 13.0, 1.5, 23.0),
    (&["orange juice", "glass of orange juice"], "1 cup orange juice", 248.0, 112.0, 1.7, 25.8, 0.5, 0.5, 20.8),
    (&["tomato", "tomatoes"], "1 medium tomato", 123.0, 22.0, 1.1, 4.8, 0.2, 1.5, 3.2),
    (&["black beans", "beans"], "1 cup cooked black beans", 172.0, 227.0, 15.2, 40.8, 0.9, 15.0, 0.6),
    (&["tofu"], "100 g firm tofu", 100.0, 144.0, 17.3, 2.8, 8.7, 2.3, 0.6),
    (&["french fries", "fries", "chips"], "1 medium serving french fries", 117.0, 365.0, 4.0, 48.0, 17.0, 4.4, 0.3),
    (&["honey", "tablespoon of honey"], "1 tablespoon honey", 21.0, 64.0, 0.1, 17.3, 0.0, 0.0, 17.2),
    (&["blueberries"], "1 cup blueberries", 148.0, 84.0, 1.1, 21.4, 0.5, 3.6, 14.7),
    (&["spinach"], "1 cup raw spinach", 30.0, 7.0, 0.9, 1.1, 0.1, 0.7, 0.1),
    (&["walnuts"], "1 ounce (28 g) walnuts", 28.0, 185.0, 4.3, 3.9, 18.5, 1.9, 0.7),
    (&["bacon", "slice of bacon"], "1 slice cooked bacon", 8.0, 43.0, 3.0, 0.1, 3.3, 0.0, 0.0),
];

fn nutrition(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:how many\s+|amount of\s+|number of\s+)?(calories|kcal|protein|carbs|carbohydrates|fat|fiber|fibre|sugar|sugars|nutrition(?: facts)?|nutritional (?:value|information)|nutrients)\s+(?:are\s+|is\s+)?(?:there\s+)?(?:in|of|for)\s+(?:(?:an|a|one|1)\s+)?(?:(\d+(?:\.\d+)?)\s*(?:g|grams?)\s+(?:of\s+)?)?(.+?)$").captures(l)?;
    let what = m[1].to_string();
    let food = m[3].trim().trim_start_matches("a ").trim_start_matches("an ");
    let f = FOODS.iter().find(|f| f.0.contains(&food))?;
    let scale = m.get(2).and_then(|g| g.as_str().parse::<f64>().ok()).map_or(1.0, |g| g / f.2);
    let serving = if scale == 1.0 { format!("{} ({} g)", f.1, n(f.2)) } else { format!("{} g", n(f.2 * scale)) };
    let v = |x: f64| x * scale;
    let rows = vec![
        vec![text("calories"), text(format!("{} kcal ({} kJ)", n(v(f.3).round()), n((v(f.3) * 4.184).round())))],
        vec![text("protein"), text(format!("{} g", n(v(f.4))))],
        vec![text("carbohydrate"), text(format!("{} g", n(v(f.5))))],
        vec![text("  of which sugars"), text(format!("{} g", n(v(f.8))))],
        vec![text("  dietary fibre"), text(format!("{} g", n(v(f.7))))],
        vec![text("fat"), text(format!("{} g", n(v(f.6))))],
    ];
    let (title, value) = match what.as_str() {
        "protein" => ("Protein", format!("{} g", n(v(f.4)))),
        "carbs" | "carbohydrates" => ("Carbohydrate", format!("{} g", n(v(f.5)))),
        "fat" => ("Fat", format!("{} g", n(v(f.6)))),
        "fiber" | "fibre" => ("Dietary fibre", format!("{} g", n(v(f.7)))),
        "sugar" | "sugars" => ("Sugars", format!("{} g", n(v(f.8)))),
        "calories" | "kcal" => ("Calories", format!("{} kcal", n(v(f.3).round()))),
        _ => ("Nutrition facts", format!("{} kcal, {} g protein, {} g carbohydrate, {} g fat", n(v(f.3).round()), n(v(f.4)), n(v(f.5)), n(v(f.6)))),
    };
    Some(simple(
        &format!("{} | {serving}", f.0[0]),
        title,
        value,
        vec![pod("Facts", &format!("Nutrition facts per {serving}"), table(None, rows)), pod("Note", "Note", text("Approximate values after USDA FoodData Central; actual foods vary with size, variety and preparation."))],
    ))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    chemistry(q)
        .or_else(|| nutrition(l))
        .or_else(|| sun(l))
        .or_else(|| holidays(l))
        .or_else(|| facts(l))
        .or_else(|| countries_q(l))
        .or_else(|| finance(l))
        .or_else(|| color(l))
        .or_else(|| encodings(q))
        .or_else(|| physics(l, q.clean.trim()))
        .or_else(|| polygons(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_match_reference_digests() {
        assert_eq!(md5(b"hello"), "5d41402abc4b2a76b9719d911017c592");
        assert_eq!(md5(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(sha1(b"hello"), "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
        assert_eq!(sha256(b"hello"), "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
        assert_eq!(sha256(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(base64_decode("aGVsbG8=").unwrap(), b"hello");
    }

    #[test]
    fn easter_dates() {
        assert_eq!(easter(2024), NaiveDate::from_ymd_opt(2024, 3, 31));
        assert_eq!(easter(2025), NaiveDate::from_ymd_opt(2025, 4, 20));
        assert_eq!(easter(2026), NaiveDate::from_ymd_opt(2026, 4, 5));
    }
}

/// Saturation temperature of water (K) at pressure `p` (Pa): the IAPWS-IF97
/// region 4 backward equation, valid from 611 Pa to 22.064 MPa.
pub fn saturation_temperature(p: f64) -> f64 {
    const N: [f64; 10] = [0.116_705_214_527_67e4, -0.724_213_167_032_06e6, -0.170_738_469_400_92e2, 0.120_208_247_024_70e5, -0.323_255_503_223_33e7, 0.149_151_086_135_30e2, -0.482_326_573_615_91e4, 0.405_113_405_420_57e6, -0.238_555_575_678_49, 0.650_175_348_447_98e3];
    let b = (p / 1e6).powf(0.25);
    let e = b * b + N[2] * b + N[5];
    let f = N[0] * b * b + N[3] * b + N[6];
    let g = N[1] * b * b + N[4] * b + N[7];
    let d = 2.0 * g / (-f - (f * f - 4.0 * e * g).sqrt());
    (N[9] + d - ((N[9] + d).powi(2) - 4.0 * (N[8] + N[9] * d)).sqrt()) / 2.0
}

#[cfg(test)]
mod iapws {
    #[test]
    fn saturation_temperature_matches_the_standard() {
        // IAPWS-IF97 verification values: 0.1 MPa → 372.755919 K, 1 MPa → 453.035632 K
        assert!((super::saturation_temperature(1e5) - 372.755_919).abs() < 1e-5);
        assert!((super::saturation_temperature(1e6) - 453.035_632).abs() < 1e-5);
    }
}

