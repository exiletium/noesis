//! Molecules as graphs: a SMILES parser (organic subset, bracket atoms,
//! branches, ring closures, aromatic atoms, charges) with implicit
//! hydrogens, and what follows from the graph — formula and mass,
//! hydrogen-bond donors and acceptors, rotatable bonds, rings, orbital
//! hybridization (VSEPR steric numbers), bond inventories with average bond
//! energies, longest chains, and topological indices (Wiener, Balaban J,
//! Randić, Hosoya Z, Harary, Schultz, Zagreb). Compounds are looked up by
//! name in the bundled Wikidata extract, or given as `SMILES ...`.

use super::chem::{by_symbol, find_db_compound, hill_formula, molar_mass};
use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

#[derive(Clone, Debug)]
pub struct Atom {
    pub el: String,
    pub aromatic: bool,
    pub charge: i32,
    /// hydrogens attached (explicit in brackets or implicit)
    pub h: u32,
}

/// Bond order: 1, 2, 3, or 4 for aromatic.
#[derive(Clone, Debug)]
pub struct Mol {
    pub atoms: Vec<Atom>,
    pub bonds: Vec<(usize, usize, u8)>,
}

fn default_valences(el: &str) -> &'static [u32] {
    match el {
        "B" => &[3],
        "C" => &[4],
        "N" => &[3, 5],
        "O" => &[2],
        "P" => &[3, 5],
        "S" => &[2, 4, 6],
        "F" | "Cl" | "Br" | "I" => &[1],
        _ => &[],
    }
}

/// Parse a SMILES string.
pub fn parse_smiles(s: &str) -> Option<Mol> {
    let chars: Vec<char> = s.trim().chars().collect();
    let mut atoms: Vec<Atom> = Vec::new();
    let mut bonds: Vec<(usize, usize, u8)> = Vec::new();
    let mut bracket: Vec<bool> = Vec::new();
    let mut stack: Vec<Option<usize>> = Vec::new();
    let mut prev: Option<usize> = None;
    let mut pending: Option<u8> = None;
    let mut rings: HashMap<u32, (usize, Option<u8>)> = HashMap::new();
    let mut i = 0;
    let order_of = |a: &Atom, b: &Atom, explicit: Option<u8>| -> u8 { explicit.unwrap_or(if a.aromatic && b.aromatic { 4 } else { 1 }) };
    while i < chars.len() {
        let c = chars[i];
        match c {
            '(' => {
                stack.push(prev);
                i += 1;
            }
            ')' => {
                prev = stack.pop()?;
                i += 1;
            }
            '-' | '/' | '\\' => {
                pending = Some(1);
                i += 1;
            }
            '=' => {
                pending = Some(2);
                i += 1;
            }
            '#' => {
                pending = Some(3);
                i += 1;
            }
            '$' => {
                pending = Some(3);
                i += 1;
            }
            ':' => {
                pending = Some(4);
                i += 1;
            }
            '.' => {
                prev = None;
                pending = None;
                i += 1;
            }
            '0'..='9' | '%' => {
                let num = if c == '%' {
                    let n: String = chars.get(i + 1..i + 3)?.iter().collect();
                    i += 3;
                    n.parse().ok()?
                } else {
                    i += 1;
                    c.to_digit(10)?
                };
                let cur = prev?;
                if let Some((other, ob)) = rings.remove(&num) {
                    let o = order_of(&atoms[other], &atoms[cur], pending.or(ob));
                    bonds.push((other, cur, o));
                } else {
                    rings.insert(num, (cur, pending));
                }
                pending = None;
            }
            '[' => {
                let end = chars[i..].iter().position(|&x| x == ']')? + i;
                let inner: String = chars[i + 1..end].iter().collect();
                let m = re!(r"^(\d*)([A-Z][a-z]?|[a-z][a-z]?|\*)(@(?:SP|TB|OH|AL|TH)\d+|@*)(?:H(\d*))?([+-]*)(\d*)(?::\d+)?$").captures(&inner)?;
                let mut el = m[2].to_string();
                let aromatic = el.chars().next()?.is_ascii_lowercase();
                if aromatic {
                    let mut cs = el.chars();
                    el = cs.next()?.to_ascii_uppercase().to_string() + cs.as_str();
                }
                let h = if inner.contains('H') && !m[2].contains('H') { m.get(4).map(|x| x.as_str().parse().unwrap_or(1)).unwrap_or(0) } else { 0 };
                let signs = &m[5];
                let mag: i32 = if m[6].is_empty() { signs.len() as i32 } else { m[6].parse().ok()? };
                let charge = if signs.starts_with('+') { mag } else if signs.starts_with('-') { -mag } else { 0 };
                atoms.push(Atom { el, aromatic, charge, h });
                bracket.push(true);
                let cur = atoms.len() - 1;
                if let Some(p) = prev {
                    let o = order_of(&atoms[p], &atoms[cur], pending);
                    bonds.push((p, cur, o));
                }
                pending = None;
                prev = Some(cur);
                i = end + 1;
            }
            _ => {
                let two: String = chars[i..chars.len().min(i + 2)].iter().collect();
                let (sym, len) = if two == "Cl" || two == "Br" { (two.clone(), 2) } else { (c.to_string(), 1) };
                let (el, aromatic) = match sym.as_str() {
                    "B" | "C" | "N" | "O" | "P" | "S" | "F" | "I" | "Cl" | "Br" => (sym.clone(), false),
                    "b" | "c" | "n" | "o" | "p" | "s" => (sym.to_uppercase(), true),
                    "*" => ("*".to_string(), false),
                    _ => return None,
                };
                atoms.push(Atom { el, aromatic, charge: 0, h: 0 });
                bracket.push(false);
                let cur = atoms.len() - 1;
                if let Some(p) = prev {
                    let o = order_of(&atoms[p], &atoms[cur], pending);
                    bonds.push((p, cur, o));
                }
                pending = None;
                prev = Some(cur);
                i += len;
            }
        }
    }
    if !rings.is_empty() || !stack.is_empty() || atoms.is_empty() {
        return None;
    }
    // implicit hydrogens on organic-subset atoms
    let mut used = vec![0u32; atoms.len()];
    let mut arom_bonds = vec![0u32; atoms.len()];
    for &(a, b, o) in &bonds {
        let v = if o == 4 { 1 } else { o as u32 };
        used[a] += v;
        used[b] += v;
        if o == 4 {
            arom_bonds[a] += 1;
            arom_bonds[b] += 1;
        }
    }
    for (k, a) in atoms.iter_mut().enumerate() {
        if bracket[k] {
            continue;
        }
        if a.aromatic && arom_bonds[k] > 0 {
            // OpenSMILES: lowest valence, less one for the π bond, never negative
            let v = default_valences(&a.el).first().copied().unwrap_or(0);
            a.h = v.saturating_sub(used[k] + 1);
            continue;
        }
        let need = used[k];
        if let Some(v) = default_valences(&a.el).iter().find(|&&v| v >= need) {
            a.h = v - need;
        }
    }
    // explicit terminal hydrogens become hydrogen counts of their neighbour
    let mut deg = vec![0usize; atoms.len()];
    for &(a, b, _) in &bonds {
        deg[a] += 1;
        deg[b] += 1;
    }
    let drop: Vec<bool> = atoms.iter().enumerate().map(|(i, a)| a.el == "H" && a.charge == 0 && deg[i] == 1).collect();
    if drop.iter().any(|&d| d) {
        for &(a, b, _) in &bonds {
            if drop[a] && !drop[b] {
                atoms[b].h += 1;
            } else if drop[b] && !drop[a] {
                atoms[a].h += 1;
            }
        }
        let mut map = vec![usize::MAX; atoms.len()];
        let mut kept = Vec::new();
        for (i, a) in atoms.into_iter().enumerate() {
            if !drop[i] {
                map[i] = kept.len();
                kept.push(a);
            }
        }
        let bonds = bonds.into_iter().filter(|&(a, b, _)| !drop[a] && !drop[b]).map(|(a, b, o)| (map[a], map[b], o)).collect();
        return Some(Mol { atoms: kept, bonds });
    }
    Some(Mol { atoms, bonds })
}

impl Mol {
    pub fn counts(&self) -> BTreeMap<String, u32> {
        let mut c = BTreeMap::new();
        for a in &self.atoms {
            *c.entry(a.el.clone()).or_insert(0) += 1;
            if a.h > 0 {
                *c.entry("H".to_string()).or_insert(0) += a.h;
            }
        }
        c
    }
    fn adj(&self) -> Vec<Vec<(usize, u8)>> {
        let mut g = vec![Vec::new(); self.atoms.len()];
        for &(a, b, o) in &self.bonds {
            g[a].push((b, o));
            g[b].push((a, o));
        }
        g
    }
    fn charge(&self) -> i32 {
        self.atoms.iter().map(|a| a.charge).sum()
    }
    /// All-pairs distances in the hydrogen-depleted graph (usize::MAX when disconnected).
    fn distances(&self) -> Vec<Vec<usize>> {
        let g = self.adj();
        let n = g.len();
        let mut d = vec![vec![usize::MAX; n]; n];
        for s in 0..n {
            d[s][s] = 0;
            let mut q = std::collections::VecDeque::from([s]);
            while let Some(u) = q.pop_front() {
                for &(v, _) in &g[u] {
                    if d[s][v] == usize::MAX {
                        d[s][v] = d[s][u] + 1;
                        q.push_back(v);
                    }
                }
            }
        }
        d
    }
    fn components(&self) -> usize {
        let d = self.distances();
        let n = self.atoms.len();
        let mut seen = vec![false; n];
        let mut c = 0;
        for i in 0..n {
            if !seen[i] {
                c += 1;
                for j in 0..n {
                    if d[i][j] != usize::MAX {
                        seen[j] = true;
                    }
                }
            }
        }
        c
    }
    fn rings(&self) -> usize {
        (self.bonds.len() + self.components()).saturating_sub(self.atoms.len())
    }
    /// Bonds that lie on a ring (not bridges).
    fn ring_bonds(&self) -> Vec<bool> {
        let g = self.adj();
        let n = g.len();
        let mut disc = vec![usize::MAX; n];
        let mut low = vec![0; n];
        let mut t = 0;
        let mut bridge: std::collections::HashSet<(usize, usize)> = Default::default();
        fn dfs(u: usize, p: usize, g: &[Vec<(usize, u8)>], disc: &mut [usize], low: &mut [usize], t: &mut usize, br: &mut std::collections::HashSet<(usize, usize)>) {
            disc[u] = *t;
            low[u] = *t;
            *t += 1;
            let mut skipped_parent = false;
            for &(v, _) in &g[u] {
                if v == p && !skipped_parent {
                    skipped_parent = true;
                    continue;
                }
                if disc[v] == usize::MAX {
                    dfs(v, u, g, disc, low, t, br);
                    low[u] = low[u].min(low[v]);
                    if low[v] > disc[u] {
                        br.insert((u.min(v), u.max(v)));
                    }
                } else {
                    low[u] = low[u].min(disc[v]);
                }
            }
        }
        for s in 0..n {
            if disc[s] == usize::MAX {
                dfs(s, usize::MAX, &g, &mut disc, &mut low, &mut t, &mut bridge);
            }
        }
        self.bonds.iter().map(|&(a, b, _)| !bridge.contains(&(a.min(b), a.max(b)))).collect()
    }
    pub fn donors(&self) -> usize {
        self.atoms.iter().filter(|a| (a.el == "N" || a.el == "O") && a.h > 0).count()
    }
    /// Acceptors in the PubChem (Cactvs) sense: O and F, and N with an
    /// available lone pair (not amide, pyrrole-type or cationic N).
    pub fn acceptors(&self) -> usize {
        let g = self.adj();
        let mut ring_atoms = vec![false; self.atoms.len()];
        for (&(x, y, _), r) in self.bonds.iter().zip(self.ring_bonds()) {
            if r {
                ring_atoms[x] = true;
                ring_atoms[y] = true;
            }
        }
        let carbonyl = |c: usize| self.atoms[c].el == "C" && g[c].iter().any(|&(o, ord)| ord == 2 && (self.atoms[o].el == "O" || self.atoms[o].el == "S"));
        (0..self.atoms.len())
            .filter(|&i| {
                let a = &self.atoms[i];
                match a.el.as_str() {
                    "O" | "F" => true,
                    "N" => {
                        if a.charge > 0 {
                            return false;
                        }
                        let deg = g[i].len() as u32 + a.h;
                        if a.aromatic && deg >= 3 {
                            return false;
                        }
                        let single_only = g[i].iter().all(|&(_, o)| o == 1);
                        // pyrrole-type ring N written in Kekulé form: its lone pair is in the π system
                        let ring_pi = ring_atoms[i] && g[i].iter().filter(|&&(v, _)| ring_atoms[v] && g[v].iter().any(|&(_, o)| o >= 2)).count() >= 2;
                        !(single_only && (ring_pi || g[i].iter().any(|&(c, _)| carbonyl(c))))
                    }
                    _ => false,
                }
            })
            .count()
    }
    pub fn rotatable(&self) -> usize {
        let g = self.adj();
        let ring = self.ring_bonds();
        self.bonds
            .iter()
            .zip(&ring)
            .filter(|((a, b, o), r)| {
                *o == 1 && !**r && g[*a].len() > 1 && g[*b].len() > 1 && !g[*a].iter().chain(&g[*b]).any(|&(_, oo)| oo == 3)
            })
            .count()
    }
    fn valence_electrons(el: &str) -> Option<i32> {
        let e = by_symbol(el)?;
        let g = e.group? as i32;
        Some(match g {
            1 | 2 => g,
            13..=18 => g - 10,
            _ => return None,
        })
    }
    /// Hybridization label from the VSEPR steric number.
    pub fn hybridization(&self, i: usize) -> Option<&'static str> {
        let a = &self.atoms[i];
        let g = self.adj();
        let v = Self::valence_electrons(&a.el)?;
        let sigma = g[i].len() as i32 + a.h as i32;
        let used2: i32 = g[i].iter().map(|&(_, o)| if o == 4 { 3 } else { 2 * o as i32 }).sum::<i32>() + 2 * a.h as i32;
        let lp = ((2 * v - used2 - 2 * a.charge) / 4).max(0);
        if a.el == "H" || sigma == 0 && lp == 0 {
            return None;
        }
        // an anionic O/S on an atom that also carries a double bond to O/S
        // (nitro, carboxylate, sulfonate) is delocalised: sp²
        let delocalised = a.charge < 0 && lp > 0 && g[i].iter().any(|&(c, o)| o == 1 && g[c].iter().any(|&(x, oo)| x != i && oo == 2 && matches!(self.atoms[x].el.as_str(), "O" | "S")));
        let lp = if delocalised { lp - 1 } else { lp };
        Some(match sigma + lp {
            1 => return None,
            2 => "sp",
            3 => "sp²",
            4 => "sp³",
            5 => "sp³d",
            6 => "sp³d²",
            _ => return None,
        })
    }
    fn longest_path(&self, only_carbon: bool) -> Option<usize> {
        let g = self.adj();
        let ok = |i: usize| !only_carbon || self.atoms[i].el == "C";
        let n = g.len();
        let mut best = 0usize;
        let mut budget = 3_000_000usize;
        fn go(u: usize, len: usize, g: &[Vec<(usize, u8)>], seen: &mut Vec<bool>, ok: &dyn Fn(usize) -> bool, best: &mut usize, budget: &mut usize) -> bool {
            if *budget == 0 {
                return false;
            }
            *budget -= 1;
            *best = (*best).max(len);
            for &(v, _) in &g[u] {
                if !seen[v] && ok(v) {
                    seen[v] = true;
                    if !go(v, len + 1, g, seen, ok, best, budget) {
                        return false;
                    }
                    seen[v] = false;
                }
            }
            true
        }
        for s in 0..n {
            if !ok(s) {
                continue;
            }
            let mut seen = vec![false; n];
            seen[s] = true;
            if !go(s, 1, &g, &mut seen, &ok, &mut best, &mut budget) {
                return None;
            }
        }
        Some(best)
    }
    fn hosoya(&self) -> Option<u128> {
        let n = self.atoms.len();
        if n > 64 {
            return None;
        }
        let g = self.adj();
        let nb: Vec<u64> = g.iter().map(|l| l.iter().fold(0u64, |m, &(v, _)| m | (1 << v))).collect();
        let mut memo: HashMap<u64, u128> = HashMap::new();
        fn z(mask: u64, nb: &[u64], memo: &mut HashMap<u64, u128>) -> Option<u128> {
            if mask == 0 {
                return Some(1);
            }
            if let Some(&v) = memo.get(&mask) {
                return Some(v);
            }
            if memo.len() > 3_000_000 {
                return None;
            }
            let v = mask.trailing_zeros() as usize;
            let rest = mask & !(1 << v);
            let mut total = z(rest, nb, memo)?;
            let mut m = nb[v] & rest;
            while m != 0 {
                let u = m.trailing_zeros() as usize;
                m &= m - 1;
                total = total.checked_add(z(rest & !(1 << u), nb, memo)?)?;
            }
            memo.insert(mask, total);
            Some(total)
        }
        let full = if n == 64 { u64::MAX } else { (1u64 << n) - 1 };
        z(full, &nb, &mut memo)
    }
    /// Topological indices of the hydrogen-depleted graph.
    pub fn indices(&self) -> Vec<(&'static str, String)> {
        let n = self.atoms.len();
        let d = self.distances();
        let g = self.adj();
        let deg: Vec<f64> = g.iter().map(|l| l.len() as f64).collect();
        let connected = self.components() == 1;
        let mut out = Vec::new();
        if connected {
            let wiener: usize = (0..n).flat_map(|i| (i + 1..n).map(move |j| (i, j))).map(|(i, j)| d[i][j]).sum();
            out.push(("Wiener index", wiener.to_string()));
            let m = self.bonds.len() as f64;
            let mu = self.rings() as f64;
            let s: Vec<f64> = (0..n).map(|i| d[i].iter().sum::<usize>() as f64).collect();
            if !self.bonds.is_empty() {
                let j = m / (mu + 1.0) * self.bonds.iter().map(|&(a, b, _)| 1.0 / (s[a] * s[b]).sqrt()).sum::<f64>();
                out.push(("Balaban index", fmt_float(j, 6)));
            }
            let harary: f64 = (0..n).flat_map(|i| (i + 1..n).map(move |j| (i, j))).map(|(i, j)| 1.0 / d[i][j] as f64).sum();
            out.push(("Harary index", fmt_float(harary, 6)));
            // Schultz molecular topological index: Σ_i [v (A + D)]_i
            let mut schultz = 0.0;
            for i in 0..n {
                for j in 0..n {
                    let a = if g[j].iter().any(|&(x, _)| x == i) { 1.0 } else { 0.0 };
                    schultz += deg[j] * (a + d[j][i] as f64);
                }
            }
            out.push(("Schultz index", fmt_float(schultz, 10)));
            let diam = d.iter().flatten().copied().max().unwrap_or(0);
            let rad = d.iter().map(|r| r.iter().copied().max().unwrap_or(0)).min().unwrap_or(0);
            out.push(("graph diameter", diam.to_string()));
            out.push(("graph radius", rad.to_string()));
        }
        if !self.bonds.is_empty() {
            let randic: f64 = self.bonds.iter().map(|&(a, b, _)| 1.0 / (deg[a] * deg[b]).sqrt()).sum();
            out.push(("Randić index", fmt_float(randic, 6)));
        }
        let m1: f64 = deg.iter().map(|x| x * x).sum();
        let m2: f64 = self.bonds.iter().map(|&(a, b, _)| deg[a] * deg[b]).sum();
        out.push(("first Zagreb index", fmt_float(m1, 10)));
        out.push(("second Zagreb index", fmt_float(m2, 10)));
        if let Some(z) = self.hosoya() {
            out.push(("Hosoya index", z.to_string()));
        }
        out
    }
    fn bond_label(&self, a: usize, b: usize, o: u8) -> String {
        let (x, y) = (&self.atoms[a].el, &self.atoms[b].el);
        let (x, y) = if element_rank(x) <= element_rank(y) { (x, y) } else { (y, x) };
        let sym = match o {
            2 => "=",
            3 => "≡",
            4 => ":",
            _ => "-",
        };
        format!("{x}{sym}{y}")
    }
    /// Bonds by type, hydrogens included.
    pub fn bond_inventory(&self) -> BTreeMap<String, usize> {
        let mut inv = BTreeMap::new();
        for &(a, b, o) in &self.bonds {
            *inv.entry(self.bond_label(a, b, o)).or_insert(0) += 1;
        }
        for a in &self.atoms {
            if a.h > 0 {
                let label = if element_rank(&a.el) <= element_rank("H") { format!("{}-H", a.el) } else { format!("H-{}", a.el) };
                *inv.entry(label).or_insert(0) += a.h as usize;
            }
        }
        inv
    }
}

fn element_rank(e: &str) -> (u32, String) {
    match e {
        "C" => (0, String::new()),
        "H" => (1, String::new()),
        _ => (2, e.to_string()),
    }
}

/// Average bond enthalpies in kJ/mol.
fn bond_energy(label: &str) -> Option<f64> {
    let norm = |s: &str| {
        let sep = s.find(['-', '=', '≡', ':'])?;
        let (a, rest) = s.split_at(sep);
        let op = rest.chars().next()?;
        let b = &rest[op.len_utf8()..];
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        Some(format!("{a}{op}{b}"))
    };
    let key = norm(label)?;
    const T: &[(&str, f64)] = &[
        ("H-H", 436.0), ("C-H", 413.0), ("C-C", 348.0), ("C=C", 614.0), ("C≡C", 839.0), ("C:C", 518.0),
        ("C-N", 293.0), ("C=N", 615.0), ("C≡N", 891.0), ("C:N", 508.0), ("C-O", 358.0), ("C=O", 745.0), ("C≡O", 1072.0), ("C:O", 500.0),
        ("C-F", 485.0), ("C-Cl", 328.0), ("Br-C", 276.0), ("C-I", 240.0), ("C-S", 259.0), ("C=S", 577.0), ("C:S", 420.0),
        ("C-P", 264.0), ("C-Si", 301.0), ("H-N", 391.0), ("N-N", 163.0), ("N=N", 418.0), ("N≡N", 941.0), ("N-O", 201.0), ("N=O", 607.0),
        ("H-O", 463.0), ("O-O", 146.0), ("O=O", 495.0), ("H-S", 339.0), ("S-S", 266.0), ("H-Si", 323.0), ("O-Si", 368.0),
        ("H-P", 322.0), ("Cl-P", 326.0), ("O-P", 335.0), ("O=P", 544.0), ("O-S", 265.0), ("O=S", 523.0), ("F-F", 155.0), ("Cl-Cl", 242.0),
        ("Br-Br", 193.0), ("I-I", 151.0), ("F-H", 567.0), ("Cl-H", 431.0), ("Br-H", 366.0), ("H-I", 299.0), ("F-N", 272.0), ("Cl-N", 200.0), ("F-O", 190.0),
        ("F-S", 327.0), ("Cl-S", 253.0), ("F-P", 490.0), ("F-Si", 565.0), ("Cl-Si", 381.0), ("F-B", 646.0), ("B-H", 389.0), ("B-O", 536.0), ("B-C", 356.0),
    ];
    T.iter().find(|(k, _)| norm(k).as_deref() == Some(&key)).map(|(_, v)| *v)
}

// ---------------------------------------------------------------------------
// queries
// ---------------------------------------------------------------------------

struct Compound {
    name: String,
    smiles: String,
    mol: Mol,
}

/// Common polyatomic ions, for structures and resonance.
const ION_SMILES: &[(&str, &str)] = &[
    ("nitrate", "[N+](=O)([O-])[O-]"),
    ("nitrite", "N(=O)[O-]"),
    ("ammonium", "[NH4+]"),
    ("hydronium|oxonium", "[OH3+]"),
    ("hydroxide", "[OH-]"),
    ("cyanide", "[C-]#N"),
    ("acetate|ethanoate", "CC(=O)[O-]"),
    ("formate|methanoate", "C(=O)[O-]"),
    ("carbonate", "C(=O)([O-])[O-]"),
    ("bicarbonate|hydrogen carbonate|hydrogencarbonate", "OC(=O)[O-]"),
    ("sulfate|sulphate", "S(=O)(=O)([O-])[O-]"),
    ("sulfite|sulphite", "S(=O)([O-])[O-]"),
    ("phosphate|orthophosphate", "P(=O)([O-])([O-])[O-]"),
    ("perchlorate", "Cl(=O)(=O)(=O)[O-]"),
    ("chlorate", "Cl(=O)(=O)[O-]"),
    ("thiocyanate", "[S-]C#N"),
    ("oxalate", "C(=O)([O-])C(=O)[O-]"),
];

fn resolve(t: &str) -> Option<Compound> {
    let t = t.trim().trim_matches(|c: char| c == '"' || c == '?' || c == '.').trim();
    if let Some(m) = re!(r"(?i)^smiles\s*:?\s*(\S+)$").captures(t) {
        let smi = m[1].to_string();
        return Some(Compound { name: smi.clone(), mol: parse_smiles(&smi)?, smiles: smi });
    }
    let t = re!(r"(?i)^(?:the |a |an )?(?:molecule |compound |chemical )?").replace(t, "").to_string();
    let t = re!(r"(?i) (?:molecule|compound|chemical|gas)$").replace(&t, "").to_string();
    let Some(c) = find_db_compound(&t).or_else(|| {
        // stereo descriptors: "L-2-aminobutyric acid", "cis cyclononene", "(R)-…"
        let bare = re!(r"(?i)^(?:dl-|l-|d-|\([rs+\-±]+\)-|cis[- ]|trans[- ]|\(?[ez]\)?-|α-|β-)").replace(&t, "").to_string();
        (bare != t).then(|| find_db_compound(&bare)).flatten()
    }) else {
        if let Some((_, smi)) = ION_SMILES.iter().find(|x| x.0.split('|').any(|n| n == t.to_lowercase().trim_end_matches(" ion"))) {
            return Some(Compound { name: t.to_lowercase(), mol: parse_smiles(smi)?, smiles: smi.to_string() });
        }
        let smi = benzene_smiles(&t)?;
        return Some(Compound { name: t.to_lowercase(), mol: parse_smiles(&smi)?, smiles: smi });
    };
    if c.smiles.is_empty() {
        return None;
    }
    Some(Compound { name: c.name.to_string(), smiles: c.smiles.to_string(), mol: parse_smiles(c.smiles)? })
}

fn resolve_many(t: &str) -> Option<Vec<Compound>> {
    if let Some(c) = resolve(t) {
        return Some(vec![c]);
    }
    let parts: Vec<&str> = re!(r"(?i)\s+(?:vs\.?|versus|and)\s+|\s*,\s+").split(t).collect();
    if parts.len() < 2 || parts.len() > 4 {
        return None;
    }
    parts.into_iter().map(resolve).collect()
}


/// Substituent prefixes that can sit on a benzene ring, as SMILES branches
/// written from the ring atom outwards.
const SUBST: &[(&str, &str)] = &[
    ("fluoro", "F"), ("chloro", "Cl"), ("bromo", "Br"), ("iodo", "I"),
    ("nitro", "[N+](=O)[O-]"), ("amino", "N"), ("hydroxy", "O"), ("methoxy", "OC"), ("ethoxy", "OCC"),
    ("methyl", "C"), ("ethyl", "CC"), ("propyl", "CCC"), ("isopropyl", "C(C)C"), ("tert-butyl", "C(C)(C)C"), ("butyl", "CCCC"),
    ("cyanomethyl", "CC#N"), ("cyano", "C#N"), ("carboxy", "C(=O)O"), ("formyl", "C=O"), ("acetyl", "C(C)=O"),
    ("trifluoromethyl", "C(F)(F)F"), ("mercapto", "S"), ("sulfanyl", "S"), ("methylthio", "SC"), ("methylsulfanyl", "SC"),
    ("phenyl", "c2ccccc2"), ("benzyl", "Cc2ccccc2"), ("vinyl", "C=C"), ("ethenyl", "C=C"), ("sulfo", "S(=O)(=O)O"),
    ("hydroxymethyl", "CO"), ("aminomethyl", "CN"), ("dimethylamino", "N(C)C"), ("acetamido", "NC(C)=O"), ("benzyloxy", "OCc2ccccc2"),
];

/// Parent names: the group on ring position 1 (empty for benzene itself).
const PARENTS: &[(&str, &str)] = &[
    ("benzene", ""), ("benzoic acid", "C(=O)O"), ("phenol", "O"), ("aniline", "N"), ("anisole", "OC"), ("toluene", "C"),
    ("benzaldehyde", "C=O"), ("benzonitrile", "C#N"), ("acetophenone", "C(C)=O"), ("benzamide", "C(N)=O"),
    ("benzenesulfonic acid", "S(=O)(=O)O"), ("benzyl alcohol", "CO"), ("styrene", "C=C"), ("phenetole", "OCC"),
    ("phenylacetic acid", "CC(=O)O"), ("benzylamine", "CN"), ("thiophenol", "S"), ("benzenethiol", "S"),
];

/// Systematic names of substituted benzenes, to SMILES:
/// "2,5-dimethoxy-3-nitrobenzoic acid", "3,4-dichloroaniline",
/// "1,4-bis(cyanomethyl)benzene", "2-(3-bromophenyl)acetic acid".
pub fn benzene_smiles(name: &str) -> Option<String> {
    let n = name.trim().to_lowercase();
    // "2-(3-bromophenyl)acetic acid" → substituents on a phenyl carrying the parent chain
    let (body, head) = if let Some(m) = re!(r"^2-\((.*)phenyl\) ?(acetic acid|ethanol|ethanamine|acetonitrile)$").captures(&n) {
        let tail = match &m[2] { "acetic acid" => "CC(=O)O", "ethanol" => "CCO", "ethanamine" => "CCN", _ => "CC#N" };
        (m[1].trim_end_matches('-').to_string(), tail.to_string())
    } else {
        let (pname, g) = PARENTS.iter().filter(|(p, _)| n.ends_with(p)).max_by_key(|(p, _)| p.len())?;
        (n[..n.len() - pname.len()].trim().trim_end_matches('-').to_string(), g.to_string())
    };
    let mut pos: [Option<String>; 6] = Default::default();
    if !head.is_empty() {
        pos[0] = Some(head);
    }
    let mut rest = body.as_str();
    let mut any = false;
    while !rest.is_empty() {
        let m = re!(r"^(\d(?:,\d)*)-(di|tri|tetra|penta|bis|tris)?\(?").captures(rest)?;
        let locs: Vec<usize> = m[1].split(',').map(|d| d.parse().unwrap()).collect();
        let mult = match m.get(2).map(|x| x.as_str()) { None => 1, Some("di" | "bis") => 2, Some("tri" | "tris") => 3, Some("tetra") => 4, _ => 5 };
        if locs.len() != mult {
            return None;
        }
        rest = &rest[m[0].len()..];
        let paren = m[0].ends_with('(');
        let (sname, frag) = SUBST.iter().filter(|(s, _)| rest.starts_with(s)).max_by_key(|(s, _)| s.len())?;
        rest = &rest[sname.len()..];
        if paren {
            rest = rest.strip_prefix(')')?;
        }
        rest = rest.trim_start_matches(['-', ' ']);
        for l in locs {
            if !(1..=6).contains(&l) || pos[l - 1].is_some() {
                return None;
            }
            pos[l - 1] = Some(frag.to_string());
        }
        any = true;
    }
    if !any {
        return None;
    }
    let mut s = String::new();
    for (i, g) in pos.iter().enumerate() {
        s.push('c');
        if i == 0 || i == 5 {
            s.push('1');
        }
        if let Some(g) = g {
            s.push_str(&format!("({g})"));
        }
    }
    Some(s)
}

fn empirical_text(counts: &BTreeMap<String, u32>) -> String {
    fn gcd(a: u32, b: u32) -> u32 { if b == 0 { a } else { gcd(b, a % b) } }
    let g = counts.values().fold(0, |a, &b| gcd(a, b)).max(1);
    hill_formula(&counts.iter().map(|(k, v)| (k.clone(), v / g)).collect())
}

/// "formula of caffeine", "X molecular formula vs Hill formula", "SMILES for X".
fn formulas(c: &str) -> Option<Answer> {
    let m = re!(r"(?i)^(?:what is |what's )?(?:the )?(?:(molecular|chemical|hill|structural|condensed) )?(formula|smiles(?: string| code)?) (?:of|for) (.+)$").captures(c);
    let (which, subject) = if let Some(m) = m {
        (format!("{} {}", m.get(1).map_or("", |x| x.as_str()), &m[2]).trim().to_lowercase(), m[3].to_string())
    } else {
        let m = re!(r"(?i)^(.+?),? (?:(molecular|chemical|hill) formula|formula|smiles(?: string| code)?)(?: vs\.? (?:the )?(?:empirical|hill|molecular|chemical) formula)?$").captures(c)?;
        let w = m.get(2).map_or_else(|| if m[0].to_lowercase().contains("smiles") && !m[0].to_lowercase().contains("formula") { "smiles".to_string() } else { "formula".to_string() }, |x| format!("{} formula", x.as_str()));
        (w.to_lowercase(), m[1].to_string())
    };
    let cs = resolve_many(&subject)?;
    let smiles = which.starts_with("smiles");
    let title = if smiles { "SMILES" } else if which.starts_with("hill") { "Hill formula" } else { "Molecular formula" };
    let rows = |c: &Compound| -> Vec<(&'static str, String)> {
        let counts = c.mol.counts();
        vec![
            ("molecular formula", formula_text(&c.mol)),
            ("Hill formula", hill_formula(&counts)),
            ("empirical formula", empirical_text(&counts)),
            ("molar mass", format!("{} g/mol", fmt_float(molar_mass(&counts), 6))),
            ("SMILES", c.smiles.clone()),
        ]
    };
    let main = if cs.len() == 1 { text(if smiles { cs[0].smiles.clone() } else { formula_text(&cs[0].mol) }) } else { multi_table(&cs, &|c| rows(c).into_iter().take(if smiles { 5 } else { 3 }).skip(if smiles { 4 } else { 0 }).collect()) };
    Some(Answer::new(title, vec![header(&cs), primary(pod("Result", title, main)), pod("Forms", "Formulas", multi_table(&cs, &rows))]))
}

fn formula_text(m: &Mol) -> String {
    let mut f = hill_formula(&m.counts());
    let q = m.charge();
    if q != 0 {
        let mag = if q.abs() == 1 { String::new() } else { q.abs().to_string() };
        f.push_str(&format!("{mag}{}", if q > 0 { "+" } else { "−" }));
    }
    f
}

fn header(cs: &[Compound]) -> Pod {
    let s = cs.iter().map(|c| format!("{} ({})", c.name, formula_text(&c.mol))).collect::<Vec<_>>().join(" | ");
    pod("Input", "Input interpretation", text(s))
}

/// One property for one or several compounds.
fn property_answer(cs: &[Compound], title: &str, f: &dyn Fn(&Compound) -> String) -> Answer {
    let main = if cs.len() == 1 {
        text(f(&cs[0]))
    } else {
        table(Some(vec!["compound", title]), cs.iter().map(|c| vec![text(c.name.clone()), text(f(c))]).collect())
    };
    Answer::new(title, vec![header(cs), primary(pod("Result", title, main))])
}

fn basic_rows(c: &Compound) -> Vec<(&'static str, String)> {
    let m = &c.mol;
    let counts = m.counts();
    vec![
        ("formula", formula_text(m)),
        ("molar mass", format!("{} g/mol", fmt_float(molar_mass(&counts), 6))),
        ("SMILES", c.smiles.clone()),
        ("heavy atoms", m.atoms.len().to_string()),
        ("hydrogen bond donors", m.donors().to_string()),
        ("hydrogen bond acceptors", m.acceptors().to_string()),
        ("rotatable bonds", m.rotatable().to_string()),
        ("rings", m.rings().to_string()),
        ("aromatic atoms", m.atoms.iter().filter(|a| a.aromatic).count().to_string()),
        ("formal charge", m.charge().to_string()),
    ]
}

fn multi_table(cs: &[Compound], rows: &dyn Fn(&Compound) -> Vec<(&'static str, String)>) -> Content {
    let per: Vec<Vec<(&'static str, String)>> = cs.iter().map(rows).collect();
    if cs.len() == 1 {
        return table(None, per[0].iter().map(|(a, b)| vec![text(*a), text(b.clone())]).collect());
    }
    let mut keys: Vec<&'static str> = Vec::new();
    for p in &per {
        for (k, _) in p {
            if !keys.contains(k) {
                keys.push(k);
            }
        }
    }
    let mut header = vec![""];
    header.extend(cs.iter().map(|c| c.name.as_str()));
    let rows = keys.iter().map(|k| {
        let mut r = vec![text(*k)];
        r.extend(per.iter().map(|p| text(p.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone()).unwrap_or_else(|| "—".into()))));
        r
    });
    table(Some(header), rows.collect())
}

fn hybrid_rows(c: &Compound) -> Vec<(&'static str, String)> {
    let mut by: BTreeMap<(String, &'static str), usize> = BTreeMap::new();
    for i in 0..c.mol.atoms.len() {
        if let Some(h) = c.mol.hybridization(i) {
            *by.entry((c.mol.atoms[i].el.clone(), h)).or_insert(0) += 1;
        }
    }
    let mut out: Vec<(&'static str, String)> = Vec::new();
    for h in ["sp", "sp²", "sp³", "sp³d", "sp³d²"] {
        let list: Vec<String> = by.iter().filter(|((_, hh), _)| *hh == h).map(|((e, _), n)| format!("{n} {e}")).collect();
        if !list.is_empty() {
            out.push((h, list.join(", ")));
        }
    }
    out
}

fn bond_rows(c: &Compound, energies: bool) -> Vec<Vec<Content>> {
    c.mol
        .bond_inventory()
        .into_iter()
        .map(|(k, n)| {
            let label = k.replace('-', "–").replace(':', "⋯ (aromatic) ");
            let mut r = vec![text(label), text(n.to_string())];
            if energies {
                r.push(text(bond_energy(&k).map(|e| format!("{} kJ/mol", fmt_float(e, 6))).unwrap_or_else(|| "—".into())));
            }
            r
        })
        .collect()
}

pub fn handle(q: &Query) -> Option<Answer> {
    let c = q.clean.trim().trim_end_matches(['?', '.']).trim();
    let l = c.to_lowercase();
    if let Some(a) = formulas(c) {
        return Some(a);
    }
    // hydrogen bond donors / acceptors
    if let Some(m) = re!(r"(?i)^(?:how many |number of )?(?:h-?bond|hydrogen[- ]bond) (donor|acceptor)s?(?: count)?(?: are)? (?:in|of|for) (.+)$").captures(c).or_else(|| re!(r"(?i)^(.+?),? (?:number of )?(?:h-?bond|hydrogen[- ]bond) (donor|acceptor)s?(?: count)?$").captures(c).map(|m| m)) {
        let (kind, subject) = if m[1].eq_ignore_ascii_case("donor") || m[1].eq_ignore_ascii_case("acceptor") { (m[1].to_lowercase(), m[2].to_string()) } else { (m[2].to_lowercase(), m[1].to_string()) };
        let cs = resolve_many(&subject)?;
        let donors = kind == "donor";
        let title = if donors { "Hydrogen bond donors" } else { "Hydrogen bond acceptors" };
        return Some(property_answer(&cs, title, &|c| if donors { c.mol.donors() } else { c.mol.acceptors() }.to_string()));
    }
    // topological indices and graph properties
    if let Some(m) = re!(r"(?i)^(?:(?:the )?(?:topological ind(?:ices|exes)|graph properties|molecular graph)(?: of| for)? (.+)|(.+?),? (?:topological ind(?:ices|exes)|graph properties|molecular graph))$").captures(c) {
        let subject = m.get(1).or(m.get(2))?.as_str();
        let cs = resolve_many(subject)?;
        let graph = l.contains("graph");
        let rows = |c: &Compound| -> Vec<(&'static str, String)> {
            let mut r = Vec::new();
            if graph {
                r.push(("vertices (heavy atoms)", c.mol.atoms.len().to_string()));
                r.push(("edges (bonds between heavy atoms)", c.mol.bonds.len().to_string()));
                r.push(("independent cycles", c.mol.rings().to_string()));
                let mut degs: Vec<usize> = c.mol.adj().iter().map(|l| l.len()).collect();
                degs.sort_unstable_by(|a, b| b.cmp(a));
                r.push(("degree sequence", degs.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ")));
            }
            r.extend(c.mol.indices());
            r
        };
        let title = if graph { "Graph properties" } else { "Topological indices" };
        return Some(Answer::new(title, vec![header(&cs), primary(pod("Result", title, multi_table(&cs, &rows))), pod("Note", "Note", text("computed on the hydrogen-depleted molecular graph"))]));
    }
    if let Some(m) = re!(r"(?i)^(?:(?:the )?(wiener|balaban|randi[cć]|hosoya|harary|schultz|(?:first |second )?zagreb)(?: index)?(?: of| for)? (.+)|(.+?),? (wiener|balaban|randi[cć]|hosoya|harary|schultz|(?:first |second )?zagreb) index)$").captures(c) {
        let (which, subject) = match m.get(1) {
            Some(w) => (w.as_str().to_lowercase(), m[2].to_string()),
            None => (m[4].to_lowercase(), m[3].to_string()),
        };
        let cs = resolve_many(&subject)?;
        let key = match which.as_str() {
            "randic" | "randić" => "Randić index".to_string(),
            "zagreb" | "first zagreb" => "first Zagreb index".to_string(),
            "second zagreb" => "second Zagreb index".to_string(),
            w => format!("{}{} index", w[..1].to_uppercase(), &w[1..]),
        };
        let title = key.clone();
        return Some(property_answer(&cs, &title, &|c| c.mol.indices().into_iter().find(|(k, _)| *k == key).map(|(_, v)| v).unwrap_or_else(|| "(not computed: graph too large)".into())));
    }
    // chains
    if let Some(m) = re!(r"(?i)^(?:(?:the )?(longest (?:overall |straight )?chain|longest carbon chain|carbon backbone)(?: in| of| for)? (.+)|(.+?),? (longest (?:overall |straight )?chain|longest carbon chain|carbon backbone))$").captures(c) {
        let (what, subject) = match m.get(1) {
            Some(w) => (w.as_str().to_lowercase(), m[2].to_string()),
            None => (m[4].to_lowercase(), m[3].to_string()),
        };
        let cs = resolve_many(&subject)?;
        let carbon = what.contains("carbon");
        let title = if carbon { "Longest carbon chain" } else { "Longest chain" };
        return Some(property_answer(&cs, title, &|c| c.mol.longest_path(carbon).map(|n| format!("{n} atoms")).unwrap_or_else(|| "(too many paths to search)".into())));
    }
    // hybridization
    if let Some(m) = re!(r"(?i)^(?:what is |what are )?(?:the )?(?:(?:orbital )?hybridi[sz]ations?(?: of| in| for)? (?:the )?(?:atoms (?:in|of) )?(.+)|(.+?),? (?:orbital )?hybridi[sz]ations?)$").captures(c) {
        let subject = m.get(1).or(m.get(2))?.as_str();
        let cs = resolve_many(subject)?;
        return Some(Answer::new("hybridization", vec![header(&cs), primary(pod("Result", "Orbital hybridization (atoms by type)", multi_table(&cs, &hybrid_rows))), pod("Method", "Method", text("steric number = σ bonds + lone pairs (2 → sp, 3 → sp², 4 → sp³); atoms in aromatic rings are sp²"))]));
    }
    // bonds
    if let Some(m) = re!(r"(?i)^(?:what are |what is )?(?:the )?(?:(bond energies|bond enthalpies|bond information|bonds|bond counts?)(?: in| of| for)? (.+)|(.+?),? (bond energies|bond enthalpies|bond information|bonds|bond counts?))$").captures(c) {
        let (what, subject) = match m.get(1) {
            Some(w) => (w.as_str().to_lowercase(), m[2].to_string()),
            None => (m[4].to_lowercase(), m[3].to_string()),
        };
        let cs = resolve_many(&subject)?;
        if cs.len() != 1 {
            return None;
        }
        let energies = what.contains("energ") || what.contains("enthalp") || what.contains("information");
        let rows = bond_rows(&cs[0], energies);
        let hdr = if energies { vec!["bond", "count", "average bond energy"] } else { vec!["bond", "count"] };
        let mut pods = vec![header(&cs), primary(pod("Result", "Bonds", table(Some(hdr), rows)))];
        if energies {
            let total: Option<f64> = cs[0].mol.bond_inventory().iter().map(|(k, n)| bond_energy(k).map(|e| e * *n as f64)).sum();
            if let Some(t) = total {
                pods.push(pod("Total", "Total of average bond energies (≈ atomization enthalpy)", text(format!("{} kJ/mol", fmt_float(t, 6)))));
            }
            pods.push(pod("Note", "Note", text("average bond enthalpies; values in a particular molecule differ by several percent")));
        }
        return Some(Answer::new("bonds", pods));
    }
    // "C4H8O2 possible structures", "isomers of C3H6O"
    if let Some(m) = re!(r"(?i)^(?:(?:the )?(?:possible |potential |known |random )?(?:structures|isomers|compounds) (?:of|for|with(?: the formula)?) ([A-Za-z0-9]+)|([A-Za-z0-9]+),? (?:possible |potential |known |random )?(?:structures|isomers))$").captures(c) {
        let f = m.get(1).or(m.get(2))?.as_str();
        if re!(r"^(?:[A-Z][a-z]?\d*){2,}$").is_match(f) {
            if let Some(a) = isomers(f) {
                return Some(a);
            }
        }
    }
    // InChI strings
    if c.starts_with("InChI=") {
        return inchi_answer(c);
    }
    // structure diagrams, Lewis and electron-dot structures
    if let Some(m) = re!(r"(?i)^(?:(?:show |draw |what is |what's )?(?:the |a )?(lewis (?:dot )?structure|electron[- ]dot (?:structure|diagram)|structure diagram|2d structure|skeletal (?:structure|formula)|structural (?:diagram|formula))s? (?:for|of) (.+)|(.+?),? (lewis (?:dot )?structure|electron[- ]dot (?:structure|diagram)|structure diagram|2d structure|skeletal (?:structure|formula)|structural (?:diagram|formula))s?)$").captures(c) {
        let (kind, subject) = match m.get(1) {
            Some(k) => (k.as_str().to_lowercase(), m[2].to_string()),
            None => (m.get(4)?.as_str().to_lowercase(), m[3].to_string()),
        };
        return structure_answer(&resolve(&subject)?, &kind);
    }
    // 3-D, space-filling and ball-and-stick models: "space filling structure of adenine and guanine", "caffeine 3D structure"
    const K: &str = r"(?:3-?d (?:structure|model|shape)|three[- ]dimensional (?:structure|model)|space[- ]filling (?:structure|model|diagram)|ball[- ]and[- ]stick (?:structure|model|diagram)|molecular (?:structure|model|geometry|shape)|chemical structure|structure|molecule model|stick model)s?";
    let models = || -> Option<Answer> {
        let m = Regex::new(&format!(r"(?i)^(?:(?:show |draw |what is |what's )?(?:me )?(?:the |a )?({K}) (?:for|of) (.+)|(.+?),? ({K}))$")).ok()?.captures(c)?;
        let (kind, subject) = match m.get(1) {
            Some(k) => (k.as_str().to_lowercase(), m[2].to_string()),
            None => (m.get(4)?.as_str().to_lowercase(), m[3].to_string()),
        };
        let parts: Vec<String> = re!(r"(?i)\s*(?:,\s*and\s+|,\s*|\s+and\s+|\s+vs\.?\s+)").split(subject.trim()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        let mut out: Option<Answer> = None;
        for p in &parts {
            let comp = resolve(p)?;
            let mut a = model_answer(&comp, &kind)?;
            match out.as_mut() {
                None => out = Some(a),
                Some(o) => {
                    for pd in a.pods.iter_mut() {
                        pd.primary = false;
                        pd.title = format!("{} — {}", comp.name, pd.title);
                        pd.id = format!("{}{}", pd.id, comp.name.replace(' ', ""));
                    }
                    o.interpretation = format!("{} | {}", o.interpretation, comp.name);
                    o.pods.extend(a.pods);
                }
            }
        }
        out
    };
    if let Some(a) = models() {
        return Some(a);
    }
    // resonance structures
    if let Some(m) = re!(r"(?i)^(?:what are )?(?:the )?(?:resonance (?:structures|diagrams|forms) (?:of|for) (.+)|(.+?),? resonance (?:structures|diagrams|forms))$").captures(c) {
        return resonance_answer(&resolve(m.get(1).or(m.get(2))?.as_str())?);
    }
    // aromatic rings
    if let Some(m) = re!(r"(?i)^(?:what is |what are )?(?:the )?(?:aromatic (?:structure|rings?|systems?) (?:of|in|for) (.+)|(.+?),? aromatic (?:structure|rings?|systems?))$").captures(c) {
        return aromatic_answer(&resolve(m.get(1).or(m.get(2))?.as_str())?);
    }
    // maximum common substructure
    if let Some(m) = re!(r"(?i)^(?:what is )?(?:the )?(?:(?:maximum |largest )?common substructure (?:of|for|between|in) (.+)|(.+?),? (?:mcs|(?:maximum |largest )?common substructure))$").captures(c) {
        return mcs_answer(&resolve_many(m.get(1).or(m.get(2))?.as_str())?);
    }
    // rotatable bonds, rings
    if let Some(m) = re!(r"(?i)^(?:how many |number of )?(rotatable bonds|rings)(?: are)? (?:in|of) (.+)$").captures(c) {
        let cs = resolve_many(&m[2])?;
        let rot = m[1].to_lowercase().starts_with("rot");
        return Some(property_answer(&cs, if rot { "Rotatable bonds" } else { "Rings" }, &|c| if rot { c.mol.rotatable() } else { c.mol.rings() }.to_string()));
    }
    // molecular properties / a bare SMILES
    let m = re!(r"(?i)^(?:(.+?),? (?:molecular|structural|cheminformatics) properties|(?:molecular|structural) properties (?:of|for) (.+)|(smiles\s*:?\s*\S+)(?: (?:properties|molecular properties))?)$").captures(c)?;
    let subject = m.get(1).or(m.get(2)).or(m.get(3))?.as_str();
    let cs = resolve_many(subject)?;
    Some(Answer::new("molecular properties", vec![header(&cs), primary(pod("Result", "Molecular properties", multi_table(&cs, &basic_rows))), pod("Hybridization", "Orbital hybridization", multi_table(&cs, &hybrid_rows))]))
}


/// Valence electrons of main-group elements.
fn valence_electrons(el: &str) -> Option<i32> {
    Some(match el {
        "H" | "Li" | "Na" | "K" => 1,
        "Be" | "Mg" | "Ca" => 2,
        "B" | "Al" | "Ga" => 3,
        "C" | "Si" | "Ge" | "Sn" => 4,
        "N" | "P" | "As" | "Sb" => 5,
        "O" | "S" | "Se" | "Te" => 6,
        "F" | "Cl" | "Br" | "I" => 7,
        _ => return None,
    })
}

fn atom_label(m: &Mol, i: usize) -> String {
    format!("{}{}", m.atoms[i].el, i + 1)
}

impl Mol {
    /// The Kekulé forms: every way of resolving aromatic bonds into single and
    /// double bonds (bond orders, in bond order), at most `cap` of them.
    pub fn kekule_forms(&self, cap: usize) -> Vec<Vec<u8>> {
        // rings drawn with explicit alternating bonds: ring bonds between atoms
        // that carry a ring double bond become aromatic-like, so every
        // placement of those double bonds is found
        let ring = self.ring_bonds();
        let mut pi = vec![false; self.atoms.len()];
        for (k, &(a, b, o)) in self.bonds.iter().enumerate() {
            if o == 2 && ring[k] {
                pi[a] = true;
                pi[b] = true;
            }
        }
        if pi.iter().any(|&p| p) {
            let mut m = self.clone();
            for (k, b) in m.bonds.iter_mut().enumerate() {
                if ring[k] && (b.2 == 1 || b.2 == 2) && pi[b.0] && pi[b.1] {
                    b.2 = 4;
                }
            }
            return m.kekule_forms_aromatic(cap);
        }
        self.kekule_forms_aromatic(cap)
    }

    fn kekule_forms_aromatic(&self, cap: usize) -> Vec<Vec<u8>> {
        let n = self.atoms.len();
        let mut arom_deg = vec![0u32; n];
        let mut fixed = vec![0u32; n];
        let mut inc: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (k, &(a, b, o)) in self.bonds.iter().enumerate() {
            for x in [a, b] {
                if o == 4 {
                    arom_deg[x] += 1;
                    inc[x].push(k);
                } else {
                    fixed[x] += o as u32;
                }
            }
        }
        let need: Vec<bool> = (0..n)
            .map(|i| {
                let a = &self.atoms[i];
                if arom_deg[i] == 0 {
                    return false;
                }
                let val: i32 = match (a.el.as_str(), a.charge) {
                    ("C", 0) => 4,
                    ("C", _) => 3,
                    ("N", 1) | ("P", 1) => 4,
                    ("N", -1) => 2,
                    ("N", _) | ("P", _) | ("B", _) => 3,
                    ("O", 1) | ("S", 1) | ("Se", 1) => 3,
                    ("O", _) | ("S", _) | ("Se", _) => 2,
                    _ => 0,
                };
                val - (arom_deg[i] + fixed[i] + a.h) as i32 >= 1
            })
            .collect();
        fn go(i0: usize, need: &[bool], matched: &mut [bool], inc: &[Vec<usize>], bonds: &[(usize, usize, u8)], cur: &mut Vec<u8>, out: &mut Vec<Vec<u8>>, cap: usize) {
            if out.len() >= cap {
                return;
            }
            let Some(i) = (i0..need.len()).find(|&i| need[i] && !matched[i]) else {
                out.push(cur.iter().map(|&o| if o == 4 { 1 } else { o }).collect());
                return;
            };
            matched[i] = true;
            for &k in &inc[i] {
                let (a, b, _) = bonds[k];
                let j = if a == i { b } else { a };
                if need[j] && !matched[j] {
                    matched[j] = true;
                    cur[k] = 2;
                    go(i + 1, need, matched, inc, bonds, cur, out, cap);
                    cur[k] = 4;
                    matched[j] = false;
                }
            }
            matched[i] = false;
        }
        let mut out = Vec::new();
        let mut cur: Vec<u8> = self.bonds.iter().map(|b| b.2).collect();
        let mut matched = vec![false; n];
        go(0, &need, &mut matched, &inc, &self.bonds, &mut cur, &mut out, cap);
        out
    }

    /// Resonance structures: the Kekulé forms combined with every placement of
    /// double bonds among equivalent terminal atoms on one centre (the three
    /// N–O bonds of nitrate, the two C–O bonds of a carboxylate). Each is
    /// (bond orders, atom charges).
    pub fn resonance_forms(&self, cap: usize) -> Vec<(Vec<u8>, Vec<i32>)> {
        let g = self.adj();
        let mut groups: Vec<(Vec<usize>, Vec<usize>, usize)> = Vec::new(); // (terminal atoms, their bonds, double bonds)
        for (x, nb) in g.iter().enumerate() {
            if nb.len() < 2 {
                continue;
            }
            for el in ["O", "S", "N"] {
                let mut ts = Vec::new();
                let mut bs = Vec::new();
                let mut doubles = 0;
                for &(t, o) in nb {
                    let a = &self.atoms[t];
                    if a.el != el || g[t].len() != 1 || a.h != 0 {
                        continue;
                    }
                    let k = self.bonds.iter().position(|&(p, q, _)| (p == x && q == t) || (p == t && q == x)).unwrap();
                    match (o, a.charge) {
                        (2, 0) => doubles += 1,
                        (1, -1) => {}
                        _ => continue,
                    }
                    ts.push(t);
                    bs.push(k);
                }
                if doubles >= 1 && ts.len() > doubles {
                    groups.push((ts, bs, doubles));
                }
            }
        }
        let mut out = Vec::new();
        for form in self.kekule_forms(cap) {
            let mut acc: Vec<(Vec<u8>, Vec<i32>)> = vec![(form.clone(), self.atoms.iter().map(|a| a.charge).collect())];
            for (ts, bs, d) in &groups {
                let mut next = Vec::new();
                for (orders, charges) in &acc {
                    // every subset of size d of the terminal atoms gets the double bonds
                    let k = ts.len();
                    for mask in 0u32..(1 << k) {
                        if mask.count_ones() as usize != *d {
                            continue;
                        }
                        let mut o = orders.clone();
                        let mut c = charges.clone();
                        for idx in 0..k {
                            let dbl = mask & (1 << idx) != 0;
                            o[bs[idx]] = if dbl { 2 } else { 1 };
                            c[ts[idx]] = if dbl { 0 } else { -1 };
                        }
                        next.push((o, c));
                        if next.len() >= cap {
                            break;
                        }
                    }
                }
                acc = next;
            }
            out.extend(acc);
            if out.len() >= cap {
                out.truncate(cap);
                break;
            }
        }
        out
    }
}

/// "C1=C2, C3=C4 | negative: O3"
fn form_text(m: &Mol, orders: &[u8], charges: &[i32]) -> String {
    let mut parts: Vec<String> = m
        .bonds
        .iter()
        .zip(orders)
        .filter(|(_, &o)| o >= 2)
        .map(|(&(a, b, _), &o)| format!("{}{}{}", atom_label(m, a), if o == 3 { "≡" } else { "=" }, atom_label(m, b)))
        .collect();
    if parts.is_empty() {
        parts.push("no double bonds".into());
    }
    let mut s = parts.join(", ");
    let charged: Vec<String> = charges.iter().enumerate().filter(|(_, &c)| c != 0).map(|(i, &c)| format!("{}{}", atom_label(m, i), if c > 0 { "⁺".repeat(c as usize) } else { "⁻".repeat((-c) as usize) })).collect();
    if !charged.is_empty() {
        s.push_str(&format!("; charges {}", charged.join(" ")));
    }
    s
}

/// Lewis structure rows from one resonance form: (atom, bonded electrons pairs, lone pairs, formal charge).
fn lewis_rows(m: &Mol, orders: &[u8], charges: &[i32]) -> Option<(Vec<Vec<Content>>, i32, i32, i32)> {
    let n = m.atoms.len();
    let mut bond_sum = vec![0i32; n];
    for (&(a, b, _), &o) in m.bonds.iter().zip(orders) {
        bond_sum[a] += o as i32;
        bond_sum[b] += o as i32;
    }
    let mut rows = Vec::new();
    let (mut valence, mut lone_total, mut bonding) = (0, 0, 0);
    for i in 0..n {
        let a = &m.atoms[i];
        let v = valence_electrons(&a.el)?;
        let bonds = bond_sum[i] + a.h as i32;
        let nonbonding = v - charges[i] - bonds;
        if nonbonding < 0 {
            return None;
        }
        valence += v;
        lone_total += nonbonding / 2;
        rows.push(vec![text(atom_label(m, i)), text(bonds.to_string()), text(format!("{}{}", nonbonding / 2, if nonbonding % 2 == 1 { " + 1 unpaired" } else { "" })), text(format!("{:+}", charges[i]).replace("+0", "0"))]);
    }
    let h: i32 = m.atoms.iter().map(|a| a.h as i32).sum();
    valence += h;
    bonding += orders.iter().map(|&o| o as i32).sum::<i32>() + h;
    valence -= charges.iter().sum::<i32>();
    Some((rows, valence, bonding, lone_total))
}

fn structure_answer(c: &Compound, kind: &str) -> Option<Answer> {
    let m = &c.mol;
    let forms = m.resonance_forms(1);
    let (orders, charges) = forms.first().cloned().unwrap_or_else(|| (m.bonds.iter().map(|b| b.2.min(3)).collect(), m.atoms.iter().map(|a| a.charge).collect()));
    let g = m.adj();
    let conn: Vec<Vec<Content>> = (0..m.atoms.len())
        .map(|i| {
            let nb: Vec<String> = g[i]
                .iter()
                .map(|&(j, _)| {
                    let k = m.bonds.iter().position(|&(p, q, _)| (p == i && q == j) || (p == j && q == i)).unwrap();
                    format!("{}{}", match orders[k] { 2 => "=", 3 => "≡", _ => "–" }, atom_label(m, j))
                })
                .collect();
            vec![text(atom_label(m, i)), text(nb.join(" ")), text(m.atoms[i].h.to_string())]
        })
        .collect();
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{} | {kind}", c.name))),
        pod("Formula", "Formula", text(formula_text(m))),
        pod("SMILES", "SMILES", text(c.smiles.clone())),
    ];
    if let Some((rows, valence, bonding, lone)) = lewis_rows(m, &orders, &charges) {
        pods.insert(1, primary(pod("Result", if kind.contains("lewis") || kind.contains("dot") { "Lewis structure" } else { "Structure" }, text(format!("{valence} valence electrons: {bonding} bonding pairs and {lone} lone pairs ({})", form_text(m, &orders, &charges))))));
        pods.push(pod("Lewis", "Atoms (bonds count shared pairs including hydrogens)", table(Some(vec!["atom", "bonds", "lone pairs", "formal charge"]), rows)));
    } else {
        pods.insert(1, primary(pod("Result", "Structure", text(form_text(m, &orders, &charges)))));
    }
    pods.push(pod("Connectivity", "Connection table", table(Some(vec!["atom", "bonded to", "H"]), conn)));
    Some(Answer::new(format!("{} {kind}", c.name), pods))
}

/// A structure answer for a 3-D, space-filling or ball-and-stick request: the
/// connectivity plus the atom radii such a model is drawn with.
fn model_answer(c: &Compound, kind: &str) -> Option<Answer> {
    let mut a = structure_answer(c, kind)?;
    let m = &c.mol;
    let mut els: Vec<String> = m.atoms.iter().map(|x| x.el.to_string()).collect();
    if m.atoms.iter().any(|x| x.h > 0) {
        els.push("H".into());
    }
    els.sort();
    els.dedup();
    // Bondi van der Waals and typical covalent radii (pm)
    let radii = |e: &str| -> Option<(u32, u32)> {
        Some(match e {
            "H" => (120, 31), "C" => (170, 76), "N" => (155, 71), "O" => (152, 66), "F" => (147, 57), "P" => (180, 107), "S" => (180, 105),
            "Cl" => (175, 102), "Br" => (185, 120), "I" => (198, 139), "Si" => (210, 111), "B" => (192, 84), "Se" => (190, 120), "Na" => (227, 166), "K" => (275, 203),
            _ => return None,
        })
    };
    let rows: Vec<Vec<Content>> = els.iter().filter_map(|e| { let (v, cv) = radii(e)?; Some(vec![text(e.clone()), text(format!("{v} pm")), text(format!("{cv} pm"))]) }).collect();
    let heavy = m.atoms.len();
    let h: usize = m.atoms.iter().map(|x| x.h as usize).sum();
    a.pods.push(pod("Model", "Model", text(format!("{} atoms ({heavy} heavy atoms and {h} hydrogens) and {} bonds; {}", heavy + h, m.bonds.len() + h, if kind.contains("space") {
        "a space-filling (CPK) model draws each atom as a sphere of its van der Waals radius"
    } else if kind.contains("ball") || kind.contains("stick") {
        "a ball-and-stick model draws atoms as small spheres joined by bonds about the covalent radii apart"
    } else {
        "3-D atom coordinates are not generated offline; the connection table gives the full bonding"
    }))));
    if !rows.is_empty() {
        a.pods.push(pod("Radii", "Atom radii", table(Some(vec!["element", "van der Waals radius", "covalent radius"]), rows)));
    }
    Some(a)
}

fn resonance_answer(c: &Compound) -> Option<Answer> {
    let m = &c.mol;
    let forms = m.resonance_forms(64);
    if forms.is_empty() {
        return None;
    }
    let rows: Vec<Vec<Content>> = forms.iter().enumerate().map(|(i, (o, ch))| vec![text((i + 1).to_string()), text(form_text(m, o, ch))]).collect();
    Some(Answer::new(format!("{} resonance structures", c.name), vec![
        pod("Input", "Input interpretation", text(format!("{} | resonance structures", c.name))),
        primary(pod("Result", "Resonance structures", text(format!("{} structure{} (Kekulé forms and equivalent double-bond placements, without extra charge separation)", forms.len(), if forms.len() == 1 { "" } else { "s" })))),
        pod("Forms", "Double bonds in each structure", table(Some(vec!["#", "double bonds"]), rows)),
        pod("SMILES", "SMILES", text(c.smiles.clone())),
    ]))
}

/// Largest common connected substructure (heavy atoms, same elements and
/// bond orders, induced), by branch-and-bound search with a time limit.
fn common_substructure(a: &Mol, b: &Mol) -> (Vec<(usize, usize)>, bool) {
    struct S<'a> {
        ga: Vec<Vec<(usize, u8)>>,
        gb: Vec<Vec<(usize, u8)>>,
        a: &'a Mol,
        b: &'a Mol,
        ma: Vec<Option<usize>>,
        mb: Vec<Option<usize>>,
        excl: Vec<bool>,
        cur: Vec<(usize, usize)>,
        best: Vec<(usize, usize)>,
        deadline: std::time::Instant,
        timed_out: bool,
    }
    fn order(g: &[Vec<(usize, u8)>], x: usize, y: usize) -> Option<u8> {
        g[x].iter().find(|e| e.0 == y).map(|e| e.1)
    }
    fn ok(s: &S, x: usize, y: usize) -> bool {
        if s.a.atoms[x].el != s.b.atoms[y].el {
            return false;
        }
        let mut touches = false;
        for &(x2, o) in &s.ga[x] {
            if let Some(y2) = s.ma[x2] {
                touches = true;
                let _ = o;
                if order(&s.gb, y, y2).is_none() {
                    return false;
                }
            }
        }
        for &(y2, _) in &s.gb[y] {
            if let Some(x2) = s.mb[y2] {
                if order(&s.ga, x, x2).is_none() {
                    return false;
                }
            }
        }
        touches || s.cur.is_empty()
    }
    fn go(s: &mut S) {
        if std::time::Instant::now() > s.deadline {
            s.timed_out = true;
            return;
        }
        if s.cur.len() > s.best.len() {
            s.best = s.cur.clone();
        }
        let free_a = (0..s.ga.len()).filter(|&x| s.ma[x].is_none() && !s.excl[x]).count();
        let free_b = (0..s.gb.len()).filter(|&y| s.mb[y].is_none()).count();
        if s.cur.len() + free_a.min(free_b) <= s.best.len() {
            return;
        }
        let Some(x) = (0..s.ga.len()).find(|&x| s.ma[x].is_none() && !s.excl[x] && s.ga[x].iter().any(|&(x2, _)| s.ma[x2].is_some())) else {
            return;
        };
        for y in 0..s.gb.len() {
            if s.mb[y].is_none() && ok(s, x, y) {
                s.ma[x] = Some(y);
                s.mb[y] = Some(x);
                s.cur.push((x, y));
                go(s);
                s.cur.pop();
                s.ma[x] = None;
                s.mb[y] = None;
                if s.timed_out {
                    return;
                }
            }
        }
        s.excl[x] = true;
        go(s);
        s.excl[x] = false;
    }
    let mut s = S {
        ga: a.adj(),
        gb: b.adj(),
        a,
        b,
        ma: vec![None; a.atoms.len()],
        mb: vec![None; b.atoms.len()],
        excl: vec![false; a.atoms.len()],
        cur: Vec::new(),
        best: Vec::new(),
        deadline: std::time::Instant::now() + std::time::Duration::from_millis(1500),
        timed_out: false,
    };
    // seeds: every atom of a, rarest elements first
    let mut seeds: Vec<usize> = (0..a.atoms.len()).collect();
    seeds.sort_by_key(|&x| (a.atoms.iter().filter(|t| t.el == a.atoms[x].el).count(), std::cmp::Reverse(s.ga[x].len())));
    for x in seeds {
        for y in 0..b.atoms.len() {
            if a.atoms[x].el != b.atoms[y].el {
                continue;
            }
            s.ma[x] = Some(y);
            s.mb[y] = Some(x);
            s.cur.push((x, y));
            go(&mut s);
            s.cur.clear();
            s.ma[x] = None;
            s.mb[y] = None;
            if s.timed_out {
                break;
            }
        }
        if s.timed_out {
            break;
        }
        // any substructure containing x has been seen
        s.excl[x] = true;
    }
    let timed_out = s.timed_out;
    (s.best, timed_out)
}

fn mcs_answer(cs: &[Compound]) -> Option<Answer> {
    if cs.len() != 2 {
        return None;
    }
    let (a, b) = (&cs[0].mol, &cs[1].mol);
    let (map, timed_out) = common_substructure(a, b);
    if map.is_empty() {
        return None;
    }
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for &(x, _) in &map {
        *counts.entry(a.atoms[x].el.clone()).or_insert(0) += 1;
    }
    let inside: std::collections::HashSet<usize> = map.iter().map(|p| p.0).collect();
    let nbonds = a.bonds.iter().filter(|&&(p, q, _)| inside.contains(&p) && inside.contains(&q)).count();
    let skeleton = hill_formula(&counts);
    let atoms_a: Vec<String> = map.iter().map(|&(x, _)| atom_label(a, x)).collect();
    let atoms_b: Vec<String> = map.iter().map(|&(_, y)| atom_label(b, y)).collect();
    Some(Answer::new("maximum common substructure", vec![
        pod("Input", "Input interpretation", text(format!("{} | {} | maximum common substructure", cs[0].name, cs[1].name))),
        primary(pod("Result", "Maximum common substructure", text(format!("{} heavy atoms and {nbonds} bonds ({skeleton} skeleton; elements and connectivity matched, bond orders ignored){}", map.len(), if timed_out { "; largest found within the search time limit" } else { "" })))),
        pod("Share", "Fraction of each molecule", text(format!("{}: {}/{} heavy atoms | {}: {}/{} heavy atoms", cs[0].name, map.len(), a.atoms.len(), cs[1].name, map.len(), b.atoms.len()))),
        pod("Atoms", "Matched atoms", table(Some(vec![cs[0].name.as_str(), cs[1].name.as_str()]), atoms_a.into_iter().zip(atoms_b).map(|(x, y)| vec![text(x), text(y)]).collect())),
    ]))
}

/// A molecule from a standard InChI string: the formula layer numbers the heavy
/// atoms, the connection (/c) and hydrogen (/h) layers give the skeleton, and
/// double and triple bonds are placed to satisfy the usual valences.
pub fn from_inchi(s: &str) -> Option<Mol> {
    let body = s.trim().strip_prefix("InChI=")?;
    let mut layers = body.split('/');
    let ver = layers.next()?;
    if !ver.starts_with('1') {
        return None;
    }
    let formula = layers.next()?;
    if formula.contains('.') || formula.is_empty() {
        return None;
    }
    let mut els: Vec<String> = Vec::new();
    for m in re!(r"([A-Z][a-z]?)(\d*)").captures_iter(formula) {
        if &m[1] == "H" {
            continue;
        }
        let k: usize = if m[2].is_empty() { 1 } else { m[2].parse().ok()? };
        for _ in 0..k {
            els.push(m[1].to_string());
        }
    }
    let n = els.len();
    let mut bonds: Vec<(usize, usize, u8)> = Vec::new();
    let mut hs = vec![0u32; n];
    for layer in layers {
        if let Some(c) = layer.strip_prefix('c') {
            let mut stack: Vec<Option<usize>> = Vec::new();
            let mut prev: Option<usize> = None;
            let mut num = String::new();
            let flush = |num: &mut String, prev: &mut Option<usize>, bonds: &mut Vec<(usize, usize, u8)>| -> Option<()> {
                if num.is_empty() {
                    return Some(());
                }
                let x: usize = num.parse::<usize>().ok()?.checked_sub(1)?;
                if x >= n {
                    return None;
                }
                if let Some(p) = *prev {
                    if !bonds.iter().any(|&(a, b, _)| (a == p && b == x) || (a == x && b == p)) {
                        bonds.push((p, x, 1));
                    }
                }
                *prev = Some(x);
                num.clear();
                Some(())
            };
            for ch in c.chars() {
                if ch.is_ascii_digit() {
                    num.push(ch);
                    continue;
                }
                flush(&mut num, &mut prev, &mut bonds)?;
                match ch {
                    '-' => {}
                    '(' => stack.push(prev),
                    ',' => prev = *stack.last()?,
                    ')' => prev = stack.pop()?,
                    _ => return None,
                }
            }
            flush(&mut num, &mut prev, &mut bonds)?;
        } else if let Some(h) = layer.strip_prefix('h') {
            if h.contains('(') {
                return None;
            }
            let mut pending: Vec<usize> = Vec::new();
            let toks: Vec<String> = re!(r"\d+|[-,H]").find_iter(h).map(|m| m.as_str().to_string()).collect();
            let mut i = 0;
            while i < toks.len() {
                match toks[i].as_str() {
                    "," => {}
                    "-" => {
                        let lo = *pending.last()?;
                        let hi: usize = toks.get(i + 1)?.parse().ok()?;
                        for x in lo + 1..=hi {
                            pending.push(x);
                        }
                        i += 1;
                    }
                    "H" => {
                        let k: u32 = match toks.get(i + 1).and_then(|t| t.parse().ok()) {
                            Some(k) => {
                                i += 1;
                                k
                            }
                            None => 1,
                        };
                        for &x in &pending {
                            *hs.get_mut(x.checked_sub(1)?)? += k;
                        }
                        pending.clear();
                    }
                    t => pending.push(t.parse().ok()?),
                }
                i += 1;
            }
        } else if layer.starts_with('q') || layer.starts_with('p') {
            return None;
        }
    }
    // place multiple bonds where atoms fall short of their valence
    let mut deficit: Vec<i32> = (0..n)
        .map(|i| {
            let used = bonds.iter().filter(|b| b.0 == i || b.1 == i).count() as i32 + hs[i] as i32;
            let val = default_valences(&els[i]).iter().map(|&v| v as i32).find(|&v| v >= used).unwrap_or(used);
            val - used
        })
        .collect();
    fn solve(deficit: &mut [i32], bonds: &mut [(usize, usize, u8)]) -> bool {
        let Some(i) = deficit.iter().position(|&d| d > 0) else {
            return true;
        };
        for k in 0..bonds.len() {
            let (a, b, o) = bonds[k];
            if (a != i && b != i) || o >= 3 {
                continue;
            }
            let j = if a == i { b } else { a };
            if deficit[j] > 0 {
                bonds[k].2 += 1;
                deficit[i] -= 1;
                deficit[j] -= 1;
                if solve(deficit, bonds) {
                    return true;
                }
                bonds[k].2 -= 1;
                deficit[i] += 1;
                deficit[j] += 1;
            }
        }
        false
    }
    if !solve(&mut deficit, &mut bonds) {
        return None;
    }
    Some(Mol { atoms: els.into_iter().zip(hs).map(|(el, h)| Atom { el, aromatic: false, charge: 0, h }).collect(), bonds })
}

/// Graph invariant that ignores bond orders: (element, hydrogens, degree, distance sum) per atom.
fn skeleton_invariant(m: &Mol) -> Vec<(String, u32, usize, usize)> {
    let g = m.adj();
    let d = m.distances();
    let mut v: Vec<_> = (0..m.atoms.len()).map(|i| (m.atoms[i].el.clone(), m.atoms[i].h, g[i].len(), d[i].iter().filter(|&&x| x != usize::MAX).sum())).collect();
    v.sort();
    v
}

fn inchi_answer(s: &str) -> Option<Answer> {
    let m = from_inchi(s)?;
    let counts = m.counts();
    let inv = skeleton_invariant(&m);
    let name = super::chem::compound_db().0.iter().find(|c| !c.smiles.is_empty() && super::chem::parse_formula(c.formula).as_ref() == Some(&counts) && parse_smiles(c.smiles).is_some_and(|x| skeleton_invariant(&x) == inv)).map(|c| (c.name.to_string(), c.smiles.to_string()));
    let mut pods = vec![pod("Input", "Input interpretation", text(s.to_string()))];
    match &name {
        Some((nm, _)) => pods.push(primary(pod("Result", "Compound", text(nm.clone())))),
        None => pods.push(primary(pod("Result", "Formula", text(formula_text(&m))))),
    }
    let c = Compound { name: name.as_ref().map_or_else(|| "InChI".to_string(), |x| x.0.clone()), smiles: name.as_ref().map_or_else(String::new, |x| x.1.clone()), mol: m };
    let orders: Vec<u8> = c.mol.bonds.iter().map(|b| b.2).collect();
    let charges: Vec<i32> = c.mol.atoms.iter().map(|a| a.charge).collect();
    pods.push(pod("Structure", "Bonds from the InChI layers", text(form_text(&c.mol, &orders, &charges))));
    pods.push(pod("Properties", "Properties", multi_table(std::slice::from_ref(&c), &basic_rows)));
    Some(Answer::new("InChI", pods))
}

fn isomers(f: &str) -> Option<Answer> {
    let counts = super::chem::parse_formula(f)?;
    let mut seen = std::collections::HashSet::new();
    let hits: Vec<&super::chem::DbCompound> = super::chem::compound_db().0.iter().filter(|c| super::chem::parse_formula(c.formula).as_ref() == Some(&counts) && !re!(r"(?i)-d\d*$|\bd\d+-|deuter|-13c|\(13c|-15n").is_match(c.name) && seen.insert(c.name.to_lowercase())).collect();
    if hits.is_empty() {
        return None;
    }
    let hill = hill_formula(&counts);
    let mut shown: Vec<&&super::chem::DbCompound> = hits.iter().collect();
    shown.sort_by_key(|c| (c.smiles.is_empty(), c.name.len()));
    let shown: Vec<_> = shown.into_iter().take(30).collect();
    Some(Answer::new(format!("compounds with formula {hill}"), vec![
        pod("Input", "Input interpretation", text(format!("{hill} | possible structures"))),
        primary(pod("Result", "Known compounds", text(format!("{} known compound{} with formula {hill}: {}", hits.len(), if hits.len() == 1 { "" } else { "s" }, shown.iter().take(8).map(|c| c.name).collect::<Vec<_>>().join(", "))))),
        pod("Table", "Structures", table(Some(vec!["compound", "SMILES"]), shown.iter().map(|c| vec![text(c.name), text(if c.smiles.is_empty() { "—" } else { c.smiles })]).collect())),
        pod("Note", "Note", text("isomers present in Noesis's compound table (from Wikidata), not an exhaustive enumeration")),
    ]))
}

fn aromatic_answer(c: &Compound) -> Option<Answer> {
    let m = &c.mol;
    let ar: Vec<&(usize, usize, u8)> = m.bonds.iter().filter(|b| b.2 == 4).collect();
    let atoms: std::collections::BTreeSet<usize> = ar.iter().flat_map(|b| [b.0, b.1]).collect();
    // ring count of the aromatic subgraph: edges - vertices + components
    let idx: Vec<usize> = atoms.iter().copied().collect();
    let sub = Mol { atoms: idx.iter().map(|&i| m.atoms[i].clone()).collect(), bonds: ar.iter().map(|b| (idx.binary_search(&b.0).unwrap(), idx.binary_search(&b.1).unwrap(), 4)).collect() };
    let rings = if idx.is_empty() { 0 } else { sub.rings() };
    let systems = if idx.is_empty() { 0 } else { sub.components() };
    let kek = m.kekule_forms(1000).len();
    Some(Answer::new(format!("{} aromatic structure", c.name), vec![
        pod("Input", "Input interpretation", text(format!("{} | aromatic structure", c.name))),
        primary(pod("Result", "Aromatic structure", text(format!("{rings} aromatic ring{} in {systems} aromatic system{} ({} aromatic atoms)", if rings == 1 { "" } else { "s" }, if systems == 1 { "" } else { "s" }, atoms.len())))),
        pod("Kekule", "Kekulé structures", text(kek.to_string())),
        pod("SMILES", "SMILES", text(c.smiles.clone())),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smiles() {
        let aromatic = parse_smiles("Cn1c(=O)c2c(ncn2C)n(C)c1=O").unwrap();
        assert_eq!(formula_text(&aromatic), "C₈H₁₀N₄O₂");
        assert_eq!((aromatic.donors(), aromatic.acceptors()), (0, 3));
        let caffeine = parse_smiles("CN1C=NC2=C1C(=O)N(C(=O)N2C)C").unwrap();
        assert_eq!(formula_text(&caffeine), "C₈H₁₀N₄O₂");
        assert_eq!(caffeine.acceptors(), 3);
        assert_eq!(caffeine.donors(), 0);
        let benzene = parse_smiles("c1ccccc1").unwrap();
        assert_eq!(formula_text(&benzene), "C₆H₆");
        assert_eq!(benzene.hybridization(0), Some("sp²"));
        let butane = parse_smiles("CCCC").unwrap();
        let idx = butane.indices();
        assert!(idx.contains(&("Wiener index", "10".into())));
        assert!(idx.contains(&("Hosoya index", "5".into())));
        let cp = parse_smiles("C1CCCC1").unwrap();
        assert!(cp.indices().contains(&("Schultz index", "80".into())));
        let pyrrole = parse_smiles("c1cc[nH]c1").unwrap();
        assert_eq!(formula_text(&pyrrole), "C₄H₅N");
        assert_eq!(pyrrole.acceptors(), 0);
        assert_eq!(parse_smiles("[NH4+]").map(|m| formula_text(&m)).unwrap(), "H₄N+");
    }
}

#[cfg(test)]
mod db_check {
    #[test]
    #[ignore]
    fn all_smiles() {
        let (list, _) = super::super::chem::compound_db();
        let (mut n, mut bad, mut mism) = (0, 0, 0);
        for c in list.iter().filter(|c| !c.smiles.is_empty()) {
            n += 1;
            match super::parse_smiles(c.smiles) {
                None => {
                    bad += 1;
                    if bad <= 15 {
                        eprintln!("PARSE {} {}", c.name, c.smiles);
                    }
                }
                Some(m) => {
                    let want = super::super::chem::parse_formula(c.formula);
                    if want.as_ref() != Some(&m.counts()) {
                        mism += 1;
                        if mism <= 40 {
                            eprintln!("FORMULA {} {} {} {:?}", c.name, c.formula, c.smiles, m.counts());
                        }
                    }
                }
            }
        }
        eprintln!("{n} smiles, {bad} unparsed, {mism} formula mismatches");
    }
}
