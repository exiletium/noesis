//! Finite (Galois) fields GF(q) with their addition and multiplication
//! tables, and geodetic reference ellipsoids and datums (GRS80, WGS84, GDA94).

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

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

// ---------------------------------------------------------------------------
// finite fields
// ---------------------------------------------------------------------------

/// q = p^n -> (p, n, defining polynomial coefficients, constant term first,
/// of the monic degree-n polynomial; the Conway polynomials).
fn field(q: u32) -> Option<(u32, u32, Vec<u32>)> {
    Some(match q {
        2 | 3 | 5 | 7 | 11 | 13 | 17 | 19 | 23 | 29 | 31 => (q, 1, vec![]),
        4 => (2, 2, vec![1, 1]),       // x^2 + x + 1
        8 => (2, 3, vec![1, 1, 0]),    // x^3 + x + 1
        9 => (3, 2, vec![2, 2]),       // x^2 + 2x + 2
        16 => (2, 4, vec![1, 1, 0, 0]), // x^4 + x + 1
        25 => (5, 2, vec![2, 4]),      // x^2 + 4x + 2
        27 => (3, 3, vec![1, 2, 0]),   // x^3 + 2x + 1
        _ => return None,
    })
}

fn digits(mut v: u32, p: u32, n: u32) -> Vec<u32> {
    (0..n).map(|_| { let d = v % p; v /= p; d }).collect()
}

fn undigits(d: &[u32], p: u32) -> u32 {
    d.iter().rev().fold(0, |acc, &x| acc * p + x)
}

fn mul(a: u32, b: u32, p: u32, n: u32, poly: &[u32]) -> u32 {
    if n == 1 {
        return a * b % p;
    }
    let (x, y) = (digits(a, p, n), digits(b, p, n));
    let mut prod = vec![0u32; (2 * n - 1) as usize];
    for i in 0..n as usize {
        for j in 0..n as usize {
            prod[i + j] = (prod[i + j] + x[i] * y[j]) % p;
        }
    }
    // reduce with x^n = -(poly)
    for k in (n as usize..prod.len()).rev() {
        let c = prod[k];
        if c == 0 {
            continue;
        }
        prod[k] = 0;
        for (i, &pc) in poly.iter().enumerate() {
            let idx = k - n as usize + i;
            prod[idx] = (prod[idx] + p * p - c * pc % p) % p;
        }
    }
    undigits(&prod[..n as usize], p)
}

fn add(a: u32, b: u32, p: u32, n: u32) -> u32 {
    let (x, y) = (digits(a, p, n), digits(b, p, n));
    undigits(&x.iter().zip(&y).map(|(u, v)| (u + v) % p).collect::<Vec<_>>(), p)
}

fn poly_text(p: u32, n: u32, poly: &[u32]) -> String {
    let _ = p;
    let mut terms = vec![format!("x^{n}")];
    for i in (0..n as usize).rev() {
        let c = poly[i];
        if c == 0 {
            continue;
        }
        let v = match i { 0 => String::new(), 1 => "x".into(), _ => format!("x^{i}") };
        terms.push(match (c, v.is_empty()) { (1, false) => v, (_, true) => c.to_string(), _ => format!("{c}{v}") });
    }
    terms.join(" + ")
}

fn finite_field(l: &str) -> Option<Answer> {
    let c = re!(r"^(?:the )?(?:gf|galois field|finite field)\s*(?:\(\s*(\d+)\s*\)|of order (\d+)|(?:with )?(\d+) elements|(\d+))$").captures(l)?;
    let q: u32 = c.get(1).or(c.get(2)).or(c.get(3)).or(c.get(4))?.as_str().parse().ok()?;
    let (p, n, poly) = field(q)?;
    let name = |v: u32| -> String {
        if n == 1 || v < p {
            v.to_string()
        } else {
            // the elements beyond the prime field are named a, b, c, … as WolframAlpha does
            ((b'a' + (v - p) as u8) as char).to_string()
        }
    };
    let header: Vec<String> = std::iter::once(String::new()).chain((0..q).map(name)).collect();
    let tab = |f: &dyn Fn(u32, u32) -> u32| -> Content {
        let rows = (0..q).map(|a| std::iter::once(text(name(a))).chain((0..q).map(|b| text(name(f(a, b))))).collect()).collect();
        table(Some(header.iter().map(|s| s.as_str()).collect()), rows)
    };
    // a primitive element: generates all q − 1 non-zero elements
    let prim = (2..q.max(3)).chain(1..2).find(|&g| {
        let mut x = 1;
        let mut seen = 0;
        loop {
            x = mul(x, g, p, n, &poly);
            seen += 1;
            if x == 1 {
                break;
            }
        }
        seen == q - 1
    });
    let mut pods = vec![
        input(format!("finite field GF({q})")),
        primary(pod("Result", "Addition table", tab(&|a, b| add(a, b, p, n)))),
        pod("Multiplication", "Multiplication table", tab(&|a, b| mul(a, b, p, n, &poly))),
    ];
    let mut props = vec![
        vec![text("order"), text(format!("{q} = {p}^{n}"))],
        vec![text("characteristic"), text(p.to_string())],
        vec![text("degree over GF(" .to_string() + &p.to_string() + ")"), text(n.to_string())],
        vec![text("multiplicative group"), text(format!("cyclic of order {}", q - 1))],
    ];
    if n > 1 {
        props.insert(3, vec![text("defining polynomial"), text(format!("{} over GF({p}) (Conway polynomial)", poly_text(p, n, &poly)))]);
        let names: Vec<String> = (p..q).map(|v| {
            let d = digits(v, p, n);
            let s: Vec<String> = (0..n as usize).rev().filter(|&i| d[i] != 0).map(|i| {
                let t = match i { 0 => String::new(), 1 => "x".into(), _ => format!("x^{i}") };
                match (d[i], t.is_empty()) { (1, false) => t, (c, true) => c.to_string(), (c, false) => format!("{c}{t}") }
            }).collect();
            format!("{} = {}", name(v), s.join(" + "))
        }).collect();
        pods.push(pod("Elements", "Elements as polynomials", text(names.join(", "))));
    }
    if let Some(g) = prim {
        props.push(vec![text("primitive element"), text(name(g))]);
    }
    pods.insert(1, pod("Properties", "Properties", table(None, props)));
    Some(Answer::new("finite field", pods))
}

// ---------------------------------------------------------------------------
// geodesy
// ---------------------------------------------------------------------------

/// names | full name | semimajor axis a (m) | inverse flattening 1/f | note
const ELLIPSOIDS: &[(&[&str], &str, f64, f64, &str)] = &[
    (&["grs80", "grs 80", "grs-80", "geodetic reference system 1980"], "Geodetic Reference System of 1980", 6378137.0, 298.257222101, "adopted by the IUGG in 1979; used by NAD83, ETRS89, GDA94 and GDA2020"),
    (&["wgs84 ellipsoid", "wgs 84 ellipsoid"], "World Geodetic System 1984 ellipsoid", 6378137.0, 298.257223563, "the ellipsoid of GPS; differs from GRS80 only by 0.1 mm in the semiminor axis"),
    (&["clarke 1866", "clarke ellipsoid"], "Clarke 1866 ellipsoid", 6378206.4, 294.978698214, "used by NAD27"),
    (&["airy 1830", "airy ellipsoid"], "Airy 1830 ellipsoid", 6377563.396, 299.3249646, "used by OSGB36 in Great Britain"),
    (&["bessel 1841", "bessel ellipsoid"], "Bessel 1841 ellipsoid", 6377397.155, 299.1528128, "used by the Tokyo datum and in central Europe"),
    (&["international 1924", "hayford ellipsoid", "hayford 1910"], "International 1924 (Hayford) ellipsoid", 6378388.0, 297.0, "used by ED50"),
    (&["krassovsky 1940", "krasovsky 1940", "krassowsky 1940"], "Krassovsky 1940 ellipsoid", 6378245.0, 298.3, "used by Pulkovo 1942"),
];

/// names | full name | ellipsoid | region | note
const DATUMS: &[(&[&str], &str, &str, &str, &str)] = &[
    (&["wgs84", "wgs 84", "wgs-84", "world geodetic system 1984"], "World Geodetic System 1984", "WGS84 ellipsoid", "global", "the datum of GPS, maintained by the US National Geospatial-Intelligence Agency"),
    (&["nad83", "nad 83", "north american datum 1983"], "North American Datum of 1983", "GRS80", "North America", "earth-centred; replaced NAD27"),
    (&["nad27", "nad 27", "north american datum 1927"], "North American Datum of 1927", "Clarke 1866", "North America", "origin at Meades Ranch, Kansas"),
    (&["gda94", "gda 94", "geocentric datum of australia 1994"], "Geocentric Datum of Australia 1994", "GRS80", "Australia", "fixed to ITRF92 at epoch 1994.0; superseded by GDA2020 (Australia has since drifted about 1.8 m north-east)"),
    (&["gda2020", "gda 2020", "geocentric datum of australia 2020"], "Geocentric Datum of Australia 2020", "GRS80", "Australia", "fixed to ITRF2014 at epoch 2020.0"),
    (&["etrs89", "etrs 89", "european terrestrial reference system 1989"], "European Terrestrial Reference System 1989", "GRS80", "Europe", "moves with the stable part of the Eurasian plate"),
    (&["ed50", "ed 50", "european datum 1950"], "European Datum 1950", "International 1924", "Europe", "origin at Potsdam"),
    (&["osgb36", "osgb 36", "ordnance survey great britain 1936"], "Ordnance Survey Great Britain 1936", "Airy 1830", "Great Britain", "basis of the British National Grid"),
    (&["tokyo datum", "tokyo 1918"], "Tokyo datum", "Bessel 1841", "Japan and Korea", "replaced by JGD2000"),
    (&["pulkovo 1942", "sk-42", "sk42"], "Pulkovo 1942", "Krassovsky 1940", "former Soviet Union and Eastern Europe", "origin at the Pulkovo Observatory"),
];

fn ellipsoid_rows(a: f64, inv_f: f64) -> Vec<Vec<Content>> {
    let f = 1.0 / inv_f;
    let b = a * (1.0 - f);
    let e2 = f * (2.0 - f);
    let r1 = (2.0 * a + b) / 3.0;
    vec![
        vec![text("semimajor axis a"), text(format!("{} m", fmt_float(a, 10)))],
        vec![text("semiminor axis b"), text(format!("{} m", fmt_float((b * 10000.0).round() / 10000.0, 11)))],
        vec![text("inverse flattening 1/f"), text(fmt_float(inv_f, 12))],
        vec![text("flattening f"), text(fmt_float(f, 8))],
        vec![text("eccentricity e"), text(fmt_float(e2.sqrt(), 8))],
        vec![text("mean radius (2a + b)/3"), text(format!("{} m", fmt_float((r1 * 1000.0).round() / 1000.0, 10)))],
    ]
}

fn geodesy(l: &str) -> Option<Answer> {
    let l = l.trim_start_matches("the ").trim_end_matches(" datum").trim_end_matches(" ellipsoid").trim();
    let l2 = format!("{l} ellipsoid");
    let exact = ELLIPSOIDS.iter().find(|e| e.0.contains(&l));
    let datum_named = DATUMS.iter().any(|d| d.0.contains(&l));
    if let Some(e) = exact.or_else(|| if datum_named { None } else { ELLIPSOIDS.iter().find(|e| e.0.contains(&l2.as_str())) }) {
        return Some(Answer::new("ellipsoid", vec![
            input(format!("{} | reference ellipsoid", e.1)),
            primary(pod("Result", "Name", text(e.1))),
            pod("Properties", "Properties", table(None, ellipsoid_rows(e.2, e.3))),
            pod("Note", "Note", text(e.4)),
        ]));
    }
    let d = DATUMS.iter().find(|d| d.0.contains(&l))?;
    let e = ELLIPSOIDS.iter().find(|e| e.1.starts_with(d.2) || e.0.iter().any(|n| n.eq_ignore_ascii_case(d.2)))?;
    Some(Answer::new("geodetic datum", vec![
        input(format!("{} | geodetic datum", d.1)),
        primary(pod("Result", "Reference ellipsoid", text(e.1))),
        pod("Datum", "Datum", table(None, vec![vec![text("name"), text(d.1)], vec![text("region"), text(d.3)], vec![text("note"), text(d.4)]])),
        pod("Properties", "Ellipsoid properties", table(None, ellipsoid_rows(e.2, e.3))),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    finite_field(l).or_else(|| geodesy(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_are_fields() {
        for q in [4, 8, 9, 16, 25, 27] {
            let (p, n, poly) = field(q).unwrap();
            // every non-zero element has an inverse
            for a in 1..q {
                assert!((1..q).any(|b| mul(a, b, p, n, &poly) == 1), "GF({q}) element {a}");
            }
        }
    }
}
