//! Named integer sequences as sets: "1000th twin prime", "fourth Lucas
//! number > 1000000", "20 greatest triangular numbers < 500", "sum of first
//! five square triangular numbers", "is 4 a member of the superperfect
//! numbers?", "primes of the form 4k+1", "integers that are not deficient
//! and are tetrahedral between 10 and 20000", plus a few divisor and digit
//! phrasings ("common divisors of 360 and 96", "Pascal's triangle mod 3").

use super::*;
use crate::numtheory as nt;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, ToPrimitive, Zero};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// Exponents p of the known Mersenne primes 2^p − 1, in order.
const MERSENNE: &[u64] = &[
    2, 3, 5, 7, 13, 17, 19, 31, 61, 89, 107, 127, 521, 607, 1279, 2203, 2281, 3217, 4253, 4423, 9689, 9941, 11213, 19937, 21701, 23209, 44497, 86243, 110503, 132049, 216091, 756839, 859433, 1257787, 1398269, 2976221, 3021377, 6972593, 13466917, 20996011, 24036583, 25964951, 30402457, 32582657, 37156667, 42643801, 43112609, 57885161, 74207281, 77232917, 82589933, 136279841,
];

enum Kind {
    /// term(k) for k = start, start + 1, … (non-decreasing)
    Formula(fn(u64) -> BigInt, u64),
    /// all positive integers passing the test
    Filter(fn(u64) -> bool),
    /// built from the Mersenne exponents: value of the nth term
    Mersenne(fn(u64) -> BigInt, fn(u64) -> String),
    /// finite known list
    Known(&'static [u64], &'static str),
}

struct Seq {
    key: &'static str,
    name: &'static str,
    alias: &'static str,
    def: &'static str,
    kind: Kind,
}

fn b(n: u64) -> BigInt {
    BigInt::from(n)
}

fn sigma(n: u64) -> u64 {
    let mut s = 1u64;
    let mut m = n;
    let mut p = 2u64;
    while p * p <= m {
        if m % p == 0 {
            let mut t = 1u64;
            let mut pk = 1u64;
            while m % p == 0 {
                m /= p;
                pk *= p;
                t += pk;
            }
            s *= t;
        }
        p += if p == 2 { 1 } else { 2 };
    }
    if m > 1 {
        s *= m + 1;
    }
    s
}

fn omega_count(n: u64) -> u32 {
    // number of prime factors with multiplicity
    let mut c = 0;
    let mut m = n;
    let mut p = 2u64;
    while p * p <= m {
        while m % p == 0 {
            m /= p;
            c += 1;
        }
        p += 1;
    }
    if m > 1 {
        c += 1;
    }
    c
}

fn squarefree(n: u64) -> bool {
    let mut p = 2u64;
    while p * p <= n {
        if n % (p * p) == 0 {
            return false;
        }
        p += 1;
    }
    n >= 1
}

fn reverse(n: u64) -> u64 {
    n.to_string().chars().rev().collect::<String>().parse().unwrap_or(0)
}

fn happy(mut n: u64) -> bool {
    let mut seen = Vec::new();
    while n != 1 && !seen.contains(&n) {
        seen.push(n);
        n = n.to_string().bytes().map(|d| ((d - b'0') as u64).pow(2)).sum();
    }
    n == 1
}

fn carmichael(n: u64) -> bool {
    if n < 3 || n % 2 == 0 || nt::is_prime_u64(n) || !squarefree(n) {
        return false;
    }
    let mut m = n;
    let mut p = 3u64;
    while p * p <= m {
        if m % p == 0 {
            if (n - 1) % (p - 1) != 0 {
                return false;
            }
            m /= p;
        }
        p += 2;
    }
    m == 1 || (n - 1) % (m - 1) == 0
}

fn perfect_power(n: u64) -> bool {
    if n == 1 {
        return true;
    }
    (2..64).any(|e| {
        let r = (n as f64).powf(1.0 / e as f64).round() as u64;
        (r.saturating_sub(1)..=r + 1).any(|c| c >= 2 && c.checked_pow(e).map_or(false, |v| v == n))
    })
}

fn pell(k: u64) -> BigInt {
    let (mut a, mut c) = (BigInt::zero(), BigInt::one());
    for _ in 0..k {
        let t = &c * 2 + &a;
        a = c;
        c = t;
    }
    a
}

fn square_triangular(k: u64) -> BigInt {
    // s(0) = 0, s(1) = 1, s(k+1) = 34 s(k) − s(k−1) + 2
    let (mut a, mut c) = (BigInt::zero(), BigInt::one());
    for _ in 0..k {
        let t = &c * 34 - &a + 2;
        a = c;
        c = t;
    }
    a
}

fn bell(k: u64) -> BigInt {
    let mut row = vec![BigInt::one()];
    for _ in 0..k {
        let mut next = vec![row.last().unwrap().clone()];
        for v in &row {
            let t = next.last().unwrap() + v;
            next.push(t);
        }
        row = next;
    }
    row[0].clone()
}

fn seqs() -> &'static [Seq] {
    static S: OnceLock<Vec<Seq>> = OnceLock::new();
    S.get_or_init(|| {
        use Kind::*;
        vec![
            Seq { key: "twin", name: "twin primes", alias: r"twin primes?(?: pairs?)?", def: "pairs of primes (p, p + 2)", kind: Filter(|n| nt::is_prime_u64(n) && nt::is_prime_u64(n + 2)) },
            Seq { key: "sophie", name: "Sophie Germain primes", alias: r"sophie germain primes?", def: "primes p with 2p + 1 also prime", kind: Filter(|n| nt::is_prime_u64(n) && nt::is_prime_u64(2 * n + 1)) },
            Seq { key: "safe", name: "safe primes", alias: r"safe primes?", def: "primes p with (p − 1)/2 also prime", kind: Filter(|n| n > 4 && nt::is_prime_u64(n) && nt::is_prime_u64((n - 1) / 2)) },
            Seq { key: "emirp", name: "emirps", alias: r"emirps?", def: "primes whose digit reversal is a different prime", kind: Filter(|n| nt::is_prime_u64(n) && reverse(n) != n && nt::is_prime_u64(reverse(n))) },
            Seq { key: "mersenneprime", name: "Mersenne primes", alias: r"mersenne primes?", def: "primes of the form 2^p − 1 (52 known)", kind: Mersenne(|p| (BigInt::one() << p as usize) - 1, |p| format!("2^{p} − 1")) },
            Seq { key: "perfect", name: "perfect numbers", alias: r"perfect(?: numbers?)?", def: "n with σ(n) = 2n; every known one is 2^(p−1)(2^p − 1) with 2^p − 1 a Mersenne prime", kind: Mersenne(|p| (BigInt::one() << (p - 1) as usize) * ((BigInt::one() << p as usize) - 1), |p| format!("2^{} (2^{p} − 1)", p - 1)) },
            Seq { key: "superperfect", name: "superperfect numbers", alias: r"super-?perfect(?: numbers?)?", def: "n with σ(σ(n)) = 2n; the even ones are 2^(p−1) with 2^p − 1 a Mersenne prime (no odd one is known)", kind: Mersenne(|p| BigInt::one() << (p - 1) as usize, |p| format!("2^{}", p - 1)) },
            Seq { key: "fermatprime", name: "Fermat primes", alias: r"fermat primes?", def: "primes of the form 2^(2^n) + 1 (only five are known)", kind: Known(&[3, 5, 17, 257, 65537], "no further Fermat prime is known; F₅ through F₃₂ are all composite") },
            Seq { key: "fermat", name: "Fermat numbers", alias: r"fermat numbers?", def: "F(n) = 2^(2^n) + 1, n ≥ 0", kind: Formula(|k| if k > 20 { BigInt::from(-1) } else { (BigInt::one() << (1usize << k)) + 1 }, 0) },
            Seq { key: "mersenne", name: "Mersenne numbers", alias: r"mersenne numbers?", def: "M(n) = 2^n − 1", kind: Formula(|k| (BigInt::one() << k as usize) - 1, 1) },
            Seq { key: "carmichael", name: "Carmichael numbers", alias: r"carmichael(?: numbers?)?", def: "composite n with a^(n−1) ≡ 1 (mod n) for every a coprime to n", kind: Filter(carmichael) },
            Seq { key: "semiprime", name: "semiprimes", alias: r"semi-?primes?", def: "products of two primes", kind: Filter(|n| omega_count(n) == 2) },
            Seq { key: "composite", name: "composite numbers", alias: r"composites?(?: numbers?)?", def: "integers > 1 that are not prime", kind: Filter(|n| n > 3 && !nt::is_prime_u64(n)) },
            Seq { key: "prime", name: "prime numbers", alias: r"primes?(?: numbers?)?", def: "integers > 1 whose only positive divisors are 1 and themselves", kind: Filter(nt::is_prime_u64) },
            Seq { key: "abundant", name: "abundant numbers", alias: r"abundant(?: numbers?)?", def: "n with σ(n) > 2n", kind: Filter(|n| sigma(n) > 2 * n) },
            Seq { key: "deficient", name: "deficient numbers", alias: r"deficient(?: numbers?)?", def: "n with σ(n) < 2n", kind: Filter(|n| sigma(n) < 2 * n) },
            Seq { key: "squarefree", name: "squarefree numbers", alias: r"square-?free(?: numbers?| integers?)?", def: "n divisible by no square greater than 1", kind: Filter(squarefree) },
            Seq { key: "palindromic", name: "palindromic numbers", alias: r"palindromic(?: numbers?)?|palindromes?", def: "numbers that read the same backwards", kind: Filter(|n| reverse(n) == n) },
            Seq { key: "happy", name: "happy numbers", alias: r"happy(?: numbers?)?", def: "iterating the sum of squared digits reaches 1", kind: Filter(happy) },
            Seq { key: "harshad", name: "Harshad numbers", alias: r"(?:harshad|niven)(?: numbers?)?", def: "n divisible by its digit sum", kind: Filter(|n| n % n.to_string().bytes().map(|d| (d - b'0') as u64).sum::<u64>() == 0) },
            Seq { key: "perfectpower", name: "perfect powers", alias: r"perfect powers?", def: "m^k with m ≥ 1 and k ≥ 2", kind: Filter(perfect_power) },
            Seq { key: "sqtri", name: "square triangular numbers", alias: r"square[- ]triangular(?: numbers?)?", def: "numbers both square and triangular; s(k+1) = 34 s(k) − s(k−1) + 2", kind: Formula(square_triangular, 1) },
            Seq { key: "triangular", name: "triangular numbers", alias: r"triangular(?: numbers?)?|triangle numbers?", def: "T(n) = n(n + 1)/2", kind: Formula(|k| b(k * (k + 1) / 2), 1) },
            Seq { key: "tetrahedral", name: "tetrahedral numbers", alias: r"tetrahedral(?: numbers?)?", def: "Te(n) = n(n + 1)(n + 2)/6", kind: Formula(|k| b(k * (k + 1) * (k + 2) / 6), 1) },
            Seq { key: "sqpyr", name: "square pyramidal numbers", alias: r"square pyramidal(?: numbers?)?", def: "n(n + 1)(2n + 1)/6", kind: Formula(|k| b(k * (k + 1) * (2 * k + 1) / 6), 1) },
            Seq { key: "pentatope", name: "pentatope numbers", alias: r"pentatope(?: numbers?)?", def: "n(n + 1)(n + 2)(n + 3)/24", kind: Formula(|k| b(k * (k + 1) * (k + 2) * (k + 3) / 24), 1) },
            Seq { key: "pentagonal", name: "pentagonal numbers", alias: r"pentagonal(?: numbers?)?", def: "n(3n − 1)/2", kind: Formula(|k| b(k * (3 * k - 1) / 2), 1) },
            Seq { key: "hexagonal", name: "hexagonal numbers", alias: r"hexagonal(?: numbers?)?", def: "n(2n − 1)", kind: Formula(|k| b(k * (2 * k - 1)), 1) },
            Seq { key: "pronic", name: "pronic numbers", alias: r"(?:pronic|oblong)(?: numbers?)?", def: "n(n + 1)", kind: Formula(|k| b(k * (k + 1)), 1) },
            Seq { key: "star", name: "star numbers", alias: r"star numbers?", def: "6n(n − 1) + 1", kind: Formula(|k| b(6 * k * (k - 1) + 1), 1) },
            Seq { key: "square", name: "square numbers", alias: r"squares|square numbers?|perfect squares?", def: "n²", kind: Formula(|k| b(k * k), 1) },
            Seq { key: "cube", name: "cube numbers", alias: r"cubes|cube numbers?|perfect cubes?", def: "n³", kind: Formula(|k| b(k * k * k), 1) },
            Seq { key: "fibonacci", name: "Fibonacci numbers", alias: r"fibonacci(?: numbers?)?", def: "F(1) = F(2) = 1, F(n) = F(n−1) + F(n−2)", kind: Formula(nt::fib, 1) },
            Seq { key: "lucas", name: "Lucas numbers", alias: r"lucas(?: numbers?)?", def: "L(0) = 2, L(1) = 1, L(n) = L(n−1) + L(n−2); listed here from L(1)", kind: Formula(nt::lucas, 1) },
            Seq { key: "pell", name: "Pell numbers", alias: r"pell(?: numbers?)?", def: "P(0) = 0, P(1) = 1, P(n) = 2P(n−1) + P(n−2)", kind: Formula(pell, 1) },
            Seq { key: "jacobsthal", name: "Jacobsthal numbers", alias: r"jacobsthal(?: numbers?)?", def: "J(n) = (2^n − (−1)^n)/3", kind: Formula(|k| ((BigInt::one() << k as usize) - if k % 2 == 0 { 1 } else { -1 }) / 3, 1) },
            Seq { key: "catalan", name: "Catalan numbers", alias: r"catalan(?: numbers?)?", def: "C(n) = binomial(2n, n)/(n + 1), n ≥ 0", kind: Formula(|k| nt::binomial(&b(2 * k), k) / b(k + 1), 0) },
            Seq { key: "bell", name: "Bell numbers", alias: r"bell(?: numbers?)?", def: "B(n): partitions of an n-element set, n ≥ 0", kind: Formula(bell, 0) },
            Seq { key: "factorial", name: "factorials", alias: r"factorials?(?: numbers?)?", def: "n! for n ≥ 1", kind: Formula(nt::factorial, 1) },
            Seq { key: "pow2", name: "powers of 2", alias: r"powers? of (?:2|two)", def: "2^n for n ≥ 0", kind: Formula(|k| BigInt::one() << k as usize, 0) },
            Seq { key: "even", name: "even numbers", alias: r"even(?: numbers?| integers?)?|evens", def: "2n", kind: Formula(|k| b(2 * k), 1) },
            Seq { key: "odd", name: "odd numbers", alias: r"odd(?: numbers?| integers?)?|odds", def: "2n − 1", kind: Formula(|k| b(2 * k - 1), 1) },
            Seq { key: "integer", name: "positive integers", alias: r"(?:positive )?integers?|natural numbers?|whole numbers?|numbers", def: "1, 2, 3, …", kind: Formula(b, 1) },
        ]
    })
}

fn seq_alt() -> &'static str {
    static S: OnceLock<String> = OnceLock::new();
    S.get_or_init(|| seqs().iter().map(|s| format!("(?:{})", s.alias)).collect::<Vec<_>>().join("|"))
}

fn find_seq(s: &str) -> Option<&'static Seq> {
    let s = s.trim().trim_start_matches("the ").trim();
    seqs().iter().find(|q| Regex::new(&format!("^(?:{})$", q.alias)).map_or(false, |r| r.is_match(s)))
}

/// Inclusive range and how many terms; `from_top` takes the largest ones.
#[derive(Default, Clone)]
struct Sel {
    lo: Option<BigInt>,
    hi: Option<BigInt>,
    count: Option<usize>,
    from_top: bool,
    form: Option<(u64, u64)>,
}

const SCAN_CAP: u64 = 30_000_000;

fn mersenne_value(s: &Seq, idx: usize) -> Option<(BigInt, String)> {
    let Kind::Mersenne(v, sym) = &s.kind else { return None };
    let p = *MERSENNE.get(idx)?;
    if p > 25_000 {
        return Some((BigInt::from(-1), sym(p)));
    }
    Some((v(p), sym(p)))
}

/// Terms within the selection, ascending. `None` when the work would be unbounded.
fn terms(s: &Seq, sel: &Sel) -> Option<Vec<BigInt>> {
    let lo = sel.lo.clone().unwrap_or_else(BigInt::one);
    let want = |v: &BigInt| -> bool { *v >= lo && sel.hi.as_ref().map_or(true, |h| v <= h) && sel.form.map_or(true, |(a, c)| (v % BigInt::from(a)) == BigInt::from(c % a)) };
    let cap = if sel.from_top { usize::MAX } else { sel.count.unwrap_or(usize::MAX) };
    let mut out: Vec<BigInt> = Vec::new();
    if sel.hi.is_none() && sel.count.is_none() {
        return None;
    }
    match &s.kind {
        Kind::Formula(f, start) => {
            let mut k = *start;
            loop {
                let v = f(k);
                if v < BigInt::zero() {
                    break;
                }
                if sel.hi.as_ref().map_or(false, |h| &v > h) {
                    break;
                }
                if want(&v) {
                    out.push(v);
                    if out.len() >= cap {
                        break;
                    }
                }
                k += 1;
                if k > 2_000_000 {
                    return None;
                }
            }
        }
        Kind::Mersenne(..) => {
            for i in 0..MERSENNE.len() {
                let (v, _) = mersenne_value(s, i)?;
                if v < BigInt::zero() {
                    return None;
                }
                if sel.hi.as_ref().map_or(false, |h| &v > h) {
                    break;
                }
                if want(&v) {
                    out.push(v);
                    if out.len() >= cap {
                        break;
                    }
                }
            }
        }
        Kind::Known(list, _) => out = list.iter().map(|&v| b(v)).filter(|v| want(v)).take(cap).collect(),
        Kind::Filter(t) => {
            let lo64 = lo.to_u64()?.max(1);
            if sel.from_top {
                let hi = sel.hi.as_ref()?.to_u64()?;
                let need = sel.count.unwrap_or(usize::MAX);
                let mut n = hi;
                while n >= lo64 && out.len() < need {
                    if hi - n > SCAN_CAP {
                        return None;
                    }
                    if t(n) && want(&b(n)) {
                        out.push(b(n));
                    }
                    n -= 1;
                }
                out.reverse();
                return Some(out);
            }
            let hi = sel.hi.as_ref().map(|h| h.to_u64().unwrap_or(u64::MAX)).unwrap_or(u64::MAX);
            let mut n = lo64;
            while n <= hi {
                if n - lo64 > SCAN_CAP {
                    return None;
                }
                if t(n) && want(&b(n)) {
                    out.push(b(n));
                    if out.len() >= cap {
                        break;
                    }
                }
                n += 1;
            }
        }
    }
    if sel.from_top {
        let k = sel.count.unwrap_or(out.len()).min(out.len());
        out = out.split_off(out.len() - k);
    }
    Some(out)
}

fn member(s: &Seq, n: &BigInt) -> Option<bool> {
    if *n < BigInt::one() {
        return Some(false);
    }
    match &s.kind {
        Kind::Filter(t) => n.to_u64().map(t),
        Kind::Known(list, _) => Some(n.to_u64().map_or(false, |v| list.contains(&v))),
        _ => Some(terms(s, &Sel { hi: Some(n.clone()), ..Default::default() })?.contains(n)),
    }
}

fn show(v: &BigInt) -> String {
    let s = v.to_string();
    if s.len() > 60 {
        format!("{}…{} ({} digits)", &s[..20], &s[s.len() - 10..], s.len())
    } else {
        s
    }
}

fn show_term(s: &Seq, v: &BigInt) -> String {
    if s.key == "twin" {
        format!("({}, {})", v, v + 2)
    } else {
        show(v)
    }
}

fn show_list(s: &Seq, v: &[BigInt]) -> String {
    if v.is_empty() {
        return "(none)".into();
    }
    if v.len() > 120 {
        let head: Vec<_> = v[..60].iter().map(|x| show_term(s, x)).collect();
        let tail: Vec<_> = v[v.len() - 10..].iter().map(|x| show_term(s, x)).collect();
        return format!("{}, …, {}", head.join(", "), tail.join(", "));
    }
    v.iter().map(|x| show_term(s, x)).collect::<Vec<_>>().join(", ")
}

fn words_to_digits(l: &str) -> String {
    const ORD: &[(&str, &str)] = &[
        ("first", "1st"), ("second", "2nd"), ("third", "3rd"), ("fourth", "4th"), ("fifth", "5th"), ("sixth", "6th"), ("seventh", "7th"), ("eighth", "8th"), ("ninth", "9th"), ("tenth", "10th"), ("eleventh", "11th"), ("twelfth", "12th"), ("twentieth", "20th"), ("hundredth", "100th"), ("thousandth", "1000th"), ("millionth", "1000000th"),
    ];
    const CARD: &[(&str, &str)] = &[("one", "1"), ("two", "2"), ("three", "3"), ("four", "4"), ("five", "5"), ("six", "6"), ("seven", "7"), ("eight", "8"), ("nine", "9"), ("ten", "10"), ("eleven", "11"), ("twelve", "12"), ("fifteen", "15"), ("twenty", "20"), ("fifty", "50"), ("hundred", "100")];
    let mut out = Vec::new();
    let toks: Vec<&str> = l.split(' ').collect();
    for (i, t) in toks.iter().enumerate() {
        // "first five" keeps "first"; a lone "first" before a sequence name means 1st
        let next_is_num = toks.get(i + 1).map_or(false, |n| n.chars().next().map_or(false, |c| c.is_ascii_digit()) || CARD.iter().any(|(w, _)| w == n));
        if let Some((_, d)) = ORD.iter().find(|(w, _)| w == t) {
            // "first five …" keeps "first"; "fourth Lucas number" becomes "4th …"
            out.push(if *t == "first" && next_is_num { t.to_string() } else { d.to_string() });
        } else if let Some((_, d)) = CARD.iter().find(|(w, _)| w == t) {
            out.push(d.to_string());
        } else {
            out.push(t.to_string());
        }
    }
    out.join(" ")
}

fn num(s: &str) -> Option<BigInt> {
    let s = s.trim();
    if let Some(m) = re!(r"^(\d+)\s*\^\s*(\d+)$").captures(s) {
        return Some(m[1].parse::<BigInt>().ok()?.pow(m[2].parse::<u32>().ok().filter(|e| *e < 4000)?));
    }
    if let Some(m) = re!(r"^(\d+(?:\.\d+)?)\s*(million|billion|thousand)$").captures(s) {
        let f: f64 = m[1].parse().ok()?;
        let mul = match &m[2] { "thousand" => 1e3, "million" => 1e6, _ => 1e9 };
        return Some(BigInt::from((f * mul).round() as u64));
    }
    s.parse().ok()
}

/// Parse a selection phrase around a sequence name: returns (sequence, selection).
fn selection(t: &str) -> Option<(&'static Seq, Sel)> {
    let alt = seq_alt();
    let n = r"(\d+(?:\s*\^\s*\d+)?(?:\s+(?:million|billion|thousand))?)";
    let lt = r"(?:<|less than|below|under|smaller than)";
    let le = r"(?:<=|≤|up to|at most|not exceeding|not greater than)";
    let gt = r"(?:>|greater than|above|over|exceeding|larger than|bigger than|after)";
    let ge = r"(?:>=|≥|at least|from)";
    let bound = format!(r"(?:\s+(?:that are\s+|which are\s+)?(?:({lt})|({le})|({gt})|({ge}))\s*{n})?(?:\s+(?:and|but)\s+(?:({lt})|({le})|({gt})|({ge}))\s*{n})?(?:\s+(?:in\s+)?(?:between|from)\s+{n}\s+(?:and|to)\s+{n})?(?:\s+(?:of the form|congruent to|≡)\s+(\d+)\s*[kn]\s*\+\s*(\d+))?");
    let lead = r"^(?:the\s+)?(?:(\d+)(?:st|nd|rd|th)\s+|(?:first|smallest|least|lowest)\s+(\d+)\s+|(\d+)\s+(?:greatest|largest|biggest|highest|last)\s+|(\d+)\s+(?:smallest|least|lowest)\s+|(?:last|greatest|largest|biggest|highest)\s+(\d+)\s+|(\d+)\s+)?";
    let pat = format!(r"{lead}((?:{alt})){bound}$");
    let r = {
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new(&pat).unwrap())
    };
    let m = r.captures(t)?;
    let s = find_seq(&m[7])?;
    let mut sel = Sel::default();
    let mut nth = None;
    if let Some(x) = m.get(1) {
        nth = Some(x.as_str().parse::<usize>().ok()?);
    }
    if let Some(x) = m.get(2).or(m.get(4)).or(m.get(6)) {
        sel.count = Some(x.as_str().parse().ok()?);
    }
    if let Some(x) = m.get(3).or(m.get(5)) {
        sel.count = Some(x.as_str().parse().ok()?);
        sel.from_top = true;
    }
    for base in [8, 13] {
        if let Some(v) = m.get(base + 4) {
            let v = num(v.as_str())?;
            if m.get(base).is_some() {
                sel.hi = Some(v - 1);
            } else if m.get(base + 1).is_some() {
                sel.hi = Some(v);
            } else if m.get(base + 2).is_some() {
                sel.lo = Some(v + 1);
            } else {
                sel.lo = Some(v);
            }
        }
    }
    if let (Some(a), Some(c)) = (m.get(18), m.get(19)) {
        let (a, c) = (num(a.as_str())?, num(c.as_str())?);
        sel.lo = Some(a.clone().min(c.clone()));
        sel.hi = Some(a.max(c));
    }
    if let (Some(a), Some(c)) = (m.get(20), m.get(21)) {
        sel.form = Some((a.as_str().parse().ok().filter(|a: &u64| *a > 0)?, c.as_str().parse().ok()?));
    }
    if nth == Some(0) {
        return None;
    }
    Some((s, sel))
}

fn input_pod(s: &str) -> Pod {
    pod("Input", "Input interpretation", text(s.to_string()))
}

fn describe_sel(s: &Seq, sel: &Sel) -> String {
    let mut parts = vec![s.name.to_string()];
    if let Some((a, c)) = sel.form {
        parts.push(format!("of the form {a}k + {c}"));
    }
    match (&sel.lo, &sel.hi) {
        (Some(a), Some(h)) => parts.push(format!("from {a} to {h}")),
        (Some(a), None) => parts.push(format!("≥ {a}")),
        (None, Some(h)) => parts.push(format!("≤ {h}")),
        _ => {}
    }
    if let Some(c) = sel.count {
        parts.insert(0, format!("{} {c}", if sel.from_top { "largest" } else { "first" }));
    }
    parts.join(" ")
}

fn nth_answer(s: &'static Seq, k: usize, sel: &Sel, raw: &str) -> Option<Answer> {
    let plain = sel.lo.is_none() && sel.hi.is_none() && sel.form.is_none();
    // direct formulas avoid listing
    let (val, sym): (Option<BigInt>, Option<String>) = match &s.kind {
        Kind::Mersenne(..) if plain => {
            let Some((v, sy)) = mersenne_value(s, k - 1) else {
                return Some(Answer::new(s.name, vec![input_pod(raw), primary(pod("Result", "Result", text(format!("only {} {} are known", MERSENNE.len(), s.name))))]));
            };
            ((v >= BigInt::zero()).then_some(v), Some(sy))
        }
        Kind::Known(list, note) if plain => match list.get(k - 1) {
            Some(v) => (Some(b(*v)), None),
            None => return Some(Answer::new(s.name, vec![input_pod(raw), primary(pod("Result", "Result", text(note.to_string())))])),
        },
        Kind::Formula(f, start) if plain => (Some(f(start + k as u64 - 1)), None),
        Kind::Filter(_) if plain && s.key == "prime" => (nt::nth_prime(k as u64).map(b), None),
        _ => (terms(s, &Sel { count: Some(k), ..sel.clone() })?.get(k - 1).cloned(), None),
    };
    let ord = match k % 100 {
        11..=13 => "th",
        _ => match k % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    let label = format!("{k}{ord} of the {}", describe_sel(s, &Sel { count: None, ..sel.clone() }));
    let mut pods = vec![input_pod(&label)];
    let res = match (&val, &sym) {
        (Some(v), Some(sy)) => format!("{} = {}", sy, show_term(s, v)),
        (Some(v), None) => show_term(s, v),
        (None, Some(sy)) => sy.clone(),
        (None, None) => return None,
    };
    pods.push(primary(pod("Result", "Result", text(res))));
    if let Some(v) = &val {
        let digits = v.to_string().len();
        if digits > 20 {
            pods.push(pod("Digits", "Number of decimal digits", text(digits.to_string())));
        }
    } else if let (Kind::Mersenne(..), Some(&p)) = (&s.kind, MERSENNE.get(k - 1)) {
        let d = (p as f64 * 2f64.log10()).floor() as u64 + 1;
        pods.push(pod("Digits", "Number of decimal digits", text(format!("about {d}"))));
    }
    pods.push(pod("Def", "Definition", text(s.def)));
    Some(Answer::new(s.name, pods))
}

fn list_answer(s: &'static Seq, sel: &Sel, agg: Option<&str>) -> Option<Answer> {
    let v = terms(s, sel)?;
    let desc = describe_sel(s, sel);
    let mut pods = vec![input_pod(&match agg {
        Some(a) => format!("{a} of the {desc}"),
        None => desc.clone(),
    })];
    match agg {
        Some("sum") => {
            let t: BigInt = v.iter().sum();
            pods.push(primary(pod("Result", "Result", text(show(&t)))));
            pods.push(pod("Terms", "Terms", text(show_list(s, &v))));
        }
        Some("product") => {
            let t: BigInt = v.iter().product();
            pods.push(primary(pod("Result", "Result", text(show(&t)))));
            pods.push(pod("Terms", "Terms", text(show_list(s, &v))));
            if t.to_string().len() > 12 {
                pods.push(pod("Decimal", "Decimal approximation", text(crate::stats::fmt(t.to_f64().unwrap_or(f64::INFINITY)))));
            }
        }
        Some("number") => {
            pods.push(primary(pod("Result", "Result", text(v.len().to_string()))));
            pods.push(pod("Terms", "Terms", text(show_list(s, &v))));
        }
        Some(_) => {
            let t: BigInt = v.iter().sum();
            let n = v.len().max(1);
            let r = num_rational::BigRational::new(t, BigInt::from(n));
            pods.push(primary(pod("Result", "Mean", text(if r.is_integer() { r.to_integer().to_string() } else { format!("{} ≈ {}", r, crate::stats::fmt(r.to_f64().unwrap_or(0.0))) }))));
            pods.push(pod("Terms", "Terms", text(show_list(s, &v))));
        }
        None => {
            pods.push(primary(pod("Result", "Result", text(show_list(s, &v)))));
            pods.push(pod("Count", "Number of terms", text(v.len().to_string())));
            if let Some((a, c)) = sel.form {
                if s.key == "prime" && a == 4 && c == 1 {
                    pods.push(pod("Note", "Fermat's theorem", text("a prime is a sum of two squares exactly when it is 2 or of the form 4k + 1")));
                }
            }
        }
    }
    pods.push(pod("Def", "Definition", text(s.def)));
    Some(Answer::new(s.name, pods))
}

fn overview(s: &'static Seq) -> Option<Answer> {
    let v = terms(s, &Sel { count: Some(if matches!(s.kind, Kind::Mersenne(..)) { 8 } else { 20 }), ..Default::default() })?;
    let mut pods = vec![input_pod(s.name), primary(pod("Result", "First terms", text(show_list(s, &v) + ", …")))];
    pods.push(pod("Def", "Definition", text(s.def)));
    Some(Answer::new(s.name, pods))
}

fn membership(n: &BigInt, s: &'static Seq, negate_q: bool) -> Option<Answer> {
    let yes = member(s, n)?;
    let label = format!("{n} ∈ {}", s.name);
    let mut pods = vec![input_pod(&label), primary(pod("Result", "Result", text(if yes ^ negate_q { "True" } else { "False" })))];
    let why = match s.key {
        "abundant" | "deficient" | "perfect" => n.to_u64().map(|v| format!("σ({v}) = {}, compared with 2·{v} = {}", sigma(v), 2 * v)),
        "superperfect" => n.to_u64().map(|v| format!("σ({v}) = {}, σ(σ({v})) = {}, compared with 2·{v} = {}", sigma(v), sigma(sigma(v)), 2 * v)),
        _ => None,
    };
    if let Some(w) = why {
        pods.push(pod("Why", "Check", text(w)));
    }
    pods.push(pod("Def", "Definition", text(s.def)));
    Some(Answer::new(s.name, pods))
}

/// "integers that are not deficient and are tetrahedral in between 10 and 20000"
fn compound(t: &str) -> Option<Answer> {
    let m = re!(r"^(?:integers|numbers|positive integers)\s+(?:that|which)\s+are\s+(not\s+)?(.+?)\s+(?:and|but)\s+(?:are\s+)?(not\s+)?(.+?)(?:\s+(?:in\s+)?between\s+(\d+)\s+and\s+(\d+)|\s+(?:less than|below|under|<)\s+(\d+))?$").captures(t)?;
    let a = find_seq(m[2].trim_end_matches(" numbers").trim_end_matches(" number"))?;
    let c = find_seq(m[4].trim_end_matches(" numbers").trim_end_matches(" number"))?;
    let (na, nc) = (m.get(1).is_some(), m.get(3).is_some());
    let mut sel = Sel::default();
    if let (Some(x), Some(y)) = (m.get(5), m.get(6)) {
        sel.lo = Some(num(x.as_str())?);
        sel.hi = Some(num(y.as_str())?);
    } else if let Some(x) = m.get(7) {
        sel.hi = Some(num(x.as_str())? - 1);
    } else {
        sel.hi = Some(b(1000));
    }
    // generate from a non-negated side
    let (gen, other, neg_other) = if !na { (a, c, nc) } else if !nc { (c, a, na) } else { return None };
    let base = terms(gen, &sel)?;
    let v: Vec<BigInt> = base.into_iter().filter(|x| member(other, x).map_or(false, |y| y != neg_other)).collect();
    let desc = format!("{} that are {}{} between {} and {}", gen.name, if neg_other { "not " } else { "" }, other.name, sel.lo.clone().unwrap_or_else(BigInt::one), sel.hi.clone()?);
    Some(Answer::new("integers", vec![input_pod(&desc), primary(pod("Result", "Result", text(show_list(gen, &v)))), pod("Count", "Number of terms", text(v.len().to_string()))]))
}

fn divisor_phrases(t: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:list\s+)?(?:the\s+)?divisors\s+(\d+)$").captures(t) {
        let n: BigInt = m[1].parse().ok()?;
        if n.is_zero() {
            return None;
        }
        let ds = nt::divisors(&n);
        return Some(Answer::new("divisors", vec![input_pod(&format!("divisors of {n}")), primary(pod("Result", "Result", text(ds.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")))), pod("Count", "Number of divisors", text(ds.len().to_string())), pod("Sum", "Sum of divisors", text(ds.iter().sum::<BigInt>().to_string()))]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?common (divisors|factors|multiples) of (\d+)(?:\s*,\s*|\s+and\s+|\s+)(\d+)(?:(?:\s*,\s*|\s+and\s+)(\d+))?$").captures(t) {
        let nums: Vec<BigInt> = [m.get(2), m.get(3), m.get(4)].iter().flatten().filter_map(|x| x.as_str().parse().ok()).collect();
        if nums.iter().any(|x| x.is_zero()) {
            return None;
        }
        if &m[1] == "multiples" {
            let l = nums.iter().fold(BigInt::one(), |a, x| a.lcm(x));
            let list: Vec<String> = (1..=10u32).map(|k| (&l * BigInt::from(k)).to_string()).collect();
            return Some(Answer::new("common multiples", vec![input_pod(&format!("common multiples of {}", nums.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))), primary(pod("Result", "Result", text(format!("{}, … (the multiples of lcm = {l})", list.join(", ")))))]));
        }
        let g = nums.iter().fold(BigInt::zero(), |a, x| a.gcd(x));
        let ds = nt::divisors(&g);
        return Some(Answer::new("common divisors", vec![input_pod(&format!("common divisors of {}", nums.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))), primary(pod("Result", "Result", text(ds.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")))), pod("Gcd", "Greatest common divisor", text(g.to_string())), pod("Count", "Number of common divisors", text(ds.len().to_string()))]));
    }
    if let Some(m) = re!(r"^(?:add up|sum|total)\s+(?:the\s+)?digits\s+(?:of|in)\s+(\d+)(?:\s+in\s+base\s+(\d+))?$|^(?:the\s+)?(?:sum|total) of (?:the\s+)?digits\s+(?:of|in)\s+(\d+)\s+in\s+base\s+(\d+)$").captures(t) {
        let n: BigInt = m.get(1).or(m.get(3))?.as_str().parse().ok()?;
        let base: u32 = m.get(2).or(m.get(4)).map_or(Some(10), |x| x.as_str().parse().ok())?;
        if !(2..=36).contains(&base) {
            return None;
        }
        let digits = nt::to_base(&n, base);
        return Some(Answer::new("digit sum", vec![input_pod(&format!("digit sum of {n} in base {base}")), primary(pod("Result", "Result", text(nt::digit_sum(&n, base).to_string()))), pod("Digits", &format!("Digits in base {base}"), text(digits))]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:prime|primes)\s+(closest|nearest|next|previous|before|after)\s+(?:to\s+)?(\d+)$|^(?:the\s+)?(closest|nearest|next|previous) prime (?:to|after|before|from)\s+(\d+)$").captures(t) {
        let n: BigInt = m.get(2).or(m.get(4))?.as_str().parse().ok()?;
        let how = m.get(1).or(m.get(3))?.as_str();
        let next = nt::next_prime(&(&n - 1));
        let prev = nt::prev_prime(&(&n + 1));
        let res = match how {
            "next" | "after" => nt::next_prime(&n),
            "previous" | "before" => nt::prev_prime(&n)?,
            _ => match prev {
                Some(p) if &n - &p <= &next - &n => p,
                _ => next,
            },
        };
        return Some(Answer::new("prime", vec![input_pod(&format!("{how} prime to {n}")), primary(pod("Result", "Result", text(res.to_string()))), pod("Dist", "Distance", text((&res - &n).to_string()))]));
    }
    // Pascal's triangle: rows, or mod m
    if let Some(m) = re!(r"^(?:(?:the\s+)?(?:first\s+)?(\d+)\s+rows\s+of\s+)?pascal'?s triangle(?:\s+(?:mod(?:ulo)?|modulo)\s+(\d+))?(?:\s+(\d+)\s+rows)?$").captures(t) {
        let modulus: Option<u64> = m.get(2).and_then(|x| x.as_str().parse().ok()).filter(|x| *x >= 2);
        let rows: usize = m.get(1).or(m.get(3)).and_then(|x| x.as_str().parse().ok()).unwrap_or(if modulus.is_some() { 16 } else { 8 }).min(if modulus.is_some() { 64 } else { 30 });
        let mut row = vec![BigInt::one()];
        let mut lines = Vec::new();
        for _ in 0..rows {
            let shown: Vec<String> = row.iter().map(|x| match modulus {
                Some(md) => (x % BigInt::from(md)).to_string(),
                None => x.to_string(),
            }).collect();
            lines.push(shown.join(" "));
            let mut nx = vec![BigInt::one()];
            for w in row.windows(2) {
                nx.push(&w[0] + &w[1]);
            }
            nx.push(BigInt::one());
            row = nx;
        }
        let width = lines.last().map_or(0, |l| l.len());
        let centred: Vec<String> = lines.iter().map(|l| format!("{}{l}", " ".repeat((width - l.len()) / 2))).collect();
        let title = match modulus {
            Some(md) => format!("Pascal's triangle mod {md}, rows 0 to {}", rows - 1),
            None => format!("Pascal's triangle, rows 0 to {}", rows - 1),
        };
        let mut pods = vec![input_pod(&title), primary(pod("Result", "Result", text(centred.join("\n"))))];
        if modulus == Some(2) {
            pods.push(pod("Note", "Note", text("the odd entries form the Sierpiński triangle")));
        } else if let Some(p) = modulus.filter(|p| nt::is_prime_u64(*p)) {
            pods.push(pod("Note", "Lucas's theorem", text(format!("C(n, k) mod {p} is the product of C(nᵢ, kᵢ) over the base-{p} digits of n and k"))));
        }
        return Some(Answer::new("Pascal's triangle", pods));
    }
    None
}

pub fn handle(q: &Query) -> Option<Answer> {
    let raw = q.clean.trim().trim_end_matches(['?', '.', '!']);
    let mut l = raw.to_lowercase();
    // 1,000,000 → 1000000
    loop {
        let next = re!(r"(\d),(\d{3})(\D|$)").replace_all(&l, "$1$2$3").to_string();
        if next == l {
            break;
        }
        l = next;
    }
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = words_to_digits(&l);
    let l = l.trim_start_matches("show ").trim_start_matches("list ").trim_start_matches("find ").trim_start_matches("compute ").trim_start_matches("what is the ").trim_start_matches("what are the ").to_string();
    // "Fibonacci sequence", "sequence of Fibonacci numbers"
    let l = match re!(r"^(?:the\s+)?(?:sequence|list) of (?:the\s+)?(.+)$|^(?:the\s+)?([a-z' -]+?) (?:sequence|numbers sequence)$").captures(&l) {
        Some(m) if m.get(1).is_some() => m[1].to_string(),
        Some(m) => format!("{} numbers", &m[2]),
        None => l,
    };
    if let Some(a) = divisor_phrases(&l) {
        return Some(a);
    }
    if let Some(a) = compound(&l) {
        return Some(a);
    }
    let alt = seq_alt();
    // membership
    let mem = {
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new(&format!(r"^(?:is\s+(\d+)\s+(not\s+)?(?:a\s+|an\s+)?(?:member of\s+|element of\s+|in\s+)?(?:the\s+)?(?:set of\s+)?({alt})|does the (?:set|sequence|list) of ({alt}) (?:contain|include) (\d+)|(\d+) (?:∈|in) (?:the\s+)?({alt}))$")).unwrap())
    };
    if let Some(m) = mem.captures(&l) {
        let n: BigInt = m.get(1).or(m.get(5)).or(m.get(6))?.as_str().parse().ok()?;
        let s = find_seq(m.get(3).or(m.get(4)).or(m.get(7))?.as_str())?;
        if s.key == "integer" {
            return None;
        }
        return membership(&n, s, m.get(2).is_some());
    }
    // aggregates
    if let Some(m) = re!(r"^(sum|product|number|count|mean|average|total) of (?:all\s+)?(?:the\s+)?(.+)$").captures(&l) {
        let agg = match &m[1] {
            "sum" | "total" => "sum",
            "product" => "product",
            "number" | "count" => "number",
            _ => "mean",
        };
        let (s, sel) = selection(&m[2])?;
        if s.key == "integer" && sel.form.is_none() {
            return None;
        }
        return list_answer(s, &sel, Some(agg));
    }
    let (s, sel) = selection(&l)?;
    let nth = re!(r"^(?:the\s+)?(\d+)(?:st|nd|rd|th)\s").captures(&l).and_then(|m| m[1].parse::<usize>().ok());
    if s.key == "integer" && sel.form.is_none() {
        return None;
    }
    if let Some(k) = nth {
        if k > 100_000_000 {
            return None;
        }
        return nth_answer(s, k, &Sel { count: None, ..sel }, raw);
    }
    if sel.lo.is_none() && sel.hi.is_none() && sel.count.is_none() {
        if sel.form.is_some() {
            return list_answer(s, &Sel { count: Some(30), ..sel }, None);
        }
        // bare name: only when clearly a set ("… numbers", plural)
        if !(l.ends_with('s') || l.ends_with("number")) {
            return None;
        }
        return overview(s);
    }
    list_answer(s, &sel, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ans(s: &str) -> String {
        serde_json::to_string(&crate::engine::query(s).pods).unwrap_or_default()
    }

    #[test]
    fn integer_sequences() {
        assert!(ans("fourth lucas number > 1000000").contains("4870847"));
        assert!(ans("sum of first five square triangular numbers").contains("1456599"));
        assert!(ans("is 4 a member of the superperfect numbers?").contains("True"));
        assert!(ans("Does the set of perfect numbers contain 18?").contains("False"));
        assert!(ans("1000th twin prime").contains("(79559, 79561)"));
        assert!(ans("4th Fermat prime").contains("257"));
        assert!(ans("20 greatest triangular numbers < 500").contains("496"));
        assert!(ans("integers that are not deficient and are tetrahedral in between 10 and 20000").contains("20, 56, 84, 120"));
        assert!(ans("common divisors of 360 and 96").contains("1, 2, 3, 4, 6, 8, 12, 24"));
        assert!(ans("sum of digits of 29225 in base 2").contains("111001000101001"));
        assert!(ans("prime closest to 169743212304").contains("169743212"));
    }
}
