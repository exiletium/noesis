//! Games of chance and simple games: dice with any number of sides, card
//! hands ("probability 3 queens 2 jacks", "at least 3 eights in 10 cards"),
//! odds and house edge, roulette stakes, lotteries (Powerball, Mega Millions,
//! EuroMillions, 6/49), classic 2×2 and 3×3 games with their Nash
//! equilibria, and two dynamical systems (the logistic map, Van der Pol).

use super::*;
use crate::numtheory as nt;
use crate::stats::fmt;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn word_num(s: &str) -> Option<u64> {
    const W: &[&str] = &["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen"];
    match s {
        "a" | "an" | "single" => Some(1),
        "a pair of" | "pair of" | "both" => Some(2),
        _ => s.parse().ok().or_else(|| W.iter().position(|w| *w == s).map(|i| i as u64)),
    }
}

fn kv(rows: Vec<(&str, String)>) -> Content {
    table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())
}

fn input(s: String) -> Pod {
    pod("Input", "Input interpretation", text(s))
}

fn rat(n: &BigInt, d: &BigInt) -> String {
    let r = BigRational::new(n.clone(), d.clone());
    if r.is_zero() {
        "0".into()
    } else {
        format!("{} ≈ {}", r, pct(r.to_f64().unwrap_or(0.0)))
    }
}

fn pct(p: f64) -> String {
    if p >= 0.001 {
        format!("{}%", fmt((p * 100000.0).round() / 1000.0))
    } else {
        format!("1 in {}", fmt((1.0 / p).round()))
    }
}

// ---------------------------------------------------------------------------
// dice
// ---------------------------------------------------------------------------

fn dice_dist(n: usize, sides: usize) -> Vec<BigInt> {
    let mut dist = vec![BigInt::one()];
    for _ in 0..n {
        let mut next = vec![BigInt::zero(); dist.len() + sides];
        for (s0, w) in dist.iter().enumerate() {
            if w.is_zero() {
                continue;
            }
            for face in 1..=sides {
                next[s0 + face] += w;
            }
        }
        dist = next;
    }
    dist
}

fn dice(l: &str) -> Option<Answer> {
    let num = r"(\d+|a|an|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|a pair of|pair of)";
    let pat = format!(r"^(?:(?:the\s+)?probability\s+(?:of\s+|that\s+)?(?:a\s+|the\s+)?)?(?:(?:roll(?:ing)?|throw(?:ing)?|sum of|total of)\s+)?{num}\s+(?:fair\s+)?(?:(\d+)[- ]sided\s+|d(\d+)\s+)?(?:dice|die)(?:\s+(?:sum|sums|total|totals|distribution))?(?:\s+(?:totals?\s+|sums?\s+|adds? up to\s+|is\s+)?(?:(between)\s+(\d+)\s+and\s+(\d+)|(at least|at most|more than|less than|exactly|over|under|>=|<=|>|<|=)\s*(\d+)))?$");
    let r = {
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new(&pat).unwrap())
    };
    let conv;
    let l = match re!(r"^(\d+)d(\d+)\b(.*)$").captures(l) {
        Some(m2) => {
            conv = format!("{} {}-sided dice{}", &m2[1], &m2[2], &m2[3]);
            conv.as_str()
        }
        None => l,
    };
    let m = r.captures(l)?;
    let n = word_num(&m[1])? as usize;
    let sides: usize = m.get(2).or(m.get(3)).map_or(Some(6), |x| x.as_str().parse().ok())?;
    if !(1..=60).contains(&n) || !(2..=120).contains(&sides) || n * sides > 3000 || (n == 1 && m.get(4).is_none() && m.get(7).is_none() && sides == 6) {
        return None;
    }
    // the plain six-sided case without a condition is answered by the general analyzer
    if sides == 6 && m.get(2).is_none() && m.get(3).is_none() && m.get(4).is_none() && m.get(7).is_none() {
        return None;
    }
    let dist = dice_dist(n, sides);
    let total = BigInt::from(sides).pow(n as u32);
    let mean = n as f64 * (sides as f64 + 1.0) / 2.0;
    let var = n as f64 * ((sides * sides) as f64 - 1.0) / 12.0;
    let label = format!("{n} fair {sides}-sided {}", if n == 1 { "die" } else { "dice" });
    let stats = pod("Stats", "Expected value and spread", kv(vec![("mean", fmt(mean)), ("standard deviation", fmt(var.sqrt())), ("outcomes", total.to_string())]));
    let cond: Option<(usize, usize, String)> = if m.get(4).is_some() {
        let (a, b): (usize, usize) = (m[5].parse().ok()?, m[6].parse().ok()?);
        Some((a.min(b), a.max(b), format!("between {} and {}", a.min(b), a.max(b))))
    } else if let Some(op) = m.get(7) {
        let k: usize = m[8].parse().ok()?;
        let hi = n * sides;
        let (a, b) = match op.as_str() {
            "at least" | ">=" => (k, hi),
            "at most" | "<=" => (0, k),
            "more than" | "over" | ">" => (k + 1, hi),
            "less than" | "under" | "<" => (0, k.saturating_sub(1)),
            _ => (k, k),
        };
        Some((a, b, format!("{} {k}", op.as_str())))
    } else {
        None
    };
    if let Some((a, b, desc)) = cond {
        let ways: BigInt = dist.iter().enumerate().filter(|(s, _)| *s >= a && *s <= b).map(|(_, w)| w.clone()).sum();
        return Some(Answer::new("dice", vec![input(format!("probability that the total of {label} is {desc}")), primary(pod("Result", "Probability", text(rat(&ways, &total)))), pod("Ways", "Favourable outcomes", text(format!("{ways} of {total}"))), stats]));
    }
    let rows: Vec<Vec<Content>> = dist.iter().enumerate().filter(|(_, w)| !w.is_zero()).map(|(s, w)| vec![text(s.to_string()), text(w.to_string()), text(BigRational::new(w.clone(), total.clone()).to_string()), text(fmt(BigRational::new(w.clone(), total.clone()).to_f64().unwrap_or(0.0)))]).collect();
    let shown = if rows.len() > 80 { rows.into_iter().step_by(1).take(80).collect() } else { rows };
    Some(Answer::new("dice", vec![input(format!("total of {label}")), primary(pod("Distribution", "Distribution of the total", table(Some(vec!["total", "ways", "probability", "decimal"]), shown))), stats]))
}

// ---------------------------------------------------------------------------
// cards
// ---------------------------------------------------------------------------

fn card_kind(w: &str) -> Option<(&'static str, u64)> {
    let w = w.trim_end_matches("es").trim_end_matches('s');
    Some(match w {
        "ace" => ("aces", 4),
        "king" => ("kings", 4),
        "queen" => ("queens", 4),
        "jack" | "knave" => ("jacks", 4),
        "ten" => ("tens", 4),
        "nine" => ("nines", 4),
        "eight" => ("eights", 4),
        "seven" => ("sevens", 4),
        "six" | "sixe" => ("sixes", 4),
        "five" => ("fives", 4),
        "four" => ("fours", 4),
        "three" => ("threes", 4),
        "two" | "deuce" => ("twos", 4),
        "heart" => ("hearts", 13),
        "spade" => ("spades", 13),
        "diamond" => ("diamonds", 13),
        "club" => ("clubs", 13),
        "face card" | "court card" => ("face cards", 12),
        "red card" => ("red cards", 26),
        "black card" => ("black cards", 26),
        _ => return None,
    })
}

fn cards(l: &str) -> Option<Answer> {
    let body = re!(r"^(?:the\s+)?(?:probability|chance|odds)\s+(?:of\s+)?(?:getting\s+|drawing\s+|being dealt\s+)?").replace(l, "").to_string();
    let rank = r"(aces?|kings?|queens?|jacks?|knaves?|tens?|nines?|eights?|sevens?|six(?:es)?|fives?|fours?|threes?|twos?|deuces?|hearts?|spades?|diamonds?|clubs?|face cards?|court cards?|red cards?|black cards?)";
    let qty = r"(?:(at least|at most|exactly|more than|fewer than|less than)\s+)?(\d+|an?|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen)";
    let item = format!(r"{qty}\s+{rank}");
    let full = format!(r"^{item}(?:\s*,?\s*(?:and\s+)?{item})*(?:\s+(?:in|from|with)\s+(?:a\s+)?(\d+|five|six|seven|ten|thirteen)[- ]card(?:s)?(?:\s+hand)?|\s+in\s+(\d+)\s+cards)?$");
    let full_re = {
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new(&full).unwrap())
    };
    let fm = full_re.captures(&body)?;
    let item_re = {
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new(&item).unwrap())
    };
    // strip the hand-size clause before scanning items
    let hand_txt = fm.get(fm.len() - 2).or(fm.get(fm.len() - 1)).map(|x| x.as_str().to_string());
    let scan = re!(r"\s+(?:in|from|with)\s+.*$").replace(&body, "").to_string();
    let mut items: Vec<(&str, u64, &'static str, u64)> = Vec::new();
    for c in item_re.captures_iter(&scan) {
        let k = word_num(&c[2])?;
        let (name, size) = card_kind(&c[3])?;
        let op = c.get(1).map_or("exactly", |x| x.as_str());
        items.push((op, k, name, size));
    }
    if items.is_empty() {
        return None;
    }
    let hand: u64 = match hand_txt {
        Some(h) => word_num(&h)?,
        None => items.iter().map(|i| i.1).sum(),
    };
    if hand == 0 || hand > 52 {
        return None;
    }
    // one kind of category at a time (ranks, or suits), each named once
    let total_cat: u64 = items.iter().map(|i| i.3).sum();
    let names: std::collections::BTreeSet<&str> = items.iter().map(|i| i.2).collect();
    if total_cat > 52 || names.len() != items.len() || items.iter().any(|i| i.3 != items[0].3) {
        return None;
    }
    let deck = BigInt::from(52);
    let all = nt::binomial(&deck, hand);
    // enumerate counts per item satisfying the conditions
    let ranges: Vec<Vec<u64>> = items.iter().map(|&(op, k, _, size)| {
        (0..=size.min(hand)).filter(|&j| match op {
            "at least" => j >= k,
            "at most" => j <= k,
            "more than" => j > k,
            "fewer than" | "less than" => j < k,
            _ => j == k,
        }).collect()
    }).collect();
    let rest = 52 - total_cat;
    let mut fav = BigInt::zero();
    let mut idx = vec![0usize; ranges.len()];
    if ranges.iter().any(|r| r.is_empty()) {
        return None;
    }
    loop {
        let js: Vec<u64> = idx.iter().zip(&ranges).map(|(i, r)| r[*i]).collect();
        let used: u64 = js.iter().sum();
        if used <= hand && hand - used <= rest {
            let mut w = nt::binomial(&BigInt::from(rest), hand - used);
            for (j, it) in js.iter().zip(&items) {
                w *= nt::binomial(&BigInt::from(it.3), *j);
            }
            fav += w;
        }
        // next combination
        let mut p = 0;
        loop {
            if p == idx.len() {
                let desc: Vec<String> = items.iter().map(|(op, k, name, _)| format!("{}{k} {name}", if *op == "exactly" { String::new() } else { format!("{op} ") })).collect();
                return Some(Answer::new("cards", vec![input(format!("probability of {} in a {hand}-card hand from a standard 52-card deck", desc.join(" and "))), primary(pod("Result", "Probability", text(rat(&fav, &all)))), pod("Ways", "Favourable hands", text(format!("{fav} of {all} = C(52, {hand})"))), pod("Method", "Method", text("hypergeometric counting: choose the required cards from each category and the rest from the other cards"))]));
            }
            idx[p] += 1;
            if idx[p] < ranges[p].len() {
                break;
            }
            idx[p] = 0;
            p += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// odds, roulette, lotteries
// ---------------------------------------------------------------------------

fn odds(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:wager|bet|stake)\s+(?:\$|€|£)?(\d+(?:\.\d+)?)\s*(?:dollars?|euros?|pounds?)?\s+at\s+(\d+)\s*(?:to|:)\s*(\d+)(?:\s+odds)?(?:\s+with\s+(?:an?\s+)?(\d+(?:\.\d+)?)\s*%\s+(?:house (?:advantage|edge)|vig|vigorish))?$").captures(l) {
        let (s, a, b): (f64, f64, f64) = (m[1].parse().ok()?, m[2].parse().ok()?, m[3].parse().ok()?);
        let h: f64 = m.get(4).map_or(Some(0.0), |x| x.as_str().parse().ok())? / 100.0;
        let o = a / b;
        let p = (1.0 - h) / (1.0 + o);
        let rows = vec![("profit if the bet wins", fmt(s * o)), ("total returned on a win", fmt(s * (1.0 + o))), ("break-even win probability", format!("{} ≈ {}", fmt(1.0 / (1.0 + o)), pct(1.0 / (1.0 + o)))), ("win probability implied by the house edge", format!("{} ≈ {}", fmt(p), pct(p))), ("expected value of the bet", fmt(s * (p * o - (1.0 - p))))];
        return Some(Answer::new("odds", vec![input(format!("stake {} at {}:{} odds{}", fmt(s), fmt(a), fmt(b), if h > 0.0 { format!(" with a {}% house edge", fmt(h * 100.0)) } else { String::new() })), primary(pod("Result", "Payout", kv(rows)))]));
    }
    let m = re!(r"^(\d+(?:\.\d+)?)\s*(?::|to|/)\s*(\d+(?:\.\d+)?)\s+(?:odds|odds against)$|^odds\s+(?:of\s+)?(\d+(?:\.\d+)?)\s*(?::|to|/)\s*(\d+(?:\.\d+)?)$").captures(l)?;
    let a: f64 = m.get(1).or(m.get(3))?.as_str().parse().ok()?;
    let b: f64 = m.get(2).or(m.get(4))?.as_str().parse().ok()?;
    if a <= 0.0 || b <= 0.0 {
        return None;
    }
    let rows = vec![("implied probability of winning", format!("{}/{} ≈ {}", fmt(b), fmt(a + b), pct(b / (a + b)))), ("implied probability of losing", format!("{}/{} ≈ {}", fmt(a), fmt(a + b), pct(a / (a + b)))), ("profit on a 1-unit stake", fmt(a / b)), ("decimal odds", fmt(1.0 + a / b)), ("American odds", if a >= b { format!("+{}", fmt(100.0 * a / b)) } else { format!("−{}", fmt(100.0 * b / a)) })];
    Some(Answer::new("odds", vec![input(format!("{}:{} odds against", fmt(a), fmt(b))), primary(pod("Result", "Odds", kv(rows)))]))
}

fn roulette(l: &str) -> Option<Answer> {
    if l == "roulette" || l == "roulette bets" || l == "roulette odds" {
        let bets: &[(&str, u32, u32)] = &[("straight (1 number)", 1, 35), ("split (2)", 2, 17), ("street (3)", 3, 11), ("corner (4)", 4, 8), ("six line (6)", 6, 5), ("dozen or column (12)", 12, 2), ("red/black, odd/even, high/low (18)", 18, 1)];
        let rows: Vec<Vec<Content>> = bets.iter().map(|(n, k, pay)| vec![text(*n), text(format!("{pay}:1")), text(format!("{k}/37 ≈ {}", pct(*k as f64 / 37.0))), text(format!("{k}/38 ≈ {}", pct(*k as f64 / 38.0)))]).collect();
        return Some(Answer::new("roulette", vec![input("roulette".into()), primary(pod("Result", "Bets", table(Some(vec!["bet", "payout", "win probability (European)", "win probability (American)"]), rows))), pod("Edge", "House edge", kv(vec![("European wheel (single zero)", "1/37 ≈ 2.70%".into()), ("American wheel (0 and 00)", "2/38 ≈ 5.26% (7.89% on the five-number bet)".into())]))]));
    }
    let m = re!(r"^(?:american |european )?roulette,?\s+(?:bet|wager|stake)\s+(?:\$|€|£)?(\d+(?:\.\d+)?)\s*(?:dollars?|euros?)?\s+on\s+(\d+|red|black|odd|even|high|low|(?:first|second|third|1st|2nd|3rd) (?:dozen|column))$|^(?:bet|wager)\s+(?:\$|€|£)?(\d+(?:\.\d+)?)\s+on\s+(\d+|red|black|odd|even)\s+(?:at|in)\s+(?:american |european )?roulette$").captures(l)?;
    let stake: f64 = m.get(1).or(m.get(3))?.as_str().parse().ok()?;
    let on = m.get(2).or(m.get(4))?.as_str();
    let (k, pay) = if on.parse::<u32>().map_or(false, |n| n <= 36) { (1.0, 35.0) } else if on.contains("dozen") || on.contains("column") { (12.0, 2.0) } else if on.parse::<u32>().is_ok() { return None } else { (18.0, 1.0) };
    let american = l.contains("american");
    let pockets = if l.contains("european") { 37.0 } else { 38.0 };
    let ev = stake * (k * pay - (pockets - k)) / pockets;
    let rows = vec![("win probability", format!("{}/{} ≈ {}", fmt(k), fmt(pockets), pct(k / pockets))), ("profit if it wins", fmt(stake * pay)), ("total returned on a win", fmt(stake * (pay + 1.0))), ("expected value", fmt((ev * 100.0).round() / 100.0))];
    let _ = american;
    Some(Answer::new("roulette", vec![input(format!("roulette, stake {} on {on} ({} wheel)", fmt(stake), if pockets == 38.0 { "American" } else { "European" })), primary(pod("Result", "Result", kv(rows)))]))
}

fn lottery(l: &str) -> Option<Answer> {
    let l = l.trim_start_matches("odds of winning ").trim_start_matches("the ").trim_end_matches(" lottery").trim_end_matches(" odds").trim_end_matches(" jackpot");
    let (name, white, pick, extra, extra_pick) = match l {
        "powerball" => ("Powerball", 69u64, 5u64, 26u64, 1u64),
        "mega millions" => ("Mega Millions", 70, 5, 24, 1),
        "euromillions" | "euro millions" => ("EuroMillions", 50, 5, 12, 2),
        "lottery" | "lotto" | "6/49" | "lotto 6/49" | "uk lotto" => ("6/49 lotto", 49, 6, 1, 0),
        _ => {
            let m = re!(r"^(?:lotto|lottery)?\s*(?:pick\s+)?(\d+)\s*(?:/|of|from)\s*(\d+)(?:\s+lott(?:o|ery))?$").captures(l)?;
            let (k, n): (u64, u64) = (m[1].parse().ok()?, m[2].parse().ok()?);
            if k == 0 || k > n || n > 200 {
                return None;
            }
            ("lotto", n, k, 1, 0)
        }
    };
    let ways = nt::binomial(&BigInt::from(white), pick) * nt::binomial(&BigInt::from(extra), extra_pick);
    let mut rows: Vec<Vec<Content>> = Vec::new();
    for w in (0..=pick).rev() {
        for e in (0..=extra_pick).rev() {
            let c = nt::binomial(&BigInt::from(pick), w) * nt::binomial(&BigInt::from(white - pick), pick - w) * nt::binomial(&BigInt::from(extra_pick), e) * nt::binomial(&BigInt::from(extra - extra_pick), extra_pick - e);
            if c.is_zero() || (w + e) < 2 && !(w == 0 && e == extra_pick && extra_pick > 0) {
                continue;
            }
            let p = BigRational::new(c.clone(), ways.clone());
            let label = if extra_pick == 0 { format!("{w} numbers") } else { format!("{w} + {e}") };
            rows.push(vec![text(label), text(format!("1 in {}", fmt((1.0 / p.to_f64().unwrap_or(1.0) * 100.0).round() / 100.0)))]);
        }
    }
    let desc = if extra_pick == 0 { format!("choose {pick} of {white}") } else { format!("choose {pick} of {white} and {extra_pick} of {extra}") };
    Some(Answer::new("lottery", vec![input(format!("{name}: {desc}")), primary(pod("Result", "Odds of the jackpot", text(format!("1 in {ways}")))), pod("Tiers", "Odds of each match", table(Some(vec!["match", "odds"]), rows))]))
}

// ---------------------------------------------------------------------------
// game theory
// ---------------------------------------------------------------------------

fn games(l: &str) -> Option<Answer> {
    let l = l.trim_end_matches(" game").trim_end_matches(" payoff matrix").trim_start_matches("the ").trim_start_matches("game of ");
    let (name, moves, pay, ne, note): (&str, &[&str], &[&[(i32, i32)]], &str, &str) = match l {
        "chicken" | "hawk-dove" | "hawk dove" => ("game of chicken", &["swerve", "straight"], &[&[(0, 0), (-1, 1)], &[(1, -1), (-10, -10)]], "two pure equilibria (one swerves, the other goes straight); a mixed equilibrium in which each goes straight with probability 1/10", "anti-coordination game"),
        "prisoner's dilemma" | "prisoners dilemma" | "prisoner dilemma" => ("prisoner's dilemma", &["cooperate", "defect"], &[&[(-1, -1), (-3, 0)], &[(0, -3), (-2, -2)]], "(defect, defect) is the unique equilibrium, in dominant strategies, though both prefer (cooperate, cooperate)", "payoffs are minus years in prison"),
        "rock-paper-scissors" | "rock paper scissors" | "rock-scissors-paper" | "rock scissors paper" | "roshambo" => ("rock–paper–scissors", &["rock", "paper", "scissors"], &[&[(0, 0), (-1, 1), (1, -1)], &[(1, -1), (0, 0), (-1, 1)], &[(-1, 1), (1, -1), (0, 0)]], "unique mixed equilibrium: each plays every move with probability 1/3; value 0", "zero-sum and symmetric; no pure equilibrium"),
        "matching pennies" => ("matching pennies", &["heads", "tails"], &[&[(1, -1), (-1, 1)], &[(-1, 1), (1, -1)]], "unique mixed equilibrium: each plays heads with probability 1/2; value 0", "zero-sum"),
        "stag hunt" => ("stag hunt", &["stag", "hare"], &[&[(4, 4), (0, 3)], &[(3, 0), (3, 3)]], "pure equilibria (stag, stag), payoff-dominant, and (hare, hare), risk-dominant; a mixed one with stag played with probability 3/4", "coordination game"),
        "battle of the sexes" => ("battle of the sexes", &["opera", "football"], &[&[(3, 2), (0, 0)], &[(0, 0), (2, 3)]], "pure equilibria (opera, opera) and (football, football); a mixed one in which each plays their preferred event with probability 3/5", "coordination game with conflicting preferences"),
        "tic-tac-toe" | "tic tac toe" | "noughts and crosses" => {
            let rows = vec![("board", "3 × 3".to_string()), ("possible games", "255,168 (131,184 won by X, 77,904 by O, 46,080 drawn)".into()), ("reachable positions", "5,478 (765 up to symmetry)".into()), ("terminal positions", "958 (138 up to symmetry)".into()), ("value with perfect play", "draw".into()), ("game-tree complexity", "about 2.6 × 10⁵".into())];
            return Some(Answer::new("game", vec![input("tic-tac-toe".into()), primary(pod("Result", "Properties", kv(rows)))]));
        }
        _ => return None,
    };
    let mut header = vec![""];
    header.extend(moves.iter().copied());
    let rows: Vec<Vec<Content>> = moves.iter().zip(pay.iter()).map(|(mv, r)| {
        let mut row = vec![text(*mv)];
        row.extend(r.iter().map(|(a, b)| text(format!("{a}, {b}"))));
        row
    }).collect();
    Some(Answer::new("game", vec![input(format!("{name} (row player's payoff, column player's payoff)")), primary(pod("Result", "Payoff matrix", table(Some(header), rows))), pod("Nash", "Nash equilibria", text(ne)), pod("Type", "Type", text(note))]))
}

// ---------------------------------------------------------------------------
// dynamical systems
// ---------------------------------------------------------------------------

fn logistic(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?logistic map(?:\s*,?\s*(?:with\s+)?(?:r|μ|mu|a|lambda)\s*=\s*([\d.]+))?(?:\s*,?\s*x0?\s*=\s*([\d.]+))?$").captures(l)?;
    let r: f64 = m.get(1).map_or(Some(3.2), |x| x.as_str().parse().ok())?;
    let x0: f64 = m.get(2).map_or(Some(0.2), |x| x.as_str().parse().ok())?;
    if !(0.0..=4.0).contains(&r) || !(0.0..=1.0).contains(&x0) {
        return None;
    }
    let f = |x: f64| r * x * (1.0 - x);
    // transient, then Lyapunov exponent and period detection
    let mut x = 0.3141592653;
    for _ in 0..100_000 {
        x = f(x);
    }
    let mut lyap = 0.0;
    let n = 200_000;
    let mut orbit = Vec::new();
    for i in 0..n {
        lyap += (r * (1.0 - 2.0 * x)).abs().max(1e-300).ln();
        x = f(x);
        if i < 4096 {
            orbit.push(x);
        }
    }
    lyap /= n as f64;
    let period = (1..=64).find(|&p| (0..256).all(|i| (orbit[i] - orbit[i + p]).abs() < 1e-7));
    let behaviour = if r == 4.0 {
        "fully chaotic on [0, 1]; conjugate to the tent map; x_n = sin²(2ⁿ θ₀) with x₀ = sin² θ₀".to_string()
    } else if (r - 3.5699456).abs() < 2e-4 {
        "the accumulation point of the period-doubling cascade (r∞ ≈ 3.5699456): onset of chaos".to_string()
    } else {
        match period {
            Some(1) => format!("converges to the fixed point {}", fmt((orbit[0] * 1e6).round() / 1e6)),
            Some(p) => {
                let mut cyc: Vec<f64> = orbit[..p].to_vec();
                cyc.sort_by(|a, b| a.partial_cmp(b).unwrap());
                format!("attracting {p}-cycle: {}", cyc.iter().map(|v| fmt((v * 1e6).round() / 1e6)).collect::<Vec<_>>().join(", "))
            }
            None if lyap > 0.0 => "chaotic (positive Lyapunov exponent)".into(),
            None => "aperiodic but not chaotic (near a bifurcation)".into(),
        }
    };
    let fp = 1.0 - 1.0 / r;
    let mut fixed = vec![("x* = 0".to_string(), format!("multiplier {} ({})", fmt(r), if r < 1.0 { "stable" } else { "unstable" }))];
    if r > 1.0 {
        let mu = 2.0 - r;
        fixed.push((format!("x* = 1 − 1/r = {}", fmt(fp)), format!("multiplier {} ({})", fmt((mu * 1e9).round() / 1e9), if mu.abs() < 1.0 { "stable" } else { "unstable" })));
    }
    let mut xs = vec![x0];
    for _ in 0..12 {
        let v = f(*xs.last().unwrap());
        xs.push(v);
    }
    let lyap_txt = if r == 4.0 { "ln 2 ≈ 0.693147".to_string() } else { fmt((lyap * 1e5).round() / 1e5) };
    Some(Answer::new("logistic map", vec![
        input(format!("logistic map x_(n+1) = {} x_n (1 − x_n)", fmt(r))),
        primary(pod("Result", "Long-term behaviour", text(behaviour))),
        pod("Lyapunov", "Lyapunov exponent", text(lyap_txt)),
        pod("Fixed", "Fixed points", table(None, fixed.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())),
        pod("Orbit", &format!("Orbit from x₀ = {}", fmt(x0)), text(xs.iter().map(|v| fmt((v * 1e6).round() / 1e6)).collect::<Vec<_>>().join(", "))),
        pod("Bifurcations", "Landmarks", text("period 2 at r = 3, period 4 at 1 + √6 ≈ 3.44949, chaos from r∞ ≈ 3.5699456, period-3 window from 1 + √8 ≈ 3.82843")),
    ]))
}

fn oscillators(l: &str) -> Option<Answer> {
    let l = l.trim_start_matches("the ").trim_end_matches(" equation").trim_end_matches(" system");
    let (name, eq, rows): (&str, &str, Vec<(&str, String)>) = match l {
        "van der pol oscillator" | "van der pol" => ("Van der Pol oscillator", "x'' − μ (1 − x²) x' + x = 0", vec![("parameter", "μ > 0: damping strength".into()), ("behaviour", "a unique stable limit cycle for every μ > 0".into()), ("limit-cycle amplitude", "≈ 2 (exactly 2 as μ → 0)".into()), ("period", "≈ 2π for small μ; ≈ (3 − 2 ln 2) μ for large μ (relaxation oscillations)".into()), ("Liénard form", "x' = μ (x − x³/3 − y), y' = x/μ".into())]),
        "duffing oscillator" | "duffing" => ("Duffing oscillator", "x'' + δ x' + α x + β x³ = γ cos(ω t)", vec![("parameters", "δ damping, α linear stiffness, β nonlinearity, γ and ω forcing".into()), ("behaviour", "hardening (β > 0) or softening (β < 0) springs, hysteresis and jumps in the frequency response, chaos for some forcings".into())]),
        "lorenz" | "lorenz attractor" => ("Lorenz system", "x' = σ (y − x), y' = x (ρ − z) − y, z' = x y − β z", vec![("classic parameters", "σ = 10, ρ = 28, β = 8/3".into()), ("fixed points", "origin, and (±√(β(ρ − 1)), ±√(β(ρ − 1)), ρ − 1)".into()), ("largest Lyapunov exponent", "≈ 0.9056 at the classic parameters".into()), ("attractor dimension", "≈ 2.06".into())]),
        "rossler" | "rössler" | "rossler attractor" | "rössler attractor" => ("Rössler system", "x' = −y − z, y' = x + a y, z' = b + z (x − c)", vec![("classic parameters", "a = 0.2, b = 0.2, c = 5.7".into()), ("behaviour", "a single-scroll chaotic attractor".into())]),
        _ => return None,
    };
    Some(Answer::new(name, vec![input(name.to_string()), primary(pod("Result", "Equation", text(eq))), pod("Properties", "Properties", kv(rows))]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = l.replace(',', "").replace("  ", " ");
    dice(&l).or_else(|| cards(&l)).or_else(|| odds(&l)).or_else(|| roulette(&l)).or_else(|| lottery(&l)).or_else(|| games(&l)).or_else(|| logistic(&l)).or_else(|| oscillators(&l))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ans(s: &str) -> String {
        serde_json::to_string(&crate::engine::query(s).pods).unwrap_or_default()
    }

    #[test]
    fn chance_answers() {
        assert!(ans("probability 3 queens 2 jacks").contains("1/108290"));
        assert!(ans("two 12-sided dice").contains("1/144"));
        assert!(ans("Powerball").contains("1 in 292201338"));
        assert!(ans("logistic map r=4").contains("ln 2"));
        assert!(ans("5:1 odds").contains("1/6"));
    }
}
