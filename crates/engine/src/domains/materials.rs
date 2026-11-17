//! Engineering materials: typical room-temperature properties of common
//! metals and alloys and of clear wood (USDA Wood Handbook, 12 % moisture),
//! comparisons between them, and the Mohs hardness scale.

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

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// A metal or alloy; NaN marks an unknown value.
struct Metal {
    names: &'static [&'static str],
    /// g/cm³
    density: f64,
    /// W/(m K)
    conductivity: f64,
    /// J/(kg K)
    heat: f64,
    /// °C
    melting: f64,
    /// GPa
    modulus: f64,
    /// ultimate tensile strength, MPa
    tensile: f64,
    /// yield strength, MPa
    yield_: f64,
    /// Ω m
    resistivity: f64,
    /// 10⁻⁶/K
    expansion: f64,
    mohs: f64,
    note: &'static str,
}

const X: f64 = f64::NAN;

#[allow(clippy::too_many_arguments)]
const fn m(names: &'static [&'static str], density: f64, conductivity: f64, heat: f64, melting: f64, modulus: f64, tensile: f64, yield_: f64, resistivity: f64, expansion: f64, mohs: f64, note: &'static str) -> Metal {
    Metal { names, density, conductivity, heat, melting, modulus, tensile, yield_, resistivity, expansion, mohs, note }
}

static METALS: &[Metal] = &[
    m(&["copper"], 8.96, 401.0, 385.0, 1084.6, 117.0, 210.0, 33.0, 1.68e-8, 16.5, 3.0, "annealed; cold-worked copper reaches about 340 MPa"),
    m(&["aluminum", "aluminium"], 2.70, 237.0, 897.0, 660.3, 69.0, 90.0, 35.0, 2.65e-8, 23.1, 2.75, "commercially pure, annealed"),
    m(&["aluminum 6061-t6", "6061 aluminum", "aluminum 6061", "6061-t6", "6061"], 2.70, 167.0, 896.0, 617.0, 68.9, 310.0, 276.0, 3.99e-8, 23.6, X, "the most common structural aluminium alloy"),
    m(&["iron"], 7.874, 80.4, 449.0, 1538.0, 211.0, 350.0, 130.0, 9.71e-8, 11.8, 4.0, "pure iron, annealed"),
    m(&["carbon steel", "mild steel", "steel", "1020 steel", "aisi 1020", "low carbon steel"], 7.87, 51.9, 486.0, 1515.0, 200.0, 420.0, 350.0, 1.59e-7, 11.7, X, "AISI 1020, cold drawn"),
    m(&["stainless steel", "304 stainless steel", "stainless steel 304", "304 stainless", "18-8 stainless steel"], 8.00, 16.2, 500.0, 1425.0, 193.0, 505.0, 215.0, 7.2e-7, 17.3, X, "AISI 304, annealed"),
    m(&["cast iron", "gray cast iron", "grey cast iron"], 7.15, 52.0, 460.0, 1175.0, 110.0, 200.0, X, 1.0e-6, 10.8, X, "gray iron, ASTM class 30"),
    m(&["brass", "cartridge brass", "c260 brass"], 8.53, 120.0, 375.0, 930.0, 110.0, 315.0, 105.0, 6.2e-8, 20.0, 3.0, "C260 cartridge brass (70/30), annealed"),
    m(&["bronze", "phosphor bronze"], 8.86, 69.0, 380.0, 1000.0, 110.0, 340.0, 140.0, 1.15e-7, 17.8, 3.0, "C510 phosphor bronze, annealed"),
    m(&["titanium"], 4.51, 21.9, 523.0, 1668.0, 116.0, 345.0, 275.0, 4.2e-7, 8.6, 6.0, "commercially pure grade 2"),
    m(&["ti-6al-4v", "titanium alloy", "grade 5 titanium", "ti 6al 4v"], 4.43, 6.7, 526.0, 1630.0, 113.8, 950.0, 880.0, 1.78e-6, 8.6, X, "annealed"),
    m(&["gold"], 19.32, 318.0, 129.0, 1064.2, 79.0, 120.0, X, 2.44e-8, 14.2, 2.5, "annealed"),
    m(&["silver"], 10.49, 429.0, 235.0, 961.8, 83.0, 140.0, X, 1.59e-8, 18.9, 2.5, "annealed"),
    m(&["lead"], 11.34, 35.3, 129.0, 327.5, 16.0, 17.0, X, 2.08e-7, 28.9, 1.5, ""),
    m(&["nickel"], 8.908, 90.9, 444.0, 1455.0, 200.0, 345.0, 148.0, 6.99e-8, 13.4, 4.0, "annealed"),
    m(&["zinc"], 7.14, 116.0, 388.0, 419.5, 108.0, 37.0, X, 5.9e-8, 30.2, 2.5, "cast"),
    m(&["tungsten"], 19.25, 173.0, 132.0, 3422.0, 411.0, 980.0, 750.0, 5.28e-8, 4.5, 7.5, ""),
    m(&["magnesium"], 1.738, 156.0, 1023.0, 650.0, 45.0, 90.0, 21.0, 4.39e-8, 24.8, 2.5, "commercially pure, cast"),
    m(&["tin"], 7.31, 66.8, 228.0, 231.9, 50.0, 15.0, X, 1.15e-7, 22.0, 1.5, ""),
    m(&["platinum"], 21.45, 71.6, 133.0, 1768.3, 168.0, 165.0, X, 1.06e-7, 8.8, 3.5, "annealed"),
    m(&["chromium"], 7.19, 93.9, 449.0, 1907.0, 279.0, 282.0, X, 1.25e-7, 4.9, 8.5, ""),
    m(&["invar"], 8.05, 10.15, 515.0, 1427.0, 141.0, 517.0, 276.0, 8.2e-7, 1.2, X, "Fe–36 % Ni; nearly zero thermal expansion"),
];

/// Clear wood at 12 % moisture content.
struct Wood {
    names: &'static [&'static str],
    /// g/cm³
    density: f64,
    /// modulus of elasticity, GPa
    modulus: f64,
    /// modulus of rupture (bending strength), MPa
    rupture: f64,
    /// compression parallel to the grain, MPa
    compression: f64,
    /// tension parallel to the grain, MPa (approximate)
    tension: f64,
    /// Janka side hardness, lbf
    janka: f64,
}

const fn w(names: &'static [&'static str], density: f64, modulus: f64, rupture: f64, compression: f64, tension: f64, janka: f64) -> Wood {
    Wood { names, density, modulus, rupture, compression, tension, janka }
}

static WOODS: &[Wood] = &[
    w(&["white pine", "eastern white pine", "pine"], 0.40, 8.5, 59.0, 33.1, 73.0, 380.0),
    w(&["sugar maple", "hard maple", "maple"], 0.71, 12.6, 109.0, 54.0, 110.0, 1450.0),
    w(&["red maple", "soft maple"], 0.61, 11.3, 92.0, 45.1, 100.0, 950.0),
    w(&["american white oak", "white oak", "oak"], 0.75, 12.3, 105.0, 51.3, 105.0, 1360.0),
    w(&["northern red oak", "red oak"], 0.70, 12.5, 99.0, 46.6, 100.0, 1290.0),
    w(&["sitka spruce", "spruce"], 0.43, 10.8, 70.0, 38.7, 95.0, 510.0),
    w(&["mahogany", "honduran mahogany", "genuine mahogany"], 0.59, 10.1, 80.0, 46.6, 90.0, 800.0),
    w(&["black walnut", "walnut"], 0.61, 11.6, 101.0, 52.3, 100.0, 1010.0),
    w(&["douglas fir", "douglas-fir"], 0.53, 13.4, 85.0, 49.9, 105.0, 710.0),
    w(&["balsa"], 0.16, 3.7, 21.6, 14.9, 20.0, 70.0),
    w(&["black cherry", "cherry"], 0.56, 10.3, 85.0, 49.0, 90.0, 950.0),
    w(&["white ash", "ash"], 0.67, 12.0, 103.0, 51.1, 105.0, 1320.0),
    w(&["yellow birch", "birch"], 0.69, 13.9, 114.0, 56.3, 115.0, 1260.0),
    w(&["teak"], 0.66, 12.3, 97.0, 54.0, 100.0, 1070.0),
    w(&["western red cedar", "red cedar", "cedar"], 0.37, 7.7, 51.7, 31.4, 60.0, 350.0),
    w(&["shagbark hickory", "hickory"], 0.80, 14.9, 139.0, 63.5, 125.0, 1880.0),
    w(&["yellow poplar", "tulip poplar", "poplar"], 0.47, 10.9, 70.0, 38.2, 80.0, 540.0),
    w(&["american beech", "beech"], 0.72, 11.9, 103.0, 50.3, 100.0, 1300.0),
    w(&["american elm", "elm"], 0.56, 9.2, 81.0, 38.1, 85.0, 830.0),
    w(&["redwood", "coast redwood"], 0.45, 9.2, 69.0, 42.4, 75.0, 450.0),
];

fn metal(s: &str) -> Option<&'static Metal> {
    let s = s.trim().trim_start_matches("the ").trim_end_matches(" metal").trim();
    METALS.iter().find(|x| x.names.iter().any(|n| *n == s))
}

fn wood(s: &str) -> Option<&'static Wood> {
    let s = s.trim().trim_start_matches("the ").trim_end_matches(" wood").trim_end_matches(" lumber").trim_end_matches(" tree").trim();
    WOODS.iter().find(|x| x.names.iter().any(|n| *n == s))
}

fn metal_rows(x: &Metal) -> Vec<(&'static str, String)> {
    let mut r = Vec::new();
    let mut add = |k: &'static str, v: f64, f: &dyn Fn(f64) -> String| {
        if v.is_finite() {
            r.push((k, f(v)));
        }
    };
    add("density", x.density, &|v| format!("{} g/cm³", n(v)));
    add("thermal conductivity", x.conductivity, &|v| format!("{} W/(m K)", n(v)));
    add("specific heat", x.heat, &|v| format!("{} J/(kg K)", n(v)));
    add("melting point", x.melting, &|v| format!("{} °C", n(v)));
    add("Young's modulus", x.modulus, &|v| format!("{} GPa", n(v)));
    add("tensile strength", x.tensile, &|v| format!("{} MPa", n(v)));
    add("yield strength", x.yield_, &|v| format!("{} MPa", n(v)));
    add("electrical resistivity", x.resistivity, &|v| format!("{} Ω m", fmt_float(v, 3)));
    add("thermal expansion", x.expansion, &|v| format!("{} × 10⁻⁶ /K", n(v)));
    add("Mohs hardness", x.mohs, &|v| n(v));
    if !x.note.is_empty() {
        r.push(("condition", x.note.to_string()));
    }
    r
}

fn wood_rows(x: &Wood) -> Vec<(&'static str, String)> {
    vec![
        ("density", format!("{} g/cm³ ({} lb/ft³)", n(x.density), n(x.density * 62.428))),
        ("Young's modulus", format!("{} GPa", n(x.modulus))),
        ("bending strength (modulus of rupture)", format!("{} MPa", n(x.rupture))),
        ("compressive strength", format!("{} MPa (parallel to grain)", n(x.compression))),
        ("tensile strength", format!("about {} MPa (parallel to grain)", n(x.tension))),
        ("Janka hardness", format!("{} lbf ({} kN)", n(x.janka), n(x.janka * 4.448_22 / 1000.0))),
        ("thermal conductivity", format!("about {} W/(m K) (across the grain)", n(0.04 + 0.17 * x.density))),
    ]
}

/// The property key a query names, matched against row labels.
fn prop_key(p: &str) -> Option<&'static str> {
    Some(match p.trim() {
        "density" | "weight" | "mass density" => "density",
        "thermal conductivity" | "heat conductivity" | "conductivity" => "thermal conductivity",
        "specific heat" | "specific heat capacity" | "heat capacity" => "specific heat",
        "melting point" | "melting temperature" => "melting point",
        "young's modulus" | "youngs modulus" | "elastic modulus" | "modulus of elasticity" | "stiffness" => "Young's modulus",
        "tensile strength" | "ultimate tensile strength" | "strength" => "tensile strength",
        "yield strength" | "yield stress" => "yield strength",
        "resistivity" | "electrical resistivity" => "electrical resistivity",
        "thermal expansion" | "coefficient of thermal expansion" | "thermal expansion coefficient" => "thermal expansion",
        "hardness" | "mohs hardness" => "hardness",
        "janka hardness" => "Janka hardness",
        "compressive strength" | "compression strength" => "compressive strength",
        "bending strength" | "modulus of rupture" | "flexural strength" => "bending strength (modulus of rupture)",
        _ => return None,
    })
}

enum Mat {
    M(&'static Metal),
    W(&'static Wood),
}

fn material(s: &str) -> Option<(String, Mat)> {
    let s = s.trim();
    if let Some(x) = metal(s) {
        return Some((x.names[0].to_string(), Mat::M(x)));
    }
    wood(s).map(|x| (x.names[0].to_string(), Mat::W(x)))
}

fn mat_rows(x: &Mat) -> Vec<(&'static str, String)> {
    match x {
        Mat::M(m) => metal_rows(m),
        Mat::W(w) => wood_rows(w),
    }
}

fn split_list(s: &str) -> Vec<String> {
    let s = s.trim();
    // "mahogany, walnut wood": the trailing "wood" applies to every item
    let suffix = if s.ends_with(" wood") { " wood" } else { "" };
    re!(r"\s*(?:,\s*(?:and\s+)?|\s+and\s+|\s+vs\.?\s+|\s+versus\s+)\s*").split(s.trim_end_matches(" wood")).map(|p| format!("{}{suffix}", p.trim())).filter(|p| !p.trim().is_empty()).collect()
}

fn materials(l: &str) -> Option<Answer> {
    // "compare thermal conductivity of copper, aluminum, iron"
    if let Some(c) = re!(r"^(?:compare\s+)?(?:the\s+)?([a-z' ]+?)\s+(?:of|for)\s+(.+)$").captures(l) {
        if let Some(key) = prop_key(&c[1]) {
            let items: Option<Vec<(String, Mat)>> = split_list(&c[2]).iter().map(|s| material(s)).collect();
            let items = items?;
            if items.is_empty() {
                return None;
            }
            let key = if key == "hardness" { if items.iter().all(|i| matches!(i.1, Mat::W(_))) { "Janka hardness" } else { "Mohs hardness" } } else { key };
            let vals: Vec<(String, String)> = items.iter().map(|(name, x)| (cap(name), mat_rows(x).into_iter().find(|(k, _)| *k == key).map(|(_, v)| v).unwrap_or_else(|| "(not available)".into()))).collect();
            if vals.iter().all(|v| v.1.starts_with("(not")) {
                return None;
            }
            let main = if vals.len() == 1 { vals[0].1.clone() } else { vals.iter().map(|(a, b)| format!("{a}: {b}")).collect::<Vec<_>>().join(" | ") };
            let mut pods = vec![input(format!("{key} | {}", vals.iter().map(|v| v.0.as_str()).collect::<Vec<_>>().join(", "))), primary(pod("Result", &cap(key), text(main)))];
            if vals.len() > 1 {
                pods.push(pod("Table", "Comparison", table(Some(vec!["material", key]), vals.iter().map(|(a, b)| vec![text(a.clone()), text(b.clone())]).collect())));
            } else {
                pods.push(pod("Properties", "Other properties", table(None, mat_rows(&items[0].1).into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())));
            }
            pods.push(pod("Note", "Note", text("typical room-temperature values; real values depend on composition, temper and (for wood) moisture and grain direction")));
            return Some(Answer::new("material properties", pods));
        }
    }
    // a bare material or a list of them: "carbon steel", "sugar maple, American white oak wood"
    let items = split_list(l);
    if items.is_empty() || items.len() > 6 {
        return None;
    }
    let found: Option<Vec<(String, Mat)>> = items.iter().map(|s| material(s)).collect();
    let found = found?;
    // single chemical elements are left to the element tables unless "metal" or "wood" is said
    if found.len() == 1 && matches!(found[0].1, Mat::M(_)) && !l.contains("steel") && !l.contains("iron") && !l.contains(' ') && !l.contains('-') {
        if !matches!(l, "brass" | "bronze" | "invar" | "steel") {
            return None;
        }
    }
    if found.len() == 1 {
        let (name, x) = &found[0];
        return Some(Answer::new(name, vec![input(format!("{} | material properties", cap(name))), primary(pod("Properties", "Typical properties", table(None, mat_rows(x).into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())))]));
    }
    let keys: Vec<&str> = mat_rows(&found[0].1).into_iter().map(|(k, _)| k).filter(|k| *k != "condition").collect();
    let mut head = vec!["property"];
    let names: Vec<String> = found.iter().map(|f| cap(&f.0)).collect();
    head.extend(names.iter().map(|s| s.as_str()));
    let rows: Vec<Vec<Content>> = keys.iter().map(|k| {
        let mut r = vec![text(*k)];
        for (_, x) in &found {
            r.push(text(mat_rows(x).into_iter().find(|(a, _)| a == k).map(|(_, v)| v).unwrap_or_else(|| "—".into())));
        }
        r
    }).collect();
    Some(Answer::new("material comparison", vec![input(names.join(" vs ")), primary(pod("Result", "Properties", table(Some(head), rows)))]))
}

// ---------------------------------------------------------------------------
// the Mohs scale
// ---------------------------------------------------------------------------

const MOHS: &[(f64, &str)] = &[(1.0, "talc"), (2.0, "gypsum"), (3.0, "calcite"), (4.0, "fluorite"), (5.0, "apatite"), (6.0, "orthoclase feldspar"), (7.0, "quartz"), (8.0, "topaz"), (9.0, "corundum"), (10.0, "diamond")];
const MOHS_COMMON: &[(f64, &str)] = &[(2.5, "fingernail"), (3.0, "copper coin (pre-1982 penny)"), (5.5, "glass, knife blade"), (6.5, "steel file, pyrite"), (7.0, "porcelain streak plate"), (6.5, "jadeite (6.5–7)"), (7.5, "garnet, zircon (7.5)"), (9.0, "ruby, sapphire")];

fn mohs(l: &str) -> Option<Answer> {
    if matches!(l, "mohs scale" | "mohs hardness scale" | "mohs scale of hardness" | "mohs scale of mineral hardness" | "the mohs scale") {
        let t: Vec<Vec<Content>> = MOHS.iter().map(|(h, m)| vec![text(n(*h)), text(cap(m))]).collect();
        return Some(Answer::new("Mohs scale", vec![input("Mohs scale of mineral hardness"), primary(pod("Result", "Reference minerals", table(Some(vec!["hardness", "mineral"]), t))), pod("Common", "Everyday references", table(None, MOHS_COMMON.iter().map(|(h, m)| vec![text(n(*h)), text(*m)]).collect()))]));
    }
    let c = re!(r"^(?:mohs|mohs hardness|hardness)\s+(\d+(?:\.\d+)?)$").captures(l)?;
    let h: f64 = c[1].parse().ok()?;
    if !(1.0..=10.0).contains(&h) {
        return None;
    }
    let below = MOHS.iter().rev().find(|(x, _)| *x <= h)?;
    let above = MOHS.iter().find(|(x, _)| *x >= h)?;
    let main = if below.0 == above.0 { format!("{} (reference mineral)", cap(below.1)) } else { format!("between {} ({}) and {} ({})", below.1, n(below.0), above.1, n(above.0)) };
    let common: Vec<Vec<Content>> = MOHS_COMMON.iter().filter(|(x, _)| (x - h).abs() < 0.6).map(|(x, s)| vec![text(n(*x)), text(*s)]).collect();
    let mut pods = vec![input(format!("Mohs hardness {}", n(h))), primary(pod("Result", "Position on the Mohs scale", text(main))), pod("Meaning", "Meaning", text(format!("scratches materials softer than {}; is scratched by harder ones", n(h))))];
    if !common.is_empty() {
        pods.push(pod("Common", "Materials of similar hardness", table(None, common)));
    }
    Some(Answer::new("Mohs hardness", pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    mohs(l).or_else(|| materials(l))
}
