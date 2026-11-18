//! Probability beyond the basic distribution pods: expectations of
//! functions of random variables (exact for polynomials through symbolic
//! moments), probabilities of events and conditional events, moments,
//! central moments and cumulants of twenty distributions, birthday
//! problems, Bernoulli-trial questions (counts, waiting times, runs),
//! probability rules, and descriptive statistics (skewness, kurtosis,
//! geometric and harmonic means, outliers).

use super::nl::ex;
use super::*;
use crate::expr::Expr::*;
use crate::expr::*;
use crate::print::text as ptext;
use crate::simplify as s;
use crate::stats::fmt;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use regex::Regex;
use statrs::function::gamma::ln_gamma;
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

fn num6(v: f64) -> String {
    crate::print::fmt_float(v, 8)
}

// ---------------------------------------------------------------------------
// distributions with symbolic or numeric parameters
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Normal,
    Exponential,
    Gamma,
    Beta,
    Uniform,
    ChiSq,
    StudentT,
    F,
    Rayleigh,
    LogNormal,
    Weibull,
    Laplace,
    Cauchy,
    Logistic,
    Moyal,
    Poisson,
    Binomial,
    Bernoulli,
    Geometric,
    Pascal,
    Zipf,
    Benford,
}

struct Dist {
    kind: Kind,
    params: Vec<Expr>,
}

impl Kind {
    fn info(self) -> (&'static str, &'static [&'static str], bool) {
        use Kind::*;
        match self {
            Normal => ("normal distribution", &["μ", "σ"], false),
            Exponential => ("exponential distribution", &["λ"], false),
            Gamma => ("gamma distribution", &["α", "β"], false),
            Beta => ("beta distribution", &["α", "β"], false),
            Uniform => ("uniform distribution", &["a", "b"], false),
            ChiSq => ("χ² distribution", &["ν"], false),
            StudentT => ("Student t distribution", &["ν"], false),
            F => ("F distribution", &["n", "m"], false),
            Rayleigh => ("Rayleigh distribution", &["σ"], false),
            LogNormal => ("log-normal distribution", &["μ", "σ"], false),
            Weibull => ("Weibull distribution", &["α", "β"], false),
            Laplace => ("Laplace distribution", &["μ", "β"], false),
            Cauchy => ("Cauchy distribution", &["a", "b"], false),
            Logistic => ("logistic distribution", &["μ", "β"], false),
            Moyal => ("Moyal distribution", &["μ", "σ"], false),
            Poisson => ("Poisson distribution", &["λ"], true),
            Binomial => ("binomial distribution", &["n", "p"], true),
            Bernoulli => ("Bernoulli distribution", &["p"], true),
            Geometric => ("geometric distribution", &["p"], true),
            Pascal => ("Pascal distribution", &["n", "p"], true),
            Zipf => ("Zipf distribution", &["ρ"], true),
            Benford => ("Benford distribution", &["b"], true),
        }
    }
}

fn r(n: i64) -> Expr {
    int(n)
}

/// Stirling numbers of the second kind S(k, j).
fn stirling2(k: usize, j: usize) -> i64 {
    let mut t = vec![vec![0i64; k + 1]; k + 1];
    t[0][0] = 1;
    for n in 1..=k {
        for m in 1..=n {
            t[n][m] = m as i64 * t[n - 1][m] + t[n - 1][m - 1];
        }
    }
    t[k][j]
}

/// Γ(1 + k/2) exactly.
fn gamma_half(k: i64) -> Expr {
    if k % 2 == 0 {
        let m = k / 2;
        r((1..=m).product::<i64>().max(1))
    } else {
        // Γ(m + 1/2) with m = (k + 1)/2: (2m)! √π/(4^m m!)
        let m = (k + 1) / 2;
        let num: i64 = (1..=2 * m).product();
        let den: i64 = 4i64.pow(m as u32) * (1..=m).product::<i64>().max(1);
        mul(vec![div(r(num), r(den)), sqrt(pi())])
    }
}

impl Dist {
    fn p(&self, i: usize) -> Expr {
        self.params[i].clone()
    }
    fn pf(&self, i: usize) -> Option<f64> {
        self.params.get(i)?.to_f64_const()
    }
    fn numeric(&self) -> bool {
        self.params.iter().all(|p| p.to_f64_const().is_some())
    }
    fn discrete(&self) -> bool {
        self.kind.info().2
    }
    fn name(&self) -> String {
        let (n, names, _) = self.kind.info();
        let ps: Vec<String> = names.iter().zip(&self.params).map(|(a, b)| if b.free_symbols().is_empty() { format!("{a} = {}", ptext(b)) } else { String::new() }).filter(|x| !x.is_empty()).collect();
        if ps.is_empty() { n.to_string() } else { format!("{n} ({})", ps.join(", ")) }
    }
    /// Cumulants for distributions given most naturally by them.
    fn cumulant_direct(&self, k: usize) -> Option<Expr> {
        if self.kind != Kind::Moyal {
            return None;
        }
        let (mu, sg) = (self.p(0), self.p(1));
        Some(match k {
            1 => add(vec![mu, mul(vec![sg, add(vec![Const(K::EulerGamma), ln(r(2))])])]),
            2 => mul(vec![div(square(pi()), r(2)), square(sg)]),
            3 => mul(vec![r(14), func("zeta", vec![r(3)]), pow(sg, r(3))]),
            4 => mul(vec![pow(pi(), r(4)), pow(sg, r(4))]),
            _ => return None,
        })
    }
    /// E[X^k], exact where possible.
    fn raw(&self, k: usize) -> Option<Expr> {
        use Kind::*;
        if k == 0 {
            return Some(one());
        }
        let ki = k as i64;
        let e = match self.kind {
            Normal => {
                let (mu, s2) = (self.p(0), square(self.p(1)));
                let mut m = vec![one(), mu.clone()];
                for j in 2..=k {
                    m.push(add(vec![mul(vec![mu.clone(), m[j - 1].clone()]), mul(vec![r(j as i64 - 1), s2.clone(), m[j - 2].clone()])]));
                }
                m[k].clone()
            }
            Exponential => div(r((1..=ki).product()), pow(self.p(0), r(ki))),
            Gamma => mul(vec![pow(self.p(1), r(ki)), mul((0..ki).map(|j| add(vec![self.p(0), r(j)])).collect())]),
            Beta => mul((0..ki).map(|j| div(add(vec![self.p(0), r(j)]), add(vec![self.p(0), self.p(1), r(j)]))).collect()),
            Uniform => div(sub(pow(self.p(1), r(ki + 1)), pow(self.p(0), r(ki + 1))), mul(vec![r(ki + 1), sub(self.p(1), self.p(0))])),
            ChiSq => mul((0..ki).map(|j| add(vec![self.p(0), r(2 * j)])).collect()),
            F => {
                let (n, m) = (self.p(0), self.p(1));
                if let Some(mv) = self.pf(1) {
                    if mv <= 2.0 * k as f64 {
                        return Some(Const(K::Inf));
                    }
                }
                mul(vec![pow(div(m.clone(), n.clone()), r(ki)), mul((0..ki).map(|j| div(add(vec![div(n.clone(), r(2)), r(j)]), sub(div(m.clone(), r(2)), r(j + 1)))).collect())])
            }
            Rayleigh => mul(vec![pow(self.p(0), r(ki)), pow(r(2), div(r(ki), r(2))), gamma_half(ki)]),
            LogNormal => exp(add(vec![mul(vec![r(ki), self.p(0)]), mul(vec![div(r(ki * ki), r(2)), square(self.p(1))])])),
            Weibull => mul(vec![pow(self.p(1), r(ki)), func("Gamma", vec![add(vec![one(), div(r(ki), self.p(0))])])]),
            Laplace => {
                // X = μ + Y, E[Y^j] = j! β^j for even j, 0 for odd
                add((0..=ki).filter(|j| j % 2 == 0).map(|j| mul(vec![r(crate::numtheory::binomial(&BigInt::from(ki), j as u64).to_i64().unwrap_or(0)), pow(self.p(0), r(ki - j)), r((1..=j).product::<i64>().max(1)), pow(self.p(1), r(j))])).collect())
            }
            Logistic => {
                if k > 2 {
                    return None;
                }
                if k == 1 { self.p(0) } else { add(vec![square(self.p(0)), div(mul(vec![square(pi()), square(self.p(1))]), r(3))]) }
            }
            Cauchy | StudentT => {
                if self.kind == StudentT {
                    if k % 2 == 1 {
                        return Some(zero());
                    }
                    if k == 2 {
                        return Some(div(self.p(0), sub(self.p(0), r(2))));
                    }
                    if k == 4 {
                        return Some(div(mul(vec![r(3), square(self.p(0))]), mul(vec![sub(self.p(0), r(2)), sub(self.p(0), r(4))])));
                    }
                }
                return None;
            }
            Moyal => {
                // moments from cumulants: m_n = Σ C(n−1, i−1) κ_i m_(n−i)
                let mut m = vec![one()];
                for n in 1..=k {
                    let mut terms = Vec::new();
                    for i in 1..=n {
                        let c = crate::numtheory::binomial(&BigInt::from(n - 1), (i - 1) as u64).to_i64()?;
                        terms.push(mul(vec![r(c), self.cumulant_direct(i)?, m[n - i].clone()]));
                    }
                    m.push(add(terms));
                }
                m[k].clone()
            }
            Poisson => add((1..=k).map(|j| mul(vec![r(stirling2(k, j)), pow(self.p(0), r(j as i64))])).collect()),
            Binomial => add((1..=k).map(|j| mul(vec![r(stirling2(k, j)), mul((0..j as i64).map(|i| sub(self.p(0), r(i))).collect()), pow(self.p(1), r(j as i64))])).collect()),
            Bernoulli => self.p(0),
            Geometric => {
                let qp = div(sub(one(), self.p(0)), self.p(0));
                add((1..=k).map(|j| mul(vec![r(stirling2(k, j)), r((1..=j as i64).product()), pow(qp.clone(), r(j as i64))])).collect())
            }
            Pascal => {
                // X = n + Y, Y failures: E[Y^(j)] = n(n+1)…(n+j−1) (q/p)^j
                let n = self.p(0);
                let qp = div(sub(one(), self.p(1)), self.p(1));
                let ey = |i: usize| -> Expr {
                    if i == 0 {
                        return one();
                    }
                    add((1..=i).map(|j| mul(vec![r(stirling2(i, j)), mul((0..j as i64).map(|t| add(vec![n.clone(), r(t)])).collect()), pow(qp.clone(), r(j as i64))])).collect())
                };
                add((0..=k).map(|i| mul(vec![r(crate::numtheory::binomial(&BigInt::from(k), i as u64).to_i64().unwrap_or(0)), pow(n.clone(), r((k - i) as i64)), ey(i)])).collect())
            }
            Zipf => {
                if let Some(rho) = self.pf(0) {
                    if rho <= k as f64 {
                        return Some(Const(K::Inf));
                    }
                }
                div(func("zeta", vec![add(vec![self.p(0), one(), r(-ki)])]), func("zeta", vec![add(vec![self.p(0), one()])]))
            }
            Benford => {
                let b = self.pf(0)? as i64;
                if !(2..=1000).contains(&b) {
                    return None;
                }
                add((1..b).map(|d| mul(vec![pow(r(d), r(ki)), div(ln(div(r(d + 1), r(d))), ln(r(b)))])).collect())
            }
        };
        Some(e)
    }
    fn central(&self, k: usize) -> Option<Expr> {
        let mu = self.raw(1)?;
        let mut terms = Vec::new();
        for i in 0..=k {
            let c = crate::numtheory::binomial(&BigInt::from(k), i as u64).to_i64()?;
            let sign = if (k - i) % 2 == 0 { 1 } else { -1 };
            terms.push(mul(vec![r(sign * c), self.raw(i)?, pow(mu.clone(), r((k - i) as i64))]));
        }
        Some(add(terms))
    }
    fn cumulant(&self, k: usize) -> Option<Expr> {
        if let Some(c) = self.cumulant_direct(k) {
            return Some(c);
        }
        let m: Vec<Expr> = (0..=k).map(|i| self.raw(i)).collect::<Option<_>>()?;
        let mut kap: Vec<Expr> = vec![zero()];
        for n in 1..=k {
            let mut t = m[n].clone();
            for i in 1..n {
                let c = crate::numtheory::binomial(&BigInt::from(n - 1), (i - 1) as u64).to_i64()?;
                t = sub(t, mul(vec![r(c), kap[i].clone(), m[n - i].clone()]));
            }
            kap.push(t);
        }
        Some(kap[k].clone())
    }
    /// Numeric density or mass function.
    fn pdf(&self, x: f64) -> f64 {
        use Kind::*;
        let p = |i: usize| self.pf(i).unwrap_or(f64::NAN);
        let lnfact = |n: f64| ln_gamma(n + 1.0);
        match self.kind {
            Normal => (-(x - p(0)).powi(2) / (2.0 * p(1).powi(2))).exp() / (p(1) * (2.0 * std::f64::consts::PI).sqrt()),
            Exponential => if x < 0.0 { 0.0 } else { p(0) * (-p(0) * x).exp() },
            Gamma => if x <= 0.0 { 0.0 } else { ((p(0) - 1.0) * x.ln() - x / p(1) - ln_gamma(p(0)) - p(0) * p(1).ln()).exp() },
            Beta => if x <= 0.0 || x >= 1.0 { 0.0 } else { ((p(0) - 1.0) * x.ln() + (p(1) - 1.0) * (1.0 - x).ln() + ln_gamma(p(0) + p(1)) - ln_gamma(p(0)) - ln_gamma(p(1))).exp() },
            Uniform => if x < p(0) || x > p(1) { 0.0 } else { 1.0 / (p(1) - p(0)) },
            ChiSq => if x <= 0.0 { 0.0 } else { ((p(0) / 2.0 - 1.0) * x.ln() - x / 2.0 - ln_gamma(p(0) / 2.0) - p(0) / 2.0 * 2f64.ln()).exp() },
            StudentT => {
                let v = p(0);
                (ln_gamma((v + 1.0) / 2.0) - ln_gamma(v / 2.0) - 0.5 * (v * std::f64::consts::PI).ln() - (v + 1.0) / 2.0 * (1.0 + x * x / v).ln()).exp()
            }
            F => {
                let (n, m) = (p(0), p(1));
                if x <= 0.0 { 0.0 } else { (0.5 * (n * (n * x).ln() + m * m.ln() - (n + m) * (n * x + m).ln()) - x.ln() - (ln_gamma(n / 2.0) + ln_gamma(m / 2.0) - ln_gamma((n + m) / 2.0))).exp() }
            }
            Rayleigh => if x < 0.0 { 0.0 } else { x / p(0).powi(2) * (-x * x / (2.0 * p(0).powi(2))).exp() },
            LogNormal => if x <= 0.0 { 0.0 } else { (-(x.ln() - p(0)).powi(2) / (2.0 * p(1).powi(2))).exp() / (x * p(1) * (2.0 * std::f64::consts::PI).sqrt()) },
            Weibull => if x < 0.0 { 0.0 } else { p(0) / p(1) * (x / p(1)).powf(p(0) - 1.0) * (-(x / p(1)).powf(p(0))).exp() },
            Laplace => (-(x - p(0)).abs() / p(1)).exp() / (2.0 * p(1)),
            Cauchy => 1.0 / (std::f64::consts::PI * p(1) * (1.0 + ((x - p(0)) / p(1)).powi(2))),
            Logistic => {
                let z = (-(x - p(0)) / p(1)).exp();
                z / (p(1) * (1.0 + z).powi(2))
            }
            Moyal => {
                let z = (x - p(0)) / p(1);
                (-(z + (-z).exp()) / 2.0).exp() / (p(1) * (2.0 * std::f64::consts::PI).sqrt())
            }
            Poisson => if x < 0.0 { 0.0 } else { (x * p(0).ln() - p(0) - lnfact(x)).exp() },
            Binomial => if x < 0.0 || x > p(0) { 0.0 } else { (lnfact(p(0)) - lnfact(x) - lnfact(p(0) - x) + x * p(1).ln() + (p(0) - x) * (1.0 - p(1)).ln()).exp() },
            Bernoulli => if x == 1.0 { p(0) } else if x == 0.0 { 1.0 - p(0) } else { 0.0 },
            Geometric => if x < 0.0 { 0.0 } else { p(0) * (1.0 - p(0)).powf(x) },
            Pascal => if x < p(0) { 0.0 } else { (lnfact(x - 1.0) - lnfact(p(0) - 1.0) - lnfact(x - p(0)) + p(0) * p(1).ln() + (x - p(0)) * (1.0 - p(1)).ln()).exp() },
            Zipf => if x < 1.0 { 0.0 } else { x.powf(-(p(0) + 1.0)) / crate::special::zeta(p(0) + 1.0) },
            Benford => if x < 1.0 || x >= p(0) { 0.0 } else { (1.0 + 1.0 / x).ln() / p(0).ln() },
        }
    }
    /// Effective support for numerics.
    fn range(&self) -> (f64, f64) {
        use Kind::*;
        let p = |i: usize| self.pf(i).unwrap_or(1.0);
        match self.kind {
            Uniform => (p(0), p(1)),
            Beta => (0.0, 1.0),
            Exponential | Gamma | ChiSq | F | Rayleigh | LogNormal | Weibull => (0.0, f64::INFINITY),
            Poisson | Geometric | Bernoulli => (0.0, f64::INFINITY),
            Binomial => (0.0, p(0)),
            Pascal => (p(0), f64::INFINITY),
            Zipf => (1.0, f64::INFINITY),
            Benford => (1.0, p(0) - 1.0),
            _ => (f64::NEG_INFINITY, f64::INFINITY),
        }
    }
    /// E[h(X)] numerically.
    fn expect(&self, h: &dyn Fn(f64) -> f64) -> Option<f64> {
        if !self.numeric() {
            return None;
        }
        if self.discrete() {
            let (a, b) = self.range();
            let mut sum = 0.0;
            let mut k = a;
            let mut tail = 0;
            while k <= b && k < a + 1e7 {
                let w = self.pdf(k);
                let t = w * h(k);
                sum += t;
                if w < 1e-18 && k > a + 50.0 {
                    tail += 1;
                    if tail > 200 {
                        break;
                    }
                }
                k += 1.0;
            }
            return Some(sum);
        }
        let (a, b) = self.range();
        let q = crate::numeric::quad(&|x| self.pdf(x) * h(x), a, b, 1e-10);
        q.value.is_finite().then_some(q.value)
    }
}

fn kw_num(l: &str, keys: &[&str]) -> Option<Expr> {
    for k in keys {
        let pat = format!(r"(?:^|[\s,(]){}\s*(?:=|is|of)?\s*(-?[\d.]+(?:/\d+)?)", regex::escape(k));
        if let Some(c) = Regex::new(&pat).ok()?.captures(l) {
            return ex(&c[1]);
        }
    }
    None
}

/// Parse a distribution description; unspecified parameters stay symbolic.
fn parse_dist(t: &str) -> Option<Dist> {
    use Kind::*;
    let t = t.trim().trim_end_matches(['.', '?']);
    let kinds: &[(&str, Kind)] = &[
        (r"standard normal", Normal),
        (r"log-?normal", LogNormal),
        (r"normal(?:ly distributed)?|gaussian|\bn\(", Normal),
        (r"exponential(?:ly distributed)?|\bexp\(", Exponential),
        (r"gamma", Gamma),
        (r"beta", Beta),
        (r"(?:continuous )?uniform(?:ly distributed)?", Uniform),
        (r"chi-?\s?squared?|chisquared?|χ²|χ\^2", ChiSq),
        (r"student'?s? ?t|\bt(?: distribution|\()", StudentT),
        (r"\bf(?: distribution|\(| dist)", F),
        (r"rayleigh", Rayleigh),
        (r"weibull", Weibull),
        (r"laplace", Laplace),
        (r"cauchy", Cauchy),
        (r"logistic", Logistic),
        (r"moyal", Moyal),
        (r"poisson", Poisson),
        (r"binomial", Binomial),
        (r"bernoulli", Bernoulli),
        (r"geometric", Geometric),
        (r"pascal|negative binomial", Pascal),
        (r"zipf", Zipf),
        (r"benford", Benford),
    ];
    // the name right before "distribution" wins, so "logistic distribution mu=2 beta=3" is not a beta distribution
    let named = |p: &str| Regex::new(&format!(r"(?i)(?:{p})\s+(?:distribution|dist)\b")).map(|r| r.is_match(t)).unwrap_or(false);
    let kind = kinds
        .iter()
        .find(|(p, _)| named(p))
        .or_else(|| kinds.iter().find(|(p, _)| Regex::new(&format!("(?i){p}")).map(|r| r.is_match(t)).unwrap_or(false)))?
        .1;
    let (_, names, _) = kind.info();
    let mut params: Vec<Option<Expr>> = vec![None; names.len()];
    if t.contains("standard normal") {
        params = vec![Some(zero()), Some(one())];
    }
    // positional arguments "(a, b)"
    if let Some(c) = re!(r"\(([^()]*)\)").captures(t) {
        let args: Vec<Option<Expr>> = c[1].split(',').map(|a| ex(a.trim())).collect();
        if args.iter().all(|a| a.as_ref().map(|e| e.free_symbols().is_empty()).unwrap_or(false)) && !args.is_empty() && args.len() <= names.len() {
            for (i, a) in args.into_iter().enumerate() {
                params[i] = a;
            }
        }
    }
    let l = t.to_lowercase();
    let keys: Vec<&[&str]> = match kind {
        Normal | LogNormal | Moyal => vec![&["mu", "μ", "mean", "location"], &["sigma", "σ", "sd", "standard deviation", "scale"]],
        Exponential | Poisson => vec![&["lambda", "λ", "rate", "mean"]],
        Gamma | Beta | Weibull => vec![&["alpha", "α", "shape", "a"], &["beta", "β", "scale", "b"]],
        Uniform | Cauchy => vec![&["a", "min", "location"], &["b", "max", "scale"]],
        ChiSq | StudentT => vec![&["dof", "df", "nu", "ν", "degrees of freedom", "k"]],
        F => vec![&["n"], &["m"]],
        Rayleigh => vec![&["sigma", "σ", "scale"]],
        Laplace | Logistic => vec![&["mu", "μ", "location"], &["beta", "β", "scale", "b"]],
        Binomial | Pascal => vec![&["n", "trials"], &["p", "probability"]],
        Bernoulli | Geometric => vec![&["p", "probability"]],
        Zipf => vec![&["rho", "ρ", "s"]],
        Benford => vec![&["b", "base"]],
    };
    for (i, ks) in keys.iter().enumerate() {
        if params[i].is_none() {
            params[i] = kw_num(&l, ks);
        }
    }
    // "with 9 dof"
    if matches!(kind, ChiSq | StudentT) && params[0].is_none() {
        if let Some(c) = re!(r"([\d.]+)\s*(?:dof|df|degrees of freedom)").captures(&l) {
            params[0] = ex(&c[1]);
        }
    }
    if kind == Benford && params[0].is_none() {
        params[0] = Some(r(10));
    }
    let params = params.into_iter().enumerate().map(|(i, p)| p.unwrap_or_else(|| sym(names[i]))).collect();
    Some(Dist { kind, params })
}

/// e^(num/den) for e a product of positive parameters and constants.
fn pos_pow(e: &Expr, num: i64, den: i64) -> Expr {
    let k = div(r(num), r(den));
    match e {
        Mul(v) => mul(v.iter().map(|f| pos_pow(f, num, den)).collect()),
        Pow(b, ex_) if matches!(**b, Sym(_) | Const(_)) => pow((**b).clone(), mul(vec![(**ex_).clone(), k])),
        _ => pow(e.clone(), k),
    }
}

fn simp(e: &Expr) -> Expr {
    let a = s::simplify(e);
    let b = s::simplify(&s::expand(&a));
    let c = s::factor(&b);
    [a, b, c].into_iter().min_by_key(|e| ptext(e).len()).unwrap()
}

fn show_val(e: &Expr) -> String {
    let e = simp(e);
    let t = ptext(&e);
    match crate::eval::const_c64(&e) {
        Some(z) if e.free_symbols().is_empty() && z.im.abs() < 1e-12 && (!e.is_numeric() || e.as_i64().is_none()) && !t.contains('.') => format!("{t} ≈ {}", num6(z.re)),
        _ => t,
    }
}

// ---------------------------------------------------------------------------
// moments and properties of distributions
// ---------------------------------------------------------------------------

fn ordinal(s: &str) -> Option<usize> {
    let w = s.trim_end_matches(|c: char| c.is_alphabetic() && !c.is_ascii_digit());
    if let Ok(n) = w.parse() {
        return Some(n);
    }
    Some(match s {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        "sixth" => 6,
        _ => return None,
    })
}

fn dist_properties(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:the )?(\w+) (raw moment|moment|central moment|cumulant)s?,?(?: of(?: the)?)? (.+)$").captures(l) {
        let k = ordinal(&m[1])?;
        if k == 0 || k > 10 {
            return None;
        }
        let d = parse_dist(&m[3])?;
        let (title, v) = match &m[2] {
            "central moment" => ("Central moment", d.central(k)?),
            "cumulant" => ("Cumulant", d.cumulant(k)?),
            _ => ("Raw moment", d.raw(k)?),
        };
        let sym_ = match &m[2] { "central moment" => format!("E[(X − μ)^{k}]"), "cumulant" => format!("κ_{k}"), _ => format!("E[X^{k}]") };
        return Some(Answer::new("moment", vec![input(format!("{} {} of the {}", &m[1], &m[2], d.name())), primary(pod("Result", title, text(format!("{sym_} = {}", show_val(&v)))))]));
    }
    if let Some(m) = re!(r"^(?:the )?(mean|expected value|expectation|variance|standard deviation|skewness|kurtosis|excess kurtosis|survival function|cdf|pdf|pmf|characteristic function)(?: of)?(?: the| a)? (.+)$").captures(l) {
        let d = parse_dist(&m[2])?;
        let what = &m[1];
        let v: Expr = match what {
            "mean" | "expected value" | "expectation" => d.raw(1)?,
            "variance" => d.central(2)?,
            "standard deviation" => sqrt(d.central(2)?),
            "skewness" => {
                let (k3, k2) = (d.cumulant(3)?, d.cumulant(2)?);
                div(k3, pos_pow(&s::simplify(&k2), 3, 2))
            }
            "kurtosis" | "excess kurtosis" => {
                let (k4, k2) = (d.cumulant(4)?, d.cumulant(2)?);
                let ex_ = div(k4, square(k2));
                if what == "kurtosis" { add(vec![ex_, r(3)]) } else { ex_ }
            }
            "survival function" => return survival(&d),
            _ => return None,
        };
        return Some(Answer::new("distribution property", vec![input(format!("{what} of the {}", d.name())), primary(pod("Result", &what[..1].to_uppercase().to_string().chars().chain(what[1..].chars()).collect::<String>(), text(show_val(&v))))]));
    }
    // a distribution with numeric parameters, maybe followed by a property:
    // "moyal distribution mu=2.3 sigma=12 skewness"
    if let Some(m) = re!(r"^(?:the |a )?(.+? (?:distribution|dist)\b.*?=.*?)(?:,? (mean|variance|standard deviation|skewness|kurtosis|excess kurtosis))?$").captures(l) {
        if let Some(p) = m.get(2) {
            return dist_properties(&format!("{} of {}", p.as_str(), &m[1]));
        }
        if let Some(d) = parse_dist(&m[1]).filter(|d| d.params.iter().all(|p| p.free_symbols().is_empty())) {
            let mut rows = Vec::new();
            for (name, v) in [
                ("mean", d.raw(1)),
                ("variance", d.central(2)),
                ("standard deviation", d.central(2).map(sqrt)),
                ("skewness", d.cumulant(3).zip(d.cumulant(2)).map(|(k3, k2)| div(k3, pos_pow(&s::simplify(&k2), 3, 2)))),
                ("excess kurtosis", d.cumulant(4).zip(d.cumulant(2)).map(|(k4, k2)| div(k4, square(k2)))),
            ] {
                let shown = v.map(|e| show_val(&e)).unwrap_or_else(|| "undefined".into());
                rows.push(vec![text(name), text(shown)]);
            }
            let (density, support) = formula(d.kind);
            let params: Vec<String> = d.kind.info().1.iter().zip(&d.params).map(|(n, p)| format!("{n} = {}", ptext(p))).collect();
            return Some(Answer::new("distribution", vec![
                input(format!("{} | {}", d.kind.info().0, params.join(", "))),
                primary(pod("Result", "Properties", table(None, rows))),
                pod("Formula", if d.discrete() { "Probability mass function" } else { "Probability density function" }, text(format!("{density} for {support}"))),
            ]));
        }
    }
    // a bare distribution name: formulas
    let m = re!(r"^(?:the )?([a-z' -]+?) (?:distribution|dist)$").captures(l)?;
    let d = parse_dist(&format!("{} distribution", &m[1]))?;
    if !d.params.iter().all(|p| !p.free_symbols().is_empty()) {
        return None;
    }
    let mean = d.raw(1).map(|e| show_val(&e)).unwrap_or_else(|| "undefined".into());
    let var = d.central(2).map(|e| show_val(&e)).unwrap_or_else(|| "undefined".into());
    let (density, support) = formula(d.kind);
    let rows = vec![
        vec![text("parameters"), text(d.kind.info().1.join(", "))],
        vec![text(if d.discrete() { "probability mass function" } else { "probability density function" }), text(density)],
        vec![text("support"), text(support)],
        vec![text("mean"), text(mean.clone())],
        vec![text("variance"), text(var)],
    ];
    Some(Answer::new("distribution", vec![input(d.kind.info().0.to_string()), primary(pod("Result", if d.discrete() { "Probability mass function" } else { "Probability density function" }, text(density))), pod("Properties", "Properties", table(None, rows))]))
}

fn formula(k: Kind) -> (&'static str, &'static str) {
    use Kind::*;
    match k {
        Normal => ("e^(−(x − μ)²/(2σ²))/(σ √(2π))", "x ∈ ℝ"),
        Exponential => ("λ e^(−λx)", "x ≥ 0"),
        Gamma => ("x^(α−1) e^(−x/β)/(Γ(α) β^α)", "x > 0"),
        Beta => ("x^(α−1) (1 − x)^(β−1)/B(α, β)", "0 < x < 1"),
        Uniform => ("1/(b − a)", "a ≤ x ≤ b"),
        ChiSq => ("x^(ν/2−1) e^(−x/2)/(2^(ν/2) Γ(ν/2))", "x > 0"),
        StudentT => ("Γ((ν+1)/2)/(√(νπ) Γ(ν/2)) (1 + x²/ν)^(−(ν+1)/2)", "x ∈ ℝ"),
        F => ("√((n x)^n m^m/(n x + m)^(n+m))/(x B(n/2, m/2))", "x > 0"),
        Rayleigh => ("(x/σ²) e^(−x²/(2σ²))", "x ≥ 0"),
        LogNormal => ("e^(−(log x − μ)²/(2σ²))/(x σ √(2π))", "x > 0"),
        Weibull => ("(α/β) (x/β)^(α−1) e^(−(x/β)^α)", "x ≥ 0"),
        Laplace => ("e^(−|x − μ|/β)/(2β)", "x ∈ ℝ"),
        Cauchy => ("1/(π b (1 + ((x − a)/b)²))", "x ∈ ℝ"),
        Logistic => ("e^(−(x−μ)/β)/(β (1 + e^(−(x−μ)/β))²)", "x ∈ ℝ"),
        Moyal => ("e^(−((x−μ)/σ + e^(−(x−μ)/σ))/2)/(σ √(2π))", "x ∈ ℝ"),
        Poisson => ("λ^k e^(−λ)/k!", "k = 0, 1, 2, …"),
        Binomial => ("C(n, k) p^k (1 − p)^(n−k)", "k = 0, 1, …, n"),
        Bernoulli => ("p for k = 1, 1 − p for k = 0", "k ∈ {0, 1}"),
        Geometric => ("p (1 − p)^k (failures before the first success)", "k = 0, 1, 2, …"),
        Pascal => ("C(k − 1, n − 1) p^n (1 − p)^(k−n) (trials until the n-th success)", "k = n, n + 1, …"),
        Zipf => ("k^(−(ρ+1))/ζ(ρ + 1)", "k = 1, 2, …"),
        Benford => ("log_b(1 + 1/d)", "d = 1, …, b − 1"),
    }
}

fn survival(d: &Dist) -> Option<Answer> {
    use Kind::*;
    let f = match d.kind {
        Pascal => "S(x) = P(X > x) = I_(1−p)(⌊x⌋ − n + 1, n) for x ≥ n (regularized incomplete beta); 1 for x < n",
        Normal => "S(x) = erfc((x − μ)/(σ√2))/2",
        Exponential => "S(x) = e^(−λx) for x ≥ 0",
        Weibull => "S(x) = e^(−(x/β)^α) for x ≥ 0",
        Rayleigh => "S(x) = e^(−x²/(2σ²)) for x ≥ 0",
        Geometric => "S(x) = (1 − p)^(⌊x⌋ + 1) for x ≥ 0",
        Poisson => "S(x) = 1 − Γ(⌊x⌋ + 1, λ)/⌊x⌋! = P(⌊x⌋ + 1, λ) (regularized gamma)",
        Binomial => "S(x) = I_p(⌊x⌋ + 1, n − ⌊x⌋)",
        Uniform => "S(x) = (b − x)/(b − a) for a ≤ x ≤ b",
        Logistic => "S(x) = 1/(1 + e^((x−μ)/β))",
        _ => return None,
    };
    Some(Answer::new("survival function", vec![input(format!("survival function of the {}", d.name())), primary(pod("Result", "Survival function", text(f)))]))
}

// ---------------------------------------------------------------------------
// random variables: expectations and probabilities
// ---------------------------------------------------------------------------

fn split_rv(l: &str) -> Option<(String, String, String)> {
    // returns (variable, distribution text, rest)
    if let Some(m) = re!(r"^(.*?)\b([a-z])\s*~\s*([a-z](?:[^,(]|\([^()]*\))*)\s*,?\s*(.*)$").captures(l) {
        let pre = re!(r"(?:,|\b(?:for|where|with|if|when))\s*$").replace(m[1].trim(), "").to_string();
        let rest = format!("{} {}", pre, &m[4]);
        return Some((m[2].to_string(), m[3].to_string(), rest.trim().trim_matches(',').trim().to_string()));
    }
    let pats = [
        r"^(.*?)(?:,| for| where| when| with| if)\s+([a-z]) (?:is |having |has |being |follows |distributed as |is distributed as )?(?:an? |the )?((?:standard normal|[a-z\-]+(?:ly distributed)?)(?: distribution| distributed| dist| random variable| variable)?(?: (?:with|and|,) [^,]*?)*?(?:\([^()]*\))?)\s*(?:,\s*(.*))?$",
        r"^([a-z]) (?:is |having |has )?(?:an? )?((?:standard normal|[a-z\-]+)(?: distribution)?(?: with [^,]*)?),\s*(.*)$",
    ];
    if let Some(m) = Regex::new(pats[0]).ok()?.captures(l) {
        let rest = format!("{} {}", &m[1], m.get(4).map(|x| x.as_str()).unwrap_or(""));
        return Some((m[2].to_string(), m[3].to_string(), rest.trim().to_string()));
    }
    if let Some(m) = Regex::new(pats[1]).ok()?.captures(l) {
        return Some((m[1].to_string(), m[2].to_string(), m[3].to_string()));
    }
    None
}

fn event(cond: &str, v: &str) -> Option<(Expr, String, Expr)> {
    let m = re!(r"^(.+?)\s*(<=|>=|≤|≥|<|>|=)\s*(.+)$").captures(cond.trim())?;
    let a = ex(&m[1].replace(v, "x"))?;
    let b = ex(&m[3].replace(v, "x"))?;
    Some((a, m[2].to_string(), b))
}

fn holds(ev: &(Expr, String, Expr), x: f64) -> bool {
    let f = |e: &Expr| crate::eval::eval_f64(e, &[("x", x)]).unwrap_or(f64::NAN);
    let (a, b) = (f(&ev.0), f(&ev.2));
    match ev.1.as_str() {
        "<" => a < b,
        ">" => a > b,
        "<=" | "≤" => a <= b,
        ">=" | "≥" => a >= b,
        _ => (a - b).abs() < 1e-12,
    }
}

fn prob_of(d: &Dist, evs: &[(Expr, String, Expr)]) -> Option<f64> {
    if d.discrete() {
        return d.expect(&|x| if evs.iter().all(|e| holds(e, x)) { 1.0 } else { 0.0 });
    }
    // continuous: integrate the density over the event, split at boundary points
    let (a, b) = d.range();
    let (lo, hi) = {
        let mean = d.raw(1).and_then(|e| e.to_f64_const()).unwrap_or(0.0);
        let sd = d.central(2).and_then(|e| e.to_f64_const()).map(|v| v.sqrt()).unwrap_or(10.0);
        let sd = if sd.is_finite() { sd } else { 50.0 };
        (a.max(mean - 60.0 * sd), b.min(mean + 60.0 * sd))
    };
    let mut breaks = Vec::new();
    for e in evs {
        let h = |x: f64| crate::eval::eval_f64(&e.0, &[("x", x)]).unwrap_or(f64::NAN) - crate::eval::eval_f64(&e.2, &[("x", x)]).unwrap_or(f64::NAN);
        breaks.extend(crate::numeric::find_roots(&h, lo, hi, 4000));
    }
    breaks.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let mut pts = vec![a.max(lo)];
    pts.extend(breaks.into_iter().filter(|x| *x > a && *x < b));
    pts.push(b.min(hi));
    let mut total = 0.0;
    for w in pts.windows(2) {
        let mid = if w[0].is_finite() && w[1].is_finite() { 0.5 * (w[0] + w[1]) } else if w[0].is_finite() { w[0] + 1.0 } else { w[1] - 1.0 };
        if evs.iter().all(|e| holds(e, mid)) {
            // integrate on the original (possibly infinite) range ends
            let (x0, x1) = (if w[0] <= lo && a == f64::NEG_INFINITY { a } else { w[0] }, if w[1] >= hi && b == f64::INFINITY { b } else { w[1] });
            total += crate::numeric::quad(&|x| d.pdf(x), x0, x1, 1e-11).value;
        }
    }
    Some(total)
}

fn random_variables(l: &str) -> Option<Answer> {
    let l = l.replace("ev[", "e[").replace("expected value of ", "e[").replace("expectation of ", "e[").replace("the expected value of ", "e[");
    let (v, dtext, rest) = split_rv(&l)?;
    let d = parse_dist(&dtext)?;
    let rest = rest.trim().trim_start_matches(',').trim().to_string();
    // expectation
    let em = re!(r"^(?:find |what is |compute )?(?:e|the mean of|mean of)\s*\[?\s*(.+?)\s*\]?$").captures(&rest);
    if let Some(m) = em.filter(|_| rest.starts_with("e[") || rest.starts_with("e ") || rest.contains("mean")) {
        let g_text = m[1].trim().trim_end_matches(']').to_string();
        let g = ex(&g_text.replace(&v, "x"))?;
        let x = sym("x");
        if g.free_symbols().iter().any(|n| &**n != "x") {
            return None;
        }
        // polynomial: exact from moments
        let poly = crate::poly::coeffs(&s::expand(&g), &x).filter(|cs| cs.iter().all(|c| !c.free_symbols().iter().any(|n| &**n == "x")));
        let val = if let Some(cs) = poly {
            let terms: Option<Vec<Expr>> = cs.iter().enumerate().map(|(k, c)| Some(mul(vec![c.clone(), d.raw(k)?]))).collect();
            show_val(&add(terms?))
        } else {
            let gg = g.clone();
            let h = move |t: f64| crate::eval::eval_f64(&gg, &[("x", t)]).unwrap_or(f64::NAN);
            let num = d.expect(&h)?;
            match crate::engine::math::identify_constant(num) {
                Some(c) => format!("{} ≈ {}", ptext(&c), num6(num)),
                None => format!("≈ {}", num6(num)),
            }
        };
        return Some(Answer::new("expectation", vec![input(format!("E[{}] for {v} ~ {}", ptext(&g).replace('x', &v), d.name())), primary(pod("Result", "Expected value", text(val)))]));
    }
    // probability, possibly conditional
    let pm = re!(r"^(?:find |what is )?(?:the )?(?:prob(?:ability)?|p)\s*\(?\s*(?:that\s+)?(.+?)\)?$").captures(&rest)?;
    let body = pm[1].to_string();
    let (a_text, b_text) = match re!(r"^(.+?)\s+(?:given(?: that)?|\|)\s+(.+)$").captures(&body) {
        Some(c) => (c[1].to_string(), Some(c[2].to_string())),
        None => (body.clone(), None),
    };
    let ea = event(&a_text, &v)?;
    let eb = match &b_text {
        Some(t) => Some(event(t, &v)?),
        None => None,
    };
    if !d.numeric() {
        return None;
    }
    let p_ab = prob_of(&d, &[vec![ea.clone()], eb.clone().into_iter().collect()].concat())?;
    let (value, label) = match &eb {
        Some(e) => {
            let pb = prob_of(&d, &[e.clone()])?;
            (p_ab / pb, format!("P({} | {})", a_text.trim(), b_text.as_deref().unwrap_or("").trim()))
        }
        None => (p_ab, format!("P({})", a_text.trim())),
    };
    let mut pods = vec![input(format!("{label} for {v} ~ {}", d.name())), primary(pod("Result", "Probability", text(format!("{label} ≈ {}", num6(value)))))];
    if eb.is_some() {
        pods.push(pod("Formula", "Formula", text("P(A | B) = P(A ∩ B)/P(B)")));
    }
    Some(Answer::new("probability", pods))
}

// ---------------------------------------------------------------------------
// birthday problems, Bernoulli trials, runs
// ---------------------------------------------------------------------------

fn birthday(l: &str) -> Option<Answer> {
    if !re!(r"birthday|born (?:on )?(?:the )?same|born in (?:the )?same").is_match(l) {
        return None;
    }
    let n: u64 = re!(r"(\d+) (?:people|persons|students|friends)").captures(l)?[1].parse().ok()?;
    let (d, unit) = if let Some(c) = re!(r"(\d+) (?:possible )?(?:birthdays|days)").captures(l) {
        (c[1].parse::<u64>().ok()?, "possible birthdays")
    } else if l.contains("day of the week") || l.contains("weekday") {
        (7, "days of the week")
    } else if l.contains("month") {
        (12, "months")
    } else {
        (365, "days of the year")
    };
    if n < 2 || d < 1 {
        return None;
    }
    let all_distinct: f64 = (0..n).map(|k| (d as f64 - k as f64).max(0.0) / d as f64).product();
    let share = 1.0 - all_distinct;
    let all_same = (d as f64).powf(1.0 - n as f64);
    let rows = vec![
        vec![text("P(at least two share)"), text(num6(share))],
        vec![text("P(all different)"), text(num6(all_distinct))],
        vec![text("P(all the same)"), text(format!("1/{}^{} ≈ {}", d, n - 1, num6(all_same)))],
    ];
    let mut need = 1u64;
    let mut prod = 1.0;
    while need <= d + 1 {
        prod *= (d as f64 - (need - 1) as f64) / d as f64;
        if 1.0 - prod >= 0.5 {
            break;
        }
        need += 1;
    }
    Some(Answer::new("birthday problem", vec![
        input(format!("birthday problem: {n} people, {d} {unit}")),
        primary(pod("Result", "Probability that at least two share", text(num6(share)))),
        pod("Details", "Details", table(None, rows)),
        pod("Formula", "Formula", text(format!("1 − {d}!/(({d} − {n})! {d}^{n})"))),
        pod("Half", "People needed for a 50% chance", text(need.to_string())),
    ]))
}

fn frac(num: &BigInt, den: &BigInt) -> String {
    let g = num_integer::Integer::gcd(num, den);
    let (a, b) = (num / &g, den / &g);
    let v = a.to_f64().unwrap_or(0.0) / b.to_f64().unwrap_or(1.0);
    if b == BigInt::from(1) { a.to_string() } else { format!("{a}/{b} ≈ {}", num6(v)) }
}

fn binom_pmf(n: u64, k: u64, p: f64) -> f64 {
    if k > n {
        return 0.0;
    }
    (ln_gamma(n as f64 + 1.0) - ln_gamma(k as f64 + 1.0) - ln_gamma((n - k) as f64 + 1.0) + k as f64 * p.ln() + (n - k) as f64 * (1.0 - p).ln()).exp()
}

fn parse_p(l: &str) -> Option<f64> {
    let c = re!(r"(?:\bp\s*=\s*|probability (?:of success )?(?:of |=|is )?\s*|with p )(\d*\.?\d+(?:/\d+)?)(?:$|[^\w/.])").captures(l)?;
    let e = ex(&c[1])?;
    e.to_f64_const().filter(|v| (0.0..=1.0).contains(v))
}

/// Probability that n Bernoulli(p) trials contain a run of at least k successes.
fn run_prob(n: usize, k: usize, p: f64) -> f64 {
    // state = current run length (< k); absorbing when it reaches k
    let mut st = vec![0.0; k];
    st[0] = 1.0;
    let mut hit = 0.0;
    for _ in 0..n {
        let mut nx = vec![0.0; k];
        for (j, &w) in st.iter().enumerate() {
            nx[0] += w * (1.0 - p);
            if j + 1 == k {
                hit += w * p;
            } else {
                nx[j + 1] += w * p;
            }
        }
        st = nx;
    }
    hit
}

fn run_count_half(n: usize, k: usize) -> BigInt {
    // number of length-n binary strings with a run of ≥ k ones
    let mut st = vec![BigInt::from(0); k];
    st[0] = BigInt::from(1);
    let mut hit = BigInt::from(0);
    for _ in 0..n {
        let mut nx = vec![BigInt::from(0); k];
        hit *= 2;
        for (j, w) in st.iter().enumerate() {
            nx[0] += w;
            if j + 1 == k {
                hit += w;
            } else {
                nx[j + 1] += w;
            }
        }
        st = nx;
    }
    hit
}

fn trials(l: &str) -> Option<Answer> {
    let coin = re!(r"heads?|tails?|toss|flip|coin").is_match(l);
    let p = parse_p(l).or(if coin { Some(0.5) } else { None });
    let half = p.map(|v| v == 0.5).unwrap_or(false);
    // runs and streaks
    if let Some(m) = re!(r"(?:streak|run) of (\d+) (?:successes|heads|wins)(?: or more)? in (\d+) (?:trials|flips|tosses|games)").captures(l) {
        let (k, n): (usize, usize) = (m[1].parse().ok()?, m[2].parse().ok()?);
        let p = p.unwrap_or(0.5);
        if k == 0 || k > n || n > 5000 {
            return None;
        }
        let v = run_prob(n, k, p);
        let main = if p == 0.5 && n <= 400 { frac(&run_count_half(n, k), &(BigInt::from(1) << n)) } else { num6(v) };
        return Some(Answer::new("runs", vec![input(format!("probability of a run of at least {k} successes in {n} trials, p = {}", fmt(p))), primary(pod("Result", "Probability", text(main))), pod("Method", "Method", text("Markov chain on the current run length"))]));
    }
    // k-th success on the n-th trial
    if let Some(m) = re!(r"(?:observe |probability (?:of )?(?:the )?|getting (?:the )?)?(\d+)(?:st|nd|rd|th) (?:success|head|tail)s? (?:on|at) (?:the )?(\d+)(?:st|nd|rd|th) (?:trial|flip|toss)").captures(l) {
        let (k, n): (u64, u64) = (m[1].parse().ok()?, m[2].parse().ok()?);
        if k == 0 || k > n {
            return None;
        }
        let c = crate::numtheory::binomial(&BigInt::from(n - 1), k - 1);
        let mut pods = vec![input(format!("probability that the {k}th success occurs on trial {n}"))];
        match p {
            Some(pv) if half => {
                pods.push(primary(pod("Result", "Probability", text(frac(&c, &(BigInt::from(1) << n))))));
                let _ = pv;
            }
            Some(pv) => pods.push(primary(pod("Result", "Probability", text(num6((ln_gamma(n as f64) - ln_gamma(k as f64) - ln_gamma((n - k + 1) as f64) + k as f64 * pv.ln() + (n - k) as f64 * (1.0 - pv).ln()).exp()))))),
            None => {
                pods.push(primary(pod("Result", "Probability", text(format!("C({}, {}) p^{k} (1 − p)^{} = {c} p^{k} (1 − p)^{}", n - 1, k - 1, n - k, n - k)))));
                pods.push(pod("Max", "Most likely p", text(format!("p = {k}/{n} ≈ {}", num6(k as f64 / n as f64)))));
                let at_half = frac(&c, &(BigInt::from(1) << n));
                pods.push(pod("Half", "For p = 1/2", text(at_half)));
            }
        }
        pods.push(pod("Formula", "Negative binomial formula", text("P = C(n − 1, k − 1) p^k (1 − p)^(n−k)")));
        return Some(Answer::new("negative binomial", pods));
    }
    // waiting time for the k-th success
    if let Some(m) = re!(r"(?:number of |expected number of )?(?:trials|flips|tosses) (?:until|before|to get) (?:the )?(\d+)(?:st|nd|rd|th) (?:success|head)").captures(l) {
        let k: f64 = m[1].parse().ok()?;
        let pv = p.unwrap_or(0.5);
        let mean = k / pv;
        let sd = (k * (1.0 - pv)).sqrt() / pv;
        let rows = vec![vec![text("mean"), text(fmt(mean))], vec![text("standard deviation"), text(num6(sd))], vec![text("mode"), text(fmt(((k - 1.0) / pv).floor() + 1.0))]];
        return Some(Answer::new("waiting time", vec![input(format!("trials until the {}th success, p = {}", fmt(k), fmt(pv))), primary(pod("Result", "Expected number of trials", text(fmt(mean)))), pod("Properties", "Pascal distribution properties", table(None, rows))]));
    }
    // exact head/tail counts "12 heads 8 tails"
    if let Some(m) = re!(r"(\d+) heads?,? (?:and )?(\d+) tails?").captures(l) {
        let (h, t): (u64, u64) = (m[1].parse().ok()?, m[2].parse().ok()?);
        let n = h + t;
        let c = crate::numtheory::binomial(&BigInt::from(n), h);
        return Some(Answer::new("coin flips", vec![input(format!("probability of exactly {h} heads and {t} tails in {n} fair tosses")), primary(pod("Result", "Probability", text(frac(&c, &(BigInt::from(1) << n))))), pod("Formula", "Formula", text(format!("C({n}, {h})/2^{n}")))]));
    }
    // counts: exactly / at least / fewer than k successes in n trials
    if let Some(m) = re!(r"(exactly |at least |at most |more than |fewer than |less than |no more than |)(\d+) (?:successes|success|heads|head|tails|wins)(?: out)? (?:in|of) (\d+) (?:trials|tosses|flips|coin flips|games|attempts)").captures(l) {
        let (k, n): (u64, u64) = (m[2].parse().ok()?, m[3].parse().ok()?);
        let pv = p?;
        if k > n || n > 100000 {
            return None;
        }
        let rel = m[1].trim();
        let ks: Vec<u64> = (0..=n).filter(|&j| match rel {
            "at least" => j >= k,
            "at most" | "no more than" => j <= k,
            "more than" => j > k,
            "fewer than" | "less than" => j < k,
            _ => j == k,
        }).collect();
        let main = if half && n <= 2000 {
            let c: BigInt = ks.iter().map(|&j| crate::numtheory::binomial(&BigInt::from(n), j)).sum();
            frac(&c, &(BigInt::from(1) << n))
        } else {
            num6(ks.iter().map(|&j| binom_pmf(n, j, pv)).sum())
        };
        let label = format!("P({}X {} {k})", "", match rel { "at least" => "≥", "at most" | "no more than" => "≤", "more than" => ">", "fewer than" | "less than" => "<", _ => "=" });
        return Some(Answer::new("binomial probability", vec![input(format!("{label} for X ~ binomial(n = {n}, p = {})", fmt(pv))), primary(pod("Result", "Probability", text(main))), pod("Formula", "Formula", text("P(X = k) = C(n, k) p^k (1 − p)^(n−k)"))]));
    }
    // a description of n trials
    if let Some(m) = re!(r"^(\d+) (?:bernoulli )?trials(?:,? (?:with )?p\s*=\s*([\d.]+))?$").captures(l) {
        let n: u64 = m[1].parse().ok()?;
        if n == 0 || n > 60 {
            return None;
        }
        let pv: f64 = m.get(2).and_then(|x| x.as_str().parse().ok()).unwrap_or(0.5);
        let rows: Vec<Vec<Content>> = (0..=n).map(|k| vec![text(k.to_string()), text(num6(binom_pmf(n, k, pv)))]).collect();
        let summary = vec![vec![text("mean"), text(fmt(n as f64 * pv))], vec![text("standard deviation"), text(num6((n as f64 * pv * (1.0 - pv)).sqrt()))], vec![text("most likely number of successes"), text(fmt(((n + 1) as f64 * pv).floor().min(n as f64)))]];
        let vals: Vec<(String, f64)> = (0..=n).map(|k| (k.to_string(), binom_pmf(n, k, pv))).collect();
        return Some(Answer::new("Bernoulli trials", vec![input(format!("{n} Bernoulli trials with p = {}", fmt(pv))), primary(pod("Result", "Number of successes", table(None, summary))), pod("PMF", "Probability of k successes", table(Some(vec!["k", "P(k successes)"]), rows)), pod("Plot", "Distribution", plot(crate::plot::bars(vals)))]));
    }
    None
}

fn rules(l: &str) -> Option<Answer> {
    let (title, main, rows): (&str, &str, Vec<(&str, &str)>) = match l.trim_start_matches("the ") {
        "conditional probability formula" | "conditional probability" | "formula for conditional probability" => ("conditional probability", "P(A | B) = P(A ∩ B)/P(B), P(B) > 0", vec![("multiplication rule", "P(A ∩ B) = P(A | B) P(B)"), ("Bayes' theorem", "P(A | B) = P(B | A) P(A)/P(B)"), ("independence", "A, B independent ⇔ P(A | B) = P(A)")]),
        "probability of a complement" | "complement rule" | "probability of the complement" => ("complement rule", "P(Aᶜ) = 1 − P(A)", vec![("reason", "A and Aᶜ are disjoint and together form the whole sample space")]),
        "probability of the union of three events" | "union of three events" | "inclusion-exclusion for three events" => ("union of three events", "P(A ∪ B ∪ C) = P(A) + P(B) + P(C) − P(A ∩ B) − P(A ∩ C) − P(B ∩ C) + P(A ∩ B ∩ C)", vec![("principle", "inclusion–exclusion")]),
        "bayes theorem" | "bayes' theorem" | "bayes rule" => ("Bayes' theorem", "P(A | B) = P(B | A) P(A)/P(B)", vec![("total probability", "P(B) = Σ P(B | A_i) P(A_i)")]),
        "law of total probability" => ("law of total probability", "P(B) = Σ_i P(B | A_i) P(A_i) for a partition {A_i}", vec![]),
        "z-test for population mean" | "z test for population mean" | "z-test" => ("z-test for a population mean", "z = (x̄ − μ₀)/(σ/√n)", vec![("assumptions", "known population σ, normal population or large n"), ("two-sided p-value", "2 (1 − Φ(|z|))"), ("reject H₀ at level α", "|z| > z_(α/2) (1.96 for α = 0.05)")]),
        "sample size for binomial parameter" | "sample size for a proportion" => ("sample size for a binomial proportion", "n = z² p (1 − p)/E²", vec![("conservative choice", "p = 1/2 gives n = z²/(4E²)"), ("95% confidence, ±3%", "n = 1.96²/(4·0.03²) ≈ 1068")]),
        _ => return None,
    };
    let mut pods = vec![input(title.to_string()), primary(pod("Result", "Formula", text(main)))];
    if !rows.is_empty() {
        pods.push(pod("Details", "Details", table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())));
    }
    Some(Answer::new(title, pods))
}

// ---------------------------------------------------------------------------
// descriptive statistics
// ---------------------------------------------------------------------------

fn data(t: &str) -> Option<Vec<f64>> {
    let t = t.trim().trim_start_matches(['{', '(']).trim_end_matches(['}', ')']);
    let v: Option<Vec<f64>> = t.split(',').map(|x| ex(x.trim()).and_then(|e| e.to_f64_const())).collect();
    v.filter(|v| v.len() >= 2)
}

fn quartiles(v: &[f64]) -> (f64, f64, f64) {
    // Tukey's hinges style: medians of the lower and upper halves
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = |x: &[f64]| {
        let n = x.len();
        if n % 2 == 1 { x[n / 2] } else { 0.5 * (x[n / 2 - 1] + x[n / 2]) }
    };
    let n = s.len();
    let (lo, hi) = if n % 2 == 0 { (&s[..n / 2], &s[n / 2..]) } else { (&s[..n / 2], &s[n / 2 + 1..]) };
    (med(lo), med(&s), med(hi))
}

fn descriptive(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?(kurtosis|excess kurtosis|skewness|geometric mean|harmonic mean|outliers?|what is the outlier for the data set|interquartile range|quartiles|mean absolute deviation)(?: of)? (.+)$").captures(l)?;
    let v = data(&m[2])?;
    let n = v.len() as f64;
    let mean = v.iter().sum::<f64>() / n;
    let mom = |k: i32| v.iter().map(|x| (x - mean).powi(k)).sum::<f64>() / n;
    let what = m[1].to_string();
    let (title, val, note): (&str, String, &str) = match what.as_str() {
        "kurtosis" => ("Kurtosis", num6(mom(4) / mom(2).powi(2)), "population kurtosis m₄/m₂² (a normal distribution has 3)"),
        "excess kurtosis" => ("Excess kurtosis", num6(mom(4) / mom(2).powi(2) - 3.0), "m₄/m₂² − 3"),
        "skewness" => ("Skewness", num6(mom(3) / mom(2).powf(1.5)), "population skewness m₃/m₂^(3/2)"),
        "geometric mean" => {
            if v.iter().any(|x| *x <= 0.0) {
                return None;
            }
            let g = (v.iter().map(|x| x.ln()).sum::<f64>() / n).exp();
            let prod: f64 = v.iter().product();
            let exact = if (g.round() - g).abs() < 1e-9 && v.iter().all(|x| x.fract() == 0.0) { fmt(g.round()) } else if v.iter().all(|x| x.fract() == 0.0) && prod < 1e15 { format!("{}^(1/{}) ≈ {}", fmt(prod), v.len(), num6(g)) } else { num6(g) };
            ("Geometric mean", exact, "(x₁ x₂ ⋯ x_n)^(1/n)")
        }
        "harmonic mean" => {
            if v.iter().any(|x| *x == 0.0) {
                return None;
            }
            let h = n / v.iter().map(|x| 1.0 / x).sum::<f64>();
            let exact = crate::engine::math::identify_constant(h).filter(|e| e.is_numeric()).map(|e| format!("{} ≈ {}", ptext(&e), num6(h))).unwrap_or_else(|| num6(h));
            ("Harmonic mean", exact, "n/(1/x₁ + ⋯ + 1/x_n)")
        }
        "interquartile range" | "quartiles" => {
            let (q1, q2, q3) = quartiles(&v);
            if what == "quartiles" { ("Quartiles", format!("Q1 = {}, Q2 = {}, Q3 = {}", fmt(q1), fmt(q2), fmt(q3)), "medians of the lower half, the data, and the upper half") } else { ("Interquartile range", fmt(q3 - q1), "Q3 − Q1") }
        }
        "mean absolute deviation" => ("Mean absolute deviation", num6(v.iter().map(|x| (x - mean).abs()).sum::<f64>() / n), "mean of |x_i − x̄|"),
        _ => {
            let (q1, _, q3) = quartiles(&v);
            let iqr = q3 - q1;
            let (lo, hi) = (q1 - 1.5 * iqr, q3 + 1.5 * iqr);
            let out: Vec<String> = v.iter().filter(|x| **x < lo || **x > hi).map(|x| fmt(*x)).collect();
            let main = if out.is_empty() { "none".to_string() } else { out.join(", ") };
            return Some(Answer::new("outliers", vec![input(format!("outliers of {{{}}}", v.iter().map(|x| fmt(*x)).collect::<Vec<_>>().join(", "))), primary(pod("Result", "Outliers", text(main))), pod("Method", "Method (1.5 × IQR rule)", text(format!("Q1 = {}, Q3 = {}, IQR = {}; fences {} and {}", fmt(q1), fmt(q3), fmt(iqr), fmt(lo), fmt(hi))))]));
        }
    };
    Some(Answer::new("statistic", vec![input(format!("{what} of {{{}}}", v.iter().map(|x| fmt(*x)).collect::<Vec<_>>().join(", "))), primary(pod("Result", title, text(val))), pod("Definition", "Definition", text(note))]))
}

/// Runs before the general distribution pods: moments and cumulants, and
/// properties of distributions whose parameters are left symbolic.
pub fn early(q: &Query) -> Option<Answer> {
    let raw = q.clean.trim().trim_end_matches(['?', '.']);
    let l = re!(r"\s+").replace_all(&raw.to_lowercase(), " ").to_string();
    if re!(r"\b(?:moment|cumulant)s?\b").is_match(&l) {
        return dist_properties(&l);
    }
    let m = re!(r"^(?:the )?(?:mean|expected value|expectation|variance|standard deviation|skewness|kurtosis|excess kurtosis)(?: of)?(?: the| a)? (.+)$").captures(&l)?;
    let d = parse_dist(&m[1])?;
    if d.params.iter().any(|p| p.free_symbols().is_empty()) || d.raw(1).is_none() {
        return None;
    }
    dist_properties(&l)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let raw = q.clean.trim().trim_end_matches(['?', '.']);
    let l = re!(r"\s+").replace_all(&raw.to_lowercase(), " ").to_string();
    rules(&l)
        .or_else(|| descriptive(&l))
        .or_else(|| birthday(&l))
        .or_else(|| trials(&l))
        .or_else(|| random_variables(&l))
        .or_else(|| dist_properties(&l))
}

#[cfg(test)]
mod tests {
    fn ans(s: &str) -> String {
        serde_json::to_string(&crate::engine::query(s).pods).unwrap_or_default()
    }

    #[test]
    fn probability() {
        assert!(ans("probability of 3rd head on 8th flip").contains("21/256"));
        assert!(ans("expectation of y^2+2y-1 for y having a gamma distribution").contains("α"));
        assert!(ans("prob X<16 for X~geometric with p=0.1").contains("0.81469"));
        assert!(ans("mean Zipf distribution, rho=2").contains("1.36843"));
        assert!(ans("birthday paradox, 4 people, 100 possible birthdays").contains("0.0589"));
    }
}
