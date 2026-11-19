//! Stellar physics: physical data for bright stars (mass, radius,
//! temperature, luminosity, colour), spectral-class decoding, stars of a
//! given temperature, luminosity or mass, stellar lifetimes and fates,
//! Hertzsprung–Russell placement, supernovae, pulsars, constellations and
//! globular-cluster data. Positions and distances come from the sky
//! module's catalogue.

use super::sky::{self, find_star, Star, CONSTELLATIONS, MONTH_NAMES, STARS};
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

const SUN_T: f64 = 5772.0;
const SUN_L_W: f64 = 3.828e26;
const SUN_M_KG: f64 = 1.988_47e30;
const SUN_R_KM: f64 = 695_700.0;
const WIEN: f64 = 2.897_771_955e-3;
const SIGMA: f64 = 5.670_374_419e-8;

fn n(v: f64) -> String {
    fmt_float(v, 4)
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn rows(r: Vec<(&str, String)>) -> Content {
    table(None, r.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())
}

/// "ngc 2808" → "NGC 2808", "alpha centauri" → "Alpha Centauri"
fn title(name: &str) -> String {
    name.split(' ')
        .map(|w| match w {
            "ngc" | "ic" | "psr" | "sn" => w.to_uppercase(),
            "and" | "of" | "in" => w.to_string(),
            _ if re!(r"^m\d+$").is_match(w) => w.to_uppercase(),
            _ => {
                let mut c = w.chars();
                c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize(s: &str) -> String {
    title(s)
}

fn years(y: f64) -> String {
    if y >= 1e12 {
        format!("about {} trillion years", n(y / 1e12))
    } else if y >= 1e9 {
        format!("about {} billion years", n(y / 1e9))
    } else if y >= 1e6 {
        format!("about {} million years", n(y / 1e6))
    } else {
        format!("about {} years", n(y))
    }
}

// ---------------------------------------------------------------------------
// physical data
// ---------------------------------------------------------------------------

struct Phys {
    mass: f64,
    radius: f64,
    teff: f64,
    lum: f64,
    bv: Option<f64>,
}

// catalogue name | mass M☉ | radius R☉ | effective temperature K | luminosity L☉ | B−V colour index
const PHYS: &str = "\
sirius|2.063|1.711|9940|25.4|0.00
canopus|8.0|71|7350|10700|0.15
alpha centauri|1.079|1.2175|5790|1.519|0.71
arcturus|1.08|25.4|4286|170|1.23
vega|2.135|2.36|9602|40.1|0.00
capella|2.57|11.98|4970|78.7|0.80
rigel|21|78.9|12100|120000|-0.03
procyon|1.499|2.048|6530|6.93|0.42
betelgeuse|16.5|764|3600|126000|1.85
hadar|12.0|8.6|25000|41700|-0.23
altair|1.86|1.79|7670|10.6|0.22
aldebaran|1.16|44.1|3910|439|1.54
antares|12|680|3570|75900|1.83
spica|11.43|7.47|25300|20500|-0.23
pollux|1.91|9.06|4586|32.7|1.00
fomalhaut|1.92|1.84|8590|16.6|0.09
deneb|19|203|8525|196000|0.09
regulus|3.8|3.09|12460|288|-0.11
bellatrix|7.7|5.75|21800|9211|-0.21
polaris|5.4|37.5|6015|1260|0.60
algol|3.17|2.73|13000|182|-0.05
eta draconis|2.55|11.0|5055|60.6|0.91
mira|1.18|350|3000|8400|
proxima centauri|0.1221|0.1542|3042|0.00155|1.82
barnard's star|0.162|0.187|3195|0.0034|1.72
wolf 359|0.11|0.144|2749|0.0011|
tau ceti|0.783|0.793|5344|0.488|0.72
epsilon eridani|0.82|0.735|5084|0.32|0.88";

fn phys(s: &Star) -> Option<Phys> {
    PHYS.lines().find(|l| l.split('|').next() == Some(s.names[0])).map(|l| {
        let f: Vec<&str> = l.split('|').collect();
        let num = |i: usize| f.get(i).and_then(|x| x.parse::<f64>().ok());
        Phys { mass: num(1).unwrap_or(0.0), radius: num(2).unwrap_or(0.0), teff: num(3).unwrap_or(0.0), lum: num(4).unwrap_or(0.0), bv: num(5) }
    })
}

/// (catalogue name, physical radius in light years, Shapley–Sawyer concentration class)
const CLUSTERS: &[(&str, f64, &str)] = &[
    ("m3", 90.0, "VI"),
    ("m4", 35.0, "IX"),
    ("m13", 84.0, "V"),
    ("m15", 88.0, "IV"),
    ("omega centauri", 86.0, "VIII"),
    ("m80", 48.0, "II"),
    ("ngc 2808", 0.0, "I"),
    ("m53", 0.0, "V"),
    ("ngc 288", 0.0, "X"),
];

/// (catalogue name, variable type, period)
const VARIABLES: &[(&str, &str, &str)] = &[
    ("mira", "Mira variable (the prototype)", "332 days"),
    ("algol", "Algol-type eclipsing binary", "2.867 days (2 d 20 h 49 m)"),
    ("polaris", "classical Cepheid", "3.97 days"),
    ("betelgeuse", "semiregular variable", "about 400 days (with a long secondary period near 2100 days)"),
    ("antares", "slow irregular variable", "about 1650 days"),
];

// ---------------------------------------------------------------------------
// spectral classes and main-sequence relations
// ---------------------------------------------------------------------------

const LETTERS: [char; 7] = ['O', 'B', 'A', 'F', 'G', 'K', 'M'];

/// main-sequence effective temperature by position (letter index × 10 + subclass)
const TEMP: &[(f64, f64)] = &[
    (3.0, 44900.0), (5.0, 41400.0), (7.0, 36500.0), (9.0, 31900.0), (10.0, 31400.0), (11.0, 26000.0), (12.0, 20600.0), (13.0, 17000.0), (15.0, 15700.0), (16.0, 14500.0), (17.0, 14000.0), (18.0, 12300.0), (19.0, 10700.0),
    (20.0, 9700.0), (21.0, 9300.0), (22.0, 8800.0), (23.0, 8600.0), (25.0, 8100.0), (27.0, 7650.0), (30.0, 7220.0), (32.0, 6810.0), (35.0, 6510.0), (38.0, 6170.0), (40.0, 5920.0), (42.0, 5770.0), (45.0, 5660.0), (48.0, 5490.0),
    (50.0, 5280.0), (52.0, 4990.0), (55.0, 4450.0), (57.0, 4050.0), (60.0, 3850.0), (62.0, 3560.0), (63.0, 3430.0), (64.0, 3210.0), (65.0, 3060.0), (66.0, 2810.0), (67.0, 2680.0), (68.0, 2570.0), (69.0, 2380.0),
];

/// main-sequence mass (M☉) by position
const MASS: &[(f64, f64)] = &[
    (3.0, 60.0), (5.0, 37.0), (7.0, 26.0), (9.0, 18.0), (10.0, 17.5), (11.0, 11.0), (12.0, 7.3), (13.0, 5.4), (15.0, 4.2), (17.0, 3.6), (18.0, 3.2), (19.0, 2.75),
    (20.0, 2.3), (22.0, 2.0), (25.0, 1.85), (27.0, 1.7), (30.0, 1.6), (35.0, 1.33), (38.0, 1.18), (40.0, 1.07), (42.0, 1.0), (45.0, 0.95), (48.0, 0.9),
    (50.0, 0.88), (52.0, 0.8), (55.0, 0.69), (57.0, 0.62), (60.0, 0.57), (62.0, 0.44), (63.0, 0.37), (64.0, 0.23), (65.0, 0.16), (66.0, 0.11), (67.0, 0.1), (68.0, 0.09), (69.0, 0.08),
];

fn interp(t: &[(f64, f64)], x: f64) -> f64 {
    if x <= t[0].0 {
        return t[0].1;
    }
    for w in t.windows(2) {
        if x <= w[1].0 {
            return w[0].1 + (w[1].1 - w[0].1) * (x - w[0].0) / (w[1].0 - w[0].0);
        }
    }
    t[t.len() - 1].1
}

/// position whose (decreasing) table value is `y`
fn inverse(t: &[(f64, f64)], y: f64) -> f64 {
    if y >= t[0].1 {
        return t[0].0;
    }
    for w in t.windows(2) {
        if y >= w[1].1 {
            return w[0].0 + (w[1].0 - w[0].0) * (w[0].1 - y) / (w[0].1 - w[1].1);
        }
    }
    t[t.len() - 1].0
}

fn class_text(pos: f64) -> String {
    let p = (pos * 2.0).round() / 2.0;
    let i = ((p / 10.0).floor() as usize).min(6);
    let sub = (p - i as f64 * 10.0).min(9.5);
    format!("{}{}", LETTERS[i], fmt_float(sub, 2))
}

fn letter_info(i: usize) -> (&'static str, &'static str) {
    match i {
        0 => ("blue", "above 30,000 K"),
        1 => ("blue-white", "10,000–30,000 K"),
        2 => ("white", "7,500–10,000 K"),
        3 => ("yellow-white", "6,000–7,500 K"),
        4 => ("yellow", "5,200–6,000 K"),
        5 => ("orange", "3,700–5,200 K"),
        _ => ("red", "2,400–3,700 K"),
    }
}

fn lum_info(c: &str) -> (&'static str, &'static str) {
    match c {
        "0" | "ia0" | "ia+" => ("0", "hypergiant"),
        "ia" => ("Ia", "luminous supergiant"),
        "iab" => ("Iab", "intermediate-luminosity supergiant"),
        "ib" => ("Ib", "less luminous supergiant"),
        "ii" => ("II", "bright giant"),
        "iii" => ("III", "giant"),
        "iv" => ("IV", "subgiant"),
        "v" => ("V", "main-sequence star (dwarf)"),
        "vi" => ("VI", "subdwarf"),
        _ => ("VII", "white dwarf"),
    }
}

fn region(c: &str) -> &'static str {
    match c {
        "0" | "ia0" | "ia+" | "ia" | "iab" | "ib" => "supergiants (top of the diagram)",
        "ii" | "iii" => "giant branch (upper right)",
        "iv" => "subgiants (just above the main sequence)",
        "v" => "main sequence",
        "vi" => "subdwarfs (just below the main sequence)",
        _ => "white dwarfs (lower left)",
    }
}

struct Spec {
    letter: usize,
    sub: Option<f64>,
    lum: Option<String>,
}

impl Spec {
    fn pos(&self) -> f64 {
        self.letter as f64 * 10.0 + self.sub.unwrap_or(5.0)
    }
    fn label(&self) -> String {
        format!("{}{}{}", LETTERS[self.letter], self.sub.map(|s| fmt_float(s, 2)).unwrap_or_default(), self.lum.as_deref().map(|c| lum_info(c).0).unwrap_or(""))
    }
}

/// Parse a spectral type at the start of `s`; returns the type and the
/// number of bytes consumed.
fn parse_spec(s: &str) -> Option<(Spec, usize)> {
    let s = s.trim().to_lowercase();
    let c = re!(r"^([obafgkm])(\d(?:\.\d+)?)?(?:[-/]\d(?:\.\d+)?)?(?:\s*(ia0|ia\+|iab|ia|ib|iii|ii|iv|vii|vi|v|0)(?:[-/](?:iab|ia|ib|iii|ii|iv|v|ab|b))?)?").captures(&s)?;
    let letter = "obafgkm".find(&c[1])?;
    let sub = c.get(2).and_then(|x| x.as_str().parse::<f64>().ok());
    Some((Spec { letter, sub, lum: c.get(3).map(|x| x.as_str().to_string()) }, c[0].len()))
}

fn spec_exact(s: &str) -> Option<Spec> {
    let s = s.trim();
    let (sp, used) = parse_spec(s)?;
    (used == s.len()).then_some(sp)
}

fn ms_lum(m: f64) -> f64 {
    if m < 0.43 {
        0.23 * m.powf(2.3)
    } else if m < 2.0 {
        m.powi(4)
    } else if m < 55.0 {
        1.4 * m.powf(3.5)
    } else {
        32000.0 * m
    }
}

fn ms_mass_from_lum(l: f64) -> f64 {
    if l < 0.033 {
        (l / 0.23).powf(1.0 / 2.3)
    } else if l < 16.0 {
        l.powf(0.25)
    } else if l < 1.73e6 {
        (l / 1.4).powf(1.0 / 3.5)
    } else {
        l / 32000.0
    }
}

/// radius in solar radii from luminosity and temperature (Stefan–Boltzmann)
fn radius_of(l: f64, t: f64) -> f64 {
    l.sqrt() / (t / SUN_T).powi(2)
}

fn lifetime(m: f64) -> f64 {
    1e10 * m / ms_lum(m)
}

/// (end state, path, remnant mass text)
fn fate(m: f64) -> (&'static str, &'static str, String) {
    if m < 0.08 {
        ("brown dwarf (never a true star)", "too light to fuse ordinary hydrogen; it slowly cools and fades", String::new())
    } else if m < 0.5 {
        ("helium white dwarf", "a red dwarf fuses hydrogen for far longer than the present age of the universe, then contracts into a helium white dwarf without becoming a giant", String::new())
    } else if m < 8.0 {
        let wd = (0.109 * m + 0.394).clamp(0.5, 1.35);
        ("carbon–oxygen white dwarf", "red giant → helium burning → asymptotic giant branch → sheds its envelope as a planetary nebula → white dwarf that cools towards a black dwarf", format!("about {} M☉", n(wd)))
    } else if m < 25.0 {
        ("neutron star", "red supergiant → fusion up to an iron core → core collapse → type II supernova → neutron star (often seen as a pulsar)", "about 1.4–2 M☉".to_string())
    } else if m < 130.0 {
        ("black hole", "supergiant (or Wolf–Rayet star) → core collapse, with a supernova or direct collapse → stellar-mass black hole", "a few to tens of M☉".to_string())
    } else if m < 250.0 {
        ("nothing (pair-instability supernova)", "electron–positron pair production destabilises the core and the star is completely disrupted, leaving no remnant", String::new())
    } else {
        ("black hole", "pair instability leads to direct collapse into a black hole", String::new())
    }
}

/// Typical main-sequence values for a spectral position: (temperature, mass, luminosity, radius).
fn ms_at(pos: f64) -> (f64, f64, f64, f64) {
    let t = interp(TEMP, pos);
    let m = interp(MASS, pos);
    let l = ms_lum(m);
    (t, m, l, radius_of(l, t))
}

fn examples(sp: &Spec) -> Vec<String> {
    STARS
        .iter()
        .filter(|s| {
            parse_spec(s.spectral).map_or(false, |(x, _)| x.letter == sp.letter && (sp.sub.is_none() || x.sub.map(|v| v.floor()) == sp.sub.map(|v| v.floor())) && (sp.lum.is_none() || x.lum == sp.lum))
        })
        .map(|s| format!("{} ({})", capitalize(s.names[0]), s.spectral))
        .collect()
}

fn spectral_class(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(?:the )?(?:spectral|stellar|morgan[- ]keenan|mk|harvard) (?:class(?:ification)?|type) |(?:an? )?(?:class|type) )?(.+?)(?: type)?(?: stars?)?$").captures(l)?;
    let sp = spec_exact(&m[1])?;
    // the bare form must be a full type such as "G2V"
    let prefixed = m.get(1).map_or(0, |x| x.start()) > 0;
    // ("M80" is Messier 80, not an M8 hypergiant)
    if !prefixed && (sp.sub.is_none() || sp.lum.is_none() || sp.lum.as_deref() == Some("0")) {
        return None;
    }
    let (colour, range) = letter_info(sp.letter);
    let (t, mass, lum, radius) = ms_at(sp.pos());
    let dwarf = sp.lum.as_deref().map_or(true, |c| c == "v");
    let kind = sp.lum.as_deref().map(|c| lum_info(c).1).unwrap_or("star");
    let label = sp.label();
    let mut r = vec![("spectral class", label.clone()), ("temperature class", format!("{}: {colour} stars, {range}", LETTERS[sp.letter]))];
    r.push((if dwarf { "typical effective temperature" } else { "typical temperature (main-sequence scale)" }, format!("about {} K", n((t / 10.0).round() * 10.0))));
    r.push(("colour", colour.to_string()));
    r.push(("peak emission wavelength", format!("about {} nm", n(WIEN / t * 1e9))));
    if let Some(c) = &sp.lum {
        let (roman, name) = lum_info(c);
        r.push(("luminosity class", format!("{roman}: {name}")));
        r.push(("position on the HR diagram", region(c).to_string()));
    }
    if dwarf {
        r.push(("typical main-sequence mass", format!("about {} M☉", n(mass))));
        r.push(("typical luminosity", format!("about {} L☉", n(lum))));
        r.push(("typical radius", format!("about {} R☉", n(radius))));
        r.push(("main-sequence lifetime", years(lifetime(mass))));
        r.push(("end state", fate(mass).0.to_string()));
    }
    let ex = examples(&sp);
    if !ex.is_empty() {
        r.push(("examples", ex.into_iter().take(6).collect::<Vec<_>>().join(", ")));
    }
    let main = format!("{label}: a {colour} {kind} with a surface temperature of about {} K", n((t / 100.0).round() * 100.0));
    Some(Answer::new("spectral class", vec![input(format!("spectral class {label}")), primary(pod("Result", "Result", text(main))), pod("Properties", "Properties", rows(r))]))
}

// ---------------------------------------------------------------------------
// stars of a given temperature, luminosity or mass
// ---------------------------------------------------------------------------

fn num(s: &str) -> Option<f64> {
    s.replace(',', "").parse::<f64>().ok().filter(|v| v.is_finite() && *v > 0.0)
}

fn star_of_temperature(l: &str) -> Option<Answer> {
    let c = re!(r"^(?:an? )?([\d,]+(?:\.\d+)?)\s*(?:k|kelvin|°k|degrees? kelvin)\s+star$").captures(l).or_else(|| re!(r"^(?:an? )?star (?:with (?:an? )?(?:surface |effective )?temperature (?:of )?|of |at )([\d,]+(?:\.\d+)?)\s*(?:k|kelvin)$").captures(l))?;
    let t = num(&c[1]).filter(|t| (1500.0..=60000.0).contains(t))?;
    let pos = inverse(TEMP, t);
    let i = (((pos * 2.0).round() / 20.0).floor() as usize).min(6);
    let (colour, _) = letter_info(i);
    let m = interp(MASS, pos);
    let lum = ms_lum(m);
    let flux = SIGMA * t.powi(4);
    let r = vec![
        ("spectral class (main sequence)", format!("about {}V", class_text(pos))),
        ("colour", colour.to_string()),
        ("peak emission wavelength (Wien)", format!("{} nm", n(WIEN / t * 1e9))),
        ("surface flux (σT⁴)", format!("{} MW/m²", n(flux / 1e6))),
        ("surface brightness relative to the Sun", format!("{} ×", n((t / SUN_T).powi(4)))),
        ("main-sequence mass", format!("about {} M☉", n(m))),
        ("main-sequence luminosity", format!("about {} L☉", n(lum))),
        ("main-sequence radius", format!("about {} R☉", n(radius_of(lum, t)))),
        ("main-sequence lifetime", years(lifetime(m))),
    ];
    Some(Answer::new("star temperature", vec![input(format!("star with effective temperature {} K", n(t))), primary(pod("Result", "Result", text(format!("a {colour} star of spectral class about {} (peak emission at {} nm)", class_text(pos), n(WIEN / t * 1e9))))), pod("Properties", "Properties (main-sequence star)", rows(r))]))
}

fn mass_answer(m: f64, stage: &str) -> Answer {
    let lum = ms_lum(m);
    let pos = inverse(MASS, m);
    let t = interp(TEMP, pos);
    let (end, path, remnant) = fate(m);
    let mut r = vec![
        ("mass", format!("{} M☉ ({} kg)", n(m), fmt_float(m * SUN_M_KG, 4))),
        ("main-sequence spectral class", format!("about {}V", class_text(pos))),
        ("main-sequence luminosity", format!("about {} L☉", n(lum))),
        ("main-sequence temperature", format!("about {} K", n((t / 10.0).round() * 10.0))),
        ("main-sequence radius", format!("about {} R☉", n(radius_of(lum, t)))),
        ("main-sequence lifetime", years(lifetime(m))),
        ("evolution", path.to_string()),
        ("end state", end.to_string()),
    ];
    if !remnant.is_empty() {
        r.push(("remnant mass", remnant));
    }
    if stage.contains("supergiant") {
        r.insert(1, ("stage", "a supergiant: the core has exhausted its hydrogen and the star has swollen to tens or hundreds of solar radii, with roughly 10⁴–10⁵ L☉".to_string()));
    } else if stage.contains("giant") {
        r.insert(1, ("stage", "a giant: hydrogen burns in a shell around an exhausted core".to_string()));
    }
    let label = format!("{} solar mass {}star", n(m), if stage.is_empty() { String::new() } else { format!("{stage} ") });
    Answer::new("stellar mass", vec![input(label), primary(pod("Result", "Result", text(format!("main-sequence lifetime {}; ends as a {end}", years(lifetime(m)).trim_start_matches("about "))))), pod("Properties", "Properties", rows(r))])
}

fn star_of_mass(l: &str) -> Option<Answer> {
    // "luminosity 10 solar masses": the piecewise main-sequence mass–luminosity relation
    if let Some(c) = re!(r"^(?:the )?(?:main[ -]sequence )?luminosity (?:of |for )?(?:an? )?([\d.]+)[\s-]*(?:solar[\s-]mass(?:es)?|m☉|msun|m_sun)(?: star)?$").captures(l) {
        let m = num(&c[1]).filter(|m| (0.01..=1000.0).contains(m))?;
        let lum = ms_lum(m);
        let band = if m < 0.43 { "0.23 M^2.3 (M < 0.43 M☉)" } else if m < 2.0 { "M^4 (0.43–2 M☉)" } else if m < 55.0 { "1.4 M^3.5 (2–55 M☉)" } else { "32000 M (M > 55 M☉)" };
        return Some(Answer::new("stellar luminosity", vec![
            input(format!("main-sequence luminosity | {} M☉", n(m))),
            primary(pod("Result", "Stellar luminosity", text(format!("{} L☉ ({} W)", n((lum * 10.0).round() / 10.0), fmt_float(lum * 3.828e26, 4))))),
            pod("Relation", "Mass–luminosity relation", text(format!("L/L☉ ≈ {band}"))),
        ]));
    }
    let c = re!(r"^(?:an? )?([\d.]+)[\s-]*(?:solar[\s-]mass(?:es)?|m☉|msun|m_sun)\s+((?:main[ -]sequence|red supergiant|blue supergiant|yellow supergiant|supergiant|red giant|giant)\s+)?star$")
        .captures(l)
        .or_else(|| re!(r"^(?:an? )?(?:((?:main[ -]sequence|supergiant|giant)\s+)?star (?:of |with (?:an? )?mass (?:of )?))([\d.]+)\s*(?:solar masses|solar mass|m☉|msun)$").captures(l))?;
    let (v, stage) = if c[1].chars().next().map_or(false, |ch| ch.is_ascii_digit()) { (&c[1], c.get(2).map_or("", |x| x.as_str())) } else { (c.get(2).map_or("", |x| x.as_str()), c.get(1).map_or("", |x| x.as_str())) };
    let m = num(v).filter(|m| (0.01..=1000.0).contains(m))?;
    Some(mass_answer(m, stage.trim()))
}

fn star_of_luminosity(l: &str) -> Option<Answer> {
    if !l.contains("star") {
        return None;
    }
    let c = re!(r"^(?:an? )?(?:star (?:of |with (?:an? )?luminosity (?:of )?)?)?([\d.]+(?:e-?\d+)?)\s*(?:solar luminosit(?:y|ies)|l☉|lsun|l_sun|times (?:the )?(?:sun'?s )?luminosity(?: of the sun)?)(?: star)?$").captures(l)?;
    let lum = num(&c[1])?;
    let m = ms_mass_from_lum(lum);
    if !(0.08..=300.0).contains(&m) {
        return None;
    }
    let pos = inverse(MASS, m);
    let t = interp(TEMP, pos);
    let r = vec![
        ("luminosity", format!("{} L☉ ({} W)", n(lum), fmt_float(lum * SUN_L_W, 4))),
        ("absolute bolometric magnitude", fmt_float(4.74 - 2.5 * lum.log10(), 3)),
        ("main-sequence mass (mass–luminosity relation)", format!("about {} M☉", n(m))),
        ("spectral class", format!("about {}V", class_text(pos))),
        ("effective temperature", format!("about {} K", n((t / 10.0).round() * 10.0))),
        ("radius", format!("about {} R☉", n(radius_of(lum, t)))),
        ("main-sequence lifetime", years(lifetime(m))),
        ("end state", fate(m).0.to_string()),
    ];
    Some(Answer::new("stellar luminosity", vec![input(format!("main-sequence star with luminosity {} L☉", n(lum))), primary(pod("Result", "Result", text(format!("a main-sequence star of about {} M☉, spectral class about {}V", n(m), class_text(pos))))), pod("Properties", "Properties", rows(r))]))
}

/// The mass behind a phrase such as "class B9V", "10 solar mass" or "Sirius".
fn subject_mass(s: &str) -> Option<(String, f64)> {
    let s = s.trim().trim_start_matches("the ").trim();
    let s = re!(r"^(?:(?:spectral )?(?:class|type) )").replace(s, "");
    let s = re!(r"\s+(?:main[ -]sequence )?stars?$").replace(&s, "").trim().to_string();
    if s == "sun" {
        return Some(("the Sun".into(), 1.0));
    }
    if let Some(c) = re!(r"^([\d.]+)[\s-]*(?:solar[\s-]mass(?:es)?|m☉|msun|times the mass of the sun)$").captures(&s) {
        return Some((format!("a {} solar mass star", &c[1]), num(&c[1])?));
    }
    if let Some(sp) = spec_exact(&s) {
        return Some((format!("a class {} star", sp.label()), interp(MASS, sp.pos())));
    }
    let st = find_star(&s)?;
    Some((capitalize(st.names[0]), phys(st)?.mass))
}

fn star_fate(l: &str) -> Option<Answer> {
    let c = re!(r"^what (?:will|would|does|do|is) (?:an? |the )?(.+?) (?:star )?(?:turn into|become|end up as|evolve into|leave behind|going to become)(?: when (?:it|they) dies?| at the end of its life| after it dies| eventually)?$")
        .captures(l)
        .or_else(|| re!(r"^(?:the )?(?:fate|end|death|final state|end state|remnant|evolution) of (?:an? |the )?(.+?)$").captures(l))
        .or_else(|| re!(r"^what happens (?:to|when) (?:an? |the )?(.+?)(?: when it)? dies$").captures(l))
        .or_else(|| re!(r"^how (?:will|does) (?:an? |the )?(.+?) die$").captures(l))?;
    let (who, m) = subject_mass(&c[1])?;
    let (end, path, remnant) = fate(m);
    let mut r = vec![("initial mass", format!("about {} M☉", n(m))), ("main-sequence lifetime", years(lifetime(m))), ("evolution", path.to_string()), ("end state", end.to_string())];
    if !remnant.is_empty() {
        r.push(("remnant mass", remnant));
    }
    Some(Answer::new("stellar fate", vec![input(format!("{who} | fate")), primary(pod("Result", "Result", text(format!("{who} will end as a {end}")))), pod("Details", "Stellar evolution", rows(r))]))
}

// ---------------------------------------------------------------------------
// supernovae and pulsars
// ---------------------------------------------------------------------------

// names | date | host | type | peak apparent magnitude | distance ly | constellation | note
const SUPERNOVAE: &str = "\
sn 1987a|23 February 1987|Large Magellanic Cloud|type II (peculiar; a blue-supergiant progenitor)|2.9|168000|Dorado|progenitor Sanduleak −69 202; the first supernova whose neutrinos were detected (about 25 at Kamiokande II, IMB and Baksan)
sn 1054;crab supernova|4 July 1054|Milky Way|type II (probably)|−6|6500|Taurus|recorded by Chinese astronomers and visible in daylight for 23 days; its remnant is the Crab Nebula with the Crab pulsar
sn 1006|30 April 1006|Milky Way|type Ia|−7.5|7200|Lupus|the brightest stellar event in recorded history
sn 1572;tycho's supernova;tychos supernova;tycho supernova|November 1572|Milky Way|type Ia|−4|8000|Cassiopeia|observed by Tycho Brahe, showing that the heavens could change
sn 1604;kepler's supernova;keplers supernova;kepler supernova|9 October 1604|Milky Way|type Ia|−2.5|20000|Ophiuchus|the most recent supernova in the Milky Way seen with the naked eye
sn 1885a;s andromedae|20 August 1885|Andromeda Galaxy|peculiar (probably type Ia)|5.85|2500000|Andromeda|the first supernova seen in another galaxy
sn 1993j|28 March 1993|M81|type IIb|10.7|11800000|Ursa Major|
sn 2011fe|24 August 2011|Pinwheel Galaxy (M101)|type Ia|9.9|21000000|Ursa Major|caught within a day of the explosion
sn 2014j|21 January 2014|Cigar Galaxy (M82)|type Ia|10.5|11400000|Ursa Major|the nearest type Ia supernova in decades";

// names | period s | distance ly | constellation | discovered | note
const PULSARS: &str = "\
crab pulsar;psr b0531+21;psr j0534+2200|0.0335|6500|Taurus|1968|the neutron star left by SN 1054, at the centre of the Crab Nebula; spin-down power about 4.5×10³¹ W
vela pulsar;psr b0833-45;psr j0835-4510|0.0893|959|Vela|1968|in the Vela supernova remnant, about 11,000 years old; known for frequent glitches
psr b1919+21;cp 1919|1.3373|2300|Vulpecula|1967|the first pulsar discovered, by Jocelyn Bell Burnell and Antony Hewish
hulse-taylor pulsar;hulse–taylor pulsar;psr b1913+16|0.059|21000|Aquila|1974|a binary pulsar whose orbital decay confirmed gravitational-wave emission (Nobel Prize 1993)
psr j1748-2446ad|0.001396|18000|Sagittarius|2004|the fastest-spinning known pulsar, 716 rotations a second, in the globular cluster Terzan 5";

fn record(data: &'static str, name: &str) -> Option<Vec<&'static str>> {
    data.lines().map(|l| l.split('|').collect::<Vec<_>>()).find(|f| f[0].split(';').any(|x| x == name))
}

fn event_name(w: &str) -> String {
    let w = w.trim().trim_start_matches("the ").trim();
    let w = re!(r"^(?:supernova|sn)\s*(\d{4}[a-z]{0,2})$").replace(w, "sn $1");
    let w = re!(r"^psr\s*").replace(&w, "psr ");
    w.replace(" pulsar pulsar", " pulsar")
}

/// A card for a supernova or pulsar: (display name, rows).
fn event_card(w: &str) -> Option<(String, Vec<(&'static str, String)>)> {
    let w = event_name(w);
    if let Some(f) = record(SUPERNOVAE, &w) {
        let id = f[0].split(';').next().unwrap().trim_start_matches("sn ");
        let (year, tag) = id.split_at(4);
        let name = format!("SN {year}{}", if tag.len() == 1 { tag.to_uppercase() } else { tag.to_string() });
        let ly: f64 = f[5].parse().ok()?;
        let mut r = vec![("date", f[1].to_string()), ("host galaxy", f[2].to_string()), ("type", f[3].to_string()), ("peak apparent magnitude", f[4].to_string()), ("distance", sky::ly_text(ly)), ("constellation", f[6].to_string())];
        if !f[7].is_empty() {
            r.push(("note", f[7].to_string()));
        }
        return Some((name, r));
    }
    let w = if w.ends_with(" pulsar") || w.starts_with("psr ") || w.starts_with("cp ") { w } else { format!("{w} pulsar") };
    let f = record(PULSARS, &w)?;
    let p: f64 = f[1].parse().ok()?;
    let ly: f64 = f[2].parse().ok()?;
    let name = f[0].split(';').next().unwrap();
    let name = if name.starts_with("psr") { name.to_uppercase() } else { capitalize(name) };
    let r = vec![
        ("rotation period", if p < 1.0 { format!("{} ms", n(p * 1000.0)) } else { format!("{} s", n(p)) }),
        ("rotation frequency", format!("{} Hz", n(1.0 / p))),
        ("distance", sky::ly_text(ly)),
        ("constellation", f[3].to_string()),
        ("discovered", f[4].to_string()),
        ("note", f[5].to_string()),
    ];
    Some((name, r))
}

// ---------------------------------------------------------------------------
// properties of named objects
// ---------------------------------------------------------------------------

fn dm(deg: f64) -> String {
    let d = deg.rem_euclid(360.0);
    let mut whole = d.floor();
    let mut min = ((d - whole) * 60.0 * 10.0).round() / 10.0;
    if min >= 60.0 {
        whole += 1.0;
        min -= 60.0;
    }
    format!("{}° {:.1}′", whole as i64 % 360, min)
}

fn colour_text(s: &Star, p: Option<&Phys>) -> Option<String> {
    let (sp, _) = parse_spec(s.spectral)?;
    let (c, _) = letter_info(sp.letter);
    Some(match p.and_then(|p| p.bv) {
        Some(bv) => format!("{c} (B−V colour index {})", fmt_float(bv, 3)),
        None => c.to_string(),
    })
}

/// The value of a property for a catalogue object, as (label, text).
fn star_value(s: &Star, prop: &str) -> Option<(String, String)> {
    let p = phys(s);
    let all = sky::star_rows(s);
    let pick = |k: &str| all.iter().find(|(a, _)| *a == k).map(|(_, v)| v.clone());
    let cluster = CLUSTERS.iter().find(|c| c.0 == s.names[0]);
    let v = match prop {
        "mass" => {
            let p = p?;
            (prop.into(), format!("{} M☉ ({} kg)", n(p.mass), fmt_float(p.mass * SUN_M_KG, 3)))
        }
        "radius" | "average radius" | "mean radius" => match cluster {
            Some(c) if c.1 > 0.0 => ("radius".into(), format!("about {} light years", n(c.1))),
            _ => {
                let p = p?;
                ("radius".into(), format!("{} R☉ ({} km)", n(p.radius), fmt_float(p.radius * SUN_R_KM, 3)))
            }
        },
        "diameter" | "size" => match cluster {
            Some(c) if c.1 > 0.0 => ("diameter".into(), format!("about {} light years", n(2.0 * c.1))),
            _ => {
                let p = p?;
                ("diameter".into(), format!("{} R☉ × 2 = {} km", n(p.radius), fmt_float(2.0 * p.radius * SUN_R_KM, 3)))
            }
        },
        "temperature" | "surface temperature" | "effective temperature" => ("effective temperature".into(), format!("{} K", n(p?.teff))),
        "luminosity" => match &p {
            Some(p) => ("luminosity".into(), format!("{} L☉ ({} W)", n(p.lum), fmt_float(p.lum * SUN_L_W, 3))),
            None => ("luminosity (visual)".into(), pick("luminosity (visual)")?),
        },
        "colour" | "color" => ("colour".into(), colour_text(s, p.as_ref())?),
        "spectral class" | "spectral type" | "stellar classification" | "type" => (if pick("spectral type").is_some() { "spectral class" } else { "type" }.into(), pick("spectral type").or_else(|| pick("type"))?),
        "distance" => ("distance from Earth".into(), pick("distance")?),
        "apparent magnitude" | "magnitude" | "brightness" | "visual magnitude" => ("apparent magnitude".into(), pick("apparent magnitude")?),
        "absolute magnitude" => (prop.into(), pick("absolute magnitude")?),
        "constellation" => (prop.into(), s.constellation.to_string()),
        "right ascension" => (prop.into(), sky::ra_text(s)),
        "declination" => (prop.into(), sky::dec_text(s)),
        "coordinates" | "position" | "location" | "equatorial location" | "equatorial coordinates" | "equatorial position" | "right ascension and declination" | "ra and dec" | "celestial coordinates" => ("equatorial coordinates".into(), format!("right ascension {} | declination {}", sky::ra_text(s), sky::dec_text(s))),
        "variability period" | "period" | "variable period" | "pulsation period" => {
            let v = VARIABLES.iter().find(|v| v.0 == s.names[0])?;
            ("variability period".into(), format!("{} ({})", v.2, v.1))
        }
        "sidereal hour angle" | "sha" => ("sidereal hour angle".into(), dm(360.0 - sky::ra_deg(s))),
        "greenwich hour angle" | "gha" => {
            let jd = sky::jd_now();
            let gmst = 280.460_618_37 + 360.985_647_366_29 * (jd - 2_451_545.0);
            ("Greenwich hour angle (now)".into(), dm(gmst - sky::ra_deg(s)))
        }
        "shapley-sawyer class" | "concentration class" => ("Shapley–Sawyer concentration class".into(), cluster?.2.to_string()),
        _ => return None,
    };
    Some(v)
}

fn event_value(w: &str, prop: &str) -> Option<(String, String, String)> {
    let (name, r) = event_card(w)?;
    let key = match prop {
        "period" | "rotation period" | "spin period" => "rotation period",
        "frequency" | "rotation frequency" | "spin frequency" => "rotation frequency",
        "distance" => "distance",
        "apparent magnitude" | "magnitude" | "peak magnitude" | "peak apparent magnitude" | "brightness" => "peak apparent magnitude",
        "date" | "discovery date" => "date",
        "host galaxy" | "galaxy" => "host galaxy",
        "type" => "type",
        "constellation" => "constellation",
        _ => return None,
    };
    let v = r.iter().find(|(k, _)| *k == key)?.1.clone();
    Some((name, key.to_string(), v))
}

fn canonical_prop(p: &str) -> String {
    let p = p.trim();
    let p = re!(r"^shapley[-–‐ ]sawyer (?:concentration )?class(?:ification)?$").replace(p, "shapley-sawyer class");
    let p = re!(r"^concentration class(?:ification)?$").replace(&p, "concentration class");
    let p = re!(r"^(?:distance (?:from|to) (?:earth|the sun)|distance from us)$").replace(&p, "distance");
    p.into_owned()
}

const PROPS: &str = r"mass|average radius|mean radius|radius|diameter|size|surface temperature|effective temperature|temperature|luminosity|colou?r|spectral (?:class|type)|stellar classification|distance(?: (?:from|to) (?:earth|the sun))?|apparent magnitude|visual magnitude|absolute magnitude|peak (?:apparent )?magnitude|magnitude|constellation|equatorial (?:location|coordinates|position)|celestial coordinates|coordinates|position|right ascension and declination|ra and dec|right ascension|declination|variability period|pulsation period|rotation period|spin period|rotation frequency|period|sidereal hour angle|greenwich hour angle|shapley[-–‐ ]sawyer (?:concentration )?class(?:ification)?|concentration class(?:ification)?|host galaxy|discovery date";

fn split_names(s: &str) -> Vec<String> {
    re!(r"\s*(?:,|/|;|\bvs\.?\s|\bversus\b|\band\b|&)\s*").split(s.trim()).map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

/// Resolve a list of names, as a whole or split on separators.
fn resolve<T>(s: &str, f: impl Fn(&str) -> Option<T>) -> Option<Vec<T>> {
    if let Some(x) = f(s) {
        return Some(vec![x]);
    }
    let parts = split_names(s);
    if parts.len() < 2 {
        return None;
    }
    parts.iter().map(|p| f(p)).collect()
}

fn properties(l: &str) -> Option<Answer> {
    static PRE: OnceLock<Regex> = OnceLock::new();
    static SUF: OnceLock<Regex> = OnceLock::new();
    let pre = PRE.get_or_init(|| Regex::new(&format!(r"^(?:what(?: is|'s) )?(?:the )?({PROPS})(?: of| for)? (?:the )?(.+?)$")).unwrap());
    let suf = SUF.get_or_init(|| Regex::new(&format!(r"^(?:the )?(.+?)(?:'s)? ({PROPS})$")).unwrap());
    let (prop, list) = if let Some(c) = re!(r"^(?:the )?distance (?:to|from) (?:the )?(.+?)(?: (?:from|to) (?:the )?(?:earth|us|sun))?$").captures(l) {
        ("distance".to_string(), c[1].to_string())
    } else if let Some(c) = pre.captures(l) {
        (canonical_prop(&c[1]), c[2].to_string())
    } else if let Some(c) = suf.captures(l) {
        (canonical_prop(&c[2]), c[1].to_string())
    } else {
        return None;
    };
    // stars and deep-sky objects
    if let Some(objs) = resolve(&list, find_star) {
        let vals: Option<Vec<(String, String)>> = objs.iter().map(|s| star_value(s, &prop)).collect();
        let vals = vals?;
        let label = vals[0].0.clone();
        if objs.len() == 1 {
            let s = objs[0];
            let mut more = sky::star_rows(s);
            if let Some(p) = phys(s) {
                more.insert(0, ("mass", format!("{} M☉", n(p.mass))));
                more.insert(1, ("radius", format!("{} R☉", n(p.radius))));
                more.insert(2, ("effective temperature", format!("{} K", n(p.teff))));
                more.insert(3, ("luminosity", format!("{} L☉", n(p.lum))));
            }
            return Some(Answer::new(s.names[0], vec![input(format!("{} | {label}", capitalize(s.names[0]))), primary(pod("Result", &capitalize(&label), text(vals[0].1.clone()))), pod("Properties", "Properties", rows(more))]));
        }
        let t: Vec<Vec<Content>> = objs.iter().zip(&vals).map(|(s, v)| vec![text(capitalize(s.names[0])), text(v.1.clone())]).collect();
        let main = objs.iter().zip(&vals).map(|(s, v)| format!("{}: {}", capitalize(s.names[0]), v.1)).collect::<Vec<_>>().join(" | ");
        return Some(Answer::new("star comparison", vec![input(format!("{} | {label}", objs.iter().map(|s| capitalize(s.names[0])).collect::<Vec<_>>().join(", "))), primary(pod("Result", &capitalize(&label), text(main))), pod("Table", "Comparison", table(Some(vec!["object", label.as_str()]), t))]));
    }
    // supernovae and pulsars
    let vals = resolve(&list, |w| event_value(w, &prop))?;
    let main = vals.iter().map(|(a, _, v)| if vals.len() == 1 { v.clone() } else { format!("{a}: {v}") }).collect::<Vec<_>>().join(" | ");
    let label = vals[0].1.clone();
    Some(Answer::new("stellar event", vec![input(format!("{} | {label}", vals.iter().map(|v| v.0.clone()).collect::<Vec<_>>().join(", "))), primary(pod("Result", &capitalize(&label), text(main)))]))
}

/// "Crab pulsar, Vela pulsar", "supernova 1987A", "alpha Centauri A, beta Centauri, gamma Centauri"
fn named_lists(l: &str) -> Option<Answer> {
    if let Some(cards) = resolve(l, event_card) {
        if cards.len() == 1 {
            let (name, r) = cards.into_iter().next()?;
            return Some(Answer::new("stellar event", vec![input(name.clone()), primary(pod("Properties", "Properties", rows(r)))]));
        }
        let keys: Vec<&str> = cards[0].1.iter().map(|(k, _)| *k).filter(|k| *k != "note").collect();
        let mut head = vec!["object"];
        head.extend(keys.iter().copied());
        let t = cards.iter().map(|(name, r)| {
            let mut row = vec![text(name.clone())];
            row.extend(keys.iter().map(|k| text(r.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone()).unwrap_or_default())));
            row
        });
        return Some(Answer::new("stellar events", vec![input(cards.iter().map(|c| c.0.clone()).collect::<Vec<_>>().join(", ")), primary(pod("Table", "Comparison", table(Some(head), t.collect())))]));
    }
    let parts = split_names(l);
    if parts.len() < 2 {
        return None;
    }
    let objs: Vec<&Star> = parts.iter().map(|p| find_star(p)).collect::<Option<_>>()?;
    let t = objs
        .iter()
        .map(|s| {
            let p = phys(s);
            vec![
                text(capitalize(s.names[0])),
                text(s.spectral),
                text(fmt_float(s.mag, 3)),
                text(n(s.ly)),
                text(s.constellation),
                text(p.as_ref().map(|p| n(p.mass)).unwrap_or_default()),
                text(p.as_ref().map(|p| n(p.teff)).unwrap_or_default()),
            ]
        })
        .collect();
    Some(Answer::new("star comparison", vec![input(objs.iter().map(|s| capitalize(s.names[0])).collect::<Vec<_>>().join(", ")), primary(pod("Table", "Comparison", table(Some(vec!["object", "type", "magnitude", "distance (ly)", "constellation", "mass (M☉)", "temperature (K)"]), t)))]))
}

fn hr_diagram(l: &str) -> Option<Answer> {
    let c = re!(r"^(?:an? )?(?:hr|h-r|hertzsprung[-–‐ ]russell) diagram(?: (?:of|for|with))? (.+)$").captures(l)?;
    let objs = resolve(&c[1], find_star)?;
    let t: Vec<Vec<Content>> = objs
        .iter()
        .map(|s| {
            let p = phys(s);
            let lum = p.as_ref().map(|p| p.lum).unwrap_or_else(|| 10f64.powf((4.83 - (s.mag - 5.0 * (s.ly / 3.261_56 / 10.0).log10())) / 2.5));
            let teff = p.as_ref().map(|p| p.teff).or_else(|| parse_spec(s.spectral).map(|(sp, _)| interp(TEMP, sp.pos())));
            let reg = parse_spec(s.spectral).and_then(|(sp, _)| sp.lum).map(|c| region(&c).to_string()).unwrap_or_default();
            vec![text(capitalize(s.names[0])), text(s.spectral), text(teff.map(|t| n(t)).unwrap_or_default()), text(n(lum)), text(fmt_float(4.74 - 2.5 * lum.log10(), 3)), text(reg)]
        })
        .collect();
    Some(Answer::new(
        "HR diagram",
        vec![
            input(format!("Hertzsprung–Russell diagram | {}", objs.iter().map(|s| capitalize(s.names[0])).collect::<Vec<_>>().join(", "))),
            primary(pod("Table", "Positions on the Hertzsprung–Russell diagram", table(Some(vec!["star", "spectral class", "temperature (K)", "luminosity (L☉)", "absolute magnitude", "region"]), t))),
            pod("Notes", "Reading the diagram", text("temperature increases to the left, luminosity upwards; the Sun sits on the main sequence at 5772 K and 1 L☉")),
        ],
    ))
}

// ---------------------------------------------------------------------------
// nearest stars
// ---------------------------------------------------------------------------

/// (name, distance ly, spectral type)
const NEAREST: &[(&str, f64, &str)] = &[
    ("Proxima Centauri", 4.2465, "M5.5Ve"),
    ("Alpha Centauri A", 4.37, "G2V"),
    ("Alpha Centauri B", 4.37, "K1V"),
    ("Barnard's Star", 5.96, "M4Ve"),
    ("Wolf 359", 7.86, "M6.5Ve"),
    ("Lalande 21185", 8.31, "M2V"),
    ("Sirius A", 8.60, "A1V"),
    ("Sirius B", 8.60, "DA2 (white dwarf)"),
    ("Luyten 726-8 A (BL Ceti)", 8.79, "M5.5Ve"),
    ("Luyten 726-8 B (UV Ceti)", 8.79, "M6Ve"),
    ("Ross 154", 9.70, "M3.5Ve"),
    ("Ross 248", 10.30, "M5.5Ve"),
    ("Epsilon Eridani", 10.47, "K2V"),
    ("Lacaille 9352", 10.72, "M0.5V"),
    ("Ross 128", 11.01, "M4V"),
    ("EZ Aquarii A", 11.11, "M5Ve"),
    ("Procyon A", 11.46, "F5IV–V"),
    ("Procyon B", 11.46, "DQZ (white dwarf)"),
    ("61 Cygni A", 11.40, "K5V"),
    ("61 Cygni B", 11.40, "K7V"),
    ("Struve 2398 A", 11.49, "M3V"),
    ("Groombridge 34 A", 11.62, "M1.5V"),
    ("Epsilon Indi A", 11.87, "K5V"),
    ("Tau Ceti", 11.91, "G8V"),
];

fn nearest(l: &str) -> Option<Answer> {
    let c = re!(r"^(?:the )?(\d+|three|five|ten|twenty) (?:nearest|closest) stars(?: (?:to|from) (?:the )?(?:earth|sun|us))?$").captures(l)?;
    let k = match &c[1] {
        "three" => 3,
        "five" => 5,
        "ten" => 10,
        "twenty" => 20,
        d => d.parse().ok()?,
    };
    if k == 0 || k > NEAREST.len() {
        return None;
    }
    let mut v: Vec<_> = NEAREST.to_vec();
    v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let t = v.iter().take(k).enumerate().map(|(i, (a, d, s))| vec![text((i + 1).to_string()), text(*a), text(n(*d)), text(*s)]).collect();
    Some(Answer::new("nearest stars", vec![input(format!("{k} nearest stars to the Sun")), primary(pod("Table", "Nearest stars (brown dwarfs excluded)", table(Some(vec!["", "star", "distance (ly)", "spectral type"]), t)))]))
}

// ---------------------------------------------------------------------------
// constellations
// ---------------------------------------------------------------------------

/// (name, IAU abbreviation, area in square degrees, brightest star)
const CON_INFO: &[(&str, &str, u32, &str)] = &[
    ("orion", "Ori", 594, "Rigel"),
    ("ursa major", "UMa", 1280, "Alioth"),
    ("ursa minor", "UMi", 256, "Polaris"),
    ("cassiopeia", "Cas", 598, "Schedar"),
    ("scorpius", "Sco", 497, "Antares"),
    ("sagittarius", "Sgr", 867, "Kaus Australis"),
    ("cygnus", "Cyg", 804, "Deneb"),
    ("lyra", "Lyr", 286, "Vega"),
    ("leo", "Leo", 947, "Regulus"),
    ("taurus", "Tau", 797, "Aldebaran"),
    ("gemini", "Gem", 514, "Pollux"),
    ("crux", "Cru", 68, "Acrux"),
    ("andromeda", "And", 722, "Alpheratz"),
    ("pegasus", "Peg", 1121, "Enif"),
    ("perseus", "Per", 615, "Mirfak"),
    ("canis major", "CMa", 380, "Sirius"),
    ("virgo", "Vir", 1294, "Spica"),
    ("aquarius", "Aqr", 980, "Sadalsuud"),
    ("pisces", "Psc", 889, "Alpherg"),
    ("aries", "Ari", 441, "Hamal"),
    ("cancer", "Cnc", 506, "Tarf"),
    ("libra", "Lib", 538, "Zubeneschamali"),
    ("capricornus", "Cap", 414, "Deneb Algedi"),
    ("boötes", "Boo", 907, "Arcturus"),
    ("bootes", "Boo", 907, "Arcturus"),
    ("draco", "Dra", 1083, "Eltanin"),
    ("centaurus", "Cen", 1060, "Alpha Centauri"),
    ("auriga", "Aur", 657, "Capella"),
    ("aquila", "Aql", 652, "Altair"),
    ("hercules", "Her", 1225, "Kornephoros"),
];

/// Names that answer on their own; the rest need the word "constellation".
const BARE_OK: &[&str] = &["orion", "ursa major", "ursa minor", "cassiopeia", "scorpius", "sagittarius", "cygnus", "lyra", "crux", "canis major", "capricornus", "bootes", "boötes", "draco", "centaurus", "auriga", "aquila"];

fn constellation(l: &str) -> Option<Answer> {
    let c = re!(r"^(?:the )?(?:constellation (?:of )?)?(.+?)(?: constellation)?$").captures(l)?;
    let name = c[1].trim();
    let explicit = l.contains("constellation");
    if !explicit && !BARE_OK.contains(&name) {
        return None;
    }
    let (_, ra, dec, note) = CONSTELLATIONS.iter().find(|c| c.0 == name)?;
    let best = sky::best_month(*ra);
    let mut r = vec![];
    if let Some(info) = CON_INFO.iter().find(|i| i.0 == name) {
        r.push(("IAU abbreviation", info.1.to_string()));
        r.push(("area", format!("{} square degrees", info.2)));
        r.push(("brightest star", info.3.to_string()));
    }
    r.push(("centre", format!("right ascension about {}h, declination about {}°", n(*ra), n(*dec))));
    r.push(("best seen", format!("{} evenings", MONTH_NAMES[best])));
    r.push(("fully visible", if *dec >= 0.0 { format!("north of about latitude {}°S", n((90.0 - dec - 15.0).max(0.0))) } else { format!("south of about latitude {}°N", n((90.0 + dec - 15.0).max(0.0))) }));
    let objs: Vec<String> = STARS.iter().filter(|s| s.constellation.to_lowercase() == name || (name == "bootes" && s.constellation == "Boötes")).map(|s| capitalize(s.names[0])).collect();
    if !objs.is_empty() {
        r.push(("catalogued objects", objs.join(", ")));
    }
    let title = capitalize(name);
    Some(Answer::new("constellation", vec![input(format!("{title} (constellation)")), primary(pod("Result", "Result", text(format!("{title}: {note}")))), pod("Properties", "Properties", rows(r))]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    spectral_class(l)
        .or_else(|| star_fate(l))
        .or_else(|| star_of_temperature(l))
        .or_else(|| star_of_mass(l))
        .or_else(|| star_of_luminosity(l))
        .or_else(|| nearest(l))
        .or_else(|| hr_diagram(l))
        .or_else(|| properties(l))
        .or_else(|| constellation(l))
        .or_else(|| named_lists(l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spectral_types_parse() {
        let (s, _) = parse_spec("M1-2Ia-ab").unwrap();
        assert_eq!((s.letter, s.lum.as_deref()), (6, Some("ia")));
        let s = spec_exact("g2v").unwrap();
        assert_eq!(s.label(), "G2V");
        assert!(spec_exact("a2ia").is_some());
        assert!(spec_exact("g2 vx").is_none());
    }

    #[test]
    fn main_sequence_relations() {
        let (t, m, l, r) = ms_at(42.0);
        assert!((t - 5770.0).abs() < 1.0 && (m - 1.0).abs() < 1e-9 && (l - 1.0).abs() < 1e-9 && (r - 1.0).abs() < 0.01);
        assert!((ms_mass_from_lum(0.65) - 0.898).abs() < 0.001);
        assert_eq!(fate(2.75).0, "carbon–oxygen white dwarf");
        assert_eq!(fate(10.0).0, "neutron star");
    }
}
