//! School graphs and quick checks: points and scatter plots, histograms,
//! dot plots and pie charts, number lines (points or systems of
//! inequalities), comparing two functions, period and amplitude, equations
//! solved on an interval, the other trigonometric ratios from one, triangles
//! from vertices (centres, incircle, circumcircle), right triangles from two
//! sides, and "is 5 a solution to …".

use super::nl::{ex, main_var};
use super::*;
use crate::expr::Expr::*;
use crate::expr::*;
use crate::plot as pl;
use crate::print::text as ptext;
use crate::simplify as s;
use crate::solve;
use crate::stats::fmt;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn input(s: String) -> Pod {
    pod("Input", "Input interpretation", text(s))
}

fn kv(rows: Vec<(&str, Content)>) -> Content {
    table(None, rows.into_iter().map(|(a, b)| vec![text(a), b]).collect())
}

fn num(e: &Expr) -> Option<f64> {
    e.to_f64_const().filter(|v| v.is_finite())
}

fn simp(e: Expr) -> Expr {
    s::simplify(&e)
}

/// "(1, 2), (2, 0)" or "{{1,1},{3,6}}" → exact points.
fn points(t: &str) -> Option<Vec<(Expr, Expr)>> {
    let t = t.trim();
    let inner = if t.starts_with("{{") && t.ends_with("}}") { &t[1..t.len() - 1] } else { t };
    let items: Vec<_> = re!(r"[({]\s*([^(){},]+?)\s*,\s*([^(){},]+?)\s*[)}]").captures_iter(inner).collect();
    if items.is_empty() {
        return None;
    }
    let rest = re!(r"[({]\s*([^(){},]+?)\s*,\s*([^(){},]+?)\s*[)}]").replace_all(inner, "");
    if !rest.replace("and", "").chars().all(|c| c == ',' || c == ' ' || c == ';' || c == '&') {
        return None;
    }
    let mut out = Vec::new();
    for c in items {
        let (a, b) = (ex(&c[1])?, ex(&c[2])?);
        num(&a)?;
        num(&b)?;
        out.push((a, b));
    }
    Some(out)
}

fn fpts(p: &[(Expr, Expr)]) -> Vec<[f64; 2]> {
    p.iter().map(|(a, b)| [num(a).unwrap_or(0.0), num(b).unwrap_or(0.0)]).collect()
}

fn dist(a: &(Expr, Expr), b: &(Expr, Expr)) -> Expr {
    simp(sqrt(add(vec![square(sub(b.0.clone(), a.0.clone())), square(sub(b.1.clone(), a.1.clone()))])))
}

fn show_pt(p: &(Expr, Expr)) -> String {
    format!("({}, {})", ptext(&p.0), ptext(&p.1))
}

fn approx(e: &Expr) -> String {
    match num(e) {
        Some(v) if !matches!(e, Num(_)) => format!("{} ≈ {}", ptext(e), fmt(v)),
        _ => ptext(e),
    }
}

// ---------------------------------------------------------------------------
// triangles
// ---------------------------------------------------------------------------

fn triangle_answer(p: &[(Expr, Expr)], want: Option<&str>) -> Option<Answer> {
    let (a, b, c) = (&p[0], &p[1], &p[2]);
    // twice the signed area
    let area2 = simp(sub(mul(vec![sub(b.0.clone(), a.0.clone()), sub(c.1.clone(), a.1.clone())]), mul(vec![sub(c.0.clone(), a.0.clone()), sub(b.1.clone(), a.1.clone())])));
    if num(&area2)?.abs() < 1e-12 {
        return Some(Answer::new("points", vec![input(format!("points {}, {}, {}", show_pt(a), show_pt(b), show_pt(c))), primary(pod("Result", "Result", text("the points are collinear: they do not form a triangle")))]));
    }
    let area = simp(div(func("abs", vec![area2.clone()]), int(2)));
    // side lengths opposite each vertex
    let (la, lb, lc) = (dist(b, c), dist(c, a), dist(a, b));
    let per = simp(add(vec![la.clone(), lb.clone(), lc.clone()]));
    let centroid = (simp(div(add(vec![a.0.clone(), b.0.clone(), c.0.clone()]), int(3))), simp(div(add(vec![a.1.clone(), b.1.clone(), c.1.clone()]), int(3))));
    let incenter = (simp(div(add(vec![mul(vec![la.clone(), a.0.clone()]), mul(vec![lb.clone(), b.0.clone()]), mul(vec![lc.clone(), c.0.clone()])]), per.clone())), simp(div(add(vec![mul(vec![la.clone(), a.1.clone()]), mul(vec![lb.clone(), b.1.clone()]), mul(vec![lc.clone(), c.1.clone()])]), per.clone())));
    // circumcenter from perpendicular bisectors
    let d = simp(mul(vec![int(2), area2.clone()]));
    let sq = |q: &(Expr, Expr)| add(vec![square(q.0.clone()), square(q.1.clone())]);
    let ux = simp(div(add(vec![mul(vec![sq(a), sub(b.1.clone(), c.1.clone())]), mul(vec![sq(b), sub(c.1.clone(), a.1.clone())]), mul(vec![sq(c), sub(a.1.clone(), b.1.clone())])]), d.clone()));
    let uy = simp(div(add(vec![mul(vec![sq(a), sub(c.0.clone(), b.0.clone())]), mul(vec![sq(b), sub(a.0.clone(), c.0.clone())]), mul(vec![sq(c), sub(b.0.clone(), a.0.clone())])]), d));
    let circ = (ux, uy);
    let ortho = (simp(sub(add(vec![a.0.clone(), b.0.clone(), c.0.clone()]), mul(vec![int(2), circ.0.clone()]))), simp(sub(add(vec![a.1.clone(), b.1.clone(), c.1.clone()]), mul(vec![int(2), circ.1.clone()]))));
    let inr = simp(div(mul(vec![int(2), area.clone()]), per.clone()));
    let circr = dist(&circ, a);
    let label = format!("triangle with vertices {}, {}, {}", show_pt(a), show_pt(b), show_pt(c));
    let pt = |q: &(Expr, Expr)| -> String {
        let exact = show_pt(q);
        match (num(&q.0), num(&q.1)) {
            (Some(x), Some(y)) if exact.len() > 40 => format!("≈ ({}, {})", fmt(x), fmt(y)),
            (Some(x), Some(y)) if !matches!((&q.0, &q.1), (Num(_), Num(_))) => format!("{} ≈ ({}, {})", show_pt(q), fmt(x), fmt(y)),
            _ => show_pt(q),
        }
    };
    let rows = vec![
        ("side lengths", text(format!("{}, {}, {}", approx(&la), approx(&lb), approx(&lc)))),
        ("area", text(approx(&area))),
        ("perimeter", text(approx(&per))),
        ("centroid", text(pt(&centroid))),
        ("incenter", text(pt(&incenter))),
        ("inradius", text(approx(&inr))),
        ("circumcenter", text(pt(&circ))),
        ("circumradius", text(approx(&circr))),
        ("orthocenter", text(pt(&ortho))),
    ];
    let mut pods = vec![input(label)];
    let result = match want {
        Some("incenter") => Some(("Incenter", pt(&incenter))),
        Some("circumcenter") => Some(("Circumcenter", pt(&circ))),
        Some("centroid") => Some(("Centroid", pt(&centroid))),
        Some("orthocenter") => Some(("Orthocenter", pt(&ortho))),
        Some("area") => Some(("Area", approx(&area))),
        Some("perimeter") => Some(("Perimeter", approx(&per))),
        Some("incircle") => Some(("Inscribed circle", format!("center {}, radius {}", pt(&incenter), approx(&inr)))),
        Some("circumcircle") => Some(("Circumscribed circle", format!("center {}, radius {}", pt(&circ), approx(&circr)))),
        _ => None,
    };
    if let Some((t, v)) = result {
        pods.push(primary(pod("Result", t, text(v))));
        pods.push(pod("Properties", "Properties", kv(rows)));
    } else {
        pods.push(primary(pod("Properties", "Properties", kv(rows))));
    }
    let mut ring = fpts(p);
    ring.push(ring[0]);
    let mut plt = pl::scatter(fpts(p), "vertices", None);
    plt.series.push(pl::Series { label: "triangle".into(), segments: vec![ring], points: false });
    plt.equal_aspect = true;
    pods.push(pod("Plot", "Triangle", plot(plt)));
    Some(Answer::new("triangle", pods))
}

fn triangles(l: &str) -> Option<Answer> {
    let what = r"(incenter|incentre|circumcenter|circumcentre|centroid|orthocenter|orthocentre|area|perimeter|inscribed circle|incircle|circumscribed circle|circumcircle)";
    let r1 = {
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new(&format!(r"^(?:the\s+)?(?:{what}\s+of\s+)?(?:the\s+|a\s+)?triangle\s+(?:with\s+(?:vertices|corners)\s+|at\s+)?(.+?)(?:\s+{what})?$|^(.+?)\s+triangle(?:\s+{what})?$")).unwrap())
    };
    if let Some(m) = r1.captures(l) {
        let (pts_txt, want) = match (m.get(2), m.get(4)) {
            (Some(p), _) => (p.as_str(), m.get(1).or(m.get(3))),
            (None, Some(p)) => (p.as_str(), m.get(5)),
            _ => return None,
        };
        if let Some(p) = points(pts_txt).filter(|p| p.len() == 3) {
            let w = want.map(|w| match w.as_str() {
                "incentre" => "incenter",
                "circumcentre" => "circumcenter",
                "orthocentre" => "orthocenter",
                "inscribed circle" | "incircle" => "incircle",
                "circumscribed circle" | "circumcircle" => "circumcircle",
                x => x,
            });
            return triangle_answer(&p, w);
        }
    }
    // "3-4-5 triangle with inscribed circle"
    if let Some(m) = re!(r"^(?:a\s+)?(\d+(?:\.\d+)?)-(\d+(?:\.\d+)?)-(\d+(?:\.\d+)?)\s+triangle\s+(?:with\s+(?:an?\s+|its\s+)?)?(inscribed circle|incircle|circumscribed circle|circumcircle|inradius|circumradius)$").captures(l) {
        let (a, b, c): (f64, f64, f64) = (m[1].parse().ok()?, m[2].parse().ok()?, m[3].parse().ok()?);
        let s_ = (a + b + c) / 2.0;
        let area = (s_ * (s_ - a) * (s_ - b) * (s_ - c)).sqrt();
        if !(area > 0.0) {
            return None;
        }
        let r = area / s_;
        let big_r = a * b * c / (4.0 * area);
        let inner = m[4].contains("in");
        let tidy = |v: f64| ptext(&crate::calculus::limit::tidy_float(v));
        let rows = vec![("inradius", text(tidy(r))), ("circumradius", text(tidy(big_r))), ("area", text(tidy(area))), ("semiperimeter", text(tidy(s_)))];
        return Some(Answer::new("triangle", vec![input(format!("triangle with sides {}, {}, {}", fmt(a), fmt(b), fmt(c))), primary(pod("Result", if inner { "Inscribed circle radius" } else { "Circumscribed circle radius" }, text(tidy(if inner { r } else { big_r })))), pod("Properties", "Properties", kv(rows)), pod("Formula", "Formulas", text("r = area/s, R = abc/(4·area), with s the semiperimeter"))]));
    }
    // right triangles from two sides
    if let Some(m) = re!(r"^(?:the\s+)?(height|base|width|leg|hypotenuse|other leg|missing side)\s+of\s+(?:a\s+|the\s+)?(?:right\s+)?triangle\s+with\s+(?:a\s+)?(height|base|width|leg|hypotenuse)\s+(?:of\s+)?([\d.]+)\s+and\s+(?:a\s+)?(height|base|width|leg|hypotenuse)\s+(?:of\s+)?([\d.]+)$").captures(l) {
        let (k1, v1, k2, v2) = (&m[2], m[3].parse::<f64>().ok()?, &m[4], m[5].parse::<f64>().ok()?);
        let want_h = &m[1] == "hypotenuse";
        let (leg, hyp) = if k1 == "hypotenuse" { (v2, v1) } else if k2 == "hypotenuse" { (v1, v2) } else { (v1, v2) };
        let res = if want_h || (k1 != "hypotenuse" && k2 != "hypotenuse") { (v1 * v1 + v2 * v2).sqrt() } else { (hyp * hyp - leg * leg).sqrt() };
        if !res.is_finite() || res <= 0.0 {
            return None;
        }
        let e = simp(if want_h || (k1 != "hypotenuse" && k2 != "hypotenuse") { sqrt(add(vec![square(ex(&m[3])?), square(ex(&m[5])?)])) } else { sqrt(sub(square(ex(&if k1 == "hypotenuse" { m[3].to_string() } else { m[5].to_string() })?), square(ex(&if k1 == "hypotenuse" { m[5].to_string() } else { m[3].to_string() })?))) });
        return Some(Answer::new("right triangle", vec![input(format!("{} of a right triangle with {k1} {} and {k2} {}", &m[1], &m[3], &m[5])), primary(pod("Result", "Result", text(approx(&e)))), pod("Formula", "Pythagorean theorem", text("a² + b² = c², with c the hypotenuse"))]));
    }
    if let Some(m) = re!(r"^(?:a\s+)?right triangle with legs\s+([\d.]+)\s+and\s+([\d.]+)$").captures(l) {
        let (a, b) = (ex(&m[1])?, ex(&m[2])?);
        let c = simp(sqrt(add(vec![square(a.clone()), square(b.clone())])));
        let (av, bv) = (num(&a)?, num(&b)?);
        let rows = vec![("hypotenuse", text(approx(&c))), ("area", text(approx(&simp(div(mul(vec![a.clone(), b.clone()]), int(2)))))), ("perimeter", text(approx(&simp(add(vec![a, b, c.clone()]))))), ("angles", text(format!("{}°, {}°, 90°", fmt(bv.atan2(av).to_degrees()), fmt(av.atan2(bv).to_degrees()))))];
        return Some(Answer::new("right triangle", vec![input(format!("right triangle with legs {} and {}", &m[1], &m[2])), primary(pod("Result", "Properties", kv(rows)))]));
    }
    None
}

// ---------------------------------------------------------------------------
// points, scatter plots, histograms
// ---------------------------------------------------------------------------

fn point_plots(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(scatter\s*plot|scatter|plot points|plot|graph|points|draw)\s+(?:of\s+|the\s+points?\s+)?)?(.+)$").captures(l)?;
    let cmd = m.get(1).map(|x| x.as_str()).unwrap_or("");
    // bare "{{1,2},{3,4}}" is a matrix; points need parentheses or a command
    if cmd.is_empty() && m[2].contains('{') {
        return None;
    }
    let pts = points(&m[2])?;
    let fp = fpts(&pts);
    let scatter = cmd.starts_with("scatter") || pts.len() > 3;
    let list = pts.iter().map(show_pt).collect::<Vec<_>>().join(", ");
    let mut pods = vec![input(format!("{} {list}", if scatter { "scatter plot of" } else { "points" }))];
    let mut plt = pl::scatter(fp.clone(), "points", None);
    plt.equal_aspect = !scatter;
    match pts.len() {
        1 => {
            let (x, y) = (fp[0][0], fp[0][1]);
            let quad = match (x.partial_cmp(&0.0)?, y.partial_cmp(&0.0)?) {
                (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => "quadrant I",
                (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => "quadrant II",
                (std::cmp::Ordering::Less, std::cmp::Ordering::Less) => "quadrant III",
                (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => "quadrant IV",
                _ => "on an axis",
            };
            let r = simp(sqrt(add(vec![square(pts[0].0.clone()), square(pts[0].1.clone())])));
            pods.push(pod("Plot", "Plot", plot(plt)));
            pods.push(primary(pod("Properties", "Properties", kv(vec![("location", text(quad)), ("distance from the origin", text(approx(&r))), ("polar angle", text(format!("{}°", fmt(y.atan2(x).to_degrees()))))]))));
        }
        2 => {
            let (a, b) = (&pts[0], &pts[1]);
            let d = dist(a, b);
            let mid = (simp(div(add(vec![a.0.clone(), b.0.clone()]), int(2))), simp(div(add(vec![a.1.clone(), b.1.clone()]), int(2))));
            let dx = simp(sub(b.0.clone(), a.0.clone()));
            let line = if num(&dx)? == 0.0 {
                format!("x = {}", ptext(&a.0))
            } else {
                let mslope = simp(div(sub(b.1.clone(), a.1.clone()), dx));
                let c = simp(sub(a.1.clone(), mul(vec![mslope.clone(), a.0.clone()])));
                format!("y = {} (slope {})", ptext(&simp(add(vec![mul(vec![mslope.clone(), sym("x")]), c]))), ptext(&mslope))
            };
            let mut segp = fp.clone();
            segp.truncate(2);
            plt.series.push(pl::Series { label: "segment".into(), segments: vec![segp], points: false });
            pods.push(pod("Plot", "Plot", plot(plt)));
            pods.push(primary(pod("Properties", "Properties", kv(vec![("distance", text(approx(&d))), ("midpoint", text(show_pt(&mid))), ("line through the points", text(line))]))));
        }
        3 if !scatter => return triangle_answer(&pts, None).map(|mut a| {
            a.pods.insert(1, pod("Note", "Note", text("three points: shown as a triangle")));
            a
        }),
        _ => {
            let n = fp.len() as f64;
            let (mx, my) = (fp.iter().map(|p| p[0]).sum::<f64>() / n, fp.iter().map(|p| p[1]).sum::<f64>() / n);
            let sxy: f64 = fp.iter().map(|p| (p[0] - mx) * (p[1] - my)).sum();
            let sxx: f64 = fp.iter().map(|p| (p[0] - mx).powi(2)).sum();
            let syy: f64 = fp.iter().map(|p| (p[1] - my).powi(2)).sum();
            let r = sxy / (sxx * syy).sqrt();
            pods.push(primary(pod("Plot", "Scatter plot", plot(plt))));
            if sxx > 0.0 {
                let slope = sxy / sxx;
                let model = add(vec![mul(vec![float(slope), sym("x")]), float(my - slope * mx)]);
                pods.push(pod("Fit", "Least-squares line", math(&simp(model))));
                pods.push(pod("Correlation", "Correlation coefficient", text(format!("r = {}", fmt(r)))));
            }
        }
    }
    Some(Answer::new("points", pods))
}

fn data_list(t: &str) -> Option<Vec<f64>> {
    let t = t.trim().trim_start_matches(['{', '[', '(']).trim_end_matches(['}', ']', ')']);
    let v: Vec<f64> = t.split(|c| c == ',' || c == ' ' || c == ';').filter(|w| !w.is_empty()).map(|w| w.parse::<f64>().ok()).collect::<Option<Vec<_>>>()?;
    (v.len() >= 2).then_some(v)
}

fn charts(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(histogram|dot\s*plot|line\s*plot|pie\s*chart|bar\s*(?:chart|graph))\s+(?:of\s+)?(.+)$").captures(l) {
        let v = data_list(&m[2])?;
        let kind = m[1].replace(' ', "");
        if kind == "histogram" {
            let n = v.len();
            let (lo, hi) = (v.iter().cloned().fold(f64::INFINITY, f64::min), v.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
            let bins = ((n as f64).log2().ceil() as usize + 1).clamp(2, 20);
            let ticks = pl::ticks(lo, hi, bins);
            let w = if ticks.len() >= 2 { ticks[1] - ticks[0] } else { ((hi - lo) / bins as f64).max(1.0) };
            let start = (lo / w).floor() * w;
            let nb = (((hi - start) / w).floor() as usize + 1).max(1);
            let mut counts = vec![0usize; nb];
            for x in &v {
                counts[(((x - start) / w).floor() as usize).min(nb - 1)] += 1;
            }
            let bars: Vec<(String, f64)> = counts.iter().enumerate().map(|(i, c)| (format!("{}–{}", fmt(start + i as f64 * w), fmt(start + (i + 1) as f64 * w)), *c as f64)).collect();
            let rows: Vec<Vec<Content>> = bars.iter().map(|(b, c)| vec![text(b.clone()), text(fmt(*c))]).collect();
            return Some(Answer::new("histogram", vec![input(format!("histogram of {} values", n)), primary(pod("Plot", "Histogram", plot(pl::bars(bars)))), pod("Bins", &format!("Bins of width {}", fmt(w)), table(Some(vec!["bin", "count"]), rows))]));
        }
        if kind == "piechart" {
            let total: f64 = v.iter().sum();
            let rows: Vec<Vec<Content>> = v.iter().map(|x| vec![text(fmt(*x)), text(format!("{}%", fmt((x / total * 1000.0).round() / 10.0))), text(format!("{}°", fmt((x / total * 3600.0).round() / 10.0)))]).collect();
            return Some(Answer::new("pie chart", vec![input("pie chart".into()), primary(pod("Result", "Shares", table(Some(vec!["value", "share", "sector angle"]), rows))), pod("Plot", "Bar chart of the shares", plot(pl::bars(v.iter().map(|x| (fmt(*x), *x)).collect())))]));
        }
        // dot plot / line plot / bar chart: counts of each value, or the values in order
        if kind.starts_with("bar") {
            return Some(Answer::new("bar chart", vec![input("bar chart".into()), primary(pod("Plot", "Bar chart", plot(pl::bars(v.iter().enumerate().map(|(i, x)| ((i + 1).to_string(), *x)).collect()))))]));
        }
        let mut uniq: Vec<f64> = v.clone();
        uniq.sort_by(|a, b| a.partial_cmp(b).unwrap());
        uniq.dedup();
        let counts: Vec<(String, f64)> = uniq.iter().map(|u| (fmt(*u), v.iter().filter(|x| *x == u).count() as f64)).collect();
        let rows: Vec<Vec<Content>> = counts.iter().map(|(u, c)| vec![text(u.clone()), text("•".repeat(*c as usize))]).collect();
        return Some(Answer::new("dot plot", vec![input(format!("dot plot of {} values", v.len())), primary(pod("Result", "Dot plot", table(Some(vec!["value", "count"]), rows))), pod("Plot", "Counts", plot(pl::bars(counts)))]));
    }
    // "1/3 pie chart"
    if let Some(m) = re!(r"^(\d+)\s*/\s*(\d+)\s+(?:of\s+a\s+)?pie(?:\s*chart)?$").captures(l) {
        let (a, b): (f64, f64) = (m[1].parse().ok()?, m[2].parse().ok()?);
        if b == 0.0 || a > b {
            return None;
        }
        let rows = vec![("share", text(format!("{}%", fmt((a / b * 1000.0).round() / 10.0)))), ("sector angle", text(format!("{}°", fmt((a / b * 3600.0).round() / 10.0)))), ("remaining", text(format!("{}/{} ({}°)", fmt(b - a), fmt(b), fmt(((b - a) / b * 3600.0).round() / 10.0))))];
        return Some(Answer::new("pie chart", vec![input(format!("{}/{} of a pie chart", &m[1], &m[2])), primary(pod("Result", "Sector", kv(rows))), pod("Plot", "Shares", plot(pl::bars(vec![(format!("{}/{}", &m[1], &m[2]), a / b), ("rest".into(), 1.0 - a / b)])))]));
    }
    None
}

// ---------------------------------------------------------------------------
// number lines
// ---------------------------------------------------------------------------

fn intersect(a: &[pl::Span], b: &[pl::Span]) -> Vec<pl::Span> {
    let mut out = Vec::new();
    for x in a {
        for y in b {
            let (lo, lo_closed) = if x.lo > y.lo { (x.lo, x.lo_closed) } else if y.lo > x.lo { (y.lo, y.lo_closed) } else { (x.lo, x.lo_closed && y.lo_closed) };
            let (hi, hi_closed) = if x.hi < y.hi { (x.hi, x.hi_closed) } else if y.hi < x.hi { (y.hi, y.hi_closed) } else { (x.hi, x.hi_closed && y.hi_closed) };
            if lo < hi || (lo == hi && lo_closed && hi_closed) {
                out.push(pl::Span { lo, hi, lo_closed, hi_closed });
            }
        }
    }
    out
}

fn span_text(sp: &pl::Span, x: &str) -> String {
    let f = |v: f64| ptext(&crate::calculus::limit::tidy_float(v));
    match (sp.lo.is_finite(), sp.hi.is_finite()) {
        (true, true) if sp.lo == sp.hi => format!("{x} = {}", f(sp.lo)),
        (true, true) => format!("{} {} {x} {} {}", f(sp.lo), if sp.lo_closed { "≤" } else { "<" }, if sp.hi_closed { "≤" } else { "<" }, f(sp.hi)),
        (true, false) => format!("{x} {} {}", if sp.lo_closed { "≥" } else { ">" }, f(sp.lo)),
        (false, true) => format!("{x} {} {}", if sp.hi_closed { "≤" } else { "<" }, f(sp.hi)),
        _ => "all real numbers".into(),
    }
}

fn number_lines(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:show\s+|plot\s+|graph\s+|draw\s+)?(?:(.+?)\s+on\s+(?:a|the)\s+number\s*line|number\s*line\s+(?:of\s+|for\s+|with\s+)?(.+))$").captures(l)?;
    let body = m.get(1).or(m.get(2))?.as_str().trim();
    if body.contains(['<', '>', '≤', '≥']) {
        let mut parts: Vec<String> = Vec::new();
        for p in re!(r"\s*(?:,|\band\b|&&)\s*").split(body).filter(|p| !p.trim().is_empty()) {
            // split chains "0 <= x < 3" into two inequalities
            match re!(r"^(.+?)\s*(<=|<|>=|>|≤|≥)\s*([a-z])\s*(<=|<|>=|>|≤|≥)\s*(.+)$").captures(p) {
                Some(c) => {
                    parts.push(format!("{} {} {}", &c[1], &c[2], &c[3]));
                    parts.push(format!("{} {} {}", &c[3], &c[4], &c[5]));
                }
                None => parts.push(p.to_string()),
            }
        }
        let mut exact: Option<String> = None;
        let mut spans: Option<Vec<pl::Span>> = None;
        let mut var = sym("x");
        for p in &parts {
            let e = ex(p)?;
            var = main_var(&e);
            let sol = match &e {
                Rel(op, a, b) => solve::solve_inequality(*op, a, b, &var)?,
                Func(n, v) if &**n == "And" => {
                    // chained "0 <= x < 3"
                    let mut acc: Option<Vec<pl::Span>> = None;
                    for r in v {
                        let Rel(op, a, b) = r else { return None };
                        let s1 = super::math::spans_of(&solve::solve_inequality(*op, a, b, &var)?, &var)?;
                        acc = Some(match acc {
                            None => s1,
                            Some(prev) => intersect(&prev, &s1),
                        });
                    }
                    let sp = acc?;
                    spans = Some(match spans {
                        None => sp,
                        Some(prev) => intersect(&prev, &sp),
                    });
                    continue;
                }
                _ => return None,
            };
            if parts.len() == 1 {
                exact = Some(ptext(&sol));
            }
            let sp = super::math::spans_of(&sol, &var)?;
            spans = Some(match spans {
                None => sp,
                Some(prev) => intersect(&prev, &sp),
            });
        }
        let sp = spans?;
        let v = ptext(&var);
        let res = exact.unwrap_or_else(|| if sp.is_empty() { "no solutions".to_string() } else { sp.iter().map(|s| span_text(s, &v)).collect::<Vec<_>>().join(" or ") });
        return Some(Answer::new("number line", vec![input(format!("{} on a number line", parts.join(" and "))), primary(pod("Result", "Solution", text(res))), pod("NumberLine", "Number line", plot(pl::number_line(sp)))]));
    }
    let items: Vec<&str> = re!(r"\s*(?:,|\band\b)\s*").split(body).filter(|p| !p.trim().is_empty()).collect();
    let mut vals = Vec::new();
    for it in &items {
        let e = ex(it)?;
        vals.push((e.clone(), num(&e)?));
    }
    if vals.is_empty() {
        return None;
    }
    let spans: Vec<pl::Span> = vals.iter().map(|(_, v)| pl::Span { lo: *v, hi: *v, lo_closed: true, hi_closed: true }).collect();
    let mut sorted = vals.clone();
    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let order = sorted.iter().map(|(e, v)| if matches!(e, Num(_)) { ptext(e) } else { format!("{} ≈ {}", ptext(e), fmt(*v)) }).collect::<Vec<_>>().join(" < ");
    let mut pods = vec![input(format!("{} on a number line", items.join(", "))), primary(pod("NumberLine", "Number line", plot(pl::number_line(spans))))];
    if vals.len() > 1 {
        pods.push(pod("Order", "Order", text(order)));
    } else {
        pods.push(pod("Value", "Value", text(format!("{} ≈ {}", ptext(&vals[0].0), fmt(vals[0].1)))));
    }
    Some(Answer::new("number line", pods))
}

// ---------------------------------------------------------------------------
// functions: compare, period, amplitude
// ---------------------------------------------------------------------------

fn pure_math(t: &str) -> bool {
    const OK: &[&str] = &["x", "t", "sin", "cos", "tan", "sec", "csc", "cot", "log", "ln", "exp", "sqrt", "abs", "e", "pi", "sinh", "cosh", "tanh", "arcsin", "arccos", "arctan", "asin", "acos", "atan"];
    !t.trim().is_empty() && re!(r"^[0-9a-z^*/+\-(). ]+$").is_match(t) && re!(r"[a-z]+").find_iter(t).all(|w| OK.contains(&w.as_str())) && t.contains('x')
}

fn compare_fns(l: &str) -> Option<Answer> {
    let m = re!(r"^compare\s+(.+?)\s+(?:and|with|to|vs\.?|versus)\s+(.+)$|^(.+?)\s+(?:vs\.?|versus)\s+(.+)$").captures(l)?;
    let (a, b) = (m.get(1).or(m.get(3))?.as_str(), m.get(2).or(m.get(4))?.as_str());
    if !pure_math(a) || !pure_math(b) {
        return None;
    }
    let (f, g) = (ex(a)?, ex(b)?);
    let x = sym("x");
    let d = simp(sub(f.clone(), g.clone()));
    let roots = solve::solve(&eq(d.clone(), zero()), &x).real();
    let mut pods = vec![input(format!("compare {} and {}", ptext(&f), ptext(&g)))];
    if let Some(p) = pl::cartesian(&[f.clone(), g.clone()], &x, None) {
        pods.push(primary(pod("Plot", "Plots", plot(p))));
    }
    let finite: Vec<(Expr, f64)> = roots.iter().filter(|r| !r.has_sym(&solve::param())).filter_map(|r| Some((r.clone(), num(r)?))).collect();
    let mut sorted = finite.clone();
    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    if !roots.is_empty() {
        pods.push(pod("Intersections", "Intersection points", text(roots.iter().map(|r| format!("x = {}", approx(r))).collect::<Vec<_>>().join(", "))));
    } else {
        pods.push(pod("Intersections", "Intersection points", text("none (real)")));
    }
    if roots.len() == finite.len() {
        // sign of f − g between the crossings
        let Sym(xn) = &x else { return None };
        let pts: Vec<f64> = sorted.iter().map(|r| r.1).collect();
        let mut probes = Vec::new();
        if pts.is_empty() {
            probes.push(0.0);
        } else {
            probes.push(pts[0] - 1.0);
            for w in pts.windows(2) {
                probes.push((w[0] + w[1]) / 2.0);
            }
            probes.push(pts[pts.len() - 1] + 1.0);
        }
        let mut rows = Vec::new();
        for (i, pr) in probes.iter().enumerate() {
            let v = crate::eval::eval_f64(&d, &[(&**xn, *pr)])?;
            let lo = if i == 0 { "−∞".to_string() } else { approx(&sorted[i - 1].0) };
            let hi = if i == pts.len() { "∞".to_string() } else { approx(&sorted[i].0) };
            let bigger = if v > 0.0 { ptext(&f) } else { ptext(&g) };
            rows.push(vec![text(format!("{lo} < x < {hi}")), text(format!("{bigger} is larger"))]);
        }
        pods.push(pod("Comparison", "Which is larger", table(None, rows)));
    }
    pods.push(pod("Difference", "Difference f − g", math(&d)));
    Some(Answer::new("comparison", pods))
}

/// Periods of the trigonometric pieces, as multiples of π (rational).
fn trig_periods(e: &Expr, x: &Expr, out: &mut Vec<Expr>) -> bool {
    match e {
        Func(n, a) if a.len() == 1 && matches!(&**n, "sin" | "cos" | "tan" | "sec" | "csc" | "cot") => {
            let Some(cs) = crate::poly::coeffs(&a[0], x) else { return false };
            if cs.len() != 2 {
                return !a[0].has_sym(x);
            }
            let base = if matches!(&**n, "tan" | "cot") { Const(K::Pi) } else { mul(vec![int(2), Const(K::Pi)]) };
            out.push(simp(div(base, func("abs", vec![cs[1].clone()]))));
            true
        }
        Func(_, a) => a.iter().all(|t| !t.has_sym(x) || trig_periods(t, x, out)),
        Add(v) | Mul(v) => v.iter().all(|t| trig_periods(t, x, out)),
        Pow(a, b) => !b.has_sym(x) && trig_periods(a, x, out),
        _ => true,
    }
}

fn period_of(f: &Expr, x: &Expr) -> Option<Expr> {
    let mut ps = Vec::new();
    if !trig_periods(f, x, &mut ps) || ps.is_empty() {
        return None;
    }
    // all periods p = (a/b) π: lcm of the rationals
    let pi = Const(K::Pi);
    let mut rats = Vec::new();
    for p in &ps {
        let r = simp(div(p.clone(), pi.clone()));
        let Num(q) = r else { return None };
        rats.push(q);
    }
    let lcm = rats.iter().skip(1).fold(rats[0].clone(), |acc, q| {
        use num_integer::Integer;
        num_rational::BigRational::new(acc.numer().lcm(q.numer()), acc.denom().gcd(q.denom()))
    });
    let mut per = simp(mul(vec![Num(lcm), pi.clone()]));
    // a smaller true period (e.g. sin² has period π)
    let Sym(xn) = x else { return None };
    let pv = num(&per)?;
    for k in [6, 4, 3, 2] {
        let cand = pv / k as f64;
        let ok = (0..12).all(|i| {
            let t = 0.37 + i as f64 * 0.91;
            match (crate::eval::eval_f64(f, &[(&**xn, t)]), crate::eval::eval_f64(f, &[(&**xn, t + cand)])) {
                (Some(a), Some(b)) => (a - b).abs() < 1e-9 * (1.0 + a.abs()),
                (None, None) => true,
                _ => false,
            }
        });
        if ok {
            per = simp(div(per, int(k)));
            break;
        }
    }
    Some(per)
}

fn fn_props(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+|what is the\s+|find the\s+)?(period|amplitude|frequency|midline|vertical shift|phase shift)\s+(?:of\s+)?(?:y\s*=\s*|f\(x\)\s*=\s*)?(.+)$").captures(l)?;
    let what = &m[1];
    let f = ex(&m[2])?;
    let x = main_var(&f);
    if f.free_symbols().len() != 1 {
        return None;
    }
    let per = period_of(&f, &x);
    // sinusoid A sin(Bx + C) + D
    let mut d = zero();
    let mut core = f.clone();
    if let Add(v) = &f {
        let (c, rest): (Vec<Expr>, Vec<Expr>) = v.iter().cloned().partition(|t| !t.has_sym(&x));
        d = simp(add(c));
        core = simp(add(rest));
    }
    let (amp, inner) = match &core {
        Mul(v) => {
            let (c, rest): (Vec<Expr>, Vec<Expr>) = v.iter().cloned().partition(|t| !t.has_sym(&x));
            (simp(func("abs", vec![mul(c)])), simp(mul(rest)))
        }
        other => (one(), other.clone()),
    };
    let sinusoid = matches!(&inner, Func(n, a) if (&**n == "sin" || &**n == "cos") && crate::poly::coeffs(&a[0], &x).map_or(false, |c| c.len() == 2));
    let res: Expr = match what {
        "period" => per.clone()?,
        "frequency" => simp(div(one(), per.clone()?)),
        "amplitude" if sinusoid => amp.clone(),
        "midline" | "vertical shift" if sinusoid => eq(sym("y"), d.clone()),
        "phase shift" if sinusoid => {
            let Func(_, a) = &inner else { return None };
            let cs = crate::poly::coeffs(&a[0], &x)?;
            simp(neg(div(cs[0].clone(), cs[1].clone())))
        }
        _ => return None,
    };
    let mut pods = vec![input(format!("{what} of {}", ptext(&f))), primary(pod("Result", "Result", text(approx(&res))))];
    if sinusoid {
        let mut rows = vec![("amplitude", text(ptext(&amp))), ("midline", text(format!("y = {}", ptext(&d))))];
        if let Some(p) = &per {
            rows.push(("period", text(ptext(p))));
        }
        pods.push(pod("Properties", "Sinusoid", kv(rows)));
    }
    if let Some(p) = pl::cartesian(&[f.clone()], &x, None) {
        pods.push(pod("Plot", "Plot", plot(p)));
    }
    Some(Answer::new("function", pods))
}

// ---------------------------------------------------------------------------
// solving on an interval; checking a solution
// ---------------------------------------------------------------------------

fn interval(t: &str, var: &str) -> Option<(f64, f64, bool, bool)> {
    let t = t.trim().trim_start_matches(&format!("{var} in ")).trim();
    if let Some(m) = re!(r"^([\[(])\s*(.+?)\s*,\s*(.+?)\s*([\])])$").captures(t) {
        return Some((num(&ex(&m[2])?)?, num(&ex(&m[3])?)?, &m[1] == "[", &m[4] == "]"));
    }
    if let Some(m) = re!(r"^(.+?)\s*(<=|≤|<)\s*([a-z])\s*(<=|≤|<)\s*(.+)$").captures(t) {
        return Some((num(&ex(&m[1])?)?, num(&ex(&m[5])?)?, &m[2] != "<", &m[4] != "<"));
    }
    if let Some(m) = re!(r"^(?:between\s+|from\s+)?(.+?)\s+(?:and|to)\s+(.+)$").captures(t) {
        return Some((num(&ex(&m[1])?)?, num(&ex(&m[2])?)?, true, true));
    }
    None
}

fn solve_interval(l: &str) -> Option<Answer> {
    let m = re!(r"^solve\s+(.+?=.+?)\s*(?:,\s*|\s+)(?:on|for|in|over|where|with|when)\s+(.+)$").captures(l)?;
    let e = ex(&m[1])?;
    let Rel(RelOp::Eq, a, b) = &e else { return None };
    let x = main_var(&e);
    let Sym(xn) = &x else { return None };
    let (lo, hi, lc, hc) = interval(&m[2], xn)?;
    let roots = solve::solve(&eq((**a).clone(), (**b).clone()), &x).real();
    let mut found: Vec<(Expr, f64)> = Vec::new();
    let pn = solve::param();
    for r in roots {
        let range: Vec<i64> = if r.has_sym(&pn) { (-60..=60).collect() } else { vec![0] };
        for k in range {
            let e2 = simp(r.subs(&pn, &int(k)));
            let Some(v) = num(&e2) else { continue };
            let inside = (v > lo || (lc && (v - lo).abs() < 1e-12)) && (v < hi || (hc && (v - hi).abs() < 1e-12));
            if inside && !found.iter().any(|(_, w)| (w - v).abs() < 1e-10) {
                found.push((e2, v));
            }
        }
    }
    found.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let res = if found.is_empty() { "no solutions in the interval".into() } else { found.iter().map(|(e, _)| format!("{xn} = {}", approx(e))).collect::<Vec<_>>().join(", ") };
    Some(Answer::new("solve", vec![input(format!("solve {} for {} in the interval {}{}, {}{}", ptext(&e), xn, if lc { "[" } else { "(" }, fmt(lo), fmt(hi), if hc { "]" } else { ")" })), primary(pod("Result", "Solutions", text(res)))]))
}

fn check_solution(l: &str) -> Option<Answer> {
    let m = re!(r"^is\s+(?:([a-z])\s*=\s*)?(-?[\d./]+)\s+(?:a\s+)?(?:solution|root)\s+(?:to|of|for)\s+(?:the\s+)?(?:equation\s+|inequality\s+)?(.+)$").captures(l)?;
    let e = ex(&m[3])?;
    let x = m.get(1).map(|v| sym(v.as_str())).unwrap_or_else(|| main_var(&e));
    let v = ex(&m[2])?;
    let Rel(op, a, b) = &e else { return None };
    let (av, bv) = (simp(a.subs(&x, &v)), simp(b.subs(&x, &v)));
    let (an, bn) = (num(&av)?, num(&bv)?);
    let tol = 1e-12 * (1.0 + an.abs() + bn.abs());
    let ok = match op {
        RelOp::Eq => (an - bn).abs() < tol,
        RelOp::Ne => (an - bn).abs() >= tol,
        RelOp::Lt => an < bn - tol,
        RelOp::Le => an <= bn + tol,
        RelOp::Gt => an > bn + tol,
        RelOp::Ge => an >= bn - tol,
    };
    let sym_op = match op {
        RelOp::Eq => "=",
        RelOp::Ne => "≠",
        RelOp::Lt => "<",
        RelOp::Le => "≤",
        RelOp::Gt => ">",
        RelOp::Ge => "≥",
    };
    Some(Answer::new("check", vec![input(format!("is {} = {} a solution of {}", ptext(&x), ptext(&v), ptext(&e))), primary(pod("Result", "Result", text(if ok { "yes" } else { "no" }))), pod("Check", "Substitution", text(format!("{} {sym_op} {} is {}", ptext(&av), ptext(&bv), if ok { "true" } else { "false" })))]))
}

/// "if sin(a) = 3/5, what is cos(a)?"
fn trig_from_one(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:if|given|suppose)\s+(sin|cos|tan|sec|csc|cot)\s*\(?\s*([a-zθα-ω])\s*\)?\s*=\s*(.+?),?\s+(?:what is|find|then|what's)\s+(sin|cos|tan|sec|csc|cot)\s*\(?\s*([a-zθα-ω])\s*\)?$").captures(l)?;
    if m[2] != m[5] {
        return None;
    }
    let v = ex(&m[3])?;
    let vf = num(&v)?;
    let one_ = one();
    // sin and cos of an angle with the given ratio (signs fixed by the quadrant)
    let (s_, c_) = match &m[1] {
        "sin" if vf.abs() <= 1.0 => (v.clone(), sqrt(sub(one_.clone(), square(v.clone())))),
        "cos" if vf.abs() <= 1.0 => (sqrt(sub(one_.clone(), square(v.clone()))), v.clone()),
        "csc" if vf.abs() >= 1.0 => {
            let sv = div(one_.clone(), v.clone());
            (sv.clone(), sqrt(sub(one_.clone(), square(sv))))
        }
        "sec" if vf.abs() >= 1.0 => {
            let cv = div(one_.clone(), v.clone());
            (sqrt(sub(one_.clone(), square(cv.clone()))), cv)
        }
        "tan" => (div(v.clone(), sqrt(add(vec![one_.clone(), square(v.clone())]))), div(one_.clone(), sqrt(add(vec![one_.clone(), square(v.clone())])))),
        "cot" => (div(one_.clone(), sqrt(add(vec![one_.clone(), square(v.clone())]))), div(v.clone(), sqrt(add(vec![one_.clone(), square(v.clone())])))),
        _ => return Some(Answer::new("trig", vec![input(format!("{}({}) = {}", &m[1], &m[2], ptext(&v))), primary(pod("Result", "Result", text("no real angle has this value")))])),
    };
    let (sv, cv) = (simp(s_), simp(c_));
    let res = match &m[4] {
        "sin" => sv.clone(),
        "cos" => cv.clone(),
        "tan" => simp(div(sv.clone(), cv.clone())),
        "cot" => simp(div(cv.clone(), sv.clone())),
        "sec" => simp(div(one(), cv.clone())),
        _ => simp(div(one(), sv.clone())),
    };
    let same_family = matches!((&m[1][..], &m[4][..]), ("tan", "cot") | ("cot", "tan") | ("sin", "csc") | ("csc", "sin") | ("cos", "sec") | ("sec", "cos"));
    let amb = !same_family && m[1] != m[4];
    let shown = if amb { format!("±{}", ptext(&res)) } else { ptext(&res) };
    let note = if amb { "the sign depends on the quadrant of the angle: the value shown with + is for the quadrant where the given ratio and this one share a sign" } else { "follows from the Pythagorean identity sin² + cos² = 1" };
    Some(Answer::new("trig", vec![input(format!("{}({}) given {}({}) = {}", &m[4], &m[2], &m[1], &m[2], ptext(&v))), primary(pod("Result", "Result", text(shown))), pod("Note", "Note", text(note))]))
}


// ---------------------------------------------------------------------------
// calculus odds and ends: implicit tangents, sums, function continued fractions
// ---------------------------------------------------------------------------

fn implicit_line(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(tangent|normal)(?:\s+line)?\s+(?:to|of)\s+(?:the\s+curve\s+)?(.+?=.+?)\s+at\s+(?:the\s+point\s+)?(?:\(?\s*x\s*=\s*(.+?)\s*,\s*y\s*=\s*(.+?)\s*\)?|\(\s*(.+?)\s*,\s*(.+?)\s*\))$").captures(l)?;
    let e = ex(&m[2])?;
    let Rel(RelOp::Eq, a, b) = &e else { return None };
    let f = sub((**a).clone(), (**b).clone());
    let (x, y) = (sym("x"), sym("y"));
    if !f.has_sym(&y) {
        return None;
    }
    let px = ex(m.get(3).or(m.get(5))?.as_str())?;
    let py = ex(m.get(4).or(m.get(6))?.as_str())?;
    let at = |g: &Expr| simp(g.subs(&x, &px).subs(&y, &py));
    if num(&at(&f))?.abs() > 1e-9 {
        return Some(Answer::new("tangent", vec![input(format!("{} line to {} at ({}, {})", &m[1], ptext(&e), ptext(&px), ptext(&py))), primary(pod("Result", "Result", text("the point is not on the curve")))]));
    }
    let fx = at(&crate::calculus::diff::diff(&f, &x));
    let fy = at(&crate::calculus::diff::diff(&f, &y));
    let tangent = &m[1] == "tangent";
    // direction of the line: tangent ⟂ ∇F, normal ∥ ∇F
    let (dx, dy) = if tangent { (fy.clone(), neg(fx.clone())) } else { (fx.clone(), fy.clone()) };
    let line = if num(&dx)?.abs() < 1e-15 {
        eq(x.clone(), px.clone())
    } else {
        let slope = simp(div(dy, dx));
        eq(y.clone(), simp(add(vec![mul(vec![slope, sub(x.clone(), px.clone())]), py.clone()])))
    };
    let slope_txt = if num(&fy)?.abs() < 1e-15 { "vertical tangent".to_string() } else { format!("dy/dx = −F_x/F_y = {}", ptext(&simp(neg(div(fx.clone(), fy.clone()))))) };
    let mut pods = vec![input(format!("{} line to {} at ({}, {})", &m[1], ptext(&e), ptext(&px), ptext(&py))), primary(pod("Result", if tangent { "Tangent line" } else { "Normal line" }, math(&line)))];
    pods.push(pod("Slope", "Slope of the curve", text(slope_txt)));
    if let Some(p) = pl::implicit(&f, &x, &y, (num(&px)? - 3.0, num(&px)? + 3.0), (num(&py)? - 3.0, num(&py)? + 3.0)) {
        pods.push(pod("Plot", "Curve", plot(p)));
    }
    Some(Answer::new("tangent", pods))
}

fn sums(l: &str, q: &Query) -> Option<Answer> {
    let index_of = |e: &Expr| -> Option<String> {
        let fs = e.free_symbols();
        (fs.len() == 1).then(|| fs.into_iter().next().unwrap().to_string()).filter(|v| ["n", "k", "i", "j", "m"].contains(&v.as_str()))
    };
    if let Some(m) = re!(r"^(?:sum\s+)?convergence\s+(?:of\s+)?(?:the\s+)?(?:sum\s+|series\s+)?(.+)$").captures(l) {
        let e = ex(&m[1])?;
        let v = index_of(&e)?;
        let r = super::query_with(&format!("does sum {} from {v}=1 to infinity converge", &m[1]), &q.opts);
        return r.success.then(|| Answer::new("convergence", r.pods));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:sum|series)\s+(?:of\s+)?(.+)$").captures(l) {
        let body = &m[1];
        if re!(r"\b(from|to|for|over|until|where)\b|=|,").is_match(body) {
            return None;
        }
        let e = ex(body)?;
        let v = index_of(&e)?;
        let r = super::query_with(&format!("sum {body} from {v}=1 to infinity"), &q.opts);
        return r.success.then(|| Answer::new("sum", r.pods));
    }
    // 1/2^2 + 1/3^2 + 1/5^2 + 1/7^2 + ... : the prime zeta function
    if l.ends_with("...") || l.ends_with('…') || l.ends_with('+') {
        let terms: Vec<&str> = l.trim_end_matches("...").trim_end_matches('…').trim_end_matches(['+', ' ']).split('+').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
        let mut exps = Vec::new();
        let mut bases = Vec::new();
        for t in &terms {
            let c = re!(r"^1\s*/\s*(\d+)\s*\^\s*(\d+)$").captures(t)?;
            bases.push(c[1].parse::<u64>().ok()?);
            exps.push(c[2].parse::<u32>().ok()?);
        }
        let s0 = *exps.first()?;
        let primes = crate::numtheory::sieve(100);
        if terms.len() < 3 || exps.iter().any(|e| *e != s0) || s0 < 2 || bases.iter().zip(&primes).any(|(b, p)| b != p) {
            return None;
        }
        let mobius = |k: u64| -> f64 {
            let (mut n, mut c, mut p) = (k, 0, 2);
            while p * p <= n {
                if n % p == 0 {
                    n /= p;
                    if n % p == 0 {
                        return 0.0;
                    }
                    c += 1;
                }
                p += 1;
            }
            if n > 1 {
                c += 1;
            }
            if c % 2 == 0 { 1.0 } else { -1.0 }
        };
        let sv = s0 as f64;
        let val: f64 = (1..=80u64).map(|k| mobius(k) / k as f64 * crate::special::zeta(k as f64 * sv).ln()).sum();
        return Some(Answer::new("prime zeta", vec![input(format!("Σ over primes p of 1/p^{s0}")), primary(pod("Result", "Result", text(format!("P({s0}) ≈ {}", crate::print::fmt_float(val, 12))))), pod("Def", "Prime zeta function", text("P(s) = Σ_p p^(−s) = Σ_(k≥1) μ(k)/k · log ζ(k s)"))]));
    }
    None
}

fn function_cf(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?continued fractions?\s+(?:of\s+|for\s+|expansion of\s+)?(.+)$").captures(l)?;
    let body = m[1].replace(' ', "");
    let (name, cf, note) = match body.as_str() {
        "tanx" | "tan(x)" => ("tan x", "x/(1 − x²/(3 − x²/(5 − x²/(7 − x²/(9 − …)))))", "Lambert (1768); converges for all x where tan x is defined"),
        "tanhx" | "tanh(x)" => ("tanh x", "x/(1 + x²/(3 + x²/(5 + x²/(7 + x²/(9 + …)))))", "Lambert"),
        "arctanx" | "arctan(x)" | "atan(x)" | "tan^-1(x)" => ("arctan x", "x/(1 + x²/(3 + 4x²/(5 + 9x²/(7 + 16x²/(9 + …)))))", "Gauss; converges in the plane cut along (−i∞, −i] and [i, i∞)"),
        "log(1+x)" | "ln(1+x)" | "log(x+1)" | "ln(x+1)" => ("log(1 + x)", "x/(1 + x/(2 + x/(3 + 4x/(4 + 4x/(5 + 9x/(6 + 9x/(7 + …)))))))", "converges in the plane cut along (−∞, −1]"),
        "sqrt(1+x^2)" | "sqrt(x^2+1)" => ("√(1 + x²)", "1 + x²/(2 + x²/(2 + x²/(2 + x²/(2 + …))))", "from y − 1 = x²/(y + 1)"),
        "sqrt(1+x)" | "sqrt(x+1)" => ("√(1 + x)", "1 + x/(2 + x/(2 + x/(2 + x/(2 + …))))", "from y − 1 = x/(y + 1)"),
        _ => return None,
    };
    Some(Answer::new("continued fraction", vec![input(format!("continued fraction of {name}")), primary(pod("Result", "Continued fraction", text(cf))), pod("Note", "Note", text(note))]))
}

fn percent_of(l: &str) -> Option<Answer> {
    let m = re!(r"^([\d.]+)\s*%\s+of\s+(\$|€|£)?\s*([\d.,]+)\s*([a-z][a-z ]*)?$").captures(l)?;
    let p: f64 = m[1].parse().ok()?;
    let v: f64 = m[3].replace(',', "").parse().ok()?;
    let cur = m.get(2).map(|x| x.as_str());
    let unit = m.get(4).map(|x| x.as_str().trim());
    if cur.is_none() && unit.is_none() {
        return None;
    }
    let r = p * v / 100.0;
    let shown = match (cur, unit) {
        (Some(c), _) => format!("{c}{} (exactly {c}{})", fmt((r * 100.0).round() / 100.0), fmt(r)),
        (None, Some(u)) => format!("{} {u}", fmt(r)),
        _ => fmt(r),
    };
    Some(Answer::new("percent", vec![input(format!("{}% of {}{}{}", &m[1], cur.unwrap_or(""), &m[3], unit.map(|u| format!(" {u}")).unwrap_or_default())), primary(pod("Result", "Result", text(shown)))]))
}

fn several_derivatives(l: &str, q: &Query) -> Option<Answer> {
    if !l.starts_with("d/d") || !l.contains(", d/d") {
        return None;
    }
    let mut pods = Vec::new();
    for part in l.split(", d/d") {
        let qq = if part.starts_with("d/d") { part.to_string() } else { format!("d/d{part}") };
        let r = super::query_with(&qq, &q.opts);
        if !r.success {
            return None;
        }
        let res = r.pods.into_iter().find(|p| p.primary)?;
        pods.push(pod(&format!("D{}", pods.len()), &qq, res.subpods.into_iter().next()?.content));
    }
    if let Some(first) = pods.first_mut() {
        first.primary = true;
    }
    Some(Answer::new("derivatives", pods))
}


// ---------------------------------------------------------------------------
// arithmetic word problems
// ---------------------------------------------------------------------------

/// Tracks counts per (owner, object) through has/gives/takes/buys clauses.
fn word_problem(l: &str) -> Option<Answer> {
    if !l.contains("how many") {
        return None;
    }
    use std::collections::BTreeMap;
    let mut have: BTreeMap<(String, String), i64> = BTreeMap::new();
    let mut last_owner = String::new();
    let mut last_obj = String::new();
    let mut steps: Vec<String> = Vec::new();
    let norm = |w: &str, last: &str| -> String {
        match w {
            "she" | "he" | "they" => last.to_string(),
            "i" | "me" | "my" => "me".into(),
            x => x.to_string(),
        }
    };
    let text_ = l.replace(['?', '!'], ".").replace("and then", ",").replace(" then ", ", ").replace(" how many", ", how many");
    let clauses: Vec<String> = re!(r"\s*(?:[.,;]|\band\b)\s*").split(&text_).map(|c| c.trim().trim_start_matches("if ").trim_start_matches("so ").trim().to_string()).filter(|c| !c.is_empty()).collect();
    let mut question: Option<String> = None;
    for c in clauses {
        if c.starts_with("how many") {
            question = Some(c);
            continue;
        }
        if question.is_some() {
            // "how many more cats does jane have than i" may be split on "and": keep the rest
            let qq = question.take().unwrap();
            question = Some(format!("{qq} {c}"));
            continue;
        }
        if let Some(m) = re!(r"^there (?:are|were|is|was)\s+(\d+)\s+([a-z]+)(?:\s+(?:all together|altogether|in total|in all))?$").captures(&c) {
            last_obj = m[2].to_string();
            have.insert(("(total)".into(), last_obj.clone()), m[1].parse().ok()?);
            last_owner = "(total)".into();
            continue;
        }
        if let Some(m) = re!(r"^([a-z]+)\s+(?:has|have|had|owns)\s+(\d+)\s+([a-z]+)$").captures(&c) {
            last_owner = norm(&m[1], &last_owner);
            last_obj = m[3].to_string();
            have.insert((last_owner.clone(), last_obj.clone()), m[2].parse().ok()?);
            continue;
        }
        if let Some(m) = re!(r"^(\d+)\s+([a-z]+)$").captures(&c) {
            if last_owner.is_empty() {
                return None;
            }
            last_obj = m[2].to_string();
            have.insert((last_owner.clone(), last_obj.clone()), m[1].parse().ok()?);
            continue;
        }
        let give = re!(r"^([a-z]+)\s+(?:gives|gave|give|hands|passes)\s+(\d+)\s*([a-z]+)?\s+to\s+([a-z]+)$").captures(&c).map(|m| (m[1].to_string(), m[2].to_string(), m.get(3).map(|x| x.as_str().to_string()), m[4].to_string()))
            .or_else(|| re!(r"^([a-z]+)\s+(?:gives|gave|give|hands)\s+([a-z]+)\s+(\d+)\s*([a-z]+)?$").captures(&c).map(|m| (m[1].to_string(), m[3].to_string(), m.get(4).map(|x| x.as_str().to_string()), m[2].to_string())));
        if let Some((from, n, obj, to)) = give {
            let from = norm(&from, &last_owner);
            let to = norm(&to, &last_owner);
            let n: i64 = n.parse().ok()?;
            let obj = obj.filter(|o| o != "of").unwrap_or_else(|| last_obj.clone());
            *have.entry((from.clone(), obj.clone())).or_insert(0) -= n;
            *have.entry((to.clone(), obj.clone())).or_insert(0) += n;
            steps.push(format!("{from} gives {n} {obj} to {to}"));
            last_owner = from;
            last_obj = obj;
            continue;
        }
        if let Some(m) = re!(r"^([a-z]+)\s+(takes|took|eats|ate|loses|lost|uses|used|sells|sold|gets|got|buys|bought|finds|found|receives|received|picks|picked)\s+(?:away\s+)?(\d+)\s*(?:more\s+)?([a-z]+)?$").captures(&c) {
            let who = norm(&m[1], &last_owner);
            let n: i64 = m[3].parse().ok()?;
            let obj = m.get(4).map(|x| x.as_str().to_string()).unwrap_or_else(|| last_obj.clone());
            let gain = matches!(&m[2], "gets" | "got" | "buys" | "bought" | "finds" | "found" | "receives" | "received" | "picks" | "picked");
            let take = matches!(&m[2], "takes" | "took");
            let owns = have.contains_key(&(who.clone(), obj.clone()));
            if take && !owns {
                // taking from a shared pile
                let pile = have.keys().find(|(o, ob)| ob == &obj && o != &who).cloned()?;
                *have.get_mut(&pile)? -= n;
                *have.entry((who.clone(), obj.clone())).or_insert(0) += n;
            } else {
                *have.entry((who.clone(), obj.clone())).or_insert(0) += if gain { n } else { -n };
            }
            steps.push(format!("{who} {} {n} {obj}", &m[2]));
            last_owner = who;
            last_obj = obj;
            continue;
        }
        return None;
    }
    let qn = question?;
    let count = |owner: &str, obj: &str| -> i64 { have.iter().filter(|((o, ob), _)| o == owner && (obj.is_empty() || ob == obj)).map(|(_, v)| *v).sum() };
    let (ans, desc): (i64, String) = if let Some(m) = re!(r"^how many more\s+(?:([a-z]+)\s+)?(?:does|do|did)\s+([a-z]+)\s+have than\s+([a-z]+)(?:\s+(?:does|do|did|has|have))?$").captures(&qn) {
        let obj = m.get(1).map(|x| x.as_str().to_string()).unwrap_or_else(|| last_obj.clone());
        let (a, b) = (norm(&m[2], &last_owner), norm(&m[3], &last_owner));
        (count(&a, &obj) - count(&b, &obj), format!("{a} has {} {obj} and {b} has {}", count(&a, &obj), count(&b, &obj)))
    } else if let Some(m) = re!(r"^how many\s+(?:([a-z]+)\s+)?(?:does|do|did|will)\s+([a-z]+)\s+(?:have|has)(?:\s+(?:now|left|in all|altogether))?$").captures(&qn) {
        let obj = m.get(1).map(|x| x.as_str().to_string()).unwrap_or_else(|| last_obj.clone());
        let who = norm(&m[2], &last_owner);
        (count(&who, &obj), format!("{who} has {} {obj}", count(&who, &obj)))
    } else if let Some(m) = re!(r"^how many\s+(?:([a-z]+)\s+)?(?:are|is|were)\s+(?:there\s+)?(in all|altogether|all together|in total|total|left|remaining)$").captures(&qn) {
        let obj = m.get(1).map(|x| x.as_str().to_string()).unwrap_or_else(|| last_obj.clone());
        if matches!(&m[2], "left" | "remaining") {
            let pile = if have.contains_key(&("(total)".to_string(), obj.clone())) { "(total)".to_string() } else { last_owner.clone() };
            (count(&pile, &obj), format!("{} {obj} remain", count(&pile, &obj)))
        } else {
            let t: i64 = have.iter().filter(|((o, ob), _)| ob == &obj && o != "(total)").map(|(_, v)| *v).sum();
            (t, format!("{t} {obj} in all"))
        }
    } else {
        return None;
    };
    let mut pods = vec![input("arithmetic word problem".into()), primary(pod("Result", "Result", text(ans.to_string()))), pod("Reasoning", "Reasoning", text(desc))];
    if !steps.is_empty() {
        pods.push(pod("Steps", "Events", text(steps.join("; "))));
    }
    Some(Answer::new("word problem", pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let raw = q.clean.trim().trim_end_matches(['?', '.']);
    let l = raw.to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    if let Some(m) = re!(r"^\|\|\s*(.+?)\s*\|\|$").captures(&l) {
        let r = super::query_with(&format!("norm of {}", &m[1]), &q.opts);
        if r.success {
            return Some(Answer::new("norm", r.pods));
        }
    }
    triangles(&l)
        .or_else(|| number_lines(&l))
        .or_else(|| charts(&l))
        .or_else(|| point_plots(&l))
        .or_else(|| compare_fns(&l))
        .or_else(|| fn_props(&l))
        .or_else(|| solve_interval(&l))
        .or_else(|| check_solution(&l))
        .or_else(|| trig_from_one(&l))
        .or_else(|| implicit_line(&l))
        .or_else(|| sums(&l, q))
        .or_else(|| function_cf(&l))
        .or_else(|| percent_of(&l))
        .or_else(|| several_derivatives(&l, q))
        .or_else(|| word_problem(&l))
}

#[cfg(test)]
mod tests {
    fn ans(s: &str) -> String {
        serde_json::to_string(&crate::engine::query(s).pods).unwrap_or_default()
    }

    #[test]
    fn school_answers() {
        assert!(ans("period of 6 sin(x/3) - 7").contains("6 π"));
        assert!(ans("solve sin(x) + 3 = 4 on 0 <= x < 2pi").contains("π/2"));
        assert!(ans("if sin(a) = 3/5, what is cos(a)?").contains("±4/5"));
        assert!(ans("{{1, 1}, {3, 6}, {9, -2}} triangle incenter").contains("Incenter"));
        assert!(ans("3-4-5 triangle with inscribed circle").contains("\"1\""));
        assert!(ans("height of triangle with width 3 and hypotenuse 5").contains("\"4\""));
        assert!(ans("is 5 a solution to 1/2x + 2 > 3/4?").contains("yes"));
        assert!(ans("compare x^2 and 4x^2 - 5").contains("Intersection"));
    }
}
