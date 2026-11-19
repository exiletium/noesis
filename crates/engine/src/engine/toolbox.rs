//! Algebra and discrete-mathematics tools asked for in words: quaternions
//! ("quaternion 1+2i-j-3k", "norm of quaternion …"), permutations in cycle
//! notation ("(1 2 3 4)^3 (1 2 3)^-1"), named groups ("alternating group
//! A_5", "number of groups of order 120"), minors and special matrices,
//! polynomials from their zeros, long division, interpolation and Vieta's
//! formulas, logarithmic and exponential forms, all n-th roots, counting
//! problems (partitions into parts, compositions, necklaces, Latin squares,
//! Frobenius numbers, the Ackermann function) and rounding rules.

use super::*;
use crate::expr::Expr::*;
use crate::expr::*;
use crate::poly::QPoly;
use crate::print::text as ptext;
use crate::simplify as s;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn normalise(s: &str) -> String {
    let mut t = s.trim().to_string();
    for _ in 0..4 {
        let n = re!(r"\b([A-Za-z]+)\[([^\[\]]*)\]").replace_all(&t, "$1($2)").to_string();
        if n == t {
            break;
        }
        t = n;
    }
    t
}

fn ex(s: &str) -> Option<Expr> {
    crate::parse::parse(normalise(s).trim()).ok().map(|p| p.expr)
}

fn show(e: &Expr) -> String {
    ptext(e)
}

fn ans(title: &str, input: String, result_title: &str, result: Content, extra: Vec<Pod>) -> Answer {
    let mut pods = vec![pod("Input", "Input interpretation", text(input)), primary(pod("Result", result_title, result))];
    pods.extend(extra);
    Answer::new(title, pods)
}

fn rows(v: Vec<(&str, String)>) -> Content {
    table(None, v.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())
}

fn big(n: u64) -> BigInt {
    BigInt::from(n)
}

fn fact(n: u64) -> BigInt {
    crate::numtheory::factorial(n)
}

fn clean(q: &Query) -> (String, String) {
    let c = q.clean.trim().trim_end_matches(['?', '.']).trim().to_string();
    let l = c.to_lowercase();
    (c, l)
}

// ---------------------------------------------------------------------------
// quaternions
// ---------------------------------------------------------------------------

type Quat = [Expr; 4];

fn parse_quat(t: &str) -> Option<Quat> {
    let t = normalise(t);
    let t = t.trim().trim_start_matches(':').trim();
    if let Some(m) = re!(r"(?i)^quaternion\s*\((.+)\)$").captures(t) {
        let parts = super::nl::split_top(&m[1]);
        if parts.len() != 4 {
            return None;
        }
        let v: Vec<Expr> = parts.iter().map(|p| ex(p).map(|e| s::simplify(&e))).collect::<Option<_>>()?;
        return Some([v[0].clone(), v[1].clone(), v[2].clone(), v[3].clone()]);
    }
    let t = t.trim_matches(|c| c == '(' || c == ')');
    if !re!(r"[ijk]").is_match(t) || re!(r"[a-hl-z]{2,}").is_match(&t.to_lowercase().replace("sin", "").replace("cos", "").replace("pi", "").replace("sqrt", "")) {
        return None;
    }
    // split at top-level + and − signs
    let mut terms: Vec<String> = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    let chars: Vec<char> = t.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        let prev = cur.trim_end().chars().last();
        if depth == 0 && (ch == '+' || ch == '-') && i > 0 && !matches!(prev, Some('^' | '*' | '/' | 'e' | 'E')) && !cur.trim().is_empty() {
            terms.push(cur.clone());
            cur.clear();
        }
        cur.push(ch);
    }
    terms.push(cur);
    let mut q: Quat = [zero(), zero(), zero(), zero()];
    for term in terms {
        let term = term.replace(' ', "");
        if term.is_empty() {
            continue;
        }
        let (coef, slot) = match term.chars().last()? {
            'i' if !term.ends_with("pi") => (&term[..term.len() - 1], 1),
            'j' => (&term[..term.len() - 1], 2),
            'k' => (&term[..term.len() - 1], 3),
            _ => (&term[..], 0),
        };
        let coef = coef.trim_end_matches('*');
        let c = match coef {
            "" | "+" => one(),
            "-" => int(-1),
            c => ex(c)?,
        };
        q[slot] = s::simplify(&add(vec![q[slot].clone(), c]));
    }
    Some(q)
}

fn qmul(a: &Quat, b: &Quat) -> Quat {
    let m = |x: &Expr, y: &Expr| mul(vec![x.clone(), y.clone()]);
    let r = [
        add(vec![m(&a[0], &b[0]), neg(m(&a[1], &b[1])), neg(m(&a[2], &b[2])), neg(m(&a[3], &b[3]))]),
        add(vec![m(&a[0], &b[1]), m(&a[1], &b[0]), m(&a[2], &b[3]), neg(m(&a[3], &b[2]))]),
        add(vec![m(&a[0], &b[2]), neg(m(&a[1], &b[3])), m(&a[2], &b[0]), m(&a[3], &b[1])]),
        add(vec![m(&a[0], &b[3]), m(&a[1], &b[2]), neg(m(&a[2], &b[1])), m(&a[3], &b[0])]),
    ];
    r.map(|e| s::simplify(&e))
}

fn qconj(a: &Quat) -> Quat {
    [a[0].clone(), s::simplify(&neg(a[1].clone())), s::simplify(&neg(a[2].clone())), s::simplify(&neg(a[3].clone()))]
}

fn qnorm2(a: &Quat) -> Expr {
    s::simplify(&add(a.iter().map(|x| square(x.clone())).collect()))
}

fn qinv(a: &Quat) -> Option<Quat> {
    let n2 = qnorm2(a);
    if n2.is_zero() {
        return None;
    }
    Some(qconj(a).map(|x| s::simplify(&div(x, n2.clone()))))
}

fn qshow(a: &Quat) -> String {
    let mut out = String::new();
    for (k, unit) in ["", "i", "j", "k"].iter().enumerate() {
        let mut c = a[k].clone();
        if c.is_zero() {
            continue;
        }
        let negative = c.to_f64_const().is_some_and(|v| v < 0.0) || show(&c).starts_with('-');
        if negative {
            c = s::simplify(&neg(c));
        }
        let c = &c;
        let mut t = show(c);
        let compound = matches!(c, Add(_)) || t.contains(' ') && !t.starts_with('-');
        if !unit.is_empty() {
            if c.is_one() {
                t = String::new();
            } else if compound || t.contains('/') {
                t = format!("({t})");
            }
        }
        let piece = format!("{t}{unit}");
        out = match (out.is_empty(), negative) {
            (true, false) => piece,
            (true, true) => format!("-{piece}"),
            (false, true) => format!("{out} - {piece}"),
            (false, false) => format!("{out} + {piece}"),
        };
    }
    if out.is_empty() {
        "0".into()
    } else {
        out
    }
}

fn quat_props(a: &Quat) -> Vec<(&'static str, String)> {
    let n2 = qnorm2(a);
    let norm = s::simplify(&func("sqrt", vec![n2.clone()]));
    let mut v = vec![("quaternion", qshow(a)), ("norm", show(&norm)), ("conjugate", qshow(&qconj(a)))];
    if let Some(i) = qinv(a) {
        v.push(("inverse", qshow(&i)));
        v.push(("unit quaternion", qshow(&a.clone().map(|x| s::simplify(&div(x, norm.clone()))))));
    }
    v.push(("scalar part", show(&a[0])));
    v.push(("vector part", format!("({}, {}, {})", show(&a[1]), show(&a[2]), show(&a[3]))));
    // rotation: angle 2 acos(a0/|q|) about the vector part
    if let (Some(a0), Some(n)) = (a[0].to_f64_const(), norm.to_f64_const()) {
        let vn = (n * n - a0 * a0).max(0.0).sqrt();
        if n > 0.0 && vn > 1e-12 {
            let ang = 2.0 * (a0 / n).clamp(-1.0, 1.0).acos();
            let ax: Vec<String> = a[1..].iter().map(|c| c.to_f64_const().map(|x| crate::stats::fmt(x / vn)).unwrap_or_default()).collect();
            v.push(("as a rotation", format!("{}° about the axis ({})", crate::stats::fmt(ang.to_degrees()), ax.join(", "))));
        }
    }
    v
}

/// "draw 1+0i+5j+2k as a rotation operator", "rotation matrix of quaternion 1+i+j+k":
/// the rotation v ↦ q v q⁻¹ as a matrix, an axis and an angle, with a picture.
fn quat_rotation(q: &Query) -> Option<Answer> {
    let (c, _) = clean(q);
    let body = if let Some(m) = re!(r"(?i)^(?:(?:draw|show|plot|visuali[sz]e|display|interpret)\s+)?(?:the\s+)?(?:quaternion\s+)?(.+?)\s+as\s+(?:an?\s+)?(?:3-?d\s+)?rotation(?:\s+(?:operator|matrix))?$").captures(&c) {
        m[1].to_string()
    } else if let Some(m) = re!(r"(?i)^(?:the\s+)?(?:3-?d\s+)?rotation\s+(?:matrix|operator|axis and angle)\s+(?:of|for|from)\s+(?:the\s+)?(?:quaternion\s+)?(.+)$").captures(&c) {
        m[1].to_string()
    } else {
        return None;
    };
    let a = parse_quat(body.trim())?;
    let n2 = s::simplify(&qnorm2(&a));
    let n2f = n2.to_f64_const()?;
    if !(n2f > 0.0) {
        return None;
    }
    let [w, x, y, z] = a.clone();
    let sq = |e: &Expr| pow(e.clone(), int(2));
    let neg = |e: Expr| mul(vec![int(-1), e]);
    let two = |p: &Expr, r: &Expr| mul(vec![int(2), p.clone(), r.clone()]);
    let e = |v: Expr| s::simplify(&div(v, n2.clone()));
    let m: [[Expr; 3]; 3] = [
        [e(add(vec![sq(&w), sq(&x), neg(sq(&y)), neg(sq(&z))])), e(sub(two(&x, &y), two(&w, &z))), e(add(vec![two(&x, &z), two(&w, &y)]))],
        [e(add(vec![two(&x, &y), two(&w, &z)])), e(add(vec![sq(&w), neg(sq(&x)), sq(&y), neg(sq(&z))])), e(sub(two(&y, &z), two(&w, &x)))],
        [e(sub(two(&x, &z), two(&w, &y))), e(add(vec![two(&y, &z), two(&w, &x)])), e(add(vec![sq(&w), neg(sq(&x)), neg(sq(&y)), sq(&z)]))],
    ];
    let mf: Vec<Vec<f64>> = m.iter().map(|r| r.iter().map(|v| v.to_f64_const()).collect::<Option<Vec<f64>>>()).collect::<Option<_>>()?;
    let (wf, n) = (w.to_f64_const()?, n2f.sqrt());
    let v: Vec<f64> = [&x, &y, &z].iter().map(|c| c.to_f64_const()).collect::<Option<_>>()?;
    let vn = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let ang = 2.0 * (wf / n).clamp(-1.0, 1.0).acos();
    let fmt = |t: f64| crate::stats::fmt(if t.abs() < 1e-12 { 0.0 } else { t });
    let mut props = vec![("quaternion", qshow(&a)), ("norm", show(&s::simplify(&func("sqrt", vec![n2.clone()]))))];
    let axis: [f64; 3] = if vn > 1e-12 { [v[0] / vn, v[1] / vn, v[2] / vn] } else { [0.0, 0.0, 1.0] };
    if vn > 1e-12 {
        props.push(("rotation angle", format!("{}° = {} rad", fmt(ang.to_degrees()), fmt(ang))));
        props.push(("rotation axis (unit vector)", format!("({})", axis.iter().map(|t| fmt(*t)).collect::<Vec<_>>().join(", "))));
    } else {
        props.push(("rotation", "the identity (no vector part)".into()));
    }
    // ZYX (yaw, pitch, roll) angles of the matrix
    let pitch = (-mf[2][0]).clamp(-1.0, 1.0).asin();
    let (yaw, roll) = (mf[1][0].atan2(mf[0][0]), mf[2][1].atan2(mf[2][2]));
    props.push(("Euler angles (z-y-x: yaw, pitch, roll)", format!("{}°, {}°, {}°", fmt(yaw.to_degrees()), fmt(pitch.to_degrees()), fmt(roll.to_degrees()))));
    let rot = |p: [f64; 3], t: f64| -> [f64; 3] {
        // Rodrigues: p cos t + (k × p) sin t + k (k·p)(1 − cos t)
        let k = axis;
        let cr = [k[1] * p[2] - k[2] * p[1], k[2] * p[0] - k[0] * p[2], k[0] * p[1] - k[1] * p[0]];
        let d = k[0] * p[0] + k[1] * p[1] + k[2] * p[2];
        [0, 1, 2].map(|i| p[i] * t.cos() + cr[i] * t.sin() + k[i] * d * (1.0 - t.cos()))
    };
    let mut segs: Vec<Vec<[f64; 3]>> = Vec::new();
    segs.push(vec![axis.map(|t| -1.4 * t), axis.map(|t| 1.4 * t)]);
    let basis = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for b in basis {
        segs.push(vec![[0.0; 3], b]);
        let img = [mf[0][0] * b[0] + mf[0][1] * b[1] + mf[0][2] * b[2], mf[1][0] * b[0] + mf[1][1] * b[1] + mf[1][2] * b[2], mf[2][0] * b[0] + mf[2][1] * b[1] + mf[2][2] * b[2]];
        segs.push(vec![[0.0; 3], img]);
        if vn > 1e-12 {
            segs.push((0..=48).map(|k| rot(b, ang * k as f64 / 48.0)).collect());
        }
    }
    let mut extra = vec![
        pod("Matrix", "Rotation matrix (v ↦ q v q⁻¹)", table(None, m.iter().map(|r| r.iter().map(|v| text(show(v))).collect()).collect())),
        pod("Properties", "Rotation", rows(props)),
    ];
    if let Some(pl) = crate::plot::segments3d(segs) {
        extra.push(pod("Plot", "Rotation of the coordinate axes (axis, original and rotated unit vectors, and their paths)", Content::Plot { plot: Box::new(pl) }));
    }
    let res = text(if vn > 1e-12 { format!("rotation by {}° about ({})", fmt(ang.to_degrees()), axis.iter().map(|t| fmt(*t)).collect::<Vec<_>>().join(", ")) } else { "identity rotation".into() });
    Some(ans("quaternion rotation", format!("{} as a rotation operator", qshow(&a)), "Rotation", res, extra))
}

fn quaternions(q: &Query) -> Option<Answer> {
    let (c, l) = clean(q);
    let body = re!(r"(?i)^(?:the\s+)?(?:(norm|conjugate|inverse|magnitude|modulus)\s*(?:of)?\s*:?\s*)?").replace(&c, "").to_string();
    let body = if re!(r"(?i)^quaternion\s*\(").is_match(&body) { body } else { re!(r"(?i)^(?:the\s+)?quaternions?\s*:?\s*").replace(&body, "").to_string() };
    if !l.contains("quaternion") && !re!(r"[\d)+-]\s*\*?\s*[jk]\b").is_match(&body) {
        return None;
    }
    let op = re!(r"(?i)^(?:the\s+)?(norm|conjugate|inverse|magnitude|modulus)").captures(&c).map(|m| m[1].to_lowercase());
    // two quaternions joined by a product
    let prod = re!(r"(?i)\s*(?:\*\*|\bmultiplied by\b|\btimes\b|\*|×)\s*").split(&body).collect::<Vec<_>>();
    let list: Vec<String> = if prod.len() == 2 { prod.iter().map(|s| s.to_string()).collect() } else { super::nl::split_top(&body).into_iter().map(|s| s.trim().to_string()).collect() };
    let qs: Vec<Quat> = list.iter().map(|t| parse_quat(t.trim().trim_start_matches(|c: char| c.to_ascii_lowercase() == 'q').trim_start_matches(|c| c == ':').trim()).or_else(|| parse_quat(t))).collect::<Option<_>>()?;
    if qs.is_empty() || (!l.contains("quaternion") && (qs.iter().all(|q| q[0].is_zero()) || qs.len() == 1 && qs[0][2].is_zero() && qs[0][3].is_zero())) {
        return None;
    }
    if prod.len() == 2 && qs.len() == 2 {
        let p = qmul(&qs[0], &qs[1]);
        let mut extra = vec![pod("Reverse", "Reverse order product", text(qshow(&qmul(&qs[1], &qs[0]))))];
        let (title, val) = match op.as_deref() {
            Some("inverse") => ("Inverse of the product", qshow(&qinv(&p)?)),
            Some("conjugate") => ("Conjugate of the product", qshow(&qconj(&p))),
            Some(_) => ("Norm of the product", show(&s::simplify(&func("sqrt", vec![qnorm2(&p)])))),
            None => ("Product", qshow(&p)),
        };
        if op.is_some() {
            extra.insert(0, pod("Product", "Product", text(qshow(&p))));
        }
        return Some(ans("quaternion product", format!("({}) × ({})", qshow(&qs[0]), qshow(&qs[1])), title, text(val), extra));
    }
    if qs.len() >= 2 && op.is_some() {
        let key = match op.as_deref() {
            Some("conjugate") => "conjugate",
            Some("inverse") => "inverse",
            _ => "norm",
        };
        let rowsv: Vec<Vec<Content>> = qs.iter().map(|a| {
            let p = quat_props(a);
            vec![text(qshow(a)), text(p.iter().find(|(n, _)| *n == key).map(|x| x.1.clone()).unwrap_or_default())]
        }).collect();
        return Some(ans("quaternions", format!("{key} of each quaternion"), &format!("{}{}", key[..1].to_uppercase(), &key[1..]), table(Some(vec!["quaternion", key]), rowsv), vec![]));
    }
    if qs.len() >= 2 {
        let rowsv: Vec<Vec<Content>> = qs.iter().map(|a| {
            let p = quat_props(a);
            let pick = |k: &str| p.iter().find(|(n, _)| *n == k).map(|x| x.1.clone()).unwrap_or_default();
            vec![text(pick("quaternion")), text(pick("norm")), text(pick("conjugate")), text(pick("inverse"))]
        }).collect();
        let mut extra = Vec::new();
        if qs.len() == 2 {
            extra.push(pod("Product", "Product q₁ q₂", text(qshow(&qmul(&qs[0], &qs[1])))));
            extra.push(pod("Sum", "Sum", text(qshow(&[0, 1, 2, 3].map(|i| s::simplify(&add(vec![qs[0][i].clone(), qs[1][i].clone()])))))));
        }
        return Some(ans("quaternions", "quaternions".into(), "Properties", table(Some(vec!["quaternion", "norm", "conjugate", "inverse"]), rowsv), extra));
    }
    let a = &qs[0];
    let props = quat_props(a);
    if let Some(o) = op {
        let key = match o.as_str() {
            "conjugate" => "conjugate",
            "inverse" => "inverse",
            _ => "norm",
        };
        let v = props.iter().find(|(k, _)| *k == key)?.1.clone();
        return Some(ans("quaternion", format!("{key} of {}", qshow(a)), &format!("{}{}", key[..1].to_uppercase(), &key[1..]), text(v), vec![pod("Properties", "Properties", rows(props))]));
    }
    Some(ans("quaternion", qshow(a), "Properties", rows(props), vec![]))
}

// ---------------------------------------------------------------------------
// permutations
// ---------------------------------------------------------------------------

fn perm_compose(p: &[usize], q: &[usize]) -> Vec<usize> {
    // apply p first, then q
    let n = p.len().max(q.len());
    (0..n).map(|i| {
        let a = p.get(i).copied().unwrap_or(i);
        q.get(a).copied().unwrap_or(a)
    }).collect()
}

fn perm_pow(p: &[usize], k: i64) -> Vec<usize> {
    let n = p.len();
    let mut base: Vec<usize> = if k < 0 {
        let mut inv = vec![0; n];
        for (i, &v) in p.iter().enumerate() {
            inv[v] = i;
        }
        inv
    } else {
        p.to_vec()
    };
    let mut e = k.unsigned_abs();
    let mut r: Vec<usize> = (0..n).collect();
    while e > 0 {
        if e & 1 == 1 {
            r = perm_compose(&r, &base);
        }
        base = perm_compose(&base, &base);
        e >>= 1;
    }
    r
}

fn cycles(p: &[usize]) -> Vec<Vec<usize>> {
    let mut seen = vec![false; p.len()];
    let mut out = Vec::new();
    for i in 0..p.len() {
        if seen[i] {
            continue;
        }
        let mut c = vec![i];
        seen[i] = true;
        let mut j = p[i];
        while j != i {
            seen[j] = true;
            c.push(j);
            j = p[j];
        }
        out.push(c);
    }
    out
}

fn cycle_text(p: &[usize]) -> String {
    let cs: Vec<String> = cycles(p).into_iter().filter(|c| c.len() > 1).map(|c| format!("({})", c.iter().map(|x| (x + 1).to_string()).collect::<Vec<_>>().join(" "))).collect();
    if cs.is_empty() {
        "identity ()".into()
    } else {
        cs.join("")
    }
}

fn permutations(q: &Query) -> Option<Answer> {
    let (_, l) = clean(q);
    let body = re!(r"^(?:the\s+)?(?:perm(?:utation)?s?\s*:?\s*)?(?:product\s+)?").replace(&l, "").to_string();
    if !re!(r"^(?:\s*\(\s*\d+(?:[\s,]+\d+)*\s*\)\s*(?:\^\s*\(?-?\d+\)?)?\s*(?:\*|∘|·)?\s*)+$").is_match(&body) {
        return None;
    }
    // a single parenthesised list of 2-3 numbers is a point, not a cycle
    let n_cycles = re!(r"\(").find_iter(&body).count();
    if n_cycles == 1 && !l.starts_with("perm") && !body.contains('^') {
        return None;
    }
    let mut total: Vec<usize> = Vec::new();
    let mut maxn = 0usize;
    let parts: Vec<(Vec<usize>, i64)> = re!(r"\(\s*([\d\s,]+?)\s*\)\s*(?:\^\s*\(?(-?\d+)\)?)?").captures_iter(&body).map(|m| {
        let c: Vec<usize> = m[1].split([' ', ',']).filter(|s| !s.is_empty()).filter_map(|s| s.parse::<usize>().ok()).collect();
        let k = m.get(2).and_then(|x| x.as_str().parse().ok()).unwrap_or(1);
        (c, k)
    }).collect();
    if parts.iter().all(|(c, _)| c.len() < 2) {
        return None;
    }
    for (c, _) in &parts {
        if c.iter().any(|&x| x == 0) || c.iter().collect::<std::collections::BTreeSet<_>>().len() != c.len() {
            return None;
        }
        maxn = maxn.max(*c.iter().max()?);
    }
    if maxn > 1000 {
        return None;
    }
    for (c, k) in &parts {
        let mut p: Vec<usize> = (0..maxn).collect();
        for w in 0..c.len() {
            p[c[w] - 1] = c[(w + 1) % c.len()] - 1;
        }
        let p = perm_pow(&p, *k);
        total = if total.is_empty() { p } else { perm_compose(&total, &p) };
    }
    let cs = cycles(&total);
    let order = cs.iter().map(|c| big(c.len() as u64)).fold(BigInt::one(), |a, b| a.lcm(&b));
    let parity = cs.iter().map(|c| c.len() - 1).sum::<usize>() % 2;
    let mut ctype: Vec<usize> = cs.iter().map(|c| c.len()).collect();
    ctype.sort_unstable_by(|a, b| b.cmp(a));
    let inv = perm_pow(&total, -1);
    let props = vec![
        ("cycle notation", cycle_text(&total)),
        ("one-line notation", format!("({})", total.iter().map(|x| (x + 1).to_string()).collect::<Vec<_>>().join(" "))),
        ("order", order.to_string()),
        ("sign", if parity == 0 { "+1 (even)".into() } else { "−1 (odd)".to_string() }),
        ("cycle type", format!("[{}]", ctype.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))),
        ("fixed points", cs.iter().filter(|c| c.len() == 1).count().to_string()),
        ("inverse", cycle_text(&inv)),
    ];
    let input = if parts.len() > 1 || parts[0].1 != 1 { format!("{} (products applied left to right)", body.trim()) } else { body.trim().to_string() };
    Some(ans("permutation", input, "Permutation", rows(props), vec![]))
}

// ---------------------------------------------------------------------------
// groups
// ---------------------------------------------------------------------------

/// Number of groups of order n (OEIS A000001), for the orders known here.
fn groups_of_order(n: u64) -> Option<u64> {
    const T: [u64; 65] = [0, 1, 1, 1, 2, 1, 2, 1, 5, 2, 2, 1, 5, 1, 2, 1, 14, 1, 5, 1, 5, 2, 2, 1, 15, 2, 2, 5, 4, 1, 4, 1, 51, 1, 2, 1, 14, 1, 2, 2, 14, 1, 6, 1, 4, 2, 2, 1, 52, 2, 5, 1, 5, 1, 15, 2, 13, 2, 2, 1, 13, 1, 2, 4, 267];
    if (n as usize) < T.len() {
        return Some(T[n as usize]);
    }
    Some(match n {
        72 => 50,
        80 => 52,
        81 => 15,
        96 => 231,
        100 => 16,
        120 => 47,
        128 => 2328,
        144 => 197,
        160 => 238,
        192 => 1543,
        256 => 56092,
        512 => 10494213,
        1024 => 49487367289,
        _ => {
            let f = crate::numtheory::factor(&big(n));
            if f.len() == 1 && f[0].1 == 1 {
                1
            } else if f.len() == 2 && f.iter().all(|x| x.1 == 1) {
                // n = p q with p < q: 2 groups when p | q − 1, else 1
                let (p, q) = (f[0].0.clone(), f[1].0.clone());
                if ((&q - 1u32) % &p).is_zero() { 2 } else { 1 }
            } else {
                return None;
            }
        }
    })
}

fn partitions_list(n: usize, max: usize) -> Vec<Vec<usize>> {
    if n == 0 {
        return vec![vec![]];
    }
    let mut out = Vec::new();
    for first in (1..=n.min(max)).rev() {
        for mut rest in partitions_list(n - first, first) {
            let mut v = vec![first];
            v.append(&mut rest);
            out.push(v);
        }
    }
    out
}

fn alternating_classes(n: usize) -> usize {
    partitions_list(n, n).iter().filter(|p| (n - p.len()) % 2 == 0).map(|p| {
        let distinct_odd = p.iter().all(|x| x % 2 == 1) && p.windows(2).all(|w| w[0] != w[1]);
        if distinct_odd && n > 1 { 2 } else { 1 }
    }).sum()
}

fn groups(q: &Query) -> Option<Answer> {
    let (_, l) = clean(q);
    if let Some(m) = re!(r"^(?:the\s+)?(?:number of\s+(?:non-?isomorphic\s+)?groups|groups)\s+of\s+order\s+(\d+)$").captures(&l) {
        let n: u64 = m[1].parse().ok()?;
        let g = groups_of_order(n)?;
        let mut extra = Vec::new();
        let ab: u64 = crate::numtheory::factor(&big(n)).iter().map(|(_, e)| crate::numtheory::partitions(*e as usize).to_u64().unwrap_or(0)).product();
        extra.push(pod("Abelian", "Abelian groups of this order", text(ab.to_string())));
        return Some(ans("groups", format!("number of groups of order {n}"), "Result", text(g.to_string()), extra));
    }
    let named: &[(&str, &str, &[(&str, &str)])] = &[
        ("monster group", "the Monster group M", &[("order", "808017424794512875886459904961710757005754368000000000 ≈ 8.08 × 10⁵³"), ("type", "largest sporadic simple group"), ("conjugacy classes", "194"), ("smallest faithful complex representation", "degree 196883")]),
        ("baby monster group", "the Baby Monster group B", &[("order", "4154781481226426191177580544000000 ≈ 4.15 × 10³³"), ("type", "sporadic simple group"), ("conjugacy classes", "184")]),
        ("quaternion group", "quaternion group Q₈", &[("order", "8"), ("elements", "±1, ±i, ±j, ±k"), ("abelian", "no (every subgroup is normal: Hamiltonian)"), ("center", "{1, −1}"), ("conjugacy classes", "5: {1}, {−1}, {±i}, {±j}, {±k}"), ("presentation", "⟨i, j | i⁴ = 1, i² = j², j i j⁻¹ = i⁻¹⟩")]),
        ("klein four-group", "Klein four-group V₄ ≅ ℤ₂ × ℤ₂", &[("order", "4"), ("abelian", "yes"), ("conjugacy classes", "4"), ("exponent", "2")]),
        ("mathieu group m11", "Mathieu group M₁₁", &[("order", "7920"), ("type", "sporadic simple group"), ("conjugacy classes", "10")]),
        ("mathieu group m12", "Mathieu group M₁₂", &[("order", "95040"), ("type", "sporadic simple group"), ("conjugacy classes", "15")]),
        ("mathieu group m22", "Mathieu group M₂₂", &[("order", "443520"), ("type", "sporadic simple group"), ("conjugacy classes", "12")]),
        ("mathieu group m23", "Mathieu group M₂₃", &[("order", "10200960"), ("type", "sporadic simple group"), ("conjugacy classes", "17")]),
        ("mathieu group m24", "Mathieu group M₂₄", &[("order", "244823040"), ("type", "sporadic simple group"), ("conjugacy classes", "26")]),
    ];
    let lq = re!(r"^(?:the\s+)?(?:order of (?:the\s+)?)?").replace(&l, "").to_string();
    let lq = lq.replace("klein 4-group", "klein four-group").replace("klein four group", "klein four-group").replace("monster", "monster group").replace("group group", "group");
    if let Some((_, title, props)) = named.iter().find(|(n, _, _)| lq == *n || lq == n.trim_end_matches(" group")) {
        let want_order = l.contains("order of");
        let props: Vec<(&str, String)> = props.iter().map(|(a, b)| (*a, b.to_string())).collect();
        if want_order {
            let o = props[0].1.clone();
            return Some(ans("group", format!("order of {title}"), "Order", text(o), vec![pod("Properties", "Properties", rows(props))]));
        }
        return Some(ans("group", title.to_string(), "Properties", rows(props), vec![]));
    }
    let m = re!(r"^(?:the\s+)?(?:order of (?:the\s+)?)?(symmetric|alternating|cyclic|dihedral)\s+group\s+(?:of\s+(?:degree|order)\s+)?([sacdz])?_?\{?(\d+)\}?$").captures(&l)?;
    let n: u64 = m[3].parse().ok()?;
    if n == 0 || n > 1000 {
        return None;
    }
    let kind = &m[1];
    let (title, order, props): (String, BigInt, Vec<(&str, String)>) = match kind {
        "symmetric" => {
            let o = fact(n);
            (format!("symmetric group S_{n}"), o.clone(), vec![
                ("order", o.to_string()),
                ("abelian", if n <= 2 { "yes" } else { "no" }.into()),
                ("conjugacy classes", crate::numtheory::partitions(n as usize).to_string()),
                ("center", if n <= 2 { "the whole group" } else { "trivial" }.into()),
                ("simple", if n == 2 { "yes" } else { "no" }.into()),
                ("solvable", if n <= 4 { "yes" } else { "no" }.into()),
                ("generators", if n >= 2 { format!("(1 2) and (1 2 … {n})") } else { "—".into() }),
            ])
        }
        "alternating" => {
            let o = if n >= 2 { fact(n) / 2 } else { BigInt::one() };
            (format!("alternating group A_{n}"), o.clone(), vec![
                ("order", o.to_string()),
                ("abelian", if n <= 3 { "yes" } else { "no" }.into()),
                ("conjugacy classes", if n <= 1 { "1".into() } else { alternating_classes(n as usize).to_string() }),
                ("simple", if n >= 5 || n == 3 { "yes" } else { "no" }.into()),
                ("solvable", if n <= 4 { "yes" } else { "no" }.into()),
                ("index in S_n", "2".into()),
            ])
        }
        "cyclic" => (format!("cyclic group C_{n} ≅ ℤ/{n}ℤ"), big(n), vec![
            ("order", n.to_string()),
            ("abelian", "yes".into()),
            ("generators", crate::numtheory::totient(&big(n)).to_string()),
            ("subgroups", crate::numtheory::divisor_count(&big(n)).to_string()),
            ("simple", if crate::numtheory::is_prime_u64(n) { "yes" } else { "no" }.into()),
        ]),
        _ => {
            // D_n: symmetries of a regular n-gon, order 2n
            let o = big(2 * n);
            let classes = if n % 2 == 1 { (n + 3) / 2 } else { n / 2 + 3 };
            (format!("dihedral group D_{n} (symmetries of a regular {n}-gon)"), o.clone(), vec![
                ("order", o.to_string()),
                ("abelian", if n <= 2 { "yes" } else { "no" }.into()),
                ("conjugacy classes", classes.to_string()),
                ("center", if n % 2 == 0 && n > 2 { "{e, r^(n/2)}".replace("n/2", &(n / 2).to_string()) } else if n <= 2 { "the whole group".into() } else { "trivial".into() }),
                ("presentation", format!("⟨r, s | r^{n} = s² = 1, s r s = r⁻¹⟩")),
            ])
        }
    };
    if l.contains("order of") {
        return Some(ans("group", format!("order of {title}"), "Order", text(order.to_string()), vec![pod("Properties", "Properties", rows(props))]));
    }
    Some(ans("group", title, "Properties", rows(props), vec![]))
}

// ---------------------------------------------------------------------------
// minors and special matrices
// ---------------------------------------------------------------------------

fn submatrix(m: &crate::linalg::Mat, r: usize, c: usize) -> crate::linalg::Mat {
    m.iter().enumerate().filter(|(i, _)| *i != r).map(|(_, row)| row.iter().enumerate().filter(|(j, _)| *j != c).map(|(_, x)| x.clone()).collect()).collect()
}

fn minor(m: &crate::linalg::Mat, r: usize, c: usize) -> Expr {
    let sm = submatrix(m, r, c);
    if sm.is_empty() {
        return one();
    }
    s::simplify(&crate::linalg::det(&sm))
}

fn matrices(q: &Query) -> Option<Answer> {
    let (c, l) = clean(q);
    // "(1,2)-minor of M", "M-23 of M", "M_23 of M", "minor 1,2 of M"
    if let Some(m) = re!(r"(?i)^(?:the\s+)?(?:\((\d+)\s*,\s*(\d+)\)[- ]?(minor|cofactor)|(m|c)[-_]?(\d)(\d)|(minor|cofactor)\s+(\d+)\s*,\s*(\d+))\s+of\s+(\{.+\})$").captures(&c) {
        let (i, j, kind): (usize, usize, &str) = if let Some(a) = m.get(1) {
            (a.as_str().parse().ok()?, m[2].parse().ok()?, if m[3].eq_ignore_ascii_case("cofactor") { "cofactor" } else { "minor" })
        } else if let Some(k) = m.get(4) {
            (m[5].parse().ok()?, m[6].parse().ok()?, if k.as_str().eq_ignore_ascii_case("c") { "cofactor" } else { "minor" })
        } else {
            (m[8].parse().ok()?, m[9].parse().ok()?, if m[7].eq_ignore_ascii_case("cofactor") { "cofactor" } else { "minor" })
        };
        let mat = crate::linalg::from_expr(&ex(&m[10])?)?;
        if i == 0 || j == 0 || i > mat.len() || j > mat[0].len() || mat.len() != mat[0].len() {
            return None;
        }
        let mut v = minor(&mat, i - 1, j - 1);
        if kind == "cofactor" && (i + j) % 2 == 1 {
            v = s::simplify(&neg(v));
        }
        let sm = submatrix(&mat, i - 1, j - 1);
        return Some(ans("minor", format!("({i}, {j}) {kind} of {}", show(&crate::linalg::to_expr(&mat))), &format!("({i}, {j}) {kind}"), math(&v), vec![pod("Submatrix", &format!("Matrix with row {i} and column {j} deleted"), math(&crate::linalg::to_expr(&sm)))]));
    }
    // matrix of minors / cofactors
    if let Some(m) = re!(r"(?i)^(?:the\s+)?(?:(?:matrix of\s+)?(minors|cofactors)|(cofactor|minor) matrix)\s*(?:of\s+)?(\{.+\})$|^(\{.+\})\s+(minors|cofactors|cofactor matrix)$").captures(&c) {
        let (kind, body) = if let Some(k) = m.get(1).or(m.get(2)) { (k.as_str().to_lowercase(), m[3].to_string()) } else { (m[5].to_lowercase(), m[4].to_string()) };
        let mat = crate::linalg::from_expr(&ex(&body)?)?;
        let n = mat.len();
        if n < 2 || mat[0].len() != n {
            return None;
        }
        let cof = kind.starts_with("cofactor");
        let res: crate::linalg::Mat = (0..n).map(|i| (0..n).map(|j| {
            let v = minor(&mat, i, j);
            if cof && (i + j) % 2 == 1 { s::simplify(&neg(v)) } else { v }
        }).collect()).collect();
        let title = if cof { "Cofactor matrix" } else { "Matrix of minors" };
        return Some(ans("minors", format!("{} of {}", title.to_lowercase(), show(&crate::linalg::to_expr(&mat))), title, math(&crate::linalg::to_expr(&res)), vec![]));
    }
    // Hilbert and Hankel matrices
    if let Some(m) = re!(r"^(?:the\s+)?(?:(\d+)\s*(?:x|×|by)\s*(\d+)\s+)?(hilbert|hankel)\s+matri(?:x|ces)(?:\s+(?:of\s+)?(?:order|size)\s+(\d+))?$|^(hilbert|hankel)\s+matrix\s+(\d+)$").captures(&l) {
        let kind = m.get(3).or(m.get(5))?.as_str();
        let n: usize = m.get(1).or(m.get(4)).or(m.get(6)).and_then(|x| x.as_str().parse().ok()).unwrap_or(4);
        if n == 0 || n > 12 {
            return None;
        }
        let mat: crate::linalg::Mat = (0..n).map(|i| (0..n).map(|j| if kind == "hilbert" { Num(Rat::new(1.into(), BigInt::from(i + j + 1))) } else { sym(&format!("c{}", i + j + 1)) }).collect()).collect();
        let mut extra = Vec::new();
        if kind == "hilbert" {
            extra.push(pod("Det", "Determinant", math(&s::simplify(&crate::linalg::det(&mat)))));
            extra.push(pod("Formula", "Definition", text("H_ij = 1/(i + j − 1)")));
        } else {
            extra.push(pod("Formula", "Definition", text("constant along anti-diagonals: H_ij = c_(i+j−1)")));
        }
        return Some(ans(kind, format!("{n} × {n} {kind} matrix"), "Matrix", math(&crate::linalg::to_expr(&mat)), extra));
    }
    None
}

// ---------------------------------------------------------------------------
// polynomials
// ---------------------------------------------------------------------------

fn main_var(e: &Expr) -> Expr {
    let fs = e.free_symbols();
    for c in ["x", "y", "z", "t", "s", "n"] {
        if fs.iter().any(|n| &**n == c) {
            return sym(c);
        }
    }
    fs.iter().next().map(|n| sym(n)).unwrap_or_else(|| sym("x"))
}

fn polynomials(q: &Query) -> Option<Answer> {
    let (c, l) = clean(q);
    let x = sym("x");
    // from zeros
    if let Some(m) = re!(r"^(?:find|determine|what is)?\s*(?:the\s+|a\s+)?(?:monic\s+)?polynomial\s+(?:function\s+)?(?:with|having|whose)\s+(?:the\s+)?(?:zeros|roots|zeroes)\s+(?:at\s+|are\s+|of\s+)?(.+?)(?:\s+(?:with|and)\s+multiplicit(?:y|ies)\s+(.+))?$").captures(&l) {
        let roots: Vec<Expr> = re!(r",\s*(?:and\s+)?|\s+and\s+").split(m[1].trim()).map(|t| ex(t.trim())).collect::<Option<_>>()?;
        let mults: Vec<i64> = match m.get(2) {
            Some(t) => re!(r",\s*(?:and\s+)?|\s+and\s+").split(t.as_str().trim()).map(|t| t.trim().parse().ok()).collect::<Option<_>>()?,
            None => vec![1; roots.len()],
        };
        if mults.len() != roots.len() || roots.is_empty() || mults.iter().any(|&k| !(1..=50).contains(&k)) {
            return None;
        }
        let factored = mul(roots.iter().zip(&mults).map(|(r, k)| pow(sub(x.clone(), r.clone()), int(*k))).collect());
        let expanded = s::simplify(&s::expand(&factored));
        return Some(ans("polynomial", format!("polynomial with zeros {}", roots.iter().zip(&mults).map(|(r, k)| if *k == 1 { show(r) } else { format!("{} (multiplicity {k})", show(r)) }).collect::<Vec<_>>().join(", ")), "Polynomial", math(&expanded), vec![pod("Factored", "Factored form", math(&s::simplify(&factored)))]));
    }
    // rational roots
    if let Some(m) = re!(r"^(?:the\s+)?(?:rational|integer)\s+(?:solutions|roots|zeros)\s+(?:of|for|to)\s+(.+)$").captures(&c.to_lowercase()) {
        let e = implicit_expr(&m[1])?;
        let v = main_var(&e);
        let p = crate::poly::to_qpoly(&e, &v)?;
        let rr = crate::poly::rational_roots(&p);
        let int_only = l.contains("integer");
        let rr: Vec<_> = rr.into_iter().filter(|(r, _)| !int_only || r.is_integer()).collect();
        let res = if rr.is_empty() { "none".to_string() } else { rr.iter().map(|(r, k)| format!("{} = {}{}", show(&v), show(&Num(r.clone())), if *k > 1 { format!(" (multiplicity {k})") } else { String::new() })).collect::<Vec<_>>().join(", ") };
        let a0 = p.coeff(0);
        let an = p.lc();
        let note = format!("candidates ±p/q with p dividing {} and q dividing {} (rational root theorem)", a0.numer().abs(), an.numer().abs());
        return Some(ans("rational roots", format!("{} roots of {} = 0", if int_only { "integer" } else { "rational" }, show(&e)), "Result", text(res), vec![pod("Method", "Method", text(note))]));
    }
    // long division / quotient and remainder
    if let Some(m) = re!(r"^\(?(.+?)\)?\s*/\s*\(?(.+?)\)?\s+(?:polynomial\s+)?(?:long division|quotient and remainder|synthetic division)$|^(?:polynomial\s+)?(?:long|synthetic)\s+division\s+(?:of\s+)?\(?(.+?)\)?\s*(?:/|by|÷)\s*\(?(.+?)\)?$|^(?:the\s+)?quotient and remainder (?:of\s+)?\(?(.+?)\)?\s*(?:/|divided by|÷)\s*\(?(.+?)\)?$").captures(&l) {
        let (a, b) = if let Some(a) = m.get(1) { (a.as_str(), &m[2]) } else if let Some(a) = m.get(3) { (a.as_str(), &m[4]) } else { (&m[5], &m[6]) };
        let (pa, pb) = (ex(a)?, ex(b)?);
        let v = main_var(&add(vec![pa.clone(), pb.clone()]));
        if !pa.has_sym(&v) && !pb.has_sym(&v) {
            // integers: "912/7 quotient and remainder"
            let (na, nb) = (pa.as_int()?.clone(), pb.as_int()?.clone());
            if nb.is_zero() {
                return None;
            }
            let (qq, r) = na.div_mod_floor(&nb);
            return Some(ans("division", format!("{na} ÷ {nb}"), "Quotient and remainder", text(format!("{qq} remainder {r}")), vec![pod("Check", "Check", text(format!("{nb} × {qq} + {r} = {na}"))), pod("Mixed", "As a mixed number", text(if r.is_zero() { qq.to_string() } else { format!("{qq} {r}/{nb}") }))]));
        }
        let (qa, qb) = (crate::poly::to_qpoly(&pa, &v)?, crate::poly::to_qpoly(&pb, &v)?);
        if qb.is_zero() {
            return None;
        }
        let (qq, r) = qa.divrem(&qb);
        let (qe, re_) = (s::simplify(&qq.to_expr(&v)), s::simplify(&r.to_expr(&v)));
        let whole = if r.is_zero() { qe.clone() } else { s::simplify(&add(vec![qe.clone(), div(re_.clone(), pb.clone())])) };
        return Some(ans("polynomial division", format!("({}) ÷ ({})", show(&pa), show(&pb)), "Quotient", math(&qe), vec![pod("Remainder", "Remainder", math(&re_)), pod("Result", "Result", math(&whole))]));
    }
    // interpolating polynomial
    if let Some(m) = re!(r"^(?:the\s+)?(?:interpolating|interpolation|lagrange(?: interpolating)?)\s+polynomial\s+(?:for\s+|through\s+|of\s+)?(.+)$").captures(&l) {
        let body = m[1].trim();
        let pts: Vec<(Rat, Rat)> = re!(r"[{(]\s*([^,{}()]+?)\s*,\s*([^,{}()]+?)\s*[})]").captures_iter(body).map(|p| Some((ex(&p[1])?.as_rat()?.clone(), ex(&p[2])?.as_rat()?.clone()))).collect::<Option<_>>()?;
        if pts.len() < 2 || pts.len() > 30 {
            return None;
        }
        let mut poly = QPoly::zero();
        for (i, (xi, yi)) in pts.iter().enumerate() {
            let mut term = QPoly::constant(yi.clone());
            for (j, (xj, _)) in pts.iter().enumerate() {
                if i != j {
                    let d = xi - xj;
                    if d.is_zero() {
                        return None;
                    }
                    term = term.mul(&QPoly::new(vec![-xj.clone() / &d, Rat::one() / &d]));
                }
            }
            poly = poly.add(&term);
        }
        let e = s::simplify(&poly.to_expr(&x));
        return Some(ans("interpolation", format!("interpolating polynomial through {}", pts.iter().map(|(a, b)| format!("({a}, {b})")).collect::<Vec<_>>().join(", ")), "Polynomial", math(&e), vec![pod("Degree", "Degree", text(poly.degree().to_string()))]));
    }
    // vertex form of a quadratic
    if let Some(m) = re!(r"^(?:the\s+)?(?:vertex form|put .+ in vertex form)\s+(?:of\s+)?(.+)$|^(.+?)\s+(?:in|to)\s+vertex form$").captures(&l) {
        let t = m.get(1).or(m.get(2))?.as_str();
        let e = implicit_expr(t.trim_start_matches("y =").trim_start_matches("y=").trim())?;
        let v = main_var(&e);
        let cs = crate::poly::coeffs(&e, &v)?;
        if cs.len() != 3 {
            return None;
        }
        let (c0, b, a) = (cs[0].clone(), cs[1].clone(), cs[2].clone());
        let h = s::simplify(&div(neg(b.clone()), mul(vec![int(2), a.clone()])));
        let k = s::simplify(&sub(c0, div(square(b), mul(vec![int(4), a.clone()]))));
        let form = s::simplify(&add(vec![mul(vec![a, pow(sub(v.clone(), h.clone()), int(2))]), k.clone()]));
        return Some(ans("vertex form", format!("vertex form of {}", show(&e)), "Vertex form", math(&form), vec![pod("Vertex", "Vertex", math(&List(vec![h, k])))]));
    }
    // Vieta's formulas in full: every elementary symmetric function of the roots
    if let Some(m) = re!(r"^vieta'?s?\s+(?:formulas?|relations?)\s+(?:for|of)?\s*(.+)$").captures(&l) {
        let e = implicit_expr(&m[1])?;
        let v = main_var(&e);
        let cs: Vec<Expr> = crate::poly::coeffs(&e, &v)?.iter().map(s::simplify).collect();
        let n = cs.len().checked_sub(1)?;
        if n == 0 || n > 12 || cs[n].is_zero() {
            return None;
        }
        let rowsv: Vec<Vec<Content>> = (1..=n).map(|k| {
            let val = s::simplify(&div(mul(vec![int(if k % 2 == 0 { 1 } else { -1 }), cs[n - k].clone()]), cs[n].clone()));
            let name = match k {
                1 => "r₁ + r₂ + … (sum)".to_string(),
                _ if k == n => "r₁ r₂ ⋯ (product)".to_string(),
                _ => format!("e{k} (sum of products of {k} roots)"),
            };
            vec![text(name), math(&val)]
        }).collect();
        return Some(ans("Vieta", format!("Vieta's formulas for {} = 0", show(&s::simplify(&s::expand(&e)))), "Elementary symmetric functions of the roots", table(None, rowsv), vec![]));
    }
    // Vieta: sum / product of roots
    if let Some(m) = re!(r"^(?:find\s+)?(?:the\s+)?(sum and product|product and sum|sum|product)\s+of\s+(?:the\s+)?(?:roots|zeros|solutions)\s+(?:of|for|to)\s+(.+)$|^(.+?)\s+(sum and product|sum|product)\s+of\s+(?:the\s+)?(?:roots|zeros|solutions)$").captures(&l) {
        let (want, t) = if let Some(w) = m.get(1) { (w.as_str().to_string(), m[2].to_string()) } else { (m[4].to_string(), m[3].to_string()) };
        let e = implicit_expr(&t)?;
        let v = main_var(&e);
        let cs: Vec<Expr> = crate::poly::coeffs(&e, &v)?.iter().map(s::simplify).collect();
        let n = cs.len().checked_sub(1)?;
        if n == 0 || cs[n].is_zero() {
            return None;
        }
        let sum = s::simplify(&div(neg(cs[n - 1].clone()), cs[n].clone()));
        let prod = s::simplify(&div(mul(vec![int(if n % 2 == 0 { 1 } else { -1 }), cs[0].clone()]), cs[n].clone()));
        let (title, res) = match want.as_str() {
            "sum" => ("Sum of roots", math(&sum)),
            "product" => ("Product of roots", math(&prod)),
            _ => ("Sum and product of roots", text(format!("sum = {}, product = {}", show(&sum), show(&prod)))),
        };
        return Some(ans("Vieta", format!("{} of the roots of {} = 0", want, show(&s::simplify(&s::expand(&e)))), title, res, vec![pod("Note", "Method", text(format!("Vieta's formulas for a degree-{n} polynomial (roots counted with multiplicity, complex roots included)")))]));
    }
    None
}

/// lhs − rhs for an equation, or the expression itself.
pub(super) fn implicit_expr(t: &str) -> Option<Expr> {
    let e = ex(t)?;
    Some(match e {
        Rel(RelOp::Eq, a, b) => s::simplify(&sub(*a, *b)),
        Rel(..) => return None,
        e => e,
    })
}

// ---------------------------------------------------------------------------
// logarithmic and exponential forms
// ---------------------------------------------------------------------------

fn log_forms(q: &Query) -> Option<Answer> {
    let (_, l) = clean(q);
    let m = re!(r"^(?:express|write|convert|rewrite|what is)?\s*(?:the\s+)?(?:(exp(?:onential)?|log(?:arithmic)?)\s+form\s+of\s+(.+)|(.+?)\s+(?:in|to|as)\s+(?:an?\s+)?(exp(?:onential)?|log(?:arithmic)?)\s+form)$").captures(&l)?;
    let (kind, eqn) = if let Some(k) = m.get(1) { (k.as_str(), m[2].trim().to_string()) } else { (&m[4], m[3].trim().to_string()) };
    let (lhs, rhs) = eqn.split_once('=')?;
    let (lhs, rhs) = (lhs.trim(), rhs.trim());
    if kind.starts_with("exp") {
        // c log_b(A) = V  →  b^(V/c) = A
        let lm = re!(r"^([\d./]*)\s*(log|ln|lg)_?\s*(\d+|e)?\s*\(?\s*(.+?)\s*\)?$").captures(lhs)?;
        let coef = if lm[1].is_empty() { one() } else { ex(&lm[1])? };
        let base = match (&lm[2], lm.get(3)) {
            ("ln", _) => "e".to_string(),
            ("lg", None) => "2".into(),
            (_, Some(b)) => b.as_str().to_string(),
            _ => "10".into(),
        };
        let v = s::simplify(&div(ex(rhs)?, coef.clone()));
        let arg = ex(&lm[4])?;
        let vt = show(&v);
        let vt = if vt.contains(' ') || vt.contains('/') { format!("({vt})") } else { vt };
        let form = format!("{base}^{vt} = {}", show(&arg));
        let mut extra = Vec::new();
        if !coef.is_one() {
            extra.push(pod("Step", "First divide by the coefficient", text(format!("log_{base}({}) = {}", show(&arg), show(&v)))));
        }
        if let Some(val) = s::simplify(&pow(if base == "e" { crate::expr::e() } else { ex(&base)? }, v.clone())).to_f64_const() {
            if arg.free_symbols().len() == 1 {
                extra.push(pod("Solution", "Solution", text(format!("{} = {}", show(&arg), show(&s::simplify(&pow(if base == "e" { crate::expr::e() } else { ex(&base)? }, v)))) + &format!(" ≈ {}", crate::stats::fmt(val)))));
            }
        }
        return Some(ans("exponential form", format!("{eqn} in exponential form"), "Exponential form", text(form), extra));
    }
    // b^E = V  →  log_b(V) = E
    let pm = re!(r"^\(?([^\^]+?)\)?\s*\^\s*\(?(.+?)\)?$").captures(lhs)?;
    let (b, e) = (pm[1].trim(), pm[2].trim());
    let form = if b == "e" { format!("ln({rhs}) = {e}") } else if b == "10" { format!("log({rhs}) = {e}  (log base 10)") } else { format!("log_{b}({rhs}) = {e}") };
    Some(ans("logarithmic form", format!("{eqn} in logarithmic form"), "Logarithmic form", text(form), vec![]))
}

// ---------------------------------------------------------------------------
// n-th roots
// ---------------------------------------------------------------------------

fn ordinal_n(t: &str) -> Option<u64> {
    match t {
        "square" => Some(2),
        "cube" | "cubic" => Some(3),
        "fourth" => Some(4),
        "fifth" => Some(5),
        "sixth" => Some(6),
        "seventh" => Some(7),
        "eighth" => Some(8),
        "ninth" => Some(9),
        "tenth" => Some(10),
        _ => re!(r"^(\d+)(?:st|nd|rd|th)$").captures(t).and_then(|m| m[1].parse().ok()),
    }
}

fn nth_roots(q: &Query) -> Option<Answer> {
    let (_, l) = clean(q);
    let m = re!(r"^(?:find\s+)?(?:the\s+|all\s+(?:the\s+)?|all\s+of\s+the\s+)?(square|cube|cubic|fourth|fifth|sixth|seventh|eighth|ninth|tenth|\d+(?:st|nd|rd|th))\s+(roots?)\s+of\s+(.+)$").captures(&l)?;
    let n = ordinal_n(&m[1])?;
    if n < 2 || n > 64 {
        return None;
    }
    let all = l.contains("all") || &m[2] == "roots";
    let z = ex(&m[3])?;
    if !z.free_symbols().is_empty() {
        return None;
    }
    let zf = crate::eval::Program::compile(&z, &[])?.eval_c(&[]);
    if !zf.re.is_finite() || !zf.im.is_finite() {
        return None;
    }
    if !all {
        // principal root of a real number
        let r = s::simplify(&pow(z.clone(), Num(Rat::new(1.into(), BigInt::from(n)))));
        let mut extra = Vec::new();
        if let Some(v) = r.to_f64_const() {
            if !r.is_integer() {
                extra.push(pod("Decimal", "Decimal approximation", text(crate::stats::fmt(v))));
            }
        }
        if zf.im == 0.0 && zf.re < 0.0 && n % 2 == 1 {
            let real = s::simplify(&neg(pow(neg(z.clone()), Num(Rat::new(1.into(), BigInt::from(n))))));
            return Some(ans("root", format!("real {}-th root of {}", n, show(&z)), "Real root", math(&real), extra));
        }
        if zf.im == 0.0 && zf.re >= 0.0 {
            return Some(ans("root", format!("{}-th root of {}", n, show(&z)), "Result", math(&r), extra));
        }
    }
    // all n roots: r^(1/n) e^(i (θ + 2πk)/n)
    let (rad, theta) = (zf.norm(), zf.arg());
    let modulus = s::simplify(&pow(func("abs", vec![z.clone()]), Num(Rat::new(1.into(), BigInt::from(n)))));
    let rowsv: Vec<Vec<Content>> = (0..n).map(|k| {
        let ang = (theta + 2.0 * std::f64::consts::PI * k as f64) / n as f64;
        let w = num_complex::Complex64::from_polar(rad.powf(1.0 / n as f64), ang);
        let exact_arg = if zf.im == 0.0 {
            let base = if zf.re >= 0.0 { Num(Rat::new(BigInt::from(2 * k), BigInt::from(n))) } else { Num(Rat::new(BigInt::from(2 * k + 1), BigInt::from(n))) };
            Some(s::simplify(&mul(vec![base, pi()])))
        } else {
            None
        };
        let polar = match &exact_arg {
            Some(a) if a.is_zero() => show(&modulus),
            Some(a) => format!("{} e^(i {})", show(&modulus), show(a)),
            None => format!("{} e^(i {})", show(&modulus), crate::stats::fmt(ang)),
        };
        let fmt_c = |w: num_complex::Complex64| {
            let re_ = if w.re.abs() < 1e-12 { 0.0 } else { w.re };
            let im = if w.im.abs() < 1e-12 { 0.0 } else { w.im };
            if im == 0.0 { crate::stats::fmt(re_) } else if re_ == 0.0 { format!("{}i", crate::stats::fmt(im)) } else { format!("{} {} {}i", crate::stats::fmt(re_), if im < 0.0 { "−" } else { "+" }, crate::stats::fmt(im.abs())) }
        };
        vec![text(k.to_string()), text(polar), text(fmt_c(w))]
    }).collect();
    Some(ans("roots", format!("all {}-th roots of {}", n, show(&z)), &format!("All {n} roots"), table(Some(vec!["k", "polar form", "value"]), rowsv), vec![]))
}

// ---------------------------------------------------------------------------
// counting
// ---------------------------------------------------------------------------

fn count_partitions(n: usize, k: Option<usize>, filter: &dyn Fn(usize) -> bool, distinct: bool) -> BigInt {
    // dp over allowed part sizes
    let parts: Vec<usize> = (1..=n).filter(|&p| filter(p)).collect();
    match k {
        None => {
            let mut dp = vec![BigInt::zero(); n + 1];
            dp[0] = BigInt::one();
            for &p in &parts {
                if distinct {
                    for s in (p..=n).rev() {
                        let v = dp[s - p].clone();
                        dp[s] += v;
                    }
                } else {
                    for s in p..=n {
                        let v = dp[s - p].clone();
                        dp[s] += v;
                    }
                }
            }
            dp[n].clone()
        }
        Some(k) => {
            // dp[j][s]: partitions of s into exactly j parts from allowed sizes
            let mut dp = vec![vec![BigInt::zero(); n + 1]; k + 1];
            dp[0][0] = BigInt::one();
            for &p in &parts {
                if distinct {
                    for j in (1..=k).rev() {
                        for s in (p..=n).rev() {
                            let v = dp[j - 1][s - p].clone();
                            dp[j][s] += v;
                        }
                    }
                } else {
                    for j in 1..=k {
                        for s in p..=n {
                            let v = dp[j - 1][s - p].clone();
                            dp[j][s] += v;
                        }
                    }
                }
            }
            dp[k][n].clone()
        }
    }
}

fn list_partitions(n: usize, k: Option<usize>, filter: &dyn Fn(usize) -> bool, distinct: bool, limit: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    fn go(rem: usize, max: usize, cur: &mut Vec<usize>, k: Option<usize>, filter: &dyn Fn(usize) -> bool, distinct: bool, out: &mut Vec<Vec<usize>>, limit: usize) {
        if out.len() >= limit {
            return;
        }
        if rem == 0 {
            if k.map_or(true, |k| cur.len() == k) {
                out.push(cur.clone());
            }
            return;
        }
        if k.is_some_and(|k| cur.len() >= k) {
            return;
        }
        for p in (1..=rem.min(max)).rev() {
            if filter(p) {
                cur.push(p);
                go(rem - p, if distinct { p - 1 } else { p }, cur, k, filter, distinct, out, limit);
                cur.pop();
            }
        }
    }
    go(n, n, &mut Vec::new(), k, filter, distinct, &mut out, limit);
    out
}

fn necklaces(n: u64, k: &BigInt) -> BigInt {
    let mut s = BigInt::zero();
    for d in crate::numtheory::divisors(&big(n)) {
        let d = d.to_u64().unwrap_or(1);
        s += crate::numtheory::totient(&big(d)) * num_traits::pow(k.clone(), (n / d) as usize);
    }
    s / big(n)
}

fn counting(q: &Query) -> Option<Answer> {
    let (_, l) = clean(q);
    // partitions with restrictions
    if let Some(m) = re!(r"^(?:the\s+)?(?:number of\s+)?(odd|even|distinct)?\s*partitions\s+of\s+(\d+)(?:\s+into\s+(?:exactly\s+)?(?:(\d+)\s+)?(odd|even|distinct)?\s*parts?)?$|^partition\s+(\d+)\s+into\s+(?:(\d+)\s+)?(odd|even|distinct)?\s*parts$").captures(&l) {
        let n: usize = m.get(2).or(m.get(5))?.as_str().parse().ok()?;
        let k: Option<usize> = m.get(3).or(m.get(6)).and_then(|x| x.as_str().parse().ok());
        let kind = m.get(1).or(m.get(4)).or(m.get(7)).map(|x| x.as_str()).unwrap_or("");
        if kind.is_empty() && k.is_none() || n > 2000 {
            return None;
        }
        let filter: Box<dyn Fn(usize) -> bool> = match kind {
            "odd" => Box::new(|p| p % 2 == 1),
            "even" => Box::new(|p| p % 2 == 0),
            _ => Box::new(|_| true),
        };
        let distinct = kind == "distinct";
        let count = count_partitions(n, k, &*filter, distinct);
        let desc = format!("partitions of {n}{}{}", k.map(|k| format!(" into {k} parts")).unwrap_or_default(), if kind.is_empty() { String::new() } else { format!(" ({kind} parts)") });
        let mut extra = Vec::new();
        if count <= big(60) {
            let ls = list_partitions(n, k, &*filter, distinct, 60);
            extra.push(pod("List", "Partitions", text(ls.iter().map(|p| p.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" + ")).collect::<Vec<_>>().join("\n"))));
        }
        if kind == "odd" && k.is_none() {
            extra.push(pod("Note", "Euler's theorem", text("the number of partitions into odd parts equals the number into distinct parts")));
        }
        return Some(ans("partitions", desc, "Number of partitions", text(count.to_string()), extra));
    }
    // compositions
    if let Some(m) = re!(r"^(?:the\s+)?(?:number of\s+)?compositions\s+of\s+(\d+)(?:\s+into\s+(?:(\d+)\s+)?(odd|even|positive)?\s*parts)?$").captures(&l) {
        let n: u64 = m[1].parse().ok()?;
        if n == 0 || n > 5000 {
            return None;
        }
        let k: Option<u64> = m.get(2).and_then(|x| x.as_str().parse().ok());
        let kind = m.get(3).map(|x| x.as_str()).unwrap_or("");
        let count: BigInt = match (kind, k) {
            ("even", None) => if n % 2 == 1 { BigInt::zero() } else { num_traits::pow(big(2), (n / 2 - 1) as usize) },
            ("odd", None) => crate::numtheory::fib(n),
            (_, Some(k)) if kind.is_empty() || kind == "positive" => if k == 0 || k > n { BigInt::zero() } else { crate::numtheory::binomial(&big(n - 1), k - 1) },
            ("", None) | ("positive", None) => num_traits::pow(big(2), (n - 1) as usize),
            _ => return None,
        };
        let formula = match (kind, k) {
            ("even", None) => "2^(n/2 − 1) for even n, 0 for odd n",
            ("odd", None) => "Fibonacci number F(n)",
            (_, Some(_)) => "C(n − 1, k − 1)",
            _ => "2^(n − 1)",
        };
        return Some(ans("compositions", format!("compositions of {n}{}{}", k.map(|k| format!(" into {k} parts")).unwrap_or_default(), if kind.is_empty() { String::new() } else { format!(" ({kind} parts)") }), "Number of compositions", text(count.to_string()), vec![pod("Formula", "Formula", text(formula))]));
    }
    // necklaces, bracelets, Lyndon words
    if let Some(m) = re!(r"^(?:the\s+)?(?:number of\s+)?(?:(fixed|free)\s+)?(necklaces|bracelets)\s+(?:with\s+)?(\d+)\s+beads?,?\s*(?:and\s+|with\s+)?(\d+)\s+colou?rs?$|^(?:the\s+)?(?:number of\s+)?(\d+)-ary\s+lyndon words\s+of\s+length\s+(\d+)$").captures(&l) {
        if let Some(kk) = m.get(5) {
            let (k, n): (u64, u64) = (kk.as_str().parse().ok()?, m[6].parse().ok()?);
            if n == 0 || n > 2000 {
                return None;
            }
            let mut s = BigInt::zero();
            for d in crate::numtheory::divisors(&big(n)) {
                let d = d.to_u64()?;
                s += BigInt::from(crate::numtheory::mobius(&big(d))) * num_traits::pow(big(k), (n / d) as usize);
            }
            return Some(ans("Lyndon words", format!("{k}-ary Lyndon words of length {n}"), "Number", text((s / big(n)).to_string()), vec![pod("Formula", "Formula", text("(1/n) Σ_{d | n} μ(d) k^(n/d)"))]));
        }
        let (n, k): (u64, u64) = (m[3].parse().ok()?, m[4].parse().ok()?);
        if n == 0 || n > 2000 {
            return None;
        }
        let kb = big(k);
        let neck = necklaces(n, &kb);
        let brace: BigInt = if n % 2 == 1 {
            (&neck + num_traits::pow(kb.clone(), ((n + 1) / 2) as usize)) / 2
        } else {
            (&neck * 2 + (&kb + 1) * num_traits::pow(kb.clone(), (n / 2) as usize)) / 4
        };
        let bracelets = &m[2] == "bracelets" || m.get(1).is_some_and(|x| x.as_str() == "free");
        let (title, v) = if bracelets { ("Bracelets (turning over allowed)", brace.clone()) } else { ("Necklaces (rotations only)", neck.clone()) };
        return Some(ans("necklaces", format!("{} with {n} beads and {k} colors", if bracelets { "bracelets" } else { "necklaces" }), title, text(v.to_string()), vec![pod("Other", if bracelets { "Necklaces (rotations only)" } else { "Bracelets (turning over allowed)" }, text(if bracelets { neck } else { brace }.to_string()))]));
    }
    // Latin squares
    if let Some(m) = re!(r"^(?:the\s+)?(?:number of\s+)?(?:(\d+)\s*(?:x|×|by)\s*(\d+)\s+)?(normali[sz]ed\s+|reduced\s+)?latin squares(?:\s+of\s+order\s+(\d+))?$").captures(&l) {
        const ALL: [&str; 11] = ["1", "2", "12", "576", "161280", "812851200", "61479419904000", "108776032459082956800", "5524751496156892842531225600", "9982437658213039871725064756920320000", "776966836171770144107444346734230682311065600000"];
        const RED: [&str; 11] = ["1", "1", "1", "4", "56", "9408", "16942080", "535281401856", "377597570964258816", "7580721483160132811489280", "5363937773277371298119673540771840"];
        let reduced = m.get(3).is_some();
        let n: Option<usize> = m.get(1).or(m.get(4)).and_then(|x| x.as_str().parse().ok());
        let tbl = if reduced { &RED } else { &ALL };
        let what = if reduced { "reduced (normalized) Latin squares" } else { "Latin squares" };
        return Some(match n {
            Some(n) => ans("Latin squares", format!("number of {what} of order {n}"), "Number", text(tbl.get(n.checked_sub(1)?)?.to_string()), vec![]),
            None => ans("Latin squares", format!("number of {what}"), "Counts by order", table(Some(vec!["order n", "count"]), tbl.iter().enumerate().map(|(i, v)| vec![text((i + 1).to_string()), text(*v)]).collect()), vec![]),
        });
    }
    // Frobenius number
    if let Some(m) = re!(r"^(?:the\s+)?frobenius number\s+(?:of\s+)?\{?\s*([\d,\s]+?)\s*\}?$").captures(&l) {
        let a: Vec<usize> = m[1].split([',', ' ']).filter(|s| !s.is_empty()).map(|s| s.parse().ok()).collect::<Option<_>>()?;
        if a.len() < 2 || a.iter().any(|&x| x == 0 || x > 100_000) {
            return None;
        }
        let g = a.iter().fold(0usize, |g, &x| g.gcd(&x));
        if g != 1 {
            return Some(ans("Frobenius number", format!("Frobenius number of {{{}}}", m[1].trim()), "Result", text("none: the numbers have a common factor, so infinitely many integers are not representable"), vec![]));
        }
        let mn = *a.iter().min()?;
        // representable flags; the answer is below the largest pair bound
        let limit = a.iter().max()? * mn + 1;
        let mut ok = vec![false; limit + 1];
        ok[0] = true;
        for s in 1..=limit {
            ok[s] = a.iter().any(|&x| x <= s && ok[s - x]);
        }
        let f = (0..=limit).rev().find(|&s| !ok[s]).map(|s| s.to_string()).unwrap_or_else(|| "-1".into());
        return Some(ans("Frobenius number", format!("Frobenius number of {{{}}}", a.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")), "Result", text(f), vec![pod("Meaning", "Meaning", text("the largest integer that is not a non-negative integer combination of the numbers"))]));
    }
    // Ackermann function
    if let Some(m) = re!(r"^(?:what is\s+)?(?:the\s+)?ackermann(?:\s+function)?\s*[\[(]?\s*(\d+)\s*,\s*(\d+)\s*[\])]?$").captures(&l) {
        let (a, b): (u64, u64) = (m[1].parse().ok()?, m[2].parse().ok()?);
        let two = big(2);
        let v: String = match a {
            0 => (b + 1).to_string(),
            1 => (b + 2).to_string(),
            2 => (2 * b + 3).to_string(),
            3 if b < 100_000 => (num_traits::pow(two, (b + 3) as usize) - BigInt::from(3)).to_string(),
            4 if b == 0 => "13".into(),
            4 if b == 1 => "65533".into(),
            4 if b == 2 => {
                let v: BigInt = num_traits::pow(two, 65536) - BigInt::from(3);
                let s = v.to_string();
                format!("2^65536 − 3 = {}…{} ({} digits)", &s[..20], &s[s.len() - 10..], s.len())
            }
            _ => format!("2↑^{}({}) − 3 (a tower of exponents too large to write out)", a - 2, b + 3),
        };
        return Some(ans("Ackermann", format!("Ackermann function A({a}, {b})"), "Result", text(v), vec![pod("Definition", "Definition", text("A(0, n) = n + 1; A(m, 0) = A(m − 1, 1); A(m, n) = A(m − 1, A(m, n − 1))"))]));
    }
    // multinomial coefficient
    if let Some(m) = re!(r"^(?:the\s+)?multinomial(?:\s+coefficient)?\s*[\[(]?\s*([\d,\s]+?)\s*[\])]?$").captures(&l) {
        let a: Vec<u64> = m[1].split([',', ' ']).filter(|s| !s.is_empty()).map(|s| s.parse().ok()).collect::<Option<_>>()?;
        let n: u64 = a.iter().sum();
        if a.len() < 2 || n > 5000 {
            return None;
        }
        let v = a.iter().fold(fact(n), |acc, &k| acc / fact(k));
        return Some(ans("multinomial", format!("multinomial coefficient ({}; {})", n, a.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")), "Result", text(v.to_string()), vec![pod("Formula", "Formula", text(format!("{n}! / ({})", a.iter().map(|k| format!("{k}!")).collect::<Vec<_>>().join(" "))))]));
    }
    // Bernoulli numbers
    if let Some(m) = re!(r"^(?:the\s+)?(?:\d+(?:st|nd|rd|th)\s+)?bernoulli(?:\s+number)?\s*[\[(]?\s*(?:b\s*[\[(_]?\s*)?(\d+)\s*[\])]*$").captures(&l) {
        let n: usize = m[1].parse().ok()?;
        if n > 2000 {
            return None;
        }
        let b = crate::numtheory::bernoulli(n);
        let e = Num(b.clone());
        let mut extra = Vec::new();
        if !b.is_integer() {
            if let Some(v) = e.to_f64_const() {
                extra.push(pod("Decimal", "Decimal approximation", text(crate::stats::fmt(v))));
            }
        }
        return Some(ans("Bernoulli number", format!("Bernoulli number B_{n}"), "Result", math(&e), extra));
    }
    // all permutations of a small set
    if let Some(m) = re!(r"^(?:all\s+)?(?:the\s+)?permutations\s+of\s+\{?\s*([a-z0-9](?:\s*,\s*[a-z0-9])*)\s*\}?$").captures(&l) {
        let items: Vec<String> = m[1].split(',').map(|s| s.trim().to_string()).collect();
        if items.len() < 2 || items.len() > 6 {
            return None;
        }
        let mut perms: Vec<Vec<String>> = vec![vec![]];
        for _ in 0..items.len() {
            let mut next = Vec::new();
            for p in &perms {
                for x in items.iter().filter(|x| !p.contains(x)) {
                    let mut q = p.clone();
                    q.push(x.clone());
                    next.push(q);
                }
            }
            perms = next;
        }
        let n = perms.len();
        let list = perms.iter().map(|p| format!("{{{}}}", p.join(", "))).collect::<Vec<_>>().join("\n");
        return Some(ans("permutations", format!("permutations of {{{}}}", items.join(", ")), &format!("{n} permutations"), text(list), vec![]));
    }
    None
}

// ---------------------------------------------------------------------------
// everyday arithmetic in words
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum RoundMode {
    HalfEven,
    HalfOdd,
    HalfUp,
    HalfDown,
    AwayFromZero,
    TowardZero,
}

fn round_with(v: &Rat, mode: RoundMode) -> BigInt {
    let fl = v.floor().to_integer();
    let frac = v - Rat::from_integer(fl.clone());
    let half = Rat::new(1.into(), 2.into());
    if frac < half {
        return fl;
    }
    if frac > half {
        return fl + 1;
    }
    let up = &fl + 1;
    match mode {
        RoundMode::HalfUp => up,
        RoundMode::HalfDown => fl,
        RoundMode::HalfEven => if fl.is_even() { fl } else { up },
        RoundMode::HalfOdd => if fl.is_odd() { fl } else { up },
        RoundMode::AwayFromZero => if v.is_negative() { fl } else { up },
        RoundMode::TowardZero => if v.is_negative() { up } else { fl },
    }
}

fn everyday(q: &Query) -> Option<Answer> {
    let (_, l) = clean(q);
    // rounding rules
    if let Some(m) = re!(r"^(?:round\s+)?(\{[^}]+\}|-?[\d.]+)\s*(?:rounded\s+)?(?:so\s+)?(?:(?:the\s+)?half\s+)?(?:goes\s+|rounds?\s+)?(half\s+up|half\s+down|up|down|away from zero|towards? zero|half\s+(?:to\s+)?even|half\s+(?:to\s+)?odd|to the nearest (?:even|odd)(?: number)?|(?:to\s+)?nearest (?:even|odd)(?: number)?|half away from zero|half towards? zero|half\s+goes away from zero)$").captures(&l)
        .or_else(|| re!(r"^round\s+(\{[^}]+\}|-?[\d.]+)\s+(?:so\s+)?(?:that\s+)?(?:the\s+)?half\s+(?:goes\s+|rounds\s+)?(away from zero|towards? zero|up|down|to the nearest (?:even|odd)(?: number)?|(?:to\s+)?(?:even|odd))$").captures(&l))
        .or_else(|| re!(r"^(-?[\d.]+)\s+(?:round(?:ed)?)\s+(half\s+up|half\s+down|half\s+(?:towards?|toward)\s+zero|half away from zero|half\s+(?:to\s+)?even|half\s+(?:to\s+)?odd)$").captures(&l)) {
        let mode_t = m[2].to_string();
        let mode = if mode_t.contains("away") { RoundMode::AwayFromZero } else if mode_t.contains("toward") { RoundMode::TowardZero } else if mode_t.contains("odd") { RoundMode::HalfOdd } else if mode_t.contains("even") { RoundMode::HalfEven } else if mode_t.contains("up") { RoundMode::HalfUp } else { RoundMode::HalfDown };
        let vals: Vec<Rat> = m[1].trim_matches(['{', '}']).split(',').map(|t| ex(t.trim())?.as_rat().cloned()).collect::<Option<_>>()?;
        let out: Vec<String> = vals.iter().map(|v| round_with(v, mode).to_string()).collect();
        let res = if out.len() == 1 { out[0].clone() } else { format!("{{{}}}", out.join(", ")) };
        return Some(ans("rounding", format!("round {} ({})", m[1].trim(), mode_t.trim()), "Result", text(res), vec![]));
    }
    if let Some(m) = re!(r"^round\s+(-?\d+\.5)$").captures(&l) {
        let v = ex(&m[1])?.as_rat()?.clone();
        return Some(ans("rounding", format!("round {}", &m[1]), "Result", text(round_with(&v, RoundMode::HalfEven).to_string()), vec![pod("Other", "Other rules", rows(vec![("half up (toward +∞)", round_with(&v, RoundMode::HalfUp).to_string()), ("half down (toward −∞)", round_with(&v, RoundMode::HalfDown).to_string()), ("half away from zero", round_with(&v, RoundMode::AwayFromZero).to_string()), ("half toward zero", round_with(&v, RoundMode::TowardZero).to_string()), ("half to even (default)", round_with(&v, RoundMode::HalfEven).to_string())]))]));
    }
    // mixed number ↔ improper fraction, percent as fraction
    if let Some(m) = re!(r"^(?:convert\s+)?(\d+)\s+(\d+)/(\d+)\s+(?:as|to|in(?:to)?)\s+(?:an?\s+)?improper fraction$").captures(&l) {
        let (w, n, d): (i64, i64, i64) = (m[1].parse().ok()?, m[2].parse().ok()?, m[3].parse().ok()?);
        if d == 0 {
            return None;
        }
        let r = Rat::new((w * d + n).into(), d.into());
        return Some(ans("fraction", format!("{} {}/{} as an improper fraction", w, n, d), "Result", math(&Num(r)), vec![pod("Step", "Steps", text(format!("{w} × {d} + {n} = {}, over {d}", w * d + n)))]));
    }
    if let Some(m) = re!(r"^(?:convert\s+)?([\d.]+)\s*%\s+(?:as|to|in(?:to)?)\s+(?:an?\s+)?(mixed (?:fraction|number)|improper fraction|fraction|decimal)$").captures(&l) {
        let r = ex(&m[1])?.as_rat()?.clone() / Rat::from_integer(100.into());
        let res = match &m[2] {
            k if k.starts_with("mixed") => {
                let w = r.trunc().to_integer();
                let f = &r - Rat::from_integer(w.clone());
                if f.is_zero() { w.to_string() } else if w.is_zero() { format!("{}/{}", f.numer(), f.denom()) } else { format!("{w} {}/{}", f.numer(), f.denom()) }
            }
            "decimal" => crate::stats::fmt(r.to_f64()?),
            _ => show(&Num(r.clone())),
        };
        return Some(ans("percent", format!("{}% as a {}", &m[1], &m[2]), "Result", text(res), vec![]));
    }
    // multiplication and addition tables
    if let Some(m) = re!(r"^(\d+)\s*(?:x|×|by)\s*(\d+)\s+(multiplication|times|addition)\s+table$|^(multiplication|times|addition)\s+table\s+(?:up\s+to\s+|of\s+)?(\d+)$").captures(&l) {
        let (r, c, kind) = if let Some(a) = m.get(1) { (a.as_str().parse::<usize>().ok()?, m[2].parse::<usize>().ok()?, m[3].to_string()) } else { let n = m[5].parse::<usize>().ok()?; (n, n, m[4].to_string()) };
        if r == 0 || c == 0 || r > 30 || c > 30 {
            return None;
        }
        let plus = kind == "addition";
        let mut header: Vec<String> = vec![if plus { "+".into() } else { "×".into() }];
        header.extend((1..=c).map(|j| j.to_string()));
        let rowsv: Vec<Vec<Content>> = (1..=r).map(|i| {
            let mut row = vec![text(i.to_string())];
            row.extend((1..=c).map(|j| text(if plus { i + j } else { i * j }.to_string())));
            row
        }).collect();
        let hdr: Vec<&str> = header.iter().map(|s| s.as_str()).collect();
        return Some(ans("table", format!("{r} × {c} {} table", if plus { "addition" } else { "multiplication" }), "Table", table(Some(hdr), rowsv), vec![]));
    }
    // "$50 + 10% + 8%"
    if let Some(m) = re!(r"^\$?\s*([\d,.]+)\s*((?:[+-]\s*[\d.]+\s*%\s*)+)$").captures(&l) {
        let mut v = ex(&m[1].replace(',', ""))?.as_rat()?.clone();
        let mut steps = Vec::new();
        for p in re!(r"([+-])\s*([\d.]+)\s*%").captures_iter(&m[2]) {
            let pc = ex(&p[2])?.as_rat()?.clone() / Rat::from_integer(100.into());
            let f = if &p[1] == "+" { Rat::one() + pc } else { Rat::one() - pc };
            v *= f;
            steps.push(format!("{}{}% → {}", &p[1], &p[2], crate::stats::fmt(v.to_f64()?)));
        }
        let dollars = l.starts_with('$');
        let val = v.to_f64()?;
        let res = if dollars { format!("${val:.2}") } else { crate::stats::fmt(val) };
        return Some(ans("percent", format!("{} {}", &m[1], m[2].trim()), "Result", text(res), vec![pod("Steps", "Steps (each percentage applied to the running total)", text(steps.join("\n")))]));
    }
    None
}

pub fn handle(q: &Query) -> Option<Answer> {
    quat_rotation(q)
        .or_else(|| quaternions(q))
        .or_else(|| permutations(q))
        .or_else(|| groups(q))
        .or_else(|| matrices(q))
        .or_else(|| polynomials(q))
        .or_else(|| log_forms(q))
        .or_else(|| nth_roots(q))
        .or_else(|| counting(q))
        .or_else(|| everyday(q))
}

#[cfg(test)]
mod tests {
    fn ask(s: &str) -> String {
        let r = crate::engine::query(s);
        serde_json::to_string(&r.pods).unwrap_or_default()
    }

    #[test]
    fn toolbox_answers() {
        assert!(ask("perm (1 2 3 4)^3(1 2 3)^-1").contains("order"));
        assert!(ask("number of groups of order 120").contains("47"));
        assert!(ask("alternating group A_5").contains("60"));
        assert!(ask("(1,2)-minor of {{1,2,3},{3,4,5},{5,4,3}}").contains("-16"));
        assert!(ask("find polynomial with zeros at 2, 3").contains("x^2 - 5 x + 6"));
        assert!(ask("interpolating polynomial {1,10},{2,3},{4,7},{8,0}").contains("Polynomial"));
        assert!(ask("find the product of roots for x^2 + 4x + 6 = 0").contains('6'));
        assert!(ask("express 2^3 = 8 in log form").contains("log_2(8) = 3"));
        assert!(ask("partitions of 38 into 6 parts").contains("1540"));
        assert!(ask("bracelets 7 beads, 4 colors").contains("1300"));
        assert!(ask("Frobenius number {4, 7, 12}").contains("17"));
        assert!(ask("what is ackermann 4,1?").contains("65533"));
        assert!(ask("multinomial(3,4,5,8)").contains("3491888400"));
        assert!(ask("round {2.5, 5.5, 7.5} so half goes to the nearest odd number").contains("{3, 5, 7}"));
        assert!(ask("912/7 quotient and remainder").contains("130 remainder 2"));
        assert!(ask("norm of quaternion 1+3i+4j+3^5k").contains("sqrt"));
    }
}
