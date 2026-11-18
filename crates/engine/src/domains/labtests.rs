//! Reference ranges of common blood tests, and where a value falls.
//!
//! "serum blood urea nitrogen", "creatinine=0.9mg/dL, adult male",
//! "normal hemoglobin level", "potassium 5.8 mmol/L", "TSH reference range".

use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

struct Test {
    /// names separated by ';' (the first is the display name)
    names: &'static str,
    unit: &'static str,
    /// (low, high) for men, then women; the same when not sex-specific
    male: (f64, f64),
    female: (f64, f64),
    /// conversion factor from `unit` to the SI unit, and that unit
    si: (f64, &'static str),
    what: &'static str,
}

macro_rules! t {
    ($n:expr, $u:expr, $m:expr, $f:expr, $si:expr, $w:expr) => {
        Test { names: $n, unit: $u, male: $m, female: $f, si: $si, what: $w }
    };
}

const INF: f64 = f64::INFINITY;

/// Typical adult reference ranges (Mayo Clinic Laboratories and standard
/// clinical references); laboratories differ slightly.
const TESTS: &[Test] = &[
    t!("blood urea nitrogen;bun;urea nitrogen;serum urea nitrogen;blood urea nitrogen level", "mg/dL", (7.0, 20.0), (7.0, 20.0), (0.357, "mmol/L urea"), "nitrogen carried as urea, a waste product of protein metabolism cleared by the kidneys; raised in kidney disease, dehydration and high protein intake"),
    t!("creatinine;serum creatinine;blood creatinine", "mg/dL", (0.74, 1.35), (0.59, 1.04), (88.4, "µmol/L"), "a waste product of muscle creatine cleared by the kidneys; used to estimate the glomerular filtration rate"),
    t!("fasting blood glucose;fasting glucose;blood glucose;blood sugar;fasting blood sugar;glucose level;blood sugar level", "mg/dL", (70.0, 99.0), (70.0, 99.0), (0.0555, "mmol/L"), "fasting plasma glucose; 100–125 mg/dL is prediabetes and 126 mg/dL or more on two tests is diabetes"),
    t!("hemoglobin a1c;hba1c;a1c;glycated hemoglobin", "%", (4.0, 5.6), (4.0, 5.6), (1.0, "%"), "average blood glucose over about three months; 5.7–6.4% is prediabetes and 6.5% or more is diabetes"),
    t!("sodium;serum sodium;blood sodium", "mmol/L", (135.0, 145.0), (135.0, 145.0), (1.0, "mmol/L"), "the main extracellular electrolyte; low is hyponatremia and high is hypernatremia"),
    t!("potassium;serum potassium;blood potassium", "mmol/L", (3.5, 5.0), (3.5, 5.0), (1.0, "mmol/L"), "the main intracellular electrolyte; values outside the range can disturb heart rhythm"),
    t!("chloride;serum chloride", "mmol/L", (96.0, 106.0), (96.0, 106.0), (1.0, "mmol/L"), "an electrolyte that follows sodium and helps keep acid–base balance"),
    t!("bicarbonate;serum bicarbonate;total co2;co2 blood test", "mmol/L", (23.0, 29.0), (23.0, 29.0), (1.0, "mmol/L"), "the main blood buffer; low in metabolic acidosis, high in metabolic alkalosis"),
    t!("calcium;serum calcium;blood calcium", "mg/dL", (8.6, 10.3), (8.6, 10.3), (0.2495, "mmol/L"), "total serum calcium; regulated by parathyroid hormone and vitamin D"),
    t!("magnesium;serum magnesium", "mg/dL", (1.7, 2.2), (1.7, 2.2), (0.4114, "mmol/L"), "an electrolyte needed by muscles, nerves and hundreds of enzymes"),
    t!("phosphorus;serum phosphorus;phosphate;serum phosphate", "mg/dL", (2.5, 4.5), (2.5, 4.5), (0.3229, "mmol/L"), "inorganic phosphate, balanced with calcium in bone"),
    t!("total cholesterol;cholesterol;cholesterol level;blood cholesterol", "mg/dL", (0.0, 199.0), (0.0, 199.0), (0.02586, "mmol/L"), "desirable below 200 mg/dL; 200–239 borderline high; 240 or more high"),
    t!("ldl cholesterol;ldl;ldl-c", "mg/dL", (0.0, 99.0), (0.0, 99.0), (0.02586, "mmol/L"), "low-density lipoprotein cholesterol; optimal below 100 mg/dL, 160 or more high"),
    t!("hdl cholesterol;hdl;hdl-c", "mg/dL", (40.0, INF), (50.0, INF), (0.02586, "mmol/L"), "high-density lipoprotein cholesterol; higher is better and 60 mg/dL or more is protective"),
    t!("triglycerides;triglyceride;serum triglycerides", "mg/dL", (0.0, 149.0), (0.0, 149.0), (0.01129, "mmol/L"), "fasting blood fats; 150–199 borderline high, 200–499 high, 500 or more very high"),
    t!("hemoglobin;haemoglobin;hgb;hb", "g/dL", (13.2, 16.6), (11.6, 15.0), (10.0, "g/L"), "the oxygen-carrying protein of red blood cells; low values mean anemia"),
    t!("hematocrit;haematocrit;hct;packed cell volume", "%", (38.3, 48.6), (35.5, 44.9), (0.01, "L/L"), "the fraction of blood volume made up of red cells"),
    t!("white blood cell count;wbc;white cell count;leukocyte count;white blood cells", "×10⁹/L", (3.4, 9.6), (3.4, 9.6), (1.0, "×10⁹/L"), "leukocytes; raised in infection and inflammation, lowered by some drugs and marrow disorders"),
    t!("red blood cell count;rbc;red cell count;erythrocyte count;red blood cells", "×10¹²/L", (4.35, 5.65), (3.92, 5.13), (1.0, "×10¹²/L"), "erythrocytes per litre of blood"),
    t!("platelet count;platelets;plt;thrombocyte count", "×10⁹/L", (135.0, 317.0), (157.0, 371.0), (1.0, "×10⁹/L"), "thrombocytes, needed for clotting"),
    t!("alanine aminotransferase;alt;sgpt", "U/L", (7.0, 55.0), (7.0, 45.0), (1.0, "U/L"), "a liver enzyme; raised in liver injury"),
    t!("aspartate aminotransferase;ast;sgot", "U/L", (8.0, 48.0), (8.0, 43.0), (1.0, "U/L"), "an enzyme of liver, heart and muscle; raised in their injury"),
    t!("alkaline phosphatase;alp", "U/L", (40.0, 129.0), (35.0, 104.0), (1.0, "U/L"), "an enzyme of liver and bone; raised in bile-duct obstruction and bone growth or disease"),
    t!("total bilirubin;bilirubin;serum bilirubin", "mg/dL", (0.1, 1.2), (0.1, 1.2), (17.1, "µmol/L"), "a breakdown product of hemoglobin; raised levels cause jaundice"),
    t!("albumin;serum albumin", "g/dL", (3.5, 5.0), (3.5, 5.0), (10.0, "g/L"), "the main blood protein, made by the liver"),
    t!("total protein;serum total protein", "g/dL", (6.3, 7.9), (6.3, 7.9), (10.0, "g/L"), "albumin plus globulins"),
    t!("thyroid stimulating hormone;tsh;thyrotropin", "mIU/L", (0.4, 4.0), (0.4, 4.0), (1.0, "mIU/L"), "high in hypothyroidism and low in hyperthyroidism"),
    t!("free t4;free thyroxine;ft4", "ng/dL", (0.9, 1.7), (0.9, 1.7), (12.87, "pmol/L"), "unbound thyroxine, the main thyroid hormone"),
    t!("uric acid;serum uric acid;urate", "mg/dL", (3.4, 7.0), (2.4, 6.0), (59.48, "µmol/L"), "the end product of purine metabolism; high levels can cause gout"),
    t!("ferritin;serum ferritin", "ng/mL", (24.0, 336.0), (11.0, 307.0), (1.0, "µg/L"), "the iron-storage protein; low values show iron deficiency"),
    t!("vitamin d;25-hydroxyvitamin d;25-oh vitamin d;vitamin d level", "ng/mL", (20.0, 50.0), (20.0, 50.0), (2.496, "nmol/L"), "below 12 ng/mL is deficient and 12–20 ng/mL may be inadequate (Institute of Medicine)"),
    t!("vitamin b12;b12;cobalamin", "pg/mL", (180.0, 914.0), (180.0, 914.0), (0.7378, "pmol/L"), "needed for red blood cells and nerves"),
    t!("c-reactive protein;crp", "mg/L", (0.0, 8.0), (0.0, 8.0), (1.0, "mg/L"), "an inflammation marker; for high-sensitivity CRP, below 1 mg/L is low cardiovascular risk and above 3 mg/L high"),
    t!("prostate specific antigen;psa", "ng/mL", (0.0, 4.0), (0.0, 4.0), (1.0, "µg/L"), "a prostate protein; values above about 4 ng/mL prompt further evaluation"),
    t!("international normalized ratio;inr", "", (0.8, 1.1), (0.8, 1.1), (1.0, ""), "clotting time relative to normal; the target on warfarin is usually 2–3"),
    t!("blood ph;arterial ph;ph of blood", "", (7.35, 7.45), (7.35, 7.45), (1.0, ""), "below 7.35 is acidemia and above 7.45 alkalemia"),
    t!("serum osmolality;plasma osmolality;osmolality", "mOsm/kg", (275.0, 295.0), (275.0, 295.0), (1.0, "mmol/kg"), "the concentration of dissolved particles in plasma"),
    t!("lactate;lactic acid;blood lactate", "mmol/L", (0.5, 2.2), (0.5, 2.2), (9.01, "mg/dL"), "raised by intense exercise, shock and sepsis"),
];

fn find(s: &str) -> Option<&'static Test> {
    let s = s.trim().trim_start_matches("the ");
    // a bare "ph" is the chemistry scale, not blood pH
    if s == "ph" {
        return None;
    }
    let s = re!(r"^(?:normal\s+|serum\s+|blood\s+|plasma\s+)+").replace(s, "").to_string();
    let s = re!(r"\s+(?:levels?|tests?|values?|count)$").replace(&s, "").to_string();
    TESTS.iter().find(|t| {
        t.names.split(';').any(|n| {
            let n = re!(r"^(?:serum|blood) ").replace(n, "");
            let n = re!(r" (?:levels?|count)$").replace(&n, "");
            n == s || format!("{n} count") == s
        })
    })
}

fn range_text(r: (f64, f64), unit: &str) -> String {
    let u = if unit.is_empty() { String::new() } else { format!(" {unit}") };
    if r.1.is_infinite() {
        format!("{} or more{u}", fmt_float(r.0, 4))
    } else if r.0 == 0.0 {
        format!("below {}{u}", fmt_float(r.1 + if r.1.fract() == 0.0 { 1.0 } else { 0.0 }, 4))
    } else {
        format!("{}–{}{u}", fmt_float(r.0, 4), fmt_float(r.1, 4))
    }
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    // "creatinine=0.9mg/dL, adult male", "potassium 5.8 mmol/l"
    if let Some(m) = re!(r"^([a-z][a-z0-9 \-]*?)\s*(?:=|:|\bof\b|\bis\b|\s)\s*(\d+(?:\.\d+)?)\s*(mg/dl|mmol/l|g/dl|g/l|%|u/l|µmol/l|umol/l|ng/ml|ng/dl|pg/ml|mg/l|miu/l)?(?:\s*,?\s*(?:(?:adult|an?|in an?)\s+)?(male|female|man|woman|men|women))?$").captures(l) {
        if let Some(t) = find(&m[1]) {
            let v: f64 = m[2].parse().ok()?;
            let unit = m.get(3).map(|x| x.as_str().replace("umol", "µmol"));
            let sex = m.get(4).map(|x| x.as_str());
            // values in the SI unit are converted back
            let si_unit = t.si.1.to_lowercase();
            let v = match &unit {
                Some(u) if *u == t.unit.to_lowercase() || u.is_empty() => v,
                Some(u) if *u == si_unit || si_unit.starts_with(u.as_str()) => v / t.si.0,
                Some(_) => return None,
                None => v,
            };
            let female = matches!(sex, Some("female" | "woman" | "women"));
            let r = if female { t.female } else { t.male };
            let status = if v < r.0 { "below the reference range" } else if v > r.1 { "above the reference range" } else { "within the reference range" };
            let who = match sex { Some(_) if female => "adult women", Some(_) => "adult men", None if t.male == t.female => "adults", None => "adult men" };
            let disp = t.names.split(';').next().unwrap();
            return Some(Answer::new(format!("{disp} test result"), vec![
                pod("Input", "Input interpretation", text(format!("{disp} = {} {}{}", fmt_float(v, 4), t.unit, sex.map(|_| format!(" ({})", if female { "adult female" } else { "adult male" })).unwrap_or_default()))),
                primary(pod("Result", "Result", text(format!("{status} ({} for {who})", range_text(r, t.unit))))),
                pod("Table", "Values", table(None, vec![
                    vec![text("value"), text(format!("{} {}", fmt_float(v, 4), t.unit))],
                    vec![text("in SI units"), text(format!("{} {}", fmt_float(v * t.si.0, 4), t.si.1))],
                    vec![text("reference range"), text(range_text(r, t.unit))],
                ])),
                pod("Description", "About the test", text(t.what)),
                pod("Note", "Note", text("reference ranges vary between laboratories; results should be interpreted by a clinician")),
            ]));
        }
    }
    let m = re!(r"^(?:(?:what is (?:a |the )?)?(?:normal|reference|healthy|typical)\s+(?:range\s+(?:of|for)\s+)?(.+?)(?:\s+(?:levels?|range|values?))?|(.+?)\s+(?:normal range|reference range|normal levels?|normal values?|range|levels?|blood test|test)|((?:serum|blood|plasma)\s+.+))$").captures(l)?;
    let name = m.get(1).or(m.get(2)).or(m.get(3))?.as_str();
    let t = find(name)?;
    let disp = t.names.split(';').next().unwrap();
    let si = |r: (f64, f64)| range_text((r.0 * t.si.0, if r.1.is_infinite() { r.1 } else { r.1 * t.si.0 }), t.si.1);
    let (result, rows) = if t.male == t.female {
        (range_text(t.male, t.unit), vec![vec![text("adults"), text(range_text(t.male, t.unit)), text(si(t.male))]])
    } else {
        (
            format!("men: {} | women: {}", range_text(t.male, t.unit), range_text(t.female, t.unit)),
            vec![vec![text("adult men"), text(range_text(t.male, t.unit)), text(si(t.male))], vec![text("adult women"), text(range_text(t.female, t.unit)), text(si(t.female))]],
        )
    };
    let si_head = format!("SI ({})", t.si.1);
    Some(Answer::new(format!("{disp} reference range"), vec![
        pod("Input", "Input interpretation", text(format!("{disp} | reference range"))),
        primary(pod("Result", "Reference range", text(result))),
        pod("Table", "Reference ranges", table(Some(vec!["group", "conventional", &si_head]), rows)),
        pod("Description", "About the test", text(t.what)),
        pod("Note", "Note", text("typical adult reference ranges (Mayo Clinic Laboratories and standard clinical references); laboratories differ slightly")),
    ]))
}
