//! Orbit geometry from any two of: periapsis and apoapsis distance,
//! semi-major axis and eccentricity.
//!
//! "semi major axis of an orbit with an apogee of 2000km and perigee of 275km",
//! "orbital radius with eccentricity 0.7 and semimajor axis 17.8 AU".

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

const UNITS: &[(&str, f64)] = &[("km", 1.0), ("kilometers", 1.0), ("kilometres", 1.0), ("m", 1e-3), ("meters", 1e-3), ("AU", 1.495978707e8), ("mi", 1.609344), ("miles", 1.609344), ("ly", 9.4607304725808e12), ("light years", 9.4607304725808e12)];

fn dist(num: &str, unit: Option<&str>) -> Option<(f64, &'static str, f64)> {
    let v: f64 = num.replace(',', "").parse().ok()?;
    let u = unit.unwrap_or("").trim();
    if u.is_empty() {
        return Some((v, "", 1.0));
    }
    let (name, f) = UNITS.iter().find(|(n, _)| n.to_lowercase() == u)?;
    Some((v * f, name, *f))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim().replace("semi-major", "semimajor").replace("semi major", "semimajor");
    if !re!(r"\b(?:orbit|orbital|apogee|perigee|apoapsis|periapsis|aphelion|perihelion|eccentricity|semimajor)").is_match(&l) {
        return None;
    }
    let num = r"(\d[\d,]*(?:\.\d+)?)\s*(km|kilometers|kilometres|m|meters|au|mi|miles|ly|light years)?\b";
    let grab = |names: &str| -> Option<(f64, &'static str, f64)> {
        let r = Regex::new(&format!(r"(?:{names})(?:\s+(?:of|is|=|distance of))?\s*(?:of\s+)?(?:=\s*)?{num}|{num}\s+(?:{names})")).ok()?;
        let m = r.captures(&l)?;
        if let Some(n) = m.get(1) {
            dist(n.as_str(), m.get(2).map(|x| x.as_str()))
        } else {
            dist(m.get(3)?.as_str(), m.get(4).map(|x| x.as_str()))
        }
    };
    let apo = grab("apogee|apoapsis|aphelion|apocenter|apocentre");
    let peri = grab("perigee|periapsis|perihelion|pericenter|pericentre");
    let semi = grab("semimajor axis");
    let ecc: Option<f64> = re!(r"eccentricity(?:\s+(?:of|is|=))?\s*=?\s*(0?\.\d+|0|1(?:\.0+)?)\b").captures(&l).and_then(|m| m[1].parse().ok())
        .or_else(|| re!(r"\b(0?\.\d+)\s+eccentricity").captures(&l).and_then(|m| m[1].parse().ok()));
    let given = [apo.is_some(), peri.is_some(), semi.is_some(), ecc.is_some()].iter().filter(|x| **x).count();
    if given < 2 {
        return None;
    }
    // working unit: the first unit mentioned, else km
    let (uname, uf) = [apo, peri, semi].iter().flatten().find(|x| !x.1.is_empty()).map(|x| (x.1, x.2)).unwrap_or(("km", 1.0));
    let (ra, rp, a, e) = match (apo, peri, semi, ecc) {
        (Some(ra), Some(rp), _, _) => {
            let (ra, rp) = (ra.0.max(rp.0), ra.0.min(rp.0));
            let a = (ra + rp) / 2.0;
            (ra, rp, a, (ra - rp) / (ra + rp))
        }
        (_, _, Some(a), Some(e)) if (0.0..1.0).contains(&e) => (a.0 * (1.0 + e), a.0 * (1.0 - e), a.0, e),
        (Some(ra), _, Some(a), _) if ra.0 >= a.0 => (ra.0, 2.0 * a.0 - ra.0, a.0, ra.0 / a.0 - 1.0),
        (_, Some(rp), Some(a), _) if rp.0 <= a.0 => (2.0 * a.0 - rp.0, rp.0, a.0, 1.0 - rp.0 / a.0),
        (Some(ra), _, _, Some(e)) if (0.0..1.0).contains(&e) => {
            let a = ra.0 / (1.0 + e);
            (ra.0, a * (1.0 - e), a, e)
        }
        (_, Some(rp), _, Some(e)) if (0.0..1.0).contains(&e) => {
            let a = rp.0 / (1.0 - e);
            (a * (1.0 + e), rp.0, a, e)
        }
        _ => return None,
    };
    if !(rp > 0.0 && a > 0.0) {
        return None;
    }
    let b = a * (1.0 - e * e).sqrt();
    let p = a * (1.0 - e * e);
    let f = |x: f64| format!("{} {}", fmt_float(x / uf, 6), if uname.is_empty() { "km" } else { uname });
    let want = if l.contains("semimajor") && semi.is_none() {
        ("Semi-major axis", f(a))
    } else if l.contains("eccentricity") && ecc.is_none() {
        ("Eccentricity", fmt_float(e, 6))
    } else if re!(r"apogee|apoapsis|aphelion").is_match(&l) && apo.is_none() {
        ("Apoapsis distance", f(ra))
    } else if re!(r"perigee|periapsis|perihelion").is_match(&l) && peri.is_none() {
        ("Periapsis distance", f(rp))
    } else {
        ("Orbital radius range", format!("{} to {}", f(rp), f(ra)))
    };
    let rows = vec![
        vec![text("periapsis distance"), text(f(rp))],
        vec![text("apoapsis distance"), text(f(ra))],
        vec![text("semi-major axis"), text(f(a))],
        vec![text("semi-minor axis"), text(f(b))],
        vec![text("eccentricity"), text(fmt_float(e, 6))],
        vec![text("semi-latus rectum"), text(f(p))],
    ];
    Some(Answer::new("orbit geometry", vec![
        pod("Input", "Input interpretation", text("orbit geometry (distances measured from the centre of the primary body)")),
        primary(pod("Result", want.0, text(want.1))),
        pod("Table", "Orbit", table(None, rows)),
        pod("Formulas", "Formulas", text("a = (r_a + r_p)/2  |  e = (r_a − r_p)/(r_a + r_p)  |  r_p = a(1 − e), r_a = a(1 + e)  |  b = a√(1 − e²)")),
    ]))
}
