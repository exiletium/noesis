//! The geomagnetic field from the International Geomagnetic Reference Field
//! (IGRF-14, IAGA): declination, inclination and intensity anywhere on Earth
//! from 1900 to 2030.

use super::geo::{find_city, home};
use super::misc::{parse_date, today};
use crate::engine::*;
use crate::print::fmt_float;
use chrono::{Datelike, NaiveDate};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

static IGRF: &str = include_str!("../../data/igrf14.txt");
const NMAX: usize = 13;

struct Coef {
    g: bool,
    n: usize,
    m: usize,
    epochs: Vec<f64>,
    sv: f64,
}

fn coefs() -> &'static Vec<Coef> {
    static C: OnceLock<Vec<Coef>> = OnceLock::new();
    C.get_or_init(|| {
        IGRF.lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .filter_map(|l| {
                let f: Vec<&str> = l.split_whitespace().collect();
                let nums: Vec<f64> = f[3..].iter().map(|x| x.parse().unwrap_or(0.0)).collect();
                let (epochs, sv) = nums.split_at(nums.len() - 1);
                Some(Coef { g: f[0] == "g", n: f[1].parse().ok()?, m: f[2].parse().ok()?, epochs: epochs.to_vec(), sv: sv[0] })
            })
            .collect()
    })
}

/// g and h at a decimal year (1900–2030).
fn gh(year: f64) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut g = vec![vec![0.0; NMAX + 1]; NMAX + 1];
    let mut h = vec![vec![0.0; NMAX + 1]; NMAX + 1];
    for c in coefs() {
        let last = c.epochs.len() - 1;
        let t = (year - 1900.0) / 5.0;
        let v = if t >= last as f64 {
            c.epochs[last] + c.sv * (year - (1900.0 + 5.0 * last as f64))
        } else {
            let i = t.floor().max(0.0) as usize;
            let f = t - i as f64;
            c.epochs[i] + f * (c.epochs[i + 1] - c.epochs[i])
        };
        if c.g {
            g[c.n][c.m] = v;
        } else {
            h[c.n][c.m] = v;
        }
    }
    (g, h)
}

/// North, east and down components (nT) at geodetic latitude/longitude
/// (degrees), altitude (km) and decimal year.
pub(super) fn field(lat: f64, lon: f64, alt: f64, year: f64) -> (f64, f64, f64) {
    let (g, h) = gh(year);
    let (a, b) = (6378.137_f64, 6356.752314245_f64);
    let (a2, b2) = (a * a, b * b);
    let colat = (90.0 - lat).to_radians();
    let (mut ct, mut st) = (colat.cos(), colat.sin().max(1e-10));
    let (one, two) = (a2 * st * st, b2 * ct * ct);
    let three = one + two;
    let rho = three.sqrt();
    let r = (alt * (alt + 2.0 * rho) + (a2 * one + b2 * two) / three).sqrt();
    let cd = (alt + rho) / r;
    let sd = (a2 - b2) / rho * ct * st / r;
    let old = ct;
    ct = ct * cd - st * sd;
    st = st * cd + old * sd;
    // Schmidt semi-normalised associated Legendre functions and θ-derivatives
    let mut p = vec![vec![0.0; NMAX + 1]; NMAX + 1];
    let mut dp = vec![vec![0.0; NMAX + 1]; NMAX + 1];
    p[0][0] = 1.0;
    for n in 1..=NMAX {
        for m in 0..=n {
            let (nf, mf) = (n as f64, m as f64);
            if m == n {
                if n == 1 {
                    p[1][1] = st;
                    dp[1][1] = ct;
                } else {
                    let k = ((2.0 * nf - 1.0) / (2.0 * nf)).sqrt();
                    p[n][n] = k * st * p[n - 1][n - 1];
                    dp[n][n] = k * (st * dp[n - 1][n - 1] + ct * p[n - 1][n - 1]);
                }
            } else {
                let k1 = ((nf - 1.0) * (nf - 1.0) - mf * mf).max(0.0).sqrt();
                let k2 = (nf * nf - mf * mf).sqrt();
                let (p2, dp2) = if n >= 2 { (p[n - 2][m], dp[n - 2][m]) } else { (0.0, 0.0) };
                p[n][m] = ((2.0 * nf - 1.0) * ct * p[n - 1][m] - k1 * p2) / k2;
                dp[n][m] = ((2.0 * nf - 1.0) * (ct * dp[n - 1][m] - st * p[n - 1][m]) - k1 * dp2) / k2;
            }
        }
    }
    let ratio = 6371.2 / r;
    let lam = lon.to_radians();
    let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
    for n in 1..=NMAX {
        let rn = ratio.powi(n as i32 + 2);
        for m in 0..=n {
            let (cm, sm) = ((m as f64 * lam).cos(), (m as f64 * lam).sin());
            let gc = g[n][m] * cm + h[n][m] * sm;
            x += rn * gc * dp[n][m];
            y += rn * m as f64 * (g[n][m] * sm - h[n][m] * cm) * p[n][m] / st;
            z -= rn * (n as f64 + 1.0) * gc * p[n][m];
        }
    }
    // back from geocentric to geodetic components
    (x * cd + z * sd, y, z * cd - x * sd)
}

fn deg_min(d: f64, pos: &str, neg: &str) -> String {
    let a = d.abs();
    let mut deg = a.floor();
    let mut min = ((a - deg) * 60.0).round();
    if min >= 60.0 {
        deg += 1.0;
        min = 0.0;
    }
    format!("{}° {}′ {}", deg, min, if d >= 0.0 { pos } else { neg })
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches('?').trim();
    let m = re!(r"^(?:what is (?:the )?|show (?:me )?(?:the )?)?(geomagnetism|geomagnetic field|geomagnetic data|earth'?s magnetic field|earth magnetic field|magnetic declination|magnetic variation|magnetic inclination|magnetic dip|compass declination|magnetic field strength|magnetic field)(?:\s+(?:in|at|for|of|near))?\s*(.*)$").captures(l)?;
    let kind = m[1].to_string();
    let mut rest = m[2].trim().to_string();
    let mut date = today();
    if let Some(d) = re!(r"^(.*?)\s*(?:\bon\s+|\bin\s+|\bfor\s+|,\s*)?(\d{4}-\d\d-\d\d|\d{1,2} [a-z]+ \d{4}|[a-z]+ \d{1,2},? \d{4}|\d{4})$").captures(&rest) {
        let ds = d[2].to_string();
        let parsed = if ds.len() == 4 { ds.parse::<i32>().ok().and_then(|y| NaiveDate::from_ymd_opt(y, 7, 1)) } else { parse_date(&ds) };
        if let Some(x) = parsed {
            date = x;
            rest = d[1].trim().to_string();
        }
    }
    let rest = rest.trim_start_matches("in ").trim_start_matches("at ").trim().to_string();
    let (place, lat, lon) = if rest.is_empty() || rest == "here" || rest == "today" || rest == "now" {
        if kind == "magnetic field" || kind == "magnetic field strength" {
            return None;
        }
        let (name, lat, lon, _) = home();
        (name, lat, lon)
    } else {
        let c = find_city(&rest)?;
        (format!("{}, {}", c.name, super::geo::countries().get(&c.country).map_or(c.country.clone(), |x| x.name.clone())), c.lat, c.lon)
    };
    if !(1900..=2030).contains(&date.year()) {
        return None;
    }
    let year = date.year() as f64 + (date.ordinal0() as f64) / if date.leap_year() { 366.0 } else { 365.0 };
    let (x, y, z) = field(lat, lon, 0.0, year);
    let hz = x.hypot(y);
    let f = hz.hypot(z);
    let decl = y.atan2(x).to_degrees();
    let incl = z.atan2(hz).to_degrees();
    // annual change from the model a year later
    let (x1, y1, z1) = field(lat, lon, 0.0, (year + 1.0).min(2030.0));
    let step = ((year + 1.0).min(2030.0) - year).max(1e-6);
    let d_decl = (y1.atan2(x1).to_degrees() - decl) * 60.0 / step;
    let d_f = (x1.hypot(y1).hypot(z1) - f) / step;
    let nt = |v: f64| format!("{} nT", fmt_float(v.round(), 6));
    let date_s = date.format("%-d %B %Y").to_string();
    let rows = vec![
        vec![text("declination"), text(format!("{} ({}°)", deg_min(decl, "E", "W"), fmt_float((decl * 100.0).round() / 100.0, 5)))],
        vec![text("inclination"), text(format!("{} ({}°)", deg_min(incl, "down", "up"), fmt_float((incl * 100.0).round() / 100.0, 5)))],
        vec![text("total intensity"), text(nt(f))],
        vec![text("horizontal intensity"), text(nt(hz))],
        vec![text("north component"), text(nt(x))],
        vec![text("east component"), text(nt(y))],
        vec![text("vertical component (down)"), text(nt(z))],
    ];
    let main = match kind.as_str() {
        "magnetic declination" | "magnetic variation" | "compass declination" => format!("declination {}", deg_min(decl, "E", "W")),
        "magnetic inclination" | "magnetic dip" => format!("inclination {}", deg_min(incl, "down", "up")),
        _ => format!("declination {} | inclination {} | total intensity {}", deg_min(decl, "E", "W"), deg_min(incl, "down", "up"), nt(f)),
    };
    Some(Answer::new("geomagnetic field", vec![
        pod("Input", "Input interpretation", text(format!("geomagnetic field | {place} | {date_s}"))),
        primary(pod("Result", "Result", text(main))),
        pod("Field", "Magnetic field components", table(None, rows)),
        pod("Change", "Annual change", text(format!("declination {}′ per year, total intensity {} nT per year", fmt_float((d_decl * 10.0).round() / 10.0, 4), fmt_float(d_f.round(), 5)))),
        pod("Notes", "Notes", text(format!("International Geomagnetic Reference Field (IGRF-14, IAGA) at sea level, {:.4}° {}, {:.4}° {}. Declination is the angle of magnetic north east (E) or west (W) of true north.", lat.abs(), if lat >= 0.0 { "N" } else { "S" }, lon.abs(), if lon >= 0.0 { "E" } else { "W" }))),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn igrf_matches_published_values() {
        // Sydney in 2025: declination about 12.7° E, inclination about −64.4°
        let (x, y, z) = field(-33.87, 151.21, 0.0, 2025.0);
        assert!((y.atan2(x).to_degrees() - 12.7).abs() < 0.3, "{}", y.atan2(x).to_degrees());
        assert!((z.atan2(x.hypot(y)).to_degrees() + 64.4).abs() < 0.3);
        // the dipole term dominates: about 30,000 nT at the equator, 60,000 at the poles
        let (x, y, z) = field(0.0, 0.0, 0.0, 2020.0);
        assert!(((x * x + y * y + z * z).sqrt() - 31_900.0).abs() < 500.0);
    }
}
