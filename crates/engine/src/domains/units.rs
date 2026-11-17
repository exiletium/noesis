//! Units of measurement, conversions and physical constants.

use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::sync::OnceLock;

/// Exponents of (length, mass, time, current, temperature, amount,
/// luminous intensity, information).
pub type Dim = [i8; 8];

const NONE: Dim = [0; 8];

#[derive(Clone, Debug)]
pub struct Quantity {
    pub value: f64,
    pub dim: Dim,
}

#[derive(Clone, Copy)]
struct UnitDef {
    names: &'static [&'static str],
    factor: f64,
    dim: Dim,
    /// Affine offset (temperatures): SI = (v + offset) * factor.
    offset: f64,
    prefixable: bool,
    long: &'static str,
}

const fn d(l: i8, m: i8, t: i8, i: i8, k: i8, n: i8, j: i8, b: i8) -> Dim {
    [l, m, t, i, k, n, j, b]
}

macro_rules! u {
    ($names:expr, $f:expr, $dim:expr, $long:expr) => {
        UnitDef { names: $names, factor: $f, dim: $dim, offset: 0.0, prefixable: false, long: $long }
    };
    ($names:expr, $f:expr, $dim:expr, $long:expr, p) => {
        UnitDef { names: $names, factor: $f, dim: $dim, offset: 0.0, prefixable: true, long: $long }
    };
}

const L: Dim = d(1, 0, 0, 0, 0, 0, 0, 0);
const M: Dim = d(0, 1, 0, 0, 0, 0, 0, 0);
const T: Dim = d(0, 0, 1, 0, 0, 0, 0, 0);
const AREA: Dim = d(2, 0, 0, 0, 0, 0, 0, 0);
const VOL: Dim = d(3, 0, 0, 0, 0, 0, 0, 0);
const SPEED: Dim = d(1, 0, -1, 0, 0, 0, 0, 0);
const ACC: Dim = d(1, 0, -2, 0, 0, 0, 0, 0);
const FORCE: Dim = d(1, 1, -2, 0, 0, 0, 0, 0);
const ENERGY: Dim = d(2, 1, -2, 0, 0, 0, 0, 0);
const POWER: Dim = d(2, 1, -3, 0, 0, 0, 0, 0);
const PRESSURE: Dim = d(-1, 1, -2, 0, 0, 0, 0, 0);
const FREQ: Dim = d(0, 0, -1, 0, 0, 0, 0, 0);
const CHARGE: Dim = d(0, 0, 1, 1, 0, 0, 0, 0);
const VOLT: Dim = d(2, 1, -3, -1, 0, 0, 0, 0);
const OHM: Dim = d(2, 1, -3, -2, 0, 0, 0, 0);
const TEMP: Dim = d(0, 0, 0, 0, 1, 0, 0, 0);
const BITS: Dim = d(0, 0, 0, 0, 0, 0, 0, 1);

static UNITS: &[UnitDef] = &[
    // length
    u!(&["m", "meter", "meters", "metre", "metres"], 1.0, L, "meters", p),
    u!(&["in", "inch", "inches", "\""], 0.0254, L, "inches"),
    u!(&["ft", "foot", "feet", "'"], 0.3048, L, "feet"),
    u!(&["yd", "yard", "yards"], 0.9144, L, "yards"),
    u!(&["mi", "mile", "miles"], 1609.344, L, "miles"),
    u!(&["nmi", "nautical mile", "nautical miles"], 1852.0, L, "nautical miles"),
    u!(&["au", "astronomical unit", "astronomical units"], 1.495978707e11, L, "astronomical units"),
    u!(&["ly", "light year", "light years", "lightyear", "lightyears", "light-year", "light-years"], 9.4607304725808e15, L, "light years"),
    u!(&["pc", "parsec", "parsecs"], 3.0856775814913673e16, L, "parsecs", p),
    u!(&["å", "angstrom", "angstroms"], 1e-10, L, "ångströms"),
    u!(&["furlong", "furlongs"], 201.168, L, "furlongs"),
    u!(&["fathom", "fathoms"], 1.8288, L, "fathoms"),
    u!(&["chain", "chains"], 20.1168, L, "chains"),
    u!(&["rod", "rods", "perch", "pole"], 5.0292, L, "rods"),
    u!(&["league", "leagues"], 4828.032, L, "leagues"),
    u!(&["hand", "hands"], 0.1016, L, "hands"),
    u!(&["mil", "mils", "thou"], 2.54e-5, L, "mils"),
    u!(&["micron", "microns"], 1e-6, L, "microns"),
    u!(&["pica", "picas"], 0.0254 / 6.0, L, "picas"),
    u!(&["point", "points"], 0.0254 / 72.0, L, "points"),
    u!(&["cubit", "cubits"], 0.4572, L, "cubits"),
    u!(&["fermi", "fermis"], 1e-15, L, "fermis"),
    // mass
    u!(&["g", "gram", "grams", "gramme"], 1e-3, M, "grams", p),
    u!(&["t", "tonne", "tonnes", "metric ton", "metric tons"], 1000.0, M, "metric tons"),
    u!(&["lb", "lbs", "pound", "pounds"], 0.45359237, M, "pounds"),
    u!(&["oz", "ounce", "ounces"], 0.028349523125, M, "ounces"),
    u!(&["st", "stone", "stones"], 6.35029318, M, "stone"),
    u!(&["ton", "tons", "short ton", "short tons"], 907.18474, M, "short tons"),
    u!(&["u", "amu", "dalton", "daltons", "da"], 1.66053906660e-27, M, "atomic mass units"),
    u!(&["ct", "carat", "carats"], 2e-4, M, "carats"),
    u!(&["gr", "grain", "grains"], 6.479891e-5, M, "grains"),
    u!(&["dr", "dram", "drams", "drachm", "drachms"], 1.7718451953125e-3, M, "drams"),
    u!(&["dwt", "pennyweight", "pennyweights"], 1.55517384e-3, M, "pennyweights"),
    u!(&["ozt", "troy ounce", "troy ounces"], 0.0311034768, M, "troy ounces"),
    u!(&["slug", "slugs"], 14.593902937, M, "slugs"),
    u!(&["long ton", "long tons", "imperial ton", "imperial tons"], 1016.0469088, M, "long tons"),
    u!(&["cwt", "hundredweight", "hundredweights"], 45.359237, M, "hundredweights"),
    // time
    u!(&["s", "sec", "secs", "second", "seconds"], 1.0, T, "seconds", p),
    u!(&["min", "mins", "minute", "minutes"], 60.0, T, "minutes"),
    u!(&["h", "hr", "hrs", "hour", "hours"], 3600.0, T, "hours"),
    u!(&["day", "days", "d"], 86400.0, T, "days"),
    u!(&["wk", "week", "weeks"], 604800.0, T, "weeks"),
    u!(&["month", "months"], 2629746.0, T, "months"),
    u!(&["yr", "year", "years", "annum"], 31556952.0, T, "years"),
    u!(&["gyr", "gigayear", "gigayears", "billion years"], 31556952.0e9, T, "billion years"),
    u!(&["myr", "megayear", "megayears", "million years"], 31556952.0e6, T, "million years"),
    u!(&["kyr", "kiloyear", "kiloyears", "thousand years"], 31556952.0e3, T, "thousand years"),
    u!(&["decade", "decades"], 315569520.0, T, "decades"),
    u!(&["century", "centuries"], 3155695200.0, T, "centuries"),
    // area & volume
    u!(&["ha", "hectare", "hectares"], 1e4, AREA, "hectares"),
    u!(&["acre", "acres", "ac"], 4046.8564224, AREA, "acres"),
    u!(&["l", "L", "liter", "liters", "litre", "litres"], 1e-3, VOL, "liters", p),
    u!(&["gal", "gallon", "gallons"], 3.785411784e-3, VOL, "US gallons"),
    u!(&["qt", "quart", "quarts"], 9.46352946e-4, VOL, "US quarts"),
    u!(&["pt", "pint", "pints"], 4.73176473e-4, VOL, "US pints"),
    u!(&["cup", "cups"], 2.365882365e-4, VOL, "US cups"),
    u!(&["floz", "fl oz", "fluid ounce", "fluid ounces"], 2.95735295625e-5, VOL, "US fluid ounces"),
    u!(&["tbsp", "tablespoon", "tablespoons"], 1.478676478125e-5, VOL, "tablespoons"),
    u!(&["tsp", "teaspoon", "teaspoons"], 4.92892159375e-6, VOL, "teaspoons"),
    u!(&["fl dr", "fluid dram", "fluid drams", "fluidram", "fluidrams"], 3.6966911953125e-6, VOL, "US fluid drams"),
    u!(&["gill", "gills"], 1.18294118250e-4, VOL, "US gills"),
    u!(&["bbl", "barrel", "barrels", "oil barrel", "oil barrels"], 0.158987294928, VOL, "oil barrels"),
    u!(&["bu", "bushel", "bushels"], 0.03523907016688, VOL, "US bushels"),
    u!(&["pk", "peck", "pecks"], 0.00880976754172, VOL, "US pecks"),
    u!(&["cc", "ccs"], 1e-6, VOL, "cubic centimeters"),
    u!(&["imperial gallon", "imperial gallons", "uk gallon", "uk gallons"], 4.54609e-3, VOL, "imperial gallons"),
    u!(&["imperial pint", "imperial pints", "uk pint", "uk pints"], 5.6826125e-4, VOL, "imperial pints"),
    u!(&["acre foot", "acre feet", "acre-foot", "acre-feet"], 1233.48183754752, VOL, "acre-feet"),
    u!(&["danish pot", "danish pots", "pot", "pots"], 0.9661e-3, VOL, "Danish pots"),
    u!(&["drop", "drops"], 5e-8, VOL, "drops"),
    u!(&["barn", "barns"], 1e-28, AREA, "barns", p),
    // speed
    u!(&["mph"], 0.44704, SPEED, "miles per hour"),
    u!(&["mpg", "miles per gallon"], 425_143.707_430_272_7, d(-2, 0, 0, 0, 0, 0, 0, 0), "miles per gallon"),
    u!(&["kph", "kmh"], 1.0 / 3.6, SPEED, "kilometers per hour"),
    u!(&["kn", "knot", "knots", "kt"], 1852.0 / 3600.0, SPEED, "knots"),
    u!(&["mach"], 340.29, SPEED, "Mach (sea level)"),
    u!(&["c", "speed of light"], 299792458.0, SPEED, "speed of light"),
    // acceleration / force
    u!(&["gee", "g0", "standard gravity"], 9.80665, ACC, "standard gravities"),
    u!(&["n", "newton", "newtons"], 1.0, FORCE, "newtons", p),
    u!(&["lbf", "pound force", "pounds force"], 4.4482216152605, FORCE, "pounds-force"),
    u!(&["dyn", "dyne", "dynes"], 1e-5, FORCE, "dynes"),
    // energy
    u!(&["j", "joule", "joules"], 1.0, ENERGY, "joules", p),
    u!(&["cal", "calorie", "calories"], 4.184, ENERGY, "calories", p),
    u!(&["kcal", "kilocalorie", "kilocalories", "food calorie", "food calories", "cal_food"], 4184.0, ENERGY, "kilocalories"),
    u!(&["ev", "electronvolt", "electronvolts", "electron volt", "electron volts"], 1.602176634e-19, ENERGY, "electronvolts", p),
    u!(&["wh", "watt hour", "watt hours", "watt-hour", "watt-hours"], 3600.0, ENERGY, "watt hours", p),
    u!(&["btu", "btus"], 1055.05585262, ENERGY, "British thermal units"),
    u!(&["erg", "ergs"], 1e-7, ENERGY, "ergs"),
    u!(&["tnt", "ton of tnt", "tons of tnt"], 4.184e9, ENERGY, "tons of TNT"),
    u!(&["ft lbf", "ft·lbf", "foot pound", "foot pounds", "foot-pound", "foot-pounds", "ft-lb", "ft lb", "ft-lbs", "foot pound force"], 1.3558179483314004, ENERGY, "foot-pounds"),
    u!(&["therm", "therms"], 1.05505585262e8, ENERGY, "therms"),
    u!(&["quad", "quads"], 1.05505585262e18, ENERGY, "quads"),
    u!(&["hartree", "hartrees"], 4.3597447222071e-18, ENERGY, "hartrees"),
    u!(&["rydberg", "rydbergs"], 2.1798723611035e-18, ENERGY, "rydbergs"),
    // power
    u!(&["w", "watt", "watts"], 1.0, POWER, "watts", p),
    u!(&["hp", "horsepower"], 745.69987158227022, POWER, "horsepower"),
    u!(&["ton of refrigeration", "tons of refrigeration", "refrigeration ton"], 3516.8528420667, POWER, "tons of refrigeration"),
    // pressure
    u!(&["pa", "pascal", "pascals"], 1.0, PRESSURE, "pascals", p),
    u!(&["bar", "bars"], 1e5, PRESSURE, "bars", p),
    u!(&["atm", "atmosphere", "atmospheres"], 101325.0, PRESSURE, "atmospheres"),
    u!(&["psi"], 6894.757293168, PRESSURE, "pounds per square inch"),
    u!(&["mmhg", "torr"], 133.322387415, PRESSURE, "millimeters of mercury"),
    u!(&["inhg", "inches of mercury"], 3386.389, PRESSURE, "inches of mercury"),
    u!(&["mmh2o", "millimeters of water"], 9.80665, PRESSURE, "millimeters of water"),
    u!(&["ksi"], 6894757.293168, PRESSURE, "kilopounds per square inch"),
    // electromagnetism and others
    u!(&["hz", "hertz"], 1.0, FREQ, "hertz", p),
    u!(&["rpm"], 1.0 / 60.0, FREQ, "revolutions per minute"),
    u!(&["a", "amp", "amps", "ampere", "amperes"], 1.0, d(0, 0, 0, 1, 0, 0, 0, 0), "amperes", p),
    u!(&["c", "coulomb", "coulombs"], 1.0, CHARGE, "coulombs", p),
    u!(&["v", "volt", "volts"], 1.0, VOLT, "volts", p),
    u!(&["ω", "ohm", "ohms"], 1.0, OHM, "ohms", p),
    u!(&["f", "farad", "farads"], 1.0, d(-2, -1, 4, 2, 0, 0, 0, 0), "farads", p),
    u!(&["T", "tesla", "teslas"], 1.0, d(0, 1, -2, -1, 0, 0, 0, 0), "teslas", p),
    u!(&["henry", "henries", "henrys"], 1.0, d(2, 1, -2, -2, 0, 0, 0, 0), "henries", p),
    u!(&["wb", "weber", "webers"], 1.0, d(2, 1, -2, -1, 0, 0, 0, 0), "webers", p),
    u!(&["gauss"], 1e-4, d(0, 1, -2, -1, 0, 0, 0, 0), "gauss", p),
    u!(&["siemens"], 1.0, d(-2, -1, 3, 2, 0, 0, 0, 0), "siemens", p),
    u!(&["solar mass", "solar masses", "msun", "m_sun", "sun masses"], 1.98847e30, M, "solar masses"),
    u!(&["earth mass", "earth masses", "mearth"], 5.9722e24, M, "Earth masses"),
    u!(&["jupiter mass", "jupiter masses"], 1.89813e27, M, "Jupiter masses"),
    u!(&["mol", "mole", "moles"], 1.0, d(0, 0, 0, 0, 0, 1, 0, 0), "moles", p),
    u!(&["ci", "curie", "curies"], 3.7e10, FREQ, "curies", p),
    u!(&["bq", "becquerel", "becquerels"], 1.0, FREQ, "becquerels", p),
    u!(&["gy", "gray", "grays"], 1.0, d(2, 0, -2, 0, 0, 0, 0, 0), "grays", p),
    u!(&["sv", "sievert", "sieverts"], 1.0, d(2, 0, -2, 0, 0, 0, 0, 0), "sieverts", p),
    u!(&["rem", "rems"], 0.01, d(2, 0, -2, 0, 0, 0, 0, 0), "rem", p),
    u!(&["roentgen", "roentgens"], 2.58e-4, d(0, -1, 1, 1, 0, 0, 0, 0), "roentgens", p),
    u!(&["poise"], 0.1, d(-1, 1, -1, 0, 0, 0, 0, 0), "poise", p),
    u!(&["cP", "centipoise"], 1e-3, d(-1, 1, -1, 0, 0, 0, 0, 0), "centipoise", p),
    u!(&["stokes"], 1e-4, d(2, 0, -1, 0, 0, 0, 0, 0), "stokes", p),
    u!(&["cSt", "centistokes"], 1e-6, d(2, 0, -1, 0, 0, 0, 0, 0), "centistokes", p),
    u!(&["lm", "lumen", "lumens"], 1.0, d(0, 0, 0, 0, 0, 0, 1, 0), "lumens", p),
    u!(&["lx", "lux"], 1.0, d(-2, 0, 0, 0, 0, 0, 1, 0), "lux", p),
    u!(&["kat", "katal", "katals"], 1.0, d(0, 0, -1, 0, 0, 1, 0, 0), "katals", p),
    u!(&["cd", "candela"], 1.0, d(0, 0, 0, 0, 0, 0, 1, 0), "candelas", p),
    // temperature (absolute scales)
    u!(&["k", "kelvin", "kelvins"], 1.0, TEMP, "kelvins", p),
    UnitDef { names: &["°c", "c", "celsius", "degc", "degrees celsius", "degree celsius", "centigrade"], factor: 1.0, dim: TEMP, offset: 273.15, prefixable: false, long: "degrees Celsius" },
    UnitDef { names: &["°f", "f", "fahrenheit", "degf", "degrees fahrenheit", "degree fahrenheit"], factor: 5.0 / 9.0, dim: TEMP, offset: 459.67, prefixable: false, long: "degrees Fahrenheit" },
    UnitDef { names: &["°r", "rankine"], factor: 5.0 / 9.0, dim: TEMP, offset: 0.0, prefixable: false, long: "degrees Rankine" },
    // information
    u!(&["bit", "bits", "b"], 1.0, BITS, "bits", p),
    u!(&["byte", "bytes", "B"], 8.0, BITS, "bytes", p),
    // angles (dimensionless)
    u!(&["rad", "radian", "radians"], 1.0, NONE, "radians"),
    u!(&["deg", "degree", "degrees", "°"], std::f64::consts::PI / 180.0, NONE, "degrees"),
    u!(&["rev", "revolution", "revolutions", "turn", "turns"], std::f64::consts::TAU, NONE, "revolutions"),
    u!(&["percent", "%"], 0.01, NONE, "percent"),
];

const PREFIXES: &[(&str, &str, f64)] = &[
    ("Y", "yotta", 1e24),
    ("Z", "zetta", 1e21),
    ("E", "exa", 1e18),
    ("P", "peta", 1e15),
    ("T", "tera", 1e12),
    ("G", "giga", 1e9),
    ("M", "mega", 1e6),
    ("k", "kilo", 1e3),
    ("h", "hecto", 1e2),
    ("da", "deca", 1e1),
    ("d", "deci", 1e-1),
    ("c", "centi", 1e-2),
    ("m", "milli", 1e-3),
    ("µ", "micro", 1e-6),
    ("μ", "micro", 1e-6),
    ("u", "micro", 1e-6),
    ("n", "nano", 1e-9),
    ("p", "pico", 1e-12),
    ("f", "femto", 1e-15),
    ("a", "atto", 1e-18),
    ("Ki", "kibi", 1024.0),
    ("Mi", "mebi", 1048576.0),
    ("Gi", "gibi", 1073741824.0),
    ("Ti", "tebi", 1099511627776.0),
];

/// A resolved unit (possibly prefixed).
#[derive(Clone, Debug)]
pub struct Unit {
    pub factor: f64,
    pub dim: Dim,
    pub offset: f64,
    pub name: String,
}

fn lookup_word(w: &str) -> Option<Unit> {
    let lw = w.to_lowercase();
    // "kN", "mV", "kJ": a lowercase prefix followed by an uppercase symbol is
    // never a plain two-letter unit such as "kn" (knots)
    let prefixed_symbol = w.chars().count() == 2 && w.chars().next().is_some_and(|c| c.is_lowercase()) && w.chars().nth(1).is_some_and(|c| c.is_uppercase());
    if prefixed_symbol {
        if let Some(u) = lookup_prefixed(w, &lw) {
            return Some(u);
        }
    }
    // exact (case-insensitive for multi-letter names, sensitive for B/b)
    for u in UNITS {
        for n in u.names {
            // capital C is the coulomb, never the speed of light
            let hit = if n.len() <= 2 { *n == w || (*n == lw && !matches!(w, "B" | "b" | "M" | "G" | "T" | "P") && !(w == "C" && u.long == "speed of light")) } else { *n == lw };
            if hit {
                return Some(Unit { factor: u.factor, dim: u.dim, offset: u.offset, name: u.long.to_string() });
            }
        }
    }
    lookup_prefixed(w, &lw)
}

fn lookup_prefixed(w: &str, lw: &str) -> Option<Unit> {
    // prefixed symbols (case-sensitive) and long names
    for (sym, long, f) in PREFIXES {
        for (pre, rest) in [(*sym, w.strip_prefix(sym)), (*long, lw.strip_prefix(long))] {
            let Some(rest) = rest else { continue };
            if rest.is_empty() {
                continue;
            }
            for u in UNITS.iter().filter(|u| u.prefixable) {
                let hit = if pre == *sym { u.names.iter().any(|n| *n == rest || (n.len() > 1 && *n == rest.to_lowercase()) || (n.len() == 1 && rest.len() == 1 && *n != "b" && *n == rest.to_lowercase())) } else { u.names.iter().any(|n| *n == rest) };
                // binary prefixes only for information units
                if hit && (!sym.ends_with('i') || u.dim == BITS) {
                    let lname = if pre == *sym { format!("{long}{}", u.long) } else { format!("{long}{}", u.long) };
                    return Some(Unit { factor: u.factor * f, dim: u.dim, offset: 0.0, name: lname });
                }
            }
        }
    }
    None
}

pub fn singular(name: &str) -> String {
    let (head, tail) = match name.split_once(' ') {
        Some(("feet", t)) => return format!("foot {t}"),
        Some((h, t)) if !h.ends_with('s') => return format!("{h} {}", singular(t)),
        Some((h, t)) => (h.to_string(), format!(" {t}")),
        None => (name.to_string(), String::new()),
    };
    let h = if head == "feet" {
        "foot".to_string()
    } else if let Some(b) = head.strip_suffix("ies") {
        format!("{b}y")
    } else if let Some(b) = head.strip_suffix("ches") {
        format!("{b}ch")
    } else if head.ends_with('s') && !head.ends_with("ss") && head != "hertz" && head != "celsius" {
        head[..head.len() - 1].to_string()
    } else {
        head
    };
    format!("{h}{tail}")
}

/// Temperature reading of single-letter unit names (`C`, `F`).
fn temperature_alt(s: &str) -> Option<Unit> {
    match s.trim() {
        "C" | "c" | "°C" => lookup_word("celsius"),
        "F" | "f" | "°F" => lookup_word("fahrenheit"),
        "K" | "k" => lookup_word("kelvin"),
        _ => None,
    }
}

/// Parse a unit expression: `km/h`, `kg m/s^2`, `m^2`, `N·m`, `square feet`.
pub fn parse_unit(s: &str) -> Option<Unit> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(u) = lookup_word(s) {
        return Some(u);
    }
    // natural-language powers and "per"
    let mut t = format!(" {} ", s.to_lowercase().replace(" per ", " / ").replace('·', " ").replace('*', " "));
    for (w, p) in [("square ", "^2"), ("sq ", "^2"), ("cubic ", "^3"), ("cu ", "^3")] {
        while let Some(k) = t.find(w) {
            // move the power after the next word
            let after = &t[k + w.len()..];
            let end = after.find([' ', '/']).unwrap_or(after.len());
            let word = after[..end].to_string();
            t = format!("{}{}{}{}", &t[..k], word, p, &after[end..]);
        }
    }
    // keep original case for symbols when possible
    let src = if t.trim() == s.to_lowercase() { s.to_string() } else { t.trim().to_string() };
    let mut factor = 1.0;
    let mut dim = NONE;
    let mut name_parts = Vec::new();
    let mut sign = 1i8;
    for tok in src.split_inclusive('/') {
        let (body, slash) = match tok.strip_suffix('/') {
            Some(b) => (b, true),
            None => (tok, false),
        };
        for w in body.split_whitespace() {
            let (base, pw) = match w.split_once('^') {
                Some((b, p)) => (b, p.parse::<i8>().ok()?),
                None => {
                    let digits: String = w.chars().rev().take_while(|c| c.is_ascii_digit()).collect();
                    if !digits.is_empty() && digits.len() < w.len() && !w.chars().next()?.is_ascii_digit() {
                        (&w[..w.len() - digits.len()], digits.chars().rev().collect::<String>().parse().ok()?)
                    } else {
                        (w, 1)
                    }
                }
            };
            let u = lookup_word(base)?;
            if u.offset != 0.0 && (pw != 1 || src.contains('/')) {
                return None;
            }
            let p = pw * sign;
            factor *= u.factor.powi(p as i32);
            for k in 0..8 {
                dim[k] += u.dim[k] * p;
            }
            let word = |n: &str, k: i8| -> String {
                match k {
                    1 => n.to_string(),
                    2 => format!("{n} squared"),
                    3 => format!("{n} cubed"),
                    _ => format!("{n}^{k}"),
                }
            };
            if p > 0 {
                name_parts.push(word(&u.name, p));
            } else {
                let single = singular(&u.name);
                name_parts.push(format!("per {}", word(&single, -p)));
            }
        }
        if slash {
            sign = -1;
        }
    }
    if name_parts.is_empty() {
        return None;
    }
    Some(Unit { factor, dim, offset: 0.0, name: name_parts.join(" ") })
}

pub fn to_si(v: f64, u: &Unit) -> f64 {
    (v + u.offset) * u.factor
}

pub fn from_si(v: f64, u: &Unit) -> f64 {
    v / u.factor - u.offset
}

/// Readable SI unit for a dimension vector.
pub fn si_name(dim: &Dim) -> String {
    let named: &[(Dim, &str)] = &[(FORCE, "N"), (ENERGY, "J"), (POWER, "W"), (PRESSURE, "Pa"), (FREQ, "Hz"), (CHARGE, "C"), (VOLT, "V"), (OHM, "Ω")];
    for (dd, n) in named {
        if dd == dim {
            return n.to_string();
        }
    }
    let syms = ["m", "kg", "s", "A", "K", "mol", "cd", "bit"];
    let mut num = Vec::new();
    let mut den = Vec::new();
    for k in 0..8 {
        let p = dim[k];
        if p > 0 {
            num.push(if p == 1 { syms[k].to_string() } else { format!("{}^{}", syms[k], p) });
        } else if p < 0 {
            den.push(if p == -1 { syms[k].to_string() } else { format!("{}^{}", syms[k], -p) });
        }
    }
    match (num.is_empty(), den.is_empty()) {
        (true, true) => String::new(),
        (false, true) => num.join(" "),
        (true, false) => format!("1/{}", den.join(" ")),
        (false, false) => format!("{}/{}", num.join(" "), if den.len() > 1 { format!("({})", den.join(" ")) } else { den[0].clone() }),
    }
}

/// Units shown as "additional conversions" for a dimension.
fn related(dim: &Dim) -> &'static [&'static str] {
    match *dim {
        L => &["m", "km", "cm", "mm", "mi", "ft", "in", "yd", "nmi"],
        M => &["kg", "g", "lb", "oz", "t", "st"],
        T => &["s", "min", "h", "day", "week", "year", "ms"],
        AREA => &["m^2", "km^2", "ft^2", "acre", "ha", "mi^2"],
        VOL => &["L", "mL", "m^3", "gal", "qt", "cup", "floz"],
        SPEED => &["m/s", "km/h", "mph", "kn", "ft/s"],
        ENERGY => &["J", "kJ", "cal", "kcal", "kWh", "eV", "BTU"],
        POWER => &["W", "kW", "hp"],
        PRESSURE => &["Pa", "kPa", "bar", "atm", "psi", "mmHg"],
        TEMP => &["K", "°C", "°F"],
        BITS => &["bit", "B", "kB", "MB", "GB", "KiB", "MiB", "GiB"],
        FORCE => &["N", "kN", "lbf", "dyn"],
        _ => &[],
    }
}

fn re(s: &str) -> Regex {
    Regex::new(s).unwrap()
}

/// Parse "number unit" (also "5 ft 3 in").
pub fn parse_quantity(s: &str) -> Option<(f64, Unit, String)> {
    static R: OnceLock<Regex> = OnceLock::new();
    let r = R.get_or_init(|| re(r"^\s*([-+]?(?:\d[\d,]*\.?\d*|\.\d+)(?:[eE][-+]?\d+)?|[a-z ]+?)\s*(.+?)\s*$"));
    // compound imperial lengths: 5 ft 3 in, 5'3"
    static R2: OnceLock<Regex> = OnceLock::new();
    let r2 = R2.get_or_init(|| re(r#"^\s*(\d+(?:\.\d+)?)\s*(?:ft|feet|foot|')\s*(\d+(?:\.\d+)?)\s*(?:in|inches|inch|")?\s*$"#));
    if let Some(m) = r2.captures(s) {
        let ft: f64 = m[1].parse().ok()?;
        let inch: f64 = m[2].parse().ok()?;
        let u = lookup_word("in")?;
        return Some((ft * 12.0 + inch, u, format!("{ft} ft {inch} in")));
    }
    let m = r.captures(s)?;
    let num_s = m[1].trim();
    let v: f64 = if let Ok(v) = num_s.replace(',', "").parse() {
        v
    } else {
        let words = crate::parse::words_to_numbers(num_s);
        words.trim().parse().ok()?
    };
    let u = parse_unit(&m[2])?;
    let shown = format!("{} {}", fmt_float(v, 10), m[2].trim());
    Some((v, u, shown))
}

// ---------------------------------------------------------------------------
// physical constants
// ---------------------------------------------------------------------------

pub struct Constant {
    pub names: &'static [&'static str],
    pub symbol: &'static str,
    pub value: f64,
    pub unit: &'static str,
    pub exact: bool,
}

pub static CONSTANTS: &[Constant] = &[
    Constant { names: &["speed of light", "speed of light in vacuum", "c"], symbol: "c", value: 299792458.0, unit: "m/s", exact: true },
    Constant { names: &["planck constant", "planck's constant", "plancks constant", "h"], symbol: "h", value: 6.62607015e-34, unit: "J s", exact: true },
    Constant { names: &["reduced planck constant", "hbar", "h bar", "dirac constant"], symbol: "ħ", value: 1.054571817e-34, unit: "J s", exact: false },
    Constant { names: &["gravitational constant", "newton's gravitational constant", "newtonian constant of gravitation", "big g"], symbol: "G", value: 6.67430e-11, unit: "m^3/(kg s^2)", exact: false },
    Constant { names: &["avogadro constant", "avogadro's number", "avogadros number", "avogadro number"], symbol: "N_A", value: 6.02214076e23, unit: "1/mol", exact: true },
    Constant { names: &["boltzmann constant", "boltzmann's constant"], symbol: "k_B", value: 1.380649e-23, unit: "J/K", exact: true },
    Constant { names: &["elementary charge", "electron charge", "charge of an electron"], symbol: "e", value: 1.602176634e-19, unit: "C", exact: true },
    Constant { names: &["electron mass", "mass of an electron", "mass of electron"], symbol: "m_e", value: 9.1093837015e-31, unit: "kg", exact: false },
    Constant { names: &["proton mass", "mass of a proton", "mass of proton"], symbol: "m_p", value: 1.67262192369e-27, unit: "kg", exact: false },
    Constant { names: &["neutron mass", "mass of a neutron", "mass of neutron"], symbol: "m_n", value: 1.67492749804e-27, unit: "kg", exact: false },
    Constant { names: &["vacuum permittivity", "permittivity of free space", "permittivity of vacuum", "permittivity of a vacuum", "electric constant", "epsilon 0"], symbol: "ε₀", value: 8.8541878128e-12, unit: "F/m", exact: false },
    Constant { names: &["vacuum permeability", "permeability of free space", "permeability of vacuum", "magnetic permeability of vacuum", "permeability of a vacuum", "magnetic constant", "mu 0"], symbol: "μ₀", value: 1.25663706212e-6, unit: "N/A^2", exact: false },
    Constant { names: &["gas constant", "molar gas constant", "ideal gas constant", "universal gas constant"], symbol: "R", value: 8.314462618, unit: "J/(mol K)", exact: true },
    Constant { names: &["standard gravity", "standard acceleration of gravity", "acceleration due to gravity", "gravity on earth"], symbol: "g", value: 9.80665, unit: "m/s^2", exact: true },
    Constant { names: &["stefan-boltzmann constant", "stefan boltzmann constant"], symbol: "σ", value: 5.670374419e-8, unit: "W/(m^2 K^4)", exact: false },
    Constant { names: &["fine structure constant", "fine-structure constant"], symbol: "α", value: 7.2973525693e-3, unit: "", exact: false },
    Constant { names: &["rydberg constant"], symbol: "R_∞", value: 10973731.568160, unit: "1/m", exact: false },
    Constant { names: &["bohr radius"], symbol: "a₀", value: 5.29177210903e-11, unit: "m", exact: false },
    Constant { names: &["atomic mass unit", "unified atomic mass unit", "dalton"], symbol: "u", value: 1.66053906660e-27, unit: "kg", exact: false },
    Constant { names: &["faraday constant"], symbol: "F", value: 96485.33212, unit: "C/mol", exact: true },
    Constant { names: &["mass of the earth", "earth mass", "mass of earth"], symbol: "M_⊕", value: 5.9722e24, unit: "kg", exact: false },
    Constant { names: &["radius of the earth", "earth radius", "radius of earth"], symbol: "R_⊕", value: 6.3710e6, unit: "m", exact: false },
    Constant { names: &["mass of the sun", "solar mass", "mass of sun", "sun mass"], symbol: "M_☉", value: 1.98847e30, unit: "kg", exact: false },
    Constant { names: &["radius of the sun", "solar radius"], symbol: "R_☉", value: 6.957e8, unit: "m", exact: false },
    Constant { names: &["mass of the moon", "lunar mass", "moon mass"], symbol: "M_☾", value: 7.342e22, unit: "kg", exact: false },
    Constant { names: &["distance to the moon", "earth moon distance", "distance from earth to moon"], symbol: "", value: 3.84399e8, unit: "m", exact: false },
    Constant { names: &["distance to the sun", "earth sun distance", "distance from earth to sun"], symbol: "AU", value: 1.495978707e11, unit: "m", exact: true },
    Constant { names: &["speed of sound", "speed of sound in air"], symbol: "", value: 343.0, unit: "m/s", exact: false },
    Constant { names: &["speed of sound in water", "speed of sound in fresh water"], symbol: "", value: 1482.0, unit: "m/s", exact: false },
    Constant { names: &["speed of sound in seawater", "speed of sound in sea water"], symbol: "", value: 1531.0, unit: "m/s", exact: false },
    Constant { names: &["speed of sound in steel"], symbol: "", value: 5960.0, unit: "m/s", exact: false },
    Constant { names: &["planck mass"], symbol: "m_P", value: 2.176434e-8, unit: "kg", exact: false },
    Constant { names: &["planck length"], symbol: "ℓ_P", value: 1.616255e-35, unit: "m", exact: false },
    Constant { names: &["planck time"], symbol: "t_P", value: 5.391247e-44, unit: "s", exact: false },
    Constant { names: &["planck temperature"], symbol: "T_P", value: 1.416784e32, unit: "K", exact: false },
    Constant { names: &["planck energy"], symbol: "E_P", value: 1.9561e9, unit: "J", exact: false },
    Constant { names: &["planck charge"], symbol: "q_P", value: 1.875545956e-18, unit: "C", exact: false },
    Constant { names: &["classical electron radius", "electron radius", "thomson scattering length"], symbol: "r_e", value: 2.8179403262e-15, unit: "m", exact: false },
    Constant { names: &["bohr magneton"], symbol: "μ_B", value: 9.2740100783e-24, unit: "J/T", exact: false },
    Constant { names: &["nuclear magneton"], symbol: "μ_N", value: 5.0507837461e-27, unit: "J/T", exact: false },
    Constant { names: &["thomson cross section", "thomson cross-section"], symbol: "σ_e", value: 6.6524587321e-29, unit: "m^2", exact: false },
    Constant { names: &["conductance quantum", "landauer conductance", "quantum of conductance"], symbol: "G₀", value: 7.748091729e-5, unit: "S", exact: true },
    Constant { names: &["magnetic flux quantum", "flux quantum", "superconducting magnetic flux quantum"], symbol: "Φ₀", value: 2.067833848e-15, unit: "Wb", exact: true },
    Constant { names: &["von klitzing constant", "klitzing constant", "resistance quantum"], symbol: "R_K", value: 25812.80745, unit: "ohm", exact: true },
    Constant { names: &["josephson constant"], symbol: "K_J", value: 483597.8484e9, unit: "Hz/V", exact: true },
    Constant { names: &["compton wavelength", "electron compton wavelength", "compton wavelength of the electron"], symbol: "λ_C", value: 2.42631023867e-12, unit: "m", exact: false },
    Constant { names: &["hartree energy", "hartree"], symbol: "E_h", value: 4.3597447222071e-18, unit: "J", exact: false },
    Constant { names: &["wien displacement constant", "wien's displacement constant", "wien constant"], symbol: "b", value: 2.897771955e-3, unit: "m K", exact: true },
    Constant { names: &["coulomb constant", "coulomb's constant", "electrostatic constant"], symbol: "k_e", value: 8.9875517923e9, unit: "N m^2/C^2", exact: false },
    Constant { names: &["impedance of free space", "characteristic impedance of vacuum", "vacuum impedance"], symbol: "Z₀", value: 376.730313668, unit: "ohm", exact: false },
    Constant { names: &["muon mass", "mass of a muon", "mass of muon"], symbol: "m_μ", value: 1.883531627e-28, unit: "kg", exact: false },
    Constant { names: &["electron g-factor", "electron g factor", "g-factor of the electron"], symbol: "g_e", value: -2.00231930436256, unit: "", exact: false },
    Constant { names: &["electron magnetic moment"], symbol: "μ_e", value: -9.2847647043e-24, unit: "J/T", exact: false },
    Constant { names: &["proton magnetic moment"], symbol: "μ_p", value: 1.41060679736e-26, unit: "J/T", exact: false },
    Constant { names: &["proton gyromagnetic ratio", "gyromagnetic ratio proton", "gyromagnetic ratio of the proton", "gyromagnetic ratio formula proton"], symbol: "γ_p", value: 2.6752218744e8, unit: "1/(s T)", exact: false },
    Constant { names: &["electron gyromagnetic ratio", "gyromagnetic ratio electron", "gyromagnetic ratio of the electron"], symbol: "γ_e", value: 1.76085963023e11, unit: "1/(s T)", exact: false },
    Constant { names: &["schwinger field strength", "schwinger electric field strength", "schwinger limit", "schwinger field", "critical electric field"], symbol: "E_S", value: 1.32329e18, unit: "V/m", exact: false },
    Constant { names: &["schwinger magnetic field", "critical magnetic field"], symbol: "B_S", value: 4.41401e9, unit: "T", exact: false },
    Constant { names: &["loschmidt constant", "loschmidt number"], symbol: "n₀", value: 2.686780111e25, unit: "1/m^3", exact: true },
    Constant { names: &["molar volume of an ideal gas", "molar volume of ideal gas", "molar volume at stp"], symbol: "V_m", value: 22.41396954e-3, unit: "m^3/mol", exact: true },
    Constant { names: &["proton charge radius", "proton radius", "radius of a proton"], symbol: "r_p", value: 8.414e-16, unit: "m", exact: false },
    Constant { names: &["solar luminosity", "luminosity of the sun", "sun luminosity"], symbol: "L_☉", value: 3.828e26, unit: "W", exact: true },
    Constant { names: &["hubble constant"], symbol: "H₀", value: 67.4, unit: "km/s/Mpc", exact: false },
    Constant { names: &["geocentric gravitational constant", "standard gravitational parameter of the earth", "gm of earth"], symbol: "GM_⊕", value: 3.986004418e14, unit: "m^3/s^2", exact: false },
    Constant { names: &["absolute zero"], symbol: "", value: 0.0, unit: "K", exact: true },
];

pub fn find_constant(q: &str) -> Option<&'static Constant> {
    let q = q.trim().trim_start_matches("the ").trim_start_matches("value of ").trim();
    CONSTANTS.iter().find(|c| c.names.iter().any(|n| *n == q && n.len() > 1))
}

// ---------------------------------------------------------------------------
// handler
// ---------------------------------------------------------------------------

fn fmt_val(v: f64) -> String {
    fmt_float(v, 8)
}

pub(crate) fn conversion_table(si: f64, dim: &Dim, exclude: &str) -> Option<Pod> {
    let rows: Vec<Vec<Content>> = related(dim)
        .iter()
        .filter_map(|u| {
            let unit = parse_unit(u)?;
            if unit.name == exclude {
                return None;
            }
            Some(vec![text(fmt_val(from_si(si, &unit))), text(unit.name.clone())])
        })
        .take(8)
        .collect();
    if rows.is_empty() {
        return None;
    }
    Some(pod("AdditionalConversions", "Additional conversions", table(None, rows)))
}

/// The other reading of an ambiguous unit name, when the first reading
/// has the wrong dimension: "ounces" of a volume are fluid ounces.
fn alt_unit(word: &str, want: &Dim) -> Option<Unit> {
    let w = word.trim().to_lowercase();
    let alt = match w.as_str() {
        "oz" | "ounce" | "ounces" => "fluid ounces",
        "dr" | "dram" | "drams" => "fluid drams",
        "lb" | "lbs" | "pound" | "pounds" => "pounds force",
        "fluid ounce" | "fluid ounces" | "fl oz" => "ounces",
        "c" => "coulombs",
        "pt" | "pint" | "pints" => "pints",
        _ => return None,
    };
    parse_unit(alt).filter(|u| u.dim == *want)
}

/// Number words after digits: "1 million miles" → "1000000 miles".
fn expand_scale_words(s: &str) -> String {
    static R: OnceLock<Regex> = OnceLock::new();
    let r = R.get_or_init(|| re(r"(?i)\b(\d+(?:\.\d+)?)\s+(thousand|million|billion|trillion)\b"));
    r.replace_all(s, |c: &regex::Captures| {
        let v: f64 = c[1].parse().unwrap_or(0.0);
        let m = match c[2].to_lowercase().as_str() {
            "thousand" => 1e3,
            "million" => 1e6,
            "billion" => 1e9,
            _ => 1e12,
        };
        fmt_float(v * m, 15).replace(',', "")
    })
    .to_string()
}

/// A product of quantities: "(20 feet)(1 mile)", "20 ft * 1 mi", "3 m x 4 m".
fn parse_product(s: &str) -> Option<(f64, Dim, String)> {
    static R: OnceLock<Regex> = OnceLock::new();
    let r = R.get_or_init(|| re(r"\)\s*\(|\s*[*×·]\s*|\s+(?:x|times|by)\s+"));
    let parts: Vec<&str> = r.split(s.trim()).map(|p| p.trim().trim_matches(|c| c == '(' || c == ')').trim()).collect();
    if parts.len() < 2 {
        return None;
    }
    let mut si = 1.0;
    let mut dim = NONE;
    let mut shown = Vec::new();
    for p in parts {
        let (v, u, sh) = parse_quantity(p)?;
        if u.offset != 0.0 {
            return None;
        }
        si *= to_si(v, &u);
        for k in 0..8 {
            dim[k] += u.dim[k];
        }
        shown.push(format!("({sh})"));
    }
    Some((si, dim, shown.join(" × ")))
}

/// The unit of a measurement system that suits a value.
fn system_unit(system: &str, dim: &Dim, si: f64) -> Option<Unit> {
    let cands: &[&str] = match (system, *dim) {
        ("imperial", L) => &["mi", "yd", "ft", "in"],
        ("imperial", M) => &["ton", "lb", "oz"],
        ("imperial", AREA) => &["mi^2", "acre", "ft^2", "in^2"],
        ("imperial", VOL) => &["gal", "qt", "pt", "floz"],
        ("imperial", SPEED) => &["mph", "ft/s"],
        ("imperial", ACC) => &["ft/s^2"],
        ("imperial", FORCE) => &["lbf"],
        ("imperial", ENERGY) => &["BTU", "ft lbf"],
        ("imperial", POWER) => &["hp"],
        ("imperial", PRESSURE) => &["psi"],
        ("imperial", TEMP) => &["°F"],
        ("metric", L) => &["km", "m", "cm", "mm"],
        ("metric", M) => &["t", "kg", "g", "mg"],
        ("metric", AREA) => &["km^2", "ha", "m^2", "cm^2"],
        ("metric", VOL) => &["m^3", "L", "mL"],
        ("metric", SPEED) => &["km/h"],
        ("metric", TEMP) => &["°C"],
        ("metric", ENERGY) => &["MJ", "kJ", "J"],
        ("metric", POWER) => &["MW", "kW", "W"],
        ("metric", PRESSURE) => &["MPa", "kPa", "Pa"],
        _ => return None,
    };
    let units: Vec<Unit> = cands.iter().filter_map(|c| parse_unit(c)).collect();
    units.iter().find(|u| u.offset != 0.0 || from_si(si, u).abs() >= 1.0).or(units.last()).cloned()
}

const PREFIX_TABLE: &[(&str, &str, &str)] = &[
    ("quetta", "Q", "10^30"), ("ronna", "R", "10^27"), ("yotta", "Y", "10^24"), ("zetta", "Z", "10^21"), ("exa", "E", "10^18"), ("peta", "P", "10^15"), ("tera", "T", "10^12"), ("giga", "G", "10^9"), ("mega", "M", "10^6"), ("kilo", "k", "10^3"), ("hecto", "h", "10^2"), ("deca", "da", "10^1"),
    ("deci", "d", "10^-1"), ("centi", "c", "10^-2"), ("milli", "m", "10^-3"), ("micro", "μ", "10^-6"), ("nano", "n", "10^-9"), ("pico", "p", "10^-12"), ("femto", "f", "10^-15"), ("atto", "a", "10^-18"), ("zepto", "z", "10^-21"), ("yocto", "y", "10^-24"), ("ronto", "r", "10^-27"), ("quecto", "q", "10^-30"),
];

fn prefix_tables(l: &str) -> Option<Answer> {
    let l = l.trim().trim_start_matches("the ").trim_start_matches("list of ").trim_start_matches("table of ");
    if matches!(l, "si prefixes" | "metric prefixes" | "si prefix" | "metric prefix" | "unit prefixes") {
        let rows = PREFIX_TABLE.iter().map(|(n, s, f)| vec![text(*n), text(*s), text(*f)]).collect();
        return Some(Answer::new("SI prefixes", vec![pod("Input", "Input interpretation", text("SI prefixes")), primary(pod("Result", "SI prefixes", table(Some(vec!["prefix", "symbol", "factor"]), rows))), pod("Note", "Note", text("ronna, quetta, ronto and quecto were adopted in 2022"))]));
    }
    if matches!(l, "binary prefixes" | "binary prefix" | "iec prefixes" | "iec binary prefixes") {
        let rows = [("kibi", "Ki", "2^10 = 1024"), ("mebi", "Mi", "2^20 = 1048576"), ("gibi", "Gi", "2^30 = 1073741824"), ("tebi", "Ti", "2^40"), ("pebi", "Pi", "2^50"), ("exbi", "Ei", "2^60"), ("zebi", "Zi", "2^70"), ("yobi", "Yi", "2^80")].iter().map(|(n, s, f)| vec![text(*n), text(*s), text(*f)]).collect();
        return Some(Answer::new("binary prefixes", vec![pod("Input", "Input interpretation", text("binary (IEC) prefixes")), primary(pod("Result", "Binary prefixes", table(Some(vec!["prefix", "symbol", "factor"]), rows)))]));
    }
    None
}

/// A unit by itself: what it measures and what it equals.
fn unit_info(clean: &str) -> Option<Answer> {
    let t = clean.trim().trim_end_matches(['?', '.']).trim();
    let tl = t.to_lowercase();
    let t = tl.strip_prefix("what is a ").or_else(|| tl.strip_prefix("what is an ")).or_else(|| tl.strip_prefix("what is ")).or_else(|| tl.strip_prefix("1 ")).map(|x| x.to_string()).unwrap_or_else(|| t.to_string());
    if t.chars().count() < 2 || t.chars().any(|c| c.is_ascii_digit() && !t.contains('^')) || t.split_whitespace().count() > 4 {
        return None;
    }
    // words that are units only in context
    if matches!(t.to_lowercase().as_str(), "a" | "in" | "s" | "t" | "h" | "d" | "pot" | "pots" | "point" | "points" | "hand" | "hands" | "drop" | "drops" | "chain" | "rod" | "pole" | "quad" | "rem" | "day" | "days" | "week" | "month" | "year" | "minute" | "second" | "hour" | "turn" | "da" | "pc" | "mil" | "gr" | "dr" | "pk" | "bu" | "lm" | "ct" | "st" | "cd" | "mach" | "percent" | "%" | "u" | "c" | "f" | "g" | "k" | "b" | "pt" | "kat" | "slug" | "slugs") {
        return None;
    }
    let u = parse_unit(&t)?;
    if u.offset != 0.0 {
        return None;
    }
    let si_unit = si_name(&u.dim);
    let quantity = quantity_name(&u.dim);
    let mut pods = vec![pod("Input", "Input interpretation", text(u.name.clone()))];
    let main = if u.factor == 1.0 && !si_unit.is_empty() { format!("SI unit: {si_unit}") } else { format!("1 {} = {} {}", singular(&u.name), fmt_val(u.factor), si_unit) };
    pods.push(primary(pod("Result", "Unit conversion", text(main))));
    if let Some(qn) = quantity {
        pods.push(pod("Quantity", "Measures", text(qn)));
    }
    let dimtext = dim_text(&u.dim);
    if !dimtext.is_empty() {
        pods.push(pod("Dimensions", "Dimensions", text(dimtext)));
    }
    if let Some(tb) = conversion_table(u.factor, &u.dim, &u.name) {
        pods.push(tb);
    }
    Some(Answer::new(u.name.clone(), pods))
}

fn dim_text(dim: &Dim) -> String {
    let names = ["length", "mass", "time", "electric current", "temperature", "amount", "luminous intensity", "information"];
    let mut parts = Vec::new();
    for k in 0..8 {
        match dim[k] {
            0 => {}
            1 => parts.push(format!("[{}]", names[k])),
            p => parts.push(format!("[{}]^{p}", names[k])),
        }
    }
    parts.join(" ")
}

fn quantity_name(dim: &Dim) -> Option<&'static str> {
    QUANTITIES.iter().find(|q| q.dim == *dim && q.primary).map(|q| q.names[0])
}

struct PhysQuantity {
    names: &'static [&'static str],
    symbol: &'static str,
    dim: Dim,
    unit: &'static str,
    definition: &'static str,
    primary: bool,
}

macro_rules! pq {
    ($names:expr, $sym:expr, $dim:expr, $unit:expr, $def:expr) => {
        PhysQuantity { names: $names, symbol: $sym, dim: $dim, unit: $unit, definition: $def, primary: true }
    };
    ($names:expr, $sym:expr, $dim:expr, $unit:expr, $def:expr, alias) => {
        PhysQuantity { names: $names, symbol: $sym, dim: $dim, unit: $unit, definition: $def, primary: false }
    };
}

static QUANTITIES: &[PhysQuantity] = &[
    pq!(&["length", "distance"], "l", L, "m (meter)", "SI base quantity"),
    pq!(&["mass"], "m", M, "kg (kilogram)", "SI base quantity"),
    pq!(&["time", "duration"], "t", T, "s (second)", "SI base quantity"),
    pq!(&["electric current", "current"], "I", d(0, 0, 0, 1, 0, 0, 0, 0), "A (ampere)", "SI base quantity"),
    pq!(&["temperature", "thermodynamic temperature"], "T", TEMP, "K (kelvin)", "SI base quantity"),
    pq!(&["amount of substance", "amount"], "n", d(0, 0, 0, 0, 0, 1, 0, 0), "mol (mole)", "SI base quantity"),
    pq!(&["luminous intensity"], "I_v", d(0, 0, 0, 0, 0, 0, 1, 0), "cd (candela)", "SI base quantity"),
    pq!(&["area"], "A", AREA, "m^2", "length × length"),
    pq!(&["volume"], "V", VOL, "m^3", "length³"),
    pq!(&["speed", "velocity"], "v", SPEED, "m/s", "distance/time"),
    pq!(&["acceleration"], "a", ACC, "m/s^2", "change of velocity/time"),
    pq!(&["force", "weight"], "F", FORCE, "N (newton) = kg m/s^2", "mass × acceleration"),
    pq!(&["energy", "work", "heat"], "E", ENERGY, "J (joule) = kg m^2/s^2", "force × distance"),
    pq!(&["torque", "moment of force"], "τ", ENERGY, "N m", "r × F", alias),
    pq!(&["enthalpy"], "H", ENERGY, "J (joule)", "H = U + p V", alias),
    pq!(&["power"], "P", POWER, "W (watt) = J/s", "energy/time"),
    pq!(&["pressure", "stress"], "p", PRESSURE, "Pa (pascal) = N/m^2", "force/area"),
    pq!(&["momentum", "linear momentum"], "p", d(1, 1, -1, 0, 0, 0, 0, 0), "kg m/s", "mass × velocity"),
    pq!(&["angular momentum"], "L", d(2, 1, -1, 0, 0, 0, 0, 0), "kg m^2/s", "r × p"),
    pq!(&["impulse"], "J", d(1, 1, -1, 0, 0, 0, 0, 0), "N s", "force × time", alias),
    pq!(&["mass density", "density"], "ρ", d(-3, 1, 0, 0, 0, 0, 0, 0), "kg/m^3", "mass/volume"),
    pq!(&["frequency"], "f", FREQ, "Hz (hertz) = 1/s", "cycles per unit time"),
    pq!(&["electric charge", "charge"], "Q", CHARGE, "C (coulomb) = A s", "current × time"),
    pq!(&["voltage", "electric potential", "electric potential difference", "electromotive force"], "V", VOLT, "V (volt) = W/A", "energy per unit charge"),
    pq!(&["electrical resistance", "resistance"], "R", OHM, "Ω (ohm) = V/A", "voltage/current"),
    pq!(&["capacitance"], "C", d(-2, -1, 4, 2, 0, 0, 0, 0), "F (farad) = C/V", "charge/voltage"),
    pq!(&["inductance"], "L", d(2, 1, -2, -2, 0, 0, 0, 0), "H (henry) = Wb/A", "magnetic flux/current"),
    pq!(&["magnetic flux"], "Φ", d(2, 1, -2, -1, 0, 0, 0, 0), "Wb (weber) = V s", "∫ B · dA"),
    pq!(&["magnetic induction", "magnetic flux density", "magnetic field"], "B", d(0, 1, -2, -1, 0, 0, 0, 0), "T (tesla) = Wb/m^2", "force on a moving charge: F = q v × B"),
    pq!(&["magnetic field strength", "magnetizing field", "h-field", "magnetic field intensity"], "H", d(-1, 0, 0, 1, 0, 0, 0, 0), "A/m", "H = B/μ₀ − M"),
    pq!(&["electric field strength", "electric field"], "E", d(1, 1, -3, -1, 0, 0, 0, 0), "V/m", "force per unit charge"),
    pq!(&["electrical conductance", "conductance"], "G", d(-2, -1, 3, 2, 0, 0, 0, 0), "S (siemens) = 1/Ω", "1/resistance"),
    pq!(&["electrical resistivity", "resistivity"], "ρ", d(3, 1, -3, -2, 0, 0, 0, 0), "Ω m", "R A/l", alias),
    pq!(&["electrical conductivity", "conductivity"], "σ", d(-3, -1, 3, 2, 0, 0, 0, 0), "S/m", "1/resistivity"),
    pq!(&["reluctance", "magnetic reluctance"], "ℛ", d(-2, -1, 2, 2, 0, 0, 0, 0), "1/H (per henry)", "magnetomotive force/magnetic flux"),
    pq!(&["reluctivity"], "ν", d(-1, -1, 2, 2, 0, 0, 0, 0), "m/H", "1/permeability"),
    pq!(&["permeability", "magnetic permeability"], "μ", d(1, 1, -2, -2, 0, 0, 0, 0), "H/m", "B/H"),
    pq!(&["permittivity"], "ε", d(-3, -1, 4, 2, 0, 0, 0, 0), "F/m", "D/E"),
    pq!(&["entropy", "heat capacity"], "S", d(2, 1, -2, 0, -1, 0, 0, 0), "J/K", "dQ_rev/T"),
    pq!(&["specific heat capacity", "specific heat"], "c", d(2, 0, -2, 0, -1, 0, 0, 0), "J/(kg K)", "heat capacity per unit mass"),
    pq!(&["thermal conductivity"], "k", d(1, 1, -3, 0, -1, 0, 0, 0), "W/(m K)", "heat flux/temperature gradient"),
    pq!(&["seebeck coefficient", "thermopower", "thermoelectric power"], "S", d(2, 1, -3, -1, -1, 0, 0, 0), "V/K", "−ΔV/ΔT (thermoelectric voltage per kelvin)"),
    pq!(&["dynamic viscosity", "viscosity"], "μ", d(-1, 1, -1, 0, 0, 0, 0, 0), "Pa s", "shear stress/shear rate"),
    pq!(&["kinematic viscosity"], "ν", d(2, 0, -1, 0, 0, 0, 0, 0), "m^2/s", "μ/ρ"),
    pq!(&["surface tension"], "γ", d(0, 1, -2, 0, 0, 0, 0, 0), "N/m", "force per unit length"),
    pq!(&["angular velocity", "angular frequency"], "ω", FREQ, "rad/s", "dθ/dt", alias),
    pq!(&["moment of inertia"], "I", d(2, 1, 0, 0, 0, 0, 0, 0), "kg m^2", "Σ m r²"),
    pq!(&["molar mass"], "M", d(0, 1, 0, 0, 0, -1, 0, 0), "kg/mol", "mass/amount"),
    pq!(&["molar concentration", "molarity", "concentration"], "c", d(-3, 0, 0, 0, 0, 1, 0, 0), "mol/m^3", "amount/volume"),
    pq!(&["radioactivity", "activity"], "A", FREQ, "Bq (becquerel) = 1/s", "decays per unit time", alias),
    pq!(&["absorbed dose"], "D", d(2, 0, -2, 0, 0, 0, 0, 0), "Gy (gray) = J/kg", "energy absorbed per unit mass"),
    pq!(&["luminous flux"], "Φ_v", d(0, 0, 0, 0, 0, 0, 1, 0), "lm (lumen) = cd sr", "luminous intensity × solid angle", alias),
    pq!(&["illuminance"], "E_v", d(-2, 0, 0, 0, 0, 0, 1, 0), "lx (lux) = lm/m^2", "luminous flux/area"),
    pq!(&["wavenumber"], "k", d(-1, 0, 0, 0, 0, 0, 0, 0), "1/m", "2π/wavelength"),
    pq!(&["sound pressure level", "logarithmic sound pressure level", "sound level"], "L_p", NONE, "dB (decibel)", "L_p = 20 log₁₀(p_rms/p₀), p₀ = 20 μPa"),
    pq!(&["retroreflection coefficient", "coefficient of retroreflection"], "R'", d(-2, 0, 0, 0, 0, 0, 0, 0), "cd/(lx m^2)", "luminous intensity per illuminance per unit area"),
    pq!(&["eckert number"], "Ec", NONE, "dimensionless", "Ec = v²/(c_p ΔT): kinetic energy relative to enthalpy difference"),
    pq!(&["reynolds number"], "Re", NONE, "dimensionless", "Re = ρ v L/μ: inertial relative to viscous forces"),
    pq!(&["prandtl number"], "Pr", NONE, "dimensionless", "Pr = c_p μ/k: momentum relative to thermal diffusivity"),
    pq!(&["mach number"], "Ma", NONE, "dimensionless", "Ma = v/c: flow speed relative to the speed of sound"),
    pq!(&["nusselt number"], "Nu", NONE, "dimensionless", "Nu = h L/k: convective relative to conductive heat transfer"),
    pq!(&["froude number"], "Fr", NONE, "dimensionless", "Fr = v/√(g L)"),
    pq!(&["brinkman number"], "Br", NONE, "dimensionless", "Br = μ v²/(k ΔT): viscous heating relative to conduction"),
    pq!(&["strain"], "ε", NONE, "dimensionless", "ΔL/L"),
    pq!(&["refractive index", "index of refraction"], "n", NONE, "dimensionless", "c/v"),
];

fn quantities(l: &str) -> Option<Answer> {
    let t = l.trim().trim_end_matches(['?', '.']).trim_start_matches("what is ").trim_start_matches("the ").trim_start_matches("physical quantity ").trim();
    let parts: Vec<&str> = t.split([',', '/']).flat_map(|p| p.split(" and ")).flat_map(|p| p.split(" vs ")).map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
    let found: Vec<&PhysQuantity> = parts.iter().map(|p| QUANTITIES.iter().find(|q| q.names.contains(p))).collect::<Option<_>>()?;
    if found.len() == 1 {
        let q = found[0];
        let rows = vec![vec![text("symbol"), text(q.symbol)], vec![text("SI unit"), text(q.unit)], vec![text("dimensions"), text(if q.dim == NONE { "dimensionless".to_string() } else { dim_text(&q.dim) })], vec![text("definition"), text(q.definition)]];
        return Some(Answer::new(q.names[0], vec![pod("Input", "Input interpretation", text(format!("{} (physical quantity)", q.names[0]))), primary(pod("Result", "SI unit", text(q.unit))), pod("Properties", "Properties", table(None, rows))]));
    }
    let rows = found.iter().map(|q| vec![text(q.names[0]), text(q.symbol), text(q.unit), text(if q.dim == NONE { "dimensionless".to_string() } else { dim_text(&q.dim) })]).collect();
    Some(Answer::new("physical quantities", vec![pod("Input", "Input interpretation", text(found.iter().map(|q| q.names[0]).collect::<Vec<_>>().join(" | "))), primary(pod("Result", "Physical quantities", table(Some(vec!["quantity", "symbol", "SI unit", "dimensions"]), rows)))]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let expanded = expand_scale_words(&q.clean);
    handle_str(&expanded)
}

fn handle_str(clean: &str) -> Option<Answer> {
    let lower = clean.to_lowercase();
    let l = lower.as_str();
    if let Some(a) = prefix_tables(l).or_else(|| quantities(l)) {
        return Some(a);
    }
    static CONV: OnceLock<Regex> = OnceLock::new();
    let conv = CONV.get_or_init(|| re(r"^(?:convert\s+)?(.+?)\s+(?:in|to|into|as|in terms of|=\s*\?|->|→)\s+(.+?)$"));
    // constants
    let const_q = l.trim_start_matches("what is ").trim_start_matches("value of ");
    if let Some(m) = conv.captures(const_q) {
        if let (Some(c), Some(u)) = (find_constant(&m[1]), parse_unit(&clean[clean.len() - m[2].len()..])) {
            let cu = parse_unit(c.unit)?;
            if cu.dim == u.dim {
                let v = from_si(to_si(c.value, &cu), &u);
                let pods = vec![pod("Input", "Input interpretation", text(format!("{} in {}", c.names[0], u.name))), primary(pod("Result", "Result", text(format!("{} {}", fmt_val(v), u.name))))];
                return Some(Answer::new(format!("{} in {}", c.names[0], u.name), pods));
            }
        }
    }
    if let Some(c) = find_constant(const_q) {
        if c.names[0] == "c" || const_q.len() <= 1 {
            return None;
        }
        let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} {}", c.names[0], if c.symbol.is_empty() { String::new() } else { format!("({})", c.symbol) })))];
        let unit = if c.unit.is_empty() { String::new() } else { format!(" {}", c.unit) };
        pods.push(primary(pod("Value", "Value", text(format!("{}{}{}", fmt_float(c.value, 12), unit, if c.exact { " (exact)" } else { "" })))));
        if !c.unit.is_empty() {
            if let Some(cu) = parse_unit(c.unit) {
                if let Some(t) = conversion_table(to_si(c.value, &cu), &cu.dim, &cu.name) {
                    pods.push(t);
                }
            }
        }
        return Some(Answer::new(c.names[0], pods));
    }
    // "feet in a mile", "seconds in a day"
    static HOW: OnceLock<Regex> = OnceLock::new();
    let how = HOW.get_or_init(|| re(r"^(?:how many\s+)?([a-z°/ ]+?)\s+(?:in|per)\s+(?:a|an|one|1)\s+([a-z°/ ]+)$"));
    if let Some(m) = how.captures(l) {
        if let (Some(mut to), Some(from)) = (parse_unit(&m[1]), parse_unit(&m[2])) {
            if to.dim != from.dim {
                if let Some(a) = alt_unit(&m[1], &from.dim) {
                    to = a;
                }
            }
            if to.dim == from.dim && to.offset == 0.0 {
                let r = from.factor / to.factor;
                let pods = vec![pod("Input", "Input interpretation", text(format!("{} per {}", to.name, singular(&from.name)))), primary(pod("Result", "Result", text(format!("{} {}", fmt_val(r), to.name))))];
                return Some(Answer::new(format!("{} in 1 {}", to.name, singular(&from.name)), pods));
            }
        }
    }
    // to a measurement system: "convert 120 km/h to imperial"
    static SYS: OnceLock<Regex> = OnceLock::new();
    let sys = SYS.get_or_init(|| re(r"^(?:convert\s+)?(.+?)\s+(?:to|in|into)\s+(imperial|metric|si|si units|us customary|us customary units|us units|english units|imperial units|metric units)$"));
    if let Some(m) = sys.captures(l) {
        let g = m.get(1)?;
        let src = clean.get(g.start()..g.end()).unwrap_or(&m[1]).trim();
        let (si, dim, shown, offset_unit) = match parse_quantity(src) {
            Some((v, u, sh)) => (to_si(v, &u), u.dim, sh, u.offset != 0.0),
            None => {
                let (si, dim, sh) = parse_product(src)?;
                (si, dim, sh, false)
            }
        };
        let _ = offset_unit;
        let system = if m[2].starts_with("metric") || m[2].starts_with("si") { if m[2].starts_with("si") { "si" } else { "metric" } } else { "imperial" };
        let (val, name) = if system == "si" {
            (si, si_name(&dim))
        } else {
            let u = system_unit(system, &dim, si)?;
            (from_si(si, &u), u.name.clone())
        };
        let pods = vec![pod("Input", "Input interpretation", text(format!("convert {shown} to {} units", if system == "si" { "SI" } else { system }))), primary(pod("Result", "Result", text(format!("{} {name}", fmt_val(val)))))];
        let mut pods = pods;
        if let Some(t) = conversion_table(si, &dim, &name) {
            pods.push(t);
        }
        return Some(Answer::new(format!("{shown} in {system} units"), pods));
    }
    // comparisons: "175lb vs 100kg"
    static VS: OnceLock<Regex> = OnceLock::new();
    let vs = VS.get_or_init(|| re(r"^(.+?)\s+(?:vs\.?|versus|compared to|compared with)\s+(.+)$"));
    if let Some(m) = vs.captures(clean) {
        if let (Some((v1, u1, s1)), Some((v2, u2, s2))) = (parse_quantity(&m[1]), parse_quantity(&m[2])) {
            if u1.dim == u2.dim && u1.offset == 0.0 && u2.offset == 0.0 {
                let (a, b) = (to_si(v1, &u1), to_si(v2, &u2));
                let ratio = a / b;
                let main = if (ratio - 1.0).abs() < 1e-12 { "equal".to_string() } else if ratio > 1.0 { format!("{s1} is {}× {s2} ({} {} larger)", fmt_float(ratio, 6), fmt_val(from_si(a - b, &u2)), u2.name) } else { format!("{s2} is {}× {s1} ({} {} larger)", fmt_float(1.0 / ratio, 6), fmt_val(from_si(b - a, &u1)), u1.name) };
                let rows = vec![vec![text(s1.clone()), text(format!("{} {}", fmt_val(from_si(a, &u2)), u2.name)), text(format!("{} {}", fmt_val(a), si_name(&u1.dim)))], vec![text(s2.clone()), text(format!("{} {}", fmt_val(from_si(b, &u1)), u1.name)), text(format!("{} {}", fmt_val(b), si_name(&u2.dim)))]];
                return Some(Answer::new(format!("{s1} vs {s2}"), vec![pod("Input", "Input interpretation", text(format!("{s1} vs {s2}"))), primary(pod("Result", "Comparison", text(main))), pod("Values", "Values", table(Some(vec!["quantity", "in the other unit", "SI"]), rows))]));
            }
        }
    }
    // conversions: try every "in/to" split point
    static SEP: OnceLock<Regex> = OnceLock::new();
    let sep = SEP.get_or_init(|| re(r"\s+(?:in|to|into|as|in terms of|->|→)\s+"));
    let mut split: Option<(String, String)> = None;
    let _ = sep;
    let words: Vec<(usize, &str)> = clean.split_whitespace().map(|w| (w.as_ptr() as usize - clean.as_ptr() as usize, w)).collect();
    for (start, w) in &words {
        if !matches!(w.to_lowercase().as_str(), "in" | "to" | "into" | "as" | "->" | "→") {
            continue;
        }
        let (a, b) = (&clean[..*start], &clean[start + w.len()..]);
        let (a, b) = (a.trim(), b.trim());
        let a = a.trim_start_matches("convert ").trim_start_matches("Convert ");
        if parse_quantity(a).is_some() && parse_unit(b).is_some() {
            split = Some((a.to_string(), b.to_string()));
            break;
        }
        if let (Some((si, dim, shown)), Some(to)) = (parse_product(a), parse_unit(b)) {
            if dim == to.dim && to.offset == 0.0 {
                let r = from_si(si, &to);
                let mut pods = vec![pod("Input", "Input interpretation", text(format!("{shown} in {}", to.name))), primary(pod("Result", "Result", text(format!("{} {}", fmt_val(r), to.name))))];
                if let Some(t) = conversion_table(si, &dim, &to.name) {
                    pods.push(t);
                }
                return Some(Answer::new(format!("{shown} in {}", to.name), pods));
            }
        }
    }
    let _ = conv;
    if let Some((qs, us)) = split {
        let (qs, us) = (qs.as_str(), us.as_str());
        if let (Some((v, mut from, shown)), Some(mut to)) = (parse_quantity(qs), parse_unit(us)) {
            let unit_word = qs.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == '-' || c == ' ' || c == ',');
            if from.dim != to.dim {
                let _ = qs.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == '-' || c == ' ' || c == ',');
                if let (Some(a), Some(b)) = (temperature_alt(unit_word).or(Some(from.clone())), temperature_alt(us).or(Some(to.clone()))) {
                    if a.dim == b.dim {
                        from = a;
                        to = b;
                    }
                }
            }
            if from.dim != to.dim {
                if let Some(a) = alt_unit(us, &from.dim) {
                    to = a;
                } else if let Some(a) = alt_unit(unit_word, &to.dim) {
                    from = a;
                }
            }
            if from.dim != to.dim {
                return Some(Answer::new(
                    format!("convert {shown} to {}", to.name),
                    vec![pod("Input", "Input interpretation", text(format!("convert {shown} to {}", to.name))), primary(pod("Result", "Result", text(format!("Incompatible units: {} and {} measure different quantities.", from.name, to.name))))],
                ));
            }
            let si = to_si(v, &from);
            let mut r = from_si(si, &to);
            if (from.offset != 0.0 || to.offset != 0.0) && r.abs() < 1e-9 * (1.0 + v.abs() + from.offset.abs() + to.offset.abs()) {
                r = 0.0;
            }
            let mut pods = vec![pod("Input", "Input interpretation", text(format!("convert {shown} ({}) to {}", from.name, to.name)))];
            pods.push(primary(pod("Result", "Result", text(format!("{} {}", fmt_val(r), to.name)))));
            if let Some(t) = conversion_table(si, &to.dim, &to.name) {
                pods.push(t);
            }
            if from.offset == 0.0 && to.offset == 0.0 && from.name != to.name {
                pods.push(pod("Factor", "Conversion factor", text(format!("1 {} = {} {}", singular(&from.name), fmt_val(from.factor / to.factor), to.name))));
            }
            return Some(Answer::new(format!("convert {shown} to {}", to.name), pods));
        }
    }
    // a bare quantity: "60 mph", "5 ft 3 in", "100 °F"
    if let Some((v, u, shown)) = parse_quantity(&clean) {
        if u.name.is_empty() || clean.split_whitespace().count() > 6 {
            return None;
        }
        // avoid stealing plain math like "2 x" or "3 a"
        let unit_part = clean.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ',' || c == ' ' || c == '-');
        if unit_part.len() <= 1 && !matches!(unit_part, "m" | "g" | "s" | "K" | "J" | "W" | "N" | "V" | "A" | "L" | "B" | "h") {
            return None;
        }
        let si = to_si(v, &u);
        let mut pods = vec![pod("Input", "Input interpretation", text(format!("{shown} ({})", u.name)))];
        let si_unit = if u.dim == FREQ && (u.name.contains("curie") || u.name.contains("becquerel")) { "Bq".to_string() } else { si_name(&u.dim) };
        // a readable primary: the first related unit giving a value from 1 to 1000
        let readable = related(&u.dim).iter().filter_map(|n| lookup_word(n).map(|x| (*n, x))).filter(|(_, x)| x.name != u.name).map(|(n, x)| (n, from_si(si, &x))).find(|(_, val)| val.abs() >= 1.0 && val.abs() < 1000.0);
        match readable {
            Some((n, val)) if u.offset == 0.0 => {
                pods.push(primary(pod("Result", "Result", text(format!("{} {n}", fmt_val(val))))));
                pods.push(pod("SI", "Value in SI units", text(format!("{} {}", fmt_val(si), si_unit))));
            }
            _ => pods.push(primary(pod("Result", "Value in SI units", text(format!("{} {}", fmt_val(si), si_unit))))),
        }
        if let Some(t) = conversion_table(si, &u.dim, &u.name) {
            pods.push(t);
        }
        return Some(Answer::new(format!("{shown}"), pods));
    }
    if let Some((si, dim, shown)) = parse_product(clean) {
        let mut pods = vec![pod("Input", "Input interpretation", text(shown.clone())), primary(pod("Result", "Result", text(format!("{} {}", fmt_val(si), si_name(&dim)))))];
        if let Some(t) = conversion_table(si, &dim, "") {
            pods.push(t);
        }
        return Some(Answer::new(shown, pods));
    }
    unit_info(clean)
}
