//! Continued fraction theory: notations, named theorems and conjectures,
//! classic expansions (Gauss's hypergeometric fraction, Bessel ratios), the
//! metric theory (Gauss–Kuzmin, Khinchin, Lévy), algorithms, and the
//! computations that go with them: convergents, nearest-integer expansions,
//! and an analysis of √n using Lagrange's, Galois's and Legendre's theorems.

use super::nl::ex;
use super::*;
use num_bigint::BigInt;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn input(s: &str) -> Pod {
    pod("Input", "Input interpretation", text(s))
}

struct Fact {
    names: &'static [&'static str],
    title: &'static str,
    statement: &'static str,
    extra: &'static [(&'static str, &'static str)],
}

const FACTS: &[Fact] = &[
    Fact { names: &["common notations for continued fractions", "continued fraction notations", "continued fraction notation", "notation for continued fractions"], title: "continued fraction notations", statement: "b₀ + a₁/(b₁ + a₂/(b₂ + a₃/(b₃ + …)))", extra: &[("regular (simple) fraction", "[a₀; a₁, a₂, a₃, …] = a₀ + 1/(a₁ + 1/(a₂ + 1/(a₃ + …)))"), ("Pringsheim", "b₀ + a₁|/|b₁ + a₂|/|b₂ + a₃|/|b₃ + …"), ("compact (Rogers–Ramanujan style)", "b₀ + a₁/b₁+ a₂/b₂+ a₃/b₃+ …"), ("Gauss's K", "b₀ + K_(n=1)^∞ (a_n/b_n)"), ("finite", "[a₀; a₁, …, a_n] (rational numbers)"), ("periodic", "[a₀; a₁, …, a_k, (a_(k+1), …, a_(k+m))‾] with the repeating block overlined")] },
    Fact { names: &["gauss-kuzmin-wirsing operator", "gauss kuzmin wirsing operator", "gauss-kuzmin operator", "gauss map transfer operator"], title: "Gauss–Kuzmin–Wirsing operator", statement: "the transfer operator of the Gauss map x ↦ {1/x}: (G f)(x) = Σ_(n≥1) f(1/(x + n))/(x + n)²", extra: &[("invariant density", "eigenvalue 1, eigenfunction 1/((1 + x) ln 2) (the Gauss measure)"), ("second eigenvalue", "−λ, λ = 0.30366300289873265859… (Gauss–Kuzmin–Wirsing constant)"), ("consequence", "|P(x_n < t) − log₂(1 + t)| decays like λⁿ (Wirsing 1974)"), ("named for", "Carl Friedrich Gauss, Rodion Kuzmin, Eduard Wirsing")] },
    Fact { names: &["gauss-kuzmin distribution", "gauss kuzmin distribution", "gauss-kuzmin theorem", "kuzmin theorem"], title: "Gauss–Kuzmin distribution", statement: "for almost every x, the partial quotient a_n of its regular continued fraction equals k with limiting probability −log₂(1 − 1/(k + 1)²)", extra: &[("P(a = 1)", "log₂(4/3) ≈ 0.41504"), ("P(a = 2)", "log₂(9/8) ≈ 0.16993"), ("P(a = 3)", "log₂(16/15) ≈ 0.09311"), ("history", "conjectured by Gauss (1812), proved by Kuzmin (1928)")] },
    Fact { names: &["khinchin's theorem", "khinchin theorem", "khinchin's constant", "khinchin constant"], title: "Khinchin's theorem", statement: "for almost every real x, the geometric mean of the partial quotients tends to Khinchin's constant: (a₁ a₂ ⋯ a_n)^(1/n) → K₀ = 2.6854520010653064453…", extra: &[("exceptions", "rationals, quadratic irrationals and e, among others"), ("year", "1934")] },
    Fact { names: &["levy's theorem", "lévy's theorem", "levy constant", "lévy's constant", "khinchin-levy theorem", "khinchin-lévy theorem"], title: "Lévy's theorem", statement: "for almost every real x, the convergent denominators satisfy q_n^(1/n) → e^(π²/(12 ln 2)) = 3.2758229187218111597… (Lévy's constant)", extra: &[("equivalently", "(1/n) ln q_n → π²/(12 ln 2) ≈ 1.1865691104 (Khinchin–Lévy constant)"), ("year", "1935–1936")] },
    Fact { names: &["worpitzky theorem", "worpitzky's theorem", "worpitzky criterion"], title: "Worpitzky's theorem", statement: "if |a_n| ≤ 1/4 for all n ≥ 1, the continued fraction K(a_n/1) = a₁/(1 + a₂/(1 + a₃/(1 + …))) converges, and every approximant and the limit lie in the disk |w| ≤ 1/2", extra: &[("sharpness", "1/4 is the largest constant that works (a_n = −1/4 − ε fails)"), ("year", "Julius Worpitzky, 1865")] },
    Fact { names: &["sleszynski-pringsheim theorem", "śleszyński–pringsheim theorem", "pringsheim theorem", "sleszynski pringsheim theorem"], title: "Śleszyński–Pringsheim theorem", statement: "if |b_n| ≥ |a_n| + 1 for all n, the continued fraction K(a_n/b_n) converges, and its value has modulus at most 1", extra: &[("year", "Śleszyński 1888, Pringsheim 1899")] },
    Fact { names: &["seidel-stern theorem", "seidel stern theorem", "stern-stolz theorem"], title: "Seidel–Stern theorem", statement: "a continued fraction 1/(b₁ + 1/(b₂ + …)) with all b_n > 0 converges if and only if Σ b_n diverges", extra: &[("Stern–Stolz", "if Σ |b_n| converges, the even and odd approximants converge to different limits")] },
    Fact { names: &["euler-minding formula", "euler minding formula", "euler-minding formulas", "euler's rule for continuants"], title: "Euler–Minding formulas", statement: "explicit sums for the numerators and denominators of the approximants: the continuant K(b₁, …, b_n) equals the sum of all products obtained from b₁b₂⋯b_n by deleting disjoint pairs of adjacent factors (each deletion of b_k b_(k+1) contributing a factor a_(k+1))", extra: &[("example", "K(x₁, x₂, x₃, x₄) = x₁x₂x₃x₄ + x₁x₂ + x₁x₄ + x₃x₄ + 1"), ("fundamental recurrences", "A_n = b_n A_(n−1) + a_n A_(n−2), B_n = b_n B_(n−1) + a_n B_(n−2)"), ("people", "Leonhard Euler; Ferdinand Minding (1869)")] },
    Fact { names: &["zaremba conjecture", "zaremba's conjecture"], title: "Zaremba's conjecture", statement: "every integer q > 1 has some p coprime to q such that all partial quotients of p/q are at most 5", extra: &[("status", "open; Bourgain and Kontorovich (2014) proved it for almost all q with bound 50, and Huang improved this to 5 for almost all q (2015)"), ("year", "Stanisław Zaremba, 1972")] },
    Fact { names: &["lagrange's continued fraction theorems", "lagrange's continued fraction theorem", "lagrange's theorem on continued fractions", "lagrange continued fraction theorem", "lagrange continued fraction theorems"], title: "Lagrange's continued fraction theorems", statement: "a real number has an eventually periodic regular continued fraction if and only if it is a quadratic irrational", extra: &[("Pell's equation", "the fundamental solution of x² − n y² = ±1 comes from the convergent before the end of the first period of √n"), ("best approximations", "every best approximation of the second kind is a convergent"), ("year", "1770 (periodicity; Euler proved the converse direction in 1737)")] },
    Fact { names: &["euler's continued fraction theorems", "euler's continued fraction formula", "euler continued fraction formula", "euler's continued fraction theorem"], title: "Euler's continued fraction theorems", statement: "Euler's formula: a₀ + a₀a₁ + a₀a₁a₂ + ⋯ + a₀⋯a_n = a₀/(1 − a₁/(1 + a₁ − a₂/(1 + a₂ − ⋯ a_n/(1 + a_n))))", extra: &[("rationals", "a number has a finite regular continued fraction if and only if it is rational"), ("periodic ⇒ quadratic", "every periodic continued fraction is a quadratic irrational (1737)"), ("e", "e = [2; 1, 2, 1, 1, 4, 1, 1, 6, …] (1737)"), ("series to fractions", "any series can be turned into a continued fraction with the same partial sums")] },
    Fact { names: &["galois theorem on continued fractions", "galois' continued fraction theorem", "galois continued fraction theorem"], title: "Galois's theorem", statement: "the continued fraction of a quadratic irrational x is purely periodic if and only if x > 1 and its conjugate lies in (−1, 0) (x is reduced); then −1/x̄ has the reversed period", extra: &[("year", "Évariste Galois, 1828")] },
    Fact { names: &["legendre's theorem on continued fractions", "legendre continued fraction theorem", "legendre's continued fraction theorem"], title: "Legendre's theorem", statement: "if |x − p/q| < 1/(2q²), then p/q is a convergent of the regular continued fraction of x", extra: &[("year", "Adrien-Marie Legendre, 1798")] },
    Fact { names: &["hurwitz's theorem", "hurwitz theorem on approximation", "hurwitz approximation theorem"], title: "Hurwitz's theorem", statement: "every irrational x has infinitely many rationals p/q with |x − p/q| < 1/(√5 q²); the constant √5 is best possible (the golden ratio)", extra: &[("proof idea", "of any three consecutive convergents, at least one satisfies the bound"), ("year", "Adolf Hurwitz, 1891")] },
    Fact { names: &["cf theorems that hold for regular continued fractions", "theorems for regular continued fractions", "regular continued fraction theorems"], title: "theorems for regular continued fractions", statement: "uniqueness, convergence, best approximation and periodicity results for [a₀; a₁, a₂, …]", extra: &[("convergence", "every infinite regular continued fraction converges, to an irrational"), ("alternation", "convergents alternate around x: p₀/q₀ < p₂/q₂ < ⋯ < x < ⋯ < p₃/q₃ < p₁/q₁"), ("determinant", "p_n q_(n−1) − p_(n−1) q_n = (−1)^(n−1)"), ("error", "1/(q_n(q_n + q_(n+1))) < |x − p_n/q_n| < 1/(q_n q_(n+1))"), ("best approximation", "convergents are the best rational approximations (Lagrange)"), ("Legendre", "|x − p/q| < 1/(2q²) ⇒ p/q is a convergent"), ("Lagrange", "eventually periodic ⇔ quadratic irrational"), ("Galois", "purely periodic ⇔ reduced quadratic irrational")] },
    Fact { names: &["continued fraction theorems involving almost all real numbers", "metric theory of continued fractions", "continued fraction theorems for almost all real numbers"], title: "metric theory of continued fractions", statement: "statements true for all real numbers outside a set of Lebesgue measure zero", extra: &[("Gauss–Kuzmin", "P(a_n = k) → −log₂(1 − 1/(k + 1)²)"), ("Khinchin", "(a₁⋯a_n)^(1/n) → 2.685452001…"), ("Lévy", "q_n^(1/n) → e^(π²/(12 ln 2)) = 3.275822918…"), ("Lochs", "each partial quotient determines ≈ π²/(6 ln 2 ln 10) ≈ 1.0306 decimal digits"), ("Khinchin (harmonic)", "n/(1/a₁ + ⋯ + 1/a_n) → 1.745405662…"), ("Borel–Bernstein", "a_n > φ(n) infinitely often iff Σ 1/φ(n) diverges"), ("arithmetic mean", "(a₁ + ⋯ + a_n)/n → ∞")] },
    Fact { names: &["continued fraction results involving quadratic irrationals", "continued fractions of quadratic irrationals", "quadratic irrational continued fractions"], title: "continued fractions of quadratic irrationals", statement: "a real number has an eventually periodic continued fraction exactly when it is a quadratic irrational (Euler, Lagrange)", extra: &[("Galois", "purely periodic ⇔ x > 1 and −1 < x̄ < 0"), ("√n", "√n = [a₀; (a₁, a₂, …, a₂, a₁, 2a₀)‾]: the period ends in 2a₀ and is a palindrome before that"), ("Pell", "p_(k−1)² − n q_(k−1)² = (−1)^k for period length k"), ("bounded quotients", "quadratic irrationals are badly approximable")] },
    Fact { names: &["stieltjes fraction", "s-fraction", "stieltjes continued fraction"], title: "Stieltjes fraction (S-fraction)", statement: "a continued fraction a₁/(1 − a₂z/(1 − a₃z/(1 − ⋯))) (equivalently K(a_n z/1)) with all a_n > 0; it corresponds to a Stieltjes moment problem", extra: &[("J-fractions", "1/(z − b₁ − a₁/(z − b₂ − a₂/(…))) correspond to the Hamburger moment problem and orthogonal polynomials"), ("convergence", "Stieltjes (1894): written as 1/(k₁z + 1/(k₂ + 1/(k₃z + ⋯))) with k_n > 0, it converges for z off the negative real axis if and only if Σ k_n diverges")] },
    Fact { names: &["backward algorithm for a continued fraction", "backward recurrence algorithm", "backward algorithm", "backward recurrence"], title: "backward recurrence algorithm", statement: "evaluate the n-th approximant from the tail inward: t_n = 0, t_(k−1) = a_k/(b_k + t_k) for k = n, …, 1, and f_n = b₀ + t₀", extra: &[("cost", "about n divisions, but n must be chosen in advance"), ("stability", "numerically very stable; preferred for evaluating special functions"), ("forward alternative", "A_n = b_n A_(n−1) + a_n A_(n−2), B_n = b_n B_(n−1) + a_n B_(n−2), f_n = A_n/B_n"), ("modified Lentz", "a forward method that avoids the A/B overflow and stops adaptively")] },
    Fact { names: &["nearest integer continued fraction expansion algorithm", "nearest integer continued fraction algorithm", "nearest integer continued fraction"], title: "nearest integer continued fraction", statement: "like the regular expansion but rounding to the nearest integer: a₀ = round(x), x₁ = 1/(x − a₀), …, so x = a₀ + ε₁/(a₁ + ε₂/(a₂ + ⋯)) with ε_k = ±1 and |a_k| ≥ 2", extra: &[("convergents", "a subsequence of the regular convergents (some are skipped)"), ("speed", "on average it needs ln φ/ln 2 ≈ 0.694 times as many steps as the regular expansion"), ("history", "Minnigerode (1873), Hurwitz (1889)")] },
    Fact { names: &["hurwitz continued fraction algorithm", "hurwitz complex continued fraction", "hurwitz continued fraction"], title: "Hurwitz complex continued fraction", statement: "for complex z: a₀ = the Gaussian integer nearest z, z₁ = 1/(z − a₀), and repeat, giving z = a₀ + 1/(a₁ + 1/(a₂ + ⋯)) with Gaussian-integer partial quotients", extra: &[("convergence", "every complete quotient has |z_k| ≥ √2 for k ≥ 1, which makes the convergents converge to z"), ("rationals", "terminates exactly for z ∈ ℚ(i)"), ("year", "Adolf Hurwitz, 1887")] },
    Fact { names: &["schmidt algorithm", "schmidt continued fraction", "schmidt's continued fractions"], title: "Schmidt's complex continued fractions", statement: "Asmus Schmidt's (1975) continued fraction algorithm for complex numbers, built on partitions of the plane into Farey circles and triangles; it gives best approximations by elements of ℚ(i)", extra: &[("relation", "an alternative to Hurwitz's nearest-Gaussian-integer algorithm with better approximation properties")] },
    Fact { names: &["jacobi-perron algorithm", "jacobi perron algorithm", "jacobi–perron algorithm"], title: "Jacobi–Perron algorithm", statement: "a multidimensional continued fraction: for (x, y) ∈ [0, 1)² with x ≠ 0, the map T(x, y) = ({y/x}, {1/x}) produces pairs of partial quotients (⌊y/x⌋, ⌊1/x⌋) and simultaneous rational approximations", extra: &[("periodicity", "Jacobi conjectured that cubic irrationals give periodic expansions; this is open (Hermite's problem)"), ("people", "Carl Gustav Jacobi (1868), Oskar Perron (1907)")] },
    Fact { names: &["continued fractions for besselj", "bessel continued fraction", "continued fraction for besselj", "continued fraction bessel j"], title: "continued fraction for Bessel functions", statement: "J_ν(x)/J_(ν−1)(x) = 1/(2ν/x − 1/(2(ν+1)/x − 1/(2(ν+2)/x − ⋯)))", extra: &[("from", "the recurrence J_(ν−1)(x) + J_(ν+1)(x) = (2ν/x) J_ν(x)"), ("equivalently", "J_ν/J_(ν−1) = (x/(2ν))/(1 − (x²/(4ν(ν+1)))/(1 − (x²/(4(ν+1)(ν+2)))/(1 − ⋯)))"), ("modified Bessel", "I_ν(x)/I_(ν−1)(x) = 1/(2ν/x + 1/(2(ν+1)/x + 1/(2(ν+2)/x + ⋯)))"), ("use", "Steed's and Miller's algorithms evaluate Bessel functions with these ratios")] },
    Fact { names: &["hypergeometric cf identities", "hypergeometric continued fraction identities", "gauss's continued fraction", "gauss continued fraction", "hypergeometric continued fractions"], title: "Gauss's continued fraction", statement: "₂F₁(a, b+1; c+1; z)/₂F₁(a, b; c; z) = 1/(1 − k₁z/(1 − k₂z/(1 − k₃z/(1 − ⋯))))", extra: &[("odd coefficients", "k_(2n+1) = (a + n)(c − b + n)/((c + 2n)(c + 2n + 1))"), ("even coefficients", "k_(2n) = (b + n)(c − a + n)/((c + 2n − 1)(c + 2n))"), ("confluent (Kummer)", "₁F₁(a+1; b+1; z)/₁F₁(a; b; z) = 1/(1 − ((b−a)/(b(b+1))) z/(1 + ((a+1)/((b+1)(b+2))) z/(1 − ⋯)))"), ("₀F₁", "₀F₁(; b+1; z)/₀F₁(; b; z) = 1/(1 + (z/(b(b+1)))/(1 + (z/((b+1)(b+2)))/(1 + ⋯)))"), ("special cases", "arctan, log(1 + z), tanh and the incomplete gamma function")] },
];

fn facts(l: &str) -> Option<Answer> {
    let l = l.trim_start_matches("what is ").trim_start_matches("the ").trim_start_matches("a ");
    let f = FACTS.iter().find(|f| f.names.contains(&l))?;
    let rows: Vec<Vec<Content>> = f.extra.iter().map(|(a, b)| vec![text(*a), text(*b)]).collect();
    let mut pods = vec![input(f.title), primary(pod("Result", "Statement", text(f.statement)))];
    if !rows.is_empty() {
        pods.push(pod("Details", "Details", table(None, rows)));
    }
    Some(Answer::new(f.title, pods))
}

/// Exact periodic expansion of √n: (a₀, period).
fn sqrt_cf(n: u64) -> Option<(u64, Vec<u64>)> {
    let a0 = (n as f64).sqrt().floor() as u64;
    let a0 = (a0.saturating_sub(1)..=a0 + 1).filter(|k| k * k <= n).max()?;
    if a0 * a0 == n {
        return None;
    }
    let (mut m, mut d, mut a) = (0u64, 1u64, a0);
    let mut period = Vec::new();
    while period.len() < 2000 {
        m = d * a - m;
        d = (n - m * m) / d;
        a = (a0 + m) / d;
        period.push(a);
        if a == 2 * a0 {
            break;
        }
    }
    Some((a0, period))
}

fn join(v: &[impl ToString]) -> String {
    v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")
}

fn apply_sqrt(n: u64) -> Option<Answer> {
    let (a0, per) = sqrt_cf(n)?;
    let k = per.len();
    let mut terms = vec![BigInt::from(a0)];
    terms.extend(per.iter().map(|&x| BigInt::from(x)));
    let conv = crate::numtheory::convergents(&terms);
    let (p, q) = conv[k - 1].clone();
    let sign = if k % 2 == 0 { "1" } else { "−1" };
    let mut rows = vec![
        vec![text("regular continued fraction"), text(format!("[{a0}; ({})‾]", join(&per)))],
        vec![text("period length"), text(k.to_string())],
        vec![text("Lagrange (periodic ⇔ quadratic irrational)"), text(format!("√{n} is a root of x² − {n} = 0, so its expansion is eventually periodic"))],
        vec![text("Galois (purely periodic ⇔ reduced)"), text(format!("√{n} + {a0} is reduced (> 1, conjugate {a0} − √{n} ∈ (−1, 0)), so √{n} + {a0} = [({}, {})‾] is purely periodic", 2 * a0, join(&per[..k - 1])))],
        vec![text("symmetry"), text(format!("the period without its last term ({}) is a palindrome; the last term is 2a₀ = {}", join(&per[..k - 1]), 2 * a0))],
        vec![text("Pell's equation"), text(format!("{p}² − {n}·{q}² = {sign} (from the convergent {p}/{q})"))],
    ];
    if k % 2 == 1 {
        let (p2, q2) = conv[2 * k - 1].clone();
        rows.push(vec![text("Pell (+1)"), text(format!("{p2}² − {n}·{q2}² = 1"))]);
    }
    let cv: Vec<String> = conv.iter().take(6).map(|(p, q)| format!("{p}/{q}")).collect();
    rows.push(vec![text("first convergents"), text(cv.join(", "))]);
    let (p5, q5) = conv[conv.len().min(6) - 1].clone();
    let x = (n as f64).sqrt();
    let approx = p5.to_string().parse::<f64>().ok()? / q5.to_string().parse::<f64>().ok()?;
    let qf: f64 = q5.to_string().parse().ok()?;
    rows.push(vec![text("Legendre / best approximation"), text(format!("|√{n} − {p5}/{q5}| ≈ {:.3e} < 1/(2·{q5}²) ≈ {:.3e}", (x - approx).abs(), 1.0 / (2.0 * qf * qf)))]);
    Some(Answer::new("continued fraction theorems", vec![input(&format!("continued fraction theorems applied to √{n}")), primary(pod("Result", "Continued fraction", text(format!("√{n} = [{a0}; ({})‾]", join(&per))))), pod("Theorems", "Theorems applied", table(None, rows))]))
}

fn nearest_integer_cf(x: f64, n: usize) -> Vec<i64> {
    let mut out = Vec::new();
    let mut x = x;
    for _ in 0..n {
        let a = x.round();
        out.push(a as i64);
        let f = x - a;
        if f.abs() < 1e-9 {
            break;
        }
        x = 1.0 / f;
    }
    out
}

fn value(s: &str) -> Option<f64> {
    let e = ex(s)?;
    if !e.free_symbols().is_empty() {
        return None;
    }
    crate::eval::eval_f64(&e, &[]).filter(|v| v.is_finite())
}

fn computations(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:apply (?:the )?continued fraction theorems to|continued fraction theorems (?:for|applied to)) (?:sqrt\((\d+)\)|√(\d+)|sqrt (\d+))$").captures(l) {
        let n: u64 = m.get(1).or(m.get(2)).or(m.get(3))?.as_str().parse().ok()?;
        return apply_sqrt(n);
    }
    if let Some(m) = re!(r"^(?:the )?nearest[- ]integer continued fraction (?:expansion )?(?:of|for) (.+)$").captures(l) {
        let x = value(&m[1])?;
        let a = nearest_integer_cf(x, 8);
        let mut s = a[0].to_string();
        let mut tail = String::new();
        for t in a.iter().skip(1) {
            tail.push_str(&format!(" + 1/({t}"));
        }
        tail.push_str(&")".repeat(a.len().saturating_sub(1)));
        s.push_str(&tail);
        let reg = crate::numtheory::continued_fraction_f64(x, 11);
        return Some(Answer::new("nearest integer continued fraction", vec![input(&format!("nearest integer continued fraction of {}", m[1].trim())), primary(pod("Result", "Nearest integer continued fraction", text(format!("{s}{}", if a.len() == 8 { " …" } else { "" })))), pod("Terms", "Partial quotients (signed)", text(format!("[{}]", join(&a)))), pod("Regular", "Regular continued fraction for comparison", text(format!("[{}; {}]", reg[0], join(&reg[1..]))))]));
    }
    if let Some(m) = re!(r"^(?:the )?(?:first (\d+) )?convergents (?:of|for) (.+)$").captures(l) {
        let k: usize = m.get(1).and_then(|x| x.as_str().parse().ok()).unwrap_or(10).clamp(1, 20);
        let src = m[2].trim();
        let terms: Vec<BigInt> = if let Some(r) = re!(r"^(-?\d+)\s*/\s*(\d+)$").captures(src) {
            crate::numtheory::continued_fraction(&r[1].parse().ok()?, &r[2].parse().ok()?)
        } else if let Some(r) = re!(r"^\[\s*(-?\d+)\s*[;,]\s*([\d,\s]+)\]$").captures(src) {
            let mut v = vec![r[1].parse::<BigInt>().ok()?];
            for t in r[2].split(',') {
                v.push(t.trim().parse().ok()?);
            }
            v
        } else {
            crate::numtheory::continued_fraction_f64(value(src)?, k.min(11)).into_iter().map(BigInt::from).collect()
        };
        let conv = crate::numtheory::convergents(&terms);
        let rows: Vec<Vec<Content>> = conv.iter().take(k).enumerate().map(|(i, (p, q))| {
            let v = p.to_string().parse::<f64>().unwrap_or(0.0) / q.to_string().parse::<f64>().unwrap_or(1.0);
            vec![text(i.to_string()), text(terms[i].to_string()), text(format!("{p}/{q}")), text(crate::print::fmt_float(v, 12))]
        }).collect();
        let list: Vec<String> = conv.iter().take(k).map(|(p, q)| if q == &BigInt::from(1) { p.to_string() } else { format!("{p}/{q}") }).collect();
        return Some(Answer::new("convergents", vec![input(&format!("convergents of {src}")), primary(pod("Result", "Convergents", text(list.join(", ")))), pod("Table", "Partial quotients and convergents", table(Some(vec!["n", "a_n", "p_n/q_n", "value"]), rows))]));
    }
    None
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    facts(&l).or_else(|| computations(&l))
}

#[cfg(test)]
mod tests {
    fn ans(s: &str) -> String {
        serde_json::to_string(&crate::engine::query(s).pods).unwrap_or_default()
    }

    #[test]
    fn continued_fraction_theory() {
        let a = ans("apply continued fraction theorems to sqrt(7)");
        assert!(a.contains("[2; (1, 1, 1, 4)‾]") && a.contains("8² − 7·3² = 1"));
        assert!(ans("convergents of pi").contains("355/113"));
        assert!(ans("Zaremba conjecture").contains("at most 5"));
        assert!(ans("nearest integer continued fraction of pi").contains("[3, 7, 16, -294"));
    }
}
