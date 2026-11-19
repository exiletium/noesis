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
    (&["starburst", "starbursts", "starburst candy", "starburst candies", "starburst fruit chews", "starburst chews"], "Starburst candies", 3.8, 0.62, "wrapped fruit chew about 23 mm × 19 mm × 9 mm (5 g)"),
    (&["lego brick", "lego bricks", "legos", "lego"], "LEGO 2×4 bricks", 3.6, 0.45, "31.8 mm × 15.8 mm × 11.4 mm brick (hollow, random fill)"),
    (&["candy corn", "candy corns"], "candy corns", 1.6, 0.6, "about 20 mm × 15 mm × 8 mm wedge"),
    (&["trade paperback", "trade paperbacks", "trade paperback book", "trade paperback books", "paperback", "paperbacks", "paperback book", "paperback books"], "trade paperback books", 766.0, 0.85, "about 5.5 in × 8.5 in × 1 in (14 cm × 21.6 cm × 2.5 cm), packed flat in stacks"),
    (&["hardcover book", "hardcover books", "hardback", "hardbacks", "hardcover", "hardcovers"], "hardcover books", 1470.0, 0.85, "about 6.25 in × 9.5 in × 1.5 in, packed flat in stacks"),
    (&["book", "books"], "books", 766.0, 0.85, "a trade paperback, about 5.5 in × 8.5 in × 1 in, packed flat in stacks"),
];

/// names | display | volume cm³ | note
const CONTAINERS: &[(&[&str], &str, f64, &str)] = &[
    (&["christmas stocking", "stocking", "xmas stocking"], "a Christmas stocking", 6000.0, "typical 18-inch (46 cm) stocking, about 6 L"),
    (&["mason jar", "quart mason jar", "quart jar"], "a quart mason jar", 946.0, "1 US quart"),
    (&["pint mason jar", "pint jar"], "a pint mason jar", 473.0, "1 US pint"),
    (&["shoebox", "shoe box"], "a shoebox", 6900.0, "about 33 cm × 19 cm × 11 cm"),
    (&["bathtub", "bath tub", "bath"], "a bathtub", 300000.0, "about 300 L to the overflow"),
    (&["school bus", "bus"], "a school bus", 5.66e7, "about 2,000 cubic feet of interior space"),
    (&["olympic swimming pool", "olympic pool", "swimming pool"], "an Olympic swimming pool", 2.5e9, "50 m × 25 m × 2 m (2,500 m³)"),
    (&["car", "sedan"], "a car", 3.0e6, "about 3 m³ of passenger space"),
    (&["fishbowl", "fish bowl"], "a fishbowl", 3785.0, "about 1 US gallon"),
    (&["coffee mug", "mug"], "a coffee mug", 350.0, "about 12 fl oz"),
    (&["suitcase", "checked suitcase", "medium suitcase", "luggage", "checked bag"], "a suitcase", 70000.0, "a medium checked suitcase, about 25 in × 18 in × 10 in inside (70 L)"),
    (&["carry-on suitcase", "carry on suitcase", "carry-on", "carry on", "carry-on bag", "carry on bag"], "a carry-on suitcase", 40000.0, "about 22 in × 14 in × 9 in outside, 40 L inside"),
    (&["backpack", "school backpack"], "a backpack", 25000.0, "a typical 25 L school backpack"),
];

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let m = re!(r"^(?:how many |number of |approximate number of |estimated number of )?(.+?) (?:can |will |would |that |could |it takes |does it take |needed |required )*(?:(?:fit|go|fill|be packed) (?:in|into|inside)|to fill|fill) (?:a |an |one |the )?(.+)$").captures(&l)?;
    let obj = OBJECTS.iter().find(|o| o.0.contains(&m[1].trim()))?;
    let cont = m[2].trim();
    let (cm3, cont, cont_note) = if let Some(c) = CONTAINERS.iter().find(|c| c.0.contains(&cont)) {
        (c.2, c.1.to_string(), Some(c.3))
    } else {
        // "a 1 gallon jar" → "1 gallon"
        let cont = re!(r"^(.*?\d.*?)[ -](?:jar|container|bucket|box|bottle|tank|bag|vase|bowl|tub)$").captures(cont).map_or(cont.to_string(), |m| m[1].to_string());
        // "a pint" → "1 pint"
        let cont_q = if cont.starts_with(|c: char| c.is_ascii_digit()) { cont.clone() } else { format!("1 {cont}") };
        let (n, u, _) = crate::domains::units::parse_quantity(&cont_q)?;
        if u.dim != [3, 0, 0, 0, 0, 0, 0, 0] {
            return None;
        }
        (crate::domains::units::to_si(n, &u) * 1e6, cont, None)
    };
    let count = cm3 * obj.3 / obj.2;
    if !(count.is_finite() && count >= 1.0) {
        return None;
    }
    let shown = if count >= 100.0 { super::geo::fmt_int((count / 10.0).round() as u64 * 10) } else { fmt_float(count.round(), 6) };
    Some(Answer::new(format!("{} in {cont}", obj.1), vec![
        pod("Input", "Input interpretation", text(format!("number of {} that fit in {cont}", obj.1))),
        primary(pod("Result", "Estimate", text(format!("about {shown}")))),
        pod("Table", "Assumptions", table(None, vec![
            vec![text("container volume"), text(format!("{} cm³{}", fmt_float((cm3 * 10.0).round() / 10.0, 7), cont_note.map_or(String::new(), |n| format!(" ({n})"))))],
            vec![text("object"), text(obj.4)],
            vec![text("object volume"), text(format!("{} cm³", fmt_float(obj.2, 5)))],
            vec![text("packing fraction"), text(fmt_float(obj.3, 3))],
        ])),
        pod("Note", "Method", text("count ≈ container volume × packing fraction ÷ object volume; walls lower the count in small containers")),
    ]))
}
