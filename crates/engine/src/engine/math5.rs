//! Assorted number theory and elementary geometry: last digits of power
//! towers, the last nonzero digit of a factorial, systems of linear
//! congruences, linear Diophantine equations in two unknowns, primitive
//! polynomial counts, metallic means, binomial success counts, triangles
//! from two sides and an angle (the ambiguous case), annuli, golden-ratio
//! lengths, and whether well-known constants are rational or transcendental.

use super::*;
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
    table(None, r.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())
}

fn gcd(a: i128, b: i128) -> i128 {
    if b == 0 { a.abs() } else { gcd(b, a % b) }
}

fn totient(mut m: u128) -> u128 {
    let mut r = m;
    let mut p = 2u128;
    while p * p <= m {
        if m % p == 0 {
            while m % p == 0 {
                m /= p;
            }
            r -= r / p;
        }
        p += 1;
    }
    if m > 1 {
        r -= r / m;
    }
    r
}

fn pow_mod(mut b: u128, mut e: u128, m: u128) -> u128 {
    if m == 1 {
        return 0;
    }
    let mut r = 1u128;
    b %= m;
    while e > 0 {
        if e & 1 == 1 {
            r = r * b % m;
        }
        b = b * b % m;
        e >>= 1;
    }
    r
}

/// The tower a₀^a₁^…, capped at `cap` (returns min(value, cap)).
fn tower_capped(t: &[u128], cap: u128) -> u128 {
    if t.len() == 1 {
        return t[0].min(cap);
    }
    let e = tower_capped(&t[1..], 128);
    let b = t[0];
    if b <= 1 {
        return b;
    }
    let mut r: u128 = 1;
    for _ in 0..e {
        r = r.saturating_mul(b);
        if r >= cap {
            return cap;
        }
    }
    r
}

/// a₀^a₁^… mod m, by the generalized Euler theorem.
fn tower_mod(t: &[u128], m: u128) -> u128 {
    if m == 1 {
        return 0;
    }
    if t.len() == 1 {
        return t[0] % m;
    }
    let phi = totient(m);
    let small = tower_capped(&t[1..], 1 << 20);
    let e = if small < (1 << 20) { small } else { tower_mod(&t[1..], phi) + phi };
    pow_mod(t[0], e, m)
}

fn last_digits(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?last (?:(\d+) )?digits? of (\d+(?:\s*\^\s*\d+)+)$").captures(l)?;
    let k: u32 = m.get(1).and_then(|s| s.as_str().parse().ok()).unwrap_or(1).min(30);
    let t: Vec<u128> = m[2].split('^').map(|s| s.trim().parse().ok()).collect::<Option<_>>()?;
    let md = 10u128.pow(k);
    let r = tower_mod(&t, md);
    let digits = format!("{:0width$}", r, width = k as usize);
    let expr = m[2].replace(' ', "");
    Some(Answer::new("last digits", vec![
        input(format!("last {} of {expr}", if k == 1 { "digit".to_string() } else { format!("{k} digits") })),
        primary(pod("Result", "Result", text(digits))),
        pod("Method", "Method", text("power towers evaluate from the top down; residues mod 10^k come from Euler's theorem, reducing exponents mod φ(10^k)")),
    ]))
}

fn last_nonzero_factorial(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?last non-?zero digit of ([\d,]+)\s*!$").captures(l)?;
    let nn: u64 = m[1].replace(',', "").parse().ok()?;
    if nn > 50_000_000 {
        return None;
    }
    let (mut prod, mut twos, mut fives) = (1u64, 0u64, 0u64);
    for i in 2..=nn {
        let mut x = i;
        while x % 2 == 0 {
            x /= 2;
            twos += 1;
        }
        while x % 5 == 0 {
            x /= 5;
            fives += 1;
        }
        prod = prod * (x % 10) % 10;
    }
    let extra = twos - fives;
    let d = prod * pow_mod(2, extra as u128, 10) as u64 % 10;
    Some(Answer::new("last nonzero digit", vec![
        input(format!("last nonzero digit of {nn}!")),
        primary(pod("Result", "Result", text(d.to_string()))),
        pod("Details", "Details", rows(vec![("trailing zeros of n!", fives.to_string())])),
    ]))
}

/// "solve 2x = 10 (mod 12), 3x = 9 (mod 12)"
fn congruences(l: &str) -> Option<Answer> {
    let body = re!(r"^solve\s+(.+)$").captures(l)?[1].to_string();
    if !body.contains("mod") {
        return None;
    }
    let mut eqs = Vec::new();
    for part in body.split([',', ';']).map(str::trim).filter(|s| !s.is_empty()) {
        let p = part.trim_start_matches("and ").trim();
        let m = re!(r"^(-?\d*)\s*\*?\s*([a-z])\s*(?:([+-])\s*(\d+))?\s*(?:=|≡)\s*(-?\d+)\s*\(?\s*mod\s*(\d+)\s*\)?$").captures(p)?;
        let a: i128 = match &m[1] { "" => 1, "-" => -1, s => s.parse().ok()? };
        let off: i128 = m.get(4).map_or(Some(0), |x| x.as_str().parse().ok())? * if m.get(3).map(|x| x.as_str()) == Some("-") { -1 } else { 1 };
        let b: i128 = m[5].parse().ok()?;
        let md: i128 = m[6].parse().ok()?;
        eqs.push((m[2].to_string(), a, b - off, md));
    }
    if eqs.len() < 2 {
        return None;
    }
    let var = eqs.first()?.0.clone();
    if eqs.iter().any(|e| e.0 != var) {
        return None;
    }
    let lcm = eqs.iter().fold(1i128, |acc, e| acc / gcd(acc, e.3) * e.3);
    if lcm > 10_000_000 {
        return None;
    }
    let sols: Vec<i128> = (0..lcm).filter(|x| eqs.iter().all(|(_, a, b, md)| (a * x - b).rem_euclid(*md) == 0)).collect();
    let shown = eqs.iter().map(|(v, a, b, md)| format!("{a}{v} ≡ {b} (mod {md})")).collect::<Vec<_>>().join(", ");
    let head = if sols.is_empty() {
        "no solutions".to_string()
    } else {
        sols.iter().map(|s| format!("{var} ≡ {s} (mod {lcm})")).collect::<Vec<_>>().join(", ")
    };
    Some(Answer::new("congruences", vec![
        input(format!("solve {shown}")),
        primary(pod("Result", "Solutions", text(head))),
        pod("Count", "Number of solutions modulo the common modulus", text(format!("{} (mod {lcm})", sols.len()))),
    ]))
}

fn ext_gcd(a: i128, b: i128) -> (i128, i128, i128) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x, y) = ext_gcd(b, a % b);
        (g, y, x - (a / b) * y)
    }
}

/// "solve 3x+4y=5 over the integers"
fn diophantine(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:solve\s+)?(-?\d*)\s*\*?\s*([a-z])\s*([+-])\s*(\d*)\s*\*?\s*([a-z])\s*=\s*(-?\d+)\s+(?:over|in) the integers$").captures(l)?;
    let a: i128 = match &m[1] { "" => 1, "-" => -1, s => s.parse().ok()? };
    let mut b: i128 = if m[4].is_empty() { 1 } else { m[4].parse().ok()? };
    if &m[3] == "-" {
        b = -b;
    }
    let c: i128 = m[6].parse().ok()?;
    let (x, y) = (&m[2], &m[5]);
    let (g, s0, t0) = ext_gcd(a.abs(), b.abs());
    let (s0, t0) = (s0 * a.signum(), t0 * b.signum());
    if c % g != 0 {
        return Some(Answer::new("Diophantine equation", vec![input(format!("{a}{x} + {b}{y} = {c} over the integers")), primary(pod("Result", "Result", text(format!("no integer solutions: gcd({a}, {b}) = {g} does not divide {c}"))))]));
    }
    let (mut x0, mut y0) = (s0 * c / g, t0 * c / g);
    let (dx, dy) = (b / g, -a / g);
    // smallest nonnegative x
    let k = (-x0).div_euclid(dx.abs()) * dx.signum();
    let k = if dx != 0 { if (x0 + k * dx) < 0 { k + dx.signum() } else { k } } else { 0 };
    x0 += k * dx;
    y0 += k * dy;
    let sign = |v: i128| if v < 0 { format!("− {}", -v) } else { format!("+ {v}") };
    let gen = format!("{x} = {x0} {} n,  {y} = {y0} {} n,  n ∈ ℤ", sign(dx), sign(dy));
    let examples: Vec<Vec<Content>> = (-2..=2).map(|k: i128| vec![text(k.to_string()), text((x0 + k * dx).to_string()), text((y0 + k * dy).to_string())]).collect();
    Some(Answer::new("Diophantine equation", vec![
        input(format!("{a}{x} {} {}{y} = {c} over the integers", if b < 0 { "−" } else { "+" }, b.abs())),
        primary(pod("Result", "Integer solutions", text(gen))),
        pod("Examples", "Particular solutions", table(Some(vec!["n", x, y]), examples)),
    ]))
}

fn primitive_polys(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?number of primitive polynomials (?:of|over|for) gf\((\d+)(?:\^(\d+))?\)$").captures(l)?;
    let q: u128 = m[1].parse().ok()?;
    let (p, k) = {
        // q = p^k for a prime p
        let mut p = 2u128;
        while p * p <= q && q % p != 0 {
            p += 1;
        }
        if q % p != 0 {
            p = q;
        }
        let mut k = 0u32;
        let mut r = q;
        while r % p == 0 {
            r /= p;
            k += 1;
        }
        if r != 1 {
            return None;
        }
        (p, k)
    };
    let extra: u32 = m.get(2).and_then(|s| s.as_str().parse().ok()).unwrap_or(1);
    let k = k * extra;
    let order = p.checked_pow(k)?;
    let count = totient(order - 1) / k as u128;
    Some(Answer::new("primitive polynomials", vec![
        input(format!("number of primitive polynomials defining GF({order}) = GF({p}^{k}) over GF({p})")),
        primary(pod("Result", "Number of primitive polynomials of degree {k} over GF(p)", text(count.to_string()))),
        pod("Formula", "Formula", text(format!("φ(p^k − 1)/k = φ({})/{k}", order - 1))),
    ]))
}

fn metallic(l: &str) -> Option<Answer> {
    let names = [("golden", 1.0), ("silver", 2.0), ("bronze", 3.0), ("copper", 4.0), ("nickel", 5.0)];
    let (name, k): (String, f64) = if let Some(m) = re!(r"^(silver|bronze|copper|nickel) (?:ratio|mean)$").captures(l) {
        let k = names.iter().find(|x| x.0 == &m[1])?.1;
        (m[1].to_string(), k)
    } else if let Some(m) = re!(r"^(\d+)(?:st|nd|rd|th)? metallic (?:mean|ratio)$").captures(l) {
        let k: f64 = m[1].parse().ok()?;
        (format!("{} metallic", m[1].to_string()), k)
    } else {
        return None;
    };
    let v = (k + (k * k + 4.0).sqrt()) / 2.0;
    let exact = if k == 2.0 { "1 + √2".to_string() } else { format!("({k} + √{})/2", k * k + 4.0) };
    Some(Answer::new(&format!("{name} ratio"), vec![
        input(format!("{name} ratio (metallic mean δ_{k})")),
        primary(pod("Result", "Exact form", text(exact))),
        pod("Decimal", "Decimal approximation", text(format!("{:.20}", v))),
        pod("Properties", "Properties", rows(vec![
            ("minimal polynomial", format!("x² − {k}x − 1")),
            ("continued fraction", format!("[{k}; {k}, {k}, {k}, …]")),
            ("reciprocal", format!("δ − {k} = {}", n(1.0 / v))),
        ])),
    ]))
}

fn binomial_successes(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?probability of (?:(at least|at most|exactly|more than|fewer than|less than) )?(\d+) successes? in (\d+) trials? (?:with|where|given) p\s*=\s*(0?\.\d+|0|1)$").captures(l)?;
    let k: u64 = m[2].parse().ok()?;
    let nn: u64 = m[3].parse().ok()?;
    let p: f64 = m[4].parse().ok()?;
    if k > nn || nn > 10_000 {
        return None;
    }
    let lnc = |j: u64| crate::special::lgamma(nn as f64 + 1.0) - crate::special::lgamma(j as f64 + 1.0) - crate::special::lgamma((nn - j) as f64 + 1.0);
    let pmf = |j: u64| (lnc(j) + j as f64 * p.ln() + (nn - j) as f64 * (1.0 - p).ln()).exp();
    let pmf = |j: u64| if p == 0.0 { if j == 0 { 1.0 } else { 0.0 } } else if p == 1.0 { if j == nn { 1.0 } else { 0.0 } } else { pmf(j) };
    let sum = |r: std::ops::RangeInclusive<u64>| r.map(pmf).sum::<f64>();
    let (label, v) = match m.get(1).map(|x| x.as_str()) {
        Some("at least") => ("P(X ≥ k)", sum(k..=nn)),
        Some("more than") => ("P(X > k)", if k < nn { sum(k + 1..=nn) } else { 0.0 }),
        Some("at most") => ("P(X ≤ k)", sum(0..=k)),
        Some("fewer than") | Some("less than") => ("P(X < k)", if k > 0 { sum(0..=k - 1) } else { 0.0 }),
        _ => ("P(X = k)", pmf(k)),
    };
    Some(Answer::new("binomial probability", vec![
        input(format!("binomial distribution | n = {nn} trials | p = {p} | k = {k} successes")),
        primary(pod("Result", label, text(n(v)))),
        pod("Distribution", "Distribution", rows(vec![("mean", n(nn as f64 * p)), ("standard deviation", n((nn as f64 * p * (1.0 - p)).sqrt())), ("P(X = k)", n(pmf(k))), ("P(X ≤ k)", n(sum(0..=k))), ("P(X ≥ k)", n(sum(k..=nn)))])),
    ]))
}

/// Law of sines from two sides and a non-included angle (the ambiguous case).
fn ssa(l: &str) -> Option<Answer> {
    if !(l.starts_with("law of sines") || l.starts_with("triangle") || l.starts_with("solve triangle") || l.starts_with("solve the triangle")) {
        return None;
    }
    let side = |name: &str| -> Option<f64> {
        let r = Regex::new(&format!(r"\b{name}\s*=\s*([\d.]+)")).ok()?;
        r.captures(l).and_then(|c| c[1].parse().ok())
    };
    let angle = |names: &[&str]| -> Option<f64> {
        for nm in names {
            let r = Regex::new(&format!(r"\b{nm}\s*=\s*([\d.]+)\s*(deg|degrees|°|rad|radians)?")).ok()?;
            if let Some(c) = r.captures(l) {
                let v: f64 = c[1].parse().ok()?;
                return Some(if c.get(2).is_some_and(|u| u.as_str().starts_with("rad")) { v } else { v.to_radians() });
            }
        }
        None
    };
    let unit = re!(r"\d\s*(mm|cm|m|km|in|ft)\b").captures(l).map(|c| format!(" {}", &c[1])).unwrap_or_default();
    let (a, b, alpha) = (side("a")?, side("b")?, angle(&["alpha", "α", "a_angle", "angle a"])?);
    if side("c").is_some() {
        return None;
    }
    let sb = b * alpha.sin() / a;
    if sb > 1.0 + 1e-12 {
        return Some(Answer::new("triangle", vec![input(format!("triangle with a = {}{unit}, b = {}{unit}, α = {}°", n(a), n(b), n(alpha.to_degrees()))), primary(pod("Result", "Result", text("no triangle exists: b sin α > a")))]));
    }
    let b1 = sb.min(1.0).asin();
    let mut sols = vec![b1];
    if (std::f64::consts::PI - b1 - alpha) > 1e-9 && (b1 - std::f64::consts::FRAC_PI_2).abs() > 1e-9 && a < b {
        sols.push(std::f64::consts::PI - b1);
    }
    let sols: Vec<_> = sols.into_iter().filter(|beta| alpha + beta < std::f64::consts::PI).collect();
    let mut pods = vec![input(format!("triangle with a = {}{unit}, b = {}{unit}, α = {}°", n(a), n(b), n(alpha.to_degrees())))];
    let mut heads = Vec::new();
    for (i, beta) in sols.iter().enumerate() {
        let gamma = std::f64::consts::PI - alpha - beta;
        let c = a * gamma.sin() / alpha.sin();
        let area = 0.5 * a * b * gamma.sin();
        heads.push(format!("β = {}°, γ = {}°, c = {}{unit}", n(beta.to_degrees()), n(gamma.to_degrees()), n(c)));
        pods.push(pod(&format!("Triangle{}", i + 1), &format!("Triangle {}", i + 1), rows(vec![
            ("angle β", format!("{}°", n(beta.to_degrees()))),
            ("angle γ", format!("{}°", n(gamma.to_degrees()))),
            ("side c", format!("{}{unit}", n(c))),
            ("area", format!("{}{}", n(area), if unit.is_empty() { String::new() } else { format!("{unit}²") })),
            ("perimeter", format!("{}{unit}", n(a + b + c))),
        ])));
    }
    pods.insert(1, primary(pod("Result", if sols.len() == 2 { "Two triangles (ambiguous case)" } else { "Solution" }, text(heads.join("\n")))));
    pods.push(pod("Formula", "Law of sines", text("a/sin α = b/sin β = c/sin γ")));
    Some(Answer::new("triangle", pods))
}

fn annulus(l: &str) -> Option<Answer> {
    if !l.starts_with("annulus") {
        return None;
    }
    let r = re!(r"inner radius\s*(?:=|of)?\s*([\d.]+)").captures(l)?[1].parse::<f64>().ok()?;
    let rr = re!(r"outer radius\s*(?:=|of)?\s*([\d.]+)").captures(l)?[1].parse::<f64>().ok()?;
    if rr <= r {
        return None;
    }
    let pi = std::f64::consts::PI;
    let area = pi * (rr * rr - r * r);
    Some(Answer::new("annulus", vec![
        input(format!("annulus | inner radius {} | outer radius {}", n(r), n(rr))),
        primary(pod("Result", "Area", text(format!("{}π ≈ {}", n(rr * rr - r * r), n(area))))),
        pod("Properties", "Properties", rows(vec![
            ("width", n(rr - r)),
            ("inner circumference", format!("{}π ≈ {}", n(2.0 * r), n(2.0 * pi * r))),
            ("outer circumference", format!("{}π ≈ {}", n(2.0 * rr), n(2.0 * pi * rr))),
            ("total perimeter", format!("{}π ≈ {}", n(2.0 * (r + rr)), n(2.0 * pi * (r + rr)))),
            ("polar moment of area", format!("π(R⁴ − r⁴)/2 ≈ {}", n(pi * (rr.powi(4) - r.powi(4)) / 2.0))),
        ])),
    ]))
}

fn golden_lengths(l: &str) -> Option<Answer> {
    let m = re!(r"^golden (?:ratio|rectangle|section)(?: with)? (?:side|length|segment|side 1|a)\s*(?:1\s*)?(?:=|of)?\s*([\d.]+)\s*([a-z]*)$").captures(l)?;
    let s: f64 = m[1].parse().ok()?;
    let u = if m[2].is_empty() { String::new() } else { format!(" {}", &m[2]) };
    let phi = (1.0 + 5f64.sqrt()) / 2.0;
    Some(Answer::new("golden ratio", vec![
        input(format!("golden ratio with one length {}{u}", n(s))),
        primary(pod("Result", "Other length", text(format!("{}{u} (longer: s φ) or {}{u} (shorter: s/φ)", n(s * phi), n(s / phi))))),
        pod("Details", "Details", rows(vec![("φ", "(1 + √5)/2 ≈ 1.618034".into()), ("sum s + sφ", format!("{}{u}", n(s + s * phi))), ("golden rectangle area (s × sφ)", n(s * s * phi))])),
    ]))
}

fn number_class(l: &str) -> Option<Answer> {
    let m = re!(r"^is (?:the )?(.+?) (rational|irrational|transcendental|algebraic)$").captures(l)?;
    let what = m[1].trim();
    // (names, rational?, algebraic?, note): None means unknown
    const T: &[(&[&str], Option<bool>, Option<bool>, &str)] = &[
        (&["pi", "π"], Some(false), Some(false), "transcendental (Lindemann, 1882)"),
        (&["e", "euler's number"], Some(false), Some(false), "transcendental (Hermite, 1873)"),
        (&["golden ratio", "phi", "φ"], Some(false), Some(true), "algebraic of degree 2: a root of x² − x − 1"),
        (&["silver ratio"], Some(false), Some(true), "algebraic of degree 2: a root of x² − 2x − 1"),
        (&["sqrt(2)", "square root of 2", "√2"], Some(false), Some(true), "algebraic of degree 2: a root of x² − 2"),
        (&["euler-mascheroni constant", "euler mascheroni constant", "euler's constant", "euler gamma", "γ"], None, None, "unknown whether it is rational"),
        (&["catalan's constant", "catalan constant"], None, None, "unknown whether it is rational"),
        (&["apery's constant", "apéry's constant", "zeta(3)", "ζ(3)"], Some(false), None, "irrational (Apéry, 1978); transcendence unknown"),
        (&["e^pi", "gelfond's constant", "e^π"], Some(false), Some(false), "transcendental (Gelfond–Schneider)"),
        (&["2^sqrt(2)", "gelfond-schneider constant"], Some(false), Some(false), "transcendental (Gelfond–Schneider)"),
        (&["ln 2", "log 2", "ln(2)", "log(2)"], Some(false), Some(false), "transcendental (Lindemann–Weierstrass)"),
        (&["pi + e", "π + e", "pi*e", "pi^e", "π^e"], None, None, "unknown whether it is rational"),
        (&["khinchin's constant", "khinchin constant"], None, None, "unknown whether it is rational"),
        (&["0.999...", "0.999…"], Some(true), Some(true), "equal to 1"),
    ];
    let (rational, algebraic, note) = if let Some(t) = T.iter().find(|t| t.0.contains(&what)) {
        (t.1, t.2, t.3.to_string())
    } else if let Some(c) = re!(r"^(\d+)(?:st|nd|rd|th) bernoulli number$").captures(what) {
        let k: u64 = c[1].parse().ok()?;
        (Some(true), Some(true), format!("every Bernoulli number is rational (B_{k} = {})", crate::numtheory::bernoulli(k as usize)))
    } else {
        return None;
    };
    let asked = &m[2];
    let verdict = match (asked, rational, algebraic) {
        ("rational", Some(r), _) => if r { "yes" } else { "no" },
        ("irrational", Some(r), _) => if r { "no" } else { "yes" },
        ("transcendental", _, Some(a)) => if a { "no" } else { "yes" },
        ("algebraic", _, Some(a)) => if a { "yes" } else { "no" },
        _ => "unknown (an open problem)",
    };
    Some(Answer::new("number classification", vec![
        input(format!("is {what} {asked}?")),
        primary(pod("Result", "Result", text(verdict))),
        pod("Details", "Details", text(note)),
    ]))
}

// best-known packings (Packomania): radius of n equal circles in a unit
// square, and container-to-circle radius ratio for n circles in a circle
const SQ_R: [f64; 30] = [0.5, 0.292893, 0.254333, 0.25, 0.207107, 0.187681, 0.174458, 0.170541, 0.166667, 0.148204, 0.142399, 0.139959, 0.133994, 0.129332, 0.127167, 0.125, 0.117197, 0.115521, 0.112265, 0.111382, 0.106860, 0.105665, 0.102802, 0.101381, 0.1, 0.096362, 0.095420, 0.093672, 0.092463, 0.091671];
const CI_R: [f64; 30] = [1.0, 2.0, 2.154701, 2.414214, 2.701302, 3.0, 3.0, 3.304765, 3.613126, 3.813026, 3.923804, 4.029621, 4.236068, 4.328428, 4.521357, 4.615383, 4.792034, 4.863703, 4.863703, 5.122307, 5.252485, 5.439736, 5.545552, 5.651309, 5.752824, 5.828427, 5.906397, 6.014992, 6.138544, 6.197741];

/// "2\"", "3in", "2 ft", "5 cm" → (value, unit label)
fn len_unit(s: &str) -> Option<(f64, String)> {
    let m = re!(r#"^(\d+(?:\.\d+)?)\s*("|''|in|inch|inches|ft|feet|foot|'|cm|mm|m|meters?|metres?)?$"#).captures(s.trim())?;
    let v: f64 = m[1].parse().ok()?;
    let u = match m.get(2).map(|x| x.as_str()) {
        Some("\"") | Some("''") | Some("in") | Some("inch") | Some("inches") => "in",
        Some("ft") | Some("feet") | Some("foot") | Some("'") => "ft",
        Some(u) => u.trim_end_matches('s'),
        None => "",
    };
    Some((v, u.to_string()))
}

fn to_base(v: f64, u: &str) -> f64 {
    v * match u {
        "ft" => 12.0,
        "cm" => 1.0 / 2.54,
        "mm" => 1.0 / 25.4,
        "m" | "meter" | "metre" => 100.0 / 2.54,
        _ => 1.0,
    }
}

fn circle_packing(l: &str) -> Option<Answer> {
    let m = re!(r#"^(?:pack(?:ing)?|fit)\s+(\d+)\s+(?:(.+?)\s+)?(?:circles|disks|discs|coins|balls|spheres)\s+(?:in|into|inside)\s+an?\s+(square|circle)(?:\s+of\s+(?:side|diameter|radius)\s+.+)?$"#).captures(l)?;
    let k: usize = m[1].parse().ok()?;
    let container = &m[3];
    let size = m.get(2).map(|x| x.as_str().trim_start_matches("unit ").trim_end_matches("diameter").trim_end_matches("-").trim().to_string());
    let (d, unit) = match size.as_deref() {
        Some(s) if !s.is_empty() => len_unit(s)?,
        _ => (1.0, String::new()),
    };
    let r = d / 2.0;
    let u = if unit.is_empty() { String::new() } else { format!(" {unit}") };
    let (main, density, ratio_txt) = if container == "square" {
        let rr = *SQ_R.get(k.checked_sub(1)?)?;
        let side = r / rr;
        let dens = k as f64 * std::f64::consts::PI * rr * rr;
        (format!("smallest square side ≈ {}{u}", n(side)), dens, format!("circle radius in a unit square: {}", n(rr)))
    } else {
        let ratio = *CI_R.get(k.checked_sub(1)?)?;
        let rad = r * ratio;
        let dens = k as f64 / (ratio * ratio);
        (format!("smallest circle diameter ≈ {}{u}  (radius {}{u})", n(2.0 * rad), n(rad)), dens, format!("container radius / circle radius: {}", n(ratio)))
    };
    let what = if unit.is_empty() && size.as_deref().map_or(true, |s| s.is_empty()) { format!("{k} unit-diameter circles") } else { format!("{k} circles of diameter {}{u}", n(d)) };
    Some(Answer::new(
        "circle packing",
        vec![
            input(format!("pack {what} in a {container}")),
            primary(pod("Result", "Result", text(main))),
            pod("Details", "Best-known packing", rows(vec![("ratio", ratio_txt), ("packing density", format!("{} ({}% of the container covered)", n(density), n((density * 1000.0).round() / 10.0)))])),
            pod("Source", "Note", text("best-known (and for small n proven optimal) packings of equal circles, as tabulated by E. Specht's Packomania")),
        ],
    ))
}

fn squares_in_circle(l: &str) -> Option<Answer> {
    let m = re!(r#"^(?:pack(?:ing)?|fit)\s+(?:(\d+)\s+)?squares\s+of\s+side\s+(.+?)\s+(?:in|into|inside)\s+an?\s+circle\s+of\s+(diameter|radius)\s+(.+)$"#).captures(l)?;
    let (a, ua) = len_unit(&m[2])?;
    let (c, uc) = len_unit(&m[4])?;
    let a_in = to_base(a, &ua);
    let mut rad = to_base(c, &uc);
    if &m[3] == "diameter" {
        rad /= 2.0;
    }
    let ratio = rad / a_in;
    if !(ratio > 0.0 && ratio < 200.0) {
        return None;
    }
    // best axis-aligned row arrangement: rows of height a at offset y0,
    // each row as wide as the circle allows at its worse edge, columns
    // independently centred per row
    let mut best = 0usize;
    for step in 0..=200 {
        let y0 = -ratio + step as f64 / 200.0;
        let mut count = 0usize;
        let mut y = y0;
        while y + 1.0 <= ratio + 1e-12 {
            let w = |yy: f64| (ratio * ratio - yy * yy).max(0.0).sqrt();
            let half = w(y).min(w(y + 1.0));
            count += (2.0 * half + 1e-12).floor() as usize;
            y += 1.0;
        }
        best = best.max(count);
    }
    let area_bound = (std::f64::consts::PI * ratio * ratio).floor() as usize;
    Some(Answer::new(
        "square packing",
        vec![
            input(format!("pack squares of side {}{} in a circle of {} {}{}", n(a), if ua.is_empty() { String::new() } else { format!(" {ua}") }, &m[3], n(c), if uc.is_empty() { String::new() } else { format!(" {uc}") })),
            primary(pod("Result", "Squares that fit", text(format!("{best} (row arrangement)")))),
            pod("Details", "Details", rows(vec![("circle radius / square side", n(ratio)), ("area upper bound", area_bound.to_string()), ("covered fraction", format!("{}%", n((best as f64 / (std::f64::consts::PI * ratio * ratio) * 1000.0).round() / 10.0)))])),
            pod("Method", "Method", text("axis-aligned rows of squares, each row as long as the circle allows and centred, with the row offset optimised; tilted arrangements can occasionally fit one or two more")),
        ],
    ))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    last_digits(&l)
        .or_else(|| last_nonzero_factorial(&l))
        .or_else(|| congruences(&l))
        .or_else(|| diophantine(&l))
        .or_else(|| primitive_polys(&l))
        .or_else(|| metallic(&l))
        .or_else(|| binomial_successes(&l))
        .or_else(|| ssa(&l))
        .or_else(|| annulus(&l))
        .or_else(|| golden_lengths(&l))
        .or_else(|| number_class(&l))
        .or_else(|| circle_packing(&l))
        .or_else(|| squares_in_circle(&l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn towers_and_factorials() {
        // 9^9^9 ends in 9; 2^3^4 = 2^81 ends in ...2952 (2^81 = 2417851639229258349412352)
        assert_eq!(tower_mod(&[9, 9, 9], 10), 9);
        assert_eq!(tower_mod(&[2, 3, 4], 10000), 2352);
        assert_eq!(tower_mod(&[3, 3], 1000), 27);
    }
}
