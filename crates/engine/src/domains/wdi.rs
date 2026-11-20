//! World Development Indicators (World Bank, CC BY 4.0) for countries,
//! World Bank regions and continents: "2005 lending interest rate in
//! Argentina", "self-employed workers in Canada", "R&D spending in Europe",
//! "migrant population of usa vs migrant population of canada",
//! "world capital investment / world population".

use super::geo::{countries, find_country};
use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

static WDI_Z: &[u8] = include_bytes!("../../data/wdi.z");

type Obs = BTreeMap<u16, f64>;

/// indicator code -> economy ISO3 -> yearly values
fn data() -> &'static HashMap<&'static str, HashMap<&'static str, Obs>> {
    static D: OnceLock<HashMap<&'static str, HashMap<&'static str, Obs>>> = OnceLock::new();
    D.get_or_init(|| {
        let raw = miniz_oxide::inflate::decompress_to_vec(WDI_Z).unwrap_or_default();
        let s: &'static str = Box::leak(String::from_utf8(raw).unwrap_or_default().into_boxed_str());
        let mut d: HashMap<&'static str, HashMap<&'static str, Obs>> = HashMap::new();
        let mut cur = "";
        for line in s.lines() {
            if let Some(h) = line.strip_prefix('#') {
                cur = h.split('\t').next().unwrap_or("");
                continue;
            }
            let mut f = line.split('\t');
            let (Some(iso), Some(_), Some(vals)) = (f.next(), f.next(), f.next()) else { continue };
            let obs: Obs = vals.split(' ').filter_map(|p| {
                let (y, v) = p.split_once('=')?;
                Some((y.parse().ok()?, v.parse().ok()?))
            }).collect();
            d.entry(cur).or_default().insert(iso, obs);
        }
        d
    })
}

#[derive(Clone, Copy, PartialEq)]
enum U {
    Pct,
    PctFirms,
    Usd,
    Lcu,
    People,
    Count(&'static str),
    Index,
    Rating,
    Years,
}

#[derive(Clone, Copy, PartialEq)]
enum Agg {
    Sum,
    /// weighted mean, weighted by another metric
    W(&'static str),
    No,
}

struct M {
    pat: &'static str,
    id: &'static str,
    label: &'static str,
    unit: U,
    agg: Agg,
}

const POP: &str = "SP.POP.TOTL";
const GDP: &str = "NY.GDP.MKTP.CD";
const LAB: &str = "SL.TLF.TOTL.IN";

const METRICS: &[M] = &[
    M { pat: r"lending (?:interest )?rates?", id: "FR.INR.LEND", label: "lending interest rate", unit: U::Pct, agg: Agg::W(POP) },
    M { pat: r"(?:inventory|inventories) changes? (?:using |in )?(?:the )?local currency|changes? in inventor(?:y|ies) (?:using |in )?(?:the )?local currency", id: "NE.GDI.STKB.CN", label: "changes in inventories (local currency)", unit: U::Lcu, agg: Agg::No },
    M { pat: r"(?:inventory|inventories) changes?|changes? in inventor(?:y|ies)", id: "NE.GDI.STKB.CD", label: "changes in inventories", unit: U::Usd, agg: Agg::Sum },
    M { pat: r"(?:total |gross )?savings as an? (?:percent|percentage|fraction|share) of (?:the )?(?:gni|gross national income)|gross savings", id: "NY.GNS.ICTR.GN.ZS", label: "gross savings (% of GNI)", unit: U::Pct, agg: Agg::W(GDP) },
    M { pat: r"gross domestic savings(?: as an? (?:percent|percentage|fraction|share) of (?:the )?(?:gross domestic product|gdp))?", id: "NY.GDS.TOTL.ZS", label: "gross domestic savings (% of GDP)", unit: U::Pct, agg: Agg::W(GDP) },
    M { pat: r"informal payments(?: to public officials)?", id: "IC.FRM.CORR.ZS", label: "firms making informal payments to public officials", unit: U::PctFirms, agg: Agg::No },
    M { pat: r"new business density", id: "IC.BUS.NDNS.ZS", label: "new business density (per 1,000 people aged 15–64)", unit: U::Index, agg: Agg::No },
    M { pat: r"(?:number of )?(?:new )?(?:businesses registered|registered businesses|business registrations)|(?:number (?:of )?)?businesses", id: "IC.BUS.NREG", label: "new businesses registered", unit: U::Count("businesses"), agg: Agg::Sum },
    M { pat: r"(?:index of )?domestic absorptions?|gross national expenditures?", id: "NE.DAB.TOTL.CD", label: "gross national expenditure (domestic absorption)", unit: U::Usd, agg: Agg::Sum },
    M { pat: r"net national income including adjustments|adjusted net national income", id: "NY.ADJ.NNTY.CD", label: "adjusted net national income", unit: U::Usd, agg: Agg::Sum },
    M { pat: r"(?:household|private) (?:final )?consumption(?: expenditures?)?", id: "NE.CON.PRVT.CD", label: "household final consumption expenditure", unit: U::Usd, agg: Agg::Sum },
    M { pat: r"government (?:final )?consumption(?: expenditures?)?", id: "NE.CON.GOVT.CD", label: "government final consumption expenditure", unit: U::Usd, agg: Agg::Sum },
    M { pat: r"capital investment|gross capital formation", id: "NE.GDI.TOTL.CD", label: "gross capital formation", unit: U::Usd, agg: Agg::Sum },
    M { pat: r"(?:measles|mcv) (?:immuni[sz]ation|vaccination)s?(?: rate| coverage)?|(?:immuni[sz]ation|vaccination)s? (?:against |for )?measles", id: "SH.IMM.MEAS", label: "measles immunization (% of children aged 12–23 months)", unit: U::Pct, agg: Agg::W("SP.POP.0014.TO") },
    M { pat: r"(?:adult )?literacy(?: rate)?", id: "SE.ADT.LITR.ZS", label: "adult literacy rate", unit: U::Pct, agg: Agg::W(POP) },
    M { pat: r"primary (?:school )?completion(?: rate)?", id: "SE.PRM.CMPT.ZS", label: "primary school completion rate", unit: U::Pct, agg: Agg::W("SE.PRM.ENRL") },
    M { pat: r"(?:what )?(?:fraction|percentage|share|percent) of (?:high school|secondary(?: school)?) teachers (?:who )?are male|male (?:high school|secondary(?: school)?) teachers(?: fraction)?", id: "MALE_TCH", label: "male secondary school teachers", unit: U::Pct, agg: Agg::W("SE.SEC.TCHR") },
    M { pat: r"(?:what )?(?:fraction|percentage|share|percent) of (?:high school|secondary(?: school)?) teachers (?:who )?are (?:female|women)|female (?:high school|secondary(?: school)?) teachers(?: fraction)?", id: "SE.SEC.TCHR.FE.ZS", label: "female secondary school teachers", unit: U::Pct, agg: Agg::W("SE.SEC.TCHR") },
    M { pat: r"(?:high school|secondary(?: school)?) teachers", id: "SE.SEC.TCHR", label: "secondary school teachers", unit: U::People, agg: Agg::Sum },
    M { pat: r"primary (?:school )?teachers", id: "SE.PRM.TCHR", label: "primary school teachers", unit: U::People, agg: Agg::Sum },
    M { pat: r"part[- ]time employment(?: fraction| rate| share)?|part[- ]time workers", id: "SL.TLF.PART.ZS", label: "part-time employment (% of employment)", unit: U::Pct, agg: Agg::W(LAB) },
    M { pat: r"self[- ]employ(?:ed|ment) (?:fraction|rate|percentage|share)", id: "SL.EMP.SELF.ZS", label: "self-employed (% of employment)", unit: U::Pct, agg: Agg::W(LAB) },
    M { pat: r"self[- ]employed(?: workers| people| persons)?", id: "SELF_N", label: "self-employed workers", unit: U::People, agg: Agg::Sum },
    M { pat: r"(?:contributing )?family workers (?:fraction|rate|percentage|share)", id: "SL.FAM.WORK.ZS", label: "contributing family workers (% of employment)", unit: U::Pct, agg: Agg::W(LAB) },
    M { pat: r"(?:contributing )?family workers", id: "FAM_N", label: "contributing family workers", unit: U::People, agg: Agg::Sum },
    M { pat: r"employment by (?:sector|industry)|sectoral employment|employment sectors", id: "SECTOR", label: "employment by sector", unit: U::Pct, agg: Agg::W(LAB) },
    M { pat: r"(?:industry|industrial) employment|employment in industry", id: "SL.IND.EMPL.ZS", label: "employment in industry (% of employment)", unit: U::Pct, agg: Agg::W(LAB) },
    M { pat: r"(?:agriculture|agricultural|farm) employment|employment in agriculture", id: "SL.AGR.EMPL.ZS", label: "employment in agriculture (% of employment)", unit: U::Pct, agg: Agg::W(LAB) },
    M { pat: r"(?:services?|service sector) employment|employment in services", id: "SL.SRV.EMPL.ZS", label: "employment in services (% of employment)", unit: U::Pct, agg: Agg::W(LAB) },
    M { pat: r"employment to population ratio", id: "SL.EMP.TOTL.SP.ZS", label: "employment to population ratio (ages 15+)", unit: U::Pct, agg: Agg::W(POP) },
    M { pat: r"labou?r force", id: "SL.TLF.TOTL.IN", label: "labor force", unit: U::People, agg: Agg::Sum },
    M { pat: r"employment|employed (?:people|persons|population)|number employed", id: "EMP_N", label: "employment", unit: U::People, agg: Agg::Sum },
    M { pat: r"(?:r ?& ?d|research and development) (?:spending|expenditures?|investment)", id: "GB.XPD.RSDV.GD.ZS", label: "research and development expenditure (% of GDP)", unit: U::Pct, agg: Agg::W(GDP) },
    M { pat: r"(?:international )?(?:visitors|tourists|tourist arrivals)(?: to)?|tourism", id: "ST.INT.ARVL", label: "international tourist arrivals", unit: U::Count("arrivals"), agg: Agg::Sum },
    M { pat: r"livestock(?: production)?(?: index)?", id: "AG.PRD.LVSK.XD", label: "livestock production index (2014–2016 = 100)", unit: U::Index, agg: Agg::No },
    M { pat: r"food production(?: index)?", id: "AG.PRD.FOOD.XD", label: "food production index (2014–2016 = 100)", unit: U::Index, agg: Agg::No },
    M { pat: r"gender equality(?: rating)?", id: "IQ.CPA.GNDR.XQ", label: "CPIA gender equality rating (1 = low to 6 = high)", unit: U::Rating, agg: Agg::No },
    M { pat: r"(?:(?:kids|children|pupils|students) (?:are )?)?repeating primary school|primary (?:school )?repeaters", id: "REP_N", label: "primary school repeaters", unit: U::People, agg: Agg::Sum },
    M { pat: r"primary (?:school )?repetition rate", id: "SE.PRM.REPT.ZS", label: "primary school repeaters (% of enrollment)", unit: U::Pct, agg: Agg::W("SE.PRM.ENRL") },
    M { pat: r"female life expectancy|life expectancy (?:of )?(?:women|females)", id: "SP.DYN.LE00.FE.IN", label: "female life expectancy at birth", unit: U::Years, agg: Agg::W(POP) },
    M { pat: r"male life expectancy|life expectancy (?:of )?(?:men|males)", id: "SP.DYN.LE00.MA.IN", label: "male life expectancy at birth", unit: U::Years, agg: Agg::W(POP) },
    M { pat: r"(?:hiv|aids) (?:prevalence|fraction|rate)|prevalence of (?:hiv|aids)", id: "SH.DYN.AIDS.ZS", label: "HIV prevalence (% of population aged 15–49)", unit: U::Pct, agg: Agg::W(POP) },
    M { pat: r"(?:international )?migrant (?:population|stock)|immigrant population|(?:number of )?(?:immigrants|migrants)", id: "SM.POP.TOTL", label: "international migrant stock", unit: U::People, agg: Agg::Sum },
    M { pat: r"child population|population (?:aged?|ages) 0[-–]14|population under 15", id: "SP.POP.0014.TO", label: "population aged 0–14", unit: U::People, agg: Agg::Sum },
    M { pat: r"(?:crop ?land|arable land)(?: area)?", id: "AG.LND.ARBL.HA", label: "arable land", unit: U::Count("hectares"), agg: Agg::Sum },
    M { pat: r"average income|gni per capita|income per capita|per capita income", id: "NY.GNP.PCAP.CD", label: "GNI per capita (Atlas method)", unit: U::Usd, agg: Agg::W(POP) },
    M { pat: r"population|pop", id: "SP.POP.TOTL", label: "population", unit: U::People, agg: Agg::Sum },
    M { pat: r"gdp|gross domestic product", id: "NY.GDP.MKTP.CD", label: "GDP", unit: U::Usd, agg: Agg::Sum },
];

fn metric(id: &str) -> &'static M {
    METRICS.iter().find(|m| m.id == id).unwrap_or(&METRICS[0])
}

/// Values of a metric for one economy, by year (derived metrics included).
fn country_series(id: &str, iso: &str) -> Obs {
    let raw = |c: &str| data().get(c).and_then(|m| m.get(iso)).cloned().unwrap_or_default();
    let combine = |a: Obs, b: Obs, f: &dyn Fn(f64, f64) -> f64| -> Obs { a.iter().filter_map(|(y, x)| Some((*y, f(*x, *b.get(y)?)))).collect() };
    match id {
        "EMP_N" => combine(raw(LAB), raw("SL.UEM.TOTL.ZS"), &|l, u| l * (1.0 - u / 100.0)),
        "SELF_N" => combine(raw("SL.EMP.SELF.ZS"), country_series("EMP_N", iso), &|p, e| p / 100.0 * e),
        "FAM_N" => combine(raw("SL.FAM.WORK.ZS"), country_series("EMP_N", iso), &|p, e| p / 100.0 * e),
        "REP_N" => combine(raw("SE.PRM.REPT.ZS"), raw("SE.PRM.ENRL"), &|p, e| p / 100.0 * e),
        "MALE_TCH" => raw("SE.SEC.TCHR.FE.ZS").into_iter().map(|(y, v)| (y, 100.0 - v)).collect(),
        _ => raw(id),
    }
}

#[derive(Clone)]
enum Place {
    One(&'static str),
    Group(Vec<&'static str>),
}

const WB_AGGS: &[(&str, &str, &str)] = &[
    ("world", "WLD", "world"),
    ("whole world", "WLD", "world"),
    ("sub-saharan africa", "SSF", "Sub-Saharan Africa"),
    ("east asia & pacific", "EAS", "East Asia & Pacific"),
    ("east asia and pacific", "EAS", "East Asia & Pacific"),
    ("europe & central asia", "ECS", "Europe & Central Asia"),
    ("europe and central asia", "ECS", "Europe & Central Asia"),
    ("latin america", "LCN", "Latin America & Caribbean"),
    ("latin america & caribbean", "LCN", "Latin America & Caribbean"),
    ("latin america and the caribbean", "LCN", "Latin America & Caribbean"),
    ("south asia", "SAS", "South Asia"),
    ("european union", "EUU", "European Union"),
    ("eu", "EUU", "European Union"),
    ("arab world", "ARB", "Arab World"),
    ("high income countries", "HIC", "high income countries"),
];

const GROUPS: &[(&str, &str, &[&str])] = &[
    ("east asia", "East Asia", &["CHN", "JPN", "KOR", "PRK", "MNG", "HKG", "MAC"]),
    ("southeast asia", "Southeast Asia", &["BRN", "KHM", "IDN", "LAO", "MYS", "MMR", "PHL", "SGP", "THA", "VNM", "TLS"]),
    ("south east asia", "Southeast Asia", &["BRN", "KHM", "IDN", "LAO", "MYS", "MMR", "PHL", "SGP", "THA", "VNM", "TLS"]),
    ("asean", "ASEAN", &["BRN", "KHM", "IDN", "LAO", "MYS", "MMR", "PHL", "SGP", "THA", "VNM"]),
    ("nato", "NATO", &["ALB", "BEL", "BGR", "CAN", "HRV", "CZE", "DNK", "EST", "FIN", "FRA", "DEU", "GRC", "HUN", "ISL", "ITA", "LVA", "LTU", "LUX", "MNE", "NLD", "MKD", "NOR", "POL", "PRT", "ROU", "SVK", "SVN", "ESP", "SWE", "TUR", "GBR", "USA"]),
    ("g7", "G7", &["CAN", "FRA", "DEU", "ITA", "JPN", "GBR", "USA"]),
    ("opec", "OPEC", &["DZA", "COG", "GNQ", "GAB", "IRN", "IRQ", "KWT", "LBY", "NGA", "SAU", "ARE", "VEN"]),
    ("central america", "Central America", &["BLZ", "CRI", "SLV", "GTM", "HND", "NIC", "PAN"]),
    ("middle east", "Middle East", &["BHR", "CYP", "EGY", "IRN", "IRQ", "ISR", "JOR", "KWT", "LBN", "OMN", "QAT", "SAU", "SYR", "TUR", "ARE", "YEM", "PSE"]),
    ("scandinavia", "Scandinavia", &["DNK", "NOR", "SWE"]),
];

fn place(s: &str) -> Option<(String, Place)> {
    let s = s.trim().trim_start_matches("the ").trim_end_matches("'s").trim_end_matches('\'').trim();
    if s.is_empty() {
        return None;
    }
    if let Some((_, code, name)) = WB_AGGS.iter().find(|a| a.0 == s) {
        return Some((name.to_string(), Place::One(code)));
    }
    if let Some((_, name, m)) = GROUPS.iter().find(|g| g.0 == s) {
        return Some((name.to_string(), Place::Group(m.to_vec())));
    }
    let cont = match s {
        "africa" | "african" => Some("Africa"),
        "europe" | "european" => Some("Europe"),
        "asia" | "asian" => Some("Asia"),
        "south america" | "south american" => Some("South America"),
        "north america" | "north american" => Some("North America"),
        "oceania" => Some("Oceania"),
        _ => None,
    };
    if let Some(c) = cont {
        let mut m: Vec<&'static str> = countries().values().filter(|x| x.continent == c).map(|x| x.iso3.as_str()).collect();
        m.sort();
        return Some((c.to_string(), Place::Group(m)));
    }
    let s = match s {
        "u.s." | "u.s" | "u.s.a." | "united states of america" => "united states",
        "u.k." | "britain" | "great britain" => "united kingdom",
        x => x,
    };
    let c = find_country(s)?;
    let iso: &'static str = c.iso3.as_str();
    data().get(POP)?.get(iso)?;
    let name = if c.name.starts_with("United ") || c.name.ends_with("Islands") || c.name.starts_with("Netherlands") || c.name.starts_with("Philippines") { format!("the {}", c.name) } else { c.name.clone() };
    Some((name, Place::One(iso)))
}

/// Values for a place: an aggregate over a group when needed. Years where
/// countries holding under 60% of the group's weight report are dropped.
fn series(m: &M, p: &Place) -> Obs {
    match p {
        Place::One(iso) => country_series(m.id, iso),
        Place::Group(members) => {
            if m.agg == Agg::No {
                return Obs::new();
            }
            let wid = match m.agg {
                Agg::W(w) => w,
                _ => POP,
            };
            let mut num: BTreeMap<u16, (f64, f64)> = BTreeMap::new();
            let mut tot: BTreeMap<u16, f64> = BTreeMap::new();
            for iso in members {
                let w = country_series(wid, iso);
                let cover = if wid == POP { w.clone() } else { country_series(POP, iso) };
                for (y, v) in &cover {
                    *tot.entry(*y).or_default() += v;
                }
                let pop = country_series(POP, iso);
                for (y, v) in country_series(m.id, iso) {
                    let Some(pw) = pop.get(&y) else { continue };
                    let e = num.entry(y).or_default();
                    match m.agg {
                        Agg::Sum => {
                            e.0 += v;
                            e.1 += pw;
                        }
                        _ => {
                            let Some(ww) = w.get(&y) else { continue };
                            e.0 += v * ww;
                            e.1 += pw;
                        }
                    }
                }
            }
            // weighted averages need the weight total of the reporting countries
            let mut wsum: BTreeMap<u16, f64> = BTreeMap::new();
            if let Agg::W(w) = m.agg {
                for iso in members {
                    let ww = country_series(w, iso);
                    for (y, _) in country_series(m.id, iso) {
                        if let Some(x) = ww.get(&y) {
                            *wsum.entry(y).or_default() += x;
                        }
                    }
                }
            }
            num.into_iter()
                .filter(|(y, (_, covered))| tot.get(y).is_some_and(|t| *covered >= 0.6 * t))
                .filter_map(|(y, (v, _))| match m.agg {
                    Agg::W(_) => wsum.get(&y).filter(|s| **s > 0.0).map(|s| (y, v / s)),
                    _ => Some((y, v)),
                })
                .collect()
        }
    }
}

fn scaled(x: f64) -> String {
    let (d, w) = if x.abs() >= 1e12 { (1e12, " trillion") } else if x.abs() >= 1e9 { (1e9, " billion") } else if x.abs() >= 1e6 { (1e6, " million") } else { (1.0, "") };
    let v = x / d;
    let num = if w.is_empty() { fmt_float(v.round(), 9) } else { fmt_float((v * 1000.0).round() / 1000.0, 4) };
    format!("{num}{w}")
}

fn show(m: &M, v: f64, p: &Place) -> String {
    match m.unit {
        U::Pct => format!("{}%", fmt_float((v * 100.0).round() / 100.0, 4)),
        U::PctFirms => format!("{}% of firms", fmt_float((v * 100.0).round() / 100.0, 4)),
        U::Usd => format!("${}{}", scaled(v), if m.id == "NY.GNP.PCAP.CD" { " per person per year" } else { " per year" }).replace("$-", "-$"),
        U::Lcu => {
            let cur = match p {
                Place::One(iso) => countries().values().find(|c| c.iso3 == *iso).map(|c| c.currency_code.clone()).unwrap_or_default(),
                _ => String::new(),
            };
            format!("{} {cur} per year", scaled(v))
        }
        U::People => format!("{} people", scaled(v.round())),
        U::Count(w) => format!("{} {w}{}", scaled(v.round()), if m.id == "IC.BUS.NREG" || m.id == "ST.INT.ARVL" { " per year" } else { "" }),
        U::Index | U::Rating => fmt_float((v * 100.0).round() / 100.0, 4),
        U::Years => format!("{} years", fmt_float((v * 10.0).round() / 10.0, 4)),
    }
}

/// Find a metric phrase in a text; returns it with the text left over.
fn find_metric(s: &str) -> Option<(&'static M, String)> {
    static RS: OnceLock<Vec<Regex>> = OnceLock::new();
    let rs = RS.get_or_init(|| METRICS.iter().map(|m| Regex::new(&format!(r"\b(?:{})\b", m.pat)).unwrap()).collect());
    for (m, r) in METRICS.iter().zip(rs) {
        if let Some(x) = r.find(s) {
            let rest = format!("{} {}", &s[..x.start()], &s[x.end()..]);
            return Some((m, rest));
        }
    }
    None
}

/// The places named in what is left of a query part (all must be understood).
fn places(rest: &str) -> Option<Vec<(String, Place)>> {
    let r = re!(r"\b(?:what|which|is|was|are|were|the|total|in|of|for|to|by|from|using|how|many|much|number|people|do|does|did|has|have|there|amount|value|level|country|countries|each|all)\b").replace_all(rest, " ");
    let r = re!(r"[^a-z0-9.&' -]|,").replace_all(&r, ",");
    let r = re!(r"\s+").replace_all(&r, " ");
    let mut out = Vec::new();
    for part in re!(r",| and ").split(r.trim()) {
        let part = part.trim().trim_matches('.').trim();
        if part.is_empty() {
            continue;
        }
        out.push(place(part)?);
    }
    Some(out)
}

struct Part {
    year: Option<u16>,
    metric: Option<&'static M>,
    places: Vec<(String, Place)>,
}

fn parse_part(s: &str) -> Option<Part> {
    let year: Option<u16> = re!(r"\b(19[89]\d|20[0-2]\d)\b").captures(s).and_then(|c| c[1].parse().ok());
    let body = re!(r"\b(?:in |during )?(?:19[89]\d|20[0-2]\d)\b").replace(s, " ").to_string();
    let s = body.as_str();
    let (metric, rest) = match find_metric(s) {
        Some((m, r)) if m.label.contains("% of GDP") || m.label.contains("% of GNI") => (Some(m), re!(r"\bas an? (?:percent|percentage|fraction|share) of (?:the )?(?:gross domestic product|gdp|gross national income|gni)\b").replace(&r, " ").to_string()),
        Some((m, r)) => (Some(m), r),
        None => (None, s.to_string()),
    };
    Some(Part { year, metric, places: places(&rest)? })
}

fn value_at(m: &M, p: &Place, year: Option<u16>) -> Option<(u16, f64)> {
    let s = series(m, p);
    match year {
        Some(y) => s.get(&y).map(|v| (y, *v)),
        None => s.iter().next_back().map(|(y, v)| (*y, *v)),
    }
}

fn source() -> Pod {
    pod("Source", "Source", text("World Bank, World Development Indicators (CC BY 4.0); continent and group figures are sums or weighted means over member countries."))
}

fn history(m: &M, p: &Place, year: Option<u16>) -> Option<Pod> {
    let s = series(m, p);
    let end = year.unwrap_or(*s.keys().next_back()?);
    let rows: Vec<Vec<Content>> = s.range(end.saturating_sub(9)..=end).rev().map(|(y, v)| vec![text(y.to_string()), text(show(m, *v, p))]).collect();
    (rows.len() > 1).then(|| pod("History", "History", table(Some(vec!["year", "value"]), rows)))
}

fn sector(name: &str, p: &Place, year: Option<u16>) -> Option<Answer> {
    let ids = [("agriculture", "SL.AGR.EMPL.ZS"), ("industry", "SL.IND.EMPL.ZS"), ("services", "SL.SRV.EMPL.ZS")];
    let mut yr = 0;
    let rows: Vec<Vec<Content>> = ids.iter().filter_map(|(k, id)| {
        let (y, v) = value_at(metric(id), p, year)?;
        yr = y;
        Some(vec![text(*k), text(format!("{}%", fmt_float((v * 10.0).round() / 10.0, 3)))])
    }).collect();
    if rows.len() < 3 {
        return None;
    }
    Some(Answer::new("world development indicator", vec![
        pod("Input", "Input interpretation", text(format!("{name} | employment by sector"))),
        primary(pod("Result", &format!("Share of total employment ({yr})"), table(Some(vec!["sector", "share"]), rows))),
        source(),
    ]))
}

fn ranking(l: &str) -> Option<Answer> {
    let c = re!(r"^(?:(.+?) (?:of|for|in) )?(?:the )?(?:top (\d+) )?(country|countries|nation|nations) (?:with|by) (?:the )?(?:(greatest|highest|largest|most|biggest|fewest|lowest|smallest|least) )?(?:number (?:of )?)?(.+)$").captures(l)?;
    let (m, rest) = find_metric(&c[5])?;
    if !rest.trim().is_empty() || m.id == "SP.POP.TOTL" || m.id == "NY.GDP.MKTP.CD" || m.id == "SECTOR" {
        return None;
    }
    let low = c.get(4).is_some_and(|w| matches!(w.as_str(), "fewest" | "lowest" | "smallest" | "least"));
    if c.get(4).is_none() && c.get(2).is_none() {
        return None;
    }
    let mut vals: Vec<(&'static str, String, u16, f64)> = countries().values().filter_map(|x| {
        let iso: &'static str = x.iso3.as_str();
        let s = country_series(m.id, iso);
        let (y, v) = s.iter().next_back()?;
        (*y >= 2015).then(|| (iso, x.name.clone(), *y, *v))
    }).collect();
    vals.sort_by(|a, b| if low { a.3.total_cmp(&b.3) } else { b.3.total_cmp(&a.3) });
    let n: usize = c.get(2).and_then(|x| x.as_str().parse().ok()).unwrap_or(if c[3].ends_with('s') { 10 } else { 5 });
    let top = vals.iter().take(n.clamp(1, 50));
    let rows: Vec<Vec<Content>> = top.clone().enumerate().map(|(i, (iso, name, y, v))| vec![text((i + 1).to_string()), text(name.clone()), text(show(m, *v, &Place::One(iso))), text(y.to_string())]).collect();
    let first = vals.first()?;
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("countries | {} | {}", m.label, if low { "lowest" } else { "highest" })))];
    if let Some(of) = c.get(1) {
        let (m2, rest2) = find_metric(of.as_str())?;
        if !rest2.trim().is_empty() {
            return None;
        }
        let p = Place::One(first.0);
        let (y, v) = value_at(m2, &p, None)?;
        pods.push(primary(pod("Result", "Result", text(format!("{}: {} ({y})", first.1, show(m2, v, &p))))));
        pods.push(pod("Country", &format!("Country with the {} {}", if low { "lowest" } else { "highest" }, m.label), text(format!("{}: {} ({})", first.1, show(m, first.3, &p), first.2))));
    } else if !c[3].ends_with('s') && c.get(2).is_none() {
        pods.push(primary(pod("Result", "Result", text(format!("{}: {} ({})", first.1, show(m, first.3, &Place::One(first.0)), first.2)))));
        pods.push(pod("Ranking", "Ranking", table(Some(vec!["rank", "country", "value", "year"]), rows)));
    } else {
        pods.push(primary(pod("Result", "Result", table(Some(vec!["rank", "country", "value", "year"]), rows))));
    }
    pods.push(source());
    Some(Answer::new("world development indicator", pods))
}

/// "France pop 2000 - France pop 1980": a difference, each side with its own year.
fn difference(l: &str) -> Option<Answer> {
    let (a, b) = l.split_once(" - ")?;
    let side = |s: &str| -> Option<(Part, u16)> {
        let y: u16 = re!(r"\b(19[89]\d|20[0-2]\d)\b").captures(s)?[1].parse().ok()?;
        let body = re!(r"\b(?:in |during )?(?:19[89]\d|20[0-2]\d)\b").replace(s, " ").to_string();
        Some((parse_part(&body)?, y))
    };
    let ((mut pa, ya), (mut pb, yb)) = (side(a)?, side(b)?);
    if pa.metric.is_none() {
        pa.metric = pb.metric;
    }
    if pb.metric.is_none() {
        pb.metric = pa.metric;
    }
    if pa.places.is_empty() {
        pa.places = pb.places.clone();
    }
    if pb.places.is_empty() {
        pb.places = pa.places.clone();
    }
    let (ma, mb) = (pa.metric?, pb.metric?);
    if ma.id != mb.id || pa.places.len() != 1 || pb.places.len() != 1 || matches!(ma.unit, U::Index | U::Rating) || ma.id == "SECTOR" {
        return None;
    }
    let ((na, xa), (nb, xb)) = (&pa.places[0], &pb.places[0]);
    let va = *series(ma, xa).get(&ya)?;
    let vb = *series(mb, xb).get(&yb)?;
    Some(Answer::new("world development indicator", vec![
        pod("Input", "Input interpretation", text(format!("({na} {} {ya}) − ({nb} {} {yb})", ma.label, mb.label))),
        primary(pod("Result", "Result", text(show(ma, va - vb, xa).replace(" per year", "")))),
        pod("Values", "Values", table(Some(vec!["quantity", "value"]), vec![
            vec![text(format!("{na} {} {ya}", ma.label)), text(show(ma, va, xa))],
            vec![text(format!("{nb} {} {yb}", mb.label)), text(show(mb, vb, xb))],
        ])),
        source(),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace("u.s.", "us").replace("vs.", "vs");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = re!(r"^(?:what (?:is|was|are|were) |how (?:many|much) (?:people |kids |children )?(?:are |is |were )?|show (?:me )?)").replace(&l, "").to_string();
    if let Some(a) = ranking(&l).or_else(|| difference(&l)) {
        return Some(a);
    }
    let year: Option<u16> = re!(r"\b(19[89]\d|20[0-2]\d)\b").captures(&l).and_then(|c| c[1].parse().ok());
    let body = l.clone();
    let (parts_txt, ratio): (Vec<&str>, bool) = if let Some((a, b)) = body.split_once('/') {
        (vec![a, b], true)
    } else {
        (re!(r"\s(?:vs|versus|compared (?:to|with))\s").split(&body).collect(), false)
    };
    let mut parts: Vec<Part> = parts_txt.iter().map(|p| parse_part(p)).collect::<Option<_>>()?;
    if parts.iter().all(|p| p.metric.is_none()) {
        return None;
    }
    // shared metric or places across the parts
    for i in 0..parts.len() {
        if parts[i].metric.is_none() {
            parts[i].metric = parts.iter().find_map(|p| p.metric);
        }
        if parts[i].places.is_empty() {
            parts[i].places = parts.iter().find(|p| !p.places.is_empty())?.places.clone();
        }
    }
    let mut rows: Vec<(String, &'static M, Place, Option<u16>)> = Vec::new();
    for p in &parts {
        for (n, pl) in &p.places {
            rows.push((n.clone(), p.metric?, pl.clone(), p.year.or(year)));
        }
    }
    let year = if rows.iter().all(|r| r.3 == year) { year } else { None };
    if ratio {
        if parts.len() != 2 || parts[0].places.len() != 1 || parts[1].places.len() != 1 {
            return None;
        }
        let (na, ma, pa, _) = &rows[0];
        let (nb, mb, pb, _) = &rows[1];
        let sa = series(ma, pa);
        let sb = series(mb, pb);
        let y = match year {
            Some(y) => y,
            None => *sa.keys().rev().find(|y| sb.contains_key(y))?,
        };
        let (a, b) = (*sa.get(&y)?, *sb.get(&y)?);
        let r = a / b;
        let rs = if r.abs() >= 1e4 || r.abs() < 1e-3 { scaled(r) } else { fmt_float(r, 4) };
        let unit = if mb.id == POP && matches!(ma.unit, U::Usd) { " US dollars per person" } else if mb.id == POP && !matches!(ma.unit, U::Pct) { " per person" } else { "" };
        return Some(Answer::new("world development indicator", vec![
            pod("Input", "Input interpretation", text(format!("({na} {}) / ({nb} {})", ma.label, mb.label))),
            primary(pod("Result", &format!("Result ({y})"), text(format!("{rs}{unit}")))),
            pod("Values", "Values", table(Some(vec!["quantity", "value"]), vec![
                vec![text(format!("{na} {}", ma.label)), text(show(ma, a, pa))],
                vec![text(format!("{nb} {}", mb.label)), text(show(mb, b, pb))],
            ])),
            source(),
        ]));
    }
    if rows.len() == 1 {
        let (n, m, p, _) = &rows[0];
        // plain populations are answered by the geography handlers
        if m.id == POP && year.is_none() {
            return None;
        }
        if m.id == "SECTOR" {
            return sector(n, p, year);
        }
        let (y, v) = value_at(m, p, year)?;
        let mut pods = vec![
            pod("Input", "Input interpretation", text(format!("{n} | {}{}", m.label, year.map_or(String::new(), |y| format!(" | {y}"))))),
            primary(pod("Result", "Result", text(format!("{} ({y})", show(m, v, p))))),
        ];
        // a count together with its share of employment
        let share = match m.id {
            "SELF_N" => Some("SL.EMP.SELF.ZS"),
            "FAM_N" => Some("SL.FAM.WORK.ZS"),
            "REP_N" => Some("SE.PRM.REPT.ZS"),
            _ => None,
        };
        if let Some((_, s)) = share.and_then(|id| value_at(metric(id), p, Some(y))) {
            pods.push(pod("Share", "Share", text(format!("{}% of {}", fmt_float((s * 100.0).round() / 100.0, 4), if m.id == "REP_N" { "primary school enrollment" } else { "total employment" }))));
        }
        if let Some(h) = history(m, p, year) {
            pods.push(h);
        }
        pods.push(source());
        return Some(Answer::new("world development indicator", pods));
    }
    if rows.iter().any(|r| r.1.id == "SECTOR") {
        return None;
    }
    let vals: Vec<Vec<Content>> = rows.iter().map(|(n, m, p, year)| {
        let v = value_at(m, p, *year);
        vec![text(n.clone()), text(m.label), text(v.map_or("(no data)".to_string(), |(_, v)| show(m, v, p))), text(v.map_or(String::new(), |(y, _)| y.to_string()))]
    }).collect();
    if rows.iter().all(|(_, m, p, y)| value_at(m, p, *y).is_none()) {
        return None;
    }
    // "population Spain + Portugal": a total
    if l.contains(" + ") && rows.iter().all(|r| r.1.id == rows[0].1.id && r.1.agg == Agg::Sum) {
        let m = rows[0].1;
        let vs: Vec<(u16, f64)> = rows.iter().map(|(_, m, p, y)| value_at(m, p, *y)).collect::<Option<_>>()?;
        let total: f64 = vs.iter().map(|v| v.1).sum();
        let names: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
        let p0 = rows[0].2.clone();
        return Some(Answer::new("world development indicator", vec![
            pod("Input", "Input interpretation", text(format!("{} | {} | total", names.join(" + "), m.label))),
            primary(pod("Result", "Result", text(show(m, total, &p0)))),
            pod("Values", "Values", table(Some(vec!["economy", "value", "year"]), vals)),
            source(),
        ]));
    }
    let names: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
    let mut labels: Vec<&str> = rows.iter().map(|r| r.1.label).collect();
    labels.dedup();
    Some(Answer::new("world development indicator", vec![
        pod("Input", "Input interpretation", text(format!("{} | {}", names.join(", "), labels.join(", ")))),
        primary(pod("Result", "Result", table(Some(vec!["economy", "indicator", "value", "year"]), vals))),
        source(),
    ]))
}
