//! School mathematics asked in words: comparisons, place value, counting,
//! coins, clock times, rates, shapes and their properties, lines from a
//! slope or a point, probability rules, betting odds, parabolas from a
//! focus and directrix, ellipses from their foci, polar coordinates.

use crate::engine::*;
use crate::expr::*;
use crate::parse;
use crate::print::{self, fmt_float};
use crate::simplify as s;
use num_traits::{Signed, ToPrimitive, Zero};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    compare(l)
        .or_else(|| place_value(l))
        .or_else(|| count(l))
        .or_else(|| multiples(l))
        .or_else(|| coins(l))
        .or_else(|| clock(l))
        .or_else(|| rate(l))
        .or_else(|| sign_rule(l))
        .or_else(|| shapes(l))
        .or_else(|| polyhedra(l))
        .or_else(|| lines(q, l))
        .or_else(|| monotonic(q, l))
        .or_else(|| probability_rules(l))
        .or_else(|| odds(l))
        .or_else(|| parabola(q, l))
        .or_else(|| ellipse(l))
        .or_else(|| rationality(l))
        .or_else(|| order(l))
        .or_else(|| polar(q, l))
        .or_else(|| arc(l))
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// An exact value when there is one, else a decimal.
fn value(s: &str) -> Option<(Expr, f64)> {
    let t = s.trim().trim_start_matches('$').replace(',', "");
    if t.is_empty() || t.chars().any(|c| c.is_ascii_alphabetic() && !"sqrtpiabse".contains(c)) {
        return None;
    }
    let e = s::simplify(&parse::parse(&t).ok()?.expr);
    let v = crate::eval::eval_f64(&e, &[])?;
    v.is_finite().then_some((e, v))
}

fn exact_rat(s: &str) -> Option<Rat> {
    match value(s)?.0 {
        Expr::Num(r) => Some(r),
        _ => None,
    }
}

fn show(e: &Expr) -> String {
    print::text(e)
}

fn n(v: f64) -> String {
    fmt_float(v, 6)
}

/// Run another query and keep its pods.
fn ask(q: &Query, text: &str) -> Option<QueryResult> {
    let r = crate::engine::query_with(text, q.opts);
    r.success.then_some(r)
}

fn small_number(w: &str) -> Option<u64> {
    const W: [&str; 21] = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen", "twenty"];
    if w == "a" || w == "an" {
        return Some(1);
    }
    W.iter().position(|x| *x == w).map(|i| i as u64).or_else(|| w.parse().ok())
}

// ---------------------------------------------------------------------------
// comparisons: "which is greater, 7/8 or 3/4", "is 5 greater than 2"
// ---------------------------------------------------------------------------

fn compare(l: &str) -> Option<Answer> {
    let (a, b, ask_big) = if let Some(m) = re!(r"^which (?:number )?(?:is|one is) (greater|larger|bigger|more|smaller|less|lesser)[,:]?\s+(.+?)\s+or\s+(.+)$").captures(l) {
        (m[2].to_string(), m[3].to_string(), Some(matches!(&m[1], "greater" | "larger" | "bigger" | "more")))
    } else if let Some(m) = re!(r"^is (.+?) (greater|larger|bigger|more|smaller|less|fewer) than (.+)$").captures(l) {
        (m[1].to_string(), m[3].to_string(), Some(matches!(&m[2], "greater" | "larger" | "bigger" | "more")))
    } else if let Some(m) = re!(r"^compare (.+?) (?:and|with|to|vs\.?|versus) (.+)$").captures(l) {
        (m[1].to_string(), m[2].to_string(), None)
    } else {
        return None;
    };
    let (ea, va) = value(&a)?;
    let (eb, vb) = value(&b)?;
    // decimals stay as typed: 2.34, not 117/50
    let typed = |src: &str, e: Expr| if src.contains('.') && !src.contains('/') { Expr::Sym(src.trim().into()) } else { e };
    let (ea, eb) = (typed(&a, ea), typed(&b, eb));
    let rel = if (va - vb).abs() <= 1e-12 * va.abs().max(vb.abs()) { "=" } else if va < vb { "<" } else { ">" };
    let statement = format!("{} {rel} {}", show(&ea), show(&eb));
    let result = match (ask_big, l.starts_with("is ")) {
        (Some(big), true) => {
            let yes = if big { rel == ">" } else { rel == "<" };
            format!("{} ({statement})", if yes { "yes" } else { "no" })
        }
        (Some(big), false) => {
            if rel == "=" {
                "they are equal".to_string()
            } else {
                let pick = if (rel == ">") == big { &ea } else { &eb };
                format!("{} is {}", show(pick), if big { "greater" } else { "smaller" })
            }
        }
        (None, _) => statement.clone(),
    };
    let mut pods = vec![pod("Input", "Input", text(format!("{} vs {}", show(&ea), show(&eb)))), primary(pod("Result", "Result", text(result))), pod("Comparison", "Comparison", text(statement))];
    if va != vb && va.abs() > 0.0 && vb.abs() > 0.0 {
        pods.push(pod("Difference", "Difference", text(format!("{} − {} = {}", show(&ea), show(&eb), show(&s::simplify(&add(vec![ea.clone(), mul(vec![int(-1), eb.clone()])])))))));
    }
    if !matches!(ea, Expr::Num(ref r) if r.is_integer()) || !matches!(eb, Expr::Num(ref r) if r.is_integer()) {
        pods.push(pod("Decimal", "Decimal values", text(format!("{} and {}", n(va), n(vb)))));
    }
    Some(Answer::new(format!("compare {} and {}", show(&ea), show(&eb)), pods))
}

// ---------------------------------------------------------------------------
// place value
// ---------------------------------------------------------------------------

const PLACES: [&str; 13] = ["ones", "tens", "hundreds", "thousands", "ten thousands", "hundred thousands", "millions", "ten millions", "hundred millions", "billions", "ten billions", "hundred billions", "trillions"];
const DECIMAL_PLACES: [&str; 6] = ["tenths", "hundredths", "thousandths", "ten-thousandths", "hundred-thousandths", "millionths"];

/// (digit, place name, value) for each digit of a number written in decimal.
fn digits_with_places(num: &str) -> Option<Vec<(char, String, String)>> {
    let t = num.replace(',', "");
    let t = t.trim_start_matches('-');
    if t.is_empty() || !t.chars().all(|c| c.is_ascii_digit() || c == '.') || t.matches('.').count() > 1 {
        return None;
    }
    let (int, frac) = t.split_once('.').unwrap_or((t, ""));
    if int.len() > PLACES.len() || frac.len() > DECIMAL_PLACES.len() {
        return None;
    }
    let mut out = Vec::new();
    for (i, c) in int.chars().enumerate() {
        let p = int.len() - 1 - i;
        let v = format!("{c}{}", "0".repeat(p));
        out.push((c, PLACES[p].to_string(), v));
    }
    for (i, c) in frac.chars().enumerate() {
        out.push((c, DECIMAL_PLACES[i].to_string(), format!("0.{}{c}", "0".repeat(i))));
    }
    Some(out)
}

fn place_value(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:what is )?(?:the )?(?:place )?value of (?:the )?(?:digit )?(\d) in (?:the number )?([\d,]+(?:\.\d+)?)$").captures(l) {
        let d = m[1].chars().next()?;
        let rows = digits_with_places(&m[2])?;
        let hits: Vec<_> = rows.iter().filter(|r| r.0 == d).collect();
        if hits.is_empty() {
            return None;
        }
        let place: Vec<String> = hits.iter().map(|r| format!("{} place (value {})", r.1, r.2)).collect();
        let pods = vec![pod("Input", "Input", text(format!("digit {d} in {}", &m[2]))), primary(pod("Result", "Place value", text(place.join("; "))))];
        return Some(Answer::new(format!("place value of {d} in {}", &m[2]), pods));
    }
    let m = re!(r"^(?:the )?place values? (?:of |for |in )?(?:the number )?([\d,]+(?:\.\d+)?)$").captures(l)?;
    let rows = digits_with_places(&m[1])?;
    let expanded: Vec<String> = rows.iter().filter(|r| r.0 != '0').map(|r| r.2.clone()).collect();
    let table_rows = rows.iter().map(|r| vec![text(r.0.to_string()), text(r.1.clone()), text(r.2.clone())]).collect();
    let pods = vec![
        pod("Input", "Input", text(format!("place values of {}", &m[1]))),
        primary(pod("Result", "Place values", table(Some(vec!["digit", "place", "value"]), table_rows))),
        pod("Expanded", "Expanded form", text(expanded.join(" + "))),
    ];
    Some(Answer::new(format!("place values of {}", &m[1]), pods))
}

// ---------------------------------------------------------------------------
// counting and multiples
// ---------------------------------------------------------------------------

fn count(l: &str) -> Option<Answer> {
    let m = re!(r"^count (?:from )?(-?\d+) (?:to|through|until) (-?\d+)(?: by (\w+))?$").captures(l)?;
    let (a, b): (i64, i64) = (m[1].parse().ok()?, m[2].parse().ok()?);
    let step = match m.get(3).map(|x| x.as_str()) {
        None => 1,
        Some(w) => small_number(w.trim_end_matches('s')).or_else(|| match w {
            "ones" => Some(1),
            "twos" => Some(2),
            "fives" => Some(5),
            "tens" => Some(10),
            "twenties" => Some(20),
            "hundreds" | "hundred" => Some(100),
            _ => None,
        })? as i64,
    };
    if step == 0 || (b - a).abs() / step > 1000 {
        return None;
    }
    let dir = if b >= a { step } else { -step };
    let mut v = Vec::new();
    let mut x = a;
    while (dir > 0 && x <= b) || (dir < 0 && x >= b) {
        v.push(x.to_string());
        x += dir;
    }
    let pods = vec![pod("Input", "Input", text(format!("count from {a} to {b} by {step}"))), primary(pod("Result", "Result", text(v.join(", ")))), pod("Count", "Number of terms", text(v.len().to_string()))];
    Some(Answer::new(format!("count from {a} to {b}"), pods))
}

fn multiples(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?(?:first (\d+) )?multiples of (\d+)(?: (?:up to|less than|below|under) (\d+))?$").captures(l)?;
    let k: u64 = m[2].parse().ok()?;
    if k == 0 {
        return None;
    }
    let list: Vec<u64> = match (m.get(1), m.get(3)) {
        (_, Some(lim)) => {
            let lim: u64 = lim.as_str().parse().ok()?;
            (1..).map(|i| i * k).take_while(|v| *v <= lim).take(500).collect()
        }
        (Some(c), None) => (1..=c.as_str().parse::<u64>().ok()?.min(500)).map(|i| i * k).collect(),
        (None, None) => (1..=12).map(|i| i * k).collect(),
    };
    let pods = vec![
        pod("Input", "Input", text(format!("multiples of {k}"))),
        primary(pod("Result", "Multiples", text(format!("{}, …", list.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))))),
        pod("Rule", "Rule", text(format!("{k} n for n = 1, 2, 3, …"))),
    ];
    Some(Answer::new(format!("multiples of {k}"), pods))
}

// ---------------------------------------------------------------------------
// coins: "how much is 3 dimes and 4 pennies"
// ---------------------------------------------------------------------------

fn coins(l: &str) -> Option<Answer> {
    let body = l.trim_start_matches("how much money is ").trim_start_matches("how much is ").trim_start_matches("what is the value of ").trim_start_matches("value of ").trim();
    let item = re!(r"^(\d+|a|an|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve)\s+(pennies|penny|cents?|nickels?|dimes?|quarters?|half[- ]dollars?|dollar coins?|dollars?|dollar bills?|\$1 bills?|\$5 bills?|\$10 bills?|\$20 bills?|five[- ]dollar bills?|ten[- ]dollar bills?|twenty[- ]dollar bills?)$");
    let parts: Vec<&str> = body.split([',', '+']).flat_map(|p| p.split(" and ")).flat_map(|p| p.split(" plus ")).map(str::trim).filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return None;
    }
    let mut cents = 0u64;
    let mut rows = Vec::new();
    for p in &parts {
        let m = item.captures(p)?;
        let k = small_number(&m[1])?;
        let w = &m[2];
        let each = if w.starts_with("penn") || w.starts_with("cent") {
            1
        } else if w.starts_with("nickel") {
            5
        } else if w.starts_with("dime") {
            10
        } else if w.starts_with("quarter") {
            25
        } else if w.starts_with("half") {
            50
        } else if w.contains("20") || w.starts_with("twenty") {
            2000
        } else if w.contains("10") || w.starts_with("ten") {
            1000
        } else if w.contains('5') || w.starts_with("five") {
            500
        } else {
            100
        };
        cents += k * each;
        rows.push(vec![text(p.to_string()), text(format!("${}.{:02}", k * each / 100, k * each % 100))]);
    }
    if !l.starts_with("how much") && !l.starts_with("value") && !l.starts_with("what is the value") && parts.len() < 2 {
        return None;
    }
    let total = format!("${}.{:02}", cents / 100, cents % 100);
    let pods = vec![
        pod("Input", "Input", text(parts.join(" + "))),
        primary(pod("Result", "Total", text(format!("{total}  ({cents} cents)")))),
        pod("Breakdown", "Breakdown", table(Some(vec!["coins", "value"]), rows)),
    ];
    Some(Answer::new(format!("value of {}", parts.join(" and ")), pods))
}

// ---------------------------------------------------------------------------
// clock times: "2:25 pm", "quarter after 3", "4:45pm to 7:15pm"
// ---------------------------------------------------------------------------

/// Minutes after midnight.
fn parse_time(s: &str) -> Option<u32> {
    let s = s.trim();
    let named = |w: &str| -> Option<u32> {
        match w {
            "noon" | "midday" => Some(12 * 60),
            "midnight" => Some(0),
            _ => {
                let m = re!(r"^(\d{1,2})(?:\s*(am|pm|a\.m\.|p\.m\.))?(?:\s*o'?clock)?$").captures(w)?;
                let h: u32 = m[1].parse().ok()?;
                if h > 23 {
                    return None;
                }
                let pm = m.get(2).is_some_and(|x| x.as_str().starts_with('p'));
                Some(match m.get(2) {
                    Some(_) => (h % 12 + if pm { 12 } else { 0 }) * 60,
                    None => h * 60,
                })
            }
        }
    };
    if let Some(m) = re!(r"^(\d{1,2}):(\d{2})\s*(am|pm|a\.m\.|p\.m\.)?$").captures(s) {
        let (h, mi): (u32, u32) = (m[1].parse().ok()?, m[2].parse().ok()?);
        if h > 23 || mi > 59 {
            return None;
        }
        let h = match m.get(3) {
            Some(x) => h % 12 + if x.as_str().starts_with('p') { 12 } else { 0 },
            None => h,
        };
        return Some(h * 60 + mi);
    }
    if let Some(m) = re!(r"^(quarter|half|\d{1,2}|five|ten|twenty|twenty-five)(?: minutes)? (past|after|to|before|till|til) (.+)$").captures(s) {
        let k = match &m[1] {
            "quarter" => 15,
            "half" => 30,
            "five" => 5,
            "ten" => 10,
            "twenty" => 20,
            "twenty-five" => 25,
            d => d.parse().ok()?,
        };
        let base = named(&m[3])?;
        return Some(if matches!(&m[2], "past" | "after") { base + k } else { (base + 24 * 60 - k) % (24 * 60) });
    }
    if s.contains("o'clock") || s.contains("oclock") || s.ends_with("am") || s.ends_with("pm") || s == "noon" || s == "midnight" {
        return named(s);
    }
    None
}

fn twelve(t: u32) -> String {
    let (h, m) = (t / 60 % 24, t % 60);
    format!("{}:{m:02} {}", if h % 12 == 0 { 12 } else { h % 12 }, if h < 12 { "AM" } else { "PM" })
}

fn clock(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:how (?:long|much time|many hours) is it )?(?:(?:time )?from )?(.+?)\s+(?:to|until|till|-)\s+(.+)$").captures(l) {
        if let (Some(a), Some(b)) = (parse_time(&m[1]), parse_time(&m[2])) {
            let d = (b + 24 * 60 - a) % (24 * 60);
            let words = match (d / 60, d % 60) {
                (0, mm) => format!("{mm} minutes"),
                (h, 0) => format!("{h} hour{}", if h == 1 { "" } else { "s" }),
                (h, mm) => format!("{h} hour{} {mm} minutes", if h == 1 { "" } else { "s" }),
            };
            let pods = vec![
                pod("Input", "Input", text(format!("from {} to {}", twelve(a), twelve(b)))),
                primary(pod("Result", "Elapsed time", text(words))),
                pod("Other", "Other units", text(format!("{} minutes = {} hours", d, n(d as f64 / 60.0)))),
            ];
            return Some(Answer::new(format!("time from {} to {}", twelve(a), twelve(b)), pods));
        }
    }
    let t = parse_time(l)?;
    let (h, mi) = (t / 60, t % 60);
    // angle between the hands
    let hour_angle = (h % 12) as f64 * 30.0 + mi as f64 * 0.5;
    let minute_angle = mi as f64 * 6.0;
    let mut ang = (hour_angle - minute_angle).abs();
    if ang > 180.0 {
        ang = 360.0 - ang;
    }
    let part = match h {
        0..=11 => "in the morning",
        12..=16 => "in the afternoon",
        17..=20 => "in the evening",
        _ => "at night",
    };
    let pods = vec![
        pod("Input", "Input interpretation", text(twelve(t))),
        primary(pod("Result", "Time", text(format!("{} ({part})", twelve(t))))),
        pod("24h", "24-hour clock", text(format!("{h:02}:{mi:02}"))),
        pod("Hands", "Angle between the clock hands", text(format!("{}°", n(ang)))),
    ];
    Some(Answer::new(twelve(t), pods))
}

// ---------------------------------------------------------------------------
// rates: "0.75 feet per 0.25 hour", "$80 for 10 items", "500 pages at 2.25 minutes per page"
// ---------------------------------------------------------------------------

fn rate(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^([\d.,/]+)\s+([a-z]+(?: [a-z]+)?)\s+per\s+([\d.,/]+)\s+(?:of an? )?([a-z]+)$").captures(l) {
        let (a, b) = (value(&m[1])?, value(&m[3])?);
        if b.1 == 0.0 {
            return None;
        }
        let r = s::simplify(&mul(vec![a.0.clone(), pow(b.0.clone(), int(-1))]));
        let unit = m[4].trim_end_matches('s');
        let res = format!("{} {} per {unit}", show(&r), &m[2]);
        let pods = vec![pod("Input", "Input", text(format!("{} {} per {} {}", &m[1], &m[2], &m[3], &m[4]))), primary(pod("Result", "Unit rate", text(res.clone())))];
        return Some(Answer::new(res, pods));
    }
    if let Some(m) = re!(r"^\$([\d.,]+)\s+for\s+(\d+)\s+([a-z]+)$").captures(l) {
        let (a, k) = (value(&m[1])?.1, m[2].parse::<f64>().ok()?);
        if k == 0.0 {
            return None;
        }
        let each = a / k;
        let unit = m[3].trim_end_matches('s');
        let pods = vec![pod("Input", "Input", text(format!("${} for {} {}", &m[1], &m[2], &m[3]))), primary(pod("Result", "Unit price", text(format!("${:.2} per {unit}", each))))];
        return Some(Answer::new(format!("unit price of ${} for {}", &m[1], &m[2]), pods));
    }
    if let Some(m) = re!(r"^([\d.,]+)\s+([a-z]+)\s+at\s+([\d.,/]+)\s+([a-z$]+)\s+(?:per|a|an|each)\s+([a-z]+)$").captures(l) {
        let (a, r) = (value(&m[1])?, value(&m[3])?);
        let tot = s::simplify(&mul(vec![a.0, r.0]));
        let res = format!("{} {}", show(&tot), &m[4]);
        let mut pods = vec![pod("Input", "Input", text(format!("{} {} × {} {} per {}", &m[1], &m[2], &m[3], &m[4], &m[5]))), primary(pod("Result", "Total", text(res.clone())))];
        if matches!(&m[4], "minutes" | "minute" | "min") {
            let v = a.1 * r.1;
            pods.push(pod("Hours", "In hours", text(format!("{} hours", n(v / 60.0)))));
        }
        return Some(Answer::new(res, pods));
    }
    None
}

fn sign_rule(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:a )?(negative|positive) (?:number|integer)?\s*(\*|×|x|times|multiplied by|/|÷|divided by) (?:a )?(negative|positive)(?: number| integer)?$").captures(l)?;
    let same = m[1] == m[3];
    let op = if matches!(&m[2], "/" | "÷" | "divided by") { "quotient" } else { "product" };
    let res = if same { "positive" } else { "negative" };
    let pods = vec![
        pod("Input", "Input", text(format!("{} {} {}", &m[1], if op == "product" { "×" } else { "÷" }, &m[3]))),
        primary(pod("Result", "Result", text(format!("{res} number")))),
        pod("Rule", "Rule", text(format!("The {op} of two numbers with {} signs is {res}.", if same { "the same" } else { "opposite" }))),
        pod("Example", "Example", text(match (&m[1], &m[3]) {
            (a, b) if a == "negative" && b == "negative" => "(−3) × (−4) = 12",
            (a, _) if a == "negative" => "(−3) × 4 = −12",
            (_, b) if b == "negative" => "3 × (−4) = −12",
            _ => "3 × 4 = 12",
        })),
    ];
    Some(Answer::new(format!("{} {op}", &m[1]), pods))
}

// ---------------------------------------------------------------------------
// shapes
// ---------------------------------------------------------------------------

struct Shape {
    names: &'static [&'static str],
    sides: u32,
    vertices: u32,
    definition: &'static str,
}

const SHAPES: &[Shape] = &[
    Shape { names: &["triangle"], sides: 3, vertices: 3, definition: "a polygon with three sides and three angles; its angles add up to 180°" },
    Shape { names: &["equilateral triangle"], sides: 3, vertices: 3, definition: "a triangle whose three sides are equal; each angle is 60°" },
    Shape { names: &["isosceles triangle"], sides: 3, vertices: 3, definition: "a triangle with (at least) two equal sides; the angles opposite them are equal" },
    Shape { names: &["scalene triangle"], sides: 3, vertices: 3, definition: "a triangle whose three sides all have different lengths, so its three angles also differ" },
    Shape { names: &["right triangle", "right-angled triangle"], sides: 3, vertices: 3, definition: "a triangle with one 90° angle; its sides satisfy a² + b² = c²" },
    Shape { names: &["acute triangle"], sides: 3, vertices: 3, definition: "a triangle whose three angles are all less than 90°" },
    Shape { names: &["obtuse triangle"], sides: 3, vertices: 3, definition: "a triangle with one angle greater than 90°" },
    Shape { names: &["quadrilateral", "quadrangle", "tetragon"], sides: 4, vertices: 4, definition: "a polygon with four sides; its angles add up to 360°" },
    Shape { names: &["square"], sides: 4, vertices: 4, definition: "a quadrilateral with four equal sides and four right angles" },
    Shape { names: &["rectangle"], sides: 4, vertices: 4, definition: "a quadrilateral with four right angles; opposite sides are equal and parallel" },
    Shape { names: &["rhombus", "rhombi"], sides: 4, vertices: 4, definition: "a quadrilateral with four equal sides; opposite sides are parallel and opposite angles equal" },
    Shape { names: &["parallelogram"], sides: 4, vertices: 4, definition: "a quadrilateral whose opposite sides are parallel (and so equal)" },
    Shape { names: &["trapezoid", "trapezium"], sides: 4, vertices: 4, definition: "a quadrilateral with exactly one pair of parallel sides (some authors say at least one pair)" },
    Shape { names: &["kite"], sides: 4, vertices: 4, definition: "a quadrilateral with two pairs of equal adjacent sides" },
    Shape { names: &["pentagon"], sides: 5, vertices: 5, definition: "a polygon with five sides; its angles add up to 540°" },
    Shape { names: &["hexagon"], sides: 6, vertices: 6, definition: "a polygon with six sides; its angles add up to 720°" },
    Shape { names: &["heptagon", "septagon"], sides: 7, vertices: 7, definition: "a polygon with seven sides; its angles add up to 900°" },
    Shape { names: &["octagon"], sides: 8, vertices: 8, definition: "a polygon with eight sides; its angles add up to 1080°" },
    Shape { names: &["nonagon", "enneagon"], sides: 9, vertices: 9, definition: "a polygon with nine sides; its angles add up to 1260°" },
    Shape { names: &["decagon"], sides: 10, vertices: 10, definition: "a polygon with ten sides; its angles add up to 1440°" },
    Shape { names: &["hendecagon", "undecagon"], sides: 11, vertices: 11, definition: "a polygon with eleven sides" },
    Shape { names: &["dodecagon"], sides: 12, vertices: 12, definition: "a polygon with twelve sides; its angles add up to 1800°" },
    Shape { names: &["circle"], sides: 0, vertices: 0, definition: "the set of points in a plane at a fixed distance (the radius) from a center; area π r², circumference 2 π r" },
    Shape { names: &["ellipse", "oval"], sides: 0, vertices: 0, definition: "the set of points whose distances to two foci add up to a constant; area π a b" },
    Shape { names: &["sphere", "ball"], sides: 0, vertices: 0, definition: "the set of points in space at a fixed distance from a center; volume 4/3 π r³, surface area 4 π r²" },
    Shape { names: &["cube"], sides: 6, vertices: 8, definition: "a solid with six square faces, twelve edges and eight vertices; volume a³" },
    Shape { names: &["cylinder"], sides: 3, vertices: 0, definition: "a solid with two parallel circular bases joined by a curved surface; volume π r² h" },
    Shape { names: &["cone"], sides: 2, vertices: 1, definition: "a solid with a circular base joined to a single apex; volume π r² h / 3" },
];

fn shape(name: &str) -> Option<&'static Shape> {
    let n = name.trim().trim_start_matches("a ").trim_start_matches("an ").trim_start_matches("the ").trim_end_matches('s');
    SHAPES.iter().find(|s| s.names.iter().any(|x| *x == n || x.trim_end_matches('s') == n))
}

/// Is every `a` also a `b`? (inclusive definitions, trapezoid exclusive)
fn is_a(a: &str, b: &str) -> Option<bool> {
    let parents: &[(&str, &[&str])] = &[
        ("square", &["rectangle", "rhombus", "parallelogram", "quadrilateral", "kite", "polygon"]),
        ("rectangle", &["parallelogram", "quadrilateral", "polygon"]),
        ("rhombus", &["parallelogram", "quadrilateral", "kite", "polygon"]),
        ("parallelogram", &["quadrilateral", "polygon"]),
        ("trapezoid", &["quadrilateral", "polygon"]),
        ("kite", &["quadrilateral", "polygon"]),
        ("equilateral triangle", &["isosceles triangle", "triangle", "acute triangle", "polygon"]),
        ("isosceles triangle", &["triangle", "polygon"]),
        ("scalene triangle", &["triangle", "polygon"]),
        ("right triangle", &["triangle", "polygon"]),
        ("triangle", &["polygon"]),
        ("circle", &["ellipse"]),
        ("cube", &["rectangular prism", "prism", "polyhedron", "cuboid"]),
    ];
    let a = shape(a).map_or(a, |s| s.names[0]);
    let b = shape(b).map_or(b.trim_start_matches("a ").trim_start_matches("an "), |s| s.names[0]);
    if a == b {
        return Some(true);
    }
    let (_, ps) = parents.iter().find(|(x, _)| *x == a)?;
    Some(ps.contains(&b))
}

fn shapes(l: &str) -> Option<Answer> {
    let mk = |title: &str, res: String, extra: Vec<Pod>| {
        let mut pods = vec![pod("Input", "Input", text(title.to_string())), primary(pod("Result", "Result", text(res)))];
        pods.extend(extra);
        Some(Answer::new(title.to_string(), pods))
    };
    if let Some(m) = re!(r"^(?:what (?:is the )?)?(?:shape|polygon|figure|name of (?:a|the) (?:shape|polygon)) (?:has |with |that has )?(\d+|three|four|five|six|seven|eight|nine|ten|eleven|twelve) sides$").captures(l).or_else(|| re!(r"^what (?:shape|polygon) has (\d+|three|four|five|six|seven|eight|nine|ten|eleven|twelve) sides$").captures(l)) {
        let k = small_number(&m[1])? as u32;
        let s = SHAPES.iter().find(|s| s.sides == k && s.vertices == k && !s.names[0].contains(' ') && s.names[0] != "square")?;
        let interior = (k as i64 - 2) * 180;
        return mk(&format!("polygon with {k} sides"), s.names[0].to_string(), vec![pod("Angles", "Sum of interior angles", text(format!("{interior}°"))), pod("Regular", "Each angle of the regular polygon", text(format!("{}°", n(interior as f64 / k as f64))))]);
    }
    if let Some(m) = re!(r"^(?:which|what) has more (sides|vertices|corners|angles|edges|faces),? (?:a |an )?([a-z ]+?) or (?:a |an )?([a-z ]+)$").captures(l) {
        let (a, b) = (shape(&m[2])?, shape(&m[3])?);
        let (va, vb) = if &m[1] == "sides" || &m[1] == "faces" { (a.sides, b.sides) } else { (a.vertices, b.vertices) };
        let res = match va.cmp(&vb) {
            std::cmp::Ordering::Greater => format!("{} ({va} vs {vb})", a.names[0]),
            std::cmp::Ordering::Less => format!("{} ({vb} vs {va})", b.names[0]),
            _ => format!("they have the same number ({va})"),
        };
        return mk(&format!("{} of {} vs {}", &m[1], a.names[0], b.names[0]), res, vec![]);
    }
    if let Some(m) = re!(r"^how many (sides|vertices|corners|angles|edges|faces) (?:on|does|do|in|has) (?:a |an )?([a-z ]+?) (?:vs\.?|versus|and|compared to) (?:a |an )?([a-z ]+?)(?: have)?$").captures(l) {
        let (a, b) = (shape(&m[2])?, shape(&m[3])?);
        let f = |s: &Shape| if &m[1] == "sides" || &m[1] == "faces" { s.sides } else { s.vertices };
        return mk(&format!("{} of {} and {}", &m[1], a.names[0], b.names[0]), format!("{}: {}  |  {}: {}", a.names[0], f(a), b.names[0], f(b)), vec![]);
    }
    if let Some(m) = re!(r"^is (?:a |an |every )?([a-z ]+?) (?:a|an|also a|also an|always a|always an) ([a-z ]+)$").captures(l) {
        let yes = is_a(&m[1], &m[2])?;
        let (a, b) = (shape(&m[1]).map_or(m[1].to_string(), |s| s.names[0].to_string()), shape(&m[2]).map_or(m[2].to_string(), |s| s.names[0].to_string()));
        let why = if yes { format!("Every {a} is a {b}.") } else { format!("A {a} is not in general a {b}.") };
        let mut extra = vec![pod("Why", "Explanation", text(why))];
        if let Some(sa) = shape(&a) {
            extra.push(pod("Def", &format!("Definition of {a}"), text(sa.definition)));
        }
        return mk(&format!("is a {a} a {b}"), if yes { "yes".into() } else { "no".into() }, extra);
    }
    if let Some(m) = re!(r"^(?:what is|what's|define|definition of|mathematical definition of|what does) (?:an? |the )?([a-z -]+?)(?: look like| mean)?$").captures(l) {
        let name = m[1].trim_start_matches("mathematical definition of ").trim_start_matches("a ").trim();
        if let Some(sh) = shape(name) {
            return mk(&format!("definition of {}", sh.names[0]), sh.definition.to_string(), shape_pods(sh));
        }
        if let Some((t, d)) = term(name) {
            return mk(&format!("definition of {t}"), d.to_string(), vec![]);
        }
        return None;
    }
    if l.contains(' ') && !l.contains(" vs") && !l.contains(" or ") && !l.starts_with("is ") {
        let name = l.trim_start_matches("a ").trim_start_matches("an ").trim_start_matches("the ");
        if let Some(sh) = shape(name).filter(|sh| sh.names[0].contains(' ')) {
            return mk(&format!("definition of {}", sh.names[0]), sh.definition.to_string(), shape_pods(sh));
        }
        if let Some((t, d)) = term(name).filter(|(t, _)| t.contains(' ')) {
            return mk(&format!("definition of {t}"), d.to_string(), vec![]);
        }
    }
    if let Some(m) = re!(r"^(?:the )?(?:types|kinds) of (triangles|quadrilaterals|angles|polygons)$").captures(l) {
        let list = match &m[1] {
            "triangles" => "by sides: equilateral, isosceles, scalene; by angles: acute, right, obtuse",
            "quadrilaterals" => "square, rectangle, rhombus, parallelogram, trapezoid, kite (and general quadrilaterals)",
            "angles" => "acute (< 90°), right (90°), obtuse (90°–180°), straight (180°), reflex (180°–360°), full (360°)",
            _ => "triangle, quadrilateral, pentagon, hexagon, heptagon, octagon, nonagon, decagon, …; regular or irregular, convex or concave",
        };
        return mk(&format!("types of {}", &m[1]), list.to_string(), vec![]);
    }
    if let Some(m) = re!(r"^(?:what is the )?(?:difference between )?(?:an? )?([a-z ]+?) (?:vs\.?|versus|and|compared to) (?:an? )?([a-z ]+?)(?: geometric properties| properties)?$").captures(l) {
        let (a, b) = (shape(&m[1])?, shape(&m[2])?);
        let rows = vec![
            vec![text("definition"), text(a.definition), text(b.definition)],
            vec![text("sides / faces"), text(a.sides.to_string()), text(b.sides.to_string())],
            vec![text("vertices"), text(a.vertices.to_string()), text(b.vertices.to_string())],
        ];
        let pods = vec![pod("Input", "Input", text(format!("{} vs {}", a.names[0], b.names[0]))), primary(pod("Result", "Comparison", table(Some(vec!["", a.names[0], b.names[0]]), rows)))];
        return Some(Answer::new(format!("{} vs {}", a.names[0], b.names[0]), pods));
    }
    if let Some(m) = re!(r"^(\d+|[a-z]+)[- ]degree angle$").captures(l) {
        let d: f64 = m[1].parse().ok()?;
        let kind = if d < 90.0 { "acute angle" } else if d == 90.0 { "right angle" } else if d < 180.0 { "obtuse angle" } else if d == 180.0 { "straight angle" } else if d < 360.0 { "reflex angle" } else { "full turn or more" };
        let rad = s::simplify(&mul(vec![num(exact_rat(&m[1])?), rat(1, 180), pi()]));
        return mk(&format!("{}° angle", &m[1]), kind.to_string(), vec![pod("Radians", "In radians", text(show(&rad))), pod("Complement", "Supplement", text(format!("{}°", n(180.0 - d))))]);
    }
    None
}

fn shape_pods(s: &Shape) -> Vec<Pod> {
    let mut v = Vec::new();
    if s.sides > 0 && s.sides == s.vertices {
        v.push(pod("Props", "Properties", table(None, vec![vec![text("sides"), text(s.sides.to_string())], vec![text("vertices"), text(s.vertices.to_string())], vec![text("sum of interior angles"), text(format!("{}°", (s.sides as i64 - 2) * 180))]])));
    }
    v
}

fn term(name: &str) -> Option<(&'static str, &'static str)> {
    const TERMS: &[(&str, &str)] = &[
        ("acute angle", "an angle smaller than 90°"),
        ("right angle", "an angle of exactly 90°, a quarter turn"),
        ("obtuse angle", "an angle between 90° and 180°"),
        ("straight angle", "an angle of exactly 180°"),
        ("reflex angle", "an angle between 180° and 360°"),
        ("parallel lines", "lines in a plane that never meet; they have the same slope"),
        ("perpendicular lines", "lines that meet at a right angle; their slopes multiply to −1"),
        ("polygon", "a closed plane figure bounded by straight line segments"),
        ("regular polygon", "a polygon whose sides are all equal and whose angles are all equal"),
        ("polyhedron", "a solid bounded by flat polygonal faces"),
        ("vertex", "a point where two or more edges or sides meet"),
        ("hypotenuse", "the side of a right triangle opposite the right angle; the longest side"),
        ("perimeter", "the total length of the boundary of a plane figure"),
        ("area", "the amount of surface a plane figure covers, in square units"),
        ("volume", "the amount of space a solid occupies, in cubic units"),
        ("prime number", "a whole number greater than 1 whose only divisors are 1 and itself"),
        ("composite number", "a whole number greater than 1 that is not prime"),
        ("integer", "a whole number: …, −2, −1, 0, 1, 2, …"),
        ("rational number", "a number that can be written as a fraction p/q of integers with q ≠ 0"),
        ("irrational number", "a real number that cannot be written as a fraction of integers, such as √2 or π"),
    ];
    let n = name.trim_end_matches('s');
    TERMS.iter().find(|(t, _)| *t == name || t.trim_end_matches('s') == n).copied()
}

// ---------------------------------------------------------------------------
// polyhedra
// ---------------------------------------------------------------------------

const POLYHEDRA: &[(&str, u32, u32, u32)] = &[
    // name, vertices, edges, faces
    ("tetrahedron (triangular pyramid)", 4, 6, 4),
    ("square pyramid", 5, 8, 5),
    ("triangular prism", 6, 9, 5),
    ("triangular bipyramid", 5, 9, 6),
    ("cube", 8, 12, 6),
    ("rectangular prism (cuboid)", 8, 12, 6),
    ("octahedron", 6, 12, 8),
    ("pentagonal pyramid", 6, 10, 6),
    ("pentagonal prism", 10, 15, 7),
    ("hexagonal pyramid", 7, 12, 7),
    ("hexagonal prism", 12, 18, 8),
    ("pentagonal bipyramid", 7, 15, 10),
    ("dodecahedron", 20, 30, 12),
    ("icosahedron", 12, 30, 20),
    ("cuboctahedron", 12, 24, 14),
    ("truncated tetrahedron", 12, 18, 8),
    ("truncated octahedron", 24, 36, 14),
    ("truncated icosahedron (soccer ball)", 60, 90, 32),
];

fn polyhedra(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:polyhedra|polyhedrons|solids|shapes) with (\d+|four|five|six|seven|eight|nine|ten|twelve) (edges|faces|vertices)$").captures(l) {
        let k = small_number(&m[1])? as u32;
        let hits: Vec<Vec<Content>> = POLYHEDRA.iter().filter(|p| match &m[2] { "edges" => p.2 == k, "faces" => p.3 == k, _ => p.1 == k }).map(|p| vec![text(p.0), text(p.1.to_string()), text(p.2.to_string()), text(p.3.to_string())]).collect();
        if hits.is_empty() {
            return None;
        }
        let pods = vec![
            pod("Input", "Input", text(format!("polyhedra with {k} {}", &m[2]))),
            primary(pod("Result", "Polyhedra", table(Some(vec!["polyhedron", "vertices", "edges", "faces"]), hits))),
            pod("Euler", "Euler's formula", text("V − E + F = 2 for every convex polyhedron")),
        ];
        return Some(Answer::new(format!("polyhedra with {k} {}", &m[2]), pods));
    }
    let m = re!(r"^how many (edges|faces|vertices|corners|sides) (?:does|do|on|has) (?:a |an )?([a-z ]+?)(?: have)?$").captures(l)?;
    let p = POLYHEDRA.iter().find(|p| p.0.split(" (").next() == Some(m[2].trim()) || p.0.contains(&format!("({})", m[2].trim())))?;
    let v = match &m[1] { "edges" => p.2, "faces" | "sides" => p.3, _ => p.1 };
    let pods = vec![
        pod("Input", "Input", text(format!("{} of a {}", &m[1], &m[2]))),
        primary(pod("Result", "Result", text(v.to_string()))),
        pod("All", "Counts", text(format!("{} vertices, {} edges, {} faces", p.1, p.2, p.3))),
    ];
    Some(Answer::new(format!("{} of {}", &m[1], &m[2]), pods))
}

// ---------------------------------------------------------------------------
// lines
// ---------------------------------------------------------------------------

fn point(s: &str) -> Option<(Expr, Expr)> {
    let t = s.trim().trim_start_matches(['(', '{', '[']).trim_end_matches([')', '}', ']']);
    let (a, b) = t.split_once(',')?;
    Some((value(a)?.0, value(b)?.0))
}

/// Slope and intercept of "y = m x + b" (or just "m x + b").
fn linear(s: &str) -> Option<(Expr, Expr)> {
    let rhs = s.trim().trim_start_matches("y").trim().trim_start_matches('=').trim();
    let e = parse::parse(rhs).ok()?.expr;
    let x = sym("x");
    let d = s::simplify(&crate::calculus::diff::diff(&e, &x));
    if !matches!(d, Expr::Num(_)) {
        return None;
    }
    let b = s::simplify(&e.subs(&x, &int(0)));
    matches!(b, Expr::Num(_)).then_some((d, b))
}

fn line_answer(q: &Query, title: &str, m: Expr, b: Expr, extra: Vec<Pod>) -> Option<Answer> {
    let x = sym("x");
    let rhs = s::simplify(&add(vec![mul(vec![m.clone(), x]), b.clone()]));
    let eq = format!("y = {}", show(&rhs));
    let mut pods = vec![pod("Input", "Input", text(title.to_string())), primary(pod("Result", "Line", text(eq.clone()))), pod("Slope", "Slope and intercept", text(format!("slope {}, y-intercept {}", show(&m), show(&b))))];
    pods.extend(extra);
    if let Some(r) = ask(q, &format!("plot {}", show(&rhs))) {
        pods.extend(r.pods.into_iter().filter(|p| p.id.contains("Plot")).take(1));
    }
    Some(Answer::new(eq, pods))
}

fn lines(q: &Query, l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:(?:the )?equation of (?:a |the )?)?(?:line|linear function) with (?:a )?slope (?:of )?([-\d./]+) and (?:a )?y-?intercept (?:of )?([-\d./]+)$").captures(l) {
        return line_answer(q, &format!("line with slope {} and y-intercept {}", &m[1], &m[2]), value(&m[1])?.0, value(&m[2])?.0, vec![]);
    }
    if let Some(m) = re!(r"^(?:(?:the )?equation of (?:a |the )?)?line (parallel|perpendicular) to (.+?) (?:through|passing through|that passes through|containing) (\(.+\))$").captures(l) {
        let (m0, _) = linear(&m[2])?;
        let (x0, y0) = point(&m[3])?;
        let slope = if &m[1] == "parallel" {
            m0
        } else {
            if matches!(&m0, Expr::Num(r) if r.is_zero()) {
                return None;
            }
            s::simplify(&mul(vec![int(-1), pow(m0, int(-1))]))
        };
        let b = s::simplify(&add(vec![y0, mul(vec![int(-1), slope.clone(), x0])]));
        return line_answer(q, &format!("line {} to {} through {}", &m[1], &m[2], &m[3]), slope, b, vec![]);
    }
    if let Some(m) = re!(r"^perpendicular bisector (?:of|for) (?:the )?(?:line|segment|line segment)?\s*(?:between|from|joining|connecting)?\s*([\(\{\[][^)\}\]]+[\)\}\]])\s*(?:and|to)\s*([\(\{\[][^)\}\]]+[\)\}\]])$").captures(l) {
        let ((x1, y1), (x2, y2)) = (point(&m[1])?, point(&m[2])?);
        let half = rat(1, 2);
        let mx = s::simplify(&mul(vec![half.clone(), add(vec![x1.clone(), x2.clone()])]));
        let my = s::simplify(&mul(vec![half, add(vec![y1.clone(), y2.clone()])]));
        let dx = s::simplify(&add(vec![x2, mul(vec![int(-1), x1])]));
        let dy = s::simplify(&add(vec![y2, mul(vec![int(-1), y1])]));
        if matches!(&dy, Expr::Num(r) if r.is_zero()) {
            let pods = vec![pod("Input", "Input", text(format!("perpendicular bisector of {} and {}", &m[1], &m[2]))), primary(pod("Result", "Line", text(format!("x = {}", show(&mx)))))];
            return Some(Answer::new(format!("x = {}", show(&mx)), pods));
        }
        let slope = s::simplify(&mul(vec![int(-1), dx, pow(dy, int(-1))]));
        let b = s::simplify(&add(vec![my.clone(), mul(vec![int(-1), slope.clone(), mx.clone()])]));
        let mid = pod("Mid", "Midpoint", text(format!("({}, {})", show(&mx), show(&my))));
        return line_answer(q, &format!("perpendicular bisector of {} and {}", &m[1], &m[2]), slope, b, vec![mid]);
    }
    if let Some(m) = re!(r"^exponential (?:function|curve|model) (?:through|passing through|containing) (\(.+?\)),? (?:and )?(\(.+?\))$").captures(l) {
        let ((x1, y1), (x2, y2)) = (point(&m[1])?, point(&m[2])?);
        let (x1f, y1f, x2f, y2f) = (crate::eval::eval_f64(&x1, &[])?, crate::eval::eval_f64(&y1, &[])?, crate::eval::eval_f64(&x2, &[])?, crate::eval::eval_f64(&y2, &[])?);
        if x1f == x2f || y1f * y2f <= 0.0 {
            return None;
        }
        // y = a b^x
        let b = s::simplify(&pow(mul(vec![y2.clone(), pow(y1.clone(), int(-1))]), pow(add(vec![x2, mul(vec![int(-1), x1.clone()])]), int(-1))));
        let a = s::simplify(&mul(vec![y1, pow(b.clone(), mul(vec![int(-1), x1]))]));
        let f = s::simplify(&mul(vec![a.clone(), pow(b.clone(), sym("x"))]));
        let _ = (x1f, y1f, x2f, y2f);
        let mut pods = vec![pod("Input", "Input", text(format!("exponential function through {} and {}", &m[1], &m[2]))), primary(pod("Result", "Function", text(format!("y = {}", show(&f))))), pod("Params", "Parameters", text(format!("a = {}, b = {}  (y = a b^x)", show(&a), show(&b))))];
        if let Some(r) = ask(q, &format!("plot {}", show(&f))) {
            pods.extend(r.pods.into_iter().filter(|p| p.id.contains("Plot")).take(1));
        }
        return Some(Answer::new(format!("y = {}", show(&f)), pods));
    }
    None
}

/// "where is f(x) = 3x^2 + 9x increasing"
fn monotonic(q: &Query, l: &str) -> Option<Answer> {
    let (f, dir) = if let Some(m) = re!(r"^(?:where|when|on what intervals?|for what values of x|over what intervals?) (?:is|does) (?:the function )?(?:f\(x\)\s*=\s*|y\s*=\s*)?(.+?) (increasing|decreasing|increase|decrease)$").captures(l) {
        (m[1].to_string(), m[2].starts_with("inc"))
    } else if let Some(m) = re!(r"^(increasing|decreasing) (?:intervals?|interval of|intervals of)?\s*(?:of|for)?\s*(?:f\(x\)\s*=\s*|y\s*=\s*)?(.+)$").captures(l) {
        (m[2].to_string(), m[1].starts_with("inc"))
    } else {
        return None;
    };
    let e = parse::parse(&f).ok()?.expr;
    let d = s::simplify(&crate::calculus::diff::diff(&e, &sym("x")));
    let cmp = if dir { ">" } else { "<" };
    let r = ask(q, &format!("solve {} {cmp} 0", show(&d)))?;
    let sol = r.pods.iter().find(|p| p.id == "Solution" || p.primary)?.clone();
    let mut pods = vec![
        pod("Input", "Input", text(format!("where {} is {}", show(&e), if dir { "increasing" } else { "decreasing" }))),
        primary(Pod { id: "Result".into(), title: if dir { "Increasing on".into() } else { "Decreasing on".into() }, subpods: sol.subpods, primary: true }),
        pod("Derivative", "Derivative", text(format!("f'(x) = {}  (f is {} where f'(x) {cmp} 0)", show(&d), if dir { "increasing" } else { "decreasing" }))),
    ];
    if let Some(r) = ask(q, &format!("plot {}", show(&e))) {
        pods.extend(r.pods.into_iter().filter(|p| p.id.contains("Plot")).take(1));
    }
    Some(Answer::new(format!("where {} is {}", show(&e), if dir { "increasing" } else { "decreasing" }), pods))
}

// ---------------------------------------------------------------------------
// probability rules: "P(A or B) if P(A) = 3/4, P(B) = 1/5, P(A and B) = 0"
// ---------------------------------------------------------------------------

fn event_key(s: &str) -> Option<&'static str> {
    let t = s.trim().replace(' ', "").replace('∩', "and").replace('∪', "or").replace('|', "given").replace('^', "");
    Some(match t.as_str() {
        "a" => "A",
        "b" => "B",
        "aandb" | "banda" | "ab" => "AB",
        "aorb" | "bora" => "AorB",
        "agivenb" => "A|B",
        "bgivena" => "B|A",
        "nota" | "a'" | "ac" | "a^c" => "notA",
        "notb" | "b'" | "bc" | "b^c" => "notB",
        _ => return None,
    })
}

fn probability_rules(l: &str) -> Option<Answer> {
    if !l.contains("p(") {
        return None;
    }
    let mut known: std::collections::BTreeMap<&str, Rat> = Default::default();
    for m in re!(r"p\(([^)]+)\)\s*=\s*([\d./]+)").captures_iter(l) {
        known.insert(event_key(&m[1])?, exact_rat(&m[2])?);
    }
    let target = re!(r"(?:what is |find |^)p\(([^)]+)\)(?:\s*(?:if|when|given|,|$))").captures(l).and_then(|m| event_key(&m[1]))?;
    if known.contains_key(target) || known.is_empty() {
        return None;
    }
    let independent = l.contains("independent");
    let one = Rat::from_integer(1.into());
    for _ in 0..6 {
        let g = |k: &str, kn: &std::collections::BTreeMap<&str, Rat>| kn.get(k).cloned();
        let mut new: Vec<(&str, Rat)> = Vec::new();
        let (a, b, ab, aorb, agb, bga, na, nb) = (g("A", &known), g("B", &known), g("AB", &known), g("AorB", &known), g("A|B", &known), g("B|A", &known), g("notA", &known), g("notB", &known));
        if let Some(na) = &na {
            new.push(("A", &one - na));
        }
        if let Some(nb) = &nb {
            new.push(("B", &one - nb));
        }
        if let Some(a) = &a {
            new.push(("notA", &one - a));
        }
        if let Some(b) = &b {
            new.push(("notB", &one - b));
        }
        if independent {
            if let (Some(a), Some(b)) = (&a, &b) {
                new.push(("AB", a * b));
            }
        }
        match (&a, &b, &ab, &aorb) {
            (Some(a), Some(b), Some(ab), None) => new.push(("AorB", a + b - ab)),
            (Some(a), Some(b), None, Some(o)) => new.push(("AB", a + b - o)),
            (Some(a), None, Some(ab), Some(o)) => new.push(("B", o - a + ab)),
            (None, Some(b), Some(ab), Some(o)) => new.push(("A", o - b + ab)),
            _ => {}
        }
        if let (Some(ab), Some(b)) = (&ab, &b) {
            if !b.is_zero() {
                new.push(("A|B", ab / b));
            }
        }
        if let (Some(ab), Some(a)) = (&ab, &a) {
            if !a.is_zero() {
                new.push(("B|A", ab / a));
            }
        }
        if let (Some(agb), Some(b)) = (&agb, &b) {
            new.push(("AB", agb * b));
        }
        if let (Some(bga), Some(a)) = (&bga, &a) {
            new.push(("AB", bga * a));
        }
        if let (Some(agb), Some(ab)) = (&agb, &ab) {
            if !agb.is_zero() {
                new.push(("B", ab / agb));
            }
        }
        if let (Some(bga), Some(ab)) = (&bga, &ab) {
            if !bga.is_zero() {
                new.push(("A", ab / bga));
            }
        }
        let mut changed = false;
        for (k, v) in new {
            if !known.contains_key(k) {
                known.insert(k, v);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let v = known.get(target)?.clone();
    if v.is_negative() || v > one {
        return None;
    }
    let label = |k: &str| match k {
        "AB" => "P(A ∩ B)".to_string(),
        "AorB" => "P(A ∪ B)".to_string(),
        "notA" => "P(not A)".to_string(),
        "notB" => "P(not B)".to_string(),
        k => format!("P({k})"),
    };
    let rule = match target {
        "AorB" => "P(A ∪ B) = P(A) + P(B) − P(A ∩ B)",
        "AB" if independent => "P(A ∩ B) = P(A) P(B) for independent events",
        "AB" => "P(A ∪ B) = P(A) + P(B) − P(A ∩ B), P(A ∩ B) = P(A | B) P(B)",
        "A|B" => "P(A | B) = P(A ∩ B) / P(B)",
        "B|A" => "P(B | A) = P(A ∩ B) / P(A)",
        "notA" | "notB" => "P(not A) = 1 − P(A)",
        _ => "P(A ∪ B) = P(A) + P(B) − P(A ∩ B)",
    };
    let dec = v.to_f64().unwrap_or(f64::NAN);
    let rows = known.iter().map(|(k, v)| vec![text(label(k)), text(v.to_string())]).collect();
    let pods = vec![
        pod("Input", "Input", text(format!("{} given the probabilities", label(target)))),
        primary(pod("Result", "Result", text(format!("{} = {}{}", label(target), v, if v.is_integer() { String::new() } else { format!(" ≈ {}", n(dec)) })))),
        pod("Rule", "Rule used", text(rule)),
        pod("All", "All probabilities", table(Some(vec!["event", "probability"]), rows)),
    ];
    Some(Answer::new(format!("{} = {v}", label(target)), pods))
}

fn odds(l: &str) -> Option<Answer> {
    let m = re!(r"^(\d+(?:\.\d+)?)\s*(?::|to|/|-)\s*(\d+(?:\.\d+)?) odds,?\s*(?:bet|wager|stake) \$?([\d.,]+)$").captures(l)?;
    let (a, b, bet): (f64, f64, f64) = (m[1].parse().ok()?, m[2].parse().ok()?, m[3].replace(',', "").parse().ok()?);
    if b == 0.0 {
        return None;
    }
    let profit = bet * a / b;
    let pods = vec![
        pod("Input", "Input", text(format!("{}:{} odds against, ${bet:.2} bet", &m[1], &m[2]))),
        primary(pod("Result", "Profit if you win", text(format!("${profit:.2}")))),
        pod("Total", "Total returned", text(format!("${:.2}", profit + bet))),
        pod("Implied", "Implied probability of winning", text(format!("{}/{} ≈ {}%", &m[2], fmt_float(a + b, 6), n(100.0 * b / (a + b))))),
    ];
    Some(Answer::new(format!("{}:{} odds, ${bet} bet", &m[1], &m[2]), pods))
}

// ---------------------------------------------------------------------------
// conics
// ---------------------------------------------------------------------------

fn parabola(q: &Query, l: &str) -> Option<Answer> {
    if !l.contains("parabola") || !l.contains("focus") || !l.contains("directrix") {
        return None;
    }
    let f = re!(r"focus\s*(?:at|of|is)?\s*(\([^)]+\))").captures(l)?;
    let d = re!(r"directrix\s*(?:at|of|is)?\s*([xy])\s*=\s*([-\d./]+)").captures(l)?;
    let (fx, fy) = point(&f[1])?;
    let dv = value(&d[2])?.0;
    let half = rat(1, 2);
    let (along, across, var_a, var_b) = if &d[1] == "y" { (fy, fx, "y", "x") } else { (fx, fy, "x", "y") };
    // vertex halfway between the focus and the directrix; p = focal distance
    let v = s::simplify(&mul(vec![half.clone(), add(vec![along.clone(), dv.clone()])]));
    let p = s::simplify(&mul(vec![half, add(vec![along, mul(vec![int(-1), dv])])]));
    if matches!(&p, Expr::Num(r) if r.is_zero()) {
        return None;
    }
    // (b - h)^2 = 4 p (a - v)  →  a = (b - h)^2 / (4 p) + v
    let b = sym(var_b);
    let explicit = s::simplify(&add(vec![mul(vec![pow(add(vec![b.clone(), mul(vec![int(-1), across.clone()])]), int(2)), pow(mul(vec![int(4), p.clone()]), int(-1))]), v.clone()]));
    let expanded = s::simplify(&s::expand(&explicit));
    let std_form = format!("({var_b} − {})² = {} ({var_a} − {})", show(&across), show(&s::simplify(&mul(vec![int(4), p.clone()]))), show(&v));
    let vertex = if var_a == "y" { format!("({}, {})", show(&across), show(&v)) } else { format!("({}, {})", show(&v), show(&across)) };
    let mut pods = vec![
        pod("Input", "Input", text(format!("parabola with focus {} and directrix {} = {}", &f[1], &d[1], &d[2]))),
        primary(pod("Result", "Equation", text(format!("{var_a} = {}", show(&expanded))))),
        pod("Standard", "Standard form", text(std_form)),
        pod("Props", "Properties", table(None, vec![vec![text("vertex"), text(vertex)], vec![text("focal length"), text(show(&s::simplify(&func("abs", vec![p.clone()]))))], vec![text("axis of symmetry"), text(format!("{var_b} = {}", show(&across)))]])),
    ];
    if var_a == "y" {
        if let Some(r) = ask(q, &format!("plot {}", show(&expanded))) {
            pods.extend(r.pods.into_iter().filter(|p| p.id.contains("Plot")).take(1));
        }
    }
    Some(Answer::new(format!("{var_a} = {}", show(&expanded)), pods))
}

fn ellipse(l: &str) -> Option<Answer> {
    let m = re!(r"^ellipse with foci (?:at )?(\([^)]+\)) and (\([^)]+\)),? and (?:a )?(major|minor) axis(?: length)?(?: of)? ([\d./]+)$").captures(l)?;
    let ((x1, y1), (x2, y2)) = (point(&m[1])?, point(&m[2])?);
    let len = value(&m[4])?.0;
    let half = rat(1, 2);
    let h = s::simplify(&mul(vec![half.clone(), add(vec![x1.clone(), x2.clone()])]));
    let k = s::simplify(&mul(vec![half.clone(), add(vec![y1.clone(), y2.clone()])]));
    let c2 = s::simplify(&mul(vec![rat(1, 4), add(vec![pow(add(vec![x2.clone(), mul(vec![int(-1), x1.clone()])]), int(2)), pow(add(vec![y2.clone(), mul(vec![int(-1), y1.clone()])]), int(2))])]));
    let given2 = s::simplify(&pow(mul(vec![half, len]), int(2)));
    let (a2, b2) = if &m[3] == "major" { (given2.clone(), s::simplify(&add(vec![given2, mul(vec![int(-1), c2.clone()])]))) } else { (s::simplify(&add(vec![given2.clone(), c2.clone()])), given2) };
    if crate::eval::eval_f64(&b2, &[])? <= 0.0 {
        return None;
    }
    let horizontal = s::simplify(&add(vec![y2, mul(vec![int(-1), y1])])) == int(0);
    let vertical = s::simplify(&add(vec![x2, mul(vec![int(-1), x1])])) == int(0);
    if !horizontal && !vertical {
        return None;
    }
    let sq = |v: &str, c: &Expr| if matches!(c, Expr::Num(r) if r.is_zero()) { format!("{v}²") } else { format!("({v} − {})²", show(c)) };
    let (dx, dy) = if horizontal { (a2.clone(), b2.clone()) } else { (b2.clone(), a2.clone()) };
    let den = |e: &Expr| if matches!(e, Expr::Num(r) if r.is_integer()) { show(e) } else { format!("({})", show(e)) };
    let eq = format!("{}/{} + {}/{} = 1", sq("x", &h), den(&dx), sq("y", &k), den(&dy));
    let ecc = s::simplify(&pow(mul(vec![c2.clone(), pow(a2.clone(), int(-1))]), rat(1, 2)));
    let pods = vec![
        pod("Input", "Input", text(format!("ellipse with foci {} and {}, {} axis {}", &m[1], &m[2], &m[3], &m[4]))),
        primary(pod("Result", "Equation", text(eq.clone()))),
        pod("Props", "Properties", table(None, vec![
            vec![text("center"), text(format!("({}, {})", show(&h), show(&k)))],
            vec![text("semi-major axis a"), text(show(&s::simplify(&pow(a2, rat(1, 2)))))],
            vec![text("semi-minor axis b"), text(show(&s::simplify(&pow(b2, rat(1, 2)))))],
            vec![text("focal distance c"), text(show(&s::simplify(&pow(c2, rat(1, 2)))))],
            vec![text("eccentricity"), text(show(&ecc))],
        ])),
    ];
    Some(Answer::new(eq, pods))
}

// ---------------------------------------------------------------------------
// rational or irrational, ordering
// ---------------------------------------------------------------------------

fn rationality(l: &str) -> Option<Answer> {
    let m = re!(r"^is (?:the )?(.+?) (?:a )?(rational|irrational)(?: number)?$").captures(l)?;
    let src = m[1].replace("square root of ", "sqrt ").replace("cube root of ", "cbrt ");
    let src = src.trim();
    let src = if let Some(r) = src.strip_prefix("sqrt ") { format!("sqrt({r})") } else if let Some(r) = src.strip_prefix("cbrt ") { format!("({r})^(1/3)") } else { src.to_string() };
    let e = s::simplify(&parse::parse(&src).ok()?.expr);
    fn irrational(e: &Expr) -> Option<bool> {
        match e {
            Expr::Num(_) => Some(false),
            Expr::Const(K::Pi) | Expr::Const(K::E) => Some(true),
            Expr::Pow(b, x) => match (b.as_ref(), x.as_ref()) {
                // simplification has already taken out perfect powers
                (Expr::Num(b), Expr::Num(x)) if !x.is_integer() && b.is_positive() => Some(true),
                _ => None,
            },
            Expr::Mul(v) => {
                let mut irr = 0;
                for f in v {
                    match f {
                        Expr::Num(r) if !r.is_zero() => {}
                        other => {
                            if irrational(other)? {
                                irr += 1;
                            } else {
                                return None;
                            }
                        }
                    }
                }
                (irr == 1).then_some(true)
            }
            Expr::Add(v) => {
                let irr = v.iter().filter(|t| !matches!(t, Expr::Num(_))).count();
                (irr == 1 && v.iter().all(|t| matches!(t, Expr::Num(_)) || irrational(t) == Some(true))).then_some(true)
            }
            _ => None,
        }
    }
    let irr = irrational(&e)?;
    let asked_rational = &m[2] == "rational";
    let yes = asked_rational != irr;
    let why = if irr { format!("{} is irrational: it cannot be written as a fraction of integers.", show(&e)) } else { format!("{} is rational.", show(&e)) };
    let mut pods = vec![pod("Input", "Input", text(format!("is {} {}?", show(&e), &m[2]))), primary(pod("Result", "Result", text(if yes { "yes" } else { "no" }))), pod("Why", "Explanation", text(why))];
    if let Some(v) = crate::eval::eval_f64(&e, &[]) {
        pods.push(pod("Decimal", "Decimal approximation", text(fmt_float(v, 15))));
    }
    Some(Answer::new(format!("is {} {}", show(&e), &m[2]), pods))
}

fn order(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:order|sort|arrange|put)\s+(.+?)(?:\s+(?:from|in) (least to greatest|smallest to largest|greatest to least|largest to smallest|ascending order|descending order|order))?$").captures(l)?;
    let items: Vec<&str> = m[1].split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    if items.len() < 2 {
        return None;
    }
    let mut vals: Vec<(String, f64)> = items.iter().map(|s| value(s).map(|(_, v)| (s.to_string(), v))).collect::<Option<_>>()?;
    let desc = m.get(2).is_some_and(|x| x.as_str().starts_with("greatest") || x.as_str().starts_with("largest") || x.as_str().starts_with("desc"));
    vals.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    if desc {
        vals.reverse();
    }
    let mut chain = vals[0].0.clone();
    for w in vals.windows(2) {
        let rel = if (w[0].1 - w[1].1).abs() < 1e-12 { "=" } else if desc { ">" } else { "<" };
        chain.push_str(&format!(" {rel} {}", w[1].0));
    }
    let rows = vals.iter().map(|(s, v)| vec![text(s.clone()), text(n(*v))]).collect();
    let pods = vec![
        pod("Input", "Input", text(format!("sort {}", items.join(", ")))),
        primary(pod("Result", if desc { "Greatest to least" } else { "Least to greatest" }, text(chain))),
        pod("Values", "Decimal values", table(Some(vec!["number", "value"]), rows)),
    ];
    Some(Answer::new(format!("sort {}", items.join(", ")), pods))
}

// ---------------------------------------------------------------------------
// polar and Cartesian coordinates, arcs
// ---------------------------------------------------------------------------

fn polar(q: &Query, l: &str) -> Option<Answer> {
    // complex number to polar form
    if let Some(m) = re!(r"^(?:convert|express|write|put)?\s*(.+?i)\s+(?:to|in|into|as)\s+(?:polar|trigonometric|exponential)(?: coordinates| form)?$").captures(l) {
        let r = ask(q, m[1].trim())?;
        let mut pods: Vec<Pod> = r.pods.into_iter().filter(|p| p.id != "Result").collect();
        let pos = pods.iter().position(|p| p.title.starts_with("Polar"))?;
        pods[pos].primary = true;
        for (i, p) in pods.iter_mut().enumerate() {
            if i != pos {
                p.primary = false;
            }
        }
        return Some(Answer::new(format!("{} in polar form", &m[1]), pods));
    }
    // a point to polar
    if let Some(m) = re!(r"^(?:convert|express|write)?\s*(\([^)]+\))\s+(?:to|in|into)\s+polar(?: coordinates| form)?$").captures(l) {
        let (x, y) = point(&m[1])?;
        let r = s::simplify(&pow(add(vec![pow(x.clone(), int(2)), pow(y.clone(), int(2))]), rat(1, 2)));
        let (xf, yf) = (crate::eval::eval_f64(&x, &[])?, crate::eval::eval_f64(&y, &[])?);
        let th = yf.atan2(xf);
        let pods = vec![
            pod("Input", "Input", text(format!("{} to polar coordinates", &m[1]))),
            primary(pod("Result", "Polar coordinates", text(format!("r = {}, θ = {} rad = {}°", show(&r), n(th), n(th.to_degrees()))))),
        ];
        return Some(Answer::new(format!("{} in polar coordinates", &m[1]), pods));
    }
    // polar to Cartesian: "(-1.5, 200 degrees)"
    let m = re!(r"^(?:convert|express|write)?\s*\(\s*([-\d./]+)\s*,\s*([-\d./]+(?:\s*pi)?)\s*(degrees|degree|deg|°|radians|rad)?\s*\)\s+(?:to|in|into)\s+(?:cartesian|rectangular|xy)(?: coordinates| form)?$").captures(l)?;
    let r = value(&m[1])?.0;
    let ang = value(&m[2])?.0;
    let rad = if m.get(3).is_some_and(|u| u.as_str().starts_with('d') || u.as_str() == "°") { s::simplify(&mul(vec![ang, pi(), rat(1, 180)])) } else { ang };
    let x = s::simplify(&mul(vec![r.clone(), func("cos", vec![rad.clone()])]));
    let y = s::simplify(&mul(vec![r, func("sin", vec![rad])]));
    let (xf, yf) = (crate::eval::eval_f64(&x, &[])?, crate::eval::eval_f64(&y, &[])?);
    let pods = vec![
        pod("Input", "Input", text(format!("polar ({}, {}{}) to Cartesian", &m[1], &m[2], m.get(3).map_or(String::new(), |u| format!(" {}", u.as_str()))))),
        primary(pod("Result", "Cartesian coordinates", text(format!("(x, y) ≈ ({}, {})", n(xf), n(yf))))),
        pod("Exact", "Exact form", text(format!("({}, {})", show(&x), show(&y)))),
    ];
    Some(Answer::new("polar to Cartesian".to_string(), pods))
}

fn arc(l: &str) -> Option<Answer> {
    let m = re!(r"^(arc length|length of (?:an |the )?arc|sector area|area of (?:a |the )?sector)(?: for| of| with)?\s+(?:r|radius)\s*=?\s*([\d./]+),? (?:and |with )?(?:angle |theta |θ )?=?\s*([\d./]+(?:\s*pi)?)\s*(degrees|degree|deg|°|radians|rad)?$").captures(l)?;
    let r = value(&m[2])?.0;
    let ang = value(&m[3])?.0;
    let degrees = m.get(4).map_or(true, |u| u.as_str().starts_with('d') || u.as_str() == "°") && !m[3].contains("pi");
    let rad = if degrees { s::simplify(&mul(vec![ang, pi(), rat(1, 180)])) } else { ang };
    let arc_len = m[1].contains("arc");
    let res = if arc_len { s::simplify(&mul(vec![r, rad.clone()])) } else { s::simplify(&mul(vec![rat(1, 2), pow(r, int(2)), rad.clone()])) };
    let v = crate::eval::eval_f64(&res, &[])?;
    let pods = vec![
        pod("Input", "Input", text(format!("{} for r = {}, θ = {}", if arc_len { "arc length" } else { "sector area" }, &m[2], show(&rad)))),
        primary(pod("Result", "Result", text(format!("{} ≈ {}", show(&res), n(v))))),
        pod("Formula", "Formula", text(if arc_len { "s = r θ (θ in radians)" } else { "A = r² θ / 2 (θ in radians)" })),
    ];
    Some(Answer::new(if arc_len { "arc length" } else { "sector area" }, pods))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first(a: Answer) -> String {
        content_text(&a.pods.iter().find(|p| p.primary).unwrap().subpods[0].content)
    }

    #[test]
    fn school() {
        assert_eq!(first(compare("which is greater, 7/8 or 3/4").unwrap()), "7/8 is greater");
        assert!(first(compare("is 162 greater than 126").unwrap()).starts_with("yes"));
        assert!(first(place_value("place value of the digit 3 in 537").unwrap()).starts_with("tens"));
        assert_eq!(first(count("count from 0 to 100 by ten").unwrap()), "0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100");
        assert!(first(coins("how much is 3 dimes and 4 pennies").unwrap()).starts_with("$0.34"));
        assert_eq!(first(clock("4:45pm to 7:15pm").unwrap()), "2 hours 30 minutes");
        assert!(first(clock("half past noon").unwrap()).starts_with("12:30 PM"));
        assert_eq!(first(rate("5 miles per 1/8 of a gallon").unwrap()), "40 miles per gallon");
        assert!(first(probability_rules("p(a) = 3/4, p(b) = 1/5, p(a and b) = 0, what is p(a or b)").unwrap()).contains("19/20"));
        assert!(first(probability_rules("p(a given b) if p(a) = 1/2, p(b) = 1/3, p(a and b) = 1/4").unwrap()).contains("3/4"));
        assert!(first(odds("8:5 odds, bet $10").unwrap()).starts_with("$16.00"));
        assert_eq!(first(shapes("shape with 5 sides").unwrap()), "pentagon");
        assert_eq!(first(shapes("is a square a rectangle").unwrap()), "yes");
        assert_eq!(first(shapes("is a trapezoid a parallelogram").unwrap()), "no");
        assert_eq!(first(rationality("is the square root of 2 rational").unwrap()), "no");
        assert!(first(order("order -1, 1/2, |0.5|, sqrt(9), -3.2").unwrap()).starts_with("-3.2 < -1 < 1/2 = |0.5| < sqrt(9)"));
        assert!(first(arc("arc length for r = 3 and 120 degrees").unwrap()).starts_with("2 π"));
        assert!(first(ellipse("ellipse with foci at (-2, 5) and (9, 5) and major axis 16").unwrap()).contains("/64"));
    }
}
