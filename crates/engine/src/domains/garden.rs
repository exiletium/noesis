//! Vegetable and herb growing: germination time, soil temperature for
//! germination, sowing depth, spacing and days to maturity.
//!
//! "germination time for garden peas", "how long do carrots take to
//! germinate", "days to maturity for sweet corn", "tomato planting depth".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names | germination days | min soil °F | optimum soil °F | sowing depth (in) | days to maturity | note
/// Typical values from US cooperative-extension planting guides.
const CROPS: &[(&[&str], &str, u32, u32, &str, &str, &str)] = &[
    (&["garden pea", "pea", "english pea", "sweet pea"], "7–14", 40, 75, "1–2", "60–70 from seed", "cool-season; sow as soon as the soil can be worked"),
    (&["snap bean", "green bean", "bush bean", "bean", "string bean"], "7–10", 60, 80, "1", "50–60 from seed", "warm-season; seeds rot in cold, wet soil"),
    (&["lima bean"], "7–14", 60, 85, "1–1.5", "65–80 from seed", "warm-season"),
    (&["beet", "beetroot"], "5–10", 40, 85, "0.5", "50–70 from seed", "each seed is a cluster, so thin seedlings"),
    (&["broccoli"], "4–7", 40, 85, "0.25–0.5", "60–80 from transplant", "cool-season"),
    (&["brussels sprout", "brussels sprouts"], "4–7", 40, 85, "0.25–0.5", "90–110 from transplant", "cool-season; flavour improves after frost"),
    (&["cabbage"], "4–7", 40, 85, "0.25–0.5", "60–100 from transplant", "cool-season"),
    (&["carrot"], "10–17", 40, 80, "0.25", "70–80 from seed", "keep the surface moist until seedlings emerge"),
    (&["cauliflower"], "4–7", 40, 80, "0.25–0.5", "55–80 from transplant", "cool-season"),
    (&["celery"], "14–21", 40, 70, "0.125", "100–130 from transplant", "seeds need light to germinate"),
    (&["chard", "swiss chard"], "7–14", 40, 85, "0.5", "50–60 from seed", ""),
    (&["sweet corn", "corn", "maize"], "7–10", 50, 95, "1–2", "60–100 from seed", "plant in blocks for good pollination"),
    (&["cucumber"], "3–10", 60, 95, "1", "50–70 from seed", "warm-season; frost-tender"),
    (&["eggplant", "aubergine"], "7–14", 60, 85, "0.25", "70–85 from transplant", "warm-season; start indoors"),
    (&["kale"], "5–8", 40, 85, "0.25–0.5", "50–65 from seed", "cool-season"),
    (&["leek"], "7–14", 40, 75, "0.25", "120–150 from seed", ""),
    (&["lettuce"], "7–10", 35, 75, "0.125–0.25", "45–60 from seed (leaf), 70–90 (head)", "seeds need light; poor germination above 85 °F"),
    (&["cantaloupe", "muskmelon", "melon"], "5–10", 60, 90, "1", "80–90 from seed", "warm-season"),
    (&["okra"], "7–14", 60, 95, "1", "50–65 from seed", "soak seeds overnight before sowing"),
    (&["onion"], "7–10", 35, 80, "0.25–0.5", "100–120 from seed", "bulbing depends on day length"),
    (&["parsley"], "14–28", 40, 75, "0.25", "70–90 from seed", "slow to germinate; soak seeds first"),
    (&["parsnip"], "15–25", 35, 70, "0.5", "100–130 from seed", "use fresh seed each year"),
    (&["pepper", "bell pepper", "chili pepper", "sweet pepper"], "10–14", 60, 85, "0.25", "60–90 from transplant", "warm-season; start indoors"),
    (&["pumpkin"], "7–10", 60, 95, "1–1.5", "90–120 from seed", "warm-season"),
    (&["radish"], "4–6", 40, 85, "0.5", "22–30 from seed", "fastest common vegetable"),
    (&["spinach"], "7–14", 35, 70, "0.5", "40–50 from seed", "bolts in long, hot days"),
    (&["summer squash", "zucchini", "courgette", "squash"], "5–10", 60, 95, "1", "50–60 from seed", "warm-season"),
    (&["winter squash", "butternut squash"], "7–10", 60, 95, "1", "80–110 from seed", "warm-season"),
    (&["tomato"], "6–14", 50, 85, "0.25", "60–85 from transplant", "warm-season; start indoors 6–8 weeks before the last frost"),
    (&["turnip"], "3–7", 40, 85, "0.5", "40–60 from seed", ""),
    (&["watermelon"], "7–10", 60, 95, "1", "80–100 from seed", "warm-season"),
    (&["basil"], "5–10", 50, 75, "0.25", "60–90 from seed", "frost-tender"),
    (&["cilantro", "coriander"], "7–10", 40, 70, "0.25–0.5", "45–70 from seed", "bolts quickly in heat"),
];

fn find(s: &str) -> Option<&'static (&'static [&'static str], &'static str, u32, u32, &'static str, &'static str, &'static str)> {
    let s = s.trim().trim_start_matches("the ").trim_start_matches("garden ").trim_start_matches("seeds of ");
    let s = s.trim_end_matches(" seeds").trim_end_matches(" seed").trim_end_matches(" plants").trim_end_matches(" plant");
    let sing = |x: &str| -> String {
        if let Some(b) = x.strip_suffix("oes") { return format!("{b}o"); }
        if let Some(b) = x.strip_suffix("ies") { return format!("{b}y"); }
        if x.ends_with("ss") || x.ends_with("us") { return x.to_string(); }
        x.strip_suffix('s').unwrap_or(x).to_string()
    };
    let cands = [s.to_string(), sing(s), format!("garden {s}"), format!("garden {}", sing(s))];
    CROPS.iter().find(|c| c.0.iter().any(|n| cands.iter().any(|x| x == n)))
}

fn f_to_c(f: u32) -> String {
    crate::print::fmt_float(((f as f64 - 32.0) * 5.0 / 9.0).round(), 4)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let (crop, prop) = if let Some(m) = re!(r"^(?:germination (?:time|period|days)|days to germination|time to germinate|germination temperature|soil temperature(?: for germination)?) (?:for|of) (.+)$").captures(&l) {
        (m[1].to_string(), if l.contains("temperature") { "temp" } else { "germ" })
    } else if let Some(m) = re!(r"^(.+?) (?:germination(?: time| period| temperature)?|days to germination)$").captures(&l) {
        (m[1].to_string(), if l.contains("temperature") { "temp" } else { "germ" })
    } else if let Some(m) = re!(r"^how (?:long|many days) (?:does it take )?(?:do|does|for|until|before)? ?(.+?) (?:seeds? )?(?:take to |to )?(?:germinate|sprout|come up)$").captures(&l) {
        (m[1].to_string(), "germ")
    } else if let Some(m) = re!(r"^(?:days to (?:maturity|harvest)|time to (?:maturity|harvest)|maturity time) (?:for|of) (.+)$|^(.+?) (?:days to (?:maturity|harvest)|maturity)$|^how long (?:does it take )?(?:do|does|for)? ?(.+?) (?:take )?to (?:grow|mature)$").captures(&l) {
        (m.get(1).or(m.get(2)).or(m.get(3))?.as_str().to_string(), "mature")
    } else if let Some(m) = re!(r"^(?:planting|sowing|seed) depth (?:for|of) (.+)$|^(.+?) (?:planting|sowing|seed) depth$|^how deep (?:to|should i|do you) (?:plant|sow) (.+?)(?: seeds)?$").captures(&l) {
        (m.get(1).or(m.get(2)).or(m.get(3))?.as_str().to_string(), "depth")
    } else {
        return None;
    };
    let c = find(&crop)?;
    let name = c.0[0];
    let (title, result) = match prop {
        "temp" => ("Soil temperature for germination", format!("minimum {} °F ({} °C), optimum {} °F ({} °C)", c.2, f_to_c(c.2), c.3, f_to_c(c.3))),
        "mature" => ("Days to maturity", c.5.replacen(" from", " days from", 1)),
        "depth" => ("Sowing depth", format!("{} in", c.4)),
        _ => ("Germination time", format!("{} days (at {} °F soil)", c.1, c.3)),
    };
    let mut rows = vec![
        vec![text("germination time"), text(format!("{} days", c.1))],
        vec![text("minimum soil temperature"), text(format!("{} °F ({} °C)", c.2, f_to_c(c.2)))],
        vec![text("optimum soil temperature"), text(format!("{} °F ({} °C)", c.3, f_to_c(c.3)))],
        vec![text("sowing depth"), text(format!("{} in", c.4))],
        vec![text("days to maturity"), text(c.5.replacen(" from", " days from", 1))],
    ];
    if !c.6.is_empty() {
        rows.push(vec![text("notes"), text(c.6)]);
    }
    Some(Answer::new(format!("{name} (vegetable)"), vec![
        pod("Input", "Input interpretation", text(format!("{name} | {}", title.to_lowercase()))),
        primary(pod("Result", title, text(result))),
        pod("Table", "Growing information", table(None, rows)),
        pod("Note", "Note", text("typical values from US cooperative-extension planting guides; germination is slower in cool soil")),
    ]))
}
