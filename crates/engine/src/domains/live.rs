//! Live data from open, key-free web services (optional, `net` feature):
//! weather (Open-Meteo), currency exchange rates (European Central Bank via
//! Frankfurter) and encyclopedia summaries (Wikipedia).
//!
//! Set `NOESIS_OFFLINE=1` (or turn off "online data" in the app) to disable
//! all network access.

use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

static ONLINE: AtomicBool = AtomicBool::new(true);

pub fn set_online(on: bool) {
    ONLINE.store(on, Ordering::Relaxed);
}

pub fn online() -> bool {
    cfg!(feature = "net") && ONLINE.load(Ordering::Relaxed) && std::env::var("NOESIS_OFFLINE").is_err()
}

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

#[cfg(feature = "net")]
fn get_json(url: &str) -> Option<serde_json::Value> {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    let agent = AGENT.get_or_init(|| ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(8)).try_proxy_from_env(true).user_agent(concat!("noesis/", env!("CARGO_PKG_VERSION"))).build());
    agent.get(url).call().ok()?.into_json().ok()
}

#[cfg(not(feature = "net"))]
fn get_json(_url: &str) -> Option<serde_json::Value> {
    None
}

fn enc(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            b' ' => "%20".into(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn source_pod(source: &str) -> Pod {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M");
    pod("Source", "Source", text(format!("{source} · retrieved {now}")))
}

// ---------------------------------------------------------------------------
// weather
// ---------------------------------------------------------------------------

fn wmo(code: i64) -> &'static str {
    match code {
        0 => "clear sky",
        1 => "mainly clear",
        2 => "partly cloudy",
        3 => "overcast",
        45 | 48 => "fog",
        51 | 53 | 55 => "drizzle",
        56 | 57 => "freezing drizzle",
        61 => "light rain",
        63 => "rain",
        65 => "heavy rain",
        66 | 67 => "freezing rain",
        71 => "light snow",
        73 => "snow",
        75 => "heavy snow",
        77 => "snow grains",
        80..=82 => "rain showers",
        85 | 86 => "snow showers",
        95 => "thunderstorm",
        96 | 99 => "thunderstorm with hail",
        _ => "unknown",
    }
}

fn locate(place: &str) -> Option<(String, f64, f64)> {
    if let Some(c) = super::geo::find_city(place) {
        let country = super::geo::countries().get(&c.country).map(|x| x.name.clone()).unwrap_or_default();
        return Some((format!("{}, {}", c.name, country), c.lat, c.lon));
    }
    let v = get_json(&format!("https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json", enc(place.trim())))?;
    let r = v.get("results")?.get(0)?;
    let name = format!("{}, {}", r.get("name")?.as_str()?, r.get("country").and_then(|c| c.as_str()).unwrap_or(""));
    Some((name, r.get("latitude")?.as_f64()?, r.get("longitude")?.as_f64()?))
}

fn weather(q: &Query) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(?:current\s+)?(weather|temperature|forecast|weather forecast)\s+(?:today\s+|now\s+|tomorrow\s+)?(?:in|at|for)\s+(.+?)(?:\s+(?:today|now|tomorrow|this week))?$").captures(&q.lower)?;
    let place = m.get(2)?.as_str();
    let (name, lat, lon) = locate(place)?;
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,wind_speed_10m,wind_direction_10m,precipitation&daily=temperature_2m_max,temperature_2m_min,weather_code,precipitation_sum&timezone=auto&forecast_days=7"
    );
    let v = get_json(&url)?;
    let cur = v.get("current")?;
    let f = |k: &str| cur.get(k).and_then(|x| x.as_f64());
    let t = f("temperature_2m")?;
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("weather | {name}")))];
    let cond = wmo(cur.get("weather_code").and_then(|x| x.as_i64()).unwrap_or(-1));
    pods.push(primary(pod("Current", "Latest recorded weather", text(format!("{} °C ({} °F), {cond}", fmt_float(t, 3), fmt_float(t * 9.0 / 5.0 + 32.0, 3))))));
    let mut rows = Vec::new();
    if let Some(a) = f("apparent_temperature") {
        rows.push(vec![text("feels like"), text(format!("{} °C", fmt_float(a, 3)))]);
    }
    if let Some(h) = f("relative_humidity_2m") {
        rows.push(vec![text("relative humidity"), text(format!("{}%", fmt_float(h, 3)))]);
    }
    if let Some(w) = f("wind_speed_10m") {
        let dir = f("wind_direction_10m").map(|d| format!(" from {:.0}°", d)).unwrap_or_default();
        rows.push(vec![text("wind"), text(format!("{} km/h{dir}", fmt_float(w, 3)))]);
    }
    if let Some(p) = f("precipitation") {
        rows.push(vec![text("precipitation (last hour)"), text(format!("{} mm", fmt_float(p, 3)))]);
    }
    pods.push(pod("Details", "Conditions", table(None, rows)));
    if let Some(d) = v.get("daily") {
        let arr = |k: &str| d.get(k).and_then(|x| x.as_array()).cloned().unwrap_or_default();
        let (days, hi, lo, codes) = (arr("time"), arr("temperature_2m_max"), arr("temperature_2m_min"), arr("weather_code"));
        let mut rows = Vec::new();
        let mut his = Vec::new();
        let mut los = Vec::new();
        for k in 0..days.len().min(hi.len()).min(lo.len()) {
            let day = days[k].as_str().unwrap_or("");
            let label = chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").map(|d| d.format("%a %b %-d").to_string()).unwrap_or_else(|_| day.to_string());
            let (h, l) = (hi[k].as_f64().unwrap_or(f64::NAN), lo[k].as_f64().unwrap_or(f64::NAN));
            his.push([k as f64, h]);
            los.push([k as f64, l]);
            rows.push(vec![text(label), text(format!("{} °C", fmt_float(h, 3))), text(format!("{} °C", fmt_float(l, 3))), text(wmo(codes.get(k).and_then(|c| c.as_i64()).unwrap_or(-1)))]);
        }
        pods.push(pod("Forecast", "Forecast for the next 7 days", table(Some(vec!["day", "high", "low", "conditions"]), rows)));
        if his.len() > 1 {
            let mut p = crate::plot::scatter(vec![], "", None);
            p.kind = crate::plot::PlotKind::Cartesian;
            p.series = vec![crate::plot::Series { label: "high °C".into(), segments: vec![his.clone()], points: false }, crate::plot::Series { label: "low °C".into(), segments: vec![los.clone()], points: false }];
            p.x_range = (0.0, (his.len() - 1) as f64);
            let all: Vec<f64> = his.iter().chain(los.iter()).map(|q| q[1]).filter(|v| v.is_finite()).collect();
            let (mn, mx) = (all.iter().cloned().fold(f64::INFINITY, f64::min), all.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
            p.y_range = (mn - 2.0, mx + 2.0);
            p.x_label = "days from today".into();
            p.y_label = "°C".into();
            pods.push(pod("ForecastPlot", "Temperature forecast", plot(p)));
        }
    }
    pods.push(pod("Location", "Location", text(format!("{name} ({lat:.2}°, {lon:.2}°)"))));
    pods.push(source_pod("Open-Meteo (open-meteo.com), CC BY 4.0"));
    Some(Answer::new(format!("weather in {name}"), pods))
}

// ---------------------------------------------------------------------------
// currency
// ---------------------------------------------------------------------------

const CURRENCIES: &[(&str, &str)] = &[
    ("dollar", "USD"),
    ("dollars", "USD"),
    ("us dollars", "USD"),
    ("euro", "EUR"),
    ("euros", "EUR"),
    ("pound", "GBP"),
    ("pounds", "GBP"),
    ("sterling", "GBP"),
    ("yen", "JPY"),
    ("yuan", "CNY"),
    ("renminbi", "CNY"),
    ("rupee", "INR"),
    ("rupees", "INR"),
    ("ringgit", "MYR"),
    ("won", "KRW"),
    ("franc", "CHF"),
    ("francs", "CHF"),
    ("real", "BRL"),
    ("reais", "BRL"),
    ("peso", "MXN"),
    ("pesos", "MXN"),
    ("baht", "THB"),
    ("rupiah", "IDR"),
    ("lira", "TRY"),
    ("zloty", "PLN"),
    ("krona", "SEK"),
    ("krone", "NOK"),
    ("rand", "ZAR"),
    ("ruble", "RUB"),
    ("rubles", "RUB"),
    ("rouble", "RUB"),
    ("roubles", "RUB"),
    ("dirham", "AED"),
    ("dirhams", "AED"),
    ("riyal", "SAR"),
    ("riyals", "SAR"),
    ("shekel", "ILS"),
    ("shekels", "ILS"),
    ("forint", "HUF"),
    ("koruna", "CZK"),
    ("kronor", "SEK"),
    ("kroner", "NOK"),
    ("hryvnia", "UAH"),
    ("naira", "NGN"),
    ("dong", "VND"),
    ("taka", "BDT"),
    ("$", "USD"),
    ("€", "EUR"),
    ("£", "GBP"),
    ("¥", "JPY"),
];

const ISO: &[&str] = &["USD", "EUR", "GBP", "JPY", "CNY", "INR", "MYR", "KRW", "CHF", "BRL", "MXN", "THB", "IDR", "TRY", "PLN", "SEK", "NOK", "DKK", "ZAR", "AUD", "CAD", "NZD", "SGD", "HKD", "PHP", "CZK", "HUF", "ILS", "ISK", "RON", "BGN"];

/// "Mexican pesos", "Canadian dollar", "Swiss francs"
const NATIONAL: &[(&str, &str)] = &[
    ("us", "USD"), ("u.s.", "USD"), ("american", "USD"), ("united states", "USD"), ("canadian", "CAD"), ("australian", "AUD"),
    ("new zealand", "NZD"), ("hong kong", "HKD"), ("singapore", "SGD"), ("singaporean", "SGD"), ("taiwan", "TWD"), ("taiwanese", "TWD"),
    ("mexican", "MXN"), ("argentine", "ARS"), ("argentinian", "ARS"), ("chilean", "CLP"), ("colombian", "COP"), ("philippine", "PHP"),
    ("british", "GBP"), ("uk", "GBP"), ("swiss", "CHF"), ("japanese", "JPY"), ("chinese", "CNY"), ("indian", "INR"), ("pakistani", "PKR"),
    ("russian", "RUB"), ("south african", "ZAR"), ("brazilian", "BRL"), ("swedish", "SEK"), ("norwegian", "NOK"), ("danish", "DKK"),
    ("polish", "PLN"), ("turkish", "TRY"), ("korean", "KRW"), ("south korean", "KRW"), ("thai", "THB"), ("indonesian", "IDR"),
    ("malaysian", "MYR"), ("czech", "CZK"), ("hungarian", "HUF"), ("israeli", "ILS"), ("egyptian", "EGP"), ("nigerian", "NGN"),
    ("vietnamese", "VND"), ("ukrainian", "UAH"), ("saudi", "SAR"), ("emirati", "AED"), ("uae", "AED"), ("bangladeshi", "BDT"),
];

fn currency_code(w: &str) -> Option<&'static str> {
    let l = w.trim().to_lowercase();
    if let Some((nat, unit)) = NATIONAL.iter().filter_map(|(n, c)| l.strip_prefix(n).filter(|u| u.starts_with(' ')).map(|u| (c, u.trim()))).next() {
        let base = currency_code(unit)?;
        // the unit word must name the same kind of currency ("Mexican pesos", not "Mexican dollars")
        let kind = |c: &str| CURRENCIES.iter().filter(|(_, k)| *k == c).map(|(n, _)| n.trim_end_matches('s')).collect::<Vec<_>>();
        let word = unit.trim_end_matches('s');
        return if base == *nat || kind(base).contains(&word) && (["dollar", "peso", "franc", "pound", "rupee", "krona", "krone", "kroner", "kronor", "won", "yen", "yuan", "real", "rand", "ruble", "rouble", "lira", "dirham", "riyal"].contains(&word)) { Some(nat) } else { None };
    }
    if let Some((_, c)) = CURRENCIES.iter().find(|(n, _)| *n == l) {
        return Some(c);
    }
    ISO.iter().find(|c| c.eq_ignore_ascii_case(&l)).copied()
}

/// Approximate units of each currency per US dollar (mid-2025), used only
/// when live rates cannot be fetched.
const APPROX_PER_USD: &[(&str, f64)] = &[
    ("USD", 1.0), ("EUR", 0.86), ("GBP", 0.74), ("JPY", 147.0), ("CNY", 7.17), ("INR", 86.5), ("CAD", 1.37), ("AUD", 1.53), ("NZD", 1.68), ("CHF", 0.80),
    ("HKD", 7.85), ("SGD", 1.28), ("KRW", 1385.0), ("MXN", 18.8), ("BRL", 5.5), ("ZAR", 17.8), ("SEK", 9.5), ("NOK", 10.1), ("DKK", 6.4), ("PLN", 3.65),
    ("TRY", 40.5), ("THB", 32.5), ("IDR", 16300.0), ("MYR", 4.25), ("PHP", 57.0), ("CZK", 21.3), ("HUF", 345.0), ("ILS", 3.4), ("AED", 3.6725), ("SAR", 3.75),
    ("RUB", 79.0), ("TWD", 29.5), ("PKR", 283.0), ("EGP", 49.5), ("NGN", 1530.0), ("VND", 26100.0), ("UAH", 41.5), ("BDT", 122.0), ("ARS", 1200.0),
    ("CLP", 940.0), ("COP", 4050.0), ("ISK", 122.0), ("RON", 4.35), ("BGN", 1.68),
];

/// Units of each currency per US dollar: live ECB rates when online, else the
/// approximate table. The second value is the note on where the rates came from.
fn per_usd(codes: &[&str]) -> Option<(Vec<f64>, String)> {
    let others: Vec<&str> = codes.iter().copied().filter(|c| *c != "USD").collect();
    if online() && !others.is_empty() {
        if let Some(v) = get_json(&format!("https://api.frankfurter.dev/v1/latest?base=USD&symbols={}", others.join(","))) {
            let rates = v.get("rates");
            let got: Option<Vec<f64>> = codes.iter().map(|c| if *c == "USD" { Some(1.0) } else { rates?.get(*c)?.as_f64() }).collect();
            if let Some(g) = got {
                let date = v.get("date").and_then(|d| d.as_str()).unwrap_or("");
                return Some((g, format!("European Central Bank reference rates via Frankfurter (rates of {date})")));
            }
        }
    }
    let per = |c: &str| APPROX_PER_USD.iter().find(|(k, _)| *k == c).map(|(_, v)| *v);
    let got: Option<Vec<f64>> = codes.iter().map(|c| per(c)).collect();
    Some((got?, "approximate mid-2025 exchange rates (live rates could not be fetched); actual rates differ".to_string()))
}

/// "150 USD", "€80", "1,300 Mexican pesos", "20k yen"
fn money_term(t: &str) -> Option<(f64, &'static str)> {
    let m = re!(r"^([$€£¥])?\s*([\d,]*\.?\d+)\s*(k|thousand|million|billion)?\s*([a-z][a-z. ]*?)?$").captures(t.trim())?;
    let mut v: f64 = m[2].replace(',', "").parse().ok()?;
    v *= match m.get(3).map(|x| x.as_str()) { Some("k" | "thousand") => 1e3, Some("million") => 1e6, Some("billion") => 1e9, _ => 1.0 };
    let code = match (m.get(1), m.get(4)) {
        (Some(sym), None) => currency_code(sym.as_str())?,
        (None, Some(w)) => currency_code(w.as_str())?,
        (Some(sym), Some(w)) => { let c = currency_code(w.as_str())?; if sym.as_str() == "$" && c.ends_with('D') { c } else { return None } }
        _ => return None,
    };
    Some((v, code))
}

/// "2500 euros", "150 USD + 80 EUR", "1,000 yuan + 20,000 yen in dollars",
/// "1300 Mexican pesos / 20,000 Russian rubles"
fn currency_expr(l: &str) -> Option<Answer> {
    let l = l.trim().trim_end_matches(['?', '.']);
    let (body, target) = match re!(r"^(.+?)\s+(?:in|to|into)\s+([a-z$€£¥][a-z. ]*)$").captures(l) {
        Some(m) => (m.get(1)?.as_str(), Some(currency_code(&m[2])?)),
        None => (l, None),
    };
    let parts: Vec<&str> = re!(r"\s+(\+|plus|-|minus|/|divided by)\s+").split(body).collect();
    let ops: Vec<&str> = re!(r"\s+(\+|plus|-|minus|/|divided by)\s+").captures_iter(body).map(|c| c.get(1).unwrap().as_str()).collect();
    let terms: Option<Vec<(f64, &str)>> = parts.iter().map(|p| money_term(p)).collect();
    let terms = terms?;
    let ratio = ops.iter().any(|o| *o == "/" || *o == "divided by");
    if ratio && (ops.len() != 1 || target.is_some()) {
        return None;
    }
    // a single amount is a conversion to dollars (or from dollars to euros)
    let target = target.unwrap_or(if terms[0].1 == "USD" && terms.len() == 1 { "EUR" } else if terms.len() == 1 { "USD" } else { terms[0].1 });
    if terms.len() == 1 && terms[0].1 == target {
        return None;
    }
    // "5 pounds" alone is a mass, "10 won" and "3 real" are words
    if terms.len() == 1 && re!(r"^[\d,.]+\s*(?:pounds?|real|won|rand|dong|lira)$").is_match(body.trim()) {
        return None;
    }
    let mut codes: Vec<&str> = terms.iter().map(|t| t.1).collect();
    codes.push(target);
    for extra in ["USD", "EUR", "GBP", "JPY", "CNY"] {
        codes.push(extra);
    }
    let (rates, note) = per_usd(&codes)?;
    let usd: Vec<f64> = terms.iter().zip(&rates).map(|((v, _), r)| v / r).collect();
    let input = terms.iter().map(|(v, c)| format!("{} {c}", fmt_float(*v, 12))).collect::<Vec<_>>();
    let mut joined = input[0].clone();
    for (k, o) in ops.iter().enumerate() {
        let sym = match *o { "+" | "plus" => "+", "-" | "minus" => "−", _ => "/" };
        joined.push_str(&format!(" {sym} {}", input[k + 1]));
    }
    let money = |x: f64, c: &str| if x.abs() < 1.0 { format!("{} {c}", fmt_float(x, 3)) } else { format!("{} {c}", fmt_float((x * 100.0).round() / 100.0, 12)) };
    if ratio {
        let r = usd[0] / usd[1];
        return Some(Answer::new("currency ratio", vec![
            pod("Input", "Input interpretation", text(joined)),
            primary(pod("Result", "Result", text(fmt_float(r, 4)))),
            pod("Values", "Values in US dollars", text(format!("{} / {}", money(usd[0], "USD"), money(usd[1], "USD")))),
            pod("Note", "Exchange rates", text(note)),
        ]));
    }
    let mut total = usd[0];
    for (k, o) in ops.iter().enumerate() {
        total += if *o == "-" || *o == "minus" { -usd[k + 1] } else { usd[k + 1] };
    }
    let tr = rates[terms.len()];
    let shown = if terms.len() == 1 { joined.clone() } else { format!("{joined} in {target}") };
    let mut conv = vec![];
    for (k, c) in codes.iter().enumerate().skip(terms.len() + 1) {
        if *c != target && !conv.iter().any(|r: &(String, String)| r.0 == *c) {
            conv.push((c.to_string(), money(total * rates[k], c)));
        }
    }
    Some(Answer::new("currency conversion", vec![
        pod("Input", "Input interpretation", text(format!("convert {shown}"))),
        primary(pod("Result", "Result", text(money(total * tr, target)))),
        pod("Table", "Other currencies", table(None, conv.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())),
        pod("Note", "Exchange rates", text(note)),
    ]))
}

fn offline_currency(amount: f64, from: &str, to: &str) -> Option<Answer> {
    let per = |c: &str| APPROX_PER_USD.iter().find(|(k, _)| *k == c).map(|(_, v)| *v);
    let rate = per(to)? / per(from)?;
    let pods = vec![
        pod("Input", "Input interpretation", text(format!("convert {} {from} to {to}", fmt_float(amount, 10)))),
        primary(pod("Result", "Approximate result", text(format!("≈ {} {to}", fmt_float(amount * rate, 4))))),
        pod("Rate", "Approximate exchange rate", text(format!("1 {from} ≈ {} {to}", fmt_float(rate, 4)))),
        pod("Note", "Note", text("Live exchange rates could not be fetched (offline mode or no network). This uses a built-in table of approximate mid-2025 rates; actual rates differ.")),
    ];
    Some(Answer::new(format!("{from} to {to} (approximate)"), pods))
}

fn currency(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    let (amount, from, to) = if let Some(m) = re!(r"^(?:convert\s+)?([$€£¥])?\s*([\d,]*\.?\d+)\s*([a-z$€£¥]+(?:\s+dollars)?)?\s+(?:in|to|into)\s+([a-z€£¥$]+(?:\s+dollars)?)$").captures(l) {
        let amt: f64 = m[2].replace(',', "").parse().ok()?;
        let from = m.get(1).map(|x| x.as_str()).or(m.get(3).map(|x| x.as_str()))?;
        (amt, currency_code(from)?, currency_code(&m[4])?)
    } else if let Some(m) = re!(r"^(?:exchange rate|rate)\s+(?:of\s+|from\s+)?([a-z]{3})\s*(?:to|/|and|-)\s*([a-z]{3})$").captures(l) {
        (1.0, currency_code(&m[1])?, currency_code(&m[2])?)
    } else {
        return currency_expr(l);
    };
    if from == to {
        return None;
    }
    let Some(v) = online().then(|| get_json(&format!("https://api.frankfurter.dev/v1/latest?base={from}&symbols={to}"))).flatten() else {
        return offline_currency(amount, from, to);
    };
    let rate = v.get("rates")?.get(to)?.as_f64()?;
    let date = v.get("date").and_then(|d| d.as_str()).unwrap_or("").to_string();
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("convert {} {from} to {to}", fmt_float(amount, 10))))];
    pods.push(primary(pod("Result", "Result", text(format!("{} {to}", fmt_float(amount * rate, 8))))));
    pods.push(pod("Rate", "Exchange rate", text(format!("1 {from} = {} {to}   ·   1 {to} = {} {from}", fmt_float(rate, 6), fmt_float(1.0 / rate, 6)))));
    let rows: Vec<Vec<Content>> = [1.0, 10.0, 100.0, 1000.0].iter().map(|a| vec![text(format!("{} {from}", fmt_float(*a, 6))), text(format!("{} {to}", fmt_float(a * rate, 8)))]).collect();
    pods.push(pod("Table", "Conversions", table(None, rows)));
    // 30-day history
    let start = (chrono::Local::now().date_naive() - chrono::Duration::days(31)).format("%Y-%m-%d");
    if let Some(h) = get_json(&format!("https://api.frankfurter.dev/v1/{start}..?base={from}&symbols={to}")) {
        if let Some(obj) = h.get("rates").and_then(|r| r.as_object()) {
            let mut pts: Vec<[f64; 2]> = obj.iter().enumerate().filter_map(|(k, (_, r))| Some([k as f64, r.get(to)?.as_f64()?])).collect();
            pts.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
            if pts.len() > 2 {
                let mut p = crate::plot::scatter(vec![], "", None);
                p.kind = crate::plot::PlotKind::Cartesian;
                let (mn, mx) = pts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |a, q| (a.0.min(q[1]), a.1.max(q[1])));
                p.series = vec![crate::plot::Series { label: format!("{from}/{to}"), segments: vec![pts.clone()], points: false }];
                p.x_range = (0.0, pts.last().unwrap()[0]);
                let pad = (mx - mn).max(1e-9) * 0.15;
                p.y_range = (mn - pad, mx + pad);
                p.x_label = "trading days".into();
                p.y_label = to.into();
                pods.push(pod("History", "Last 30 days", plot(p)));
            }
        }
    }
    pods.push(source_pod(&format!("European Central Bank reference rates via Frankfurter (rates of {date})")));
    Some(Answer::new(format!("{from} to {to}"), pods))
}

// ---------------------------------------------------------------------------
// encyclopedia
// ---------------------------------------------------------------------------

fn encyclopedia(q: &Query) -> Option<Answer> {
    let raw_l = q.raw.trim().trim_end_matches(['?', '.', '!']).to_lowercase();
    let m = re!(r"^(?:who\s+(?:is|was|were|are)|what\s+(?:is|was|are|were)(?:\s+an?)?|what's(?:\s+an?)?|tell me about|define|definition of|information about|info on|wiki|wikipedia)\s+(.+?)$").captures(&raw_l)?;
    lookup(q, m.get(1)?.as_str())
}

/// Last-resort lookup for short word-only inputs nothing else understood.
pub fn fallback(q: &Query) -> Option<Answer> {
    if !online() {
        return None;
    }
    let l = q.lower.trim();
    let words = l.split_whitespace().count();
    if words == 0 || words > 6 || !l.chars().all(|c| c.is_alphabetic() || c == ' ' || c == '-' || c == '\'') {
        return None;
    }
    lookup(q, l.trim_start_matches("about "))
}

fn lookup(q: &Query, topic_in: &str) -> Option<Answer> {
    let topic_l = topic_in.trim().trim_start_matches("the ").trim();
    if topic_l.is_empty() || topic_l.chars().any(|c| "+*/^=<>".contains(c)) || topic_l.chars().next()?.is_ascii_digit() {
        return None;
    }
    // keep the user's capitalisation
    let src = q.raw.trim();
    let topic = match src.to_lowercase().rfind(topic_l) {
        Some(start) if src.is_char_boundary(start) && src.is_char_boundary(start + topic_l.len()) => &src[start..start + topic_l.len()],
        _ => topic_l,
    };
    let title = topic.split_whitespace().map(|w| w.to_string()).collect::<Vec<_>>().join("_");
    let mut v = get_json(&format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", enc(&title)));
    if v.as_ref().and_then(|x| x.get("extract")).is_none() {
        // search for the best-matching article
        let s = get_json(&format!("https://en.wikipedia.org/w/rest.php/v1/search/title?q={}&limit=1", enc(topic)))?;
        let key = s.get("pages")?.get(0)?.get("key")?.as_str()?.to_string();
        v = get_json(&format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", enc(&key)));
    }
    let v = v?;
    if v.get("type").and_then(|t| t.as_str()) == Some("disambiguation") {
        return None;
    }
    let extract = v.get("extract")?.as_str()?.to_string();
    let name = v.get("title").and_then(|t| t.as_str()).unwrap_or(topic).to_string();
    let mut pods = vec![pod("Input", "Input interpretation", text(name.clone()))];
    if let Some(d) = v.get("description").and_then(|d| d.as_str()) {
        pods.push(pod("Description", "Description", text(d.to_string())));
    }
    pods.push(primary(pod("Summary", "Summary", text(extract))));
    if let Some(u) = v.get("content_urls").and_then(|c| c.get("desktop")).and_then(|c| c.get("page")).and_then(|p| p.as_str()) {
        pods.push(pod("Link", "Read more", text(u.to_string())));
    }
    pods.push(source_pod("Wikipedia, CC BY-SA 4.0"));
    Some(Answer::new(name, pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    if !online() {
        // currency conversion still answers offline, from approximate rates
        return currency(q);
    }
    weather(q).or_else(|| currency(q)).or_else(|| encyclopedia(q))
}
