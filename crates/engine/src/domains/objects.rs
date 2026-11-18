//! Everyday objects and Fermi estimates: typical sizes and masses of
//! familiar things (a football field, a school bus, an elephant, a penny),
//! comparisons between them, counts such as legs or flag stripes, and how
//! many small objects it takes to cover an area, stretch a distance or
//! fill a volume.

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
    fmt_float(v, 6)
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

/// Typical object: names, display name, length (m), cross-section area
/// when laid flat (m²), volume (m³), mass (kg), note. Zero means unknown.
struct Obj {
    names: &'static [&'static str],
    name: &'static str,
    length: f64,
    flat_area: f64,
    volume: f64,
    mass: f64,
    height: f64,
    note: &'static str,
}

const OBJECTS: &[Obj] = &[
    Obj { names: &["penny", "pennies", "us penny", "cent", "cents"], name: "US penny", length: 0.01905, flat_area: 2.850e-4, volume: 4.33e-7, mass: 0.0025, height: 0.00152, note: "diameter 19.05 mm, thickness 1.52 mm, mass 2.5 g" },
    Obj { names: &["grain of rice", "grains of rice", "rice grain", "rice grains", "rice"], name: "grain of rice", length: 0.007, flat_area: 1.4e-5, volume: 2.0e-8, mass: 2.5e-5, height: 0.0, note: "long-grain rice: about 7 mm long, 25 mg" },
    Obj { names: &["toothpick", "toothpicks"], name: "toothpick", length: 0.066, flat_area: 0.0, volume: 1.1e-7, mass: 1.0e-4, height: 0.0, note: "2.6 inches (6.6 cm) long" },
    Obj { names: &["m&m", "m&m's", "m&ms", "m & m", "m and m", "m and ms"], name: "M&M", length: 0.013, flat_area: 0.0, volume: 6.4e-7, mass: 9.1e-4, height: 0.0, note: "plain M&M: about 13 mm across, 0.91 g" },
    Obj { names: &["baseball", "baseballs"], name: "baseball", length: 0.0737, flat_area: 0.0, volume: 2.10e-4, mass: 0.145, height: 0.0, note: "circumference 9–9.25 in, mass 5–5.25 oz" },
    Obj { names: &["golf ball", "golf balls"], name: "golf ball", length: 0.04267, flat_area: 0.0, volume: 4.07e-5, mass: 0.04593, height: 0.0, note: "diameter 42.67 mm, at most 45.93 g" },
    Obj { names: &["tennis ball", "tennis balls"], name: "tennis ball", length: 0.067, flat_area: 0.0, volume: 1.57e-4, mass: 0.058, height: 0.0, note: "diameter 6.54–6.86 cm" },
    Obj { names: &["ping pong ball", "ping pong balls", "table tennis ball", "table tennis balls"], name: "table tennis ball", length: 0.04, flat_area: 0.0, volume: 3.35e-5, mass: 0.0027, height: 0.0, note: "diameter 40 mm, 2.7 g" },
    Obj { names: &["sheet of paper", "sheets of paper", "piece of paper", "pieces of paper", "paper"], name: "sheet of letter paper", length: 0.2794, flat_area: 0.06032, volume: 6.0e-6, mass: 0.0045, height: 1.0e-4, note: "US letter, 8.5 × 11 in, about 0.1 mm thick" },
    Obj { names: &["dollar bill", "dollar bills", "bill", "bills", "us dollar bill"], name: "US dollar bill", length: 0.1561, flat_area: 0.01036, volume: 1.1e-6, mass: 0.001, height: 1.1e-4, note: "6.14 × 2.61 in, 0.0043 in thick, about 1 g" },
    Obj { names: &["football field", "american football field", "football fields"], name: "American football field", length: 109.728, flat_area: 5351.0, volume: 0.0, mass: 0.0, height: 0.0, note: "120 yd long including end zones (100 yd between goal lines), 53⅓ yd wide" },
    Obj { names: &["soccer field", "soccer pitch", "football pitch"], name: "soccer field (FIFA standard)", length: 105.0, flat_area: 7140.0, volume: 0.0, mass: 0.0, height: 0.0, note: "105 × 68 m" },
    Obj { names: &["basketball court"], name: "basketball court (NBA)", length: 28.65, flat_area: 436.6, volume: 0.0, mass: 0.0, height: 0.0, note: "94 × 50 ft" },
    Obj { names: &["school bus", "bus", "school buses"], name: "school bus", length: 12.2, flat_area: 29.8, volume: 0.0, mass: 11000.0, height: 3.2, note: "full-size type C: about 40 ft long, 8 ft wide" },
    Obj { names: &["tyrannosaurus rex", "t-rex", "t rex", "tyrannosaurus"], name: "Tyrannosaurus rex", length: 12.3, flat_area: 0.0, volume: 0.0, mass: 8400.0, height: 3.7, note: "largest specimens (Sue): about 12.3 m long, 3.7 m at the hips, 8.4 t" },
    Obj { names: &["triceratops"], name: "Triceratops", length: 9.0, flat_area: 0.0, volume: 0.0, mass: 9000.0, height: 3.0, note: "about 8–9 m long, 6–12 t" },
    Obj { names: &["blue whale"], name: "blue whale", length: 25.0, flat_area: 0.0, volume: 0.0, mass: 140000.0, height: 0.0, note: "typically 24–30 m, up to about 190 t" },
    Obj { names: &["elephant", "african elephant", "african bush elephant"], name: "African bush elephant", length: 6.5, flat_area: 0.0, volume: 0.0, mass: 6000.0, height: 3.2, note: "adult males about 6 t and 3.2 m at the shoulder; females about 3 t" },
    Obj { names: &["asian elephant"], name: "Asian elephant", length: 6.0, flat_area: 0.0, volume: 0.0, mass: 4000.0, height: 2.75, note: "adult males about 4 t" },
    Obj { names: &["giraffe"], name: "giraffe", length: 4.5, flat_area: 0.0, volume: 0.0, mass: 1200.0, height: 5.5, note: "males up to 5.5 m tall, about 1.2 t" },
    Obj { names: &["airplane", "aeroplane", "plane", "jet airliner", "airliner", "boeing 737", "737"], name: "Boeing 737-800 (typical airliner)", length: 39.5, flat_area: 0.0, volume: 0.0, mass: 41400.0, height: 12.5, note: "length 39.5 m, wingspan 35.8 m, empty mass 41.4 t" },
    Obj { names: &["boeing 747", "747", "jumbo jet"], name: "Boeing 747-400", length: 70.6, flat_area: 0.0, volume: 876.0, mass: 178_800.0, height: 19.4, note: "length 70.6 m, wingspan 64.4 m; interior cabin about 876 m³" },
    Obj { names: &["basketball hoop", "basketball rim", "basketball basket"], name: "basketball hoop", length: 0.4572, flat_area: 0.0, volume: 0.0, mass: 0.0, height: 3.048, note: "rim 10 ft (3.05 m) above the floor, 18 in diameter" },
    Obj { names: &["basketball", "basketballs"], name: "basketball (size 7)", length: 0.239, flat_area: 0.0, volume: 7.15e-3, mass: 0.62, height: 0.0, note: "circumference 29.5 in, 22 oz" },
    Obj { names: &["pencil", "pencils"], name: "wooden pencil", length: 0.19, flat_area: 0.0, volume: 6.0e-6, mass: 0.006, height: 0.0, note: "unsharpened #2 pencil: about 7.5 in long, 6 g" },
    Obj { names: &["can of coke", "cans of coke", "can of soda", "cans of soda", "soda can", "soda cans", "coke can", "coke cans"], name: "can of soda", length: 0.122, flat_area: 0.0, volume: 3.55e-4, mass: 0.383, height: 0.122, note: "12 US fl oz = 355 mL" },
    Obj { names: &["car", "cars", "automobile"], name: "car (mid-size sedan)", length: 4.8, flat_area: 8.9, volume: 0.0, mass: 1500.0, height: 1.45, note: "about 4.8 m long, 1.5 t" },
    Obj { names: &["empire state building"], name: "Empire State Building", length: 0.0, flat_area: 0.0, volume: 0.0, mass: 331_000_000.0, height: 443.2, note: "443.2 m to the tip, 381 m to the roof" },
    Obj { names: &["eiffel tower"], name: "Eiffel Tower", length: 0.0, flat_area: 0.0, volume: 0.0, mass: 10_100_000.0, height: 330.0, note: "330 m tall including antennas" },
    Obj { names: &["olympic swimming pool", "olympic pool", "swimming pool"], name: "Olympic swimming pool", length: 50.0, flat_area: 1250.0, volume: 2500.0, mass: 0.0, height: 2.0, note: "50 × 25 m, at least 2 m deep: 2500 m³" },
];

/// Large targets: names, display name, length (m), area (m²), volume (m³).
const TARGETS: &[(&[&str], &str, f64, f64, f64)] = &[
    (&["the moon", "moon"], "the Moon (circumference / surface)", 10_921_000.0, 3.793e13, 2.1958e19),
    (&["the earth", "earth", "the world", "world", "the equator", "equator"], "the Earth (equatorial circumference / surface)", 40_075_017.0, 5.10072e14, 1.08321e21),
    (&["the sun", "sun"], "the Sun", 4.3713e9, 6.09e18, 1.41e27),
    (&["mars"], "Mars", 21_344_000.0, 1.4437e14, 1.6318e20),
    (&["united states", "the united states", "usa", "the us", "us"], "the United States (land area)", 0.0, 9.147e12, 0.0),
];

fn find_obj(s: &str) -> Option<&'static Obj> {
    let s = s.trim().trim_start_matches("the ").trim_start_matches("an ").trim_start_matches("a ").trim();
    OBJECTS.iter().find(|o| o.names.contains(&s))
}

fn fermi(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let l = l.trim_start_matches("how many ").trim_start_matches("number of ").trim_start_matches("the number of ");
    let (thing, verb, target) = if let Some(m) = re!(r"^(.+?) (?:would |does |do |will |could |can )?(?:it )?(?:take |are needed |needed )?to (cover|stretch|fill|reach)(?: (?:around|across|along|to))? (.+)$").captures(l) {
        (m[1].to_string(), m[2].to_string(), m[3].to_string())
    } else if let Some(m) = re!(r"^(.+?) (?:would |will |could |can )?fit (?:in|into|inside) (?:an? |the )?(.+)$").captures(l) {
        (m[1].to_string(), "fill".to_string(), m[2].to_string())
    } else {
        return None;
    };
    let thing = thing.trim_start_matches("would it take ").trim();
    let thing = re!(r"\s+(?:would|does|do) it take$").replace(thing, "").to_string();
    let obj = find_obj(&thing)?;
    let target = target.trim().trim_start_matches("the distance of ").trim();
    // the target: a quantity or a named object
    let (amount, what): (f64, String) = if let Some((v, u, label)) = crate::domains::units::parse_quantity(target) {
        let need = match verb.as_str() { "cover" => 2, "fill" => 3, _ => 1 };
        if u.dim[0] != need || u.dim[1..].iter().any(|d| *d != 0) || u.offset != 0.0 {
            return None;
        }
        let _ = label;
        (v * u.factor, target.to_string())
    } else if let Some(t) = TARGETS.iter().find(|t| t.0.contains(&target)) {
        let v = match verb.as_str() { "cover" => t.3, "fill" => t.4, _ => t.2 };
        if v == 0.0 {
            return None;
        }
        (v, t.1.to_string())
    } else if let Some(o) = find_obj(target) {
        let v = match verb.as_str() { "cover" => o.flat_area, "fill" => o.volume, _ => o.length };
        if v == 0.0 {
            return None;
        }
        (v, o.name.to_string())
    } else {
        return None;
    };
    let (count, how) = match verb.as_str() {
        "cover" => {
            if obj.flat_area == 0.0 {
                return None;
            }
            // coins and balls pack hexagonally when laid flat: 90.7% coverage
            let round = obj.name.contains("penny");
            let per = if round { obj.flat_area / 0.9069 } else { obj.flat_area };
            (amount / per, if round { "laid flat without overlapping, in hexagonal packing (90.7% of the area covered)" } else { "laid flat edge to edge" })
        }
        "fill" => {
            if obj.volume == 0.0 {
                return None;
            }
            (amount * 0.64 / obj.volume, "random close packing (about 64% of the volume filled)")
        }
        _ => {
            if obj.length == 0.0 {
                return None;
            }
            (amount / obj.length, "laid end to end along their longest dimension")
        }
    };
    let mut details = vec![vec![text("object"), text(format!("{} ({})", obj.name, obj.note))], vec![text("assumption"), text(how)]];
    if obj.mass > 0.0 {
        details.push(vec![text("total mass"), text(format!("{} kg", n(count * obj.mass)))]);
    }
    if obj.name.contains("penny") {
        details.push(vec![text("total value"), text(format!("${}", n((count / 100.0).round())))]);
    }
    Some(Answer::new("estimate", vec![
        input(format!("number of {}s to {verb} {what}", obj.name)),
        primary(pod("Result", "Estimate", text(format!("about {}", n(count.round()))))),
        pod("Details", "Details", table(None, details)),
    ]))
}

fn property(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let (prop, name, unit) = if let Some(m) = re!(r"^(?:what is )?(?:the )?(length|height|weight|mass|size|volume) of (?:an? |the )?(.+?)(?: in ([a-z ]+))?$").captures(l) {
        (m[1].to_string(), m[2].to_string(), m.get(3).map(|x| x.as_str().to_string()))
    } else if let Some(m) = re!(r"^how (heavy|tall|long|big|high) is (?:an? |the )?(.+)$").captures(l) {
        let p = match &m[1] { "heavy" => "weight", "tall" | "high" => "height", _ => "length" };
        (p.to_string(), m[2].to_string(), None)
    } else if let Some(m) = re!(r"^(?:an? |the )?(.+?) (length|height|weight|mass) in ([a-z ]+)$").captures(l) {
        (m[2].to_string(), m[1].to_string(), Some(m[3].to_string()))
    } else {
        return None;
    };
    let o = find_obj(&name)?;
    let (v, base, label) = match prop.as_str() {
        "weight" | "mass" => (o.mass, "kg", "mass"),
        "height" if o.height > 0.0 => (o.height, "m", "height"),
        "volume" => (o.volume, "m^3", "volume"),
        _ => (if o.length > 0.0 { o.length } else { o.height }, "m", "length"),
    };
    if v == 0.0 {
        return None;
    }
    let shown = if let Some(u) = unit {
        let (_, tu, _) = crate::domains::units::parse_quantity(&format!("1 {u}"))?;
        format!("{} {}", n(v / tu.factor), u.trim())
    } else if base == "kg" {
        format!("{} kg ({} lb)", n(v), n(v / 0.45359237))
    } else if base == "m" {
        format!("{} m ({} ft)", n(v), n(v / 0.3048))
    } else {
        format!("{} m³ ({} L)", n(v), n(v * 1000.0))
    };
    Some(Answer::new(label, vec![
        input(format!("{label} of a typical {}", o.name)),
        primary(pod("Result", "Typical value", text(shown))),
        pod("Notes", "Notes", text(o.note)),
    ]))
}

fn compare(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^(?:compare (?:the )?(?:lengths?|sizes?|heights?|weights?|masses) (?:of )?|difference between (?:the )?(?:lengths? of )?)(?:an? |the )?(.+?) (?:and|vs\.?|versus) (?:an? |the )?(.+?)(?: lengths?| sizes?)?$").captures(l)?;
    let (a, b) = (find_obj(&m[1])?, find_obj(&m[2])?);
    let heavy = l.contains("weight") || l.contains("mass");
    let (va, vb, unit) = if heavy { (a.mass, b.mass, "kg") } else { (a.length, b.length, "m") };
    if va == 0.0 || vb == 0.0 {
        return None;
    }
    let ft = |v: f64| if unit == "m" { format!("{} m ({} ft)", n(v), n(v / 0.3048)) } else { format!("{} kg ({} lb)", n(v), n(v / 0.45359237)) };
    Some(Answer::new("comparison", vec![
        input(format!("{} vs {} ({})", a.name, b.name, if heavy { "mass" } else { "length" })),
        primary(pod("Result", "Difference", text(ft((va - vb).abs())))),
        pod("Values", "Values", table(None, vec![vec![text(a.name), text(ft(va))], vec![text(b.name), text(ft(vb))], vec![text("ratio"), text(n(va / vb))]])),
    ]))
}

/// Simple counts: legs of animals, sides and vertices of shapes, flag features.
const COUNTS: &[(&str, &[&str], u32)] = &[
    ("legs", &["spider"], 8), ("legs", &["cow", "horse", "dog", "cat", "elephant", "lion", "giraffe", "sheep", "pig", "goat"], 4), ("legs", &["insect", "ant", "bee", "beetle", "fly", "butterfly"], 6),
    ("legs", &["human", "person", "bird", "chicken", "duck", "ostrich", "penguin"], 2), ("legs", &["octopus"], 8), ("legs", &["crab", "lobster", "shrimp"], 10), ("legs", &["snake", "fish"], 0), ("legs", &["starfish"], 5),
    ("arms", &["octopus"], 8), ("arms", &["starfish"], 5), ("arms", &["squid"], 8), ("tentacles", &["squid"], 2),
    ("vertices", &["triangle"], 3), ("vertices", &["square", "rectangle", "rhombus", "quadrilateral", "trapezoid", "parallelogram"], 4), ("vertices", &["pentagon"], 5), ("vertices", &["hexagon"], 6), ("vertices", &["octagon"], 8), ("vertices", &["cube"], 8), ("vertices", &["tetrahedron"], 4), ("vertices", &["circle"], 0),
    ("sides", &["triangle"], 3), ("sides", &["square", "rectangle", "rhombus", "quadrilateral", "trapezoid", "parallelogram"], 4), ("sides", &["pentagon"], 5), ("sides", &["hexagon"], 6), ("sides", &["heptagon"], 7), ("sides", &["octagon", "stop sign"], 8), ("sides", &["decagon"], 10),
    ("faces", &["cube", "die"], 6), ("faces", &["tetrahedron"], 4), ("faces", &["octahedron"], 8), ("faces", &["dodecahedron"], 12), ("faces", &["icosahedron"], 20),
    ("edges", &["cube"], 12), ("edges", &["tetrahedron"], 6),
    ("stripes", &["american flag", "us flag", "united states flag", "flag of the united states"], 13), ("stars", &["american flag", "us flag", "united states flag", "flag of the united states"], 50),
    ("stars", &["australian flag", "flag of australia"], 6), ("stars", &["chinese flag", "flag of china"], 5), ("stars", &["brazilian flag", "flag of brazil"], 27), ("stars", &["new zealand flag", "flag of new zealand"], 4), ("stars", &["eu flag", "european flag", "flag of europe"], 12),
    ("keys", &["piano"], 88), ("strings", &["guitar"], 6), ("strings", &["violin"], 4), ("cards", &["deck of cards", "deck"], 52), ("squares", &["chessboard", "chess board"], 64), ("pieces", &["chess set"], 32),
    ("stitches", &["baseball", "major league baseball"], 108), ("stitches", &["softball"], 88), ("dimples", &["golf ball"], 336), ("panels", &["soccer ball", "football (soccer)"], 32), ("holes", &["golf course"], 18), ("players", &["soccer team", "football team"], 11), ("players", &["basketball team"], 5), ("players", &["baseball team"], 9),
    ("teeth", &["adult human", "adult", "human"], 32), ("bones", &["human body", "adult human", "human"], 206),
];

fn counts(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^(?:how many )?([a-z]+) (?:are )?(?:on|in|does|do|has|have) (?:an? |the )?(.+?)(?: have| has)?$").captures(l)?;
    let what = m[1].to_string();
    let parts: Vec<String> = m[2].split(" vs ").flat_map(|p| p.split(" versus ")).flat_map(|p| p.split(" and ")).map(|p| {
        let p = p.trim().trim_start_matches("a ").trim_start_matches("an ").trim_start_matches("the ");
        let p = p.trim_start_matches(&format!("{what} on ")).trim_start_matches(&format!("{what} of ")).trim_start_matches("a ").trim_start_matches("an ").trim_start_matches("the ");
        p.to_string()
    }).collect();
    let mut rows = Vec::new();
    for p in &parts {
        let single = p.trim_end_matches('s');
        let c = COUNTS.iter().find(|c| c.0 == what && (c.1.contains(&p.as_str()) || c.1.contains(&single)))?;
        rows.push((p.clone(), c.2));
    }
    if rows.is_empty() {
        return None;
    }
    let head = rows.iter().map(|(p, c)| if rows.len() == 1 { c.to_string() } else { format!("{p}: {c}") }).collect::<Vec<_>>().join(", ");
    let mut pods = vec![input(format!("number of {what} on {}", parts.join(" vs "))), primary(pod("Result", "Result", text(head)))];
    if rows.len() == 2 {
        pods.push(pod("Difference", "Difference", text((rows[0].1 as i64 - rows[1].1 as i64).abs().to_string())));
    }
    Some(Answer::new("count", pods))
}

/// "how many liters in 6 cans of coke"
fn object_units(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^(?:how many )?([a-z ]+?) (?:are )?in (\d+(?:\.\d+)?) (.+)$").captures(l)?;
    let o = find_obj(&m[3])?;
    let k: f64 = m[2].parse().ok()?;
    let (_, u, _) = crate::domains::units::parse_quantity(&format!("1 {}", m[1].trim()))?;
    let label = m[1].trim().to_string();
    let total = match u.dim {
        d if d[0] == 3 && d[1..].iter().all(|x| *x == 0) => k * o.volume,
        d if d[0] == 1 && d[1..].iter().all(|x| *x == 0) => k * o.length,
        d if d[1] == 1 && d[0] == 0 => k * o.mass,
        _ => return None,
    };
    if total == 0.0 {
        return None;
    }
    Some(Answer::new("quantity", vec![
        input(format!("{} × {} in {label}", n(k), o.name)),
        primary(pod("Result", "Result", text(format!("{} {label}", n(total / u.factor))))),
        pod("Notes", "Notes", text(o.note)),
    ]))
}

/// US banknotes in circulation: value, portrait, back design, year of the current design.
const BILLS: &[(u32, &str, &str, &str)] = &[
    (1, "George Washington", "the Great Seal of the United States", "1963 (small-size note design of 1935)"),
    (2, "Thomas Jefferson", "John Trumbull's painting Declaration of Independence", "1976"),
    (5, "Abraham Lincoln", "the Lincoln Memorial", "2008"),
    (10, "Alexander Hamilton", "the US Treasury Building", "2006"),
    (20, "Andrew Jackson", "the White House", "2003"),
    (50, "Ulysses S. Grant", "the US Capitol", "2004"),
    (100, "Benjamin Franklin", "Independence Hall", "2013"),
];

/// "width of a dollar bill", "what does a US $2 bill look like", "who is on the $50 bill", "$100 bill"
fn banknote(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^(?:(?:what is |what's )?(?:the )?(width|length|thickness|size|dimensions|weight|mass|height) of (?:an? |the |one )?|what does (?:an? |the )?|who(?:'s| is) on (?:an? |the )?)?(?:us |u\.s\. |american )?(?:\$(\d+)|(one|two|five|ten|twenty|fifty|hundred|one hundred|\d+)[- ]dollar|dollar)( bill| banknote| note)(?: look like)?$").captures(l)?;
    let v: u32 = match (m.get(2), m.get(3)) {
        (Some(d), _) => d.as_str().parse().ok()?,
        (None, Some(w)) => match w.as_str() { "one" => 1, "two" => 2, "five" => 5, "ten" => 10, "twenty" => 20, "fifty" => 50, "hundred" | "one hundred" => 100, d => d.parse().ok()? },
        _ => 1,
    };
    let &(_, portrait, back, design) = BILLS.iter().find(|b| b.0 == v)?;
    let prop = m.get(1).map(|x| x.as_str());
    let result = match prop {
        Some("width" | "height") => ("Width", "2.61 in (66.3 mm)".to_string()),
        Some("length") => ("Length", "6.14 in (156 mm)".to_string()),
        Some("thickness") => ("Thickness", "0.0043 in (0.11 mm)".to_string()),
        Some("weight" | "mass") => ("Mass", "about 1 g (0.035 oz)".to_string()),
        Some(_) => ("Dimensions", "6.14 in × 2.61 in (156 mm × 66.3 mm), 0.0043 in (0.11 mm) thick".to_string()),
        None if l.starts_with("who") => ("Portrait", portrait.to_string()),
        None => ("Design", format!("front: portrait of {portrait}; back: {back}")),
    };
    Some(Answer::new("US banknote", vec![
        input(format!("US ${v} bill{}", prop.map(|p| format!(" | {p}")).unwrap_or_default())),
        primary(pod("Result", result.0, text(result.1))),
        pod("Table", "Basic information", table(None, vec![
            vec![text("face value"), text(format!("${v}"))],
            vec![text("portrait (front)"), text(portrait)],
            vec![text("back"), text(back)],
            vec![text("current design series"), text(design)],
            vec![text("dimensions"), text("6.14 in × 2.61 in (156 mm × 66.3 mm)")],
            vec![text("thickness"), text("0.0043 in (0.11 mm)")],
            vec![text("mass"), text("about 1 g")],
            vec![text("material"), text("75% cotton, 25% linen")],
        ])),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    banknote(q).or_else(|| fermi(q)).or_else(|| compare(q)).or_else(|| counts(q)).or_else(|| object_units(q)).or_else(|| property(q))
}
