//! Numbers written in other bases and figurate numbers: arithmetic on base
//! literals ("10101 base 2 + 110 base 2", "1021_4 - 113_4", "2^^101101",
//! "0xff + 1", "675 & 0x00ff"), comparisons ("is 123 base 5 the same as 46
//! base 8?"), conversions of fractions and to large bases ("convert 0.25 to
//! ternary", "convert 815 to base 200"), and figurate numbers ("10th
//! triangular number", "1250th decagonal number", "is 56 a tetrahedral
//! number?").

use super::*;
use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn base_name(s: &str) -> Option<u32> {
    Some(match s.trim() {
        "binary" => 2,
        "ternary" | "trinary" => 3,
        "quaternary" => 4,
        "quinary" => 5,
        "senary" | "heximal" => 6,
        "septenary" => 7,
        "octal" => 8,
        "nonary" => 9,
        "decimal" => 10,
        "undecimal" => 11,
        "duodecimal" | "dozenal" => 12,
        "hexadecimal" | "hex" => 16,
        "vigesimal" => 20,
        "sexagesimal" => 60,
        s => {
            let t = s.trim_start_matches("base").trim();
            t.parse().ok().filter(|b| (2..=1000).contains(b))?
        }
    })
}

fn parse_digits(d: &str, b: u32) -> Option<BigInt> {
    if b > 36 || d.is_empty() {
        return None;
    }
    let mut v = BigInt::zero();
    for c in d.chars() {
        let x = c.to_digit(36)?;
        if x >= b {
            return None;
        }
        v = v * b + x;
    }
    Some(v)
}

fn digits_of(n: &BigInt, b: u32) -> Vec<u32> {
    if n.is_zero() {
        return vec![0];
    }
    let mut v = Vec::new();
    let mut m = n.abs();
    let bb = BigInt::from(b);
    while !m.is_zero() {
        let (q, r) = m.div_rem(&bb);
        v.push(r.to_u32().unwrap_or(0));
        m = q;
    }
    v.reverse();
    v
}

fn show_in_base(n: &BigInt, b: u32) -> String {
    let d = digits_of(n, b);
    let sign = if n.is_negative() { "-" } else { "" };
    if b <= 36 {
        format!("{sign}{}", d.iter().map(|x| std::char::from_digit(*x, 36).unwrap().to_ascii_lowercase()).collect::<String>())
    } else {
        format!("{sign}{{{}}}", d.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
    }
}

/// Expansion of a non-negative rational in base b, with the repeating block in parentheses.
fn frac_in_base(r: &BigRational, b: u32) -> String {
    let int = r.numer().div_floor(r.denom());
    let mut rem = r.numer() - &int * r.denom();
    let den = r.denom().clone();
    let mut digits = Vec::new();
    let mut seen: Vec<BigInt> = Vec::new();
    let mut start = None;
    for _ in 0..200 {
        if rem.is_zero() {
            break;
        }
        if let Some(p) = seen.iter().position(|x| *x == rem) {
            start = Some(p);
            break;
        }
        seen.push(rem.clone());
        rem *= b;
        let (q, r2) = rem.div_rem(&den);
        digits.push(q.to_u32().unwrap_or(0));
        rem = r2;
    }
    let ch = |x: &u32| if b <= 36 { std::char::from_digit(*x, 36).unwrap().to_string() } else { format!("[{x}]") };
    let ip = show_in_base(&int, b);
    if digits.is_empty() {
        return ip;
    }
    match start {
        Some(p) => format!("{ip}.{}({})", digits[..p].iter().map(ch).collect::<String>(), digits[p..].iter().map(ch).collect::<String>()),
        None => format!("{ip}.{}{}", digits.iter().map(ch).collect::<String>(), if seen.len() >= 200 { "…" } else { "" }),
    }
}

fn base_literals(l: &str) -> Option<Answer> {
    let t = l.trim_start_matches("what is ").trim_start_matches("compute ").trim_start_matches("calculate ").to_string();
    let t = t.replace(" the same as ", " == ").replace(" equal to ", " == ").replace(" equals ", " == ");
    let t = re!(r"^(?:is|are|does)\s+").replace(&t, "").to_string();
    let mut bases: Vec<u32> = Vec::new();
    let mut any = false;
    let mut fail = false;
    let mut sub = |re: &Regex, s: &str, f: &dyn Fn(&regex::Captures) -> Option<(BigInt, u32)>| -> String {
        re.replace_all(s, |c: &regex::Captures| match f(c) {
            Some((v, b)) => {
                any = true;
                bases.push(b);
                format!("({v})")
            }
            None => {
                fail = true;
                c[0].to_string()
            }
        }).to_string()
    };
    let prefix_only = !re!(r"\bbase\b|_\(?\d|\^\^").is_match(&t);
    let mut s = t.clone();
    s = sub(re!(r"\(?\b([0-9a-z]+)\s+(?:in\s+)?base\s*(\d+)\)?"), &s, &|c| {
        let b: u32 = c[2].parse().ok()?;
        Some((parse_digits(&c[1], b)?, b))
    });
    s = sub(re!(r"\b([0-9a-z]+)_\(?(\d+)\)?"), &s, &|c| {
        // a single letter with a subscript is a variable, not a numeral
        if c[1].len() == 1 && c[1].chars().all(|ch| ch.is_ascii_alphabetic()) {
            return None;
        }
        let b: u32 = c[2].parse().ok()?;
        Some((parse_digits(&c[1], b)?, b))
    });
    s = sub(re!(r"\b(\d+)\s*\^\^\s*([0-9a-z]+)"), &s, &|c| {
        let b: u32 = c[1].parse().ok()?;
        Some((parse_digits(&c[2], b)?, b))
    });
    s = sub(re!(r"\b0([xbo])([0-9a-f]+)\b"), &s, &|c| {
        let b = match &c[1] {
            "x" => 16,
            "b" => 2,
            _ => 8,
        };
        Some((parse_digits(&c[2], b)?, b))
    });
    if !any || fail {
        return None;
    }
    // what is left must be arithmetic on the decimal values
    if !re!(r"^[\d\s()+\-*/^&|=<>!%]+(?:xor|and|or|mod)?[\d\s()+\-*/^&|=<>!%]*$").is_match(&s) {
        return None;
    }
    let common = bases.iter().all(|b| *b == bases[0]).then(|| bases[0]);
    let out_base = common.unwrap_or(bases[0]);
    // comparisons
    if let Some(m) = re!(r"^\s*(.+?)\s*(==|!=|<=|>=|<|>|=)\s*(.+?)\s*$").captures(&s) {
        let ev = |e: &str| -> Option<BigRational> { eval_rat(e) };
        let (a, b) = (ev(&m[1])?, ev(&m[3])?);
        let res = match &m[2] {
            "==" | "=" => a == b,
            "!=" => a != b,
            "<" => a < b,
            ">" => a > b,
            "<=" => a <= b,
            _ => a >= b,
        };
        return Some(Answer::new("base comparison", vec![
            pod("Input", "Input interpretation", text(format!("{} {} {}", show_rat(&a), &m[2], show_rat(&b)))),
            primary(pod("Result", "Result", text(if res { "True" } else { "False" }))),
            pod("Decimal", "Decimal values", text(format!("{} and {}", show_rat(&a), show_rat(&b)))),
        ]));
    }
    // bitwise operations
    if let Some(m) = re!(r"^\s*\(?(\d+)\)?\s*(&|\||xor|and|or)\s*\(?(\d+)\)?\s*$").captures(&s) {
        let (a, b): (BigInt, BigInt) = (m[1].parse().ok()?, m[3].parse().ok()?);
        let (name, v) = match &m[2] {
            "&" | "and" => ("bitwise AND", &a & &b),
            "|" | "or" => ("bitwise OR", &a | &b),
            _ => ("bitwise XOR", &a ^ &b),
        };
        return Some(Answer::new("bitwise", vec![
            pod("Input", "Input interpretation", text(format!("{name} of {a} and {b}"))),
            primary(pod("Result", "Result", text(v.to_string()))),
            pod("Bases", "In other bases", table(None, vec![vec![text("binary"), text(show_in_base(&v, 2))], vec![text("hexadecimal"), text(show_in_base(&v, 16))], vec![text("operands in binary"), text(format!("{} {} {}", show_in_base(&a, 2), &m[2], show_in_base(&b, 2)))]])),
        ]));
    }
    let v = eval_rat(&s)?;
    let in_base = if v.is_integer() { show_in_base(v.numer(), out_base) } else { format!("{}/{}", show_in_base(v.numer(), out_base), show_in_base(v.denom(), out_base)) };
    let mut pods = vec![pod("Input", "Input interpretation", text(s.trim().to_string()))];
    if prefix_only {
        pods.push(primary(pod("Decimal", "Result", text(show_rat(&v)))));
        pods.push(pod("Result", &format!("Result in base {out_base}"), text(format!("{in_base}_{out_base}"))));
    } else {
        pods.push(primary(pod("Result", &format!("Result in base {out_base}"), text(format!("{in_base}_{out_base}")))));
        pods.push(pod("Decimal", "Decimal form", text(show_rat(&v))));
    }
    if common.is_none() {
        pods.push(pod("Note", "Note", text(format!("the operands are in different bases; the result is shown in base {out_base}, the base of the first operand"))));
    }
    Some(Answer::new("base arithmetic", pods))
}

fn show_rat(r: &BigRational) -> String {
    if r.is_integer() { r.numer().to_string() } else { format!("{}/{}", r.numer(), r.denom()) }
}

/// Exact rational arithmetic for + − × ÷ ^ and parentheses.
fn eval_rat(s: &str) -> Option<BigRational> {
    let toks: Vec<String> = re!(r"\d+|[()+\-*/^%]|mod").find_iter(s).map(|m| m.as_str().to_string()).collect();
    let joined: String = toks.concat();
    if joined.len() != s.chars().filter(|c| !c.is_whitespace()).count() {
        return None;
    }
    let mut pos = 0;
    let v = expr(&toks, &mut pos)?;
    (pos == toks.len()).then_some(v)
}

fn expr(t: &[String], p: &mut usize) -> Option<BigRational> {
    let mut v = term(t, p)?;
    while *p < t.len() && (t[*p] == "+" || t[*p] == "-") {
        let op = t[*p].clone();
        *p += 1;
        let r = term(t, p)?;
        v = if op == "+" { v + r } else { v - r };
    }
    Some(v)
}

fn term(t: &[String], p: &mut usize) -> Option<BigRational> {
    let mut v = power(t, p)?;
    while *p < t.len() && matches!(t[*p].as_str(), "*" | "/" | "%" | "mod") {
        let op = t[*p].clone();
        *p += 1;
        let r = power(t, p)?;
        v = match op.as_str() {
            "*" => v * r,
            "/" => {
                if r.is_zero() {
                    return None;
                }
                v / r
            }
            _ => {
                if !v.is_integer() || !r.is_integer() || r.is_zero() {
                    return None;
                }
                BigRational::from_integer(v.numer().mod_floor(r.numer()))
            }
        };
    }
    Some(v)
}

fn power(t: &[String], p: &mut usize) -> Option<BigRational> {
    let b = unary(t, p)?;
    if *p < t.len() && t[*p] == "^" {
        *p += 1;
        let e = power(t, p)?;
        let e = e.is_integer().then(|| e.numer().to_i32()).flatten().filter(|e| e.abs() <= 10000)?;
        if e < 0 && b.is_zero() {
            return None;
        }
        return Some(num_traits::pow::Pow::pow(b, e));
    }
    Some(b)
}

fn unary(t: &[String], p: &mut usize) -> Option<BigRational> {
    match t.get(*p)?.as_str() {
        "-" => {
            *p += 1;
            Some(-unary(t, p)?)
        }
        "(" => {
            *p += 1;
            let v = expr(t, p)?;
            if t.get(*p)? != ")" {
                return None;
            }
            *p += 1;
            Some(v)
        }
        n => {
            let v: BigInt = n.parse().ok()?;
            *p += 1;
            Some(BigRational::from_integer(v))
        }
    }
}

fn conversions(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:convert\s+)?(-?\d+(?:\.\d+)?|\d+/\d+)\s+(?:to|in(?:to)?)\s+(base\s*\d+|[a-z]+)$").captures(l)?;
    let b = base_name(&m[2]).filter(|b| *b != 10)?;
    let src = &m[1];
    let r: BigRational = if let Some((a, c)) = src.split_once('/') {
        BigRational::new(a.parse().ok()?, c.parse().ok()?)
    } else if let Some((i, f)) = src.split_once('.') {
        let den = BigInt::from(10u32).pow(f.len() as u32);
        let neg = i.starts_with('-');
        let iv: BigInt = i.trim_start_matches('-').parse().ok()?;
        let fv: BigInt = f.parse().ok()?;
        let v = BigRational::new(iv * &den + fv, den);
        if neg { -v } else { v }
    } else {
        BigRational::from_integer(src.parse().ok()?)
    };
    // integers in bases up to 36 are handled elsewhere
    if r.is_integer() && b <= 36 {
        return None;
    }
    let neg = r.is_negative();
    let body = frac_in_base(&r.abs(), b);
    let res = format!("{}{body}", if neg { "-" } else { "" });
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{src} in base {b}"))), primary(pod("Result", "Result", text(format!("{res}_{b}"))))];
    if b > 36 {
        pods.push(pod("Note", "Digits", text(format!("digits of base {b} are written as decimal numbers{}", if r.is_integer() { " in braces" } else { " in brackets" }))));
    }
    if !r.is_integer() && res.contains('(') {
        pods.push(pod("Note2", "Notation", text("the digits in parentheses repeat forever")));
    }
    Some(Answer::new("base conversion", pods))
}

// ---------------------------------------------------------------------------
// figurate numbers
// ---------------------------------------------------------------------------

fn polygon_sides(name: &str) -> Option<i64> {
    Some(match name {
        "triangular" => 3,
        "square" => 4,
        "pentagonal" => 5,
        "hexagonal" => 6,
        "heptagonal" => 7,
        "octagonal" => 8,
        "nonagonal" | "enneagonal" => 9,
        "decagonal" => 10,
        "hendecagonal" | "undecagonal" => 11,
        "dodecagonal" => 12,
        "tridecagonal" => 13,
        "tetradecagonal" => 14,
        "pentadecagonal" => 15,
        "hexadecagonal" => 16,
        "icosagonal" => 20,
        _ => return None,
    })
}

/// (display name, n-th term, formula)
fn figurate(name: &str) -> Option<(String, Box<dyn Fn(&BigInt) -> BigInt>, String)> {
    let name = name.trim().trim_end_matches(" number").trim_end_matches(" numbers");
    if let Some(k) = polygon_sides(name) {
        return Some((format!("{name} number"), Box::new(move |n: &BigInt| ((k - 2) * n * n - (k - 4) * n) / 2), if k == 3 { "n(n + 1)/2".into() } else { format!("({}n² − {}n)/2", k - 2, k - 4) }));
    }
    if let Some(rest) = name.strip_prefix("centered ") {
        let rest = rest.trim_start_matches("perfect ");
        if let Some(k) = polygon_sides(rest) {
            return Some((format!("centered {rest} number"), Box::new(move |n: &BigInt| k * n * (n - 1) / 2 + 1), format!("{k}n(n − 1)/2 + 1")));
        }
        if rest == "cube" || rest == "cubic" {
            return Some(("centered cube number".into(), Box::new(|n: &BigInt| n * n * n + (n - 1) * (n - 1) * (n - 1)), "n³ + (n − 1)³".into()));
        }
    }
    let one = |f: fn(&BigInt) -> BigInt, d: &str, fm: &str| Some((d.to_string(), Box::new(f) as Box<dyn Fn(&BigInt) -> BigInt>, fm.to_string()));
    match name {
        "tetrahedral" | "triangular pyramidal" => one(|n| n * (n + 1) * (n + 2) / 6, "tetrahedral number", "n(n + 1)(n + 2)/6"),
        "square pyramidal" | "pyramidal" => one(|n| n * (n + 1) * (2 * n + 1) / 6, "square pyramidal number", "n(n + 1)(2n + 1)/6"),
        "pentagonal pyramidal" => one(|n| n * n * (n + 1) / 2, "pentagonal pyramidal number", "n²(n + 1)/2"),
        "octahedral" => one(|n| n * (2 * n * n + 1) / 3, "octahedral number", "n(2n² + 1)/3"),
        "cubic" | "cube" => one(|n| n * n * n, "cube number", "n³"),
        "pentatope" => one(|n| n * (n + 1) * (n + 2) * (n + 3) / 24, "pentatope number", "n(n + 1)(n + 2)(n + 3)/24"),
        "pronic" | "oblong" | "rectangular" => one(|n| n * (n + 1), "pronic number", "n(n + 1)"),
        "star" => one(|n| 6 * n * (n - 1) + 1, "star number", "6n(n − 1) + 1"),
        "stella octangula" => one(|n| n * (2 * n * n - 1), "stella octangula number", "n(2n² − 1)"),
        "dodecahedral" => one(|n| n * (3 * n - 1) * (3 * n - 2) / 2, "dodecahedral number", "n(3n − 1)(3n − 2)/2"),
        "icosahedral" => one(|n| n * (5 * n * n - 5 * n + 2) / 2, "icosahedral number", "n(5n² − 5n + 2)/2"),
        "square triangular" => None,
        _ => None,
    }
}

fn ordinal(s: &str) -> Option<BigInt> {
    let s = s.trim();
    let words = ["first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth", "eleventh", "twelfth"];
    if let Some(p) = words.iter().position(|w| *w == s) {
        return Some(BigInt::from(p + 1));
    }
    re!(r"^(\d+)(?:st|nd|rd|th)?$").captures(s).and_then(|m| m[1].parse().ok())
}

fn figurates(l: &str) -> Option<Answer> {
    // n-th term
    if let Some(m) = re!(r"^(?:the\s+)?(\w+)\s+((?:centered\s+)?(?:perfect\s+)?[a-z ]+?)\s+number$").captures(l) {
        let n = ordinal(&m[1])?;
        let (name, f, formula) = figurate(&m[2])?;
        if n < BigInt::one() || n.bits() > 64 {
            return None;
        }
        let v = f(&n);
        let first: Vec<String> = (1..=10).map(|i| f(&BigInt::from(i)).to_string()).collect();
        return Some(Answer::new("figurate number", vec![
            pod("Input", "Input interpretation", text(format!("{n}th {name}"))),
            primary(pod("Result", "Result", text(v.to_string()))),
            pod("Formula", "Formula", text(formula)),
            pod("First", "First terms", text(first.join(", "))),
        ]));
    }
    // membership
    let m = re!(r"^(?:is\s+)?(\d+)\s+(?:a|an)\s+((?:centered\s+)?(?:perfect\s+)?[a-z ]+?)\s+number$|^can\s+(\d+)\s+(?:things|objects|balls|spheres|items|dots)\s+(?:make|form)\s+(?:a|an)\s+([a-z ]+?)(?:\s+shape)?$").captures(l)?;
    let target: BigInt = m.get(1).or(m.get(3))?.as_str().parse().ok()?;
    let kind = m.get(2).or(m.get(4))?.as_str();
    let kind = match kind {
        "tetrahedral" | "tetrahedron" | "triangular pyramid" => "tetrahedral",
        "triangle" | "triangular" => "triangular",
        "square" => "square",
        "pyramid" | "square pyramid" => "square pyramidal",
        "octahedron" => "octahedral",
        "cube" => "cubic",
        k => k,
    };
    let (name, f, formula) = figurate(kind)?;
    let mut n = BigInt::one();
    let mut found = None;
    for _ in 0..2_000_000 {
        let v = f(&n);
        if v == target {
            found = Some(n.clone());
            break;
        }
        if v > target {
            break;
        }
        n += 1;
    }
    let res = match &found {
        Some(i) => format!("yes: {target} is the {i}th {name}"),
        None => format!("no: {target} is not a {name}"),
    };
    Some(Answer::new("figurate number", vec![
        pod("Input", "Input interpretation", text(format!("is {target} a {name}?"))),
        primary(pod("Result", "Result", text(res))),
        pod("Formula", "Formula", text(formula)),
    ]))
}

fn roman_value(s: &str) -> Option<u32> {
    let val = |c: char| match c {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    };
    let v: Vec<u32> = s.chars().map(val).collect::<Option<_>>()?;
    let mut total = 0;
    for i in 0..v.len() {
        if i + 1 < v.len() && v[i] < v[i + 1] { total -= v[i] as i64 } else { total += v[i] as i64 }
    }
    let n = u32::try_from(total).ok()?;
    // only canonical spellings count
    (to_roman(n)? == s).then_some(n)
}

fn to_roman(mut n: u32) -> Option<String> {
    if n == 0 || n >= 4000 {
        return None;
    }
    let t = [(1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"), (50, "L"), (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")];
    let mut s = String::new();
    for (v, r) in t {
        while n >= v {
            s.push_str(r);
            n -= v;
        }
    }
    Some(s)
}

fn roman_arith(raw: &str) -> Option<Answer> {
    let t = raw.trim().trim_end_matches(['?', '.', '=']).trim();
    if !re!(r"^[MDCLXVI]+(?:\s*[-+*/×÷]\s*[MDCLXVI]+)+$").is_match(t) {
        return None;
    }
    let mut dec = String::new();
    let mut last = 0;
    for m in re!(r"[MDCLXVI]+").find_iter(t) {
        dec.push_str(&t[last..m.start()]);
        dec.push_str(&roman_value(m.as_str())?.to_string());
        last = m.end();
    }
    dec.push_str(&t[last..]);
    let dec = dec.replace('×', "*").replace('÷', "/");
    let v = eval_rat(&dec)?;
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{t} = {dec}")))];
    let as_int = v.is_integer().then(|| v.numer().to_u32()).flatten();
    match as_int.and_then(to_roman) {
        Some(r) => {
            pods.push(primary(pod("Result", "Result", text(r))));
            pods.push(pod("Decimal", "Decimal form", text(show_rat(&v))));
        }
        None => pods.push(primary(pod("Decimal", "Result", text(show_rat(&v))))),
    }
    Some(Answer::new("Roman numerals", pods))
}

fn mayan(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:convert\s+)?(\d+)\s+(?:to|in(?:to)?)\s+mayan(?:\s+numerals?)?$").captures(l)?;
    let n: BigInt = m[1].parse().ok()?;
    let digits = digits_of(&n, 20);
    let glyph = |d: u32| -> String {
        if d == 0 {
            return "shell (0)".into();
        }
        let bars = d / 5;
        let dots = d % 5;
        let mut parts = Vec::new();
        if dots > 0 {
            parts.push("•".repeat(dots as usize));
        }
        for _ in 0..bars {
            parts.push("—".into());
        }
        format!("{} ({d})", parts.join(" / "))
    };
    let rows: Vec<Vec<Content>> = digits.iter().enumerate().map(|(i, d)| vec![text(format!("20^{}", digits.len() - 1 - i)), text(glyph(*d))]).collect();
    Some(Answer::new("Mayan numerals", vec![
        pod("Input", "Input interpretation", text(format!("{n} in Mayan (base-20) numerals"))),
        primary(pod("Result", "Base-20 digits, highest place first", text(format!("{{{}}}", digits.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", "))))),
        pod("Glyphs", "Glyphs (dots are ones, bars are fives)", table(Some(vec!["place", "digit"]), rows)),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    if let Some(a) = roman_arith(&q.clean) {
        return Some(a);
    }
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    base_literals(&l).or_else(|| conversions(&l)).or_else(|| figurates(&l)).or_else(|| mayan(&l))
}

#[cfg(test)]
mod tests {
    fn short(s: &str) -> String {
        let r = crate::engine::query(s);
        serde_json::to_string(&r.pods).unwrap_or_default()
    }

    #[test]
    fn number_answers() {
        assert!(short("10101 base 2 + 110 base 2").contains("11011_2"));
        assert!(short("1021_4 - 113_4").contains("302_4"));
        assert!(short("0xff + 1").contains("256"));
        assert!(short("675 & 0x00ff").contains("163"));
        assert!(short("is 123 base 5 the same as 46 base 8").contains("True"));
        assert!(short("convert 0.25 to ternary").contains("0.(02)_3"));
        assert!(short("convert 815 to base 200").contains("{4, 15}_200"));
        assert!(short("1250th decagonal number").contains("6246250"));
        assert!(short("is 56 a tetrahedral number").contains("yes"));
    }
}
