//! Geography: countries, cities, distances and local times.
//! Data: GeoNames (CC BY 4.0) via geonamescache.

use crate::engine::*;
use crate::print::fmt_float;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Deserialize, Clone, Debug)]
pub struct Country {
    pub name: String,
    pub iso2: String,
    pub iso3: String,
    pub capital: String,
    pub continent: String,
    pub area_km2: f64,
    /// land area (CIA World Factbook), when known
    #[serde(default)]
    pub land_km2: Option<f64>,
    pub population: u64,
    pub tld: String,
    pub currency_code: String,
    pub currency_name: String,
    pub calling_code: String,
    pub languages: String,
    pub neighbours: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct City {
    pub name: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    pub population: u64,
    pub timezone: String,
    pub capital: bool,
    /// year of the population figure when it comes from Wikidata, else empty (GeoNames)
    pub population_year: String,
    /// first-level region: a US state abbreviation, else a GeoNames admin1 code
    pub admin1: String,
}

pub fn countries() -> &'static HashMap<String, Country> {
    static C: OnceLock<HashMap<String, Country>> = OnceLock::new();
    C.get_or_init(|| serde_json::from_str(include_str!("../../data/countries.json")).unwrap_or_default())
}

fn city_population(c: &City) -> String {
    if c.population_year.is_empty() {
        format!("{} people", fmt_int(c.population))
    } else {
        format!("{} people ({} figure)", fmt_int(c.population), c.population_year)
    }
}

pub fn cities() -> &'static Vec<City> {
    static C: OnceLock<Vec<City>> = OnceLock::new();
    C.get_or_init(|| {
        #[derive(Deserialize)]
        struct Raw {
            rows: Vec<Vec<serde_json::Value>>,
        }
        let raw: Raw = serde_json::from_str(include_str!("../../data/cities.json")).unwrap_or(Raw { rows: vec![] });
        // name, country, lat, lon, population, timezone, capital, admin1[, population year]
        let s = |r: &[serde_json::Value], k: usize| r.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let f = |r: &[serde_json::Value], k: usize| r.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
        raw.rows
            .iter()
            .map(|r| City { name: s(r, 0), country: s(r, 1), lat: f(r, 2), lon: f(r, 3), population: f(r, 4) as u64, timezone: s(r, 5), capital: f(r, 6) == 1.0, population_year: s(r, 8), admin1: s(r, 7) })
            .collect()
    })
}

const ALIASES: &[(&str, &str)] = &[
    ("usa", "US"),
    ("us", "US"),
    ("united states", "US"),
    ("america", "US"),
    ("united states of america", "US"),
    ("uk", "GB"),
    ("britain", "GB"),
    ("great britain", "GB"),
    ("england", "GB"),
    ("russia", "RU"),
    ("south korea", "KR"),
    ("north korea", "KP"),
    ("vietnam", "VN"),
    ("iran", "IR"),
    ("syria", "SY"),
    ("laos", "LA"),
    ("bolivia", "BO"),
    ("venezuela", "VE"),
    ("tanzania", "TZ"),
    ("czech republic", "CZ"),
    ("czechia", "CZ"),
    ("holland", "NL"),
    ("the netherlands", "NL"),
    ("ivory coast", "CI"),
    ("uae", "AE"),
    ("taiwan", "TW"),
    ("moldova", "MD"),
    ("macedonia", "MK"),
];

pub fn find_country(q: &str) -> Option<&'static Country> {
    let l = q.trim().to_lowercase();
    let l = l.trim_start_matches("the ");
    if let Some((_, code)) = ALIASES.iter().find(|(a, _)| *a == l) {
        return countries().get(*code);
    }
    countries().values().find(|c| {
        let name = c.name.to_lowercase();
        name == l || name.trim_start_matches("the ") == l || (l.len() == 3 && c.iso3.to_lowercase() == l)
    })
}

/// US states and DC: (postal abbreviation, name).
pub const US_STATES: &[(&str, &str)] = &[("AL", "Alabama"), ("AK", "Alaska"), ("AZ", "Arizona"), ("AR", "Arkansas"), ("CA", "California"), ("CO", "Colorado"), ("CT", "Connecticut"), ("DE", "Delaware"), ("DC", "District of Columbia"), ("FL", "Florida"), ("GA", "Georgia"), ("HI", "Hawaii"), ("ID", "Idaho"), ("IL", "Illinois"), ("IN", "Indiana"), ("IA", "Iowa"), ("KS", "Kansas"), ("KY", "Kentucky"), ("LA", "Louisiana"), ("ME", "Maine"), ("MD", "Maryland"), ("MA", "Massachusetts"), ("MI", "Michigan"), ("MN", "Minnesota"), ("MS", "Mississippi"), ("MO", "Missouri"), ("MT", "Montana"), ("NE", "Nebraska"), ("NV", "Nevada"), ("NH", "New Hampshire"), ("NJ", "New Jersey"), ("NM", "New Mexico"), ("NY", "New York"), ("NC", "North Carolina"), ("ND", "North Dakota"), ("OH", "Ohio"), ("OK", "Oklahoma"), ("OR", "Oregon"), ("PA", "Pennsylvania"), ("RI", "Rhode Island"), ("SC", "South Carolina"), ("SD", "South Dakota"), ("TN", "Tennessee"), ("TX", "Texas"), ("UT", "Utah"), ("VT", "Vermont"), ("VA", "Virginia"), ("WA", "Washington"), ("WV", "West Virginia"), ("WI", "Wisconsin"), ("WY", "Wyoming")];

/// The postal abbreviation of a US state written as a name or an abbreviation.
pub fn us_state(s: &str) -> Option<&'static str> {
    let s = s.trim().trim_end_matches('.');
    US_STATES.iter().find(|(a, n)| a.eq_ignore_ascii_case(s) || n.eq_ignore_ascii_case(s)).map(|(a, _)| *a)
}

/// Lower-case Latin letters without diacritics ("zürich" → "zurich").
pub fn fold_accents(s: &str) -> String {
    s.chars().map(|c| match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
        'ç' | 'ć' | 'č' => 'c',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => 'e',
        'ì' | 'í' | 'î' | 'ï' | 'ī' => 'i',
        'ñ' | 'ń' | 'ň' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ő' => 'o',
        'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' => 'u',
        'ý' | 'ÿ' => 'y',
        'ś' | 'š' | 'ş' => 's',
        'ź' | 'ż' | 'ž' => 'z',
        'ł' => 'l',
        'ř' => 'r',
        'ţ' | 'ť' => 't',
        'ḩ' => 'h',
        other => other,
    }).collect()
}

/// The user's place for questions that name none: the city in NOESIS_LOCATION, else London.
/// (name, latitude, longitude, time zone)
pub fn home() -> (String, f64, f64, String) {
    let c = std::env::var("NOESIS_LOCATION").ok().and_then(|s| find_city(s.trim())).or_else(|| find_city("London"));
    match c {
        Some(c) => (format!("{}, {}", c.name, countries().get(&c.country).map_or(c.country.clone(), |x| x.name.clone())), c.lat, c.lon, c.timezone.clone()),
        None => ("London, United Kingdom".into(), 51.5074, -0.1278, "Europe/London".into()),
    }
}

pub fn find_city(q: &str) -> Option<&'static City> {
    let l = q.trim().to_lowercase();
    let (name, cc) = match l.split_once(',') {
        Some((n, c)) => {
            let cc = find_country(c.trim()).map(|c| c.iso2.clone());
            // "Springfield, MA", "Portland, Oregon"
            if cc.is_none() {
                if let Some(st) = us_state(c) {
                    let n = n.trim().to_lowercase();
                    return cities().iter().filter(|x| x.country == "US" && x.admin1 == st && x.name.to_lowercase() == n).max_by_key(|x| x.population);
                }
            }
            // "Madison, WI" keeps a short region; "Phoenix of traffic" is not one
            if cc.is_none() && c.split_whitespace().count() > 2 {
                return None;
            }
            (n.trim().to_string(), cc)
        }
        None => (l.clone(), None),
    };
    let name = match name.as_str() {
        "nyc" | "new york" => "new york city".to_string(),
        "la" => "los angeles".to_string(),
        "sf" | "san fran" => "san francisco".to_string(),
        "dc" | "washington dc" | "washington d.c." => "washington".to_string(),
        "bombay" => "mumbai".to_string(),
        "peking" => "beijing".to_string(),
        "mecca" | "mekka" => "makkah".to_string(),
        "kiev" => "kyiv".to_string(),
        "cologne" => "koln".to_string(),
        "hanover" => "hannover".to_string(),
        "seville" => "sevilla".to_string(),
        "quebec city" => "quebec".to_string(),
        // nicknames
        "the windy city" | "windy city" | "chi-town" | "chitown" => "chicago".to_string(),
        "the big apple" | "big apple" => "new york city".to_string(),
        "the city of angels" | "city of angels" => "los angeles".to_string(),
        "the motor city" | "motor city" | "motown" => "detroit".to_string(),
        "the big easy" | "big easy" => "new orleans".to_string(),
        "sin city" => "las vegas".to_string(),
        "the mile high city" | "mile high city" => "denver".to_string(),
        "the emerald city" | "emerald city" => "seattle".to_string(),
        "beantown" => "boston".to_string(),
        "the eternal city" | "eternal city" => "rome".to_string(),
        "the city of light" | "city of light" | "city of lights" => "paris".to_string(),
        "the city of brotherly love" | "city of brotherly love" | "philly" => "philadelphia".to_string(),
        "saigon" => "ho chi minh city".to_string(),
        other => other.to_string(),
    };
    // most populous match (exact, then ignoring accents: "Zurich", "Sao Paulo")
    cities().iter().filter(|c| c.name.to_lowercase() == name && cc.as_ref().map_or(true, |k| &c.country == k)).max_by_key(|c| c.population).or_else(|| {
        let n = fold_accents(&name);
        cities().iter().filter(|c| fold_accents(&c.name.to_lowercase()) == n && cc.as_ref().map_or(true, |k| &c.country == k)).max_by_key(|c| c.population)
    })
}

/// Great-circle distance in km (haversine on a spherical Earth).
pub fn distance_km(a: &City, b: &City) -> f64 {
    let r = 6371.0088;
    let (p1, p2) = (a.lat.to_radians(), b.lat.to_radians());
    let dp = p2 - p1;
    let dl = (b.lon - a.lon).to_radians();
    let h = (dp / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
    2.0 * r * h.sqrt().asin()
}

fn initial_bearing(a: &City, b: &City) -> f64 {
    let (p1, p2) = (a.lat.to_radians(), b.lat.to_radians());
    let dl = (b.lon - a.lon).to_radians();
    let y = dl.sin() * p2.cos();
    let x = p1.cos() * p2.sin() - p1.sin() * p2.cos() * dl.cos();
    (y.atan2(x).to_degrees() + 360.0) % 360.0
}

fn compass(deg: f64) -> &'static str {
    const D: [&str; 16] = ["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"];
    D[(((deg + 11.25) % 360.0) / 22.5) as usize]
}

pub(crate) fn fmt_int(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (k, c) in s.chars().enumerate() {
        if k > 0 && (s.len() - k) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn coords(c: &City) -> String {
    format!("{:.2}° {}, {:.2}° {}", c.lat.abs(), if c.lat >= 0.0 { "N" } else { "S" }, c.lon.abs(), if c.lon >= 0.0 { "E" } else { "W" })
}

pub fn local_time(tz: &str) -> Option<chrono::DateTime<chrono_tz::Tz>> {
    let tz: chrono_tz::Tz = tz.parse().ok()?;
    Some(chrono::Utc::now().with_timezone(&tz))
}

impl Country {
    /// Area used for population density: land area when known.
    pub fn density_area(&self) -> f64 {
        self.land_km2.unwrap_or(self.area_km2).max(1e-9)
    }
}

pub(crate) fn country_pods(c: &Country) -> Vec<Pod> {
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} (country)", c.name)))];
    let density = c.population as f64 / c.density_area();
    pods.push(primary(pod(
        "Facts",
        "Basic facts",
        table(
            None,
            vec![
                vec![text("capital city"), text(c.capital.clone())],
                vec![text("continent"), text(continent(&c.continent))],
                vec![text("population"), text(format!("{} people", fmt_int(c.population)))],
                vec![text("area"), text(format!("{} km² ({} mi²)", fmt_int(c.area_km2 as u64), fmt_int((c.area_km2 / 2.589988) as u64)))],
                vec![text("land area"), text(c.land_km2.map(|a| format!("{} km²", fmt_int(a as u64))).unwrap_or_else(|| "not available".into()))],
                vec![text("population density"), text(format!("{} people/km² of {}", fmt_float(density, 4), if c.land_km2.is_some() { "land" } else { "total area" }))],
                vec![text("currency"), text(format!("{} ({})", c.currency_name, c.currency_code))],
                vec![text("calling code"), text(format!("+{}", c.calling_code))],
                vec![text("internet code"), text(c.tld.clone())],
                vec![text("ISO codes"), text(format!("{} / {}", c.iso2, c.iso3))],
            ],
        ),
    )));
    if !c.neighbours.is_empty() {
        let names: Vec<String> = c.neighbours.iter().filter_map(|n| countries().get(n)).map(|n| n.name.clone()).collect();
        pods.push(pod("Borders", "Bordering countries", text(names.join(" | "))));
    }
    if !c.languages.is_empty() {
        pods.push(pod("Languages", "Language codes", text(c.languages.replace(',', ", "))));
    }
    let mut big: Vec<&City> = cities().iter().filter(|x| x.country == c.iso2).collect();
    big.sort_by(|a, b| b.population.cmp(&a.population));
    if !big.is_empty() {
        let rows: Vec<Vec<Content>> = big.iter().take(6).map(|x| vec![text(x.name.clone()), text(fmt_int(x.population))]).collect();
        pods.push(pod("Cities", "Largest cities", table(Some(vec!["city", "population"]), rows)));
        if let Some(t) = local_time(&big[0].timezone) {
            pods.push(pod("Time", &format!("Local time ({})", big[0].name), text(t.format("%-I:%M %p %Z, %A, %B %-d, %Y").to_string())));
        }
    }
    pods
}

fn continent(code: &str) -> String {
    match code {
        "AF" => "Africa",
        "AS" => "Asia",
        "EU" => "Europe",
        "NA" => "North America",
        "SA" => "South America",
        "OC" => "Oceania",
        "AN" => "Antarctica",
        other => other,
    }
    .to_string()
}

fn city_pods(c: &City) -> Vec<Pod> {
    let country = countries().get(&c.country).map(|x| x.name.clone()).unwrap_or_else(|| c.country.clone());
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{}, {} (city)", c.name, country)))];
    let mut rows = vec![vec![text("country"), text(country)], vec![text("population"), text(city_population(c))], vec![text("coordinates"), text(coords(c))], vec![text("time zone"), text(c.timezone.clone())]];
    if c.capital {
        rows.insert(1, vec![text("status"), text("national capital")]);
    }
    pods.push(primary(pod("Facts", "Basic information", table(None, rows))));
    if let Some(t) = local_time(&c.timezone) {
        pods.push(pod("Time", "Current local time", text(t.format("%-I:%M:%S %p %Z, %A, %B %-d, %Y").to_string())));
    }
    let mut near: Vec<(&City, f64)> = cities().iter().filter(|x| x.name != c.name && x.population > 300_000).map(|x| (x, distance_km(c, x))).filter(|(_, d)| *d < 600.0).collect();
    near.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    if !near.is_empty() {
        let rows: Vec<Vec<Content>> = near.iter().take(5).map(|(x, d)| vec![text(x.name.clone()), text(format!("{} km", fmt_float(*d, 4)))]).collect();
        pods.push(pod("Nearby", "Nearby cities", table(None, rows)));
    }
    pods
}

/// "New York, Chicago, Denver", "London, Istanbul, Sydney time zones"
fn city_list(l: &str) -> Option<Answer> {
    let (body, what) = match re_suffix().captures(l) {
        Some(m) => (m.get(1)?.as_str(), m.get(2).map(|x| x.as_str().to_string())),
        None => (l, None),
    };
    let parts: Vec<&str> = body.split(',').flat_map(|p| p.split(" and ")).flat_map(|p| p.split(" vs ")).map(str::trim).filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return None;
    }
    let list: Vec<&City> = parts.iter().map(|p| find_city(p).filter(|c| c.population >= 50_000)).collect::<Option<_>>()?;
    let mut header = vec!["".to_string()];
    header.extend(list.iter().map(|c| c.name.clone()));
    let country = |c: &City| countries().get(&c.country).map(|x| x.name.clone()).unwrap_or_else(|| c.country.clone());
    let mut rows: Vec<Vec<Content>> = vec![
        std::iter::once(text("country")).chain(list.iter().map(|c| text(country(c)))).collect(),
        std::iter::once(text("population")).chain(list.iter().map(|c| text(city_population(c)))).collect(),
        std::iter::once(text("coordinates")).chain(list.iter().map(|c| text(coords(c)))).collect(),
        std::iter::once(text("time zone")).chain(list.iter().map(|c| text(c.timezone.clone()))).collect(),
    ];
    rows.push(std::iter::once(text("local time")).chain(list.iter().map(|c| text(local_time(&c.timezone).map_or(String::new(), |t| t.format("%-I:%M %p %Z").to_string())))).collect());
    let key = match what.as_deref() {
        Some(w) if w.contains("time") => 4,
        Some(w) if w.contains("coordinate") || w.contains("location") => 2,
        _ => 1,
    };
    let names = list.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(", ");
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("{names} (cities)"))),
        primary(pod("Result", "Comparison", table(Some(header.iter().map(|s| s.as_str()).collect()), rows.clone()))),
        pod("Key", content_text(&rows[key][0]).as_str(), text(list.iter().zip(&rows[key][1..]).map(|(c, v)| format!("{}: {}", c.name, content_text(v))).collect::<Vec<_>>().join("  |  "))),
    ];
    Some(Answer::new(format!("{names} (cities)"), pods))
}

fn re_suffix() -> &'static regex::Regex {
    static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    R.get_or_init(|| regex::Regex::new(r"^(.+?)\s+(time zones?|local times?|times|populations?|coordinates|locations?)$").unwrap())
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    if let Some(a) = city_list(l) {
        return Some(a);
    }
    // distance between cities
    let dist_re = ["distance from ", "distance between ", "how far is ", "how far from "];
    for p in dist_re {
        if let Some(rest) = l.strip_prefix(p) {
            let (a, b) = rest.split_once(" to ").or_else(|| rest.split_once(" and ")).or_else(|| rest.split_once(" from "))?;
            let (ca, cb) = (find_city(a)?, find_city(b)?);
            let d = distance_km(ca, cb);
            let mut pods = vec![pod("Input", "Input interpretation", text(format!("distance from {} to {}", ca.name, cb.name)))];
            pods.push(primary(pod("Result", "Result", text(format!("{} km  ({} miles)", fmt_float(d, 5), fmt_float(d / 1.609344, 5))))));
            let b = initial_bearing(ca, cb);
            pods.push(pod("Direction", "Direction", text(format!("{} ({:.0}° bearing from {})", compass(b), b, ca.name))));
            pods.push(pod(
                "TravelTimes",
                "Approximate travel times",
                table(None, vec![vec![text("aircraft (800 km/h)"), text(fmt_hours(d / 800.0))], vec![text("car (100 km/h)"), text(fmt_hours(d / 100.0))], vec![text("light in fiber (≈200,000 km/s)"), text(format!("{} ms", fmt_float(d / 200.0, 4)))]]),
            ));
            pods.push(pod("Locations", "Locations", table(None, vec![vec![text(ca.name.clone()), text(coords(ca))], vec![text(cb.name.clone()), text(coords(cb))]])));
            return Some(Answer::new(format!("distance {} to {}", ca.name, cb.name), pods));
        }
    }
    // time difference between two places
    if let Some(rest) = l.strip_prefix("time difference between ").or_else(|| l.strip_prefix("time difference from ")).or_else(|| l.strip_prefix("time zone difference between ")) {
        let (a, b) = rest.split_once(" and ").or_else(|| rest.split_once(" to "))?;
        let zone = |p: &str| -> Option<(String, String)> {
            if let Some(c) = find_city(p) {
                return Some((c.name.clone(), c.timezone.clone()));
            }
            let c = find_country(p)?;
            let big = cities().iter().filter(|x| x.country == c.iso2 && x.capital).max_by_key(|x| x.population).or_else(|| cities().iter().filter(|x| x.country == c.iso2).max_by_key(|x| x.population))?;
            Some((c.name.clone(), big.timezone.clone()))
        };
        let ((na, za), (nb, zb)) = (zone(a)?, zone(b)?);
        use chrono::Offset;
        let (ta, tb) = (local_time(&za)?, local_time(&zb)?);
        let (oa, ob) = (ta.offset().fix().local_minus_utc(), tb.offset().fix().local_minus_utc());
        let diff = ob - oa;
        let hm = |s: i32| if s % 3600 == 0 { format!("{} hour{}", s.abs() / 3600, if s.abs() == 3600 { "" } else { "s" }) } else { format!("{} h {} min", s.abs() / 3600, s.abs() % 3600 / 60) };
        let result = match diff {
            0 => format!("{na} and {nb} are on the same time right now"),
            d if d > 0 => format!("{nb} is {} ahead of {na}", hm(d)),
            d => format!("{nb} is {} behind {na}", hm(d)),
        };
        let pods = vec![
            pod("Input", "Input interpretation", text(format!("time difference between {na} and {nb}"))),
            primary(pod("Result", "Result", text(result))),
            pod("Clocks", "Current local times", table(None, vec![vec![text(na.clone()), text(ta.format("%-I:%M %p %Z (UTC%:z)").to_string())], vec![text(nb.clone()), text(tb.format("%-I:%M %p %Z (UTC%:z)").to_string())]])),
            pod("Note", "Note", text("Differences can change during the year when only one of the places observes daylight saving time.")),
        ];
        return Some(Answer::new(format!("time difference {na} – {nb}"), pods));
    }
    // time in a place
    for p in ["time in ", "current time in ", "what time is it in ", "local time in "] {
        if let Some(rest) = l.strip_prefix(p) {
            let (name, tz) = if let Some(c) = find_city(rest) {
                (c.name.clone(), c.timezone.clone())
            } else if let Some(c) = find_country(rest) {
                let big = cities().iter().filter(|x| x.country == c.iso2).max_by_key(|x| x.population)?;
                (c.name.clone(), big.timezone.clone())
            } else {
                return None;
            };
            let t = local_time(&tz)?;
            let pods = vec![pod("Input", "Input interpretation", text(format!("current time in {name}"))), primary(pod("Result", "Result", text(t.format("%-I:%M:%S %p %Z, %A, %B %-d, %Y").to_string()))), pod("TimeZone", "Time zone", text(format!("{tz} (UTC{})", t.format("%:z"))))];
            return Some(Answer::new(format!("time in {name}"), pods));
        }
    }
    // population / capital / area of X
    if let Some((prop, place)) = l.split_once(" of ") {
        let prop = prop.trim_start_matches("the ");
        if let Some(c) = find_country(place) {
            let v = match prop {
                "population" => Some(format!("{} people", fmt_int(c.population))),
                "capital" | "capital city" => Some(c.capital.clone()),
                "area" | "size" | "total area" => Some(format!("{} km²", fmt_int(c.area_km2 as u64))),
                "land area" => c.land_km2.map(|a| format!("{} km²", fmt_int(a as u64))),
                "currency" => Some(format!("{} ({})", c.currency_name, c.currency_code)),
                "calling code" | "country code" | "dialing code" => Some(format!("+{}", c.calling_code)),
                _ => None,
            };
            if let Some(v) = v {
                let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} | {prop}", c.name))), primary(pod("Result", "Result", text(v)))];
                pods.extend(country_pods(c).into_iter().skip(1).map(|mut p| {
                    p.primary = false;
                    p
                }));
                return Some(Answer::new(format!("{prop} of {}", c.name), pods));
            }
        }
        if prop == "population" {
            if let Some(c) = find_city(place) {
                let pods = vec![pod("Input", "Input interpretation", text(format!("{} | population", c.name))), primary(pod("Result", "Result", text(city_population(c))))];
                return Some(Answer::new(format!("population of {}", c.name), pods));
            }
        }
    }
    if let Some(c) = find_country(l) {
        return Some(Answer::new(format!("{} (country)", c.name), country_pods(c)));
    }
    if l.chars().all(|c| c.is_alphabetic() || c == ' ' || c == ',' || c == '.' || c == '-') && l.len() >= 3 {
        if let Some(c) = find_city(l) {
            if c.population >= 50_000 {
                return Some(Answer::new(format!("{} (city)", c.name), city_pods(c)));
            }
        }
    }
    None
}

fn fmt_hours(h: f64) -> String {
    let total = (h * 60.0).round() as i64;
    let (hh, mm) = (total / 60, total % 60);
    if hh == 0 {
        format!("{mm} min")
    } else {
        format!("{hh} h {mm} min")
    }
}
