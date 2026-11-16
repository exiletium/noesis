//! Named geometric objects: Platonic and Archimedean solids ("icosahedron",
//! "truncated icosahedron with edge 2", "Archimedean solids"), regular
//! 4-polytopes and cross polytopes ("tesseract", "600-cell", "cross
//! polytope"), the glome, spherical caps, Steinmetz solids, the Reuleaux
//! triangle and the pseudosphere; triangles from three sides ("5, 12, 13
//! triangle"); named plane and space curves ("cardioid", "arc length of an
//! astroid", "tangent vector Viviani's curve"); knots ("trefoil knot",
//! "Stevedore knot, figure eight knot"); homotopy groups of spheres
//! ("pi_5(S^2)"); real intervals ("[3,8)"); and counts of polyforms
//! ("polyominoes", "hexiamonds").
//!
//! Polyhedron measures are the standard closed forms for unit edge; knot
//! invariants follow the Rolfsen table; homotopy groups follow Toda's
//! tables; polyform counts are the OEIS sequences A000105, A000988,
//! A001168, A000577, A000228, A038119, A000162 and A006074.

use crate::engine::*;
use crate::expr::*;
use crate::print::text as ptext;
use crate::simplify as s;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn ex(t: &str) -> Expr {
    if let Ok(v) = t.parse::<f64>() {
        if t.contains('.') {
            return float(v);
        }
    }
    crate::parse::parse(t).map(|p| p.expr).unwrap_or_else(|_| undef())
}

fn fmt(v: f64) -> String {
    crate::stats::fmt(v)
}

/// "exact ≈ decimal" for a closed form, or just the value.
fn show(e: &Expr) -> String {
    let t = ptext(e);
    if e.free_symbols().is_empty() && !e.is_integer() && !e.has_float() {
        if let Some(v) = e.to_f64_const() {
            return format!("{t} ≈ {}", fmt(v));
        }
    }
    t
}

fn num_arg(t: &str) -> Option<Expr> {
    let e = crate::parse::parse(t.trim()).ok()?.expr;
    e.to_f64_const().filter(|v| v.is_finite() && *v > 0.0).map(|_| e)
}

fn input(t: String) -> Pod {
    pod("Input", "Input interpretation", text(t))
}

fn rows(v: Vec<(&str, String)>) -> Content {
    table(None, v.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())
}

/// Strip a leading article and trailing punctuation, lower case.
fn tidy(q: &Query) -> String {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    re!(r"^(?:(?:what is|what's|show|show me|describe)\s+)?(?:an?\s+|the\s+)?").replace(l, "").trim().to_string()
}

// ---------------------------------------------------------------------------
// polyhedra
// ---------------------------------------------------------------------------

struct Poly {
    names: &'static [&'static str],
    class: &'static str,
    faces: &'static str,
    f: u32,
    e: u32,
    v: u32,
    vol: &'static str,
    area: &'static str,
    circum: &'static str,
    inradius: &'static str,
    dihedral: &'static str,
    dual: &'static str,
    group: &'static str,
    vertex_config: &'static str,
}

macro_rules! ph {
    ($n:expr, $c:expr, $fc:expr, $f:expr, $e:expr, $v:expr, $vol:expr, $a:expr, $r:expr, $ir:expr, $d:expr, $du:expr, $g:expr, $vc:expr) => {
        Poly { names: $n, class: $c, faces: $fc, f: $f, e: $e, v: $v, vol: $vol, area: $a, circum: $r, inradius: $ir, dihedral: $d, dual: $du, group: $g, vertex_config: $vc }
    };
}

static POLYHEDRA: &[Poly] = &[
    ph!(&["tetrahedron", "regular tetrahedron"], "Platonic solid", "4 equilateral triangles", 4, 6, 4, "sqrt(2)/12", "sqrt(3)", "sqrt(6)/4", "sqrt(6)/12", "arccos(1/3)", "tetrahedron (self-dual)", "T_d", "3.3.3"),
    ph!(&["cube", "hexahedron", "regular hexahedron"], "Platonic solid", "6 squares", 6, 12, 8, "1", "6", "sqrt(3)/2", "1/2", "pi/2", "octahedron", "O_h", "4.4.4"),
    ph!(&["octahedron", "regular octahedron"], "Platonic solid", "8 equilateral triangles", 8, 12, 6, "sqrt(2)/3", "2*sqrt(3)", "sqrt(2)/2", "sqrt(6)/6", "arccos(-1/3)", "cube", "O_h", "3.3.3.3"),
    ph!(&["dodecahedron", "regular dodecahedron"], "Platonic solid", "12 regular pentagons", 12, 30, 20, "(15+7*sqrt(5))/4", "3*sqrt(25+10*sqrt(5))", "sqrt(3)*(1+sqrt(5))/4", "sqrt(250+110*sqrt(5))/20", "arccos(-sqrt(5)/5)", "icosahedron", "I_h", "5.5.5"),
    ph!(&["icosahedron", "regular icosahedron"], "Platonic solid", "20 equilateral triangles", 20, 30, 12, "5*(3+sqrt(5))/12", "5*sqrt(3)", "sqrt(10+2*sqrt(5))/4", "sqrt(3)*(3+sqrt(5))/12", "arccos(-sqrt(5)/3)", "dodecahedron", "I_h", "3.3.3.3.3"),
    ph!(&["truncated tetrahedron"], "Archimedean solid", "4 triangles, 4 hexagons", 8, 18, 12, "23*sqrt(2)/12", "7*sqrt(3)", "sqrt(22)/4", "", "", "triakis tetrahedron", "T_d", "3.6.6"),
    ph!(&["cuboctahedron"], "Archimedean solid", "8 triangles, 6 squares", 14, 24, 12, "5*sqrt(2)/3", "6+2*sqrt(3)", "1", "", "", "rhombic dodecahedron", "O_h", "3.4.3.4"),
    ph!(&["truncated cube", "truncated hexahedron"], "Archimedean solid", "8 triangles, 6 octagons", 14, 36, 24, "(21+14*sqrt(2))/3", "2*(6+6*sqrt(2)+sqrt(3))", "sqrt(7+4*sqrt(2))/2", "", "", "triakis octahedron", "O_h", "3.8.8"),
    ph!(&["truncated octahedron"], "Archimedean solid", "6 squares, 8 hexagons", 14, 36, 24, "8*sqrt(2)", "6+12*sqrt(3)", "sqrt(10)/2", "", "", "tetrakis hexahedron", "O_h", "4.6.6"),
    ph!(&["rhombicuboctahedron", "small rhombicuboctahedron"], "Archimedean solid", "8 triangles, 18 squares", 26, 48, 24, "(12+10*sqrt(2))/3", "2*(9+sqrt(3))", "sqrt(5+2*sqrt(2))/2", "", "", "deltoidal icositetrahedron", "O_h", "3.4.4.4"),
    ph!(&["truncated cuboctahedron", "great rhombicuboctahedron"], "Archimedean solid", "12 squares, 8 hexagons, 6 octagons", 26, 72, 48, "22+14*sqrt(2)", "12*(2+sqrt(2)+sqrt(3))", "sqrt(13+6*sqrt(2))/2", "", "", "disdyakis dodecahedron", "O_h", "4.6.8"),
    ph!(&["snub cube", "snub cuboctahedron"], "Archimedean solid", "32 triangles, 6 squares", 38, 60, 24, "7.889477400", "6+8*sqrt(3)", "1.343713374", "", "", "pentagonal icositetrahedron", "O", "3.3.3.3.4"),
    ph!(&["icosidodecahedron"], "Archimedean solid", "20 triangles, 12 pentagons", 32, 60, 30, "(45+17*sqrt(5))/6", "5*sqrt(3)+3*sqrt(25+10*sqrt(5))", "(1+sqrt(5))/2", "", "", "rhombic triacontahedron", "I_h", "3.5.3.5"),
    ph!(&["truncated dodecahedron"], "Archimedean solid", "20 triangles, 12 decagons", 32, 90, 60, "5*(99+47*sqrt(5))/12", "5*(sqrt(3)+6*sqrt(5+2*sqrt(5)))", "sqrt(74+30*sqrt(5))/4", "", "", "triakis icosahedron", "I_h", "3.10.10"),
    ph!(&["truncated icosahedron", "soccer ball", "buckyball"], "Archimedean solid", "12 pentagons, 20 hexagons", 32, 90, 60, "(125+43*sqrt(5))/4", "3*(10*sqrt(3)+sqrt(25+10*sqrt(5)))", "sqrt(58+18*sqrt(5))/4", "", "", "pentakis dodecahedron", "I_h", "5.6.6"),
    ph!(&["rhombicosidodecahedron", "small rhombicosidodecahedron"], "Archimedean solid", "20 triangles, 30 squares, 12 pentagons", 62, 120, 60, "(60+29*sqrt(5))/3", "30+5*sqrt(3)+3*sqrt(25+10*sqrt(5))", "sqrt(11+4*sqrt(5))/2", "", "", "deltoidal hexecontahedron", "I_h", "3.4.5.4"),
    ph!(&["truncated icosidodecahedron", "great rhombicosidodecahedron"], "Archimedean solid", "30 squares, 20 hexagons, 12 decagons", 62, 180, 120, "95+50*sqrt(5)", "30*(1+sqrt(3)+sqrt(5+2*sqrt(5)))", "sqrt(31+12*sqrt(5))/2", "", "", "disdyakis triacontahedron", "I_h", "4.6.10"),
    ph!(&["snub dodecahedron", "snub icosidodecahedron"], "Archimedean solid", "80 triangles, 12 pentagons", 92, 150, 60, "37.61664996", "20*sqrt(3)+3*sqrt(25+10*sqrt(5))", "2.155837375", "", "", "pentagonal hexecontahedron", "I", "3.3.3.3.5"),
];

fn find_poly(name: &str) -> Option<&'static Poly> {
    POLYHEDRA.iter().find(|p| p.names.contains(&name))
}

/// Edge length given as "edge 2", "edge length 3", "side 1/2", "a = 2".
fn edge_param(rest: &str) -> Option<Option<Expr>> {
    let rest = rest.trim().trim_start_matches([',', ';']).trim();
    if rest.is_empty() {
        return Some(None);
    }
    let m = re!(r"^(?:with\s+|of\s+)?(?:(?:unit|an?)\s+)?(?:edge|side)(?:\s+length)?\s*(?:=|:|of|is)?\s*(\S+)$|^(?:with\s+)?a\s*=\s*(\S+)$").captures(rest)?;
    num_arg(m.get(1).or(m.get(2))?.as_str()).map(Some)
}

/// coef · a^k: a number when the edge is numeric, else "coef aᵏ ≈ dec aᵏ".
fn scaled(coef: &Expr, a: &Expr, k: i64) -> String {
    if a.free_symbols().is_empty() {
        return show(&s::simplify(&mul(vec![coef.clone(), pow(a.clone(), int(k))])));
    }
    let c = s::simplify(coef);
    let ak = match k {
        1 => ptext(a),
        2 => format!("{}²", ptext(a)),
        3 => format!("{}³", ptext(a)),
        _ => format!("{}⁴", ptext(a)),
    };
    if c.is_one() {
        return ak;
    }
    let ct = ptext(&c);
    let ct = if matches!(c, Expr::Num(_)) || !ct.contains(' ') { ct } else { format!("({ct})") };
    match c.to_f64_const() {
        Some(v) if !c.is_integer() && !c.has_float() => format!("{ct} {ak} ≈ {} {ak}", fmt(v)),
        _ => format!("{ct} {ak}"),
    }
}

fn poly_props(p: &Poly, a: &Expr) -> Vec<(&'static str, String)> {
    let scale = |t: &str, k: i64| scaled(&ex(t), a, k);
    let mut v = vec![
        ("class", p.class.to_string()),
        ("faces", format!("{} ({})", p.f, p.faces)),
        ("edges", p.e.to_string()),
        ("vertices", p.v.to_string()),
        ("Euler characteristic", format!("{}", p.v as i64 - p.e as i64 + p.f as i64)),
        ("vertex configuration", p.vertex_config.to_string()),
        ("volume", scale(p.vol, 3)),
        ("surface area", scale(p.area, 2)),
        ("circumradius", scale(p.circum, 1)),
    ];
    // midradius: sqrt(R² − a²/4)
    let mid = if p.circum.contains('.') { float((ex(p.circum).to_f64_const().unwrap_or(f64::NAN).powi(2) - 0.25).sqrt()) } else { s::simplify(&s::denest(&func("sqrt", vec![s::simplify(&s::expand(&sub(square(ex(p.circum)), ex("1/4"))))]))) };
    v.push(("midradius", scaled(&mid, a, 1)));
    if !p.inradius.is_empty() {
        v.push(("inradius", scale(p.inradius, 1)));
    }
    if !p.dihedral.is_empty() {
        let d = ex(p.dihedral);
        let deg = d.to_f64_const().map(|r| r.to_degrees()).unwrap_or(f64::NAN);
        let exact = s::simplify(&d);
        v.push(("dihedral angle", format!("{} ≈ {}°", ptext(&exact), fmt(deg))));
    }
    v.push(("dual polyhedron", p.dual.to_string()));
    v.push(("symmetry group", p.group.to_string()));
    v
}

fn polyhedra(q: &Query) -> Option<Answer> {
    let l = tidy(q);
    // lists
    if let Some(m) = re!(r"^(?:list of\s+)?(?:the\s+)?(platonic|archimedean)\s+solids$").captures(&l) {
        let class = if &m[1] == "platonic" { "Platonic solid" } else { "Archimedean solid" };
        let rows: Vec<Vec<Content>> = POLYHEDRA.iter().filter(|p| p.class == class).map(|p| vec![text(p.names[0]), text(p.faces), text(p.f.to_string()), text(p.e.to_string()), text(p.v.to_string()), text(p.vertex_config)]).collect();
        let n = rows.len();
        return Some(Answer::new(format!("{class}s"), vec![input(format!("{class}s")), primary(pod("Result", &format!("{n} {class}s"), table(Some(vec!["name", "faces", "F", "E", "V", "vertex configuration"]), rows)))]));
    }
    // comparisons: "icosahedron, truncated icosahedron"
    let parts: Vec<&str> = l.split([',', ';']).map(str::trim).filter(|s| !s.is_empty()).flat_map(|s| s.split(" vs ")).flat_map(|s| s.split(" versus ")).map(str::trim).collect();
    if parts.len() >= 2 && parts.iter().all(|s| find_poly(s).is_some()) {
        let ps: Vec<&Poly> = parts.iter().filter_map(|s| find_poly(s)).collect();
        let a = one();
        let props: Vec<Vec<(&str, String)>> = ps.iter().map(|p| poly_props(p, &a)).collect();
        let mut header = vec![""];
        header.extend(ps.iter().map(|p| p.names[0]));
        let rows: Vec<Vec<Content>> = props[0].iter().enumerate().map(|(i, (k, _))| {
            let mut r = vec![text(*k)];
            r.extend(props.iter().map(|pr| text(pr.get(i).map(|x| x.1.clone()).unwrap_or_default())));
            r
        }).collect();
        return Some(Answer::new(parts.join(" vs "), vec![input(format!("{} (unit edge)", ps.iter().map(|p| p.names[0]).collect::<Vec<_>>().join(" | "))), primary(pod("Result", "Properties", table(Some(header), rows)))]));
    }
    // one polyhedron, optionally with an edge length
    let (p, rest) = POLYHEDRA.iter().flat_map(|p| p.names.iter().map(move |n| (p, *n))).filter(|(_, n)| l.starts_with(n)).max_by_key(|(_, n)| n.len()).map(|(p, n)| (p, &l[n.len()..]))?;
    // the cube has its own handler for "cube with side 2"
    let edge = edge_param(rest)?;
    if p.names[0] == "cube" {
        return None;
    }
    let a = edge.clone().unwrap_or_else(|| sym("a"));
    let label = match &edge {
        Some(e) => format!("{} | edge length {}", p.names[0], ptext(e)),
        None => format!("{} | edge length a", p.names[0]),
    };
    Some(Answer::new(label.clone(), vec![input(label), primary(pod("Result", "Properties", rows(poly_props(p, &a))))]))
}

// ---------------------------------------------------------------------------
// 4-polytopes, cross polytopes, glome
// ---------------------------------------------------------------------------

struct Polytope {
    names: &'static [&'static str],
    schlafli: &'static str,
    cells: &'static str,
    c: u32,
    f: u32,
    e: u32,
    v: u32,
    hypervolume: &'static str,
    circum: &'static str,
    dual: &'static str,
}

static POLYTOPES: &[Polytope] = &[
    Polytope { names: &["5-cell", "pentachoron", "4-simplex", "pentatope", "hypertetrahedron"], schlafli: "{3,3,3}", cells: "tetrahedra", c: 5, f: 10, e: 10, v: 5, hypervolume: "sqrt(5)/96", circum: "sqrt(10)/5", dual: "5-cell (self-dual)" },
    Polytope { names: &["tesseract", "8-cell", "hypercube", "4-cube", "octachoron"], schlafli: "{4,3,3}", cells: "cubes", c: 8, f: 24, e: 32, v: 16, hypervolume: "1", circum: "1", dual: "16-cell" },
    Polytope { names: &["16-cell", "hexadecachoron", "4-orthoplex", "4-dimensional cross polytope", "hyperoctahedron"], schlafli: "{3,3,4}", cells: "tetrahedra", c: 16, f: 32, e: 24, v: 8, hypervolume: "1/6", circum: "sqrt(2)/2", dual: "tesseract" },
    Polytope { names: &["24-cell", "icositetrachoron", "octaplex"], schlafli: "{3,4,3}", cells: "octahedra", c: 24, f: 96, e: 96, v: 24, hypervolume: "2", circum: "1", dual: "24-cell (self-dual)" },
    Polytope { names: &["120-cell", "hecatonicosachoron", "dodecaplex"], schlafli: "{5,3,3}", cells: "dodecahedra", c: 120, f: 720, e: 1200, v: 600, hypervolume: "15*(105+47*sqrt(5))/4", circum: "sqrt(2)*(3+sqrt(5))/2", dual: "600-cell" },
    Polytope { names: &["600-cell", "hexacosichoron", "tetraplex"], schlafli: "{3,3,5}", cells: "tetrahedra", c: 600, f: 1200, e: 720, v: 120, hypervolume: "25*(2+sqrt(5))/4", circum: "(1+sqrt(5))/2", dual: "120-cell" },
];

fn factorial(n: u32) -> u64 {
    (1..=n as u64).product()
}

fn polytopes(q: &Query) -> Option<Answer> {
    let l = tidy(q);
    // the glome (3-sphere)
    if let Some(m) = re!(r"^(?:glome|3-sphere|hypersphere|4-ball|4-dimensional (?:hyper)?sphere)(?:\s+(?:of|with)\s+radius\s+(\S+)|\s*,?\s*r\s*=\s*(\S+))?$").captures(&l) {
        let r = m.get(1).or(m.get(2)).and_then(|x| num_arg(x.as_str())).unwrap_or_else(|| sym("r"));
        let hv = s::simplify(&mul(vec![ex("pi^2/2"), pow(r.clone(), int(4))]));
        let ha = s::simplify(&mul(vec![ex("2*pi^2"), pow(r.clone(), int(3))]));
        let label = format!("glome (3-sphere) | radius {}", ptext(&r));
        return Some(Answer::new(label.clone(), vec![input(label), primary(pod("Result", "Properties", rows(vec![("hypervolume (4-volume enclosed)", show(&hv)), ("surface volume (3-volume)", show(&ha)), ("equation", "x² + y² + z² + w² = r²".into()), ("Euler characteristic", "0".into())])))]));
    }
    // n-dimensional cross polytopes, hypercubes and simplices
    if let Some(m) = re!(r"^(?:(\d+)[- ]?(?:d|dimensional)\s+)?(cross[- ]polytope|orthoplex|hypercube|measure polytope|simplex)$").captures(&l) {
        let kind = if m[2].starts_with("cross") || &m[2] == "orthoplex" { 0 } else if &m[2] == "simplex" { 2 } else { 1 };
        let ns: Vec<u32> = match m.get(1) {
            Some(n) => vec![n.as_str().parse().ok().filter(|n| (1..=20).contains(n))?],
            None if kind == 1 && &m[2] == "hypercube" => return None,
            None => (2..=8).collect(),
        };
        let name = ["cross polytope", "hypercube", "simplex"][kind];
        let row = |n: u32| -> Vec<Content> {
            let nn = n as i64;
            let (v, e, facets, vol) = match kind {
                0 => (2 * nn, 2 * nn * (nn - 1), 1i64 << n, s::simplify(&div(pow(func("sqrt", vec![int(2)]), int(nn)), int(factorial(n) as i64)))),
                1 => (1i64 << n, nn * (1i64 << (n - 1)), 2 * nn, one()),
                _ => (nn + 1, (nn + 1) * nn / 2, nn + 1, s::simplify(&div(func("sqrt", vec![int(nn + 1)]), mul(vec![int(factorial(n) as i64), func("sqrt", vec![pow(int(2), int(nn))])])))),
            };
            vec![text(n.to_string()), text(v.to_string()), text(e.to_string()), text(facets.to_string()), text(show(&vol))]
        };
        let formulas = match kind {
            0 => "vertices 2n · edges 2n(n − 1) · facets 2ⁿ simplices · volume (√2 a)ⁿ / n!",
            1 => "vertices 2ⁿ · edges n 2ⁿ⁻¹ · facets 2n cubes · volume aⁿ",
            _ => "vertices n + 1 · edges n(n + 1)/2 · facets n + 1 · volume √(n + 1) aⁿ / (n! √(2ⁿ))",
        };
        let label = match m.get(1) {
            Some(n) => format!("{}-dimensional {name}", n.as_str()),
            None => name.to_string(),
        };
        return Some(Answer::new(label.clone(), vec![input(label), primary(pod("Result", "Counts and volume (unit edge)", table(Some(vec!["dimension n", "vertices", "edges", "facets", "volume"]), ns.into_iter().map(row).collect()))), pod("Formulas", "Formulas", text(formulas))]));
    }
    let (p, rest) = POLYTOPES.iter().flat_map(|p| p.names.iter().map(move |n| (p, *n))).filter(|(_, n)| l.starts_with(n)).max_by_key(|(_, n)| n.len()).map(|(p, n)| (p, &l[n.len()..]))?;
    let edge = edge_param(rest)?;
    let a = edge.clone().unwrap_or_else(|| sym("a"));
    let hv = scaled(&ex(p.hypervolume), &a, 4);
    let r = scaled(&ex(p.circum), &a, 1);
    let label = format!("{} | edge length {}", p.names[0], ptext(&a));
    let props = vec![
        ("class", "regular 4-polytope".to_string()),
        ("Schläfli symbol", p.schlafli.to_string()),
        ("cells", format!("{} {}", p.c, p.cells)),
        ("faces", p.f.to_string()),
        ("edges", p.e.to_string()),
        ("vertices", p.v.to_string()),
        ("Euler characteristic", format!("{}", p.v as i64 - p.e as i64 + p.f as i64 - p.c as i64)),
        ("hypervolume", hv),
        ("circumradius", r),
        ("dual polytope", p.dual.to_string()),
    ];
    Some(Answer::new(label.clone(), vec![input(label), primary(pod("Result", "Properties", rows(props)))]))
}

// ---------------------------------------------------------------------------
// other solids and plane shapes with formulas
// ---------------------------------------------------------------------------

struct Shape {
    names: &'static [&'static str],
    title: &'static str,
    /// (symbol, description, words that introduce it)
    params: &'static [(&'static str, &'static str, &'static [&'static str])],
    props: &'static [(&'static str, &'static str)],
}

static SHAPES: &[Shape] = &[
    Shape { names: &["spherical cap"], title: "spherical cap", params: &[("r", "sphere radius", &["sphere radius", "radius", "r"]), ("h", "cap height", &["height", "h"])], props: &[("volume", "pi*h^2*(3*r-h)/3"), ("curved surface area", "2*pi*r*h"), ("base radius", "sqrt(h*(2*r-h))"), ("total surface area", "2*pi*r*h + pi*h*(2*r-h)")] },
    Shape { names: &["steinmetz 2-solid", "steinmetz solid", "bicylinder", "steinmetz bicylinder"], title: "Steinmetz solid (bicylinder)", params: &[("r", "cylinder radius", &["radius", "r"])], props: &[("volume", "16*r^3/3"), ("surface area", "16*r^2")] },
    Shape { names: &["steinmetz 3-solid", "tricylinder", "steinmetz tricylinder"], title: "Steinmetz solid (tricylinder)", params: &[("r", "cylinder radius", &["radius", "r"])], props: &[("volume", "8*(2-sqrt(2))*r^3"), ("surface area", "24*(2-sqrt(2))*r^2")] },
    Shape { names: &["reuleaux triangle"], title: "Reuleaux triangle", params: &[("w", "width", &["width", "side", "w", "s"])], props: &[("area", "(pi-sqrt(3))*w^2/2"), ("perimeter", "pi*w"), ("interior angle at each corner", "120 degrees (2π/3)")] },
    Shape { names: &["pseudosphere", "tractricoid"], title: "pseudosphere", params: &[("a", "radius", &["radius", "a", "r"])], props: &[("volume", "2*pi*a^3/3"), ("surface area", "4*pi*a^2"), ("Gaussian curvature", "-1/a^2")] },
    Shape { names: &["ellipsoid"], title: "ellipsoid", params: &[("a", "semi-axis a", &["a"]), ("b", "semi-axis b", &["b"]), ("c", "semi-axis c", &["c"])], props: &[("volume", "4*pi*a*b*c/3")] },
    Shape { names: &["spherical shell", "hollow sphere"], title: "spherical shell", params: &[("R", "outer radius", &["outer radius", "r_outer", "R"]), ("r", "inner radius", &["inner radius", "r_inner", "r"])], props: &[("volume", "4*pi*(R^3-r^3)/3")] },
];

fn shape_param(rest: &str, words: &[&str]) -> Option<Expr> {
    for w in words {
        let pat = format!(r"(?:^|[\s,;(])(?i:{})\s*(?:=|:|of|is)?\s*([-\d.]+(?:/\d+)?(?:\s*pi)?)", regex::escape(w));
        if let Some(m) = Regex::new(&pat).ok()?.captures(rest) {
            return num_arg(&m[1]);
        }
    }
    None
}

fn named_shapes(q: &Query) -> Option<Answer> {
    let l = tidy(q);
    let raw = q.clean.trim().trim_end_matches(['?', '.']).to_string();
    let (sh, rest_len) = SHAPES.iter().flat_map(|s| s.names.iter().map(move |n| (s, *n))).filter(|(_, n)| l.starts_with(n)).max_by_key(|(_, n)| n.len()).map(|(s, n)| (s, l.len() - n.len()))?;
    let rest = &raw[raw.len().saturating_sub(rest_len)..];
    if !rest.trim().is_empty() && !re!(r"(?i)^\s*(?:,|with|of|where|;)?.*[=\d]").is_match(rest) {
        return None;
    }
    let mut pairs = Vec::new();
    let mut given = Vec::new();
    for (sy, desc, words) in sh.params {
        let v = shape_param(rest, words);
        if let Some(v) = &v {
            given.push(format!("{desc} {}", ptext(v)));
            pairs.push((sym(sy), v.clone()));
        }
    }
    if !rest.trim().is_empty() && given.is_empty() {
        return None;
    }
    let props: Vec<(&str, String)> = sh.props.iter().map(|(k, f)| {
        if f.contains("degrees") {
            return (*k, f.to_string());
        }
        (*k, show(&s::simplify(&ex(f).subs_many(&pairs))))
    }).collect();
    let mut label = sh.title.to_string();
    if !given.is_empty() {
        label = format!("{label} | {}", given.join(" | "));
    } else {
        label = format!("{label} | {}", sh.params.iter().map(|(sy, d, _)| format!("{d} {sy}")).collect::<Vec<_>>().join(" | "));
    }
    Some(Answer::new(label.clone(), vec![input(label), primary(pod("Result", "Properties", rows(props)))]))
}

// ---------------------------------------------------------------------------
// triangles from three sides
// ---------------------------------------------------------------------------

fn triangle_sides(q: &Query) -> Option<Answer> {
    let l = tidy(q);
    let n = r"(\d+(?:\.\d+)?)";
    let pats = [
        format!(r"^{n}\s*,\s*{n}\s*,\s*(?:and\s+)?{n}\s+(?:right\s+)?triangle$"),
        format!(r"^{n}\s*-\s*{n}\s*-\s*{n}\s+(?:right\s+)?triangle$"),
        format!(r"^(?:right\s+)?triangle\s*(?:,|with)?\s*(?:side lengths|sides|side|lengths|edges)?\s*(?:of\s+)?{n}\s*,?\s*{n}\s*,?\s*(?:and\s+)?{n}$"),
        format!(r"^(?:right\s+)?triangle\s*(?:,|with)?\s*(?:side lengths|sides|lengths)?\s*\(\s*{n}\s*,\s*{n}\s*,\s*{n}\s*\)$"),
    ];
    let m = pats.iter().find_map(|p| Regex::new(p).ok()?.captures(&l).map(|m| [m[1].to_string(), m[2].to_string(), m[3].to_string()]))?;
    let sides: Vec<Expr> = m.iter().map(|t| ex(t)).collect();
    let v: Vec<f64> = sides.iter().map(|e| e.to_f64_const()).collect::<Option<_>>()?;
    let (a, b, c) = (v[0], v[1], v[2]);
    if a <= 0.0 || b <= 0.0 || c <= 0.0 {
        return None;
    }
    let label = format!("triangle | sides {}, {}, {}", m[0], m[1], m[2]);
    if a + b <= c || a + c <= b || b + c <= a {
        return Some(Answer::new(label.clone(), vec![input(label), primary(pod("Result", "Result", text("no such triangle: the triangle inequality fails (the longest side must be shorter than the sum of the other two)")))]));
    }
    let (sa, sb, sc) = (sides[0].clone(), sides[1].clone(), sides[2].clone());
    let sp = s::simplify(&div(add(vec![sa.clone(), sb.clone(), sc.clone()]), int(2)));
    let area = s::simplify(&func("sqrt", vec![mul(vec![sp.clone(), sub(sp.clone(), sa.clone()), sub(sp.clone(), sb.clone()), sub(sp.clone(), sc.clone())])]));
    let perim = s::simplify(&add(vec![sa.clone(), sb.clone(), sc.clone()]));
    let inr = s::simplify(&div(area.clone(), sp.clone()));
    let circ = s::simplify(&div(mul(vec![sa.clone(), sb.clone(), sc.clone()]), mul(vec![int(4), area.clone()])));
    let angle = |x: f64, y: f64, z: f64| ((y * y + z * z - x * x) / (2.0 * y * z)).clamp(-1.0, 1.0).acos().to_degrees();
    let angs = [angle(a, b, c), angle(b, a, c), angle(c, a, b)];
    let deg = |d: f64| if (d - d.round()).abs() < 1e-9 { format!("{}°", d.round()) } else { format!("{}°", fmt(d)) };
    let mx = angs.iter().cloned().fold(0.0, f64::max);
    let kind = if (mx - 90.0).abs() < 1e-9 { "right" } else if mx > 90.0 { "obtuse" } else { "acute" };
    let eq = |x: f64, y: f64| (x - y).abs() < 1e-12;
    let sides_kind = if eq(a, b) && eq(b, c) { "equilateral" } else if eq(a, b) || eq(b, c) || eq(a, c) { "isosceles" } else { "scalene" };
    let height = |side: &Expr| show(&s::simplify(&div(mul(vec![int(2), area.clone()]), side.clone())));
    let props = vec![
        ("classification", format!("{kind} {sides_kind} triangle")),
        ("angles", format!("{} | {} | {} (opposite the sides in order)", deg(angs[0]), deg(angs[1]), deg(angs[2]))),
        ("area", show(&area)),
        ("perimeter", show(&perim)),
        ("semiperimeter", show(&sp)),
        ("inradius", show(&inr)),
        ("circumradius", show(&circ)),
        ("heights", format!("{} | {} | {}", height(&sa), height(&sb), height(&sc))),
    ];
    Some(Answer::new(label.clone(), vec![input(label), primary(pod("Result", "Properties", rows(props)))]))
}

// ---------------------------------------------------------------------------
// named curves
// ---------------------------------------------------------------------------

struct Curve {
    names: &'static [&'static str],
    title: &'static str,
    /// parametric components in t with parameter a (b for a second one)
    param: &'static [&'static str],
    range: (&'static str, &'static str),
    equations: &'static [(&'static str, &'static str)],
    /// closed forms (in a); empty when none
    area: &'static str,
    length: &'static str,
    length_note: &'static str,
    notes: &'static str,
}

static CURVES: &[Curve] = &[
    Curve { names: &["cardioid"], title: "cardioid", param: &["a*(1+cos(t))*cos(t)", "a*(1+cos(t))*sin(t)"], range: ("0", "2*pi"), equations: &[("polar", "r = a (1 + cos θ)"), ("Cartesian", "(x² + y² − a x)² = a² (x² + y²)")], area: "3*pi*a^2/2", length: "8*a", length_note: "", notes: "epicycloid with one cusp: traced by a point on a circle of radius a/2 rolling around a circle of radius a/2" },
    Curve { names: &["astroid", "tetracuspid"], title: "astroid", param: &["a*cos(t)^3", "a*sin(t)^3"], range: ("0", "2*pi"), equations: &[("Cartesian", "x^(2/3) + y^(2/3) = a^(2/3)")], area: "3*pi*a^2/8", length: "6*a", length_note: "", notes: "hypocycloid with four cusps" },
    Curve { names: &["deltoid", "tricuspoid", "steiner curve", "deltoid curve"], title: "deltoid", param: &["2*a*cos(t)+a*cos(2*t)", "2*a*sin(t)-a*sin(2*t)"], range: ("0", "2*pi"), equations: &[("Cartesian", "(x² + y²)² + 18 a² (x² + y²) − 27 a⁴ = 8 a (x³ − 3 x y²)")], area: "2*pi*a^2", length: "16*a", length_note: "", notes: "hypocycloid with three cusps (rolling circle radius a inside a circle of radius 3a)" },
    Curve { names: &["nephroid"], title: "nephroid", param: &["3*a*cos(t)-a*cos(3*t)", "3*a*sin(t)-a*sin(3*t)"], range: ("0", "2*pi"), equations: &[("Cartesian", "(x² + y² − 4 a²)³ = 108 a⁴ y²")], area: "12*pi*a^2", length: "24*a", length_note: "", notes: "epicycloid with two cusps" },
    Curve { names: &["ranunculoid"], title: "ranunculoid", param: &["6*a*cos(t)-a*cos(6*t)", "6*a*sin(t)-a*sin(6*t)"], range: ("0", "2*pi"), equations: &[], area: "42*pi*a^2", length: "48*a", length_note: "", notes: "epicycloid with five cusps (rolling circle radius a around a circle of radius 5a)" },
    Curve { names: &["lemniscate", "lemniscate of bernoulli", "bernoulli lemniscate"], title: "lemniscate of Bernoulli", param: &["a*cos(t)/(1+sin(t)^2)", "a*sin(t)*cos(t)/(1+sin(t)^2)"], range: ("0", "2*pi"), equations: &[("polar", "r² = a² cos 2θ"), ("Cartesian", "(x² + y²)² = a² (x² − y²)")], area: "a^2", length: "gamma(1/4)^2*a/sqrt(2*pi)", length_note: "(2 ϖ a, ϖ the lemniscate constant)", notes: "" },
    Curve { names: &["cycloid"], title: "cycloid (one arch)", param: &["a*(t-sin(t))", "a*(1-cos(t))"], range: ("0", "2*pi"), equations: &[], area: "3*pi*a^2", length: "8*a", length_note: "", notes: "traced by a point on a circle of radius a rolling along a line; area is under one arch" },
    Curve { names: &["bicorn", "bicorn curve", "cocked hat curve"], title: "bicorn", param: &["a*sin(t)", "a*cos(t)^2*(2+cos(t))/(3+sin(t)^2)"], range: ("0", "2*pi"), equations: &[("Cartesian", "y² (a² − x²) = (x² + 2 a y − a²)²")], area: "", length: "", length_note: "", notes: "" },
    Curve { names: &["logarithmic spiral", "equiangular spiral", "growth spiral"], title: "logarithmic spiral", param: &[], range: ("", ""), equations: &[("polar", "r = a e^(b θ)"), ("parametric", "x = a e^(b t) cos t, y = a e^(b t) sin t")], area: "", length: "", length_note: "", notes: "the tangent makes the constant angle arccot(b) with the radius; arc length from the pole to radius r is r √(1 + b²) / b" },
    Curve { names: &["archimedean spiral", "archimedes spiral", "spiral of archimedes"], title: "Archimedean spiral", param: &[], range: ("", ""), equations: &[("polar", "r = a θ")], area: "", length: "", length_note: "", notes: "successive turns are a constant distance 2π a apart; area swept in the first turn is 4π³a²/3" },
    Curve { names: &["folium of descartes", "folium"], title: "folium of Descartes", param: &[], range: ("", ""), equations: &[("Cartesian", "x³ + y³ = 3 a x y"), ("parametric", "x = 3 a t / (1 + t³), y = 3 a t² / (1 + t³)")], area: "", length: "", length_note: "", notes: "area of the loop 3a²/2; asymptote x + y + a = 0" },
    Curve { names: &["viviani's curve", "vivianis curve", "viviani curve", "viviani's window"], title: "Viviani's curve", param: &["a*(1+cos(t))", "a*sin(t)", "2*a*sin(t/2)"], range: ("-2*pi", "2*pi"), equations: &[("as an intersection", "x² + y² + z² = 4 a², (x − a)² + y² = a²")], area: "", length: "", length_note: "", notes: "intersection of a sphere of radius 2a with a cylinder of radius a through its centre" },
    Curve { names: &["helix", "circular helix"], title: "circular helix", param: &["a*cos(t)", "a*sin(t)", "b*t"], range: ("0", "2*pi"), equations: &[], area: "", length: "2*pi*sqrt(a^2+b^2)", length_note: "per turn", notes: "curvature a/(a² + b²), torsion b/(a² + b²)" },
    Curve { names: &["conical spiral", "conical helix", "conic spiral"], title: "conical spiral", param: &["a*t*cos(t)", "a*t*sin(t)", "b*t"], range: ("0", "2*pi"), equations: &[], area: "", length: "", length_note: "", notes: "lies on the cone b² (x² + y²) = a² z²" },
    Curve { names: &["trefoil curve", "trefoil knot curve"], title: "trefoil knot curve", param: &["sin(t)+2*sin(2*t)", "cos(t)-2*cos(2*t)", "-sin(3*t)"], range: ("0", "2*pi"), equations: &[], area: "", length: "", length_note: "", notes: "" },
];

fn find_curve(name: &str) -> Option<&'static Curve> {
    let n = name.trim().trim_start_matches("the ").trim_start_matches("an ").trim_start_matches("a ");
    CURVES.iter().find(|c| c.names.contains(&n))
}

fn curve_numeric(c: &Curve, a: f64, b: f64) -> Option<(f64, f64)> {
    let comps: Vec<Expr> = c.param.iter().map(|t| ex(t)).collect();
    let progs: Vec<crate::eval::Program> = comps.iter().map(|e| crate::eval::Program::compile(e, &["t", "a", "b"])).collect::<Option<_>>()?;
    let ders: Vec<crate::eval::Program> = comps.iter().map(|e| crate::eval::Program::compile(&s::simplify(&crate::calculus::diff(e, &sym("t"))), &["t", "a", "b"])).collect::<Option<_>>()?;
    let (t0, t1) = (ex(c.range.0).to_f64_const()?, ex(c.range.1).to_f64_const()?);
    let len = crate::numeric::quad(&|t| ders.iter().map(|d| d.eval(&[t, a, b]).powi(2)).sum::<f64>().sqrt(), t0, t1, 1e-10).value;
    let area = if progs.len() == 2 {
        let w = |t: f64| 0.5 * (progs[0].eval(&[t, a, b]) * ders[1].eval(&[t, a, b]) - progs[1].eval(&[t, a, b]) * ders[0].eval(&[t, a, b]));
        let signed = crate::numeric::quad(&w, t0, t1, 1e-10).value.abs();
        let unsigned = crate::numeric::quad(&|t| w(t).abs(), t0, t1, 1e-10).value;
        // lobes traced in opposite senses cancel (lemniscate): use |x y′ − y x′|
        if signed < 1e-6 * unsigned { unsigned } else { signed }
    } else {
        f64::NAN
    };
    Some((area, len))
}

fn curves(q: &Query) -> Option<Answer> {
    let l = tidy(q);
    // comparisons: "deltoid vs ranunculoid"
    let parts: Vec<&str> = re!(r"\s+(?:vs\.?|versus|and)\s+|\s*,\s*").split(&l).collect();
    if parts.len() >= 2 && parts.iter().all(|p| find_curve(p).is_some()) {
        let cs: Vec<&Curve> = parts.iter().filter_map(|p| find_curve(p)).collect();
        let mut header = vec![""];
        header.extend(cs.iter().map(|c| c.title));
        let a = one();
        let cell = |f: &str| if f.is_empty() { "—".to_string() } else { show(&s::simplify(&ex(f).subs(&sym("a"), &a))) };
        let rows: Vec<Vec<Content>> = vec![
            { let mut r = vec![text("parametric (a = 1)")]; r.extend(cs.iter().map(|c| text(if c.param.is_empty() { "—".into() } else { format!("({})", c.param.iter().map(|t| ptext(&s::simplify(&ex(t).subs(&sym("a"), &a)))).collect::<Vec<_>>().join(", ")) }))); r },
            { let mut r = vec![text("area enclosed (a = 1)")]; r.extend(cs.iter().map(|c| text(cell(c.area)))); r },
            { let mut r = vec![text("arc length (a = 1)")]; r.extend(cs.iter().map(|c| text(cell(c.length)))); r },
            { let mut r = vec![text("description")]; r.extend(cs.iter().map(|c| text(c.notes))); r },
        ];
        return Some(Answer::new(parts.join(" vs "), vec![input(cs.iter().map(|c| c.title).collect::<Vec<_>>().join(" | ")), primary(pod("Result", "Comparison", table(Some(header), rows)))]));
    }
    let (want, name) = if let Some(m) = re!(r"^(arc length|length|perimeter|area(?: enclosed)?|area inside|unit tangent(?: vector)?|tangent vector|tangent|curvature)\s+(?:of\s+|by\s+)?(?:the\s+|an?\s+)?(.+)$").captures(&l) {
        (Some(m[1].to_string()), m[2].to_string())
    } else if let Some(m) = re!(r"^(.+?)\s+(area enclosed|area|arc length|length|perimeter|tangent vector|curvature)$").captures(&l) {
        (Some(m[2].to_string()), m[1].to_string())
    } else {
        (None, l.clone())
    };
    let c = find_curve(&name)?;
    let a = sym("a");
    let mut pods = vec![input(format!("{}{}", c.title, want.as_ref().map(|w| format!(" | {w}")).unwrap_or_default()))];
    let comps: Vec<Expr> = c.param.iter().map(|t| ex(t)).collect();
    let t = sym("t");
    match want.as_deref() {
        Some(w) if w.contains("tangent") || w == "curvature" => {
            if comps.is_empty() {
                return None;
            }
            let d: Vec<Expr> = comps.iter().map(|e| s::simplify(&crate::calculus::diff(e, &t))).collect();
            if w == "curvature" {
                if d.len() != 2 && d.len() != 3 {
                    return None;
                }
                let dd: Vec<Expr> = d.iter().map(|e| s::simplify(&crate::calculus::diff(e, &t))).collect();
                let k = if d.len() == 2 {
                    div(func("abs", vec![sub(mul(vec![d[0].clone(), dd[1].clone()]), mul(vec![d[1].clone(), dd[0].clone()]))]), pow(add(vec![square(d[0].clone()), square(d[1].clone())]), ex("3/2")))
                } else {
                    let cr = [sub(mul(vec![d[1].clone(), dd[2].clone()]), mul(vec![d[2].clone(), dd[1].clone()])), sub(mul(vec![d[2].clone(), dd[0].clone()]), mul(vec![d[0].clone(), dd[2].clone()])), sub(mul(vec![d[0].clone(), dd[1].clone()]), mul(vec![d[1].clone(), dd[0].clone()]))];
                    div(func("sqrt", vec![add(cr.iter().map(|x| square(x.clone())).collect())]), pow(add(d.iter().map(|x| square(x.clone())).collect()), ex("3/2")))
                };
                pods.push(primary(pod("Result", "Curvature", math(&s::simplify(&k)))));
            } else {
                let speed = s::simplify(&func("sqrt", vec![add(d.iter().map(|x| square(x.clone())).collect())]));
                let unit: Vec<Expr> = d.iter().map(|x| s::simplify(&div(x.clone(), speed.clone()))).collect();
                pods.push(primary(pod("Result", "Tangent vector r′(t)", math(&Expr::List(d.clone())))));
                pods.push(pod("Unit", "Unit tangent vector", math(&Expr::List(unit))));
                pods.push(pod("Speed", "Speed |r′(t)|", math(&speed)));
            }
            pods.push(pod("Parametric", "Parametric equations", math(&Expr::List(comps.clone()))));
        }
        Some(w) if w.contains("length") || w == "perimeter" => {
            if c.length.is_empty() {
                let (_, len) = curve_numeric(c, 1.0, 1.0)?;
                if comps.is_empty() || !len.is_finite() {
                    return None;
                }
                pods.push(primary(pod("Result", "Arc length (a = 1)", text(format!("≈ {}", fmt(len))))));
            } else {
                pods.push(primary(pod("Result", &format!("Arc length{}", if c.length_note.is_empty() { String::new() } else { format!(" {}", c.length_note) }), math(&s::simplify(&ex(c.length))))));
            }
        }
        Some(_) => {
            if c.area.is_empty() {
                if comps.len() != 2 {
                    return None;
                }
                let (area, _) = curve_numeric(c, 1.0, 1.0)?;
                let exact = crate::engine::math::identify_constant(area);
                pods.push(primary(pod("Result", "Area enclosed (a = 1)", text(match exact {
                    Some(e) => format!("{} ≈ {}", ptext(&e), fmt(area)),
                    None => format!("≈ {}", fmt(area)),
                }))));
            } else {
                pods.push(primary(pod("Result", "Area enclosed", math(&s::simplify(&ex(c.area))))));
            }
        }
        None => {
            let mut props: Vec<(&str, String)> = Vec::new();
            if !comps.is_empty() {
                props.push(("parametric", format!("({}), t from {} to {}", comps.iter().map(ptext).collect::<Vec<_>>().join(", "), ptext(&ex(c.range.0)), ptext(&ex(c.range.1)))));
            }
            for (k, v) in c.equations {
                props.push((k, v.to_string()));
            }
            if !c.area.is_empty() {
                props.push(("area enclosed", ptext(&s::simplify(&ex(c.area)))));
            } else if comps.len() == 2 {
                if let Some((ar, _)) = curve_numeric(c, 1.0, 1.0) {
                    props.push(("area enclosed (a = 1)", format!("≈ {}", fmt(ar))));
                }
            }
            if !c.length.is_empty() {
                props.push(("arc length", format!("{}{}", ptext(&s::simplify(&ex(c.length))), if c.length_note.is_empty() { String::new() } else { format!(" {}", c.length_note) })));
            } else if !comps.is_empty() && !c.param.iter().any(|p| p.contains('b')) {
                if let Some((_, len)) = curve_numeric(c, 1.0, 1.0) {
                    props.push(("arc length (a = 1)", format!("≈ {}", fmt(len))));
                }
            }
            if !c.notes.is_empty() {
                props.push(("description", c.notes.to_string()));
            }
            pods.push(primary(pod("Result", "Properties", rows(props))));
            if comps.len() >= 2 && !c.param.iter().any(|p| p.contains('b')) {
                let one_a: Vec<Expr> = comps.iter().map(|e| e.subs(&a, &one())).collect();
                let cmd = format!("parametric plot ({}) for t = {} to {}", one_a.iter().map(ptext).collect::<Vec<_>>().join(", "), ptext(&ex(c.range.0)), ptext(&ex(c.range.1)));
                let r = crate::engine::query_with(&cmd, q.opts);
                if let Some(p) = r.pods.into_iter().find(|p| p.subpods.iter().any(|s| matches!(s.content, Content::Plot { .. }))) {
                    let mut p = p;
                    p.primary = false;
                    p.title = "Plot (a = 1)".into();
                    pods.push(p);
                }
            }
        }
    }
    Some(Answer::new(c.title, pods))
}

// ---------------------------------------------------------------------------
// knots
// ---------------------------------------------------------------------------

struct Knot {
    names: &'static [&'static str],
    title: &'static str,
    crossing: u32,
    unknotting: u32,
    genus: u32,
    alexander: &'static str,
    jones: &'static str,
    determinant: u32,
    kind: &'static str,
    chirality: &'static str,
    volume: &'static str,
}

static KNOTS: &[Knot] = &[
    Knot { names: &["unknot", "trivial knot", "0_1"], title: "unknot (0₁)", crossing: 0, unknotting: 0, genus: 0, alexander: "1", jones: "1", determinant: 1, kind: "trivial knot", chirality: "amphichiral", volume: "" },
    Knot { names: &["trefoil", "trefoil knot", "3_1", "3_1 knot", "overhand knot", "torus knot (2,3)"], title: "trefoil knot (3₁)", crossing: 3, unknotting: 1, genus: 1, alexander: "t − 1 + t⁻¹", jones: "q⁻¹ + q⁻³ − q⁻⁴ (mirror image: q + q³ − q⁴)", determinant: 3, kind: "torus knot T(2,3), twist knot", chirality: "chiral", volume: "" },
    Knot { names: &["figure eight knot", "figure-eight knot", "figure 8 knot", "4_1", "4_1 knot", "listing's knot"], title: "figure-eight knot (4₁)", crossing: 4, unknotting: 1, genus: 1, alexander: "−t + 3 − t⁻¹", jones: "q² − q + 1 − q⁻¹ + q⁻²", determinant: 5, kind: "twist knot, hyperbolic", chirality: "amphichiral", volume: "2.029883212" },
    Knot { names: &["cinquefoil", "cinquefoil knot", "pentafoil knot", "5_1", "5_1 knot", "solomon's seal knot"], title: "cinquefoil knot (5₁)", crossing: 5, unknotting: 2, genus: 2, alexander: "t² − t + 1 − t⁻¹ + t⁻²", jones: "−q⁻⁷ + q⁻⁶ − q⁻⁵ + q⁻⁴ + q⁻²", determinant: 5, kind: "torus knot T(2,5)", chirality: "chiral", volume: "" },
    Knot { names: &["three-twist knot", "three twist knot", "5_2", "5_2 knot"], title: "three-twist knot (5₂)", crossing: 5, unknotting: 1, genus: 1, alexander: "2t − 3 + 2t⁻¹", jones: "−q⁻⁶ + q⁻⁵ − q⁻⁴ + 2q⁻³ − q⁻² + q⁻¹", determinant: 7, kind: "twist knot, hyperbolic", chirality: "chiral", volume: "2.828122088" },
    Knot { names: &["stevedore knot", "stevedore", "6_1", "6_1 knot"], title: "stevedore knot (6₁)", crossing: 6, unknotting: 1, genus: 1, alexander: "−2t + 5 − 2t⁻¹", jones: "q² − q + 2 − 2q⁻¹ + q⁻² − q⁻³ + q⁻⁴", determinant: 9, kind: "twist knot, slice knot, hyperbolic", chirality: "chiral", volume: "3.163963229" },
    Knot { names: &["6_2", "6_2 knot", "miller institute knot"], title: "6₂ knot", crossing: 6, unknotting: 1, genus: 2, alexander: "−t² + 3t − 3 + 3t⁻¹ − t⁻²", jones: "", determinant: 11, kind: "hyperbolic", chirality: "chiral", volume: "" },
    Knot { names: &["6_3", "6_3 knot"], title: "6₃ knot", crossing: 6, unknotting: 1, genus: 2, alexander: "t² − 3t + 5 − 3t⁻¹ + t⁻²", jones: "", determinant: 13, kind: "hyperbolic", chirality: "amphichiral", volume: "" },
    Knot { names: &["7_1", "7_1 knot", "septafoil knot", "heptafoil knot"], title: "septafoil knot (7₁)", crossing: 7, unknotting: 3, genus: 3, alexander: "t³ − t² + t − 1 + t⁻¹ − t⁻² + t⁻³", jones: "", determinant: 7, kind: "torus knot T(2,7)", chirality: "chiral", volume: "" },
    Knot { names: &["8_1", "8_1 knot"], title: "8₁ knot", crossing: 8, unknotting: 1, genus: 1, alexander: "−3t + 7 − 3t⁻¹", jones: "", determinant: 13, kind: "twist knot, hyperbolic", chirality: "chiral", volume: "" },
];

fn knot_props(k: &Knot) -> Vec<(&'static str, String)> {
    let mut v = vec![
        ("crossing number", k.crossing.to_string()),
        ("unknotting number", k.unknotting.to_string()),
        ("genus", k.genus.to_string()),
        ("Alexander polynomial", k.alexander.to_string()),
    ];
    if !k.jones.is_empty() {
        v.push(("Jones polynomial", k.jones.to_string()));
    }
    v.push(("determinant", k.determinant.to_string()));
    v.push(("type", k.kind.to_string()));
    v.push(("chirality", k.chirality.to_string()));
    if !k.volume.is_empty() {
        v.push(("hyperbolic volume", k.volume.to_string()));
    }
    v
}

fn knots(q: &Query) -> Option<Answer> {
    let l = tidy(q).replace("figure-eight", "figure eight").replace("knot ", "");
    let find = |n: &str| {
        let n = n.trim();
        KNOTS.iter().find(|k| k.names.contains(&n) || k.names.contains(&n.trim_end_matches(" knot")))
    };
    let parts: Vec<&str> = re!(r"\s*,\s*|\s+(?:vs\.?|versus|and)\s+").split(&l).collect();
    if parts.len() >= 2 && parts.iter().all(|p| find(p).is_some()) {
        let ks: Vec<&Knot> = parts.iter().filter_map(|p| find(p)).collect();
        let props: Vec<Vec<(&str, String)>> = ks.iter().map(|k| knot_props(k)).collect();
        let keys = ["crossing number", "unknotting number", "genus", "Alexander polynomial", "Jones polynomial", "determinant", "type", "chirality", "hyperbolic volume"];
        let mut header = vec![""];
        header.extend(ks.iter().map(|k| k.title));
        let rows: Vec<Vec<Content>> = keys.iter().filter(|key| props.iter().any(|p| p.iter().any(|(k, _)| k == *key))).map(|key| {
            let mut r = vec![text(*key)];
            r.extend(props.iter().map(|p| text(p.iter().find(|(k, _)| k == key).map(|x| x.1.clone()).unwrap_or_else(|| "—".into()))));
            r
        }).collect();
        return Some(Answer::new(parts.join(" vs "), vec![input(ks.iter().map(|k| k.title).collect::<Vec<_>>().join(" | ")), primary(pod("Result", "Comparison", table(Some(header), rows)))]));
    }
    let k = find(&l)?;
    let mut pods = vec![input(k.title.to_string()), primary(pod("Result", "Knot invariants", rows(knot_props(k))))];
    if k.crossing == 3 {
        let r = crate::engine::query_with("parametric plot (sin(t)+2 sin(2t), cos(t)-2 cos(2t), -sin(3t)) for t = 0 to 2pi", q.opts);
        if let Some(mut p) = r.pods.into_iter().find(|p| p.subpods.iter().any(|s| matches!(s.content, Content::Plot { .. }))) {
            p.primary = false;
            p.title = "Space curve (sin t + 2 sin 2t, cos t − 2 cos 2t, −sin 3t)".into();
            pods.push(p);
        }
    }
    Some(Answer::new(k.title, pods))
}

// ---------------------------------------------------------------------------
// homotopy groups of spheres
// ---------------------------------------------------------------------------

/// Stable stems π_{n+k}(Sⁿ), n > k + 1, for k = 0..19.
const STABLE: &[&str] = &["ℤ", "ℤ₂", "ℤ₂", "ℤ₂₄", "0", "0", "ℤ₂", "ℤ₂₄₀", "ℤ₂²", "ℤ₂³", "ℤ₆", "ℤ₅₀₄", "0", "ℤ₃", "ℤ₂²", "ℤ₄₈₀ ⊕ ℤ₂", "ℤ₂²", "ℤ₂⁴", "ℤ₈ ⊕ ℤ₂", "ℤ₂₆₄ ⊕ ℤ₂"];

/// π_k(S³) for k = 3..16 (also π_k(S²) for k ≥ 3).
const S3: &[&str] = &["ℤ", "ℤ₂", "ℤ₂", "ℤ₁₂", "ℤ₂", "ℤ₂", "ℤ₃", "ℤ₁₅", "ℤ₂", "ℤ₂²", "ℤ₁₂ ⊕ ℤ₂", "ℤ₈₄ ⊕ ℤ₂²", "ℤ₂²", "ℤ₆"];

/// Unstable entries π_{n+k}(Sⁿ) for 4 ≤ n ≤ k + 1, k ≤ 7.
fn unstable(n: u32, k: u32) -> Option<&'static str> {
    Some(match (k, n) {
        (3, 4) => "ℤ ⊕ ℤ₁₂",
        (4, 4) => "ℤ₂²",
        (4, 5) => "ℤ₂",
        (5, 4) => "ℤ₂²",
        (5, 5) => "ℤ₂",
        (5, 6) => "ℤ",
        (6, 4) => "ℤ₂₄ ⊕ ℤ₃",
        (6, 5) | (6, 6) | (6, 7) => "ℤ₂",
        (7, 4) => "ℤ₁₅",
        (7, 5) => "ℤ₃₀",
        (7, 6) => "ℤ₆₀",
        (7, 7) => "ℤ₁₂₀",
        (7, 8) => "ℤ ⊕ ℤ₁₂₀",
        _ => return None,
    })
}

fn homotopy_group(i: u32, n: u32) -> Option<String> {
    if n == 0 {
        return None;
    }
    if i == 0 {
        return Some(if n == 0 { "two points".into() } else { "0 (connected)".into() });
    }
    if i < n {
        return Some("0".into());
    }
    if i == n {
        return Some("ℤ".into());
    }
    if n == 1 {
        return Some("0".into());
    }
    if n == 2 {
        return S3.get((i - 3) as usize).map(|s| s.to_string());
    }
    if n == 3 {
        return S3.get((i - 3) as usize).map(|s| s.to_string());
    }
    let k = i - n;
    if n >= k + 2 {
        return STABLE.get(k as usize).map(|s| s.to_string());
    }
    unstable(n, k).map(str::to_string)
}

fn homotopy(q: &Query) -> Option<Answer> {
    let l = tidy(q).replace(['{', '}'], "").replace("π", "pi");
    if let Some(m) = re!(r"^(?:(?:the\s+)?homotopy groups?\s+(?:of\s+)?)?pi_?\s*(\d+)\s*\(\s*s\^?\s*(\d+)\s*\)$").captures(&l) {
        let (i, n): (u32, u32) = (m[1].parse().ok()?, m[2].parse().ok()?);
        let g = homotopy_group(i, n)?;
        let label = format!("π_{i}(S^{n})");
        let mut pods = vec![input(format!("homotopy group {label}")), primary(pod("Result", "Result", text(g)))];
        if i > n && n >= 2 && n >= i - n + 2 {
            pods.push(pod("Note", "Stable range", text(format!("n = {n} > k + 1 with k = {}: this is the stable stem π_{}^s", i - n, i - n))));
        }
        return Some(Answer::new(label, pods));
    }
    let m = re!(r"^(?:the\s+)?homotopy(?:\s+groups?)?\s+(?:of\s+)?(?:the\s+)?(?:s\^?(\d+)|(\d+)-sphere)$").captures(&l)?;
    let n: u32 = m.get(1).or(m.get(2))?.as_str().parse().ok()?;
    let rows: Vec<Vec<Content>> = (1..=n + 12).filter_map(|i| homotopy_group(i, n).map(|g| vec![text(format!("π_{i}(S^{n})")), text(g)])).collect();
    Some(Answer::new(format!("homotopy groups of S^{n}"), vec![input(format!("homotopy groups of the {n}-sphere")), primary(pod("Result", "Homotopy groups", table(None, rows)))]))
}

// ---------------------------------------------------------------------------
// intervals
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Interval {
    lo: f64,
    hi: f64,
    lc: bool,
    hc: bool,
}

fn endpoint(t: &str) -> Option<f64> {
    match t.trim() {
        "oo" | "inf" | "infinity" | "∞" | "+oo" | "+inf" | "+∞" => Some(f64::INFINITY),
        "-oo" | "-inf" | "-infinity" | "-∞" | "−∞" => Some(f64::NEG_INFINITY),
        x => crate::parse::parse(x).ok()?.expr.to_f64_const(),
    }
}

fn parse_intervals(t: &str) -> Option<Vec<Interval>> {
    let mut out = Vec::new();
    let mut rest = t.trim();
    let r = re!(r"^\s*([\[(])\s*([^,\[\]()]+?)\s*,\s*([^,\[\]()]+?)\s*([\])])\s*(?:,|and|∪|u|union)?\s*");
    while !rest.is_empty() {
        let m = r.captures(rest)?;
        let (lo, hi) = (endpoint(&m[2])?, endpoint(&m[3])?);
        if lo > hi {
            return None;
        }
        out.push(Interval { lo, hi, lc: &m[1] == "[" && lo.is_finite(), hc: &m[4] == "]" && hi.is_finite() });
        rest = &rest[m.get(0)?.end()..];
    }
    (!out.is_empty()).then_some(out)
}

fn ep(v: f64) -> String {
    if v == f64::INFINITY {
        "∞".into()
    } else if v == f64::NEG_INFINITY {
        "−∞".into()
    } else {
        fmt(v)
    }
}

fn iv_text(i: &Interval) -> String {
    if i.lo == i.hi && i.lc && i.hc {
        return format!("{{{}}}", ep(i.lo));
    }
    format!("{}{}, {}{}", if i.lc { '[' } else { '(' }, ep(i.lo), ep(i.hi), if i.hc { ']' } else { ')' })
}

fn iv_ineq(i: &Interval) -> String {
    match (i.lo.is_finite(), i.hi.is_finite()) {
        (true, true) => format!("{} {} x {} {}", ep(i.lo), if i.lc { "≤" } else { "<" }, if i.hc { "≤" } else { "<" }, ep(i.hi)),
        (true, false) => format!("x {} {}", if i.lc { "≥" } else { ">" }, ep(i.lo)),
        (false, true) => format!("x {} {}", if i.hc { "≤" } else { "<" }, ep(i.hi)),
        _ => "all real x".into(),
    }
}

fn intervals(q: &Query) -> Option<Answer> {
    let l = tidy(q);
    let (named, body) = match re!(r"^(?:the\s+)?intervals?\s+(.+)$").captures(&l) {
        Some(m) => (true, m[1].to_string()),
        None => (false, l.clone()),
    };
    let ivs = parse_intervals(&body)?;
    // a bare (a, b) is a point; a bare [a, b] may be a list
    if !named && ivs.len() == 1 && !(body.starts_with('[') ^ body.ends_with(']')) {
        return None;
    }
    if !named && ivs.len() > 1 {
        return None;
    }
    let mut pods = vec![input(format!("interval{} {}", if ivs.len() > 1 { "s" } else { "" }, ivs.iter().map(iv_text).collect::<Vec<_>>().join(", ")))];
    let describe = |i: &Interval| -> Vec<(&'static str, String)> {
        let bounded = i.lo.is_finite() && i.hi.is_finite();
        let open = (!i.lc) && (!i.hc);
        let closed = (i.lc || !i.lo.is_finite()) && (i.hc || !i.hi.is_finite());
        let mut v = vec![("inequality", iv_ineq(i))];
        v.push(("length", if bounded { fmt(i.hi - i.lo) } else { "∞".into() }));
        if bounded {
            v.push(("midpoint", fmt((i.lo + i.hi) / 2.0)));
        }
        v.push(("interior", iv_text(&Interval { lc: false, hc: false, ..i.clone() })));
        v.push(("closure", iv_text(&Interval { lc: i.lo.is_finite(), hc: i.hi.is_finite(), ..i.clone() })));
        let mut bd: Vec<String> = Vec::new();
        if i.lo.is_finite() {
            bd.push(ep(i.lo));
        }
        if i.hi.is_finite() && i.hi != i.lo {
            bd.push(ep(i.hi));
        }
        v.push(("boundary", if bd.is_empty() { "∅".into() } else { format!("{{{}}}", bd.join(", ")) }));
        let mut props = vec![if bounded { "bounded" } else { "unbounded" }];
        props.push(match (open, closed) {
            (true, true) => "open and closed",
            (true, false) => "open",
            (false, true) => "closed",
            _ => "neither open nor closed",
        });
        props.push(if bounded && closed { "compact" } else { "not compact" });
        props.push("connected");
        v.push(("properties", props.join(", ")));
        v
    };
    if ivs.len() == 1 {
        pods.push(primary(pod("Result", "Properties", rows(describe(&ivs[0])))));
    } else {
        // union and intersection of the intervals
        let mut sorted = ivs.clone();
        sorted.sort_by(|a, b| a.lo.partial_cmp(&b.lo).unwrap());
        let mut union: Vec<Interval> = Vec::new();
        for i in sorted {
            if let Some(last) = union.last_mut() {
                if i.lo < last.hi || (i.lo == last.hi && (last.hc || i.lc)) {
                    if i.hi > last.hi || (i.hi == last.hi && i.hc) {
                        last.hi = i.hi;
                        last.hc = i.hc || (i.hi == last.hi && last.hc && i.hi == last.hi && false);
                    }
                    continue;
                }
            }
            union.push(i);
        }
        let mut inter = ivs[0].clone();
        for i in &ivs[1..] {
            if i.lo > inter.lo || (i.lo == inter.lo && !i.lc) {
                inter.lo = i.lo;
                inter.lc = i.lc;
            }
            if i.hi < inter.hi || (i.hi == inter.hi && !i.hc) {
                inter.hi = i.hi;
                inter.hc = i.hc;
            }
        }
        let empty = inter.lo > inter.hi || (inter.lo == inter.hi && !(inter.lc && inter.hc));
        let total: f64 = union.iter().map(|i| i.hi - i.lo).sum();
        pods.push(primary(pod("Result", "Union", text(union.iter().map(iv_text).collect::<Vec<_>>().join(" ∪ ")))));
        pods.push(pod("Intersection", "Intersection", text(if empty { "∅".into() } else { iv_text(&inter) })));
        pods.push(pod("Measure", "Total length", text(if total.is_finite() { fmt(total) } else { "∞".into() })));
        for (k, i) in ivs.iter().enumerate() {
            let mut p = pod(&format!("Interval{k}"), &iv_text(i), rows(describe(i)));
            p.primary = false;
            pods.push(p);
        }
    }
    Some(Answer::new("interval", pods))
}

// ---------------------------------------------------------------------------
// polyforms
// ---------------------------------------------------------------------------

struct Polyform {
    stem: &'static str,
    plural: &'static str,
    cell: &'static str,
    free: &'static [u64],
    one_sided: &'static [u64],
    fixed: &'static [u64],
}

static POLYFORMS: &[Polyform] = &[
    Polyform { stem: "omino", plural: "polyominoes", cell: "squares", free: &[1, 1, 2, 5, 12, 35, 108, 369, 1285, 4655, 17073, 63600, 238591, 901971], one_sided: &[1, 1, 2, 7, 18, 60, 196, 704, 2500, 9189, 33896, 126759], fixed: &[1, 2, 6, 19, 63, 216, 760, 2725, 9910, 36446, 135268, 505861] },
    Polyform { stem: "iamond", plural: "polyiamonds", cell: "equilateral triangles", free: &[1, 1, 1, 3, 4, 12, 24, 66, 160, 448, 1186, 3334, 9235], one_sided: &[], fixed: &[] },
    Polyform { stem: "hex", plural: "polyhexes", cell: "regular hexagons", free: &[1, 1, 3, 7, 22, 82, 333, 1448, 6572, 30490], one_sided: &[], fixed: &[] },
    Polyform { stem: "cube", plural: "polycubes", cell: "cubes", free: &[1, 1, 2, 7, 23, 112, 607, 3811], one_sided: &[1, 1, 2, 8, 29, 166, 1023, 6922], fixed: &[] },
    Polyform { stem: "abolo", plural: "polyaboloes (polytans)", cell: "isosceles right triangles", free: &[1, 3, 4, 14, 30, 107, 318, 1116, 3743], one_sided: &[], fixed: &[] },
];

const SIZE_PREFIX: &[(&str, usize)] = &[("mon", 1), ("d", 2), ("tr", 3), ("tetr", 4), ("pent", 5), ("hex", 6), ("hept", 7), ("oct", 8), ("enne", 9), ("non", 9), ("dec", 10), ("undec", 11), ("dodec", 12)];

fn polyforms(q: &Query) -> Option<Answer> {
    let l = tidy(q);
    let l = re!(r"^(?:(?:how many|number of|count of|all)\s+)?(?:the\s+)?").replace(&l, "").trim().to_string();
    let l = l.trim_end_matches(" are there").trim_end_matches(" exist").trim();
    // "polyominoes", "polyominoes of size 6", "hexiamonds"
    let (pf, size) = if let Some(m) = re!(r"^poly(omino|iamond|hex|cube|abolo|tan)(?:e?s)?(?:\s+(?:of\s+)?(?:size|order|with)\s+(\d+)(?:\s+cells)?)?$").captures(l) {
        let stem = if &m[1] == "tan" { "abolo" } else { &m[1] };
        (POLYFORMS.iter().find(|p| p.stem == stem)?, m.get(2).and_then(|x| x.as_str().parse::<usize>().ok()))
    } else {
        let m = re!(r"^([a-z]+?)a?(omino|iamond|hex|cube|abolo)(?:e?s)?$").captures(l)?;
        let n = SIZE_PREFIX.iter().find(|(p, _)| *p == &m[1] || format!("{p}a") == m[1] || format!("{p}o") == m[1] || format!("{p}i") == m[1])?.1;
        if &m[2] == "hex" && n == 6 {
            return None;
        }
        (POLYFORMS.iter().find(|p| p.stem == &m[2])?, Some(n))
    };
    let mut pods = Vec::new();
    match size {
        Some(n) => {
            let free = *pf.free.get(n.checked_sub(1)?)?;
            pods.push(input(format!("{} with {n} cells", pf.plural)));
            pods.push(primary(pod("Result", "Number of free forms (rotations and reflections identified)", text(free.to_string()))));
            if let Some(v) = pf.one_sided.get(n - 1) {
                pods.push(pod("OneSided", "One-sided (reflections distinct)", text(v.to_string())));
            }
            if let Some(v) = pf.fixed.get(n - 1) {
                pods.push(pod("Fixed", "Fixed (rotations and reflections distinct)", text(v.to_string())));
            }
        }
        None => {
            pods.push(input(format!("{} (shapes made of {} joined edge to edge)", pf.plural, pf.cell)));
            let header = {
                let mut h = vec!["cells", "free"];
                if !pf.one_sided.is_empty() {
                    h.push("one-sided");
                }
                if !pf.fixed.is_empty() {
                    h.push("fixed");
                }
                h
            };
            let rows: Vec<Vec<Content>> = pf.free.iter().enumerate().map(|(i, f)| {
                let mut r = vec![text((i + 1).to_string()), text(f.to_string())];
                if !pf.one_sided.is_empty() {
                    r.push(text(pf.one_sided.get(i).map(|x| x.to_string()).unwrap_or_else(|| "—".into())));
                }
                if !pf.fixed.is_empty() {
                    r.push(text(pf.fixed.get(i).map(|x| x.to_string()).unwrap_or_else(|| "—".into())));
                }
                r
            }).collect();
            pods.push(primary(pod("Result", "Counts", table(Some(header), rows))));
        }
    }
    Some(Answer::new(pf.plural, pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    polyhedra(q).or_else(|| polytopes(q)).or_else(|| named_shapes(q)).or_else(|| triangle_sides(q)).or_else(|| curves(q)).or_else(|| knots(q)).or_else(|| homotopy(q)).or_else(|| intervals(q)).or_else(|| polyforms(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(s: &str) -> String {
        let r = crate::engine::query(s);
        serde_json::to_string(&r.pods).unwrap_or_default()
    }

    #[test]
    fn curve_closed_forms_match_numerics() {
        for c in CURVES {
            if c.param.is_empty() || c.param.iter().any(|p| p.contains('b')) {
                continue;
            }
            let (area, len) = curve_numeric(c, 1.0, 1.0).unwrap();
            if !c.area.is_empty() {
                let exact = ex(c.area).subs(&sym("a"), &one()).to_f64_const().unwrap();
                assert!((exact - area).abs() < 1e-6 * exact, "{} area {exact} vs {area}", c.title);
            }
            if !c.length.is_empty() {
                let exact = ex(c.length).subs(&sym("a"), &one()).to_f64_const().unwrap();
                assert!((exact - len).abs() < 1e-6 * exact, "{} length {exact} vs {len}", c.title);
            }
        }
    }

    #[test]
    fn polyhedron_measures_are_consistent() {
        for p in POLYHEDRA {
            assert_eq!(p.v as i64 - p.e as i64 + p.f as i64, 2, "{}", p.names[0]);
            // inradius × area / 3 = volume for the Platonic solids
            if !p.inradius.is_empty() {
                let (v, a, r) = (ex(p.vol).to_f64_const().unwrap(), ex(p.area).to_f64_const().unwrap(), ex(p.inradius).to_f64_const().unwrap());
                assert!((r * a / 3.0 - v).abs() < 1e-9, "{}", p.names[0]);
            }
        }
    }

    #[test]
    fn shape_lookups() {
        assert!(ask("truncated icosahedron").contains("55.2877"));
        assert!(ask("tesseract").contains("16"));
        assert!(ask("glome of radius 2").contains("8 π^2"));
        assert!(ask("5, 12, 13 triangle").contains("right scalene"));
        assert!(ask("arc length of an astroid").contains("6 a"));
        assert!(ask("pi_5(S^2)").contains("ℤ₂"));
        assert!(ask("pi_719(S^700)").contains("ℤ₂₆₄ ⊕ ℤ₂"));
        assert!(ask("hexiamonds").contains("12"));
        assert!(ask("[3,8)").contains("3 ≤ x < 8"));
        assert!(ask("8_1 knot").contains("13"));
    }
}
