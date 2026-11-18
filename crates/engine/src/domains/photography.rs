//! Photography calculators: angle of view, depth of field, exposure value,
//! diffraction spot size, numerical aperture and print sizes.
//!
//! "angle of view 50mm", "field of view", "depth of field f=1m",
//! "exposure value f/8 1/125", "spot size f/8", "numerical aperture D=30mm",
//! "8\"x10\" photograph", "5\" by 7\" photo vs 8\" by 10\" photo".

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

fn f4(v: f64) -> String {
    fmt_float((v * 1e4).round() / 1e4, 6)
}

/// A length with a unit, in millimetres: "50mm", "1m", "3 ft", "12 in".
fn mm(num: &str, unit: &str) -> Option<f64> {
    let v: f64 = num.parse().ok()?;
    Some(v * match unit.trim() {
        "" | "mm" => 1.0,
        "cm" => 10.0,
        "m" => 1000.0,
        "in" | "\"" | "inch" | "inches" => 25.4,
        "ft" | "'" | "feet" | "foot" => 304.8,
        "um" | "μm" | "µm" => 0.001,
        _ => return None,
    })
}

/// key=value pairs: "f=1m", "D=30mm", "N=8", "s=3 m"
fn params(l: &str) -> Vec<(String, f64)> {
    re!(r"\b([a-z]+)\s*=\s*(\d+(?:\.\d+)?)\s*(mm|cm|m|in|ft|um|μm)?\b")
        .captures_iter(l)
        .filter_map(|m| Some((m[1].to_string(), mm(&m[2], m.get(3).map_or("", |u| u.as_str()))?)))
        .collect()
}

fn get(p: &[(String, f64)], keys: &[&str]) -> Option<f64> {
    p.iter().find(|(k, _)| keys.contains(&k.as_str())).map(|x| x.1)
}

fn input(s: String) -> Pod {
    pod("Input", "Input interpretation", text(s))
}

fn angle_of_view(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(?:angle|field) of view|aov|fov)(?:\s+(?:of|for)\s+(?:an?\s+)?|\s+)?(?:(\d+(?:\.\d+)?)\s*mm(?:\s+lens)?)?(?:\s+(?:on\s+)?(?:an?\s+)?(full[- ]frame|35\s*mm|aps-?c|micro four thirds|m43|four thirds)(?:\s+(?:camera|sensor))?)?$").captures(l)?;
    let f: f64 = m.get(1).map_or(Some(50.0), |x| x.as_str().parse().ok())?;
    let (w, h, name) = match m.get(2).map(|x| x.as_str().replace(['-', ' '], "")) {
        Some(s) if s.starts_with("apsc") => (23.6, 15.7, "APS-C (23.6 × 15.7 mm)"),
        Some(s) if s.contains("four") || s == "m43" => (17.3, 13.0, "Four Thirds (17.3 × 13 mm)"),
        _ => (36.0, 24.0, "full frame (36 × 24 mm)"),
    };
    if f <= 0.0 {
        return None;
    }
    let d = (w * w + h * h as f64).sqrt();
    let a = |x: f64| (2.0 * (x / (2.0 * f)).atan()).to_degrees();
    let assumed = if m.get(1).is_none() { " (assumed)" } else { "" };
    Some(Answer::new("angle of view", vec![
        input(format!("angle of view | focal length {} mm{assumed} | sensor {name}", fmt_float(f, 6))),
        primary(pod("Result", "Diagonal angle of view", text(format!("{}°", f4(a(d)))))),
        pod("Table", "Angles", table(None, vec![
            vec![text("horizontal"), text(format!("{}°", f4(a(w))))],
            vec![text("vertical"), text(format!("{}°", f4(a(h))))],
            vec![text("diagonal"), text(format!("{}°", f4(a(d))))],
        ])),
        pod("Formula", "Formula", text("α = 2 arctan(d/(2f)), with d the sensor width, height or diagonal and f the focal length (lens focused at infinity)")),
    ]))
}

fn depth_of_field(l: &str) -> Option<Answer> {
    let rest = re!(r"^(?:depth of field|dof\b)(?:\s+calculator)?(.*)$").captures(l)?[1].to_string();
    let p = params(&rest);
    // values not given as key=value
    let bare = re!(r"\b[a-z]+\s*=\s*\d+(?:\.\d+)?\s*(?:mm|cm|m|in|ft|um|μm)?\b").replace_all(&rest, "").to_string();
    let rest = bare;
    let num = re!(r"f/(\d+(?:\.\d+)?)").captures(&rest).and_then(|m| m[1].parse::<f64>().ok());
    let mut assumed = Vec::new();
    let mut pick = |v: Option<f64>, d: f64, what: &str| {
        v.unwrap_or_else(|| {
            assumed.push(what.to_string());
            d
        })
    };
    let f = pick(get(&p, &["f", "focal"]).or_else(|| re!(r"(\d+(?:\.\d+)?)\s*mm\b").captures(&rest).and_then(|m| m[1].parse().ok())), 50.0, "focal length 50 mm");
    let n = pick(num.or_else(|| get(&p, &["n", "aperture"])), 8.0, "f-number 8");
    let s = pick(get(&p, &["s", "d", "distance", "u"]).or_else(|| re!(r"(\d+(?:\.\d+)?)\s*(m|ft)\b").captures(&rest).and_then(|m| mm(&m[1], &m[2]))), 3000.0, "subject distance 3 m");
    let c = pick(get(&p, &["c", "coc"]), 0.03, "circle of confusion 0.03 mm");
    if f <= 0.0 || n <= 0.0 || s <= f || c <= 0.0 {
        return None;
    }
    let h = f * f / (n * c) + f;
    let near = s * (h - f) / (h + s - 2.0 * f);
    let far = if s < h { Some(s * (h - f) / (h - s)) } else { None };
    let m_ = |v: f64| format!("{} m", f4(v / 1000.0));
    let total = far.map_or("∞ (infinite)".to_string(), |fa| m_(fa - near));
    let mut pods = vec![
        input(format!("depth of field | focal length {} mm, f/{}, subject distance {}, circle of confusion {} mm", fmt_float(f, 6), fmt_float(n, 4), m_(s), fmt_float(c, 4))),
        primary(pod("Result", "Depth of field", text(total))),
        pod("Table", "Results", table(None, vec![
            vec![text("near limit"), text(m_(near))],
            vec![text("far limit"), text(far.map_or("∞".to_string(), m_))],
            vec![text("hyperfocal distance"), text(m_(h))],
        ])),
        pod("Formula", "Formulas", text("H = f²/(N c) + f; near = s(H − f)/(H + s − 2f); far = s(H − f)/(H − s) for s < H")),
    ];
    if !assumed.is_empty() {
        pods.push(pod("Assumptions", "Assumed values", text(assumed.join(", "))));
    }
    Some(Answer::new("depth of field", pods))
}

fn exposure_value(l: &str) -> Option<Answer> {
    let rest = re!(r"^(?:exposure value|ev\b)(?:\s+calculator)?(.*)$").captures(l)?[1].to_string();
    if l.starts_with("ev") && rest.trim().is_empty() {
        return None;
    }
    let n: Option<f64> = re!(r"f/?\s*(\d+(?:\.\d+)?)").captures(&rest).and_then(|m| m[1].parse().ok());
    let t: Option<f64> = re!(r"(?:^|\s)(\d+)\s*/\s*(\d+)\s*(?:s|sec|seconds?)?\b").captures(&rest).and_then(|m| Some(m[1].parse::<f64>().ok()? / m[2].parse::<f64>().ok()?)).or_else(|| re!(r"(\d+(?:\.\d+)?)\s*(?:s|sec|seconds?)\b").captures(&rest).and_then(|m| m[1].parse().ok()));
    let iso: Option<f64> = re!(r"iso\s*(\d+)").captures(&rest).and_then(|m| m[1].parse().ok());
    let mut assumed = vec![];
    let n = n.unwrap_or_else(|| {
        assumed.push("f/8");
        8.0
    });
    let t = t.unwrap_or_else(|| {
        assumed.push("1/125 s");
        1.0 / 125.0
    });
    if n <= 0.0 || t <= 0.0 {
        return None;
    }
    let ev = (n * n / t).log2();
    let tt = if t < 1.0 { format!("1/{} s", fmt_float((1.0 / t).round(), 6)) } else { format!("{} s", fmt_float(t, 4)) };
    let mut rows = vec![vec![text("EV (at the given settings)"), text(f4(ev))]];
    if let Some(iso) = iso {
        rows.push(vec![text(format!("EV₁₀₀ (scene, at ISO {})", fmt_float(iso, 6))), text(f4(ev - (iso / 100.0).log2()))]);
    }
    let mut pods = vec![
        input(format!("exposure value | f/{} | {tt}{}", fmt_float(n, 4), iso.map(|i| format!(" | ISO {}", fmt_float(i, 6))).unwrap_or_default())),
        primary(pod("Result", "Exposure value", text(f4(ev)))),
        pod("Table", "Values", table(None, rows)),
        pod("Formula", "Formula", text("EV = log₂(N²/t), N the f-number and t the exposure time in seconds; EV₁₀₀ = EV − log₂(ISO/100)")),
    ];
    if !assumed.is_empty() {
        pods.push(pod("Assumptions", "Assumed values", text(assumed.join(", "))));
    }
    Some(Answer::new("exposure value", pods))
}

fn spot_size(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:airy disk|diffraction|spot) (?:size|diameter)(?: (?:at|for))?\s*(?:f/\s*(\d+(?:\.\d+)?))?(?:,?\s*(?:λ|lambda|wavelength)\s*=?\s*(\d+(?:\.\d+)?)\s*nm)?$").captures(l)?;
    let n: f64 = m.get(1).map_or(Some(8.0), |x| x.as_str().parse().ok())?;
    let lam: f64 = m.get(2).map_or(Some(550.0), |x| x.as_str().parse().ok())?;
    let d = 2.44 * lam / 1000.0 * n;
    Some(Answer::new("diffraction spot size", vec![
        input(format!("Airy disk diameter | f/{} | wavelength {} nm", fmt_float(n, 4), fmt_float(lam, 5))),
        primary(pod("Result", "Airy disk diameter (to the first dark ring)", text(format!("{} μm", f4(d))))),
        pod("Formula", "Formula", text("d = 2.44 λ N")),
    ]))
}

fn numerical_aperture(l: &str) -> Option<Answer> {
    let rest = re!(r"^numerical aperture(.*)$").captures(l)?[1].to_string();
    let p = params(&rest);
    let d = get(&p, &["d", "diameter"]);
    let f = get(&p, &["f", "focal"]);
    let n = get(&p, &["n"]).unwrap_or(1.0);
    let mut pods = vec![input(format!("numerical aperture{}", if rest.trim().is_empty() { String::new() } else { format!(" |{rest}") }))];
    match (d, f) {
        (Some(d), Some(f)) if f > 0.0 => {
            let theta = (d / (2.0 * f)).atan();
            let na = n * theta.sin();
            pods.push(primary(pod("Result", "Numerical aperture", text(f4(na)))));
            pods.push(pod("Table", "Values", table(None, vec![vec![text("half-angle θ"), text(format!("{}°", f4(theta.to_degrees())))], vec![text("f-number f/D"), text(f4(f / d))], vec![text("NA ≈ D/(2f)"), text(f4(d / (2.0 * f)))]])));
        }
        _ => {
            pods.push(primary(pod("Result", "Definition", text("NA = n sin θ, with n the refractive index of the medium and θ the half-angle of the maximum cone of light"))));
            pods.push(pod("Table", "Relations", table(None, vec![
                vec![text("for a lens of diameter D and focal length f"), text("θ = arctan(D/(2f)), NA ≈ D/(2f) for small angles")],
                vec![text("f-number"), text("N ≈ 1/(2 NA)")],
                vec![text("resolution limit (Abbe)"), text("d = λ/(2 NA)")],
            ])));
            if d.is_some() {
                pods.push(pod("Note", "Note", text("give the focal length too (for example f=50mm) to compute the value")));
            }
        }
    }
    Some(Answer::new("numerical aperture", pods))
}

/// "8\"x10\" photograph", "5 by 7 photo vs 8 by 10 photo"
fn print_sizes(raw: &str) -> Option<Answer> {
    let l = raw.to_lowercase().replace(['”', '″'], "\"");
    let one = r#"(\d+(?:\.\d+)?)\s*(?:"|in|inch|inches)?\s*(?:x|×|by)\s*(\d+(?:\.\d+)?)\s*(?:"|in|inch|inches)?\s*(?:photo(?:graph)?|print|picture)s?"#;
    static ONE: OnceLock<Regex> = OnceLock::new();
    let r = ONE.get_or_init(|| Regex::new(one).unwrap());
    let all: Vec<(f64, f64)> = r.captures_iter(&l).filter_map(|m| Some((m[1].parse().ok()?, m[2].parse().ok()?))).collect();
    let rest = r.replace_all(&l, "");
    if all.is_empty() || all.len() > 4 || !re!(r"^[\s,]*(?:(?:vs\.?|versus|and|compared to)\s*)*$").is_match(&rest) {
        return None;
    }
    if all.iter().any(|(w, h)| *w <= 0.0 || *h <= 0.0 || *w > 100.0 || *h > 100.0) {
        return None;
    }
    let g = |a: f64, b: f64| -> f64 {
        let (mut a, mut b) = ((a * 100.0).round() as i64, (b * 100.0).round() as i64);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a as f64 / 100.0
    };
    let name = |(w, h): (f64, f64)| format!("{}\" × {}\"", fmt_float(w, 4), fmt_float(h, 4));
    let rows: Vec<Vec<Content>> = all.iter().map(|&(w, h)| {
        let k = g(w, h);
        vec![
            text(name((w, h))),
            text(format!("{} × {} cm", fmt_float((w * 2.54 * 10.0).round() / 10.0, 4), fmt_float((h * 2.54 * 10.0).round() / 10.0, 4))),
            text(format!("{}:{}", fmt_float(w / k, 4), fmt_float(h / k, 4))),
            text(format!("{} in²", fmt_float(w * h, 5))),
            text(format!("{} × {} px ({} MP)", fmt_float(w * 300.0, 6), fmt_float(h * 300.0, 6), fmt_float((w * h * 90_000.0 / 1e5).round() / 10.0, 4))),
        ]
    }).collect();
    let short = if all.len() == 2 {
        let r = all[1].0 * all[1].1 / (all[0].0 * all[0].1);
        format!("the {} print has {} times the area of the {} print", name(all[1]), fmt_float((r * 1000.0).round() / 1000.0, 4), name(all[0]))
    } else {
        let (w, h) = all[0];
        format!("{} × {} cm", fmt_float((w * 2.54 * 10.0).round() / 10.0, 4), fmt_float((h * 2.54 * 10.0).round() / 10.0, 4))
    };
    Some(Answer::new("print size", vec![
        input(all.iter().map(|&x| format!("{} print", name(x))).collect::<Vec<_>>().join(" | ")),
        primary(pod("Result", if all.len() == 2 { "Comparison" } else { "Metric size" }, text(short))),
        pod("Table", "Properties", table(Some(vec!["size", "metric", "aspect ratio", "area", "pixels at 300 ppi"]), rows)),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = re!(r"^(?:what is (?:the |an? )?|calculate (?:the )?|compute (?:the )?)").replace(l, "").to_string();
    let l = l.as_str();
    angle_of_view(l)
        .or_else(|| depth_of_field(l))
        .or_else(|| exposure_value(l))
        .or_else(|| spot_size(l))
        .or_else(|| numerical_aperture(l))
        .or_else(|| print_sizes(q.raw.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculators() {
        let a = angle_of_view("angle of view 50mm").unwrap();
        assert!(serde_json::to_string(&a.pods).unwrap().contains("46.7930°") || serde_json::to_string(&a.pods).unwrap().contains("46.793°"));
        let e = exposure_value("exposure value f/8 1/125").unwrap();
        assert!(serde_json::to_string(&e.pods).unwrap().contains("12.9658"));
    }
}
