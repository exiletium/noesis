//! Country indicators from the CIA World Factbook (public domain, via the
//! CC0 factbook.json export): demographics, health, economy, energy,
//! communications, transport, military, government and geography.
//!
//! "life expectancy in New Zealand", "unemployment rate Spain vs Italy",
//! "life expectancy U.S., Sweden, Japan", "countries with the highest
//! inflation", "Russia vs China economy", "flag of Sri Lanka".
//!
//! `tools/datagen/factbook_stats.py` builds `data/factbook.z`.

use super::geo::{countries, find_country};
use crate::engine::*;
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

static FACTBOOK_Z: &[u8] = include_bytes!("../../data/factbook.z");

type Rows = Vec<(&'static str, &'static str, &'static str)>;

fn data() -> &'static HashMap<String, (&'static str, Rows)> {
    static D: OnceLock<HashMap<String, (&'static str, Rows)>> = OnceLock::new();
    D.get_or_init(|| {
        let bytes = miniz_oxide::inflate::decompress_to_vec(FACTBOOK_Z).unwrap_or_default();
        let text: &'static str = Box::leak(String::from_utf8(bytes).unwrap_or_default().into_boxed_str());
        let mut m: HashMap<String, (&'static str, Rows)> = HashMap::new();
        for line in text.lines() {
            let c: Vec<&'static str> = line.splitn(4, '\t').collect();
            if c.len() == 4 {
                m.entry(c[0].to_lowercase()).or_insert_with(|| (c[0], Vec::new())).1.push((c[1], c[2], c[3]));
            }
        }
        m
    })
}

/// The Factbook entity named by `s` (a country, "the European Union", "the world").
fn entity(s: &str) -> Option<&'static (&'static str, Rows)> {
    let s = s.trim().trim_end_matches(['?', '.']).trim();
    let l = s.to_lowercase();
    let l = l.trim_start_matches("the ").trim();
    let l = match l {
        "eu" | "european union" | "europe union" => "european union",
        "world" | "whole world" | "worldwide" | "earth" | "globe" | "global" => "world",
        "u.s." | "u.s" | "us" | "usa" | "u.s.a." | "america" | "united states of america" => "united states",
        "uk" | "u.k." | "britain" | "great britain" => "united kingdom",
        x => x,
    };
    if let Some(e) = data().get(l) {
        return Some(e);
    }
    let c = find_country(l)?;
    data().get(&c.name.to_lowercase())
}

/// aliases | field | subfield ("" = all subfields, "=" = the unnamed value) | label
const PROPS: &[(&str, &str, &str, &str)] = &[
    ("life expectancy|life expectancy at birth|average life expectancy|average lifespan|lifespan", "Life expectancy at birth", "total population", "life expectancy at birth"),
    ("male life expectancy|life expectancy male|life expectancy of men|life expectancy for men|men's life expectancy", "Life expectancy at birth", "male", "male life expectancy at birth"),
    ("female life expectancy|life expectancy female|life expectancy of women|life expectancy for women|women's life expectancy", "Life expectancy at birth", "female", "female life expectancy at birth"),
    ("median age|average age", "Median age", "total", "median age"),
    ("population growth|population growth rate", "Population growth rate", "=", "population growth rate"),
    ("birth rate|birthrate", "Birth rate", "=", "birth rate"),
    ("death rate|mortality rate", "Death rate", "=", "death rate"),
    ("net migration rate|migration rate|net migration", "Net migration rate", "=", "net migration rate"),
    ("fertility rate|total fertility rate|births per woman|fertility", "Total fertility rate", "=", "total fertility rate"),
    ("infant mortality|infant mortality rate", "Infant mortality rate", "total", "infant mortality rate"),
    ("maternal mortality|maternal mortality rate|maternal mortality ratio", "Maternal mortality ratio", "=", "maternal mortality ratio"),
    ("literacy|literacy rate", "Literacy", "total population", "literacy rate"),
    ("underweight children|children underweight|child underweight|underweight|children under 5 underweight", "Children under the age of 5 years underweight", "=", "children under 5 who are underweight"),
    ("household consumption|household consumption share|private consumption", "GDP - composition, by end use", "household consumption", "household consumption (share of GDP)"),
    ("government consumption|government consumption share|public consumption", "GDP - composition, by end use", "government consumption", "government consumption (share of GDP)"),
    ("capital investment|fixed investment|investment in fixed capital|investment", "GDP - composition, by end use", "investment in fixed capital", "investment in fixed capital (share of GDP)"),
    ("labor force by occupation|employment by sector|labor force by sector|employment by occupation|labour force by occupation|jobs by sector", "Labor force - by occupation", "", "labor force by occupation"),
    ("child marriage|child marriage rate", "Child marriage", "", "child marriage"),
    ("gross reproduction rate", "Gross reproduction rate", "=", "gross reproduction rate"),
    ("currently married women|married women", "Currently married women (ages 15-49)", "=", "currently married women (ages 15–49)"),
    ("particulate matter emissions|particulate emissions|pm2.5 emissions|pm2.5", "Particulate matter emissions", "=", "particulate matter emissions"),
    ("urban population|urbanization|urbanisation|urbanization rate|percent urban", "Urbanization", "urban population", "urban population"),
    ("obesity|obesity rate|adult obesity", "Obesity - adult prevalence rate", "=", "adult obesity rate"),
    ("alcohol consumption|alcohol consumption per capita|alcohol use", "Alcohol consumption per capita", "total", "alcohol consumption per capita"),
    ("tobacco use|smoking rate|smoking|tobacco", "Tobacco use", "total", "tobacco use"),
    ("physician density|doctors|doctors per capita|number of doctors|physicians|medical personnel", "Physician density", "=", "physician density"),
    ("hospital beds|hospital bed density", "Hospital bed density", "=", "hospital bed density"),
    ("health expenditure|health expenditures|healthcare expenditure|healthcare expenditures|health care expenditures|health spending|healthcare spending", "Health expenditure", "", "health expenditure"),
    ("education expenditure|education expenditures|education spending", "Education expenditure", "", "education expenditure"),
    ("ethnic groups|ethnicity|ethnicities", "Ethnic groups", "", "ethnic groups"),
    ("religions|religion|religious groups", "Religions", "", "religions"),
    ("sex ratio|gender ratio", "Sex ratio", "total population", "sex ratio"),
    ("age structure|age distribution|age pyramid|population pyramid", "Age structure", "", "age structure"),
    ("dependency ratio|dependency ratios", "Dependency ratios", "", "dependency ratios"),
    ("school life expectancy", "School life expectancy (primary to tertiary education)", "total", "school life expectancy"),
    ("drinking water|drinking water access|access to drinking water", "Drinking water source", "", "drinking water source"),
    ("sanitation|sanitation access", "Sanitation facility access", "", "sanitation facility access"),
    ("major cities|largest cities|major urban areas|biggest cities", "Major urban areas - population", "=", "major urban areas"),
    ("mother's mean age at first birth|mean age at first birth|age at first birth", "Mother's mean age at first birth", "=", "mother's mean age at first birth"),
    ("gdp ppp|gdp (ppp)|ppp gdp|gdp purchasing power parity|real gdp", "Real GDP (purchasing power parity)", "=", "real GDP (purchasing power parity)"),
    ("gdp growth|gdp growth rate|economic growth|economic growth rate|growth rate", "Real GDP growth rate", "=", "real GDP growth rate"),
    ("gdp per capita ppp|ppp gdp per capita|real gdp per capita|gdp per capita (ppp)", "Real GDP per capita", "=", "real GDP per capita (PPP)"),
    ("gdp official exchange rate|gdp at official exchange rate", "GDP (official exchange rate)", "=", "GDP (official exchange rate)"),
    ("inflation|inflation rate|consumer price inflation", "Inflation rate (consumer prices)", "=", "inflation rate (consumer prices)"),
    ("unemployment|unemployment rate|jobless rate", "Unemployment rate", "=", "unemployment rate"),
    ("youth unemployment|youth unemployment rate", "Youth unemployment rate (ages 15-24)", "total", "youth unemployment rate (ages 15–24)"),
    ("poverty|poverty rate|population below poverty line|poverty line|people in poverty|population in poverty|people below the poverty line", "Population below poverty line", "=", "population below poverty line"),
    ("gini|gini index|gini coefficient|income inequality|inequality", "Gini Index coefficient - distribution of family income", "=", "Gini index"),
    ("exports|total exports|export value", "Exports", "=", "exports"),
    ("imports|total imports|import value", "Imports", "=", "imports"),
    ("export partners|exports partners|main export partners|trading partners", "Exports - partners", "=", "export partners"),
    ("import partners|imports partners|main import partners", "Imports - partners", "=", "import partners"),
    ("main exports|export commodities|exports commodities|exported goods", "Exports - commodities", "=", "export commodities"),
    ("main imports|import commodities|imports commodities|imported goods", "Imports - commodities", "=", "import commodities"),
    ("public debt|national debt|government debt|debt", "Public debt", "=", "public debt"),
    ("external debt|foreign debt", "Debt - external", "=", "external debt"),
    ("budget|government budget|government revenue|government spending", "Budget", "", "budget"),
    ("budget balance|budget deficit|budget surplus|deficit", "Budget surplus (+) or deficit (-)", "=", "budget surplus (+) or deficit (−)"),
    ("labor force|labour force|workforce|work force", "Labor force", "=", "labor force"),
    ("industries|main industries|industry", "Industries", "=", "industries"),
    ("agricultural products|agriculture|crops|main crops|farm products", "Agricultural products", "=", "agricultural products"),
    ("industrial production growth|industrial production growth rate", "Industrial production growth rate", "=", "industrial production growth rate"),
    ("current account|current account balance", "Current account balance", "=", "current account balance"),
    ("foreign exchange reserves|reserves|currency reserves|gold reserves|reserves of foreign exchange and gold", "Reserves of foreign exchange and gold", "=", "reserves of foreign exchange and gold"),
    ("taxes|tax revenue|taxes and other revenues|tax rate", "Taxes and other revenues", "=", "taxes and other revenues"),
    ("gdp by sector|gdp composition|economic sectors|economy by sector", "GDP - composition, by sector of origin", "", "GDP composition by sector of origin"),
    ("electricity consumption|electric power consumption|power consumption|electricity use|electricity usage", "Electricity", "consumption", "electricity consumption"),
    ("electricity exports", "Electricity", "exports", "electricity exports"),
    ("electricity imports", "Electricity", "imports", "electricity imports"),
    ("installed generating capacity|generating capacity|power generation capacity|electricity generating capacity|installed capacity", "Electricity", "installed generating capacity", "installed generating capacity"),
    ("electricity access|electrification|access to electricity", "Electricity access", "electrification - total population", "electrification (total population)"),
    ("electricity sources|energy mix|electricity generation sources|electricity mix|power sources|electricity production by source", "Electricity generation sources", "", "electricity generation sources"),
    ("nuclear energy|nuclear power|nuclear reactors|nuclear power plants", "Nuclear energy", "", "nuclear energy"),
    ("oil production|petroleum production|crude oil production|crude production", "Petroleum", "total petroleum production", "total petroleum production"),
    ("oil consumption|petroleum consumption", "Petroleum", "refined petroleum consumption", "refined petroleum consumption"),
    ("oil reserves|crude oil reserves|petroleum reserves", "Petroleum", "crude oil estimated reserves", "crude oil reserves"),
    ("natural gas production|gas production", "Natural gas", "production", "natural gas production"),
    ("natural gas consumption|gas consumption", "Natural gas", "consumption", "natural gas consumption"),
    ("natural gas reserves|gas reserves", "Natural gas", "proven reserves", "natural gas reserves"),
    ("coal production", "Coal", "production", "coal production"),
    ("coal consumption", "Coal", "consumption", "coal consumption"),
    ("coal reserves", "Coal", "proven reserves", "coal reserves"),
    ("energy consumption per capita|energy use per capita|per capita energy consumption", "Energy consumption per capita", "=", "energy consumption per capita"),
    ("co2 emissions|carbon dioxide emissions|carbon emissions|greenhouse gas emissions|emissions", "Carbon dioxide emissions", "total emissions", "carbon dioxide emissions"),
    ("methane emissions", "Methane emissions", "", "methane emissions"),
    ("internet users|internet usage|internet access|internet penetration", "Internet users", "percent of population", "internet users (share of population)"),
    ("mobile phones|cell phones|mobile phone subscriptions|cellphones|mobile subscriptions|cell phone subscriptions", "Telephones - mobile cellular", "total subscriptions", "mobile phone subscriptions"),
    ("telephones|landlines|fixed telephone lines|telephone lines|fixed lines", "Telephones - fixed lines", "total subscriptions", "fixed telephone lines"),
    ("broadband|broadband subscriptions|fixed broadband", "Broadband - fixed subscriptions", "total", "fixed broadband subscriptions"),
    ("airports|number of airports", "Airports", "=", "airports"),
    ("heliports", "Heliports", "=", "heliports"),
    ("railways|railway length|rail network|railroads|railroad length|railway network|rail length|length of railways|railway", "Railways", "total", "railways"),
    ("merchant marine|merchant fleet|merchant ships", "Merchant marine", "total", "merchant marine"),
    ("ports|seaports|major ports", "Ports", "", "ports"),
    ("military expenditures|military expenditure|military spending|defense spending|defence spending|military budget|defense budget|defence budget", "Military expenditures", "=", "military expenditures"),
    ("military personnel|military size|armed forces|army size|active military|soldiers|military strength|size of the military|military manpower", "Military and security service personnel strengths", "=", "military and security personnel"),
    ("military service age|conscription|military service|draft age", "Military service age and obligation", "=", "military service age and obligation"),
    ("coastline|coastline length|length of coastline|coast", "Coastline", "=", "coastline"),
    ("highest point|highest mountain|highest peak|tallest mountain", "Elevation", "highest point", "highest point"),
    ("lowest point", "Elevation", "lowest point", "lowest point"),
    ("mean elevation|average elevation|elevation", "Elevation", "mean elevation", "mean elevation"),
    ("climate", "Climate", "=", "climate"),
    ("terrain|topography|landscape", "Terrain", "=", "terrain"),
    ("natural resources|resources|mineral resources", "Natural resources", "=", "natural resources"),
    ("land use", "Land use", "", "land use"),
    ("irrigated land", "Irrigated land", "=", "irrigated land"),
    ("natural hazards|natural disasters", "Natural hazards", "", "natural hazards"),
    ("major rivers|rivers|longest rivers", "Major rivers (by length in km)", "=", "major rivers"),
    ("major lakes|lakes|largest lakes", "Major lakes (area sq km)", "", "major lakes"),
    ("land boundaries|border length|total border length|land borders", "Land boundaries", "", "land boundaries"),
    ("water withdrawal|total water withdrawal|water use", "Total water withdrawal", "", "water withdrawal"),
    ("renewable water resources|water resources", "Total renewable water resources", "=", "renewable water resources"),
    ("municipal waste|waste|waste and recycling|recycling", "Waste and recycling", "", "waste and recycling"),
    ("environmental issues|environmental problems|environment issues", "Environmental issues", "=", "environmental issues"),
    ("government type|type of government|form of government|government system|political system", "Government type", "=", "government type"),
    ("independence|independence day|date of independence|declares independence|declared independence|declare independence|declaration of independence", "Independence", "=", "independence"),
    ("national holiday|national day", "National holiday", "=", "national holiday"),
    ("constitution", "Constitution", "", "constitution"),
    ("legal system", "Legal system", "=", "legal system"),
    ("suffrage|voting age", "Suffrage", "=", "suffrage"),
    ("flag|flag description|flag of", "Flag", "description", "flag"),
    ("flag meaning|meaning of the flag|flag symbolism", "Flag", "meaning", "meaning of the flag"),
    ("national anthem|anthem", "National anthem(s)", "", "national anthem"),
    ("national symbols|national symbol|national animal|national emblem", "National symbol(s)", "=", "national symbols"),
    ("national colors|national colours", "National color(s)", "=", "national colors"),
    ("national heritage|world heritage sites|unesco sites|unesco world heritage sites", "National heritage", "", "national heritage"),
    ("administrative divisions|provinces|states|regions|subdivisions", "Administrative divisions", "=", "administrative divisions"),
    ("location|geographic location", "Location", "=", "location"),
    ("geographic coordinates|coordinates", "Geographic coordinates", "=", "geographic coordinates"),
];

fn prop(s: &str) -> Option<&'static (&'static str, &'static str, &'static str, &'static str)> {
    let s = s.trim().trim_start_matches("the ").trim_start_matches("current ").trim_start_matches("annual ").trim();
    let s = s.trim_end_matches(" rate").to_string();
    PROPS.iter().find(|p| p.0.split('|').any(|a| a == s || a.trim_end_matches(" rate") == s))
}

/// The rows of `field`/`sub` for an entity: (subfield label, text).
/// `sub` "" gives every subfield; "=" the unlabelled value, or else the first.
fn values(rows: &Rows, field: &str, sub: &str) -> Vec<(String, &'static str)> {
    let all: Vec<_> = rows.iter().filter(|r| r.0 == field).collect();
    let pick: Vec<_> = match sub {
        "" => all,
        "=" => all.iter().find(|r| r.1.is_empty()).or(all.first()).into_iter().copied().collect(),
        s => all.into_iter().filter(|r| r.1 == s).collect(),
    };
    pick.into_iter().map(|r| (r.1.to_string(), r.2)).collect()
}

/// First number in a Factbook value, scaled by million/billion/trillion.
fn number(t: &str) -> Option<f64> {
    let m = re!(r"(-?)\$?(\d[\d,]*(?:\.\d+)?)\s*(thousand|million|billion|trillion)?").captures(t)?;
    let mut v: f64 = m[2].replace(',', "").parse().ok()?;
    v *= match m.get(3).map(|x| x.as_str()) {
        Some("thousand") => 1e3,
        Some("million") => 1e6,
        Some("billion") => 1e9,
        Some("trillion") => 1e12,
        _ => 1.0,
    };
    Some(if &m[1] == "-" { -v } else { v })
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn source() -> Pod {
    pod("Source", "Source", text("CIA World Factbook (US government, public domain); figures are the latest estimates it gives, with their year"))
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn single(name: &str, rows: &Rows, p: &(&str, &str, &str, &str)) -> Option<Answer> {
    let v = values(rows, p.1, p.2);
    if v.is_empty() {
        return None;
    }
    let main = if v.len() == 1 { v[0].1.to_string() } else { v.iter().map(|(s, t)| if s.is_empty() { t.to_string() } else { format!("{s}: {t}") }).collect::<Vec<_>>().join("; ") };
    let mut pods = vec![input(format!("{name} | {}", p.3)), primary(pod("Result", &capitalise(p.3), text(main)))];
    if v.len() > 1 {
        pods.push(pod("Table", "Details", table(None, v.iter().map(|(s, t)| vec![text(s.clone()), text(*t)]).collect())));
    }
    // related values in the same field ("male", "female" for life expectancy)
    if !p.2.is_empty() && p.2 != "=" {
        let rel: Vec<Vec<Content>> = rows.iter().filter(|r| r.0 == p.1 && r.1 != p.2 && !r.1.is_empty()).map(|r| vec![text(r.1), text(r.2)]).collect();
        if !rel.is_empty() && rel.len() <= 12 {
            pods.push(pod("Related", "Related values", table(None, rel)));
        }
    }
    pods.push(source());
    Some(Answer::new("country data", pods))
}

fn several(ents: &[&'static (&'static str, Rows)], p: &(&str, &str, &str, &str)) -> Option<Answer> {
    let mut rows = vec![];
    let mut short = vec![];
    for e in ents {
        let v = values(&e.1, p.1, p.2);
        let t = v.first().map(|x| x.1).unwrap_or("(not available)");
        short.push(format!("{}: {t}", e.0));
        rows.push(vec![text(e.0), text(t)]);
    }
    if rows.iter().all(|r| content_text(&r[1]) == "(not available)") {
        return None;
    }
    Some(Answer::new("country comparison", vec![
        input(format!("{} | {}", ents.iter().map(|e| e.0).collect::<Vec<_>>().join(", "), p.3)),
        primary(pod("Result", &capitalise(p.3), text(short.join(" | ")))),
        pod("Table", "Comparison", table(Some(vec!["", p.3]), rows)),
        source(),
    ]))
}

fn continent_name(s: &str) -> Option<&'static str> {
    Some(match s {
        "africa" | "african" => "Africa",
        "europe" | "european" => "Europe",
        "asia" | "asian" => "Asia",
        "south america" | "south american" => "South America",
        "north america" | "north american" => "North America",
        "oceania" | "oceanian" => "Oceania",
        _ => return None,
    })
}

/// Country groups besides the continents: (name, members as ISO 3166 alpha-2 codes).
const GROUPS: &[(&str, &[&str])] = &[
    ("OPEC", &["DZ", "CG", "GQ", "GA", "IR", "IQ", "KW", "LY", "NG", "SA", "AE", "VE"]),
    ("G7", &["CA", "FR", "DE", "IT", "JP", "GB", "US"]),
];

fn group_name(s: &str) -> Option<&'static str> {
    continent_name(s).or_else(|| GROUPS.iter().find(|g| g.0.eq_ignore_ascii_case(s.trim_start_matches("the "))).map(|g| g.0))
}

/// Factbook entities of the countries on a continent (or in a group), with their populations.
fn members(cont: &str) -> Vec<(&'static (&'static str, Rows), f64)> {
    let group = GROUPS.iter().find(|g| g.0 == cont);
    countries().values().filter(|c| c.population > 0 && group.map_or(c.continent == cont, |g| g.1.contains(&c.iso2.as_str()))).filter_map(|c| Some((entity(&c.name)?, c.population as f64))).collect()
}

fn member_area(cont: &str) -> f64 {
    let group = GROUPS.iter().find(|g| g.0 == cont);
    countries().values().filter(|c| c.population > 0 && group.map_or(c.continent == cont, |g| g.1.contains(&c.iso2.as_str()))).map(|c| c.area_km2).sum()
}

/// A continent total or population-weighted average: (value text, countries used, countries on the continent).
fn continent_value(cont: &str, what: &str) -> Option<(String, String, usize, usize)> {
    let ms = members(cont);
    let total = ms.len();
    if what == "area" || what == "total area" || what == "land area" {
        return Some(("area".into(), format!("about {} km²", super::geo::fmt_int(member_area(cont) as u64)), total, total));
    }
    if what == "population" {
        let pop: f64 = ms.iter().map(|m| m.1).sum();
        return Some(("population".into(), format!("about {} people", super::geo::fmt_int(pop as u64)), total, total));
    }
    let (field, sub, label) = match what {
        "gdp" | "nominal gdp" | "economy" | "gdp official exchange rate" => ("GDP (official exchange rate)", "=", "GDP (official exchange rate)"),
        _ => {
            let p = prop(what)?;
            (p.1, p.2, p.3)
        }
    };
    let vals: Vec<(f64, f64, &str)> = ms.iter().filter_map(|(e, pop)| {
        let t = values(&e.1, field, sub).first()?.1;
        Some((number(t)?, *pop, t))
    }).collect();
    if vals.len() * 2 < total.max(2) {
        return None;
    }
    let sample = vals[0].2;
    let countable = sample.trim_start().starts_with('$') || re!(r" (?:million|billion|trillion)\b|kWh|bbl|cu m|\bkm\b").is_match(sample) || re!(r"airports|heliports|railways|merchant|coastline|labor force|military personnel|telephones|subscriptions").is_match(&label.to_lowercase());
    let rate = !countable;
    let text_v = if rate {
        let w: f64 = vals.iter().map(|v| v.1).sum();
        let mean = vals.iter().map(|v| v.0 * v.1).sum::<f64>() / w;
        let unit = if sample.contains("years") { " years" } else if sample.contains('%') { "%" } else { "" };
        format!("{:.1}{unit} (population-weighted average)", mean)
    } else {
        let s: f64 = vals.iter().map(|v| v.0).sum();
        let money = sample.trim_start().starts_with('$');
        let (x, word) = if s >= 1e12 { (s / 1e12, " trillion") } else if s >= 1e9 { (s / 1e9, " billion") } else if s >= 1e6 { (s / 1e6, " million") } else { (s, "") };
        format!("{}{:.2}{word} (sum over countries)", if money { "$" } else { "" }, x)
    };
    Some((label.to_string(), text_v, vals.len(), total))
}

/// "Europe life expectancy", "internet users in South America", "Africa population vs Africa GDP"
fn continent(l: &str) -> Option<Answer> {
    // "Oceania, South America GDP" and "Asian GINI index"
    const C: &str = r"(?:africa|europe|asia|south america|north america|oceania|opec|g7)";
    let l = &if let Some(m) = Regex::new(&format!(r"^({C}(?:,? (?:and )?{C})+) (.+)$")).ok()?.captures(l) {
        re!(r",\s*(?:and\s+)?|\s+and\s+").split(&m[1]).map(|c| format!("{c} {}", &m[2])).collect::<Vec<_>>().join(" vs ")
    } else if let Some(m) = re!(r"^(african|european|asian|south american|north american|oceanian) (.+)$").captures(l) {
        format!("{} {}", continent_name(&m[1])?.to_lowercase(), &m[2])
    } else {
        l.to_string()
    };
    // a bare comparison of continents or groups: "Europe vs OPEC"
    let bare: Option<Vec<&str>> = l.split(" vs ").map(|p| group_name(p.trim())).collect();
    if let Some(gs) = bare.filter(|g| g.len() >= 2) {
        let mut rows = Vec::new();
        for what in ["population", "area", "gdp", "life expectancy"] {
            let mut r = vec![text(what)];
            for g in &gs {
                r.push(text(continent_value(g, what).map_or("—".to_string(), |v| v.1)));
            }
            rows.push(r);
        }
        let mut head = vec![""];
        head.extend(gs.iter().copied());
        return Some(Answer::new("group comparison", vec![
            input(gs.join(" vs ")),
            primary(pod("Result", "Comparison", table(Some(head), rows))),
            pod("Note", "Note", text("sums and population-weighted averages of the member countries' Factbook values; OPEC has 12 members and the G7 seven")),
            source(),
        ]));
    }
    let parse = |s: &str| -> Option<(&'static str, String)> {
        let s = s.trim();
        if let Some(m) = re!(r"^(.+?) (?:of|in|for) (?:the )?(africa|europe|asia|south america|north america|oceania|opec|g7)$").captures(s) {
            return Some((group_name(&m[2])?, m[1].to_string()));
        }
        let m = re!(r"^(?:the )?(africa|europe|asia|south america|north america|oceania|opec|g7)(?:'s)? (.+)$").captures(s)?;
        Some((group_name(&m[1])?, m[2].to_string()))
    };
    let parts: Vec<&str> = l.split(" vs ").collect();
    let items: Vec<(&str, String)> = parts.iter().map(|p| parse(p)).collect::<Option<Vec<_>>>()?;
    let mut rows = Vec::new();
    let mut short = Vec::new();
    for (cont, what) in &items {
        let (label, v, used, total) = continent_value(cont, what)?;
        short.push(if items.len() == 1 { v.clone() } else { format!("{cont} {label}: {v}") });
        rows.push(vec![text(format!("{cont} | {label}")), text(v), text(format!("{used} of {total} countries"))]);
    }
    Some(Answer::new("continent data", vec![
        input(items.iter().map(|(c, w)| format!("{c} | {w}")).collect::<Vec<_>>().join(" vs ")),
        primary(pod("Result", "Result", text(short.join(" | ")))),
        pod("Table", "Details", table(Some(vec!["", "value", "countries with data"]), rows)),
        pod("Note", "Note", text("computed from the Factbook values of each country on the continent; rates are averaged with population weights")),
        source(),
    ]))
}

/// A numeric value for an entity, including counts derived from rates:
/// (label, value, shown text, unit word).
fn quantity(what: &str, e: &(&'static str, Rows)) -> Option<(String, f64, String)> {
    let what = what.trim().trim_start_matches("the ").trim_start_matches("annual ").trim_start_matches("number of ").trim();
    let first = |f: &str, s: &str| -> Option<f64> { number(values(&e.1, f, s).first()?.1) };
    let pop = first("Population", "total");
    let count = |label: &str, v: f64| Some((label.to_string(), v, format!("about {} per year", super::geo::fmt_int(v.abs().round() as u64))));
    match what {
        "deaths" | "deaths per year" | "yearly deaths" => count("deaths", first("Death rate", "=")? * pop? / 1000.0),
        "births" | "births per year" | "yearly births" | "babies born" => count("births", first("Birth rate", "=")? * pop? / 1000.0),
        "net migration" | "net migrants" | "migrants" => {
            let r = values(&e.1, "Net migration rate", "=").first()?.1;
            let sign = if r.trim_start().starts_with('-') { -1.0 } else { 1.0 };
            let v = sign * number(r)? * pop? / 1000.0;
            Some(("net migration".into(), v, format!("{}about {} per year", if v < 0.0 { "−" } else { "" }, super::geo::fmt_int(v.abs().round() as u64))))
        }
        "female population" | "females" | "women" | "number of women" => Some(("female population".into(), first("Population", "female")?, format!("{} people", super::geo::fmt_int(first("Population", "female")? as u64)))),
        "male population" | "males" | "men" | "number of men" => Some(("male population".into(), first("Population", "male")?, format!("{} people", super::geo::fmt_int(first("Population", "male")? as u64)))),
        "pop" | "population" | "total population" => Some(("population".into(), pop?, format!("{} people", super::geo::fmt_int(pop? as u64)))),
        "trade surplus" | "trade balance" | "trade deficit" | "balance of trade" => {
            let v = first("Exports", "=")? - first("Imports", "=")?;
            let (x, w) = if v.abs() >= 1e12 { (v / 1e12, " trillion") } else if v.abs() >= 1e9 { (v / 1e9, " billion") } else { (v / 1e6, " million") };
            Some(("trade balance (exports − imports)".into(), v, format!("{}${:.1}{w} ({})", if v < 0.0 { "−" } else { "" }, x.abs(), if v < 0.0 { "deficit" } else { "surplus" })))
        }
        _ => {
            let p = prop(what)?;
            let t = values(&e.1, p.1, p.2).first()?.1;
            Some((p.3.to_string(), number(t)?, t.to_string()))
        }
    }
}

/// "deaths Japan", "Norway trade surplus", "Japan female population / male population", "net migration of spain / population of spain"
fn derived(l: &str) -> Option<Answer> {
    const D: &str = r"(?:number of )?(?:deaths(?: per year)?|yearly deaths|births(?: per year)?|yearly births|babies born|net migration|net migrants|female population|male population|females|males|trade surplus|trade balance|trade deficit|balance of trade)";
    let side = |s: &str, dflt: Option<&'static (&'static str, Rows)>| -> Option<(&'static (&'static str, Rows), String)> {
        let s = s.trim();
        if let Some(m) = re!(r"^(.+?) (?:of|in|for) (.+)$").captures(s) {
            if let Some(e) = entity(&m[2]) {
                return Some((e, m[1].to_string()));
            }
        }
        let w: Vec<&str> = s.split(' ').collect();
        for k in 1..w.len() {
            if let Some(e) = entity(&w[..k].join(" ")) {
                return Some((e, w[k..].join(" ")));
            }
            if let Some(e) = entity(&w[k..].join(" ")) {
                return Some((e, w[..k].join(" ")));
            }
        }
        Some((dflt?, s.to_string()))
    };
    if let Some((a, b)) = l.split_once(" / ") {
        // only ratios involving a derived count; plain Factbook ratios are handled elsewhere
        if !Regex::new(&format!(r"\b{D}\b")).ok()?.is_match(l) {
            return None;
        }
        let (ea, wa) = side(a, None)?;
        let (eb, wb) = side(b, Some(ea))?;
        let (la, va, ta) = quantity(&wa, ea)?;
        let (lb, vb, tb) = quantity(&wb, eb)?;
        // a percentage or share divided by a count (or the reverse) is meaningless
        if vb == 0.0 || ta.contains('%') != tb.contains('%') {
            return None;
        }
        let r = va / vb;
        return Some(Answer::new("country data ratio", vec![
            input(format!("({} {la}) / ({} {lb})", ea.0, eb.0)),
            primary(pod("Result", "Result", text(format!("{}", fmt_ratio(r))))),
            pod("Table", "Values", table(None, vec![vec![text(format!("{} {la}", ea.0)), text(ta)], vec![text(format!("{} {lb}", eb.0)), text(tb)]])),
            source(),
        ]));
    }
    let (e, what) = Regex::new(&format!(r"^({D}) (?:of |in |for )?(?:the )?(.+)$")).ok()?.captures(l).map(|m| (m[2].to_string(), m[1].to_string()))
        .or_else(|| Regex::new(&format!(r"^(.+?)(?:'s)? ({D})$")).ok()?.captures(l).map(|m| (m[1].to_string(), m[2].to_string())))?;
    let e = entity(&e)?;
    let (label, _, t) = quantity(&what, e)?;
    Some(Answer::new("country data", vec![
        input(format!("{} | {label}", e.0)),
        primary(pod("Result", "Result", text(t))),
        pod("Note", "Note", text(match label.as_str() {
            "deaths" | "births" | "net migration" => "estimated from the Factbook rate per 1,000 people times the population",
            _ => "from the Factbook values",
        })),
        source(),
    ]))
}

fn fmt_ratio(r: f64) -> String {
    if r.abs() >= 100.0 { format!("{:.0}", r) } else if r.abs() >= 0.01 { format!("{:.4}", r).trim_end_matches('0').trim_end_matches('.').to_string() } else { format!("{:.3e}", r) }
}

/// "European countries with unemployment < Denmark"
fn threshold(l: &str) -> Option<Answer> {
    let m = re!(r"^(african|european|asian|south american|north american|oceanian)? ?countries with (?:an? |the )?(.+?) (<|>|less than|lower than|below|under|greater than|higher than|above|more than|over) (.+)$").captures(l)?;
    let cont = m.get(1).map(|c| continent_name(c.as_str())).flatten();
    let p = prop(&m[2])?;
    let below = matches!(&m[3], "<" | "less than" | "lower than" | "below" | "under");
    let rhs = m[4].trim().trim_start_matches("that of ").trim_start_matches("the ");
    let (limit, against) = match rhs.trim_end_matches('%').trim().parse::<f64>() {
        Ok(x) => (x, rhs.to_string()),
        Err(_) => {
            let e = entity(rhs)?;
            let t = values(&e.1, p.1, p.2).first()?.1;
            (number(t)?, format!("{} ({t})", e.0))
        }
    };
    let sovereign: std::collections::HashSet<String> = countries().values().filter(|c| c.population > 0 && cont.is_none_or(|k| c.continent == k)).map(|c| c.name.to_lowercase()).collect();
    let mut v: Vec<(&str, f64, &str)> = data().iter().filter(|(k, e)| sovereign.contains(*k) && !e.1.iter().any(|r| r.0 == "Dependency status")).filter_map(|(_, e)| {
        let t = values(&e.1, p.1, p.2).first()?.1;
        Some((e.0, number(t)?, t))
    }).filter(|x| if below { x.1 < limit } else { x.1 > limit }).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.1.total_cmp(&b.1));
    let scope = cont.map_or_else(|| "countries".to_string(), |k| format!("countries in {k}"));
    Some(Answer::new("country filter", vec![
        input(format!("{scope} | {} {} {against}", p.3, if below { "<" } else { ">" })),
        primary(pod("Result", "Countries", text(v.iter().map(|x| x.0).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Values", table(Some(vec!["country", p.3]), v.iter().map(|x| vec![text(x.0), text(x.2)]).collect())),
        source(),
    ]))
}

/// "countries with the highest inflation", "lowest life expectancy in the world"
fn ranking(l: &str) -> Option<Answer> {
    // "African country with highest life expectancy", "countries with the highest inflation in Europe"
    let (l, cont) = match re!(r"^(?:which |what )?(african|european|asian|south american|north american|oceanian) (.+)$").captures(l) {
        Some(m) => (m[2].to_string(), Some(continent_name(&m[1])?)),
        None => match re!(r"^(.+?) in (africa|europe|asia|south america|north america|oceania)$").captures(l) {
            Some(m) if m[1].contains("countr") => (m[1].to_string(), Some(continent_name(&m[2])?)),
            _ => (l.to_string(), None),
        },
    };
    let l = l.as_str();
    let m = re!(r"^(?:which |what )?(?:countries|country|nations?)?\s*(?:with|has|have)?\s*(?:the )?(highest|largest|biggest|most|lowest|smallest|least|longest|shortest|top \d+)\s+(.+?)(?: in the world| worldwide| of all countries| by country)?$").captures(l)
        .or_else(|| re!(r"^(?:countries|nations) (?:ranked |sorted |ordered )?by (.+?)()$").captures(l).map(|c| c))?;
    let (dir, what) = if m.get(2).map_or(false, |x| !x.as_str().is_empty()) { (m[1].to_string(), m[2].to_string()) } else { ("highest".to_string(), m[1].to_string()) };
    if !(l.contains("countr") || l.contains("nation") || l.contains("world")) {
        return None;
    }
    let p = prop(what.trim_start_matches("in "))?;
    let low = ["lowest", "smallest", "least", "shortest"].contains(&dir.as_str());
    let n = dir.strip_prefix("top ").and_then(|x| x.parse().ok()).unwrap_or(10usize).min(50);
    let sovereign: std::collections::HashSet<String> = countries().values().filter(|c| c.population > 0 && cont.is_none_or(|k| c.continent == k)).map(|c| c.name.to_lowercase()).collect();
    let mut v: Vec<(&str, f64, &str)> = data()
        .iter()
        .filter(|(k, _)| sovereign.contains(*k))
        .filter(|(_, e)| cont.is_none() || !e.1.iter().any(|r| r.0 == "Dependency status"))
        .filter_map(|(_, e)| {
            let t = values(&e.1, p.1, p.2).first()?.1;
            Some((e.0, number(t)?, t))
        })
        .collect();
    if v.len() < if cont.is_some() { 3 } else { 5 } {
        return None;
    }
    v.sort_by(|a, b| if low { a.1.total_cmp(&b.1) } else { b.1.total_cmp(&a.1) });
    let where_ = cont.map_or_else(|| "countries".to_string(), |k| format!("countries in {k}"));
    let rows = v.iter().take(n).enumerate().map(|(i, x)| vec![text(format!("{}", i + 1)), text(x.0), text(x.2)]).collect();
    Some(Answer::new("country ranking", vec![
        input(format!("{where_} | {} | {}", p.3, if low { "lowest" } else { "highest" })),
        primary(pod("Result", &format!("{} {}", if low { "Lowest" } else { "Highest" }, p.3), text(format!("{} ({})", v[0].0, v[0].2)))),
        pod("Table", &format!("Top {n}"), table(Some(vec!["rank", "country", p.3]), rows)),
        source(),
    ]))
}

/// Summary of one topic: "Russia vs China economy", "United Kingdom rail transport".
const TOPICS: &[(&str, &[&str])] = &[
    ("economy|economic data|economic indicators|economics|economy statistics", &["GDP (official exchange rate)", "Real GDP (purchasing power parity)", "Real GDP growth rate", "Real GDP per capita", "Inflation rate (consumer prices)", "Unemployment rate", "Public debt", "Exports", "Imports"]),
    ("demographics|demography|demographic data|people|population statistics|society", &["Population", "Median age", "Population growth rate", "Birth rate", "Death rate", "Total fertility rate", "Life expectancy at birth", "Urbanization"]),
    ("health|health statistics|healthcare|health care", &["Life expectancy at birth", "Infant mortality rate", "Maternal mortality ratio", "Health expenditure", "Physician density", "Hospital bed density", "Obesity - adult prevalence rate"]),
    ("energy|energy statistics|electricity|power", &["Electricity", "Electricity access", "Electricity generation sources", "Petroleum", "Natural gas", "Coal", "Energy consumption per capita"]),
    ("communications|telecommunications|telecom", &["Telephones - fixed lines", "Telephones - mobile cellular", "Internet users", "Broadband - fixed subscriptions", "Internet country code"]),
    ("transportation|transport|transportation statistics|infrastructure", &["Airports", "Heliports", "Railways", "Merchant marine", "Ports"]),
    ("rail transport|rail|railroads|railway transport|rail transportation", &["Railways"]),
    ("air transport|air transportation|aviation", &["Airports", "Heliports", "Civil aircraft registration country code prefix"]),
    ("water transport|maritime transport|shipping", &["Merchant marine", "Ports"]),
    ("military|armed forces|defense|defence|military statistics", &["Military expenditures", "Military and security service personnel strengths", "Military service age and obligation"]),
    ("government|politics|political data", &["Government type", "Capital", "Independence", "National holiday", "Legal system", "Suffrage"]),
    ("environment|environmental data", &["Environmental issues", "Carbon dioxide emissions", "Methane emissions", "Waste and recycling", "Total water withdrawal"]),
    ("geography|geographic data|physical geography", &["Location", "Area", "Coastline", "Climate", "Terrain", "Elevation", "Natural resources"]),
];

fn topic_rows(rows: &Rows, fields: &[&str]) -> Vec<(String, &'static str)> {
    let mut out = vec![];
    for f in fields {
        let v: Vec<_> = rows.iter().filter(|r| r.0 == *f).collect();
        let take = if v.iter().any(|r| r.1.is_empty()) { v.iter().filter(|r| r.1.is_empty()).take(1).collect::<Vec<_>>() } else { v.iter().take(4).collect() };
        for r in take {
            out.push((if r.1.is_empty() { f.to_string() } else { format!("{f}: {}", r.1) }, r.2));
        }
    }
    out
}

fn topic(l: &str) -> Option<Answer> {
    for (names, fields) in TOPICS {
        for t in names.split('|') {
            let rest = l.strip_suffix(&format!(" {t}")).or_else(|| l.strip_prefix(&format!("{t} of "))).or_else(|| l.strip_prefix(&format!("{t} in "))).or_else(|| l.strip_prefix(&format!("{t} ")));
            let Some(rest) = rest else { continue };
            let rest = rest.trim().trim_end_matches("'s").trim();
            let ents: Vec<_> = re!(r"\s*(?:,\s*(?:and\s+)?|\s+vs\.?\s+|\s+versus\s+|\s+and\s+)\s*").split(rest).map(entity).collect::<Option<_>>()?;
            if ents.len() == 1 {
                let e = ents[0];
                let r = topic_rows(&e.1, fields);
                if r.is_empty() {
                    return None;
                }
                let main = r.iter().take(3).map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(" | ");
                return Some(Answer::new("country data", vec![
                    input(format!("{} | {t}", e.0)),
                    primary(pod("Result", &capitalise(t), text(main))),
                    pod("Table", "Details", table(None, r.iter().map(|(k, v)| vec![text(k.clone()), text(*v)]).collect())),
                    source(),
                ]));
            }
            let all: Vec<Vec<(String, &str)>> = ents.iter().map(|e| topic_rows(&e.1, fields)).collect();
            let mut keys: Vec<String> = vec![];
            for r in &all {
                for (k, _) in r {
                    if !keys.contains(k) {
                        keys.push(k.clone());
                    }
                }
            }
            let rows = keys.iter().map(|k| std::iter::once(text(k.clone())).chain(all.iter().map(|r| text(r.iter().find(|x| &x.0 == k).map_or("—", |x| x.1)))).collect()).collect();
            let mut header = vec![""];
            header.extend(ents.iter().map(|e| e.0));
            let main = keys.first().map(|k| ents.iter().zip(&all).map(|(e, r)| format!("{}: {}", e.0, r.iter().find(|x| &x.0 == k).map_or("—", |x| x.1))).collect::<Vec<_>>().join(" | ")).unwrap_or_default();
            return Some(Answer::new("country comparison", vec![
                input(format!("{} | {t}", ents.iter().map(|e| e.0).collect::<Vec<_>>().join(" vs "))),
                primary(pod("Result", &capitalise(keys.first().map_or(t, |k| k.as_str())), text(main))),
                pod("Table", "Comparison", table(Some(header), rows)),
                source(),
            ]));
        }
    }
    None
}

/// Split "A, B and C" / "A vs B" into entities (at least one).
fn entities(s: &str) -> Option<Vec<&'static (&'static str, Rows)>> {
    let s = s.trim().trim_end_matches("'s").trim();
    if let Some(e) = entity(s) {
        return Some(vec![e]);
    }
    re!(r"\s*(?:,\s*(?:and\s+)?|\s+vs\.?\s+|\s+versus\s+|\s+compared (?:to|with)\s+|\s+and\s+)\s*").split(s).map(|x| entity(x.trim_end_matches("'s"))).collect()
}

/// "world gdp per capita", "EU GDP": nominal and purchasing-power figures for
/// the aggregates that the per-country GDP tables do not cover.
fn aggregate_gdp(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(?:the )?(world|global|european union|eu)(?:'s)? (gdp per capita|gdp per person|gdp)|(gdp per capita|gdp per person|gdp) (?:of |in |for )?(?:the )?(world|european union|eu))$").captures(l)?;
    let who = m.get(1).or(m.get(4))?.as_str();
    let per = m.get(2).or(m.get(3))?.as_str() != "gdp";
    let e = entity(if who == "global" { "world" } else { who })?;
    let first = |f: &str, sub: &str| values(&e.1, f, sub).first().map(|x| x.1);
    let nominal = first("GDP (official exchange rate)", "")?;
    let pop = first("Population", "total").and_then(number);
    let (title, main) = if per {
        let n = number(nominal)?;
        let p = pop?;
        ("GDP per capita (nominal)", format!("${} per person (2024 est.)", super::geo::fmt_int(((n / p / 10.0).round() * 10.0) as u64)))
    } else {
        ("GDP (nominal)", nominal.to_string())
    };
    let mut rows = vec![vec![text("GDP at official exchange rates"), text(nominal)]];
    if let Some(v) = first("Real GDP (purchasing power parity)", "") {
        rows.push(vec![text("GDP at purchasing power parity"), text(v)]);
    }
    if let Some(v) = first("Real GDP per capita", "") {
        rows.push(vec![text("GDP per capita at purchasing power parity"), text(v)]);
    }
    if let Some(v) = first("Real GDP growth rate", "") {
        rows.push(vec![text("real growth rate"), text(v)]);
    }
    if let Some(v) = first("Population", "total") {
        rows.push(vec![text("population"), text(v)]);
    }
    Some(Answer::new("country data", vec![
        input(format!("{} | {}", e.0, if per { "GDP per capita" } else { "GDP" })),
        primary(pod("Result", title, text(main))),
        pod("Table", "Economy", table(None, rows)),
        pod("Note", "Note", text(if per { "nominal GDP at official exchange rates divided by the population; the purchasing-power figure is the Factbook's" } else { "the Factbook's latest estimate" })),
        source(),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let mut l = l.as_str();
    for p in ["what is the ", "what's the ", "what are the ", "what is ", "what are ", "show me the ", "show the ", "tell me the ", "the "] {
        if let Some(r) = l.strip_prefix(p) {
            l = r;
        }
    }
    // "how much does Ireland spend on education", "US education spending vs GDP"
    let rewritten;
    if let Some(m) = re!(r"^how much does (?:the )?(.+?) spend on (education|health|health care|healthcare|the military|military|defense|defence)$").captures(l).or_else(|| re!(r"^(.+?) (education|health|healthcare|military|defense) spending(?: vs\.? gdp| as a share of gdp| % of gdp| percent of gdp)$").captures(l)) {
        let what = match &m[2] { "education" => "education expenditure", "health" | "health care" | "healthcare" => "health expenditure", _ => "military expenditures" };
        rewritten = format!("{what} of {}", &m[1]);
        l = rewritten.as_str();
    }
    if let Some(a) = ranking(l).or_else(|| threshold(l)).or_else(|| continent(l)).or_else(|| derived(l)) {
        return Some(a);
    }
    if let Some(a) = aggregate_gdp(l) {
        return Some(a);
    }
    if let Some(a) = topic(l) {
        return Some(a);
    }
    // a bare entity the other domains do not cover: "European Union", "World"
    if let Some(e) = ["european union", "eu", "the european union", "world"].iter().find(|x| **x == l).and_then(|x| entity(x)) {
        let r = topic_rows(&e.1, &["Population", "Area", "Real GDP (purchasing power parity)", "Real GDP per capita", "Life expectancy at birth", "Languages", "Religions"]);
        return Some(Answer::new("country data", vec![
            input(e.0),
            primary(pod("Result", "Summary", text(r.iter().take(2).map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Facts", table(None, r.iter().map(|(k, v)| vec![text(k.clone()), text(*v)]).collect())),
            source(),
        ]));
    }
    // "PROP of/in A", "A's PROP", "A PROP", "PROP A, B, C"
    let tries: Vec<(String, String)> = {
        let mut t = vec![];
        if let Some(m) = re!(r"^(.+?) (?:of|in|for) (.+)$").captures(l) {
            t.push((m[1].to_string(), m[2].to_string()));
        }
        if let Some(m) = re!(r"^(.+) (?:of|in|for) (.+)$").captures(l) {
            t.push((m[1].to_string(), m[2].to_string()));
        }
        if let Some(m) = re!(r"^(.+?)'s (.+)$").captures(l) {
            t.push((m[2].to_string(), m[1].to_string()));
        }
        let w: Vec<&str> = l.split(' ').collect();
        for k in 1..w.len() {
            t.push((w[..k].join(" "), w[k..].join(" ")));
            t.push((w[k..].join(" "), w[..k].join(" ")));
        }
        t
    };
    for (ps, es) in tries {
        let Some(p) = prop(&ps) else { continue };
        let Some(ents) = entities(&es) else { continue };
        let ans = if ents.len() == 1 { single(ents[0].0, &ents[0].1, p) } else { several(&ents, p) };
        if ans.is_some() {
            return ans;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_loads() {
        assert!(data().len() > 250);
        let us = entity("United States").unwrap();
        assert!(!values(&us.1, "Life expectancy at birth", "total population").is_empty());
    }
}
