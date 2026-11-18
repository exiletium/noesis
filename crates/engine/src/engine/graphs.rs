//! Graphs by name or by edge list: complete, cycle, path, wheel, star,
//! complete bipartite, hypercube, Turán and Petersen graphs, k-ary trees,
//! the smallest (k, g)-cages, random graphs G(n, m), and graphs typed as
//! "1->2, 2->3, 3->1". Small graphs get exact invariants (degree sequence,
//! connectivity, diameter, girth, bipartiteness, chromatic number, the
//! matching polynomial) and Eulerian or Hamiltonian cycles on request.

use super::*;
use regex::Regex;
use std::collections::VecDeque;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

#[derive(Clone)]
struct Graph {
    n: usize,
    edges: Vec<(usize, usize)>,
    directed: bool,
    labels: Option<Vec<String>>,
}

impl Graph {
    fn new(n: usize) -> Graph {
        Graph { n, edges: Vec::new(), directed: false, labels: None }
    }
    fn add(&mut self, a: usize, b: usize) {
        if a != b && !self.edges.iter().any(|&(x, y)| (x, y) == (a, b) || (!self.directed && (x, y) == (b, a))) {
            self.edges.push((a, b));
        }
    }
    fn adj(&self) -> Vec<Vec<usize>> {
        let mut v = vec![Vec::new(); self.n];
        for &(a, b) in &self.edges {
            v[a].push(b);
            if !self.directed {
                v[b].push(a);
            }
        }
        v
    }
    fn label(&self, i: usize) -> String {
        self.labels.as_ref().map(|l| l[i].clone()).unwrap_or_else(|| (i + 1).to_string())
    }
}

// ---------------------------------------------------------------------------
// constructions
// ---------------------------------------------------------------------------

fn complete(n: usize) -> Graph {
    let mut g = Graph::new(n);
    for a in 0..n {
        for b in a + 1..n {
            g.add(a, b);
        }
    }
    g
}

fn cycle(n: usize) -> Graph {
    let mut g = Graph::new(n);
    for a in 0..n {
        g.add(a, (a + 1) % n);
    }
    g
}

fn path(n: usize) -> Graph {
    let mut g = Graph::new(n);
    for a in 1..n {
        g.add(a - 1, a);
    }
    g
}

/// Wheel with n vertices: a hub joined to a cycle of n − 1.
fn wheel(n: usize) -> Graph {
    let mut g = Graph::new(n);
    for a in 1..n {
        g.add(0, a);
        g.add(a, if a + 1 < n { a + 1 } else { 1 });
    }
    g
}

fn star(n: usize) -> Graph {
    let mut g = Graph::new(n);
    for a in 1..n {
        g.add(0, a);
    }
    g
}

fn bipartite(m: usize, n: usize) -> Graph {
    let mut g = Graph::new(m + n);
    for a in 0..m {
        for b in 0..n {
            g.add(a, m + b);
        }
    }
    g
}

fn hypercube(d: usize) -> Graph {
    let n = 1usize << d;
    let mut g = Graph::new(n);
    for a in 0..n {
        for k in 0..d {
            let b = a ^ (1 << k);
            if a < b {
                g.add(a, b);
            }
        }
    }
    g
}

fn turan(n: usize, r: usize) -> Graph {
    let mut g = Graph::new(n);
    for a in 0..n {
        for b in a + 1..n {
            if a % r != b % r {
                g.add(a, b);
            }
        }
    }
    g
}

fn petersen() -> Graph {
    let mut g = Graph::new(10);
    for i in 0..5 {
        g.add(i, (i + 1) % 5);
        g.add(i, i + 5);
        g.add(5 + i, 5 + (i + 2) % 5);
    }
    g
}

/// Skeleton of the octahedron: K(2,2,2), every vertex joined to all but its opposite.
fn octahedral() -> Graph {
    let mut g = Graph::new(6);
    for a in 0..6 {
        for b in a + 1..6 {
            if b != a + 3 {
                g.add(a, b);
            }
        }
    }
    g
}

/// Skeleton of the icosahedron: apex, two staggered pentagons, nadir.
fn icosahedral() -> Graph {
    let mut g = Graph::new(12);
    for i in 0..5 {
        let (u, u2, w, w2) = (1 + i, 1 + (i + 1) % 5, 6 + i, 6 + (i + 1) % 5);
        g.add(0, u);
        g.add(u.min(u2), u.max(u2));
        g.add(w.min(w2), w.max(w2));
        g.add(w, 11);
        g.add(u, w);
        g.add(u2.min(w), u2.max(w));
    }
    g
}

/// A cubic Hamiltonian graph from LCF notation: the cycle 0..n plus i ~ i + s_i.
fn lcf(shifts: &[i64], repeat: usize) -> Graph {
    let n = shifts.len() * repeat;
    let mut g = Graph::new(n);
    for i in 0..n {
        g.add(i.min((i + 1) % n), i.max((i + 1) % n));
        let j = (i as i64 + shifts[i % shifts.len()]).rem_euclid(n as i64) as usize;
        if !g.edges.contains(&(i.min(j), i.max(j))) {
            g.add(i.min(j), i.max(j));
        }
    }
    g
}

/// Named cubic graphs: names, LCF shifts, repeats, notes.
const LCF_GRAPHS: &[(&[&str], &[i64], usize, &str)] = &[
    (&["pappus graph"], &[5, 7, -7, 7, -7, -5], 3, "bipartite, distance-regular; the Levi graph of the Pappus configuration; LCF [5,7,−7,7,−7,−5]³"),
    (&["heawood graph"], &[5, -5], 7, "the (3,6)-cage; the Levi graph of the Fano plane; LCF [5,−5]⁷"),
    (&["mobius-kantor graph", "möbius-kantor graph", "mobius kantor graph", "möbius–kantor graph"], &[5, -5], 8, "generalized Petersen graph GP(8,3); LCF [5,−5]⁸"),
    (&["desargues graph"], &[5, -5, 9, -9], 5, "generalized Petersen graph GP(10,3); the Levi graph of the Desargues configuration; LCF [5,−5,9,−9]⁵"),
    (&["franklin graph"], &[5, -5], 6, "a 3-regular graph on 12 vertices that is bipartite; LCF [5,−5]⁶"),
    (&["frucht graph"], &[-5, -2, -4, 2, 5, -2, 2, 5, -2, -5, 4, 2], 1, "the smallest cubic graph with no nontrivial automorphism (Frucht, 1939)"),
    (&["nauru graph"], &[5, -9, 7, -7, 9, -5], 4, "generalized Petersen graph GP(12,5); LCF [5,−9,7,−7,9,−5]⁴"),
    (&["mcgee graph"], &[12, 7, -7], 8, "the (3,7)-cage; LCF [12,7,−7]⁸"),
    (&["tutte-coxeter graph", "tutte–coxeter graph", "tutte coxeter graph", "tutte 8-cage"], &[-13, -9, 7, -7, 9, 13], 5, "the (3,8)-cage; LCF [−13,−9,7,−7,9,13]⁵"),
    (&["dodecahedral graph", "dodecahedron graph"], &[10, 7, 4, -4, -7, 10, -4, 7, -7, 4], 2, "the skeleton of the regular dodecahedron; LCF [10,7,4,−4,−7,10,−4,7,−7,4]²"),
    (&["cubical graph", "cube graph"], &[3, -3], 4, "the skeleton of the cube, the hypercube Q₃; LCF [3,−3]⁴"),
];

fn kary_tree(k: usize, levels: usize) -> Option<Graph> {
    // levels counts the rows of vertices, root included
    let mut n: u128 = 0;
    let mut row: u128 = 1;
    for _ in 0..levels {
        n += row;
        row = row.checked_mul(k as u128)?;
    }
    if n > 2000 {
        return None;
    }
    let n = n as usize;
    let mut g = Graph::new(n);
    for v in 1..n {
        g.add((v - 1) / k, v);
    }
    Some(g)
}

// ---------------------------------------------------------------------------
// invariants
// ---------------------------------------------------------------------------

fn bfs(adj: &[Vec<usize>], s: usize) -> Vec<Option<usize>> {
    let mut d = vec![None; adj.len()];
    d[s] = Some(0);
    let mut q = VecDeque::from([s]);
    while let Some(u) = q.pop_front() {
        for &w in &adj[u] {
            if d[w].is_none() {
                d[w] = Some(d[u].unwrap() + 1);
                q.push_back(w);
            }
        }
    }
    d
}

fn undirected(g: &Graph) -> Vec<Vec<usize>> {
    let mut v = vec![Vec::new(); g.n];
    for &(a, b) in &g.edges {
        v[a].push(b);
        v[b].push(a);
    }
    v
}

fn components(g: &Graph) -> usize {
    let adj = undirected(g);
    let mut seen = vec![false; g.n];
    let mut c = 0;
    for s in 0..g.n {
        if !seen[s] {
            c += 1;
            for (i, d) in bfs(&adj, s).iter().enumerate() {
                if d.is_some() {
                    seen[i] = true;
                }
            }
        }
    }
    c
}

fn diameter(g: &Graph) -> Option<usize> {
    let adj = undirected(g);
    let mut best = 0;
    for s in 0..g.n {
        for d in bfs(&adj, s) {
            best = best.max(d?);
        }
    }
    Some(best)
}

fn girth(g: &Graph) -> Option<usize> {
    let adj = undirected(g);
    let mut best: Option<usize> = None;
    for s in 0..g.n {
        let mut dist = vec![usize::MAX; g.n];
        let mut parent = vec![usize::MAX; g.n];
        dist[s] = 0;
        let mut q = VecDeque::from([s]);
        while let Some(u) = q.pop_front() {
            for &w in &adj[u] {
                if dist[w] == usize::MAX {
                    dist[w] = dist[u] + 1;
                    parent[w] = u;
                    q.push_back(w);
                } else if parent[u] != w {
                    let c = dist[u] + dist[w] + 1;
                    best = Some(best.map_or(c, |b| b.min(c)));
                }
            }
        }
    }
    best
}

fn is_bipartite(g: &Graph) -> bool {
    let adj = undirected(g);
    let mut col = vec![-1i8; g.n];
    for s in 0..g.n {
        if col[s] >= 0 {
            continue;
        }
        col[s] = 0;
        let mut q = VecDeque::from([s]);
        while let Some(u) = q.pop_front() {
            for &w in &adj[u] {
                if col[w] < 0 {
                    col[w] = 1 - col[u];
                    q.push_back(w);
                } else if col[w] == col[u] {
                    return false;
                }
            }
        }
    }
    true
}

fn chromatic_number(g: &Graph) -> Option<usize> {
    if g.n > 40 {
        return None;
    }
    let adj = undirected(g);
    if g.edges.is_empty() {
        return Some(if g.n == 0 { 0 } else { 1 });
    }
    if is_bipartite(g) {
        return Some(2);
    }
    // order vertices by degree, then backtrack with k colours
    let mut order: Vec<usize> = (0..g.n).collect();
    order.sort_by_key(|&v| std::cmp::Reverse(adj[v].len()));
    for k in 3..=g.n {
        let mut col = vec![usize::MAX; g.n];
        let mut budget = 2_000_000u64;
        if color(&adj, &order, 0, k, &mut col, &mut budget) {
            return Some(k);
        }
        if budget == 0 {
            return None;
        }
    }
    Some(g.n)
}

fn color(adj: &[Vec<usize>], order: &[usize], i: usize, k: usize, col: &mut [usize], budget: &mut u64) -> bool {
    if i == order.len() {
        return true;
    }
    if *budget == 0 {
        return false;
    }
    *budget -= 1;
    let v = order[i];
    let used: Vec<usize> = adj[v].iter().map(|&w| col[w]).collect();
    let max_used = col.iter().filter(|&&c| c != usize::MAX).max().map_or(0, |m| m + 1);
    for c in 0..k.min(max_used + 1) {
        if !used.contains(&c) {
            col[v] = c;
            if color(adj, order, i + 1, k, col, budget) {
                return true;
            }
            col[v] = usize::MAX;
        }
    }
    false
}

/// Matching polynomial Σ (−1)^k m_k x^(n−2k), from counts of k-edge matchings.
fn matching_counts(g: &Graph) -> Option<Vec<u128>> {
    if g.edges.len() > 40 {
        return None;
    }
    let mut counts = vec![0u128; g.n / 2 + 1];
    fn rec(edges: &[(usize, usize)], i: usize, used: u64, k: usize, counts: &mut [u128]) {
        if i == edges.len() {
            counts[k] += 1;
            return;
        }
        rec(edges, i + 1, used, k, counts);
        let (a, b) = edges[i];
        if used & (1 << a) == 0 && used & (1 << b) == 0 {
            rec(edges, i + 1, used | (1 << a) | (1 << b), k + 1, counts);
        }
    }
    if g.n > 64 {
        return None;
    }
    rec(&g.edges, 0, 0, 0, &mut counts);
    Some(counts)
}

fn matching_poly(g: &Graph) -> Option<String> {
    let c = matching_counts(g)?;
    let mut terms = Vec::new();
    for (k, &m) in c.iter().enumerate() {
        if m == 0 {
            continue;
        }
        let p = g.n - 2 * k;
        let sign = if k % 2 == 0 { "+" } else { "−" };
        let coef = if m == 1 && p > 0 { String::new() } else { m.to_string() };
        let var = match p {
            0 => String::new(),
            1 => "x".into(),
            _ => format!("x^{p}"),
        };
        terms.push(format!("{sign} {coef}{var}"));
    }
    let s = terms.join(" ");
    Some(s.trim_start_matches("+ ").to_string())
}

fn eulerian(g: &Graph) -> Option<Vec<usize>> {
    // Hierholzer's algorithm; directed or undirected
    if g.edges.is_empty() {
        return None;
    }
    let m = g.edges.len();
    let mut adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); g.n];
    for (i, &(a, b)) in g.edges.iter().enumerate() {
        adj[a].push((b, i));
        if !g.directed {
            adj[b].push((a, i));
        }
    }
    let start = if g.directed {
        let mut outd = vec![0i64; g.n];
        for &(a, b) in &g.edges {
            outd[a] += 1;
            outd[b] -= 1;
        }
        if outd.iter().any(|d| *d != 0) {
            return None;
        }
        g.edges[0].0
    } else {
        if adj.iter().any(|v| v.len() % 2 == 1) {
            return None;
        }
        g.edges[0].0
    };
    let mut used = vec![false; m];
    let mut ptr = vec![0usize; g.n];
    let mut stack = vec![start];
    let mut circuit = Vec::new();
    while let Some(&v) = stack.last() {
        let mut advanced = false;
        while ptr[v] < adj[v].len() {
            let (w, e) = adj[v][ptr[v]];
            ptr[v] += 1;
            if !used[e] {
                used[e] = true;
                stack.push(w);
                advanced = true;
                break;
            }
        }
        if !advanced {
            circuit.push(stack.pop()?);
        }
    }
    if used.iter().any(|u| !u) {
        return None;
    }
    circuit.reverse();
    Some(circuit)
}

fn hamiltonian(g: &Graph) -> Option<Vec<usize>> {
    if g.n > 20 || g.n < 3 {
        return None;
    }
    let adj = if g.directed { g.adj() } else { undirected(g) };
    let mut path = vec![0usize];
    let mut seen = vec![false; g.n];
    seen[0] = true;
    let mut budget = 3_000_000u64;
    fn go(adj: &[Vec<usize>], path: &mut Vec<usize>, seen: &mut [bool], budget: &mut u64) -> bool {
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        let v = *path.last().unwrap();
        if path.len() == adj.len() {
            return adj[v].contains(&path[0]);
        }
        for &w in &adj[v] {
            if !seen[w] {
                seen[w] = true;
                path.push(w);
                if go(adj, path, seen, budget) {
                    return true;
                }
                path.pop();
                seen[w] = false;
            }
        }
        false
    }
    go(&adj, &mut path, &mut seen, &mut budget).then(|| {
        path.push(0);
        path
    })
}

// ---------------------------------------------------------------------------
// answer
// ---------------------------------------------------------------------------

fn describe(g: &Graph, name: &str, extra_rows: Vec<(&str, String)>, want: Option<&str>) -> Answer {
    let adj = if g.directed { undirected(g) } else { g.adj() };
    let mut degs: Vec<usize> = adj.iter().map(|v| v.len()).collect();
    degs.sort_unstable_by(|a, b| b.cmp(a));
    let regular = degs.first() == degs.last();
    let mut rows: Vec<(&str, String)> = vec![("vertices", g.n.to_string()), ("edges", g.edges.len().to_string())];
    if g.n <= 60 {
        rows.push(("degree sequence", format!("{:?}", degs).replace(['[', ']'], "")));
    } else {
        rows.push(("degrees", format!("from {} to {}", degs.last().unwrap_or(&0), degs.first().unwrap_or(&0))));
    }
    if regular && g.n > 0 {
        rows.push(("regular", format!("yes, {}-regular", degs[0])));
    }
    let comps = components(g);
    rows.push(("connected", if comps == 1 { "yes".into() } else { format!("no ({comps} components)") }));
    if g.n <= 3000 {
        if comps == 1 {
            if let Some(d) = diameter(g) {
                rows.push(("diameter", d.to_string()));
            }
        }
        rows.push(("girth", girth(g).map(|x| x.to_string()).unwrap_or_else(|| "∞ (acyclic)".into())));
        rows.push(("bipartite", if is_bipartite(g) { "yes".into() } else { "no".into() }));
    }
    if g.n <= 40 {
        if let Some(c) = chromatic_number(g) {
            rows.push(("chromatic number", c.to_string()));
        }
    }
    rows.extend(extra_rows);
    let mut pods = vec![pod("Input", "Input interpretation", text(name.to_string()))];
    match want {
        Some("matching") => {
            if let Some(p) = matching_poly(g) {
                pods.push(primary(pod("Result", "Matching polynomial", text(p))));
            }
        }
        Some("euler") => {
            let res = match eulerian(g) {
                Some(c) => c.iter().map(|&v| g.label(v)).collect::<Vec<_>>().join(" → "),
                None => "none: an Eulerian cycle needs every vertex to have even degree (equal in- and out-degree when directed) and all edges in one component".into(),
            };
            pods.push(primary(pod("Result", "Eulerian cycle", text(res))));
        }
        Some("hamilton") => {
            let res = match hamiltonian(g) {
                Some(c) => c.iter().map(|&v| g.label(v)).collect::<Vec<_>>().join(" → "),
                None => "none found".into(),
            };
            pods.push(primary(pod("Result", "Hamiltonian cycle", text(res))));
        }
        Some("edges") => pods.push(primary(pod("Result", "Number of edges", text(g.edges.len().to_string())))),
        _ => {}
    }
    let has_primary = pods.iter().any(|p| p.primary);
    let props = pod("Properties", "Properties", table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect()));
    pods.push(if has_primary { props } else { primary(props) });
    if g.edges.len() <= 60 {
        let arrow = if g.directed { "→" } else { "–" };
        pods.push(pod("Edges", "Edge list", text(g.edges.iter().map(|&(a, b)| format!("{}{arrow}{}", g.label(a), g.label(b))).collect::<Vec<_>>().join(", "))));
    }
    if !g.directed && g.edges.len() <= 40 && want != Some("matching") {
        if let Some(p) = matching_poly(g) {
            pods.push(pod("Matching", "Matching polynomial", text(p)));
        }
    }
    Answer::new("graph", pods)
}

/// Smallest (k, g)-cages: (degree, girth) → (vertices, name).
const CAGES: &[(usize, usize, usize, &str)] = &[
    (3, 3, 4, "complete graph K₄"), (3, 4, 6, "complete bipartite graph K₃,₃"), (3, 5, 10, "Petersen graph"), (3, 6, 14, "Heawood graph"), (3, 7, 24, "McGee graph"), (3, 8, 30, "Tutte–Coxeter graph"), (3, 9, 58, "18 cages on 58 vertices"), (3, 10, 70, "3 cages on 70 vertices"), (3, 11, 112, "Balaban 11-cage"), (3, 12, 126, "Tutte 12-cage"),
    (4, 3, 5, "complete graph K₅"), (4, 4, 8, "complete bipartite graph K₄,₄"), (4, 5, 19, "Robertson graph"), (4, 6, 26, "incidence graph of the generalized triangle GH(3,3)"),
    (5, 5, 30, "four cages including the Foster cage"), (6, 5, 40, "unique cage"), (7, 5, 50, "Hoffman–Singleton graph"),
];

fn prime_power(q: usize) -> bool {
    if q < 2 {
        return false;
    }
    let p = (2..=q).find(|d| q % d == 0).unwrap();
    let mut r = q;
    while r % p == 0 {
        r /= p;
    }
    r == 1
}

fn named(l: &str) -> Option<(Graph, String, Vec<(&'static str, String)>)> {
    let num = |s: &str| s.parse::<usize>().ok().filter(|n| *n >= 1 && *n <= 400);
    if let Some(m) = re!(r"^(?:the\s+)?(?:complete graph\s+k_?\s*(\d+)|k_?(\d+)\s+graph|(\d+)[- ]?complete graph|complete graph (?:on|with) (\d+) vertices)$").captures(l) {
        let n = num(m.get(1).or(m.get(2)).or(m.get(3)).or(m.get(4))?.as_str())?;
        return Some((complete(n), format!("complete graph K_{n}"), vec![]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:(\d+)[- ]?(?:cycle|cycle graph)|cycle graph\s+c_?\s*(\d+)|c_?(\d+)\s+graph)$").captures(l) {
        let n = num(m.get(1).or(m.get(2)).or(m.get(3))?.as_str()).filter(|n| *n >= 3)?;
        return Some((cycle(n), format!("cycle graph C_{n}"), vec![]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:(\d+)[- ]?(?:path|path graph)|path graph\s+p_?\s*(\d+))$").captures(l) {
        let n = num(m.get(1).or(m.get(2))?.as_str())?;
        return Some((path(n), format!("path graph P_{n}"), vec![]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:(\d+)[- ]?wheel(?: graph)?|wheel graph\s+w_?\s*(\d+))$").captures(l) {
        let n = num(m.get(1).or(m.get(2))?.as_str()).filter(|n| *n >= 4)?;
        return Some((wheel(n), format!("wheel graph W_{n} ({n} vertices: a hub and a {}-cycle)", n - 1), vec![]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:(\d+)[- ]?star(?: graph)?|star graph\s+s_?\s*(\d+))$").captures(l) {
        let n = num(m.get(1).or(m.get(2))?.as_str()).filter(|n| *n >= 2)?;
        return Some((star(n), format!("star graph with {n} vertices"), vec![]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:complete bipartite graph\s+)?k_?\s*\{?(\d+)\s*,\s*(\d+)\}?(?:\s+graph)?$").captures(l) {
        let (a, b) = (num(&m[1])?, num(&m[2])?);
        return Some((bipartite(a, b), format!("complete bipartite graph K_{a},{b}"), vec![]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:(\d+)[- ]?(?:dimensional\s+)?(?:hypercube|cube) graph|hypercube graph\s+q_?\s*(\d+))$").captures(l) {
        let d = num(m.get(1).or(m.get(2))?.as_str()).filter(|d| *d <= 10)?;
        return Some((hypercube(d), format!("{d}-dimensional hypercube graph Q_{d}"), vec![]));
    }
    if let Some((names, sh, r, note)) = LCF_GRAPHS.iter().find(|(names, ..)| names.contains(&l.trim_start_matches("the "))) {
        let name = names[0].split(' ').map(|w| w[..1].to_uppercase() + &w[1..]).collect::<Vec<_>>().join(" ").replace(" Graph", " graph");
        return Some((lcf(sh, *r), name, vec![("note", note.to_string())]));
    }
    match l.trim_start_matches("the ") {
        "tetrahedral graph" | "tetrahedron graph" => return Some((complete(4), "tetrahedral graph".into(), vec![("note", "the skeleton of the tetrahedron, the complete graph K₄".into())])),
        "octahedral graph" | "octahedron graph" => return Some((octahedral(), "octahedral graph".into(), vec![("note", "the skeleton of the octahedron, the complete tripartite graph K(2,2,2)".into())])),
        "icosahedral graph" | "icosahedron graph" => return Some((icosahedral(), "icosahedral graph".into(), vec![("note", "the skeleton of the icosahedron; 5-regular, planar, distance-regular; automorphism group of order 120".into())])),
        _ => {}
    }
    if l == "petersen graph" || l == "the petersen graph" {
        return Some((petersen(), "Petersen graph".into(), vec![("automorphisms", "120 (the symmetric group S₅)".into()), ("note", "the (3,5)-cage; not Hamiltonian, but every vertex-deleted subgraph is".into())]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:\((\d+)\s*,\s*(\d+)\)[- ]?turan graph|turan graph\s+t\s*\(\s*(\d+)\s*,\s*(\d+)\s*\))$").captures(l) {
        let (n, r) = (num(m.get(1).or(m.get(3))?.as_str())?, num(m.get(2).or(m.get(4))?.as_str())?);
        let e = turan(n, r).edges.len();
        return Some((turan(n, r), format!("Turán graph T({n},{r})"), vec![("Turán's theorem", format!("the largest graph on {n} vertices with no K_{} has {e} edges", r + 1))]));
    }
    if let Some(m) = re!(r"^(?:the\s+)?(?:(\d+)[- ]level\s+)?(?:(\d+)[- ]?ary|binary|ternary)\s+tree(?:\s+(?:with|of)\s+(\d+)\s+levels)?$").captures(l) {
        let k = if l.contains("binary") { 2 } else if l.contains("ternary") { 3 } else { num(m.get(2)?.as_str())? };
        let levels = m.get(1).or(m.get(3)).and_then(|x| num(x.as_str())).unwrap_or(4);
        let total: f64 = (0..levels).map(|i| (k as f64).powi(i as i32)).sum();
        let leaves = (k as f64).powi(levels as i32 - 1);
        let rows = vec![("vertices (formula)", format!("(k^L − 1)/(k − 1) = {}", crate::stats::fmt(total))), ("leaves", crate::stats::fmt(leaves)), ("height", (levels - 1).to_string())];
        if let Some(g) = kary_tree(k, levels) {
            return Some((g, format!("complete {k}-ary tree with {levels} levels"), rows));
        }
        // too large to build: report the counts only
        let mut empty = Graph::new(0);
        empty.n = 0;
        return Some((empty, format!("complete {k}-ary tree with {levels} levels (too large to draw)"), rows));
    }
    None
}

fn random_graph(l: &str) -> Option<(Graph, String)> {
    let m = re!(r"^random graph(?:\s+on)?\s+(\d+)\s+vertices(?:\s*,?\s*(?:and\s+)?(\d+)\s+edges)?$").captures(l)?;
    let n: usize = m[1].parse().ok().filter(|n| (2..=200).contains(n))?;
    let maxe = n * (n - 1) / 2;
    let e: usize = m.get(2).and_then(|x| x.as_str().parse().ok()).unwrap_or(maxe / 3).min(maxe);
    let mut seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1) | 1;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut g = Graph::new(n);
    while g.edges.len() < e {
        let (a, b) = ((next() % n as u64) as usize, (next() % n as u64) as usize);
        g.add(a.min(b), a.max(b));
    }
    Some((g, format!("random graph G({n}, {e}) (uniform, freshly sampled)")))
}

fn edge_list(raw: &str) -> Option<Graph> {
    let items: Vec<_> = re!(r"([A-Za-z0-9]+)\s*(->|→|<->|--|—|-)\s*([A-Za-z0-9]+)").captures_iter(raw).collect();
    if items.len() < 2 {
        return None;
    }
    // everything else must be separators
    let rest = re!(r"([A-Za-z0-9]+)\s*(->|→|<->|--|—|-)\s*([A-Za-z0-9]+)").replace_all(raw, "");
    if !rest.chars().all(|c| c == ',' || c == ' ' || c == '{' || c == '}' || c == ';') {
        return None;
    }
    // edges are separated by commas or semicolons ("carbon-14 half-life" is not two edges)
    if rest.chars().filter(|c| *c == ',' || *c == ';').count() + 1 < items.len() {
        return None;
    }
    let directed = items.iter().all(|c| &c[2] == "->" || &c[2] == "→");
    let mut labels: Vec<String> = Vec::new();
    let idx = |s: &str, labels: &mut Vec<String>| -> usize {
        match labels.iter().position(|x| x == s) {
            Some(i) => i,
            None => {
                labels.push(s.to_string());
                labels.len() - 1
            }
        }
    };
    let pairs: Vec<(usize, usize)> = items.iter().map(|c| (idx(&c[1], &mut labels), idx(&c[3], &mut labels))).collect();
    let mut g = Graph::new(labels.len());
    g.directed = directed;
    for (a, b) in pairs {
        g.add(a, b);
    }
    g.labels = Some(labels);
    Some(g)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let raw = q.clean.trim().trim_end_matches(['?', '.']);
    let l = raw.to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    // what is asked
    let want = if l.contains("matching polynomial") {
        Some("matching")
    } else if l.contains("euler") {
        Some("euler")
    } else if l.contains("hamilton") {
        Some("hamilton")
    } else if l.contains("edge count") || l.starts_with("number of edges") {
        Some("edges")
    } else {
        None
    };
    let body = re!(r"^(?:(?:the\s+)?(?:matching polynomial|eulerian (?:cycle|circuit|path)|hamiltonian (?:cycle|path)|edge count|number of edges)\s+(?:of|in|for)\s+(?:the\s+)?)|\s+(?:eulerian (?:cycle|circuit|path)|hamiltonian (?:cycle|path)|edge count|number of edges)$").replace_all(&l, "").trim().to_string();
    // cages
    if let Some(m) = re!(r"^\(?(\d+)\s*,\s*(\d+)\)?[- ]?cage(?: graph)?$").captures(&body) {
        let (k, g): (usize, usize) = (m[1].parse().ok()?, m[2].parse().ok()?);
        let res = match CAGES.iter().find(|c| c.0 == k && c.1 == g && c.2 > 0) {
            Some(c) => format!("{} vertices: {}", c.2, c.3),
            None if g == 3 => format!("{} vertices: the complete graph K_{}", k + 1, k + 1),
            None if g == 4 => format!("{} vertices: the complete bipartite graph K_{k},{k}", 2 * k),
            None if matches!(g, 6 | 8 | 12) && prime_power(k - 1) => {
                // generalized polygons over GF(q) meet the Moore bound
                let q = k - 1;
                let half: usize = (0..g / 2).map(|i| q.pow(i as u32)).sum();
                let what = match g { 6 => "projective plane", 8 => "generalized quadrangle", _ => "generalized hexagon" };
                format!("{} vertices: the incidence graph of the {what} of order {q} (meets the Moore bound)", 2 * half)
            }
            None => {
                // Moore bound
                let moore = if g % 2 == 1 {
                    let d = (g - 1) / 2;
                    1 + k * (0..d).map(|i| (k - 1).pow(i as u32)).sum::<usize>()
                } else {
                    let d = g / 2;
                    2 * (0..d).map(|i| (k - 1).pow(i as u32)).sum::<usize>()
                };
                format!("the order is not known exactly; the Moore bound gives at least {moore} vertices")
            }
        };
        return Some(Answer::new("cage", vec![pod("Input", "Input interpretation", text(format!("({k},{g})-cage: smallest {k}-regular graph of girth {g}"))), primary(pod("Result", "Smallest such graph", text(res)))]));
    }
    // symbolic families
    if body == "n-complete graph" || body == "complete graph k_n" || body == "complete graph" {
        return Some(Answer::new("complete graph", vec![pod("Input", "Input interpretation", text("complete graph K_n")), primary(pod("Result", "Properties", table(None, vec![vec![text("vertices"), text("n")], vec![text("edges"), text("n(n − 1)/2")], vec![text("degree"), text("n − 1")], vec![text("chromatic number"), text("n")], vec![text("diameter"), text("1 (n ≥ 2)")], vec![text("girth"), text("3 (n ≥ 3)")], vec![text("Hamiltonian cycles"), text("(n − 1)!/2 (n ≥ 3)")], vec![text("spanning trees"), text("n^(n − 2) (Cayley's formula)")]])))]));
    }
    if re!(r"^\(n\s*,\s*k\)[- ]?turan graph").is_match(&body) {
        return Some(Answer::new("Turán graph", vec![pod("Input", "Input interpretation", text("Turán graph T(n, k)")), primary(pod("Result", "Edge count", text("(1 − 1/k) n²/2 when k divides n; in general (n² − r²)(k − 1)/(2k) + C(r, 2), with r = n mod k"))), pod("Def", "Definition", text("the complete k-partite graph with parts as equal as possible; the extremal graph with no K_(k+1) (Turán's theorem)"))]));
    }
    // several named graphs compared: "Petersen graph, icosahedral graph"
    let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\.?|\bversus\b|\band\b)\s*").split(&body).filter(|s| !s.is_empty()).collect();
    if parts.len() >= 2 && parts.len() <= 6 {
        let gs: Option<Vec<(Graph, String, Vec<(&'static str, String)>)>> = parts.iter().map(|p| named(p).filter(|g| g.0.n > 0 && g.0.n <= 60)).collect();
        if let Some(gs) = gs {
            let opt = |v: Option<usize>| v.map(|x| x.to_string()).unwrap_or_else(|| "∞".into());
            let degs = |g: &Graph| {
                let mut d = vec![0usize; g.n];
                for (a, b) in &g.edges {
                    d[*a] += 1;
                    d[*b] += 1;
                }
                let (lo, hi) = (*d.iter().min().unwrap_or(&0), *d.iter().max().unwrap_or(&0));
                if lo == hi { format!("{lo}-regular") } else { format!("{lo}–{hi}") }
            };
            let mut head = vec!["property".to_string()];
            head.extend(gs.iter().map(|g| g.1.clone()));
            let props: Vec<(&str, Box<dyn Fn(&Graph) -> String>)> = vec![
                ("vertices", Box::new(|g: &Graph| g.n.to_string())),
                ("edges", Box::new(|g: &Graph| g.edges.len().to_string())),
                ("degree", Box::new(degs)),
                ("diameter", Box::new(move |g: &Graph| opt(diameter(g)))),
                ("girth", Box::new(move |g: &Graph| opt(girth(g)))),
                ("chromatic number", Box::new(move |g: &Graph| opt(chromatic_number(g)))),
            ];
            let rows: Vec<Vec<Content>> = props.iter().map(|(k, f)| {
                let mut r = vec![text(*k)];
                r.extend(gs.iter().map(|g| text(f(&g.0))));
                r
            }).collect();
            let short = gs.iter().map(|g| format!("{}: {} vertices, {} edges", g.1, g.0.n, g.0.edges.len())).collect::<Vec<_>>().join(" | ");
            return Some(Answer::new("graph comparison", vec![
                pod("Input", "Input interpretation", text(gs.iter().map(|g| g.1.clone()).collect::<Vec<_>>().join(" | "))),
                primary(pod("Result", "Comparison", text(short))),
                pod("Table", "Properties", table(Some(head.iter().map(String::as_str).collect()), rows)),
            ]));
        }
    }
    if let Some((g, name, rows)) = named(&body) {
        if g.n == 0 {
            return Some(Answer::new("tree", vec![pod("Input", "Input interpretation", text(name)), primary(pod("Result", "Properties", table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())))]));
        }
        return Some(describe(&g, &name, rows, want));
    }
    if let Some((g, name)) = random_graph(&body) {
        return Some(describe(&g, &name, vec![], want));
    }
    let rb = re!(r"(?i)^(?:(?:the\s+)?(?:matching polynomial|eulerian (?:cycle|circuit|path)|hamiltonian (?:cycle|path))\s+(?:of|in|for)\s+(?:the\s+)?(?:graph\s+)?)|\s+(?:eulerian (?:cycle|circuit|path)|hamiltonian (?:cycle|path))$|^graph\s+").replace_all(raw, "").trim().to_string();
    if let Some(g) = edge_list(&rb) {
        let name = format!("{} graph with {} vertices and {} edges", if g.directed { "directed" } else { "undirected" }, g.n, g.edges.len());
        return Some(describe(&g, &name, vec![], want));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_invariants() {
        let p = petersen();
        assert_eq!((p.edges.len(), girth(&p), diameter(&p), chromatic_number(&p)), (15, Some(5), Some(2), Some(3)));
        assert_eq!(matching_counts(&p).unwrap(), vec![1, 15, 75, 145, 90, 6]);
        assert_eq!(wheel(12).edges.len(), 22);
        let ico = icosahedral();
        assert_eq!((ico.edges.len(), girth(&ico), diameter(&ico), chromatic_number(&ico)), (30, Some(3), Some(3), Some(4)));
        assert_eq!(octahedral().edges.len(), 12);
        assert_eq!(turan(10, 3).edges.len(), 33);
        let mut g = edge_list("1->2, 2->3, 3->1").unwrap();
        assert_eq!(eulerian(&g).map(|c| c.len()), Some(4));
        g.add(2, 1);
        assert_eq!(eulerian(&g), None);
    }
}
