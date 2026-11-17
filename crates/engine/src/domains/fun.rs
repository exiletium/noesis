//! Lighter queries: greetings, jokes, tongue twisters, nursery rhymes and
//! proverbs, Star Trek warp factors, words spelled backwards, words read
//! as numbers in bases up to 36, the look-and-say sequence, obfuscated
//! numbers, and cat and dog years.

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

fn reply(title: &str, inp: &str, body: &str) -> Answer {
    Answer::new(title, vec![input(inp), primary(pod("Result", "Response", text(body)))])
}

/// Stable choice from a list, varying by query text.
fn pick<'a>(q: &str, items: &[&'a str]) -> &'a str {
    let h = q.bytes().fold(5381u64, |h, b| h.wrapping_mul(33) ^ b as u64);
    let day = chrono::Local::now().timestamp() as u64 / 86400;
    items[((h ^ day) % items.len() as u64) as usize]
}

const MATH_JOKES: &[&str] = &[
    "Why was six afraid of seven? Because seven ate nine.",
    "There are 10 kinds of people: those who understand binary and those who don't.",
    "Parallel lines have so much in common. It's a shame they'll never meet.",
    "Why did the student get upset when the teacher called him average? It was a mean thing to say.",
    "I'd tell you a joke about infinity, but it doesn't have an ending.",
    "A statistician drowned crossing a river that was three feet deep on average.",
];
const PHYSICS_JOKES: &[&str] = &[
    "A neutron walks into a bar and asks how much a drink costs. The bartender says: for you, no charge.",
    "Why can't you trust atoms? They make up everything.",
    "Schrödinger's cat walks into a bar. And doesn't.",
    "What did the photon say when asked if it needed help with its luggage? No thanks, I'm traveling light.",
    "Heisenberg gets pulled over. \"Do you know how fast you were going?\" \"No, but I know exactly where I am.\"",
];
const CHEM_JOKES: &[&str] = &[
    "I would tell you a chemistry joke, but I know I wouldn't get a reaction.",
    "Two chemists walk into a bar. The first says \"I'll have some H₂O.\" The second says \"I'll have some H₂O too.\" The second one died.",
    "If you're not part of the solution, you're part of the precipitate.",
    "What do you do with a sick chemist? If you can't helium and you can't curium, you might as well barium.",
    "Sodium walks into a bar with chlorine. Things get salty.",
];
const GENERAL_JOKES: &[&str] = &[
    "I told my computer I needed a break, and now it won't stop sending me vacation ads.",
    "Why do programmers prefer dark mode? Because light attracts bugs.",
    "I'm reading a book about anti-gravity. It's impossible to put down.",
    "Why did the scarecrow win an award? He was outstanding in his field.",
];

const RHYMES: &[(&[&str], &str, &str)] = &[
    (&["she sells seashells", "she sells sea shells", "she sells seashells by the seashore"], "tongue twister", "She sells seashells by the seashore,\nThe shells she sells are surely seashells.\nSo if she sells shells on the seashore,\nI'm sure she sells seashore shells."),
    (&["flute tooting tutor", "a tutor who tooted the flute"], "tongue twister (Carolyn Wells)", "A tutor who tooted the flute\nTried to tutor two tooters to toot.\nSaid the two to the tutor,\n\"Is it harder to toot, or\nTo tutor two tooters to toot?\""),
    (&["peter piper", "peter piper picked a peck of pickled peppers"], "tongue twister", "Peter Piper picked a peck of pickled peppers.\nA peck of pickled peppers Peter Piper picked.\nIf Peter Piper picked a peck of pickled peppers,\nWhere's the peck of pickled peppers Peter Piper picked?"),
    (&["monday's child", "monday's child poem", "mondays child"], "traditional fortune-telling rhyme", "Monday's child is fair of face,\nTuesday's child is full of grace,\nWednesday's child is full of woe,\nThursday's child has far to go,\nFriday's child is loving and giving,\nSaturday's child works hard for a living,\nAnd the child that is born on the Sabbath day\nIs bonny and blithe, and good and gay."),
    (&["hickory dickory dock", "hickory, dickory, dock"], "nursery rhyme", "Hickory, dickory, dock,\nThe mouse ran up the clock.\nThe clock struck one,\nThe mouse ran down,\nHickory, dickory, dock."),
    (&["twinkle twinkle little star", "twinkle, twinkle, little star"], "nursery rhyme (Jane Taylor, 1806)", "Twinkle, twinkle, little star,\nHow I wonder what you are!\nUp above the world so high,\nLike a diamond in the sky."),
    (&["humpty dumpty"], "nursery rhyme", "Humpty Dumpty sat on a wall,\nHumpty Dumpty had a great fall.\nAll the king's horses and all the king's men\nCouldn't put Humpty together again."),
    (&["a penny saved", "a penny saved is"], "proverb", "A penny saved is a penny earned."),
    (&["an apple a day"], "proverb", "An apple a day keeps the doctor away."),
    (&["two things are infinite"], "quotation (attributed to Albert Einstein, unverified)", "Two things are infinite: the universe and human stupidity; and I'm not sure about the universe."),
    (&["early to bed"], "proverb (Benjamin Franklin, Poor Richard's Almanack)", "Early to bed and early to rise makes a man healthy, wealthy, and wise."),
    (&["the quick brown fox"], "pangram", "The quick brown fox jumps over the lazy dog. (It uses every letter of the English alphabet.)"),
];

fn chat(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = l.trim_start_matches("noesis, ").trim_start_matches("hey, ");
    if l.contains("astronomers") && l.contains("light bulb") {
        return Some(reply("joke", l, "None: astronomers prefer the dark."));
    }
    if let Some((_, kind, body)) = RHYMES.iter().find(|r| r.0.contains(&l)) {
        return Some(reply(kind, l, body));
    }
    let ans = match l {
        "hello" | "hi" | "hey" | "hello there" | "good morning" | "good afternoon" | "good evening" | "greetings" => Some(("greeting", "Hello! Ask me to compute something: an integral, a unit conversion, a chemical property, a date, a loan payment.")),
        "thank you" | "thanks" | "thanks a lot" => Some(("thanks", "You're welcome.")),
        "who are you" | "what are you" | "what is noesis" | "what is your name" => Some(("about", "I am Noesis, an open-source computational knowledge engine. I compute answers from built-in algorithms and data rather than searching the web.")),
        "how old is wolfram|alpha" | "how old is wolfram alpha" | "how old is wolframalpha" => {
            let days = (chrono::Local::now().date_naive() - chrono::NaiveDate::from_ymd_opt(2009, 5, 18)?).num_days();
            return Some(reply("age", "age of Wolfram|Alpha", &format!("Wolfram|Alpha launched publicly on May 18, 2009: {} years ({} days) ago.", n((days as f64 / 365.2425).floor()), days)));
        }
        "how do i win the lottery" => Some(("lottery", "Buy every possible combination, then hope nobody else picks the same numbers. For a 6-of-49 lottery that is 13,983,816 tickets. More practically: the expected value of a ticket is negative, so the winning move is not to play.")),
        "can you fail the turing test" => Some(("Turing test", "Only if I answer like a computer. I'd rather pass on that question.")),
        "how much wood could a woodchuck chuck if a woodchuck could chuck wood" | "how much wood would a woodchuck chuck if a woodchuck could chuck wood" | "how much wood could a woodchuck chuck" => Some(("woodchuck", "About 700 pounds, by a widely quoted estimate from the volume of soil a woodchuck digs out of a burrow; a woodchuck would chuck as much wood as a woodchuck could chuck if a woodchuck could chuck wood.")),
        "what is the square root of a carrot" | "square root of a carrot" => Some(("joke", "A square root is what grows when you plant carrots in a square garden.")),
        "how many astronomers does it take to change a light bulb" => Some(("joke", "None: astronomers prefer the dark.")),
        "possibility of successfully navigating an asteroid field" | "odds of successfully navigating an asteroid field" => Some(("asteroid field", "Approximately 3,720 to 1, according to a nervous protocol droid. Real asteroid belts are so sparse that spacecraft cross them without maneuvering: typical separations are hundreds of thousands of kilometers.")),
        "who's the cat that won't cop out when there's danger all about" | "who is the cat that won't cop out when there's danger all about" => Some(("Shaft", "John Shaft (from Isaac Hayes' \"Theme from Shaft\", 1971).")),
        "do you know a dirty joke" | "tell me a dirty joke" => Some(("joke", "A white horse fell in the mud.")),
        _ => None,
    };
    if let Some((t, body)) = ans {
        return Some(reply(t, l, body));
    }
    let m = re!(r"^(?:tell me |give me |say |know )?(?:an? |another |some )?(math(?:s|ematics)?|physics|chemistry|science|programming|computer)?\s*(?:joke|jokes|something funny(?: about (math(?:s|ematics)?|physics|chemistry))?)$").captures(l)?;
    let topic = m.get(1).or(m.get(2)).map(|x| x.as_str()).unwrap_or("");
    let list = if topic.starts_with("math") { MATH_JOKES } else if topic == "physics" { PHYSICS_JOKES } else if topic == "chemistry" { CHEM_JOKES } else { GENERAL_JOKES };
    Some(reply("joke", l, pick(l, list)))
}

// ---- warp factors ----

/// TNG warp scale beyond warp 9 (Star Trek: The Next Generation Technical
/// Manual), interpolated in log space; below 9 it follows w^(10/3).
const TNG: &[(f64, f64)] = &[(9.0, 1516.0), (9.2, 1649.0), (9.6, 1909.0), (9.9, 3053.0), (9.975, 5754.0), (9.99, 7912.0), (9.9999, 199_516.0)];

fn tng_speed(w: f64) -> Option<f64> {
    if w <= 9.0 {
        return Some(w.powf(10.0 / 3.0));
    }
    if w >= 10.0 {
        return None;
    }
    let i = TNG.iter().position(|p| p.0 >= w).unwrap_or(TNG.len() - 1).max(1);
    let (a, b) = (TNG[i - 1], TNG[i]);
    // interpolate log(speed) against −log(10 − w)
    let x = |w: f64| -(10.0 - w).ln();
    let t = (x(w) - x(a.0)) / (x(b.0) - x(a.0));
    Some((a.1.ln() + t * (b.1.ln() - a.1.ln())).exp())
}

fn tng_factor(c: f64) -> f64 {
    if c <= 1516.0 {
        return c.powf(0.3);
    }
    let (mut lo, mut hi) = (9.0, 9.999_999);
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        if tng_speed(mid).unwrap_or(f64::INFINITY) < c { lo = mid } else { hi = mid }
    }
    (lo + hi) / 2.0
}

fn warp(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let tos = l.contains("original series") || l.contains("tos");
    let tng = l.contains("next generation") || l.contains("tng");
    let from_c = re!(r"(?:what (?:warp factor|warp) is |^)([\d.]+)\s*c\b(?:\s+in)?\s*(?:warp(?: factor)?)?").captures(l).filter(|_| l.contains("warp"));
    if let Some(m) = from_c {
        let c: f64 = m[1].parse().ok()?;
        let wt = c.cbrt();
        let wn = tng_factor(c);
        let head = if tos { format!("warp {}", n(wt)) } else { format!("warp {}", n(wn)) };
        return Some(Answer::new("warp factor", vec![
            input(format!("{} times the speed of light as a warp factor{}", n(c), if tos { " (original series)" } else { " (The Next Generation)" })),
            primary(pod("Result", "Warp factor", text(head))),
            pod("Scales", "Both scales", table(None, vec![vec![text("The Original Series (v = w³ c)"), text(format!("warp {}", n(wt)))], vec![text("The Next Generation (v = w^(10/3) c up to warp 9)"), text(format!("warp {}", n(wn)))]])),
        ]));
    }
    let m = re!(r"^(?:what is )?warp(?: factor| speed)? ([\d.]+)(?: in .+)?$").captures(l)?;
    if !l.starts_with("warp") && !l.starts_with("what is warp") {
        return None;
    }
    let w: f64 = m[1].parse().ok()?;
    let tos_v = w.powi(3);
    let tng_v = tng_speed(w);
    let use_tos = tos && !tng;
    let v = if use_tos { tos_v } else { tng_v? };
    let ly_per_day = v / 365.25;
    let rows = vec![
        vec![text("speed (The Original Series, w³)"), text(format!("{} c", n(tos_v)))],
        vec![text("speed (The Next Generation)"), text(tng_v.map_or("unattainable (warp 10 is infinite speed)".into(), |s| format!("{} c", n(s))))],
        vec![text("distance per day"), text(format!("{} light years", n(ly_per_day)))],
        vec![text("time to Proxima Centauri (4.2465 ly)"), text(format!("{} days", n(4.2465 / ly_per_day)))],
        vec![text("time across the Milky Way (about 100,000 ly)"), text(format!("{} years", n(100_000.0 / v)))],
    ];
    Some(Answer::new("warp speed", vec![
        input(format!("warp factor {}{}", n(w), if use_tos { " (original series)" } else { " (The Next Generation)" })),
        primary(pod("Result", "Speed", text(format!("{} times the speed of light", n(v))))),
        pod("Details", "Details", table(None, rows)),
    ]))
}

fn warp_general(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    if l != "warp speed" && l != "warp factor" && l != "warp factors" {
        return None;
    }
    let rows: Vec<Vec<Content>> = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 9.6, 9.9, 9.99]
        .iter()
        .map(|w: &f64| vec![text(n(*w)), text(format!("{} c", n(w.powi(3)))), text(format!("{} c", n(tng_speed(*w).unwrap_or(f64::NAN))))])
        .collect();
    Some(Answer::new("warp speed", vec![
        input("warp speed scales (Star Trek)"),
        primary(pod("Result", "Warp factor and speed", table(Some(vec!["warp", "original series (w³)", "The Next Generation"]), rows))),
    ]))
}

// ---- strings and numbers ----

fn backwards(q: &Query) -> Option<Answer> {
    let raw = q.raw.trim().trim_end_matches(['?', '.']);
    let m = re!(r#"(?i)^(?:what is |how do you spell |spell )?"?(.+?)"? (?:spelled|spelt|written|reversed) backwards?$"#).captures(raw).or_else(|| re!(r#"(?i)^(?:reverse|backwards) (?:of )?"?(.+?)"?$"#).captures(raw))?;
    let s = &m[1];
    let r: String = s.chars().rev().collect();
    let pal = s.chars().filter(|c| c.is_alphanumeric()).map(|c| c.to_ascii_lowercase()).eq(s.chars().rev().filter(|c| c.is_alphanumeric()).map(|c| c.to_ascii_lowercase()));
    Some(Answer::new("reverse", vec![
        input(format!("reverse \"{s}\"")),
        primary(pod("Result", "Result", text(r))),
        pod("Palindrome", "Palindrome", text(if pal { "yes" } else { "no" })),
    ]))
}

fn digits_value(s: &str, base: u32) -> Option<u128> {
    let mut v: u128 = 0;
    for c in s.chars() {
        let d = c.to_digit(36)?;
        if d >= base {
            return None;
        }
        v = v.checked_mul(base as u128)?.checked_add(d as u128)?;
    }
    Some(v)
}

fn to_base(mut v: u128, base: u32) -> String {
    if v == 0 {
        return "0".into();
    }
    let mut out = Vec::new();
    while v > 0 {
        out.push(std::char::from_digit((v % base as u128) as u32, base).unwrap_or('?'));
        v /= base as u128;
    }
    out.iter().rev().collect()
}

fn word_bases(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^(?:convert )?([a-z0-9]*[a-z][a-z0-9]*)_(\d+) to base (\d+)$").captures(l).map(|c| (c[1].to_string(), c[2].parse::<u32>().ok(), c[3].parse::<u32>().ok()))
        .or_else(|| re!(r"^(?:convert )?([a-z0-9]*[a-z][a-z0-9]*) from base (\d+) to base (\d+)$").captures(l).map(|c| (c[1].to_string(), c[2].parse::<u32>().ok(), c[3].parse::<u32>().ok())))?;
    let (s, from, to) = (m.0, m.1?, m.2?);
    if !(2..=36).contains(&from) || !(2..=36).contains(&to) {
        return None;
    }
    let v = digits_value(&s, from)?;
    let out = to_base(v, to);
    Some(Answer::new("base conversion", vec![
        input(format!("{s} in base {from} to base {to}")),
        primary(pod("Result", &format!("Base {to}"), text(format!("{out}_{to}")))),
        pod("Decimal", "Decimal value", text(v.to_string())),
        pod("Digits", "Digit values", text(s.chars().map(|c| format!("{c} = {}", c.to_digit(36).unwrap_or(0))).collect::<Vec<_>>().join(", "))),
    ]))
}

fn look_and_say(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^(?:the )?look[- ]and[- ]say(?: sequence)?(?: (?:starting )?(?:from|with) (\d+))?(?:,? (\d+) terms)?$").captures(l)?;
    let mut cur: String = m.get(1).map_or("1".into(), |s| s.as_str().to_string());
    let k: usize = m.get(2).and_then(|s| s.as_str().parse().ok()).unwrap_or(10).min(20);
    let mut terms = vec![cur.clone()];
    for _ in 1..k {
        let b = cur.as_bytes();
        let mut next = String::new();
        let mut i = 0;
        while i < b.len() {
            let mut j = i;
            while j < b.len() && b[j] == b[i] {
                j += 1;
            }
            next.push_str(&(j - i).to_string());
            next.push(b[i] as char);
            i = j;
        }
        terms.push(next.clone());
        cur = next;
    }
    let lens: Vec<String> = terms.iter().map(|t| t.len().to_string()).collect();
    Some(Answer::new("look-and-say sequence", vec![
        input(format!("look-and-say sequence starting with {}", terms[0])),
        primary(pod("Result", "Terms", text(terms.join(", ")))),
        pod("Lengths", "Lengths of the terms", text(lens.join(", "))),
        pod("Growth", "Growth", text("each term describes the previous one; lengths grow by Conway's constant λ ≈ 1.303577269 per step (OEIS A005150)")),
    ]))
}

fn obfuscate(q: &Query) -> Option<Answer> {
    let l = q.lower.trim();
    let m = re!(r"^obfuscate (-?\d{1,12})$").captures(l)?;
    let v: i64 = m[1].parse().ok()?;
    let a = (v.abs() % 7) + 3;
    let b = v - a * a;
    let forms = vec![
        format!("{a}² {} {}", if b < 0 { "−" } else { "+" }, b.abs()),
        format!("−e^(iπ) {} {}", if v - 1 < 0 { "−" } else { "+" }, (v - 1).abs()),
        format!("∫₀^{} 1 dx", v),
        format!("Σ_(k=1)^{} 1", v),
        format!("log₂(2^{v})"),
        format!("{} − ⌊π⌋ + 3", v),
    ];
    Some(Answer::new("obfuscated number", vec![
        input(format!("obfuscate {v}")),
        primary(pod("Result", "Expression", text(forms[0].clone()))),
        pod("More", "More expressions equal to it", text(forms[1..].join("\n"))),
    ]))
}

fn pet_years(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']);
    let m = re!(r"^([\d.]+)[- ](?:years?|yrs?)[- ]old(?: human| person)? in (cat|dog) years$").captures(l)
        .or_else(|| re!(r"^([\d.]+) (?:human )?years? in (cat|dog) years$").captures(l))?;
    let h: f64 = m[1].parse().ok()?;
    let cat = &m[2] == "cat";
    // first year 15, second 9, then 4 (cats) or 5 (medium dogs) per year
    let (y1, y2, per) = if cat { (15.0, 9.0, 4.0) } else { (15.0, 9.0, 5.0) };
    let to_human = |a: f64| if a <= 1.0 { a * y1 } else if a <= 2.0 { y1 + (a - 1.0) * y2 } else { y1 + y2 + (a - 2.0) * per };
    let to_pet = |hh: f64| if hh <= y1 { hh / y1 } else if hh <= y1 + y2 { 1.0 + (hh - y1) / y2 } else { 2.0 + (hh - y1 - y2) / per };
    let animal = if cat { "cat" } else { "dog" };
    Some(Answer::new(&format!("{animal} years"), vec![
        input(format!("{} human years in {animal} years", n(h))),
        primary(pod("Result", &format!("Equivalent {animal} age"), text(format!("{} {animal} years", n(to_pet(h)))))),
        pod("Reverse", &format!("A {} year old {animal} in human years", n(h)), text(format!("{} human years", n(to_human(h))))),
        pod("Method", "Method", text(format!("first year = 15 human years, second = 9, then {} per year{}", n(per), if cat { "" } else { " (medium-sized dog)" }))),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    chat(q)
        .or_else(|| warp(q))
        .or_else(|| warp_general(q))
        .or_else(|| backwards(q))
        .or_else(|| word_bases(q))
        .or_else(|| look_and_say(q))
        .or_else(|| obfuscate(q))
        .or_else(|| pet_years(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warp_scales() {
        assert!((tng_speed(6.0).unwrap() - 392.0).abs() < 1.0);
        assert!((tng_speed(9.9).unwrap() - 3053.0).abs() < 1e-6);
        assert!((tng_factor(3053.0) - 9.9).abs() < 1e-6);
        assert_eq!(to_base(digits_value("cat", 36).unwrap(), 10), "15941");
    }
}
