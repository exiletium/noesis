//! Computational complexity classes and the proven inclusions between them.
//!
//! "complexity class BPP", "non-deterministic polynomial time",
//! "NC vs NLINSPACE", "Is BPL in QP?", "P vs NP".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// abbreviation (with equal classes after ';') | full name | definition | bare-name safe
const CLASSES: &[(&str, &str, &str, bool)] = &[
    ("AC0", "constant-depth circuits", "languages decided by polynomial-size, constant-depth circuits of unbounded fan-in AND, OR and NOT gates", true),
    ("NC1", "logarithmic-depth circuits", "languages decided by polynomial-size, O(log n)-depth circuits of fan-in-2 gates", true),
    ("L;SL;LOGSPACE", "deterministic logarithmic space", "languages decided by a deterministic Turing machine using O(log n) work space (SL = L, Reingold 2005)", true),
    ("RL", "randomized logarithmic space", "languages decided in O(log n) space by a probabilistic machine with one-sided error, always halting in polynomial time", false),
    ("BPL", "bounded-error probabilistic logarithmic space", "languages decided in O(log n) space and polynomial time by a probabilistic machine with two-sided error at most 1/3", true),
    ("NL;coNL", "nondeterministic logarithmic space", "languages decided by a nondeterministic Turing machine using O(log n) space; closed under complement (Immerman–Szelepcsényi), so NL = coNL", false),
    ("NC", "Nick's class", "languages decided in polylogarithmic time on a parallel computer with polynomially many processors (polynomial-size, polylog-depth circuits)", false),
    ("polyL", "polylogarithmic space", "languages decided by a deterministic Turing machine in O(logᵏ n) space for some k", true),
    ("P;PTIME", "polynomial time", "languages decided by a deterministic Turing machine in time O(nᵏ) for some k", false),
    ("ZPP", "zero-error probabilistic polynomial time", "languages decided by a probabilistic machine that never errs and runs in expected polynomial time (ZPP = RP ∩ coRP)", true),
    ("RP", "randomized polynomial time", "languages decided in polynomial time by a probabilistic machine that never accepts a no-instance and accepts a yes-instance with probability at least 1/2", false),
    ("coRP", "complement of randomized polynomial time", "complements of languages in RP: no false negatives, false positives with probability at most 1/2", true),
    ("NP", "nondeterministic polynomial time", "languages whose yes-instances have certificates checkable in polynomial time; decided by a nondeterministic Turing machine in polynomial time", false),
    ("coNP", "complement of nondeterministic polynomial time", "languages whose no-instances have polynomial-time checkable certificates", true),
    ("BPP", "bounded-error probabilistic polynomial time", "languages decided in polynomial time by a probabilistic machine whose error probability is at most 1/3 on every input", true),
    ("BQP", "bounded-error quantum polynomial time", "languages decided by a polynomial-size quantum circuit family with error probability at most 1/3", true),
    ("MA", "Merlin–Arthur", "languages with a one-message interactive proof: a polynomial-size proof checked by a randomized polynomial-time verifier", false),
    ("AM", "Arthur–Merlin", "languages with a constant-round public-coin interactive proof", false),
    ("PH", "polynomial hierarchy", "the union of the levels Σₖᴾ (NP, NP^NP, …) and Πₖᴾ of the polynomial hierarchy", false),
    ("PP", "probabilistic polynomial time", "languages decided in polynomial time by a probabilistic machine that is correct with probability greater than 1/2", false),
    ("QP;QPTIME", "quasi-polynomial time", "languages decided by a deterministic Turing machine in time 2^(O(logᵏ n)) for some k", false),
    ("LINSPACE", "deterministic linear space", "languages decided by a deterministic Turing machine in O(n) space", true),
    ("NLINSPACE", "nondeterministic linear space", "languages decided by a nondeterministic Turing machine in O(n) space; exactly the context-sensitive languages", true),
    ("PSPACE;NPSPACE;IP", "polynomial space", "languages decided by a Turing machine in polynomial space (PSPACE = NPSPACE by Savitch's theorem, = IP by Shamir's theorem)", true),
    ("EXPTIME;EXP", "exponential time", "languages decided by a deterministic Turing machine in time 2^(nᵏ) for some k", true),
    ("NEXPTIME;NEXP", "nondeterministic exponential time", "languages decided by a nondeterministic Turing machine in time 2^(nᵏ)", true),
    ("EXPSPACE", "exponential space", "languages decided by a Turing machine in space 2^(nᵏ)", true),
    ("R", "decidable languages", "languages decided by some Turing machine that always halts (the recursive languages)", false),
    ("RE", "recursively enumerable languages", "languages whose yes-instances are accepted by some Turing machine (semi-decidable)", false),
];

/// Proven inclusions A ⊆ B; `true` marks inclusions known to be proper.
const EDGES: &[(&str, &str, bool)] = &[
    ("AC0", "NC1", true), ("NC1", "L", false), ("L", "RL", false), ("RL", "NL", false), ("RL", "BPL", false),
    ("BPL", "P", false), ("BPL", "polyL", false), ("NL", "NC", false), ("NL", "polyL", false), ("NC", "polyL", false),
    ("NC", "P", false), ("polyL", "LINSPACE", true), ("LINSPACE", "NLINSPACE", false), ("NLINSPACE", "PSPACE", true),
    ("P", "ZPP", false), ("ZPP", "RP", false), ("ZPP", "coRP", false), ("RP", "NP", false), ("coRP", "coNP", false),
    ("RP", "BPP", false), ("coRP", "BPP", false), ("BPP", "BQP", false), ("BPP", "MA", false), ("NP", "MA", false),
    ("MA", "AM", false), ("AM", "PH", false), ("BPP", "PH", false), ("coNP", "PH", false), ("NP", "PP", false),
    ("coNP", "PP", false), ("MA", "PP", false), ("BQP", "PP", false), ("PH", "PSPACE", false), ("PP", "PSPACE", false),
    ("P", "QP", false), ("QP", "EXPTIME", true), ("P", "EXPTIME", true), ("PSPACE", "EXPTIME", false),
    ("EXPTIME", "NEXPTIME", false), ("NP", "NEXPTIME", true), ("NEXPTIME", "EXPSPACE", false), ("PSPACE", "EXPSPACE", true),
    ("EXPSPACE", "R", true), ("R", "RE", true),
];

fn class(s: &str) -> Option<usize> {
    let s = s.trim().trim_start_matches("the ");
    let s = re!(r"^(?i:complexity class(?:es)?)\s+").replace(s, "");
    let s = re!(r"\s+(?i:complexity class)$").replace(&s, "");
    let s = s.trim();
    let full = s.to_lowercase().replace('-', " ").replace("non deterministic", "nondeterministic");
    CLASSES.iter().position(|c| c.0.split(';').any(|a| a == s) || c.1.replace('-', " ") == full)
}

fn key(i: usize) -> &'static str {
    CLASSES[i].0.split(';').next().unwrap()
}

/// Is there a chain of proven inclusions from a to b? Returns whether one link is proper.
fn path(a: &str, b: &str) -> Option<bool> {
    let mut best: Option<bool> = None;
    let mut stack = vec![(a, false)];
    let mut seen: Vec<(&str, bool)> = Vec::new();
    while let Some((x, strict)) = stack.pop() {
        if x == b {
            best = Some(best.unwrap_or(false) || strict);
            continue;
        }
        if seen.contains(&(x, strict)) {
            continue;
        }
        seen.push((x, strict));
        for e in EDGES.iter().filter(|e| e.0 == x) {
            stack.push((e.1, strict || e.2));
        }
    }
    best
}

fn relation(a: usize, b: usize) -> String {
    let (x, y) = (key(a), key(b));
    if a == b {
        return format!("{x} = {y}");
    }
    if (x, y) == ("P", "polyL") || (x, y) == ("polyL", "P") {
        return "P ≠ polyL (polyL has no complete problems under log-space reductions), but neither inclusion is known".into();
    }
    match (path(x, y), path(y, x)) {
        (Some(true), _) => format!("{x} ⊊ {y} (proper inclusion, proven)"),
        (Some(false), _) => format!("{x} ⊆ {y} (whether {x} = {y} is an open problem)"),
        (_, Some(true)) => format!("{y} ⊊ {x} (proper inclusion, proven)"),
        (_, Some(false)) => format!("{y} ⊆ {x} (whether {x} = {y} is an open problem)"),
        _ => format!("no inclusion between {x} and {y} is known"),
    }
}

fn card(i: usize) -> Vec<Pod> {
    let c = CLASSES[i];
    let k = key(i);
    let sub: Vec<&str> = CLASSES.iter().enumerate().filter(|(j, _)| *j != i && path(key(*j), k).is_some()).map(|(j, _)| key(j)).collect();
    let sup: Vec<&str> = CLASSES.iter().enumerate().filter(|(j, _)| *j != i && path(k, key(*j)).is_some()).map(|(j, _)| key(j)).collect();
    let eq: Vec<&str> = c.0.split(';').skip(1).collect();
    let mut rows = vec![vec![text("name"), text(c.1)]];
    if !eq.is_empty() {
        rows.push(vec![text("equal classes"), text(eq.join(", "))]);
    }
    rows.push(vec![text("contains"), text(if sub.is_empty() { "(none listed)".to_string() } else { sub.join(", ") })]);
    rows.push(vec![text("contained in"), text(if sup.is_empty() { "(none listed)".to_string() } else { sup.join(", ") })]);
    vec![
        pod("Input", "Input interpretation", text(format!("{k} (complexity class)"))),
        primary(pod("Result", "Definition", text(c.2))),
        pod("Table", "Properties", table(None, rows)),
    ]
}

pub fn handle(q: &Query) -> Option<Answer> {
    let raw = q.raw.trim().trim_end_matches(['?', '.']).trim();
    // "Is BPL in QP?", "is P a subset of NP"
    if let Some(m) = re!(r"^(?i:is|does)\s+(\S+)\s+(?i:in|a subset of|contained in|included in|equal(?: to)?|=)\s+(\S+)$").captures(raw) {
        let (a, b) = (class(&m[1])?, class(&m[2])?);
        let (x, y) = (key(a), key(b));
        let ans = if a == b {
            "Yes (equal classes)".to_string()
        } else if path(x, y).is_some() {
            format!("Yes: {}", relation(a, b))
        } else if path(y, x) == Some(true) {
            format!("No: {}", relation(a, b))
        } else {
            format!("Not known: {}", relation(a, b))
        };
        return Some(Answer::new("complexity classes", vec![
            pod("Input", "Input interpretation", text(format!("{x} ⊆ {y}?"))),
            primary(pod("Result", "Result", text(ans))),
            pod("Table", "Definitions", table(None, vec![vec![text(x), text(CLASSES[a].2)], vec![text(y), text(CLASSES[b].2)]])),
        ]));
    }
    // "NC vs NLINSPACE", "P vs NP"
    if let Some(m) = re!(r"^(?:complexity classes? )?(\S+)\s+(?i:vs\.?|versus|and)\s+(\S+)$").captures(raw) {
        if let (Some(a), Some(b)) = (class(&m[1]), class(&m[2])) {
            let (x, y) = (key(a), key(b));
            let mut pods = vec![pod("Input", "Input interpretation", text(format!("{x} | {y} (complexity classes)"))), primary(pod("Result", "Relation", text(relation(a, b))))];
            if (x, y) == ("P", "NP") || (x, y) == ("NP", "P") {
                pods.push(pod("Note", "Status", text("P vs NP is one of the Millennium Prize Problems; most researchers believe P ≠ NP")));
            }
            pods.push(pod("Table", "Definitions", table(None, vec![vec![text(x), text(CLASSES[a].1), text(CLASSES[a].2)], vec![text(y), text(CLASSES[b].1), text(CLASSES[b].2)]])));
            return Some(Answer::new("complexity classes", pods));
        }
    }
    // "complexity class BPP", "non-deterministic polynomial time", "BQP"
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let explicit = l.starts_with("complexity class ") || l.ends_with(" complexity class");
    let i = class(raw)?;
    // bare abbreviations only when unambiguous (P, NP, L, … are elements or words)
    let full = CLASSES[i].1.replace('-', " ") == l.replace('-', " ").replace("non deterministic", "nondeterministic");
    if !explicit && !full && !CLASSES[i].3 {
        return None;
    }
    Some(Answer::new("complexity class", card(i)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inclusions() {
        assert_eq!(path("BPL", "QP"), Some(false));
        assert_eq!(path("NC", "NLINSPACE"), Some(true));
        assert_eq!(path("P", "NEXPTIME"), Some(true));
        assert_eq!(path("BPP", "EXPTIME"), Some(false));
        assert_eq!(path("PSPACE", "P"), None);
        assert!(relation(class("P").unwrap(), class("NP").unwrap()).contains("open"));
    }
}
