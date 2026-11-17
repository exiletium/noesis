//! A low-precision ephemeris: where the planets and the Moon are *now*.
//!
//! Planets use the mean Keplerian elements and rates published by JPL
//! ("Approximate Positions of the Planets", valid 1800–2050; errors of a
//! few arcminutes at worst).  The Moon's distance uses the leading periodic
//! terms of the lunar theory as tabulated by Meeus (accurate to a few tens
//! of kilometres).  Good to about four significant figures in distance.

use chrono::{DateTime, Utc};

/// Julian centuries (TT) since J2000.0 for a UTC instant.
pub fn centuries(at: DateTime<Utc>) -> f64 {
    let jd = at.timestamp() as f64 / 86400.0 + 2_440_587.5 + 69.2 / 86400.0;
    (jd - 2_451_545.0) / 36525.0
}

pub fn now() -> f64 {
    centuries(Utc::now())
}

/// (name, [a, e, I, L, long. of perihelion, long. of node], rates per century); au and degrees.
const ELEMENTS: &[(&str, [f64; 6], [f64; 6])] = &[
    ("Mercury", [0.38709927, 0.20563593, 7.00497902, 252.25032350, 77.45779628, 48.33076593], [0.00000037, 0.00001906, -0.00594749, 149472.67411175, 0.16047689, -0.12534081]),
    ("Venus", [0.72333566, 0.00677672, 3.39467605, 181.97909950, 131.60246718, 76.67984255], [0.00000390, -0.00004107, -0.00078890, 58517.81538729, 0.00268329, -0.27769418]),
    ("Earth", [1.00000261, 0.01671123, -0.00001531, 100.46457166, 102.93768193, 0.0], [0.00000562, -0.00004392, -0.01294668, 35999.37244981, 0.32327364, 0.0]),
    ("Mars", [1.52371034, 0.09339410, 1.84969142, -4.55343205, -23.94362959, 49.55953891], [0.00001847, 0.00007882, -0.00813131, 19140.30268499, 0.44441088, -0.29257343]),
    ("Jupiter", [5.20288700, 0.04838624, 1.30439695, 34.39644051, 14.72847983, 100.47390909], [-0.00011607, -0.00013253, -0.00183714, 3034.74612775, 0.21252668, 0.20469106]),
    ("Saturn", [9.53667594, 0.05386179, 2.48599187, 49.95424423, 92.59887831, 113.66242448], [-0.00125060, -0.00050991, 0.00193609, 1222.49362201, -0.41897216, -0.28867794]),
    ("Uranus", [19.18916464, 0.04725744, 0.77263783, 313.23810451, 170.95427630, 74.01692503], [-0.00196176, -0.00004397, -0.00242939, 428.48202785, 0.40805281, 0.04240589]),
    ("Neptune", [30.06992276, 0.00859048, 1.77004347, -55.12002969, 44.96476227, 131.78422574], [0.00026291, 0.00005105, 0.00035372, 218.45945325, -0.32241464, -0.00508664]),
    ("Pluto", [39.48211675, 0.24882730, 17.14001206, 238.92903833, 224.06891629, 110.30393684], [-0.00031596, 0.00005170, 0.00004818, 145.20780515, -0.04062942, -0.01183482]),
];

/// Semi-major axis (au) and eccentricity at `t` centuries from J2000.
pub fn orbit_shape(name: &str, t: f64) -> Option<(f64, f64)> {
    let (_, el, rate) = ELEMENTS.iter().find(|(n, _, _)| *n == name)?;
    Some((el[0] + rate[0] * t, el[1] + rate[1] * t))
}

/// Heliocentric ecliptic position (au) at `t` centuries from J2000.
/// "Earth" is the Earth–Moon barycentre; the Sun is the origin.
pub fn heliocentric(name: &str, t: f64) -> Option<[f64; 3]> {
    if name == "Sun" {
        return Some([0.0; 3]);
    }
    let name = if name == "Moon" { "Earth" } else { name };
    let (_, el, rate) = ELEMENTS.iter().find(|(n, _, _)| *n == name)?;
    let v: Vec<f64> = (0..6).map(|k| el[k] + rate[k] * t).collect();
    let (a, e, inc, l, peri, node) = (v[0], v[1], v[2].to_radians(), v[3], v[4], v[5]);
    let w = (peri - node).to_radians();
    let node = node.to_radians();
    let m = ((l - peri) % 360.0).to_radians();
    let mut ecc = m;
    for _ in 0..30 {
        ecc -= (ecc - e * ecc.sin() - m) / (1.0 - e * ecc.cos());
    }
    let (x, y) = (a * (ecc.cos() - e), a * (1.0 - e * e).sqrt() * ecc.sin());
    let (cw, sw, cn, sn, ci, si) = (w.cos(), w.sin(), node.cos(), node.sin(), inc.cos(), inc.sin());
    Some([(cw * cn - sw * sn * ci) * x + (-sw * cn - cw * sn * ci) * y, (cw * sn + sw * cn * ci) * x + (-sw * sn + cw * cn * ci) * y, sw * si * x + cw * si * y])
}

/// The Moon's fundamental arguments (degrees): D, M, M', F.
fn lunar_arguments(t: f64) -> [f64; 4] {
    [
        297.8501921 + 445267.1114034 * t - 0.0018819 * t * t + t * t * t / 545868.0,
        357.5291092 + 35999.0502909 * t - 0.0001536 * t * t,
        134.9633964 + 477198.8675055 * t + 0.0087414 * t * t + t * t * t / 69699.0,
        93.2720950 + 483202.0175233 * t - 0.0036539 * t * t,
    ]
}

/// Periodic terms of the Moon's distance: multiples of D, M, M', F and the
/// cosine coefficient in metres.
const MOON_R: &[(i8, i8, i8, i8, f64)] = &[
    (0, 0, 1, 0, -20905355.0), (2, 0, -1, 0, -3699111.0), (2, 0, 0, 0, -2955968.0), (0, 0, 2, 0, -569925.0), (0, 1, 0, 0, 48888.0),
    (0, 0, 0, 2, -3149.0), (2, 0, -2, 0, 246158.0), (2, -1, -1, 0, -152138.0), (2, 0, 1, 0, -170733.0), (2, -1, 0, 0, -204586.0),
    (0, 1, -1, 0, -129620.0), (1, 0, 0, 0, 108743.0), (0, 1, 1, 0, 104755.0), (2, 0, 0, -2, 10321.0), (0, 0, 1, -2, 79661.0),
    (4, 0, -1, 0, -34782.0), (0, 0, 3, 0, -23210.0), (4, 0, -2, 0, -21636.0), (2, 1, -1, 0, 24208.0), (2, 1, 0, 0, 30824.0),
    (1, 0, -1, 0, -8379.0), (1, 1, 0, 0, -16675.0), (2, -1, 1, 0, -12831.0), (2, 0, 2, 0, -10445.0), (4, 0, 0, 0, -11650.0),
    (2, 0, -3, 0, 14403.0), (0, 1, -2, 0, -7003.0), (2, -1, -2, 0, 10056.0), (1, 0, 1, 0, 6322.0), (2, -2, 0, 0, -9884.0),
    (0, 1, 2, 0, 5751.0), (2, -2, -1, 0, -4950.0), (2, 0, 1, -2, 4130.0), (4, -1, -1, 0, -3958.0), (3, 0, -1, 0, 3258.0),
    (2, 1, 1, 0, 2616.0), (4, -1, -2, 0, -1897.0), (0, 2, -1, 0, -2117.0), (2, 2, -1, 0, 2354.0),
];

/// Distance between the centres of the Earth and the Moon, metres.
pub fn moon_distance(t: f64) -> f64 {
    let [d, m, mp, f] = lunar_arguments(t).map(f64::to_radians);
    let e = 1.0 - 0.002516 * t - 0.0000074 * t * t;
    let sum: f64 = MOON_R
        .iter()
        .map(|&(kd, km, kmp, kf, c)| {
            let arg = kd as f64 * d + km as f64 * m + kmp as f64 * mp + kf as f64 * f;
            c * e.powi(km.abs() as i32) * arg.cos()
        })
        .sum();
    385_000_560.0 + sum
}

/// Illuminated fraction of the Moon's disc and whether it is waxing.
pub fn moon_phase(t: f64) -> (f64, bool, f64) {
    let [d, m, mp, _] = lunar_arguments(t);
    let i = 180.0 - d - 6.289 * mp.to_radians().sin() + 2.100 * m.to_radians().sin() - 1.274 * (2.0 * d - mp).to_radians().sin() - 0.658 * (2.0 * d).to_radians().sin();
    let k = (1.0 + i.to_radians().cos()) / 2.0;
    let elong = d.rem_euclid(360.0);
    (k, elong < 180.0, elong)
}

const AU: f64 = 1.495_978_707e11;

/// Distance between two solar-system bodies right now, in metres.
pub fn separation(a: &str, b: &str, t: f64) -> Option<f64> {
    if a == b {
        return None;
    }
    if (a == "Earth" && b == "Moon") || (a == "Moon" && b == "Earth") {
        return Some(moon_distance(t));
    }
    let (p, q) = (heliocentric(a, t)?, heliocentric(b, t)?);
    Some(AU * ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn positions_match_published_values() {
        // 2026-11-15 04:15 UTC: Mars 1.314 au from Earth; the Moon near apogee
        // (a published topocentric value is 404,812 km; the geocentric one is a
        // few hundred kilometres smaller)
        let t = centuries(Utc.with_ymd_and_hms(2026, 11, 15, 4, 15, 0).unwrap());
        let mars = separation("Earth", "Mars", t).unwrap() / AU;
        assert!((mars - 1.314).abs() < 0.002, "{mars}");
        let moon = moon_distance(t) / 1000.0;
        assert!((moon - 404_812.0).abs() < 400.0, "{moon}");
        // J2000.0: Earth 0.9833 au from the Sun (perihelion on 3 January), Moon 402,449 km
        let sun = separation("Earth", "Sun", 0.0).unwrap() / AU;
        assert!((sun - 0.98333).abs() < 2e-4, "{sun}");
        assert!((moon_distance(0.0) / 1000.0 - 402_449.0).abs() < 100.0);
    }
}

#[cfg(test)]
mod meeus {
    #[test]
    fn example_47a() {
        // Meeus, example 47.a: 1992 April 12, 0h TD: 368,409.7 km
        let t = (2_448_724.5 - 2_451_545.0) / 36525.0;
        let d = super::moon_distance(t) / 1000.0;
        assert!((d - 368_409.7).abs() < 30.0, "{d}");
    }
}
