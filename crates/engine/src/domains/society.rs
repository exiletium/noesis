//! Countries, languages and religions: country lookup by ISO code or
//! internet domain, country comparisons and filters ("countries with
//! population > 125 million"), languages (family, speakers, script,
//! where official), language-speaking countries, world religions and
//! religious composition from national censuses and surveys.

use super::geo::{country_pods, countries, find_country, fmt_int, Country};
use super::knowledge::language_name;
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

fn n(v: f64) -> String {
    fmt_float(v, 4)
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn rows(r: Vec<(&str, String)>) -> Content {
    table(None, r.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())
}

fn people(v: f64) -> String {
    if v >= 1e9 {
        format!("{} billion", n(v / 1e9))
    } else if v >= 1e6 {
        format!("{} million", n(v / 1e6))
    } else {
        fmt_int(v.round() as u64)
    }
}

fn split_list(s: &str) -> Vec<String> {
    re!(r"\s*(?:,|/|;|\bvs\.?\s|\bversus\b|\band\b|&)\s*").split(s.trim()).map(|x| x.trim().trim_start_matches("and ").to_string()).filter(|x| !x.is_empty()).collect()
}

// ---------------------------------------------------------------------------
// countries
// ---------------------------------------------------------------------------

fn country_by_code(code: &str) -> Option<&'static Country> {
    let c = code.trim().to_lowercase();
    if let Some(tld) = c.strip_prefix('.') {
        return countries().values().find(|x| x.tld.trim_start_matches('.').eq_ignore_ascii_case(tld));
    }
    match c.len() {
        2 => countries().values().find(|x| x.iso2.eq_ignore_ascii_case(&c)),
        3 => countries().values().find(|x| x.iso3.eq_ignore_ascii_case(&c)),
        _ => None,
    }
}

fn country_lookup(l: &str) -> Option<Answer> {
    let c = re!(r"^(?:the )?country (?:with (?:the )?)?(?:(?:iso|internet|domain|country) )?(?:code |domain )?(\.?[a-z]{2,3})$").captures(l).or_else(|| re!(r"^(\.[a-z]{2})(?: (?:domain|tld|country))?$").captures(l))?;
    let country = country_by_code(&c[1])?;
    let mut pods = country_pods(country);
    pods[0] = input(format!("{} ({})", country.name, c[1].to_uppercase()));
    Some(Answer::new(&format!("{} (country)", country.name), pods))
}

fn languages_text(c: &Country) -> String {
    let mut names: Vec<String> = Vec::new();
    for code in c.languages.split(',') {
        if let Some(nm) = language_name(code.trim()) {
            let base = nm.split(" (").next().unwrap_or(&nm).to_string();
            if !names.contains(&base) {
                names.push(base);
            }
        }
    }
    names.join(", ")
}

fn compare_countries(l: &str) -> Option<Answer> {
    let parts = split_list(l);
    if parts.len() < 2 || !(l.contains(',') || l.contains(" vs") || l.contains(" versus ")) {
        return None;
    }
    let cs: Vec<&Country> = parts.iter().map(|p| find_country(p)).collect::<Option<_>>()?;
    let t = cs
        .iter()
        .map(|c| {
            vec![
                text(c.name.clone()),
                text(c.capital.clone()),
                text(fmt_int(c.population)),
                text(format!("{} km²", fmt_int(c.area_km2 as u64))),
                text(format!("{}/km²", fmt_float(c.population as f64 / c.density_area(), 3))),
                text(c.currency_name.clone()),
                text(languages_text(c)),
            ]
        })
        .collect();
    let total: u64 = cs.iter().map(|c| c.population).sum();
    let area: f64 = cs.iter().map(|c| c.area_km2).sum();
    Some(Answer::new(
        "country comparison",
        vec![
            input(cs.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(", ")),
            primary(pod("Table", "Comparison", table(Some(vec!["country", "capital", "population", "area", "density", "currency", "languages"]), t))),
            pod("Totals", "Totals", rows(vec![("population", fmt_int(total)), ("area", format!("{} km²", fmt_int(area as u64)))])),
        ],
    ))
}

fn attr(c: &Country, a: &str) -> f64 {
    match a {
        "area" => c.area_km2,
        "land area" => c.land_km2.unwrap_or(c.area_km2),
        "density" => c.population as f64 / c.density_area(),
        _ => c.population as f64,
    }
}

fn attr_name(a: &str) -> &'static str {
    match a {
        "area" | "size" | "total area" => "area",
        "land area" => "land area",
        "population density" | "density" => "density",
        _ => "population",
    }
}

fn attr_text(c: &Country, a: &str) -> String {
    match a {
        "area" | "land area" => format!("{} km²", fmt_int(attr(c, a) as u64)),
        "density" => format!("{} people/km²", fmt_float(attr(c, a), 4)),
        _ => fmt_int(c.population),
    }
}

/// A threshold such as "125 million", "1,000,000 km2" or "India" (that country's value).
fn threshold(s: &str, a: &str) -> Option<f64> {
    let s = s.trim().trim_start_matches("that of ").trim();
    if let Some(c) = find_country(s) {
        return Some(attr(c, a));
    }
    let c = re!(r"^([\d.,]+)\s*(thousand|million|billion|k|m|bn)?\s*(?:people|inhabitants|km2|km²|square kilometers|square kilometres|sq km|per km2|/km2|/km²)?$").captures(s)?;
    let v: f64 = c[1].replace(',', "").parse().ok()?;
    Some(v * match c.get(2).map(|x| x.as_str()) {
        Some("thousand") | Some("k") => 1e3,
        Some("million") | Some("m") => 1e6,
        Some("billion") | Some("bn") => 1e9,
        _ => 1.0,
    })
}

fn continent_code(w: &str) -> Option<&'static str> {
    Some(match w {
        "european" | "europe" => "EU",
        "asian" | "asia" => "AS",
        "african" | "africa" => "AF",
        "north american" | "north america" => "NA",
        "south american" | "south america" => "SA",
        "oceanian" | "oceania" => "OC",
        _ => return None,
    })
}

fn country_filter(l: &str) -> Option<Answer> {
    let c = re!(r"^(?:which |what |list of |all |the )?(?:(european|asian|african|north american|south american|oceanian) )?countries (?:with|that have|having|where)(?: an?| the)? (.+?)(?: in (europe|asia|africa|north america|south america|oceania))?$").captures(l)?;
    let cont = c.get(1).or(c.get(3)).and_then(|x| continent_code(x.as_str()));
    let mut conds = vec![];
    for part in c[2].split(" and ") {
        let m = re!(r"^(?:an? |the )?(population density|density|land area|total area|area|size|population)(?: is)? (>=|<=|>|<|=|greater than|more than|larger than|bigger than|higher than|over|above|exceeding|less than|smaller than|fewer than|lower than|under|below)(?: or equal to)? (.+)$").captures(part.trim())?;
        let a = attr_name(&m[1]);
        let op = match &m[2] {
            ">" | "greater than" | "more than" | "larger than" | "bigger than" | "higher than" | "over" | "above" | "exceeding" => ">",
            ">=" => ">=",
            "<=" => "<=",
            "=" => "=",
            _ => "<",
        };
        conds.push((a, op, threshold(&m[3], a)?, m[3].trim().to_string()));
    }
    if conds.is_empty() {
        return None;
    }
    let mut cs: Vec<&Country> = countries()
        .values()
        .filter(|x| x.population > 0 && cont.map_or(true, |k| x.continent == k))
        .filter(|x| {
            conds.iter().all(|(a, op, v, _)| {
                let y = attr(x, a);
                match *op {
                    ">" => y > *v,
                    ">=" => y >= *v,
                    "<=" => y <= *v,
                    "=" => (y - v).abs() < 0.5,
                    _ => y < *v,
                }
            })
        })
        .collect();
    let key = conds[0].0;
    cs.sort_by(|a, b| attr(b, key).partial_cmp(&attr(a, key)).unwrap());
    let desc = conds.iter().map(|(a, op, _, raw)| format!("{a} {op} {raw}")).collect::<Vec<_>>().join(" and ");
    let mut head = vec!["country"];
    let mut keys: Vec<&str> = conds.iter().map(|c| c.0).collect();
    keys.dedup();
    head.extend(keys.iter().copied());
    let t: Vec<Vec<Content>> = cs.iter().map(|x| {
        let mut r = vec![text(x.name.clone())];
        r.extend(keys.iter().map(|k| text(attr_text(x, k))));
        r
    }).collect();
    let names = if cs.is_empty() { "none".to_string() } else { cs.iter().map(|x| x.name.clone()).collect::<Vec<_>>().join(" | ") };
    Some(Answer::new(
        "country filter",
        vec![
            input(format!("countries with {desc}")),
            primary(pod("Result", &format!("{} countries", cs.len()), text(names))),
            pod("Table", "Details", table(Some(head), t)),
        ],
    ))
}

// ---------------------------------------------------------------------------
// languages
// ---------------------------------------------------------------------------

struct Lang {
    names: Vec<&'static str>,
    family: &'static str,
    l1: f64,
    total: f64,
    script: &'static str,
    iso1: &'static str,
    iso3: &'static str,
    official: &'static str,
}

// names | family | native speakers (millions) | total speakers (millions) | script | ISO 639-1 | ISO 639-3 | official in (ISO country codes; * = countries whose main listed language it is)
const LANGS: &str = "\
English|Indo-European › Germanic › West Germanic|380|1456|Latin|en|eng|*
Mandarin Chinese;mandarin;chinese;standard chinese|Sino-Tibetan › Sinitic|939|1138|Chinese characters (simplified and traditional)|zh|cmn|CN TW SG HK MO
Hindi|Indo-European › Indo-Iranian › Indo-Aryan|345|609|Devanagari|hi|hin|IN
Spanish;castilian|Indo-European › Italic › Romance|485|559|Latin|es|spa|ES MX CO AR PE VE CL EC GT CU BO DO HN PY SV NI CR PA UY GQ PR
French|Indo-European › Italic › Romance|81|310|Latin|fr|fra|FR BE CH LU MC CA HT BJ BF BI CM CF TD KM CD CG CI DJ GQ GA GN MG ML NE RW SN SC TG VU
Arabic;standard arabic|Afro-Asiatic › Semitic|362|400|Arabic|ar|ara|DZ BH KM DJ EG IQ JO KW LB LY MR MA OM PS QA SA SO SD SY TN AE YE TD
Bengali;bangla|Indo-European › Indo-Iranian › Indo-Aryan|234|273|Bengali–Assamese|bn|ben|BD
Portuguese|Indo-European › Italic › Romance|236|264|Latin|pt|por|PT BR AO MZ GW CV ST TL GQ
Russian|Indo-European › Balto-Slavic › East Slavic|148|255|Cyrillic|ru|rus|RU BY KZ KG
Urdu|Indo-European › Indo-Iranian › Indo-Aryan|70|232|Perso-Arabic (Nastaliq)|ur|urd|PK
Indonesian;bahasa indonesia|Austronesian › Malayo-Polynesian|43|199|Latin|id|ind|ID
German|Indo-European › Germanic › West Germanic|75|134|Latin|de|deu|DE AT CH LI LU BE
Japanese|Japonic|123|123|kanji, hiragana and katakana|ja|jpn|JP
Marathi|Indo-European › Indo-Iranian › Indo-Aryan|83|99|Devanagari|mr|mar|
Telugu|Dravidian|83|96|Telugu|te|tel|
Turkish|Turkic › Oghuz|82|90|Latin|tr|tur|TR CY
Tamil|Dravidian|79|86|Tamil|ta|tam|LK SG
Vietnamese|Austroasiatic › Vietic|85|86|Latin (chữ Quốc ngữ)|vi|vie|VN
Korean|Koreanic|81|82|Hangul|ko|kor|KR KP
Italian|Indo-European › Italic › Romance|64|68|Latin|it|ita|IT SM VA CH
Persian;farsi|Indo-European › Indo-Iranian › Iranian|57|79|Perso-Arabic|fa|fas|IR AF TJ
Swahili;kiswahili|Niger–Congo › Bantu|16|72|Latin|sw|swa|TZ KE UG RW
Thai|Kra–Dai › Tai|21|61|Thai|th|tha|TH
Tagalog;filipino|Austronesian › Philippine|29|83|Latin|tl|tgl|PH
Amharic|Afro-Asiatic › Semitic|33|57|Ge'ez (Ethiopic)|am|amh|ET
Hausa|Afro-Asiatic › Chadic|51|79|Latin (Boko) and Arabic (Ajami)|ha|hau|
Yoruba|Niger–Congo › Volta–Niger|46|47|Latin|yo|yor|
Polish|Indo-European › Balto-Slavic › West Slavic|40|41|Latin|pl|pol|PL
Ukrainian|Indo-European › Balto-Slavic › East Slavic|33|39|Cyrillic|uk|ukr|UA
Romanian|Indo-European › Italic › Romance|24|28|Latin|ro|ron|RO MD
Dutch|Indo-European › Germanic › West Germanic|25|30|Latin|nl|nld|NL BE SR
Zulu|Niger–Congo › Bantu|12|28|Latin|zu|zul|ZA
Greek|Indo-European › Hellenic|13|13.5|Greek|el|ell|GR CY
Hungarian|Uralic › Ugric|13|13|Latin|hu|hun|HU
Swedish|Indo-European › Germanic › North Germanic|10|13|Latin|sv|swe|SE FI
Catalan;valencian|Indo-European › Italic › Romance|4.1|9.2|Latin|ca|cat|AD
Hebrew|Afro-Asiatic › Semitic|5|9|Hebrew|he|heb|IL
Danish|Indo-European › Germanic › North Germanic|6|6|Latin|da|dan|DK
Finnish|Uralic › Finnic|5|5.8|Latin|fi|fin|FI
Norwegian|Indo-European › Germanic › North Germanic|5|5.3|Latin|no|nor|NO
Galician|Indo-European › Italic › Romance|2.4|3|Latin|gl|glg|
Estonian|Uralic › Finnic|1.1|1.2|Latin|et|est|EE
Basque;euskara|language isolate|0.75|0.9|Latin|eu|eus|
Welsh|Indo-European › Celtic › Brittonic|0.56|0.88|Latin|cy|cym|GB
Icelandic|Indo-European › Germanic › North Germanic|0.33|0.33|Latin|is|isl|IS
Romansh|Indo-European › Italic › Romance|0.06|0.06|Latin|rm|roh|CH
Latin|Indo-European › Italic › Latino-Faliscan|0|0|Latin|la|lat|VA
Esperanto|constructed language|0.001|2|Latin|eo|epo|";

fn langs() -> &'static Vec<Lang> {
    static L: OnceLock<Vec<Lang>> = OnceLock::new();
    L.get_or_init(|| {
        LANGS
            .lines()
            .filter_map(|l| {
                let f: Vec<&'static str> = l.split('|').collect();
                (f.len() == 8).then(|| Lang { names: f[0].split(';').collect(), family: f[1], l1: f[2].parse().unwrap_or(0.0), total: f[3].parse().unwrap_or(0.0), script: f[4], iso1: f[5], iso3: f[6], official: f[7] })
            })
            .collect()
    })
}

fn find_lang(w: &str) -> Option<&'static Lang> {
    let w = w.trim().trim_start_matches("the ").trim();
    let w = w.trim_end_matches(" language").trim();
    langs().iter().find(|x| x.names.iter().any(|n| n.eq_ignore_ascii_case(w)))
}

/// Countries in which a language is official.
fn official_in(x: &Lang) -> Vec<&'static Country> {
    let mut v: Vec<&Country> = if x.official == "*" {
        countries().values().filter(|c| c.languages.split(',').next().map_or(false, |f| f.split('-').next() == Some(x.iso1))).collect()
    } else {
        x.official.split(' ').filter_map(|k| countries().get(k)).collect()
    };
    v.sort_by(|a, b| a.name.cmp(&b.name));
    v
}

fn millions(v: f64) -> String {
    if v >= 1000.0 {
        format!("about {} billion", n(v / 1000.0))
    } else if v >= 1.0 {
        format!("about {} million", n(v))
    } else if v > 0.0 {
        format!("about {}", fmt_int((v * 1e6).round() as u64))
    } else {
        "none (no native speakers)".into()
    }
}

fn lang_rows(x: &Lang) -> Vec<(&'static str, String)> {
    let mut r = vec![("language family", x.family.to_string()), ("native speakers", millions(x.l1)), ("total speakers (first and second language)", millions(x.total)), ("writing system", x.script.to_string()), ("ISO 639 codes", format!("{} / {}", x.iso1, x.iso3))];
    let off = official_in(x);
    if !off.is_empty() {
        r.push(("official language in", format!("{} countries and territories: {}", off.len(), off.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(", "))));
    }
    r
}

fn languages(l: &str) -> Option<Answer> {
    // "Arabic speaking countries", "german speaking country with the largest land area"
    if let Some(c) = re!(r"^(?:which |what |list of |all )?(?:the )?([a-z ]+?)[- ]speaking countr(?:y|ies)(?: (?:with|by) (?:the )?(largest|biggest|smallest|highest|most|lowest|least) (land area|area|population|people))?$").captures(l)
        .or_else(|| re!(r"^(?:which |what )?countries (?:that )?speak ([a-z ]+?)()()$").captures(l))
        .or_else(|| re!(r"^(?:which |what |list of )?countries (?:where|in which) ([a-z ]+?) is (?:spoken|official|an official language)()()$").captures(l))
    {
        let x = find_lang(&c[1])?;
        let mut cs = official_in(x);
        let name = x.names[0];
        if let (Some(w), Some(a)) = (c.get(2).filter(|m| !m.as_str().is_empty()), c.get(3).filter(|m| !m.as_str().is_empty())) {
            let a = if a.as_str().contains("area") { a.as_str() } else { "population" };
            let a = attr_name(a);
            let small = matches!(w.as_str(), "smallest" | "lowest" | "least");
            cs.sort_by(|p, q| attr(p, a).partial_cmp(&attr(q, a)).unwrap());
            let best = if small { cs.first()? } else { cs.last()? };
            return Some(Answer::new(
                "language countries",
                vec![input(format!("{name}-speaking country with the {} {a}", w.as_str())), primary(pod("Result", "Result", text(format!("{} ({})", best.name, attr_text(best, a))))), pod("Table", "Countries where it is official", table(Some(vec!["country", a]), cs.iter().rev().map(|c| vec![text(c.name.clone()), text(attr_text(c, a))]).collect()))],
            ));
        }
        cs.sort_by(|p, q| q.population.cmp(&p.population));
        let pop: u64 = cs.iter().map(|c| c.population).sum();
        return Some(Answer::new(
            "language countries",
            vec![
                input(format!("countries where {name} is an official language")),
                primary(pod("Result", &format!("{} countries and territories", cs.len()), text(cs.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(" | ")))),
                pod("Table", "Details", table(Some(vec!["country", "population"]), cs.iter().map(|c| vec![text(c.name.clone()), text(fmt_int(c.population))]).collect())),
                pod("Total", "Total population of these countries", text(fmt_int(pop))),
            ],
        ));
    }
    // "languages Spain"
    if let Some(c) = re!(r"^(?:the )?languages? (?:spoken )?(?:in )?(.+)$").captures(l) {
        if let Some(country) = find_country(&c[1]) {
            let t = languages_text(country);
            if !t.is_empty() {
                return Some(Answer::new("languages", vec![input(format!("{} | languages", country.name)), primary(pod("Result", "Languages", text(t)))]));
            }
        }
    }
    // "Romance languages", "Germanic languages"
    if let Some(c) = re!(r"^(?:the |list of )?(romance|germanic|slavic|semitic|uralic|dravidian|celtic|indo-aryan|iranian|bantu|turkic|austronesian|indo-european|afro-asiatic|niger–congo|niger-congo|finnic|north germanic|west germanic|east slavic|west slavic) languages$").captures(l) {
        let fam = c[1].replace('-', "–").to_lowercase();
        let fam2 = c[1].to_lowercase();
        let mut v: Vec<&Lang> = langs().iter().filter(|x| {
            let f = x.family.to_lowercase();
            f.contains(&fam) || f.contains(&fam2)
        }).collect();
        if v.is_empty() {
            return None;
        }
        v.sort_by(|a, b| b.l1.partial_cmp(&a.l1).unwrap());
        let t = v.iter().map(|x| vec![text(x.names[0]), text(millions(x.l1)), text(millions(x.total))]).collect();
        let title = format!("{} languages", crate::domains::sky::capitalize(&c[1]));
        return Some(Answer::new("language family", vec![input(title.clone()), primary(pod("Result", &title, text(v.iter().map(|x| x.names[0]).collect::<Vec<_>>().join(" | ")))), pod("Table", "Speakers", table(Some(vec!["language", "native speakers", "total speakers"]), t))]));
    }
    // "native speakers of French", "how many people speak German"
    if let Some(c) = re!(r"^(?:number of |how many )?(native|first[- ]language|total|second[- ]language)? ?(?:speakers of|people (?:who )?speak|people speak|speakers of the) (.+?)(?: language)?(?: are there| worldwide| in the world)?$").captures(l) {
        // "the official languages of Canada"
        let target = c[2].trim();
        let xs: Vec<&Lang> = if let Some(k) = re!(r"^(?:the )?official languages? (?:of|in) (.+)$").captures(target) {
            let country = find_country(&k[1])?;
            langs().iter().filter(|x| x.official != "*" && x.official.split(' ').any(|code| code == country.iso2) || (x.official == "*" && country.languages.split(',').next().map_or(false, |f| f.split('-').next() == Some(x.iso1)))).collect()
        } else {
            split_list(target).iter().map(|p| find_lang(p)).collect::<Option<_>>()?
        };
        if xs.is_empty() {
            return None;
        }
        let native = c.get(1).map_or(false, |m| m.as_str().starts_with("native") || m.as_str().starts_with("first"));
        let main = xs.iter().map(|x| format!("{}{}", if xs.len() > 1 { format!("{}: ", x.names[0]) } else { String::new() }, if native { millions(x.l1) } else { format!("{} total ({} native)", millions(x.total), millions(x.l1).trim_start_matches("about ")) })).collect::<Vec<_>>().join(" | ");
        let t = xs.iter().map(|x| vec![text(x.names[0]), text(millions(x.l1)), text(millions(x.total))]).collect();
        return Some(Answer::new("speakers", vec![input(format!("{} | {}", xs.iter().map(|x| x.names[0]).collect::<Vec<_>>().join(", "), if native { "native speakers" } else { "speakers" })), primary(pod("Result", "Result", text(main))), pod("Table", "Speakers", table(Some(vec!["language", "native speakers", "total speakers"]), t)), pod("Source", "Note", text("approximate worldwide estimates (Ethnologue-style figures)"))]));
    }
    // "Chinese vs English speakers"
    if let Some(c) = re!(r"^(.+?) (?:speakers|speakers worldwide)$").captures(l) {
        let xs: Vec<&Lang> = split_list(&c[1]).iter().map(|p| find_lang(p)).collect::<Option<_>>()?;
        let t = xs.iter().map(|x| vec![text(x.names[0]), text(millions(x.l1)), text(millions(x.total))]).collect();
        let main = xs.iter().map(|x| format!("{}: {}", x.names[0], millions(x.total))).collect::<Vec<_>>().join(" | ");
        return Some(Answer::new("speakers", vec![input(format!("{} | speakers", xs.iter().map(|x| x.names[0]).collect::<Vec<_>>().join(", "))), primary(pod("Result", "Total speakers", text(main))), pod("Table", "Speakers", table(Some(vec!["language", "native speakers", "total speakers"]), t))]));
    }
    // "Mandarin language", "Basque, Finnish", "French language"
    let explicit = l.ends_with(" language") || l.starts_with("language ") || l.ends_with(" languages");
    let body = l.trim_start_matches("language ").trim_end_matches(" languages").trim_end_matches(" language");
    let xs: Vec<&Lang> = if explicit {
        split_list(body).iter().map(|p| find_lang(p)).collect::<Option<_>>()?
    } else {
        let parts = split_list(body);
        if parts.len() < 2 || !body.contains(',') {
            return None;
        }
        parts.iter().map(|p| find_lang(p)).collect::<Option<_>>()?
    };
    if xs.len() == 1 {
        let x = xs[0];
        return Some(Answer::new(x.names[0], vec![input(format!("{} (language)", x.names[0])), primary(pod("Result", "Result", text(format!("{}: {}; {} native speakers", x.names[0], x.family, millions(x.l1).trim_start_matches("about "))))), pod("Properties", "Properties", rows(lang_rows(x)))]));
    }
    let t = xs.iter().map(|x| vec![text(x.names[0]), text(x.family), text(millions(x.l1)), text(millions(x.total)), text(x.script)]).collect();
    Some(Answer::new("language comparison", vec![input(xs.iter().map(|x| x.names[0]).collect::<Vec<_>>().join(", ")), primary(pod("Table", "Comparison", table(Some(vec!["language", "family", "native speakers", "total speakers", "script"]), t)))]))
}

// ---------------------------------------------------------------------------
// religions
// ---------------------------------------------------------------------------

struct Religion {
    names: Vec<&'static str>,
    adherent: Vec<&'static str>,
    founded: &'static str,
    founder: &'static str,
    followers: &'static str,
    origin: &'static str,
    texts: &'static str,
    kind: &'static str,
    note: &'static str,
}

// names | adherent words | founded | founder | adherents | origin | sacred texts | type | note
const RELIGIONS: &str = "\
Christianity|christian;christians|c. 30 AD|followers of Jesus of Nazareth|about 2.4 billion|Judea (Roman Levant)|the Bible (Old and New Testaments)|monotheistic (Trinitarian)|the largest religion; main branches Catholic, Protestant and Eastern Orthodox
Islam|muslim;muslims;islamic|610 AD (first revelation to Muhammad)|Muhammad|about 1.9 billion|Mecca, Arabia|the Quran (with the Hadith)|monotheistic|main branches Sunni (about 85–90%) and Shia (about 10–15%)
Hinduism|hindu;hindus|roots in the Vedic period, c. 1500 BC|no single founder|about 1.2 billion|Indian subcontinent|the Vedas, Upanishads, Bhagavad Gita, Ramayana and Mahabharata|diverse: henotheistic, monotheistic and polytheistic traditions|about 94% of Hindus live in India
Buddhism|buddhist;buddhists|5th century BC|Siddhartha Gautama (the Buddha)|about 500 million|northern India and Nepal (Lumbini, Bodh Gaya)|the Tripitaka (Pali Canon) and the Mahayana sutras|nontheistic|main schools Theravada, Mahayana and Vajrayana
Sikhism|sikh;sikhs|c. 1500 AD (Guru Nanak, born 1469)|Guru Nanak|about 26 million|Punjab|the Guru Granth Sahib|monotheistic|
Judaism|jew;jews;jewish|c. 2000–1500 BC (traditional)|Abraham (patriarch) and Moses (lawgiver)|about 15.7 million|the Levant|the Tanakh (Torah, Prophets, Writings) and the Talmud|monotheistic|
Zoroastrianism;mazdayasna|zoroastrian;zoroastrians;parsi;parsis|between c. 1500 and 500 BC (dates disputed)|Zoroaster (Zarathustra)|about 100,000–200,000|ancient Iran|the Avesta (including the Gathas)|monotheistic with dualist elements (Ahura Mazda)|largest communities are the Parsis of India and the Zoroastrians of Iran
Jainism|jain;jains|6th century BC (Mahavira; tradition of 24 tirthankaras)|Mahavira, the 24th tirthankara|about 4.5 million|India|the Agamas|nontheistic|
Shinto|shintoist;shintoists|prehistoric Japan|none|about 100 million (varies with the definition)|Japan|the Kojiki and the Nihon Shoki|polytheistic (kami)|
Taoism;daoism|taoist;taoists|c. 4th century BC|Laozi (traditional)|about 12 million (estimates vary widely)|China|the Tao Te Ching and the Zhuangzi|varied|
Confucianism|confucian;confucians|5th century BC|Confucius|no formal membership|China|the Analects and the Five Classics|ethical and philosophical tradition|
Bahá'í Faith;bahai faith;baha'i faith;bahai;baha'i|bahais|1863|Bahá'u'lláh|about 8 million|Persia (Iran) and Baghdad|the Kitáb-i-Aqdas and the Kitáb-i-Íqán|monotheistic|
Church of Jesus Christ of Latter-day Saints;mormonism;lds church;the church of jesus christ of latter-day saints;mormon church|mormon;mormons|6 April 1830, Fayette, New York|Joseph Smith|about 17 million members (church-reported)|western New York, United States|the Bible, the Book of Mormon, the Doctrine and Covenants and the Pearl of Great Price|Christian restorationist|headquartered in Salt Lake City, Utah
Scientology;church of scientology|scientologist;scientologists|1954 (first church, Los Angeles; Dianetics published 1950)|L. Ron Hubbard|disputed: the church claims millions; surveys find tens of thousands|Los Angeles, United States|the writings of L. Ron Hubbard (Dianetics)|new religious movement|
Catholicism;roman catholic church;catholic church;roman catholicism|catholic;catholics|1st century (traditional)|traces its origin to Jesus and the apostle Peter|about 1.39 billion|Rome|the Bible and Sacred Tradition|Christian|led by the Pope in Vatican City
Protestantism|protestant;protestants|1517 (Martin Luther's Ninety-five Theses)|Martin Luther, John Calvin and other reformers|about 900 million|Germany and Switzerland|the Bible|Christian|
Eastern Orthodoxy;eastern orthodox church;orthodox church|orthodox christians|1st century; separated from Rome in the Great Schism of 1054|traces its origin to the apostles|about 220 million|eastern Mediterranean|the Bible and Holy Tradition|Christian|
Jehovah's Witnesses|jehovah's witnesses|1870s (Bible Student movement)|Charles Taze Russell|about 8.6 million publishers|Pittsburgh, United States|the Bible (New World Translation)|Christian restorationist (nontrinitarian)|
Rastafari;rastafarianism|rastafarian;rastafarians|1930s|— (inspired by Marcus Garvey; venerates Haile Selassie)|about 1 million|Jamaica|the Bible (especially the Holy Piby and the Kebra Nagast)|Abrahamic new religious movement|";

fn religions() -> &'static Vec<Religion> {
    static R: OnceLock<Vec<Religion>> = OnceLock::new();
    R.get_or_init(|| {
        RELIGIONS
            .lines()
            .filter_map(|l| {
                let f: Vec<&'static str> = l.split('|').collect();
                (f.len() == 9).then(|| Religion { names: f[0].split(';').collect(), adherent: f[1].split(';').collect(), founded: f[2], founder: f[3], followers: f[4], origin: f[5], texts: f[6], kind: f[7], note: f[8] })
            })
            .collect()
    })
}

fn find_religion(w: &str) -> Option<&'static Religion> {
    let w = w.trim().trim_start_matches("the ").trim().to_lowercase();
    let w = w.trim_end_matches(" religion").trim_end_matches(" faith").trim();
    religions().iter().find(|r| r.names.iter().any(|n| n.eq_ignore_ascii_case(w) || n.trim_start_matches("the ").eq_ignore_ascii_case(w)) || r.adherent.iter().any(|a| *a == w))
}

fn religion_rows(r: &Religion) -> Vec<(&'static str, String)> {
    let mut v = vec![("founded", r.founded.to_string()), ("founder", r.founder.to_string()), ("adherents", r.followers.to_string()), ("place of origin", r.origin.to_string()), ("sacred texts", r.texts.to_string()), ("type", r.kind.to_string())];
    if !r.note.is_empty() {
        v.push(("note", r.note.to_string()));
    }
    v
}

/// religious composition: country | source | religion=share%=count…
const COMPOSITION: &[(&str, &str, &[(&str, f64, f64)])] = &[
    ("IN", "2011 census of India", &[("Hindus", 79.8, 966.3e6), ("Muslims", 14.2, 172.2e6), ("Christians", 2.3, 27.8e6), ("Sikhs", 1.7, 20.8e6), ("Buddhists", 0.7, 8.4e6), ("Jains", 0.4, 4.5e6), ("other religions", 0.7, 7.9e6), ("not stated", 0.2, 2.9e6)]),
    ("BR", "2010 census of Brazil", &[("Christians (Catholics and Protestants)", 86.8, 165.6e6), ("Catholics", 64.6, 123.3e6), ("Protestants (evangelicals)", 22.2, 42.3e6), ("no religion", 8.0, 15.3e6), ("Spiritists", 2.0, 3.8e6), ("other religions", 3.2, 6.1e6)]),
    ("GB", "2021 census of England and Wales", &[("Christians", 46.2, 27.5e6), ("no religion", 37.2, 22.2e6), ("Muslims", 6.5, 3.87e6), ("Hindus", 1.7, 1.03e6), ("Sikhs", 0.9, 0.524e6), ("Buddhists", 0.5, 0.273e6), ("Jews", 0.5, 0.271e6), ("other religions", 0.6, 0.348e6), ("not stated", 6.0, 3.6e6)]),
    ("US", "Pew Research Center Religious Landscape Study, 2014 (adults)", &[("Christians", 70.6, 173.0e6), ("Protestants", 46.5, 114.0e6), ("Catholics", 20.8, 51.0e6), ("Mormons", 1.6, 3.9e6), ("Jews", 1.9, 4.7e6), ("Muslims", 0.9, 2.2e6), ("Buddhists", 0.7, 1.7e6), ("Hindus", 0.7, 1.7e6), ("unaffiliated", 22.8, 56.0e6)]),
    ("FR", "Pew Research Center estimate, 2010", &[("Christians", 63.0, 39.6e6), ("unaffiliated", 28.0, 17.6e6), ("Muslims", 7.5, 4.7e6), ("Jews", 0.5, 0.31e6), ("Buddhists", 0.5, 0.31e6), ("other religions", 0.5, 0.31e6)]),
];

fn group_matches(group: &str, word: &str) -> bool {
    let g = group.to_lowercase();
    let r = match find_religion(word) {
        Some(r) => r,
        None => return g.starts_with(word),
    };
    r.adherent.iter().chain(r.names.iter()).any(|a| g.starts_with(&a.to_lowercase()) || g.starts_with(&format!("{}s", a.to_lowercase())))
        || (r.names[0] == "Christianity" && g.starts_with("christians"))
        || (r.names[0] == "Protestantism" && g.starts_with("protestants"))
}

fn composition(l: &str) -> Option<Answer> {
    // find the country at the end of the query
    let (what, place) = if let Some(c) = re!(r"^(?:the )?(united states|united kingdom|us|usa|uk|india|brazil|france|england and wales|england)(?:'s)? (religious adherents|religions?|religious composition|religious demographics|religious breakdown)$").captures(l) {
        (c[2].to_string(), c[1].to_string())
    } else if let Some(c) = re!(r"^(?:how many |number of |the )?(.*?)\s*(?:populations?|adherents|people|believers|followers)?\s*(?:are there |live )?(?:in|of) (?:the )?(.+?)$").captures(l).or_else(|| re!(r"^(?:the )?([a-z].*?) ((?:united states|united kingdom|us|usa|uk|india|brazil|france|england and wales|england))$").captures(l)) {
        (c[1].to_string(), c[2].to_string())
    } else {
        return None;
    };
    let place = place.trim().trim_start_matches("the ");
    let place = if matches!(place, "england and wales" | "england" | "britain" | "great britain") { "uk" } else { place };
    let country = find_country(place)?;
    let (_, source, groups) = COMPOSITION.iter().find(|x| x.0 == country.iso2)?;
    let what = what.trim();
    let what = re!(r"\b(?:religious|religion|religions|adherents|composition|population|populations|people|breakdown|demographics?)\b").replace_all(what, " ");
    let what = what.split_whitespace().collect::<Vec<_>>().join(" ");
    let sum = what.contains('+');
    let ratio = what.contains(" / ") || what.contains('/');
    let words: Vec<String> = re!(r"\s*(?:\+|/|,|\bvs\.?\s|\bversus\b|\band\b)\s*").split(&what).map(|w| w.trim().to_string()).filter(|w| !w.is_empty()).collect();
    let picked: Vec<&(&str, f64, f64)> = words.iter().filter_map(|w| groups.iter().find(|g| group_matches(g.0, w))).collect();
    if !words.is_empty() && picked.len() != words.len() {
        return None;
    }
    if words.is_empty() && !l.contains("relig") {
        return None;
    }
    let t = groups.iter().map(|g| vec![text(g.0), text(format!("{}%", fmt_float(g.1, 3))), text(people(g.2))]).collect();
    let main = if picked.is_empty() {
        groups.iter().take(4).map(|g| format!("{}: {}%", g.0, fmt_float(g.1, 3))).collect::<Vec<_>>().join(" | ")
    } else if sum {
        let (p, k): (f64, f64) = picked.iter().fold((0.0, 0.0), |a, g| (a.0 + g.1, a.1 + g.2));
        format!("{} ({}% of the population)", people(k), fmt_float(p, 3))
    } else if ratio && picked.len() == 2 {
        format!("{}", fmt_float(picked[0].2 / picked[1].2, 4))
    } else {
        picked.iter().map(|g| format!("{}: {} ({}%)", g.0, people(g.2), fmt_float(g.1, 3))).collect::<Vec<_>>().join(" | ")
    };
    Some(Answer::new(
        "religious composition",
        vec![input(format!("{} | {}", country.name, if picked.is_empty() { "religious composition".to_string() } else { picked.iter().map(|g| g.0).collect::<Vec<_>>().join(if sum { " + " } else if ratio { " / " } else { ", " }) })), primary(pod("Result", "Result", text(main))), pod("Table", "Religious composition", table(Some(vec!["group", "share", "people"]), t)), pod("Source", "Source", text(*source))],
    ))
}

fn religion(l: &str) -> Option<Answer> {
    if re!(r"^when (?:was|were|did) (?:martin )?luther'?s? (?:95 |ninety-five )?(?:thesis|theses)(?: (?:written|posted|published|nailed))?$").is_match(l) || re!(r"^(?:the )?(?:95|ninety-five) theses$").is_match(l) {
        return Some(Answer::new("Ninety-five Theses", vec![input("Ninety-five Theses (Martin Luther)"), primary(pod("Result", "Result", text("31 October 1517"))), pod("Details", "Details", rows(vec![("author", "Martin Luther".into()), ("place", "Wittenberg, Electorate of Saxony".into()), ("title", "Disputation on the Power and Efficacy of Indulgences".into()), ("significance", "traditionally regarded as the start of the Protestant Reformation".into())]))]));
    }
    let pats = [
        (re!(r"^when (?:was|were|did) (?:the )?(.+?) (?:founded|established|started|begin|began|created)$"), "founded"),
        (re!(r"^(?:who (?:founded|started|created)|(?:the )?founders? of) (?:the )?(.+)$"), "founder"),
        (re!(r"^(?:what is |what's )?(?:the )?(?:major|main|holy|sacred|central|principal|religious) (?:text|texts|book|books|scripture|scriptures) (?:of|in) (?:the )?(.+)$"), "texts"),
        (re!(r"^(?:where did|where was) (?:the )?(.+?) (?:originate|begin|start|founded)$"), "origin"),
        (re!(r"^how many (.+?) are there(?: in the world)?$"), "followers"),
        (re!(r"^(?:number of )?(.+?) (?:adherents|followers|worldwide)$"), "followers"),
    ];
    for (re, what) in pats {
        if let Some(c) = re.captures(l) {
            let r = find_religion(&c[1])?;
            let rows_v = religion_rows(r);
            let key = match what {
                "founded" => "founded",
                "founder" => "founder",
                "texts" => "sacred texts",
                "origin" => "place of origin",
                _ => "adherents",
            };
            let v = rows_v.iter().find(|x| x.0 == key)?.1.clone();
            return Some(Answer::new(r.names[0], vec![input(format!("{} | {key}", r.names[0])), primary(pod("Result", "Result", text(v))), pod("Properties", "Properties", rows(rows_v))]));
        }
    }
    // a bare name or a comparison
    let parts = split_list(l);
    let rs: Vec<&Religion> = if let Some(r) = find_religion(l) { vec![r] } else if parts.len() >= 2 { parts.iter().map(|p| find_religion(p)).collect::<Option<_>>()? } else { return None };
    if rs.len() == 1 {
        let r = rs[0];
        // adherent words alone ("catholics") are too vague to answer bare
        if r.adherent.iter().any(|a| *a == l.trim()) {
            return None;
        }
        return Some(Answer::new(r.names[0], vec![input(format!("{} (religion)", r.names[0])), primary(pod("Properties", "Properties", rows(religion_rows(r))))]));
    }
    let t = rs.iter().map(|r| vec![text(r.names[0]), text(r.founded), text(r.founder), text(r.followers), text(r.texts)]).collect();
    Some(Answer::new("religion comparison", vec![input(rs.iter().map(|r| r.names[0]).collect::<Vec<_>>().join(", ")), primary(pod("Table", "Comparison", table(Some(vec!["religion", "founded", "founder", "adherents", "sacred texts"]), t)))]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    country_lookup(l)
        .or_else(|| country_filter(l))
        .or_else(|| religion(l))
        .or_else(|| composition(l))
        .or_else(|| languages(l))
        .or_else(|| compare_countries(l))
}
