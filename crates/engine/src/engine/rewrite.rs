//! Rewordings tried when no handler understands a question: number words,
//! "?" placeholders, arithmetic in words, suffix commands ("42 factors"),
//! command synonyms ("local maxima f"), "if x = 4, what is 3x + 50".
//!
//! Each candidate is run through the handlers again; the first one that is
//! understood is the answer, with an assumption saying how it was read.

use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

const SMALL: &[&str] = &["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen"];
const TENS: &[&str] = &["", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"];

fn small_value(w: &str) -> Option<u64> {
    if let Some(i) = SMALL.iter().position(|s| *s == w) {
        return Some(i as u64);
    }
    if let Some(i) = TENS.iter().position(|s| !s.is_empty() && *s == w) {
        return Some(10 * i as u64);
    }
    match w {
        "a" | "an" => None,
        "dozen" => Some(12),
        _ => None,
    }
}

/// Denominators: "third", "thirds", "quarter", "halves".
fn ordinal_denominator(w: &str) -> Option<u64> {
    let w = w.trim_end_matches('s');
    Some(match w {
        "half" | "halve" => 2,
        "third" => 3,
        "fourth" | "quarter" => 4,
        "fifth" => 5,
        "sixth" => 6,
        "seventh" => 7,
        "eighth" => 8,
        "ninth" => 9,
        "tenth" => 10,
        "eleventh" => 11,
        "twelfth" => 12,
        "thirteenth" => 13,
        "fourteenth" => 14,
        "fifteenth" => 15,
        "sixteenth" => 16,
        "seventeenth" => 17,
        "eighteenth" => 18,
        "nineteenth" => 19,
        "twentieth" => 20,
        "thirtieth" => 30,
        "fortieth" => 40,
        "fiftieth" => 50,
        "sixtieth" => 60,
        "sixty-fourth" | "sixty fourth" => 64,
        "hundredth" => 100,
        "thousandth" => 1000,
        _ => return None,
    })
}

/// Replace spelled-out numbers by digits: "four liters" → "4 liters",
/// "two thirds plus three fourths" → "2/3 plus 3/4",
/// "one hundred twenty-five" → "125".
pub fn number_words(s: &str) -> String {
    // hyphens only inside spelled numbers ("twenty-five"), never minus signs
    let s = re!(r"([A-Za-z])-([A-Za-z])").replace_all(s, "$1 $2").to_string();
    let words: Vec<String> = s.split_whitespace().map(|w| w.to_string()).collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let lw = words[i].to_lowercase();
        let core = lw.trim_end_matches(|c: char| !c.is_alphanumeric());
        let tail = &lw[core.len()..];
        if small_value(core).is_some() || (matches!(core, "hundred" | "thousand" | "million" | "billion") && i > 0 && out.last().is_some_and(|w| w.parse::<u64>().is_ok())) {
            // read a whole number phrase
            let mut total: u64 = 0;
            let mut current: u64 = 0;
            let mut j = i;
            let mut last_tail = String::new();
            let mut any = false;
            while j < words.len() {
                let w = words[j].to_lowercase();
                let c = w.trim_end_matches(|c: char| !c.is_alphanumeric()).to_string();
                let t = w[c.len()..].to_string();
                if let Some(v) = small_value(&c) {
                    current += v;
                } else if c == "hundred" && any {
                    current = current.max(1) * 100;
                } else if let Some(m) = match c.as_str() {
                    "thousand" => Some(1_000),
                    "million" => Some(1_000_000),
                    "billion" => Some(1_000_000_000),
                    _ => None,
                } {
                    if !any {
                        break;
                    }
                    total += current.max(1) * m;
                    current = 0;
                } else if c == "and" && any && j + 1 < words.len() && small_value(&words[j + 1].to_lowercase()).is_some() {
                    j += 1;
                    continue;
                } else {
                    break;
                }
                any = true;
                last_tail = t.clone();
                j += 1;
                if !t.is_empty() {
                    break;
                }
            }
            if !any {
                out.push(words[i].clone());
                i += 1;
                continue;
            }
            let n = total + current;
            // "two thirds", "three quarters"
            if last_tail.is_empty() && j < words.len() {
                let nw = words[j].to_lowercase();
                let c = nw.trim_end_matches(|c: char| !c.is_alphanumeric());
                if let Some(d) = ordinal_denominator(c) {
                    if !(n == 1 && c.ends_with('s')) {
                        out.push(format!("{n}/{d}{}", &nw[c.len()..]));
                        i = j + 1;
                        continue;
                    }
                }
            }
            out.push(format!("{n}{last_tail}"));
            i = j;
            continue;
        }
        // "a half", "a third", "a quarter"
        if core == "a" && i + 1 < words.len() {
            let nw = words[i + 1].to_lowercase();
            let c = nw.trim_end_matches(|c: char| !c.is_alphanumeric());
            if let Some(d) = ordinal_denominator(c).filter(|_| !c.ends_with('s')) {
                out.push(format!("1/{d}{}", &nw[c.len()..]));
                i += 2;
                continue;
            }
        }
        let _ = tail;
        out.push(words[i].clone());
        i += 1;
    }
    out.join(" ")
}

/// Arithmetic words to symbols.
fn arithmetic_words(s: &str) -> String {
    let mut t = format!(" {s} ");
    for (a, b) in [(" plus ", " + "), (" minus ", " - "), (" times ", " * "), (" multiplied by ", " * "), (" divided by ", " / "), (" over ", " / "), (" to the power of ", " ^ "), (" squared ", " ^2 "), (" cubed ", " ^3 "), (" equals ", " = "), (" is equal to ", " = ")] {
        t = t.replace(a, b);
    }
    t.trim().to_string()
}

fn push(out: &mut Vec<String>, s: String, orig: &str) {
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if !s.is_empty() && s != orig && !out.contains(&s) {
        out.push(s);
    }
}

/// Candidate rewordings of a cleaned question, best first.
/// School phrasings of algebra and arithmetic: "distribute 9 (y + 2)",
/// "isolate b in a^2 + b^2 = c^2", "39.276 rounded to the nearest tenth".
fn school_rules(l: &str, out: &mut Vec<String>, orig: &str, g: &dyn Fn(&regex::Captures, usize) -> String) {
    if let Some(m) = re!(r"^(?:combine like terms(?: in| of)?|collect like terms(?: in| of)?|reduce)\s+(.+)$").captures(l) {
        push(out, format!("simplify {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(?:distribute|multiply out|expanded form(?: of)?|expand out|foil|use the distributive property(?: on| for)?)\s+(.+)$").captures(l) {
        push(out, format!("expand {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^factor(?:ise|ize)?\s+(?:by grouping|completely|fully)\s+(.+)$").captures(l) {
        push(out, format!("factor {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(.+?)\s+(?:solution|solutions|solved)$").captures(l) {
        push(out, format!("solve {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(?:show (?:the )?solution(?:s)? (?:of|to)|find (?:the )?solutions? (?:of|to)|solution(?:s)? (?:of|to))\s+(.+)$").captures(l) {
        push(out, format!("solve {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(solve .+?)\s+(?:over|in) the (?:reals|real numbers)$").captures(l) {
        push(out, g(&m, 1), orig);
    }
    if let Some(m) = re!(r"^(?:isolate|solve for|make)\s+([a-z])\s+(?:in|from|the subject of)\s+(.+)$").captures(l) {
        push(out, format!("solve {} for {}", g(&m, 2), g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(?:what is|what's|find|solve for)\s+([a-z])\s+(?:if|when|given)\s+(.+)$").captures(l) {
        push(out, format!("solve {} for {}", g(&m, 2), g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^([a-z])\s+(?:if|when|given)\s+(.+=.+)$").captures(l) {
        push(out, format!("solve {} for {}", g(&m, 2), g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(?:where|when) (?:do|does)\s+(.+?)\s+and\s+(.+?)\s+(?:intersect|meet|cross)$").captures(l) {
        push(out, format!("solve {{{}, {}}}", g(&m, 1), g(&m, 2)), orig);
    }
    if let Some(m) = re!(r"^(?:intersection|intersection points?|points? of intersection) of\s+(.+?)\s+and\s+(.+)$").captures(l) {
        push(out, format!("solve {{{}, {}}}", g(&m, 1), g(&m, 2)), orig);
    }
    if let Some(m) = re!(r"^f\^\(-1\)\(x\)\s+(?:if|when|for|given)\s+f\(x\)\s*=\s*(.+)$").captures(l) {
        push(out, format!("inverse of {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(.+?)\s+inverse(?: function)?$").captures(l) {
        push(out, format!("inverse of {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(?:quotient and remainder|quotient|remainder|long division|polynomial long division|synthetic division)(?: of| for)?\s+(.+)$").captures(l) {
        push(out, format!("polynomial division {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(.+?)\s+(?:zeros|zeroes|roots|x-intercepts)$").captures(l) {
        push(out, format!("zeros of {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(.+?)\s+(?:rounded|round(?:ed)?)\s+(?:off\s+)?to the nearest (tenth|hundredth|thousandth|ten-thousandth|whole number|integer|one|unit|ten|hundred|thousand)s?$").captures(l) {
        let what = match &m[2] {
            "tenth" => "1 decimal place",
            "hundredth" => "2 decimal places",
            "thousandth" => "3 decimal places",
            "ten-thousandth" => "4 decimal places",
            "ten" => "the nearest 10",
            "hundred" => "the nearest 100",
            "thousand" => "the nearest 1000",
            _ => "0 decimal places",
        };
        push(out, format!("round {} to {what}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^what percent(?:age)? (?:is|of)\s+(.+?)\s+(?:out of|is of|of)\s+(.+)$").captures(l) {
        push(out, format!("({})/({}) as a percent", g(&m, 1), g(&m, 2)), orig);
    }
    if let Some(m) = re!(r"^(?:scientific notation(?: of| for)?|standard form(?: of)?)\s+(.+)$").captures(l) {
        push(out, format!("{} in scientific notation", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(.+?)\s+(?:in\s+)?(decimal|percent|fraction) form$").captures(l) {
        push(out, format!("{} as a {}", g(&m, 1), &m[2]), orig);
    }
    if let Some(m) = re!(r"^\$?([\d.,]+)\s+(?:with|at|after)\s+(?:a\s+)?([\d.]+)%\s+(?:markdown|discount|off|reduction)$").captures(l) {
        push(out, format!("{}% off {}", &m[2], &m[1]), orig);
    }
    if let Some(m) = re!(r"^(?:probability|chance|odds) of (?:rolling|getting|throwing) (?:a |an )?(\d+) (?:on|with) (two|2|three|3) (?:6-sided |six-sided |fair )?dice$").captures(l) {
        push(out, format!("probability of rolling {} with {} dice", &m[1], &m[2]), orig);
    }
    if let Some(m) = re!(r"^(?:linear function|line|straight line) (?:through|passing through|containing)\s+(.+)$").captures(l) {
        push(out, format!("line through {}", g(&m, 1)), orig);
    }
    // arithmetic steps in words
    if let Some(m) = re!(r"^add ([\d.]+) and ([\d.]+),? (?:and )?then (multiply|divide) (?:it |the result |that )?by ([\d.]+)$").captures(l) {
        let op = if &m[3] == "multiply" { "*" } else { "/" };
        push(out, format!("({} + {}) {op} {}", &m[1], &m[2], &m[4]), orig);
    }
    if let Some(m) = re!(r"^([\d.]+) (?:added to|plus) ([\d.]+)$").captures(l) {
        push(out, format!("{} + {}", &m[2], &m[1]), orig);
    }
    if let Some(m) = re!(r"^([\d.]+) (?:subtracted from|taken away from|less than) ([\d.]+)$").captures(l) {
        push(out, format!("{} - {}", &m[2], &m[1]), orig);
    }
    if let Some(m) = re!(r"^([\d.]+) (two|three|four|five|six|seven|eight|nine|ten|\d+) times$").captures(l) {
        let k = small_value(&m[2]).map(|v| v.to_string()).unwrap_or_else(|| m[2].to_string());
        push(out, format!("{} * {k}", &m[1]), orig);
    }
    if let Some(m) = re!(r"^what (times|divided by|plus|minus) ([\d.]+) (?:is|equals|makes) ([\d.]+)$").captures(l) {
        let op = match &m[1] { "times" => "*", "divided by" => "/", "plus" => "+", _ => "-" };
        push(out, format!("x {op} {} = {}", &m[2], &m[3]), orig);
    }
    if let Some(m) = re!(r"^(?:pythagorean theorem|pythagoras)\s+a\s*=\s*([\d.]+),?\s*(?:and\s+)?b\s*=\s*([\d.]+)$").captures(l) {
        push(out, format!("sqrt({}^2 + {}^2)", &m[1], &m[2]), orig);
    }
}

pub fn candidates(clean: &str) -> Vec<String> {
    let mut out = Vec::new();
    let c = clean.trim();
    let l = c.to_lowercase();
    // number words, then arithmetic words
    let nw = number_words(c);
    let aw = arithmetic_words(&nw);
    let mut bases = vec![c.to_string()];
    // typographic minus signs and dashes
    if c.contains(['–', '−', '—']) {
        let d = c.replace(['–', '−', '—'], "-");
        push(&mut out, d.clone(), c);
        bases.push(d);
    }
    if nw != c {
        bases.push(nw.clone());
    }
    if aw != nw {
        bases.push(aw.clone());
    }
    for b in &bases {
        if b != c {
            push(&mut out, b.clone(), c);
        }
    }
    for b in bases.iter().rev() {
        let bl = b.to_lowercase();
        rules(b, &bl, &mut out, c);
    }
    let _ = l;
    out
}

fn rules(c: &str, l: &str, out: &mut Vec<String>, orig: &str) {
    let g = |m: &regex::Captures, k: usize| -> String { m.get(k).map(|x| c[x.start()..x.end()].trim().to_string()).unwrap_or_default() };
    school_rules(l, out, orig, &g);
    // "?" or "what" as the unknown
    if let Some(m) = re!(r"^(?:solve\s+)?(.*?)\s*=\s*(?:\?|what)?\s*$").captures(l) {
        if !m[1].contains('=') && !m[1].trim().is_empty() && (l.ends_with('=') || l.ends_with('?') || l.ends_with("what")) {
            push(out, g(&m, 1), orig);
        }
    }
    if l.contains('?') || re!(r"\bwhat\b").is_match(l) {
        let t = re!(r"\?|\bwhat\b").replace_all(c, "x").to_string();
        let t = t.trim_start_matches("solve ").to_string();
        if t.contains('=') && t.matches('x').count() == 1 {
            push(out, format!("solve {t}"), orig);
        }
    }
    // arithmetic in words
    if let Some(m) = re!(r"^subtract\s+(.+?)\s+from\s+(.+)$").captures(l) {
        push(out, format!("{} - ({})", g(&m, 2), g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(?:add|sum)\s+(.+?)\s+(?:and|to|with|plus)\s+(.+)$").captures(l) {
        push(out, format!("{} + {}", g(&m, 1), g(&m, 2)), orig);
    }
    if let Some(m) = re!(r"^multiply\s+(.+?)\s+(?:and|by|with|times)\s+(.+)$").captures(l) {
        push(out, format!("({}) * ({})", g(&m, 1), g(&m, 2)), orig);
    }
    if let Some(m) = re!(r"^divide\s+(.+?)\s+(?:by|into)\s+(.+)$").captures(l) {
        let (a, b) = (g(&m, 1), g(&m, 2));
        push(out, if l.contains(" into ") { format!("({b}) / ({a})") } else { format!("({a}) / ({b})") }, orig);
    }
    if let Some(m) = re!(r"^(?:the\s+)?(sum|product|difference|quotient|ratio)\s+(?:of|between)\s+(.+?)\s+and\s+(.+)$").captures(l) {
        let op = match &m[1] {
            "sum" => "+",
            "product" => "*",
            "difference" => "-",
            _ => "/",
        };
        push(out, format!("({}) {op} ({})", g(&m, 2), g(&m, 3)), orig);
    }
    // "1.5 teaspoons tripled", "a recipe doubled"
    if let Some(m) = re!(r"^(.+?)\s+(doubled|tripled|quadrupled|halved)$").captures(l) {
        let k = match &m[2] {
            "doubled" => "2",
            "tripled" => "3",
            "quadrupled" => "4",
            _ => "1/2",
        };
        let body = g(&m, 1);
        if let Some(n) = re!(r"^([\d.]+(?:/\d+)?)\s*(.*)$").captures(&body) {
            if let (Ok(a), Some(f)) = (n[1].parse::<f64>(), match k {
                "2" => Some(2.0),
                "3" => Some(3.0),
                "4" => Some(4.0),
                _ => Some(0.5),
            }) {
                let v = crate::print::fmt_float(a * f, 10);
                if !n[2].trim().is_empty() {
                    // keep the recipe's unit: "4.5 teaspoons in teaspoons"
                    push(out, format!("{v} {} in {}", n[2].trim(), n[2].trim()), orig);
                }
                push(out, format!("{v} {}", &n[2]), orig);
            }
            push(out, format!("{k} * {} {}", &n[1], &n[2]), orig);
        }
    }
    // "if x = 4, what is 3x + 50"
    if let Some(m) = re!(r"^(?:if|when|given|for)\s+([a-z])\s*=\s*([^,]+?)\s*,?\s*(?:then\s+)?(?:what is|what's|find|evaluate|compute)?\s*(.+)$").captures(l) {
        let (v, val, e) = (g(&m, 1), g(&m, 2), g(&m, 3));
        let r = Regex::new(&format!(r"\b{v}\b|(\d){v}\b")).unwrap();
        let sub = r.replace_all(&e, |cap: &regex::Captures| match cap.get(1) {
            Some(d) => format!("{}*({val})", d.as_str()),
            None => format!("({val})"),
        });
        push(out, sub.to_string(), orig);
    }
    // command synonyms and word order
    if let Some(m) = re!(r"^(?:find\s+)?(?:the\s+)?(?:local|global|absolute|relative)?\s*(?:extrema|extremum|maxima|minima|maximum|minimum|max|min|stationary points?|critical points?|turning points?)\s+(?:points?\s+)?(?:of|for)?\s*(.+)$").captures(l) {
        push(out, format!("extrema of {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(.+?)\s+(factors|divisors|prime factors|factorization|prime factorization)$").captures(l) {
        let what = if m[2].starts_with("prime") || m[2] == *"factorization" { "factor" } else { "divisors of" };
        push(out, format!("{what} {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(continued fraction|partial fractions?|taylor series|series|derivative|integral|antiderivative|inverse|domain|range|roots|zeros|asymptotes|period)\s+(?:for\s+)?(.+)$").captures(l) {
        if !m[2].starts_with("of ") {
            push(out, format!("{} of {}", &m[1], g(&m, 2)), orig);
        }
    }
    if let Some(m) = re!(r"^(.+?)\s+(?:in partial fractions|partial fractions|partial fraction decomposition)$").captures(l) {
        push(out, format!("partial fractions {}", g(&m, 1)), orig);
    }
    if let Some(m) = re!(r"^(.+?)\s+(?:measured|expressed|converted|written|given)\s+(?:in|to|as)\s+(.+)$").captures(l) {
        push(out, format!("{} in {}", g(&m, 1), g(&m, 2)), orig);
    }
    if let Some(m) = re!(r"^(?:convert|change|turn)\s+(.+?)\s+(?:in|to|into)\s+(.+)$").captures(l) {
        push(out, format!("{} in {}", g(&m, 1), g(&m, 2)), orig);
    }
    // trailing noise
    if let Some(m) = re!(r"^(.+?)\s+(?:calculator|calculation|solver|value|values|number|result)$").captures(l) {
        push(out, g(&m, 1), orig);
    }
    if let Some(m) = re!(r"^(?:how (?:do (?:i|you)|to)|can you|please)\s+(.+)$").captures(l) {
        push(out, g(&m, 1), orig);
    }
    // "fructose molar mass" → "molar mass of fructose"
    let words: Vec<&str> = c.split_whitespace().collect();
    // (a list such as "traffic Los Angeles, Phoenix" is not a property phrase)
    if words.len() >= 2 && !l.contains(" of ") && !c.contains(['=', '+', '*', '^', '"']) {
        for k in 1..=3.min(words.len() - 1) {
            if words[words.len() - k..].iter().any(|w| w.contains(',')) || words[words.len() - k - 1].ends_with(',') && k < 2 {
                continue;
            }
            push(out, format!("{} of {}", words[words.len() - k..].join(" "), words[..words.len() - k].join(" ")), orig);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_words_become_digits() {
        assert_eq!(number_words("how many cups are in four liters"), "how many cups are in 4 liters");
        assert_eq!(number_words("two thirds plus three fourths"), "2/3 plus 3/4");
        assert_eq!(number_words("one hundred twenty-five"), "125");
        assert_eq!(number_words("three plus what equals ten?"), "3 plus what equals 10?");
        assert_eq!(number_words("a quarter of 80"), "1/4 of 80");
        assert_eq!(number_words("twenty one thousand"), "21000");
    }

    #[test]
    fn rewordings() {
        assert!(candidates("subtract 6 from 17").contains(&"17 - (6)".to_string()));
        assert!(candidates("solve 4 + ? = 10").contains(&"solve 4 + x = 10".to_string()));
        assert!(candidates("three plus what equals ten").contains(&"solve 3 + x = 10".to_string()));
        assert!(candidates("local maxima x^5 - 10x^3 + 30x").contains(&"extrema of x^5 - 10x^3 + 30x".to_string()));
        assert!(candidates("if x = 4, what is 3x + 50").contains(&"3*(4) + 50".to_string()));
        assert!(candidates("42 factors").contains(&"divisors of 42".to_string()));
    }
}
