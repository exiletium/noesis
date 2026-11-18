//! US states: capital, largest city, 2020 census population, area,
//! admission to the Union, nickname, state bird and flower, and state tax
//! rates.
//!
//! "Texas", "capital of Vermont", "Texas state flower", "New Mexico's state
//! bird", "when did Hawaii become a state", "population of California",
//! "illinois income tax vs. virginia income tax vs. oregon income tax",
//! "sales tax Washington state", "largest states by area".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

pub struct State {
    pub abbr: &'static str,
    pub name: &'static str,
    capital: &'static str,
    largest: &'static str,
    /// 2020 census resident population
    pub population: u64,
    /// total area, square miles
    area_mi2: f64,
    /// admission date (ISO) and order; the 13 original states ratified the Constitution on that date
    admitted: &'static str,
    order: u32,
    nickname: &'static str,
    bird: &'static str,
    flower: &'static str,
    /// top marginal individual income-tax rate, % (2025 tax year)
    income_tax: &'static str,
    /// statewide sales tax, %
    sales_tax: f64,
}

macro_rules! st {
    ($a:expr, $n:expr, $c:expr, $l:expr, $p:expr, $ar:expr, $d:expr, $o:expr, $nick:expr, $b:expr, $f:expr, $it:expr, $s:expr) => {
        State { abbr: $a, name: $n, capital: $c, largest: $l, population: $p, area_mi2: $ar, admitted: $d, order: $o, nickname: $nick, bird: $b, flower: $f, income_tax: $it, sales_tax: $s }
    };
}

pub const STATES: &[State] = &[
    st!("AL", "Alabama", "Montgomery", "Huntsville", 5_024_279, 52_420.0, "1819-12-14", 22, "Yellowhammer State", "yellowhammer (northern flicker)", "camellia", "5% (over $3,000)", 4.0),
    st!("AK", "Alaska", "Juneau", "Anchorage", 733_391, 665_384.0, "1959-01-03", 49, "The Last Frontier", "willow ptarmigan", "forget-me-not", "none", 0.0),
    st!("AZ", "Arizona", "Phoenix", "Phoenix", 7_151_502, 113_990.0, "1912-02-14", 48, "Grand Canyon State", "cactus wren", "saguaro cactus blossom", "2.5% (flat)", 5.6),
    st!("AR", "Arkansas", "Little Rock", "Little Rock", 3_011_524, 53_179.0, "1836-06-15", 25, "The Natural State", "northern mockingbird", "apple blossom", "3.9%", 6.5),
    st!("CA", "California", "Sacramento", "Los Angeles", 39_538_223, 163_695.0, "1850-09-09", 31, "Golden State", "California quail", "California poppy", "13.3% (12.3% plus 1% on income over $1 million)", 7.25),
    st!("CO", "Colorado", "Denver", "Denver", 5_773_714, 104_094.0, "1876-08-01", 38, "Centennial State", "lark bunting", "Rocky Mountain columbine", "4.4% (flat)", 2.9),
    st!("CT", "Connecticut", "Hartford", "Bridgeport", 3_605_944, 5_543.0, "1788-01-09", 5, "Constitution State", "American robin", "mountain laurel", "6.99%", 6.35),
    st!("DE", "Delaware", "Dover", "Wilmington", 989_948, 2_489.0, "1787-12-07", 1, "The First State", "Delaware Blue Hen", "peach blossom", "6.6%", 0.0),
    st!("FL", "Florida", "Tallahassee", "Jacksonville", 21_538_187, 65_758.0, "1845-03-03", 27, "Sunshine State", "northern mockingbird", "orange blossom", "none", 6.0),
    st!("GA", "Georgia", "Atlanta", "Atlanta", 10_711_908, 59_425.0, "1788-01-02", 4, "Peach State", "brown thrasher", "Cherokee rose", "5.19% (flat)", 4.0),
    st!("HI", "Hawaii", "Honolulu", "Honolulu", 1_455_271, 10_932.0, "1959-08-21", 50, "Aloha State", "nene (Hawaiian goose)", "yellow hibiscus (pua aloalo)", "11%", 4.0),
    st!("ID", "Idaho", "Boise", "Boise", 1_839_106, 83_569.0, "1890-07-03", 43, "Gem State", "mountain bluebird", "syringa (mock orange)", "5.3% (flat)", 6.0),
    st!("IL", "Illinois", "Springfield", "Chicago", 12_812_508, 57_914.0, "1818-12-03", 21, "Prairie State (Land of Lincoln)", "northern cardinal", "violet", "4.95% (flat)", 6.25),
    st!("IN", "Indiana", "Indianapolis", "Indianapolis", 6_785_528, 36_420.0, "1816-12-11", 19, "Hoosier State", "northern cardinal", "peony", "3% (flat; counties add local income tax)", 7.0),
    st!("IA", "Iowa", "Des Moines", "Des Moines", 3_190_369, 56_273.0, "1846-12-28", 29, "Hawkeye State", "eastern goldfinch", "wild prairie rose", "3.8% (flat)", 6.0),
    st!("KS", "Kansas", "Topeka", "Wichita", 2_937_880, 82_278.0, "1861-01-29", 34, "Sunflower State", "western meadowlark", "sunflower", "5.58%", 6.5),
    st!("KY", "Kentucky", "Frankfort", "Louisville", 4_505_836, 40_408.0, "1792-06-01", 15, "Bluegrass State", "northern cardinal", "goldenrod", "4% (flat)", 6.0),
    st!("LA", "Louisiana", "Baton Rouge", "New Orleans", 4_657_757, 52_378.0, "1812-04-30", 18, "Pelican State", "brown pelican", "magnolia", "3% (flat)", 5.0),
    st!("ME", "Maine", "Augusta", "Portland", 1_362_359, 35_380.0, "1820-03-15", 23, "Pine Tree State", "black-capped chickadee", "white pine cone and tassel", "7.15%", 5.5),
    st!("MD", "Maryland", "Annapolis", "Baltimore", 6_177_224, 12_406.0, "1788-04-28", 7, "Old Line State", "Baltimore oriole", "black-eyed Susan", "6.5% (on income over $1 million; counties add local income tax)", 6.0),
    st!("MA", "Massachusetts", "Boston", "Boston", 7_029_917, 10_554.0, "1788-02-06", 6, "Bay State", "black-capped chickadee", "mayflower (trailing arbutus)", "9% (5% plus a 4% surtax on income over about $1 million)", 6.25),
    st!("MI", "Michigan", "Lansing", "Detroit", 10_077_331, 96_714.0, "1837-01-26", 26, "Great Lakes State (Wolverine State)", "American robin", "apple blossom", "4.25% (flat)", 6.0),
    st!("MN", "Minnesota", "Saint Paul", "Minneapolis", 5_706_494, 86_936.0, "1858-05-11", 32, "North Star State (Land of 10,000 Lakes)", "common loon", "pink and white lady's slipper", "9.85%", 6.875),
    st!("MS", "Mississippi", "Jackson", "Jackson", 2_961_279, 48_432.0, "1817-12-10", 20, "Magnolia State", "northern mockingbird", "magnolia", "4.4% (flat, above $10,000)", 7.0),
    st!("MO", "Missouri", "Jefferson City", "Kansas City", 6_154_913, 69_707.0, "1821-08-10", 24, "Show-Me State", "eastern bluebird", "hawthorn", "4.7%", 4.225),
    st!("MT", "Montana", "Helena", "Billings", 1_084_225, 147_040.0, "1889-11-08", 41, "Treasure State", "western meadowlark", "bitterroot", "5.9%", 0.0),
    st!("NE", "Nebraska", "Lincoln", "Omaha", 1_961_504, 77_348.0, "1867-03-01", 37, "Cornhusker State", "western meadowlark", "goldenrod", "5.2%", 5.5),
    st!("NV", "Nevada", "Carson City", "Las Vegas", 3_104_614, 110_572.0, "1864-10-31", 36, "Silver State", "mountain bluebird", "sagebrush", "none", 6.85),
    st!("NH", "New Hampshire", "Concord", "Manchester", 1_377_529, 9_349.0, "1788-06-21", 9, "Granite State", "purple finch", "purple lilac", "none (the tax on interest and dividends ended in 2025)", 0.0),
    st!("NJ", "New Jersey", "Trenton", "Newark", 9_288_994, 8_723.0, "1787-12-18", 3, "Garden State", "eastern goldfinch", "common meadow violet", "10.75%", 6.625),
    st!("NM", "New Mexico", "Santa Fe", "Albuquerque", 2_117_522, 121_590.0, "1912-01-06", 47, "Land of Enchantment", "greater roadrunner", "yucca flower", "5.9%", 4.875),
    st!("NY", "New York", "Albany", "New York City", 20_201_249, 54_555.0, "1788-07-26", 11, "Empire State", "eastern bluebird", "rose", "10.9% (New York City adds up to 3.876%)", 4.0),
    st!("NC", "North Carolina", "Raleigh", "Charlotte", 10_439_388, 53_819.0, "1789-11-21", 12, "Tar Heel State", "northern cardinal", "flowering dogwood", "4.25% (flat)", 4.75),
    st!("ND", "North Dakota", "Bismarck", "Fargo", 779_094, 70_698.0, "1889-11-02", 39, "Peace Garden State", "western meadowlark", "wild prairie rose", "2.5%", 5.0),
    st!("OH", "Ohio", "Columbus", "Columbus", 11_799_448, 44_826.0, "1803-03-01", 17, "Buckeye State", "northern cardinal", "red carnation", "3.5%", 5.75),
    st!("OK", "Oklahoma", "Oklahoma City", "Oklahoma City", 3_959_353, 69_899.0, "1907-11-16", 46, "Sooner State", "scissor-tailed flycatcher", "Oklahoma rose (wildflower: Indian blanket)", "4.75%", 4.5),
    st!("OR", "Oregon", "Salem", "Portland", 4_237_256, 98_379.0, "1859-02-14", 33, "Beaver State", "western meadowlark", "Oregon grape", "9.9%", 0.0),
    st!("PA", "Pennsylvania", "Harrisburg", "Philadelphia", 13_002_700, 46_054.0, "1787-12-12", 2, "Keystone State", "ruffed grouse", "mountain laurel", "3.07% (flat)", 6.0),
    st!("RI", "Rhode Island", "Providence", "Providence", 1_097_379, 1_545.0, "1790-05-29", 13, "Ocean State", "Rhode Island Red (chicken)", "violet", "5.99%", 7.0),
    st!("SC", "South Carolina", "Columbia", "Charleston", 5_118_425, 32_020.0, "1788-05-23", 8, "Palmetto State", "Carolina wren", "yellow jessamine", "6.2%", 6.0),
    st!("SD", "South Dakota", "Pierre", "Sioux Falls", 886_667, 77_116.0, "1889-11-02", 40, "Mount Rushmore State", "ring-necked pheasant", "American pasque flower", "none", 4.2),
    st!("TN", "Tennessee", "Nashville", "Nashville", 6_910_840, 42_144.0, "1796-06-01", 16, "Volunteer State", "northern mockingbird", "iris", "none", 7.0),
    st!("TX", "Texas", "Austin", "Houston", 29_145_505, 268_596.0, "1845-12-29", 28, "Lone Star State", "northern mockingbird", "bluebonnet", "none", 6.25),
    st!("UT", "Utah", "Salt Lake City", "Salt Lake City", 3_271_616, 84_897.0, "1896-01-04", 45, "Beehive State", "California gull", "sego lily", "4.5% (flat)", 6.1),
    st!("VT", "Vermont", "Montpelier", "Burlington", 643_077, 9_616.0, "1791-03-04", 14, "Green Mountain State", "hermit thrush", "red clover", "8.75%", 6.0),
    st!("VA", "Virginia", "Richmond", "Virginia Beach", 8_631_393, 42_775.0, "1788-06-25", 10, "Old Dominion", "northern cardinal", "American dogwood", "5.75% (on income over $17,000)", 5.3),
    st!("WA", "Washington", "Olympia", "Seattle", 7_705_281, 71_298.0, "1889-11-11", 42, "Evergreen State", "American goldfinch (willow goldfinch)", "coast rhododendron", "none on wages (7% on long-term capital gains over about $270,000)", 6.5),
    st!("WV", "West Virginia", "Charleston", "Charleston", 1_793_716, 24_230.0, "1863-06-20", 35, "Mountain State", "northern cardinal", "rhododendron", "4.82%", 6.0),
    st!("WI", "Wisconsin", "Madison", "Milwaukee", 5_893_718, 65_496.0, "1848-05-29", 30, "Badger State", "American robin", "wood violet", "7.65%", 5.0),
    st!("WY", "Wyoming", "Cheyenne", "Cheyenne", 576_851, 97_813.0, "1890-07-10", 44, "Equality State (Cowboy State)", "western meadowlark", "Indian paintbrush", "none", 4.0),
];

fn group(n: u64) -> String {
    super::geo::fmt_int(n)
}

fn date(iso: &str) -> String {
    const M: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
    let p: Vec<&str> = iso.split('-').collect();
    let m: usize = p[1].parse().unwrap_or(1);
    format!("{} {}, {}", M[m - 1], p[2].trim_start_matches('0'), p[0])
}

fn ordinal(n: u32) -> String {
    let suf = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suf}")
}

/// A state named in `s`: "Texas", "texas state", "state of New York", "NM",
/// "New Mexico's". Ambiguous names (Georgia, Washington, New York) need a
/// "state" qualifier unless `loose`.
fn find(s: &str, loose: bool) -> Option<&'static State> {
    let s = s.trim().trim_end_matches("'s").trim_end_matches("’s").trim();
    let low = s.to_lowercase();
    let (core, qualified) = if let Some(x) = low.strip_prefix("state of ").or_else(|| low.strip_prefix("the state of ")) {
        (x.to_string(), true)
    } else if let Some(x) = low.strip_suffix(" state") {
        (x.to_string(), true)
    } else {
        (low.clone(), false)
    };
    let core = core.trim().trim_start_matches("u.s. ").trim_start_matches("us ");
    let st = STATES.iter().find(|x| x.name.to_lowercase() == core).or_else(|| {
        // a postal abbreviation only in capitals, as written
        // ("CO" alone is carbon monoxide, "PA" a pascal)
        (loose && s.len() == 2 && s.chars().all(|c| c.is_ascii_uppercase())).then(|| STATES.iter().find(|x| x.abbr == s)).flatten()
    })?;
    if !qualified && !loose && ["Georgia", "Washington", "New York"].contains(&st.name) {
        return None;
    }
    Some(st)
}

fn prop(p: &str) -> Option<&'static str> {
    Some(match p.trim().trim_start_matches("state ").trim() {
        "capital" | "capital city" => "capital",
        "largest city" | "biggest city" | "most populous city" => "largest city",
        "population" | "people" | "how many people" => "population",
        "area" | "size" | "total area" | "land area" => "area",
        "admission" | "admission date" | "statehood" | "date of statehood" | "admission to the union" => "admitted",
        "nickname" | "nicknames" => "nickname",
        "bird" => "bird",
        "flower" => "flower",
        "income tax" | "income tax rate" | "state income tax" | "top income tax rate" => "income tax",
        "sales tax" | "sales tax rate" | "state sales tax" => "sales tax",
        "abbreviation" | "postal code" | "postal abbreviation" => "abbreviation",
        "population density" | "density" => "density",
        _ => return None,
    })
}

fn rows(s: &State) -> Vec<(&'static str, String)> {
    vec![
        ("abbreviation", s.abbr.to_string()),
        ("capital", s.capital.to_string()),
        ("largest city", s.largest.to_string()),
        ("population", format!("{} people (2020 census)", group(s.population))),
        ("area", format!("{} mi² ({} km²)", group(s.area_mi2 as u64), group((s.area_mi2 * 2.589_988).round() as u64))),
        ("density", format!("{:.1} people/mi²", s.population as f64 / s.area_mi2)),
        ("admitted", format!("{} ({} state)", date(s.admitted), ordinal(s.order))),
        ("nickname", s.nickname.to_string()),
        ("bird", s.bird.to_string()),
        ("flower", s.flower.to_string()),
        ("income tax", format!("{} top rate (2025)", s.income_tax)),
        ("sales tax", if s.sales_tax == 0.0 { "none statewide".to_string() } else { format!("{}% statewide (local taxes extra)", crate::print::fmt_float(s.sales_tax, 6)) }),
    ]
}

fn value(s: &State, p: &str) -> String {
    rows(s).into_iter().find(|(k, _)| *k == p).map(|x| x.1).unwrap_or_default()
}

fn label(p: &str) -> String {
    match p {
        "admitted" => "Admission to the Union".to_string(),
        "bird" => "State bird".to_string(),
        "flower" => "State flower".to_string(),
        p => p[..1].to_uppercase() + &p[1..],
    }
}

fn note() -> Pod {
    pod("Source", "Notes", text("populations from the 2020 census; areas are total area (land and water); tax rates are the 2025 top statutory rates and change often"))
}

fn one(s: &'static State, p: Option<&str>) -> Answer {
    let r = rows(s);
    let (title, v) = match p {
        Some(p) => (label(p), value(s, p)),
        None => ("Basic information".to_string(), format!("US state | capital {} | population {}", s.capital, group(s.population))),
    };
    Answer::new(s.name, vec![
        pod("Input", "Input interpretation", text(format!("{} (US state){}", s.name, p.map(|p| format!(" | {}", label(p).to_lowercase())).unwrap_or_default()))),
        primary(pod("Result", &title, text(v))),
        pod("Table", "Properties", table(None, r.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())),
        note(),
    ])
}

fn many(ss: Vec<&'static State>, p: Option<&str>) -> Answer {
    let keys: Vec<&str> = match p {
        Some(p) => vec![p],
        None => vec!["capital", "population", "area", "admitted"],
    };
    let mut header = vec!["state"];
    header.extend(keys.iter().copied());
    let trs = ss.iter().map(|s| {
        let mut row = vec![text(s.name)];
        for k in &keys {
            row.push(text(value(s, k)));
        }
        row
    });
    let short = match p {
        Some(p) => ss.iter().map(|s| format!("{}: {}", s.name, value(s, p))).collect::<Vec<_>>().join(" | "),
        None => ss.iter().map(|s| s.name).collect::<Vec<_>>().join(" | "),
    };
    Answer::new("US states", vec![
        pod("Input", "Input interpretation", text(format!("{}{}", ss.iter().map(|s| s.name).collect::<Vec<_>>().join(" | "), p.map(|p| format!(" | {}", label(p).to_lowercase())).unwrap_or_default()))),
        primary(pod("Result", "Comparison", text(short))),
        pod("Table", "Properties", table(Some(header), trs.collect())),
        note(),
    ])
}

fn ranking(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?(\d+ )?(largest|biggest|smallest|most populous|least populous|most populated|least populated|oldest|newest|youngest|most densely populated|least densely populated) (?:us |u\.s\. |american )?states(?: by (area|population|size))?$").captures(l)?;
    let n: usize = m.get(1).and_then(|x| x.as_str().trim().parse().ok()).unwrap_or(10);
    let by = m.get(3).map(|x| x.as_str());
    let mut v: Vec<&'static State> = STATES.iter().collect();
    let (key, desc): (&str, bool) = match (&m[2], by) {
        ("largest" | "biggest", Some("population")) => ("population", true),
        ("smallest", Some("population")) => ("population", false),
        ("largest" | "biggest", _) => ("area", true),
        ("smallest", _) => ("area", false),
        ("most populous" | "most populated", _) => ("population", true),
        ("least populous" | "least populated", _) => ("population", false),
        ("oldest", _) => ("admitted", false),
        ("newest" | "youngest", _) => ("admitted", true),
        ("most densely populated", _) => ("density", true),
        _ => ("density", false),
    };
    let f = |s: &State| match key {
        "population" => s.population as f64,
        "area" => s.area_mi2,
        "admitted" => s.order as f64,
        _ => s.population as f64 / s.area_mi2,
    };
    v.sort_by(|a, b| f(a).partial_cmp(&f(b)).unwrap_or(std::cmp::Ordering::Equal));
    if desc {
        v.reverse();
    }
    v.truncate(n.clamp(1, 50));
    let mut a = many(v, Some(key));
    a.interpretation = format!("{} US states", &m[2]);
    Some(a)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let raw = q.clean.trim().trim_end_matches(['?', '.', '!']).trim();
    let raw = re!(r"\s+").replace_all(raw, " ").to_string();
    let l = raw.to_lowercase();
    let l = l.trim_start_matches("what is the ").trim_start_matches("what's the ").trim_start_matches("what are the ").trim_start_matches("the ");
    if let Some(a) = ranking(l) {
        return Some(a);
    }
    let cut = raw.len() - l.len();
    let r = &raw[cut..];
    // "when did Hawaii become a state", "when was Texas admitted"
    if let Some(m) = re!(r"(?i)^when (?:did|was) (.+?) (?:become a state|admitted(?: to the union)?|join the union|achieve statehood)$").captures(r) {
        return find(&m[1], true).map(|s| one(s, Some("admitted")));
    }
    // "P of A (vs B)", "A('s) (state) P", "P A"
    let (p, body) = if let Some(m) = re!(r"(?i)^(?:state )?([a-z ]+?) (?:of|in|for) (.+)$").captures(r).filter(|m| prop(&m[1]).is_some()) {
        (prop(&m[1]), m[2].to_string())
    } else if let Some(m) = re!(r"(?i)^(.+?)(?:'s|’s)? (?:state )?(capital|capital city|largest city|biggest city|population|area|nickname|nicknames|bird|flower|income tax rate|income tax|sales tax rate|sales tax|abbreviation|postal code|admission date|statehood|population density)$").captures(r) {
        (prop(&m[2]), m[1].to_string())
    } else if let Some(m) = re!(r"(?i)^(sales tax|income tax|state bird|state flower|capital|population|nickname) (.+)$").captures(r) {
        (prop(&m[1]), m[2].to_string())
    } else {
        (None, r.to_string())
    };
    // "illinois income tax vs. virginia income tax vs. oregon income tax"
    let parts: Vec<String> = re!(r"(?i)\s*(?:,|\bvs\.?|\bversus\b|\band\b|\bcompared (?:to|with)\b)\s*").split(&body).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let mut p = p;
    let mut names = Vec::new();
    for part in &parts {
        // a repeated property in each part
        let (pp, nm) = match re!(r"(?i)^(.+?) (income tax|sales tax|population|area|capital)$").captures(part) {
            Some(m) if find(&m[1], true).is_some() => (prop(&m[2]), m[1].to_string()),
            _ => (None, part.clone()),
        };
        if pp.is_some() {
            if p.is_some() && p != pp {
                return None;
            }
            p = pp;
        }
        names.push(nm);
    }
    let loose = p.is_some() || names.len() > 1;
    let ss: Option<Vec<&'static State>> = names.iter().map(|n| find(n, loose)).collect();
    let ss = ss?;
    if ss.is_empty() {
        return None;
    }
    // "capital of Georgia" means the country unless "state" is said: leave it to geography
    if p == Some("capital") && ss.len() == 1 && ss[0].name == "Georgia" && !l.contains("state") {
        return None;
    }
    Some(if ss.len() == 1 { one(ss[0], p) } else { many(ss, p) })
}
