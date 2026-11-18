//! Tables of standard integrals involving a given function.
//!
//! "integrals containing cos(u)", "sample integrals besselj", "definite
//! integrals with Si(x)", "integrals with li(x)".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// keys (';'-separated) | display name | indefinite integrals | definite integrals
/// (formulas separated by '|')
const TABLES: &[(&str, &str, &str, &str)] = &[
    ("cos;cosine;cos(u);cos(x);cos u;cos x", "cos(u)",
     "∫cos(u) du = sin(u) + C|∫u cos(u) du = cos(u) + u sin(u) + C|∫cos²(u) du = u/2 + sin(2u)/4 + C|∫cos³(u) du = sin(u) − sin³(u)/3 + C|∫cos(a u) du = sin(a u)/a + C|∫e^(a u) cos(b u) du = e^(a u) (a cos(b u) + b sin(b u))/(a² + b²) + C|∫cos(u)/u du = Ci(u) + C|∫sec(u) du = log(sec(u) + tan(u)) + C",
     "∫₀^π cos²(u) du = π/2|∫₀^(π/2) cos(u) du = 1|∫₀^∞ cos(u)/(1 + u²) du = π/(2e)|∫₀^∞ cos(u²) du = √(π/8)|∫₀^(2π) cos(m u) cos(n u) du = π δₘₙ (m, n ≥ 1)|∫₀^∞ e^(−a u) cos(b u) du = a/(a² + b²)"),
    ("sin;sine;sin(u);sin(x)", "sin(u)",
     "∫sin(u) du = −cos(u) + C|∫u sin(u) du = sin(u) − u cos(u) + C|∫sin²(u) du = u/2 − sin(2u)/4 + C|∫sin³(u) du = cos³(u)/3 − cos(u) + C|∫e^(a u) sin(b u) du = e^(a u) (a sin(b u) − b cos(b u))/(a² + b²) + C|∫sin(u)/u du = Si(u) + C",
     "∫₀^π sin(u) du = 2|∫₀^∞ sin(u)/u du = π/2|∫₀^∞ sin²(u)/u² du = π/2|∫₀^∞ sin(u²) du = √(π/8)|∫₀^(2π) sin(m u) sin(n u) du = π δₘₙ (m, n ≥ 1)"),
    ("exp;exp(t);exp(x);e^x;e^t;exponential;exponential function", "exp(t)",
     "∫e^t dt = e^t + C|∫e^(a t) dt = e^(a t)/a + C|∫t e^(a t) dt = e^(a t) (a t − 1)/a² + C|∫t² e^t dt = e^t (t² − 2t + 2) + C|∫e^(−t²) dt = (√π/2) erf(t) + C|∫e^t/t dt = Ei(t) + C",
     "∫₀^∞ e^(−t) dt = 1|∫₀^∞ tⁿ e^(−t) dt = n! = Γ(n + 1)|∫₋∞^∞ e^(−t²) dt = √π|∫₀^∞ e^(−a t) sin(b t) dt = b/(a² + b²)|∫₀^∞ e^(−a t²) dt = √(π/a)/2|∫₀^1 e^t dt = e − 1"),
    ("besselj;bessel j;bessel function;bessel functions;besselj(n,x);j0;bessel j0", "Bessel J",
     "∫J₁(x) dx = −J₀(x) + C|∫x J₀(x) dx = x J₁(x) + C|∫x^ν J_(ν−1)(x) dx = x^ν J_ν(x) + C|∫x^(−ν) J_(ν+1)(x) dx = −x^(−ν) J_ν(x) + C|∫J₀(x) dx = x J₀(x) + (π x/2)(J₁(x) H₀(x) − J₀(x) H₁(x)) + C (H = Struve)",
     "∫₀^∞ J_ν(x) dx = 1 (ν > −1)|∫₀^∞ J_ν(x)/x dx = 1/ν (ν > 0)|∫₀^∞ e^(−a x) J₀(b x) dx = 1/√(a² + b²)|∫₀^1 x J_ν(α x) J_ν(β x) dx = 0 for distinct zeros α, β of J_ν (orthogonality)|∫₀^∞ J₀(a x) cos(b x) dx = 1/√(a² − b²) (0 ≤ b < a)"),
    ("airy;airy function;airy functions;airyai;airy ai;ai(x)", "Airy Ai",
     "∫x Ai(x) dx = Ai′(x) + C (since Ai″(x) = x Ai(x))|∫x² Ai(x) dx = x Ai′(x) − Ai(x) + C|∫Ai(x)² dx = x Ai(x)² − Ai′(x)² + C",
     "∫₀^∞ Ai(x) dx = 1/3|∫₋∞^0 Ai(x) dx = 2/3|∫₋∞^∞ Ai(x) dx = 1|∫₀^∞ Ai(x)² dx = Ai′(0)² = 1/(3^(2/3) Γ(1/3)²) ≈ 0.0670|∫₋∞^∞ Ai(x) e^(k x) dx = e^(k³/3) (k > 0)"),
    ("li;li(x);logarithmic integral;logintegral", "li(x)",
     "∫li(x) dx = x li(x) − Ei(2 log x) + C|∫li(x)/x dx = log(x) li(x) − x + C|d/dx li(x) = 1/log(x)",
     "∫₀^1 li(x) dx = −log 2|li(x) = ∫₀^x dt/log(t) (Cauchy principal value)"),
    ("product log;productlog;lambert w;lambertw;w(x);lambert w function", "product log W(x)",
     "∫W(x) dx = x (W(x) − 1 + 1/W(x)) + C|∫W(x)/x dx = W(x) + W(x)²/2 + C",
     "∫₀^e W(x) dx = e − 1|∫₀^∞ W(1/x²) dx = √(2π)|∫₀^∞ W(x)/(x √x) dx = 2√(2π)"),
    ("legendrep;legendre p;legendre polynomial;legendre polynomials;legendre", "Legendre P",
     "∫Pₙ(x) dx = (P_(n+1)(x) − P_(n−1)(x))/(2n + 1) + C (n ≥ 1)|∫P₁(x) dx = x²/2 + C|∫P₂(x) dx = (x³ − x)/2 + C",
     "∫₋₁^1 Pₘ(x) Pₙ(x) dx = 2 δₘₙ/(2n + 1)|∫₋₁^1 Pₙ(x) dx = 2 δₙ₀|∫₋₁^1 xᵐ Pₙ(x) dx = 0 for m < n|∫₀^1 P₂ₖ₊₁(x) dx = (−1)ᵏ (2k)!/(2^(2k+1) k! (k + 1)!)"),
    ("si;si(x);sine integral;sinintegral", "Si(x)",
     "∫Si(x) dx = x Si(x) + cos(x) + C|∫x Si(x) dx = (x² Si(x) + x cos(x) − sin(x))/2 + C|d/dx Si(x) = sin(x)/x",
     "∫₀^1 Si(x) dx = Si(1) + cos(1) − 1 ≈ 0.4864|∫₀^∞ (π/2 − Si(x)) dx = 1|∫₀^∞ e^(−a x) Si(x) dx = arccot(a)/a = arctan(1/a)/a|Si(∞) = ∫₀^∞ sin(t)/t dt = π/2"),
];

fn clean(f: &str) -> String {
    // "Si", "LegendreP", "exp(t)": drop spaces and the argument
    let f = f.trim().trim_start_matches("the ").trim_end_matches(" function").trim_end_matches(" functions");
    f.to_string()
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let m = re!(r"^(?:sample |some |list of |table of )?(definite |indefinite )?integrals\s+(?:containing|with|using(?: the)?|involving)\s+(.+)$|^(?:sample |table of )(definite |indefinite )?integrals\s+(?:of\s+)?(.+)$").captures(l)?;
    let def = m.get(1).or(m.get(3)).map(|x| x.as_str().trim());
    let f = clean(m.get(2).or(m.get(4))?.as_str());
    let (_, name, indef, defs) = TABLES.iter().find(|t| t.0.split(';').any(|k| k == f || k == f.replace(' ', "")))?;
    let usable = |s: &str| -> Vec<String> { s.split('|').map(|x| x.trim().to_string()).collect() };
    let (i, d) = (usable(indef), usable(defs));
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{}integrals involving {name}", def.map(|d| format!("{d} ")).unwrap_or_default())))];
    let rows = |v: &[String]| -> Vec<Vec<Content>> { v.iter().map(|x| vec![text(x.clone())]).collect() };
    match def {
        Some("definite") => {
            pods.push(primary(pod("Result", "Definite integrals", table(None, rows(&d)))));
        }
        Some(_) => {
            pods.push(primary(pod("Result", "Indefinite integrals", table(None, rows(&i)))));
        }
        None => {
            pods.push(primary(pod("Result", "Indefinite integrals", table(None, rows(&i)))));
            pods.push(pod("Definite", "Definite integrals", table(None, rows(&d))));
        }
    }
    Some(Answer::new(format!("integrals involving {name}"), pods))
}
