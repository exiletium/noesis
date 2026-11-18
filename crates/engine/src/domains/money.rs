//! Money calculators: time value of money from PV/FV/rate/periods in any
//! combination, loans with balloon payments and adjustable rates, credit
//! card payoff, salary conversions, advertising CPM, trip fuel cost, bond
//! prices and durations, Black–Scholes option prices, option strategy
//! payoffs, and US federal income tax (tax year 2026 brackets).

use crate::domains::misc::money;
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

fn num(s: &str) -> Option<f64> {
    let t = s.trim().trim_start_matches(['$', '€', '£']).replace(',', "");
    let (t, mult) = if let Some(x) = t.strip_suffix('k') {
        (x.to_string(), 1e3)
    } else if let Some(x) = t.strip_suffix(" million") {
        (x.to_string(), 1e6)
    } else if let Some(x) = t.strip_suffix('m') {
        (x.to_string(), 1e6)
    } else {
        (t, 1.0)
    };
    t.trim().parse::<f64>().ok().map(|v| v * mult)
}

fn rows(r: Vec<(&str, String)>) -> Content {
    table(None, r.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())
}

// ---------------------------------------------------------------------------
// time value of money
// ---------------------------------------------------------------------------

fn field(l: &str, re: &Regex) -> Option<f64> {
    re.captures(l).and_then(|m| num(&m[1]))
}

/// Compounding periods per year (∞ for continuous).
fn compounding(l: &str) -> (f64, &'static str) {
    if l.contains("continuous") {
        (f64::INFINITY, "continuously")
    } else if l.contains("daily") {
        (365.0, "daily")
    } else if l.contains("weekly") {
        (52.0, "weekly")
    } else if l.contains("monthly") {
        (12.0, "monthly")
    } else if l.contains("quarterly") {
        (4.0, "quarterly")
    } else if l.contains("semiannual") || l.contains("semi-annual") || l.contains("biannual") {
        (2.0, "semiannually")
    } else {
        (1.0, "annually")
    }
}

fn growth(r: f64, m: f64, t: f64) -> f64 {
    if m.is_infinite() {
        (r * t).exp()
    } else {
        (1.0 + r / m).powf(m * t)
    }
}

fn tvm(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let simple = re!(r"^\$?([\d,.]+k?) (?:invested |deposited )?at ([\d.]+) ?(?:%|percent)(?: interest| annual interest| apr| a year| per year)?(?:,? compounded (\w+))? (?:for|over) ([\d.]+) years?(?:,? compounded (\w+))?$").captures(l);
    let (pv, fv, rate, t) = if let Some(m) = &simple {
        (num(&m[1]), None, num(&m[2]), num(&m[4]))
    } else {
        if !(l.contains("interest") || l.contains("pv") || l.contains("fv") || l.contains("present value") || l.contains("future value")) || l.contains("formula") && !l.contains('=') {
            return None;
        }
        let pv = field(l, re!(r"\b(?:pv|present value|principal)\s*(?:=|of|:)\s*\$?([\d,.]+k?)"));
        let fv = field(l, re!(r"\b(?:fv|future value)\s*(?:=|of|:)\s*\$?([\d,.]+k?)"));
        let rate = field(l, re!(r"\b(?:rate|i|r|interest rate|apr)\s*(?:=|of|:)\s*([\d.]+)\s*%?")).or_else(|| field(l, re!(r"([\d.]+)\s*%")));
        let t = field(l, re!(r"\b(?:n|t|periods|years|time|term)\s*(?:=|of|:)\s*([\d.]+)")).or_else(|| field(l, re!(r"([\d.]+)\s*years?\b")));
        (pv, fv, rate, t)
    };
    let (m, how) = match &simple {
        Some(c) => compounding(c.get(3).or(c.get(5)).map_or("", |x| x.as_str())),
        None => compounding(l),
    };
    let known = [pv.is_some(), fv.is_some(), rate.is_some(), t.is_some()].iter().filter(|b| **b).count();
    if known < 3 {
        return None;
    }
    let r = rate.map(|x| x / 100.0);
    let (pv, fv, r, t, solved) = match (pv, fv, r, t) {
        (Some(p), _, Some(r), Some(t)) => (p, p * growth(r, m, t), r, t, "future value"),
        (None, Some(f), Some(r), Some(t)) => (f / growth(r, m, t), f, r, t, "present value"),
        (Some(p), Some(f), Some(r), None) => {
            let g = (f / p).ln();
            let t = if m.is_infinite() { g / r } else { g / (m * (1.0 + r / m).ln()) };
            (p, f, r, t, "time")
        }
        (Some(p), Some(f), None, Some(t)) => {
            let g = (f / p).ln() / t;
            let r = if m.is_infinite() { g } else { m * ((g / m).exp() - 1.0) };
            (p, f, r, t, "interest rate")
        }
        _ => return None,
    };
    if !(pv.is_finite() && fv.is_finite() && r.is_finite() && t.is_finite()) {
        return None;
    }
    let ear = growth(r, m, 1.0) - 1.0;
    let headline = match solved {
        "future value" => money(fv),
        "present value" => money(pv),
        "time" => format!("{} years", n(t)),
        _ => format!("{}% per year", n(r * 100.0)),
    };
    let mut table_rows = Vec::new();
    let years = t.ceil().max(1.0) as usize;
    let step = (years / 20).max(1);
    for y in (0..=years).step_by(step) {
        table_rows.push(vec![text(y.to_string()), text(money(pv * growth(r, m, y as f64)))]);
    }
    let formula = if m.is_infinite() { "FV = PV e^(r t)".to_string() } else { format!("FV = PV (1 + r/{m})^({m} t)") };
    Some(Answer::new("time value of money", vec![
        input(format!("{} interest | present value: {} | future value: {} | rate: {}% | time: {} years", how, money(pv), money(fv), n(r * 100.0), n(t))),
        primary(pod("Result", &format!("{} (solved for)", solved), text(headline))),
        pod("Summary", "Summary", rows(vec![
            ("present value", money(pv)),
            ("future value", money(fv)),
            ("interest earned", money(fv - pv)),
            ("annual rate", format!("{}%", n(r * 100.0))),
            ("effective annual rate", format!("{}%", n(ear * 100.0))),
            ("time", format!("{} years", n(t))),
            ("doubling time", format!("{} years", n(2f64.ln() / (1.0 + ear).ln()))),
        ])),
        pod("Formula", "Formula", text(formula)),
        pod("Growth", "Balance by year", table(Some(vec!["year", "balance"]), table_rows)),
    ]))
}

// ---------------------------------------------------------------------------
// loans
// ---------------------------------------------------------------------------

fn payment(p: f64, r: f64, k: f64, balloon: f64) -> f64 {
    if r == 0.0 {
        (p - balloon) / k
    } else {
        (p - balloon / (1.0 + r).powf(k)) * r / (1.0 - (1.0 + r).powf(-k))
    }
}

fn loans(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let Some(m) = re!(r"^(adjustable(?:[- ]rate)? |arm |fixed(?:[- ]rate)? |balloon )?(?:home |car |auto |student )?(?:loan|mortgage)(?: of| for)? \$?([\d,.]+k?),? (?:at )?([\d.]+) ?(?:%|percent)(?: apr| interest)?,? (?:for |over )?([\d.]+) ?(years?|months?)(?:,? (?:with )?(?:an? )?\$?([\d,.]+k?) balloon(?: payment)?)?$").captures(l) else {
        return loan_defaults(l);
    };
    let principal = num(&m[2])?;
    let apr = num(&m[3])? / 100.0;
    let months = if m[5].starts_with('y') { num(&m[4])? * 12.0 } else { num(&m[4])? };
    let balloon = m.get(6).and_then(|b| num(b.as_str())).unwrap_or(0.0);
    let adjustable = m.get(1).is_some_and(|a| a.as_str().starts_with("adj") || a.as_str().starts_with("arm"));
    loan_answer(principal, apr, months, balloon, adjustable, None)
}

/// "mortgage", "car loan $20,000", "how much would I spend on a car with loan
/// amount $20,000", "student loan 30000": typical rates and terms are assumed.
fn loan_defaults(l: &str) -> Option<Answer> {
    let kind = if l.contains("mortgage") || l.contains("home loan") || l.contains("house") {
        "mortgage"
    } else if re!(r"\b(?:car|auto|vehicle)\b").is_match(l) {
        "car"
    } else if l.contains("student loan") {
        "student"
    } else {
        "loan"
    };
    if !re!(r"\b(?:loan|mortgage)\b").is_match(l) {
        return None;
    }
    let amount = field(l, re!(r"\$\s*([\d,.]+k?)")).or_else(|| field(l, re!(r"(?:loan amount|loan of|loan|mortgage(?: of)?|borrow(?:ing)?)\s*\$?([\d,.]+k?)\b")));
    let rate = field(l, re!(r"([\d.]+)\s*(?:%|percent)"));
    let term = re!(r"([\d.]+)[- ]?(years?|yr|months?)").captures(l).and_then(|c| Some(num(&c[1])? * if c[2].starts_with('m') { 1.0 } else { 12.0 }));
    // only the loan word and an amount, rate or term: "mortgage", "car loan $20,000"
    let rest = re!(r"\$?\s*[\d,.]+k?\s*(?:%|percent|[- ]?years?|[- ]?yr|[- ]?months?)?|\b(?:how much (?:would|will|do) i (?:spend|pay)|on an?|for an?|with an?|with|loan amount|amount|monthly payments?|payments? (?:on|for)|calculator|calculate|the|a|of|at|over|for|loan|mortgage|car|auto|vehicle|new|used|student|home|house|fixed|rate|interest|apr|term|borrow(?:ing)?)\b").replace_all(l, "");
    if !rest.trim().is_empty() {
        return None;
    }
    let (p0, r0, k0) = match kind {
        "mortgage" => (300_000.0, 6.5, 360.0),
        "car" => (30_000.0, 7.0, 60.0),
        "student" => (30_000.0, 6.5, 120.0),
        _ => (10_000.0, 10.0, 36.0),
    };
    let mut assumed = vec![];
    if amount.is_none() {
        assumed.push(format!("amount {}", money(p0)));
    }
    if rate.is_none() {
        assumed.push(format!("{}% APR", n(r0)));
    }
    if term.is_none() {
        assumed.push(format!("{} years", n(k0 / 12.0)));
    }
    let note = (!assumed.is_empty()).then(|| format!("Assumed typical values for a {}: {}. Give them in the query to change them, as in \"{} $250,000 at 6% for 15 years\".", if kind == "loan" { "personal loan".to_string() } else if kind == "mortgage" { "30-year fixed mortgage".to_string() } else { format!("{kind} loan") }, assumed.join(", "), if kind == "mortgage" { "mortgage" } else { "loan" }));
    loan_answer(amount.unwrap_or(p0), rate.unwrap_or(r0) / 100.0, term.unwrap_or(k0), 0.0, false, note)
}

fn loan_answer(principal: f64, apr: f64, months: f64, balloon: f64, adjustable: bool, note: Option<String>) -> Option<Answer> {
    let r = apr / 12.0;
    let pay = payment(principal, r, months, balloon);
    let total = pay * months + balloon;
    let mut pods = vec![
        input(format!("loan of {} at {}% APR over {} months{}", money(principal), n(apr * 100.0), n(months), if balloon > 0.0 { format!(" with a {} balloon payment", money(balloon)) } else { String::new() })),
        primary(pod("Result", "Monthly payment", text(money(pay)))),
        pod("Totals", "Totals", rows(vec![("total paid", money(total)), ("total interest", money(total - principal))])),
    ];
    // amortization by year
    let mut bal = principal;
    let mut am = Vec::new();
    for y in 1..=((months / 12.0).ceil() as usize) {
        let (mut ip, mut pp) = (0.0, 0.0);
        for _ in 0..12 {
            if bal <= balloon + 1e-6 && balloon == 0.0 {
                break;
            }
            let i = bal * r;
            ip += i;
            pp += pay - i;
            bal -= pay - i;
        }
        if y <= 5 || y % 5 == 0 || y == (months / 12.0).ceil() as usize {
            am.push(vec![text(y.to_string()), text(money(pp)), text(money(ip)), text(money(bal.max(0.0)))]);
        }
    }
    pods.push(pod("Amortization", "Amortization by year", table(Some(vec!["year", "principal", "interest", "balance"]), am)));
    if let Some(nt) = note {
        pods.push(pod("Assumptions", "Assumptions", text(nt)));
    }
    if adjustable {
        // 5/1 ARM scenarios: rate resets after five years
        let after = |rate: f64| {
            let mut b = principal;
            for _ in 0..60 {
                b -= pay - b * r;
            }
            payment(b, rate / 12.0, months - 60.0, 0.0)
        };
        let sc: Vec<Vec<Content>> = [-1.0, 0.0, 1.0, 2.0, 3.0, 5.0]
            .iter()
            .map(|d| vec![text(format!("{}%", n((apr * 100.0 + d).max(0.0)))), text(money(after((apr + d / 100.0).max(0.0))))])
            .collect();
        pods.push(pod("Adjustable", "Payment after a reset at year 5 (5/1 ARM)", table(Some(vec!["new rate", "monthly payment"]), sc)));
    }
    Some(Answer::new("loan payment", pods))
}

fn credit_card(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    if !l.contains("credit card") {
        return None;
    }
    let bal = field(l, re!(r"(?:balance(?: of)?|card|pay off|pay)\s*\$?([\d,.]+k?)")).or_else(|| field(l, re!(r"\$([\d,.]+k?)")))?;
    let apr_given = field(l, re!(r"(?:apr|interest|rate)\s*(?:of|=|:)?\s*([\d.]+)\s*%")).or_else(|| field(l, re!(r"([\d.]+)\s*%\s*(?:apr|interest)")));
    let apr = apr_given.unwrap_or(21.0) / 100.0;
    let r = apr / 12.0;
    let monthly = field(l, re!(r"\$?([\d,.]+)\s*(?:/|per |a )\s*(?:month|mo)\b"));
    let over = re!(r"(?:over|in|within)\s+([\d.]+)\s*(years?|months?)").captures(l).and_then(|c| Some(num(&c[1])? * if c[2].starts_with('y') { 12.0 } else { 1.0 }));
    let assumed = if apr_given.is_none() { " (assumed typical APR)" } else { "" };
    if let Some(p) = monthly {
        if p <= bal * r {
            return Some(Answer::new("credit card payoff", vec![
                input(format!("credit card | balance {} | APR {}%{assumed} | payment {}/month", money(bal), n(apr * 100.0), money(p))),
                primary(pod("Result", "Result", text(format!("never paid off: the payment does not cover the monthly interest of {}", money(bal * r))))),
            ]));
        }
        let k = -(1.0 - r * bal / p).ln() / (1.0 + r).ln();
        let total = p * k;
        return Some(Answer::new("credit card payoff", vec![
            input(format!("credit card | balance {} | APR {}%{assumed} | payment {}/month", money(bal), n(apr * 100.0), money(p))),
            primary(pod("Result", "Time to pay off", text(format!("{} months ({} years)", n(k.ceil()), n(k / 12.0))))),
            pod("Totals", "Totals", rows(vec![("total paid", money(total)), ("total interest", money(total - bal)), ("first month's interest", money(bal * r))])),
        ]));
    }
    let Some(k) = over else {
        // no payment given: payoff times for a range of fixed monthly payments
        let step = if bal < 2000.0 { 10.0 } else if bal < 20000.0 { 50.0 } else { 100.0 };
        let pays: Vec<f64> = [0.03, 0.05, 0.1, 0.2].iter().map(|f| ((bal * f / step).ceil() * step).max(bal * r + step)).collect();
        let months_for = |p: f64| -(1.0 - r * bal / p).ln() / (1.0 + r).ln();
        let p = pays[1];
        let k = months_for(p);
        let rows_t: Vec<Vec<Content>> = pays.iter().map(|p| { let k = months_for(*p); vec![text(format!("{}/month", money(*p))), text(format!("{} months", n(k.ceil()))), text(money(p * k - bal))] }).collect();
        return Some(Answer::new("credit card payoff", vec![
            input(format!("credit card | balance {} | APR {}%{assumed} | payment {}/month (assumed, 5% of the balance)", money(bal), n(apr * 100.0), money(p))),
            primary(pod("Result", "Time to pay off", text(format!("{} months ({} years)", n(k.ceil()), n((k / 12.0 * 10.0).round() / 10.0))))),
            pod("Totals", "Totals", rows(vec![("total paid", money(p * k)), ("total interest", money(p * k - bal))])),
            pod("Table", "By monthly payment", table(Some(vec!["payment", "time to pay off", "total interest"]), rows_t)),
        ]));
    };
    let p = payment(bal, r, k, 0.0);
    Some(Answer::new("credit card payoff", vec![
        input(format!("credit card | balance {} | APR {}%{assumed} | pay off in {} months", money(bal), n(apr * 100.0), n(k))),
        primary(pod("Result", "Monthly payment needed", text(money(p)))),
        pod("Totals", "Totals", rows(vec![("total paid", money(p * k)), ("total interest", money(p * k - bal))])),
    ]))
}

// ---------------------------------------------------------------------------
// salary, CPM, fuel cost
// ---------------------------------------------------------------------------

fn salary(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^(?:salary |wage |pay )?(\$|€|£)?([\d,.]+k?)\s*(dollars?|euros?|pounds?|usd|eur|gbp)?\s*(?:/|per |an? )\s*(yr|year|annum|hr|hour|month|mo|week|wk|day)(?: for ([\d.]+) years?)?$").captures(l)?;
    let sym = match (m.get(1).map(|s| s.as_str()), m.get(3).map(|s| s.as_str())) {
        (Some("€"), _) | (_, Some("euro" | "euros" | "eur")) => "€",
        (Some("£"), _) | (_, Some("pound" | "pounds" | "gbp")) => "£",
        (Some("$"), _) | (_, Some(_)) => "$",
        _ => return None,
    };
    let v = num(&m[2])?;
    let per_year = match &m[4] {
        "yr" | "year" | "annum" => 1.0,
        "hr" | "hour" => 2080.0,
        "month" | "mo" => 12.0,
        "week" | "wk" => 52.0,
        _ => 260.0,
    };
    let yearly = v * per_year;
    let cur = |x: f64| money(x).replacen('$', sym, 1);
    let mut r = vec![
        ("hourly", cur(yearly / 2080.0)),
        ("daily", cur(yearly / 260.0)),
        ("weekly", cur(yearly / 52.0)),
        ("biweekly", cur(yearly / 26.0)),
        ("semimonthly", cur(yearly / 24.0)),
        ("monthly", cur(yearly / 12.0)),
        ("yearly", cur(yearly)),
    ];
    let years = m.get(5).and_then(|y| num(y.as_str()));
    if let Some(y) = years {
        r.push(("total", format!("{} over {} years", cur(yearly * y), n(y))));
    }
    let headline = if per_year == 1.0 { format!("{} per hour", cur(yearly / 2080.0)) } else { format!("{} per year", cur(yearly)) };
    Some(Answer::new("salary conversion", vec![
        input(format!("{} per {}", cur(v), &m[4])),
        primary(pod("Result", "Equivalent pay", text(headline))),
        pod("Table", "Pay by period", rows(r)),
        pod("Assumptions", "Assumptions", text("40-hour weeks, 52 weeks and 260 working days a year; no unpaid leave or taxes")),
    ]))
}

fn per_day(unit: &str) -> f64 {
    match unit {
        u if u.starts_with('s') => 86400.0,
        u if u.starts_with("min") => 1440.0,
        u if u.starts_with('h') => 24.0,
        u if u.starts_with('w') => 1.0 / 7.0,
        u if u.starts_with("mo") => 12.0 / 365.0,
        _ => 1.0,
    }
}

fn cpm(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    if !(l.starts_with("cpm") || l.contains("cost per mille") || l.contains("cost per thousand")) {
        return None;
    }
    let im = re!(r"([\d,.]+k?)\s*(?:imp(?:ression)?s?|hits|views|impressions)\s*(?:/|per |a )\s*(s|sec|second|min|minute|hour|hr|day|week|month)").captures(l)?;
    let cost = re!(r"\$?([\d,.]+k?)\s*(?:dollars?|usd)?\s*(?:/|per |a )\s*(s|sec|second|min|minute|hour|hr|day|week|month)").captures_iter(l).find(|c| c.get(0).map(|x| x.start()) != im.get(0).map(|x| x.start()))?;
    let imps_day = num(&im[1])? * per_day(&im[2]);
    let cost_day = num(&cost[1])? * per_day(&cost[2]);
    let cpm = cost_day / imps_day * 1000.0;
    Some(Answer::new("CPM", vec![
        input(format!("cost per mille | impressions: {} per day | cost: {} per day", n(imps_day), money(cost_day))),
        primary(pod("Result", "Cost per thousand impressions", text(money(cpm)))),
        pod("Details", "Details", rows(vec![("cost per impression", format!("${}", n(cost_day / imps_day))), ("impressions per day", n(imps_day)), ("cost per month (30 days)", money(cost_day * 30.0))])),
    ]))
}

fn fuel_cost(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    if !re!(r"^(?:fuel|gas|gasoline|petrol|trip|driving) cost").is_match(l) && !l.starts_with("cost of gas") {
        return None;
    }
    let dist = re!(r"([\d,.]+)\s*(miles?|mi|km|kilometers?)\b").captures(l)?;
    let mpg = field(l, re!(r"([\d.]+)\s*mpg"));
    let l100 = field(l, re!(r"([\d.]+)\s*l/100\s*km"));
    let price = re!(r"\$?([\d.]+)\s*(?:/|per )\s*(gal|gallon|l|liter|litre)\b").captures(l)?;
    let miles = num(&dist[1])? * if dist[2].starts_with('k') { 0.621371192 } else { 1.0 };
    let gallons = match (mpg, l100) {
        (Some(m), _) => miles / m,
        (_, Some(x)) => miles * 1.609344 * x / 100.0 / 3.785411784,
        _ => return None,
    };
    let per_gal = num(&price[1])? * if price[2].starts_with('l') { 3.785411784 } else { 1.0 };
    let cost = gallons * per_gal;
    Some(Answer::new("fuel cost", vec![
        input(format!("fuel cost | distance: {} miles | fuel used: {} gallons | price: {}/gallon", n(miles), n(gallons), money(per_gal))),
        primary(pod("Result", "Fuel cost", text(money(cost)))),
        pod("Details", "Details", rows(vec![("fuel used", format!("{} gallons = {} liters", n(gallons), n(gallons * 3.785411784))), ("cost per mile", format!("${}", n(cost / miles))), ("round trip", money(2.0 * cost))])),
    ]))
}

// ---------------------------------------------------------------------------
// bonds and options
// ---------------------------------------------------------------------------

fn bond(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    if !re!(r"^bond (?:pric|valu|yield|duration)").is_match(l) {
        return None;
    }
    let face = field(l, re!(r"(?:face value|par|face)?\s*\$([\d,.]+k?)")).unwrap_or(1000.0);
    let coupon = field(l, re!(r"coupon(?: rate)?\s*(?:of|=|:)?\s*([\d.]+)\s*%"))? / 100.0;
    let y = field(l, re!(r"(?:yield|ytm|market rate|discount rate)\s*(?:of|=|:)?\s*([\d.]+)\s*%"))? / 100.0;
    let years_given = field(l, re!(r"([\d.]+)\s*years?"));
    let years = years_given.unwrap_or(10.0);
    let freq = if l.contains("semiannual") || l.contains("semi-annual") { 2.0 } else { 1.0 };
    let price_at = |t: f64| {
        let k = (t * freq).round();
        let (c, yy) = (face * coupon / freq, y / freq);
        if yy == 0.0 { c * k + face } else { c * (1.0 - (1.0 + yy).powf(-k)) / yy + face * (1.0 + yy).powf(-k) }
    };
    let price = price_at(years);
    let k = (years * freq).round() as usize;
    let (c, yy) = (face * coupon / freq, y / freq);
    let mac: f64 = (1..=k).map(|i| {
        let cf = c + if i == k { face } else { 0.0 };
        i as f64 / freq * cf / (1.0 + yy).powi(i as i32)
    }).sum::<f64>() / price;
    let modd = mac / (1.0 + yy);
    let mat: Vec<Vec<Content>> = [1.0, 2.0, 5.0, 10.0, 20.0, 30.0].iter().map(|t| vec![text(format!("{t} years")), text(money(price_at(*t)))]).collect();
    Some(Answer::new("bond price", vec![
        input(format!("bond | face value {} | coupon rate {}% | yield {}% | {} years to maturity{} | {} coupons", money(face), n(coupon * 100.0), n(y * 100.0), n(years), if years_given.is_none() { " (assumed)" } else { "" }, if freq == 2.0 { "semiannual" } else { "annual" })),
        primary(pod("Result", "Bond price", text(money(price)))),
        pod("Formula", "Formula", text("P = C (1 − (1+y)^(−n))/y + F (1+y)^(−n)")),
        pod("Measures", "Bond measures", rows(vec![
            ("coupon payment", money(c)),
            ("current yield", format!("{}%", n(face * coupon / price * 100.0))),
            ("Macaulay duration", format!("{} years", n(mac))),
            ("modified duration", format!("{} years", n(modd))),
            ("trades at", (if price > face + 0.005 { "a premium" } else if price < face - 0.005 { "a discount" } else { "par" }).to_string()),
        ])),
        pod("Maturities", "Price by time to maturity", table(Some(vec!["maturity", "price"]), mat)),
    ]))
}

fn norm_cdf(x: f64) -> f64 {
    0.5 * crate::special::erfc(-x / std::f64::consts::SQRT_2)
}

fn black_scholes(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    if !(l.starts_with("black-scholes") || l.starts_with("black scholes") || l.starts_with("option pric")) {
        return None;
    }
    let s = field(l, re!(r"\b(?:s|spot|stock price|price)\s*=\s*\$?([\d.]+)"));
    let k = field(l, re!(r"\b(?:k|x|strike)\s*=\s*\$?([\d.]+)"));
    let r = field(l, re!(r"\b(?:r|rate)\s*=\s*([\d.]+)"));
    let v = field(l, re!(r"\b(?:sigma|σ|vol|volatility|v)\s*=\s*([\d.]+)"));
    let t = field(l, re!(r"\b(?:t|time|expiry|maturity)\s*=\s*([\d.]+)"));
    let formula = pod("Formula", "Black–Scholes formula", text("C = S N(d₁) − K e^(−r T) N(d₂),  P = K e^(−r T) N(−d₂) − S N(−d₁),\nd₁ = (ln(S/K) + (r + σ²/2) T)/(σ √T),  d₂ = d₁ − σ √T"));
    let (Some(s), Some(k), Some(r), Some(v), Some(t)) = (s, k, r, v, t) else {
        return Some(Answer::new("Black–Scholes formula", vec![
            input("Black–Scholes option pricing formula"),
            primary(formula),
            pod("Variables", "Variables", rows(vec![("S", "spot price of the underlying".into()), ("K", "strike price".into()), ("r", "risk-free interest rate (continuous)".into()), ("σ", "volatility of returns".into()), ("T", "time to expiry in years".into()), ("N", "standard normal CDF".into())])),
        ]));
    };
    let pct = |x: f64| if x > 1.0 { x / 100.0 } else { x };
    let (r, v) = (pct(r), pct(v));
    let d1 = ((s / k).ln() + (r + v * v / 2.0) * t) / (v * t.sqrt());
    let d2 = d1 - v * t.sqrt();
    let call = s * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2);
    let put = k * (-r * t).exp() * norm_cdf(-d2) - s * norm_cdf(-d1);
    let pdf = (-d1 * d1 / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt();
    Some(Answer::new("Black–Scholes", vec![
        input(format!("Black–Scholes | S = {} | K = {} | r = {}% | σ = {}% | T = {} years", n(s), n(k), n(r * 100.0), n(v * 100.0), n(t))),
        primary(pod("Result", "Option values", rows(vec![("call", n(call)), ("put", n(put))]))),
        pod("Greeks", "Greeks", table(Some(vec!["", "call", "put"]), vec![
            vec![text("delta"), text(n(norm_cdf(d1))), text(n(norm_cdf(d1) - 1.0))],
            vec![text("gamma"), text(n(pdf / (s * v * t.sqrt()))), text(n(pdf / (s * v * t.sqrt())))],
            vec![text("vega (per 1.00 of σ)"), text(n(s * pdf * t.sqrt())), text(n(s * pdf * t.sqrt()))],
            vec![text("theta (per year)"), text(n(-s * pdf * v / (2.0 * t.sqrt()) - r * k * (-r * t).exp() * norm_cdf(d2))), text(n(-s * pdf * v / (2.0 * t.sqrt()) + r * k * (-r * t).exp() * norm_cdf(-d2)))],
            vec![text("rho"), text(n(k * t * (-r * t).exp() * norm_cdf(d2))), text(n(-k * t * (-r * t).exp() * norm_cdf(-d2)))],
        ])),
        pod("Intermediate", "Intermediate values", rows(vec![("d₁", n(d1)), ("d₂", n(d2))])),
        formula,
    ]))
}

/// Option strategies: legs of (+1 long / −1 short, call?, strike offset).
const STRATEGIES: &[(&[&str], &str, &[(f64, bool, f64)], &str)] = &[
    (&["butterfly", "long butterfly", "butterfly spread", "call butterfly"], "long call butterfly", &[(1.0, true, -10.0), (-2.0, true, 0.0), (1.0, true, 10.0)], "buy a call at K₁, sell two calls at K₂, buy a call at K₃; profits when the price stays near K₂"),
    (&["strangle", "long strangle"], "long strangle", &[(1.0, false, -10.0), (1.0, true, 10.0)], "buy an out-of-the-money put and an out-of-the-money call; profits from a large move either way"),
    (&["straddle", "long straddle"], "long straddle", &[(1.0, false, 0.0), (1.0, true, 0.0)], "buy a put and a call at the same strike; profits from a large move either way"),
    (&["short straddle"], "short straddle", &[(-1.0, false, 0.0), (-1.0, true, 0.0)], "sell a put and a call at the same strike; profits when the price stays put"),
    (&["short strangle"], "short strangle", &[(-1.0, false, -10.0), (-1.0, true, 10.0)], "sell an out-of-the-money put and call; profits when the price stays in range"),
    (&["iron condor"], "iron condor", &[(1.0, false, -20.0), (-1.0, false, -10.0), (-1.0, true, 10.0), (1.0, true, 20.0)], "a short strangle protected by long wings; limited gain and limited loss"),
    (&["iron butterfly"], "iron butterfly", &[(1.0, false, -10.0), (-1.0, false, 0.0), (-1.0, true, 0.0), (1.0, true, 10.0)], "a short straddle protected by long wings"),
    (&["bull call spread", "bull spread", "call spread"], "bull call spread", &[(1.0, true, -10.0), (-1.0, true, 10.0)], "buy a call at K₁ and sell a call at K₂ > K₁; limited bullish bet"),
    (&["bear put spread", "bear spread", "put spread"], "bear put spread", &[(1.0, false, 10.0), (-1.0, false, -10.0)], "buy a put at K₂ and sell a put at K₁ < K₂; limited bearish bet"),
    (&["covered call"], "covered call", &[(-1.0, true, 10.0)], "own the stock and sell a call above the price"),
    (&["protective put", "married put"], "protective put", &[(1.0, false, -10.0)], "own the stock and buy a put below the price"),
    (&["call", "call option", "long call"], "long call", &[(1.0, true, 0.0)], "the right to buy at the strike price"),
    (&["put", "put option", "long put"], "long put", &[(1.0, false, 0.0)], "the right to sell at the strike price"),
];

fn strategy(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^(?:an? )?([a-z ]+?)(?: option)?(?: strategy| spread| payoff| payoff diagram)?$").captures(l)?;
    if !(l.contains("option") || l.contains("strategy") || l.contains("payoff") || l.contains("spread") || l.contains("straddle") || l.contains("strangle") || l.contains("condor")) {
        return None;
    }
    let name = m[1].trim();
    let (_, title, legs, desc) = STRATEGIES.iter().find(|s| s.0.contains(&name) || s.1 == name)?;
    let stock = matches!(*title, "covered call" | "protective put");
    let payoff = |s: f64| {
        let mut v = if stock { s - 100.0 } else { 0.0 };
        for (q, call, off) in legs.iter() {
            let k = 100.0 + off;
            v += q * if *call { (s - k).max(0.0) } else { (k - s).max(0.0) };
        }
        v
    };
    let xs: Vec<f64> = (0..=160).map(|i| 50.0 + i as f64 * 0.625).collect();
    let ys: Vec<f64> = xs.iter().map(|&s| payoff(s)).collect();
    let leg_rows: Vec<Vec<Content>> = legs.iter().map(|(q, call, off)| vec![text(if *q > 0.0 { format!("buy {}", q.abs()) } else { format!("sell {}", q.abs()) }), text(if *call { "call" } else { "put" }), text(n(100.0 + off))]).collect();
    Some(Answer::new(*title, vec![
        input(format!("{title} option strategy")),
        primary(pod("Description", "Description", text(*desc))),
        pod("Legs", &format!("Legs (example strikes around a price of 100{})", if stock { ", plus 100 shares of stock" } else { "" }), table(Some(vec!["position", "type", "strike"]), leg_rows)),
        pod("Plot", "Payoff at expiry, before premiums", plot(crate::plot::lines(vec![("payoff", xs, ys)], "underlying price at expiry", "payoff"))),
    ]))
}

// ---------------------------------------------------------------------------
// US federal income tax, tax year 2026 (Rev. Proc. 2025-32)
// ---------------------------------------------------------------------------

const RATES: [f64; 7] = [0.10, 0.12, 0.22, 0.24, 0.32, 0.35, 0.37];
const SINGLE: [f64; 6] = [12_400.0, 50_400.0, 105_700.0, 201_775.0, 256_225.0, 640_600.0];
const JOINT: [f64; 6] = [24_800.0, 100_800.0, 211_400.0, 403_550.0, 512_450.0, 768_700.0];

fn tax_on(taxable: f64, tops: &[f64; 6]) -> (f64, f64) {
    let mut tax = 0.0;
    let mut lo = 0.0;
    for (i, rate) in RATES.iter().enumerate() {
        let hi = if i < 6 { tops[i] } else { f64::INFINITY };
        if taxable > lo {
            tax += (taxable.min(hi) - lo) * rate;
        }
        if taxable <= hi {
            return (tax, *rate);
        }
        lo = hi;
    }
    (tax, 0.37)
}

fn income_tax(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    if !re!(r"(?:income tax|tax rate|federal tax|tax on|taxes on|tax bracket)").is_match(l) || l.contains("state") || re!(r"\b(?:19|20)\d\d\b").is_match(l) || l.contains("receipts") || l.contains(" vs") {
        return None;
    }
    let joint = l.contains("married") || l.contains("joint");
    let (tops, std_ded, status) = if joint { (&JOINT, 32_200.0, "married filing jointly") } else { (&SINGLE, 16_100.0, "single") };
    let bracket_rows = |tops: &[f64; 6]| -> Vec<Vec<Content>> {
        let mut lo = 0.0;
        RATES.iter().enumerate().map(|(i, r)| {
            let hi = if i < 6 { money(tops[i]) } else { "and above".into() };
            let row = vec![text(format!("{}%", r * 100.0)), text(format!("{} – {hi}", money(lo)))];
            if i < 6 {
                lo = tops[i];
            }
            row
        }).collect()
    };
    let income = field(l, re!(r"\$([\d,.]+k?)")).or_else(|| field(l, re!(r"(?:on|earning|income of|of)\s+([\d,.]+k?)\b")));
    let Some(gross) = income else {
        if !(l.contains("bracket") || l.contains("calculate") || l.contains("income tax")) {
            return None;
        }
        return Some(Answer::new("US income tax brackets", vec![
            input(format!("US federal income tax brackets | tax year 2026 | {status}")),
            primary(pod("Brackets", "Tax brackets (taxable income)", table(Some(vec!["rate", "taxable income"]), bracket_rows(tops)))),
            pod("Deduction", "Standard deduction", text(format!("{} ({status})", money(std_ded)))),
        ]));
    };
    let taxable = (gross - std_ded).max(0.0);
    let (tax, marginal) = tax_on(taxable, tops);
    let headline = if l.contains("marginal") {
        format!("{}%", marginal * 100.0)
    } else if l.contains("effective") {
        format!("{}%", n(tax / gross * 100.0))
    } else {
        money(tax)
    };
    Some(Answer::new("US income tax", vec![
        input(format!("US federal income tax | income {} | {status} | standard deduction | tax year 2026", money(gross))),
        primary(pod("Result", if l.contains("marginal") { "Marginal tax rate" } else if l.contains("effective") { "Effective tax rate" } else { "Federal income tax" }, text(headline))),
        pod("Details", "Details", rows(vec![
            ("gross income", money(gross)),
            ("standard deduction", money(std_ded)),
            ("taxable income", money(taxable)),
            ("federal income tax", money(tax)),
            ("marginal rate", format!("{}%", marginal * 100.0)),
            ("effective rate (of gross income)", format!("{}%", n(if gross > 0.0 { tax / gross * 100.0 } else { 0.0 }))),
            ("income after federal income tax", money(gross - tax)),
        ])),
        pod("Brackets", "Tax brackets (taxable income)", table(Some(vec!["rate", "taxable income"]), bracket_rows(tops))),
        pod("Assumptions", "Assumptions", text("ordinary income only, standard deduction, no credits; excludes payroll and state taxes")),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    loans(q)
        .or_else(|| credit_card(q))
        .or_else(|| bond(q))
        .or_else(|| black_scholes(q))
        .or_else(|| strategy(q))
        .or_else(|| income_tax(q))
        .or_else(|| cpm(q))
        .or_else(|| fuel_cost(q))
        .or_else(|| salary(q))
        .or_else(|| tvm(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tax_and_payments() {
        // 2026 single: 10% of 12,400 + 12% of (23,900 − 12,400)
        let (t, m) = tax_on(23_900.0, &SINGLE);
        assert!((t - 2620.0).abs() < 1e-6 && m == 0.12);
        // standard 30-year mortgage payment
        let p = payment(200_000.0, 0.06 / 12.0, 360.0, 0.0);
        assert!((p - 1199.10).abs() < 0.01, "{p}");
    }
}
