//! US demographics from the American Community Survey (2019–2023 5-year
//! estimates, US Census Bureau) for the nation, states, counties, cities,
//! metro areas and congressional districts: "black population in
//! Philadelphia", "Los Angeles vs New York naturalized citizens",
//! "US county with the highest fraction of separated women",
//! "median home value Madison, WI", "PA citizenship breakdown".

use super::geo::{us_state, US_STATES};
use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

static ACS_Z: &[u8] = include_bytes!("../../data/acs.z");

struct Geo {
    lvl: &'static str,
    st: &'static str,
    name: &'static str,
    vals: Vec<Option<f64>>,
}

struct Acs {
    vars: HashMap<&'static str, usize>,
    geos: Vec<Geo>,
    /// lower-case place name -> indices (cities, counties, metro areas)
    index: HashMap<String, Vec<usize>>,
}

fn acs() -> &'static Acs {
    static D: OnceLock<Acs> = OnceLock::new();
    D.get_or_init(|| {
        let raw = miniz_oxide::inflate::decompress_to_vec(ACS_Z).unwrap_or_default();
        let s: &'static str = Box::leak(String::from_utf8(raw).unwrap_or_default().into_boxed_str());
        let mut d = Acs { vars: HashMap::new(), geos: Vec::new(), index: HashMap::new() };
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("#V\t") {
                d.vars = v.split(',').enumerate().map(|(i, k)| (k, i)).collect();
                continue;
            }
            let f: Vec<&'static str> = line.split('\t').collect();
            if f.len() != 4 {
                continue;
            }
            let vals = f[3].split(',').map(|x| x.parse().ok()).collect();
            let i = d.geos.len();
            d.geos.push(Geo { lvl: f[0], st: f[1], name: f[2], vals });
            let base = f[2].split(", ").next().unwrap_or("").to_lowercase();
            match f[0] {
                "160" => {
                    let b = re!(r" (?:city and borough|city|town|village|borough|cdp|municipality|urban county|unified government|consolidated government|metro government|metropolitan government)(?: \(balance\))?$").replace(&base, "").to_string();
                    for k in [b.clone(), b.split(['-', '/']).next().unwrap_or("").trim().to_string()] {
                        d.index.entry(k).or_default().push(i);
                    }
                }
                "050" => d.index.entry(base).or_default().push(i),
                "310" => {
                    let first = base.split(['-', '/']).next().unwrap_or("").trim().to_string();
                    d.index.entry(format!("{first} metro")).or_default().push(i);
                }
                _ => {}
            }
        }
        d
    })
}

fn val(g: &Geo, v: &str) -> Option<f64> {
    let neg = v.starts_with('-');
    let x = (*g.vals.get(*acs().vars.get(v.trim_start_matches('-'))?)?)?;
    Some(if neg { -x } else { x })
}

fn sum(g: &Geo, vs: &[&str]) -> Option<f64> {
    vs.iter().map(|v| val(g, v)).sum()
}

fn pop(g: &Geo) -> f64 {
    val(g, "pop").unwrap_or(0.0)
}

#[derive(Clone, Copy, PartialEq)]
enum K {
    People,
    Money,
    MoneyTotal,
    Index,
    Years,
    Minutes,
    Table(&'static [(&'static str, &'static str)]),
}

struct Met {
    pat: &'static str,
    label: &'static str,
    num: &'static [&'static str],
    den: &'static str,
    kind: K,
}

const RACE: &[(&str, &str)] = &[("white", "white"), ("black", "black or African American"), ("asian", "Asian"), ("native", "American Indian or Alaska Native"), ("pacific", "Native Hawaiian or Pacific Islander"), ("other_race", "some other race"), ("multiracial", "two or more races"), ("hispanic", "Hispanic or Latino (any race)")];
const CITIZENSHIP: &[(&str, &str)] = &[("born_us", "US citizen, born in the US"), ("born_territories", "US citizen, born in Puerto Rico or US island areas"), ("born_abroad_us_parents", "US citizen, born abroad to American parents"), ("naturalized", "US citizen by naturalization"), ("noncitizen", "not a US citizen")];
const MARITAL: &[(&str, &str)] = &[("never_married_men+never_married_women", "never married"), ("married_men+married_women", "now married"), ("separated_men+separated_women", "separated"), ("widowed_men+widowed_women", "widowed"), ("divorced_men+divorced_women", "divorced")];
const EDUCATION: &[(&str, &str)] = &[("no_schooling", "no schooling completed"), ("high_school", "high school graduate or higher"), ("some_college", "some college or more"), ("bachelors_plus", "bachelor's degree or higher"), ("masters", "master's degree"), ("professional", "professional degree"), ("doctorate", "doctorate")];

const METS: &[Met] = &[
    Met { pat: r"population by race|racial (?:makeup|composition|breakdown)|races|ethnicit(?:y|ies)|ethnic (?:makeup|composition|breakdown)|race", label: "population by race", num: &[], den: "pop", kind: K::Table(RACE) },
    Met { pat: r"citizenship (?:breakdown|status)?", label: "citizenship", num: &[], den: "pop", kind: K::Table(CITIZENSHIP) },
    Met { pat: r"marital status", label: "marital status (age 15+)", num: &[], den: "marital_pop", kind: K::Table(MARITAL) },
    Met { pat: r"educational attainment|education levels?", label: "educational attainment (age 25+)", num: &[], den: "adults25", kind: K::Table(EDUCATION) },
    Met { pat: r"female senior citizens|female seniors|senior (?:citizen )?women|elderly women|women (?:over|aged) 65", label: "women aged 65 and over", num: &["female65"], den: "pop", kind: K::People },
    Met { pat: r"male senior citizens|male seniors|senior (?:citizen )?men|elderly men|men (?:over|aged) 65", label: "men aged 65 and over", num: &["male65"], den: "pop", kind: K::People },
    Met { pat: r"seniors|senior citizens|elderly(?: people)?|people (?:over|aged) 65|65 and (?:over|older)", label: "people aged 65 and over", num: &["over65"], den: "pop", kind: K::People },
    Met { pat: r"children|kids|minors|people under 18", label: "people under 18", num: &["under18"], den: "pop", kind: K::People },
    Met { pat: r"median age", label: "median age", num: &["median_age"], den: "", kind: K::Years },
    Met { pat: r"white hispanics?(?: or latinos?)?(?: people| population)?", label: "white Hispanic people", num: &["white_hispanic"], den: "pop", kind: K::People },
    Met { pat: r"black hispanics?(?: or latinos?)?(?: people| population)?", label: "black Hispanic people", num: &["black_hispanic"], den: "pop", kind: K::People },
    Met { pat: r"asian hispanics?(?: or latinos?)?(?: people| population)?", label: "Asian Hispanic people", num: &["asian_hispanic"], den: "pop", kind: K::People },
    Met { pat: r"cuban(?:s| hispanics?| americans?)?(?: people| population)?", label: "Cuban Hispanic people", num: &["cuban"], den: "pop", kind: K::People },
    Met { pat: r"puerto ricans?(?: hispanics?)?(?: people| population)?", label: "Puerto Rican Hispanic people", num: &["puerto_rican"], den: "pop", kind: K::People },
    Met { pat: r"mexican(?:s| hispanics?| americans?)?(?: people| population)?", label: "Mexican Hispanic people", num: &["mexican"], den: "pop", kind: K::People },
    Met { pat: r"dominican(?:s| hispanics?| americans?)?(?: people| population)?", label: "Dominican Hispanic people", num: &["dominican"], den: "pop", kind: K::People },
    Met { pat: r"central americans?(?: hispanics?)?(?: people| population)?", label: "Central American Hispanic people", num: &["central_american"], den: "pop", kind: K::People },
    Met { pat: r"south americans?(?: hispanics?)?(?: people| population)?", label: "South American Hispanic people", num: &["south_american"], den: "pop", kind: K::People },
    Met { pat: r"colombian(?:s| hispanics?| americans?)?(?: people| population)?", label: "Colombian Hispanic people", num: &["colombian"], den: "pop", kind: K::People },
    Met { pat: r"salvadoran(?:s| hispanics?)?(?: people| population)?", label: "Salvadoran Hispanic people", num: &["salvadoran"], den: "pop", kind: K::People },
    Met { pat: r"guatemalan(?:s| hispanics?)?(?: people| population)?", label: "Guatemalan Hispanic people", num: &["guatemalan"], den: "pop", kind: K::People },
    Met { pat: r"honduran(?:s| hispanics?)?(?: people| population)?", label: "Honduran Hispanic people", num: &["honduran"], den: "pop", kind: K::People },
    Met { pat: r"ecuadorian(?:s| hispanics?)?(?: people| population)?", label: "Ecuadorian Hispanic people", num: &["ecuadorian"], den: "pop", kind: K::People },
    Met { pat: r"peruvian(?:s| hispanics?)?(?: people| population)?", label: "Peruvian Hispanic people", num: &["peruvian"], den: "pop", kind: K::People },
    Met { pat: r"venezuelan(?:s| hispanics?)?(?: people| population)?", label: "Venezuelan Hispanic people", num: &["venezuelan"], den: "pop", kind: K::People },
    Met { pat: r"hispanics?(?: or latinos?)?(?: people| population)?|latinos?(?: people| population)?", label: "Hispanic or Latino people", num: &["hispanic"], den: "pop", kind: K::People },
    Met { pat: r"black(?: people| population|s)|african americans?(?: people| population)?", label: "black or African American people", num: &["black"], den: "pop", kind: K::People },
    Met { pat: r"asians?(?: americans?)?(?: people| population)?", label: "Asian people", num: &["asian"], den: "pop", kind: K::People },
    Met { pat: r"white(?: people| population|s)", label: "white people", num: &["white"], den: "pop", kind: K::People },
    Met { pat: r"native americans?(?: people| population)?|american indians?(?: people| population)?", label: "American Indian and Alaska Native people", num: &["native"], den: "pop", kind: K::People },
    Met { pat: r"pacific islanders?(?: people| population)?|native hawaiians?", label: "Native Hawaiian and Pacific Islander people", num: &["pacific"], den: "pop", kind: K::People },
    Met { pat: r"multiracial(?: people| population)?|(?:people of )?two or more races", label: "people of two or more races", num: &["multiracial"], den: "pop", kind: K::People },
    Met { pat: r"naturali[sz]ed(?: us)?(?: citizens| people)?", label: "naturalized US citizens", num: &["naturalized"], den: "pop", kind: K::People },
    Met { pat: r"non-? ?citizens|not citizens|people who are not citizens", label: "non-citizens", num: &["noncitizen"], den: "pop", kind: K::People },
    Met { pat: r"citizens born in (?:the )?(?:us |u\.s\. )?territories|(?:people )?born in (?:the )?(?:us |u\.s\. )?territories|born in puerto rico", label: "US citizens born in Puerto Rico or US island areas", num: &["born_territories"], den: "pop", kind: K::People },
    Met { pat: r"(?:people )?born outside (?:of )?the (?:us|united states)|foreign[- ]born(?: people| population)?|(?:people )?born abroad|immigrants", label: "people born outside the US", num: &["naturalized", "noncitizen"], den: "pop", kind: K::People },
    Met { pat: r"(?:us )?citizens", label: "US citizens", num: &["born_us", "born_territories", "born_abroad_us_parents", "naturalized"], den: "pop", kind: K::People },
    Met { pat: r"(?:people )?(?:who )?moved from abroad", label: "people who moved from abroad in the past year", num: &["moved_abroad"], den: "mobility_pop", kind: K::People },
    Met { pat: r"(?:people|population)? ?(?:who )?moved within (?:the )?same county", label: "people who moved within the same county in the past year", num: &["moved_same_county"], den: "mobility_pop", kind: K::People },
    Met { pat: r"(?:people )?(?:who )?(?:have )?(?:moved|relocated) from (?:other|another|a different) states?", label: "people who moved from another state in the past year", num: &["moved_other_state"], den: "mobility_pop", kind: K::People },
    Met { pat: r"(?:people )?(?:who )?(?:have )?stayed in the same house", label: "people living in the same house as a year ago", num: &["same_house"], den: "mobility_pop", kind: K::People },
    Met { pat: r"unmarried women|single women", label: "unmarried women (age 15+)", num: &["women15", "-married_women"], den: "women15", kind: K::People },
    Met { pat: r"unmarried men|single men", label: "unmarried men (age 15+)", num: &["men15", "-married_men"], den: "men15", kind: K::People },
    Met { pat: r"men are married|married men", label: "married men (age 15+)", num: &["married_men"], den: "men15", kind: K::People },
    Met { pat: r"women are married|married women", label: "married women (age 15+)", num: &["married_women"], den: "women15", kind: K::People },
    Met { pat: r"separated women", label: "separated women (age 15+)", num: &["separated_women"], den: "women15", kind: K::People },
    Met { pat: r"separated men", label: "separated men (age 15+)", num: &["separated_men"], den: "men15", kind: K::People },
    Met { pat: r"divorced women", label: "divorced women (age 15+)", num: &["divorced_women"], den: "women15", kind: K::People },
    Met { pat: r"divorced men", label: "divorced men (age 15+)", num: &["divorced_men"], den: "men15", kind: K::People },
    Met { pat: r"never married(?: people| population)?", label: "never-married people (age 15+)", num: &["never_married_men", "never_married_women"], den: "marital_pop", kind: K::People },
    Met { pat: r"divorced(?: people| population)?", label: "divorced people (age 15+)", num: &["divorced_men", "divorced_women"], den: "marital_pop", kind: K::People },
    Met { pat: r"widowed(?: people| population)?|widows", label: "widowed people (age 15+)", num: &["widowed_men", "widowed_women"], den: "marital_pop", kind: K::People },
    Met { pat: r"(?:people )?(?:with )?no (?:formal )?schooling", label: "adults with no schooling completed (age 25+)", num: &["no_schooling"], den: "adults25", kind: K::People },
    Met { pat: r"women with (?:phds?|doctorates?|doctoral degrees?)|female (?:phds?|doctorates?)", label: "women with a doctorate (age 25+)", num: &["doctorate_women"], den: "adults25", kind: K::People },
    Met { pat: r"men with (?:phds?|doctorates?|doctoral degrees?)|male (?:phds?|doctorates?)", label: "men with a doctorate (age 25+)", num: &["doctorate_men"], den: "adults25", kind: K::People },
    Met { pat: r"(?:people )?with (?:a )?(?:phds?|doctorates?|doctoral degrees?)|phds", label: "adults with a doctorate (age 25+)", num: &["doctorate"], den: "adults25", kind: K::People },
    Met { pat: r"(?:people )?with (?:a )?professional (?:school )?degrees?", label: "adults with a professional degree (age 25+)", num: &["professional"], den: "adults25", kind: K::People },
    Met { pat: r"(?:people )?with (?:a )?master'?s degrees?", label: "adults with a master's degree (age 25+)", num: &["masters"], den: "adults25", kind: K::People },
    Met { pat: r"(?:people )?with (?:a )?(?:bachelor'?s|college) degrees?|college graduates", label: "adults with a bachelor's degree or higher (age 25+)", num: &["bachelors_plus"], den: "adults25", kind: K::People },
    Met { pat: r"(?:people )?(?:who )?(?:attended|went to) college|(?:people )?with some college", label: "adults who attended college (age 25+)", num: &["some_college"], den: "adults25", kind: K::People },
    Met { pat: r"high school graduates", label: "high school graduates (age 25+)", num: &["high_school"], den: "adults25", kind: K::People },
    Met { pat: r"(?:people )?in (?:grad|graduate) school|graduate (?:school )?students", label: "people enrolled in graduate or professional school", num: &["grad_school"], den: "pop", kind: K::People },
    Met { pat: r"(?:undergraduate|college) students|(?:people )?in college", label: "people enrolled in college (undergraduate)", num: &["undergrad"], den: "pop", kind: K::People },
    Met { pat: r"spanish speakers|(?:people who )?speak spanish", label: "Spanish speakers (age 5+)", num: &["spanish"], den: "language_pop", kind: K::People },
    Met { pat: r"chinese speakers|(?:people who )?speak chinese", label: "Chinese speakers (age 5+)", num: &["chinese"], den: "language_pop", kind: K::People },
    Met { pat: r"french speakers|(?:people who )?speak french", label: "French, Haitian or Cajun speakers (age 5+)", num: &["french"], den: "language_pop", kind: K::People },
    Met { pat: r"german speakers|(?:people who )?speak german", label: "German or other West Germanic speakers (age 5+)", num: &["german"], den: "language_pop", kind: K::People },
    Met { pat: r"korean speakers|(?:people who )?speak korean", label: "Korean speakers (age 5+)", num: &["korean"], den: "language_pop", kind: K::People },
    Met { pat: r"vietnamese speakers|(?:people who )?speak vietnamese", label: "Vietnamese speakers (age 5+)", num: &["vietnamese"], den: "language_pop", kind: K::People },
    Met { pat: r"tagalog speakers|filipino speakers|(?:people who )?speak tagalog", label: "Tagalog speakers (age 5+)", num: &["tagalog"], den: "language_pop", kind: K::People },
    Met { pat: r"arabic speakers|(?:people who )?speak arabic", label: "Arabic speakers (age 5+)", num: &["arabic"], den: "language_pop", kind: K::People },
    Met { pat: r"(?:people who speak )?only english|english[- ]only speakers", label: "people who speak only English (age 5+)", num: &["english_only"], den: "language_pop", kind: K::People },
    Met { pat: r"(?:female|women)(?: population| people)? (?:in|below the|under the) poverty(?: line| level)?|women below the poverty line", label: "women below the poverty level", num: &["poverty_female"], den: "poverty_pop", kind: K::People },
    Met { pat: r"(?:male|men)(?: population| people)? (?:in|below the|under the) poverty(?: line| level)?", label: "men below the poverty level", num: &["poverty_male"], den: "poverty_pop", kind: K::People },
    Met { pat: r"(?:people|population)? ?(?:in poverty|below the poverty (?:line|level)|living in poverty)|poverty(?: rate| level)?", label: "people below the poverty level", num: &["poverty"], den: "poverty_pop", kind: K::People },
    Met { pat: r"median (?:household )?income", label: "median household income", num: &["median_household_income"], den: "", kind: K::Money },
    Met { pat: r"(?:average|mean|per capita) income|income per (?:capita|person)", label: "per capita income", num: &["per_capita_income"], den: "", kind: K::Money },
    Met { pat: r"(?:total|aggregate|cumulative|combined) household income", label: "aggregate household income", num: &["aggregate_household_income"], den: "", kind: K::MoneyTotal },
    Met { pat: r"gini (?:index|coefficient)|income (?:in)?equality(?: measurements?)?|(?:income )?inequality(?: measurements?)?", label: "Gini index of income inequality", num: &["gini"], den: "", kind: K::Index },
    Met { pat: r"median (?:home|house|housing) values?|(?:home|house|housing) (?:prices|values)", label: "median home value", num: &["median_home_value"], den: "", kind: K::Money },
    Met { pat: r"(?:average |mean )?(?:commuting|commute) times?|(?:average |mean )?travel time to work", label: "mean travel time to work", num: &["commute_minutes"], den: "commuters", kind: K::Minutes },
    Met { pat: r"(?:female population|women|females)", label: "women", num: &["female"], den: "pop", kind: K::People },
    Met { pat: r"(?:male population|men|males)", label: "men", num: &["male"], den: "pop", kind: K::People },
];

fn met_res() -> &'static Vec<Regex> {
    static R: OnceLock<Vec<Regex>> = OnceLock::new();
    R.get_or_init(|| METS.iter().map(|m| Regex::new(&format!(r"\b(?:{})\b", m.pat)).unwrap()).collect())
}

fn find_met(s: &str) -> Option<(&'static Met, String)> {
    for (m, r) in METS.iter().zip(met_res()) {
        if let Some(x) = r.find(s) {
            return Some((m, format!("{} {}", &s[..x.start()], &s[x.end()..])));
        }
    }
    None
}

fn state_name(st: &str) -> &'static str {
    US_STATES.iter().find(|x| x.0 == st).map_or("", |x| x.1)
}

fn display(g: &Geo) -> String {
    match g.lvl {
        "010" => "United States".into(),
        "160" => {
            let base = g.name.split(", ").next().unwrap_or("");
            let b = re!(r" (?:city and borough|city|town|village|borough|CDP|municipality|urban county|unified government|consolidated government|metro government|metropolitan government)(?: \(balance\))?$").replace(base, "");
            format!("{b}, {}", state_name(g.st))
        }
        "500" => {
            let n = re!(r"District (\d+|\(at Large\))").captures(g.name).map(|c| c[1].to_string()).unwrap_or_default();
            if n.parse::<u32>().is_ok() {
                let k: u32 = n.parse().unwrap_or(0);
                let suf = match (k % 10, k % 100) {
                    (1, x) if x != 11 => "st",
                    (2, x) if x != 12 => "nd",
                    (3, x) if x != 13 => "rd",
                    _ => "th",
                };
                format!("{} {k}{suf} Congressional District", state_name(g.st))
            } else {
                format!("{} at-large congressional district", state_name(g.st))
            }
        }
        _ => g.name.to_string(),
    }
}

fn state_geo(st: &str) -> Option<usize> {
    acs().geos.iter().position(|g| g.lvl == "040" && g.st == st)
}

/// State nicknames ("sunshine state").
const NICKNAMES: &[(&str, &str)] = &[("sunshine state", "FL"), ("lone star state", "TX"), ("golden state", "CA"), ("empire state", "NY"), ("garden state", "NJ"), ("keystone state", "PA"), ("buckeye state", "OH"), ("peach state", "GA"), ("show me state", "MO"), ("bay state", "MA"), ("grand canyon state", "AZ"), ("evergreen state", "WA"), ("beehive state", "UT"), ("prairie state", "IL"), ("hawkeye state", "IA"), ("aloha state", "HI"), ("silver state", "NV"), ("bluegrass state", "KY"), ("volunteer state", "TN"), ("pelican state", "LA"), ("centennial state", "CO"), ("treasure state", "MT"), ("first state", "DE"), ("ocean state", "RI"), ("green mountain state", "VT"), ("granite state", "NH"), ("pine tree state", "ME"), ("old dominion", "VA"), ("tar heel state", "NC"), ("palmetto state", "SC"), ("magnolia state", "MS"), ("yellowhammer state", "AL"), ("natural state", "AR"), ("sooner state", "OK"), ("sunflower state", "KS"), ("cornhusker state", "NE"), ("mount rushmore state", "SD"), ("peace garden state", "ND"), ("north star state", "MN"), ("badger state", "WI"), ("great lakes state", "MI"), ("hoosier state", "IN"), ("mountain state", "WV"), ("old line state", "MD"), ("constitution state", "CT"), ("land of enchantment", "NM"), ("equality state", "WY"), ("gem state", "ID"), ("beaver state", "OR"), ("last frontier", "AK")];

#[derive(Clone, Copy, PartialEq)]
enum Pref {
    State,
    City,
}

/// Resolve a place; `pref` decides a name that is both a state and a city.
fn place(s: &str, pref: Pref) -> Option<usize> {
    let s = s.trim().trim_start_matches("the ").trim_end_matches("'s").trim_end_matches('\'').trim();
    let s = s.trim_end_matches('.').trim();
    if s.is_empty() {
        return None;
    }
    if matches!(s, "us" | "usa" | "u.s" | "united states" | "united states of america" | "america" | "nation" | "country" | "whole country") {
        return acs().geos.iter().position(|g| g.lvl == "010");
    }
    if let Some((_, st)) = NICKNAMES.iter().find(|n| n.0 == s.trim_start_matches("the ")) {
        return state_geo(st);
    }
    let city_alias = match s {
        "nyc" | "new york city" | "ny city" | "manhattan" => Some(("new york", "NY")),
        "la" | "l.a." => Some(("los angeles", "CA")),
        "sf" | "san fran" => Some(("san francisco", "CA")),
        "philly" => Some(("philadelphia", "PA")),
        "dc" | "washington dc" | "washington d.c" | "washington, dc" | "washington, d.c" | "d.c" => return state_geo("DC"),
        _ => None,
    };
    if let Some((c, st)) = city_alias {
        return acs().index.get(c)?.iter().copied().find(|i| acs().geos[*i].lvl == "160" && acs().geos[*i].st == st);
    }
    // congressional districts: "Colorado 3rd Congressional District", "Texas 7th district"
    if let Some(c) = re!(r"^(.+?)(?:'s)? (\d+)(?:st|nd|rd|th)? (?:congressional )?district$").captures(s).or_else(|| re!(r"^(.+?) congressional district (\d+)$").captures(s)) {
        let st = us_state(&c[1])?;
        let n: u32 = c[2].parse().ok()?;
        return acs().geos.iter().position(|g| g.lvl == "500" && g.st == st && re!(r"District (\d+)").captures(g.name).is_some_and(|m| m[1].parse::<u32>().ok() == Some(n)));
    }
    // "new york state", "state of washington"
    if let Some(x) = s.strip_suffix(" state").or_else(|| s.strip_prefix("state of ")) {
        if let Some(st) = us_state(x) {
            return state_geo(st);
        }
    }
    // a trailing state: "Beverly Hills CA", "Madison, WI", "Allendale County SC"
    let trailing = |re: &Regex| re.captures(s).and_then(|c| Some((c[1].to_string(), Some(us_state(&c[2])?))));
    let whole = us_state(s).is_some();
    let (core, st) = match trailing(re!(r"^(.+?),? ([a-z]{2})$")).or_else(|| if whole { None } else { trailing(re!(r"^(.+?),? ((?:new|north|south|west|rhode|district of) [a-z]+|[a-z]+)$")) }) {
        Some((c, st)) if c.split(' ').count() <= 4 => (c, st),
        _ => (s.to_string(), None),
    };
    let pick = |key: &str, lvl: &str| -> Option<usize> {
        acs().index.get(key)?.iter().copied().filter(|i| acs().geos[*i].lvl == lvl && st.is_none_or(|s| acs().geos[*i].st == s)).max_by(|a, b| pop(&acs().geos[*a]).total_cmp(&pop(&acs().geos[*b])))
    };
    if re!(r" (?:county|parish|borough)$").is_match(&core) {
        return pick(&core, "050");
    }
    if let Some(m) = re!(r"^(.+?) (?:metro(?:politan)?|micro(?:politan)?)(?: statistical)?(?: area)?$").captures(&core) {
        return pick(&format!("{} metro", &m[1]), "310");
    }
    let state = if st.is_none() { us_state(&core).and_then(state_geo) } else { None };
    let city = pick(&core, "160");
    match (state, city) {
        (Some(s), Some(c)) => Some(if pref == Pref::City { c } else { s }),
        (Some(s), None) => Some(s),
        (None, Some(c)) => Some(c),
        (None, None) => pick(&format!("{core} county"), "050"),
    }
}

fn is_state_and_city(s: &str) -> bool {
    let s = s.trim().trim_start_matches("the ");
    us_state(s).is_some() && acs().index.get(s).is_some_and(|v| v.iter().any(|i| acs().geos[*i].lvl == "160"))
}

const STOP: &str = r"\b(?:what|which|is|was|are|were|the|total|in|of|for|from|how|many|much|number|do|does|did|has|have|there|amount|live|living|lives|who|people|persons|population|fraction|percentage|percent|share|proportion|rate|ratio|count|estimate|by|that)\b";

fn places_in(rest: &str) -> Option<Vec<String>> {
    let r = re!(STOP).replace_all(rest, " ");
    let r = re!(r"[%?!;:()]").replace_all(&r, " ");
    let r = re!(r"\s+").replace_all(&r, " ");
    let mut out = Vec::new();
    for part in re!(r",(?:\s*(?:and|&)\s+)?|\s(?:and|&|vs|versus)\s").split(r.trim()) {
        let p = part.trim();
        if !p.is_empty() {
            out.push(p.to_string());
        }
    }
    // "Madison, WI": a lone state abbreviation after a place belongs to it
    let mut merged: Vec<String> = Vec::new();
    for p in out {
        if p.len() == 2 && us_state(&p).is_some() && !merged.is_empty() {
            let last = merged.pop().unwrap_or_default();
            merged.push(format!("{last} {p}"));
        } else {
            merged.push(p);
        }
    }
    Some(merged)
}

fn resolve(names: &[String]) -> Option<Vec<usize>> {
    let firsts: Vec<Option<usize>> = names.iter().map(|n| if is_state_and_city(n) { None } else { place(n, Pref::State) }).collect();
    let any_city = firsts.iter().flatten().any(|i| acs().geos[*i].lvl == "160");
    names.iter().zip(firsts).map(|(n, f)| f.or_else(|| place(n, if any_city { Pref::City } else { Pref::State }))).collect()
}

fn people(x: f64) -> String {
    format!("{} people", super::geo::fmt_int(x.round() as u64))
}

fn pct(x: f64) -> String {
    format!("{}%", fmt_float((x * 1000.0).round() / 10.0, 3))
}

fn money(x: f64) -> String {
    if x >= 1e9 {
        format!("${} billion", fmt_float((x / 1e9 * 1000.0).round() / 1000.0, 4))
    } else {
        format!("${}", super::geo::fmt_int(x.round() as u64))
    }
}

/// The value of a metric for one geography: (number used for sorting, text).
fn measure(m: &Met, g: &Geo, frac: bool) -> Option<(f64, String)> {
    let n = sum(g, m.num)?;
    match m.kind {
        K::People => {
            let d = if m.den.is_empty() { None } else { val(g, m.den).filter(|d| *d > 0.0) };
            match (frac, d) {
                (true, Some(d)) => Some((n / d, pct(n / d))),
                (_, Some(d)) => Some((n, format!("{} ({} of {})", people(n), pct(n / d), den_label(m.den)))),
                _ => Some((n, people(n))),
            }
        }
        K::Money => Some((n, format!("{} per year", money(n)).replace(" per year", if m.num[0] == "median_home_value" { "" } else { " per year" }))),
        K::MoneyTotal => Some((n, format!("{} per year", money(n)))),
        K::Index => Some((n, fmt_float((n * 1e4).round() / 1e4, 4))),
        K::Years => Some((n, format!("{} years", fmt_float((n * 10.0).round() / 10.0, 4)))),
        K::Minutes => {
            let d = val(g, m.den).filter(|d| *d > 0.0)?;
            Some((n / d, format!("{} minutes", fmt_float((n / d * 10.0).round() / 10.0, 3))))
        }
        K::Table(_) => None,
    }
}

fn den_label(d: &str) -> &'static str {
    match d {
        "pop" => "population",
        "mobility_pop" => "population aged 1+",
        "marital_pop" => "population aged 15+",
        "men15" => "men aged 15+",
        "women15" => "women aged 15+",
        "adults25" => "population aged 25+",
        "language_pop" => "population aged 5+",
        "poverty_pop" => "population for whom poverty status is determined",
        _ => "total",
    }
}

fn source() -> Pod {
    pod("Source", "Source", text("US Census Bureau, American Community Survey 2019–2023 5-year estimates."))
}

fn table_answer(m: &Met, idx: &[usize]) -> Option<Answer> {
    let K::Table(cols) = m.kind else { return None };
    let names: Vec<String> = idx.iter().map(|i| display(&acs().geos[*i])).collect();
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} | {}", names.join(", "), m.label)))];
    let cell = |g: &Geo, key: &str| -> Option<String> {
        let parts: Vec<&str> = key.split('+').collect();
        let n = sum(g, &parts)?;
        let d = val(g, m.den).filter(|d| *d > 0.0)?;
        Some(format!("{} ({})", super::geo::fmt_int(n.round() as u64), pct(n / d)))
    };
    if idx.len() == 1 {
        let g = &acs().geos[idx[0]];
        let rows: Vec<Vec<Content>> = cols.iter().filter_map(|(k, l)| Some(vec![text(*l), text(cell(g, k)?)])).collect();
        pods.push(primary(pod("Result", "Result", table(Some(vec!["group", "people"]), rows))));
    } else {
        let mut head = vec!["group"];
        let hs: Vec<String> = names.clone();
        head.extend(hs.iter().map(|s| s.as_str()));
        let rows: Vec<Vec<Content>> = cols.iter().map(|(k, l)| {
            let mut r = vec![text(*l)];
            r.extend(idx.iter().map(|i| text(cell(&acs().geos[*i], k).unwrap_or_default())));
            r
        }).collect();
        pods.push(primary(pod("Result", "Result", table(Some(head), rows))));
    }
    pods.push(source());
    Some(Answer::new("us demographics", pods))
}

fn frac_words(s: &str) -> bool {
    re!(r"\b(?:fraction|percentage|percent|share|proportion|rate)\b|%").is_match(s)
}

/// Lists and rankings of states, counties, districts, metro areas or cities.
fn ranking(l: &str) -> Option<Answer> {
    let lvl_of = |w: &str| -> Option<(&'static str, &'static str)> {
        Some(match w.trim_end_matches('s') {
            "state" => ("040", "states"),
            "county" | "countie" => ("050", "counties"),
            "congressional district" | "district" => ("500", "congressional districts"),
            "metro area" | "metropolitan area" => ("310", "metro areas"),
            "city" | "citie" => ("160", "cities"),
            _ => return None,
        })
    };
    const LV: &str = r"(states?|counties|county|congressional districts?|metro areas?|metropolitan areas?|cities|city)";
    let (scope, lvl, how, what, thresh): (Option<&str>, (&str, &str), Option<String>, String, Option<(bool, f64)>);
    let scope_s;
    if let Some(c) = Regex::new(&format!(r"^(?:the )?(?:(us|u\.s\.|american|[a-z]+(?: [a-z]+)?) )?{LV}(?: (?:in|of) (?:the )?([a-z .]+?))? (?:with|by|having) (?:the |an? )?(?:(highest|lowest|largest|biggest|smallest|most|fewest|greatest|least|top|bottom) )?(.+?)(?: (>|<|above|below|over|under|greater than|less than) (\d+(?:\.\d+)?)%)?$")).ok()?.captures(l) {
        let sc = c.get(3).or(c.get(1)).map(|x| x.as_str().to_string());
        scope_s = sc;
        scope = scope_s.as_deref();
        lvl = lvl_of(&c[2])?;
        how = c.get(4).map(|x| x.as_str().to_string());
        what = c[5].to_string();
        thresh = c.get(7).and_then(|v| Some((matches!(c.get(6)?.as_str(), ">" | "above" | "over" | "greater than"), v.as_str().parse::<f64>().ok()? / 100.0)));
    } else if let Some(c) = Regex::new(&format!(r"^(.+?) (?:in|of|for|across) (?:the )?(?:(us|u\.s\.|[a-z]+(?: [a-z]+)?)(?:'s)? )?{LV}$")).ok()?.captures(l) {
        scope_s = c.get(2).map(|x| x.as_str().to_string());
        scope = scope_s.as_deref();
        lvl = lvl_of(&c[3])?;
        how = None;
        what = c[1].to_string();
        thresh = None;
    } else {
        return None;
    }
    let st = match scope {
        None | Some("us" | "u.s." | "american") => None,
        Some(s) => Some(us_state(s).or_else(|| NICKNAMES.iter().find(|n| n.0 == s).map(|n| n.1))?),
    };
    if st.is_some() && lvl.0 == "040" {
        return None;
    }
    let (m, rest) = find_met(&what)?;
    if !re!(STOP).replace_all(&rest, " ").trim().is_empty() || matches!(m.kind, K::Table(_)) {
        return None;
    }
    let frac = frac_words(&what) || thresh.is_some() || (how.is_some() && m.kind == K::People && re!(r"\b(?:highest|lowest)\b").is_match(how.as_deref().unwrap_or("")) && frac_words(l));
    let mut vals: Vec<(usize, f64, String)> = acs().geos.iter().enumerate().filter(|(_, g)| g.lvl == lvl.0 && st.is_none_or(|s| g.st == s) && (st.is_some() || g.st != "PR")).filter_map(|(i, g)| {
        let (v, t) = measure(m, g, frac)?;
        Some((i, v, t))
    }).collect();
    if let Some((above, t)) = thresh {
        vals.retain(|x| if above { x.1 > t } else { x.1 < t });
    }
    let low = how.as_deref().is_some_and(|h| matches!(h, "lowest" | "smallest" | "fewest" | "least" | "bottom"));
    vals.sort_by(|a, b| if low { a.1.total_cmp(&b.1) } else { b.1.total_cmp(&a.1) });
    if vals.is_empty() {
        let where_ = st.map_or("US".to_string(), |s| state_name(s).to_string());
        return thresh.map(|(above, t)| Answer::new("us demographics", vec![
            pod("Input", "Input interpretation", text(format!("{where_} {} | {} (fraction) {} {}", lvl.1, m.label, if above { ">" } else { "<" }, pct(t)))),
            primary(pod("Result", "Result", text(format!("none (no {where_} {} in the 2019–2023 estimates)", lvl.1)))),
            source(),
        ]));
    }
    let single = how.is_some() && !l.contains(&format!("{} ", lvl.1)) && re!(r"^(?:the )?(?:(?:us|u\.s\.|american|[a-z]+) )?(?:state|county|congressional district|metro area|city) ").is_match(l);
    let n = if single { 1 } else if how.is_some() { 10 } else { vals.len().min(60) };
    let rows: Vec<Vec<Content>> = vals.iter().take(n.max(5).min(vals.len()).max(if thresh.is_some() { vals.len().min(60) } else { 0 })).map(|(i, _, t)| vec![text(display(&acs().geos[*i])), text(t.clone())]).collect();
    let where_ = st.map_or("US".to_string(), |s| state_name(s).to_string());
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{where_} {} | {}{}", lvl.1, m.label, if frac { " (fraction)" } else { "" })))];
    if single {
        let (i, _, t) = &vals[0];
        pods.push(primary(pod("Result", "Result", text(format!("{}: {t}", display(&acs().geos[*i]))))));
        pods.push(pod("Ranking", "Ranking", table(Some(vec!["place", "value"]), rows)));
    } else {
        pods.push(primary(pod("Result", if thresh.is_some() { "Matching places" } else { "Result" }, table(Some(vec!["place", "value"]), rows))));
        if thresh.is_some() {
            pods.push(pod("Count", "Count", text(vals.len().to_string())));
        }
    }
    pods.push(source());
    Some(Answer::new("us demographics", pods))
}

fn sexed(var: &str) -> Option<(&'static str, &'static str)> {
    Some(match var {
        "same_house" => ("same_house_male", "same_house_female"),
        "poverty" => ("poverty_male", "poverty_female"),
        "over65" => ("male65", "female65"),
        "doctorate" => ("doctorate_men", "doctorate_women"),
        "never_married_men" => ("never_married_men", "never_married_women"),
        "divorced_men" => ("divorced_men", "divorced_women"),
        "pop" => ("male", "female"),
        _ => return None,
    })
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace("vs.", "vs").replace("u.s.", "us");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = re!(r"^(?:what (?:is|was|are|were) |how many |show (?:me )?|list )").replace(&l, "").to_string();
    if let Some(a) = ranking(&l) {
        return Some(a);
    }
    let frac = frac_words(&l);
    // "iowa people who stayed in the same house males / females"
    if let Some(c) = re!(r"^(.+?) (?:males?|men) ?/ ?(?:females?|women)$").captures(&l) {
        let (m, rest) = find_met(&c[1])?;
        let (a, b) = sexed(m.num[0])?;
        let names = places_in(&rest)?;
        let idx = resolve(&names)?;
        if idx.len() != 1 {
            return None;
        }
        let g = &acs().geos[idx[0]];
        let (x, y) = (val(g, a)?, val(g, b)?);
        return Some(Answer::new("us demographics", vec![
            pod("Input", "Input interpretation", text(format!("{} | {} | male / female", display(g), m.label))),
            primary(pod("Result", "Result", text(fmt_float(x / y, 4)))),
            pod("Values", "Values", table(Some(vec!["group", "people"]), vec![vec![text("male"), text(people(x))], vec![text("female"), text(people(y))]])),
            source(),
        ]));
    }
    // ratios: "unmarried women in NYC / unmarried men in NYC"
    if let Some((a, b)) = l.split_once('/') {
        let (ma, ra) = find_met(a)?;
        let (mb, rb) = find_met(b).or_else(|| Some((ma, b.to_string())))?;
        let (na, nb) = (places_in(&ra)?, places_in(&rb)?);
        let na = if na.is_empty() { nb.clone() } else { na };
        let nb = if nb.is_empty() { na.clone() } else { nb };
        if na.len() != 1 || nb.len() != 1 || matches!(ma.kind, K::Table(_)) || matches!(mb.kind, K::Table(_)) {
            return None;
        }
        let (ga, gb) = (&acs().geos[place(&na[0], Pref::State)?], &acs().geos[place(&nb[0], Pref::State)?]);
        let (va, ta) = measure(ma, ga, false)?;
        let (vb, tb) = measure(mb, gb, false)?;
        let r = va / vb;
        return Some(Answer::new("us demographics", vec![
            pod("Input", "Input interpretation", text(format!("({} {}) / ({} {})", display(ga), ma.label, display(gb), mb.label))),
            primary(pod("Result", "Result", text(if r < 1.0 && ma.den == mb.den && ma.kind == K::People { format!("{} ({})", fmt_float(r, 4), pct(r)) } else { fmt_float(r, 4) }))),
            pod("Values", "Values", table(Some(vec!["quantity", "value"]), vec![vec![text(format!("{} {}", display(ga), ma.label)), text(ta)], vec![text(format!("{} {}", display(gb), mb.label)), text(tb)]])),
            source(),
        ]));
    }
    // "Los Angeles vs New York naturalized citizens", "black population in Philadelphia"
    let parts: Vec<&str> = re!(r"\s(?:vs|versus|compared (?:to|with))\s").split(&l).collect();
    let mut mets: Vec<Option<&'static Met>> = Vec::new();
    let mut names: Vec<Vec<String>> = Vec::new();
    for p in &parts {
        match find_met(p) {
            Some((m, rest)) => {
                mets.push(Some(m));
                names.push(places_in(&rest)?);
            }
            None => {
                mets.push(None);
                names.push(places_in(p)?);
            }
        }
    }
    let m = mets.iter().flatten().next().copied()?;
    if mets.iter().flatten().any(|x| x.label != m.label) {
        return None;
    }
    let all: Vec<String> = names.into_iter().flatten().collect();
    if all.is_empty() {
        return None;
    }
    let idx = resolve(&all)?;
    // the populations of single cities and states are answered elsewhere
    if idx.len() == 1 && m.num == ["male"] && !frac {
        return None;
    }
    if let K::Table(_) = m.kind {
        return table_answer(m, &idx);
    }
    let rows: Vec<(String, f64, String)> = idx.iter().map(|i| {
        let g = &acs().geos[*i];
        let (v, t) = measure(m, g, frac)?;
        Some((display(g), v, t))
    }).collect::<Option<_>>()?;
    let names: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} | {}{}", names.join(" | "), m.label, if frac && m.kind == K::People { " (fraction)" } else { "" })))];
    if rows.len() == 1 {
        pods.push(primary(pod("Result", "Result", text(rows[0].2.clone()))));
    } else {
        let tr: Vec<Vec<Content>> = rows.iter().map(|r| vec![text(r.0.clone()), text(r.2.clone())]).collect();
        pods.push(primary(pod("Result", "Result", table(Some(vec!["place", "value"]), tr))));
    }
    pods.push(source());
    Some(Answer::new("us demographics", pods))
}
