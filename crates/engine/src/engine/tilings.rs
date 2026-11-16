//! Periodic tilings of the plane: the three regular tilings, the
//! Archimedean (semiregular) tilings that have a simple lattice
//! construction, and the rhombille tiling, drawn in a window with their
//! vertex configurations, symmetry groups and duals.
//!
//! Edge-to-edge tilings with unit edges are drawn from their vertex set
//! alone: two vertices are joined exactly when they are one unit apart.

use super::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

type P = [f64; 2];

const WINDOW: f64 = 6.0;

/// Regular polygon vertices: centre, circumradius, first angle (degrees), sides.
fn polygon(c: P, r: f64, start_deg: f64, k: usize) -> Vec<P> {
    (0..k).map(|i| {
        let a = (start_deg + 360.0 * i as f64 / k as f64).to_radians();
        [c[0] + r * a.cos(), c[1] + r * a.sin()]
    }).collect()
}

/// Lattice points a·i + b·j inside a padded window.
fn lattice(a: P, b: P) -> Vec<P> {
    let mut out = Vec::new();
    for i in -12..=12 {
        for j in -12..=12 {
            let p = [a[0] * i as f64 + b[0] * j as f64, a[1] * i as f64 + b[1] * j as f64];
            if p[0].abs() <= WINDOW + 3.0 && p[1].abs() <= WINDOW + 3.0 {
                out.push(p);
            }
        }
    }
    out
}

fn dedup(mut v: Vec<P>) -> Vec<P> {
    v.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    let mut out: Vec<P> = Vec::new();
    for p in v {
        if !out.iter().rev().take(64).any(|q| (q[0] - p[0]).abs() < 1e-6 && (q[1] - p[1]).abs() < 1e-6) {
            out.push(p);
        }
    }
    out
}

/// Join vertices one unit apart.
fn unit_edges(verts: Vec<P>) -> Vec<[P; 2]> {
    let v = dedup(verts);
    let mut segs = Vec::new();
    for i in 0..v.len() {
        for j in i + 1..v.len() {
            if v[j][0] - v[i][0] > 1.01 {
                break;
            }
            let d = ((v[i][0] - v[j][0]).powi(2) + (v[i][1] - v[j][1]).powi(2)).sqrt();
            if (d - 1.0).abs() < 1e-6 {
                segs.push([v[i], v[j]]);
            }
        }
    }
    segs
}

fn clip(segs: Vec<[P; 2]>) -> Vec<[P; 2]> {
    segs.into_iter().filter(|s| s.iter().all(|p| p[0].abs() <= WINDOW && p[1].abs() <= WINDOW)).collect()
}

fn polys_vertices(centres: &[P], r: f64, start: f64, k: usize) -> Vec<P> {
    centres.iter().flat_map(|c| polygon(*c, r, start, k)).collect()
}

struct Tiling {
    names: &'static [&'static str],
    title: &'static str,
    config: &'static str,
    symmetry: &'static str,
    dual: &'static str,
    faces: &'static str,
}

const TILINGS: &[Tiling] = &[
    Tiling { names: &["triangular", "triangle", "deltille"], title: "triangular tiling", config: "3.3.3.3.3.3 (3⁶)", symmetry: "p6m (*632)", dual: "hexagonal tiling", faces: "equilateral triangles" },
    Tiling { names: &["square", "quadrille"], title: "square tiling", config: "4.4.4.4 (4⁴)", symmetry: "p4m (*442)", dual: "square tiling (self-dual)", faces: "squares" },
    Tiling { names: &["hexagonal", "hexagon", "honeycomb", "hextille"], title: "hexagonal tiling", config: "6.6.6 (6³)", symmetry: "p6m (*632)", dual: "triangular tiling", faces: "regular hexagons" },
    Tiling { names: &["trihexagonal", "kagome", "hexadeltille"], title: "trihexagonal tiling", config: "3.6.3.6", symmetry: "p6m (*632)", dual: "rhombille tiling", faces: "triangles and hexagons" },
    Tiling { names: &["snub square", "snub quadrille"], title: "snub square tiling", config: "3.3.4.3.4", symmetry: "p4g (4*2)", dual: "Cairo pentagonal tiling", faces: "triangles and squares" },
    Tiling { names: &["truncated square", "truncated quadrille", "octagonal"], title: "truncated square tiling", config: "4.8.8", symmetry: "p4m (*442)", dual: "tetrakis square tiling", faces: "squares and octagons" },
    Tiling { names: &["elongated triangular", "isosnub quadrille"], title: "elongated triangular tiling", config: "3.3.3.4.4", symmetry: "cmm (2*22)", dual: "prismatic pentagonal tiling", faces: "triangles and squares" },
    Tiling { names: &["rhombitrihexagonal", "rhombihexadeltille", "small rhombitrihexagonal"], title: "rhombitrihexagonal tiling", config: "3.4.6.4", symmetry: "p6m (*632)", dual: "deltoidal trihexagonal tiling", faces: "triangles, squares and hexagons" },
    Tiling { names: &["truncated hexagonal", "truncated hextille"], title: "truncated hexagonal tiling", config: "3.12.12", symmetry: "p6m (*632)", dual: "triakis triangular tiling", faces: "triangles and dodecagons" },
    Tiling { names: &["truncated trihexagonal", "great rhombitrihexagonal", "truncated hexadeltille"], title: "truncated trihexagonal tiling", config: "4.6.12", symmetry: "p6m (*632)", dual: "kisrhombille tiling", faces: "squares, hexagons and dodecagons" },
    Tiling { names: &["rhombille", "rhombic", "tumbling blocks", "reversible cubes", "dice lattice"], title: "rhombille tiling", config: "V3.6.3.6 (face configuration)", symmetry: "p6m (*632)", dual: "trihexagonal tiling", faces: "60°–120° rhombi" },
];

fn draw(title: &str) -> Vec<[P; 2]> {
    let s3 = 3f64.sqrt();
    match title {
        "triangular tiling" => unit_edges(lattice([1.0, 0.0], [0.5, s3 / 2.0])),
        "square tiling" => unit_edges(lattice([1.0, 0.0], [0.0, 1.0])),
        "hexagonal tiling" => unit_edges(polys_vertices(&lattice([s3, 0.0], [s3 / 2.0, 1.5]), 1.0, 30.0, 6)),
        "trihexagonal tiling" => unit_edges(polys_vertices(&lattice([2.0, 0.0], [1.0, s3]), 1.0, 0.0, 6)),
        "snub square tiling" => {
            let l = (1.0 + s3) / 2f64.sqrt();
            let a = lattice([l, 0.0], [0.0, l]);
            let b: Vec<P> = a.iter().map(|p| [p[0] + l / 2.0, p[1] + l / 2.0]).collect();
            let r = 1.0 / 2f64.sqrt();
            let mut v = polys_vertices(&a, r, 60.0, 4);
            v.extend(polys_vertices(&b, r, 30.0, 4));
            unit_edges(v)
        }
        "truncated square tiling" => {
            let r = 1.0 / (2.0 * (std::f64::consts::PI / 8.0).sin());
            let sp = 1.0 + 2f64.sqrt();
            unit_edges(polys_vertices(&lattice([sp, 0.0], [0.0, sp]), r, 22.5, 8))
        }
        "elongated triangular tiling" => {
            let mut v = Vec::new();
            let h = 1.0 + s3 / 2.0;
            for row in -6..=6 {
                for i in -10..=10 {
                    let off = if row % 2 == 0 { 0.0 } else { 0.5 };
                    let y = row as f64 * h;
                    v.push([i as f64 + off, y]);
                    v.push([i as f64 + off, y + 1.0]);
                }
            }
            unit_edges(v)
        }
        "rhombitrihexagonal tiling" => {
            let d = 1.0 + s3;
            unit_edges(polys_vertices(&lattice([d, 0.0], [d / 2.0, d * s3 / 2.0]), 1.0, 30.0, 6))
        }
        "truncated hexagonal tiling" => {
            let r = 1.0 / (2.0 * (std::f64::consts::PI / 12.0).sin());
            let d = 2.0 + s3;
            unit_edges(polys_vertices(&lattice([d, 0.0], [d / 2.0, d * s3 / 2.0]), r, 15.0, 12))
        }
        "truncated trihexagonal tiling" => {
            let r = 1.0 / (2.0 * (std::f64::consts::PI / 12.0).sin());
            let d = 3.0 + s3;
            unit_edges(polys_vertices(&lattice([d, 0.0], [d / 2.0, d * s3 / 2.0]), r, 15.0, 12))
        }
        _ => {
            // rhombille: each hexagon split into three rhombi from its centre
            let centres = lattice([s3, 0.0], [s3 / 2.0, 1.5]);
            let mut segs = unit_edges(polys_vertices(&centres, 1.0, 30.0, 6));
            for c in centres {
                for v in polygon(c, 1.0, 30.0, 6).into_iter().step_by(2) {
                    segs.push([c, v]);
                }
            }
            segs
        }
    }
}

fn mid(a: P, b: P) -> P {
    [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]
}

fn lerp(a: P, b: P, t: f64) -> P {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

/// Pinwheel (Conway–Radin): a 1–2–√5 right triangle given as (right-angle
/// vertex, short-leg end, long-leg end) splits into five copies scaled 1/√5.
fn pinwheel(steps: usize) -> Vec<[P; 3]> {
    let mut tris = vec![[[0.0, 0.0], [0.0, 1.0], [2.0, 0.0]]];
    for _ in 0..steps {
        let mut next = Vec::with_capacity(tris.len() * 5);
        for [r, s, l] in tris {
            // foot of the altitude from r onto the hypotenuse s–l
            let (dx, dy) = (l[0] - s[0], l[1] - s[1]);
            let t = ((r[0] - s[0]) * dx + (r[1] - s[1]) * dy) / (dx * dx + dy * dy);
            let h = lerp(s, l, t);
            next.push([h, s, r]);
            let (m1, m2, m3) = (mid(h, r), mid(h, l), mid(r, l));
            next.push([h, m1, m2]);
            next.push([m1, r, m3]);
            next.push([m2, m3, l]);
            next.push([m3, m2, m1]);
        }
        tris = next;
    }
    tris
}

/// Penrose P3 via Robinson triangles: (thin?, apex, base, base).
fn penrose(steps: usize) -> Vec<(bool, [P; 3])> {
    let phi = (1.0 + 5f64.sqrt()) / 2.0;
    let mut tris = Vec::new();
    for i in 0..10 {
        let ang = |k: i32| (k as f64) * std::f64::consts::PI / 10.0;
        let mut b = [ang(2 * i - 1).cos() * 6.0, ang(2 * i - 1).sin() * 6.0];
        let mut c = [ang(2 * i + 1).cos() * 6.0, ang(2 * i + 1).sin() * 6.0];
        if i % 2 == 0 {
            std::mem::swap(&mut b, &mut c);
        }
        tris.push((true, [[0.0, 0.0], b, c]));
    }
    for _ in 0..steps {
        let mut next = Vec::new();
        for (thin, [a, b, c]) in tris {
            if thin {
                let p = lerp(a, b, 1.0 / phi);
                next.push((true, [c, p, b]));
                next.push((false, [p, c, a]));
            } else {
                let q = lerp(b, a, 1.0 / phi);
                let r = lerp(b, c, 1.0 / phi);
                next.push((false, [r, c, a]));
                next.push((false, [q, r, b]));
                next.push((true, [r, q, a]));
            }
        }
        tris = next;
    }
    tris
}

/// Domino substitution: a 2×1 rectangle (origin, long side, short side)
/// becomes two crosswise dominoes at the ends and two lengthwise ones
/// stacked in the middle, each scaled by 1/2.
fn domino(steps: usize) -> Vec<[P; 4]> {
    let mut rects = vec![([0.0, 0.0], [8.0, 0.0], [0.0, 4.0])];
    for _ in 0..steps {
        let mut next = Vec::new();
        for (o, u, v) in rects {
            let at = |a: f64, b: f64| [o[0] + u[0] * a + v[0] * b, o[1] + u[1] * a + v[1] * b];
            let sc = |w: P, k: f64| [w[0] * k, w[1] * k];
            // ends: long side along v (length |v|), short side |u|/4
            next.push((at(0.0, 0.0), v, sc(u, 0.25)));
            next.push((at(0.75, 0.0), v, sc(u, 0.25)));
            // middle: long side |u|/2, short side |v|/2
            next.push((at(0.25, 0.0), sc(u, 0.5), sc(v, 0.5)));
            next.push((at(0.25, 0.5), sc(u, 0.5), sc(v, 0.5)));
        }
        rects = next;
    }
    rects.into_iter().map(|(o, u, v)| [o, [o[0] + u[0], o[1] + u[1]], [o[0] + u[0] + v[0], o[1] + u[1] + v[1]], [o[0] + v[0], o[1] + v[1]]]).collect()
}

fn substitution(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let m = re!(r"^(?:draw |show |plot )?(?:an? |the )?(pinwheel|penrose(?: rhomb| triangle| robinson)?|robinson triangle|domino) (?:tiling|tessellation|substitution)(?:,? (?:after |with )?(\d+) (?:steps?|iterations?|generations?))?$").captures(l)?;
    let kind = &m[1];
    let steps: usize = m.get(2).and_then(|s| s.as_str().parse().ok()).unwrap_or(if kind == "pinwheel" { 3 } else { 4 });
    let mut segs: Vec<[P; 2]> = Vec::new();
    let (title, rows): (String, Vec<(&str, String)>);
    match kind {
        "pinwheel" => {
            let s = steps.min(5);
            let tris = pinwheel(s);
            for [a, b, c] in &tris {
                segs.extend([[*a, *b], [*b, *c], [*c, *a]]);
            }
            title = format!("pinwheel tiling ({s} steps)");
            rows = vec![
                ("prototile", "right triangle with legs 1, 2 and hypotenuse √5".into()),
                ("substitution", "1 → 5 copies, linear scale 1/√5".into()),
                ("tiles after n steps", format!("5^{s} = {}", tris.len())),
                ("orientations", "infinitely many (statistically circular symmetry)".into()),
                ("discovered", "John Conway and Charles Radin (1994)".into()),
            ];
        }
        "domino" => {
            let s = steps.min(6);
            let rects = domino(s);
            for r in &rects {
                segs.extend([[r[0], r[1]], [r[1], r[2]], [r[2], r[3]], [r[3], r[0]]]);
            }
            title = format!("domino tiling ({s} steps)");
            rows = vec![
                ("prototile", "2 × 1 rectangle".into()),
                ("substitution", "1 → 4 copies, linear scale 1/2".into()),
                ("tiles after n steps", format!("4^{s} = {}", rects.len())),
                ("orientations", "2".into()),
            ];
        }
        _ => {
            let triangles = kind.contains("triangle") || kind.contains("robinson");
            let s = steps.min(8);
            let tris = penrose(s);
            for (_, [a, b, c]) in &tris {
                // rhombs: omit the shared base so triangle pairs merge
                segs.extend([[*c, *a], [*a, *b]]);
                if triangles {
                    segs.push([*b, *c]);
                }
            }
            let thin = tris.iter().filter(|t| t.0).count();
            title = format!("Penrose {} tiling ({s} steps)", if triangles { "triangle" } else { "rhomb" });
            rows = vec![
                ("prototiles", if triangles { "Robinson triangles (36°–72°–72° and 108°–36°–36°)".into() } else { "thin (36°) and thick (72°) rhombs".into() }),
                ("substitution", "linear scale 1/φ, φ = (1+√5)/2".into()),
                ("triangles after n steps", format!("{} ({thin} thin, {} thick)", tris.len(), tris.len() - thin)),
                ("thick : thin ratio", format!("→ φ ≈ {:.6}", (1.0 + 5f64.sqrt()) / 2.0)),
                ("symmetry", "5-fold rotational (aperiodic)".into()),
                ("discovered", "Roger Penrose (1974)".into()),
            ];
        }
    }
    let rows = rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect();
    Some(Answer::new(&title, vec![
        pod("Input", "Input interpretation", text(&title)),
        primary(pod("Image", "Tiling", plot(crate::plot::segments(segs, &title)))),
        pod("Properties", "Properties", table(None, rows)),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    if let Some(a) = substitution(q) {
        return Some(a);
    }
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let m = re!(r"^(?:draw |show |plot )?(?:an? |the )?([a-z -]+?) (?:tiling|tessellation|tesselation)$").captures(l)?;
    let name = m[1].trim();
    let t = TILINGS.iter().find(|t| t.names.contains(&name))?;
    let segs = clip(draw(t.title));
    let rows = vec![
        vec![text("vertex configuration"), text(t.config)],
        vec![text("faces"), text(t.faces)],
        vec![text("wallpaper group"), text(t.symmetry)],
        vec![text("dual tiling"), text(t.dual)],
        vec![text("edge-to-edge"), text("yes")],
    ];
    Some(Answer::new(t.title, vec![
        pod("Input", "Input interpretation", text(t.title)),
        primary(pod("Image", "Tiling", plot(crate::plot::segments(segs, t.title)))),
        pod("Properties", "Properties", table(None, rows)),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_degrees() {
        // interior vertex degrees match the vertex configurations
        let degree = |title: &str| {
            let segs = draw(title);
            let mut count = std::collections::HashMap::new();
            for s in &segs {
                for p in s {
                    if p[0].abs() < 1.5 && p[1].abs() < 1.5 {
                        *count.entry(((p[0] * 1e4).round() as i64, (p[1] * 1e4).round() as i64)).or_insert(0) += 1;
                    }
                }
            }
            count.values().copied().max().unwrap_or(0)
        };
        assert_eq!(degree("triangular tiling"), 6);
        assert_eq!(degree("hexagonal tiling"), 3);
        assert_eq!(degree("snub square tiling"), 5);
        assert_eq!(degree("truncated square tiling"), 3);
        assert_eq!(degree("rhombitrihexagonal tiling"), 4);
        assert_eq!(degree("truncated trihexagonal tiling"), 3);
        assert_eq!(degree("elongated triangular tiling"), 5);
    }
}
