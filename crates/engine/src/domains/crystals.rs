//! Crystallography: the seven crystal systems, the 32 crystallographic point
//! groups (crystal classes) and the crystal structures of the elements.
//!
//! "trigonal crystal system", "ditetragonal pyramidal point group",
//! "point group 4mm", "carbon crystallography", "crystal structure of iron".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// system | lattice parameters | Bravais lattices | example minerals
const SYSTEMS: &[(&str, &str, &str, &str)] = &[
    ("triclinic", "a ≠ b ≠ c; α ≠ β ≠ γ", "primitive (aP)", "kyanite, albite, microcline, turquoise, wollastonite"),
    ("monoclinic", "a ≠ b ≠ c; α = γ = 90° ≠ β", "primitive (mP), base-centred (mS)", "gypsum, orthoclase, muscovite, talc, malachite"),
    ("orthorhombic", "a ≠ b ≠ c; α = β = γ = 90°", "primitive (oP), base-centred (oS), body-centred (oI), face-centred (oF)", "olivine, topaz, aragonite, barite, sulfur"),
    ("tetragonal", "a = b ≠ c; α = β = γ = 90°", "primitive (tP), body-centred (tI)", "zircon, rutile, cassiterite, chalcopyrite, white tin"),
    ("trigonal", "rhombohedral axes: a = b = c, α = β = γ ≠ 90° (or hexagonal axes: a = b ≠ c, γ = 120°)", "rhombohedral (hR); trigonal groups on a hexagonal lattice use hP", "quartz, calcite, corundum, hematite, tourmaline"),
    ("hexagonal", "a = b ≠ c; α = β = 90°, γ = 120°", "primitive (hP)", "beryl, apatite, graphite, molybdenite, ice Ih"),
    ("cubic", "a = b = c; α = β = γ = 90°", "primitive (cP), body-centred (cI), face-centred (cF)", "halite, pyrite, garnet, diamond, fluorite, magnetite"),
];

/// Hermann–Mauguin symbol | Schoenflies | class name | system | order
const GROUPS: &[(&str, &str, &str, &str, u32)] = &[
    ("1", "C1", "pedial", "triclinic", 1),
    ("-1", "Ci", "pinacoidal", "triclinic", 2),
    ("2", "C2", "sphenoidal", "monoclinic", 2),
    ("m", "Cs", "domatic", "monoclinic", 2),
    ("2/m", "C2h", "prismatic", "monoclinic", 4),
    ("222", "D2", "rhombic disphenoidal", "orthorhombic", 4),
    ("mm2", "C2v", "rhombic pyramidal", "orthorhombic", 4),
    ("mmm", "D2h", "rhombic dipyramidal", "orthorhombic", 8),
    ("4", "C4", "tetragonal pyramidal", "tetragonal", 4),
    ("-4", "S4", "tetragonal disphenoidal", "tetragonal", 4),
    ("4/m", "C4h", "tetragonal dipyramidal", "tetragonal", 8),
    ("422", "D4", "tetragonal trapezohedral", "tetragonal", 8),
    ("4mm", "C4v", "ditetragonal pyramidal", "tetragonal", 8),
    ("-42m", "D2d", "tetragonal scalenohedral", "tetragonal", 8),
    ("4/mmm", "D4h", "ditetragonal dipyramidal", "tetragonal", 16),
    ("3", "C3", "trigonal pyramidal", "trigonal", 3),
    ("-3", "C3i", "rhombohedral", "trigonal", 6),
    ("32", "D3", "trigonal trapezohedral", "trigonal", 6),
    ("3m", "C3v", "ditrigonal pyramidal", "trigonal", 6),
    ("-3m", "D3d", "ditrigonal scalenohedral", "trigonal", 12),
    ("6", "C6", "hexagonal pyramidal", "hexagonal", 6),
    ("-6", "C3h", "trigonal dipyramidal", "hexagonal", 6),
    ("6/m", "C6h", "hexagonal dipyramidal", "hexagonal", 12),
    ("622", "D6", "hexagonal trapezohedral", "hexagonal", 12),
    ("6mm", "C6v", "dihexagonal pyramidal", "hexagonal", 12),
    ("-6m2", "D3h", "ditrigonal dipyramidal", "hexagonal", 12),
    ("6/mmm", "D6h", "dihexagonal dipyramidal", "hexagonal", 24),
    ("23", "T", "tetartoidal", "cubic", 12),
    ("m-3", "Th", "diploidal", "cubic", 24),
    ("432", "O", "gyroidal", "cubic", 24),
    ("-43m", "Td", "hextetrahedral", "cubic", 24),
    ("m-3m", "Oh", "hexoctahedral", "cubic", 48),
];

/// Crystal structure of each element in its standard state (most stable
/// form at room temperature, or on freezing for gases and liquids).
const ELEMENTS: &[(&str, &str)] = &[
    ("H", "hexagonal close-packed (solid)"), ("He", "hexagonal close-packed (solid, under pressure)"), ("Li", "body-centred cubic"), ("Be", "hexagonal close-packed"),
    ("B", "rhombohedral (β-boron)"), ("C", "hexagonal (graphite); diamond is face-centred cubic (diamond cubic)"), ("N", "hexagonal (solid β-nitrogen)"), ("O", "monoclinic (solid α-oxygen)"),
    ("F", "monoclinic (solid α-fluorine)"), ("Ne", "face-centred cubic (solid)"), ("Na", "body-centred cubic"), ("Mg", "hexagonal close-packed"), ("Al", "face-centred cubic"),
    ("Si", "diamond cubic (face-centred cubic)"), ("P", "orthorhombic (black phosphorus); white phosphorus is cubic"), ("S", "orthorhombic (α-sulfur)"), ("Cl", "orthorhombic (solid)"),
    ("Ar", "face-centred cubic (solid)"), ("K", "body-centred cubic"), ("Ca", "face-centred cubic"), ("Sc", "hexagonal close-packed"), ("Ti", "hexagonal close-packed"),
    ("V", "body-centred cubic"), ("Cr", "body-centred cubic"), ("Mn", "body-centred cubic (complex α-Mn, 58 atoms per cell)"), ("Fe", "body-centred cubic (α-iron; face-centred cubic γ-iron from 912 °C to 1394 °C)"),
    ("Co", "hexagonal close-packed"), ("Ni", "face-centred cubic"), ("Cu", "face-centred cubic"), ("Zn", "hexagonal close-packed"), ("Ga", "orthorhombic"),
    ("Ge", "diamond cubic (face-centred cubic)"), ("As", "rhombohedral (grey arsenic)"), ("Se", "hexagonal (trigonal grey selenium)"), ("Br", "orthorhombic (solid)"),
    ("Kr", "face-centred cubic (solid)"), ("Rb", "body-centred cubic"), ("Sr", "face-centred cubic"), ("Y", "hexagonal close-packed"), ("Zr", "hexagonal close-packed"),
    ("Nb", "body-centred cubic"), ("Mo", "body-centred cubic"), ("Tc", "hexagonal close-packed"), ("Ru", "hexagonal close-packed"), ("Rh", "face-centred cubic"),
    ("Pd", "face-centred cubic"), ("Ag", "face-centred cubic"), ("Cd", "hexagonal close-packed"), ("In", "body-centred tetragonal"), ("Sn", "body-centred tetragonal (white β-tin); grey α-tin below 13.2 °C is diamond cubic"),
    ("Sb", "rhombohedral"), ("Te", "hexagonal (trigonal)"), ("I", "orthorhombic"), ("Xe", "face-centred cubic (solid)"), ("Cs", "body-centred cubic"), ("Ba", "body-centred cubic"),
    ("La", "double hexagonal close-packed"), ("Ce", "face-centred cubic (γ-cerium)"), ("Pr", "double hexagonal close-packed"), ("Nd", "double hexagonal close-packed"), ("Pm", "double hexagonal close-packed"),
    ("Sm", "rhombohedral"), ("Eu", "body-centred cubic"), ("Gd", "hexagonal close-packed"), ("Tb", "hexagonal close-packed"), ("Dy", "hexagonal close-packed"), ("Ho", "hexagonal close-packed"),
    ("Er", "hexagonal close-packed"), ("Tm", "hexagonal close-packed"), ("Yb", "face-centred cubic"), ("Lu", "hexagonal close-packed"), ("Hf", "hexagonal close-packed"),
    ("Ta", "body-centred cubic"), ("W", "body-centred cubic"), ("Re", "hexagonal close-packed"), ("Os", "hexagonal close-packed"), ("Ir", "face-centred cubic"), ("Pt", "face-centred cubic"),
    ("Au", "face-centred cubic"), ("Hg", "rhombohedral (solid, below −38.8 °C)"), ("Tl", "hexagonal close-packed"), ("Pb", "face-centred cubic"), ("Bi", "rhombohedral"),
    ("Po", "simple cubic (α-polonium, the only element with this structure)"), ("Ra", "body-centred cubic"), ("Ac", "face-centred cubic"), ("Th", "face-centred cubic"),
    ("Pa", "body-centred tetragonal"), ("U", "orthorhombic (α-uranium)"), ("Np", "orthorhombic"), ("Pu", "monoclinic (α-plutonium)"), ("Am", "double hexagonal close-packed"),
];

fn norm(s: &str) -> String {
    s.to_lowercase().replace(['-', '_'], " ").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn hm(s: &str) -> String {
    // "4mm", "m-3m", "m3m", "-42m", "4/mmm" (also with ̄ or "bar")
    s.trim().replace(' ', "").replace("bar", "-").replace('¯', "-").to_lowercase()
}

fn group_card(g: &(&str, &str, &str, &str, u32)) -> Answer {
    let centro = matches!(g.0, "-1" | "2/m" | "mmm" | "4/m" | "4/mmm" | "-3" | "-3m" | "6/m" | "6/mmm" | "m-3" | "m-3m");
    let laue = GROUPS.iter().filter(|x| x.3 == g.3 && matches!(x.0, "-1" | "2/m" | "mmm" | "4/m" | "4/mmm" | "-3" | "-3m" | "6/m" | "6/mmm" | "m-3" | "m-3m")).map(|x| x.0).collect::<Vec<_>>().join(", ");
    Answer::new("crystallographic point group", vec![
        pod("Input", "Input interpretation", text(format!("{} point group (crystal class)", g.2))),
        primary(pod("Result", "Symbols", text(format!("Hermann–Mauguin {} | Schoenflies {}", g.0, g.1)))),
        pod("Table", "Properties", table(None, vec![
            vec![text("crystal class"), text(g.2)],
            vec![text("crystal system"), text(g.3)],
            vec![text("group order"), text(g.4.to_string())],
            vec![text("centrosymmetric"), text(if centro { "yes" } else { "no" })],
            vec![text("Laue classes of the system"), text(laue)],
        ])),
    ])
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let l = l.trim_start_matches("what is the ").trim_start_matches("the ");
    // crystal systems
    if let Some(m) = re!(r"^(triclinic|monoclinic|orthorhombic|tetragonal|trigonal|rhombohedral|hexagonal|cubic|isometric)(?: crystal)? (?:crystal )?(?:system|family|lattice)$").captures(l) {
        let name = match &m[1] { "isometric" => "cubic", "rhombohedral" => "trigonal", x => x };
        let s = SYSTEMS.iter().find(|s| s.0 == name)?;
        let classes: Vec<String> = GROUPS.iter().filter(|g| g.3 == name).map(|g| format!("{} ({})", g.0, g.2)).collect();
        return Some(Answer::new("crystal system", vec![
            pod("Input", "Input interpretation", text(format!("{} crystal system", s.0))),
            primary(pod("Result", "Lattice parameters", text(s.1))),
            pod("Table", "Properties", table(None, vec![
                vec![text("Bravais lattices"), text(s.2)],
                vec![text("point groups"), text(format!("{}: {}", classes.len(), classes.join(", ")))],
                vec![text("example minerals"), text(s.3)],
            ])),
        ]));
    }
    if re!(r"^(?:list of |the )?(?:7 |seven )?crystal systems$").is_match(l) {
        let rows = SYSTEMS.iter().map(|s| vec![text(s.0), text(s.1), text(GROUPS.iter().filter(|g| g.3 == s.0).count().to_string()), text(s.3)]).collect();
        return Some(Answer::new("crystal systems", vec![
            pod("Input", "Input interpretation", text("crystal systems")),
            primary(pod("Result", "Result", text(SYSTEMS.iter().map(|s| s.0).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Crystal systems", table(Some(vec!["system", "lattice parameters", "point groups", "examples"]), rows)),
        ]));
    }
    if re!(r"^(?:list of |the )?(?:32 )?(?:crystallographic point groups|crystal classes|point groups)$").is_match(l) {
        let rows = GROUPS.iter().map(|g| vec![text(g.0), text(g.1), text(g.2), text(g.3), text(g.4.to_string())]).collect();
        return Some(Answer::new("point groups", vec![
            pod("Input", "Input interpretation", text("crystallographic point groups")),
            primary(pod("Result", "Count", text("32 crystal classes in 7 crystal systems"))),
            pod("Table", "Point groups", table(Some(vec!["Hermann–Mauguin", "Schoenflies", "class", "system", "order"]), rows)),
        ]));
    }
    // point groups by class name or symbol
    if let Some(m) = re!(r"^(?:crystal class |point group |crystallographic point group )?(.+?)(?: point group| crystal class| class)?$").captures(l) {
        let n = norm(&m[1]);
        let explicit = l.contains("point group") || l.contains("class");
        let by_name = GROUPS.iter().find(|g| g.2 == n && (explicit || n.contains(' ') || n.len() > 9));
        let by_symbol = if explicit { GROUPS.iter().find(|g| hm(g.0) == hm(&m[1]) || g.1.eq_ignore_ascii_case(m[1].trim())) } else { None };
        if let Some(g) = by_name.or(by_symbol) {
            return Some(group_card(g));
        }
    }
    // element crystal structures
    if let Some(m) = re!(r"^(?:crystal (?:structure|system|lattice)|crystallography|lattice(?: structure)?) (?:of |for )?(.+)$|^(.+?)(?:'s)? (?:crystal (?:structure|system|lattice)|crystallography|lattice structure)$").captures(l) {
        let name = m.get(1).or(m.get(2))?.as_str();
        let e = super::chem::find_element(name)?;
        let s = ELEMENTS.iter().find(|x| x.0 == e.symbol)?.1;
        return Some(Answer::new(e.name.clone(), vec![
            pod("Input", "Input interpretation", text(format!("{} | crystal structure", e.name))),
            primary(pod("Result", "Crystal structure", text(s))),
        ]));
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn tables_are_consistent() {
        assert_eq!(super::GROUPS.len(), 32);
        for s in super::SYSTEMS {
            assert!(super::GROUPS.iter().any(|g| g.3 == s.0));
        }
    }
}
