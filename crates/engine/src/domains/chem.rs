//! Chemistry: the periodic table, element properties, chemical formulas
//! and molar masses.

use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

#[derive(Deserialize, Clone, Debug)]
pub struct Element {
    pub z: u32,
    pub symbol: String,
    pub name: String,
    pub mass: f64,
    pub group: Option<u32>,
    pub period: Option<u32>,
    pub block: Option<String>,
    pub series: Option<String>,
    pub config: Option<String>,
    pub electronegativity: Option<f64>,
    pub melting_k: Option<f64>,
    pub boiling_k: Option<f64>,
    pub density_g_cm3: Option<f64>,
    pub atomic_radius_pm: Option<f64>,
    #[serde(default)]
    pub covalent_radius_pm: Option<f64>,
    #[serde(default)]
    pub vdw_radius_pm: Option<f64>,
    pub first_ionization_ev: Option<f64>,
    pub electron_affinity_ev: Option<f64>,
    pub discovery_year: Option<serde_json::Value>,
    pub discoverers: Option<String>,
    pub oxidation_states: Option<Vec<i32>>,
    #[serde(default)]
    pub oxidation_states_all: Option<Vec<i32>>,
    pub abundance_crust_mg_kg: Option<f64>,
    pub is_radioactive: Option<bool>,
    pub name_origin: Option<String>,
    pub uses: Option<String>,
    pub description: Option<String>,
}

pub fn elements() -> &'static Vec<Element> {
    static E: OnceLock<Vec<Element>> = OnceLock::new();
    E.get_or_init(|| serde_json::from_str(include_str!("../../data/elements.json")).unwrap_or_default())
}

pub fn by_symbol(s: &str) -> Option<&'static Element> {
    elements().iter().find(|e| e.symbol == s)
}

pub fn find_element(q: &str) -> Option<&'static Element> {
    let q = q.trim();
    let lq = q.to_lowercase();
    let lq = lq.trim_start_matches("element ").trim_end_matches(" element").trim();
    if let Ok(z) = lq.parse::<u32>() {
        return elements().iter().find(|e| e.z == z);
    }
    let lq = if lq == "aluminium" { "aluminum" } else if lq == "sulphur" { "sulfur" } else if lq == "caesium" { "cesium" } else { lq };
    elements().iter().find(|e| e.name.to_lowercase() == lq).or_else(|| if q.len() <= 2 && q.chars().next()?.is_uppercase() { by_symbol(q) } else { None })
}

/// Common compound names → formulas.
const COMPOUNDS: &[(&str, &str)] = &[
    ("water", "H2O"),
    ("caffeine", "C8H10N4O2"),
    ("glucose", "C6H12O6"),
    ("fructose", "C6H12O6"),
    ("sucrose", "C12H22O11"),
    ("sugar", "C12H22O11"),
    ("table salt", "NaCl"),
    ("salt", "NaCl"),
    ("sodium chloride", "NaCl"),
    ("ethanol", "C2H5OH"),
    ("alcohol", "C2H5OH"),
    ("methanol", "CH3OH"),
    ("methane", "CH4"),
    ("ethane", "C2H6"),
    ("propane", "C3H8"),
    ("butane", "C4H10"),
    ("octane", "C8H18"),
    ("benzene", "C6H6"),
    ("acetone", "C3H6O"),
    ("acetic acid", "CH3COOH"),
    ("vinegar", "CH3COOH"),
    ("aspirin", "C9H8O4"),
    ("paracetamol", "C8H9NO2"),
    ("acetaminophen", "C8H9NO2"),
    ("ibuprofen", "C13H18O2"),
    ("carbon dioxide", "CO2"),
    ("carbon monoxide", "CO"),
    ("ammonia", "NH3"),
    ("ozone", "O3"),
    ("oxygen gas", "O2"),
    ("nitrogen gas", "N2"),
    ("hydrogen gas", "H2"),
    ("hydrogen peroxide", "H2O2"),
    ("sulfuric acid", "H2SO4"),
    ("hydrochloric acid", "HCl"),
    ("nitric acid", "HNO3"),
    ("phosphoric acid", "H3PO4"),
    ("sodium hydroxide", "NaOH"),
    ("baking soda", "NaHCO3"),
    ("sodium bicarbonate", "NaHCO3"),
    ("calcium carbonate", "CaCO3"),
    ("limestone", "CaCO3"),
    ("rust", "Fe2O3"),
    ("iron oxide", "Fe2O3"),
    ("silicon dioxide", "SiO2"),
    ("quartz", "SiO2"),
    ("chlorophyll", "C55H72MgN4O5"),
    ("cholesterol", "C27H46O"),
    ("dopamine", "C8H11NO2"),
    ("serotonin", "C10H12N2O"),
    ("nicotine", "C10H14N2"),
    ("vitamin c", "C6H8O6"),
    ("ascorbic acid", "C6H8O6"),
    ("urea", "CH4N2O"),
    ("glycine", "C2H5NO2"),
    ("adrenaline", "C9H13NO3"),
    ("penicillin", "C16H18N2O4S"),
    ("tnt", "C7H5N3O6"),
    ("copper sulfate", "CuSO4"),
    ("potassium permanganate", "KMnO4"),
    ("chloroform", "CHCl3"),
];

pub fn compound_formula(name: &str) -> Option<&'static str> {
    let l = name.trim().to_lowercase();
    COMPOUNDS.iter().find(|(n, _)| *n == l).map(|(_, f)| *f).or_else(|| find_db_compound(&l).map(|c| c.formula))
}

/// A compound from the bundled Wikidata extract (data/compounds.z, CC0).
pub struct DbCompound {
    pub name: &'static str,
    pub formula: &'static str,
    /// Kelvin
    pub mp_k: Option<f64>,
    pub bp_k: Option<f64>,
    /// kg/m³
    pub density: Option<f64>,
    /// SMILES (isomeric where known), empty when unknown.
    pub smiles: &'static str,
}

static COMPOUNDS_Z: &[u8] = include_bytes!("../../data/compounds.z");

/// Synonyms in the Wikidata compound list that are street names, brand
/// names, abbreviations or ordinary words ("snow" for cocaine, "gravel"
/// for α-PVP, "air" for a ribotide); they are not indexed.
pub(crate) const SYNONYM_BLOCK: &[&str] = &[
    "aba", "abate", "ace", "aces", "acid", "add", "ado", "advance", "age", "ags", "aha", "air",
    "ait", "ala", "aldehyde", "alf", "alpha", "als", "amen", "ammine", "amp", "amyl", "anon",
    "apollo", "asp", "auf", "aye", "azo", "badge", "bal", "balance", "bap", "bas", "base", "basen",
    "beans", "beg", "bet", "bez", "biocide", "bit", "blow", "bro", "bru", "caa", "caddy", "camp",
    "cap", "caps", "carbonyl", "cash", "cat", "caz", "cetane", "cha", "chalk", "chamomile", "chaps",
    "chi", "cid", "cis", "cit", "clarity", "cod", "coke", "cop", "copper", "cor", "cot", "counter",
    "crack", "crank", "crypto", "crystal", "dab", "dago", "damp", "dap", "deb", "def", "deg", "den",
    "dex", "diamide", "diamine", "diethyl", "dim", "dimethyl", "dip", "dit", "dob", "doc", "dod",
    "doh", "dom", "don", "dop", "dot", "dots", "dump", "dust", "dynamos", "ean", "ecstasy", "eds",
    "eld", "ems", "encore", "erg", "essence", "eta", "eth", "ethanoyl", "eureka", "eve", "faa",
    "fad", "fantasy", "fas", "fast", "ferry", "flash", "flea", "foh", "formyl", "foster", "fou",
    "foxy", "gad", "gan", "gap", "gaucho", "geo", "glass", "goey", "goog", "gravel", "gun", "han",
    "hat", "hem", "henna", "him", "his", "hits", "hmm", "hog", "hoi", "hug", "hydrastis",
    "hydroxyl", "hyp", "ice", "imp", "imperator", "intellect", "intercept", "jak", "jeff", "lac",
    "lah", "leu", "lime", "limestone", "lost", "lovely", "maa", "mac", "magenta", "mauve",
    "mediator", "meg", "mem", "men", "meow", "merit", "merlin", "mes", "met", "mib", "mic",
    "microdot", "molly", "mops", "muse", "mustard", "myc", "nag", "nam", "nana", "nas", "nexus",
    "nis", "nit", "nitro", "nitrous", "nix", "noo", "nor", "nos", "oca", "oda", "ole", "one",
    "ozone", "pac", "pah", "palladium", "pan", "panorama", "pap", "para", "pas", "pat", "patrol",
    "pax", "pea", "ped", "pegasus", "pent", "pep", "pia", "pic", "pinky", "pip", "pix", "poa",
    "polo", "poppers", "pos", "prink", "pro", "pure", "pursuit", "pyx", "qin", "raiser", "ran",
    "ras", "revolution", "ripcord", "roofies", "sac", "sad", "sam", "same", "sandal", "sau",
    "scoop", "sea", "sec", "ser", "sham", "shh", "sic", "silicon", "silver", "snap", "snow",
    "speed", "stronghold", "tai", "tao", "tar", "tee", "teg", "tel", "telos", "ten", "tetra",
    "today", "tranq", "triac", "tub", "tweak", "uppers", "urd", "vac", "vanilla", "voltage", "wack",
    "was", "water", "wax", "whiz", "yay", "zol",
];

pub fn compound_db() -> &'static (Vec<DbCompound>, std::collections::HashMap<String, usize>) {
    static DB: OnceLock<(Vec<DbCompound>, std::collections::HashMap<String, usize>)> = OnceLock::new();
    DB.get_or_init(|| {
        let bytes = miniz_oxide::inflate::decompress_to_vec(COMPOUNDS_Z).unwrap_or_default();
        let text: &'static str = Box::leak(String::from_utf8(bytes).unwrap_or_default().into_boxed_str());
        let mut list = Vec::new();
        let mut index = std::collections::HashMap::new();
        for line in text.lines() {
            let f: Vec<&'static str> = line.split('\t').collect();
            if f.len() < 5 || f[1].is_empty() {
                continue;
            }
            let num = |s: &str| s.parse::<f64>().ok();
            let i = list.len();
            list.push(DbCompound { name: f[0], formula: f[1], mp_k: num(f[2]), bp_k: num(f[3]), density: num(f[4]), smiles: f.get(6).copied().unwrap_or("") });
            index.insert(f[0].to_lowercase(), i);
            for a in f.get(5).map(|a| a.split('|')).into_iter().flatten() {
                let a = a.to_lowercase();
                if SYNONYM_BLOCK.binary_search(&a.as_str()).is_ok() {
                    continue;
                }
                index.entry(a).or_insert(i);
            }
        }
        (list, index)
    })
}

pub fn find_db_compound(name: &str) -> Option<&'static DbCompound> {
    let (list, index) = compound_db();
    let l = name.trim().to_lowercase();
    let l = l.trim_start_matches("the ").trim();
    index.get(l).map(|&i| &list[i])
}

/// Parse a chemical formula into element counts.  Supports parentheses,
/// brackets and hydrates: `Ca(OH)2`, `CuSO4·5H2O`, `[Fe(CN)6]3`.
pub fn parse_formula(f: &str) -> Option<BTreeMap<String, u32>> {
    let f = f.trim();
    if f.is_empty() || !f.chars().next()?.is_ascii_alphanumeric() && !"([".contains(f.chars().next()?) {
        return None;
    }
    let mut total = BTreeMap::new();
    for part in f.split(['·', '*', '.']) {
        let part = part.trim();
        // leading multiplier for hydrates
        let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
        let mult: u32 = if digits.is_empty() { 1 } else { digits.parse().ok()? };
        let body = &part[digits.len()..];
        let (counts, rest) = parse_group(body)?;
        if !rest.is_empty() {
            return None;
        }
        for (k, v) in counts {
            *total.entry(k).or_insert(0) += v * mult;
        }
    }
    if total.is_empty() {
        None
    } else {
        Some(total)
    }
}

fn parse_group(s: &str) -> Option<(BTreeMap<String, u32>, &str)> {
    let mut out: BTreeMap<String, u32> = BTreeMap::new();
    let mut rest = s;
    loop {
        let c = match rest.chars().next() {
            Some(c) => c,
            None => break,
        };
        if c == '(' || c == '[' {
            let (inner, after) = parse_group(&rest[1..])?;
            let after = after.strip_prefix([')', ']'])?;
            let n: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            let k: u32 = if n.is_empty() { 1 } else { n.parse().ok()? };
            for (e, v) in inner {
                *out.entry(e).or_insert(0) += v * k;
            }
            rest = &after[n.len()..];
        } else if c == ')' || c == ']' {
            break;
        } else if c.is_ascii_uppercase() {
            let mut sym = c.to_string();
            let mut it = rest[1..].chars();
            let mut used = 1;
            if let Some(d) = it.next() {
                if d.is_ascii_lowercase() {
                    sym.push(d);
                    used += 1;
                }
            }
            by_symbol(&sym)?;
            let after = &rest[used..];
            let n: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            let k: u32 = if n.is_empty() { 1 } else { n.parse().ok()? };
            *out.entry(sym).or_insert(0) += k;
            rest = &after[n.len()..];
        } else {
            return None;
        }
    }
    Some((out, rest))
}

pub fn molar_mass(counts: &BTreeMap<String, u32>) -> f64 {
    counts.iter().map(|(e, n)| by_symbol(e).map_or(0.0, |el| el.mass) * *n as f64).sum()
}

pub fn hill_formula(counts: &BTreeMap<String, u32>) -> String {
    let mut keys: Vec<&String> = counts.keys().collect();
    let has_c = counts.contains_key("C");
    keys.sort_by_key(|k| match (has_c, k.as_str()) {
        (true, "C") => (0, String::new()),
        (true, "H") => (1, String::new()),
        _ => (2, k.to_string()),
    });
    keys.iter().map(|k| if counts[*k] == 1 { k.to_string() } else { format!("{k}{}", subscript(counts[*k])) }).collect()
}

fn subscript(n: u32) -> String {
    n.to_string().chars().map(|c| char::from_u32(0x2080 + c.to_digit(10).unwrap()).unwrap()).collect()
}

fn element_pods(e: &Element) -> Vec<Pod> {
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} (chemical element)", e.name)))];
    let mut basic = vec![vec![text("element symbol"), text(e.symbol.clone())], vec![text("atomic number"), text(e.z.to_string())], vec![text("atomic mass"), text(format!("{} u", fmt_float(e.mass, 8)))]];
    if let Some(c) = &e.config {
        basic.push(vec![text("electron configuration"), text(c.clone())]);
    }
    if let (Some(g), Some(p)) = (e.group, e.period) {
        basic.push(vec![text("group, period"), text(format!("{g}, {p}"))]);
    } else if let Some(p) = e.period {
        basic.push(vec![text("period"), text(p.to_string())]);
    }
    if let Some(s) = &e.series {
        basic.push(vec![text("element type"), text(s.to_lowercase())]);
    }
    if let Some(b) = &e.block {
        basic.push(vec![text("block"), text(b.clone())]);
    }
    pods.push(primary(pod("BasicProperties", "Basic elemental properties", table(None, basic))));
    let mut thermo = Vec::new();
    let k2c = |k: f64| format!("{} K ({} °C)", fmt_float(k, 6), fmt_float(k - 273.15, 6));
    if let Some(m) = e.melting_k {
        thermo.push(vec![text("melting point"), text(k2c(m))]);
    }
    if let Some(b) = e.boiling_k {
        thermo.push(vec![text("boiling point"), text(k2c(b))]);
    }
    if let Some(d) = e.density_g_cm3 {
        thermo.push(vec![text("density"), text(format!("{} g/cm³", fmt_float(d, 6)))]);
    }
    if let (Some(m), Some(b)) = (e.melting_k, e.boiling_k) {
        let phase = if 293.15 < m {
            "solid"
        } else if 293.15 < b {
            "liquid"
        } else {
            "gas"
        };
        thermo.push(vec![text("phase at STP"), text(phase)]);
    }
    if !thermo.is_empty() {
        pods.push(pod("Thermo", "Thermodynamic properties", table(None, thermo)));
    }
    let mut atomic = Vec::new();
    if let Some(v) = e.electronegativity {
        atomic.push(vec![text("electronegativity (Pauling)"), text(fmt_float(v, 4))]);
    }
    if let Some(v) = e.first_ionization_ev {
        atomic.push(vec![text("first ionization energy"), text(format!("{} eV", fmt_float(v, 6)))]);
    }
    if let Some(v) = e.electron_affinity_ev {
        atomic.push(vec![text("electron affinity"), text(format!("{} eV", fmt_float(v, 6)))]);
    }
    if let Some(v) = e.atomic_radius_pm {
        atomic.push(vec![text("atomic radius"), text(format!("{} pm", fmt_float(v, 6)))]);
    }
    if let Some(ox) = &e.oxidation_states {
        if !ox.is_empty() {
            atomic.push(vec![text("oxidation states"), text(ox.iter().map(|o| if *o > 0 { format!("+{o}") } else { o.to_string() }).collect::<Vec<_>>().join(", "))]);
        }
    }
    if !atomic.is_empty() {
        pods.push(pod("Atomic", "Atomic properties", table(None, atomic)));
    }
    let mut hist = Vec::new();
    if let Some(y) = &e.discovery_year {
        let y = match y {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            _ => String::new(),
        };
        if !y.is_empty() {
            hist.push(vec![text("discovered"), text(y)]);
        }
    }
    if let Some(d) = &e.discoverers {
        if !d.is_empty() {
            hist.push(vec![text("discoverers"), text(d.clone())]);
        }
    }
    if let Some(o) = &e.name_origin {
        if !o.is_empty() {
            hist.push(vec![text("name origin"), text(o.clone())]);
        }
    }
    if !hist.is_empty() {
        pods.push(pod("History", "History", table(None, hist)));
    }
    if let Some(a) = e.abundance_crust_mg_kg {
        pods.push(pod("Abundance", "Abundance in Earth's crust", text(format!("{} mg/kg", fmt_float(a, 6)))));
    }
    if let Some(r) = e.is_radioactive {
        if r {
            pods.push(pod("Radioactivity", "Nuclear properties", text("radioactive (no stable isotopes)")));
        }
    }
    if let Some(u) = &e.uses {
        if !u.is_empty() {
            pods.push(pod("Uses", "Uses", text(u.clone())));
        }
    }
    pods
}

pub(super) fn compound_pods(label: &str, formula: &str, counts: &BTreeMap<String, u32>) -> Vec<Pod> {
    let mm = molar_mass(counts);
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{label} | molar mass")))];
    pods.push(primary(pod("Result", "Molar mass", text(format!("{} g/mol", fmt_float(mm, 7))))));
    pods.push(pod("Formula", "Chemical formula", text(format!("{formula}   (Hill formula: {})", hill_formula(counts)))));
    let rows: Vec<Vec<Content>> = counts
        .iter()
        .map(|(e, n)| {
            let el = by_symbol(e);
            let m = el.map_or(0.0, |x| x.mass) * *n as f64;
            vec![text(el.map_or(e.clone(), |x| x.name.clone())), text(e.clone()), text(n.to_string()), text(format!("{} g/mol", fmt_float(m, 6))), text(format!("{}%", fmt_float(100.0 * m / mm, 4)))]
        })
        .collect();
    pods.push(pod("Composition", "Elemental composition", table(Some(vec!["element", "symbol", "atoms", "mass", "mass fraction"]), rows)));
    let atoms: u32 = counts.values().sum();
    pods.push(pod("Counts", "Atom count", text(format!("{atoms} atoms ({} elements)", counts.len()))));
    pods
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    // molar mass / molecular weight
    for prefix in ["molar mass of ", "molecular weight of ", "molecular mass of ", "mass of one mole of ", "formula weight of "] {
        if let Some(rest) = l.strip_prefix(prefix) {
            let orig = &q.clean[q.clean.len() - rest.len()..];
            let formula = compound_formula(rest).unwrap_or(orig.trim());
            let counts = parse_formula(formula)?;
            let label = if compound_formula(rest).is_some() { format!("{rest} ({formula})") } else { formula.to_string() };
            return Some(Answer::new(format!("molar mass of {label}"), compound_pods(&label, formula, &counts)));
        }
    }
    // element lookups: "iron", "element 26", "properties of gold", "Fe"
    let el_q = l.trim_start_matches("properties of ").trim_start_matches("the element ").trim_start_matches("chemical element ");
    let orig = &q.clean[q.clean.len() - el_q.len()..];
    let numeric = el_q.trim().chars().all(|c| c.is_ascii_digit());
    if let Some(e) = find_element(orig).filter(|_| !numeric || l.starts_with("element")) {
        if el_q.len() > 2 || q.raw.trim().len() <= 2 || l.starts_with("element") {
            return Some(Answer::new(format!("{} (element)", e.name), element_pods(e)));
        }
    }
    // "atomic mass of carbon", "Kr density", "valence electrons in Bi",
    // "compare the atomic radii of F, Cl, Br and I"
    if let Some(a) = element_question(q) {
        return Some(a);
    }
    // a bare formula like "H2SO4" or a compound name like "caffeine"
    if let Some(f) = compound_formula(l) {
        let counts = parse_formula(f)?;
        let label = format!("{l} ({f})");
        let mut pods = compound_pods(&label, f, &counts);
        pods.extend(super::chemcalc::property_pod(l));
        return Some(Answer::new(label.clone(), pods));
    }
    let raw = q.clean.trim();
    // digits, or at least two capitals and a small letter as in "NaCl", "KBr", "NaOH"
    let formula_like = raw.chars().any(|c| c.is_ascii_digit()) || (raw.chars().filter(|c| c.is_ascii_uppercase()).count() >= 2 && raw.chars().any(|c| c.is_ascii_lowercase()) && raw != "NaN");
    if raw.len() >= 3 && formula_like && raw.chars().next()?.is_ascii_uppercase() && !raw.contains(' ') {
        if let Some(counts) = parse_formula(raw) {
            if counts.len() >= 2 || counts.values().any(|v| *v > 1) {
                return Some(Answer::new(raw.to_string(), compound_pods(raw, raw, &counts)));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// element properties by name
// ---------------------------------------------------------------------------

/// Property names and their synonyms (canonical name first).
const PROPS: &[&[&str]] = &[
    &["atomic mass", "atomic weight", "mass", "relative atomic mass", "standard atomic weight"],
    &["atomic number", "number of protons", "protons", "proton number", "proton count"],
    &["neutrons", "number of neutrons", "neutron number", "neutron count"],
    &["electrons", "number of electrons", "electron count"],
    &["valence electrons", "number of valence electrons", "valence electron count", "valence"],
    &["melting point", "melting temperature", "mp"],
    &["boiling point", "boiling temperature", "bp"],
    &["density"],
    &["electronegativity", "pauling electronegativity"],
    &["electron configuration", "electronic configuration", "full electron configuration", "full electronic configuration", "configuration"],
    &["abbreviated electron configuration", "short electron configuration", "short electronic configuration", "abbreviated electronic configuration", "shorthand electron configuration", "noble gas configuration", "condensed electron configuration"],
    &["orbital diagram", "orbital box diagram", "electron orbital diagram"],
    &["atomic radius", "atomic radii", "radius", "size"],
    &["covalent radius", "covalent radii"],
    &["van der waals radius", "van der waals radii", "vdw radius"],
    &["ionization energy", "first ionization energy", "ionisation energy", "first ionisation energy", "ionization potential"],
    &["electron affinity"],
    &["oxidation states", "oxidation numbers", "oxidation state", "oxidation number", "charges", "charge", "ion charge", "ions"],
    &["group", "group number"],
    &["period", "period number"],
    &["block"],
    &["classification", "series", "category", "type", "element type"],
    &["discovery", "discovered", "discovery year", "year discovered", "discoverer", "discoverers", "who discovered"],
    &["name origin", "etymology", "origin of name", "name meaning"],
    &["uses", "applications", "used for"],
    &["abundance", "crustal abundance", "abundance in earth's crust", "abundance in the earth's crust"],
    &["symbol", "chemical symbol", "element symbol"],
    &["radioactive", "radioactivity", "is it radioactive"],
];

fn canonical_prop(p: &str) -> Option<&'static str> {
    let p = p.trim().trim_start_matches("the ").trim();
    let single = p.replace("points", "point").replace("radii", "radius").replace("densities", "density").replace("energies", "energy").replace("masses", "mass").replace("numbers", "number").replace("configurations", "configuration").replace("diagrams", "diagram");
    PROPS.iter().find(|syns| syns.iter().any(|s| *s == p || *s == single)).map(|syns| syns[0])
}

fn noble_core(e: &Element) -> Option<(&'static Element, String)> {
    let cfg = e.config.as_ref()?;
    let toks: Vec<&str> = cfg.split_whitespace().collect();
    for z in [86u32, 54, 36, 18, 10, 2] {
        if z >= e.z {
            continue;
        }
        let ng = elements().iter().find(|x| x.z == z)?;
        let core: Vec<&str> = ng.config.as_ref()?.split_whitespace().collect();
        if core.iter().all(|t| toks.contains(t)) {
            let rest: Vec<&str> = toks.iter().filter(|t| !core.contains(t)).copied().collect();
            return Some((ng, format!("[{}] {}", ng.symbol, rest.join(" "))));
        }
    }
    None
}

fn subshells(cfg: &str) -> Vec<(u32, char, u32)> {
    cfg.split_whitespace()
        .filter_map(|t| {
            let mut c = t.chars();
            let n = c.next()?.to_digit(10)?;
            let l = c.next()?;
            let k: u32 = c.as_str().parse().ok()?;
            Some((n, l, k))
        })
        .collect()
}

pub fn valence_electrons(e: &Element) -> Option<u32> {
    let sh = subshells(e.config.as_ref()?);
    let nmax = sh.iter().map(|x| x.0).max()?;
    let mut v: u32 = sh.iter().filter(|x| x.0 == nmax && (x.1 == 's' || x.1 == 'p')).map(|x| x.2).sum();
    match e.block.as_deref() {
        Some("d") => v += sh.iter().filter(|x| x.1 == 'd' && x.0 + 1 == nmax).map(|x| x.2).sum::<u32>(),
        Some("f") => v += sh.iter().filter(|x| (x.1 == 'f' && x.0 + 2 == nmax) || (x.1 == 'd' && x.0 + 1 == nmax)).map(|x| x.2).sum::<u32>(),
        _ => {}
    }
    // a full shell of eight (or two for helium) counts as the noble-gas valence
    Some(v)
}

fn orbital_diagram(cfg: &str) -> String {
    subshells(cfg)
        .iter()
        .map(|(n, l, k)| {
            let boxes = match l {
                's' => 1,
                'p' => 3,
                'd' => 5,
                'f' => 7,
                _ => 1,
            };
            let mut slots = vec![0u32; boxes];
            for i in 0..*k as usize {
                slots[i % boxes] += 1;
            }
            let cells: Vec<&str> = slots.iter().map(|c| match c {
                0 => "[  ]",
                1 => "[↑ ]",
                _ => "[↑↓]",
            }).collect();
            format!("{n}{l}: {}", cells.join(""))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn kelvin_text(k: f64) -> String {
    format!("{} K  ({} °C, {} °F)", fmt_float(k, 6), fmt_float(k - 273.15, 6), fmt_float((k - 273.15) * 1.8 + 32.0, 6))
}

/// A property of an element as text.
pub fn element_property(e: &Element, prop: &str) -> Option<String> {
    Some(match prop {
        "atomic mass" => format!("{} u", fmt_float(e.mass, 8)),
        "atomic number" => e.z.to_string(),
        "electrons" => format!("{} (neutral atom)", e.z),
        "neutrons" => format!("{} (most common isotope, mass number {})", (e.mass.round() as u32).saturating_sub(e.z), e.mass.round() as u32),
        "valence electrons" => valence_electrons(e)?.to_string(),
        "melting point" => kelvin_text(e.melting_k?),
        "boiling point" => kelvin_text(e.boiling_k?),
        "density" => format!("{} g/cm³", fmt_float(e.density_g_cm3?, 6)),
        "electronegativity" => format!("{} (Pauling scale)", fmt_float(e.electronegativity?, 4)),
        "electron configuration" => e.config.clone()?,
        "abbreviated electron configuration" => noble_core(e).map(|x| x.1).or_else(|| e.config.clone())?,
        "orbital diagram" => orbital_diagram(e.config.as_ref()?),
        "atomic radius" => format!("{} pm (calculated)", fmt_float(e.atomic_radius_pm?, 5)),
        "covalent radius" => format!("{} pm", fmt_float(e.covalent_radius_pm?, 5)),
        "van der waals radius" => format!("{} pm", fmt_float(e.vdw_radius_pm?, 5)),
        "ionization energy" => {
            let v = e.first_ionization_ev?;
            format!("{} kJ/mol  ({} eV)", fmt_float(v * 96.485_332_12, 5), fmt_float(v, 6))
        }
        "electron affinity" => {
            let v = e.electron_affinity_ev?;
            format!("{} kJ/mol  ({} eV)", fmt_float(v * 96.485_332_12, 5), fmt_float(v, 6))
        }
        "oxidation states" => {
            let all = e.oxidation_states_all.clone().or_else(|| e.oxidation_states.clone())?;
            let sign = |v: &i32| if *v > 0 { format!("+{v}") } else { v.to_string() };
            let common = e.oxidation_states.clone().unwrap_or_default();
            let mut t = all.iter().map(sign).collect::<Vec<_>>().join(", ");
            if !common.is_empty() {
                t.push_str(&format!("  (common: {})", common.iter().map(sign).collect::<Vec<_>>().join(", ")));
            }
            t
        }
        "group" => e.group?.to_string(),
        "period" => e.period?.to_string(),
        "block" => e.block.clone()?,
        "classification" => e.series.clone()?,
        "discovery" => {
            let year = e.discovery_year.as_ref().map(|v| v.to_string().trim_matches('"').to_string()).filter(|y| y != "null");
            match (year, e.discoverers.clone()) {
                (Some(y), Some(d)) => format!("{y}, {d}"),
                (Some(y), None) => y,
                (None, Some(d)) => d,
                _ => return None,
            }
        }
        "name origin" => e.name_origin.clone()?,
        "uses" => e.uses.clone()?,
        "abundance" => format!("{} mg/kg in Earth's crust ({}%)", fmt_float(e.abundance_crust_mg_kg?, 5), fmt_float(e.abundance_crust_mg_kg? / 1e4, 4)),
        "symbol" => e.symbol.clone(),
        "radioactive" => if e.is_radioactive.unwrap_or(false) { "yes".into() } else { "no (has stable isotopes)".into() },
        _ => return None,
    })
}

/// An element named by name, symbol (any case when unambiguous in context)
/// or "element 12".
fn element_ref(s: &str) -> Option<&'static Element> {
    let t = s.trim().trim_end_matches('?').trim().trim_start_matches("the ").trim_start_matches("an ").trim_start_matches("a ").trim();
    let t = t.trim_end_matches(" atom").trim_end_matches(" atoms").trim();
    find_element(t).or_else(|| if t.len() <= 2 && !t.is_empty() { elements().iter().find(|e| e.symbol.eq_ignore_ascii_case(t)) } else { None })
}

fn element_list(s: &str) -> Option<Vec<&'static Element>> {
    let parts: Vec<&str> = s.split(|c| c == ',' || c == '/').flat_map(|p| p.split(" and ")).flat_map(|p| p.split(" vs ")).flat_map(|p| p.split(" vs. ")).flat_map(|p| p.split(" versus ")).map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
    // bare numbers are arithmetic ("6/3"), not atomic numbers
    if parts.len() < 2 || parts.iter().any(|p| p.trim_start_matches('-').parse::<f64>().is_ok()) {
        return None;
    }
    parts.iter().map(|p| element_ref(p)).collect()
}

fn element_question(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?').trim();
    let l = re!(r"^(?:please )?(?:write|give|show|find|list|draw|what (?:is|are))\s+(?:me\s+)?").replace(l, "").to_string();
    let l = l.trim_start_matches("compare ").trim_start_matches("the ").trim();
    // split into (property, element text) in several word orders
    let mut tries: Vec<(String, String)> = Vec::new();
    if let Some(m) = Regex::new(r"^(?:how many\s+)?(.+?)\s+(?:of|in|for|does|do|on)\s+(.+?)(?:\s+(?:have|has|contain))?$").ok()?.captures(l) {
        tries.push((m[1].to_string(), m[2].to_string()));
    }
    let words: Vec<&str> = l.split_whitespace().collect();
    for k in 1..=5.min(words.len().saturating_sub(1)) {
        // "Kr density", "element 12 boiling point"
        tries.push((words[words.len() - k..].join(" "), words[..words.len() - k].join(" ")));
        // "density Kr", "abundance Na"
        tries.push((words[..k].join(" "), words[k..].join(" ")));
    }
    for (p, el) in tries {
        let Some(prop) = canonical_prop(&p) else { continue };
        let el_text = el.trim().trim_start_matches("element ").to_string();
        let el_text = if el.trim().starts_with("element ") && el_text.chars().all(|c| c.is_ascii_digit()) { el.trim().to_string() } else { el_text };
        if let Some(e) = element_ref(&el_text) {
            let v = element_property(e, prop)?;
            let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} | {prop}", e.name))), primary(pod("Result", "Result", text(v)))];
            if prop.contains("configuration") || prop == "orbital diagram" || prop == "valence electrons" {
                for extra in ["electron configuration", "abbreviated electron configuration", "valence electrons"] {
                    if extra != prop {
                        if let Some(x) = element_property(e, extra) {
                            pods.push(pod("Related", extra, text(x)));
                        }
                    }
                }
            }
            pods.extend(element_pods(e).into_iter().skip(1).take(2).map(|mut p| {
                p.primary = false;
                p
            }));
            return Some(Answer::new(format!("{prop} of {}", e.name), pods));
        }
        if let Some(list) = element_list(&el_text) {
            return Some(element_table(&list, &[prop]));
        }
        if let Some((series, list)) = element_group(&el_text) {
            return Some(group_table(series, &list, prop));
        }
    }
    if let Some(a) = element_ranking(l) {
        return Some(a);
    }
    if let Some(a) = group_comparison(l) {
        return Some(a);
    }
    if re!(r"^(?:how many|number of) (?:chemical )?elements (?:are )?(?:there )?(?:in the periodic table|are there|exist|known)?$").is_match(l) || l == "elements in the periodic table" {
        let pods = vec![pod("Input", "Input interpretation", text("number of chemical elements")), primary(pod("Result", "Result", text("118 known elements (hydrogen, Z = 1, to oganesson, Z = 118)"))), pod("Natural", "Occurring naturally", text("94 occur in nature; the rest have only been made artificially"))];
        return Some(Answer::new("number of chemical elements", pods));
    }
    // a bare list of elements: "silver, gold", "N, O, F"
    if let Some(list) = element_list(l) {
        return Some(element_table(&list, &["atomic number", "atomic mass", "electron configuration", "density", "melting point", "boiling point", "electronegativity", "atomic radius", "ionization energy"]));
    }
    None
}

/// "lanthanides", "alkaline earth metals" → the members.
pub(super) fn element_group(t: &str) -> Option<(&'static str, Vec<&'static Element>)> {
    let t = t.trim().trim_start_matches("the ").trim_end_matches(" elements").trim();
    let by_group = |g: u32| -> Vec<&'static Element> { elements().iter().filter(|e| e.group == Some(g) && e.z <= 118).collect() };
    if matches!(t, "metals" | "metal" | "metallic elements") {
        const METAL_SERIES: [&str; 6] = ["Alkali metals", "Alkaline earth metals", "Transition metals", "Lanthanides", "Actinides", "Poor metals"];
        let list: Vec<&Element> = elements().iter().filter(|e| e.z <= 103 && e.series.as_deref().is_some_and(|s| METAL_SERIES.contains(&s))).collect();
        return (!list.is_empty()).then_some(("Metals", list));
    }
    // blocks of the periodic table (He is s-block by its configuration)
    if let Some(b) = re!(r"^([spdf])[- ]?block$").captures(t).map(|m| m[1].to_string()) {
        let in_block = |e: &&Element| match (b.as_str(), e.group) {
            ("s", Some(g)) => g <= 2 || e.z == 2,
            ("p", Some(g)) => g >= 13 && e.z != 2,
            ("d", Some(g)) => (3..=12).contains(&g),
            ("f", None) => true,
            _ => false,
        };
        let list: Vec<&Element> = elements().iter().filter(|e| e.z <= 118).filter(in_block).collect();
        let title = match b.as_str() { "s" => "s-block elements", "p" => "p-block elements", "d" => "d-block elements", _ => "f-block elements" };
        return (!list.is_empty()).then_some((title, list));
    }
    if t == "coinage metals" || t == "coinage metal" {
        return Some(("Coinage metals", by_group(11).into_iter().filter(|e| e.z < 100).collect()));
    }
    if let Some(g) = t.strip_prefix("group ").and_then(|g| g.parse::<u32>().ok()).filter(|g| (1..=18).contains(g)) {
        let list = by_group(g);
        return (!list.is_empty()).then_some(("Group", list));
    }
    let (_, series) = super::knowledge::GROUPS.iter().find(|(names, _)| names.contains(&t))?;
    let list: Vec<&Element> = elements().iter().filter(|e| e.series.as_deref() == Some(*series)).collect();
    (!list.is_empty()).then_some((*series, list))
}

/// "nonmetals vs alkali metals": members and the median of each property.
fn group_comparison(l: &str) -> Option<Answer> {
    let l = l.trim_start_matches("compare ");
    let m = re!(r"^(.+?)\s+(?:vs\.?|versus|compared (?:to|with)|and)\s+(.+)$").captures(l)?;
    let (a, b) = (element_group(&m[1])?, element_group(&m[2])?);
    let median = |list: &[&'static Element], prop: &str| -> Option<f64> {
        let mut v: Vec<f64> = list.iter().filter(|e| e.z <= 103).filter_map(|e| rank_value(e, prop)).collect();
        if v.is_empty() {
            return None;
        }
        v.sort_by(|x, y| x.total_cmp(y));
        let n = v.len();
        Some(if n % 2 == 1 { v[n / 2] } else { 0.5 * (v[n / 2 - 1] + v[n / 2]) })
    };
    let props = [("melting point", "K"), ("boiling point", "K"), ("density", "g/cm³"), ("electronegativity", ""), ("atomic radius", "pm"), ("ionization energy", "eV"), ("atomic mass", "u")];
    let f = |v: Option<f64>, u: &str| v.map_or("—".to_string(), |x| format!("{}{}{u}", crate::print::fmt_float(x, 4), if u.is_empty() { "" } else { " " }));
    let mut rows = vec![vec![text("number of elements"), text(a.1.len().to_string()), text(b.1.len().to_string())]];
    for (p, u) in props {
        rows.push(vec![text(format!("median {p}")), text(f(median(&a.1, p), u)), text(f(median(&b.1, p), u))]);
    }
    let members = |list: &[&'static Element]| list.iter().map(|e| e.name.clone()).collect::<Vec<_>>().join(", ");
    let (na, nb) = (a.0.to_lowercase(), b.0.to_lowercase());
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("{na} | {nb}"))),
        primary(pod("Result", "Comparison (medians over the members)", table(Some(vec!["property", a.0, b.0]), rows))),
        pod("MembersA", a.0, text(members(&a.1))),
        pod("MembersB", b.0, text(members(&b.1))),
    ];
    Some(Answer::new(format!("{na} vs {nb}"), pods))
}

fn group_table(series: &str, list: &[&'static Element], prop: &str) -> Answer {
    let rows = list.iter().map(|e| vec![text(e.name.clone()), text(element_property(e, prop).unwrap_or_else(|| "—".into()))]).collect();
    let pods = vec![pod("Input", "Input interpretation", text(format!("{} | {prop}", series.to_lowercase()))), primary(pod("Result", "Result", table(Some(vec!["element", prop]), rows)))];
    Answer::new(format!("{prop} of {}", series.to_lowercase()), pods)
}

/// A numeric property for ranking: (value, property name).
fn rank_value(e: &Element, prop: &str) -> Option<f64> {
    match prop {
        "density" => e.density_g_cm3,
        "atomic mass" => Some(e.mass),
        "melting point" => e.melting_k,
        "boiling point" => e.boiling_k,
        "electronegativity" => e.electronegativity,
        "atomic radius" => e.atomic_radius_pm,
        "ionization energy" => e.first_ionization_ev,
        "abundance" => e.abundance_crust_mg_kg,
        _ => None,
    }
}

/// "10 densest elements", "lightest element", "element with the highest melting point"
fn element_ranking(l: &str) -> Option<Answer> {
    let (count, prop, high) = if let Some(m) = re!(r"^(?:the )?(?:top )?(\d+ )?(densest|most dense|least dense|lightest|heaviest|most electronegative|least electronegative|largest|biggest|smallest|most abundant|least abundant)(?: (?:chemical )?elements?)?(?: in the periodic table)?$").captures(l) {
        let (p, h) = match &m[2] {
            "densest" | "most dense" => ("density", true),
            "least dense" => ("density", false),
            "lightest" => ("atomic mass", false),
            "heaviest" => ("atomic mass", true),
            "most electronegative" => ("electronegativity", true),
            "least electronegative" => ("electronegativity", false),
            "largest" | "biggest" => ("atomic radius", true),
            "smallest" => ("atomic radius", false),
            "most abundant" => ("abundance", true),
            _ => ("abundance", false),
        };
        (m.get(1).and_then(|x| x.as_str().trim().parse::<usize>().ok()), p, h)
    } else if let Some(m) = re!(r"^(?:the )?(\d+ )?(?:chemical )?elements? with (?:the )?(highest|lowest|largest|smallest|greatest|biggest) (.+)$").captures(l) {
        let p = canonical_prop(&m[3])?;
        (m.get(1).and_then(|x| x.as_str().trim().parse::<usize>().ok()), p, matches!(&m[2], "highest" | "largest" | "greatest" | "biggest"))
    } else {
        return None;
    };
    // superheavy elements have only predicted bulk properties
    let mut list: Vec<(&Element, f64)> = elements().iter().filter(|e| e.z <= 103 || prop == "atomic mass").filter_map(|e| rank_value(e, prop).map(|v| (e, v))).collect();
    if list.is_empty() {
        return None;
    }
    list.sort_by(|a, b| if high { b.1.partial_cmp(&a.1).unwrap() } else { a.1.partial_cmp(&b.1).unwrap() });
    let k = count.unwrap_or(1).clamp(1, 118);
    let top = &list[..k.min(list.len())];
    let rows: Vec<Vec<Content>> = top.iter().enumerate().map(|(i, (e, _))| vec![text((i + 1).to_string()), text(e.name.clone()), text(element_property(e, prop).unwrap_or_default())]).collect();
    let res = if k == 1 { format!("{} ({})", top[0].0.name, element_property(top[0].0, prop).unwrap_or_default()) } else { top.iter().map(|(e, _)| e.name.clone()).collect::<Vec<_>>().join(", ") };
    let word = if high { "highest" } else { "lowest" };
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("elements with the {word} {prop}"))), primary(pod("Result", "Result", text(res)))];
    pods.push(pod("Table", "Ranking", table(Some(vec!["rank", "element", prop]), if k == 1 { list.iter().take(5).enumerate().map(|(i, (e, _))| vec![text((i + 1).to_string()), text(e.name.clone()), text(element_property(e, prop).unwrap_or_default())]).collect() } else { rows })));
    Some(Answer::new(format!("elements by {prop}"), pods))
}

fn element_table(list: &[&'static Element], props: &[&str]) -> Answer {
    let mut header = vec!["property"];
    header.extend(list.iter().map(|e| e.name.as_str()));
    let rows = props
        .iter()
        .map(|p| {
            let mut r = vec![text(p.to_string())];
            r.extend(list.iter().map(|e| text(element_property(e, p).unwrap_or_else(|| "—".into()))));
            r
        })
        .collect();
    let names = list.iter().map(|e| e.name.clone()).collect::<Vec<_>>().join(" | ");
    let pods = vec![pod("Input", "Input interpretation", text(format!("{names} | {}", props.join(", ")))), primary(pod("Result", "Comparison", table(Some(header), rows)))];
    Answer::new(format!("{names} comparison"), pods)
}
