//! "How many X fit in Y" estimates: object volume and random packing
//! fraction against a container volume.
//!
//! "How many marbles fit in 1000 cubic inches?", "number of M&Ms that fit in
//! a pint?", "how many golf balls fit in a gallon".

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

/// names | display | volume cm³ | random packing fraction | size note
const OBJECTS: &[(&[&str], &str, f64, f64, &str)] = &[
    (&["marble", "marbles"], "marbles", 2.145, 0.64, "standard 16 mm glass marble (sphere)"),
    (&["m&m", "m&ms", "m&m's", "m and ms", "plain m&ms", "milk chocolate m&ms"], "plain M&M's", 0.637, 0.68, "oblate ellipsoid about 13 mm × 13 mm × 7.2 mm"),
    (&["peanut m&m", "peanut m&ms", "peanut m&m's"], "peanut M&M's", 2.5, 0.64, "roughly 20 mm × 16 mm × 15 mm"),
    (&["golf ball", "golf balls"], "golf balls", 40.68, 0.64, "42.67 mm sphere"),
    (&["ping pong ball", "ping pong balls", "table tennis ball", "table tennis balls"], "ping pong balls", 33.51, 0.64, "40 mm sphere"),
    (&["tennis ball", "tennis balls"], "tennis balls", 157.5, 0.64, "6.7 cm sphere"),
    (&["baseball", "baseballs"], "baseballs", 212.2, 0.64, "7.4 cm sphere"),
    (&["softball", "softballs"], "softballs", 496.6, 0.64, "9.83 cm sphere (12-inch softball)"),
    (&["basketball", "basketballs"], "basketballs", 7238.0, 0.64, "24 cm sphere (size 7)"),
    (&["soccer ball", "soccer balls", "football", "footballs"], "soccer balls", 5575.0, 0.64, "22 cm sphere (size 5)"),
    (&["jelly bean", "jelly beans", "jellybean", "jellybeans"], "jelly beans", 1.9, 0.66, "about 20 mm × 15 mm × 12 mm"),
    (&["gumball", "gumballs"], "gumballs", 8.18, 0.64, "25 mm sphere"),
    (&["skittle", "skittles"], "Skittles", 0.75, 0.68, "oblate ellipsoid about 14 mm × 14 mm × 7.5 mm"),
    (&["bb", "bbs", "bb pellet", "bb pellets"], "BBs", 0.0382, 0.64, "4.5 mm sphere"),
    (&["penny", "pennies"], "US pennies", 0.36, 0.60, "19.05 mm × 1.52 mm disk (random fill)"),
];

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let m = re!(r"^(?:how many |number of |approximate number of |estimated number of )?(.+?) (?:can |will |would |that |could )*(?:fit|go|fill|be packed) (?:in|into|inside) (?:a |an |one |the )?(.+)$").captures(&l)?;
    let obj = OBJECTS.iter().find(|o| o.0.contains(&m[1].trim()))?;
    let cont = m[2].trim();
    // "a pint" → "1 pint"
    let cont_q = if cont.starts_with(|c: char| c.is_ascii_digit()) { cont.to_string() } else { format!("1 {cont}") };
    let (n, u, _) = crate::domains::units::parse_quantity(&cont_q)?;
    if u.dim != [3, 0, 0, 0, 0, 0, 0, 0] {
        return None;
    }
    let cm3 = crate::domains::units::to_si(n, &u) * 1e6;
    let count = cm3 * obj.3 / obj.2;
    if !(count.is_finite() && count >= 1.0) {
        return None;
    }
    let shown = if count >= 100.0 { super::geo::fmt_int((count / 10.0).round() as u64 * 10) } else { fmt_float(count.round(), 6) };
    Some(Answer::new(format!("{} in {cont}", obj.1), vec![
        pod("Input", "Input interpretation", text(format!("number of {} that fit in {cont}", obj.1))),
        primary(pod("Result", "Estimate", text(format!("about {shown}")))),
        pod("Table", "Assumptions", table(None, vec![
            vec![text("container volume"), text(format!("{} cm³", fmt_float((cm3 * 10.0).round() / 10.0, 7)))],
            vec![text("object"), text(obj.4)],
            vec![text("object volume"), text(format!("{} cm³", fmt_float(obj.2, 5)))],
            vec![text("random packing fraction"), text(fmt_float(obj.3, 3))],
        ])),
        pod("Note", "Method", text("count ≈ container volume × packing fraction ÷ object volume; walls lower the count in small containers")),
    ]))
}
