//! More mathematics asked in words: number facts, lines and circles,
//! discriminants, Fourier series, curvature, matrix decompositions and
//! tests, and statistical inference.

use super::nl::{again, answer, caps, decimal_pod, ex, main_var};
use super::nl_math::{definite_expr, fx, g, mat_of, plot_pod, point, value_pods, vec_expr, vec_of};
use super::*;
use crate::calculus;
use crate::expr::Expr::*;
use crate::expr::*;
use crate::linalg::{self, Mat};
use crate::numtheory as nt;
use crate::plot as pl;
use crate::poly;
use crate::print::text as ptext;
use crate::simplify as s;
use crate::solve;
use num_bigint::BigInt;
use num_traits::{One, Zero};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn zero_expr(e: &Expr) -> bool {
    s::simplify(e).is_zero()
}

// ---------------------------------------------------------------------------
// numbers
// ---------------------------------------------------------------------------

const PERFECT: [u64; 8] = [6, 28, 496, 8128, 33_550_336, 8_589_869_056, 137_438_691_328, 2_305_843_008_139_952_128];

fn is_carmichael(n: &BigInt) -> bool {
    if n < &BigInt::from(561) || nt::is_prime(n) || (n % 2u32).is_zero() {
        return false;
    }
    let f = nt::factor(n);
    f.iter().all(|(_, e)| *e == 1) && f.iter().all(|(p, _)| ((n - 1u32) % (p - 1u32)).is_zero())
}

pub fn numbers(q: &Query, l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    // phrasing variants handled elsewhere
    if let Some(m) = cap(re!(r"^digits\s+(?:are\s+)?(?:there\s+)?in\s+(.+)$")) {
        return again(q, &format!("number of digits in {}", g(&m, 1)));
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?roman numerals?\s+(?:for|of)\s+(\d+)$")) {
        return again(q, &format!("{} in roman numerals", g(&m, 1)));
    }
    if let Some(m) = cap(re!(r"^is\s+(\d+)\s+a\s+carmichael number$")) {
        let n: BigInt = g(&m, 1).parse().ok()?;
        let yes = is_carmichael(&n);
        let f = nt::factor(&n);
        let fs = f.iter().map(|(p, e)| if *e == 1 { p.to_string() } else { format!("{p}^{e}") }).collect::<Vec<_>>().join(" × ");
        let why = if nt::is_prime(&n) {
            format!("{n} is prime; Carmichael numbers are composite")
        } else if yes {
            format!("{n} = {fs} is square-free and p − 1 divides {} for every prime factor p (Korselt's criterion)", &n - 1u32)
        } else {
            format!("{n} = {fs} fails Korselt's criterion (square-free, and p − 1 divides n − 1 for every prime factor p)")
        };
        let label = format!("is {n} a Carmichael number?");
        return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(format!("{} — {n} {} a Carmichael number", if yes { "Yes" } else { "No" }, if yes { "is" } else { "is not" })))), pod("Reason", "Reason", text(why)), pod("First", "First Carmichael numbers", text("561, 1105, 1729, 2465, 2821, 6601, 8911, 10585, 15841, 29341, …"))]);
    }
    // decimals, fractions and percentages
    if let Some(m) = cap(re!(r"^(?:convert\s+|write\s+|express\s+)?(.+?)\s+(?:as|to|into|in)\s+(?:a\s+|an\s+)?(fraction|decimal|percent(?:age)?|mixed number|scientific notation|engineering notation)(?:\s+form)?$")) {
        let body = g(&m, 1);
        let want = g(&m, 2).to_lowercase();
        let t = body.trim().trim_end_matches('%');
        let pct_in = body.trim().ends_with('%');
        // exact value: decimals are read exactly as fractions
        let exact = if let Some(d) = re!(r"^-?\d*\.\d+$").find(t) {
            let txt = d.as_str();
            let neg = txt.starts_with('-');
            let digits = txt.trim_start_matches('-');
            let (ip, fp) = digits.split_once('.')?;
            let den = BigInt::from(10).pow(fp.len() as u32);
            let numer: BigInt = format!("{ip}{fp}").trim_start_matches('0').parse().unwrap_or_else(|_| BigInt::zero());
            num(Rat::new(if neg { -numer } else { numer }, den))
        } else {
            ex(t)?
        };
        let exact = if pct_in { s::simplify(&div(exact, int(100))) } else { exact };
        if !exact.free_symbols().is_empty() {
            return None;
        }
        let v = exact.to_f64_const()?;
        let label = format!("{} as {}", body.trim(), want);
        let (title, content): (&str, Content) = match want.as_str() {
            "fraction" => {
                let r = exact.as_rat().cloned().or_else(|| exact.as_int().map(|i| Rat::from_integer(i.clone())))?;
                ("Fraction", math(&num(r)))
            }
            "mixed number" => {
                let r = exact.as_rat().cloned()?;
                let (whole, rem) = (r.numer() / r.denom(), r.numer() % r.denom());
                ("Mixed number", text(if rem.is_zero() { whole.to_string() } else if whole.is_zero() { format!("{rem}/{}", r.denom()) } else { format!("{whole} {}/{}", num_traits::Signed::abs(&rem), r.denom()) }))
            }
            "decimal" => {
                if let Some(r) = exact.as_rat() {
                    if let Some((ip, pre, rep)) = nt::repeating_decimal(r.numer(), r.denom(), 200) {
                        let shown = if rep.is_empty() { format!("{ip}{}{pre}", if pre.is_empty() { "" } else { "." }) } else { format!("{ip}.{pre}{rep}{rep}{rep}…  (the digits {rep} repeat)") };
                        ("Decimal", text(shown))
                    } else {
                        ("Decimal", text(crate::print::fmt_float(v, 15)))
                    }
                } else {
                    ("Decimal", text(crate::bigfloat::decimal(&exact, 20).unwrap_or_else(|| crate::print::fmt_float(v, 15))))
                }
            }
            w if w.starts_with("percent") => ("Percentage", text(format!("{}%", crate::print::fmt_float(v * 100.0, 12)))),
            "engineering notation" => {
                let e = if v == 0.0 { 0 } else { (v.abs().log10().floor() as i32).div_euclid(3) * 3 };
                ("Engineering notation", text(format!("{} × 10^{e}", crate::print::fmt_float(v / 10f64.powi(e), 12))))
            }
            _ => {
                let e = if v == 0.0 { 0 } else { v.abs().log10().floor() as i32 };
                ("Scientific notation", text(format!("{} × 10^{e}", crate::print::fmt_float(v / 10f64.powi(e), 12))))
            }
        };
        let mut pods = vec![primary(pod("Result", title, content))];
        if want != "decimal" {
            pods.push(pod("Decimal", "Decimal form", text(crate::print::fmt_float(v, 15))));
        }
        if want != "fraction" {
            if let Some(r) = exact.as_rat() {
                pods.push(pod("Fraction", "Fraction", math(&num(r.clone()))));
            }
        }
        if !want.starts_with("percent") {
            pods.push(pod("Percent", "Percentage", text(format!("{}%", crate::print::fmt_float(v * 100.0, 12)))));
        }
        return answer(label.clone(), label, pods);
    }
    // polygon sides
    if let Some(m) = cap(re!(r"^(?:(?:how many|number of)\s+)?(sides|vertices|corners|angles|edges|faces)\s+(?:does|do|are there in|in|on|of)\s+(?:a|an|the)\s+([a-z]+)(?:\s+have)?$")) {
        const SHAPES: &[(&str, u32, u32, u32)] = &[
            // name, faces (0 = plane figure), edges or sides, vertices
            ("triangle", 0, 3, 3), ("square", 0, 4, 4), ("rectangle", 0, 4, 4), ("rhombus", 0, 4, 4), ("trapezoid", 0, 4, 4), ("quadrilateral", 0, 4, 4), ("parallelogram", 0, 4, 4),
            ("pentagon", 0, 5, 5), ("hexagon", 0, 6, 6), ("heptagon", 0, 7, 7), ("octagon", 0, 8, 8), ("nonagon", 0, 9, 9), ("decagon", 0, 10, 10), ("hendecagon", 0, 11, 11), ("dodecagon", 0, 12, 12), ("icosagon", 0, 20, 20),
            ("tetrahedron", 4, 6, 4), ("cube", 6, 12, 8), ("octahedron", 8, 12, 6), ("dodecahedron", 12, 30, 20), ("icosahedron", 20, 30, 12), ("square pyramid", 5, 8, 5), ("pyramid", 5, 8, 5), ("triangular prism", 5, 9, 6), ("prism", 5, 9, 6), ("cuboid", 6, 12, 8),
        ];
        let shape = g(&m, 2).to_lowercase();
        let (_, faces, edges, verts) = SHAPES.iter().find(|x| x.0 == shape)?;
        let what = g(&m, 1).to_lowercase();
        let v = match what.as_str() {
            "faces" if *faces > 0 => *faces,
            "faces" => return None,
            "vertices" | "corners" => *verts,
            _ => *edges,
        };
        let mut rows = vec![vec![text(if *faces > 0 { "edges" } else { "sides" }), text(edges.to_string())], vec![text("vertices"), text(verts.to_string())]];
        if *faces > 0 {
            rows.insert(0, vec![text("faces"), text(faces.to_string())]);
            rows.push(vec![text("Euler characteristic V − E + F"), text((*verts as i64 - *edges as i64 + *faces as i64).to_string())]);
        } else {
            rows.push(vec![text("interior angle sum"), text(format!("{}°", 180 * (edges - 2)))]);
        }
        let label = format!("number of {what} of a {shape}");
        return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(v.to_string()))), pod("Facts", "Properties", table(None, rows))]);
    }
    // trailing zeros
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:number of\s+)?trailing zero(?:e?s)?\s+(?:in|of|at the end of)\s+(.+)$")) {
        let body = g(&m, 1);
        let label = format!("number of trailing zeros of {body}");
        if let Some(f) = re!(r"^(\d+)\s*!$").captures(body.trim()) {
            let n: u64 = f[1].parse().ok()?;
            let mut terms = Vec::new();
            let (mut p, mut total) = (5u64, 0u64);
            while p <= n {
                terms.push(format!("⌊{n}/{p}⌋ = {}", n / p));
                total += n / p;
                p = p.checked_mul(5)?;
            }
            let how = if terms.is_empty() { "there are no factors of 5".to_string() } else { format!("{} — the number of factors of 5 (Legendre's formula)", terms.join(" + ")) };
            return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(total.to_string()))), pod("Method", "Method", text(how))]);
        }
        let n = ex(&body)?.as_int().cloned()?;
        let digits = n.to_string();
        let zeros = digits.len() - digits.trim_end_matches('0').len();
        return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(if n.is_zero() { "1".to_string() } else { zeros.to_string() })))]);
    }
    // lists of special numbers below a bound
    if let Some(m) = cap(re!(r"^(?:all\s+|the\s+|list\s+(?:of\s+)?(?:all\s+)?(?:the\s+)?)?(perfect|square|cube|triangular|fibonacci|palindromic|abundant|deficient|carmichael|twin prime|mersenne prime|composite|even|odd|prime)\s*(?:numbers?|s)?\s+(?:below|under|less than|up to|smaller than|not exceeding|<=?|until)\s+(\d+)$")) {
        let kind = g(&m, 1).to_lowercase();
        let limit: u64 = g(&m, 2).parse().ok()?;
        if limit > 5_000_000 && kind != "perfect" && kind != "fibonacci" && kind != "mersenne prime" {
            return None;
        }
        let list: Vec<String> = match kind.as_str() {
            "perfect" => PERFECT.iter().filter(|p| **p <= limit).map(|p| p.to_string()).collect(),
            "prime" => nt::sieve(limit as usize).iter().map(|p| p.to_string()).collect(),
            "twin prime" => nt::sieve(limit as usize).windows(2).filter(|w| w[1] - w[0] == 2).map(|w| format!("({}, {})", w[0], w[1])).collect(),
            "mersenne prime" => (2..62u32).map(|p| (1u64 << p) - 1).filter(|v| *v <= limit && nt::is_prime_u64(*v)).map(|v| v.to_string()).collect(),
            "square" => (1..).map(|k: u64| k * k).take_while(|v| *v <= limit).map(|v| v.to_string()).collect(),
            "cube" => (1..).map(|k: u64| k * k * k).take_while(|v| *v <= limit).map(|v| v.to_string()).collect(),
            "triangular" => (1..).map(|k: u64| k * (k + 1) / 2).take_while(|v| *v <= limit).map(|v| v.to_string()).collect(),
            "fibonacci" => (1..94).map(nt::fib).take_while(|v| v <= &BigInt::from(limit)).map(|v| v.to_string()).collect::<Vec<_>>().into_iter().skip(1).collect(),
            "palindromic" => (1..=limit.min(2_000_000)).filter(|v| { let t = v.to_string(); t.chars().rev().collect::<String>() == t }).map(|v| v.to_string()).collect(),
            "even" => (2..=limit.min(100_000)).step_by(2).map(|v| v.to_string()).collect(),
            "odd" => (1..=limit.min(100_000)).step_by(2).map(|v| v.to_string()).collect(),
            "composite" => { let ps = nt::sieve(limit.min(200_000) as usize); (4..=limit.min(200_000)).filter(|v| ps.binary_search(v).is_err()).map(|v| v.to_string()).collect() }
            "carmichael" => (561..=limit.min(200_000)).step_by(2).filter(|v| is_carmichael(&BigInt::from(*v))).map(|v| v.to_string()).collect(),
            _ => {
                // abundant / deficient via a divisor-sum sieve
                let n = limit.min(200_000) as usize;
                let mut sums = vec![0u64; n + 1];
                for d in 1..=n / 2 {
                    let mut k = 2 * d;
                    while k <= n {
                        sums[k] += d as u64;
                        k += d;
                    }
                }
                (1..=n).filter(|v| if kind == "abundant" { sums[*v] > *v as u64 } else { sums[*v] < *v as u64 }).map(|v| v.to_string()).collect()
            }
        };
        let shown = if list.len() > 300 { format!("{}, … ({} more)", list[..300].join(", "), list.len() - 300) } else if list.is_empty() { "none".to_string() } else { list.join(", ") };
        let label = format!("{kind} numbers up to {limit}");
        return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(shown))), pod("Count", "Count", text(list.len().to_string()))]);
    }
    // combinations with repetition
    if let Some(m) = cap(re!(r"^(\d+)\s+choose\s+(\d+)\s+with\s+repetitions?(?:\s+allowed)?$"))
        .map(|m| (g(&m, 1), g(&m, 2)))
        .or_else(|| cap(re!(r"^(?:number of\s+)?(?:combinations|multisets|selections)\s+(?:with repetitions?\s+)?of\s+(\d+)\s+(?:from|out of|of)\s+(\d+)\s+with\s+repetitions?(?:\s+allowed)?$")).map(|m| (g(&m, 2), g(&m, 1))))
    {
        let (n, k): (u64, u64) = (m.0.parse().ok()?, m.1.parse().ok()?);
        if n == 0 || n + k > 100_000 {
            return None;
        }
        let v = nt::binomial(&BigInt::from(n + k - 1), k);
        let label = format!("choose {k} from {n} with repetition allowed");
        return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(v.to_string()))), pod("Formula", "Formula", text(format!("C(n + k − 1, k) = C({}, {k})", n + k - 1))), pod("Without", "Without repetition", text(format!("C({n}, {k}) = {}", nt::binomial(&BigInt::from(n), k))))]);
    }
    // handshakes
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:number of\s+)?handshakes\s+(?:are there\s+)?(?:with|among|between|for|in a group of|in a room of|if)\s+(\d+)\s*(?:people|persons|guests|friends)?(?:\s+(?:all\s+)?shake hands)?$")) {
        let n: u64 = g(&m, 1).parse().ok()?;
        let v = nt::binomial(&BigInt::from(n), 2);
        let label = format!("handshakes among {n} people");
        return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(v.to_string()))), pod("Formula", "Formula", text(format!("each pair shakes hands once: C({n}, 2) = {n}·{}/2", n.saturating_sub(1))))]);
    }
    None
}

// ---------------------------------------------------------------------------
// algebra and analytic geometry
// ---------------------------------------------------------------------------

fn as_zero_form(e: &Expr) -> Expr {
    if let Rel(RelOp::Eq, a, b) = e { s::expand(&sub((**a).clone(), (**b).clone())) } else { s::expand(e) }
}

/// `A x + B y + C` coefficients of a linear expression.
fn line_coeffs(e: &Expr, x: &Expr, y: &Expr) -> Option<(Expr, Expr, Expr)> {
    let cx = poly::coeffs(e, x)?;
    if cx.len() > 2 {
        return None;
    }
    let a = cx.get(1).cloned().unwrap_or_else(zero);
    let cy = poly::coeffs(cx.first()?, y)?;
    if cy.len() > 2 || a.has_sym(y) {
        return None;
    }
    let b = cy.get(1).cloned().unwrap_or_else(zero);
    let c0 = cy.first().cloned().unwrap_or_else(zero);
    (!c0.has_sym(x) && !c0.has_sym(y)).then_some((a, b, c0))
}

fn points_in(t: &str) -> Vec<(Expr, Expr)> {
    re!(r"\(\s*([^,()]+)\s*,\s*([^,()]+)\s*\)").captures_iter(t).filter_map(|m| Some((ex(&m[1])?, ex(&m[2])?))).collect()
}

fn circle_pods(h: &Expr, k: &Expr, r2: &Expr) -> Vec<Pod> {
    let (x, y) = (sym("x"), sym("y"));
    let r = s::simplify(&sqrt(r2.clone()));
    let standard = eq(add(vec![pow(sub(x.clone(), h.clone()), int(2)), pow(sub(y.clone(), k.clone()), int(2))]), r2.clone());
    let general = eq(s::expand(&sub(add(vec![pow(sub(x.clone(), h.clone()), int(2)), pow(sub(y.clone(), k.clone()), int(2))]), r2.clone())), zero());
    let mut pods = vec![primary(pod("Result", "Equation of the circle", math(&standard))), pod("General", "Expanded form", math(&general)), pod("Center", "Center", math(&point(h, k))), pod("Radius", "Radius", math(&r))];
    pods.push(pod("Measures", "Area and circumference", table(None, vec![vec![text("area"), math(&s::simplify(&mul(vec![pi(), r2.clone()])))], vec![text("circumference"), math(&s::simplify(&mul(vec![int(2), pi(), r.clone()])))], vec![text("diameter"), math(&s::simplify(&mul(vec![int(2), r.clone()])))]])));
    if let (Some(hf), Some(kf), Some(rf)) = (h.to_f64_const(), k.to_f64_const(), r.to_f64_const()) {
        let t = sym("t");
        let _ = t;
        let upper = add(vec![k.clone(), sqrt(sub(r2.clone(), pow(sub(x.clone(), h.clone()), int(2))))]);
        let lower = sub(k.clone(), sqrt(sub(r2.clone(), pow(sub(x.clone(), h.clone()), int(2)))));
        let _ = kf;
        if let Some(p) = plot_pod(&[upper, lower], &x, Some((hf - rf, hf + rf))) {
            pods.push(p);
        }
    }
    pods
}

pub fn algebra(_q: &Query, l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    let (x, y) = (sym("x"), sym("y"));

    // discriminant
    if let Some(m) = cap(re!(r"^(?:the\s+)?discriminant\s+(?:of|for)\s+(.+)$")) {
        let e = as_zero_form(&fx(&g(&m, 1))?);
        let v = main_var(&e);
        let cs = poly::coeffs(&e, &v)?;
        let n = cs.len().checked_sub(1)?;
        if !(2..=8).contains(&n) {
            return None;
        }
        let (d, formula): (Expr, String) = match n {
            2 => (s::simplify(&sub(square(cs[1].clone()), mul(vec![int(4), cs[2].clone(), cs[0].clone()]))), "Δ = b² − 4 a c".into()),
            3 => {
                let (a, b, c0, d0) = (cs[3].clone(), cs[2].clone(), cs[1].clone(), cs[0].clone());
                let v = add(vec![
                    mul(vec![int(18), a.clone(), b.clone(), c0.clone(), d0.clone()]),
                    mul(vec![int(-4), pow(b.clone(), int(3)), d0.clone()]),
                    mul(vec![square(b.clone()), square(c0.clone())]),
                    mul(vec![int(-4), a.clone(), pow(c0.clone(), int(3))]),
                    mul(vec![int(-27), square(a.clone()), square(d0.clone())]),
                ]);
                (s::simplify(&v), "Δ = 18abcd − 4b³d + b²c² − 4ac³ − 27a²d²".into())
            }
            _ => {
                let res = solve::resultant(&e, &calculus::diff::diff(&e, &v), &v)?;
                let sign = if (n * (n - 1) / 2) % 2 == 0 { 1 } else { -1 };
                (s::simplify(&div(mul(vec![int(sign), res]), cs[n].clone())), "Δ = (−1)^(n(n−1)/2) · Res(p, p′) / aₙ".into())
            }
        };
        let mut pods = vec![primary(pod("Result", "Discriminant", math(&d))), pod("Formula", "Formula", text(formula))];
        if let Some(dv) = d.to_f64_const() {
            let nature = match (n, dv) {
                (_, v) if v == 0.0 => "The discriminant is zero, so the polynomial has a repeated root.",
                (2, v) if v > 0.0 => "The discriminant is positive, so there are two distinct real roots.",
                (2, _) => "The discriminant is negative, so the roots are a pair of complex conjugates.",
                (3, v) if v > 0.0 => "The discriminant is positive, so there are three distinct real roots.",
                (3, _) => "The discriminant is negative, so there is one real root and a pair of complex conjugate roots.",
                (_, _) => "The discriminant is nonzero, so all roots are distinct.",
            };
            pods.push(pod("Nature", "Nature of the roots", text(nature)));
            if n == 2 && dv > 0.0 {
                if let Some(r) = d.as_int().filter(|i| nt::is_square(i)) {
                    pods.push(pod("Rational", "Rational roots", text(format!("{r} is a perfect square, so the roots are rational when the coefficients are."))));
                }
            }
        }
        let roots = solve::solve(&eq(e.clone(), zero()), &v);
        let all: Vec<Expr> = roots.real().into_iter().chain(roots.complex()).collect();
        if !all.is_empty() && all.len() <= 8 {
            pods.push(pod("Roots", "Roots", table(None, all.iter().map(|r| vec![math(&eq(v.clone(), r.clone()))]).collect())));
        }
        return answer(format!("discriminant of {}", ptext(&e)), format!("discriminant of {}", ptext(&e)), pods);
    }
    // irreducibility
    if let Some(m) = cap(re!(r"^is\s+(.+?)\s+(irreducible|reducible|factorable|factorizable|factorisable|prime)(?:\s+over\s+(?:the\s+)?(?:rationals|integers|rational numbers|q|z))?$")) {
        let e = s::expand(&fx(&g(&m, 1))?);
        if e.free_symbols().len() != 1 {
            return None;
        }
        let v = main_var(&e);
        let p = poly::to_qpoly(&e, &v)?;
        if p.degree() < 1 {
            return None;
        }
        let irreducible = poly::is_irreducible(&p);
        let wants_reducible = matches!(g(&m, 2).to_lowercase().as_str(), "reducible" | "factorable" | "factorizable" | "factorisable");
        let yes = irreducible != wants_reducible;
        let (content, factors) = poly::factor(&p);
        let mut fs: Vec<Expr> = Vec::new();
        if !content.is_one() {
            fs.push(num(content));
        }
        fs.extend(factors.iter().map(|(f, k)| pow(f.to_expr(&v), int(*k as i64))));
        let mut pods = vec![primary(pod("Result", "Result", text(format!("{} — {} is {} over the rational numbers", if yes { "Yes" } else { "No" }, ptext(&e), if irreducible { "irreducible" } else { "reducible" }))))];
        if !irreducible {
            pods.push(pod("Factorization", "Factorization", math(&eq(e.clone(), raw_fn("Times", fs.clone())))).clone());
            pods.pop();
            pods.push(pod("Factorization", "Factorization", text(fs.iter().map(|f| if matches!(f, Add(_)) { format!("({})", ptext(f)) } else { ptext(f) }).collect::<Vec<_>>().join(" "))));
        } else {
            let roots = solve::solve(&eq(e.clone(), zero()), &v);
            let cx = roots.complex();
            let re_roots = roots.real();
            if p.degree() >= 2 && !(cx.is_empty() && re_roots.is_empty()) {
                let note = if re_roots.is_empty() { "It has no real roots; it factors only over the complex numbers." } else { "It has real roots, but they are irrational, so it factors over the reals but not over the rationals." };
                pods.push(pod("Note", "Over larger number systems", text(note)));
                pods.push(pod("Roots", "Roots", table(None, re_roots.iter().chain(cx.iter()).take(8).map(|r| vec![math(&eq(v.clone(), r.clone()))]).collect())));
            }
        }
        let label = format!("is {} {}?", ptext(&e), g(&m, 2).to_lowercase());
        return answer(label.clone(), label, pods);
    }
    // slope and intercepts of a line given by an equation
    if let Some(m) = cap(re!(r"^(?:the\s+)?(slope and y-?\s?intercept|slope and intercepts?|slope|gradient|y-?\s?intercept|x-?\s?intercept|intercepts|slope-?\s?intercept form)\s+(?:of|for)\s+(?:the\s+line\s+)?(.+=.+)$")) {
        let e = ex(&g(&m, 2))?;
        if !e.free_symbols().iter().all(|n| matches!(&**n, "x" | "y")) {
            return None;
        }
        let (a, b, c0) = line_coeffs(&as_zero_form(&e), &x, &y)?;
        let what = g(&m, 1).to_lowercase();
        let mut pods = Vec::new();
        if b.is_zero() {
            if a.is_zero() {
                return None;
            }
            let x0 = s::simplify(&neg(div(c0.clone(), a.clone())));
            pods.push(primary(pod("Result", "Slope", text("undefined — the line is vertical"))));
            pods.push(pod("XIntercept", "x-intercept", math(&point(&x0, &zero()))));
        } else {
            let slope = s::simplify(&neg(div(a.clone(), b.clone())));
            let yint = s::simplify(&neg(div(c0.clone(), b.clone())));
            let form = eq(y.clone(), add(vec![mul(vec![slope.clone(), x.clone()]), yint.clone()]));
            let slope_pod = pod("Slope", "Slope", math(&slope));
            let y_pod = pod("YIntercept", "y-intercept", math(&point(&zero(), &yint)));
            let x_pod = (!a.is_zero()).then(|| pod("XIntercept", "x-intercept", math(&point(&s::simplify(&neg(div(c0.clone(), a.clone()))), &zero()))));
            let form_pod = pod("Form", "Slope-intercept form", math(&form));
            let lead = if what.starts_with("y") { "YIntercept" } else if what.starts_with("x") { "XIntercept" } else if what.contains("form") { "Form" } else if what == "intercepts" { "XIntercept" } else { "Slope" };
            for p in [Some(slope_pod), Some(y_pod), x_pod, Some(form_pod)].into_iter().flatten() {
                if p.id == lead {
                    pods.insert(0, primary(p));
                } else {
                    pods.push(p);
                }
            }
            if let Some(ang) = slope.to_f64_const() {
                pods.push(pod("Angle", "Angle of inclination", text(format!("{}°", crate::print::fmt_float(ang.atan().to_degrees(), 6)))));
            }
            if let Some(p) = plot_pod(&[add(vec![mul(vec![slope, x.clone()]), yint])], &x, None) {
                pods.push(p);
            }
        }
        let label = format!("{what} of the line {}", ptext(&e));
        return answer(label.clone(), label, pods);
    }
    // line through a point with a given slope, or parallel / perpendicular to a line
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:equation of\s+)?(?:the\s+|a\s+)?line\s+(?:passing\s+|that passes\s+|going\s+)?through\s+(?:the point\s+)?\(\s*([^,()]+)\s*,\s*([^,()]+)\s*\)\s*(?:,\s*)?(?:(?:with|and|having|of)\s+(?:a\s+)?(?:slope|gradient)\s*(?:of|=|is)?\s*(.+)|(?:and\s+|that is\s+)?(parallel|perpendicular|normal)\s+to\s+(?:the line\s+)?(.+))$")) {
        let (x0, y0) = (ex(&g(&m, 1))?, ex(&g(&m, 2))?);
        let (slope, how) = if !g(&m, 3).is_empty() {
            (Some(ex(&g(&m, 3))?), "the given slope".to_string())
        } else {
            let other = ex(&g(&m, 5))?;
            let other = if matches!(other, Rel(..)) { as_zero_form(&other) } else { sub(other, y.clone()) };
            let (a, b, _) = line_coeffs(&s::expand(&other), &x, &y)?;
            let para = g(&m, 4).to_lowercase() == "parallel";
            if para {
                ((!b.is_zero()).then(|| s::simplify(&neg(div(a.clone(), b.clone())))), "parallel lines have equal slopes".to_string())
            } else {
                ((!a.is_zero()).then(|| s::simplify(&div(b.clone(), a.clone()))), "perpendicular slopes multiply to −1".to_string())
            }
        };
        let mut pods = Vec::new();
        match slope {
            Some(mv) => {
                let rhs = s::expand(&add(vec![mul(vec![mv.clone(), sub(x.clone(), x0.clone())]), y0.clone()]));
                pods.push(primary(pod("Result", "Equation of the line", math(&eq(y.clone(), rhs.clone())))));
                pods.push(pod("PointSlope", "Point-slope form", math(&eq(sub(y.clone(), y0.clone()), mul(vec![mv.clone(), raw_fn("Paren", vec![sub(x.clone(), x0.clone())])]))).clone()));
                pods.pop();
                pods.push(pod("PointSlope", "Point-slope form", text(format!("y − ({}) = {} (x − ({}))", ptext(&y0), ptext(&mv), ptext(&x0)))));
                pods.push(pod("Slope", "Slope", text(format!("{}  ({how})", ptext(&mv)))));
                let yint = s::simplify(&sub(y0.clone(), mul(vec![mv.clone(), x0.clone()])));
                pods.push(pod("Intercepts", "Intercepts", table(None, vec![vec![text("y-intercept"), math(&point(&zero(), &yint))], vec![text("x-intercept"), if mv.is_zero() { text("none") } else { math(&point(&s::simplify(&neg(div(yint.clone(), mv.clone()))), &zero())) }]])));
                if let Some(p) = plot_pod(&[rhs], &x, None) {
                    pods.push(p);
                }
            }
            None => {
                pods.push(primary(pod("Result", "Equation of the line", math(&eq(x.clone(), x0.clone())))));
                pods.push(pod("Slope", "Slope", text("undefined — the line is vertical")));
            }
        }
        let label = format!("line through ({}, {})", ptext(&x0), ptext(&y0));
        return answer(label.clone(), format!("{label} | {}", if g(&m, 3).is_empty() { format!("{} to {}", g(&m, 4), g(&m, 5)) } else { format!("slope {}", g(&m, 3)) }), pods);
    }
    // circles
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:equation of\s+)?(?:a\s+|the\s+)?circle\s+(?:with\s+|of\s+|having\s+)?cent(?:er|re)\s*(?:at\s+)?\(\s*([^,()]+)\s*,\s*([^,()]+)\s*\)\s*(?:,|and|with)?\s*(?:a\s+)?(radius|diameter)\s*(?:of\s+|=\s*)?(.+)$")) {
        let (h, k) = (ex(&g(&m, 1))?, ex(&g(&m, 2))?);
        let mut r = ex(&g(&m, 4))?;
        if g(&m, 3).to_lowercase() == "diameter" {
            r = s::simplify(&div(r, int(2)));
        }
        let label = format!("circle with center ({}, {}) and radius {}", ptext(&h), ptext(&k), ptext(&r));
        return answer(label.clone(), label, circle_pods(&h, &k, &s::simplify(&square(r))));
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:equation of\s+)?(?:a\s+|the\s+)?circle\s+(?:passing\s+|that passes\s+)?through\s+(?:the points\s+)?(.+)$")) {
        let pts = points_in(&g(&m, 1));
        if pts.len() != 3 {
            return None;
        }
        let sq = |p: &(Expr, Expr)| add(vec![square(p.0.clone()), square(p.1.clone())]);
        let ((x1, y1), (x2, y2), (x3, y3)) = (pts[0].clone(), pts[1].clone(), pts[2].clone());
        let d = s::simplify(&mul(vec![int(2), add(vec![mul(vec![x1.clone(), sub(y2.clone(), y3.clone())]), mul(vec![x2.clone(), sub(y3.clone(), y1.clone())]), mul(vec![x3.clone(), sub(y1.clone(), y2.clone())])])]));
        let label = format!("circle through {}", pts.iter().map(|p| format!("({}, {})", ptext(&p.0), ptext(&p.1))).collect::<Vec<_>>().join(", "));
        if d.is_zero() {
            return answer(label.clone(), label, vec![primary(pod("Result", "Result", text("The three points lie on a line, so no circle passes through all of them.")))]);
        }
        let h = s::simplify(&div(add(vec![mul(vec![sq(&pts[0]), sub(y2.clone(), y3.clone())]), mul(vec![sq(&pts[1]), sub(y3.clone(), y1.clone())]), mul(vec![sq(&pts[2]), sub(y1.clone(), y2.clone())])]), d.clone()));
        let k = s::simplify(&div(add(vec![mul(vec![sq(&pts[0]), sub(x3.clone(), x2.clone())]), mul(vec![sq(&pts[1]), sub(x1.clone(), x3.clone())]), mul(vec![sq(&pts[2]), sub(x2.clone(), x1.clone())])]), d));
        let r2 = s::simplify(&add(vec![square(sub(x1, h.clone())), square(sub(y1, k.clone()))]));
        return answer(label.clone(), label, circle_pods(&h, &k, &r2));
    }
    // center and radius from an equation
    if let Some(m) = cap(re!(r"^(?:the\s+)?(center and radius|centre and radius|radius and center|center|centre|radius)\s+of\s+(?:the\s+circle\s+)?(.+=.+)$")) {
        let e = as_zero_form(&ex(&g(&m, 2))?);
        let cx = poly::coeffs(&e, &x)?;
        if cx.len() != 3 || cx[2].has_sym(&y) || cx[1].has_sym(&y) {
            return None;
        }
        let cy = poly::coeffs(&cx[0], &y)?;
        if cy.len() != 3 || !zero_expr(&sub(cy[2].clone(), cx[2].clone())) {
            return None;
        }
        let a = cx[2].clone();
        let h = s::simplify(&neg(div(cx[1].clone(), mul(vec![int(2), a.clone()]))));
        let k = s::simplify(&neg(div(cy[1].clone(), mul(vec![int(2), a.clone()]))));
        let r2 = s::simplify(&sub(add(vec![square(h.clone()), square(k.clone())]), div(cy[0].clone(), a)));
        if r2.to_f64_const().is_some_and(|v| v <= 0.0) {
            return None;
        }
        let mut pods = circle_pods(&h, &k, &r2);
        let what = g(&m, 1).to_lowercase();
        let lead = if what == "radius" { "Radius" } else { "Center" };
        pods.iter_mut().for_each(|p| p.primary = p.id == lead);
        if what.contains("and") {
            pods.insert(0, primary(pod("Summary", "Center and radius", text(format!("center ({}, {}), radius {}", ptext(&h), ptext(&k), ptext(&s::simplify(&sqrt(r2.clone()))))))));
            pods.iter_mut().skip(1).for_each(|p| p.primary = false);
        }
        let label = format!("{what} of the circle {}", g(&m, 2));
        return answer(label.clone(), label, pods);
    }
    // composition of two functions
    if let Some(m) = cap(re!(r"^(?:compose|composition of|compositions of)\s+(?:the functions\s+)?([a-z])\s*\(\s*([a-z])\s*\)\s*=\s*(.+?)\s*(?:and|with|,|;)\s*([a-z])\s*\(\s*([a-z])\s*\)\s*=\s*(.+)$"))
        .or_else(|| cap(re!(r"^(?:find\s+)?(?:[a-z]\s*\(\s*[a-z]\s*\(\s*[a-z]\s*\)\s*\)|\(?[a-z]\s*(?:o|∘|°)\s*[a-z]\)?(?:\s*\(\s*[a-z]\s*\))?)\s*(?:,|where|if|for|when|given)\s*([a-z])\s*\(\s*([a-z])\s*\)\s*=\s*(.+?)\s*(?:and|,|;)\s*([a-z])\s*\(\s*([a-z])\s*\)\s*=\s*(.+)$")))
    {
        let (fname, gname) = (g(&m, 1), g(&m, 4));
        let (fv, gv) = (sym(&g(&m, 2)), sym(&g(&m, 5)));
        let (f, gg) = (ex(&g(&m, 3))?, ex(&g(&m, 6))?);
        let fog = s::simplify(&f.subs(&fv, &gg));
        let gof = s::simplify(&gg.subs(&gv, &f.subs(&fv, &gv)));
        let var = ptext(&gv);
        let mut pods = vec![
            primary(pod("Result", &format!("{fname}({gname}({var}))"), math(&fog))),
            pod("Other", &format!("{gname}({fname}({var}))"), math(&gof)),
        ];
        let ef = s::expand(&fog);
        if ptext(&ef) != ptext(&fog) {
            pods.push(pod("Expanded", &format!("{fname}({gname}({var})) expanded"), math(&ef)));
        }
        pods.push(pod("Commute", "Do they commute?", text(if zero_expr(&sub(fog.clone(), gof.clone())) { "Yes — both compositions are the same function." } else { "No — the two compositions are different functions." })));
        if let Some(p) = plot_pod(&[fog, gof], &gv, None) {
            pods.push(p);
        }
        let label = format!("compositions of {fname}({var}) = {} and {gname}({var}) = {}", ptext(&f.subs(&fv, &gv)), ptext(&gg));
        return answer(label.clone(), label, pods);
    }
    None
}

// ---------------------------------------------------------------------------
// calculus
// ---------------------------------------------------------------------------

pub fn calculus_q(q: &Query, l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);

    // "integrate f dx dy x=0..1 y=0..2": add the separator the general rule expects
    if let Some(m) = cap(re!(r"^((?:double\s+)?integra(?:l|te)(?:\s+of)?\s+.+?\s+d[a-z]\s*d[a-z])\s*,?\s+([a-z]\s*=\s*\S+?\s*(?:\.\.|to)\s*\S+?)\s+([a-z]\s*=\s*.+)$")) {
        return again(q, &format!("{}, {}, {}", g(&m, 1), g(&m, 2), g(&m, 3)));
    }
    // "tangent line to y = f at x = a"
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:equation of\s+)?(?:the\s+)?(tangent|normal)(?: line)?\s+(?:to|of)\s+(?:the curve\s+|the graph of\s+)?(?:y|[a-z]\s*\(\s*[a-z]\s*\))\s*=\s*(.+?)\s+(?:at|when|where)\s+(?:the point\s+)?(?:where\s+)?([a-z])\s*=\s*(.+)$")) {
        let f = ex(&g(&m, 2))?;
        let x = sym(&g(&m, 3));
        let a = ex(&g(&m, 4))?;
        let d = calculus::diff::diff(&f, &x);
        let (y0, slope) = (s::simplify(&f.subs(&x, &a)), s::simplify(&d.subs(&x, &a)));
        let normal = g(&m, 1).to_lowercase() == "normal";
        let mv = if normal {
            if slope.is_zero() {
                let label = format!("normal line to y = {} at {} = {}", ptext(&f), ptext(&x), ptext(&a));
                return answer(label.clone(), label, vec![primary(pod("Result", "Normal line", math(&eq(x.clone(), a.clone())))), pod("Note", "Note", text("The tangent is horizontal there, so the normal line is vertical."))]);
            }
            s::simplify(&neg(div(one(), slope.clone())))
        } else {
            slope.clone()
        };
        let line = s::expand(&add(vec![y0.clone(), mul(vec![mv.clone(), sub(x.clone(), a.clone())])]));
        let mut pods = vec![primary(pod("Result", if normal { "Normal line" } else { "Tangent line" }, math(&eq(sym("y"), line.clone()))))];
        pods.push(pod("Point", "Point of tangency", math(&point(&a, &y0))));
        pods.push(pod("Derivative", "Derivative", math(&eq(raw_fn("Derivative", vec![f.clone(), x.clone()]), d.clone()))));
        pods.push(pod("Slope", if normal { "Slope of the tangent and of the normal" } else { "Slope at the point" }, if normal { text(format!("{} and {}", ptext(&slope), ptext(&mv))) } else { math(&slope) }));
        if let Some(af) = a.to_f64_const() {
            if let Some(p) = plot_pod(&[f.clone(), line], &x, Some((af - 3.0, af + 3.0))) {
                pods.push(p);
            }
        }
        let label = format!("{} line to y = {} at {} = {}", g(&m, 1).to_lowercase(), ptext(&f), ptext(&x), ptext(&a));
        return answer(label.clone(), label, pods);
    }
    // implicit differentiation
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:implicit (?:derivative|differentiation)(?:\s+of)?|d\s*y\s*/\s*d\s*x\s+(?:of|for|if|when|given|where)|find\s+d\s*y\s*/\s*d\s*x\s*(?:of|for|if|when|given|where)?|differentiate implicitly|implicitly differentiate)\s*:?\s+(.+=.+)$")) {
        let e = ex(&g(&m, 1))?;
        let (x, y) = (sym("x"), sym("y"));
        if !(e.has_sym(&x) && e.has_sym(&y)) {
            return None;
        }
        let d = solve::implicit_derivative(&e, &x, &y)?;
        let d = s::simplify(&d);
        let fz = as_zero_form(&e);
        let (fx_, fy_) = (calculus::diff::diff(&fz, &x), calculus::diff::diff(&fz, &y));
        let mut pods = vec![primary(pod("Result", "Implicit derivative", math(&eq(raw_fn("user:dy/dx", vec![]), d.clone())))).clone()];
        pods.clear();
        pods.push(primary(pod("Result", "Implicit derivative dy/dx", math(&d))));
        pods.push(pod(
            "Steps",
            "Step-by-step solution",
            steps_content(&[
                crate::steps::Step { text: "Write the equation as F(x, y) = 0:".into(), math: Some(eq(fz.clone(), zero())), depth: 0 },
                crate::steps::Step { text: "Differentiate both sides with respect to x, treating y as a function of x (chain rule): F_x + F_y · dy/dx = 0.".into(), math: None, depth: 0 },
                crate::steps::Step { text: "Partial derivative with respect to x:".into(), math: Some(fx_.clone()), depth: 1 },
                crate::steps::Step { text: "Partial derivative with respect to y:".into(), math: Some(fy_.clone()), depth: 1 },
                crate::steps::Step { text: "Solve for dy/dx = −F_x / F_y:".into(), math: Some(d.clone()), depth: 0 },
            ]),
        ));
        let label = format!("implicit derivative dy/dx of {}", ptext(&e));
        return answer(label.clone(), label, pods);
    }
    // curvature
    if let Some(m) = cap(re!(r"^(?:the\s+)?(radius of curvature|curvature)\s+of\s+(.+?)(?:\s+(?:at|when)\s+(?:the point\s+)?(?:[a-z]\s*=\s*)?(.+))?$")) {
        let f = fx(&g(&m, 2))?;
        if f.free_symbols().len() != 1 {
            return None;
        }
        let x = main_var(&f);
        let (d1, d2) = (calculus::diff::diff(&f, &x), calculus::diff::diff_n(&f, &x, 2));
        let kappa = s::simplify(&div(f1("abs", d2.clone()), pow(add(vec![one(), square(d1.clone())]), rat(3, 2))));
        let radius = g(&m, 1).to_lowercase().starts_with("radius");
        let mut pods = Vec::new();
        let general = pod("General", "Curvature as a function", math(&eq(raw_fn("user:κ", vec![x.clone()]), kappa.clone())));
        if g(&m, 3).is_empty() {
            let mut gp = general;
            gp.primary = true;
            pods.push(gp);
        } else {
            let a = ex(&g(&m, 3))?;
            let k = s::simplify(&kappa.subs(&x, &a));
            let value = if radius {
                if k.is_zero() { None } else { Some(s::simplify(&div(one(), k.clone()))) }
            } else {
                Some(k.clone())
            };
            match value {
                Some(v) => pods.extend(value_pods(if radius { "Radius of curvature" } else { "Curvature" }, None, &v)),
                None => pods.push(primary(pod("Result", "Radius of curvature", text("infinite — the curvature is zero at this point")))),
            }
            if radius {
                pods.push(pod("Curvature", "Curvature there", math(&k)));
            } else if !k.is_zero() {
                pods.push(pod("Radius", "Radius of curvature there", math(&s::simplify(&div(one(), k.clone())))));
            }
            pods.push(general);
        }
        pods.push(pod("Formula", "Formula", text("κ = |y″| / (1 + y′²)^(3/2)")));
        pods.push(pod("Derivatives", "Derivatives", table(None, vec![vec![text("y′"), math(&d1)], vec![text("y″"), math(&d2)]])));
        let label = format!("{} of y = {}{}", g(&m, 1).to_lowercase(), ptext(&f), if g(&m, 3).is_empty() { String::new() } else { format!(" at {} = {}", ptext(&x), g(&m, 3)) });
        return answer(label.clone(), label, pods);
    }
    // Fourier series on [-π, π]
    if let Some(m) = cap(re!(r"^(?:the\s+)?fourier (?:series|expansion|coefficients)\s+(?:of|for)\s+(.+?)(?:\s+(?:on|over|in|for)\s+(?:the interval\s+)?[\[(]?\s*-\s*(?:pi|π)\s*(?:,|to|\.\.)\s*(?:pi|π)\s*[\])]?)?$")) {
        let f = fx(&g(&m, 1))?;
        if f.free_symbols().len() > 1 {
            return None;
        }
        let x = if f.free_symbols().is_empty() { sym("x") } else { main_var(&f) };
        let integral = |h: Expr| definite_expr(&h, &x, &neg(pi()), &pi()).map(|v| s::simplify(&v));
        let a0 = s::simplify(&div(integral(f.clone())?, mul(vec![int(2), pi()])));
        let mut terms = vec![a0.clone()];
        let mut rows = vec![vec![text("a₀ (mean value)"), math(&a0), text("")]];
        const N: i64 = 6;
        for n in 1..=N {
            let an = s::simplify(&div(integral(mul(vec![f.clone(), f1("cos", mul(vec![int(n), x.clone()]))]))?, pi()));
            let bn = s::simplify(&div(integral(mul(vec![f.clone(), f1("sin", mul(vec![int(n), x.clone()]))]))?, pi()));
            rows.push(vec![text(format!("n = {n}")), math(&an), math(&bn)]);
            if !an.is_zero() {
                terms.push(mul(vec![an, f1("cos", mul(vec![int(n), x.clone()]))]));
            }
            if !bn.is_zero() {
                terms.push(mul(vec![bn, f1("sin", mul(vec![int(n), x.clone()]))]));
            }
        }
        let partial = add(terms);
        let mut pods = vec![primary(pod("Result", &format!("Fourier series on [−π, π] (terms up to n = {N})"), math(&add(vec![partial.clone(), raw_fn("user:…", vec![])])))).clone()];
        pods.clear();
        pods.push(primary(pod("Result", &format!("Fourier series on [−π, π], terms up to n = {N}"), math(&partial))));
        pods.push(pod("Coefficients", "Coefficients", table(Some(vec!["", "aₙ (cosine)", "bₙ (sine)"]), rows)));
        let even = zero_expr(&sub(f.subs(&x, &neg(x.clone())), f.clone()));
        let odd = zero_expr(&add(vec![f.subs(&x, &neg(x.clone())), f.clone()]));
        if even || odd {
            pods.push(pod("Symmetry", "Symmetry", text(if even { "The function is even, so the series contains only cosine terms." } else { "The function is odd, so the series contains only sine terms." })));
        }
        pods.push(pod("Formula", "Formulas", text("a₀ = (1/2π) ∫ f dx,   aₙ = (1/π) ∫ f cos(n x) dx,   bₙ = (1/π) ∫ f sin(n x) dx,   all over [−π, π]")));
        if let Some(p) = pl::cartesian(&[f.clone(), partial], &x, Some((-std::f64::consts::PI, std::f64::consts::PI))) {
            pods.push(pod("Plot", "Function and partial sum", plot(p)));
        }
        let label = format!("Fourier series of {} on [−π, π]", ptext(&f));
        return answer(label.clone(), label, pods);
    }
    None
}

// ---------------------------------------------------------------------------
// matrices
// ---------------------------------------------------------------------------

fn mat_eq(a: &Mat, b: &Mat) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(r1, r2)| r1.len() == r2.len() && r1.iter().zip(r2).all(|(p, q)| zero_expr(&sub(p.clone(), q.clone()))))
}

/// `P·A = L·U` with exact arithmetic; rows are swapped only when a pivot is zero.
fn lu(a: &Mat) -> (Mat, Mat, Mat, bool) {
    let n = a.len();
    let mut u = a.clone();
    let mut l = linalg::identity(n);
    let mut p = linalg::identity(n);
    let mut swapped = false;
    for k in 0..n.min(a[0].len()) {
        let Some(r) = (k..n).find(|&r| !zero_expr(&u[r][k])) else { continue };
        if r != k {
            u.swap(r, k);
            p.swap(r, k);
            for j in 0..k {
                let t = l[r][j].clone();
                l[r][j] = l[k][j].clone();
                l[k][j] = t;
            }
            swapped = true;
        }
        for i in k + 1..n {
            let f = s::simplify(&div(u[i][k].clone(), u[k][k].clone()));
            if f.is_zero() {
                continue;
            }
            l[i][k] = f.clone();
            for j in 0..u[i].len() {
                u[i][j] = s::simplify(&sub(u[i][j].clone(), mul(vec![f.clone(), u[k][j].clone()])));
            }
        }
    }
    (p, l, u, swapped)
}

pub fn matrices(q: &Query, l: &str, c: &str) -> Option<Answer> {
    let cap = |r: &Regex| caps(r, l, c);
    let _ = q;
    if !(l.contains("{{") || l.contains("[[")) {
        return None;
    }
    // LU decomposition
    if let Some(m) = cap(re!(r"^(?:the\s+)?lu[- ]?(?:decomposition|factorization|factorisation|decompose|factor)?\s+(?:of\s+)?(.+)$")) {
        let a = mat_of(&g(&m, 1))?;
        if a.len() != a[0].len() {
            return None;
        }
        let (p, lo, up, swapped) = lu(&a);
        let mut pods = vec![primary(pod_multi("Result", "LU decomposition", vec![("L (unit lower triangular)".to_string(), math(&linalg::to_expr(&lo))), ("U (upper triangular)".to_string(), math(&linalg::to_expr(&up)))]))];
        if swapped {
            pods.push(pod("Permutation", "Row permutation P (so that P·A = L·U)", math(&linalg::to_expr(&p))));
        }
        let check = linalg::matmul(&lo, &up)?;
        let target = linalg::matmul(&p, &a)?;
        pods.push(pod("Check", "Verification", text(if mat_eq(&check, &target) { if swapped { "L·U equals P·A ✓" } else { "L·U equals A ✓" } } else { "verification failed" })));
        pods.push(pod("Determinant", "Determinant (product of the pivots, with the sign of the permutation)", math(&linalg::det(&a))));
        let label = format!("LU decomposition of {}", ptext(&linalg::to_expr(&a)));
        return answer(label.clone(), label, pods);
    }
    // property tests
    if let Some(m) = cap(re!(r"^is\s+(?:the\s+matrix\s+)?(.+?)\s+(?:a\s+|an\s+)?(symmetric|skew-?\s?symmetric|antisymmetric|invertible|non-?singular|singular|orthogonal|diagonal|diagonali[sz]able|positive definite|positive semi-?definite|idempotent|nilpotent|upper triangular|lower triangular|triangular|the identity|identity|square|involutory|stochastic)(?:\s+matrix)?$")) {
        let a = mat_of(&g(&m, 1))?;
        let (n, cols) = (a.len(), a[0].len());
        let sq = n == cols;
        let prop = g(&m, 2).to_lowercase();
        let t = linalg::transpose(&a);
        let ident = linalg::identity(n);
        let zero_m: Mat = vec![vec![zero(); cols]; n];
        let minors_positive = |strict: bool| -> Option<bool> {
            if !sq || !mat_eq(&a, &t) {
                return Some(false);
            }
            for k in 1..=n {
                let sub_m: Mat = a.iter().take(k).map(|r| r.iter().take(k).cloned().collect()).collect();
                let d = linalg::det(&sub_m).to_f64_const()?;
                if (strict && d <= 0.0) || (!strict && d < 0.0) {
                    return Some(false);
                }
            }
            Some(true)
        };
        let (yes, why): (bool, String) = match prop.as_str() {
            "square" => (sq, format!("it has {n} rows and {cols} columns")),
            "symmetric" => (sq && mat_eq(&a, &t), "a matrix is symmetric when it equals its transpose".into()),
            p if p.starts_with("skew") || p == "antisymmetric" => (sq && mat_eq(&t, &a.iter().map(|r| r.iter().map(|e| neg(e.clone())).collect()).collect()), "a matrix is skew-symmetric when its transpose equals its negative".into()),
            "invertible" | "nonsingular" | "non-singular" | "non singular" => (sq && !zero_expr(&linalg::det(&a)), if sq { format!("its determinant is {}", ptext(&linalg::det(&a))) } else { "only square matrices can be invertible".into() }),
            "singular" => (sq && zero_expr(&linalg::det(&a)), if sq { format!("its determinant is {}", ptext(&linalg::det(&a))) } else { "the matrix is not square".into() }),
            "orthogonal" => (sq && linalg::matmul(&t, &a).is_some_and(|p| mat_eq(&p, &ident)), "a matrix is orthogonal when AᵀA is the identity".into()),
            "diagonal" => (sq && (0..n).all(|i| (0..n).all(|j| i == j || zero_expr(&a[i][j]))), "all entries off the main diagonal must be zero".into()),
            "upper triangular" => (sq && (0..n).all(|i| (0..i).all(|j| zero_expr(&a[i][j]))), "all entries below the main diagonal must be zero".into()),
            "lower triangular" => (sq && (0..n).all(|i| (i + 1..n).all(|j| zero_expr(&a[i][j]))), "all entries above the main diagonal must be zero".into()),
            "triangular" => (sq && ((0..n).all(|i| (0..i).all(|j| zero_expr(&a[i][j]))) || (0..n).all(|i| (i + 1..n).all(|j| zero_expr(&a[i][j])))), "all entries on one side of the main diagonal must be zero".into()),
            "identity" | "the identity" => (sq && mat_eq(&a, &ident), "ones on the diagonal and zeros elsewhere".into()),
            "idempotent" => (sq && linalg::matmul(&a, &a).is_some_and(|p| mat_eq(&p, &a)), "a matrix is idempotent when A² = A".into()),
            "involutory" => (sq && linalg::matmul(&a, &a).is_some_and(|p| mat_eq(&p, &ident)), "a matrix is involutory when A² = I".into()),
            "nilpotent" => (sq && linalg::matpow(&a, n as i64).is_some_and(|p| mat_eq(&p, &zero_m)), format!("a matrix is nilpotent when some power is zero (here A^{n} is checked)")),
            "stochastic" => (a.iter().all(|r| r.iter().all(|e| e.to_f64_const().is_some_and(|v| v >= 0.0)) && zero_expr(&sub(add(r.clone()), one()))), "every entry must be non-negative and every row must sum to 1".into()),
            p if p.starts_with("diagonali") => {
                let vecs = if sq { linalg::eigenvectors(&a).len() } else { 0 };
                (sq && vecs == n, format!("it has {vecs} linearly independent eigenvector{} (a {n}×{n} matrix needs {n})", if vecs == 1 { "" } else { "s" }))
            }
            "positive definite" => (minors_positive(true)?, "a symmetric matrix is positive definite when all leading principal minors are positive (Sylvester's criterion)".into()),
            _ => {
                let ev = if sq && mat_eq(&a, &t) { linalg::eigenvalues(&a) } else { vec![] };
                (!ev.is_empty() && ev.iter().all(|(v, _)| v.to_f64_const().is_some_and(|x| x >= -1e-12)), "a symmetric matrix is positive semidefinite when all eigenvalues are non-negative".into())
            }
        };
        let label = format!("is {} {prop}?", ptext(&linalg::to_expr(&a)));
        let mut pods = vec![primary(pod("Result", "Result", text(format!("{} — the matrix {} {prop}", if yes { "Yes" } else { "No" }, if yes { "is" } else { "is not" })))), pod("Reason", "Reason", text(why))];
        if sq {
            pods.push(pod("Facts", "Related facts", table(None, vec![vec![text("determinant"), math(&linalg::det(&a))], vec![text("trace"), math(&linalg::trace(&a))], vec![text("rank"), text(linalg::rank(&a).to_string())], vec![text("transpose"), math(&linalg::to_expr(&t))]])));
        }
        return answer(label.clone(), label, pods);
    }
    // linear systems in matrix form: "solve {{1,2},{3,4}} x = {5,6}"
    if let Some(m) = cap(re!(r"^(?:solve\s+)?((?:\{\{|\[\[).+?(?:\}\}|\]\]))\s*[.*·]?\s*([a-z])?\s*=\s*([{\[(].+[}\])])$")) {
        let a = mat_of(&g(&m, 1))?;
        let b = vec_of(&g(&m, 3))?;
        if b.len() != a.len() {
            return None;
        }
        let label = format!("solve {} · x = {}", ptext(&linalg::to_expr(&a)), ptext(&vec_expr(&b)));
        let aug: Mat = a.iter().zip(&b).map(|(r, v)| r.iter().cloned().chain(std::iter::once(v.clone())).collect()).collect();
        let (ra, rb) = (linalg::rank(&a), linalg::rank(&aug));
        let cols = a[0].len();
        let mut pods = Vec::new();
        if ra < rb {
            pods.push(primary(pod("Result", "Result", text("The system is inconsistent: it has no solution."))));
        } else if ra == cols && a.len() == cols {
            let inv = linalg::inverse(&a)?;
            let bm: Mat = b.iter().map(|v| vec![v.clone()]).collect();
            let xs: Vec<Expr> = linalg::matmul(&inv, &bm)?.into_iter().map(|r| s::simplify(&r[0])).collect();
            pods.push(primary(pod("Result", "Solution", math(&eq(sym("x"), vec_expr(&xs))))));
            pods.push(pod("Components", "Components", table(None, xs.iter().enumerate().map(|(k, v)| vec![math(&eq(raw_fn("user:x", vec![int(k as i64 + 1)]), v.clone()))]).collect())).clone());
            pods.pop();
            pods.push(pod("Components", "Components", table(None, xs.iter().enumerate().map(|(k, v)| vec![text(format!("x{}", k + 1)), math(v)]).collect())));
            pods.push(pod("Inverse", "Using the inverse, x = A⁻¹ b", math(&linalg::to_expr(&inv))));
        } else {
            let (r, pivots) = linalg::rref(&aug);
            let free = cols - ra;
            pods.push(primary(pod("Result", "Result", text(if free > 0 { format!("The system has infinitely many solutions ({free} free parameter{}).", if free == 1 { "" } else { "s" }) } else { "The system has a unique solution.".to_string() }))));
            let _ = pivots;
            pods.push(pod("RREF", "Reduced augmented matrix", math(&linalg::to_expr(&r))));
        }
        pods.push(pod("Ranks", "Ranks", text(format!("rank of A = {ra},  rank of [A | b] = {rb},  unknowns = {cols}"))));
        return answer(label.clone(), label, pods);
    }
    None
}

// ---------------------------------------------------------------------------
// statistics
// ---------------------------------------------------------------------------

fn number_list(t: &str) -> Option<Vec<f64>> {
    let inner = t.trim().trim_start_matches(['{', '[', '(']).trim_end_matches(['}', ']', ')']);
    let parts: Vec<&str> = if inner.contains(',') { inner.split(',').collect() } else { inner.split_whitespace().collect() };
    let v: Option<Vec<f64>> = parts.iter().map(|p| p.trim().parse::<f64>().ok()).collect();
    v.filter(|v| v.len() >= 2)
}

fn median(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n % 2 == 1 { sorted[n / 2] } else { (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0 }
}

fn pr(v: f64) -> String {
    crate::print::fmt_float(v, 6)
}

fn word_count(t: &str) -> Option<u32> {
    Some(match t.trim() {
        "a" | "an" | "one" | "single" => 1,
        "two" | "double" | "a pair of" => 2,
        "three" | "triple" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        n => n.parse().ok()?,
    })
}

pub fn statistics(q: &Query, l: &str, c: &str) -> Option<Answer> {
    use statrs::distribution::{ChiSquared, ContinuousCDF, Normal, StudentsT};
    let cap = |r: &Regex| caps(r, l, c);
    let _ = q;
    let std = Normal::new(0.0, 1.0).ok()?;

    // quartiles and five-number summaries
    if let Some(m) = cap(re!(r"^(?:the\s+)?(quartiles|five[- ]number summary|5[- ]number summary|interquartile range|iqr|outliers|box ?plot|box and whisker plot|range|midrange)\s+(?:of|for)?\s*(.+)$")) {
        let mut xs = number_list(&g(&m, 2))?;
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = xs.len();
        let med = median(&xs);
        // quartiles as medians of the lower and upper halves (excluding the median for odd n)
        let (lower, upper) = (&xs[..n / 2], &xs[n.div_ceil(2)..]);
        let (q1, q3) = (median(lower), median(upper));
        let iqr = q3 - q1;
        let (lo_f, hi_f) = (q1 - 1.5 * iqr, q3 + 1.5 * iqr);
        let out: Vec<String> = xs.iter().filter(|v| **v < lo_f || **v > hi_f).map(|v| pr(*v)).collect();
        let what = g(&m, 1).to_lowercase();
        let five = table(None, vec![vec![text("minimum"), text(pr(xs[0]))], vec![text("first quartile Q₁"), text(pr(q1))], vec![text("median"), text(pr(med))], vec![text("third quartile Q₃"), text(pr(q3))], vec![text("maximum"), text(pr(xs[n - 1]))]]);
        let mut pods = Vec::new();
        match what.as_str() {
            "quartiles" => pods.push(primary(pod("Result", "Quartiles", text(format!("Q₁ = {},  Q₂ (median) = {},  Q₃ = {}", pr(q1), pr(med), pr(q3)))))),
            "interquartile range" | "iqr" => pods.push(primary(pod("Result", "Interquartile range", text(format!("{}  (Q₃ − Q₁ = {} − {})", pr(iqr), pr(q3), pr(q1)))))),
            "outliers" => pods.push(primary(pod("Result", "Outliers (1.5 × IQR rule)", text(if out.is_empty() { "none".to_string() } else { out.join(", ") })))),
            "range" => pods.push(primary(pod("Result", "Range", text(format!("{}  ({} − {})", pr(xs[n - 1] - xs[0]), pr(xs[n - 1]), pr(xs[0])))))),
            "midrange" => pods.push(primary(pod("Result", "Midrange", text(pr((xs[n - 1] + xs[0]) / 2.0))))),
            _ => {}
        }
        let mut fp = pod("FiveNumber", "Five-number summary", five);
        fp.primary = pods.is_empty();
        pods.push(fp);
        pods.push(pod("Spread", "Spread", table(None, vec![vec![text("interquartile range"), text(pr(iqr))], vec![text("range"), text(pr(xs[n - 1] - xs[0]))], vec![text("fences (1.5 × IQR)"), text(format!("{} to {}", pr(lo_f), pr(hi_f)))], vec![text("outliers"), text(if out.is_empty() { "none".to_string() } else { out.join(", ") })]])));
        pods.push(pod("Sorted", "Sorted data", text(xs.iter().map(|v| pr(*v)).collect::<Vec<_>>().join(", "))));
        pods.push(pod("Method", "Quartile method", text("Q₁ and Q₃ are the medians of the lower and upper halves of the sorted data (the overall median is left out when the count is odd).")));
        let label = format!("{what} of {{{}}}", xs.iter().map(|v| pr(*v)).collect::<Vec<_>>().join(", "));
        return answer(label.clone(), label, pods);
    }
    // z-scores and normal probabilities
    let zq = cap(re!(r"^(?:the\s+)?(?:p-?\s?value|probability|area)\s+(?:for|of|under|with)\s+(?:a\s+)?(?:z|z-?\s?score|z statistic|z-?stat)\s*(?:=|of|is)?\s*(-?[\d.]+)$"))
        .or_else(|| cap(re!(r"^z-?\s?score\s*(?:=|of|is)?\s*(-?[\d.]+)(?:\s+(?:to\s+)?(?:p-?\s?value|percentile|probability))?$")))
        .or_else(|| cap(re!(r"^(?:the\s+)?(?:percentile|area to the left)\s+(?:for|of)\s+(?:a\s+)?z(?:-?\s?score)?\s*(?:=|of|is)?\s*(-?[\d.]+)$")));
    if let Some(m) = zq {
        let z: f64 = g(&m, 1).parse().ok()?;
        let left = std.cdf(z);
        let two = 2.0 * (1.0 - std.cdf(z.abs()));
        let wants_p = l.contains("p-value") || l.contains("p value") || l.contains("pvalue");
        let rows = vec![
            vec![text(format!("P(Z < {z})  (left tail, percentile)")), text(format!("{}  ({}th percentile)", pr(left), pr(left * 100.0)))],
            vec![text(format!("P(Z > {z})  (right tail)")), text(pr(1.0 - left))],
            vec![text(format!("P(|Z| > {})  (two-tailed)", z.abs())), text(pr(two))],
            vec![text(format!("P(|Z| < {})  (central area)", z.abs())), text(pr(1.0 - two))],
        ];
        let mut pods = Vec::new();
        if wants_p {
            pods.push(primary(pod("Result", "p-value", text(format!("two-tailed: {}   ·   one-tailed: {}", pr(two), pr(two / 2.0))))));
            pods.push(pod("Decision", "Significance", text(format!("At the 5% level the result is {} (two-tailed).", if two < 0.05 { "statistically significant" } else { "not statistically significant" }))));
        } else {
            pods.push(primary(pod("Result", "Cumulative probability", text(format!("P(Z < {z}) = {}", pr(left))))));
        }
        pods.push(pod("Tails", "Standard normal probabilities", table(None, rows)));
        if let Some(p) = pl::cartesian(&[mul(vec![div(one(), sqrt(mul(vec![int(2), pi()]))), exp(neg(div(square(sym("z")), int(2))))])], &sym("z"), Some((-4.0, 4.0))) {
            pods.push(pod("Plot", "Standard normal density", plot(p)));
        }
        let label = format!("standard normal distribution | z = {z}");
        return answer(label.clone(), label, pods);
    }
    if let Some(m) = cap(re!(r"^p\s*\(\s*(?:(-?[\d.]+)\s*<=?\s*)?z\s*(<=?|>=?)\s*(-?[\d.]+)\s*\)$")) {
        let b: f64 = g(&m, 3).parse().ok()?;
        let less = g(&m, 2).starts_with('<');
        let (p, label) = if !g(&m, 1).is_empty() && less {
            let a: f64 = g(&m, 1).parse().ok()?;
            (std.cdf(b) - std.cdf(a), format!("P({a} < Z < {b})"))
        } else if less {
            (std.cdf(b), format!("P(Z < {b})"))
        } else {
            (1.0 - std.cdf(b), format!("P(Z > {b})"))
        };
        return answer(label.clone(), format!("{label} for the standard normal distribution"), vec![primary(pod("Result", "Probability", text(format!("{label} = {}", pr(p))))), pod("Percent", "As a percentage", text(format!("{}%", pr(p * 100.0))))]);
    }
    // z for a percentile
    if let Some(m) = cap(re!(r"^(?:the\s+)?z-?\s?score\s+(?:for|of|at)\s+(?:the\s+)?(\d+(?:\.\d+)?)(?:st|nd|rd|th)?\s*(?:%|percent(?:ile)?)(?:\s+percentile)?$")) {
        let p: f64 = g(&m, 1).parse::<f64>().ok()? / 100.0;
        if !(0.0 < p && p < 1.0) {
            return None;
        }
        let label = format!("z-score of the {}th percentile", g(&m, 1));
        return answer(label.clone(), label, vec![primary(pod("Result", "z-score", text(pr(std.inverse_cdf(p))))), pod("Meaning", "Meaning", text(format!("{}% of a normal population lies below this many standard deviations from the mean.", pr(p * 100.0))))]);
    }
    // p-values for t and chi-square statistics
    if let Some(m) = cap(re!(r"^(?:the\s+)?p-?\s?value\s+(?:for|of)\s+(?:a\s+)?(t|chi-?\s?squared?|χ²|x\^2)\s*(?:statistic\s*)?(?:=|of|is)?\s*(-?[\d.]+)\s*(?:,|with|and|for)?\s*(?:df|degrees of freedom|dof|d\.f\.)\s*(?:=|of|is)?\s*(\d+)$"))
        .or_else(|| cap(re!(r"^(?:the\s+)?p-?\s?value\s+(?:for|of)\s+(?:a\s+)?(t|chi-?\s?squared?|χ²|x\^2)\s*(?:statistic\s*)?(?:=|of|is)?\s*(-?[\d.]+)\s*(?:,|with|and|for)?\s*(\d+)\s*(?:df|degrees of freedom|dof)$")))
    {
        let (v, df): (f64, f64) = (g(&m, 2).parse().ok()?, g(&m, 3).parse().ok()?);
        if g(&m, 1).to_lowercase() == "t" {
            let d = StudentsT::new(0.0, 1.0, df).ok()?;
            let two = 2.0 * (1.0 - d.cdf(v.abs()));
            let label = format!("p-value for t = {v} with {df} degrees of freedom");
            return answer(label.clone(), label, vec![primary(pod("Result", "p-value", text(format!("two-tailed: {}   ·   one-tailed: {}", pr(two), pr(two / 2.0))))), pod("Decision", "Significance", text(format!("At the 5% level the result is {} (two-tailed).", if two < 0.05 { "statistically significant" } else { "not statistically significant" })))]);
        }
        let d = ChiSquared::new(df).ok()?;
        let p = 1.0 - d.cdf(v);
        let label = format!("p-value for χ² = {v} with {df} degrees of freedom");
        return answer(label.clone(), label, vec![primary(pod("Result", "p-value", text(format!("{}  (right tail)", pr(p))))), pod("Decision", "Significance", text(format!("At the 5% level the result is {}.", if p < 0.05 { "statistically significant" } else { "not statistically significant" })))]);
    }
    // critical values
    let crit = cap(re!(r"^(?:the\s+)?(chi-?\s?squared?|χ²|t|z)[- ]?(?:critical value|crit(?:ical)?)\s*(?:for|at|with)?\s*(?:alpha|α|significance(?: level)?)?\s*(?:=|of)?\s*([\d.]+)\s*(%)?\s*(?:,|with|and|for)?\s*(?:(?:df|degrees of freedom|dof)\s*(?:=|of)?\s*(\d+)|(\d+)\s*(?:df|degrees of freedom|dof))?$"))
        .or_else(|| cap(re!(r"^(?:the\s+)?critical value\s+(?:of|for)\s+(?:the\s+)?(chi-?\s?squared?|χ²|t|z)(?:[- ]distribution)?\s*(?:for|at|with)?\s*(?:alpha|α|significance(?: level)?)?\s*(?:=|of)?\s*([\d.]+)\s*(%)?\s*(?:,|with|and|for)?\s*(?:(?:df|degrees of freedom|dof)\s*(?:=|of)?\s*(\d+)|(\d+)\s*(?:df|degrees of freedom|dof))?$")));
    if let Some(m) = crit {
        let mut alpha: f64 = g(&m, 2).parse().ok()?;
        if !g(&m, 3).is_empty() || alpha >= 1.0 {
            alpha /= 100.0;
        }
        if alpha > 0.5 {
            alpha = 1.0 - alpha;
        }
        if !(0.0 < alpha && alpha < 1.0) {
            return None;
        }
        let df: Option<f64> = [g(&m, 4), g(&m, 5)].iter().find(|t| !t.is_empty()).and_then(|t| t.parse().ok());
        let kind = g(&m, 1).to_lowercase();
        let (label, rows): (String, Vec<(String, f64)>) = if kind == "z" {
            ("z critical values".into(), vec![("two-tailed".into(), std.inverse_cdf(1.0 - alpha / 2.0)), ("one-tailed".into(), std.inverse_cdf(1.0 - alpha))])
        } else if kind == "t" {
            let d = StudentsT::new(0.0, 1.0, df?).ok()?;
            (format!("t critical values with {} degrees of freedom", df?), vec![("two-tailed".into(), d.inverse_cdf(1.0 - alpha / 2.0)), ("one-tailed".into(), d.inverse_cdf(1.0 - alpha))])
        } else {
            let d = ChiSquared::new(df?).ok()?;
            (format!("χ² critical values with {} degrees of freedom", df?), vec![("right tail".into(), d.inverse_cdf(1.0 - alpha)), ("left tail".into(), d.inverse_cdf(alpha))])
        };
        let label = format!("{label} at α = {alpha}");
        let mut pods = vec![primary(pod("Result", "Critical value", text(format!("{}  ({})", pr(rows[0].1), rows[0].0))))];
        pods.push(pod("Both", "Critical values", table(None, rows.iter().map(|(k, v)| vec![text(k.clone()), text(if kind == "chi" || kind.starts_with('c') || kind == "χ²" { pr(*v) } else if k.starts_with("two") { format!("±{}", pr(*v)) } else { pr(*v) })]).collect())));
        return answer(label.clone(), label, pods);
    }
    // confidence intervals for a mean
    if let Some(m) = cap(re!(r"^(\d+(?:\.\d+)?)\s*%\s+confidence interval\s+(?:for\s+)?(?:a\s+|the\s+)?(?:sample\s+)?(?:with\s+)?mean\s*(?:=|of|is)?\s*(-?[\d.]+)\s*,?\s*(?:and\s+)?(?:sample\s+)?(?:sd|standard deviation|sigma|std|stdev|s|σ)\s*(?:=|of|is)?\s*([\d.]+)\s*,?\s*(?:and\s+)?(?:n|sample size|size)\s*(?:=|of|is)?\s*(\d+)$")) {
        let (level, mean, sd, n): (f64, f64, f64, f64) = (g(&m, 1).parse().ok()?, g(&m, 2).parse().ok()?, g(&m, 3).parse().ok()?, g(&m, 4).parse().ok()?);
        if !(0.0 < level && level < 100.0) || n < 2.0 {
            return None;
        }
        let alpha = 1.0 - level / 100.0;
        let se = sd / n.sqrt();
        let z = std.inverse_cdf(1.0 - alpha / 2.0);
        let t = StudentsT::new(0.0, 1.0, n - 1.0).ok()?.inverse_cdf(1.0 - alpha / 2.0);
        let label = format!("{level}% confidence interval | mean {mean}, standard deviation {sd}, n = {n}");
        let pods = vec![
            primary(pod("Result", "Confidence interval (normal)", text(format!("{} to {}   ({mean} ± {})", pr(mean - z * se), pr(mean + z * se), pr(z * se))))),
            pod("T", "Using Student's t (σ estimated from the sample)", text(format!("{} to {}   ({mean} ± {})", pr(mean - t * se), pr(mean + t * se), pr(t * se)))),
            pod("Details", "Details", table(None, vec![vec![text("standard error  s/√n"), text(pr(se))], vec![text("z critical value"), text(pr(z))], vec![text(format!("t critical value ({} df)", n - 1.0)), text(pr(t))], vec![text("margin of error (z)"), text(pr(z * se))]])),
            pod("Formula", "Formula", text("mean ± critical value × s/√n")),
        ];
        return answer(label.clone(), label, pods);
    }
    // confidence interval for a proportion
    if let Some(m) = cap(re!(r"^(\d+(?:\.\d+)?)\s*%\s+confidence interval\s+(?:for\s+)?(?:a\s+|the\s+)?(?:sample\s+)?proportion\s*(?:=|of|is)?\s*([\d.]+)\s*(%)?\s*,?\s*(?:and\s+|with\s+)?(?:n|sample size|size)\s*(?:=|of|is)?\s*(\d+)$")) {
        let (level, mut p, n): (f64, f64, f64) = (g(&m, 1).parse().ok()?, g(&m, 2).parse().ok()?, g(&m, 4).parse().ok()?);
        if !g(&m, 3).is_empty() || p > 1.0 {
            p /= 100.0;
        }
        let z = std.inverse_cdf(1.0 - (1.0 - level / 100.0) / 2.0);
        let se = (p * (1.0 - p) / n).sqrt();
        let label = format!("{level}% confidence interval | proportion {p}, n = {n}");
        return answer(label.clone(), label, vec![primary(pod("Result", "Confidence interval", text(format!("{} to {}   ({p} ± {})", pr((p - z * se).max(0.0)), pr((p + z * se).min(1.0)), pr(z * se))))), pod("Details", "Details", table(None, vec![vec![text("standard error  √(p(1−p)/n)"), text(pr(se))], vec![text("z critical value"), text(pr(z))]]))]);
    }
    // dice: expectation
    if let Some(m) = cap(re!(r"^(?:the\s+)?(expected value|expectation|mean|average|variance|standard deviation)\s+(?:of|for|when)\s+(?:rolling\s+|throwing\s+)?(a|an|one|two|three|four|five|six|\d+)?\s*(?:fair\s+)?(?:(?:standard|six-sided|(\d+)[- ]sided)\s+)?(?:die|dice)(?:\s+rolls?)?$")) {
        let k = if g(&m, 2).is_empty() { 1 } else { word_count(&g(&m, 2))? } as f64;
        let sides: f64 = if g(&m, 3).is_empty() { 6.0 } else { g(&m, 3).parse().ok()? };
        let (mean, var) = (k * (sides + 1.0) / 2.0, k * (sides * sides - 1.0) / 12.0);
        let what = g(&m, 1).to_lowercase();
        let value = match what.as_str() {
            "variance" => var,
            "standard deviation" => var.sqrt(),
            _ => mean,
        };
        let label = format!("{what} of the total of {k} fair {sides}-sided {}", if k == 1.0 { "die" } else { "dice" });
        return answer(label.clone(), label, vec![primary(pod("Result", "Result", text(pr(value)))), pod("Moments", "Distribution of the total", table(None, vec![vec![text("expected value"), text(pr(mean))], vec![text("variance"), text(pr(var))], vec![text("standard deviation"), text(pr(var.sqrt()))], vec![text("range"), text(format!("{k} to {}", k * sides))]])), pod("Formula", "Formula", text("each die: mean (s + 1)/2, variance (s² − 1)/12; independent dice add"))]);
    }
    // dice and coins: runs of one outcome
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:probability|chance|odds|likelihood) of (?:(?:rolling|throwing|getting)\s+)?(a|an|one|two|three|four|five|six|\d+|double|triple)\s+(ones?|twos?|threes?|fours?|fives?|six(?:es)?|[1-6]s?)(?:\s+in a row)?(?:\s+(?:with|on|using)\s+(?:a|one|two|three|\d+)\s+(?:fair\s+)?(?:die|dice))?$")) {
        let k = word_count(&g(&m, 1))?;
        if k == 0 || k > 30 {
            return None;
        }
        let p = Rat::new(BigInt::one(), BigInt::from(6).pow(k));
        let label = format!("probability of rolling {k} {} {}", g(&m, 2), if k == 1 { "on one die" } else { "in a row (or on that many dice)" });
        let v = num(p.clone());
        let mut pods = vec![primary(pod("Result", "Probability", math(&v)))];
        pods.extend(decimal_pod(&v));
        pods.push(pod("Odds", "Odds", text(format!("1 in {}", p.denom()))));
        pods.push(pod("Method", "Method", text(format!("each roll shows the required face with probability 1/6, and the rolls are independent: (1/6)^{k}"))));
        return answer(label.clone(), label, pods);
    }
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:probability|chance|odds|likelihood) of (?:(?:flipping|tossing|getting)\s+)?(a|an|one|two|three|four|five|six|seven|eight|nine|ten|\d+)\s+(heads?|tails?)(?:\s+in a row)$")) {
        let k: u32 = match g(&m, 1).to_lowercase().as_str() {
            "seven" => 7,
            "eight" => 8,
            "nine" => 9,
            "ten" => 10,
            w => word_count(w)?,
        };
        if k == 0 || k > 200 {
            return None;
        }
        let p = Rat::new(BigInt::one(), BigInt::from(2).pow(k));
        let v = num(p.clone());
        let label = format!("probability of {k} {} in a row", g(&m, 2));
        let mut pods = vec![primary(pod("Result", "Probability", math(&v)))];
        pods.extend(decimal_pod(&v));
        pods.push(pod("Odds", "Odds", text(format!("1 in {}", p.denom()))));
        return answer(label.clone(), label, pods);
    }
    // playing cards
    if let Some(m) = cap(re!(r"^(?:the\s+)?(?:probability|chance|odds|likelihood) of (?:drawing|getting|picking|being dealt)\s+(?:a|an|the)\s+(.+?)(?:\s+card)?(?:\s+from\s+(?:a\s+)?(?:standard\s+|shuffled\s+)?(?:deck|deck of cards|52[- ]card deck))?$")) {
        let what = g(&m, 1).to_lowercase();
        let count: u64 = match what.as_str() {
            "ace" | "king" | "queen" | "jack" | "two" | "three" | "four" | "five" | "six" | "seven" | "eight" | "nine" | "ten" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" => 4,
            "heart" | "spade" | "club" | "diamond" => 13,
            "red" | "black" => 26,
            "face" | "picture" | "court" => 12,
            "red ace" | "black ace" | "red king" | "black king" | "red queen" | "black queen" | "red jack" | "black jack" => 2,
            "ace of spades" | "queen of hearts" | "king of hearts" | "specific" => 1,
            w if w.contains(" of ") => 1,
            _ => return None,
        };
        let gcd = { let (mut a, mut b) = (count, 52u64); while b != 0 { (a, b) = (b, a % b); } a };
        let v = rat((count / gcd) as i64, (52 / gcd) as i64);
        let label = format!("probability of drawing a {what} from a standard 52-card deck");
        let mut pods = vec![primary(pod("Result", "Probability", math(&v)))];
        pods.extend(decimal_pod(&v));
        pods.push(pod("Method", "Method", text(format!("{count} of the 52 cards qualify: {count}/52"))));
        return answer(label.clone(), label, pods);
    }
    // birthday problem
    if let Some(m) = cap(re!(r"^(?:the\s+)?birthday (?:problem|paradox)\s*(?:with|for|,|:)?\s*(\d+)\s*(?:people|persons)?$"))
        .or_else(|| cap(re!(r"^(?:the\s+)?(?:probability|chance) (?:that\s+)?(?:at least\s+)?two (?:people\s+)?(?:of|in|among|out of)\s+(?:a (?:group|room) of\s+)?(\d+)\s*(?:people\s+)?(?:share|have the same|sharing) (?:a\s+)?birthday$")))
    {
        let n: u32 = g(&m, 1).parse().ok()?;
        if n > 5000 {
            return None;
        }
        let none = |k: u32| (0..k).fold(1.0f64, |p, i| p * (365.0 - i as f64).max(0.0) / 365.0);
        let p = 1.0 - none(n);
        let rows: Vec<Vec<Content>> = [5u32, 10, 20, 23, 30, 40, 50, 57, 70].iter().map(|k| vec![text(k.to_string()), text(format!("{}%", pr(100.0 * (1.0 - none(*k)))))]).collect();
        let label = format!("birthday problem with {n} people");
        return answer(label.clone(), label, vec![
            primary(pod("Result", "Probability that at least two share a birthday", text(format!("{}  ({}%)", pr(p), pr(p * 100.0))))),
            pod("Complement", "Probability that all birthdays differ", text(pr(1.0 - p))),
            pod("Table", "For other group sizes", table(Some(vec!["people", "probability of a shared birthday"]), rows)),
            pod("Method", "Method", text("1 − (365/365)(364/365)…((365 − n + 1)/365), assuming 365 equally likely birthdays")),
        ]);
    }
    None
}
