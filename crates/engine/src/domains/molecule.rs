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

fn resolve(t: &str) -> Option<Compound> {
    let t = t.trim().trim_matches(|c: char| c == '"' || c == '?' || c == '.').trim();
    if let Some(m) = re!(r"(?i)^smiles\s*:?\s*(\S+)$").captures(t) {
        let smi = m[1].to_string();
        return Some(Compound { name: smi.clone(), mol: parse_smiles(&smi)?, smiles: smi });
    }
    let t = re!(r"(?i)^(?:the |a |an )?(?:molecule |compound |chemical )?").replace(t, "").to_string();
    let t = re!(r"(?i) (?:molecule|compound|chemical|gas)$").replace(&t, "").to_string();
    let c = find_db_compound(&t).or_else(|| {
        // stereo descriptors: "L-2-aminobutyric acid", "cis cyclononene", "(R)-…"
        let bare = re!(r"(?i)^(?:dl-|l-|d-|\([rs+\-±]+\)-|cis[- ]|trans[- ]|\(?[ez]\)?-|α-|β-)").replace(&t, "").to_string();
        (bare != t).then(|| find_db_compound(&bare)).flatten()
    })?;
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
