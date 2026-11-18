//! A tool named without input ("linear fit", "Z-transform calculator",
//! "row reduction calculator") is answered, as WolframAlpha does, by
//! working a labelled example.

use super::*;
use std::cell::Cell;

thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// tool names (lower case, without "calculator") | example query
const EXAMPLES: &[(&[&str], &str)] = &[
    (&["row reduction", "row reduce", "rref", "reduced row echelon form", "gaussian elimination", "row echelon form"], "row reduce {{1,2,3},{4,5,6}}"),
    (&["interpolating polynomial", "polynomial interpolation", "lagrange interpolation"], "interpolating polynomial {(1,1),(2,4),(3,9)}"),
    (&["fourier transform"], "Fourier transform of exp(-x^2)"),
    (&["inverse fourier transform"], "inverse Fourier transform of exp(-w^2)"),
    (&["z-transform", "z transform"], "Z transform of 2^n"),
    (&["inverse z-transform", "inverse z transform"], "inverse Z transform of z/(z-2)"),
    (&["laplace transform"], "Laplace transform of sin(t)"),
    (&["inverse laplace transform"], "inverse Laplace transform of 1/(s^2+1)"),
    (&["linear fit", "linear regression", "least squares fit", "line of best fit", "best fit line"], "linear fit {1,2},{2,3.9},{3,6.1},{4,8.2}"),
    (&["quadratic fit", "quadratic regression"], "quadratic fit {1,1},{2,4.1},{3,9.2},{4,15.8}"),
    (&["cubic fit", "cubic regression"], "cubic fit {0,1},{1,2},{2,9},{3,28},{4,65}"),
    (&["normalize vector", "vector normalization", "unit vector"], "normalize (1,2,2)"),
    (&["matrix inverse", "inverse matrix", "matrix inversion"], "inverse {{2,1},{1,1}}"),
    (&["determinant"], "determinant {{1,2},{3,4}}"),
    (&["eigenvalues", "eigenvalue"], "eigenvalues {{2,1},{1,2}}"),
    (&["derivative", "differentiation"], "derivative of x^2 sin(x)"),
    (&["integral", "integration", "antiderivative"], "integrate x^2 sin(x)"),
    (&["definite integral"], "integrate x^2 from 0 to 1"),
    (&["limit"], "limit sin(x)/x as x->0"),
    (&["taylor series", "series expansion"], "Taylor series of e^x at x=0"),
    (&["partial fractions", "partial fraction decomposition"], "partial fractions (x+3)/(x^2-1)"),
    (&["quadratic formula", "quadratic equation solver"], "solve x^2-5x+6=0"),
];

/// "log plot e^x - x": a logarithmic y axis, drawn as log₁₀ of the function.
fn log_plot(q: &Query) -> Option<Answer> {
    let m = regex::Regex::new(r"(?i)^(?:log|log-linear|semilog|semi-log|logarithmic)\s+plot\s+(?:of\s+)?(.+?)(\s*,\s*[a-z]\s*=.+|\s+from\s+.+)?$").ok()?.captures(q.clean.trim())?;
    let f = m[1].trim();
    let rewritten = format!("plot log10({f}){}", m.get(2).map_or("", |x| x.as_str()));
    DEPTH.with(|d| d.set(1));
    let r = run_handlers(&rewritten, q.opts);
    DEPTH.with(|d| d.set(0));
    if !r.success {
        return None;
    }
    let mut pods = vec![pod("Scale", "Scale", text(format!("logarithmic y axis: the curve shown is log₁₀({f}), so each unit up is a factor of 10")))];
    pods.extend(r.pods);
    Some(Answer::new(format!("log plot of {f}"), pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    if DEPTH.with(|d| d.get()) > 0 {
        return None;
    }
    if let Some(a) = log_plot(q) {
        return Some(a);
    }
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = l.trim_end_matches(" calculator").trim_end_matches(" solver").trim_end_matches(" tool").trim();
    let (_, example) = EXAMPLES.iter().find(|(names, _)| names.contains(&l))?;
    DEPTH.with(|d| d.set(1));
    let r = run_handlers(example, q.opts);
    DEPTH.with(|d| d.set(0));
    if !r.success {
        return None;
    }
    let mut pods = vec![pod("Example", "Example", text(format!("no input was given, so here is an example: {example}")))];
    pods.extend(r.pods);
    Some(Answer::new(format!("{l} (example)"), pods))
}
