//! Polynomials over the prime field GF(p): factorisation (square-free,
//! distinct-degree and Cantor–Zassenhaus equal-degree splitting),
//! irreducibility, and the cyclic codes of length n over GF(p).
//!
//! "factor x^36-1 mod 2", "factor x^4+1 over GF(3)", "is x^4+x+1 irreducible
//! over GF(2)", "cyclic code 36, 2", "binary cyclic codes of length 7".

use super::nl::{ex, main_var};
use super::*;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// Coefficients mod p, lowest degree first, without trailing zeros.
type P = Vec<u64>;

fn trim(mut a: P) -> P {
    while a.last() == Some(&0) {
        a.pop();
    }
    a
}

fn deg(a: &P) -> usize {
    a.len().saturating_sub(1)
}

fn mulm(a: u64, b: u64, p: u64) -> u64 {
    ((a as u128 * b as u128) % p as u128) as u64
}

fn powm(mut a: u64, mut e: u64, p: u64) -> u64 {
    let mut r = 1 % p;
    a %= p;
    while e > 0 {
        if e & 1 == 1 {
            r = mulm(r, a, p);
        }
        a = mulm(a, a, p);
        e >>= 1;
    }
    r
}

fn inv(a: u64, p: u64) -> u64 {
    powm(a, p - 2, p)
}

fn sub(a: &P, b: &P, p: u64) -> P {
    let n = a.len().max(b.len());
    trim((0..n).map(|i| (a.get(i).copied().unwrap_or(0) + p - b.get(i).copied().unwrap_or(0)) % p).collect())
}

fn add(a: &P, b: &P, p: u64) -> P {
    let n = a.len().max(b.len());
    trim((0..n).map(|i| (a.get(i).copied().unwrap_or(0) + b.get(i).copied().unwrap_or(0)) % p).collect())
}

fn mul(a: &P, b: &P, p: u64) -> P {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let mut r = vec![0u64; a.len() + b.len() - 1];
    for (i, x) in a.iter().enumerate() {
        if *x == 0 {
            continue;
        }
        for (j, y) in b.iter().enumerate() {
            r[i + j] = (r[i + j] + mulm(*x, *y, p)) % p;
        }
    }
    trim(r)
}

fn divrem(a: &P, b: &P, p: u64) -> (P, P) {
    let mut r = a.clone();
    if b.is_empty() || a.len() < b.len() {
        return (vec![], r);
    }
    let il = inv(*b.last().unwrap(), p);
    let mut q = vec![0u64; a.len() - b.len() + 1];
    while r.len() >= b.len() && !r.is_empty() {
        let s = r.len() - b.len();
        let c = mulm(*r.last().unwrap(), il, p);
        q[s] = c;
        for (j, y) in b.iter().enumerate() {
            r[s + j] = (r[s + j] + p - mulm(c, *y, p)) % p;
        }
        r = trim(r);
    }
    (trim(q), r)
}

fn rem(a: &P, b: &P, p: u64) -> P {
    divrem(a, b, p).1
}

fn monic(a: &P, p: u64) -> P {
    match a.last() {
        Some(l) => {
            let i = inv(*l, p);
            a.iter().map(|c| mulm(*c, i, p)).collect()
        }
        None => vec![],
    }
}

fn gcd(a: &P, b: &P, p: u64) -> P {
    let (mut a, mut b) = (a.clone(), b.clone());
    while !b.is_empty() {
        let r = rem(&a, &b, p);
        a = b;
        b = r;
    }
    monic(&a, p)
}

fn powmod(a: &P, e: &BigUint, f: &P, p: u64) -> P {
    let mut r: P = vec![1];
    let mut b = rem(a, f, p);
    for i in 0..e.bits() {
        if e.bit(i) {
            r = rem(&mul(&r, &b, p), f, p);
        }
        b = rem(&mul(&b, &b, p), f, p);
    }
    r
}

fn deriv(a: &P, p: u64) -> P {
    trim(a.iter().enumerate().skip(1).map(|(i, c)| mulm(*c, i as u64 % p, p)).collect())
}

fn is_one(a: &P) -> bool {
    a.len() == 1 && a[0] == 1
}

/// The p-th root of a polynomial whose derivative vanishes (only powers x^(kp)).
fn pth_root(a: &P, p: u64) -> P {
    a.iter().step_by(p as usize).copied().collect()
}

/// Square-free factorisation: (square-free part, multiplicity).
fn sqf(f: &P, p: u64) -> Vec<(P, u32)> {
    let mut out = Vec::new();
    let g = deriv(f, p);
    if g.is_empty() {
        for (h, m) in sqf(&pth_root(f, p), p) {
            out.push((h, m * p as u32));
        }
        return out;
    }
    let mut c = gcd(f, &g, p);
    let mut w = divrem(f, &c, p).0;
    let mut i = 1;
    while !is_one(&w) {
        let y = gcd(&w, &c, p);
        let fac = divrem(&w, &y, p).0;
        if !is_one(&fac) {
            out.push((monic(&fac, p), i));
        }
        w = y;
        c = divrem(&c, &w, p).0;
        i += 1;
    }
    if !is_one(&c) && !c.is_empty() {
        for (h, m) in sqf(&pth_root(&c, p), p) {
            out.push((h, m * p as u32));
        }
    }
    out
}

/// Distinct-degree factorisation of a square-free monic polynomial.
fn ddf(f: &P, p: u64) -> Vec<(P, usize)> {
    let mut out = Vec::new();
    let mut fs = f.clone();
    let x: P = vec![0, 1];
    let mut h = x.clone();
    let pe = BigUint::from(p);
    let mut i = 1;
    while deg(&fs) >= 2 * i {
        h = powmod(&h, &pe, &fs, p);
        let g = gcd(&fs, &sub(&h, &x, p), p);
        if !is_one(&g) {
            out.push((g.clone(), i));
            fs = divrem(&fs, &g, p).0;
            h = rem(&h, &fs, p);
        }
        i += 1;
    }
    if deg(&fs) > 0 {
        let d = deg(&fs);
        out.push((fs, d));
    }
    out
}

/// Split a product of distinct irreducibles of degree d (Cantor–Zassenhaus).
fn edf(f: &P, d: usize, p: u64, seed: &mut u64) -> Vec<P> {
    let n = deg(f);
    if n <= d {
        return vec![f.clone()];
    }
    let mut rnd = || {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*seed >> 33) % p
    };
    let e = (BigUint::from(p).pow(d as u32) - 1u32) / 2u32;
    for _ in 0..200 {
        let a: P = trim((0..n).map(|_| rnd()).collect());
        if deg(&a) == 0 {
            continue;
        }
        let g = if p == 2 {
            // the trace a + a² + a⁴ + … + a^(2^(d−1))
            let mut t = rem(&a, f, p);
            let mut s = t.clone();
            for _ in 1..d {
                t = rem(&mul(&t, &t, p), f, p);
                s = add(&s, &t, p);
            }
            gcd(f, &s, p)
        } else {
            let b = powmod(&a, &e, f, p);
            gcd(f, &sub(&b, &vec![1], p), p)
        };
        if deg(&g) > 0 && deg(&g) < n {
            let h = divrem(f, &g, p).0;
            let mut out = edf(&monic(&g, p), d, p, seed);
            out.extend(edf(&monic(&h, p), d, p, seed));
            return out;
        }
    }
    vec![f.clone()]
}

/// Monic irreducible factors with multiplicities, sorted by degree then coefficients.
fn factor(f: &P, p: u64) -> Vec<(P, u32)> {
    let f = monic(f, p);
    let mut seed = 0x9e3779b97f4a7c15u64;
    let mut out: Vec<(P, u32)> = Vec::new();
    for (s, m) in sqf(&f, p) {
        for (g, d) in ddf(&s, p) {
            for h in edf(&g, d, p, &mut seed) {
                match out.iter_mut().find(|x| x.0 == h) {
                    Some(x) => x.1 += m,
                    None => out.push((h, m)),
                }
            }
        }
    }
    out.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then_with(|| a.0.iter().rev().cmp(b.0.iter().rev())));
    out
}

fn show(a: &P, x: &str) -> String {
    if a.is_empty() {
        return "0".into();
    }
    let mut parts = Vec::new();
    for (i, c) in a.iter().enumerate().rev() {
        if *c == 0 {
            continue;
        }
        let pw = match i {
            0 => String::new(),
            1 => x.to_string(),
            _ => format!("{x}^{i}"),
        };
        parts.push(match (i, *c) {
            (0, c) => c.to_string(),
            (_, 1) => pw,
            (_, c) => format!("{c} {pw}"),
        });
    }
    parts.join(" + ")
}

fn show_factors(fs: &[(P, u32)], lead: u64, x: &str) -> String {
    let mut s: Vec<String> = Vec::new();
    if lead != 1 {
        s.push(lead.to_string());
    }
    for (h, m) in fs {
        let b = if fs.len() == 1 && lead == 1 && *m == 1 { show(h, x) } else { format!("({})", show(h, x)) };
        s.push(if *m > 1 { format!("{b}^{m}") } else { b });
    }
    s.join(" ")
}

fn is_prime(p: u64) -> bool {
    p >= 2 && (2..).take_while(|d| d * d <= p).all(|d| p % d != 0)
}

/// Parse a polynomial in one variable and reduce it mod p.
fn parse_poly(s: &str, p: u64) -> Option<(P, String)> {
    let e = ex(s)?;
    if e.free_symbols().len() != 1 {
        return None;
    }
    let x = main_var(&e);
    let q = crate::poly::to_qpoly(&e, &x)?;
    let pb = num_bigint::BigInt::from(p);
    let mut c = Vec::new();
    for r in &q.0 {
        let n = (r.numer() % &pb + &pb) % &pb;
        let d = (r.denom() % &pb + &pb) % &pb;
        let (n, d) = (n.to_u64()?, d.to_u64()?);
        if d == 0 {
            return None;
        }
        c.push(mulm(n, inv(d, p), p));
    }
    let c = trim(c);
    if deg(&c) < 1 {
        return None;
    }
    Some((c, crate::print::text(&x)))
}

fn field_name(p: u64) -> String {
    format!("GF({p})")
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.raw.trim().trim_end_matches(['?', '.']).trim();
    let field = r"(?:mod(?:ulo)?\s+(\d+)|over\s+(?:the\s+)?(?:(?:galois\s+)?field\s+)?(?:gf\s*\(\s*(\d+)\s*\)|gf\s*(\d+)|f_?\s*(\d+)|z\s*/\s*(\d+)\s*z?|zz\s*/\s*(\d+)|(?:the\s+)?(?:integers|field)\s+mod(?:ulo)?\s+(\d+)))";
    let pick = |m: &regex::Captures| (2..=8).find_map(|i| m.get(i)).and_then(|x| x.as_str().parse::<u64>().ok());
    // cyclic codes
    if let Some(m) = re!(r"(?i)^(binary\s+|ternary\s+)?cyclic\s+codes?\s+(?:of\s+)?(?:length\s+)?(?:n\s*=\s*)?(\d+)(?:\s*,\s*(?:q\s*=\s*)?(\d+)|\s+over\s+(?:gf\s*\(\s*(\d+)\s*\)|gf\s*(\d+)|f_?(\d+)))?$").captures(l) {
        let n: usize = m[2].parse().ok()?;
        let p: u64 = match (m.get(1).map(|x| x.as_str().trim().to_lowercase()), (3..=6).find_map(|i| m.get(i))) {
            (_, Some(x)) => x.as_str().parse().ok()?,
            (Some(b), None) if b == "ternary" => 3,
            _ => 2,
        };
        return cyclic_codes(n, p);
    }
    // irreducibility
    if let Some(m) = Regex::new(&format!(r"(?i)^is\s+(.+?)\s+(?:irreducible|prime)\s+{field}$")).ok()?.captures(l) {
        let p = pick(&m)?;
        if !is_prime(p) || p > 1 << 31 {
            return None;
        }
        let (f, x) = parse_poly(&m[1], p)?;
        let fs = factor(&f, p);
        let irr = fs.len() == 1 && fs[0].1 == 1;
        let lead = *f.last()?;
        return Some(Answer::new(format!("irreducibility over {}", field_name(p)), vec![
            pod("Input", "Input interpretation", text(format!("{} irreducible over {}?", show(&f, &x), field_name(p)))),
            primary(pod("Result", "Result", text(if irr { "yes, irreducible" } else { "no, reducible" }))),
            pod("Factor", &format!("Factorization over {}", field_name(p)), text(show_factors(&fs, lead, &x))),
        ]));
    }
    // factorisation
    let m = Regex::new(&format!(r"(?i)^(?:factor|factori[sz]e|factorization\s+of|factorisation\s+of)\s+(.+?)\s*,?\s+{field}$")).ok()?.captures(l)?;
    let p = pick(&m)?;
    if !is_prime(p) || p > 1 << 31 {
        return None;
    }
    let (f, x) = parse_poly(&m[1], p)?;
    if deg(&f) > 400 {
        return None;
    }
    let fs = factor(&f, p);
    let lead = *f.last()?;
    let mut rows = Vec::new();
    for (h, mlt) in &fs {
        rows.push(vec![text(show(h, &x)), text(deg(h).to_string()), text(mlt.to_string())]);
    }
    Some(Answer::new(format!("factor over {}", field_name(p)), vec![
        pod("Input", "Input interpretation", text(format!("factor {} over {}", show(&f, &x), field_name(p)))),
        primary(pod("Result", "Result", text(show_factors(&fs, lead, &x)))),
        pod("Factors", "Irreducible factors", table(Some(vec!["factor", "degree", "multiplicity"]), rows)),
    ]))
}

fn cyclic_codes(n: usize, p: u64) -> Option<Answer> {
    if n < 1 || n > 2000 || !is_prime(p) || p > 1000 {
        return None;
    }
    let mut f = vec![0u64; n + 1];
    f[0] = p - 1;
    f[n] = 1;
    let fs = factor(&f, p);
    let count: u128 = fs.iter().map(|(_, m)| *m as u128 + 1).product();
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("cyclic codes of length {n} over {}", field_name(p)))),
        primary(pod("Result", "Number of cyclic codes", text(count.to_string()))),
        pod("Factor", &format!("Factorization of x^{n} − 1 over {}", field_name(p)), text(show_factors(&fs, 1, "x"))),
    ];
    // every code is generated by a monic divisor g of x^n − 1; dimension n − deg g
    if count <= 4096 {
        let mut gens: Vec<(P, usize)> = vec![(vec![1], 0)];
        for (h, m) in &fs {
            let mut next = Vec::new();
            for (g, dg) in &gens {
                let mut t = g.clone();
                for e in 0..=*m {
                    next.push((t.clone(), dg + e as usize * deg(h)));
                    t = mul(&t, h, p);
                }
            }
            gens = next;
        }
        gens.sort_by_key(|g| g.1);
        let mut by_dim: std::collections::BTreeMap<usize, usize> = Default::default();
        for (_, dg) in &gens {
            *by_dim.entry(n - dg).or_default() += 1;
        }
        if gens.len() <= 32 {
            let rows = gens.iter().map(|(g, dg)| vec![text(show(g, "x")), text(format!("[{n}, {}]", n - dg))]).collect();
            pods.push(pod("Codes", "Generator polynomials", table(Some(vec!["generator polynomial", "[n, k]"]), rows)));
        } else {
            let rows = by_dim.iter().rev().map(|(k, c)| vec![text(k.to_string()), text(c.to_string())]).collect();
            pods.push(pod("Codes", "Codes by dimension", table(Some(vec!["dimension k", "number of codes"]), rows)));
        }
    }
    pods.push(pod("Note", "Note", text(format!("Cyclic codes of length n over GF(q) correspond to the monic divisors g(x) of x^n − 1; a code generated by g has dimension n − deg g. With x^n − 1 = Π fᵢ^eᵢ there are Π (eᵢ + 1) of them, including the zero code and the whole space."))));
    Some(Answer::new(format!("cyclic codes {n} over GF({p})"), pods))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factors_mod_p() {
        let f36 = { let mut f = vec![0u64; 37]; f[0] = 1; f[36] = 1; f };
        let fs = factor(&f36, 2);
        assert_eq!(show_factors(&fs, 1, "x"), "(x + 1)^4 (x^2 + x + 1)^4 (x^6 + x^3 + 1)^4");
        // x^4 + 1 over GF(3) = (x^2 + x + 2)(x^2 + 2x + 2)
        let fs = factor(&vec![1, 0, 0, 0, 1], 3);
        assert_eq!(fs.len(), 2);
        assert!(fs.iter().all(|(h, m)| deg(h) == 2 && *m == 1));
        // x^8 − x over GF(2) is the product of all irreducibles of degree 1 and 3
        let fs = factor(&vec![0, 1, 0, 0, 0, 0, 0, 0, 1], 2);
        assert_eq!(fs.iter().map(|(h, _)| deg(h)).collect::<Vec<_>>(), vec![1, 1, 3, 3]);
        // x^5 - 1 over GF(11) splits into linear factors
        let fs = factor(&vec![10, 0, 0, 0, 0, 1], 11);
        assert_eq!(fs.len(), 5);
    }
}
