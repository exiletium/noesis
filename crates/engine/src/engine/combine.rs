//! Arithmetic on two looked-up values, tried only when nothing else
//! understood the input: "population Madrid / population Spain",
//! "GDP per capita Norway / United States", "elevation Denver - Los Angeles".
//! A side that names only an entity borrows the property from the other side.

use super::*;
use regex::Regex;
use std::cell::Cell;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// Another quantity of the answer in the given unit, as in
/// "0.1221 M☉ (2.43×10^29 kg)" read in kg.
fn value_in(text: &str, unit: &str) -> Option<f64> {
    let line = text.lines().next()?;
    let r = Regex::new(&format!(r"(\d[\d,]*(?:\.\d+)?)(?:\s*×\s*10\^\(?(-?\d+)\)?)?\s*{}(?:[\s,;)]|$)", regex::escape(unit))).ok()?;
    let m = r.captures(line)?;
    let mut v: f64 = m[1].replace(',', "").parse().ok()?;
    if let Some(e) = m.get(2) {
        v *= 10f64.powi(e.as_str().parse().ok()?);
    }
    Some(v)
}

/// The leading quantity of an answer: value and unit ("$" for money).
fn value(text: &str) -> Option<(f64, String)> {
    let line = text.lines().next()?;
    let m = re!(r"(-?)(\$|€|£)?\s?(\d[\d,]*(?:\.\d+)?|\.\d+)(?:\s*×\s*10\^\(?(-?\d+)\)?)?(?:\s*(thousand|million|billion|trillion))?(?:\s+([^\s,;()]+))?").captures(line)?;
    // a table or a label before the number means this is not a plain quantity
    let before = &line[..m.get(0)?.start()];
    if before.contains('|') || before.chars().filter(|c| c.is_alphabetic()).count() > 3 {
        return None;
    }
    let mut v: f64 = m[3].replace(',', "").parse().ok()?;
    if let Some(e) = m.get(4) {
        v *= 10f64.powi(e.as_str().parse().ok()?);
    }
    v *= match m.get(5).map(|x| x.as_str()) {
        Some("thousand") => 1e3,
        Some("million") => 1e6,
        Some("billion") => 1e9,
        Some("trillion") => 1e12,
        _ => 1.0,
    };
    if &m[1] == "-" {
        v = -v;
    }
    // a percentage or a rate ("per 1,000") is not a plain quantity
    let after = &line[m.get(3)?.end()..];
    if after.trim_start().starts_with('%') {
        return Some((v, "%".to_string()));
    }
    let tok = after.split_whitespace().next().unwrap_or("");
    if tok.contains('/') {
        return Some((v, tok.to_string()));
    }
    let unit = match (m.get(2), m.get(6)) {
        (Some(c), _) => c.as_str().to_string(),
        (None, Some(u)) if !["per", "and", "or", "to", "in", "at", "on", "from", "for"].contains(&u.as_str()) => u.as_str().to_string(),
        _ => String::new(),
    };
    Some((v, unit))
}

fn lookup(s: &str, opts: &Options) -> Option<(f64, String, String)> {
    let r = run_handlers(s, opts);
    if !r.success {
        return None;
    }
    let t = r.short_answer()?;
    let (v, u) = value(&t)?;
    Some((v, u, t.lines().next().unwrap_or("").to_string()))
}

fn sig(v: f64) -> String {
    if v == 0.0 || !v.is_finite() {
        return "0".into();
    }
    let d = (3 - v.abs().log10().floor() as i32).max(0) as usize;
    let s = format!("{:.*}", d, v);
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    crate::print::fmt_float(s.parse().unwrap_or(v), 10)
}

pub fn combine(q: &Query) -> Option<Answer> {
    if DEPTH.with(|d| d.get()) > 0 {
        return None;
    }
    let m = re!(r"^(.+?)\s+(/|-|divided by|minus)\s+(.+)$").captures(q.clean.trim())?;
    let (a, op, b) = (m[1].trim().to_string(), &m[2], m[3].trim().trim_end_matches('?').to_string());
    let wordy = |s: &str| re!(r"[A-Za-z]{3,}").is_match(s) && !re!(r"^[\d\s.,]+$").is_match(s);
    if !wordy(&a) || !wordy(&b) {
        return None;
    }
    DEPTH.with(|d| d.set(1));
    let out = (|| {
        let (va, ua, ta) = lookup(&a, q.opts)?;
        let aw: Vec<&str> = a.split_whitespace().collect();
        let shares = b.split_whitespace().any(|w| w.len() > 2 && aw.iter().any(|x| x.eq_ignore_ascii_case(w)));
        let divide = op == "/" || op == "divided by";
        let mut tries = vec![];
        if shares {
            tries.push(b.clone());
        }
        for k in (1..aw.len()).rev() {
            tries.push(format!("{} {b}", aw[..k].join(" ")));
            if !aw[k - 1].eq_ignore_ascii_case("of") {
                tries.push(format!("{} of {b}", aw[..k].join(" ")));
            }
        }
        for k in 1..aw.len() {
            tries.push(format!("{b} {}", aw[k..].join(" ")));
        }
        if !shares {
            tries.push(b.clone());
        }
        // "mass Sun" -> "mass of Sun"
        let bw: Vec<&str> = b.split_whitespace().collect();
        for k in 1..bw.len().min(3) {
            tries.push(format!("{} of {}", bw[..k].join(" "), bw[k..].join(" ")));
        }
        let (mut vb, mut ub, tb, bq) = tries.iter().take(12).find_map(|t| {
            if budget::remaining() <= 0.3 {
                return None;
            }
            let (v, u, txt) = lookup(t, q.opts)?;
            // the borrowed property must give the same kind of quantity
            (u == ua || t == &b || divide).then(|| (v, u, txt, t.clone()))
        })?;
        // the same quantity in different units: use the other unit given in either answer
        let (mut va, mut ua) = (va, ua);
        if ua != ub && !ua.is_empty() && !ub.is_empty() {
            if let Some(v) = value_in(&ta, &ub) {
                (va, ua) = (v, ub.clone());
            } else if let Some(v) = value_in(&tb, &ua) {
                (vb, ub) = (v, ua.clone());
            }
        }
        let (res, unit) = if divide {
            if vb == 0.0 {
                return None;
            }
            let rate = |u: &str| u == "%" || u.contains('/');
            if ua != ub && (rate(&ua) || rate(&ub)) {
                return None;
            }
            let unit = if ua == ub { String::new() } else if ub.is_empty() { ua.clone() } else if ub == "people" { format!("{ua} per person") } else { format!("{ua}/{ub}") };
            (va / vb, unit)
        } else {
            if ua != ub {
                return None;
            }
            (va - vb, ua.clone())
        };
        let shown = match unit.as_str() {
            "" => sig(res),
            "$" | "€" | "£" => format!("{unit}{}", sig(res)),
            u => format!("{} {u}", sig(res)),
        };
        let shown = if divide && unit.is_empty() && res.abs() < 1.0 { format!("{shown} ({}%)", sig(res * 100.0)) } else { shown };
        Some(Answer::new("combined", vec![
            pod("Input", "Input interpretation", text(format!("({a}) {} ({bq})", if divide { "/" } else { "−" }))),
            primary(pod("Result", "Result", text(shown))),
            pod("Values", "Values", table(None, vec![vec![text(a.clone()), text(ta)], vec![text(bq.clone()), text(tb)]])),
        ]))
    })();
    DEPTH.with(|d| d.set(0));
    out
}
