//! Common Core State Standards for mathematics, by identifier: decodes
//! "CCSS.Math.Content.8.G.A.1", "fifth grade common core standard MD.C.5" or
//! "congruence standard B7 high school common core" into grade, domain,
//! cluster and standard, with the official page for the wording; lists the
//! domains of a grade, or the grades of a domain.

use super::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

const DOMAINS: &[(&str, &str)] = &[
    ("CC", "Counting and Cardinality"),
    ("OA", "Operations and Algebraic Thinking"),
    ("NBT", "Number and Operations in Base Ten"),
    ("NF", "Number and Operations—Fractions"),
    ("MD", "Measurement and Data"),
    ("G", "Geometry"),
    ("RP", "Ratios and Proportional Relationships"),
    ("NS", "The Number System"),
    ("EE", "Expressions and Equations"),
    ("F", "Functions"),
    ("SP", "Statistics and Probability"),
];

const HS: &[(&str, &str, &str)] = &[
    ("N", "RN", "The Real Number System"),
    ("N", "Q", "Quantities"),
    ("N", "CN", "The Complex Number System"),
    ("N", "VM", "Vector and Matrix Quantities"),
    ("A", "SSE", "Seeing Structure in Expressions"),
    ("A", "APR", "Arithmetic with Polynomials and Rational Expressions"),
    ("A", "CED", "Creating Equations"),
    ("A", "REI", "Reasoning with Equations and Inequalities"),
    ("F", "IF", "Interpreting Functions"),
    ("F", "BF", "Building Functions"),
    ("F", "LE", "Linear, Quadratic, and Exponential Models"),
    ("F", "TF", "Trigonometric Functions"),
    ("G", "CO", "Congruence"),
    ("G", "SRT", "Similarity, Right Triangles, and Trigonometry"),
    ("G", "C", "Circles"),
    ("G", "GPE", "Expressing Geometric Properties with Equations"),
    ("G", "GMD", "Geometric Measurement and Dimension"),
    ("G", "MG", "Modeling with Geometry"),
    ("S", "ID", "Interpreting Categorical and Quantitative Data"),
    ("S", "IC", "Making Inferences and Justifying Conclusions"),
    ("S", "CP", "Conditional Probability and the Rules of Probability"),
    ("S", "MD", "Using Probability to Make Decisions"),
];

const CATEGORIES: &[(&str, &str)] = &[("N", "Number and Quantity"), ("A", "Algebra"), ("F", "Functions"), ("G", "Geometry"), ("S", "Statistics and Probability")];

fn grade_domains(g: &str) -> &'static [&'static str] {
    match g {
        "K" => &["CC", "OA", "NBT", "MD", "G"],
        "1" | "2" => &["OA", "NBT", "MD", "G"],
        "3" | "4" | "5" => &["OA", "NBT", "NF", "MD", "G"],
        "6" | "7" => &["RP", "NS", "EE", "G", "SP"],
        "8" => &["NS", "EE", "F", "G", "SP"],
        _ => &[],
    }
}

fn grade_of(s: &str) -> Option<String> {
    let s = s.trim();
    Some(match s {
        "k" | "kindergarten" => "K".into(),
        "first" | "1st" | "1" => "1".into(),
        "second" | "2nd" | "2" => "2".into(),
        "third" | "3rd" | "3" => "3".into(),
        "fourth" | "4th" | "4" => "4".into(),
        "fifth" | "5th" | "5" => "5".into(),
        "sixth" | "6th" | "6" => "6".into(),
        "seventh" | "7th" | "7" => "7".into(),
        "eighth" | "8th" | "8" => "8".into(),
        "high school" | "hs" => "HS".into(),
        _ => return None,
    })
}

/// Domain code from a code or from words, for a grade (or high school).
fn domain_of(s: &str, grade: Option<&str>) -> Option<String> {
    let s = s.trim().trim_end_matches(" common core").trim();
    let hs = grade == Some("HS");
    let words: &[(&str, &str, &str)] = &[
        ("counting and cardinality", "CC", ""),
        ("operations and algebraic thinking", "OA", ""),
        ("algebraic thinking", "OA", ""),
        ("base ten", "NBT", ""),
        ("base ten numbers", "NBT", ""),
        ("number and operations in base ten", "NBT", ""),
        ("fractions", "NF", ""),
        ("measurement and data", "MD", ""),
        ("measurement", "MD", ""),
        ("ratios and proportional relationships", "RP", ""),
        ("ratios & proportions", "RP", ""),
        ("ratios and proportions", "RP", ""),
        ("ratios", "RP", ""),
        ("number system", "NS", "N-RN"),
        ("expressions and equations", "EE", ""),
        ("statistics and probability", "SP", "S"),
        ("statistics", "SP", "S"),
        ("functions", "F", "F"),
        ("geometry", "G", "G"),
        ("algebra", "", "A"),
        ("number and quantity", "", "N"),
        ("congruence", "", "G-CO"),
        ("creating equations", "", "A-CED"),
        ("trigonometric functions", "", "F-TF"),
        ("interpreting functions", "", "F-IF"),
        ("building functions", "", "F-BF"),
        ("circles", "", "G-C"),
        ("quantities", "", "N-Q"),
    ];
    if let Some((_, k, h)) = words.iter().find(|(w, _, _)| *w == s) {
        let v = if hs || k.is_empty() { h } else { k };
        return (!v.is_empty()).then(|| v.to_string());
    }
    let up = s.to_uppercase().replace(' ', "");
    if hs {
        if let Some(c) = re!(r"^(?:HS)?([NAFGS])-?([A-Z]+)$").captures(&up) {
            if HS.iter().any(|(cat, d, _)| *cat == &c[1] && *d == &c[2]) {
                return Some(format!("{}-{}", &c[1], &c[2]));
            }
        }
        if let Some((cat, d, _)) = HS.iter().find(|(_, d, _)| *d == up) {
            // MD under high school is the statistics domain
            return Some(format!("{cat}-{d}"));
        }
        return None;
    }
    DOMAINS.iter().any(|(d, _)| *d == up).then_some(up)
}

fn domain_name(code: &str) -> Option<String> {
    if let Some((cat, d)) = code.split_once('-') {
        let name = HS.iter().find(|(c, x, _)| *c == cat && *x == d)?.2;
        let catn = CATEGORIES.iter().find(|(c, _)| *c == cat)?.1;
        return Some(format!("{name} (conceptual category: {catn})"));
    }
    if let Some((_, n)) = CATEGORIES.iter().find(|(c, _)| *c == code).filter(|_| code.len() == 1 && !DOMAINS.iter().any(|(d, _)| *d == code)) {
        return Some(n.to_string());
    }
    DOMAINS.iter().find(|(d, _)| *d == code).map(|(_, n)| n.to_string())
}

fn grade_name(g: &str) -> String {
    match g {
        "K" => "kindergarten".into(),
        "HS" => "high school".into(),
        "1" => "grade 1".into(),
        x => format!("grade {x}"),
    }
}

fn standard(grade: &str, dom: &str, cluster: &str, num: &str, sub: Option<&str>) -> Option<Answer> {
    let cluster = cluster.to_uppercase();
    let n: u32 = num.parse().ok().filter(|n| (1..=12).contains(n))?;
    if !("A"..="E").contains(&cluster.as_str()) {
        return None;
    }
    if grade != "HS" && !grade_domains(grade).contains(&dom) {
        return None;
    }
    let sub = sub.map(|s| s.to_lowercase());
    let (id, path) = if grade == "HS" {
        let (cat, d) = dom.split_once('-')?;
        (format!("HS{cat}-{d}.{cluster}.{n}{}", sub.clone().unwrap_or_default()), format!("HS{cat}/{d}/{cluster}/{n}/{}", sub.clone().map(|s| format!("{s}/")).unwrap_or_default()))
    } else {
        (format!("{grade}.{dom}.{cluster}.{n}{}", sub.clone().unwrap_or_default()), format!("{grade}/{dom}/{cluster}/{n}/{}", sub.clone().map(|s| format!("{s}/")).unwrap_or_default()))
    };
    let mut rows = vec![("identifier", format!("CCSS.Math.Content.{id}")), ("grade", grade_name(grade)), ("domain", format!("{dom}: {}", domain_name(dom)?)), ("cluster", cluster.clone()), ("standard", n.to_string())];
    if let Some(s) = &sub {
        rows.push(("part", s.clone()));
    }
    rows.push(("official wording", format!("https://www.corestandards.org/Math/Content/{path}")));
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("Common Core math standard {id}"))), primary(pod("Result", "Standard", table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())))];
    if grade != "HS" {
        pods.push(pod("Domains", &format!("Domains in {}", grade_name(grade)), text(grade_domains(grade).iter().map(|d| format!("{d} ({})", domain_name(d).unwrap_or_default())).collect::<Vec<_>>().join("; "))));
    }
    Some(Answer::new("Common Core standard", pods))
}

fn listing(grade: Option<String>, dom: Option<String>) -> Option<Answer> {
    let (title, rows): (String, Vec<Vec<Content>>) = match (grade.as_deref(), dom.as_deref()) {
        (Some("HS"), None) => ("high school Common Core math domains".into(), HS.iter().map(|(c, d, n)| vec![text(format!("{c}-{d}")), text(*n)]).collect()),
        (Some("HS"), Some(d)) if d.len() == 1 => (format!("high school Common Core standards: {}", domain_name(d)?), HS.iter().filter(|(c, _, _)| *c == d).map(|(c, x, n)| vec![text(format!("{c}-{x}")), text(*n)]).collect()),
        (Some(g), None) => (format!("{} Common Core math domains", grade_name(g)), grade_domains(g).iter().map(|d| vec![text(format!("{g}.{d}")), text(domain_name(d).unwrap_or_default())]).collect()),
        (g, Some(d)) => {
            let grades: Vec<&str> = ["K", "1", "2", "3", "4", "5", "6", "7", "8"].into_iter().filter(|x| g.map_or(true, |g| g == *x) && grade_domains(x).contains(&d)).collect();
            let mut rows: Vec<Vec<Content>> = grades.iter().map(|x| vec![text(format!("{x}.{d}")), text(format!("{}, {}", grade_name(x), domain_name(d).unwrap_or_default()))]).collect();
            if g.is_none() {
                let cat = match d {
                    "G" => Some("G"),
                    "F" => Some("F"),
                    "SP" => Some("S"),
                    "NS" => Some("N"),
                    "EE" | "OA" => Some("A"),
                    _ => None,
                };
                if let Some(c) = cat {
                    rows.extend(HS.iter().filter(|(cc, _, _)| *cc == c).map(|(cc, x, n)| vec![text(format!("HS{cc}-{x}")), text(format!("high school, {n}"))]));
                }
            }
            if rows.is_empty() {
                return None;
            }
            (format!("Common Core standards in {}", domain_name(d)?), rows)
        }
        _ => return None,
    };
    if rows.is_empty() {
        return None;
    }
    Some(Answer::new("Common Core standards", vec![pod("Input", "Input interpretation", text(title)), primary(pod("Result", "Domains", table(Some(vec!["code", "domain"]), rows))), pod("Source", "Source", text("https://www.corestandards.org/Math/"))]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    // full identifiers
    if let Some(m) = re!(r"^(?:ccss\.)?math\.content\.(k|\d)\.([a-z]+)\.([a-e])\.(\d+)([a-f])?$").captures(&l) {
        let g = grade_of(&m[1])?;
        return standard(&g, &m[2].to_uppercase(), &m[3], &m[4], m.get(5).map(|x| x.as_str()));
    }
    if let Some(m) = re!(r"^(?:ccss\.)?math\.content\.hs([nafgs])-([a-z]+)\.([a-e])\.(\d+)([a-f])?$").captures(&l) {
        let d = domain_of(&format!("{}-{}", &m[1], &m[2]), Some("HS"))?;
        return standard("HS", &d, &m[3], &m[4], m.get(5).map(|x| x.as_str()));
    }
    if !l.contains("common core") {
        return None;
    }
    let body = re!(r"\b(?:common core(?: math)?(?: state)? standards?|standards?)\b").replace_all(&l, " ").to_string();
    let body = re!(r"\s+").replace_all(body.trim(), " ").to_string();
    // "5.md.c.3a", "7.ns.a.1d"
    if let Some(m) = re!(r"^(?:.*?\s)?(k|\d)\.([a-z]+)\.([a-e])\.(\d+)([a-f])?(?:\s.*)?$").captures(&body) {
        let g = grade_of(&m[1])?;
        return standard(&g, &m[2].to_uppercase(), &m[3], &m[4], m.get(5).map(|x| x.as_str()));
    }
    let grade_re = r"(kindergarten|first|second|third|fourth|fifth|sixth|seventh|eighth|\dth|1st|2nd|3rd|high school)(?: grade)?|grade (k|\d)";
    let gm = Regex::new(grade_re).ok()?.captures(&body);
    let grade = gm.as_ref().and_then(|m| grade_of(m.get(1).or(m.get(2))?.as_str()));
    let rest = gm.as_ref().map(|m| body.replacen(m.get(0).unwrap().as_str(), " ", 1)).unwrap_or_else(|| body.clone());
    let rest = re!(r"\s+").replace_all(rest.trim(), " ").to_string();
    let rest = re!(r"^(?:(?:in|for|about|on|of)\b\s*)+|\s*\b(?:in|for|about|on|of)$").replace_all(&rest, "").trim().to_string();
    // "md.c.5", "nbt b5", "geometry b5", "ratios & proportions a.2", "functions a2", "vm.b.4.b", "tf.b.5"
    if let Some(m) = re!(r"^(?:([a-z& ]+?)\s+)?([a-z]+)?[. ]?([a-e])\.?(\d+)(?:\.?([a-f]))?$").captures(&rest) {
        let words = m.get(1).map(|x| x.as_str().trim()).unwrap_or("");
        let code = m.get(2).map(|x| x.as_str()).unwrap_or("");
        let g = grade.clone().unwrap_or_else(|| "HS".into());
        let dom = if !code.is_empty() && domain_of(code, Some(&g)).is_some() {
            domain_of(code, Some(&g))?
        } else {
            let w = format!("{words} {code}");
            domain_of(w.trim(), Some(&g))?
        };
        // a high-school category alone needs its domain
        if g == "HS" && !dom.contains('-') {
            return None;
        }
        return standard(&g, &dom, &m[3], &m[4], m.get(5).map(|x| x.as_str()));
    }
    let dom = if rest.is_empty() { None } else { Some(domain_of(&rest, grade.as_deref())?) };
    listing(grade, dom)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ans(s: &str) -> String {
        serde_json::to_string(&crate::engine::query(s).pods).unwrap_or_default()
    }

    #[test]
    fn common_core() {
        assert!(ans("CCSS.Math.Content.HSG-SRT.A.2").contains("HSG/SRT/A/2/"));
        assert!(ans("fifth grade common core standard MD.C.5").contains("5/MD/C/5/"));
        assert!(ans("7.NS.A.1d common core standard").contains("7/NS/A/1/d/"));
        assert!(ans("congruence standard B7 high school common core").contains("HSG/CO/B/7/"));
        assert!(ans("third grade algebraic thinking common core standard D8").contains("3/OA/D/8/"));
        assert!(ans("common core standard vm.b.4.b").contains("HSN/VM/B/4/b/"));
    }
}
