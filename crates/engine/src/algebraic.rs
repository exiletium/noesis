//! Exact minimal polynomials of algebraic numbers built from rationals with
//! +, −, ×, ÷ and rational powers (radicals). Each operation produces an
//! annihilating polynomial through a resultant, computed exactly by
//! evaluating at integer points and interpolating; the minimal polynomial is
//! the irreducible factor that vanishes at the number's numeric value.

use crate::expr::Expr::*;
use crate::expr::{Expr, Rat};
use crate::poly::QPoly;
use num_bigint::BigInt;
use num_traits::{One, Signed, ToPrimitive, Zero};

fn ri(n: i64) -> Rat {
    Rat::from_integer(BigInt::from(n))
}

/// Resultant of two univariate polynomials over Q (Euclidean algorithm).
fn res(a: &QPoly, b: &QPoly) -> Rat {
    if a.is_zero() || b.is_zero() {
        return Rat::zero();
    }
    let (m, n) = (a.degree(), b.degree());
    if n == 0 {
        return num_traits::pow(b.lc(), m);
    }
    if m == 0 {
        return num_traits::pow(a.lc(), n);
    }
    let r = a.rem(b);
    if r.is_zero() {
        return Rat::zero();
    }
    let sign = if (m * n) % 2 == 1 { -Rat::one() } else { Rat::one() };
    sign * num_traits::pow(b.lc(), m - r.degree()) * res(b, &r)
}

/// The polynomial of degree ≤ n through (xs[i], ys[i]) (Newton form).
fn interpolate(xs: &[Rat], ys: &[Rat]) -> QPoly {
    let n = xs.len();
    let mut coef = ys.to_vec();
    for j in 1..n {
        for i in (j..n).rev() {
            coef[i] = (&coef[i] - &coef[i - 1]) / (&xs[i] - &xs[i - j]);
        }
    }
    let mut p = QPoly::constant(coef[n - 1].clone());
    for i in (0..n - 1).rev() {
        p = p.mul(&QPoly::new(vec![-xs[i].clone(), Rat::one()])).add(&QPoly::constant(coef[i].clone()));
    }
    p
}

/// R(x) = Res_y(P(y), Q_x(y)) where `qx(x0)` builds Q at x = x0; R has degree ≤ deg.
fn resultant_in_x(p: &QPoly, qx: &dyn Fn(&Rat) -> QPoly, deg: usize) -> QPoly {
    let xs: Vec<Rat> = (0..=deg as i64).map(|k| ri(k - (deg as i64) / 2)).collect();
    let ys: Vec<Rat> = xs.iter().map(|x0| res(p, &qx(x0))).collect();
    interpolate(&xs, &ys)
}

fn sum_poly(p: &QPoly, q: &QPoly) -> QPoly {
    // Res_y(P(y), Q(x − y))
    let deg = p.degree() * q.degree();
    resultant_in_x(p, &|x0| q.compose(&QPoly::new(vec![x0.clone(), -Rat::one()])), deg)
}

fn prod_poly(p: &QPoly, q: &QPoly) -> QPoly {
    // Res_y(P(y), yⁿ Q(x/y)) with yⁿ Q(x/y) = Σ q_k x^k y^(n−k)
    let n = q.degree();
    let deg = p.degree() * n;
    resultant_in_x(p, &|x0| {
        let mut c = vec![Rat::zero(); n + 1];
        for k in 0..=n {
            c[n - k] = q.coeff(k) * num_traits::pow(x0.clone(), k);
        }
        QPoly::new(c)
    }, deg)
}

fn int_pow_poly(p: &QPoly, k: usize) -> QPoly {
    // Res_y(P(y), x − y^k)
    resultant_in_x(p, &|x0| {
        let mut c = vec![Rat::zero(); k + 1];
        c[0] = x0.clone();
        c[k] = -Rat::one();
        QPoly::new(c)
    }, p.degree())
}

fn reverse(p: &QPoly) -> QPoly {
    let mut c = p.0.clone();
    c.reverse();
    QPoly::new(c)
}

fn root_poly(p: &QPoly, q: usize) -> QPoly {
    // P(x^q)
    let mut c = vec![Rat::zero(); p.degree() * q + 1];
    for (k, a) in p.0.iter().enumerate() {
        c[k * q] = a.clone();
    }
    QPoly::new(c)
}

/// Keep the irreducible factor of `p` that has `value` as a root.
fn pick_factor(p: &QPoly, value: num_complex::Complex64) -> Option<QPoly> {
    let (_, fs) = crate::poly::factor(p);
    let mut best: Option<(f64, QPoly)> = None;
    for (f, _) in fs {
        let d = f.roots_c().iter().map(|r| (r - value).norm()).fold(f64::INFINITY, f64::min);
        if best.as_ref().map(|b| d < b.0).unwrap_or(true) {
            best = Some((d, f));
        }
    }
    let (d, f) = best?;
    (d < 1e-6 * (1.0 + value.norm())).then_some(f)
}

const MAX_DEG: usize = 16;

/// An annihilating polynomial (reduced to the minimal one at each step).
fn ann(e: &Expr) -> Option<QPoly> {
    let value = crate::eval::const_c64(e)?;
    let p = match e {
        Num(r) => return Some(QPoly::new(vec![-r.clone(), Rat::one()])),
        Add(v) => {
            let mut acc = ann(&v[0])?;
            for t in &v[1..] {
                let q = ann(t)?;
                if acc.degree() * q.degree() > MAX_DEG {
                    return None;
                }
                acc = sum_poly(&acc, &q);
            }
            acc
        }
        Mul(v) => {
            let mut acc = ann(&v[0])?;
            for t in &v[1..] {
                let q = ann(t)?;
                if acc.degree() * q.degree() > MAX_DEG {
                    return None;
                }
                acc = prod_poly(&acc, &q);
            }
            acc
        }
        Pow(b, k) => {
            let r = k.as_rat()?;
            let (num, den) = (r.numer().to_i64()?, r.denom().to_usize()?);
            if den == 0 || den > 8 || num.unsigned_abs() > 12 {
                return None;
            }
            let mut p = ann(b)?;
            if num < 0 {
                p = reverse(&p);
            }
            if num.unsigned_abs() > 1 {
                p = int_pow_poly(&p, num.unsigned_abs() as usize);
            }
            if den > 1 {
                if p.degree() * den > MAX_DEG {
                    return None;
                }
                p = root_poly(&p, den);
            }
            p
        }
        _ => return None,
    };
    pick_factor(&p, value)
}

/// The minimal polynomial over Q of a constant expression, as a primitive
/// integer polynomial with positive leading coefficient.
pub fn minimal_polynomial(e: &Expr) -> Option<QPoly> {
    let p = ann(e)?;
    let p = p.primitive();
    Some(if p.lc().is_negative() { p.neg() } else { p })
}

/// The monic minimal polynomial.
pub fn monic_minimal_polynomial(e: &Expr) -> Option<QPoly> {
    minimal_polynomial(e).map(|p| p.monic())
}

pub fn is_integral(monic: &QPoly) -> bool {
    monic.0.iter().all(|c| c.is_integer())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mp(s: &str) -> String {
        let e = crate::parse::parse(s).unwrap().expr;
        let p = minimal_polynomial(&crate::simplify::simplify(&e)).unwrap();
        crate::print::text(&p.to_expr(&crate::expr::sym("x")))
    }

    #[test]
    fn minimal_polynomials() {
        assert_eq!(mp("sqrt(2)+sqrt(3)"), "x^4 - 10 x^2 + 1");
        assert_eq!(mp("1/(1+sqrt(5))"), "4 x^2 + 2 x - 1");
        assert_eq!(mp("2^(1/3)+1"), "x^3 - 3 x^2 + 3 x - 3");
    }
}
