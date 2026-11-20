//! Crop and livestock production and food supply from FAOSTAT (FAO,
//! CC BY 4.0): "soybean harvest in Mexico", "corn yield in China vs Russia",
//! "Germany cattle vs hogs", "US chickens + turkeys + ducks",
//! "beef production Brazil vs Argentina", "Italy coffee consumption".

use super::geo::find_country;
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

static FAO_Z: &[u8] = include_bytes!("../../data/fao.z");
const P0: u16 = 2014;
const F0: u16 = 2021;

struct Fao {
    /// area code -> (ISO3, name)
    areas: HashMap<u32, (&'static str, &'static str)>,
    iso: HashMap<&'static str, u32>,
    /// item name -> item code ("p…" production, "f…" food balance)
    items: HashMap<&'static str, &'static str>,
    names: HashMap<&'static str, &'static str>,
    /// (area, item, element) -> (unit, values from P0 or F0)
    rows: HashMap<(u32, &'static str, char), (&'static str, Vec<Option<f64>>)>,
}

fn fao() -> &'static Fao {
    static D: OnceLock<Fao> = OnceLock::new();
    D.get_or_init(|| {
        let raw = miniz_oxide::inflate::decompress_to_vec(FAO_Z).unwrap_or_default();
        let s: &'static str = Box::leak(String::from_utf8(raw).unwrap_or_default().into_boxed_str());
        let mut d = Fao { areas: HashMap::new(), iso: HashMap::new(), items: HashMap::new(), names: HashMap::new(), rows: HashMap::new() };
        let mut sec = "";
        for line in s.lines() {
            if line.starts_with('#') {
                sec = line;
                continue;
            }
            let f: Vec<&'static str> = line.split('\t').collect();
            let vals = |t: &str| t.split(',').map(|x| x.parse().ok()).collect::<Vec<Option<f64>>>();
            match sec {
                "#A" if f.len() == 3 => {
                    let Ok(c) = f[0].parse() else { continue };
                    d.areas.insert(c, (f[1], f[2]));
                    if !f[1].is_empty() {
                        d.iso.insert(f[1], c);
                    }
                }
                "#I" if f.len() == 2 => {
                    d.items.entry(f[1]).or_insert(f[0]);
                    if f[0].starts_with('f') {
                        d.items.insert(Box::leak(format!("f:{}", f[1]).into_boxed_str()), f[0]);
                    }
                    d.names.insert(f[0], f[1]);
                }
                "#P" if f.len() == 5 => {
                    let (Ok(a), Some(e)) = (f[0].parse(), f[2].chars().next()) else { continue };
                    d.rows.insert((a, f[1], e), (f[3], vals(f[4])));
                }
                "#F" if f.len() == 4 => {
                    let (Ok(a), Some(e)) = (f[0].parse(), f[2].chars().next()) else { continue };
                    d.rows.insert((a, f[1], e), ("", vals(f[3])));
                }
                _ => {}
            }
        }
        d
    })
}

fn item_code(name: &str) -> Option<&'static str> {
    fao().items.get(name).copied().filter(|c| c.starts_with('p'))
}

fn food_code(name: &str) -> Option<&'static str> {
    fao().items.get(format!("f:{name}").as_str()).copied()
}

/// What is asked about an item.
#[derive(Clone, Copy, PartialEq)]
enum Measure {
    Production,
    Area,
    Yield,
    Stocks,
    Consumption,
}

/// Words for production items: (pattern, FAO production item, meat item when
/// the word names an animal, food balance item).
const ITEMS: &[(&str, &str, &str, &str)] = &[
    ("soybeans?|soya beans?|soy", "Soya beans", "", "Soyabeans"),
    ("corn|maize", "Maize (corn)", "", "Maize and products"),
    ("rice", "Rice", "", "Rice and products"),
    ("wheat", "Wheat", "", "Wheat and products"),
    ("barley", "Barley", "", "Barley and products"),
    ("oats", "Oats", "", "Oats"),
    ("rye", "Rye", "", "Rye and products"),
    ("sorghum", "Sorghum", "", "Sorghum and products"),
    ("millet", "Millet", "", "Millet and products"),
    ("cereals?|grains?", "Cereals, primary", "", "Cereals - Excluding Beer"),
    ("coffee", "Coffee, green", "", "Coffee and products"),
    ("cocoa|cacao", "Cocoa beans", "", "Cocoa Beans and products"),
    ("tea", "Tea leaves", "", "Tea (including mate)"),
    ("potato(?:es)?", "Potatoes", "", "Potatoes and products"),
    ("sweet potato(?:es)?", "Sweet potatoes", "", "Sweet potatoes"),
    ("cassava", "Cassava, fresh", "", "Cassava and products"),
    ("sugar ?cane", "Sugar cane", "", "Sugar cane"),
    ("sugar beets?", "Sugar beet", "", "Sugar beet"),
    ("sugar", "Raw cane or beet sugar (centrifugal only)", "", "Sugar (Raw Equivalent)"),
    ("tomato(?:es)?", "Tomatoes", "", "Tomatoes and products"),
    ("onions?", "Onions and shallots, dry (excluding dehydrated)", "", "Onions"),
    ("bananas?", "Bananas", "", "Bananas"),
    ("apples?", "Apples", "", "Apples and products"),
    ("oranges?", "Oranges", "", "Oranges, Mandarines"),
    ("grapes?", "Grapes", "", "Grapes and products (excl wine)"),
    ("wine", "Wine", "", "Wine"),
    ("beer", "Beer of barley, malted", "", "Beer"),
    ("olives?", "Olives", "", "Olives (including preserved)"),
    ("olive oil", "Olive oil", "", "Olive Oil"),
    ("palm oil", "Palm oil", "", "Palm Oil"),
    ("peanuts?|groundnuts?", "Groundnuts, excluding shelled", "", "Groundnuts"),
    ("cotton", "Seed cotton, unginned", "", ""),
    ("tobacco", "Unmanufactured tobacco", "", ""),
    ("avocados?", "Avocados", "", ""),
    ("strawberries", "Strawberries", "", ""),
    ("lemons?|limes?", "Lemons and limes", "", "Lemons, Limes and products"),
    ("pineapples?", "Pineapples", "", "Pineapples and products"),
    ("mangoes|mangos?", "Mangoes, guavas and mangosteens", "", ""),
    ("almonds?", "Almonds, in shell", "", ""),
    ("cashews?", "Cashew nuts, in shell", "", ""),
    ("rapeseed|canola", "Rape or colza seed", "", "Rape and Mustardseed"),
    ("sunflower seeds?", "Sunflower seed", "", "Sunflower seed"),
    ("vegetables", "Vegetables Primary", "", "Vegetables"),
    ("fruits?", "Fruit Primary", "", "Fruits - Excluding Wine"),
    ("wool", "Shorn wool, greasy, including fleece-washed shorn wool", "", ""),
    ("natural rubber|rubber", "Natural rubber in primary forms", "", ""),
    ("honey", "Natural honey", "", "Honey"),
    ("eggs", "Hen eggs in shell, fresh", "", "Eggs"),
    ("milk", "Raw milk of cattle", "", "Milk - Excluding Butter"),
    ("cheese", "Cheese (All Kinds)", "", ""),
    ("butter", "Butter of cow milk", "", "Butter, Ghee"),
    ("beef", "Meat of cattle with the bone, fresh or chilled", "", "Bovine Meat"),
    ("pork", "Meat of pig with the bone, fresh or chilled", "", "Pigmeat"),
    ("lamb|mutton", "Meat of sheep, fresh or chilled", "", "Mutton & Goat Meat"),
    ("poultry(?: meat)?", "Meat, Poultry", "", "Poultry Meat"),
    ("chicken meat", "Meat of chickens, fresh or chilled", "", "Poultry Meat"),
    ("goat meat", "Meat of goat, fresh or chilled", "", "Mutton & Goat Meat"),
    ("meat", "Meat, Total", "", "Meat"),
    ("fish|seafood", "", "", "Fish, Seafood"),
    ("cattle|cows?", "Cattle", "Meat of cattle with the bone, fresh or chilled", "Bovine Meat"),
    ("hogs?|pigs?|swine", "Swine / pigs", "Meat of pig with the bone, fresh or chilled", "Pigmeat"),
    ("sheep", "Sheep", "Meat of sheep, fresh or chilled", "Mutton & Goat Meat"),
    ("goats?", "Goats", "Meat of goat, fresh or chilled", "Mutton & Goat Meat"),
    ("chickens?", "Chickens", "Meat of chickens, fresh or chilled", "Poultry Meat"),
    ("turkeys?", "Turkeys", "Meat of turkeys, fresh or chilled", "Poultry Meat"),
    ("ducks?", "Ducks", "Meat of ducks, fresh or chilled", "Poultry Meat"),
    ("geese|goose", "Geese", "Meat of geese, fresh or chilled", "Poultry Meat"),
    ("horses?", "Horses", "Horse meat, fresh or chilled", ""),
    ("donkeys?|asses", "Asses", "", ""),
    ("mules?", "Mules and hinnies", "", ""),
    ("camels?", "Camels", "Meat of camels, fresh or chilled", ""),
    ("buffalo(?:es|s)?", "Buffalo", "Meat of buffalo, fresh or chilled", ""),
    ("rabbits?", "Rabbits and hares", "Meat of rabbits and hares, fresh or chilled", ""),
    ("beehives|bee ?hives|bees", "Bees", "", ""),
    ("food", "", "", "Grand Total"),
];

fn items_re() -> &'static Vec<Regex> {
    static R: OnceLock<Vec<Regex>> = OnceLock::new();
    R.get_or_init(|| ITEMS.iter().map(|(p, ..)| Regex::new(&format!(r"\b(?:{p})\b")).unwrap()).collect())
}

/// Region names FAOSTAT aggregates: (query name, FAO area codes summed).
const REGIONS: &[(&str, &str, &[u32])] = &[
    ("world", "world", &[5000]),
    ("africa", "Africa", &[5100]),
    ("eastern africa", "Eastern Africa", &[5101]),
    ("east africa", "Eastern Africa", &[5101]),
    ("middle africa", "Middle Africa", &[5102]),
    ("central africa", "Middle Africa", &[5102]),
    ("northern africa", "Northern Africa", &[5103]),
    ("north africa", "Northern Africa", &[5103]),
    ("southern africa", "Southern Africa", &[5104]),
    ("western africa", "Western Africa", &[5105]),
    ("west africa", "Western Africa", &[5105]),
    ("americas", "the Americas", &[5200]),
    ("the americas", "the Americas", &[5200]),
    ("north america", "North America", &[5203, 5204, 5206]),
    ("northern america", "Northern America", &[5203]),
    ("central america", "Central America", &[5204]),
    ("caribbean", "the Caribbean", &[5206]),
    ("the caribbean", "the Caribbean", &[5206]),
    ("south america", "South America", &[5207]),
    ("asia", "Asia", &[5300]),
    ("central asia", "Central Asia", &[5301]),
    ("east asia", "Eastern Asia", &[5302]),
    ("eastern asia", "Eastern Asia", &[5302]),
    ("south asia", "Southern Asia", &[5303]),
    ("southern asia", "Southern Asia", &[5303]),
    ("southeast asia", "South-eastern Asia", &[5304]),
    ("south-east asia", "South-eastern Asia", &[5304]),
    ("south east asia", "South-eastern Asia", &[5304]),
    ("western asia", "Western Asia", &[5305]),
    ("europe", "Europe", &[5400]),
    ("eastern europe", "Eastern Europe", &[5401]),
    ("northern europe", "Northern Europe", &[5402]),
    ("southern europe", "Southern Europe", &[5403]),
    ("western europe", "Western Europe", &[5404]),
    ("oceania", "Oceania", &[5500]),
    ("european union", "the European Union", &[5707]),
    ("eu", "the European Union", &[5707]),
];

fn place(s: &str) -> Option<(String, Vec<u32>)> {
    let s = s.trim().trim_start_matches("the ").trim_end_matches("'s").trim();
    let s = re!(r"\s+(?:countries|nations|region)$").replace(s, "").to_string();
    if s.is_empty() {
        return None;
    }
    if let Some((_, name, codes)) = REGIONS.iter().find(|r| r.0 == s) {
        return Some((name.to_string(), codes.to_vec()));
    }
    let s = match s.as_str() {
        "u.s." | "u.s" | "u.s.a." | "united states of america" => "united states",
        "u.k." | "britain" | "great britain" => "united kingdom",
        x => x,
    };
    let c = find_country(s)?;
    let code = *fao().iso.get(c.iso3.as_str())?;
    let name = if c.name.starts_with("United ") || c.name.starts_with("Netherlands") || c.name.starts_with("Philippines") { format!("the {}", c.name) } else { c.name.clone() };
    Some((name, vec![code]))
}

/// Yearly values of one element of an item, summed over the area codes.
fn values(codes: &[u32], item: &'static str, e: char) -> Option<(&'static str, u16, Vec<Option<f64>>)> {
    let mut out: Vec<Option<f64>> = Vec::new();
    let mut unit = "";
    for c in codes {
        let (u, v) = fao().rows.get(&(*c, item, e))?;
        unit = u;
        if out.len() < v.len() {
            out.resize(v.len(), Some(0.0));
        }
        for (i, x) in v.iter().enumerate() {
            out[i] = match (out[i], x) {
                (Some(a), Some(b)) => Some(a + b),
                _ => None,
            };
        }
    }
    let start = if item.starts_with('f') { F0 } else { P0 };
    Some((unit, start, out))
}

fn latest(v: &[Option<f64>], start: u16, year: Option<u16>) -> Option<(u16, f64)> {
    match year {
        Some(y) => Some((y, (*v.get(y.checked_sub(start)? as usize)?)?)),
        None => v.iter().enumerate().rev().find_map(|(i, x)| x.map(|x| (start + i as u16, x))),
    }
}

fn big(x: f64) -> String {
    let (d, w) = if x.abs() >= 1e9 { (1e9, " billion") } else if x.abs() >= 1e6 { (1e6, " million") } else { (1.0, "") };
    if w.is_empty() {
        fmt_float(x.round(), 9)
    } else {
        format!("{}{w}", fmt_float((x / d * 1000.0).round() / 1000.0, 4))
    }
}

/// A value with its unit: (amount, display text).
fn shown(m: Measure, unit: &str, item: &str, v: f64) -> String {
    match m {
        Measure::Production => format!("{} metric tons per year", big(v)),
        Measure::Area => format!("{} hectares", big(v)),
        Measure::Yield => format!("{} metric tons per hectare", fmt_float((v * 1000.0).round() / 1000.0, 4)),
        Measure::Stocks => {
            let n = if unit.starts_with("1000") { v * 1000.0 } else { v };
            format!("{} {}", big(n), if item == "p1181" { "beehives" } else { "animals" })
        }
        Measure::Consumption => format!("{} kg per person per year", fmt_float((v * 10.0).round() / 10.0, 4)),
    }
}

/// One answer row: (place name, codes, item code, measure) -> (year, value, text).
fn row(codes: &[u32], item: &'static str, m: Measure, year: Option<u16>) -> Option<(u16, f64, String)> {
    let (_, unit, start, v) = match m {
        Measure::Production => {
            let (u, s, v) = values(codes, item, 'P')?;
            ('P', u, s, v)
        }
        Measure::Area => {
            let (u, s, v) = values(codes, item, 'H')?;
            ('H', u, s, v)
        }
        Measure::Stocks => {
            let (u, s, v) = values(codes, item, 'S')?;
            ('S', u, s, v)
        }
        Measure::Consumption => match values(codes, item, 'Q') {
            Some((u, s, v)) => ('Q', u, s, v),
            None => {
                let (_, s, v) = values(codes, item, 'K')?;
                let (y, x) = latest(&v, s, year)?;
                return Some((y, x, format!("{} kcal per person per day", fmt_float(x.round(), 6))));
            }
        },
        Measure::Yield => {
            let (_, s, p) = values(codes, item, 'P')?;
            let (_, _, h) = values(codes, item, 'H')?;
            let v: Vec<Option<f64>> = p.iter().zip(h.iter()).map(|(a, b)| Some(a.as_ref()? / b.filter(|b| *b > 0.0)?)).collect();
            ('Y', "", s, v)
        }
    };
    let (y, x) = latest(&v, start, year)?;
    Some((y, x, shown(m, unit, item, x)))
}

fn history(codes: &[u32], item: &'static str, m: Measure) -> Option<Pod> {
    let mut rows = Vec::new();
    for y in (P0..=2024).rev() {
        if let Some((y, _, t)) = row(codes, item, m, Some(y)) {
            rows.push(vec![text(y.to_string()), text(t)]);
        }
    }
    (rows.len() > 2).then(|| pod("History", "History", table(Some(vec!["year", "value"]), rows.into_iter().take(10).collect())))
}

struct Seg {
    items: Vec<usize>,
    places: Vec<(String, Vec<u32>)>,
}

fn segment(s: &str) -> Option<Seg> {
    let mut rest = format!(" {} ", s.trim());
    let mut items = Vec::new();
    loop {
        let found = items_re().iter().enumerate().filter_map(|(i, r)| r.find(&rest).map(|m| (i, m.start(), m.end()))).min_by_key(|(i, st, en)| (*st, usize::MAX - (en - st), *i));
        let Some((i, st, en)) = found else { break };
        items.push(i);
        rest = format!("{} {}", &rest[..st], &rest[en..]);
    }
    let rest = re!(r"\b(?:in|of|for|from|the|total|number|numbers|amount|annual|yearly|all|by|country|countries|and|population|populations|head|stocks?|inventory)\b").replace_all(&rest, " ");
    let rest = re!(r"\s+").replace_all(rest.trim(), " ").to_string();
    let mut places = Vec::new();
    if !rest.is_empty() {
        places.push(place(&rest)?);
    }
    Some(Seg { items, places })
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace("u.s.", "us").replace("vs.", "vs");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = re!(r"^(?:what (?:is|was|are|were) |how (?:many|much) |show (?:me )?)").replace(&l, "").to_string();
    let year: Option<u16> = re!(r"\b(20[12]\d)\b").captures(&l).and_then(|c| c[1].parse().ok());
    let l = re!(r"\b(?:in |during )?20[12]\d\b").replace(&l, " ").to_string();
    let (measure, l) = if let Some(m) = re!(r"\b(?:production|harvests?|output|produced|crops?)\b").find(&l) {
        (Some(Measure::Production), format!("{} {}", &l[..m.start()], &l[m.end()..]))
    } else if let Some(m) = re!(r"\b(?:yields?|productivity)\b").find(&l) {
        (Some(Measure::Yield), format!("{} {}", &l[..m.start()], &l[m.end()..]))
    } else if let Some(m) = re!(r"\b(?:area harvested|harvested area|acreage|planted area)\b").find(&l) {
        (Some(Measure::Area), format!("{} {}", &l[..m.start()], &l[m.end()..]))
    } else if let Some(m) = re!(r"\b(?:consumption|consumed|food supply|intake)\b").find(&l) {
        (Some(Measure::Consumption), format!("{} {}", &l[..m.start()], &l[m.end()..]))
    } else {
        (None, l.clone())
    };
    let plus = l.contains('+');
    let segs_txt: Vec<&str> = re!(r"\s(?:vs|versus|compared (?:to|with))\s|\+|,|/|\band\b").split(&l).collect();
    let mut segs: Vec<Seg> = segs_txt.iter().filter(|s| !s.trim().is_empty()).map(|s| segment(s)).collect::<Option<_>>()?;
    if segs.iter().all(|s| s.items.is_empty()) {
        return None;
    }
    // stocks are the default for animals named alone ("Germany cattle vs hogs")
    let measure = match measure {
        Some(m) => m,
        None if segs.iter().all(|s| s.items.iter().all(|i| !ITEMS[*i].2.is_empty() || ITEMS[*i].1 == "Bees")) => Measure::Stocks,
        None => return None,
    };
    // "wheat production": the world total
    if segs.iter().all(|s| s.places.is_empty()) {
        if measure == Measure::Stocks || segs.len() > 1 {
            return None;
        }
        segs[0].places.push(("world".into(), vec![5000]));
    }
    for i in 0..segs.len() {
        if segs[i].places.is_empty() {
            segs[i].places = segs.iter().find(|s| !s.places.is_empty())?.places.clone();
        }
        if segs[i].items.is_empty() {
            segs[i].items = segs.iter().find(|s| !s.items.is_empty())?.items.clone();
        }
    }
    // (place name, codes, item code, item label)
    let mut rows: Vec<(String, Vec<u32>, &'static str)> = Vec::new();
    for s in &segs {
        for i in &s.items {
            let (_, prod, meat, food) = ITEMS[*i];
            let code = match measure {
                Measure::Consumption => food_code(food)?,
                Measure::Production if !meat.is_empty() => item_code(meat)?,
                Measure::Stocks if meat.is_empty() && ITEMS[*i].1 != "Bees" => return None,
                _ => item_code(prod)?,
            };
            for (n, c) in &s.places {
                rows.push((n.clone(), c.clone(), code));
            }
        }
    }
    let label = |code: &str| -> String {
        let n = fao().names.get(code).copied().unwrap_or("");
        let n = match n {
            "Swine / pigs" => "pigs",
            "Maize (corn)" => "corn",
            "Grand Total" => "all food",
            x => x,
        };
        n.split(", fresh").next().unwrap_or(n).replace(" with the bone", "").to_lowercase()
    };
    let what = match measure {
        Measure::Production => "production",
        Measure::Area => "area harvested",
        Measure::Yield => "yield",
        Measure::Stocks => "livestock",
        Measure::Consumption => "food supply",
    };
    let source = pod("Source", "Source", text("FAOSTAT, Food and Agriculture Organization of the United Nations (CC BY 4.0)."));
    if rows.len() == 1 {
        let (n, codes, item) = &rows[0];
        let (y, _, t) = row(codes, item, measure, year)?;
        let mut pods = vec![
            pod("Input", "Input interpretation", text(format!("{} | {} | {what}", n, label(item)))),
            primary(pod("Result", "Result", text(format!("{t} ({y})")))),
        ];
        if measure == Measure::Consumption {
            if let Some((_, _, v)) = values(codes, item, 'K') {
                if let Some((_, k)) = latest(&v, F0, Some(y)) {
                    pods.push(pod("Energy", "Food energy", text(format!("{} kcal per person per day", fmt_float(k.round(), 6)))));
                }
            }
            if let Some((_, _, v)) = values(codes, item, 'T') {
                if let Some((_, t)) = latest(&v, F0, Some(y)) {
                    pods.push(pod("Total", "Total food use", text(format!("{} metric tons per year", big(t * 1000.0)))));
                }
            }
        } else if let Some(h) = history(codes, item, measure) {
            pods.push(h);
        }
        pods.push(source);
        return Some(Answer::new("agriculture", pods));
    }
    let mut out = Vec::new();
    let mut total = 0.0;
    let mut years = Vec::new();
    for (n, codes, item) in &rows {
        let (y, v, t) = row(codes, item, measure, year)?;
        let v = if measure == Measure::Stocks && fao().rows.get(&(codes[0], *item, 'S')).is_some_and(|r| r.0.starts_with("1000")) { v * 1000.0 } else { v };
        total += v;
        years.push(y);
        out.push(vec![text(n.clone()), text(label(item)), text(t), text(y.to_string())]);
    }
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} | {what}", rows.iter().map(|r| format!("{} {}", r.0, label(r.2))).collect::<Vec<_>>().join(if plus { " + " } else { " vs " }))))];
    if plus && matches!(measure, Measure::Production | Measure::Area | Measure::Stocks) {
        let t = match measure {
            Measure::Production => format!("{} metric tons per year", big(total)),
            Measure::Area => format!("{} hectares", big(total)),
            _ => format!("{} animals", big(total)),
        };
        pods.push(primary(pod("Result", "Total", text(t))));
        pods.push(pod("Values", "Values", table(Some(vec!["place", "item", "value", "year"]), out)));
    } else {
        pods.push(primary(pod("Result", "Result", table(Some(vec!["place", "item", "value", "year"]), out))));
    }
    pods.push(source);
    Some(Answer::new("agriculture", pods))
}
