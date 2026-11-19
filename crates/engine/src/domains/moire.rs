//! Moiré patterns: two overlaid line gratings or lattices, rotated against each
//! other or with slightly different spacings, and the period of the fringes.
//!
//! "moire pattern", "moire pattern at 30 degrees",
//! "moire pattern with a 0.2 spacing shift", "hexagonal moire pattern".

use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// Clip the line {p : p·n = c} to the square |x|, |y| ≤ w.
fn clip(n: [f64; 2], c: f64, w: f64) -> Option<[[f64; 2]; 2]> {
    let p0 = [n[0] * c, n[1] * c];
    let d = [-n[1], n[0]];
    let (mut t0, mut t1) = (f64::NEG_INFINITY, f64::INFINITY);
    for i in 0..2 {
        if d[i].abs() < 1e-12 {
            if p0[i].abs() > w {
                return None;
            }
        } else {
            let (a, b) = ((-w - p0[i]) / d[i], (w - p0[i]) / d[i]);
            t0 = t0.max(a.min(b));
            t1 = t1.min(a.max(b));
        }
    }
    (t1 > t0).then(|| [[p0[0] + t0 * d[0], p0[1] + t0 * d[1]], [p0[0] + t1 * d[0], p0[1] + t1 * d[1]]])
}

/// One family of parallel lines with normal at angle `phi` and spacing `a`.
fn grating(phi: f64, a: f64, w: f64, out: &mut Vec<[[f64; 2]; 2]>) {
    let n = [phi.cos(), phi.sin()];
    let k = (w * std::f64::consts::SQRT_2 / a).ceil() as i64;
    for i in -k..=k {
        if let Some(s) = clip(n, i as f64 * a, w) {
            out.push(s);
        }
    }
}

fn f(x: f64) -> String {
    fmt_float((x * 1000.0).round() / 1000.0, 6)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let m = re!(r"^(?:(?:plot|draw|show|make|generate)(?: me)? )?(?:an? |the )?(?:(hexagonal|triangular|square|line|linear|grating) )?moir[eé](?: (?:pattern|fringes|effect))?(?:(?: of| with| for)? (hexagonal|triangular|square) (?:lattices|grids))?(?: (?:at|with|of|for|rotated by|rotation) (?:an? )?(?:angle (?:of )?|rotation (?:of )?)?(-?[\d.]+) ?(?:degrees?|°|deg))?(?: (?:with|and|at) (?:an? )?(?:spacing (?:shift|difference|change|mismatch) (?:of )?([\d.]+)|([\d.]+) (?:spacing|period|pitch) (?:shift|difference|change|mismatch)))?$").captures(&l)?;
    let kind = m.get(1).or(m.get(2)).map_or("line", |x| x.as_str());
    let kind = match kind {
        "hexagonal" | "triangular" => "hexagonal",
        "square" => "square",
        _ => "line",
    };
    let shift: Option<f64> = m.get(4).or(m.get(5)).and_then(|x| x.as_str().parse().ok());
    let angle_deg: f64 = match m.get(3) {
        Some(x) => x.as_str().parse().ok()?,
        None if shift.is_some() => 0.0,
        None => 10.0,
    };
    if !(angle_deg.abs() <= 90.0) || shift.is_some_and(|s| !(s > 0.0 && s < 5.0)) {
        return None;
    }
    let th = angle_deg.to_radians();
    let (a1, a2) = (1.0, 1.0 + shift.unwrap_or(0.0));
    // difference of the two wave vectors (1/a1)n1 − (1/a2)n2 gives the fringes
    let kx = 1.0 / a1 - th.cos() / a2;
    let ky = -th.sin() / a2;
    let kd = (kx * kx + ky * ky).sqrt();
    let period = if kd > 1e-9 { Some(1.0 / kd) } else { None };
    let w = period.map_or(8.0, |p| (2.0 * p).clamp(8.0, 16.0));
    let pl = if kind == "line" {
        let mut segs = Vec::new();
        for (a, rot) in [(a1, 0.0), (a2, th)] {
            grating(rot, a, w, &mut segs);
        }
        let mut pl = crate::plot::segments(segs, "moiré");
        pl.x_range = (-w, w);
        pl.y_range = (-w, w);
        pl
    } else {
        // lattice points of both layers; the moiré shows where they line up
        let (w, basis) = (w.min(11.0), if kind == "hexagonal" { [[1.0, 0.0], [0.5, 3f64.sqrt() / 2.0]] } else { [[1.0, 0.0], [0.0, 1.0]] });
        let mut pts = Vec::new();
        for (a, rot) in [(a1, 0.0), (a2, th)] {
            let (c, sn) = (f64::cos(rot), f64::sin(rot));
            let k = (2.0 * w / a).ceil() as i64;
            for i in -k..=k {
                for j in -k..=k {
                    let x = a * (i as f64 * basis[0][0] + j as f64 * basis[1][0]);
                    let y = a * (i as f64 * basis[0][1] + j as f64 * basis[1][1]);
                    let (xr, yr) = (c * x - sn * y, sn * x + c * y);
                    if xr.abs() <= w && yr.abs() <= w {
                        pts.push([xr, yr]);
                    }
                }
            }
        }
        let mut pl = crate::plot::scatter(pts, "lattice points", None);
        pl.equal_aspect = true;
        pl
    };
    let what = match kind {
        "hexagonal" => "two triangular (hexagonal) lattices of lattice constant a",
        "square" => "two square grids of spacing a",
        _ => "two line gratings of period a",
    };
    let mut rows = vec![
        vec![text("layers"), text(what)],
        vec![text("periods"), text(if shift.is_some() { format!("a₁ = 1, a₂ = {}", f(a2)) } else { "a = 1".into() })],
        vec![text("relative rotation"), text(format!("θ = {}°", f(angle_deg)))],
    ];
    let formula = if shift.is_none() {
        "D = a / (2 sin(θ/2))"
    } else if angle_deg == 0.0 {
        "D = a₁ a₂ / |a₂ − a₁|"
    } else {
        "D = a₁ a₂ / √(a₁² + a₂² − 2 a₁ a₂ cos θ)"
    };
    rows.push(vec![text("moiré period"), text(match period {
        Some(p) => format!("{formula} = {}", f(p)),
        None => "none (the layers coincide)".into(),
    })]);
    if let Some(p) = period {
        let dir = (ky.atan2(kx).to_degrees() + 90.0).rem_euclid(180.0);
        rows.push(vec![text("fringe direction"), text(format!("{}° from the x-axis{}", f(dir), if kind == "line" { "" } else { " (one of the superlattice directions)" }))]);
        rows.push(vec![text("magnification"), text(format!("D/a = {}", f(p)))]);
    }
    let title = format!("{} moiré pattern{}{}", match kind { "hexagonal" => "hexagonal", "square" => "square-grid", _ => "line" }, if angle_deg != 0.0 { format!(", θ = {}°", f(angle_deg)) } else { String::new() }, shift.map_or(String::new(), |s| format!(", spacing shift {}", f(s))));
    let mut pods = vec![
        pod("Input", "Input interpretation", text(title.clone())),
        primary(pod("Plot", "Moiré pattern", plot(pl))),
        pod("Properties", "Properties", table(None, rows)),
    ];
    if kind == "hexagonal" && shift.is_none() {
        pods.push(pod("Note", "Note", text("The same superlattice appears in twisted bilayer graphene, whose moiré period is a/(2 sin(θ/2)) with a = 0.246 nm.")));
    }
    Some(Answer::new(title, pods))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_axis_line() {
        let s = clip([1.0, 0.0], 0.5, 2.0).unwrap();
        assert!((s[0][0] - 0.5).abs() < 1e-12 && (s[0][1].abs() - 2.0).abs() < 1e-12);
        assert!(clip([1.0, 0.0], 3.0, 2.0).is_none());
    }
}
