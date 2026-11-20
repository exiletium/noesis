//! US economic time series from the federal statistical agencies (BEA,
//! Census Bureau, Federal Reserve Board, Treasury, BLS), as a snapshot
//! taken through FRED: "US retail sales", "US real gdp annual growth rate",
//! "US reserve bank credit March 2009", "Alaska real GDP", "treasury yield curve".

use super::geo::{us_state, US_STATES};
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

static ECON_Z: &[u8] = include_bytes!("../../data/econ.z");

struct Series {
    id: &'static str,
    title: &'static str,
    units: &'static str,
    source: &'static str,
    obs: Vec<(&'static str, f64)>,
}

fn series() -> &'static Vec<Series> {
    static S: OnceLock<Vec<Series>> = OnceLock::new();
    S.get_or_init(|| {
        let bytes = miniz_oxide::inflate::decompress_to_vec(ECON_Z).unwrap_or_default();
        let text: &'static str = Box::leak(String::from_utf8(bytes).unwrap_or_default().into_boxed_str());
        text.lines()
            .filter_map(|l| {
                let c: Vec<&'static str> = l.split('\t').collect();
                (c.len() >= 5).then(|| Series {
                    id: c[0],
                    title: c[1],
                    units: c[2],
                    source: c[3],
                    obs: c[4].split(' ').filter_map(|p| p.split_once('=')).filter_map(|(d, v)| Some((d, v.parse().ok()?))).collect(),
                })
            })
            .collect()
    })
}

fn get(id: &str) -> Option<&'static Series> {
    series().iter().find(|s| s.id == id)
}

/// (phrase, series id, needs a "US" qualifier)
const PHRASES: &[(&str, &str, bool)] = &[
    ("real gdp", "GDPC1", true),
    ("real gross domestic product", "GDPC1", true),
    ("gdp", "GDP", true),
    ("gross domestic product", "GDP", true),
    ("retail sales", "RSAFS", true),
    ("consumer credit", "TOTALSL", false),
    ("consumer credit outstanding", "TOTALSL", false),
    ("industrial production for mining", "IPMINE", true),
    ("industrial production mining", "IPMINE", true),
    ("mining industrial production", "IPMINE", true),
    ("industrial production", "INDPRO", true),
    ("federal debt", "GFDEBTN", true),
    ("national debt", "GFDEBTN", true),
    ("public debt", "GFDEBTN", true),
    ("m1 money supply", "M1SL", false),
    ("m1 money stock", "M1SL", false),
    ("m1", "M1SL", true),
    ("m2 money supply", "M2SL", false),
    ("m2 money stock", "M2SL", false),
    ("m2", "M2SL", true),
    ("reserve bank credit", "WRESCRT", false),
    ("assets abroad", "IIPUSASSQ", true),
    ("foreign assets", "IIPUSASSQ", true),
    ("liabilities to foreigners", "IIPUSLIAQ", true),
    ("foreign liabilities", "IIPUSLIAQ", true),
    ("income receipts", "IEAXI", true),
    ("primary income receipts", "IEAXI", true),
    ("unemployment rate", "UNRATE", true),
    ("consumer price index", "CPIAUCSL", true),
    ("cpi", "CPIAUCSL", true),
    ("housing starts", "HOUST", true),
    ("building permits", "PERMIT", true),
    ("housing permits", "PERMIT", true),
    ("nonfarm payrolls", "PAYEMS", false),
    ("nonfarm employment", "PAYEMS", true),
    ("federal funds rate", "FEDFUNDS", false),
    ("fed funds rate", "FEDFUNDS", false),
    ("30 year treasury bond", "DGS30", false),
    ("30 year treasury", "DGS30", false),
    ("30-year treasury", "DGS30", false),
    ("30 year treasury yield", "DGS30", false),
    ("10 year treasury", "DGS10", false),
    ("10-year treasury", "DGS10", false),
    ("10 year treasury yield", "DGS10", false),
    ("federal minimum wage", "FEDMINNFRWG", false),
    ("exports", "EXPGS", true),
    ("imports", "IMPGS", true),
    ("personal consumption expenditures", "PCE", true),
    ("personal saving rate", "PSAVERT", true),
    ("savings rate", "PSAVERT", true),
];

fn when(id: &str, d: &str) -> String {
    const M: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
    let (y, m, day) = (&d[..4], d[5..7].parse::<usize>().unwrap_or(1), &d[8..10]);
    let s = get(id);
    let n = s.map_or(0, |s| s.obs.len());
    let step = s.filter(|_| n > 2).map(|s| {
        let a = &s.obs[n - 2].0;
        let b = &s.obs[n - 1].0;
        let ym = |x: &str| x[..4].parse::<i32>().unwrap_or(0) * 12 + x[5..7].parse::<i32>().unwrap_or(0);
        (ym(b) - ym(a), a[8..10] != b[8..10])
    });
    match step {
        Some((12, _)) => y.to_string(),
        Some((3, _)) => format!("Q{} {y}", (m - 1) / 3 + 1),
        Some((_, true)) | Some((0, _)) => format!("{} {}, {y}", M[m - 1], day.trim_start_matches('0')),
        _ => format!("{} {y}", M[m - 1]),
    }
}

fn amount(s: &Series, v: f64) -> String {
    let u = s.units.to_lowercase();
    let scaled = |x: f64, unit: &str| -> String {
        let (d, w) = if x.abs() >= 1e12 { (1e12, "trillion") } else if x.abs() >= 1e9 { (1e9, "billion") } else if x.abs() >= 1e6 { (1e6, "million") } else { (1.0, "") };
        let num = fmt_float((x / d * 1000.0).round() / 1000.0, 5);
        if unit == "$" { format!("${num} {w}").trim().to_string() } else { format!("{num} {w} {unit}").replace("  ", " ") }
    };
    if u.starts_with("millions of chained 2017 dollars") {
        format!("{} (chained 2017 dollars)", scaled(v * 1e6, "$"))
    } else if u.starts_with("billions of chained 2017 dollars") {
        format!("{} (chained 2017 dollars)", scaled(v * 1e9, "$"))
    } else if u.starts_with("millions of") && u.contains("dollars") {
        scaled(v * 1e6, "$")
    } else if u.starts_with("billions of") && u.contains("dollars") {
        scaled(v * 1e9, "$")
    } else if u.starts_with("thousands of units") {
        format!("{} units", fmt_float((v * 1000.0).round(), 9))
    } else if u.starts_with("thousands of persons") {
        format!("{} people", fmt_float((v * 1000.0).round(), 9))
    } else if u.starts_with("percent") {
        format!("{}%", fmt_float(v, 4))
    } else if u.starts_with("dollars per hour") {
        format!("${}/hour", fmt_float(v, 4))
    } else {
        format!("{} ({})", fmt_float(v, 6), s.units.split(',').next().unwrap_or(""))
    }
}

fn pct(a: f64, b: f64) -> String {
    let p = (b / a - 1.0) * 100.0;
    format!("{}{}%", if p >= 0.0 { "+" } else { "" }, fmt_float((p * 100.0).round() / 100.0, 4))
}

fn yield_curve() -> Answer {
    let ids = [("1 month", "DGS1MO"), ("3 months", "DGS3MO"), ("6 months", "DGS6MO"), ("1 year", "DGS1"), ("2 years", "DGS2"), ("3 years", "DGS3"), ("5 years", "DGS5"), ("7 years", "DGS7"), ("10 years", "DGS10"), ("20 years", "DGS20"), ("30 years", "DGS30")];
    let rows: Vec<Vec<Content>> = ids.iter().filter_map(|(k, id)| {
        let s = get(id)?;
        let (_, v) = s.obs.last()?;
        Some(vec![text(*k), text(format!("{}%", fmt_float(*v, 4)))])
    }).collect();
    let date = get("DGS10").and_then(|s| s.obs.last()).map(|o| when("DGS10", o.0)).unwrap_or_default();
    let spread = (|| Some(get("DGS10")?.obs.last()?.1 - get("DGS2")?.obs.last()?.1))().unwrap_or(f64::NAN);
    Answer::new("treasury yield curve", vec![
        pod("Input", "Input interpretation", text("US Treasury yield curve | constant-maturity market yields")),
        primary(pod("Result", format!("Yields on {date}").as_str(), table(Some(vec!["maturity", "yield"]), rows))),
        pod("Spread", "10-year minus 2-year spread", text(format!("{} percentage points{}", fmt_float((spread * 100.0).round() / 100.0, 3), if spread < 0.0 { " (inverted)" } else { "" }))),
        pod("Source", "Source", text("Federal Reserve Board H.15 release, via FRED; snapshot of the latest published values.")),
    ])
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?').trim().replace("u.s.", "us").replace("u.s", "us");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    if re!(r"^(?:the |us |current |today'?s )*(?:treasury )?yield curve$").is_match(&l) {
        return Some(yield_curve());
    }
    // "USA national debt / USA GDP"
    if re!(r"^(?:the )?(?:us|usa|united states)'?s? (?:national|federal|public) debt ?/ ?(?:the )?(?:us|usa|united states)'?s? gdp$").is_match(&l) || re!(r"^(?:us|usa|united states) (?:national |federal )?debt to gdp(?: ratio)?$").is_match(&l) {
        return answer(get("GFDEGDQ188S")?, None, None, "US national debt / US GDP");
    }
    let m = re!(r"^(?:what (?:is|was|were) )?(?:the )?(?:(us|usa|united states|american|america's|federal)'?s? )?(.+?)(?: (seasonally adjusted growth rate|annual growth rate|year over year growth rate|year-over-year growth rate|growth rate|growth|annual change|change))?(?: (?:in|for|of|during) )?(?: ?((?:january|february|march|april|may|june|july|august|september|october|november|december) )?(\d{4}))?$").captures(&l)?;
    let us = m.get(1).is_some();
    let mut phrase = m[2].trim().to_string();
    let growth = m.get(3).map(|x| x.as_str().to_string());
    let date = m.get(5).map(|y| format!("{}{}", y.as_str(), m.get(4).map_or(String::new(), |mo| format!("-{:02}", ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"].iter().position(|x| *x == mo.as_str().trim()).unwrap_or(0) + 1))));
    // annual totals: "US annual income receipts 1987"
    let annual = phrase.starts_with("annual ");
    phrase = phrase.trim_start_matches("annual ").trim_start_matches("total ").to_string();
    // state real GDP: "Alaska real GDP", "real gdp of california"
    if !us {
        if let Some(c) = re!(r"^(?:(.+?) real gdp|real gdp (?:of|in) (.+))$").captures(&phrase) {
            let st = us_state(c.get(1).or(c.get(2))?.as_str())?;
            let s = get(&format!("{st}RGSP"))?;
            let name = US_STATES.iter().find(|x| x.0 == st)?.1;
            return answer(s, date.as_deref(), growth.as_deref(), &format!("{name} real GDP"));
        }
    }
    let (_, id, needs_us) = PHRASES.iter().find(|p| p.0 == phrase)?;
    if *needs_us && !us {
        return None;
    }
    // growth-rate series published directly
    let id = match (*id, growth.as_deref()) {
        ("GDPC1", Some("annual growth rate")) => "A191RL1A225NBEA",
        (i, _) => i,
    };
    let s = get(id)?;
    if annual {
        let y = date.as_deref().map(|d| &d[..4])?;
        let vals: Vec<f64> = s.obs.iter().filter(|o| o.0.starts_with(y)).map(|o| o.1).collect();
        if vals.len() != 4 {
            return None;
        }
        let total: f64 = vals.iter().sum();
        return Some(Answer::new("economic data", vec![
            pod("Input", "Input interpretation", text(format!("United States | annual {} | {y}", s.title.to_lowercase()))),
            primary(pod("Result", "Result", text(amount(s, total)))),
            pod("Quarters", "Quarterly values", text(vals.iter().map(|v| amount(s, *v)).collect::<Vec<_>>().join(" | "))),
            pod("Source", "Source", text(format!("{}, via FRED; sum of the four quarters.", s.source))),
        ]));
    }
    let label = format!("US {}", if id == "A191RL1A225NBEA" { "real GDP annual growth rate".to_string() } else { phrase.clone() });
    answer(s, date.as_deref(), if id == "A191RL1A225NBEA" { None } else { growth.as_deref() }, &label)
}

fn answer(s: &'static Series, date: Option<&str>, growth: Option<&str>, label: &str) -> Option<Answer> {
    let idx = match date {
        Some(d) => s.obs.iter().rposition(|o| o.0.starts_with(d)).or_else(|| (d.len() == 4).then(|| s.obs.iter().rposition(|o| o.0.starts_with(d))).flatten())?,
        None => s.obs.len().checked_sub(1)?,
    };
    let (d, v) = s.obs[idx];
    let ym = |x: &str| x[..4].parse::<i32>().unwrap_or(0) * 12 + x[5..7].parse::<i32>().unwrap_or(0);
    let year_ago = s.obs[..idx].iter().rposition(|o| ym(o.0) <= ym(d) - 12).map(|k| s.obs[k]);
    let prev = (idx > 0).then(|| s.obs[idx - 1]);
    let main = match growth {
        Some(g) if g.contains("seasonally") || (g == "growth rate" && s.units.contains("Seasonally Adjusted") && !s.units.contains("Annual Rate") && prev.is_some() && ym(d) - ym(prev?.0) == 1) => {
            let (pd, pv) = prev?;
            format!("{} ({} to {})", pct(pv, v), when(s.id, pd), when(s.id, d))
        }
        Some(_) => {
            let (pd, pv) = year_ago?;
            format!("{} per year ({} to {})", pct(pv, v), when(s.id, pd), when(s.id, d))
        }
        None => format!("{} ({})", amount(s, v), when(s.id, d)),
    };
    let lo = s.obs.len().saturating_sub(8).min(idx.saturating_sub(4));
    let rows: Vec<Vec<Content>> = s.obs[lo..=(lo + 7).min(s.obs.len() - 1)].iter().map(|(d, v)| vec![text(when(s.id, d)), text(amount(s, *v))]).collect();
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{label}{}{}", growth.map_or(String::new(), |g| format!(" | {g}")), date.map_or(String::new(), |_| format!(" | {}", when(s.id, d)))))),
        primary(pod("Result", "Result", text(main))),
        pod("Recent", "Values", table(Some(vec!["date", "value"]), rows)),
    ];
    if let (Some((pd, pv)), None) = (year_ago, growth) {
        pods.push(pod("Change", "Change from a year earlier", text(format!("{} (from {} in {})", pct(pv, v), amount(s, pv), when(s.id, pd)))));
    }
    let (maxd, maxv) = s.obs.iter().max_by(|a, b| a.1.total_cmp(&b.1))?;
    let (mind, minv) = s.obs.iter().min_by(|a, b| a.1.total_cmp(&b.1))?;
    if s.obs.len() > 12 {
        pods.push(pod("Range", format!("Range since {}", &s.obs[0].0[..4]).as_str(), text(format!("high {} ({}) | low {} ({})", amount(s, *maxv), when(s.id, maxd), amount(s, *minv), when(s.id, mind)))));
    }
    pods.push(pod("Source", "Source", text(format!("{}: {} ({}), via FRED; values as published at the time of the snapshot.", s.source, s.title, s.units))));
    Some(Answer::new("economic data", pods))
}
