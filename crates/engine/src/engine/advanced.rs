//! Advanced topics asked in words: modular equations and number theory,
//! special functions, matrix decompositions and exponentials, systems of
//! linear ODEs, linear recurrences and generating functions, integral and
//! discrete transforms, and residues.

use super::nl::{again, answer, caps, decimal_pod, ex, split_top};
use super::nl_math::{g, mat_of};
use super::*;
use crate::calculus::diff::diff;
use crate::calculus::integrate::{definite, integrate, Definite};
use crate::calculus::limit::{limit, Dir};
use crate::expr::Expr::*;
use crate::expr::*;
use crate::linalg::{self, Mat};
use crate::numtheory as nt;
use crate::poly;
use crate::print::{fmt_float, text as ptext};
use crate::simplify as s;
use crate::solve;
use num_bigint::BigInt;
use num_traits::{One, ToPrimitive, Zero};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn z(e: &Expr) -> bool {
    s::simplify(e).is_zero()
}

// ---------------------------------------------------------------------------
// number theory
// ---------------------------------------------------------------------------

fn modpow(mut b: i128, mut e: u64, m: i128) -> i128 {
    let mut r = 1i128 % m;
    b = b.rem_euclid(m);
    while e > 0 {
        if e & 1 == 1 {
            r = r * b % m;
        }
        b = b * b % m;
        e >>= 1;
    }
    r
}

/// Integer polynomial coefficients (lowest first), scaled to clear denominators.
fn int_coeffs(e: &Expr, x: &Expr) -> Option<Vec<i128>> {
    let p = poly::to_qpoly(&s::expand(e), x)?;
    let den = p.0.iter().fold(BigInt::one(), |l, r| num_integer::Integer::lcm(&l, r.denom()));
    p.0.iter().map(|r| (r.numer() * (&den / r.denom())).to_i128()).collect()
}

fn congruence(lhs: &str, rhs: &str, modulus: i128) -> Option<Answer> {
    let e = sub(ex(lhs)?, ex(rhs)?);
    let vars = e.free_symbols();
    if vars.len() != 1 || !(2..=2_000_000).contains(&modulus) {
        return None;
    }
    let x = sym(vars.iter().next()?);
    let cs = int_coeffs(&e, &x)?;
    let sols: Vec<i128> = (0..modulus).filter(|v| {
        let mut acc = 0i128;
        for c in cs.iter().rev() {
            acc = (acc * v + c).rem_euclid(modulus);
        }
        acc == 0
    }).collect();
    let xn = ptext(&x);
    let shown = if sols.is_empty() { "no solutions".to_string() } else { sols.iter().map(|v| format!("{xn} ≡ {v}")).collect::<Vec<_>>().join(",  ") + &format!("  (mod {modulus})") };
    let label = format!("solve {} ≡ {} (mod {modulus})", lhs.trim(), rhs.trim());
    let mut pods = vec![primary(pod("Result", "Solutions", text(shown)))];
    pods.push(pod("Count", "Number of solutions", text(format!("{} of {} residues", sols.len(), modulus))));
    if cs.len() == 2 {
        let (b, a) = (cs[0], cs[1]);
        let gcd = num_integer::Integer::gcd(&a.rem_euclid(modulus), &modulus);
        pods.push(pod("Method", "Method", text(format!("a linear congruence a·{xn} ≡ b has solutions exactly when gcd(a, m) = {gcd} divides b; then there are {gcd} of them. Here a = {a}, b = {}.", -b))));
    } else {
        pods.push(pod("Method", "Method", text("every residue class was tested")));
    }
    answer(label.clone(), label, pods)
}

pub fn number_theory(q: &Query, l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    let _ = q;
    // polynomial congruences
    if let Some(m) = cap(re!(r"^(?:solve\s+)?(.+?)\s*(?:\(\s*mod(?:ulo)?\s+(\d+)\s*\)|,?\s+mod(?:ulo)?\s+(\d+))$")) {
        let body = g(&m, 1);
        let modulus: i128 = if g(&m, 2).is_empty() { g(&m, 3).parse().ok()? } else { g(&m, 2).parse().ok()? };
        if let Some((lhs, rhs)) = body.split_once('=').or_else(|| body.split_once('≡')) {
            if let Some(a) = congruence(lhs, rhs, modulus) {
                return Some(a);
            }
        }
    }
    // modular inverse
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:modular\s+|multiplicative\s+)?inverse\s+of\s+(-?\d+)\s+(?:mod(?:ulo)?|in\s+z_?)\s*(\d+)$")) {
        let (a, md): (BigInt, BigInt) = (g(&m, 1).parse().ok()?, g(&m, 2).parse().ok()?);
        let label = format!("inverse of {a} modulo {md}");
        return match nt::mod_inv(&a, &md) {
            Some(v) => answer(label.clone(), label, vec![primary(pod("Result", "Modular inverse", text(v.to_string()))), pod("Check", "Check", text(format!("{a} × {v} = {} ≡ 1 (mod {md})", &a * &v)))]),
            None => answer(label.clone(), label, vec![primary(pod("Result", "Result", text(format!("{a} has no inverse modulo {md}: they share the factor {}", num_integer::Integer::gcd(&a, &md)))))]),
        };
    }
    // primitive roots
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:smallest\s+)?primitive roots?\s+(?:of|mod(?:ulo)?)\s+(\d+)$")) {
        let p: BigInt = g(&m, 1).parse().ok()?;
        let label = format!("primitive roots modulo {p}");
        let Some(r) = nt::primitive_root(&p) else {
            return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(format!("{p} is not prime here; primitive roots are only listed for primes"))))]);
        };
        let pv = p.to_u64()?;
        let mut all: Vec<u64> = Vec::new();
        if pv < 100_000 {
            let phi = pv - 1;
            let fs: Vec<u64> = nt::factor(&BigInt::from(phi)).iter().filter_map(|(f, _)| f.to_u64()).collect();
            for gg in 2..pv {
                if fs.iter().all(|f| modpow(gg as i128, phi / f, pv as i128) != 1) {
                    all.push(gg);
                    if all.len() >= 40 {
                        break;
                    }
                }
            }
        }
        let mut pods = vec![primary(pod("Result", "Smallest primitive root", text(r.to_string())))];
        if !all.is_empty() {
            pods.push(pod("All", "Primitive roots", text(all.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "))));
            pods.push(pod("Count", "Number of primitive roots", text(format!("φ(p − 1) = {}", nt::totient(&(p.clone() - 1u32))))));
        }
        return answer(label.clone(), label, pods);
    }
    // Pell equations
    if let Some(m) = cap(re!(r"^(?:solve\s+)?(?:the\s+)?(?:pell(?:'s)? equation\s*:?\s*)?x\^2\s*-\s*(\d+)\s*\*?\s*y\^2\s*=\s*1$")).filter(|_| l.contains("pell") || l.contains("integer")) {
        let d: u64 = g(&m, 1).parse().ok()?;
        let r = (d as f64).sqrt() as u64;
        if r * r == d || d > 10_000_000 {
            return None;
        }
        // continued fraction of √d gives the fundamental solution
        let (mut mm, mut dd, mut a) = (0i128, 1i128, r as i128);
        let a0 = a;
        let (mut h1, mut h0) = (BigInt::one(), BigInt::from(a0));
        let (mut k1, mut k0) = (BigInt::zero(), BigInt::one());
        let di = d as i128;
        for _ in 0..10_000 {
            if &h0 * &h0 - BigInt::from(d) * &k0 * &k0 == BigInt::one() {
                break;
            }
            mm = dd * a - mm;
            dd = (di - mm * mm) / dd;
            a = (a0 + mm) / dd;
            let h2 = BigInt::from(a) * &h0 + &h1;
            let k2 = BigInt::from(a) * &k0 + &k1;
            h1 = std::mem::replace(&mut h0, h2);
            k1 = std::mem::replace(&mut k0, k2);
        }
        let (x1, y1) = (h0.clone(), k0.clone());
        let mut rows = Vec::new();
        let (mut xk, mut yk) = (x1.clone(), y1.clone());
        for k in 1..=4 {
            rows.push(vec![text(k.to_string()), text(xk.to_string()), text(yk.to_string())]);
            let nx = &xk * &x1 + BigInt::from(d) * &yk * &y1;
            let ny = &xk * &y1 + &yk * &x1;
            xk = nx;
            yk = ny;
        }
        let label = format!("Pell equation x² − {d} y² = 1");
        return answer(label.clone(), label, vec![
            primary(pod("Result", "Fundamental solution", text(format!("x = {x1}, y = {y1}")))),
            pod("More", "Further solutions", table(Some(vec!["k", "x", "y"]), rows)),
            pod("Method", "Method", text(format!("from the continued fraction of √{d}; all positive solutions are given by x + y√{d} = ({x1} + {y1}√{d})^k"))),
        ]);
    }
    // partitions
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:number of\s+)?(?:integer\s+)?partitions\s+of\s+(\d+)$")) {
        let n: usize = g(&m, 1).parse().ok()?;
        if n > 20_000 {
            return None;
        }
        let p = nt::partitions(n);
        let label = format!("number of partitions of {n}");
        let mut pods = vec![primary(pod("Result", "Number of partitions p(n)", text(p.to_string())))];
        if n <= 12 {
            fn gen(n: usize, max: usize, cur: &mut Vec<usize>, out: &mut Vec<String>) {
                if n == 0 {
                    out.push(cur.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" + "));
                    return;
                }
                for k in (1..=n.min(max)).rev() {
                    cur.push(k);
                    gen(n - k, k, cur, out);
                    cur.pop();
                }
            }
            let mut out = Vec::new();
            gen(n, n, &mut vec![], &mut out);
            pods.push(pod("List", "Partitions", text(out.join("\n"))));
        }
        let hr = (std::f64::consts::PI * (2.0 * n as f64 / 3.0).sqrt()).exp() / (4.0 * n as f64 * 3f64.sqrt());
        pods.push(pod("Asymptotic", "Hardy–Ramanujan estimate", text(format!("e^(π√(2n/3)) / (4n√3) ≈ {}", fmt_float(hr, 6)))));
        return answer(label.clone(), label, pods);
    }
    // Collatz
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:collatz|3n\s*\+\s*1|hailstone)(?:\s+(?:sequence|steps|conjecture))?\s+(?:of|for|starting (?:at|from))?\s*(\d+)$")) {
        let n: BigInt = g(&m, 1).parse().ok()?;
        let steps = nt::collatz_steps(&n)?;
        let mut seq = vec![n.clone()];
        let mut v = n.clone();
        while !v.is_one() && seq.len() < 300 {
            v = if (&v % 2u32).is_zero() { v / 2u32 } else { v * 3u32 + 1u32 };
            seq.push(v.clone());
        }
        let peak = seq.iter().max()?.clone();
        let label = format!("Collatz sequence starting at {n}");
        let pts: Vec<[f64; 2]> = seq.iter().enumerate().filter_map(|(k, v)| Some([k as f64, v.to_f64()?])).collect();
        let mut pods = vec![primary(pod("Result", "Steps to reach 1", text(steps.to_string()))), pod("Peak", "Largest value", text(peak.to_string()))];
        pods.push(pod("Sequence", "Sequence", text(seq.iter().take(120).map(|v| v.to_string()).collect::<Vec<_>>().join(", ") + if seq.len() > 120 { ", …" } else { "" })));
        pods.push(pod("Plot", "Trajectory", plot(crate::plot::scatter(pts, "value", None))));
        return answer(label.clone(), label, pods);
    }
    // "perfect square 144"
    if let Some(m) = cap(re!(r"^(?:is\s+)?(?:perfect (square|cube))\s+(\d+)\??$")) {
        return again(q, &format!("is {} a perfect {}", g(&m, 2), g(&m, 1)));
    }
    None
}

// ---------------------------------------------------------------------------
// special functions
// ---------------------------------------------------------------------------

fn hyp2f1(a: f64, b: f64, c: f64, z: f64) -> Option<f64> {
    if z.abs() >= 1.0 {
        return None;
    }
    let (mut term, mut sum) = (1.0, 1.0);
    for n in 0..5000 {
        let nf = n as f64;
        term *= (a + nf) * (b + nf) / ((c + nf) * (nf + 1.0)) * z;
        sum += term;
        if term.abs() < 1e-17 * sum.abs() {
            break;
        }
    }
    sum.is_finite().then_some(sum)
}

pub fn special(q: &Query, l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    // phrasing rewrites onto the built-in functions
    if let Some(m) = cap(re!(r"^bessel\s*([jyik])\s*_?\s*(\d+)\s*\(\s*(.+?)\s*\)$")) {
        return again(q, &format!("bessel{}({}, {})", g(&m, 1).to_lowercase(), g(&m, 2), g(&m, 3)));
    }
    if let Some(m) = cap(re!(r"^bessel\s*([jyik])\s*\(\s*(.+?)\s*\)$")) {
        return again(q, &format!("bessel{}({})", g(&m, 1).to_lowercase(), g(&m, 2)));
    }
    if let Some(m) = cap(re!(r"^(?:lambert\s*w|product\s*log|productlog|w)\s*\(\s*(.+?)\s*\)$")).filter(|_| l.contains("lambert") || l.contains("product")) {
        return again(q, &format!("W({})", g(&m, 1)));
    }
    if let Some(m) = cap(re!(r"^airy\s*(ai|bi)\s*\(\s*(.+?)\s*\)$")) {
        return again(q, &format!("airy{}({})", g(&m, 1).to_lowercase(), g(&m, 2)));
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:riemann\s+)?zeta(?:\s+function)?\s+(?:at|of)\s+(.+)$")) {
        return again(q, &format!("zeta({})", g(&m, 1)));
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:gamma)\s+function\s+(?:at|of)\s+(.+)$")) {
        return again(q, &format!("gamma({})", g(&m, 1)));
    }
    // complete elliptic integrals (parameter m)
    if let Some(m) = cap(re!(r"^(?:complete\s+)?elliptic\s*(?:integral\s*)?(?:of the (first|second) kind\s*)?(k|e)?\s*\(\s*(.+?)\s*\)$")) {
        let which = if g(&m, 1) == "second" || g(&m, 2).to_lowercase() == "e" { "E" } else { "K" };
        let mv = ex(&g(&m, 3))?.to_f64_const()?;
        if mv >= 1.0 {
            return None;
        }
        let (k, e) = crate::special::elliptic_ke(mv);
        let v = if which == "K" { k } else { e };
        let label = format!("complete elliptic integral {which}({}) (parameter m)", g(&m, 3));
        return answer(label.clone(), label, vec![
            primary(pod("Result", "Result", text(fmt_float(v, 15)))),
            pod("Both", "Both kinds", table(None, vec![vec![text("K(m) = ∫₀^(π/2) dθ / √(1 − m sin²θ)"), text(fmt_float(k, 12))], vec![text("E(m) = ∫₀^(π/2) √(1 − m sin²θ) dθ"), text(fmt_float(e, 12))]])),
        ]);
    }
    // Gauss hypergeometric function
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:gauss\s+)?hypergeometric(?:\s+function)?\s*(?:2f1)?\s*\(\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,]+)\)$")) {
        let vals: Option<Vec<f64>> = (1..=4).map(|k| ex(&g(&m, k)).and_then(|e| e.to_f64_const())).collect();
        let v = vals?;
        let r = hyp2f1(v[0], v[1], v[2], v[3])?;
        let label = format!("₂F₁({}, {}; {}; {})", g(&m, 1), g(&m, 2), g(&m, 3), g(&m, 4));
        return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(fmt_float(r, 15)))), pod("Series", "Definition", text("Σ (a)ₙ (b)ₙ / ((c)ₙ n!) zⁿ, evaluated by its power series (|z| < 1)"))]);
    }
    // Stirling
    if l.contains("gamma") && (l.contains("asymptotic") || l.contains("stirling")) {
        return answer("asymptotic expansion of the gamma function", "asymptotic expansion of Γ(x) as x → ∞", vec![
            primary(pod("Result", "Stirling series", text("Γ(x) ~ √(2π/x) (x/e)^x (1 + 1/(12x) + 1/(288x²) − 139/(51840x³) − 571/(2488320x⁴) + …)"))),
            pod("Log", "For log Γ", text("log Γ(x) ~ (x − ½) log x − x + ½ log(2π) + 1/(12x) − 1/(360x³) + 1/(1260x⁵) − …")),
            pod("Factorial", "Stirling's formula", text("n! ~ √(2πn) (n/e)^n")),
        ]);
    }
    None
}

// ---------------------------------------------------------------------------
// matrices
// ---------------------------------------------------------------------------

/// e^(A t) for a 2×2 matrix, in real form.
fn expm2(a: &Mat, t: &Expr) -> Mat {
    let tr = s::simplify(&add(vec![a[0][0].clone(), a[1][1].clone()]));
    let det = linalg::det(a);
    let half = s::simplify(&div(tr.clone(), int(2)));
    let delta = s::simplify(&sub(square(half.clone()), det));
    let shifted: Mat = (0..2).map(|i| (0..2).map(|j| if i == j { sub(a[i][j].clone(), half.clone()) } else { a[i][j].clone() }).collect()).collect();
    let (cpart, spart) = match delta.to_f64_const() {
        Some(d) if d.abs() < 1e-15 => (one(), t.clone()),
        Some(d) if d > 0.0 => {
            let w = s::simplify(&sqrt(delta.clone()));
            (f1("cosh", mul(vec![w.clone(), t.clone()])), div(f1("sinh", mul(vec![w.clone(), t.clone()])), w))
        }
        _ => {
            let w = s::simplify(&sqrt(neg(delta.clone())));
            (f1("cos", mul(vec![w.clone(), t.clone()])), div(f1("sin", mul(vec![w.clone(), t.clone()])), w))
        }
    };
    let ex_ = exp(mul(vec![half, t.clone()]));
    (0..2)
        .map(|i| (0..2).map(|j| s::simplify(&mul(vec![ex_.clone(), add(vec![if i == j { cpart.clone() } else { zero() }, mul(vec![spart.clone(), shifted[i][j].clone()])])]))).collect())
        .collect()
}

/// e^(A t) by Putzer's algorithm when every eigenvalue is real.
fn expm_putzer(a: &Mat, t: &Expr) -> Option<Mat> {
    let n = a.len();
    let mut lams: Vec<Expr> = Vec::new();
    for (v, m) in linalg::eigenvalues(a) {
        if v.has_const(K::I) || v.to_f64_const().is_none() {
            return None;
        }
        for _ in 0..m {
            lams.push(v.clone());
        }
    }
    if lams.len() != n {
        return None;
    }
    let sv = sym("σ");
    let mut r: Vec<Expr> = vec![exp(mul(vec![lams[0].clone(), t.clone()]))];
    for k in 1..n {
        let inner = mul(vec![exp(mul(vec![lams[k].clone(), sub(t.clone(), sv.clone())])), r[k - 1].subs(t, &sv)]);
        let v = match definite(&inner, &sv, &zero(), t) {
            Definite::Exact(v) => s::simplify(&v),
            _ => return None,
        };
        r.push(v);
    }
    let ident = linalg::identity(n);
    let mut p = ident.clone();
    let mut total: Mat = vec![vec![zero(); n]; n];
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                total[i][j] = add(vec![total[i][j].clone(), mul(vec![r[k].clone(), p[i][j].clone()])]);
            }
        }
        let shifted: Mat = (0..n).map(|i| (0..n).map(|j| if i == j { sub(a[i][j].clone(), lams[k].clone()) } else { a[i][j].clone() }).collect()).collect();
        p = linalg::matmul(&p, &shifted)?;
    }
    Some(total.into_iter().map(|row| row.into_iter().map(|e| s::simplify(&e)).collect()).collect())
}

fn expm(a: &Mat, t: &Expr) -> Option<Mat> {
    if a.len() == 2 { Some(expm2(a, t)) } else { expm_putzer(a, t) }
}

fn jordan(a: &Mat) -> Option<(Mat, Vec<(Expr, usize, usize)>)> {
    let n = a.len();
    let mut blocks: Vec<(Expr, usize)> = Vec::new();
    let mut info = Vec::new();
    for (lam, alg) in linalg::eigenvalues(a) {
        let shifted: Mat = (0..n).map(|i| (0..n).map(|j| if i == j { sub(a[i][j].clone(), lam.clone()) } else { a[i][j].clone() }).collect()).collect();
        let geo = n - linalg::rank(&shifted);
        info.push((lam.clone(), alg, geo));
        // block sizes from the ranks of powers of (A − λI)
        let mut ranks = vec![n];
        let mut pw = linalg::identity(n);
        for _ in 0..alg {
            pw = linalg::matmul(&pw, &shifted)?;
            ranks.push(linalg::rank(&pw));
        }
        // number of blocks of size ≥ k is r_{k−1} − r_k
        let at_least: Vec<usize> = (1..=alg).map(|k| ranks[k - 1] - ranks[k]).collect();
        for k in 1..=alg {
            let ge = at_least[k - 1];
            let ge_next = if k < alg { at_least[k] } else { 0 };
            for _ in 0..ge.saturating_sub(ge_next) {
                blocks.push((lam.clone(), k));
            }
        }
    }
    if blocks.iter().map(|b| b.1).sum::<usize>() != n {
        return None;
    }
    let mut j: Mat = vec![vec![zero(); n]; n];
    let mut at = 0;
    for (lam, size) in &blocks {
        for k in 0..*size {
            j[at + k][at + k] = lam.clone();
            if k + 1 < *size {
                j[at + k][at + k + 1] = one();
            }
        }
        at += size;
    }
    Some((j, info))
}

pub fn matrices(q: &Query, l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    let _ = q;
    if !(l.contains("{{") || l.contains("[[")) {
        return None;
    }
    let m = cap(re!(r"^(?:the\s+)?(singular values|singular value decomposition|svd|qr decomposition|qr factorization|qr|matrix exponential|exponential|exp|jordan (?:normal |canonical )?form|jordan decomposition|matrix norm|frobenius norm|spectral norm|condition number|2-norm)\s+(?:of\s+)?(.+)$"))?;
    let a = mat_of(&g(&m, 2))?;
    let (rows, cols) = (a.len(), a[0].len());
    let op = g(&m, 1).to_lowercase();
    let label = format!("{op} of {}", ptext(&linalg::to_expr(&a)));
    let at_a = linalg::matmul(&linalg::transpose(&a), &a)?;
    let mut pods = Vec::new();
    match op.as_str() {
        o if o.starts_with("singular") || o == "svd" || o.contains("norm") || o.starts_with("condition") || o == "2-norm" => {
            if o == "frobenius norm" || o == "matrix norm" {
                let fro = s::simplify(&sqrt(add(a.iter().flatten().map(|e| square(e.clone())).collect())));
                pods.extend(vec![primary(pod("Result", "Frobenius norm", math(&fro)))]);
                pods.extend(decimal_pod(&fro));
            }
            let mut svs: Vec<Expr> = linalg::eigenvalues(&at_a).into_iter().flat_map(|(v, m)| std::iter::repeat(s::simplify(&sqrt(v))).take(m)).collect();
            svs.sort_by(|x, y| y.to_f64_const().partial_cmp(&x.to_f64_const()).unwrap_or(std::cmp::Ordering::Equal));
            if svs.is_empty() {
                let fa: Vec<Vec<f64>> = at_a.iter().map(|r| r.iter().map(|e| e.to_f64_const().unwrap_or(f64::NAN)).collect()).collect();
                let mut ev = linalg::sym_eigen(&fa)?;
                ev.sort_by(|x, y| y.partial_cmp(x).unwrap());
                svs = ev.into_iter().map(|v| float(v.max(0.0).sqrt())).collect();
            }
            let rows_t: Vec<Vec<Content>> = svs.iter().enumerate().map(|(k, v)| vec![text(format!("σ{}", k + 1)), math(v), text(v.to_f64_const().map(|f| fmt_float(f, 10)).unwrap_or_default())]).collect();
            let sv_pod = pod("SingularValues", "Singular values", table(None, rows_t));
            let (smax, smin) = (svs.first()?.to_f64_const()?, svs.last()?.to_f64_const()?);
            if o.starts_with("condition") {
                pods.push(primary(pod("Result", "Condition number (2-norm)", text(if smin > 1e-14 { fmt_float(smax / smin, 10) } else { "∞ (the matrix is singular)".to_string() }))));
                pods.push(sv_pod);
            } else if o == "spectral norm" || o == "2-norm" {
                pods.push(primary(pod("Result", "Spectral norm (largest singular value)", math(&svs[0]))));
                pods.push(sv_pod);
            } else if pods.is_empty() {
                pods.push(primary(sv_pod));
            } else {
                pods.push(sv_pod);
            }
            pods.push(pod("Method", "Method", text("singular values are the square roots of the eigenvalues of AᵀA")));
        }
        o if o.starts_with("qr") => {
            // Gram–Schmidt on the columns
            let colv: Vec<Vec<Expr>> = (0..cols).map(|j| a.iter().map(|r| r[j].clone()).collect()).collect();
            let mut qs: Vec<Vec<Expr>> = Vec::new();
            let mut r: Mat = vec![vec![zero(); cols]; cols];
            for (j, v) in colv.iter().enumerate() {
                let mut w = v.clone();
                for (i, qi) in qs.iter().enumerate() {
                    let d = s::simplify(&add(qi.iter().zip(v).map(|(x, y)| mul(vec![x.clone(), y.clone()])).collect()));
                    r[i][j] = d.clone();
                    w = w.iter().zip(qi).map(|(wi, qv)| s::simplify(&sub(wi.clone(), mul(vec![d.clone(), qv.clone()])))).collect();
                }
                let norm = s::simplify(&sqrt(add(w.iter().map(|e| square(e.clone())).collect())));
                if z(&norm) {
                    return answer(label.clone(), label, vec![primary(pod("Result", "Result", text("The columns are linearly dependent, so the QR factorization with an invertible R does not exist.")))]);
                }
                r[qs.len()][j] = norm.clone();
                qs.push(w.iter().map(|e| s::simplify(&div(e.clone(), norm.clone()))).collect());
            }
            let qm: Mat = (0..rows).map(|i| qs.iter().map(|qc| qc[i].clone()).collect()).collect();
            pods.push(primary(pod_multi("Result", "QR decomposition", vec![("Q (orthonormal columns)".into(), math(&linalg::to_expr(&qm))), ("R (upper triangular)".into(), math(&linalg::to_expr(&r)))])));
            pods.push(pod("Method", "Method", text("Gram–Schmidt orthonormalization of the columns of A; A = Q R")));
        }
        o if o.starts_with("jordan") => {
            if rows != cols {
                return None;
            }
            let (j, info) = jordan(&a)?;
            pods.push(primary(pod("Result", "Jordan normal form", math(&linalg::to_expr(&j)))));
            let rws: Vec<Vec<Content>> = info.iter().map(|(v, alg, geo)| vec![math(v), text(alg.to_string()), text(geo.to_string())]).collect();
            pods.push(pod("Eigen", "Eigenvalues", table(Some(vec!["eigenvalue", "algebraic multiplicity", "geometric multiplicity"]), rws)));
            let diag = info.iter().all(|(_, alg, geo)| alg == geo);
            pods.push(pod("Diagonalizable", "Diagonalizable", text(if diag { "yes — every eigenvalue has a full set of eigenvectors" } else { "no — some eigenvalue has fewer eigenvectors than its multiplicity" })));
        }
        _ => {
            if rows != cols {
                return None;
            }
            let t = sym("t");
            let e1 = expm(&a, &one());
            let et = expm(&a, &t)?;
            if let Some(e1) = e1 {
                pods.push(primary(pod("Result", "Matrix exponential e^A", math(&linalg::to_expr(&e1)))));
                pods.push(pod("Flow", "e^(A t)", math(&linalg::to_expr(&et))));
            } else {
                pods.push(primary(pod("Result", "e^(A t)", math(&linalg::to_expr(&et)))));
            }
            pods.push(pod("Method", "Method", text(if rows == 2 { "for a 2×2 matrix, e^(At) = e^(τt/2) [c(t) I + s(t) (A − τ/2 I)] with τ the trace" } else { "Putzer's algorithm from the eigenvalues" })));
        }
    }
    answer(label.clone(), label, pods)
}

// ---------------------------------------------------------------------------
// systems of linear ODEs
// ---------------------------------------------------------------------------

pub fn ode_systems(q: &Query, l: &str, c: &str) -> Option<Answer> {
    let _ = (q, l);
    let body = re!(r"(?i)^(?:solve\s+)?(?:the\s+)?(?:system\s*:?\s*)?").replace(c, "").to_string();
    let body = body.trim_matches(|ch| ch == '{' || ch == '}');
    let parts = split_top(body);
    let deq: Vec<(String, String)> = parts.iter().filter_map(|p| re!(r"^\s*([a-z])'\s*=\s*(.+)$").captures(p).map(|m| (m[1].to_string(), m[2].to_string()))).collect();
    if deq.len() < 2 || deq.len() > 4 {
        return None;
    }
    let names: Vec<String> = deq.iter().map(|(n, _)| n.clone()).collect();
    let t = if names.contains(&"t".to_string()) { sym("s") } else { sym("t") };
    let vars: Vec<Expr> = names.iter().map(|n| sym(n)).collect();
    let n = vars.len();
    let mut a: Mat = vec![vec![zero(); n]; n];
    let mut forcing = false;
    for (i, (_, rhs)) in deq.iter().enumerate() {
        let e = s::expand(&ex(rhs)?);
        let mut rest = e.clone();
        for (j, v) in vars.iter().enumerate() {
            let cs = poly::coeffs(&e, v)?;
            if cs.len() > 2 {
                return None;
            }
            a[i][j] = cs.get(1).cloned().unwrap_or_else(zero);
            if a[i][j].free_symbols().iter().any(|f| names.contains(&f.to_string()) || **f == *ptext(&t)) {
                return None;
            }
            rest = s::expand(&sub(rest, mul(vec![a[i][j].clone(), v.clone()])));
        }
        if !z(&rest) {
            forcing = true;
        }
    }
    if forcing {
        return None;
    }
    // initial conditions x(0) = …
    let mut init: Vec<Option<Expr>> = vec![None; n];
    for p in &parts {
        if let Some(m) = re!(r"^\s*([a-z])\s*\(\s*0\s*\)\s*=\s*(.+)$").captures(p) {
            if let Some(k) = names.iter().position(|nm| *nm == m[1]) {
                init[k] = ex(&m[2]);
            }
        }
    }
    let et = expm(&a, &t)?;
    let cvec: Vec<Expr> = (0..n).map(|k| init[k].clone().unwrap_or_else(|| sym(&format!("c_{}", k + 1)))).collect();
    let sol: Vec<Expr> = (0..n).map(|i| s::simplify(&add((0..n).map(|j| mul(vec![et[i][j].clone(), cvec[j].clone()])).collect()))).collect();
    let tn = ptext(&t);
    let rows: Vec<Vec<Content>> = names.iter().zip(&sol).map(|(nm, e)| vec![math(&eq(func(nm, vec![t.clone()]), e.clone()))]).collect();
    let mut pods = vec![primary(pod("Result", if init.iter().all(|v| v.is_some()) { "Solution of the initial value problem" } else { "General solution" }, table(None, rows)))];
    pods.push(pod("Matrix", "Coefficient matrix A (x′ = A x)", math(&linalg::to_expr(&a))));
    let ev = linalg::eigenvalues(&a);
    if !ev.is_empty() {
        pods.push(pod("Eigenvalues", "Eigenvalues", text(ev.iter().map(|(v, _)| ptext(v)).collect::<Vec<_>>().join(", "))));
        let reals: Vec<Option<f64>> = ev.iter().map(|(v, _)| crate::funcs::re_im(v).and_then(|(re, _)| re.to_f64_const())).collect();
        if reals.iter().all(|r| r.is_some()) {
            let mx = reals.iter().flatten().cloned().fold(f64::NEG_INFINITY, f64::max);
            let complex = ev.iter().any(|(v, _)| v.has_const(K::I));
            let kind = match (mx, complex) {
                (m, true) if m.abs() < 1e-12 => "center (closed orbits)",
                (m, true) if m < 0.0 => "stable spiral",
                (_, true) => "unstable spiral",
                (m, false) if m < 0.0 => "stable node",
                (_, false) if reals.iter().flatten().any(|r| *r < 0.0) => "saddle point (unstable)",
                _ => "unstable node",
            };
            pods.push(pod("Stability", "Equilibrium at the origin", text(kind)));
        }
    }
    pods.push(pod("Method", "Method", text(format!("x({tn}) = e^(A{tn}) x(0)"))));
    let label = format!("system {}", deq.iter().map(|(nm, r)| format!("{nm}' = {r}")).collect::<Vec<_>>().join(", "));
    answer(label.clone(), label, pods)
}

// ---------------------------------------------------------------------------
// recurrences and generating functions
// ---------------------------------------------------------------------------

pub fn recurrences(q: &Query, l: &str, c: &str) -> Option<Answer> {
    let _ = q;
    // named generating functions
    if let Some(m) = re!(r"^(?:the\s+)?(?:ordinary\s+)?generating function\s+(?:of|for)\s+(?:the\s+)?(.+?)(?:\s+numbers|\s+sequence)?$").captures(l) {
        let (name, gf, note) = match m[1].trim() {
            "fibonacci" => ("Fibonacci numbers", "x/(1 - x - x^2)", "F(n) = F(n−1) + F(n−2), F(0) = 0, F(1) = 1"),
            "lucas" => ("Lucas numbers", "(2 - x)/(1 - x - x^2)", "L(n) = L(n−1) + L(n−2), L(0) = 2, L(1) = 1"),
            "catalan" => ("Catalan numbers", "(1 - sqrt(1 - 4x))/(2x)", "C(n) = binomial(2n, n)/(n + 1)"),
            "natural" | "natural numbers" | "positive integers" => ("natural numbers 1, 2, 3, …", "x/(1 - x)^2", "aₙ = n"),
            "squares" | "square" => ("squares", "x (1 + x)/(1 - x)^3", "aₙ = n²"),
            "powers of 2" | "powers of two" => ("powers of 2", "1/(1 - 2x)", "aₙ = 2ⁿ"),
            "triangular" => ("triangular numbers", "x/(1 - x)^3", "aₙ = n(n + 1)/2"),
            "pell" => ("Pell numbers", "x/(1 - 2x - x^2)", "P(n) = 2P(n−1) + P(n−2), P(0) = 0, P(1) = 1"),
            _ => return None,
        };
        let e = ex(gf)?;
        let ser = crate::calculus::series::series(&e, &sym("x"), &zero(), 10);
        let mut pods = vec![primary(pod("Result", "Generating function", math(&e))), pod("Sequence", "Definition", text(note))];
        if let Some(se) = ser {
            let shown = match &se.big_o {
                Some(o) => add(vec![se.poly.clone(), func("O", vec![o.clone()])]),
                None => se.poly.clone(),
            };
            pods.push(pod("Series", "Series expansion", math(&shown)));
        }
        return answer(format!("generating function of the {name}"), format!("generating function of the {name}"), pods);
    }
    // linear recurrences with constant coefficients
    let body = re!(r"(?i)^(?:solve\s+)?(?:the\s+)?(?:recurrence(?:\s+relation)?|difference equation|rsolve)?\s*:?\s*").replace(c, "").to_string();
    let parts = split_top(&body);
    let main = parts.first()?;
    let m = re!(r"^\s*([a-zA-Z])\s*[(\[]\s*n\s*[)\]]\s*=\s*(.+)$").captures(main)?;
    let fname = m[1].to_string();
    let rhs_text = m[2].to_string();
    // a(n-1) → placeholders
    let mut order = 0usize;
    let pat = Regex::new(&format!(r"{}\s*[(\[]\s*n\s*-\s*(\d+)\s*[)\]]", regex::escape(&fname))).ok()?;
    let replaced = pat.replace_all(&rhs_text, |cp: &regex::Captures| {
        let k: usize = cp[1].parse().unwrap_or(0);
        order = order.max(k);
        format!("(r_{k})")
    }).to_string();
    if order == 0 || order > 6 || !l.contains('(') {
        return None;
    }
    let e = s::expand(&ex(&replaced)?);
    let mut coef = vec![zero(); order + 1];
    let mut rest = e.clone();
    for k in 1..=order {
        let v = sym(&format!("r_{k}"));
        let cs = poly::coeffs(&e, &v)?;
        if cs.len() > 2 {
            return None;
        }
        coef[k] = cs.get(1).cloned().unwrap_or_else(zero);
        rest = s::expand(&sub(rest, mul(vec![coef[k].clone(), v])));
    }
    if coef.iter().any(|c| c.has_sym(&sym("n"))) || !z(&rest) {
        return None;
    }
    // initial values
    let mut init: Vec<(i64, Expr)> = Vec::new();
    let ip = Regex::new(&format!(r"^\s*{}\s*[(\[]\s*(\d+)\s*[)\]]\s*=\s*(.+)$", regex::escape(&fname))).ok()?;
    for p in parts.iter().skip(1) {
        if let Some(cp) = ip.captures(p) {
            init.push((cp[1].parse().ok()?, ex(&cp[2])?));
        }
    }
    init.sort_by_key(|(k, _)| *k);
    // characteristic equation r^k = Σ c_j r^(k−j)
    let r = sym("ρ");
    let charp = s::expand(&sub(pow(r.clone(), int(order as i64)), add((1..=order).map(|j| mul(vec![coef[j].clone(), pow(r.clone(), int((order - j) as i64))])).collect())));
    let roots = solve::solve(&eq(charp.clone(), zero()), &r);
    let mut basis: Vec<Expr> = Vec::new();
    let nn = sym("n");
    let mut seen: Vec<Expr> = Vec::new();
    for root in roots.roots.iter() {
        if seen.iter().any(|x| s::equal(x, root)) {
            continue;
        }
        seen.push(root.clone());
        let mut mult = 0;
        let mut qd = charp.clone();
        while mult < order && z(&qd.subs(&r, root)) {
            mult += 1;
            qd = diff(&qd, &r);
        }
        for j in 0..mult.max(1) {
            basis.push(mul(vec![pow(nn.clone(), int(j as i64)), pow(root.clone(), nn.clone())]));
        }
    }
    if basis.len() != order {
        return None;
    }
    let consts: Vec<Expr> = (1..=order).map(|k| sym(&format!("c_{k}"))).collect();
    let general = add(basis.iter().zip(&consts).map(|(b, cst)| mul(vec![cst.clone(), b.clone()])).collect());
    let mut closed = general.clone();
    if init.len() >= order {
        let eqs: Vec<Expr> = init.iter().take(order).map(|(k, v)| sub(general.subs(&nn, &int(*k)), v.clone())).collect();
        let sol = solve::solve_system(&eqs, &consts);
        if let Some(first) = sol.sols.first() {
            closed = s::simplify(&general.subs_many(first));
        }
    }
    let mut pods = vec![primary(pod("Result", if closed == general { "General solution" } else { "Closed form" }, math(&eq(func(&fname, vec![nn.clone()]), closed.clone()))))];
    pods.push(pod("Characteristic", "Characteristic equation", math(&eq(charp, zero()))));
    // first terms by iteration
    if init.len() >= order {
        let mut vals: Vec<Expr> = init.iter().take(order).map(|(_, v)| v.clone()).collect();
        let start = init[0].0;
        while vals.len() < 15 {
            let k = vals.len();
            let next = s::simplify(&add((1..=order).map(|j| mul(vec![coef[j].clone(), vals[k - j].clone()])).collect()));
            vals.push(next);
        }
        pods.push(pod("Terms", &format!("Terms from n = {start}"), text(vals.iter().map(ptext).collect::<Vec<_>>().join(", "))));
        // generating function Σ aₙ xⁿ = P(x)/Q(x)
        if start == 0 {
            let x = sym("x");
            let qx = sub(one(), add((1..=order).map(|j| mul(vec![coef[j].clone(), pow(x.clone(), int(j as i64))])).collect()));
            let px = add((0..order).map(|i| mul(vec![sub(vals[i].clone(), add((1..=i).map(|j| mul(vec![coef[j].clone(), vals[i - j].clone()])).collect())), pow(x.clone(), int(i as i64))])).collect());
            pods.push(pod("GF", "Generating function", math(&s::simplify(&div(px, qx)))));
        }
        // the closed form must reproduce the iterated terms
        let ok = (0..10).all(|k| {
            let a = s::simplify(&closed.subs(&nn, &int(start + k as i64)));
            match (a.to_f64_const(), vals[k].to_f64_const()) {
                (Some(x), Some(y)) => (x - y).abs() < 1e-9 * (1.0 + y.abs()),
                _ => true,
            }
        });
        pods.push(pod("Check", "Verification", text(if ok { "the closed form reproduces the first terms ✓" } else { "the closed form does not match the iteration" })));
        if !ok {
            return None;
        }
    }
    let label = format!("recurrence {}", main.trim());
    answer(label.clone(), label, pods)
}

// ---------------------------------------------------------------------------
// transforms
// ---------------------------------------------------------------------------

/// Fourier transform with WolframAlpha's default convention
/// F(ω) = 1/√(2π) ∫ f(t) e^(iωt) dt, for a table of forms and their sums.
fn fourier_term(f: &Expr, x: &Expr, w: &Expr) -> Option<Expr> {
    let norm = div(one(), sqrt(mul(vec![int(2), pi()])));
    // constant multiple
    let (k, core): (Vec<Expr>, Vec<Expr>) = f.factors().into_iter().partition(|t| !t.has_sym(x));
    let k = mul(k);
    let core = mul(core);
    let res: Expr = match &core {
        // e^(−a x²)
        Pow(b, e) if matches!(&**b, Const(K::E)) => {
            let cs = poly::coeffs(&s::expand(e), x)?;
            if cs.len() != 3 || !cs[1].is_zero() || !cs[0].is_zero() {
                // e^(−a|x|) appears as a power of e with |x|
                return None;
            }
            let a = s::simplify(&neg(cs[2].clone()));
            div(exp(neg(div(square(w.clone()), mul(vec![int(4), a.clone()])))), sqrt(mul(vec![int(2), a])))
        }
        Func(n, args) if &**n == "exp" => {
            let e = &args[0];
            if let Some(cs) = poly::coeffs(&s::expand(e), x).filter(|c| c.len() == 3 && c[1].is_zero() && c[0].is_zero()) {
                let a = s::simplify(&neg(cs[2].clone()));
                div(exp(neg(div(square(w.clone()), mul(vec![int(4), a.clone()])))), sqrt(mul(vec![int(2), a])))
            } else {
                // e^(−a|x|)
                let a = s::simplify(&neg(div(e.clone(), f1("abs", x.clone()))));
                if a.has_sym(x) {
                    return None;
                }
                mul(vec![sqrt(div(int(2), pi())), div(a.clone(), add(vec![square(a), square(w.clone())]))])
            }
        }
        // 1/(x² + a²)
        Pow(b, e) if e.as_i64() == Some(-1) => {
            let cs = poly::coeffs(b, x)?;
            if cs.len() != 3 || !cs[1].is_zero() || !cs[2].is_one() {
                return None;
            }
            let a = s::simplify(&sqrt(cs[0].clone()));
            mul(vec![sqrt(div(pi(), int(2))), div(exp(neg(mul(vec![a.clone(), f1("abs", w.clone())]))), a)])
        }
        Func(n, args) if (&**n == "cos" || &**n == "sin") => {
            let a = s::simplify(&div(args[0].clone(), x.clone()));
            if a.has_sym(x) {
                return None;
            }
            let d1 = func("DiracDelta", vec![sub(w.clone(), a.clone())]);
            let d2 = func("DiracDelta", vec![add(vec![w.clone(), a.clone()])]);
            if &**n == "cos" {
                mul(vec![sqrt(div(pi(), int(2))), add(vec![d1, d2])])
            } else {
                mul(vec![i(), sqrt(div(pi(), int(2))), sub(d1, d2)])
            }
        }
        // sin(a x)/x: a box of half-width a
        Mul(v) if v.len() == 2 && v.iter().any(|t| *t == pow(x.clone(), int(-1))) && v.iter().any(|t| matches!(t, Func(n, _) if &**n == "sin")) => {
            let Some(Func(_, args)) = v.iter().find(|t| matches!(t, Func(n, _) if &**n == "sin")) else { return None };
            let a = s::simplify(&div(args[0].clone(), x.clone()));
            if a.has_sym(x) {
                return None;
            }
            let sgn = f1("sign", a.clone());
            let a = s::simplify(&f1("abs", a));
            mul(vec![sgn, sqrt(div(pi(), int(2))), f1("heaviside", sub(a, f1("abs", w.clone())))])
        }
        // sinc(a x) = sin(a x)/(a x): a box of half-width a
        Func(n, args) if &**n == "sinc" => {
            let a = s::simplify(&div(args[0].clone(), x.clone()));
            if a.has_sym(x) {
                return None;
            }
            let a = s::simplify(&f1("abs", a));
            mul(vec![div(sqrt(div(pi(), int(2))), a.clone()), f1("heaviside", sub(a, f1("abs", w.clone())))])
        }
        Func(n, _) if &**n == "DiracDelta" => norm.clone(),
        _ if core.is_one() => mul(vec![sqrt(mul(vec![int(2), pi()])), func("DiracDelta", vec![w.clone()])]),
        _ => return None,
    };
    Some(s::simplify(&mul(vec![k, res])))
}

/// Z transform of c · n^k · a^n (k ≤ 2), sin(ωn) and cos(ωn).
fn z_term(t: &Expr, n: &Expr, zz: &Expr) -> Option<Expr> {
    let (k, core): (Vec<Expr>, Vec<Expr>) = t.factors().into_iter().partition(|f| !f.has_sym(n));
    let k = mul(k);
    let mut a = one();
    let mut deg = 0i64;
    let mut trig: Option<(String, Expr)> = None;
    for f in core {
        match &f {
            Pow(b, e) if !b.has_sym(n) && s::simplify(&div((**e).clone(), n.clone())).is_one() => a = mul(vec![a, (**b).clone()]),
            Pow(b, e) if **b == *n => deg += e.as_i64()?,
            _ if f == *n => deg += 1,
            Func(name, args) if (&**name == "sin" || &**name == "cos") && trig.is_none() => {
                let w = s::simplify(&div(args[0].clone(), n.clone()));
                if w.has_sym(n) {
                    return None;
                }
                trig = Some((name.to_string(), w));
            }
            _ => return None,
        }
    }
    let a = s::simplify(&a);
    let r = match (trig, deg) {
        (None, 0) => div(zz.clone(), sub(zz.clone(), a)),
        (None, 1) => div(mul(vec![a.clone(), zz.clone()]), pow(sub(zz.clone(), a), int(2))),
        (None, 2) => div(mul(vec![a.clone(), zz.clone(), add(vec![zz.clone(), a.clone()])]), pow(sub(zz.clone(), a), int(3))),
        (None, k) if (3..=12).contains(&k) => {
            // Z{n^k aⁿ} = a z E_k(z, a)/(z − a)^(k+1), E_k with Eulerian numbers
            let k = k as usize;
            let mut eul = vec![vec![1i64]];
            for nn in 1..=k {
                let prev = &eul[nn - 1];
                let row: Vec<i64> = (0..nn).map(|m| {
                    let a1 = if m >= 1 && m - 1 < prev.len() { (nn - m) as i64 * prev[m - 1] } else { 0 };
                    let a2 = if m < prev.len() { (m + 1) as i64 * prev[m] } else { 0 };
                    a1 + a2
                }).collect();
                eul.push(row);
            }
            let row = &eul[k];
            let poly_ = add(row.iter().enumerate().map(|(m, c)| mul(vec![int(*c), pow(zz.clone(), int((k - 1 - m) as i64)), pow(a.clone(), int(m as i64))])).collect());
            div(mul(vec![a.clone(), zz.clone(), poly_]), pow(sub(zz.clone(), a), int(k as i64 + 1)))
        }
        (Some((name, w)), 0) if a.is_one() => {
            let den = add(vec![square(zz.clone()), mul(vec![int(-2), zz.clone(), f1("cos", w.clone())]), one()]);
            if name == "sin" { div(mul(vec![zz.clone(), f1("sin", w)]), den) } else { div(mul(vec![zz.clone(), sub(zz.clone(), f1("cos", w))]), den) }
        }
        _ => return None,
    };
    Some(mul(vec![k, r]))
}

pub fn transforms(q: &Query, l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    let _ = q;
    if let Some(m) = cap(re!(r"^(?:the\s+)?(inverse\s+)?(?:fourier transform|ft)\s+(?:of\s+)?(.+?)(?:\s+with respect to\s+([a-z]))?$")) {
        let inverse = !g(&m, 1).is_empty();
        let f = ex(&g(&m, 2))?;
        let x = if g(&m, 3).is_empty() { super::nl::main_var(&f) } else { sym(&g(&m, 3)) };
        let w = sym("ω");
        let terms = s::expand(&f).terms();
        let parts: Option<Vec<Expr>> = terms.iter().map(|t| fourier_term(t, &x, &w)).collect();
        let res = s::simplify(&add(parts?));
        if inverse {
            // the inverse transform is the forward one evaluated at −t
            let out = if x == sym("x") { sym("t") } else { sym("x") };
            let r = s::simplify(&res.subs(&w, &neg(out.clone())));
            let label = format!("inverse Fourier transform of {}", ptext(&f));
            return answer(label.clone(), label, vec![
                primary(pod("Result", "Inverse Fourier transform", math(&eq(func("f", vec![out]), r)))),
                pod("Convention", "Convention", text(format!("f(t) = 1/√(2π) ∫ F({0}) e^(−i{0}t) d{0} (WolframAlpha's default)", ptext(&x)))),
            ]);
        }
        let label = format!("Fourier transform of {}", ptext(&f));
        return answer(label.clone(), label, vec![
            primary(pod("Result", "Fourier transform", math(&eq(raw_fn("user:ℱ", vec![w.clone()]), res)))),
            pod("Convention", "Convention", text("F(ω) = 1/√(2π) ∫ f(t) e^(iωt) dt (WolframAlpha's default; other conventions differ by constant factors and the sign of ω)")),
        ]);
    }
    // Mellin transform: ∫₀^∞ x^(s−1) f(x) dx, from a table
    if let Some(m) = cap(re!(r"^(?:the\s+)?mellin transform\s+(?:of\s+)?(.+)$")) {
        let f = ex(&g(&m, 1))?;
        let x = super::nl::main_var(&f);
        let sv = sym("s");
        let (k, core): (Vec<Expr>, Vec<Expr>) = f.factors().into_iter().partition(|t| !t.has_sym(&x));
        let k = mul(k);
        let core = mul(core);
        let lin = |e: &Expr| -> Option<Expr> {
            let a = s::simplify(&div(e.clone(), x.clone()));
            (!a.has_sym(&x)).then_some(a)
        };
        let gam = |e: Expr| f1("gamma", e);
        let (res, strip): (Expr, &str) = match &core {
            Func(n, a) if &**n == "sin" => {
                let c = lin(&a[0])?;
                (div(mul(vec![gam(sv.clone()), f1("sin", div(mul(vec![pi(), sv.clone()]), int(2)))]), pow(c, sv.clone())), "−1 < Re s < 1")
            }
            Func(n, a) if &**n == "cos" => {
                let c = lin(&a[0])?;
                (div(mul(vec![gam(sv.clone()), f1("cos", div(mul(vec![pi(), sv.clone()]), int(2)))]), pow(c, sv.clone())), "0 < Re s < 1")
            }
            Pow(b, e) if matches!(**b, Const(K::E)) => {
                let c = lin(&neg((**e).clone())).or_else(|| None);
                match c {
                    Some(c) => (div(gam(sv.clone()), pow(c, sv.clone())), "Re s > 0"),
                    None => {
                        let cs = poly::coeffs(&s::expand(e), &x)?;
                        if cs.len() != 3 || !cs[1].is_zero() || !cs[0].is_zero() {
                            return None;
                        }
                        let c = s::simplify(&neg(cs[2].clone()));
                        (div(gam(div(sv.clone(), int(2))), mul(vec![int(2), pow(c, div(sv.clone(), int(2)))])), "Re s > 0")
                    }
                }
            }
            Pow(b, e) if e.as_i64() == Some(-1) => {
                let cs = poly::coeffs(b, &x)?;
                match cs.len() {
                    2 if cs[0].is_one() && cs[1].is_one() => (div(pi(), f1("sin", mul(vec![pi(), sv.clone()]))), "0 < Re s < 1"),
                    3 if cs[0].is_one() && cs[1].is_zero() && cs[2].is_one() => (div(pi(), mul(vec![int(2), f1("sin", div(mul(vec![pi(), sv.clone()]), int(2)))])), "0 < Re s < 2"),
                    _ => return None,
                }
            }
            _ => return None,
        };
        let res = s::simplify(&mul(vec![k, res]));
        let label = format!("Mellin transform of {}", ptext(&f));
        return answer(label.clone(), label, vec![primary(pod("Result", "Mellin transform", math(&res))), pod("Strip", "Fundamental strip", text(strip)), pod("Definition", "Definition", text(format!("M[f](s) = ∫₀^∞ {0}^(s−1) f({0}) d{0}", ptext(&x))))]);
    }
    // Z transform: Σ f(n) z^(−n)
    if let Some(m) = cap(re!(r"^(?:the\s+)?z[- ]transform\s+(?:of\s+)?(.+)$")) {
        let f = ex(&g(&m, 1))?;
        let n = sym("n");
        let zz = sym("z");
        let table: Option<Vec<Expr>> = s::expand(&f).terms().iter().map(|t| z_term(t, &n, &zz)).collect();
        let res = match table {
            Some(parts) => {
                // keep the denominator factored, as (z − a)^k
                let t = s::simplify(&s::together(&add(parts)));
                let (nu, de) = t.numer_denom();
                let df = s::factor(&de);
                if df.size() < de.size() { div(nu, df) } else { t }
            }
            None => match crate::calculus::sum::sum(&mul(vec![f.clone(), pow(zz.clone(), neg(n.clone()))]), &n, &zero(), &inf()) {
                crate::calculus::sum::SumResult::Exact(v) => s::simplify(&s::together(&v)),
                _ => return None,
            },
        };
        let label = format!("Z transform of {}", ptext(&f));
        return answer(label.clone(), label, vec![primary(pod("Result", "Z transform", math(&res))), pod("Definition", "Definition", text("Z{f}(z) = Σ_(n=0)^∞ f(n) z^(−n), for |z| large enough that the series converges"))]);
    }
    // inverse Z transform by partial fractions of F(z)/z
    if let Some(m) = cap(re!(r"^(?:the\s+)?inverse z[- ]transform\s+(?:of\s+)?(.+)$")) {
        let f = ex(&g(&m, 1))?;
        let zz = sym("z");
        let n = sym("n");
        let pf = s::apart(&div(f.clone(), zz.clone()), &zz)?;
        let mut terms = Vec::new();
        for t in s::expand(&pf).terms() {
            let (num_, den) = t.numer_denom();
            if num_.has_sym(&zz) {
                return None;
            }
            // den = c (z − p)^k
            let base_pow: Vec<Expr> = den.factors().into_iter().filter(|fct| fct.has_sym(&zz)).collect();
            let cst = s::simplify(&div(num_, mul(den.factors().into_iter().filter(|fct| !fct.has_sym(&zz)).collect())));
            let (b, k) = match base_pow.as_slice() {
                [Pow(b, e)] => ((**b).clone(), e.as_i64()?),
                [b] => (b.clone(), 1),
                _ => return None,
            };
            let cs = poly::coeffs(&b, &zz)?;
            if cs.len() != 2 {
                return None;
            }
            let p = s::simplify(&neg(div(cs[0].clone(), cs[1].clone())));
            let cst = s::simplify(&div(cst, pow(cs[1].clone(), int(k))));
            // z/(z − p)^k ↔ C(n, k − 1) p^(n − k + 1)
            let term = if p.is_zero() {
                if k == 1 { mul(vec![cst, func("DiscreteDelta", vec![n.clone()])]) } else { return None }
            } else {
                match k {
                    1 => mul(vec![cst, pow(p, n.clone())]),
                    2 => mul(vec![cst, n.clone(), pow(p, sub(n.clone(), one()))]),
                    _ => return None,
                }
            };
            terms.push(term);
        }
        let res = s::simplify(&add(terms));
        let label = format!("inverse Z transform of {}", ptext(&f));
        return answer(label.clone(), label, vec![primary(pod("Result", "Inverse Z transform", math(&eq(func("f", vec![n.clone()]), res)))), pod("Method", "Method", text("partial fractions of F(z)/z, then z/(z − p) ↔ pⁿ and z/(z − p)² ↔ n pⁿ⁻¹"))]);
    }
    // convolution
    if let Some(m) = cap(re!(r"^(?:the\s+)?convolution\s+of\s+(.+?)\s+(?:and|with|,)\s+(.+?)$")) {
        let (f, gg) = (ex(&g(&m, 1))?, ex(&g(&m, 2))?);
        let t = if f.has_sym(&sym("t")) || gg.has_sym(&sym("t")) { sym("t") } else { sym("x") };
        let tau = sym("τ");
        let integrand = mul(vec![f.subs(&t, &tau), gg.subs(&t, &sub(t.clone(), tau.clone()))]);
        let v = match definite(&integrand, &tau, &zero(), &t) {
            Definite::Exact(v) => s::simplify(&v),
            _ => return None,
        };
        let label = format!("convolution of {} and {}", ptext(&f), ptext(&gg));
        return answer(label.clone(), label, vec![primary(pod("Result", "Convolution (f ∗ g)(t)", math(&v))), pod("Definition", "Definition", text(format!("(f ∗ g)({0}) = ∫₀^{0} f(τ) g({0} − τ) dτ (causal functions, as for Laplace transforms)", ptext(&t))))]);
    }
    let _ = integrate;
    None
}

// ---------------------------------------------------------------------------
// complex analysis
// ---------------------------------------------------------------------------

type Cx = num_complex::Complex64;

/// Rewrite reciprocal trigonometric functions so that their poles show up in a denominator.
fn trig_to_sin_cos(e: &Expr) -> Expr {
    match e {
        Func(n, a) if a.len() == 1 => {
            let x = trig_to_sin_cos(&a[0]);
            match &**n {
                "csc" => pow(func("sin", vec![x]), int(-1)),
                "sec" => pow(func("cos", vec![x]), int(-1)),
                "cot" => div(func("cos", vec![x.clone()]), func("sin", vec![x])),
                "tan" => div(func("sin", vec![x.clone()]), func("cos", vec![x])),
                "csch" => pow(func("sinh", vec![x]), int(-1)),
                "sech" => pow(func("cosh", vec![x]), int(-1)),
                "coth" => div(func("cosh", vec![x.clone()]), func("sinh", vec![x])),
                "tanh" => div(func("sinh", vec![x.clone()]), func("cosh", vec![x])),
                _ => func(n, vec![x]),
            }
        }
        Func(n, a) => func(n, a.iter().map(trig_to_sin_cos).collect()),
        Add(v) => add(v.iter().map(trig_to_sin_cos).collect()),
        Mul(v) => mul(v.iter().map(trig_to_sin_cos).collect()),
        Pow(a, b) => pow(trig_to_sin_cos(a), trig_to_sin_cos(b)),
        _ => e.clone(),
    }
}

/// Values for the free parameters of a root (n, C1, …): `k` for every one.
fn bind_params(r: &Expr, zv: &Expr, k: i64) -> Expr {
    let mut out = r.clone();
    for name in r.free_symbols() {
        let sy = sym(&name);
        if &sy != zv {
            out = out.subs(&sy, &int(k));
        }
    }
    out
}

fn eval_at(e: &Expr, zv: &Expr, z: Cx) -> Option<Cx> {
    let Sym(name) = zv else { return None };
    crate::eval::eval_c64(e, &[(&**name, z)]).filter(|v| v.re.is_finite() && v.im.is_finite())
}

/// Order of the zero of `h` at z0 (0 when h(z0) ≠ 0), by successive derivatives.
fn zero_order(h: &Expr, zv: &Expr, z0: Cx) -> Option<usize> {
    let mut d = h.clone();
    for k in 0..10 {
        let v = eval_at(&d, zv, z0)?;
        if v.norm() > 1e-8 {
            return Some(k);
        }
        d = s::simplify(&diff(&d, zv));
    }
    None
}

/// Zeros of a denominator: periodic families (with integer n) for sin, cos,
/// sinh, cosh and exp(u) + c of a linear argument; the solver for the rest.
fn zero_families(den: &Expr, zv: &Expr) -> Vec<Expr> {
    let n = sym("n");
    let pi = Const(K::Pi);
    let i = Const(K::I);
    let mut out = Vec::new();
    let linear = |u: &Expr| -> Option<(Expr, Expr)> {
        let cs = poly::coeffs(u, zv)?;
        (cs.len() == 2 && !cs[1].is_zero()).then(|| (cs[1].clone(), cs[0].clone()))
    };
    for fct in den.factors() {
        let base = match &fct {
            Pow(b, e) if e.as_i64().map_or(false, |k| k > 0) => (**b).clone(),
            _ => fct.clone(),
        };
        if !base.has_sym(zv) {
            continue;
        }
        let fam = match &base {
            Func(name, a) if a.len() == 1 => linear(&a[0]).and_then(|(al, be)| {
                let u0 = match &**name {
                    "sin" => mul(vec![pi.clone(), n.clone()]),
                    "cos" => add(vec![div(pi.clone(), int(2)), mul(vec![pi.clone(), n.clone()])]),
                    "sinh" => mul(vec![i.clone(), pi.clone(), n.clone()]),
                    "cosh" => mul(vec![i.clone(), add(vec![div(pi.clone(), int(2)), mul(vec![pi.clone(), n.clone()])])]),
                    _ => return None,
                };
                Some(s::simplify(&div(sub(u0, be), al)))
            }),
            Add(v) if v.len() == 2 => {
                let (e, c) = if v[0].has_sym(zv) { (&v[0], &v[1]) } else { (&v[1], &v[0]) };
                let as_exp = |x: &Expr| -> Option<Expr> {
                    match x {
                        Func(nm, a) if &**nm == "exp" => Some(a[0].clone()),
                        Pow(bb, ee) if matches!(**bb, Const(K::E)) => Some((**ee).clone()),
                        _ => None,
                    }
                };
                let (k, u) = match e {
                    Mul(w) if w.len() == 2 => match (as_exp(&w[0]), as_exp(&w[1])) {
                        (None, Some(u)) if !w[0].has_sym(zv) => (w[0].clone(), u),
                        (Some(u), None) if !w[1].has_sym(zv) => (w[1].clone(), u),
                        _ => (zero(), zero()),
                    },
                    x => match as_exp(x) {
                        Some(u) => (one(), u),
                        None => (zero(), zero()),
                    },
                };
                if k.is_zero() || c.has_sym(zv) {
                    None
                } else {
                    linear(&u).map(|(al, be)| {
                        let u0 = add(vec![ln(neg(div(c.clone(), k))), mul(vec![int(2), pi.clone(), i.clone(), n.clone()])]);
                        s::simplify(&div(sub(u0, be), al))
                    })
                }
            }
            _ => None,
        };
        match fam {
            Some(r) => out.push(r),
            None => {
                let roots = solve::solve(&eq(base.clone(), zero()), zv);
                out.extend(roots.real().into_iter().chain(roots.complex()));
            }
        }
    }
    out
}

/// The member of a pole family equal to `a`, if any.
fn family_matches(p: &Expr, zv: &Expr, a: &Expr) -> bool {
    if s::equal(p, a) {
        return true;
    }
    let Some(av) = crate::eval::const_c64(a) else { return false };
    let periodic = p.free_symbols().iter().any(|n| sym(n) != *zv);
    let range: Vec<i64> = if periodic { (-300..=300).collect() } else { vec![0] };
    range.into_iter().any(|n| crate::eval::const_c64(&bind_params(p, zv, n)).map_or(false, |v| (v - av).norm() < 1e-9))
}

/// Poles with their orders.
fn poles(f: &Expr, zv: &Expr) -> Option<Vec<(Expr, usize)>> {
    let f = trig_to_sin_cos(f);
    let (num_, den) = s::together(&f).numer_denom();
    if !den.has_sym(zv) {
        return Some(vec![]);
    }
    let mut out: Vec<(Expr, usize)> = Vec::new();
    for r in zero_families(&den, zv) {
        if out.iter().any(|(p, _)| s::equal(p, &r)) {
            continue;
        }
        let z0 = crate::eval::const_c64(&bind_params(&r, zv, 1))?;
        let order = match (zero_order(&den, zv, z0), zero_order(&num_, zv, z0)) {
            (Some(d), Some(n)) => d.saturating_sub(n),
            _ => continue,
        };
        if order > 0 {
            out.push((r, order));
        }
    }
    Some(out)
}

/// Residue by the trapezoidal rule on a small circle (spectrally accurate).
fn residue_numeric(f: &Expr, zv: &Expr, a: Cx, radius: f64) -> Option<Cx> {
    let n = 512;
    let mut sum = Cx::new(0.0, 0.0);
    for k in 0..n {
        let w = Cx::from_polar(radius, 2.0 * std::f64::consts::PI * k as f64 / n as f64);
        sum += eval_at(f, zv, a + w)? * w;
    }
    Some(sum / n as f64)
}

fn recognise(v: Cx) -> Expr {
    let part = |x: f64| -> Expr {
        if x.abs() < 1e-9 {
            return zero();
        }
        for q in 1..=4096i64 {
            let p = (x * q as f64).round();
            if (x * q as f64 - p).abs() < 1e-9 * q as f64 {
                return s::simplify(&div(int(p as i64), int(q)));
            }
        }
        float(x)
    };
    s::simplify(&add(vec![part(v.re), mul(vec![part(v.im), Const(K::I)])]))
}

fn residue(f: &Expr, zv: &Expr, a: &Expr, order: usize) -> Option<Expr> {
    let ft = trig_to_sin_cos(f);
    let symbolic = || -> Option<Expr> {
        if order == 1 {
            let (nn, dd) = s::together(&ft).numer_denom();
            let r = s::simplify(&div(nn.subs(zv, a), diff(&dd, zv).subs(zv, a)));
            if crate::eval::const_c64(&bind_params(&r, zv, 1)).is_some() {
                return Some(r);
            }
        }
        if order > 4 {
            return None;
        }
        let mut g_ = s::cancel(&mul(vec![pow(sub(zv.clone(), a.clone()), int(order as i64)), ft.clone()]));
        for _ in 1..order {
            g_ = diff(&g_, zv);
        }
        let fact: i64 = (1..order as i64).product::<i64>().max(1);
        let v = limit(&g_, zv, a, Dir::Both)?;
        Some(s::simplify(&div(v, int(fact))))
    };
    let a1 = crate::eval::const_c64(&bind_params(a, zv, 1))?;
    // shrink the circle until two radii agree (no other pole inside)
    let mut numeric = None;
    let mut prev: Option<Cx> = None;
    for r in [0.4, 0.2, 0.1, 0.05, 0.02] {
        let v = residue_numeric(&ft, zv, a1, r);
        if let (Some(v), Some(p)) = (v, prev) {
            if (v - p).norm() < 1e-8 * (1.0 + v.norm()) {
                numeric = Some(v);
                break;
            }
        }
        prev = v;
    }
    let numeric = numeric.or(prev);
    match (symbolic(), numeric) {
        (Some(r), Some(nv)) => {
            let rv = crate::eval::const_c64(&bind_params(&r, zv, 1));
            if rv.map_or(false, |rv| (rv - nv).norm() < 1e-6 * (1.0 + nv.norm())) {
                // a residue that is the same for every member of a family is a constant
                let periodic = r.free_symbols().iter().any(|n| sym(n) != *zv);
                let vals: Vec<Option<Cx>> = (-2..=3).map(|k| crate::eval::const_c64(&bind_params(&r, zv, k))).collect();
                if periodic && vals.iter().all(|v| v.map_or(false, |v| (v - nv).norm() < 1e-9 * (1.0 + nv.norm()))) {
                    return Some(recognise(nv));
                }
                Some(r)
            } else {
                Some(recognise(nv))
            }
        }
        (Some(r), None) => Some(r),
        (None, Some(nv)) if a.free_symbols().iter().all(|n| sym(n) == *zv) => Some(recognise(nv)),
        _ => None,
    }
}

/// Region conditions such as "|z| < 3pi" or "0 < Im z < 12".
fn in_region(cond: &str, z: Cx) -> Option<bool> {
    let mut ok = true;
    for part in cond.split(" and ") {
        let part = part.trim();
        let m = re!(r"^(?:(.+?)\s*(<=|<)\s*)?(\|\s*[a-z]\s*\||im\s*\(?\s*[a-z]\s*\)?|re\s*\(?\s*[a-z]\s*\)?)\s*(<=|<)\s*(.+)$").captures(part)?;
        let what = &m[3];
        let val = if what.starts_with('|') { z.norm() } else if what.starts_with("im") { z.im } else { z.re };
        let hi = crate::eval::const_c64(&ex(&m[5])?)?.re;
        if val > hi + 1e-12 || (&m[4] == "<" && (val - hi).abs() < 1e-12) {
            ok = false;
        }
        if let Some(lo) = m.get(1) {
            let lo = crate::eval::const_c64(&ex(lo.as_str())?)?.re;
            if val < lo - 1e-12 || (&m[2] == "<" && (val - lo).abs() < 1e-12) {
                ok = false;
            }
        }
    }
    Some(ok)
}

/// Explicit poles inside a region, enumerating the integer parameter of periodic families.
fn poles_in(ps: &[(Expr, usize)], zv: &Expr, cond: &str) -> Option<Vec<(Expr, usize)>> {
    let mut out: Vec<(Expr, usize, Cx)> = Vec::new();
    for (p, k) in ps {
        let periodic = p.free_symbols().iter().any(|n| sym(n) != *zv);
        let range: Vec<i64> = if periodic { (-200..=200).collect() } else { vec![0] };
        for n in range {
            let e = s::simplify(&bind_params(p, zv, n));
            let v = crate::eval::const_c64(&e)?;
            if in_region(cond, v)? && !out.iter().any(|(_, _, w)| (w - v).norm() < 1e-9) {
                out.push((e, *k, v));
            }
        }
    }
    out.sort_by(|a, b| a.2.norm().partial_cmp(&b.2.norm()).unwrap_or(std::cmp::Ordering::Equal).then(a.2.im.partial_cmp(&b.2.im).unwrap_or(std::cmp::Ordering::Equal)));
    Some(out.into_iter().map(|(e, k, _)| (e, k)).collect())
}

pub fn complex(q: &Query, l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    let _ = q;
    if let Some(m) = cap(re!(r"^(?:show\s+|find\s+)?(?:the\s+)?(poles|pole points|singularities|singular points)\s+of\s+(.+?)(?:\s+(?:with|for|in|where)\s+(.*[<>].*))?$")) {
        let f = ex(&g(&m, 2))?;
        let zv = super::nl::main_var(&f);
        let mut ps = poles(&f, &zv)?;
        if !g(&m, 3).is_empty() {
            ps = poles_in(&ps, &zv, &g(&m, 3).to_lowercase())?;
        }
        let rows: Vec<Vec<Content>> = ps.iter().map(|(p, k)| vec![math(p), text(k.to_string()), residue(&f, &zv, p, *k).map(|r| math(&r)).unwrap_or_else(|| text("?"))]).collect();
        let label = format!("poles of {}", ptext(&f));
        return answer(label.clone(), label, vec![primary(pod("Result", "Poles", if ps.is_empty() { text("none — the function is entire") } else { table(Some(vec!["pole", "order", "residue"]), rows) }))]);
    }
    let residue_re = re!(r"^(?:show\s+|find\s+)?(?:the\s+)?residues?\s+(?:of\s+)?(.+?)(?:\s+at\s+([a-z])\s*=\s*(.+?)|\s+(?:with|for|in|where)\s+(.*[<>].*))?$");
    let alt = re!(r"^(.+?)\s+residues?\s+at\s+([a-z])\s*=\s*(.+)$");
    let m = match cap(alt) {
        Some(m) => Some(m),
        None => cap(residue_re),
    };
    if let Some(m) = m {
        let f = ex(&g(&m, 1))?;
        let zv = if g(&m, 2).is_empty() { super::nl::main_var(&f) } else { sym(&g(&m, 2)) };
        let mut ps = poles(&f, &zv)?;
        if !g(&m, 4).is_empty() {
            ps = poles_in(&ps, &zv, &g(&m, 4).to_lowercase())?;
        }
        let label = format!("residue of {}", ptext(&f));
        if !g(&m, 3).is_empty() {
            let a = ex(&g(&m, 3))?;
            let order = ps.iter().find(|(p, _)| family_matches(p, &zv, &a)).map(|(_, k)| *k);
            let Some(order) = order else {
                return answer(label.clone(), label, vec![primary(pod("Result", "Residue", text(format!("0 — the function is analytic at {}", ptext(&a)))))]);
            };
            let r = residue(&f, &zv, &a, order)?;
            let mut pods = vec![primary(pod("Result", &format!("Residue at {} = {}", ptext(&zv), ptext(&a)), math(&r)))];
            pods.push(pod("Order", "Pole order", text(order.to_string())));
            pods.push(pod("Method", "Method", text(if order == 1 { "Res = lim (z − a) f(z)" } else { "Res = 1/(m − 1)! · lim d^(m−1)/dz^(m−1) [(z − a)^m f(z)]" })));
            return answer(label.clone(), label, pods);
        }
        let rows: Vec<Vec<Content>> = ps.iter().map(|(p, k)| vec![math(p), text(k.to_string()), residue(&f, &zv, p, *k).map(|r| math(&r)).unwrap_or_else(|| text("?"))]).collect();
        return answer(label.clone(), label, vec![primary(pod("Result", "Residues", table(Some(vec!["pole", "order", "residue"]), rows)))]);
    }
    // contour integrals around circles
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:contour|closed|line)?\s*integral\s+(?:of\s+)?(.+?)\s+(?:around|over|along|on)\s+(?:the\s+)?(unit circle|circle\s+\|?\s*([a-z])\s*(?:-\s*([^|]+?))?\s*\|?\s*=\s*([\d.]+)|\|\s*([a-z])\s*(?:-\s*([^|]+?))?\s*\|\s*=\s*([\d.]+))$")) {
        let f = ex(&g(&m, 1))?;
        let zv = super::nl::main_var(&f);
        let (center, radius) = if g(&m, 2) == "unit circle" {
            (zero(), 1.0)
        } else {
            let cen = [g(&m, 4), g(&m, 7)].into_iter().find(|t| !t.is_empty()).map(|t| ex(&t)).unwrap_or(Some(zero()))?;
            let rad: f64 = [g(&m, 5), g(&m, 8)].into_iter().find(|t| !t.is_empty())?.parse().ok()?;
            (cen, rad)
        };
        let ps = poles(&f, &zv)?;
        let cc = crate::eval::const_c64(&center)?;
        let mut inside = Vec::new();
        for (p, k) in &ps {
            let pc = crate::eval::const_c64(p)?;
            let d = (pc - cc).norm();
            if (d - radius).abs() < 1e-12 {
                return answer("contour integral", "contour integral", vec![primary(pod("Result", "Result", text(format!("A pole lies on the contour (at {}), so the integral is not defined as an ordinary contour integral.", ptext(p)))))]);
            }
            if d < radius {
                inside.push((p.clone(), residue(&f, &zv, p, *k)?));
            }
        }
        let total = s::simplify(&mul(vec![int(2), pi(), i(), add(inside.iter().map(|(_, r)| r.clone()).collect())]));
        let label = format!("∮ {} d{} around |{} − {}| = {}", ptext(&f), ptext(&zv), ptext(&zv), ptext(&center), fmt_float(radius, 6));
        let rows: Vec<Vec<Content>> = inside.iter().map(|(p, r)| vec![math(p), math(r)]).collect();
        return answer(label.clone(), label, vec![
            primary(pod("Result", "Contour integral (counterclockwise)", math(&total))),
            pod("Poles", "Poles inside the contour and their residues", if rows.is_empty() { text("none") } else { table(Some(vec!["pole", "residue"]), rows) }),
            pod("Method", "Method", text("residue theorem: ∮ f dz = 2πi Σ residues inside")),
        ]);
    }
    None
}

pub fn handle(q: &Query, l: &str, c: &str) -> Option<Answer> {
    number_theory(q, l, c)
        .or_else(|| special(q, l, c))
        .or_else(|| matrices(q, l, c))
        .or_else(|| ode_systems(q, l, c))
        .or_else(|| recurrences(q, l, c))
        .or_else(|| transforms(q, l, c))
        .or_else(|| complex(q, l, c))
}
