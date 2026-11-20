//! Liquids, gases and fuels: viscosity, surface tension, flash point,
//! autoignition temperature, flammability limits, heat of combustion and
//! heating values, heat of vaporization, speed of sound, refractive index,
//! dielectric constant and ACGIH threshold limit values, for single
//! substances and comparisons ("flash point methane, butane, octane").
//!
//! Values are standard handbook figures (CRC Handbook, NIST WebBook,
//! ACGIH); a value written "v@T" was measured at T °C, otherwise at 25 °C.

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
    fmt_float(v, 5)
}

/// names | formula | molar mass (g/mol) | properties
///
/// visc: dynamic viscosity, mPa·s · st: surface tension, mN/m (dst: its
/// slope, mN/(m·K)) · fp: flash point (closed cup), °C · ait: autoignition
/// temperature, °C · fl: flammability limits, % by volume in air · hc, lhv:
/// heat of combustion (higher and lower heating value), kJ/mol · hcm:
/// heating value, MJ/kg · hvap: enthalpy of vaporization at the normal
/// boiling point, kJ/mol · sos: speed of sound, m/s · n: refractive index
/// (589 nm, 20 °C) · eps: dielectric constant · tlv: ACGIH TLV-TWA.
const DATA: &str = "\
water,h2o|H2O|18.015|visc=0.890;st=71.97;dst=-0.157;sos=1497;n=1.3330;eps=80.1@20;hvap=40.65
methanol,methyl alcohol,ch3oh|CH3OH|32.042|visc=0.544;st=22.07;dst=-0.0773;sos=1103;n=1.3288;eps=32.7;fp=11;ait=464;fl=6.0-36;hc=726.1;lhv=638.1;hvap=35.21;tlv=200 ppm
ethanol,ethyl alcohol,alcohol,c2h5oh|C2H5OH|46.069|visc=1.074;st=21.97;dst=-0.0832;sos=1143;n=1.3611;eps=24.5;fp=14;ait=363;fl=3.3-19;hc=1366.8;lhv=1234.8;hvap=38.56;tlv=1000 ppm (STEL)
acetone,propanone|C3H6O|58.080|visc=0.306;st=23.46;dst=-0.112;sos=1174;n=1.3588;eps=20.7;fp=-20;ait=465;fl=2.5-12.8;hc=1789.9;hvap=29.10;tlv=250 ppm
benzene,c6h6|C6H6|78.114|visc=0.604;st=28.22;dst=-0.1291;sos=1295;n=1.5011;eps=2.27;fp=-11;ait=498;fl=1.2-7.8;hc=3267.6;hvap=30.72;tlv=0.5 ppm
toluene,methylbenzene|C7H8|92.141|visc=0.560;st=27.93;dst=-0.1189;sos=1306;n=1.4961;eps=2.38;fp=6;ait=480;fl=1.1-7.1;hc=3910.3;hvap=33.18;tlv=20 ppm
glycerol,glycerin,glycerine|C3H8O3|92.094|visc=934;st=63.0@20;sos=1904;n=1.4746;eps=42.5;fp=160;ait=370;hc=1654.3
hexane,n-hexane,c6h14|C6H14|86.178|visc=0.300;st=17.89;dst=-0.1022;sos=1078;n=1.3749;eps=1.89;fp=-26;ait=234;fl=1.1-7.5;hc=4163.2;hvap=28.85;tlv=50 ppm
pentane,n-pentane,c5h12|C5H12|72.151|visc=0.224;st=15.49;dst=-0.110;n=1.3575;eps=1.84;fp=-49;ait=260;fl=1.5-7.8;hc=3509.0;hvap=25.79;tlv=1000 ppm
heptane,n-heptane,c7h16|C7H16|100.205|visc=0.387;st=19.66;dst=-0.0980;sos=1131;n=1.3876;eps=1.92;fp=-4;ait=223;fl=1.05-6.7;hc=4817.0;hvap=31.77;tlv=400 ppm
octane,n-octane,c8h18|C8H18|114.232|visc=0.508;st=21.14;dst=-0.0951;sos=1171;n=1.3974;eps=1.95;fp=13;ait=206;fl=1.0-6.5;hc=5470.0;lhv=5074.0;hvap=34.41;tlv=300 ppm
hexadecane,n-hexadecane,cetane,c16h34|C16H34|226.446|visc=3.03;st=27.05;dst=-0.0854;n=1.4345;fp=135;ait=202;hc=10699.1
chloroform,trichloromethane,chcl3|CHCl3|119.377|visc=0.537;st=26.67;dst=-0.1295;sos=987;n=1.4459;eps=4.81;hvap=29.24;tlv=10 ppm;note=not flammable
diethyl ether,ether,ethyl ether|C4H10O|74.122|visc=0.224;st=16.65;dst=-0.1159;sos=985;n=1.3526;eps=4.33;fp=-45;ait=160;fl=1.9-36;hc=2723.9;hvap=26.17;tlv=400 ppm
acetic acid,ethanoic acid,glacial acetic acid|CH3COOH|60.052|visc=1.056;st=27.10;n=1.3720;eps=6.2;fp=39;ait=427;fl=4.0-16;hc=874.2;hvap=23.70;tlv=10 ppm
carbon tetrachloride,tetrachloromethane,ccl4|CCl4|153.823|visc=0.908;st=26.43;dst=-0.1224;sos=926;n=1.4601;eps=2.24;hvap=29.82;tlv=5 ppm;note=not flammable
ethylene glycol,ethane-1,2-diol,antifreeze|C2H6O2|62.068|visc=16.1;st=47.7;sos=1658;n=1.4318;eps=37.7;fp=111;ait=410;fl=3.2-15.3;hc=1189.2;tlv=25 ppm (vapour fraction)
isopropanol,isopropyl alcohol,2-propanol,rubbing alcohol|C3H8O|60.096|visc=2.04;st=20.93;n=1.3776;eps=20.18@20;fp=12;ait=399;fl=2.0-12.7;hc=2005.8;hvap=39.85;tlv=200 ppm
cyclohexane,c6h12|C6H12|84.162|visc=0.894;st=24.65;dst=-0.1211;sos=1255;n=1.4262;eps=2.02;fp=-20;ait=245;fl=1.3-8.0;hc=3919.6;hvap=29.97;tlv=100 ppm
dichloromethane,methylene chloride,ch2cl2|CH2Cl2|84.930|visc=0.413;st=27.20;n=1.4242;eps=8.93;ait=556;hvap=28.06;tlv=50 ppm
dimethyl sulfoxide,dmso|C2H6OS|78.13|visc=1.987;st=42.92;n=1.4793;eps=46.7;fp=89;ait=215
mercury|Hg|200.59|visc=1.526;st=485.5;sos=1450;hvap=59.11;tlv=0.025 mg/m³
methane,ch4|CH4|16.043|gas=1;visc=0.0111;fp=-188;ait=537;fl=5.0-15;hc=890.8;lhv=802.3;hvap=8.19;tlv=1000 ppm
ethane,c2h6|C2H6|30.069|gas=1;visc=0.0094;fp=-135;ait=472;fl=3.0-12.4;hc=1560.7;lhv=1428.6;hvap=14.69;tlv=1000 ppm
propane,c3h8|C3H8|44.096|gas=1;visc=0.0082;fp=-104;ait=470;fl=2.1-9.5;hc=2219.2;lhv=2043.1;hvap=19.04;tlv=1000 ppm
butane,n-butane,c4h10|C4H10|58.122|gas=1;visc=0.0074;fp=-60;ait=405;fl=1.8-8.4;hc=2877.5;lhv=2657.3;hvap=22.44;tlv=1000 ppm (STEL)
hydrogen,hydrogen gas,h2|H2|2.016|gas=1;visc=0.00889;ait=500;fl=4.0-75;hc=285.8;lhv=241.8;hvap=0.904
carbon monoxide,co|CO|28.010|gas=1;visc=0.0177;ait=609;fl=12.5-74;hc=283.0;tlv=25 ppm
ammonia,nh3|NH3|17.031|gas=1;visc=0.0101;ait=651;fl=15-28;hc=382.8;lhv=316.8;hvap=23.35;tlv=25 ppm
air||28.96|gas=1;visc=0.0185;sos=346;n=1.000293;eps=1.00059
carbon dioxide,co2|CO2|44.009|gas=1;visc=0.0149;sos=267;tlv=5000 ppm;note=not flammable
nitrogen,nitrogen gas,n2|N2|28.014|gas=1;visc=0.0178;sos=353
oxygen,oxygen gas,o2|O2|31.998|gas=1;visc=0.0206;sos=330
helium,helium gas|He|4.0026|gas=1;visc=0.0198;sos=1007
hydrogen sulfide,h2s|H2S|34.08|gas=1;ait=232;fl=4.0-46;hc=562.0;tlv=1 ppm
gasoline,petrol|||visc=0.6;fp=-43;ait=280;fl=1.4-7.6;hcm=46.4;n=1.44
diesel,diesel fuel|||visc=2.7@40;fp=52;ait=210;fl=0.6-7.5;hcm=45.6
kerosene,jet fuel,jet a|||visc=1.6;fp=38;ait=210;fl=0.7-5;hcm=46.2
natural gas|||gas=1;ait=580;fl=5-15;hcm=52.2
olive oil|||visc=84@20;st=32@20;n=1.467;eps=3.1;fp=225
castor oil|||visc=985@20;n=1.479;eps=4.7;fp=229
honey|||visc=10000@20;n=1.49
anthracite,anthracite coal,hard coal|||hcm=30
bituminous coal|||hcm=27
lignite,brown coal|||hcm=15
coal|||hcm=24
wood,firewood,dry wood|||hcm=18
diamond,natural diamond||12.011|n=2.417;eps=5.7
ice||18.015|n=1.309
quartz||60.08|n=1.544
glass,soda-lime glass,window glass|||n=1.52;eps=7
crown glass|||n=1.52
flint glass|||n=1.62
sapphire,ruby,corundum|||n=1.77
cubic zirconia|||n=2.15
moissanite|||n=2.65
acrylic,pmma,plexiglass|||n=1.49
polycarbonate|||n=1.586
sodium chloride,salt,table salt,halite|NaCl|58.44|n=1.544;eps=5.9
";

struct Fluid {
    names: Vec<&'static str>,
    formula: &'static str,
    molar: Option<f64>,
    gas: bool,
    props: Vec<(&'static str, &'static str)>,
}

fn data() -> &'static [Fluid] {
    static D: OnceLock<Vec<Fluid>> = OnceLock::new();
    D.get_or_init(|| {
        DATA.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let mut p = l.split('|');
                let names: Vec<&str> = p.next()?.split(',').map(str::trim).collect();
                let formula = p.next()?.trim();
                let molar = p.next()?.trim().parse().ok();
                let props: Vec<(&str, &str)> = p.next()?.split(';').filter_map(|kv| kv.split_once('=')).collect();
                let gas = props.iter().any(|(k, _)| *k == "gas");
                Some(Fluid { names, formula, molar, gas, props })
            })
            .collect()
    })
}

fn find(name: &str) -> Option<&'static Fluid> {
    let l = name.trim().trim_start_matches("the ").trim_start_matches("pure ").trim_start_matches("liquid ").trim();
    let l = l.trim_end_matches(" liquid").trim_end_matches(" fuel oil").trim();
    data().iter().find(|f| f.names.iter().any(|n| *n == l) || (!f.formula.is_empty() && f.formula.len() > 1 && f.formula.to_lowercase() == l))
}

impl Fluid {
    fn get(&self, k: &str) -> Option<&'static str> {
        self.props.iter().find(|(a, _)| *a == k).map(|(_, v)| *v)
    }
    /// A value and its reference temperature (25 °C unless written "v@T").
    fn num(&self, k: &str) -> Option<(f64, f64)> {
        let v = self.get(k)?;
        match v.split_once('@') {
            Some((a, t)) => Some((a.parse().ok()?, t.parse().ok()?)),
            None => Some((v.parse().ok()?, 25.0)),
        }
    }
    fn title(&self) -> String {
        let t = cap(self.names[0]);
        if self.formula.is_empty() {
            t
        } else {
            format!("{t} ({})", super::chemcalc::pretty(self.formula))
        }
    }
}

fn degf(c: f64) -> String {
    n(c * 9.0 / 5.0 + 32.0)
}

fn at(t: f64) -> String {
    format!("at {} °C", n(t))
}

/// The property a phrase names.
fn prop_key(p: &str) -> Option<&'static str> {
    Some(match p.trim() {
        "viscosity" | "dynamic viscosity" | "absolute viscosity" => "visc",
        "kinematic viscosity" => "kvisc",
        "surface tension" => "st",
        "flash point" | "flashpoint" | "flash temperature" => "fp",
        "autoignition temperature" | "auto-ignition temperature" | "autoignition point" | "autoignition" | "ignition temperature" | "self-ignition temperature" | "auto ignition temperature" => "ait",
        "flammability limits" | "flammability limit" | "explosive limits" | "explosive limit" | "flammable limits" | "explosive range" | "flammable range" | "flammability range" | "lower explosive limit" | "upper explosive limit" | "lel" | "uel" => "fl",
        "heat of combustion" | "enthalpy of combustion" | "combustion heat" | "combustion enthalpy" | "energy content" | "heating value" | "calorific value" | "energy density" | "specific energy" | "standard enthalpy of combustion" | "molar heat of combustion" => "hc",
        "heat of vaporization" | "enthalpy of vaporization" | "latent heat of vaporization" | "heat of vaporisation" | "enthalpy of vaporisation" | "latent heat of vaporisation" | "vaporization enthalpy" => "hvap",
        "speed of sound" | "sound speed" | "velocity of sound" => "sos",
        "refractive index" | "index of refraction" | "refraction index" => "n",
        "dielectric constant" | "relative permittivity" | "permittivity" | "relative dielectric constant" => "eps",
        "tlv" | "threshold limit value" | "exposure limit" | "occupational exposure limit" | "tlv-twa" => "tlv",
        _ => return None,
    })
}

const PROPS: &str = r"viscosity|dynamic viscosity|absolute viscosity|kinematic viscosity|surface tension|flash ?point|flash temperature|auto-?ignition temperature|auto ignition temperature|autoignition point|autoignition|ignition temperature|self-ignition temperature|flammability limits?|explosive limits?|flammable limits|explosive range|flammable range|flammability range|lower explosive limit|upper explosive limit|lel|uel|(?:standard |molar )?(?:heat|enthalpy) of combustion|combustion heat|combustion enthalpy|energy content|heating value|calorific value|energy density|specific energy|(?:latent )?heat of vapori[sz]ation|enthalpy of vapori[sz]ation|vaporization enthalpy|speed of sound|sound speed|velocity of sound|refractive index|index of refraction|refraction index|(?:relative )?dielectric constant|relative permittivity|permittivity|tlv(?:-twa)?|threshold limit value|(?:occupational )?exposure limit";

const LABELS: &[(&str, &str)] = &[
    ("visc", "dynamic viscosity"),
    ("kvisc", "kinematic viscosity"),
    ("st", "surface tension"),
    ("fp", "flash point"),
    ("ait", "autoignition temperature"),
    ("fl", "flammability limits in air"),
    ("hc", "heat of combustion"),
    ("hvap", "enthalpy of vaporization"),
    ("sos", "speed of sound"),
    ("n", "refractive index"),
    ("eps", "dielectric constant"),
    ("tlv", "threshold limit value"),
];

fn label(k: &str) -> &'static str {
    LABELS.iter().find(|(a, _)| *a == k).map(|(_, b)| *b).unwrap_or("value")
}

/// Liquid density in g/cm³, from the substance tables.
fn density(f: &Fluid) -> Option<f64> {
    if f.gas {
        return None;
    }
    f.names.iter().find_map(|nm| super::chemcalc::find_substance(nm)).and_then(|s| s.density).filter(|d| *d > 0.1)
}

/// The value of property `k` for `f`, at temperature `t` °C when asked.
fn value(f: &Fluid, k: &str, t: Option<f64>) -> Option<String> {
    let mut s = match k {
        "visc" => {
            let (v, t0) = f.num("visc")?;
            if f.gas {
                format!("{} µPa·s ({} mPa·s) {} and 1 atm", n(v * 1000.0), n(v), at(t0))
            } else {
                format!("{} mPa·s (cP) {}", n(v), at(t0))
            }
        }
        "kvisc" => {
            let (v, t0) = f.num("visc")?;
            let d = density(f)?;
            format!("{} mm²/s (cSt) {}", n(v / d), at(t0))
        }
        "st" => {
            let (v, t0) = f.num("st")?;
            match (t, f.num("dst")) {
                (Some(t), Some((slope, _))) if (t - t0).abs() > 0.5 && (t - t0).abs() <= 60.0 => {
                    return Some(format!("≈ {} mN/m {} (from {} mN/m {}, slope {} mN/(m·K))", n(v + slope * (t - t0)), at(t), n(v), at(t0), n(slope)));
                }
                _ => format!("{} mN/m {}", n(v), at(t0)),
            }
        }
        "fp" => {
            let (v, _) = f.num("fp")?;
            format!("{} °C ({} °F), closed cup", n(v), degf(v))
        }
        "ait" => {
            let (v, _) = f.num("ait")?;
            format!("{} °C ({} °F)", n(v), degf(v))
        }
        "fl" => {
            let (a, b) = f.get("fl")?.split_once('-')?;
            format!("{a}–{b} % by volume in air (lower and upper flammability limits)")
        }
        "hc" => {
            if let Some((v, _)) = f.num("hc") {
                let mut s = format!("ΔcH° = −{} kJ/mol", n(v));
                if let Some(m) = f.molar {
                    s += &format!(" ({} MJ/kg higher heating value", n(v / m));
                    if let Some((l, _)) = f.num("lhv") {
                        s += &format!("; lower heating value {} MJ/kg", n(l / m));
                    }
                    s += ")";
                }
                s
            } else {
                let (v, _) = f.num("hcm")?;
                format!("about {} MJ/kg ({} BTU/lb; higher heating value, typical)", n(v), n((v * 429.923).round()))
            }
        }
        "hvap" => {
            let (v, _) = f.num("hvap")?;
            match f.molar {
                Some(m) => format!("{} kJ/mol ({} J/g) at the normal boiling point", n(v), n((v * 1000.0 / m).round())),
                None => format!("{} kJ/mol at the normal boiling point", n(v)),
            }
        }
        "sos" => {
            let (v, t0) = f.num("sos")?;
            format!("{} m/s {}", n(v), at(t0))
        }
        "n" => format!("{} (sodium D line, 589 nm)", f.get("n")?),
        "eps" => {
            let (v, t0) = f.num("eps")?;
            format!("{} {}", n(v), at(t0))
        }
        "tlv" => format!("{} (ACGIH TLV-TWA, 8-hour time-weighted average)", f.get("tlv")?).replace("(STEL) (ACGIH TLV-TWA, 8-hour time-weighted average)", "(ACGIH short-term exposure limit)"),
        _ => return None,
    };
    if let (Some(t), "visc" | "kvisc" | "sos" | "eps") = (t, k) {
        let t0 = f.num(if k == "kvisc" { "visc" } else { k }).map(|x| x.1).unwrap_or(25.0);
        if (t - t0).abs() > 0.5 {
            s += &format!(" (no value stored {})", at(t));
        }
    }
    Some(s)
}

/// Rows for a compound card: every stored property.
pub(crate) fn card_rows(name: &str) -> Vec<(String, String)> {
    let Some(f) = find(name) else { return vec![] };
    LABELS.iter().filter_map(|(k, l)| value(f, k, None).map(|v| (l.to_string(), v))).collect()
}

fn split_items(s: &str) -> Vec<String> {
    re!(r"\s*(?:,\s*(?:and\s+)?|/|\s+and\s+|\s+vs\.?\s+|\s+versus\s+|\s+&\s+)\s*").split(s.trim()).map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
}

fn temp_c(v: &str, u: &str) -> Option<f64> {
    let v: f64 = v.parse().ok()?;
    let u = u.trim().trim_start_matches('°').trim();
    Some(match u {
        "c" | "celsius" | "degrees celsius" => v,
        "f" | "fahrenheit" | "degrees fahrenheit" => (v - 32.0) * 5.0 / 9.0,
        "k" | "kelvin" | "kelvins" => v - 273.15,
        _ => return None,
    })
}

fn answer(key: &str, items: &[String], t: Option<f64>) -> Option<Answer> {
    if items.is_empty() || items.len() > 8 {
        return None;
    }
    let found: Option<Vec<&Fluid>> = items.iter().map(|s| find(s)).collect();
    let found = found?;
    // the speed of sound in water, air and steel is a stored constant elsewhere
    if key == "sos" && found.len() == 1 && matches!(found[0].names[0], "water" | "air") {
        return None;
    }
    let vals: Vec<(String, Option<String>)> = found.iter().map(|f| (f.title(), value(f, key, t))).collect();
    if vals.iter().all(|v| v.1.is_none()) {
        return None;
    }
    let lab = label(key);
    let input = format!("{} | {lab}{}", vals.iter().map(|v| v.0.as_str()).collect::<Vec<_>>().join(", "), t.map(|t| format!(" | {}", at(t))).unwrap_or_default());
    let mut pods = vec![pod("Input", "Input interpretation", text(input))];
    if found.len() == 1 {
        pods.push(primary(pod("Result", &cap(lab), text(vals[0].1.clone()?))));
        let f = found[0];
        let others: Vec<Vec<Content>> = LABELS.iter().filter(|(k, _)| *k != key).filter_map(|(k, l)| value(f, k, None).map(|v| vec![text(*l), text(v)])).collect();
        if !others.is_empty() {
            pods.push(pod("Properties", "Other properties", table(None, others)));
        }
        if let Some(note) = f.get("note") {
            pods.push(pod("Note", "Note", text(note)));
        }
    } else {
        let main = vals.iter().map(|(a, b)| format!("{a}: {}", b.clone().unwrap_or_else(|| "(not available)".into()))).collect::<Vec<_>>().join(" | ");
        pods.push(primary(pod("Result", &cap(lab), text(main))));
        // rank by the leading number when every value has one
        let nums: Vec<Option<f64>> = found.iter().map(|f| match key {
            "fl" => f.get("fl").and_then(|v| v.split('-').next()?.parse().ok()),
            "hc" => f.num("hc").and_then(|(v, _)| Some(v / f.molar?)).or_else(|| f.num("hcm").map(|x| x.0)),
            "kvisc" => f.num("visc").and_then(|(v, _)| Some(v / density(f)?)),
            "n" => f.get("n")?.parse().ok(),
            "tlv" => None,
            k => f.num(k).map(|x| x.0),
        }).collect();
        if nums.iter().all(Option::is_some) {
            let mut order: Vec<usize> = (0..found.len()).collect();
            order.sort_by(|a, b| nums[*b].partial_cmp(&nums[*a]).unwrap_or(std::cmp::Ordering::Equal));
            let unit_note = if key == "hc" { " (per kilogram)" } else { "" };
            pods.push(pod("Ranking", &format!("Ranking, highest first{unit_note}"), text(order.iter().map(|i| vals[*i].0.clone()).collect::<Vec<_>>().join(" > "))));
        }
    }
    let src = match key {
        "tlv" => "ACGIH threshold limit values: airborne concentrations to which nearly all workers may be repeatedly exposed without adverse effect",
        "fp" | "ait" | "fl" => "standard fire-safety data (NFPA, NIOSH); closed-cup flash points",
        "hc" => "standard enthalpies of combustion to liquid water and CO₂ at 25 °C (higher heating value); fuels: typical values",
        _ => "handbook values (CRC Handbook of Chemistry and Physics, NIST)",
    };
    pods.push(pod("Source", "Source", text(src)));
    Some(Answer::new(format!("{lab} of {}", items.join(", ")), pods))
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// A bare mixture or fuel with no single formula: "gasoline", "air",
/// "honey". Called last, after the materials tables have had their turn.
pub(crate) fn mixture_card(l: &str) -> Option<Answer> {
    if l.split(' ').count() > 3 {
        return None;
    }
    let fuel = find(l).filter(|f| f.formula.is_empty() && f.molar.is_none() && ["visc", "hcm", "fp", "fl"].iter().any(|k| f.get(k).is_some()));
    let mix = super::chemcalc::find_substance(l).filter(|x| x.formula.is_empty());
    if fuel.is_none() && mix.is_none() {
        return None;
    }
    let title = cap(fuel.map(|f| f.names[0]).unwrap_or(l));
    let mut p = super::chemcalc::property_pod(l)?;
    p.primary = true;
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{title} | physical properties"))), p];
    if let Some(note) = fuel.and_then(|f| f.get("note")) {
        pods.push(pod("Note", "Note", text(note)));
    }
    pods.push(pod("Source", "Note on values", text("typical values; mixtures vary with composition")));
    Some(Answer::new(title, pods))
}

/// Ideal-gas properties of air (standard tables; reference u = h = 0 at 0 K):
/// T (K), h (kJ/kg), u (kJ/kg), s° (kJ/(kg·K)).
const AIR_TABLE: &[(f64, f64, f64, f64)] = &[
    (300.0, 300.19, 214.07, 1.70203),
    (400.0, 400.98, 286.16, 1.99194),
    (500.0, 503.02, 359.49, 2.21952),
    (600.0, 607.02, 434.78, 2.40902),
    (800.0, 821.95, 592.30, 2.71787),
    (1000.0, 1046.04, 758.94, 2.96770),
];

/// "internal energy of air", "enthalpy of air at 500 K"
fn air_props(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:specific )?(internal energy|enthalpy|entropy|thermodynamic properties) of (?:dry )?air(?: at (\d+(?:\.\d+)?) ?(k|kelvin|°c|c|degrees celsius))?$").captures(l)?;
    let t = match m.get(2) {
        Some(v) => {
            let x: f64 = v.as_str().parse().ok()?;
            if m[3].starts_with('k') { x } else { x + 273.15 }
        }
        None => 300.0,
    };
    let i = AIR_TABLE.windows(2).position(|w| t >= w[0].0 && t <= w[1].0)?;
    let (a, b) = (AIR_TABLE[i], AIR_TABLE[i + 1]);
    let f = (t - a.0) / (b.0 - a.0);
    let lerp = |x: f64, y: f64| x + f * (y - x);
    let (h, u, s) = (lerp(a.1, b.1), lerp(a.2, b.2), lerp(a.3, b.3));
    let main = match &m[1] {
        "internal energy" => format!("{:.1} kJ/kg", u),
        "enthalpy" => format!("{:.1} kJ/kg", h),
        "entropy" => format!("{:.4} kJ/(kg·K) (s°, at 1 atm)", s),
        _ => format!("u = {:.1} kJ/kg, h = {:.1} kJ/kg", u, h),
    };
    Some(Answer::new("air properties", vec![
        pod("Input", "Input interpretation", text(format!("air | {} | T = {} K", &m[1], t))),
        primary(pod("Result", "Result", text(main))),
        pod("Table", "Ideal-gas properties", table(None, vec![
            vec![text("specific internal energy u"), text(format!("{:.2} kJ/kg", u))],
            vec![text("specific enthalpy h"), text(format!("{:.2} kJ/kg", h))],
            vec![text("standard entropy s°"), text(format!("{:.4} kJ/(kg·K)", s))],
            vec![text("h − u = R T"), text(format!("{:.2} kJ/kg (R = 0.287 kJ/(kg·K))", 0.287 * t))],
        ])),
        pod("Note", "Note", text(format!("ideal-gas air tables with u = h = 0 at 0 K{}", if m.get(2).is_none() { "; 300 K assumed" } else { "; linear interpolation between table rows" }))),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    if let Some(a) = air_props(&l) {
        return Some(a);
    }
    let l = re!(r"^(?:what is |what are |what's |find |compare |show |give me )?(?:the )?").replace(&l, "").to_string();
    // "how fast does sound travel in chloroform"
    if let Some(m) = re!(r"^how fast does sound (?:travel|move|go|propagate) (?:in|through) (.+)$").captures(&l) {
        return answer("sos", &split_items(&m[1]), None);
    }
    // an optional trailing temperature: "... at 25C", "... at 300 K"
    let (body, t) = match re!(r"^(.+?),? at (-?\d+(?:\.\d+)?) ?(°c|° c|c|°f|f|k|kelvins?|celsius|fahrenheit|degrees celsius|degrees fahrenheit)$").captures(&l) {
        Some(m) => (m[1].to_string(), temp_c(&m[2], &m[3])),
        None => (l.clone(), None),
    };
    static FRONT: OnceLock<Regex> = OnceLock::new();
    static BACK: OnceLock<Regex> = OnceLock::new();
    let front = FRONT.get_or_init(|| Regex::new(&format!(r"^({PROPS})(?: of| for| in)? (.+)$")).unwrap());
    let back = BACK.get_or_init(|| Regex::new(&format!(r"^(.+?)(?:'s)? ({PROPS})$")).unwrap());
    let (p, items) = if let Some(m) = front.captures(&body) {
        (m[1].to_string(), m[2].to_string())
    } else if let Some(m) = back.captures(&body) {
        (m[2].to_string(), m[1].to_string())
    } else {
        return None;
    };
    let p = p.replace("flashpoint", "flash point");
    let key = prop_key(&p)?;
    answer(key, &split_items(&items), t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_parses() {
        // binary search needs the compound synonym blocklist sorted
        assert!(crate::domains::chem::SYNONYM_BLOCK.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(data().len(), DATA.lines().filter(|l| !l.trim().is_empty()).count());
        for f in data() {
            for (k, v) in &f.props {
                if !matches!(*k, "tlv" | "note" | "fl" | "gas") {
                    assert!(f.num(k).is_some(), "{} {k}={v}", f.names[0]);
                }
            }
        }
    }
}
