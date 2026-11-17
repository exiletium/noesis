//! Physical chemistry calculations: vapour pressures and boiling points at
//! other pressures (Antoine equation), Raoult's law for ideal mixtures,
//! the Goff–Gratch saturation pressures over water and ice, the phase of
//! water at a temperature and pressure, the Redlich–Kwong equation of state,
//! strong acid–strong base titration curves, saline solutions and seawater,
//! the hydrogen emission spectrum, and heat transfer laws.

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

fn n(v: f64) -> String {
    fmt_float(v, 6)
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

/// Antoine constants (log₁₀ P[mmHg] = A − B/(C + t[°C])), Dean's handbook
/// and NIST, with the validity range in °C.
const ANTOINE: &[(&[&str], f64, f64, f64, f64, f64)] = &[
    (&["water", "h2o"], 8.07131, 1730.63, 233.426, 1.0, 100.0),
    (&["ethanol", "ethyl alcohol"], 8.20417, 1642.89, 230.300, -57.0, 80.0),
    (&["methanol", "methyl alcohol"], 8.08097, 1582.271, 239.726, 15.0, 84.0),
    (&["benzene"], 6.90565, 1211.033, 220.790, 8.0, 103.0),
    (&["toluene"], 6.95464, 1344.800, 219.482, 6.0, 137.0),
    (&["acetone", "propanone"], 7.02447, 1161.0, 224.0, -13.0, 55.0),
    (&["hexane", "n-hexane"], 6.87601, 1171.17, 224.41, -25.0, 92.0),
    (&["heptane", "n-heptane"], 6.89385, 1264.37, 216.636, -2.0, 124.0),
    (&["octane", "n-octane"], 6.91868, 1351.99, 209.155, 19.0, 152.0),
    (&["pentane", "n-pentane"], 6.87632, 1075.78, 233.205, -50.0, 58.0),
    (&["cyclohexane"], 6.84130, 1201.53, 222.65, 20.0, 81.0),
    (&["chloroform", "trichloromethane"], 6.95465, 1170.966, 226.232, -10.0, 60.0),
    (&["carbon tetrachloride", "tetrachloromethane", "ccl4"], 6.87926, 1212.021, 226.41, -14.0, 77.0),
    (&["diethyl ether", "ether", "ethyl ether"], 6.92032, 1064.07, 228.80, -61.0, 20.0),
    (&["acetic acid", "ethanoic acid"], 7.38782, 1533.313, 222.309, 29.8, 126.5),
    (&["isopropanol", "isopropyl alcohol", "2-propanol"], 8.11778, 1580.92, 219.61, 0.0, 101.0),
    (&["ethyl acetate"], 7.10179, 1244.95, 217.88, -20.0, 150.0),
];

fn antoine(name: &str) -> Option<(&'static str, f64, f64, f64, f64, f64)> {
    let l = name.trim().trim_start_matches("the ").to_lowercase();
    ANTOINE.iter().find(|r| r.0.contains(&l.as_str())).map(|r| (r.0[0], r.1, r.2, r.3, r.4, r.5))
}

fn pressure_pa(v: f64, unit: &str) -> Option<f64> {
    Some(v * match unit.to_lowercase().as_str() {
        "torr" | "mmhg" | "mm hg" => 133.322_368,
        "atm" | "atmospheres" | "atmosphere" => 101_325.0,
        "bar" | "bars" => 1e5,
        "mbar" | "millibar" | "hpa" => 100.0,
        "kpa" => 1000.0,
        "mpa" => 1e6,
        "pa" | "pascal" | "pascals" => 1.0,
        "psi" => 6894.757,
        "inhg" | "in hg" => 3386.389,
        _ => return None,
    })
}

const PUNIT: &str = r"(torr|mm ?hg|atm|atmospheres?|bars?|mbar|millibar|hpa|kpa|mpa|pa|pascals?|psi|in ?hg)";

fn temp_c(v: f64, unit: &str) -> Option<f64> {
    let u = unit.to_lowercase().replace('°', "");
    Some(match u.trim() {
        "c" | "celsius" | "degrees celsius" | "degc" => v,
        "f" | "fahrenheit" | "degrees fahrenheit" | "degf" => (v - 32.0) * 5.0 / 9.0,
        "k" | "kelvin" | "kelvins" => v - 273.15,
        _ => return None,
    })
}

const TUNIT: &str = r"(°\s?c|°\s?f|degrees celsius|degrees fahrenheit|celsius|fahrenheit|kelvins?|c|f|k)\b";

fn temp_text(c: f64) -> String {
    format!("{} °C ({} K, {} °F)", n(c), n(c + 273.15), n(c * 9.0 / 5.0 + 32.0))
}

fn vapour(l: &str) -> Option<Answer> {
    // boiling point at a pressure
    let bp = Regex::new(&format!(r"^(?:the )?(?:boiling point|boiling temperature|bp) of (.+?) (?:at|under) (-?[\d.]+)\s*{PUNIT}$")).ok()?;
    if let Some(m) = bp.captures(l) {
        let (name, a, b, c, lo, hi) = antoine(&m[1])?;
        let p = pressure_pa(m[2].parse().ok()?, &m[3])?;
        let mmhg = p / 133.322_368;
        let t = b / (a - mmhg.log10()) - c;
        let normal = b / (a - 760f64.log10()) - c;
        let mut pods = vec![
            input(format!("boiling point of {name} at {} {}", &m[2], &m[3])),
            primary(pod("Result", "Boiling point", text(temp_text(t)))),
            pod("Normal", "Normal boiling point (1 atm)", text(temp_text(normal))),
            pod("Method", "Antoine equation", text(format!("log₁₀(P/mmHg) = {a} − {b}/({c} + t/°C)"))),
        ];
        if t < lo - 5.0 || t > hi + 5.0 {
            pods.push(pod("Note", "Note", text(format!("outside the fitted range {lo} °C to {hi} °C; the value is an extrapolation"))));
        }
        return Some(Answer::new("boiling point", pods));
    }
    let vp = Regex::new(&format!(r"^(?:the )?(?:saturation |equilibrium )?vapou?r pressure of (.+?) at (-?[\d.]+)\s*{TUNIT}$")).ok()?;
    if let Some(m) = vp.captures(l) {
        let (name, a, b, c, lo, hi) = antoine(&m[1])?;
        let t = temp_c(m[2].parse().ok()?, &m[3])?;
        let mmhg = 10f64.powf(a - b / (c + t));
        let pa = mmhg * 133.322_368;
        let rows = vec![
            vec![text("mmHg"), text(n(mmhg))],
            vec![text("kPa"), text(n(pa / 1000.0))],
            vec![text("atm"), text(n(pa / 101_325.0))],
            vec![text("psi"), text(n(pa / 6894.757))],
        ];
        let mut pods = vec![
            input(format!("vapor pressure of {name} at {}", temp_text(t))),
            primary(pod("Result", "Vapor pressure", text(format!("{} kPa ({} mmHg)", n(pa / 1000.0), n(mmhg))))),
            pod("Units", "Unit conversions", table(None, rows)),
            pod("Method", "Antoine equation", text(format!("log₁₀(P/mmHg) = {a} − {b}/({c} + t/°C)"))),
        ];
        if t < lo - 5.0 || t > hi + 5.0 {
            pods.push(pod("Note", "Note", text(format!("outside the fitted range {lo} °C to {hi} °C; the value is an extrapolation"))));
        }
        let xs: Vec<f64> = (0..=60).map(|k| lo + (hi - lo) * k as f64 / 60.0).collect();
        let ys: Vec<f64> = xs.iter().map(|t| 10f64.powf(a - b / (c + t)) * 0.133_322_368).collect();
        pods.push(pod("Plot", "Vapor pressure curve (kPa against °C)", plot(crate::plot::lines(vec![("vapor pressure", xs, ys)], "T (°C)", "P (kPa)"))));
        return Some(Answer::new("vapor pressure", pods));
    }
    // Raoult's law for two liquids
    let m = re!(r"^(?:raoult'?s law|ideal solution|vapou?r pressure of (?:a )?mixture of)(?: for| of)? (.+?)(?:,| and) (.+?)$").captures(l)?;
    let (a1, b1) = (antoine(&m[1])?, antoine(&m[2])?);
    let p = |r: &(&str, f64, f64, f64, f64, f64), t: f64| 10f64.powf(r.1 - r.2 / (r.3 + t)) * 0.133_322_368;
    let (pa, pb) = (p(&a1, 25.0), p(&b1, 25.0));
    let total = 0.5 * pa + 0.5 * pb;
    let ya = 0.5 * pa / total;
    let rows = vec![
        vec![text(format!("P*({})", a1.0)), text(format!("{} kPa", n(pa)))],
        vec![text(format!("P*({})", b1.0)), text(format!("{} kPa", n(pb)))],
        vec![text("total pressure (equimolar liquid)"), text(format!("{} kPa", n(total)))],
        vec![text(format!("vapor mole fraction of {}", a1.0)), text(n(ya))],
    ];
    let xs: Vec<f64> = (0..=20).map(|k| k as f64 / 20.0).collect();
    let tot: Vec<f64> = xs.iter().map(|x| x * pa + (1.0 - x) * pb).collect();
    Some(Answer::new("Raoult's law", vec![
        input(format!("Raoult's law for {} and {}", a1.0, b1.0)),
        primary(pod("Result", "Raoult's law", text(format!("P = x_A P*_A + x_B P*_B; partial pressure p_i = x_i P*_i")))),
        pod("Values", "At 25 °C", table(None, rows)),
        pod("Plot", format!("Total vapor pressure at 25 °C against mole fraction of {} (kPa)", a1.0).as_str(), plot(crate::plot::lines(vec![("total pressure", xs, tot)], "mole fraction", "P (kPa)"))),
    ]))
}

/// Goff–Gratch saturation vapour pressure (hPa) over liquid water.
fn goff_gratch_water(t_k: f64) -> f64 {
    let ts = 373.15;
    let x = -7.90298 * (ts / t_k - 1.0) + 5.02808 * (ts / t_k).log10() - 1.3816e-7 * (10f64.powf(11.344 * (1.0 - t_k / ts)) - 1.0) + 8.1328e-3 * (10f64.powf(-3.49149 * (ts / t_k - 1.0)) - 1.0) + 1013.246f64.log10();
    10f64.powf(x)
}

/// Goff–Gratch saturation vapour pressure (hPa) over ice.
fn goff_gratch_ice(t_k: f64) -> f64 {
    let t0 = 273.16;
    let x = -9.09718 * (t0 / t_k - 1.0) - 3.56654 * (t0 / t_k).log10() + 0.876793 * (1.0 - t_k / t0) + 6.1071f64.log10();
    10f64.powf(x)
}

/// "saturation vapor pressure 30C", "saturation vapor pressure over ice at -10 C"
fn saturation(l: &str) -> Option<Answer> {
    let r = Regex::new(&format!(r"^(?:the )?saturation vapou?r pressure(?: of water| over (water|ice))?(?: at)? (-?[\d.]+)\s*{TUNIT}$")).ok()?;
    let m = r.captures(l)?;
    let t = temp_c(m[2].parse().ok()?, &m[3])?;
    let tk = t + 273.15;
    if !(173.0..=473.0).contains(&tk) {
        return None;
    }
    let over_ice = m.get(1).is_some_and(|x| x.as_str() == "ice");
    let e = if over_ice { goff_gratch_ice(tk) } else { goff_gratch_water(tk) };
    let over = if over_ice { "ice" } else { "liquid water" };
    let mut pods = vec![
        input(format!("saturation vapor pressure over {over} at {}", temp_text(t))),
        primary(pod("Result", "Saturation vapor pressure", text(format!("{} hPa = {} kPa ({} mmHg, {} psi)", n(e), n(e / 10.0), n(e * 100.0 / 133.322_368), n(e * 100.0 / 6894.757))))),
    ];
    if t < 0.0 && !over_ice {
        pods.push(pod("Ice", "Over ice at the same temperature", text(format!("{} hPa (supercooled water has the higher vapor pressure)", n(goff_gratch_ice(tk))))));
    }
    pods.push(pod("Method", "Method", text("Goff–Gratch equation (World Meteorological Organization reference formulation until 2008)")));
    Some(Answer::new("saturation vapor pressure", pods))
}

fn goff(l: &str) -> Option<Answer> {
    if !l.contains("goff") {
        return None;
    }
    let ice = l.contains("ice");
    let formula = if ice {
        "log₁₀ e_i = −9.09718 (T₀/T − 1) − 3.56654 log₁₀(T₀/T) + 0.876793 (1 − T/T₀) + log₁₀ 6.1071, T₀ = 273.16 K, e_i in hPa"
    } else {
        "log₁₀ e_w = −7.90298 (T_st/T − 1) + 5.02808 log₁₀(T_st/T) − 1.3816×10⁻⁷ (10^(11.344 (1 − T/T_st)) − 1) + 8.1328×10⁻³ (10^(−3.49149 (T_st/T − 1)) − 1) + log₁₀ 1013.246, T_st = 373.15 K, e_w in hPa"
    };
    let mut pods = vec![input(format!("Goff–Gratch equation for {}", if ice { "ice" } else { "water" })), primary(pod("Result", "Saturation vapor pressure", text(formula)))];
    if let Some(m) = Regex::new(&format!(r"at (-?[\d.]+)\s*{TUNIT}")).ok()?.captures(l) {
        let t = temp_c(m[1].parse().ok()?, &m[2])? + 273.15;
        let e = if ice { goff_gratch_ice(t) } else { goff_gratch_water(t) };
        pods[1].primary = false;
        pods.insert(1, primary(pod("Value", format!("Saturation vapor pressure at {}", temp_text(t - 273.15)).as_str(), text(format!("{} hPa = {} kPa = {} mmHg", n(e), n(e / 10.0), n(e * 100.0 / 133.322_368))))));
    }
    pods.push(pod("Source", "Source", text("Goff and Gratch (1946), the World Meteorological Organization reference formulation until 2008")));
    Some(Answer::new("Goff–Gratch equation", pods))
}

/// Phase of water at a temperature and pressure (below 200 MPa).
fn water_phase(l: &str) -> Option<Answer> {
    let r = Regex::new(&format!(r"^(?:phase of )?(water|ice|steam|h2o) at (-?[\d.]+)\s*{TUNIT},?\s*(?:and )?([\d.]+)\s*{PUNIT}$")).ok()?;
    let m = r.captures(l)?;
    let t = temp_c(m[2].parse().ok()?, &m[3])?;
    let p = pressure_pa(m[4].parse().ok()?, &m[5])?;
    let tk = t + 273.15;
    // ice Ih melting curve (Clapeyron slope ≈ −7.4 mK per atm) and saturation lines
    let t_melt = 273.15 - 7.4e-3 * (p / 101_325.0 - 1.0);
    let (phase, note) = if p > 2.0e8 {
        return None;
    } else if tk < t_melt {
        if p < goff_gratch_ice(tk) * 100.0 { ("gas (water vapor)", "below the sublimation pressure of ice") } else { ("solid (ice Ih)", "below the melting curve of ice Ih") }
    } else if tk >= 647.096 && p >= 22.064e6 {
        ("supercritical fluid", "above the critical point (647.096 K, 22.064 MPa)")
    } else if tk < 647.096 && p >= water_psat(tk) {
        ("liquid", "above the saturation pressure at this temperature")
    } else {
        ("gas (water vapor)", "below the saturation pressure at this temperature")
    };
    let mut rows = vec![vec![text("temperature"), text(temp_text(t))], vec![text("pressure"), text(format!("{} kPa ({} atm)", n(p / 1000.0), n(p / 101_325.0)))], vec![text("melting point at this pressure"), text(temp_text(t_melt - 273.15))]];
    if phase.starts_with("solid") {
        rows.push(vec![text("density of ice Ih"), text("≈ 0.917 g/cm³")]);
        rows.push(vec![text("sublimation pressure"), text(format!("{} Pa", n(goff_gratch_ice(tk) * 100.0)))]);
    }
    Some(Answer::new("phase of water", vec![input(format!("water at {} and {} kPa", temp_text(t), n(p / 1000.0))), primary(pod("Result", "Phase", text(phase))), pod("Reason", "Reason", text(note)), pod("Conditions", "Conditions", table(None, rows))]))
}

/// Saturation pressure of water (Pa), Wagner–Pruss style fit (IAPWS-IF97 region 4).
fn water_psat(t: f64) -> f64 {
    let n = [0.116_705_214_527_67e4, -0.724_213_167_032_06e6, -0.170_738_469_400_92e2, 0.120_208_247_024_70e5, -0.323_255_503_223_33e7, 0.149_151_086_135_30e2, -0.482_326_573_615_91e4, 0.405_113_405_420_57e6, -0.238_555_575_678_49, 0.650_175_348_447_98e3];
    let th = t + n[8] / (t - n[9]);
    let a = th * th + n[0] * th + n[1];
    let b = n[2] * th * th + n[3] * th + n[4];
    let c = n[5] * th * th + n[6] * th + n[7];
    (2.0 * c / (-b + (b * b - 4.0 * a * c).sqrt())).powi(4) * 1e6
}

fn titration(l: &str) -> Option<Answer> {
    if !l.contains("titrat") {
        return None;
    }
    let num = r"([\d.]+)";
    let full = Regex::new(&format!(r"titration of {num}\s*ml of {num}\s*m (?:of )?(?:a )?strong acid with {num}\s*m (?:of )?(?:a )?strong base")).ok()?;
    let (va, ca, cb, generic) = match full.captures(l) {
        Some(m) => (m[1].parse::<f64>().ok()?, m[2].parse::<f64>().ok()?, m[3].parse::<f64>().ok()?, false),
        None if re!(r"^(?:strong acid(?:[- ]strong base)? titration|titration (?:of a )?strong acid(?: with (?:a )?strong base)?|strong acid titration curve)$").is_match(l) => (25.0, 0.1, 0.1, true),
        None => return None,
    };
    let veq = va * ca / cb;
    let ph = |vb: f64| -> f64 {
        let (na, nb) = (va * ca, vb * cb);
        let v = va + vb;
        let excess = (na - nb) / v;
        // include water autoionisation: [H+] − [OH−] = excess
        let h = 0.5 * (excess + (excess * excess + 4e-14).sqrt());
        -h.log10()
    };
    let xs: Vec<f64> = (0..=200).map(|k| 2.0 * veq * k as f64 / 200.0).collect();
    let ys: Vec<f64> = xs.iter().map(|v| ph(*v)).collect();
    let rows = vec![
        vec![text("equivalence volume of base"), text(format!("{} mL", n(veq)))],
        vec![text("initial pH"), text(n(ph(0.0)))],
        vec![text("pH at half equivalence"), text(n(ph(veq / 2.0)))],
        vec![text("pH at equivalence"), text("7.00 (25 °C)")],
        vec![text("pH after 10% excess base"), text(n(ph(1.1 * veq)))],
        vec![text("moles of acid"), text(format!("{} mmol", n(va * ca)))],
    ];
    let label = format!("titration of {} mL of {} M strong acid with {} M strong base", n(va), n(ca), n(cb));
    let mut pods = vec![
        input(label),
        primary(pod("Result", "Equivalence point", text(format!("{} mL of base, pH 7", n(veq))))),
        pod("Details", "Titration details", table(None, rows)),
        pod("Plot", "Titration curve (pH against mL of base added)", plot(crate::plot::lines(vec![("pH", xs, ys)], "base added (mL)", "pH"))),
        pod("Formula", "Equivalence condition", text("C_acid V_acid = C_base V_base")),
    ];
    if generic {
        pods.push(pod("Note", "Note", text("example values; ask e.g. \"titration of 50 mL of 2M strong acid with 3M strong base\"")));
    }
    Some(Answer::new("titration", pods))
}

fn solutions(l: &str) -> Option<Answer> {
    let t = l.trim_start_matches("the ").trim_start_matches("what is ").trim();
    let saline: Option<(&str, f64)> = match t {
        "normal saline" | "physiological saline" | "0.9% saline" | "isotonic saline" => Some(("normal saline", 0.9)),
        "half-normal saline" | "half normal saline" | "0.45% saline" => Some(("half-normal saline", 0.45)),
        "quarter-normal saline" | "quarter normal saline" | "0.225% saline" => Some(("quarter-normal saline", 0.225)),
        "hypertonic saline" | "3% saline" => Some(("hypertonic saline", 3.0)),
        _ => None,
    };
    if let Some((name, pct)) = saline {
        let gl = pct * 10.0;
        let mm = gl / 58.443 * 1000.0;
        let rows = vec![
            vec![text("NaCl concentration"), text(format!("{} % w/v ({} g/L)", n(pct), n(gl)))],
            vec![text("molarity"), text(format!("{} mmol/L", n(mm)))],
            vec![text("Na⁺ and Cl⁻"), text(format!("{} mEq/L each", n(mm)))],
            vec![text("osmolarity (ideal)"), text(format!("{} mOsm/L", n(2.0 * mm)))],
            vec![text("tonicity"), text(if pct < 0.85 { "hypotonic" } else if pct > 0.95 { "hypertonic" } else { "isotonic" })],
        ];
        return Some(Answer::new(name, vec![input(name), primary(pod("Result", "Composition", text(format!("{} % sodium chloride in water ({} mM NaCl)", n(pct), n(mm))))), pod("Properties", "Properties", table(None, rows))]));
    }
    let sea = re!(r"^(?:sea ?water|salt ?water|ocean water|brackish water)(?: (density|conductivity|salinity|freezing point|boiling point|ph|speed of sound|properties))?$|^(density|conductivity|salinity|freezing point|boiling point|ph|speed of sound) of (?:sea ?water|salt ?water|ocean water|brackish water)$").captures(t)?;
    let what = sea.get(1).or(sea.get(2)).map(|m| m.as_str()).unwrap_or("properties");
    if t.contains("brackish") {
        let rows = vec![vec![text("salinity"), text("0.5 to 30 g/kg (between fresh water and seawater)")], vec![text("electrical conductivity"), text("≈ 1 to 45 mS/cm (0.1 to 4.5 S/m), rising with salinity")], vec![text("density"), text("≈ 1000 to 1023 kg/m³")]];
        let main = match what {
            "conductivity" => "≈ 1 to 45 mS/cm, depending on salinity",
            "salinity" => "0.5 to 30 g/kg",
            "density" => "≈ 1000 to 1023 kg/m³",
            _ => "water with 0.5 to 30 g/kg dissolved salts",
        };
        return Some(Answer::new("brackish water", vec![input("brackish water"), primary(pod("Result", "Result", text(main))), pod("Properties", "Typical properties", table(None, rows))]));
    }
    let props: &[(&str, &str)] = &[
        ("salinity", "35 g/kg (35 ‰, practical salinity 35)"),
        ("density", "1025 kg/m³ at the surface (1023.3 kg/m³ at 25 °C, 1028.1 kg/m³ at 0 °C, salinity 35)"),
        ("freezing point", "−1.9 °C (salinity 35, 1 atm)"),
        ("boiling point", "≈ 100.6 °C at 1 atm"),
        ("ph", "≈ 8.1"),
        ("conductivity", "≈ 5.3 S/m at 25 °C (4.3 S/m at 15 °C), salinity 35"),
        ("speed of sound", "≈ 1500 m/s (1531 m/s at 25 °C, 1449 m/s at 0 °C, salinity 35)"),
        ("specific heat", "≈ 3.99 kJ/(kg K)"),
        ("main ions", "Cl⁻ 19.4 g/kg, Na⁺ 10.8 g/kg, SO₄²⁻ 2.7 g/kg, Mg²⁺ 1.29 g/kg, Ca²⁺ 0.41 g/kg, K⁺ 0.40 g/kg"),
    ];
    let main = props.iter().find(|(k, _)| *k == what).map(|(_, v)| *v).unwrap_or("water with about 35 g/kg dissolved salts, mostly sodium chloride");
    let rows = props.iter().map(|(k, v)| vec![text(*k), text(*v)]).collect();
    Some(Answer::new("seawater", vec![input(format!("seawater {what}")), primary(pod("Result", "Result", text(main))), pod("Properties", "Typical properties of seawater", table(None, rows))]))
}

/// Strong lines (nm, in air) of neutral atoms in or near the visible range,
/// from the NIST Atomic Spectra Database persistent lines, with the colour
/// of a discharge tube or flame.
const SPECTRA: &[(&str, &str, &str, &str)] = &[
    ("He", "helium", "388.9, 447.1, 471.3, 492.2, 501.6, 587.6 (D₃), 667.8, 706.5", "pinkish yellow (discharge tube)"),
    ("Li", "lithium", "460.3, 610.4, 670.8", "crimson (flame)"),
    ("Na", "sodium", "568.8, 589.0 and 589.6 (the D lines), 615.4, 818.3, 819.5", "intense yellow (flame)"),
    ("K", "potassium", "404.4, 404.7, 691.1, 693.9, 766.5, 769.9", "lilac (flame)"),
    ("Rb", "rubidium", "420.2, 421.6, 780.0, 794.8", "red-violet (flame)"),
    ("Cs", "cesium", "455.5, 459.3, 852.1, 894.3", "blue-violet (flame)"),
    ("Ca", "calcium", "422.7 (Ca I); 393.4 and 396.8 (Ca II, the H and K lines)", "orange-red (flame)"),
    ("Sr", "strontium", "407.8, 421.6 (Sr II); 460.7 (Sr I)", "crimson red (flame)"),
    ("Ba", "barium", "455.4, 493.4 (Ba II); 553.5 (Ba I)", "pale green (flame)"),
    ("Cu", "copper", "324.8, 327.4, 510.6, 515.3, 521.8", "blue-green (flame)"),
    ("Zn", "zinc", "468.0, 472.2, 481.1, 636.2", "bluish white"),
    ("Cd", "cadmium", "467.8, 480.0, 508.6, 643.8", "red and blue lines"),
    ("Hg", "mercury", "404.7, 435.8, 546.1, 577.0, 579.1", "blue-violet (discharge tube)"),
    ("Ne", "neon", "540.1, 585.2, 588.2, 594.5, 609.6, 614.3, 626.6, 633.4, 640.2, 650.7, 659.9, 692.9, 703.2", "red-orange (discharge tube)"),
    ("Ar", "argon", "696.5, 706.7, 750.4, 763.5, 811.5, 912.3; Ar II 488.0 and 514.5", "violet-lavender (discharge tube)"),
    ("Kr", "krypton", "557.0, 587.1, 760.2, 811.3", "greyish white to lavender (discharge tube)"),
    ("Xe", "xenon", "462.4, 467.1, 823.2, 881.9", "blue-violet (discharge tube)"),
    ("N", "nitrogen", "742.4, 744.2, 746.8, 818.5, 821.6, 868.0 (N I); N II 463.0, 500.5, 567.9", "pink-violet (discharge tube, mostly N₂ molecular bands)"),
    ("O", "oxygen", "777.2 (triplet), 844.6; forbidden lines 557.7 (green aurora) and 630.0 (red aurora)", "pale violet (discharge tube)"),
];

/// "Ne emission spectrum", "emission spectrum of potassium", "atomic spectrum of nitrogen"
fn element_spectrum(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?(.+?) (?:emission|atomic|line|visible|spectral)(?: line)? spectrum$|^(?:the )?(?:emission|atomic|line|visible) spectrum (?:of|for) (.+)$|^(?:the )?spectral lines (?:of|for) (.+)$|^(.+?) spectral lines$").captures(l)?;
    let name = m.get(1).or(m.get(2)).or(m.get(3)).or(m.get(4))?.as_str().trim();
    let name = name.trim_end_matches(" gas").trim_end_matches(" atom");
    let key = if name.len() <= 2 {
        let mut c = name.chars();
        c.next()?.to_uppercase().collect::<String>() + c.as_str()
    } else {
        name.replace("caesium", "cesium")
    };
    let (sym, el, lines, colour) = SPECTRA.iter().find(|(s, n, _, _)| *s == key || *n == key)?;
    let wl: Vec<f64> = re!(r"\d{3}\.\d").find_iter(lines).filter_map(|x| x.as_str().parse().ok()).collect();
    let band = |w: f64| match w {
        w if w < 380.0 => "ultraviolet",
        w if w < 450.0 => "violet",
        w if w < 495.0 => "blue",
        w if w < 570.0 => "green",
        w if w < 590.0 => "yellow",
        w if w < 620.0 => "orange",
        w if w < 750.0 => "red",
        _ => "infrared",
    };
    let rows: Vec<Vec<Content>> = wl.iter().map(|w| vec![text(format!("{w} nm")), text(band(*w)), text(format!("{} eV", n(1239.841_98 / w)))]).collect();
    Some(Answer::new(format!("{el} emission spectrum"), vec![
        input(format!("{} ({sym}) | emission spectrum", { let mut c = el.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() })),
        primary(pod("Result", "Strong lines (wavelength in air)", text(format!("{lines} nm")))),
        pod("Lines", "Lines", table(Some(vec!["wavelength", "colour", "photon energy"]), rows)),
        pod("Colour", "Observed colour", text(*colour)),
        pod("Source", "Source", text("NIST Atomic Spectra Database (persistent lines)")),
    ]))
}

fn hydrogen_spectrum(l: &str) -> Option<Answer> {
    if !re!(r"^(?:the )?(?:hydrogen|h) (?:emission|atomic|line) spectrum$|^(?:the )?(?:emission|line|atomic) spectrum of hydrogen$|^spectral lines of hydrogen$").is_match(l) {
        return None;
    }
    let r = 1.096_775_83e7; // m⁻¹, hydrogen (reduced mass)
    let series = [("Lyman", 1u32, "ultraviolet"), ("Balmer", 2, "visible"), ("Paschen", 3, "infrared"), ("Brackett", 4, "infrared"), ("Pfund", 5, "infrared")];
    let mut rows = Vec::new();
    for (name, lo, region) in series {
        for k in 1..=4 {
            let hi = lo + k;
            let inv = r * (1.0 / (lo * lo) as f64 - 1.0 / (hi * hi) as f64);
            rows.push(vec![text(name), text(format!("{hi} → {lo}")), text(format!("{} nm", fmt_float(1e9 / inv, 6))), text(region)]);
        }
    }
    let visible = "656.47 nm (red, Hα), 486.27 nm (cyan, Hβ), 434.17 nm (violet, Hγ), 410.29 nm (violet, Hδ) — vacuum wavelengths; in air 656.28, 486.13, 434.05, 410.17 nm";
    Some(Answer::new("hydrogen spectrum", vec![
        input("hydrogen emission spectrum"),
        primary(pod("Result", "Visible lines (Balmer series)", text(visible))),
        pod("Lines", "Lines by series (vacuum wavelengths)", table(Some(vec!["series", "transition", "wavelength", "region"]), rows)),
        pod("Formula", "Rydberg formula", text("1/λ = R_H (1/n₁² − 1/n₂²), R_H = 1.09678 × 10⁷ m⁻¹")),
    ]))
}

fn formulas(l: &str) -> Option<Answer> {
    let t = l.trim_start_matches("the ").trim_end_matches(" calculator").trim();
    let (title, main, rows): (&str, &str, Vec<(&str, &str)>) = match t {
        "redlich kwong" | "redlich-kwong" | "redlich kwong equation" | "redlich-kwong equation" | "redlich kwong equation of state" | "redlich-kwong equation of state" => (
            "Redlich–Kwong equation of state",
            "P = R T/(V_m − b) − a/(√T V_m (V_m + b))",
            vec![("a", "0.42748 R² T_c^(5/2)/P_c"), ("b", "0.08664 R T_c/P_c"), ("R", "8.314462618 J/(mol K)"), ("T_c, P_c", "critical temperature and pressure of the gas"), ("compressibility at the critical point", "Z_c = 1/3")],
        ),
        "van der waals equation" | "van der waals equation of state" => ("van der Waals equation of state", "(P + a n²/V²)(V − n b) = n R T", vec![("a", "27 R² T_c²/(64 P_c)"), ("b", "R T_c/(8 P_c)")]),
        "heat transfer equation" | "heat transfer equations" | "heat transfer" | "heat transfer formula" => (
            "heat transfer",
            "Q = m c ΔT (heat to change the temperature of a mass m)",
            vec![("conduction (Fourier's law)", "q = −k ∇T; through a slab: Q/t = k A ΔT/d"), ("convection (Newton's law of cooling)", "Q/t = h A (T_s − T_∞)"), ("radiation (Stefan–Boltzmann)", "Q/t = ε σ A (T⁴ − T_surr⁴), σ = 5.670374 × 10⁻⁸ W/(m² K⁴)"), ("heat equation", "∂T/∂t = α ∇²T, α = k/(ρ c_p)"), ("thermal resistance", "R = d/(k A); Q/t = ΔT/R")],
        ),
        "clausius-clapeyron equation" | "clausius clapeyron equation" | "clausius-clapeyron" | "clausius clapeyron" => ("Clausius–Clapeyron equation", "ln(P₂/P₁) = −(ΔH_vap/R) (1/T₂ − 1/T₁)", vec![("differential form", "dP/dT = L/(T Δv)"), ("assumptions", "ideal vapor, liquid volume negligible, ΔH_vap constant")]),
        "antoine equation" => ("Antoine equation", "log₁₀ P = A − B/(C + T)", vec![("A, B, C", "fitted constants for each substance and temperature range")]),
        _ => return None,
    };
    let mut pods = vec![input(title), primary(pod("Result", "Formula", text(main)))];
    pods.push(pod("Details", "Details", table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())));
    Some(Answer::new(title, pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).replace('’', "'");
    let l = re!(r"^(?:what is |what are |find |calculate |compute )").replace(&l, "").to_string();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    vapour(&l).or_else(|| saturation(&l)).or_else(|| goff(&l)).or_else(|| water_phase(&l)).or_else(|| titration(&l)).or_else(|| solutions(&l)).or_else(|| hydrogen_spectrum(&l)).or_else(|| element_spectrum(&l)).or_else(|| formulas(&l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn antoine_normal_boiling_points() {
        // each fit reproduces the normal boiling point within 0.5 °C
        for (name, bp) in [("water", 100.0), ("ethanol", 78.37), ("benzene", 80.1), ("toluene", 110.6), ("acetone", 56.05), ("hexane", 68.7), ("chloroform", 61.2), ("acetic acid", 118.1), ("methanol", 64.7), ("diethyl ether", 34.6)] {
            let (_, a, b, c, _, _) = antoine(name).unwrap();
            let t = b / (a - 760f64.log10()) - c;
            assert!((t - bp).abs() < 0.5, "{name}: {t}");
        }
        assert!((goff_gratch_water(283.15) - 12.27).abs() < 0.05);
        assert!((water_psat(373.15) - 101_418.0).abs() < 50.0);
    }
}
