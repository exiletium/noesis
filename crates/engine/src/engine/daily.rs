//! Everyday calculations: energy used by exercise ("walking 45min, 4mph",
//! "playing broomball, 1hr, body weight 150lbs"), heart-rate zones, weight
//! loss plans, calendar facts (zodiac signs, birthstones, birth flowers,
//! anniversary gifts, months with 30 days, "how long till 5pm"), time-zone
//! conversions ("convert noon ART to CAT"), betting odds and roulette,
//! photography (hyperfocal distance, f-stops, print sizes) and craft sizes
//! (ring sizes, knitting needles, lumber, paint coverage).
//!
//! Exercise energy uses MET values from the Compendium of Physical
//! Activities (Ainsworth et al., 2011): kcal = MET × body mass (kg) × hours.
//! Walking and running on a grade use the ACSM metabolic equations.

use super::*;
use chrono::{Datelike, NaiveDate, NaiveTime};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn fmt(v: f64) -> String {
    crate::stats::fmt(v)
}

fn r1(v: f64) -> String {
    fmt((v * 10.0).round() / 10.0)
}

fn kv(rows: Vec<(&str, String)>) -> Content {
    table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())
}

fn ans(title: &str, input: String, res_title: &str, res: Content, extra: Vec<Pod>) -> Answer {
    let mut pods = vec![pod("Input", "Input interpretation", text(input)), primary(pod("Result", res_title, res))];
    pods.extend(extra);
    Answer::new(title, pods)
}

// ---------------------------------------------------------------------------
// exercise
// ---------------------------------------------------------------------------

/// (pattern, name, MET)
const ACTIVITIES: &[(&str, &str, f64)] = &[
    (r"water jogging|aqua jogging|deep water running", "water jogging", 9.8),
    (r"water polo", "water polo", 10.0),
    (r"broomball", "broomball", 7.0),
    (r"skate ?board", "skateboarding", 5.0),
    (r"unicycl", "unicycling", 5.0),
    (r"jump(?:ing)? rope|skipping rope", "jumping rope", 11.8),
    (r"rock climbing|bouldering", "rock climbing", 8.0),
    (r"stair|steps", "climbing stairs", 8.8),
    (r"hik(?:e|ing)", "hiking", 6.0),
    (r"elliptical", "elliptical trainer", 5.0),
    (r"rowing machine|ergometer", "rowing machine", 7.0),
    (r"rowing|canoe|kayak", "rowing / kayaking", 5.0),
    (r"yoga", "yoga (hatha)", 2.5),
    (r"pilates", "Pilates", 3.0),
    (r"weight (?:lifting|training)|lifting weights|strength training", "weight training", 3.5),
    (r"aerobics", "aerobics", 7.3),
    (r"zumba|danc(?:e|ing)", "dancing", 5.0),
    (r"tennis", "tennis", 7.3),
    (r"badminton", "badminton", 5.5),
    (r"squash|racquetball", "squash", 7.3),
    (r"basketball", "basketball", 6.5),
    (r"soccer|football \(soccer\)", "soccer", 7.0),
    (r"american football|football", "American football", 8.0),
    (r"volleyball", "volleyball", 4.0),
    (r"baseball|softball", "baseball", 5.0),
    (r"ice hockey|hockey", "ice hockey", 8.0),
    (r"golf", "golf (walking)", 4.8),
    (r"bowling", "bowling", 3.8),
    (r"frisbee", "frisbee", 3.0),
    (r"boxing", "boxing (sparring)", 7.8),
    (r"martial arts|karate|judo|taekwondo|kickboxing", "martial arts", 10.3),
    (r"ice skating|skating", "ice skating", 7.0),
    (r"cross[- ]country ski", "cross-country skiing", 9.0),
    (r"ski(?:ing)?|snowboard", "downhill skiing", 5.3),
    (r"horse(?:back)? riding", "horseback riding", 5.5),
    (r"surf", "surfing", 3.0),
    (r"scuba|diving\b|dive\b", "scuba diving", 7.0),
    (r"garden", "gardening", 3.8),
    (r"mow", "mowing the lawn", 5.5),
    (r"clean(?:ing)? (?:the )?house|housework|vacuum", "house cleaning", 3.3),
    (r"sleep", "sleeping", 0.95),
    (r"watching tv|television", "watching television", 1.3),
    (r"sitting|reading", "sitting quietly", 1.3),
    (r"standing", "standing", 1.8),
    (r"typing|office work|desk work", "desk work", 1.5),
];

fn walking_met(mph: f64) -> f64 {
    let t = [(2.0, 2.8), (2.5, 3.0), (3.0, 3.5), (3.5, 4.3), (4.0, 5.0), (4.5, 7.0), (5.0, 8.3)];
    interp(&t, mph)
}

fn running_met(mph: f64) -> f64 {
    let t = [(4.0, 6.0), (5.0, 8.3), (5.2, 9.0), (6.0, 9.8), (6.7, 10.5), (7.0, 11.0), (7.5, 11.5), (8.0, 11.8), (8.6, 12.3), (9.0, 12.8), (10.0, 14.5), (11.0, 16.0), (12.0, 19.0), (13.0, 19.8), (14.0, 23.0)];
    interp(&t, mph)
}

fn cycling_met(mph: f64) -> f64 {
    let t = [(8.0, 4.0), (11.0, 6.8), (13.0, 8.0), (15.0, 10.0), (17.5, 12.0), (20.0, 15.8)];
    interp(&t, mph)
}

fn interp(t: &[(f64, f64)], x: f64) -> f64 {
    if x <= t[0].0 {
        return t[0].1;
    }
    for w in t.windows(2) {
        if x <= w[1].0 {
            let f = (x - w[0].0) / (w[1].0 - w[0].0);
            return w[0].1 + f * (w[1].1 - w[0].1);
        }
    }
    t[t.len() - 1].1
}

fn duration_hours(l: &str) -> Option<f64> {
    let mut total = 0.0;
    let mut any = false;
    for m in re!(r"(\d+(?:\.\d+)?)\s*(hours?|hrs?|h\b|minutes?|mins?|m\b)").captures_iter(l) {
        let v: f64 = m[1].parse().ok()?;
        any = true;
        total += if m[2].starts_with('h') { v } else { v / 60.0 };
    }
    any.then_some(total)
}

fn body_kg(l: &str) -> Option<f64> {
    let m = re!(r"(\d+(?:\.\d+)?)\s*(kg|kilograms?|lbs?|pounds?)\b").captures(l)?;
    let v: f64 = m[1].parse().ok()?;
    Some(if m[2].starts_with('k') { v } else { v * 0.45359237 })
}

fn speed_mph(l: &str) -> Option<f64> {
    let m = re!(r"(\d+(?:\.\d+)?)\s*(mph|miles? per hour|km/h|kph|km per hour|m(?:eters)? per min(?:ute)?|m/min)").captures(l)?;
    let v: f64 = m[1].parse().ok()?;
    Some(match &m[2] {
        u if u.starts_with("mph") || u.starts_with("mile") => v,
        u if u.starts_with('k') => v / 1.609344,
        _ => v * 60.0 / 1609.344,
    })
}

fn exercise(l: &str) -> Option<Answer> {
    if re!(r"\b(?:vs\.?|versus|compared to)\b|/").is_match(l) && !re!(r"\d").is_match(l) {
        // compare activities: "soccer vs water polo", "watching tv / sleeping"
        let parts: Vec<&str> = re!(r"\s*(?:\bvs\.?|\bversus\b|\bcompared to\b|/)\s*").split(l).collect();
        let found: Vec<(&str, f64)> = parts.iter().filter_map(|p| ACTIVITIES.iter().find(|a| Regex::new(&format!("^(?:playing\\s+)?(?:{})", a.0)).map(|r| r.is_match(p.trim())).unwrap_or(false)).map(|a| (a.1, a.2))).collect();
        if found.len() == parts.len() && found.len() >= 2 {
            let rowsv = found.iter().map(|(n, met)| vec![text(*n), text(fmt(*met)), text(format!("{} kcal", fmt((met * 70.0).round())))]).collect();
            return Some(ans("exercise", format!("{} compared", found.iter().map(|f| f.0).collect::<Vec<_>>().join(" vs ")), "Energy use", table(Some(vec!["activity", "MET", "per hour for 70 kg"]), rowsv), vec![pod("Note", "Note", text("MET values from the Compendium of Physical Activities (2011); 1 MET ≈ 1 kcal per kg per hour"))]));
        }
        return None;
    }
    let speed = speed_mph(l);
    let grade = re!(r"(\d+(?:\.\d+)?)\s*%\s*(?:grade|incline|slope)").captures(l).and_then(|m| m[1].parse::<f64>().ok());
    let (name, met): (String, f64) = if re!(r"\b(?:running|run|jogging|jog)\b").is_match(l) && !l.contains("water") {
        let mph = speed.unwrap_or(6.0);
        match grade {
            Some(g) => {
                let s = mph * 26.8224;
                (format!("running at {} mph on a {}% grade", fmt(mph), fmt(g)), (3.5 + 0.2 * s + 0.9 * s * g / 100.0) / 3.5)
            }
            None => (format!("running at {} mph", fmt(mph)), running_met(mph)),
        }
    } else if re!(r"\b(?:walking|walk)\b").is_match(l) {
        let mph = speed.unwrap_or(3.0);
        match grade {
            Some(g) => {
                let s = mph * 26.8224;
                (format!("walking at {} mph on a {}% grade", fmt(mph), fmt(g)), (3.5 + 0.1 * s + 1.8 * s * g / 100.0) / 3.5)
            }
            None => (format!("walking at {} mph", fmt(mph)), walking_met(mph)),
        }
    } else if re!(r"\b(?:cycling|biking|bicycling|bike ride|cycle)\b").is_match(l) {
        let mph = speed.unwrap_or(11.0);
        (format!("cycling at {} mph", fmt(mph)), cycling_met(mph))
    } else if re!(r"\bswim").is_match(l) {
        match speed {
            Some(mph) => {
                let m_per_min = mph * 1609.344 / 60.0;
                let met = if m_per_min < 40.0 { 5.8 } else if m_per_min < 65.0 { 8.3 } else { 9.8 };
                (format!("swimming freestyle at {} m/min", fmt(m_per_min.round())), met)
            }
            None => ("swimming (leisurely)".into(), 6.0),
        }
    } else if let Some(a) = ACTIVITIES.iter().find(|a| Regex::new(&format!(r"\b(?:{})", a.0)).map(|r| r.is_match(l)).unwrap_or(false)) {
        let met = if a.1 == "climbing stairs" {
            match re!(r"(\d+)\s*steps?\s*(?:/|per)\s*min").captures(l).and_then(|m| m[1].parse::<f64>().ok()) {
                Some(r) if r < 70.0 => 4.0,
                _ => a.2,
            }
        } else {
            a.2
        };
        (a.1.to_string(), met)
    } else {
        return None;
    };
    // time: an explicit duration, or a number of steps at a stepping rate
    let no_rates = re!(r"\d+(?:\.\d+)?\s*(?:steps?\s*(?:/|per)\s*min\w*|m(?:eters?)?\s*(?:/|per)\s*min\w*|km/h|kph|mph|miles? per hour|km per hour)").replace_all(l, "");
    let hours = duration_hours(&no_rates).or_else(|| {
        let m = re!(r"(\d+)\s*steps\s+at\s+(\d+)\s*steps?\s*(?:/|per)\s*min").captures(l)?;
        Some(m[1].parse::<f64>().ok()? / m[2].parse::<f64>().ok()? / 60.0)
    });
    let wants_fat = re!(r"\bfat\b").is_match(l);
    // an activity counts as a calculation only with a duration, a body mass or an energy word
    if hours.is_none() && body_kg(l).is_none() && !wants_fat && !re!(r"\b(?:calories|energy|kcal|burn(?:ed|s)?)\b").is_match(l) {
        return None;
    }
    // and nothing that looks like mathematics
    if re!(r"[=^*()\[\]{}]|\b(?:of|derivative|integral|solve)\b x").is_match(l) {
        return None;
    }
    let hours = hours.unwrap_or(1.0);
    let kg = body_kg(l).unwrap_or(70.0);
    let kcal = met * kg * hours;
    let fat_g = kcal / 7.7;
    let rows = vec![
        ("energy expended", format!("{} kcal ({} kJ)", fmt(kcal.round()), fmt((kcal * 4.184).round()))),
        ("metabolic equivalent", format!("{} MET", r1(met))),
        ("oxygen consumption", format!("{} L", r1(met * 3.5 * kg * hours * 60.0 / 1000.0))),
        ("equivalent fat", format!("{} g ({} oz)", fmt(fat_g.round()), r1(fat_g / 28.3495))),
        ("duration", format!("{} min", fmt((hours * 60.0).round()))),
        ("body mass", format!("{} kg ({} lb)", r1(kg), r1(kg / 0.45359237))),
    ];
    let (title, res) = if wants_fat { ("Fat burned", format!("{} g", fmt(fat_g.round()))) } else { ("Energy expended", format!("{} kcal", fmt(kcal.round()))) };
    let mut extra = vec![pod("Details", "Details", kv(rows))];
    if body_kg(l).is_none() {
        extra.push(pod("Assume", "Assumption", text("body mass 70 kg (154 lb); give your weight, e.g. \"body weight 150 lb\"")));
    }
    extra.push(pod("Source", "Method", text("kcal = MET × mass (kg) × hours; MET values from the Compendium of Physical Activities, ACSM equations on a grade; 7.7 kcal per gram of body fat")));
    Some(ans("exercise", format!("{name}, {} min", fmt((hours * 60.0).round())), title, text(res), extra))
}

fn heart_rate(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:target|max(?:imum)?|training)\s+heart\s+rate\s+(?:zone\s+)?(?:for\s+)?(?:a\s+|an\s+)?(?:age\s+)?(\d{1,3})(?:[- ]?(?:year|yr)s?[- ]?old)?\s*(woman|female|man|male|person)?(?:\s+with\s+(?:a\s+)?resting\s+(?:heart\s+rate\s+)?(?:of\s+)?(\d{2,3}))?$").captures(l)?;
    let age: f64 = m[1].parse().ok()?;
    if !(5.0..=110.0).contains(&age) {
        return None;
    }
    let female = m.get(2).is_some_and(|s| s.as_str().starts_with('w') || s.as_str().starts_with('f'));
    let max_fox = 220.0 - age;
    let max_tanaka = 208.0 - 0.7 * age;
    let max_gulati = 206.0 - 0.88 * age;
    let hrmax = max_fox;
    let mut rows = vec![("maximum (220 − age)", format!("{} bpm", fmt(max_fox))), ("maximum (Tanaka: 208 − 0.7 × age)", format!("{} bpm", r1(max_tanaka)))];
    if female {
        rows.push(("maximum for women (Gulati: 206 − 0.88 × age)", format!("{} bpm", r1(max_gulati))));
    }
    rows.push(("moderate intensity (50–70%)", format!("{}–{} bpm", (0.5 * hrmax).round(), (0.7 * hrmax).round())));
    rows.push(("vigorous intensity (70–85%)", format!("{}–{} bpm", (0.7 * hrmax).round(), (0.85 * hrmax).round())));
    if let Some(r) = m.get(3).and_then(|x| x.as_str().parse::<f64>().ok()) {
        rows.push(("Karvonen 50–85% of heart-rate reserve", format!("{}–{} bpm", (r + 0.5 * (hrmax - r)).round(), (r + 0.85 * (hrmax - r)).round())));
    }
    let who = if female { "woman" } else if m.get(2).is_some() { "man" } else { "person" };
    Some(ans("heart rate", format!("target heart rate for a {} year old {who}", fmt(age)), "Target zone (50–85% of maximum)", text(format!("{}–{} beats per minute", (0.5 * hrmax).round(), (0.85 * hrmax).round())), vec![pod("Zones", "Zones", kv(rows)), pod("Source", "Source", text("American Heart Association target zones; maximum by 220 − age"))]))
}

fn weight_loss(l: &str) -> Option<Answer> {
    if !re!(r"weight loss|lose|losing").is_match(l) {
        return None;
    }
    let age: f64 = re!(r"(\d{2})\s*(?:year|yr|yo\b|y/o)").captures(l)?[1].parse().ok()?;
    let male = re!(r"\b(?:male|man)\b").is_match(l);
    let female = re!(r"\b(?:female|woman)\b").is_match(l);
    if !male && !female {
        return None;
    }
    let h_cm: f64 = if let Some(m) = re!(r"(\d{3})\s*cm").captures(l) {
        m[1].parse().ok()?
    } else {
        let m = re!(r#"(\d)\s*(?:'|ft|feet)\s*(\d{1,2})\s*(?:"|in|inches)?"#).captures(l)?;
        (m[1].parse::<f64>().ok()? * 12.0 + m[2].parse::<f64>().ok()?) * 2.54
    };
    let kgs: Vec<f64> = re!(r"(\d+(?:\.\d+)?)\s*(kg|lbs?|pounds?)").captures_iter(l).filter_map(|m| {
        let v: f64 = m[1].parse().ok()?;
        Some(if m[2].starts_with('k') { v } else { v * 0.45359237 })
    }).collect();
    let (w, loss) = (*kgs.first()?, *kgs.get(1)?);
    let bmr = 10.0 * w + 6.25 * h_cm - 5.0 * age + if male { 5.0 } else { -161.0 };
    let deficit_total = loss * 7700.0;
    let bmi = |kg: f64| kg / (h_cm / 100.0).powi(2);
    let rows = vec![
        ("basal metabolic rate (Mifflin–St Jeor)", format!("{} kcal/day", fmt(bmr.round()))),
        ("maintenance, sedentary (×1.2)", format!("{} kcal/day", fmt((bmr * 1.2).round()))),
        ("maintenance, moderately active (×1.55)", format!("{} kcal/day", fmt((bmr * 1.55).round()))),
        ("total energy deficit needed", format!("{} kcal", fmt(deficit_total))),
        ("time at a 500 kcal/day deficit", format!("{} days (about {} weeks)", fmt((deficit_total / 500.0).round()), fmt((deficit_total / 3500.0).round()))),
        ("time at a 1000 kcal/day deficit", format!("{} days", fmt((deficit_total / 1000.0).round()))),
        ("BMI now → after", format!("{} → {}", r1(bmi(w)), r1(bmi(w - loss)))),
    ];
    Some(ans("weight loss", format!("{} year old {}, {} cm, {} kg, losing {} kg", fmt(age), if male { "male" } else { "female" }, fmt(h_cm.round()), r1(w), r1(loss)), "Time needed at a 500 kcal/day deficit", text(format!("{} weeks", fmt((deficit_total / 3500.0).round()))), vec![pod("Plan", "Plan", kv(rows)), pod("Note", "Assumptions", text("about 7700 kcal per kg of body weight lost; actual loss slows as weight falls"))]))
}

// ---------------------------------------------------------------------------
// calendar facts
// ---------------------------------------------------------------------------

const MONTHS: [&str; 12] = ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];

fn month_index(s: &str) -> Option<usize> {
    let s = s.trim();
    MONTHS.iter().position(|m| *m == s || (s.len() >= 3 && m.starts_with(s)))
}

fn zodiac(m: u32, d: u32) -> (&'static str, &'static str) {
    let signs = [(1, 20, "Capricorn", "December 22 – January 19"), (2, 19, "Aquarius", "January 20 – February 18"), (3, 21, "Pisces", "February 19 – March 20"), (4, 20, "Aries", "March 21 – April 19"), (5, 21, "Taurus", "April 20 – May 20"), (6, 21, "Gemini", "May 21 – June 20"), (7, 23, "Cancer", "June 21 – July 22"), (8, 23, "Leo", "July 23 – August 22"), (9, 23, "Virgo", "August 23 – September 22"), (10, 23, "Libra", "September 23 – October 22"), (11, 22, "Scorpio", "October 23 – November 21"), (12, 22, "Sagittarius", "November 22 – December 21")];
    let (_, start, name, range) = signs[m as usize - 1];
    if d < start {
        (name, range)
    } else {
        let (_, _, n2, r2) = signs[m as usize % 12];
        (n2, r2)
    }
}

const BIRTHSTONES: [&str; 12] = ["garnet", "amethyst", "aquamarine (also bloodstone)", "diamond", "emerald", "pearl (also alexandrite and moonstone)", "ruby", "peridot (also spinel and sardonyx)", "sapphire", "opal (also tourmaline)", "topaz (also citrine)", "turquoise (also tanzanite and zircon)"];
const FLOWERS: [&str; 12] = ["carnation and snowdrop", "violet and primrose", "daffodil and jonquil", "daisy and sweet pea", "lily of the valley and hawthorn", "rose and honeysuckle", "larkspur and water lily", "gladiolus and poppy", "aster and morning glory", "marigold and cosmos", "chrysanthemum", "narcissus and holly"];

fn calendar(l: &str) -> Option<Answer> {
    if re!(r"^(?:which|what) months? (?:have|has|with) (30|31|28|29) days$|^months? with (30|31) days$").is_match(l) {
        let n: u32 = re!(r"(\d\d)").captures(l)?[1].parse().ok()?;
        let list = match n {
            30 => "April, June, September, November",
            31 => "January, March, May, July, August, October, December",
            _ => "February (28 days, 29 in leap years)",
        };
        return Some(ans("months", format!("months with {n} days"), "Result", text(list), vec![pod("Rhyme", "Mnemonic", text("Thirty days hath September, April, June and November; all the rest have thirty-one, save February alone, which has twenty-eight, and twenty-nine in each leap year."))]));
    }
    if let Some(m) = re!(r"^(?:what is the |what's the )?(?:zodiac|star|astrological|sun) sign (?:for |of |on )?(?:someone born (?:on )?)?(.+)$|^(.+?) zodiac sign$").captures(l) {
        let d = crate::domains::misc::parse_date(m.get(1).or(m.get(2))?.as_str()).or_else(|| {
            let t = m.get(1).or(m.get(2))?.as_str();
            let c = re!(r"^([a-z]+)\s+(\d{1,2})(?:st|nd|rd|th)?$").captures(t)?;
            NaiveDate::from_ymd_opt(2000, month_index(&c[1])? as u32 + 1, c[2].parse().ok()?)
        })?;
        let (sign, range) = zodiac(d.month(), d.day());
        let chinese = ["Monkey", "Rooster", "Dog", "Pig", "Rat", "Ox", "Tiger", "Rabbit", "Dragon", "Snake", "Horse", "Goat"];
        let mut extra = vec![pod("Range", "Dates", text(range))];
        if !l.contains("2000") || true {
            extra.push(pod("Chinese", "Chinese zodiac animal of the year", text(format!("{} (approximately: the Chinese year starts in late January or February)", chinese[(d.year().rem_euclid(12)) as usize]))));
        }
        return Some(ans("zodiac", format!("zodiac sign for {}", d.format("%B %-d")), "Western zodiac sign", text(sign), extra));
    }
    if let Some(m) = re!(r"^(?:what is the |what's the )?birth ?stones? (?:for |of )?([a-z]+)$|^([a-z]+) birth ?stones?$").captures(l) {
        let i = month_index(m.get(1).or(m.get(2))?.as_str())?;
        return Some(ans("birthstone", format!("birthstone for {}", capital(MONTHS[i])), "Birthstone", text(BIRTHSTONES[i]), vec![pod("All", "All months", table(None, (0..12).map(|k| vec![text(capital(MONTHS[k])), text(BIRTHSTONES[k])]).collect()))]));
    }
    if let Some(m) = re!(r"^(?:what is the )?(?:birth )?flowers? (?:for |of )?(?:people born in )?([a-z]+)$|^([a-z]+) (?:birth )?flowers?$").captures(l) {
        let i = month_index(m.get(1).or(m.get(2))?.as_str())?;
        return Some(ans("birth flower", format!("birth flower for {}", capital(MONTHS[i])), "Birth flowers", text(FLOWERS[i]), vec![]));
    }
    if re!(r"^(?:list of )?(?:wedding )?anniversary gifts?(?: list)?$|^(?:traditional )?anniversary gifts? by year$").is_match(l) || re!(r"^(\d+)(?:st|nd|rd|th)? (?:wedding )?anniversary gift$").is_match(l) {
        let gifts: &[(u32, &str)] = &[(1, "paper"), (2, "cotton"), (3, "leather"), (4, "fruit and flowers (linen)"), (5, "wood"), (6, "iron (sugar)"), (7, "wool (copper)"), (8, "bronze"), (9, "pottery (willow)"), (10, "tin (aluminium)"), (11, "steel"), (12, "silk (linen)"), (13, "lace"), (14, "ivory"), (15, "crystal"), (20, "china"), (25, "silver"), (30, "pearl"), (35, "coral"), (40, "ruby"), (45, "sapphire"), (50, "gold"), (55, "emerald"), (60, "diamond"), (70, "platinum")];
        if let Some(m) = re!(r"^(\d+)").captures(l) {
            let y: u32 = m[1].parse().ok()?;
            let g = gifts.iter().find(|g| g.0 == y)?;
            return Some(ans("anniversary", format!("{y}th wedding anniversary"), "Traditional gift", text(g.1), vec![]));
        }
        return Some(ans("anniversary", "wedding anniversary gifts".into(), "Traditional gifts", table(Some(vec!["year", "gift"]), gifts.iter().map(|g| vec![text(g.0.to_string()), text(g.1)]).collect()), vec![]));
    }
    // fun days and observances
    let fun: &[(&str, u32, u32)] = &[("talk like a pirate day", 9, 19), ("national chocolate day", 10, 28), ("world chocolate day", 7, 7), ("star wars day", 5, 4), ("pi day", 3, 14), ("tau day", 6, 28), ("towel day", 5, 25), ("earth day", 4, 22), ("world kindness day", 11, 13), ("international women's day", 3, 8), ("world environment day", 6, 5), ("national donut day", 6, 6), ("groundhog day", 2, 2), ("april fools' day", 4, 1), ("april fools day", 4, 1), ("cinco de mayo", 5, 5), ("bastille day", 7, 14), ("juneteenth", 6, 19), ("veterans day", 11, 11), ("remembrance day", 11, 11), ("boxing day", 12, 26)];
    if let Some(m) = re!(r"^(?:when is |next |days until |how many days until |date of )?(?:the )?(?:next )?(.+?)(?: \d{4})?$").captures(l) {
        if let Some((name, mo, d)) = fun.iter().find(|f| f.0 == m[1].trim()) {
            let today = crate::domains::misc::today();
            let mut date = NaiveDate::from_ymd_opt(today.year(), *mo, *d)?;
            if date < today {
                date = NaiveDate::from_ymd_opt(today.year() + 1, *mo, *d)?;
            }
            let days = (date - today).num_days();
            return Some(ans("observance", capital(name), "Next date", text(date.format("%A, %B %-d, %Y").to_string()), vec![pod("Until", "Time from today", text(format!("{days} days")))]));
        }
    }
    // relative days
    let today = crate::domains::misc::today();
    let rel = match l {
        "tomorrow" => Some((today + chrono::Duration::days(1), "tomorrow")),
        "yesterday" => Some((today - chrono::Duration::days(1), "yesterday")),
        "today" | "what is today" | "what's today" | "today's date" | "what is the date" | "what is today's date" => Some((today, "today")),
        "next week" | "a week from today" | "one week from today" => Some((today + chrono::Duration::days(7), "one week from today")),
        "last week" => Some((today - chrono::Duration::days(7), "one week ago")),
        _ => None,
    };
    let rel = rel.map(|(d, l)| (d, l.to_string())).or_else(|| {
        let m = re!(r"^(next|last|this|previous|coming) (monday|tuesday|wednesday|thursday|friday|saturday|sunday|month|year)$").captures(l)?;
        let fwd = matches!(&m[1], "next" | "coming" | "this");
        let unit = &m[2];
        let d = match unit {
            "month" | "year" => {
                let months: i32 = if &m[1] == "this" { 0 } else { (if unit == "month" { 1 } else { 12 }) * if fwd { 1 } else { -1 } };
                let total = today.year() * 12 + today.month0() as i32 + months;
                let (y, mo) = (total.div_euclid(12), total.rem_euclid(12) as u32 + 1);
                NaiveDate::from_ymd_opt(y, mo, 1)?
            }
            w => {
                let target: chrono::Weekday = w.parse().ok()?;
                let cur = today.weekday().num_days_from_monday() as i64;
                let tgt = target.num_days_from_monday() as i64;
                let mut delta = if fwd { (tgt - cur).rem_euclid(7) } else { -((cur - tgt).rem_euclid(7)) };
                if delta == 0 && &m[1] != "this" {
                    delta = if fwd { 7 } else { -7 };
                }
                today + chrono::Duration::days(delta)
            }
        };
        Some((d, m[0].to_string()))
    });
    if let Some(m) = re!(r"^(next|last|this) (month|year)$").captures(l) {
        let (d, _) = rel?;
        let (title, main) = if &m[2] == "month" { (format!("{} month", &m[1]), d.format("%B %Y").to_string()) } else { (format!("{} year", &m[1]), d.format("%Y").to_string()) };
        let last = if &m[2] == "month" { NaiveDate::from_ymd_opt(if d.month() == 12 { d.year() + 1 } else { d.year() }, d.month() % 12 + 1, 1)? - chrono::Duration::days(1) } else { NaiveDate::from_ymd_opt(d.year(), 12, 31)? };
        return Some(ans("date", title, "Result", text(main), vec![pod("Span", "Span", text(format!("{} – {}", d.format("%A, %B %-d, %Y"), last.format("%A, %B %-d, %Y")))), pod("Days", "Length", text(format!("{} days", (last - d).num_days() + 1)))]));
    }
    if let Some((d, label)) = rel {
        let mut extra = vec![pod("Week", "Week", text(format!("ISO week {} of {}", d.iso_week().week(), d.iso_week().year())))];
        if label.contains("week") {
            let monday = d - chrono::Duration::days(d.weekday().num_days_from_monday() as i64);
            extra.push(pod("Span", "That week (Monday to Sunday)", text(format!("{} – {}", monday.format("%B %-d"), (monday + chrono::Duration::days(6)).format("%B %-d, %Y")))));
        }
        extra.push(pod("DayOfYear", "Day of the year", text(format!("day {} of {}", d.ordinal(), if d.leap_year() { 366 } else { 365 }))));
        return Some(ans("date", label.to_string(), "Date", text(d.format("%A, %B %-d, %Y").to_string()), extra));
    }
    if let Some(m) = re!(r"^(?:how long|how much time|time) (?:till|until|to|before) (\d{1,2})(?::(\d\d))?\s*(am|pm)?$").captures(l) {
        let mut h: u32 = m[1].parse().ok()?;
        let mi: u32 = m.get(2).map(|x| x.as_str().parse().unwrap_or(0)).unwrap_or(0);
        match m.get(3).map(|x| x.as_str()) {
            Some("pm") if h < 12 => h += 12,
            Some("am") if h == 12 => h = 0,
            _ => {}
        }
        let target = NaiveTime::from_hms_opt(h, mi, 0)?;
        let now = chrono::Local::now();
        let mut diff = target.signed_duration_since(now.time());
        if diff.num_seconds() < 0 {
            diff += chrono::Duration::days(1);
        }
        let secs = diff.num_seconds();
        return Some(ans("time until", format!("time until {}", target.format("%-I:%M %p")), "Time remaining", text(format!("{} h {} min", secs / 3600, (secs % 3600) / 60)), vec![pod("Now", "Current local time", text(now.format("%-I:%M %p").to_string()))]));
    }
    // Orthodox Easter
    if let Some(m) = re!(r"^(?:will |is )?orthodox easter (?:be )?(?:on |in )?(.+?)$|^(?:when is )?orthodox easter (\d{4})$").captures(l) {
        let t = m.get(1).or(m.get(2))?.as_str();
        let (year, check) = if let Ok(y) = t.parse::<i32>() { (y, None) } else {
            let d = crate::domains::misc::parse_date(t)?;
            (d.year(), Some(d))
        };
        let e = orthodox_easter(year)?;
        let res = match check {
            Some(d) => format!("{}: Orthodox Easter {year} is on {}", if d == e { "yes" } else { "no" }, e.format("%A, %B %-d, %Y")),
            None => e.format("%A, %B %-d, %Y").to_string(),
        };
        return Some(ans("Orthodox Easter", format!("Orthodox Easter {year}"), "Result", text(res), vec![pod("Method", "Method", text("Julian computus (Meeus), converted to the Gregorian calendar"))]));
    }
    None
}

fn orthodox_easter(y: i32) -> Option<NaiveDate> {
    let a = y % 4;
    let b = y % 7;
    let c = y % 19;
    let d = (19 * c + 15) % 30;
    let e = (2 * a + 4 * b - d + 34) % 7;
    let month = (d + e + 114) / 31;
    let day = (d + e + 114) % 31 + 1;
    let julian = NaiveDate::from_ymd_opt(y, month as u32, day as u32)?;
    // Julian → Gregorian offset for March–May of year y
    let offset = y / 100 - y / 400 - 2;
    Some(julian + chrono::Duration::days(offset as i64))
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// time zones
// ---------------------------------------------------------------------------

pub(crate) fn zone(name: &str) -> Option<(&'static str, &'static str)> {
    let n = name.trim().trim_end_matches(" time zone").trim_end_matches(" time").trim_end_matches(" standard").trim();
    Some(match n {
        "est" | "edt" | "et" | "eastern" | "us eastern" => ("America/New_York", "US Eastern Time"),
        "cst" | "cdt" | "ct" | "central" | "us central" => ("America/Chicago", "US Central Time"),
        "mst" | "mdt" | "mt" | "mountain" => ("America/Denver", "US Mountain Time"),
        "pst" | "pdt" | "pt" | "pacific" | "us pacific" => ("America/Los_Angeles", "US Pacific Time"),
        "akst" | "alaska" => ("America/Anchorage", "Alaska Time"),
        "hst" | "hawaii" => ("Pacific/Honolulu", "Hawaii Time"),
        "ast" | "atlantic" => ("America/Halifax", "Atlantic Time"),
        "nst" | "newfoundland" => ("America/St_Johns", "Newfoundland Time"),
        "gmt" | "utc" | "zulu" | "z" | "greenwich mean" | "greenwich" | "coordinated universal" | "universal" => ("Etc/UTC", "Coordinated Universal Time"),
        "bst" | "uk" | "british" | "london" => ("Europe/London", "UK time (GMT/BST)"),
        "cet" | "cest" | "central european" | "central europe" => ("Europe/Paris", "Central European Time"),
        "eet" | "eest" | "eastern european" => ("Europe/Athens", "Eastern European Time"),
        "wet" | "western european" => ("Europe/Lisbon", "Western European Time"),
        "msk" | "moscow" => ("Europe/Moscow", "Moscow Time"),
        "ist" | "india" | "indian" => ("Asia/Kolkata", "India Standard Time (UTC+5:30)"),
        "pkt" | "pakistan" => ("Asia/Karachi", "Pakistan Standard Time"),
        "jst" | "japan" => ("Asia/Tokyo", "Japan Standard Time"),
        "kst" | "korea" => ("Asia/Seoul", "Korea Standard Time"),
        "china" | "beijing" | "cst china" => ("Asia/Shanghai", "China Standard Time"),
        "hkt" | "hong kong" => ("Asia/Hong_Kong", "Hong Kong Time"),
        "sgt" | "singapore" => ("Asia/Singapore", "Singapore Time"),
        "wib" | "jakarta" => ("Asia/Jakarta", "Western Indonesia Time"),
        "pht" | "philippine" => ("Asia/Manila", "Philippine Time"),
        "aest" | "aedt" | "australian eastern" | "australia eastern" | "eastern australia" | "sydney" => ("Australia/Sydney", "Australian Eastern Time"),
        "acst" | "australian central" | "australia central" | "central australia" => ("Australia/Adelaide", "Australian Central Time"),
        "awst" | "australian western" | "australia western" | "western australia" | "perth" => ("Australia/Perth", "Australian Western Time"),
        "nzst" | "nzdt" | "new zealand" | "nz" => ("Pacific/Auckland", "New Zealand Time"),
        "art" | "argentina" => ("America/Argentina/Buenos_Aires", "Argentina Time"),
        "brt" | "brasilia" | "brazil" => ("America/Sao_Paulo", "Brasília Time"),
        "cat" | "central africa" | "central african" => ("Africa/Maputo", "Central Africa Time"),
        "eat" | "east africa" | "east african" => ("Africa/Nairobi", "East Africa Time"),
        "wat" | "west africa" | "west african" => ("Africa/Lagos", "West Africa Time"),
        "sast" | "south africa" => ("Africa/Johannesburg", "South Africa Standard Time"),
        "gst" | "gulf" | "dubai" => ("Asia/Dubai", "Gulf Standard Time"),
        "irst" | "iran" => ("Asia/Tehran", "Iran Standard Time"),
        _ => return None,
    })
}

fn offset_str(secs: i32) -> String {
    let sign = if secs < 0 { '−' } else { '+' };
    let a = secs.abs();
    if a % 3600 == 0 { format!("UTC{sign}{}", a / 3600) } else { format!("UTC{sign}{}:{:02}", a / 3600, (a % 3600) / 60) }
}

fn timezones(l: &str) -> Option<Answer> {
    use chrono::{Offset, TimeZone};
    let now = chrono::Utc::now();
    let info = |tzname: &str| -> Option<(chrono::DateTime<chrono_tz::Tz>, i32)> {
        let tz: chrono_tz::Tz = tzname.parse().ok()?;
        let t = now.with_timezone(&tz);
        let off = t.offset().fix().local_minus_utc();
        Some((t, off))
    };
    // conversion: "convert noon ART to CAT", "3pm EST in CET", "convert New Zealand time zone to central European time zone", "EST to PST"
    if let Some(m) = re!(r"^(?:convert\s+)?(?:(noon|midnight|\d{1,2}(?::\d\d)?\s*(?:am|pm)?)\s+)?([a-z ]+?)(?:\s+time zone|\s+time)?\s+(?:to|in|into)\s+([a-z ]+?)(?:\s+time zone|\s+time)?$").captures(l) {
        let (from, from_name) = zone(&m[2])?;
        let (to, to_name) = zone(&m[3])?;
        let (_, off1) = info(from)?;
        let (_, off2) = info(to)?;
        let diff = off2 - off1;
        let mut rows = vec![(from_name, offset_str(off1)), (to_name, offset_str(off2)), ("difference", format!("{}{} h", if diff >= 0 { "+" } else { "−" }, fmt((diff.abs() as f64) / 3600.0)))];
        let res = if let Some(t) = m.get(1) {
            let t = t.as_str();
            let (h, mi) = match t {
                "noon" => (12, 0),
                "midnight" => (0, 0),
                _ => {
                    let c = re!(r"(\d{1,2})(?::(\d\d))?\s*(am|pm)?").captures(t)?;
                    let mut h: u32 = c[1].parse().ok()?;
                    let mi: u32 = c.get(2).map(|x| x.as_str().parse().unwrap_or(0)).unwrap_or(0);
                    match c.get(3).map(|x| x.as_str()) {
                        Some("pm") if h < 12 => h += 12,
                        Some("am") if h == 12 => h = 0,
                        _ => {}
                    }
                    (h, mi)
                }
            };
            let tz1: chrono_tz::Tz = from.parse().ok()?;
            let tz2: chrono_tz::Tz = to.parse().ok()?;
            let today = now.with_timezone(&tz1).date_naive();
            let local = tz1.from_local_datetime(&today.and_hms_opt(h, mi, 0)?).single()?;
            let conv = local.with_timezone(&tz2);
            let day_note = match (conv.date_naive() - today).num_days() {
                1 => " (next day)",
                -1 => " (previous day)",
                _ => "",
            };
            rows.insert(0, ("source time", local.format("%-I:%M %p %Z, %A").to_string()));
            format!("{}{day_note}", conv.format("%-I:%M %p %Z"))
        } else {
            let (t1, _) = info(from)?;
            let (t2, _) = info(to)?;
            rows.insert(0, ("now in both zones", format!("{} = {}", t1.format("%-I:%M %p %Z"), t2.format("%-I:%M %p %Z"))));
            format!("{} is {} {} {}", to_name, fmt((diff.abs() as f64) / 3600.0), if diff >= 0 { "hours ahead of" } else { "hours behind" }, from_name)
        };
        return Some(ans("time zones", format!("{from_name} to {to_name}"), "Result", text(res), vec![pod("Details", "Details", kv(rows))]));
    }
    // single zone
    let m = re!(r"^(?:what is |current time in |time in )?([a-z ]+?)(?:\s+time zone|\s+time|\s+timezone)$").captures(l)?;
    let (tzn, name) = zone(&m[1])?;
    let (t, off) = info(tzn)?;
    Some(ans("time zone", name.to_string(), "Current time", text(t.format("%-I:%M %p %Z, %A, %B %-d, %Y").to_string()), vec![pod("Offset", "Offset from UTC now", text(offset_str(off))), pod("Zone", "IANA zone", text(tzn))]))
}

// ---------------------------------------------------------------------------
// betting
// ---------------------------------------------------------------------------

fn betting(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(\d+)\s*(?::|to|/)\s*(\d+)\s+odds,?\s*(?:bet|stake|wager)\s+(?:\$|€|£)?(\d+(?:\.\d+)?)\s*([a-z$€£]*)$").captures(l) {
        let (a, b, s): (f64, f64, f64) = (m[1].parse().ok()?, m[2].parse().ok()?, m[3].parse().ok()?);
        let cur = m[4].to_string();
        let win = s * a / b;
        let rows = vec![("profit if the bet wins", format!("{} {cur}", fmt((win * 100.0).round() / 100.0))), ("total returned", format!("{} {cur}", fmt(((win + s) * 100.0).round() / 100.0))), ("decimal odds", fmt(1.0 + a / b)), ("implied probability", format!("{}/{} ≈ {}%", fmt(b), fmt(a + b), r1(100.0 * b / (a + b))))];
        return Some(ans("odds", format!("{}:{} odds, stake {} {cur}", fmt(a), fmt(b), fmt(s)), "Payout", kv(rows), vec![]));
    }
    let m = re!(r"^(?:bet (?:the |a |on (?:a |the )?)?)?(straight(?: up)?|single number|split|street|corner|square|five[- ]number|basket|six[- ]line|line|dozen|column|red|black|odd|even|high|low)(?: bet)?(?: (?:at|in) (?:american |european )?roulette)$|^roulette (straight|split|street|corner|six line|dozen|column|red|black|odd|even)(?: bet)?$").captures(l)?;
    let kind = m.get(1).or(m.get(2))?.as_str();
    let (n, pay) = match kind {
        "straight" | "straight up" | "single number" => (1.0, 35.0),
        "split" => (2.0, 17.0),
        "street" => (3.0, 11.0),
        "corner" | "square" => (4.0, 8.0),
        "five-number" | "five number" | "basket" => (5.0, 6.0),
        "six-line" | "six line" | "line" => (6.0, 5.0),
        "dozen" | "column" => (12.0, 2.0),
        _ => (18.0, 1.0),
    };
    let ev = |pockets: f64| (n * (pay + 1.0) - pockets) / pockets;
    let rows = vec![("numbers covered", fmt(n)), ("payout", format!("{}:1", fmt(pay))), ("win probability, European (37 pockets)", format!("{}/37 ≈ {}%", fmt(n), r1(100.0 * n / 37.0))), ("win probability, American (38 pockets)", format!("{}/38 ≈ {}%", fmt(n), r1(100.0 * n / 38.0))), ("expected return per unit, European", format!("{} ({}% house edge)", fmt((ev(37.0) * 10000.0).round() / 10000.0), r1(-100.0 * ev(37.0)))), ("expected return per unit, American", format!("{} ({}% house edge)", fmt((ev(38.0) * 10000.0).round() / 10000.0), r1(-100.0 * ev(38.0))))];
    Some(ans("roulette", format!("{kind} bet at roulette"), "Bet", kv(rows), vec![]))
}

// ---------------------------------------------------------------------------
// photography and craft sizes
// ---------------------------------------------------------------------------

fn photography(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r#"^(\d+(?:\.\d+)?)\s*(?:"|in(?:ch(?:es)?)?)?\s*(?:x|×|by)\s*(\d+(?:\.\d+)?)\s*(?:"|in(?:ch(?:es)?)?)?\s*(?:photo|print|image|picture)?\s*(?:at|@)\s*(\d+)\s*(?:ppi|dpi)$"#).captures(l) {
        let (w, h, d): (f64, f64, f64) = (m[1].parse().ok()?, m[2].parse().ok()?, m[3].parse().ok()?);
        let (pw, ph) = ((w * d).round(), (h * d).round());
        return Some(ans("print size", format!("{} in × {} in print at {} ppi", fmt(w), fmt(h), fmt(d)), "Pixels needed", text(format!("{} × {} pixels", fmt(pw), fmt(ph))), vec![pod("Details", "Details", kv(vec![("total", format!("{} megapixels", fmt(pw * ph / 1e6))), ("for 300 ppi photo quality", format!("{} × {} pixels", fmt(w * 300.0), fmt(h * 300.0)))]))]));
    }
    if let Some(m) = re!(r"^hyperfocal distance,?\s*(?:f[- ]?(?:stop|number)?\s*|aperture\s*)?(?:f/)?(\d+(?:\.\d+)?)(?:,?\s*(?:focal length\s*)?(\d+(?:\.\d+)?)\s*mm)?(?:,?\s*(?:coc|circle of confusion)\s*(\d*\.\d+)\s*mm)?$").captures(l) {
        let n: f64 = m[1].parse().ok()?;
        let f: f64 = m.get(2).and_then(|x| x.as_str().parse().ok()).unwrap_or(50.0);
        let c: f64 = m.get(3).and_then(|x| x.as_str().parse().ok()).unwrap_or(0.03);
        let h = f * f / (n * c) + f;
        let mut extra = vec![pod("Details", "Details", kv(vec![("near limit of sharpness when focused there", format!("{} m", fmt((h / 2000.0 * 100.0).round() / 100.0))), ("formula", "H = f²/(N c) + f".into())]))];
        if m.get(2).is_none() || m.get(3).is_none() {
            extra.push(pod("Assume", "Assumptions", text(format!("focal length {} mm, circle of confusion {} mm (35 mm full frame)", fmt(f), fmt(c)))));
        }
        return Some(ans("hyperfocal distance", format!("hyperfocal distance at f/{} with a {} mm lens", fmt(n), fmt(f)), "Hyperfocal distance", text(format!("{} m ({} ft)", fmt((h / 1000.0 * 100.0).round() / 100.0), fmt((h / 304.8 * 10.0).round() / 10.0))), extra));
    }
    if let Some(m) = re!(r"^f[- ]?stop (?:arithmetic|calculator|scale)?\s*(?:f/)?(\d+(?:\.\d+)?)$").captures(l) {
        let n: f64 = m[1].parse().ok()?;
        let stops = 2.0 * n.log2();
        let full = [1.0, 1.4, 2.0, 2.8, 4.0, 5.6, 8.0, 11.0, 16.0, 22.0, 32.0, 45.0, 64.0];
        let idx = full.iter().position(|v| (v - n).abs() < 0.05);
        let mut rows = vec![("stops from f/1", r1(stops)), ("light relative to f/1", format!("1/{}", fmt((2f64.powf(stops) * 10.0).round() / 10.0))), ("one stop wider (2× light)", format!("f/{}", r1(n / 2f64.sqrt()))), ("one stop narrower (½ light)", format!("f/{}", r1(n * 2f64.sqrt())))];
        if let Some(i) = idx {
            if i > 0 {
                rows[2].1 = format!("f/{}", fmt(full[i - 1]));
            }
            if i + 1 < full.len() {
                rows[3].1 = format!("f/{}", fmt(full[i + 1]));
            }
        }
        rows.push(("full-stop scale", full.iter().map(|v| format!("f/{}", fmt(*v))).collect::<Vec<_>>().join(", ")));
        return Some(ans("f-stop", format!("f/{}", fmt(n)), "Exposure value offset", text(format!("{} stops below f/1", r1(stops))), vec![pod("Details", "Details", kv(rows))]));
    }
    None
}

/// US knitting needle size → mm; UK/imperial size → mm.
const NEEDLES_US: &[(&str, f64)] = &[("000", 1.5), ("00", 1.75), ("0", 2.0), ("1", 2.25), ("1.5", 2.5), ("2", 2.75), ("2.5", 3.0), ("3", 3.25), ("4", 3.5), ("5", 3.75), ("6", 4.0), ("7", 4.5), ("8", 5.0), ("9", 5.5), ("10", 6.0), ("10.5", 6.5), ("11", 8.0), ("13", 9.0), ("15", 10.0), ("17", 12.0), ("19", 15.0), ("35", 19.0), ("50", 25.0)];
const NEEDLES_UK: &[(&str, f64)] = &[("14", 2.0), ("13", 2.25), ("12", 2.75), ("11", 3.0), ("10", 3.25), ("9", 3.75), ("8", 4.0), ("7", 4.5), ("6", 5.0), ("5", 5.5), ("4", 6.0), ("3", 6.5), ("2", 7.0), ("1", 7.5), ("0", 8.0), ("00", 9.0), ("000", 10.0)];

fn crafts(l: &str) -> Option<Answer> {
    // ring sizes: US size s has inner diameter 11.63 + 0.8128 s mm
    if let Some(m) = re!(r"^(?:(?:inner )?(diameter|circumference) of (?:a |an )?)?(?:us )?ring size (\d+(?:\.\d+)?)(?: in mm)?$|^(?:what is the )?german equivalent ring size (?:to|for|of) (?:us )?ring size (\d+(?:\.\d+)?)$").captures(l) {
        let s: f64 = m.get(2).or(m.get(3))?.as_str().parse().ok()?;
        if s > 16.0 {
            return None;
        }
        let d = 11.63 + 0.8128 * s;
        let c = d * std::f64::consts::PI;
        let rows = vec![("inner diameter", format!("{} mm ({} in)", fmt((d * 100.0).round() / 100.0), fmt((d / 25.4 * 1000.0).round() / 1000.0))), ("inner circumference", format!("{} mm", r1(c))), ("German size (diameter in mm)", r1(d)), ("ISO / French size (circumference in mm)", r1(c)), ("UK size", uk_ring(c))];
        let want_c = m.get(1).is_some_and(|x| x.as_str() == "circumference");
        let (title, res) = if m.get(3).is_some() { ("German ring size", r1(d)) } else if want_c { ("Inner circumference", format!("{} mm", r1(c))) } else { ("Inner diameter", format!("{} mm", fmt((d * 100.0).round() / 100.0))) };
        return Some(ans("ring size", format!("US ring size {}", fmt(s)), title, text(res), vec![pod("Details", "Sizes", kv(rows))]));
    }
    // knitting needles
    if l.contains("knitting needle") {
        let target = re!(r"\bin (us|uk|imperial|metric|mm)\b.*$").captures(l).map(|m| m[1].to_string());
        let body = re!(r"\bin (us|uk|imperial|metric|mm)\b.*$").replace(l, "").to_string();
        let sm = re!(r"(\d+(?:\.\d+)?|000|00)\s*(mm|millimeters?)?").captures(&body)?;
        let size = sm[1].to_string();
        let sys = if sm.get(2).is_some() || body.contains("metric") { "metric" } else if re!(r"\b(?:uk|imperial|british)\b").is_match(&body) { "uk" } else { "us" };
        let mm: f64 = match sys {
            "metric" => size.parse().ok()?,
            "uk" => NEEDLES_UK.iter().find(|n| n.0 == size)?.1,
            _ => NEEDLES_US.iter().find(|n| n.0 == size)?.1,
        };
        let us = NEEDLES_US.iter().find(|n| (n.1 - mm).abs() < 0.01).map(|n| n.0.to_string()).unwrap_or_else(|| "none".into());
        let uk = NEEDLES_UK.iter().find(|n| (n.1 - mm).abs() < 0.01).map(|n| n.0.to_string()).unwrap_or_else(|| "none".into());
        let rows = vec![("metric", format!("{} mm", fmt(mm))), ("US", us.clone()), ("UK / imperial", uk.clone()), ("diameter in inches", fmt((mm / 25.4 * 1000.0).round() / 1000.0)), ("radius", format!("{} mm", fmt(mm / 2.0)))];
        let (title, res) = if body.contains("radius") {
            ("Radius", format!("{} mm", fmt(mm / 2.0)))
        } else {
            match target.as_deref() {
                Some("us") => ("US size", us),
                Some("uk" | "imperial") => ("UK size", uk),
                _ => ("Diameter", format!("{} mm", fmt(mm))),
            }
        };
        return Some(ans("knitting needle", format!("{} size {size} knitting needle", match sys { "uk" => "UK", "metric" => "metric", _ => "US" }), title, text(res), vec![pod("Sizes", "Equivalent sizes", kv(rows))]));
    }
    // lumber: nominal → actual
    if let Some(m) = re!(r"^(?:actual (thickness|width|size|dimensions?) of (?:a )?)?(\d+)\s*(?:x|×|by)\s*(\d+)(?:\s*(?:x|×|by)\s*(\d+))?(?:\s+[a-z ]*?)?(?:\s+lumber|\s+board)?(?:\s+actual (thickness|width|size|dimensions?))?$").captures(l) {
        if !re!(r"actual|lumber|board|pine|spruce|fir|cedar|oak").is_match(l) {
            return None;
        }
        let act = |n: u32| -> Option<f64> {
            Some(match n {
                1 => 0.75,
                2 => 1.5,
                3 => 2.5,
                4 => 3.5,
                5 => 4.5,
                6 => 5.5,
                8 => 7.25,
                10 => 9.25,
                12 => 11.25,
                14 => 13.25,
                16 => 15.25,
                _ => return None,
            })
        };
        let (a, b): (u32, u32) = (m[2].parse().ok()?, m[3].parse().ok()?);
        let (ta, tb) = (act(a)?, act(b)?);
        let what = m.get(1).or(m.get(5)).map(|x| x.as_str()).unwrap_or("size");
        let res = match what {
            "thickness" => format!("{} in ({} mm)", fmt(ta), r1(ta * 25.4)),
            "width" => format!("{} in ({} mm)", fmt(tb), r1(tb * 25.4)),
            _ => format!("{} in × {} in ({} mm × {} mm)", fmt(ta), fmt(tb), r1(ta * 25.4), r1(tb * 25.4)),
        };
        let mut rows = vec![("nominal size", format!("{a} × {b}")), ("actual dressed size", format!("{} in × {} in", fmt(ta), fmt(tb)))];
        if let Some(len) = m.get(4) {
            rows.push(("length", format!("{} ft (lengths are not reduced)", len.as_str())));
        }
        return Some(ans("lumber", format!("nominal {a} × {b} softwood lumber"), &format!("Actual {what}"), text(res), vec![pod("Details", "Details", kv(rows)), pod("Note", "Standard", text("US softwood lumber, dry and surfaced four sides (American Softwood Lumber Standard PS 20)"))]));
    }
    // paint coverage
    if let Some(m) = re!(r"^(\d+(?:\.\d+)?)\s*(?:sq(?:uare)?\.?\s*(?:ft|feet|foot)|ft2|ft²)\s+(?:of\s+)?(gloss |semi-gloss |satin |eggshell |flat |matte )?(paint|primer)$").captures(l) {
        let area: f64 = m[1].parse().ok()?;
        let per = if &m[3] == "primer" { 300.0 } else { 375.0 };
        let gal = area / per;
        return Some(ans("paint", format!("{} sq ft of {}{}", fmt(area), m.get(2).map(|x| x.as_str()).unwrap_or(""), &m[3]), "Paint needed (one coat)", text(format!("{} US gallons ({} L)", r1(gal), r1(gal * 3.785411784))), vec![pod("Details", "Details", kv(vec![("two coats", format!("{} gallons", r1(2.0 * gal))), ("coverage assumed", format!("{} sq ft per gallon", fmt(per)))]))]));
    }
    if let Some(m) = re!(r"^(\d+(?:\.\d+)?)\s*(?:us\s+)?gallons? of (gloss |semi-gloss |satin |eggshell |flat |matte )?(paint|primer)$").captures(l) {
        let g: f64 = m[1].parse().ok()?;
        let per = if &m[3] == "primer" { 300.0 } else { 375.0 };
        return Some(ans("paint", format!("{} gallons of {}", fmt(g), &m[3]), "Area covered (one coat)", text(format!("about {} sq ft ({} m²)", fmt(g * per), fmt((g * per * 0.09290304).round()))), vec![pod("Details", "Details", kv(vec![("coverage assumed", format!("{} sq ft per gallon (typical range {})", fmt(per), if per == 300.0 { "200–300" } else { "350–400" })), ("volume", format!("{} L", r1(g * 3.785411784)))]))]));
    }
    None
}

fn uk_ring(circ_mm: f64) -> String {
    // UK sizes A..Z step about 1.25 mm of circumference from A = 37.8 mm
    let idx = ((circ_mm - 37.8) / 1.25).round();
    if (0.0..26.0).contains(&idx) { ((b'A' + idx as u8) as char).to_string() } else { "out of range".into() }
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").replace('‐', "-").to_string();
    heart_rate(&l).or_else(|| weight_loss(&l)).or_else(|| calendar(&l)).or_else(|| timezones(&l)).or_else(|| betting(&l)).or_else(|| photography(&l)).or_else(|| crafts(&l)).or_else(|| exercise(&l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orthodox_easter_dates() {
        assert_eq!(orthodox_easter(2024), NaiveDate::from_ymd_opt(2024, 5, 5));
        assert_eq!(orthodox_easter(2025), NaiveDate::from_ymd_opt(2025, 4, 20));
        assert_eq!(orthodox_easter(2023), NaiveDate::from_ymd_opt(2023, 4, 16));
    }

    #[test]
    fn zodiac_edges() {
        assert_eq!(zodiac(11, 22).0, "Sagittarius");
        assert_eq!(zodiac(11, 21).0, "Scorpio");
        assert_eq!(zodiac(1, 19).0, "Capricorn");
        assert_eq!(zodiac(12, 25).0, "Capricorn");
    }
}
