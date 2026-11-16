//! Mathematical facts: minimal polynomials and questions about algebraic,
//! rational and transcendental numbers; famous problems, theorems and
//! conjectures; standard representations of functions and constants
//! (series, products, integrals, limits); lattices and packings; named
//! constants; vector-calculus identities and gradients in polar and
//! spherical coordinates.

use super::nl::ex;
use super::*;
use crate::expr::Expr::*;
use crate::expr::*;
use crate::print::text as ptext;
use crate::simplify as s;
use num_traits::{One, Signed, Zero};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn input(s: String) -> Pod {
    pod("Input", "Input interpretation", text(s))
}

fn kv(rows: &[(&str, &str)]) -> Content {
    table(None, rows.iter().map(|(a, b)| vec![text(*a), text(*b)]).collect())
}

// ---------------------------------------------------------------------------
// algebraic numbers
// ---------------------------------------------------------------------------

fn trig_rational_pi(e: &Expr) -> Option<(String, Rat)> {
    // sin/cos/tan of a rational multiple of π
    if let Func(n, a) = e {
        if a.len() == 1 && matches!(&**n, "sin" | "cos" | "tan" | "cot" | "sec" | "csc") {
            let r = s::simplify(&div(a[0].clone(), pi()));
            if let Some(q) = r.as_rat() {
                return Some((n.to_string(), q.clone()));
            }
        }
    }
    None
}

fn algebraic(_l: &str, raw: &str) -> Option<Answer> {
    let x = sym("x");
    if let Some(m) = re!(r"(?i)^(?:the )?minimal polynomial (?:of|for) (.+?)(?: over (?:q|the rationals))?$").captures(raw) {
        let e = s::simplify(&ex(&m[1])?);
        if !e.free_symbols().is_empty() {
            return None;
        }
        let p = crate::algebraic::minimal_polynomial(&e)?;
        let monic = p.monic();
        let mut pods = vec![input(format!("minimal polynomial of {} over ℚ", ptext(&e))), primary(pod("Result", "Result", math(&p.to_expr(&x))))];
        pods.push(pod("Degree", "Degree", text(p.degree().to_string())));
        if monic != p {
            pods.push(pod("Monic", "Monic form", math(&monic.to_expr(&x))));
        }
        pods.push(pod("Integral", "Algebraic integer", text(if crate::algebraic::is_integral(&monic) { "yes (the monic minimal polynomial has integer coefficients)" } else { "no (the monic minimal polynomial has non-integer coefficients)" })));
        return Some(Answer::new("minimal polynomial", pods));
    }
    let m = re!(r"(?i)^(?:is|are) (.+?) (?:an? )?(irrational|rational|algebraic integer|algebraic unit|algebraic|transcendental)(?: numbers?)?$").captures(raw)
        .or_else(|| re!(r"(?i)^do the (algebraic units|algebraic integers|algebraic numbers) contain (.+)$").captures(raw))?;
    let (subject, prop) = if m[1].to_lowercase().starts_with("algebraic") {
        (m[2].to_string(), m[1].to_lowercase().trim_end_matches('s').to_string())
    } else {
        (m[1].to_string(), m[2].to_lowercase())
    };
    let e = s::simplify(&ex(&subject)?);
    if !e.free_symbols().is_empty() {
        return None;
    }
    let shown = ptext(&e);
    let mp = crate::algebraic::minimal_polynomial(&e);
    // what we know about the number
    let (status, reason): (&str, String) = if let Some(p) = &mp {
        let monic = p.monic();
        let integral = crate::algebraic::is_integral(&monic);
        let unit = integral && (monic.coeff(0) == Rat::one() || monic.coeff(0) == -Rat::one());
        let deg = p.degree();
        let mpt = ptext(&p.to_expr(&x));
        let base = format!("its minimal polynomial is {mpt} (degree {deg})");
        match prop.as_str() {
            "rational" | "irrational" => {
                let rational = deg == 1;
                let yes = (prop == "rational") == rational;
                (if yes { "yes" } else { "no" }, format!("{base}; {}", if rational { "degree 1, so it is rational" } else { "degree > 1, so it is irrational" }))
            }
            "algebraic" => ("yes", base),
            "transcendental" => ("no", format!("it is algebraic: {base}")),
            "algebraic integer" => (if integral { "yes" } else { "no" }, format!("{base}; the monic form {} {} integer coefficients", ptext(&monic.to_expr(&x)), if integral { "has" } else { "does not have" })),
            _ => (if unit { "yes" } else { "no" }, format!("{base}; {}", if !integral { "it is not even an algebraic integer".to_string() } else if unit { "an algebraic integer whose minimal polynomial has constant term ±1, so its inverse is also an algebraic integer".to_string() } else { format!("an algebraic integer with norm ±{}, not ±1", monic.coeff(0).abs()) })),
        }
    } else if let Some((f, q)) = trig_rational_pi(&e) {
        let yes = matches!(prop.as_str(), "algebraic" | "irrational") && !(prop == "irrational" && false);
        if prop == "transcendental" {
            ("no", format!("{f}(rπ) is algebraic for every rational r = {q}"))
        } else if prop == "algebraic" {
            ("yes", format!("{f}(rπ) is algebraic for every rational r (it lies in a cyclotomic field); here r = {q}"))
        } else {
            let _ = yes;
            return None;
        }
    } else {
        // transcendence results
        let why = match &e {
            Const(K::Pi) => Some("Lindemann (1882): π is transcendental"),
            Const(K::E) => Some("Hermite (1873): e is transcendental"),
            Pow(b, k) if matches!(&**b, Const(K::E)) && crate::algebraic::minimal_polynomial(k).is_some() && !k.is_zero() => Some("Lindemann–Weierstrass: e^a is transcendental for every nonzero algebraic a"),
            Pow(b, k) => {
                let pb = crate::algebraic::minimal_polynomial(b);
                let pk = crate::algebraic::minimal_polynomial(k);
                match (pb, pk) {
                    (Some(_), Some(pk)) if pk.degree() > 1 && !s::equal(b, &zero()) && !s::equal(b, &one()) => Some("Gelfond–Schneider (1934): a^b is transcendental for algebraic a ≠ 0, 1 and algebraic irrational b"),
                    _ => None,
                }
            }
            Func(n, a) if &**n == "log" && a.len() == 1 && crate::algebraic::minimal_polynomial(&a[0]).is_some() && !s::equal(&a[0], &one()) => Some("Lindemann–Weierstrass: log a is transcendental for algebraic a ≠ 0, 1"),
            Func(n, a) if matches!(&**n, "sin" | "cos" | "tan") && a.len() == 1 && crate::algebraic::minimal_polynomial(&a[0]).map(|p| !(p.degree() == 1 && p.coeff(0).is_zero())).unwrap_or(false) => Some("Lindemann–Weierstrass: sin, cos and tan of a nonzero algebraic number are transcendental"),
            _ => None,
        }?;
        let yes = matches!(prop.as_str(), "transcendental" | "irrational");
        (if yes { "yes" } else { "no" }, why.to_string())
    };
    Some(Answer::new("number property", vec![input(format!("is {shown} {}?", if prop.starts_with('a') && !prop.contains("unit") && !prop.contains("integer") { prop.clone() } else { format!("a {prop}") })), primary(pod("Result", "Result", text(status))), pod("Reason", "Reason", text(reason))]))
}

// ---------------------------------------------------------------------------
// famous problems, theorems and definitions
// ---------------------------------------------------------------------------

struct Fact {
    names: &'static [&'static str],
    title: &'static str,
    statement: &'static str,
    rows: &'static [(&'static str, &'static str)],
}

const FACTS: &[Fact] = &[
    Fact { names: &["riemann hypothesis", "the riemann hypothesis"], title: "Riemann hypothesis", statement: "every nontrivial zero of the Riemann zeta function ζ(s) has real part 1/2", rows: &[("status", "open; a Millennium Prize Problem (Clay Mathematics Institute, $1,000,000)"), ("proposed", "Bernhard Riemann, 1859"), ("verified", "the first 10¹³ nontrivial zeros lie on the critical line"), ("equivalent", "π(x) = li(x) + O(√x log x)")] },
    Fact { names: &["fermat's little theorem", "fermats little theorem", "fermat little theorem"], title: "Fermat's little theorem", statement: "if p is prime and a is an integer, then a^p ≡ a (mod p); if p ∤ a, then a^(p−1) ≡ 1 (mod p)", rows: &[("stated", "Pierre de Fermat, 1640"), ("first proof", "Leonhard Euler, 1736"), ("generalization", "Euler's theorem: a^φ(n) ≡ 1 (mod n) for gcd(a, n) = 1"), ("use", "Fermat primality test, RSA")] },
    Fact { names: &["fermat's last theorem", "fermats last theorem"], title: "Fermat's last theorem", statement: "aⁿ + bⁿ = cⁿ has no solutions in positive integers for n > 2", rows: &[("stated", "Pierre de Fermat, c. 1637 (margin note)"), ("proved", "Andrew Wiles, with Richard Taylor, 1994 (published 1995)"), ("method", "modularity of semistable elliptic curves")] },
    Fact { names: &["continuum hypothesis", "the continuum hypothesis"], title: "continuum hypothesis", statement: "there is no set whose cardinality is strictly between that of the integers and that of the real numbers: 2^ℵ₀ = ℵ₁", rows: &[("proposed", "Georg Cantor, 1878"), ("status", "independent of ZFC: consistent (Gödel, 1940) and its negation consistent (Cohen, 1963)"), ("Hilbert", "the first of Hilbert's 23 problems (1900)")] },
    Fact { names: &["ternary goldbach conjecture", "weak goldbach conjecture", "odd goldbach conjecture"], title: "ternary (weak) Goldbach conjecture", statement: "every odd integer greater than 5 is the sum of three primes", rows: &[("status", "proved by Harald Helfgott (2013)"), ("history", "Vinogradov (1937) proved it for all sufficiently large odd numbers")] },
    Fact { names: &["goldbach conjecture", "goldbach's conjecture", "strong goldbach conjecture", "binary goldbach conjecture"], title: "Goldbach's conjecture", statement: "every even integer greater than 2 is the sum of two primes", rows: &[("proposed", "Christian Goldbach, in a 1742 letter to Euler"), ("status", "open; verified up to 4×10¹⁸"), ("related", "the ternary version was proved by Helfgott in 2013")] },
    Fact { names: &["twin prime conjecture"], title: "twin prime conjecture", statement: "there are infinitely many primes p such that p + 2 is also prime", rows: &[("status", "open"), ("progress", "Zhang (2013): infinitely many prime gaps below 7×10⁷; Polymath/Maynard: below 246")] },
    Fact { names: &["collatz conjecture", "3n+1 problem", "3x+1 problem"], title: "Collatz conjecture", statement: "repeatedly applying n ↦ n/2 (n even), 3n + 1 (n odd) reaches 1 from every positive integer", rows: &[("proposed", "Lothar Collatz, 1937"), ("status", "open; verified beyond 2⁶⁸"), ("progress", "Tao (2019): almost all orbits attain almost bounded values")] },
    Fact { names: &["smale's fifteenth problem", "smale's 15th problem", "smales fifteenth problem"], title: "Smale's 15th problem", statement: "do the Navier–Stokes equations in ℝ³ always have a unique smooth solution that extends for all time?", rows: &[("source", "Stephen Smale, 'Mathematical problems for the next century' (1998)"), ("status", "open; also a Millennium Prize Problem")] },
    Fact { names: &["halting problem", "who formulated the halting problem", "the halting problem"], title: "halting problem", statement: "there is no algorithm that decides, for every program and input, whether the program eventually halts", rows: &[("formulated and proved undecidable by", "Alan Turing, 1936 ('On Computable Numbers')"), ("name", "the term 'halting problem' was introduced by Martin Davis (1950s)"), ("related", "Church's undecidability of the Entscheidungsproblem (1936)")] },
    Fact { names: &["erdos-turan conjecture", "erdős–turán conjecture", "erdos turan conjecture", "statement of the erdos-turan conjecture", "erdős-turán conjecture"], title: "Erdős–Turán conjecture on additive bases", statement: "if B ⊆ ℕ is an additive basis of order 2 (every sufficiently large integer is a sum of two elements of B), then the number of representations of n as such a sum is unbounded", rows: &[("proposed", "Paul Erdős and Pál Turán, 1941"), ("status", "open"), ("Erdős prize", "$500")] },
    Fact { names: &["transcendence of pi", "pi is transcendental", "is pi transcendental", "lindemann's theorem", "lindemann theorem"], title: "transcendence of π", statement: "π is not a root of any nonzero polynomial with rational coefficients", rows: &[("proved", "Ferdinand von Lindemann, 1882"), ("method", "e^(iπ) = −1 with the Lindemann–Weierstrass theorem"), ("consequence", "squaring the circle with compass and straightedge is impossible")] },
    Fact { names: &["angle trisection", "trisecting an angle", "trisection of an angle"], title: "angle trisection", statement: "an arbitrary angle cannot be trisected with compass and straightedge alone", rows: &[("proved", "Pierre Wantzel, 1837"), ("reason", "trisecting 60° requires cos 20°, a root of the irreducible cubic 8x³ − 6x − 1, but constructible numbers have degree a power of 2"), ("possible with", "a marked ruler (neusis), origami, or the quadratrix")] },
    Fact { names: &["knapsack problem", "the knapsack problem", "0-1 knapsack problem"], title: "knapsack problem", statement: "given items with weights w_i and values v_i and a capacity W, choose items maximizing Σ v_i subject to Σ w_i ≤ W", rows: &[("complexity", "the decision version is NP-complete (one of Karp's 21 problems, 1972)"), ("algorithms", "dynamic programming in O(nW) time (pseudo-polynomial); a fully polynomial-time approximation scheme exists"), ("variants", "0–1, bounded, unbounded, fractional (greedy optimal), multidimensional")] },
    Fact { names: &["cantor's paradox", "cantors paradox", "cantor paradox"], title: "Cantor's paradox", statement: "there is no set of all sets (or of all cardinal numbers): its power set would have strictly larger cardinality by Cantor's theorem, yet be contained in it", rows: &[("noticed", "Georg Cantor, 1899"), ("resolution", "in ZFC the universe of sets is a proper class, not a set")] },
    Fact { names: &["konigsberg theorem", "königsberg theorem", "seven bridges of konigsberg", "seven bridges of königsberg", "konigsberg bridge problem", "königsberg bridge problem"], title: "Seven Bridges of Königsberg", statement: "there is no walk crossing each of Königsberg's seven bridges exactly once", rows: &[("proved", "Leonhard Euler, 1736: the beginning of graph theory"), ("reason", "all four land masses have an odd number of bridges, but an Euler path allows at most two odd-degree vertices"), ("Euler's theorem", "a connected graph has an Euler circuit iff every vertex has even degree")] },
    Fact { names: &["four color theorem", "four colour theorem", "four color theorem msc"], title: "four color theorem", statement: "the regions of every planar map can be colored with four colors so that adjacent regions differ", rows: &[("proved", "Kenneth Appel and Wolfgang Haken, 1976 (computer-assisted); simplified by Robertson, Sanders, Seymour, Thomas (1997); formally verified in Coq (Gonthier, 2005)"), ("MSC classification", "05C15 (coloring of graphs and hypergraphs)"), ("proposed", "Francis Guthrie, 1852")] },
    Fact { names: &["basic definition of prime number", "definition of prime number", "definition of a prime number", "what is a prime number", "prime number definition"], title: "prime number", statement: "an integer p > 1 whose only positive divisors are 1 and p", rows: &[("first primes", "2, 3, 5, 7, 11, 13, 17, 19, 23, 29, …"), ("fundamental theorem of arithmetic", "every integer > 1 is a product of primes in exactly one way up to order"), ("Euclid", "there are infinitely many primes"), ("prime number theorem", "π(x) ~ x/ln x")] },
    Fact { names: &["p vs np", "p versus np", "p vs np problem", "p = np"], title: "P versus NP", statement: "can every problem whose solutions can be verified in polynomial time also be solved in polynomial time?", rows: &[("status", "open; a Millennium Prize Problem"), ("formulated", "Stephen Cook (1971) and Leonid Levin (1973)")] },
    Fact { names: &["poincare conjecture", "poincaré conjecture"], title: "Poincaré conjecture", statement: "every simply connected, closed 3-manifold is homeomorphic to the 3-sphere", rows: &[("proposed", "Henri Poincaré, 1904"), ("proved", "Grigori Perelman, 2002–2003 (Ricci flow with surgery); the only Millennium Problem solved so far")] },
    Fact { names: &["what is a regular continued fraction", "regular continued fraction", "simple continued fraction"], title: "regular continued fraction", statement: "a continued fraction a₀ + 1/(a₁ + 1/(a₂ + ⋯)) with all partial numerators 1, a₀ an integer and a₁, a₂, … positive integers, written [a₀; a₁, a₂, …]", rows: &[("finite", "exactly for rational numbers"), ("periodic", "exactly for quadratic irrationals"), ("convergents", "p_n/q_n are the best rational approximations")] },
    Fact { names: &["what is a unitary matrix", "unitary matrix", "definition of unitary matrix"], title: "unitary matrix", statement: "a complex square matrix U with U* U = U U* = I, where U* is the conjugate transpose", rows: &[("eigenvalues", "all of modulus 1"), ("determinant", "|det U| = 1"), ("preserves", "inner products and lengths: ⟨Ux, Uy⟩ = ⟨x, y⟩"), ("real case", "orthogonal matrices")] },
    Fact { names: &["radon transform", "the radon transform"], title: "Radon transform", statement: "Rf(θ, s) = ∫_ℝ f(s cos θ − t sin θ, s sin θ + t cos θ) dt: the integrals of f over all lines", rows: &[("introduced", "Johann Radon, 1917"), ("inversion", "filtered back-projection; the Fourier slice theorem relates it to the 2-D Fourier transform"), ("applications", "computed tomography (CT), seismology")] },
    Fact { names: &["fibonacci(n) recurrence", "fibonacci recurrence", "fibonacci recurrence relation"], title: "Fibonacci recurrence", statement: "F(n) = F(n − 1) + F(n − 2), F(0) = 0, F(1) = 1", rows: &[("closed form (Binet)", "F(n) = (φⁿ − ψⁿ)/√5, φ = (1 + √5)/2, ψ = (1 − √5)/2"), ("first terms", "0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, …"), ("generating function", "x/(1 − x − x²)")] },
];

fn facts(l: &str) -> Option<Answer> {
    let k = re!(r"^(?:what is |what's |define |definition of )?(?:the |an? )?").replace(l, "").to_string();
    let f = FACTS.iter().find(|f| f.names.contains(&k.as_str()) || f.names.contains(&l) || f.names.iter().any(|n| n.trim_start_matches("what is a ") == k))?;
    let mut pods = vec![input(f.title.to_string()), primary(pod("Result", "Statement", text(f.statement)))];
    if !f.rows.is_empty() {
        pods.push(pod("Details", "Details", kv(f.rows)));
    }
    Some(Answer::new(f.title, pods))
}

// ---------------------------------------------------------------------------
// representations of functions and constants
// ---------------------------------------------------------------------------

/// (function keys, kind, formula, condition)
const REPS: &[(&[&str], &str, &str, &str)] = &[
    (&["gamma(x)", "gamma(z)", "gamma", "gamma function", "γ(x)"], "integral", "Γ(z) = ∫₀^∞ t^(z−1) e^(−t) dt", "Re z > 0 (Euler's integral of the second kind)"),
    (&["gamma(x)", "gamma(z)", "gamma", "gamma function"], "limit", "Γ(z) = lim_(n→∞) n! n^z/(z (z+1) ⋯ (z+n))", "z ∉ {0, −1, −2, …} (Gauss)"),
    (&["gamma(x)", "gamma(z)", "gamma", "gamma function"], "product", "1/Γ(z) = z e^(γz) Π_(k=1)^∞ (1 + z/k) e^(−z/k)", "all z (Weierstrass)"),
    (&["log x", "log(x)", "ln x", "log", "logarithm"], "integral", "log x = ∫₁^x dt/t", "x > 0"),
    (&["log x", "log(x)", "ln x", "log", "logarithm"], "series", "log x = Σ_(k=1)^∞ (−1)^(k+1) (x − 1)^k/k", "|x − 1| < 1 (and x = 2)"),
    (&["log x", "log(x)", "ln x", "log", "logarithm"], "limit", "log x = lim_(n→∞) n (x^(1/n) − 1)", "x > 0"),
    (&["fresnel s(x)", "fresnels(x)", "fresnel s", "fresnel s integral"], "integral", "S(x) = ∫₀^x sin(π t²/2) dt", "all x"),
    (&["fresnel c(x)", "fresnelc(x)", "fresnel c"], "integral", "C(x) = ∫₀^x cos(π t²/2) dt", "all x"),
    (&["erfc(z)", "erfc(x)", "erfc"], "integral", "erfc(z) = (2/√π) ∫_z^∞ e^(−t²) dt", "all z (path to +∞ along a ray)"),
    (&["erf(z)", "erf(x)", "erf"], "integral", "erf(z) = (2/√π) ∫₀^z e^(−t²) dt", "all z"),
    (&["erf(z)", "erf(x)", "erf"], "series", "erf(z) = (2/√π) Σ_(k=0)^∞ (−1)^k z^(2k+1)/(k! (2k+1))", "all z"),
    (&["sin z", "sin(z)", "sin x", "sin(x)", "sine"], "product", "sin z = z Π_(k=1)^∞ (1 − z²/(k² π²))", "all z (Euler)"),
    (&["sin z", "sin(z)", "sin x", "sin(x)", "sine"], "series", "sin z = Σ_(k=0)^∞ (−1)^k z^(2k+1)/(2k+1)!", "all z"),
    (&["sin (pi x)", "sin(pi x)", "sin(πx)"], "series", "sin(πx) = Σ_(k=0)^∞ (−1)^k (πx)^(2k+1)/(2k+1)!", "all x"),
    (&["sin (pi x)", "sin(pi x)", "sin(πx)"], "product", "sin(πx) = πx Π_(k=1)^∞ (1 − x²/k²)", "all x"),
    (&["cos z", "cos(z)", "cos x", "cos(x)", "cosine"], "product", "cos z = Π_(k=1)^∞ (1 − 4z²/((2k − 1)² π²))", "all z"),
    (&["zeta(s)", "ζ(s)", "zeta", "riemann zeta"], "product", "ζ(s) = Π_(p prime) 1/(1 − p^(−s))", "Re s > 1 (Euler product)"),
    (&["zeta(s)", "ζ(s)", "zeta", "riemann zeta"], "series", "ζ(s) = Σ_(k=1)^∞ 1/k^s", "Re s > 1"),
    (&["zeta(s)", "ζ(s)", "zeta", "riemann zeta"], "integral", "ζ(s) = (1/Γ(s)) ∫₀^∞ t^(s−1)/(e^t − 1) dt", "Re s > 1"),
    (&["zeta(s) containing csch(t)", "zeta(s) with csch"], "integral", "ζ(s) = 1/(2 (1 − 2^(−s)) Γ(s)) ∫₀^∞ t^(s−1) csch(t) dt", "Re s > 1"),
    (&["exp(z)", "exp(x)", "e^z", "e^x", "exp"], "limit", "e^z = lim_(n→∞) (1 + z/n)^n", "all z"),
    (&["exp(z)", "exp(x)", "e^z", "e^x", "exp"], "series", "e^z = Σ_(k=0)^∞ z^k/k!", "all z"),
    (&["si(x)", "si(z)", "sine integral"], "integral", "Si(x) = ∫₀^x sin(t)/t dt", "all x"),
    (&["si(x)", "si(z)", "sine integral"], "series", "Si(x) = Σ_(k=0)^∞ (−1)^k x^(2k+1)/((2k+1) (2k+1)!)", "all x"),
    (&["si(x)", "si(z)", "sine integral"], "limit", "Si(x) = lim_(n→∞) Σ_(k=0)^n (−1)^k x^(2k+1)/((2k+1) (2k+1)!)", "all x (limit of the partial sums of its Maclaurin series)"),
    (&["li(x)", "logarithmic integral"], "series", "li(x) = γ + log(log x) + Σ_(k=1)^∞ (log x)^k/(k · k!)", "x > 1 (Ramanujan's faster series also exists)"),
    (&["li(x)", "logarithmic integral"], "integral", "li(x) = ∫₀^x dt/log t (principal value)", "x > 0, x ≠ 1"),
    (&["pi", "π"], "product", "π/2 = Π_(k=1)^∞ 4k²/(4k² − 1) = (2/1)(2/3)(4/3)(4/5)(6/5)(6/7)⋯", "Wallis product (1656)"),
    (&["pi", "π"], "series", "π/4 = Σ_(k=0)^∞ (−1)^k/(2k + 1)", "Madhava–Leibniz series"),
    (&["pi", "π"], "integral", "π = 4 ∫₀^1 √(1 − x²) dx = ∫_(−∞)^∞ dx/(1 + x²)", ""),
    (&["pi^2/6", "π^2/6", "π²/6"], "series", "π²/6 = Σ_(k=1)^∞ 1/k² = ζ(2)", "Basel problem (Euler, 1734)"),
    (&["pi^2/6", "π^2/6", "π²/6"], "product", "π²/6 = Π_(p prime) 1/(1 − p^(−2))", "Euler product for ζ(2)"),
    (&["euler gamma", "euler's constant", "euler-mascheroni constant", "euler mascheroni constant", "γ"], "limit", "γ = lim_(n→∞) (Σ_(k=1)^n 1/k − log n) = 0.5772156649…", ""),
    (&["euler gamma", "euler's constant", "euler-mascheroni constant", "euler mascheroni constant", "γ"], "integral", "γ = −∫₀^∞ e^(−t) log t dt", ""),
    (&["euler gamma", "euler's constant", "euler-mascheroni constant", "euler mascheroni constant", "γ"], "series", "γ = Σ_(k=2)^∞ (−1)^k ζ(k)/k", ""),
    (&["elliptic nome q(m)", "nome q(m)", "elliptic nome", "nome"], "series", "q(m) = m/16 + m²/32 + 21 m³/1024 + 31 m⁴/2048 + 6257 m⁵/524288 + O(m⁶)", "q = exp(−π K(1 − m)/K(m)), small m"),
    (&["e", "euler's number"], "series", "e = Σ_(k=0)^∞ 1/k!", ""),
    (&["e", "euler's number"], "limit", "e = lim_(n→∞) (1 + 1/n)^n", ""),
];

fn representations(l: &str) -> Option<Answer> {
    let l = &re!(r"^series (elliptic nome.*|nome.*)$").replace(l, "series representation of $1").to_string();
    let m = re!(r"^(?:the )?(?:(integral|series|product|limit) rep(?:resentation)?s?\.? (?:of |for )?(.+?)|(.+?) (integral|series|product|limit) rep(?:resentation)?s?\.?(?: (?:of|for) (.+?))?|(integral|series|product|limit) representations? of (.+?) containing (.+))$").captures(l)?;
    let (kind, key) = if let Some(k) = m.get(1) {
        (k.as_str().to_string(), m[2].to_string())
    } else if let Some(k) = m.get(4) {
        (k.as_str().to_string(), m[3].to_string())
    } else {
        (m[6].to_string(), format!("{} containing {}", &m[7], &m[8]))
    };
    let key = key.trim().trim_start_matches("the ").to_string();
    let key2 = re!(r"\s+containing\s+.*$").replace(&key, "").to_string();
    let hits: Vec<_> = REPS.iter().filter(|(ks, kd, _, _)| *kd == kind && ks.contains(&key.as_str())).collect();
    let hits = if hits.is_empty() { REPS.iter().filter(|(ks, kd, _, _)| *kd == kind && ks.contains(&key2.as_str())).collect() } else { hits };
    if hits.is_empty() {
        return None;
    }
    let rows: Vec<Vec<Content>> = hits.iter().map(|(_, _, f, c)| vec![text(*f), text(*c)]).collect();
    let others: Vec<&str> = REPS.iter().filter(|(ks, kd, _, _)| *kd != kind && (ks.contains(&key.as_str()) || ks.contains(&key2.as_str()))).map(|(_, _, f, _)| *f).collect();
    let mut pods = vec![input(format!("{kind} representation of {key}")), primary(pod("Result", &format!("{} representation", kind[..1].to_uppercase() + &kind[1..]), text(hits[0].2))), pod("All", "Representations", table(Some(vec!["formula", "conditions"]), rows))];
    if !others.is_empty() {
        pods.push(pod("Other", "Other representations", text(others.join("\n"))));
    }
    Some(Answer::new("representation", pods))
}

// ---------------------------------------------------------------------------
// lattices and constants
// ---------------------------------------------------------------------------

const LATTICES: &[(&[&str], &str, &[(&str, &str)])] = &[
    (&["face-centered cubic", "face centered cubic", "fcc", "fcc lattice", "face-centered cubic lattice", "cubic close packing"], "face-centered cubic lattice (A₃ = D₃)", &[("dimension", "3"), ("kissing number", "12"), ("packing density", "π/(3√2) ≈ 0.74048 (optimal: Kepler conjecture, Hales 2005/2014)"), ("nearest-neighbour distance", "a/√2 for cube edge a"), ("atoms per cubic cell", "4"), ("examples", "copper, aluminium, gold, silver")]),
    (&["hexagonal close packing", "hexagonal close packed", "hcp"], "hexagonal close packing", &[("dimension", "3"), ("kissing number", "12"), ("packing density", "π/(3√2) ≈ 0.74048 (same as fcc)"), ("stacking", "ABAB… (fcc is ABCABC…)"), ("ideal c/a ratio", "√(8/3) ≈ 1.633"), ("examples", "magnesium, titanium, zinc"), ("note", "not a lattice: a lattice with a two-point basis")]),
    (&["simple cubic", "simple cubic lattice", "primitive cubic"], "simple cubic lattice (ℤ³)", &[("dimension", "3"), ("kissing number", "6"), ("packing density", "π/6 ≈ 0.52360"), ("atoms per cell", "1"), ("example", "polonium")]),
    (&["simple hexagonal", "simple hexagonal lattice", "hexagonal lattice"], "simple hexagonal lattice", &[("dimension", "3"), ("kissing number", "8"), ("packing density", "π/(3√3) ≈ 0.60460"), ("structure", "stacked triangular layers directly above each other (AA…)")]),
    (&["body-centered cubic", "body centered cubic", "bcc", "bcc lattice"], "body-centered cubic lattice", &[("dimension", "3"), ("kissing number", "8"), ("packing density", "π√3/8 ≈ 0.68017"), ("atoms per cubic cell", "2"), ("examples", "iron (α), chromium, tungsten")]),
    (&["a4 root lattice", "a4 lattice", "a_4 lattice"], "A₄ root lattice", &[("dimension", "4"), ("definition", "{x ∈ ℤ⁵ : x₁ + ⋯ + x₅ = 0}"), ("kissing number", "20"), ("determinant", "5"), ("minimal norm", "2"), ("packing density", "π²/(8√5) ≈ 0.55173"), ("roots", "20 vectors e_i − e_j (Weyl group S₅)")]),
    (&["d4 lattice", "d4 root lattice", "24-cell lattice"], "D₄ root lattice", &[("dimension", "4"), ("kissing number", "24 (the vertices of the 24-cell)"), ("packing density", "π²/16 ≈ 0.61685 (densest lattice packing in 4-D)")]),
    (&["e8 lattice", "e8 root lattice", "e_8 lattice"], "E₈ lattice", &[("dimension", "8"), ("kissing number", "240"), ("packing density", "π⁴/384 ≈ 0.25367 (optimal, Viazovska 2016)"), ("properties", "even unimodular; the root lattice of E₈")]),
    (&["leech lattice"], "Leech lattice Λ₂₄", &[("dimension", "24"), ("kissing number", "196560"), ("packing density", "π¹²/12! ≈ 0.0019296 (optimal, Cohn–Kumar–Miller–Radchenko–Viazovska 2016)"), ("properties", "the unique even unimodular lattice in 24 dimensions with no vectors of norm 2"), ("automorphism group", "Conway group Co₀, order 8315553613086720000"), ("discovered", "John Leech, 1967")]),
];

const CONSTANTS: &[(&[&str], &str, &str, &[(&str, &str)])] = &[
    (&["champernowne constant", "champernowne's constant"], "Champernowne constant C₁₀", "0.12345678910111213141516171819202122…", &[("definition", "concatenate the decimal digits of 1, 2, 3, …"), ("properties", "transcendental (Mahler, 1937); normal in base 10 (Champernowne, 1933)"), ("continued fraction", "[0; 8, 9, 1, 149083, 1, 1, 1, 4, 1, 1, 1, 3, 4, 1, 1, 1, 15, …], with occasional enormous partial quotients")]),
    (&["feigenbaum alpha", "feigenbaum's alpha", "feigenbaum constant alpha", "second feigenbaum constant"], "Feigenbaum constant α", "2.502907875095892822283902873218…", &[("definition", "the limiting ratio of successive widths of the tines of the bifurcation diagram (period-doubling)"), ("discovered", "Mitchell Feigenbaum, 1975"), ("companion", "δ = 4.669201609102990671853…")]),
    (&["feigenbaum delta", "feigenbaum's delta", "feigenbaum constant", "feigenbaum constant delta", "first feigenbaum constant"], "Feigenbaum constant δ", "4.669201609102990671853203820466…", &[("definition", "the limiting ratio of successive intervals between period-doubling bifurcations"), ("discovered", "Mitchell Feigenbaum, 1975"), ("companion", "α = 2.502907875095892822283…")]),
];

fn lattices(l: &str) -> Option<Answer> {
    let parts: Vec<&str> = l.split(", ").map(str::trim).collect();
    let mut pods = Vec::new();
    for p in &parts {
        let (_, name, rows) = LATTICES.iter().find(|(ks, _, _)| ks.contains(p))?;
        let r: Vec<Vec<Content>> = rows.iter().map(|(a, b)| vec![text(*a), text(*b)]).collect();
        if pods.is_empty() {
            pods.push(input(parts.join(" | ")));
            pods.push(primary(pod("Result", &format!("{name}: kissing number"), text(rows.iter().find(|r| r.0 == "kissing number").map(|r| r.1).unwrap_or("")))));
        }
        pods.push(pod(&format!("Props{name}"), name, table(None, r)));
    }
    if pods.is_empty() {
        return None;
    }
    Some(Answer::new("lattice", pods))
}

fn constants(l: &str) -> Option<Answer> {
    let (_, name, value, rows) = CONSTANTS.iter().find(|(ks, _, _, _)| ks.contains(&l))?;
    Some(Answer::new(*name, vec![input(name.to_string()), primary(pod("Result", "Decimal approximation", text(*value))), pod("Details", "Details", kv(rows))]))
}

// ---------------------------------------------------------------------------
// vector calculus identities and curvilinear gradients
// ---------------------------------------------------------------------------

fn vector_identities(l: &str) -> Option<Answer> {
    let l = l.replace("del", "∇").replace("nabla", "∇");
    let (title, main, rows): (&str, &str, Vec<(&str, &str)>) = match l.trim() {
        "curl (curl f)" | "curl curl f" | "curl(curl f)" | "curl of curl" | "curl (curl a)" => ("curl of curl", "∇ × (∇ × F) = ∇(∇ · F) − ∇²F", vec![("Cartesian", "the vector Laplacian ∇²F acts componentwise")]),
        "div (grad f)" | "div grad f" | "div(grad f)" | "divergence of gradient" => ("divergence of the gradient", "∇ · (∇f) = ∇²f = ∂²f/∂x² + ∂²f/∂y² + ∂²f/∂z²", vec![("name", "the Laplacian of f")]),
        "curl (grad f)" | "curl grad f" | "curl(grad f)" | "curl of gradient" => ("curl of a gradient", "∇ × (∇f) = 0", vec![("condition", "f twice continuously differentiable")]),
        "div (curl f)" | "div curl f" | "div(curl f)" | "divergence of curl" => ("divergence of a curl", "∇ · (∇ × F) = 0", vec![("condition", "F twice continuously differentiable")]),
        "grad (f . g)" | "grad(f . g)" | "grad (f·g)" | "grad of dot product" | "gradient of dot product" => ("gradient of a dot product", "∇(F · G) = (F · ∇)G + (G · ∇)F + F × (∇ × G) + G × (∇ × F)", vec![]),
        "div (f x g)" | "div(f x g)" | "divergence of cross product" => ("divergence of a cross product", "∇ · (F × G) = G · (∇ × F) − F · (∇ × G)", vec![]),
        "curl (f x g)" | "curl(f x g)" | "curl of cross product" => ("curl of a cross product", "∇ × (F × G) = F (∇ · G) − G (∇ · F) + (G · ∇)F − (F · ∇)G", vec![]),
        "grad of a scalar field" | "gradient of a scalar field" | "grad" | "gradient" | "gradient operator" => ("gradient", "∇f = (∂f/∂x, ∂f/∂y, ∂f/∂z)", vec![("cylindrical (r, θ, z)", "∇f = f_r r̂ + (1/r) f_θ θ̂ + f_z ẑ"), ("spherical (r, θ, φ)", "∇f = f_r r̂ + (1/r) f_θ θ̂ + (1/(r sin θ)) f_φ φ̂"), ("meaning", "points in the direction of steepest increase; its length is the rate of increase")]),
        "rotor operator" | "rotor" | "curl operator" | "curl" => ("curl (rotor)", "∇ × F = (∂F_z/∂y − ∂F_y/∂z, ∂F_x/∂z − ∂F_z/∂x, ∂F_y/∂x − ∂F_x/∂y)", vec![("also called", "rotor (rot F), rotation"), ("meaning", "twice the local angular velocity of the flow F"), ("Stokes' theorem", "∬_S (∇ × F) · dS = ∮_∂S F · dr")]),
        "divergence calculator" | "divergence" | "divergence operator" => ("divergence", "∇ · F = ∂F_x/∂x + ∂F_y/∂y + ∂F_z/∂z", vec![("cylindrical", "(1/r) ∂(r F_r)/∂r + (1/r) ∂F_θ/∂θ + ∂F_z/∂z"), ("spherical", "(1/r²) ∂(r² F_r)/∂r + (1/(r sin θ)) ∂(sin θ F_θ)/∂θ + (1/(r sin θ)) ∂F_φ/∂φ"), ("example", "try: div {x^2, y z, x z}")]),
        "laplacian calculator" | "laplacian" | "laplace operator" | "laplacian operator" => ("Laplacian", "∇²f = ∂²f/∂x² + ∂²f/∂y² + ∂²f/∂z²", vec![("polar", "f_rr + f_r/r + f_θθ/r²"), ("spherical", "(1/r²)(r² f_r)_r + (1/(r² sin θ))(sin θ f_θ)_θ + f_φφ/(r² sin² θ)"), ("example", "try: laplacian x^2 y + z^3")]),
        _ => return None,
    };
    let mut pods = vec![input(title.to_string()), primary(pod("Result", "Identity", text(main)))];
    if !rows.is_empty() {
        pods.push(pod("Details", "Details", kv(&rows)));
    }
    Some(Answer::new(title, pods))
}

fn curvilinear_grad(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:grad|gradient)(?: of)? (.+)$").captures(l)?;
    if !m[1].contains("theta") && !m[1].contains("phi") && !m[1].contains('θ') && !m[1].contains('φ') {
        return None;
    }
    // parse with placeholder letters, then rename
    let body = m[1].replace("theta", "q").replace("phi", "w").replace('θ', "q").replace('φ', "w");
    let f0 = ex(&body)?;
    let fs = f0.free_symbols();
    if fs.iter().any(|v| !matches!(&**v, "r" | "q" | "w")) {
        return None;
    }
    let (r, th, ph) = (sym("r"), sym("θ"), sym("φ"));
    let f = f0.subs_many(&[(sym("q"), th.clone()), (sym("w"), ph.clone())]);
    let d = |v: &Expr| s::simplify(&crate::calculus::diff::diff(&f, v));
    let spherical = f.has_sym(&ph);
    let comps = if spherical {
        vec![("r̂", d(&r)), ("θ̂", s::simplify(&div(d(&th), r.clone()))), ("φ̂", s::simplify(&div(d(&ph), mul(vec![r.clone(), func("sin", vec![th.clone()])]))))]
    } else {
        vec![("r̂", d(&r)), ("θ̂", s::simplify(&div(d(&th), r.clone())))]
    };
    let shown: Vec<String> = comps.iter().map(|(u, c)| format!("({}) {u}", ptext(c))).collect();
    Some(Answer::new("gradient", vec![
        input(format!("∇({}) in {} coordinates", ptext(&f), if spherical { "spherical" } else { "polar" })),
        primary(pod("Result", "Gradient", text(shown.join(" + ")))),
        pod("Formula", "Formula", text(if spherical { "∇f = f_r r̂ + (1/r) f_θ θ̂ + (1/(r sin θ)) f_φ φ̂" } else { "∇f = f_r r̂ + (1/r) f_θ θ̂" })),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let raw0 = re!(r"(?i)\b(sqrt|sin|cos|exp|log)\[([^\[\]]*)\]").replace_all(q.clean.trim().trim_end_matches(['?', '.']), "$1($2)").to_string();
    let raw = raw0.as_str();
    let l = re!(r"\s+").replace_all(&raw.to_lowercase(), " ").to_string();
    algebraic(&l, raw)
        .or_else(|| facts(&l))
        .or_else(|| representations(&l))
        .or_else(|| lattices(&l))
        .or_else(|| constants(&l))
        .or_else(|| vector_identities(&l))
        .or_else(|| curvilinear_grad(&l))
}

#[cfg(test)]
mod tests {
    fn ans(s: &str) -> String {
        serde_json::to_string(&crate::engine::query(s).pods).unwrap_or_default()
    }

    #[test]
    fn math_facts() {
        assert!(ans("minimal polynomial of sqrt(2)+sqrt(3)").contains("x^4 - 10 x^2 + 1"));
        assert!(ans("Is 1/(1+sqrt(5)) an algebraic integer?").contains("\"no\""));
        assert!(ans("do the algebraic units contain Sqrt[2]+Sqrt[3]?").contains("\"yes\""));
        assert!(ans("Is sqrt(2)^sqrt(2) transcendental?").contains("Gelfond"));
        assert!(ans("sin z product representation").contains("1 − z²/(k² π²)"));
        assert!(ans("Leech lattice").contains("196560"));
    }
}
