//! Number sequences: "1, 4, 9, 16, 25, ...", "5, 14, 23, 32, 41",
//! "1, 1, 2, 3, 5, 8", "1, 2, 3, 2, 1, 2, 3, 2, 1" and series written out
//! term by term, "1/2 + 1/4 + 1/8 + 1/16 + ...".
//!
//! Rules tried, simplest first: polynomial (finite differences), geometric,
//! linear recurrence of order 2, periodic.  Every rule must reproduce all
//! the given terms and be confirmed by at least one term beyond those
//! needed to determine it.

use super::*;
use crate::expr::*;
use crate::parse;
use crate::simplify as s;
use num_traits::{One, Signed, Zero};

#[derive(Clone, Debug)]
pub enum Rule {
    /// Coefficients of Newton's forward form: a(n) = Σ d_k C(n-1, k).
    Poly(Vec<Rat>),
    Geometric(Rat, Rat),
    /// a(n) = p a(n-1) + q a(n-2)
    Recurrence(Rat, Rat, Rat, Rat),
    Periodic(Vec<Rat>),
}

fn binom_rat(n: i64, k: usize) -> Rat {
    let mut r = Rat::one();
    for j in 0..k as i64 {
        r = r * Rat::from_integer((n - j).into()) / Rat::from_integer((j + 1).into());
    }
    r
}

impl Rule {
    /// The n-th term, n starting at 1.
    pub fn term(&self, n: i64) -> Rat {
        match self {
            Rule::Poly(d) => d.iter().enumerate().fold(Rat::zero(), |acc, (k, c)| acc + c * binom_rat(n - 1, k)),
            Rule::Geometric(a, r) => {
                let mut v = a.clone();
                for _ in 1..n {
                    v *= r;
                }
                v
            }
            Rule::Recurrence(p, q, a1, a2) => {
                let (mut x, mut y) = (a1.clone(), a2.clone());
                if n == 1 {
                    return x;
                }
                for _ in 2..n {
                    let z = p * &y + q * &x;
                    x = y;
                    y = z;
                }
                y
            }
            Rule::Periodic(p) => p[((n - 1) as usize) % p.len()].clone(),
        }
    }

    /// A closed form in n, when there is a simple one.
    pub fn closed_form(&self) -> Option<Expr> {
        let n = sym("n");
        match self {
            Rule::Poly(d) => {
                let mut terms = Vec::new();
                for (k, c) in d.iter().enumerate() {
                    let mut f = num(c.clone());
                    for j in 0..k as i64 {
                        f = mul(vec![f, add(vec![n.clone(), int(-1 - j)])]);
                    }
                    let fact: i64 = (1..=k as i64).product();
                    terms.push(mul(vec![f, rat(1, fact)]));
                }
                Some(s::simplify(&s::expand(&add(terms))))
            }
            Rule::Geometric(a, r) => Some(s::simplify(&mul(vec![num(a.clone()), pow(num(r.clone()), add(vec![n, int(-1)]))]))),
            _ => None,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Rule::Poly(d) => match d.len() {
                1 => "constant sequence".into(),
                2 => format!("arithmetic sequence with common difference {}", d[1]),
                3 => "quadratic sequence (constant second differences)".into(),
                3.. => format!("polynomial sequence of degree {}", d.len() - 1),
                _ => String::new(),
            },
            Rule::Geometric(_, r) => format!("geometric sequence with common ratio {r}"),
            Rule::Recurrence(p, q, ..) => {
                let pt = if p.is_one() { "a(n-1)".to_string() } else { format!("{p} a(n-1)") };
                let qt = if q.is_one() { "a(n-2)".to_string() } else { format!("{} a(n-2)", q.abs()) };
                format!("linear recurrence a(n) = {pt} {} {qt}", if q.is_negative() { "-" } else { "+" })
            }
            Rule::Periodic(p) => format!("periodic sequence with period {}", p.len()),
        }
    }
}

/// Find the simplest rule reproducing `a`.
pub fn find_rule(a: &[Rat]) -> Option<Rule> {
    let n = a.len();
    if n < 3 {
        return None;
    }
    // polynomial by finite differences
    let mut rows = vec![a.to_vec()];
    while rows.last().unwrap().len() > 1 {
        let r = rows.last().unwrap();
        let next: Vec<Rat> = r.windows(2).map(|w| &w[1] - &w[0]).collect();
        let constant = next.iter().all(|x| *x == next[0]);
        rows.push(next);
        if constant && rows.last().unwrap().len() >= 2 {
            let deg = rows.len() - 1;
            // need a confirming term: n >= deg + 2, and a degree at most 4
            if n >= deg + 2 && deg <= 4 {
                return Some(Rule::Poly(rows.iter().map(|r| r[0].clone()).collect()));
            }
            break;
        }
    }
    if a.iter().all(|x| *x == a[0]) {
        return Some(Rule::Poly(vec![a[0].clone()]));
    }
    // geometric
    if a.iter().all(|x| !x.is_zero()) {
        let r = &a[1] / &a[0];
        if a.windows(2).all(|w| &w[1] / &w[0] == r) {
            return Some(Rule::Geometric(a[0].clone(), r));
        }
    }
    // order-2 linear recurrence, needs 5 terms
    if n >= 5 {
        let (a1, a2, a3, a4) = (&a[0], &a[1], &a[2], &a[3]);
        let det = a2 * a2 - a1 * a3;
        if !det.is_zero() {
            let p = (a3 * a2 - a4 * a1) / &det;
            let q = (a4 * a2 - a3 * a3) / &det;
            let rule = Rule::Recurrence(p, q, a1.clone(), a2.clone());
            if (1..=n as i64).all(|k| rule.term(k) == a[(k - 1) as usize]) {
                return Some(rule);
            }
        }
    }
    // periodic
    for p in 2..=n / 2 {
        if (p..n).all(|i| a[i] == a[i - p]) && n >= p + 2 {
            return Some(Rule::Periodic(a[..p].to_vec()));
        }
    }
    None
}

fn parse_terms(parts: &[&str]) -> Option<Vec<Rat>> {
    parts
        .iter()
        .map(|t| {
            let e = parse::parse(t.trim()).ok()?.expr;
            match s::simplify(&e) {
                Expr::Num(r) => Some(r),
                _ => None,
            }
        })
        .collect()
}

fn list(a: &[Rat]) -> String {
    a.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_start_matches("sequence ").trim_start_matches("the sequence ").trim_start_matches("next term ").trim_start_matches("next terms ").trim_start_matches("of ").trim();
    let body = l.trim_end_matches("...").trim_end_matches('…').trim().trim_end_matches(',').trim_end_matches('+').trim();
    let series = body.contains('+') && !body.contains(',');
    let parts: Vec<&str> = if series { body.split('+').collect() } else { body.split(',').collect() };
    let raw = q.raw.trim().trim_end_matches('?').trim();
    let open_ended = raw.ends_with("...") || raw.ends_with('…') || raw.ends_with('+') || l.ends_with('+');
    if parts.len() < 4 || (series && !open_ended) {
        return None;
    }
    if parts.iter().any(|p| p.trim().is_empty() || p.chars().any(|c| c.is_ascii_alphabetic())) {
        return None;
    }
    let a = parse_terms(&parts)?;
    let rule = find_rule(&a)?;
    let n = a.len() as i64;
    let next: Vec<Rat> = (n + 1..=n + 5).map(|k| rule.term(k)).collect();
    let mut pods = vec![pod("Input", "Input", text(format!("{}, …", list(&a))))];
    if series {
        // the sum of the series
        let term = rule.closed_form()?;
        let sum_q = format!("sum {} for n = 1 to infinity", print::text(&term));
        let inner = Query { raw: q.raw, clean: sum_q.clone(), lower: sum_q.to_lowercase(), opts: q.opts };
        let mut ans = super::commands::handle(&inner)?;
        ans.pods.insert(0, pod("Input", "Input interpretation", text(format!("{} + …  =  Σ_(n=1)^∞ {}", a.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" + "), print::text(&term)))));
        ans.pods.retain(|p| p.id != "Input" || p.title == "Input interpretation");
        return Some(ans);
    }
    pods.push(primary(pod("Next", "Next terms", text(format!("{}, …", list(&next))))));
    pods.push(pod("Pattern", "Pattern", text(rule.describe())));
    if let Some(cf) = rule.closed_form() {
        pods.push(pod("Formula", "Possible closed form", text(format!("a(n) = {}", print::text(&cf)))));
        if let Rule::Poly(d) = &rule {
            // partial sums: S(n) = Σ d_k C(n, k+1)
            let mut sd = vec![Rat::zero()];
            sd.extend(d.iter().cloned());
            if let Some(sf) = Rule::Poly(sd).closed_form() {
                let sf = s::simplify(&s::expand(&sf));
                pods.push(pod("Sum", "Sum of the first n terms", text(format!("{}", print::text(&sf)))));
            }
        }
    }
    if let Rule::Recurrence(..) = rule {
        pods.push(pod("Recurrence", "Recurrence", text(format!("{}, a(1) = {}, a(2) = {}", rule.describe().trim_start_matches("linear recurrence "), a[0], a[1]))));
    }
    Some(Answer::new(format!("sequence {}", list(&a)), pods))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(v: &[i64]) -> Vec<Rat> {
        v.iter().map(|x| Rat::from_integer((*x).into())).collect()
    }

    #[test]
    fn rules() {
        let sq = find_rule(&r(&[1, 4, 9, 16, 25])).unwrap();
        assert_eq!(sq.term(6), Rat::from_integer(36.into()));
        assert_eq!(print::text(&sq.closed_form().unwrap()), "n^2");
        let ar = find_rule(&r(&[5, 14, 23, 32, 41])).unwrap();
        assert_eq!(ar.term(6), Rat::from_integer(50.into()));
        let fib = find_rule(&r(&[1, 1, 2, 3, 5, 8])).unwrap();
        assert_eq!(fib.term(7), Rat::from_integer(13.into()));
        let per = find_rule(&r(&[1, 2, 3, 2, 1, 2, 3, 2, 1])).unwrap();
        assert_eq!(per.term(10), Rat::from_integer(2.into()));
        let geo = find_rule(&r(&[3, 6, 12, 24])).unwrap();
        assert_eq!(geo.term(5), Rat::from_integer(48.into()));
    }
}
