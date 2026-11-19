//! Practical tables and trade formulas: US consumer-price inflation,
//! shoe sizes, Unified (UTS) screw threads, log scaling rules and
//! psychrometrics of moist air.

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

fn rows(r: Vec<(&str, String)>) -> Content {
    table(None, r.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())
}

fn money(v: f64) -> String {
    super::misc::money(v)
}

// ---------------------------------------------------------------------------
// inflation: BLS CPI-U, US city average, annual averages (1982–84 = 100)
// ---------------------------------------------------------------------------

const CPI_FIRST: i32 = 1913;
const CPI: [f64; 112] = [
    9.9, 10.0, 10.1, 10.9, 12.8, 15.1, 17.3, 20.0, 17.9, 16.8, // 1913–1922
    17.1, 17.1, 17.5, 17.7, 17.4, 17.1, 17.1, 16.7, 15.2, 13.7, // 1923–1932
    13.0, 13.4, 13.7, 13.9, 14.4, 14.1, 13.9, 14.0, 14.7, 16.3, // 1933–1942
    17.3, 17.6, 18.0, 19.5, 22.3, 24.1, 23.8, 24.1, 26.0, 26.5, // 1943–1952
    26.7, 26.9, 26.8, 27.2, 28.1, 28.9, 29.1, 29.6, 29.9, 30.2, // 1953–1962
    30.6, 31.0, 31.5, 32.4, 33.4, 34.8, 36.7, 38.8, 40.5, 41.8, // 1963–1972
    44.4, 49.3, 53.8, 56.9, 60.6, 65.2, 72.6, 82.4, 90.9, 96.5, // 1973–1982
    99.6, 103.9, 107.6, 109.6, 113.6, 118.3, 124.0, 130.7, 136.2, 140.3, // 1983–1992
    144.5, 148.2, 152.4, 156.9, 160.5, 163.0, 166.6, 172.2, 177.1, 179.9, // 1993–2002
    184.0, 188.9, 195.3, 201.6, 207.342, 215.303, 214.537, 218.056, 224.939, 229.594, // 2003–2012
    232.957, 236.736, 237.017, 240.007, 245.120, 251.107, 255.657, 258.811, 270.970, 292.655, // 2013–2022
    304.702, 313.689, // 2023–2024
];

fn cpi(y: i32) -> Option<f64> {
    CPI.get(usize::try_from(y - CPI_FIRST).ok()?).copied()
}

fn last_cpi_year() -> i32 {
    CPI_FIRST + CPI.len() as i32 - 1
}

fn inflation(l: &str) -> Option<Answer> {
    if !(l.contains('$') || l.contains("dollar") || l.contains("usd") || l.contains("inflation")) {
        return None;
    }
    let amount = r"(?:us\s*)?\$?\s*([\d,]+(?:\.\d+)?)\s*(?:k\b)?\s*(?:us\s+)?(?:dollars?|usd|\$)?";
    let (v, from, to) = if let Some(m) = Regex::new(&format!(r"^{amount}\s*\(?\s*(?:in\s+)?(\d{{4}})\s+(?:us\s+)?(?:dollars?|usd|\$)\s*\)?\s*(?:in|to|into|in terms of)\s+(\d{{4}}|today|now|current)(?:\s+(?:us\s+)?(?:dollars?|usd))?$")).ok()?.captures(l) {
        (m[1].replace(',', "").parse::<f64>().ok()?, m[2].parse::<i32>().ok()?, m[3].to_string())
    } else if let Some(m) = Regex::new(&format!(r"^{amount}\s+(?:in|from)\s+(\d{{4}})(?:\s+(?:in|to|adjusted to|in today'?s?|in current)\s*(\d{{4}}|today|now|current|dollars|money)?(?:\s+(?:us\s+)?(?:dollars?|money))?)?$")).ok()?.captures(l) {
        (m[1].replace(',', "").parse::<f64>().ok()?, m[2].parse::<i32>().ok()?, m.get(3).map_or("today".into(), |x| x.as_str().to_string()))
    } else if let Some(m) = Regex::new(&format!(r"^{amount}\s*(?:in\s+)?\(?(\d{{4}})\s+(?:us\s+)?dollars?\)?$")).ok()?.captures(l) {
        (m[1].replace(',', "").parse::<f64>().ok()?, m[2].parse::<i32>().ok()?, "today".to_string())
    } else if let Some(m) = Regex::new(&format!(r"^{amount}\s+(?:current|today'?s?)\s+(?:us\s+)?dollars?\s+in\s+(\d{{4}})$")).ok()?.captures(l) {
        // "10,000 current US dollars in 1910": today's money expressed in that year
        let v: f64 = m[1].replace(',', "").parse().ok()?;
        let y: i32 = m[2].parse().ok()?;
        (v, -y, "today".to_string())
    } else if let Some(m) = re!(r"^(?:us\s+)?(?:inflation|cumulative inflation|cpi inflation)\s+(?:rate\s+)?(?:from|between)\s+(\d{4})\s+(?:to|and)\s+(\d{4})$").captures(l) {
        (100.0, m[1].parse().ok()?, m[2].to_string())
    } else {
        return None;
    };
    let rate_only = re!(r"^(?:us\s+)?(?:inflation|cumulative inflation|cpi inflation)").is_match(l);
    let latest = last_cpi_year();
    let to_year = |s: &str| -> Option<i32> { if s.chars().all(|c| c.is_ascii_digit()) { s.parse().ok() } else { Some(latest) } };
    // a negative "from" marks the reverse direction (today's money in a past year)
    let (from, to) = if from < 0 { (latest, -from) } else { (from, to_year(&to)?) };
    let (c1, c2) = (cpi(from)?, cpi(to)?);
    let out = v * c2 / c1;
    let years = (to - from).abs() as f64;
    let rate = if years > 0.0 { ((c2 / c1).powf(1.0 / years * (to - from).signum() as f64) - 1.0) * 100.0 } else { 0.0 };
    let pct = (c2 / c1 - 1.0) * 100.0;
    let cumulative = format!("{}{}%", if pct >= 0.0 { "+" } else { "" }, n((pct * 100.0).round() / 100.0));
    Some(Answer::new(
        "inflation",
        vec![
            input(if rate_only { format!("US consumer price inflation from {from} to {to}") } else { format!("${} in {from} US dollars → {to} US dollars", money(v).trim_start_matches('$')) }),
            primary(pod("Result", "Result", text(if rate_only { format!("{cumulative} cumulative ({}% per year)", n((rate * 100.0).round() / 100.0)) } else { money(out) }))),
            pod("Details", "Details", rows(vec![
                ("CPI-U annual average", format!("{from}: {}, {to}: {}", n(c1), n(c2))),
                ("cumulative price change", cumulative),
                ("average annual inflation", format!("{}% per year", n((rate * 100.0).round() / 100.0))),
                ("one dollar", format!("$1 in {from} ≈ {} in {to}", money(c2 / c1))),
            ])),
            pod("Source", "Note", text(format!("US Bureau of Labor Statistics consumer price index for all urban consumers (CPI-U), annual averages {CPI_FIRST}–{latest}; \"today\" uses the {latest} average"))),
        ],
    ))
}

/// US federal minimum wage (Fair Labor Standards Act): year it took effect, $/hour.
const MIN_WAGE: &[(i32, f64)] = &[
    (1938, 0.25), (1939, 0.30), (1945, 0.40), (1950, 0.75), (1956, 1.00), (1961, 1.15), (1963, 1.25), (1967, 1.40), (1968, 1.60),
    (1974, 2.00), (1975, 2.10), (1976, 2.30), (1978, 2.65), (1979, 2.90), (1980, 3.10), (1981, 3.35), (1990, 3.80), (1991, 4.25),
    (1996, 4.75), (1997, 5.15), (2007, 5.85), (2008, 6.55), (2009, 7.25),
];

fn min_wage_in(y: i32) -> Option<f64> {
    MIN_WAGE.iter().rev().find(|w| w.0 <= y).map(|w| w.1)
}

fn min_wage(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:what (?:was|is) )?(?:the )?(?:(?:us|u\.s\.|united states|american|federal|us federal) )*minimum wage(?: history| over time| by year|(?: in| for| of)? (\d{4}))?(?: (adjusted for inflation|inflation[- ]adjusted|in today'?s dollars|in (?:current|real) dollars|in (\d{4}) dollars|real))?$").captures(l)?;
    let latest = last_cpi_year();
    let to = m.get(3).and_then(|x| x.as_str().parse::<i32>().ok()).unwrap_or(latest);
    cpi(to)?;
    let real = |y: i32, w: f64| cpi(y).map(|c| w * cpi(to).unwrap_or(c) / c);
    // history plot: nominal and real wage each year
    let years: Vec<i32> = (1938..=latest).collect();
    let xs: Vec<f64> = years.iter().map(|y| *y as f64).collect();
    let nominal: Vec<f64> = years.iter().map(|y| min_wage_in(*y).unwrap_or(f64::NAN)).collect();
    let realv: Vec<f64> = years.iter().map(|y| min_wage_in(*y).and_then(|w| real(*y, w)).unwrap_or(f64::NAN)).collect();
    let real_label = format!("{to} dollars");
    let pl = crate::plot::lines(vec![("nominal", xs.clone(), nominal), (real_label.as_str(), xs, realv.clone())], "year", "$/hour");
    let hist: Vec<Vec<Content>> = MIN_WAGE.iter().map(|(y, w)| vec![text(y.to_string()), text(money(*w)), text(real(*y, *w).map_or("—".into(), money))]).collect();
    let peak = years.iter().zip(&realv).filter(|(_, v)| v.is_finite()).max_by(|a, b| a.1.total_cmp(b.1)).map(|(y, v)| (*y, *v))?;
    let mut pods = Vec::new();
    if let Some(y) = m.get(1).and_then(|x| x.as_str().parse::<i32>().ok()) {
        let w = min_wage_in(y)?;
        let r = real(y, w)?;
        let adjusted = m.get(2).is_some();
        pods.push(pod("Input", "Input interpretation", text(format!("US federal minimum wage | {y}{}", if adjusted { format!(" | in {to} dollars") } else { String::new() }))));
        pods.push(primary(pod("Result", "Result", text(if adjusted { format!("{} per hour ({} per hour in {y} dollars)", money(r), money(w)) } else { format!("{} per hour", money(w)) }))));
        pods.push(pod("Details", "Details", rows(vec![
            ("nominal wage", format!("{} per hour", money(w))),
            ("in today's money", format!("{} per hour in {to} dollars (CPI-U)", money(r))),
            ("full-time annual (2,080 h)", format!("{} ({} in {to} dollars)", money(w * 2080.0), money(r * 2080.0))),
            ("current federal minimum", format!("{} per hour (since July 24, 2009)", money(min_wage_in(latest)?))),
            ("ratio to current", format!("{}× the current minimum in real terms", fmt_float((r / min_wage_in(latest)? * 100.0).round() / 100.0, 4))),
        ])));
    } else {
        pods.push(pod("Input", "Input interpretation", text("US federal minimum wage | history")));
        pods.push(primary(pod("Result", "Current federal minimum wage", text(format!("{} per hour (since July 24, 2009)", money(min_wage_in(latest)?))))));
    }
    pods.push(pod("Plot", &format!("Federal minimum wage, nominal and in {to} dollars"), plot(pl)));
    pods.push(pod("History", "History", table(Some(vec!["effective", "nominal", format!("in {to} dollars").as_str()]), hist)));
    pods.push(pod("Peak", "Highest real value", text(format!("{} per hour in {to} dollars ({})", money(peak.1), peak.0))));
    pods.push(pod("Source", "Note", text(format!("Fair Labor Standards Act rates for covered nonexempt workers; many states set higher minimums. Adjusted with the BLS CPI-U annual averages ({latest} latest)."))));
    Some(Answer::new("US federal minimum wage", pods))
}

// ---------------------------------------------------------------------------
// shoe sizes
// ---------------------------------------------------------------------------

// (US size, EU size) — common retail charts
const MEN_EU: &[(f64, f64)] = &[(4.0, 36.0), (5.0, 37.5), (6.0, 39.0), (6.5, 39.5), (7.0, 40.0), (7.5, 40.5), (8.0, 41.0), (8.5, 42.0), (9.0, 42.5), (9.5, 43.0), (10.0, 44.0), (10.5, 44.5), (11.0, 45.0), (11.5, 45.5), (12.0, 46.0), (13.0, 47.5), (14.0, 48.5), (15.0, 49.5), (16.0, 50.5)];
const WOMEN_EU: &[(f64, f64)] = &[(4.0, 34.5), (5.0, 35.5), (5.5, 36.0), (6.0, 36.5), (6.5, 37.0), (7.0, 37.5), (7.5, 38.0), (8.0, 38.5), (8.5, 39.0), (9.0, 40.0), (9.5, 40.5), (10.0, 41.0), (10.5, 42.0), (11.0, 42.5), (12.0, 43.5), (13.0, 44.5)];

fn interp(t: &[(f64, f64)], x: f64, inverse: bool) -> Option<f64> {
    let pts: Vec<(f64, f64)> = t.iter().map(|&(a, b)| if inverse { (b, a) } else { (a, b) }).collect();
    if x < pts[0].0 || x > pts[pts.len() - 1].0 {
        return None;
    }
    for w in pts.windows(2) {
        if x >= w[0].0 && x <= w[1].0 {
            let f = (x - w[0].0) / (w[1].0 - w[0].0);
            return Some(w[0].1 + f * (w[1].1 - w[0].1));
        }
    }
    None
}

fn shoe_sizes(l: &str) -> Option<Answer> {
    if !l.contains("shoe") && !l.contains("size") {
        return None;
    }
    let sys = r"(u\.?s\.?|american|uk|u\.?k\.?|british|eu|european|french|italian|german|japan(?:ese)?|jp|mondopoint|cm)";
    let m = Regex::new(&format!(r"^(?:(?:a|an)\s+)?{sys}?\s*(?:(men|women|male|female|ladies|man|woman)(?:'s|s)?\s+)?{sys}?\s*(?:(men|women)(?:'s|s)?\s+)?(?:shoe\s+)?size\s+(\d+(?:\.\d+)?)(?:\s+{sys})?(?:\s+(men|women)(?:'s|s)?)?(?:\s+shoes?)?(?:\s+(?:in|to|into|as)\s+{sys}(?:\s+shoe)?(?:\s+sizes?)?)?$")).ok()?.captures(l)?;
    let norm = |s: &str| -> &'static str {
        let s = s.replace('.', "");
        match s.as_str() {
            "us" | "american" => "US",
            "uk" | "british" => "UK",
            "eu" | "european" | "french" | "italian" | "german" => "EU",
            "japan" | "japanese" | "jp" | "mondopoint" | "cm" => "JP",
            _ => "US",
        }
    };
    let from = m.get(1).or(m.get(3)).or(m.get(6)).map(|x| norm(x.as_str())).unwrap_or("US");
    let to = m.get(8).map(|x| norm(x.as_str()));
    let g = m.get(2).or(m.get(4)).or(m.get(7)).map(|x| x.as_str()).unwrap_or("men");
    let women = g.starts_with('w') || g.starts_with('f') || g.starts_with('l');
    let v: f64 = m[5].parse().ok()?;
    let (uk_off, jp_off, tab) = if women { (2.0, 17.0, WOMEN_EU) } else { (1.0, 18.0, MEN_EU) };
    let us = match from {
        "US" => v,
        "UK" => v + uk_off,
        "JP" => v - jp_off,
        _ => interp(tab, v, true)?,
    };
    let eu = interp(tab, us, false)?;
    let half = |x: f64| (x * 2.0).round() / 2.0;
    let vals = [("US", half(us)), ("UK", half(us - uk_off)), ("EU", half(eu)), ("JP", us + jp_off)];
    let label = |k: &str, x: f64| match k {
        "JP" => format!("{} cm (Japan / Mondopoint {} mm)", n(x), n(x * 10.0)),
        "EU" => format!("{} (EU / French)", n(x)),
        _ => format!("{} ({k})", n(x)),
    };
    let who = if women { "women's" } else { "men's" };
    let main = match to {
        Some(t) => label(t, vals.iter().find(|(k, _)| *k == t)?.1),
        None => vals.iter().filter(|(k, _)| *k != from).map(|(k, x)| label(k, *x)).collect::<Vec<_>>().join(", "),
    };
    Some(Answer::new(
        "shoe size",
        vec![
            input(format!("{from} {who} shoe size {}{}", n(v), to.map(|t| format!(" → {t}")).unwrap_or_default())),
            primary(pod("Result", "Result", text(main))),
            pod("All", "Equivalent sizes", rows(vals.iter().map(|(k, x)| (*k, label(k, *x))).collect())),
            pod("Note", "Note", text("standard retail conversion charts; brands differ by up to half a size")),
        ],
    ))
}

// ---------------------------------------------------------------------------
// Unified screw threads (UNC / UNF)
// ---------------------------------------------------------------------------

// (name, major diameter in inches, coarse TPI, fine TPI)
const UTS: &[(&str, f64, u32, u32)] = &[
    ("#0", 0.060, 0, 80), ("#1", 0.073, 64, 72), ("#2", 0.086, 56, 64), ("#3", 0.099, 48, 56), ("#4", 0.112, 40, 48), ("#5", 0.125, 40, 44),
    ("#6", 0.138, 32, 40), ("#8", 0.164, 32, 36), ("#10", 0.190, 24, 32), ("#12", 0.216, 24, 28), ("1/4", 0.250, 20, 28), ("5/16", 0.3125, 18, 24),
    ("3/8", 0.375, 16, 24), ("7/16", 0.4375, 14, 20), ("1/2", 0.500, 13, 20), ("9/16", 0.5625, 12, 18), ("5/8", 0.625, 11, 18), ("3/4", 0.750, 10, 16),
    ("7/8", 0.875, 9, 14), ("1", 1.000, 8, 12), ("1-1/8", 1.125, 7, 12), ("1-1/4", 1.250, 7, 12), ("1-1/2", 1.500, 6, 12),
];

fn thread_rows(d: f64, tpi: f64) -> Vec<(&'static str, String)> {
    let p = 1.0 / tpi;
    let h = 0.866_025 * p;
    let pitch_d = d - 0.649_519 * p;
    let minor_ext = d - 1.226_869 * p; // external thread minor diameter (UNR root)
    let minor_int = d - 1.082_532 * p; // basic minor diameter (internal)
    let area = std::f64::consts::FRAC_PI_4 * (d - 0.9743 / tpi).powi(2);
    let mm = |x: f64| format!("{} in ({} mm)", n((x * 1e4).round() / 1e4), n((x * 25.4 * 1e3).round() / 1e3));
    vec![
        ("basic major diameter", mm(d)),
        ("threads per inch", n(tpi)),
        ("pitch", mm(p)),
        ("basic pitch diameter", mm(pitch_d)),
        ("basic minor diameter (internal)", mm(minor_int)),
        ("minor diameter (external, UNR)", mm(minor_ext)),
        ("thread height H", mm(h)),
        ("tensile stress area", format!("{} in² ({} mm²)", n((area * 1e5).round() / 1e5), n((area * 645.16 * 100.0).round() / 100.0))),
        ("tap drill (≈75% thread)", mm(d - 0.974_3 * p)),
    ]
}

fn bolts(l: &str) -> Option<Answer> {
    // explicit thread: "threading, basic major diameter 0.2 inch, 24 threads per inch", "1/4-20"
    if let Some(m) = re!(r"(?:major diameter|diameter)\s*(?:of\s*)?(\d*\.\d+|\d+)\s*(?:in|inch|inches|\x22)?,?\s*(?:and\s*)?(\d+)\s*(?:threads per inch|tpi)").captures(l) {
        let d: f64 = m[1].parse().ok()?;
        let tpi: f64 = m[2].parse().ok()?;
        return Some(Answer::new("screw thread", vec![
            input(format!("Unified thread, major diameter {} in, {} threads per inch", n(d), n(tpi))),
            primary(pod("Result", "Thread dimensions", rows(thread_rows(d, tpi)))),
            pod("Formula", "Formulas", text("P = 1/n; H = 0.866025 P; pitch diameter D − 0.649519 P; basic minor diameter D − 1.082532 P; tensile stress area (π/4)(D − 0.9743/n)²")),
        ]));
    }
    let m = re!(r"^(?:what is the\s+)?(?:(thread count|threads per inch|tpi|minor diameter|major diameter|pitch diameter|tensile (?:stress )?area|tap drill(?: size)?)\s+(?:of\s+)?(?:an?\s+)?)?(coarse|fine|unc|unf)?\s*(?:(?:#|number\s+|no\.?\s*)(\d+)|(\d+(?:-\d+)?/\d+|1)(?:\s*(?:-|inch|in|\x22)\s*)*)(?:-(\d+))?\s*(coarse|fine|unc|unf)?\s*(?:bolt|screw|thread|machine screw)s?(?:\s*/.*)?$").captures(l)?;
    let name = match (m.get(3), m.get(4)) {
        (Some(k), _) => format!("#{}", k.as_str()),
        (_, Some(f)) => f.as_str().replace(['"', ' '], ""),
        _ => return None,
    };
    let &(nm, d, c, f) = UTS.iter().find(|u| u.0 == name)?;
    let series = m.get(2).or(m.get(6)).map(|x| x.as_str());
    let tpi = match (m.get(5), series) {
        (Some(t), _) => t.as_str().parse().ok()?,
        (_, Some("fine" | "unf")) => f,
        _ => if c > 0 { c } else { f },
    };
    let ser = if tpi == c { "UNC (coarse)" } else if tpi == f { "UNF (fine)" } else { "special pitch" };
    let r = thread_rows(d, tpi as f64);
    let what = m.get(1).map(|x| x.as_str()).unwrap_or("");
    let main = if what.starts_with("thread") || what == "tpi" {
        format!("{tpi} threads per inch")
    } else if what.starts_with("minor") {
        r[5].1.clone()
    } else if what.starts_with("major") {
        r[0].1.clone()
    } else if what.starts_with("pitch") {
        r[3].1.clone()
    } else if what.starts_with("tensile") {
        r[7].1.clone()
    } else {
        format!("{nm}-{tpi} {ser}: major diameter {}", r[0].1)
    };
    Some(Answer::new(
        "screw thread",
        vec![
            input(format!("{nm}-{tpi} {ser} screw thread")),
            primary(pod("Result", "Result", text(main))),
            pod("Dimensions", "Thread dimensions", rows(r)),
            pod("Series", "Standard pitches", text(format!("{nm}: UNC {}, UNF {} threads per inch", if c > 0 { c.to_string() } else { "—".into() }, f))),
        ],
    ))
}

// ---------------------------------------------------------------------------
// log scaling rules (board feet in a log)
// ---------------------------------------------------------------------------

fn log_rules(l: &str) -> Option<Answer> {
    let generic = matches!(l, "doyle rule" | "doyle log rule" | "scribner rule" | "scribner log rule" | "international log rule" | "log rules" | "log scale");
    let (d_in, len_ft) = if generic {
        (16.0, 16.0)
    } else {
        let m = re!(r"^(?:lumber volume|log volume|board feet(?: in a log)?|log scale|doyle rule|scribner rule|log rules?)[,:]?\s*(?:of a log\s+)?(?:with\s+)?(?:small[- ]end\s+)?diameter\s*=?\s*(\d+(?:\.\d+)?)\s*(in|inch|inches|ft|feet|foot|cm)?,?\s*(?:and\s+)?length\s*=?\s*(\d+(?:\.\d+)?)\s*(ft|feet|foot|m|meters?)?$").captures(l)?;
        let d: f64 = m[1].parse().ok()?;
        let d = match m.get(2).map(|x| x.as_str()) { Some("ft" | "feet" | "foot") => d * 12.0, Some("cm") => d / 2.54, _ => d };
        let len: f64 = m[3].parse().ok()?;
        let len = match m.get(4).map(|x| x.as_str()) { Some(u) if u.starts_with('m') => len / 0.3048, _ => len };
        (d, len)
    };
    let doyle = (d_in - 4.0).powi(2) * len_ft / 16.0;
    let scribner = ((0.79 * d_in * d_in - 2.0 * d_in - 4.0) * len_ft / 16.0).max(0.0);
    // International 1/4-inch rule: 4-ft sections, diameter grows 1/2 inch per section
    let mut intl = 0.0;
    let mut rem = len_ft;
    let mut d = d_in;
    while rem >= 4.0 - 1e-9 {
        intl += 0.22 * d * d - 0.71 * d;
        d += 0.5;
        rem -= 4.0;
    }
    if rem > 1e-9 {
        intl += (0.22 * d * d - 0.71 * d) * rem / 4.0;
    }
    let cubic = std::f64::consts::PI * (d_in / 24.0).powi(2) * len_ft;
    Some(Answer::new(
        "log rules",
        vec![
            input(format!("log, small-end diameter {} in, length {} ft", n(d_in), n(len_ft))),
            primary(pod("Result", "Board feet", rows(vec![("Doyle rule", format!("{} board feet", n(doyle.round()))), ("Scribner (decimal C formula)", format!("{} board feet", n(scribner.round()))), ("International ¼-inch rule", format!("{} board feet", n(intl.round())))]))),
            pod("Cylinder", "Cylinder volume", text(format!("{} ft³ ({} m³)", n((cubic * 100.0).round() / 100.0), n((cubic * 0.0283168 * 1000.0).round() / 1000.0)))),
            pod("Formula", "Formulas", text("Doyle: (D − 4)² L/16; Scribner: (0.79 D² − 2D − 4) L/16; International ¼″: Σ (0.22 D² − 0.71 D) per 4-ft section with ½″ taper; D in inches, L in feet")),
        ],
    ))
}

// ---------------------------------------------------------------------------
// psychrometrics
// ---------------------------------------------------------------------------

fn temp_c(s: &str) -> Option<f64> {
    let m = re!(r"(-?\d+(?:\.\d+)?)\s*(?:°\s*|degrees?\s*)?(f|c|k)\b").captures(s)?;
    let v: f64 = m[1].parse().ok()?;
    Some(match &m[2] { "f" => (v - 32.0) / 1.8, "k" => v - 273.15, _ => v })
}

/// saturation vapour pressure over water, kPa (Buck 1996)
fn es(t: f64) -> f64 {
    0.61121 * ((18.678 - t / 234.5) * (t / (257.14 + t))).exp()
}

fn dew_from_e(e: f64) -> f64 {
    // invert Magnus form numerically (bisection on es)
    let (mut lo, mut hi) = (-80.0, 80.0);
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        if es(mid) < e { lo = mid } else { hi = mid }
    }
    (lo + hi) / 2.0
}

fn psychrometrics(l: &str) -> Option<Answer> {
    let has = |w: &str| l.contains(w);
    if has("absorption") || has("sound") {
        return None;
    }
    if !(has("dry bulb") || has("drybulb") || has("dew point") || has("dewpoint") || has("moist air") || has("psychrometric") || has("wet bulb") || has("wetbulb") || has("humidity ratio")) {
        return None;
    }
    let db = re!(r"(?:dry[- ]?bulb|temperature|temp|air)\s*(?:of\s*|=\s*)?(-?\d+(?:\.\d+)?\s*(?:°\s*|degrees?\s*)?[fck])\b").captures(l).and_then(|m| temp_c(&m[1]));
    let wb = re!(r"wet[- ]?bulb\s*(?:of\s*|=\s*)?(-?\d+(?:\.\d+)?\s*(?:°\s*|degrees?\s*)?[fck])\b").captures(l).and_then(|m| temp_c(&m[1]));
    let rh = re!(r"(\d+(?:\.\d+)?)\s*%").captures(l).and_then(|m| m[1].parse::<f64>().ok());
    let db = db.or_else(|| {
        // "dew point 72F 50% humidity": the first temperature is the air temperature
        if wb.is_none() { re!(r"(-?\d+(?:\.\d+)?\s*(?:°\s*|degrees?\s*)?[fck])\b").captures(l).and_then(|m| temp_c(&m[1])) } else { None }
    })?;
    let p = if let Some(m) = re!(r"(\d+(?:\.\d+)?)\s*(inhg|in hg|kpa|hpa|mbar|psi|atm|mmhg)").captures(l) {
        let v: f64 = m[1].parse().ok()?;
        v * match &m[2] { "inhg" | "in hg" => 3.386_389, "hpa" | "mbar" => 0.1, "psi" => 6.894_757, "atm" => 101.325, "mmhg" => 0.133_322, _ => 1.0 }
    } else {
        101.325
    };
    let e = match (wb, rh) {
        (Some(w), _) => es(w) - 0.000_662 * p * (db - w),
        (None, Some(r)) => es(db) * r / 100.0,
        _ => return None,
    };
    if e <= 0.0 {
        return None;
    }
    let rhv = 100.0 * e / es(db);
    let dew = dew_from_e(e);
    let w = 0.621_945 * e / (p - e);
    let h = 1.006 * db + w * (2501.0 + 1.86 * db);
    let rho = (p - e) * 1000.0 / (287.058 * (db + 273.15)) + e * 1000.0 / (461.495 * (db + 273.15));
    let f = |c: f64| c * 1.8 + 32.0;
    let both = |c: f64| format!("{} °C ({} °F)", n((c * 10.0).round() / 10.0), n((f(c) * 10.0).round() / 10.0));
    let main = if (has("dew point") || has("dewpoint")) && wb.is_none() { format!("dew point {}", both(dew)) } else { format!("relative humidity {}%, dew point {}", n((rhv * 10.0).round() / 10.0), both(dew)) };
    let mut r = vec![("dry-bulb temperature", both(db))];
    if let Some(w) = wb {
        r.push(("wet-bulb temperature", both(w)));
    }
    r.extend([
        ("pressure", format!("{} kPa ({} inHg)", n((p * 1000.0).round() / 1000.0), n((p / 3.386_389 * 100.0).round() / 100.0))),
        ("relative humidity", format!("{}%", n((rhv * 10.0).round() / 10.0))),
        ("dew point", both(dew)),
        ("vapour pressure", format!("{} kPa", n((e * 1e4).round() / 1e4))),
        ("humidity ratio", format!("{} kg/kg ({} grains/lb)", n((w * 1e5).round() / 1e5), n((w * 7000.0 * 10.0).round() / 10.0))),
        ("specific enthalpy", format!("{} kJ/kg dry air", n((h * 10.0).round() / 10.0))),
        ("moist air density", format!("{} kg/m³", n((rho * 1e4).round() / 1e4))),
    ]);
    Some(Answer::new(
        "psychrometrics",
        vec![
            input("moist air properties"),
            primary(pod("Result", "Result", text(main))),
            pod("Properties", "Moist air properties", rows(r)),
            pod("Method", "Method", text("Buck saturation vapour pressure; ventilated psychrometer equation e = e_s(T_w) − 6.62×10⁻⁴ p (T − T_w)")),
        ],
    ))
}

/// ISO 9613-1 atmospheric absorption of sound, dB per metre.
fn iso9613(f: f64, t_c: f64, rh: f64, pa_kpa: f64) -> f64 {
    let t = t_c + 273.15;
    let (t0, t01, pr) = (293.15, 273.16, 101.325);
    let c = -6.8346 * (t01 / t).powf(1.261) + 4.6151;
    let h = rh * 10f64.powf(c) * pr / pa_kpa;
    let pp = pa_kpa / pr;
    let fro = pp * (24.0 + 4.04e4 * h * (0.02 + h) / (0.391 + h));
    let frn = pp * (t / t0).powf(-0.5) * (9.0 + 280.0 * h * (-4.170 * ((t / t0).powf(-1.0 / 3.0) - 1.0)).exp());
    8.686 * f * f * (1.84e-11 / pp * (t / t0).sqrt() + (t / t0).powf(-2.5) * (0.01275 * (-2239.1 / t).exp() / (fro + f * f / fro) + 0.1068 * (-3352.0 / t).exp() / (frn + f * f / frn)))
}

fn sound_absorption(l: &str) -> Option<Answer> {
    if !(l.contains("sound") && l.contains("absorption") && l.contains("air")) {
        return None;
    }
    let t = re!(r"(-?\d+(?:\.\d+)?)\s*(?:°\s*|degrees?\s*)?([fck])\b").captures(l).and_then(|m| temp_c(&m[0])).unwrap_or(20.0);
    let rh: f64 = re!(r"(\d+(?:\.\d+)?)\s*%").captures(l).and_then(|m| m[1].parse().ok()).unwrap_or(50.0);
    let f_given: Option<f64> = re!(r"(\d+(?:\.\d+)?)\s*(hz|khz)\b").captures(l).and_then(|m| m[1].parse::<f64>().ok().map(|v| if &m[2] == "khz" { v * 1e3 } else { v }));
    let freqs = [63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0];
    let table_rows: Vec<Vec<Content>> = freqs.iter().map(|&f| {
        let a = iso9613(f, t, rh, 101.325);
        vec![text(format!("{} Hz", n(f))), text(format!("{} dB/km", n((a * 1e3 * 100.0).round() / 100.0)))]
    }).collect();
    let main = match f_given {
        Some(f) => format!("{} dB/km at {} Hz", n((iso9613(f, t, rh, 101.325) * 1e5).round() / 100.0), n(f)),
        None => format!("{} dB/km at 1 kHz", n((iso9613(1000.0, t, rh, 101.325) * 1e5).round() / 100.0)),
    };
    Some(Answer::new(
        "sound absorption in air",
        vec![
            input(format!("atmospheric absorption of sound | {} °C | {}% relative humidity | 1 atm", n(t), n(rh))),
            primary(pod("Result", "Absorption coefficient", text(main))),
            pod("Spectrum", "By frequency", table(Some(vec!["frequency", "absorption"]), table_rows)),
            pod("Method", "Method", text("ISO 9613-1: classical and rotational absorption plus oxygen and nitrogen vibrational relaxation")),
        ],
    ))
}

/// Ainslie & McColm (1998) absorption of sound in sea water, dB/km; f in kHz, depth in km.
fn seawater_absorption(f: f64, t: f64, s: f64, ph: f64, z: f64) -> f64 {
    let f1 = 0.78 * (s / 35.0).sqrt() * (t / 26.0).exp();
    let f2 = 42.0 * (t / 17.0).exp();
    let f2sq = f * f;
    0.106 * f1 * f2sq / (f1 * f1 + f2sq) * ((ph - 8.0) / 0.56).exp() + 0.52 * (1.0 + t / 43.0) * (s / 35.0) * f2 * f2sq / (f2 * f2 + f2sq) * (-z / 6.0).exp() + 0.00049 * f2sq * (-(t / 27.0 + z / 17.0)).exp()
}

fn sea_absorption(l: &str) -> Option<Answer> {
    if !(l.contains("sound") && l.contains("absorption") && (l.contains("sea") || l.contains("ocean") || l.contains("seawater"))) {
        return None;
    }
    let t = re!(r"(-?\d+(?:\.\d+)?)\s*(?:°\s*|degrees?\s*)?([fck])\b").captures(l).and_then(|m| temp_c(&m[0])).unwrap_or(10.0);
    let sal: f64 = re!(r"salinity\s*(?:of\s*)?(\d+(?:\.\d+)?)").captures(l).and_then(|m| m[1].parse().ok()).unwrap_or(35.0);
    let ph: f64 = re!(r"ph\s*(?:of\s*)?(\d+(?:\.\d+)?)").captures(l).and_then(|m| m[1].parse().ok()).unwrap_or(8.0);
    let z: f64 = re!(r"(\d+(?:\.\d+)?)\s*(m|km)\s*(?:deep|depth)|depth\s*(?:of\s*)?(\d+(?:\.\d+)?)\s*(m|km)").captures(l).and_then(|m| {
        let (v, u) = if m.get(1).is_some() { (m[1].parse::<f64>().ok()?, m[2].to_string()) } else { (m[3].parse::<f64>().ok()?, m[4].to_string()) };
        Some(if u == "m" { v / 1000.0 } else { v })
    }).unwrap_or(0.0);
    let freqs = [0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0];
    let rows_t: Vec<Vec<Content>> = freqs.iter().map(|&f| vec![text(format!("{} kHz", n(f))), text(format!("{} dB/km", n((seawater_absorption(f, t, sal, ph, z) * 1e4).round() / 1e4)))]).collect();
    let f_given: Option<f64> = re!(r"(\d+(?:\.\d+)?)\s*(hz|khz)\b").captures(l).and_then(|m| m[1].parse::<f64>().ok().map(|v| if &m[2] == "hz" { v / 1e3 } else { v }));
    let f = f_given.unwrap_or(10.0);
    Some(Answer::new(
        "sound absorption in sea water",
        vec![
            input(format!("absorption of sound in sea water | {} °C | salinity {} | pH {} | depth {} km", n(t), n(sal), n(ph), n(z))),
            primary(pod("Result", "Absorption coefficient", text(format!("{} dB/km at {} kHz", n((seawater_absorption(f, t, sal, ph, z) * 1e3).round() / 1e3), n(f))))),
            pod("Spectrum", "By frequency", table(Some(vec!["frequency", "absorption"]), rows_t)),
            pod("Method", "Method", text("Ainslie & McColm (1998): boric acid and magnesium sulfate relaxation plus pure-water viscosity; defaults 10 °C, salinity 35, pH 8, surface")),
        ],
    ))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    inflation(l).or_else(|| min_wage(l)).or_else(|| sea_absorption(l)).or_else(|| sound_absorption(l)).or_else(|| shoe_sizes(l)).or_else(|| bolts(l)).or_else(|| log_rules(l)).or_else(|| psychrometrics(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpi_table_years() {
        assert_eq!(cpi(1913), Some(9.9));
        assert_eq!(cpi(1980), Some(82.4));
        assert_eq!(cpi(2000), Some(172.2));
        assert_eq!(cpi(2024), Some(313.689));
    }

    #[test]
    fn iso9613_reference() {
        // ISO 9613-1 table: 20 °C, 70% RH, 1 kHz → about 4.98 dB/km
        let a = iso9613(1000.0, 20.0, 70.0, 101.325) * 1e3;
        assert!((a - 4.98).abs() < 0.1, "{a}");
    }

    #[test]
    fn psychrometric_values() {
        // 25 °C at 50% relative humidity: dew point about 13.9 °C
        let e = es(25.0) * 0.5;
        assert!((dew_from_e(e) - 13.86).abs() < 0.1);
    }
}
