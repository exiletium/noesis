//! Small practical calculators: tire sizes, tips, poker hand odds and
//! family relationships ("father's mother's sister's son").

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

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    tire(l).or_else(|| tip(l)).or_else(|| poker(l)).or_else(|| relationship(l))
}

fn n(v: f64, d: usize) -> String {
    fmt_float(v, d)
}

// ---------------------------------------------------------------------------
// tires
// ---------------------------------------------------------------------------

const SPEED_RATINGS: &[(&str, u32)] = &[("L", 120), ("M", 130), ("N", 140), ("P", 150), ("Q", 160), ("R", 170), ("S", 180), ("T", 190), ("U", 200), ("H", 210), ("V", 240), ("W", 270), ("Y", 300)];

/// Load index → maximum load in kg (ETRTO table, indices 60–126).
fn load_kg(li: u32) -> Option<u32> {
    const T: [u32; 67] = [250, 257, 265, 272, 280, 290, 300, 307, 315, 325, 335, 345, 355, 365, 375, 387, 400, 412, 425, 437, 450, 462, 475, 487, 500, 515, 530, 545, 560, 580, 600, 615, 630, 650, 670, 690, 710, 730, 750, 775, 800, 825, 850, 875, 900, 925, 950, 975, 1000, 1030, 1060, 1090, 1120, 1150, 1180, 1215, 1250, 1285, 1320, 1360, 1400, 1450, 1500, 1550, 1600, 1650, 1700];
    T.get(li.checked_sub(60)? as usize).copied()
}

fn tire_pods(title: String, width_mm: f64, sidewall_mm: f64, rim_in: f64, extra: Vec<Vec<Content>>, want: &str) -> Answer {
    let dia = rim_in * 25.4 + 2.0 * sidewall_mm;
    let circ = std::f64::consts::PI * dia;
    let mut rows = vec![
        vec![text("overall diameter"), text(format!("{} mm  ({} in)", n(dia, 5), n(dia / 25.4, 4)))],
        vec![text("section width"), text(format!("{} mm  ({} in)", n(width_mm, 4), n(width_mm / 25.4, 4)))],
        vec![text("sidewall height"), text(format!("{} mm  ({} in)", n(sidewall_mm, 4), n(sidewall_mm / 25.4, 4)))],
        vec![text("rim diameter"), text(format!("{} in  ({} mm)", n(rim_in, 4), n(rim_in * 25.4, 4)))],
        vec![text("circumference"), text(format!("{} mm  ({} in)", n(circ, 5), n(circ / 25.4, 4)))],
        vec![text("revolutions per mile"), text(n(1_609_344.0 / circ, 5))],
    ];
    rows.extend(extra);
    let pick = |key: &str| rows.iter().find(|r| content_text(&r[0]) == key).map(|r| content_text(&r[1]));
    let primary_text = match want {
        w if w.contains("circumference") => pick("circumference"),
        w if w.contains("sidewall") => pick("sidewall height"),
        w if w.contains("rim") => pick("rim diameter"),
        w if w.contains("width") => pick("section width"),
        w if w.contains("speed") => pick("maximum speed"),
        w if w.contains("load") => pick("maximum load"),
        _ => pick("overall diameter"),
    }
    .unwrap_or_default();
    let pods = vec![pod("Input", "Input interpretation", text(title.clone())), primary(pod("Result", "Result", text(primary_text))), pod("Dimensions", "Tire dimensions", table(None, rows))];
    Answer::new(title, pods)
}

fn tire(l: &str) -> Option<Answer> {
    if !(l.contains("tire") || l.contains("tyre") || re!(r"\b(?:p|lt|st|t)?\d{3}/\d{2,3}\s*z?[a-z]?r\d{2}\b").is_match(l)) {
        return None;
    }
    // P-metric and LT metric: P215/65R15 95H, 225/50SR16, LT245/75R16 E
    if let Some(m) = re!(r"\b(p|lt|st|t)?(\d{3})/(\d{2,3})\s*(z?[a-z]?)r(\d{2})(?:\s*(\d{2,3})\s*/?\s*\d*\s*([a-z]))?").captures(l) {
        let w: f64 = m[2].parse().ok()?;
        let ar: f64 = m[3].parse().ok()?;
        let rim: f64 = m[5].parse().ok()?;
        let class = match m.get(1).map(|x| x.as_str()) {
            Some("p") => "passenger (P-metric)",
            Some("lt") => "light truck",
            Some("st") => "special trailer",
            Some("t") => "temporary spare",
            _ => "metric (Euro)",
        };
        let mut extra = vec![vec![text("aspect ratio"), text(format!("{}%", n(ar, 3)))], vec![text("tire class"), text(class)]];
        let speed_letter = m.get(7).map(|x| x.as_str().to_uppercase()).or_else(|| m.get(4).map(|x| x.as_str().trim_start_matches('z').to_uppercase()).filter(|s| !s.is_empty()));
        if let Some(sl) = speed_letter {
            if let Some((_, v)) = SPEED_RATINGS.iter().find(|(k, _)| *k == sl) {
                extra.push(vec![text("maximum speed"), text(format!("{v} km/h  ({} mph)", n(*v as f64 / 1.609344, 3)))]);
            }
        }
        if let Some(li) = m.get(6).and_then(|x| x.as_str().parse::<u32>().ok()) {
            if let Some(kg) = load_kg(li) {
                extra.push(vec![text("maximum load"), text(format!("{kg} kg  ({} lb), load index {li}", n(kg as f64 * 2.20462262, 4)))]);
            }
        }
        return Some(tire_pods(format!("tire size {}{}/{}{}R{}", m.get(1).map_or("", |x| x.as_str()).to_uppercase(), &m[2], &m[3], m[4].to_uppercase(), &m[5]), w, w * ar / 100.0, rim, extra, l));
    }
    // flotation: 37X12.5R17LT, 31x10.50R15
    if let Some(m) = re!(r"\b(\d{2}(?:\.\d+)?)\s*x\s*(\d{1,2}(?:\.\d+)?)\s*r\s*(\d{2}(?:\.\d)?)").captures(l) {
        let (d, w, rim): (f64, f64, f64) = (m[1].parse().ok()?, m[2].parse().ok()?, m[3].parse().ok()?);
        return Some(tire_pods(format!("flotation tire {}x{}R{}", &m[1], &m[2], &m[3]), w * 25.4, (d - rim) * 25.4 / 2.0, rim, vec![], l));
    }
    // bicycle: ETRTO 32-498, 22-622, 700x35C, 700C
    let bike = re!(r"\b(\d{2})\s*[-–]\s*(\d{3})\b").captures(l).map(|m| (m[1].parse::<f64>().ok(), m[2].parse::<f64>().ok())).or_else(|| re!(r"\b700\s*x\s*(\d{2})\s*c\b").captures(l).map(|m| (m[1].parse::<f64>().ok(), Some(622.0)))).or_else(|| re!(r"\b700\s*c\b").captures(l).map(|_| (Some(25.0), Some(622.0))));
    if let Some((Some(w), Some(bead))) = bike {
        let dia = bead + 2.0 * w;
        let circ = std::f64::consts::PI * dia;
        let rows = vec![
            vec![text("tire width"), text(format!("{} mm", n(w, 4)))],
            vec![text("bead seat diameter"), text(format!("{} mm", n(bead, 4)))],
            vec![text("approximate outer diameter"), text(format!("{} mm  ({} in)", n(dia, 4), n(dia / 25.4, 4)))],
            vec![text("circumference"), text(format!("{} mm", n(circ, 5)))],
        ];
        let main = if l.contains("circumference") { format!("{} mm", n(circ, 5)) } else { format!("{} mm  ({} in)", n(dia, 4), n(dia / 25.4, 4)) };
        let title = format!("bicycle tire {}-{} (ETRTO)", w, bead);
        return Some(Answer::new(title.clone(), vec![pod("Input", "Input interpretation", text(title)), primary(pod("Result", "Result", text(main))), pod("Dimensions", "Tire dimensions", table(None, rows))]));
    }
    None
}

// ---------------------------------------------------------------------------
// tips
// ---------------------------------------------------------------------------

fn tip(l: &str) -> Option<Answer> {
    if !l.contains("tip") || l.contains("tipping point") {
        return None;
    }
    let bill: f64 = re!(r"\$\s*([\d,]+(?:\.\d+)?)|([\d,]+(?:\.\d+)?)\s*(?:dollars|usd|euros?)").captures(l).and_then(|m| m.get(1).or(m.get(2)).map(|x| x.as_str().replace(',', "")))?.parse().ok()?;
    let pct: Option<f64> = re!(r"([\d.]+)\s*%").captures(l).and_then(|m| m[1].parse().ok());
    let people: f64 = re!(r"(\d+)\s*(?:people|persons|ways|diners|guests)").captures(l).and_then(|m| m[1].parse().ok()).or_else(|| re!(r"split\s+(\d+)").captures(l).and_then(|m| m[1].parse().ok())).unwrap_or(1.0);
    let money = super::misc::money;
    let rows_for = |p: f64| -> Vec<Content> {
        let t = bill * p / 100.0;
        let mut r = vec![text(format!("{}%", n(p, 3))), text(money(t)), text(money(bill + t))];
        if people > 1.0 {
            r.push(text(money((bill + t) / people)));
        }
        r
    };
    let mut header = vec!["tip rate", "tip", "total"];
    if people > 1.0 {
        header.push("per person");
    }
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("tip on {}{}", money(bill), if people > 1.0 { format!(", split {} ways", people) } else { String::new() })))];
    match pct {
        Some(p) => {
            let t = bill * p / 100.0;
            let main = if people > 1.0 { format!("tip {}, total {}, {} per person", money(t), money(bill + t), money((bill + t) / people)) } else { format!("tip {}, total {}", money(t), money(bill + t)) };
            pods.push(primary(pod("Result", "Result", text(main))));
            pods.push(pod("Rates", "Other tip rates", table(Some(header), [10.0, 15.0, 18.0, 20.0, 25.0].iter().map(|r| rows_for(*r)).collect())));
        }
        None => {
            pods.push(primary(pod("Result", "Typical tips", table(Some(header), [15.0, 18.0, 20.0].iter().map(|r| rows_for(*r)).collect()))));
            pods.push(pod("Rates", "Other tip rates", table(None, [10.0, 22.0, 25.0].iter().map(|r| rows_for(*r)).collect())));
        }
    }
    Some(Answer::new("tip calculation", pods))
}

// ---------------------------------------------------------------------------
// poker
// ---------------------------------------------------------------------------

const HANDS: &[(&str, &[&str], u64)] = &[
    ("royal flush", &["royal flush"], 4),
    ("straight flush", &["straight flush"], 36),
    ("four of a kind", &["four of a kind", "quads", "4 of a kind"], 624),
    ("full house", &["full house", "full boat"], 3744),
    ("flush", &["flush"], 5108),
    ("straight", &["straight"], 10200),
    ("three of a kind", &["three of a kind", "trips", "set", "3 of a kind"], 54912),
    ("two pair", &["two pair", "two pairs", "2 pair"], 123552),
    ("one pair", &["one pair", "a pair", "pair"], 1098240),
    ("high card", &["high card", "no pair", "nothing"], 1302540),
];

fn poker(l: &str) -> Option<Answer> {
    if !l.contains("poker") && !l.contains("probability of") && !l.contains("odds of") {
        return None;
    }
    let body = l.replace("poker", " ").replace("probability of", " ").replace("odds of", " ").replace("getting", " ").replace(" a ", " ").replace("hand", " ");
    let body = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let (name, _, count) = HANDS.iter().find(|(_, names, _)| names.iter().any(|x| body == *x || body.starts_with(&format!("{x} ")) || body.ends_with(&format!(" {x}"))))?;
    const TOTAL: f64 = 2_598_960.0;
    let p = *count as f64 / TOTAL;
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("poker hand: {name} (5 cards from a 52-card deck)"))),
        primary(pod("Result", "Probability", text(format!("{} ≈ {}  ({}%)", ratio_text(*count, 2_598_960), n(p, 6), n(100.0 * p, 5))))),
        pod("Odds", "Odds against", text(format!("{} : 1", n((TOTAL - *count as f64) / *count as f64, 6)))),
        pod("Count", "Number of hands", text(format!("{count} of 2,598,960"))),
        pod("Table", "All hands", table(Some(vec!["hand", "count", "probability"]), HANDS.iter().map(|(h, _, c)| vec![text(*h), text(c.to_string()), text(format!("{}%", n(100.0 * *c as f64 / TOTAL, 4)))]).collect())),
    ];
    Some(Answer::new(format!("poker {name}"), pods))
}

fn ratio_text(a: u64, b: u64) -> String {
    fn gcd(a: u64, b: u64) -> u64 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
    let g = gcd(a, b);
    format!("{}/{}", a / g, b / g)
}

// ---------------------------------------------------------------------------
// family relationships
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Sex {
    M,
    F,
    U,
}

/// (generations up, generations down, sex) for one relationship word.
fn rel_word(w: &str) -> Option<(i32, i32, Sex)> {
    let w = w.trim().trim_end_matches("'s").trim_end_matches('’').trim_end_matches("’s");
    let mut greats = 0;
    let mut rest = w;
    loop {
        if let Some(r) = rest.strip_prefix("great-").or_else(|| rest.strip_prefix("great ")).or_else(|| rest.strip_prefix("great")) {
            greats += 1;
            rest = r.trim_start_matches('-');
        } else {
            break;
        }
    }
    let (u, d, s) = match rest {
        "father" | "dad" => (1, 0, Sex::M),
        "mother" | "mom" | "mum" => (1, 0, Sex::F),
        "parent" => (1, 0, Sex::U),
        "son" => (0, 1, Sex::M),
        "daughter" => (0, 1, Sex::F),
        "child" | "kid" => (0, 1, Sex::U),
        "brother" => (1, 1, Sex::M),
        "sister" => (1, 1, Sex::F),
        "sibling" => (1, 1, Sex::U),
        "grandfather" | "grandpa" => (2, 0, Sex::M),
        "grandmother" | "grandma" => (2, 0, Sex::F),
        "grandparent" => (2, 0, Sex::U),
        "grandson" => (0, 2, Sex::M),
        "granddaughter" => (0, 2, Sex::F),
        "grandchild" => (0, 2, Sex::U),
        "uncle" | "granduncle" => (2 + i32::from(rest.starts_with("grand")), 1, Sex::M),
        "aunt" | "auntie" | "grandaunt" => (2 + i32::from(rest.starts_with("grand")), 1, Sex::F),
        "nephew" | "grandnephew" => (1, 2 + i32::from(rest.starts_with("grand")), Sex::M),
        "niece" | "grandniece" => (1, 2 + i32::from(rest.starts_with("grand")), Sex::F),
        "cousin" | "first cousin" => (2, 2, Sex::U),
        "second cousin" => (3, 3, Sex::U),
        "third cousin" => (4, 4, Sex::U),
        "fourth cousin" => (5, 5, Sex::U),
        _ => return None,
    };
    // "great" moves an ancestor up or a descendant down
    Some(if u > 0 && d == 0 {
        (u + greats, d, s)
    } else if d > 0 && u == 0 {
        (u, d + greats, s)
    } else if d == 1 {
        (u + greats, d, s)
    } else {
        (u, d + greats, s)
    })
}

fn ordinal(k: i32) -> String {
    match k {
        1 => "first".into(),
        2 => "second".into(),
        3 => "third".into(),
        4 => "fourth".into(),
        5 => "fifth".into(),
        6 => "sixth".into(),
        7 => "seventh".into(),
        8 => "eighth".into(),
        _ => format!("{k}th"),
    }
}

fn times(k: i32) -> String {
    match k {
        1 => "once".into(),
        2 => "twice".into(),
        3 => "thrice".into(),
        _ => format!("{k} times"),
    }
}

fn greats(k: i32) -> String {
    "great-".repeat(k.max(0) as usize)
}

fn rel_name(u: i32, d: i32, s: Sex) -> String {
    let pick = |m: &str, f: &str, x: &str| match s {
        Sex::M => m.to_string(),
        Sex::F => f.to_string(),
        Sex::U => x.to_string(),
    };
    match (u, d) {
        (0, 0) => "yourself".into(),
        (1, 0) => pick("father", "mother", "parent"),
        (u, 0) => format!("{}{}", greats(u - 2), pick("grandfather", "grandmother", "grandparent")),
        (0, 1) => pick("son", "daughter", "child"),
        (0, d) => format!("{}{}", greats(d - 2), pick("grandson", "granddaughter", "grandchild")),
        (1, 1) => pick("brother", "sister", "sibling"),
        (u, 1) => format!("{}{}", greats(u - 2), pick("uncle", "aunt", "aunt or uncle")),
        (1, d) => format!("{}{}", greats(d - 2), pick("nephew", "niece", "niece or nephew")),
        (u, d) => {
            let deg = u.min(d) - 1;
            let removed = (u - d).abs();
            if removed == 0 {
                format!("{} cousin", ordinal(deg))
            } else {
                format!("{} cousin {} removed", ordinal(deg), times(removed))
            }
        }
    }
}

fn relationship(l: &str) -> Option<Answer> {
    let l = l.trim_start_matches("my ").trim_start_matches("who is my ").trim_start_matches("relationship ").trim_start_matches("of ").trim();
    // split "father's mother's sister's son" into words
    let parts: Vec<String> = re!(r"'s\s+|’s\s+|\s+of\s+(?:my\s+|the\s+)?").split(l).map(|s| s.trim().to_string()).collect();
    if parts.is_empty() || parts.len() > 12 {
        return None;
    }
    // "X of Y" reads right to left
    let ordered: Vec<String> = if l.contains(" of ") { parts.into_iter().rev().collect() } else { parts };
    let mut u = 0;
    let mut d = 0;
    let mut sex = Sex::U;
    for p in &ordered {
        let (pu, pd, ps) = rel_word(p)?;
        // going up from a descendant comes back towards ego (or a partner)
        let back = pu.min(d);
        d -= back;
        u += pu - back;
        d += pd;
        sex = ps;
    }
    if ordered.len() == 1 && !ordered[0].contains("cousin") && !ordered[0].contains("great") && !ordered[0].starts_with("grand") {
        // a single plain word ("mother") is a dictionary question, not this
        return None;
    }
    let name = rel_name(u, d, sex);
    // expected shared DNA for blood relatives through full siblings
    let r = if u == 0 || d == 0 { 0.5f64.powi(u + d) } else { 2.0 * 0.5f64.powi(u + d) };
    let pods = vec![
        pod("Input", "Input interpretation", text(ordered.join("'s "))),
        primary(pod("Result", "Relationship", text(name.clone()))),
        pod("Generations", "Generations", text(format!("{u} up, {d} down from you"))),
        pod("Blood", "Expected fraction of shared DNA", text(format!("{}%  (coefficient of relationship {})", n(100.0 * r, 5), n(r, 6)))),
    ];
    Some(Answer::new(format!("relationship: {name}"), pods))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rel(s: &str) -> String {
        let a = relationship(s).unwrap();
        content_text(&a.pods[1].subpods[0].content)
    }

    #[test]
    fn relationships() {
        assert_eq!(rel("father's mother's sister's son"), "first cousin once removed");
        assert_eq!(rel("great aunt's grandson"), "second cousin");
        assert_eq!(rel("third cousin's great-grandson"), "third cousin thrice removed");
        assert_eq!(rel("granddaughter"), "granddaughter");
        assert_eq!(rel("grandmother's aunt"), "great-great-aunt");
        assert_eq!(rel("great great grandmother"), "great-great-grandmother");
    }

    #[test]
    fn tires() {
        let a = tire("tire size 195/55r16").unwrap();
        assert!(content_text(&a.pods[1].subpods[0].content).starts_with("620.9 mm"), "{}", content_text(&a.pods[1].subpods[0].content));
        let a = tire("what is the maximum speed of a p215/65r15 95h m+s tire").unwrap();
        assert!(content_text(&a.pods[1].subpods[0].content).starts_with("210 km/h"));
    }
}
