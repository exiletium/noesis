//! Polymers, construction and bulk materials, minerals and famous gems:
//! typical properties (density, melting and glass-transition temperatures,
//! strength, stiffness, thermal conductivity, R-value, heating value, bulk
//! density, angle of repose, Mohs hardness, crystal system), comparisons,
//! and the mass of a volume of a bulk material ("35 cubic yards crushed
//! stone").

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

/// names | kind | properties. Ranges are written "a-b".
///
/// f: chemical formula (repeat unit for polymers) · d: density, g/cm³ ·
/// bulk: loose bulk density, kg/m³ · mp: melting point, °C · tg: glass
/// transition, °C · ts: tensile strength, MPa · e: Young's modulus, GPa ·
/// k: thermal conductivity, W/(m·K) · r: R-value per inch, ft²·°F·h/BTU ·
/// hcm: heating value, MJ/kg · rep: angle of repose, ° · mohs · sys:
/// crystal system · ct: carats · comp: composition · col: colour.
const DATA: &str = "\
polyvinyl chloride,pvc,vinyl,rigid pvc,poly(vinyl chloride)|thermoplastic polymer|f=(C2H3Cl)n;d=1.38;tg=82;ts=52;e=3.0;k=0.19;hcm=18;note=softens above its glass transition and decomposes from about 140 °C rather than melting cleanly; flexible PVC (with plasticizer) is 1.1-1.35 g/cm³
polyethylene,pe,polythene|thermoplastic polymer|f=(C2H4)n;d=0.91-0.97;mp=105-135;tg=-125;ts=10-40;e=0.2-1.2;k=0.33-0.5;hcm=46;note=the most common plastic; grades: LDPE, LLDPE, HDPE
low density polyethylene,ldpe,molded low density polyethylene|thermoplastic polymer|f=(C2H4)n;d=0.92;mp=110;tg=-125;ts=10;e=0.2;k=0.33;hcm=46;note=branched chains; films, bags, squeeze bottles
high density polyethylene,hdpe|thermoplastic polymer|f=(C2H4)n;d=0.95;mp=130;ts=26;e=1.0;k=0.48;hcm=46;note=linear chains; bottles, pipes, cutting boards
polypropylene,pp|thermoplastic polymer|f=(C3H6)n;d=0.905;mp=165;tg=-10;ts=35;e=1.5;k=0.22;hcm=46
polystyrene,ps|thermoplastic polymer|f=(C8H8)n;d=1.05;tg=100;ts=45;e=3.2;k=0.14;hcm=41
expanded polystyrene,eps foam,styrofoam,polystyrene foam|foam insulation|f=(C8H8)n;d=0.015-0.03;k=0.035;r=4.0
extruded polystyrene,xps foam|foam insulation|f=(C8H8)n;d=0.03;k=0.029;r=5.0
polyethylene terephthalate,pet,pete,polyester|thermoplastic polymer|f=(C10H8O4)n;d=1.38;mp=260;tg=75;ts=55;e=2.8;k=0.2;hcm=23
nylon,nylon 6,6,nylon 66,nylon 6-6,polyamide|thermoplastic polymer|f=(C12H22N2O2)n;d=1.14;mp=264;tg=50;ts=80;e=3.0;k=0.25;hcm=31;note=polyhexamethylene adipamide, first made by Wallace Carothers at DuPont in 1935
nylon 6,polycaprolactam|thermoplastic polymer|f=(C6H11NO)n;d=1.14;mp=220;tg=47;ts=75;e=2.6;k=0.25
polytetrafluoroethylene,ptfe,teflon,dupont teflon|fluoropolymer|f=(C2F4)n;d=2.2;mp=327;ts=25;e=0.5;k=0.25;note=very low friction (coefficient 0.05-0.10); chemically inert; does not burn in air
poly(methyl methacrylate),pmma,acrylic,plexiglass,plexiglas,perspex,lucite,acrylic glass|thermoplastic polymer|f=(C5H8O2)n;d=1.18;tg=105;ts=70;e=3.2;k=0.19;hcm=26;n=1.49
polycarbonate,lexan|thermoplastic polymer|f=(C16H14O3)n;d=1.20;tg=147;ts=65;e=2.4;k=0.2;hcm=31;n=1.586
acrylonitrile butadiene styrene,abs,abs plastic|thermoplastic polymer|comp=copolymer of acrylonitrile (C₃H₃N), butadiene (C₄H₆) and styrene (C₈H₈);d=1.05;tg=105;ts=40;e=2.3;k=0.17;hcm=35
natural rubber,rubber,polyisoprene|elastomer|f=(C5H8)n;d=0.92;tg=-70;ts=25;hcm=45
silicone rubber,silicone,pdms,polydimethylsiloxane|elastomer|f=(C2H6OSi)n;d=0.97-1.2;tg=-125;ts=7;k=0.2
epoxy,epoxy resin|thermoset polymer|d=1.2;tg=120;ts=60;e=3.0;k=0.2
kevlar,aramid,kevlar 49|aramid fibre|f=(C14H10N2O2)n;d=1.44;ts=3600;e=112;note=para-aramid; decomposes near 450 °C without melting
cellulose|natural polymer|f=(C6H10O5)n;d=1.5;note=the main structural component of plant cell walls
cellulose acetate|thermoplastic polymer|d=1.3;tg=190;ts=50;e=2.0
portland cement,cement|construction material|comp=CaO 61-67 %, SiO₂ 19-23 %, Al₂O₃ 2.5-6 %, Fe₂O₃ 0-6 %, SO₃ 1.5-4.5 %;d=3.15;bulk=1440;note=a 94 lb bag holds about one cubic foot; reacts with water (hydration) to form calcium silicate hydrates
asphalt,asphalt concrete,blacktop,bitumen|construction material|d=2.36;k=0.75;r=0.19;note=asphalt concrete pavement; bitumen binder alone is about 1.03 g/cm³
brick|construction material|d=1.9;k=0.72;r=0.2
gypsum board,drywall,plasterboard|construction material|d=0.7;k=0.16;r=0.9
plywood|construction material|d=0.55;k=0.12;r=1.25
fiberglass insulation,fiberglass batt,glass wool|insulation|d=0.012-0.05;k=0.04;r=3.2
mineral wool,rock wool,rockwool|insulation|d=0.03-0.2;k=0.038;r=3.3
cellulose insulation,blown cellulose|insulation|d=0.04-0.06;k=0.04;r=3.5
spray foam,polyurethane foam,spray polyurethane foam|insulation|d=0.03;k=0.024;r=6.5
polyisocyanurate,polyiso|insulation|d=0.03;k=0.023;r=6.0
kaolin,kaolin clay,china clay|clay mineral|f=Al2Si2O5(OH)4;d=2.6;mohs=2-2.5;sys=triclinic;bulk=700;note=white clay made mostly of kaolinite; used for porcelain, paper coating and ceramics
dry sand,sand,loose sand|bulk material|bulk=1600;rep=34;note=dry, loose
wet sand|bulk material|bulk=1920;rep=45
gravel,natural gravel|bulk material|bulk=1680;rep=25-30;note=natural gravel with sand
crushed stone,crushed gravel,crushed rock|bulk material|bulk=1600;rep=45
topsoil,soil,loose earth,dirt|bulk material|bulk=1280;rep=30-45;note=loose, dry
clay|bulk material|bulk=1600;rep=25-40;note=dry lumps; wet excavated clay rests at about 15°
snow|bulk material|bulk=100-500;rep=38;note=fresh snow is about 100 kg/m³, settled snow 200-500 kg/m³
wheat,wheat grain|bulk material|bulk=790;rep=27
wheat flour,flour|bulk material|bulk=590;rep=45
ashes|bulk material|bulk=650;rep=40
anthracite coal,anthracite,hard coal|coal|d=1.5;bulk=865;hcm=30;comp=86-98 % carbon
bituminous coal,soft coal|coal|d=1.3;bulk=830;hcm=27;comp=45-86 % carbon
olivine,peridot|mineral|f=(Mg,Fe)2SiO4;sys=orthorhombic;mohs=6.5-7;d=3.27-4.37;col=olive green to yellow-green;note=a major mineral of Earth's upper mantle; gem variety peridot
gypsum|mineral|f=CaSO4·2H2O;sys=monoclinic;mohs=2;d=2.31-2.33;col=colourless, white, grey
talc|mineral|f=Mg3Si4O10(OH)2;sys=monoclinic;mohs=1;d=2.58-2.83;col=white, pale green
calcite|mineral|f=CaCO3;sys=trigonal;mohs=3;d=2.71;col=colourless, white
fluorite|mineral|f=CaF2;sys=cubic;mohs=4;d=3.18;col=purple, green, yellow, colourless
apatite|mineral|f=Ca5(PO4)3(F,Cl,OH);sys=hexagonal;mohs=5;d=3.16-3.22;col=green, blue, yellow
orthoclase,orthoclase feldspar|mineral|f=KAlSi3O8;sys=monoclinic;mohs=6;d=2.55-2.63;col=pink, white
plagioclase,feldspar|mineral|f=(Na,Ca)(Si,Al)4O8;sys=triclinic;mohs=6-6.5;d=2.62-2.76;col=white, grey
quartz|mineral|f=SiO2;sys=trigonal;mohs=7;d=2.65;col=colourless and many tints
topaz|mineral|f=Al2SiO4(F,OH)2;sys=orthorhombic;mohs=8;d=3.49-3.57;col=colourless, yellow, blue
corundum,ruby,sapphire|mineral|f=Al2O3;sys=trigonal;mohs=9;d=3.95-4.10;col=red (ruby), blue and other colours (sapphire)
pyrite,fool's gold|mineral|f=FeS2;sys=cubic;mohs=6-6.5;d=4.95-5.10;col=brass yellow, metallic
magnetite|mineral|f=Fe3O4;sys=cubic;mohs=5.5-6.5;d=5.17;col=black, metallic; strongly magnetic
hematite|mineral|f=Fe2O3;sys=trigonal;mohs=5.5-6.5;d=5.26;col=steel grey to red-brown; red streak
galena|mineral|f=PbS;sys=cubic;mohs=2.5-2.75;d=7.4-7.6;col=lead grey, metallic
halite,rock salt|mineral|f=NaCl;sys=cubic;mohs=2-2.5;d=2.17;col=colourless, white
muscovite,mica|mineral|f=KAl2(AlSi3O10)(F,OH)2;sys=monoclinic;mohs=2-2.5;d=2.76-2.88;col=colourless, silvery
biotite|mineral|f=K(Mg,Fe)3AlSi3O10(F,OH)2;sys=monoclinic;mohs=2.5-3;d=2.7-3.3;col=black, dark brown
garnet,almandine|mineral|f=Fe3Al2(SiO4)3;sys=cubic;mohs=6.5-7.5;d=3.1-4.3;col=deep red (almandine)
tourmaline|mineral|f=(Na,Ca)(Mg,Fe,Li,Al)3Al6(BO3)3Si6O18(OH)4;sys=trigonal;mohs=7-7.5;d=3.0-3.26;col=black (schorl) and many colours
beryl,emerald,aquamarine|mineral|f=Be3Al2Si6O18;sys=hexagonal;mohs=7.5-8;d=2.63-2.92;col=green (emerald), blue (aquamarine)
jadeite,jade|mineral|f=NaAlSi2O6;sys=monoclinic;mohs=6.5-7;d=3.24-3.43;col=green, white, lavender
nephrite|mineral|f=Ca2(Mg,Fe)5Si8O22(OH)2;sys=monoclinic;mohs=6-6.5;d=2.9-3.03;col=green, cream
opal|mineraloid|f=SiO2·nH2O;sys=amorphous;mohs=5.5-6.5;d=1.9-2.3;col=play of colour on white, black or orange
malachite|mineral|f=Cu2CO3(OH)2;sys=monoclinic;mohs=3.5-4;d=3.6-4.05;col=banded green
azurite|mineral|f=Cu3(CO3)2(OH)2;sys=monoclinic;mohs=3.5-4;d=3.77-3.89;col=deep blue
turquoise|mineral|f=CuAl6(PO4)4(OH)8·4H2O;sys=triclinic;mohs=5-6;d=2.6-2.9;col=blue to green
chalcopyrite|mineral|f=CuFeS2;sys=tetragonal;mohs=3.5-4;d=4.1-4.3;col=brass yellow, metallic
sphalerite,zinc blende|mineral|f=ZnS;sys=cubic;mohs=3.5-4;d=3.9-4.1;col=brown, black, yellow
kaolinite|mineral|f=Al2Si2O5(OH)4;sys=triclinic;mohs=2-2.5;d=2.16-2.68;col=white
dolomite|mineral|f=CaMg(CO3)2;sys=trigonal;mohs=3.5-4;d=2.84-2.86;col=white, pink
serpentine|mineral|f=(Mg,Fe)3Si2O5(OH)4;sys=monoclinic;mohs=2.5-4;d=2.5-2.6;col=green
asbestos,chrysotile,white asbestos|mineral|f=Mg3(Si2O5)(OH)4;sys=monoclinic;mohs=2.5-3;d=2.53-2.65;col=white fibres;note=asbestos is the commercial name of six fibrous silicate minerals; chrysotile (white asbestos), a serpentine, made up about 95 % of the asbestos used; the others are amphiboles such as crocidolite (blue) and amosite (brown)
spinel|mineral|f=MgAl2O4;sys=cubic;mohs=7.5-8;d=3.58-3.61;col=red, pink, blue
zircon|mineral|f=ZrSiO4;sys=tetragonal;mohs=7.5;d=4.6-4.7;col=colourless, brown, blue
cinnabar|mineral|f=HgS;sys=trigonal;mohs=2-2.5;d=8.1;col=scarlet red
barite,baryte|mineral|f=BaSO4;sys=orthorhombic;mohs=3-3.5;d=4.3-4.5;col=colourless, white
hope diamond|famous gem|ct=45.52;col=deep greyish blue;note=Smithsonian National Museum of Natural History, Washington, since 1958; type IIb diamond (boron gives the blue colour and red phosphorescence)
cullinan diamond|famous gem|ct=3106.75;note=the largest gem-quality rough diamond ever found (Premier Mine, South Africa, 1905); cut into nine major stones
great star of africa,cullinan i|famous gem|ct=530.2;col=colourless;note=the largest stone cut from the Cullinan diamond; set in the Sovereign's Sceptre with Cross (British Crown Jewels)
koh-i-noor,koh i noor,kohinoor|famous gem|ct=105.6;col=colourless;note=part of the British Crown Jewels, set in the Crown of Queen Elizabeth The Queen Mother
golden jubilee diamond|famous gem|ct=545.67;col=yellow-brown;note=the largest cut and faceted diamond in the world
delong star ruby|famous gem|ct=100.32;col=purplish red star ruby;note=from Burma (Myanmar); American Museum of Natural History, New York
star of india|famous gem|ct=563.35;col=greyish blue star sapphire;note=from Sri Lanka; American Museum of Natural History, New York
logan sapphire|famous gem|ct=422.99;col=blue;note=from Sri Lanka; Smithsonian National Museum of Natural History
regent diamond|famous gem|ct=140.64;col=colourless;note=Louvre, Paris
orlov diamond|famous gem|ct=189.62;col=colourless with a slight bluish-green tint;note=set in the Imperial Sceptre of Catherine the Great, Kremlin Diamond Fund
tiffany yellow diamond|famous gem|ct=128.54;col=yellow
dresden green diamond|famous gem|ct=41;col=green;note=the largest natural green diamond; Dresden Green Vault
olympic australis|famous gem|ct=17000;col=opal;note=the largest and most valuable gem opal ever found (Coober Pedy, Australia, 1956)
";

struct Item {
    names: Vec<&'static str>,
    kind: &'static str,
    props: Vec<(&'static str, &'static str)>,
}

fn data() -> &'static [Item] {
    static D: OnceLock<Vec<Item>> = OnceLock::new();
    D.get_or_init(|| {
        DATA.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let mut p = l.split('|');
                let names = p.next()?.split(',').map(str::trim).collect();
                let kind = p.next()?.trim();
                let props = p.next()?.split(';').filter_map(|kv| kv.split_once('=')).collect();
                Some(Item { names, kind, props })
            })
            .collect()
    })
}

fn find(name: &str) -> Option<&'static Item> {
    let l = name.trim().trim_start_matches("the ").trim();
    let l = l.trim_end_matches(" plastic").trim_end_matches(" mineral").trim_end_matches(" gemstone").trim_end_matches(" gem");
    // the comma in "nylon 6,6" survives list splitting as "nylon 6" + "6"
    data().iter().find(|x| x.names.iter().any(|n| *n == l))
}

impl Item {
    fn get(&self, k: &str) -> Option<&'static str> {
        self.props.iter().find(|(a, _)| *a == k).map(|(_, v)| *v)
    }
    fn title(&self) -> String {
        cap(self.names[0])
    }
    /// the representative number of a property (the middle of a range)
    fn num(&self, k: &str) -> Option<f64> {
        let v = self.get(k)?;
        match re!(r"^(-?[\d.]+)-(-?[\d.]+)$").captures(v) {
            Some(m) => Some((m[1].parse::<f64>().ok()? + m[2].parse::<f64>().ok()?) / 2.0),
            None => v.parse().ok(),
        }
    }
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// "a-b" → "a–b"; a minus sign at the start stays.
fn dash(v: &str) -> String {
    re!(r"(\d)-(-?\d)").replace_all(v, "$1–$2").to_string()
}

/// Pretty chemical formula: subscript digits that follow a letter or ')'.
fn formula(f: &str) -> String {
    let mut out = String::new();
    let mut prev = ' ';
    for c in f.chars() {
        if c.is_ascii_digit() && (prev.is_ascii_alphabetic() || prev == ')' || "₀₁₂₃₄₅₆₇₈₉".contains(prev)) && prev != '·' {
            out.push(char::from_u32(0x2080 + c.to_digit(10).unwrap()).unwrap());
            prev = out.chars().last().unwrap();
        } else {
            out.push(c);
            prev = c;
        }
    }
    out.replace(")n", ")ₙ")
}

const LABELS: &[(&str, &str)] = &[
    ("f", "chemical formula"),
    ("comp", "composition"),
    ("sys", "crystal system"),
    ("mohs", "Mohs hardness"),
    ("col", "colour"),
    ("ct", "mass"),
    ("d", "density"),
    ("bulk", "bulk density"),
    ("rep", "angle of repose"),
    ("mp", "melting point"),
    ("tg", "glass transition temperature"),
    ("ts", "tensile strength"),
    ("e", "Young's modulus"),
    ("k", "thermal conductivity"),
    ("r", "R-value"),
    ("n", "refractive index"),
    ("hcm", "heating value"),
];

fn value(x: &Item, k: &str) -> Option<String> {
    // a loose material's density is its bulk density
    if k == "d" && x.get("d").is_none() && x.get("bulk").is_some() {
        return value(x, "bulk");
    }
    let v = x.get(k)?;
    let d = dash(v);
    Some(match k {
        "f" => formula(v),
        "d" => match x.num("d") {
            Some(g) if !v.contains('-') => format!("{d} g/cm³ ({} kg/m³, {} lb/ft³)", n(g * 1000.0), n((g * 62.428 * 10.0).round() / 10.0)),
            _ => format!("{d} g/cm³"),
        },
        "bulk" => match x.num("bulk") {
            Some(b) if !v.contains('-') => format!("{d} kg/m³ ({} lb/ft³; {} short tons per cubic yard)", n((b * 0.062_428 * 10.0).round() / 10.0), n((b * 0.764_555 / 907.185 * 100.0).round() / 100.0)),
            _ => format!("{d} kg/m³"),
        },
        "rep" => format!("{d}°"),
        "mp" | "tg" => match x.num(k) {
            Some(c) if !v[1..].contains('-') => format!("{d} °C ({} °F)", n(c * 9.0 / 5.0 + 32.0)),
            _ => format!("{d} °C"),
        },
        "ts" => format!("{d} MPa"),
        "e" => format!("{d} GPa"),
        "k" => format!("{d} W/(m·K)"),
        "r" => format!("R-{d} per inch (ft²·°F·h/BTU; RSI {} m²·K/W per 25 mm)", n((x.num("r")? * 0.1761 * 1000.0).round() / 1000.0)),
        "hcm" => format!("about {d} MJ/kg"),
        "ct" => {
            let c = x.num("ct")?;
            format!("{d} carats ({} g, {} oz)", n(c * 0.2), n((c * 0.2 / 28.3495 * 1000.0).round() / 1000.0))
        }
        "mohs" => d,
        _ => v.to_string(),
    })
}

fn rows(x: &Item) -> Vec<Vec<Content>> {
    let mut r: Vec<Vec<Content>> = vec![vec![text("type"), text(x.kind)]];
    r.extend(LABELS.iter().filter_map(|(k, l)| value(x, k).map(|v| vec![text(*l), text(v)])));
    r
}

fn prop_key(p: &str) -> Option<&'static str> {
    Some(match p.trim() {
        "chemical formula" | "formula" | "molecular formula" | "chemical composition" => "f",
        "composition" => "comp",
        "crystal system" | "crystal structure" | "crystallography" => "sys",
        "mohs hardness" | "hardness" | "mohs scale" => "mohs",
        "colour" | "color" => "col",
        "mass" | "weight" | "carats" | "size" => "ct",
        "density" | "specific gravity" => "d",
        "bulk density" => "bulk",
        "angle of repose" | "repose angle" => "rep",
        "melting point" | "melting temperature" => "mp",
        "glass transition" | "glass transition temperature" | "tg" => "tg",
        "tensile strength" | "ultimate tensile strength" | "strength" => "ts",
        "young's modulus" | "youngs modulus" | "elastic modulus" | "modulus of elasticity" | "stiffness" => "e",
        "thermal conductivity" | "conductivity" => "k",
        "r-value" | "r value" | "rvalue" | "insulation value" | "thermal resistance" => "r",
        "heat of combustion" | "combustion heat" | "heating value" | "calorific value" | "energy content" => "hcm",
        _ => return None,
    })
}

const PROPS: &str = r"chemical formula|molecular formula|chemical composition|formula|composition|crystal system|crystal structure|crystallography|mohs hardness|mohs scale|hardness|colou?r|mass|weight|carats|size|density|specific gravity|bulk density|angle of repose|repose angle|melting point|melting temperature|glass transition(?: temperature)?|tg|(?:ultimate )?tensile strength|strength|young'?s modulus|elastic modulus|modulus of elasticity|stiffness|thermal conductivity|conductivity|r-? ?value|insulation value|thermal resistance|heat of combustion|combustion heat|heating value|calorific value|energy content";

fn split_items(s: &str) -> Vec<String> {
    // keep "nylon 6,6" whole
    let s = s.replace("6,6", "66");
    re!(r"\s*(?:,\s*(?:and\s+)?|/|\s+and\s+|\s+vs\.?\s+|\s+versus\s+)\s*").split(s.trim()).map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
}

fn property(key: &str, items: &[&'static Item]) -> Option<Answer> {
    let lab = LABELS.iter().find(|(k, _)| *k == key).map(|(_, l)| *l)?;
    let vals: Vec<(String, Option<String>)> = items.iter().map(|x| (x.title(), value(x, key))).collect();
    if vals.iter().all(|v| v.1.is_none()) {
        return None;
    }
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} | {lab}", vals.iter().map(|v| v.0.as_str()).collect::<Vec<_>>().join(", "))))];
    if items.len() == 1 {
        pods.push(primary(pod("Result", &cap(lab), text(vals[0].1.clone()?))));
        pods.push(pod("Properties", "Typical properties", table(None, rows(items[0]))));
    } else {
        let main = vals.iter().map(|(a, b)| format!("{a}: {}", b.clone().unwrap_or_else(|| "(not available)".into()))).collect::<Vec<_>>().join(" | ");
        pods.push(primary(pod("Result", &cap(lab), text(main))));
    }
    if let Some(note) = items.iter().find_map(|x| (items.len() == 1).then(|| x.get("note")).flatten()) {
        pods.push(pod("Note", "Note", text(dash(note))));
    }
    pods.push(pod("Source", "Note on values", text("typical values; real materials vary with grade, processing, moisture and compaction")));
    Some(Answer::new(format!("{lab} of {}", vals.iter().map(|v| v.0.clone()).collect::<Vec<_>>().join(", ")), pods))
}

fn card(items: &[&'static Item]) -> Option<Answer> {
    if items.len() == 1 {
        let x = items[0];
        let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} | {}", x.title(), x.kind))), primary(pod("Properties", "Typical properties", table(None, rows(x))))];
        if let Some(note) = x.get("note") {
            pods.push(pod("Note", "Note", text(dash(note))));
        }
        return Some(Answer::new(x.title(), pods));
    }
    let keys: Vec<(&str, &str)> = LABELS.iter().copied().filter(|(k, _)| items.iter().any(|x| x.get(k).is_some())).collect();
    let names: Vec<String> = items.iter().map(|x| x.title()).collect();
    let mut head = vec!["property"];
    head.extend(names.iter().map(String::as_str));
    let t: Vec<Vec<Content>> = keys.iter().map(|(k, l)| {
        let mut r = vec![text(*l)];
        r.extend(items.iter().map(|x| text(value(x, k).unwrap_or_else(|| "—".into()))));
        r
    }).collect();
    Some(Answer::new("material comparison", vec![pod("Input", "Input interpretation", text(names.join(" vs "))), primary(pod("Result", "Typical properties", table(Some(head), t)))]))
}

/// "35 cubic yards crushed stone", "1000 m^3 dry sand"
fn volume_mass(l: &str) -> Option<Answer> {
    let m = re!(r"^([\d.,]+)\s*(m\^?3|m³|cubic met(?:er|re)s?|cu\.? ?m|yd\^?3|yd³|cubic yards?|cu\.? ?yds?|ft\^?3|ft³|cubic f(?:ee|oo)t|cu\.? ?ft|l|liters?|litres?|gallons?|gal)\s+(?:of\s+)?(.+)$").captures(l)?;
    let x = find(&m[3])?;
    let v: f64 = m[1].replace(',', "").parse().ok()?;
    let unit = m[2].to_string();
    let m3 = v * match unit.as_str() {
        u if u.starts_with('m') || u.starts_with("cubic met") || u.starts_with("cu m") || u.starts_with("cu. m") || u.starts_with("cum") => 1.0,
        u if u.starts_with("yd") || u.contains("yard") || u.contains("yd") => 0.764_554_858,
        u if u.starts_with("ft") || u.contains("feet") || u.contains("foot") || u.contains("ft") => 0.028_316_846_6,
        u if u.starts_with('g') => 0.003_785_411_784,
        _ => 0.001,
    };
    let (rho, basis) = match (x.num("bulk"), x.num("d")) {
        (Some(b), _) => (b, "loose bulk density"),
        (None, Some(d)) => (d * 1000.0, "solid density"),
        _ => return None,
    };
    let kg = m3 * rho;
    let main = format!("{} t ({} short tons, {} lb)", n((kg / 1000.0 * 1000.0).round() / 1000.0), n((kg / 907.185 * 100.0).round() / 100.0), n((kg / 0.453_592_37).round()));
    Some(Answer::new(format!("mass of {} {} of {}", &m[1], unit, x.names[0]), vec![
        pod("Input", "Input interpretation", text(format!("{} {} of {}", &m[1], unit, x.title()))),
        primary(pod("Result", "Mass", text(main))),
        pod("Volume", "Volume", text(format!("{} m³ ({} yd³, {} ft³)", n(m3), n(m3 / 0.764_554_858), n(m3 / 0.028_316_846_6)))),
        pod("Basis", "Basis", text(format!("{basis} {} kg/m³ (typical)", n(rho)))),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = re!(r"^(?:what is |what are |what's |find |compare |show |give me )?(?:the )?").replace(&l, "").to_string();
    if let Some(a) = volume_mass(&l) {
        return Some(a);
    }
    static FRONT: OnceLock<Regex> = OnceLock::new();
    static BACK: OnceLock<Regex> = OnceLock::new();
    let front = FRONT.get_or_init(|| Regex::new(&format!(r"^({PROPS}) (?:of|for|in) (?:an? |the )?(.+)$")).unwrap());
    let back = BACK.get_or_init(|| Regex::new(&format!(r"^(.+?)(?:'s)? ({PROPS})$")).unwrap());
    let pair = front.captures(&l).map(|m| (m[1].to_string(), m[2].to_string())).or_else(|| back.captures(&l).map(|m| (m[2].to_string(), m[1].to_string())));
    if let Some((p, items)) = pair {
        if let Some(key) = prop_key(&p.replace("r value", "r-value").replace("rvalue", "r-value")) {
            let found: Option<Vec<&Item>> = split_items(&items).iter().map(|s| find(s)).collect();
            if let Some(found) = found.filter(|f| !f.is_empty() && f.len() <= 8) {
                return property(key, &found);
            }
        }
        return None;
    }
    // a bare material, mineral or gem, or a short list of them
    let items = split_items(&l);
    if items.is_empty() || items.len() > 6 {
        return None;
    }
    // abbreviations that mean other things on their own
    if items.len() == 1 && matches!(items[0].as_str(), "abs" | "pet" | "pete" | "pp" | "ps" | "pe" | "pc" | "eps" | "xps" | "tg") {
        return None;
    }
    let found: Option<Vec<&Item>> = items.iter().map(|s| find(s)).collect();
    match found {
        Some(f) => card(&f),
        None => super::fluids::mixture_card(&l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_parses() {
        assert_eq!(data().len(), DATA.lines().filter(|l| !l.trim().is_empty()).count());
        for x in data() {
            for k in ["d", "bulk", "mp", "tg", "ts", "e", "k", "r", "hcm", "ct", "rep"] {
                if x.get(k).is_some() {
                    assert!(x.num(k).is_some(), "{} {k}", x.names[0]);
                }
            }
        }
        assert_eq!(formula("CaSO4·2H2O"), "CaSO₄·2H₂O");
        assert_eq!(formula("(C2H4)n"), "(C₂H₄)ₙ");
    }
}
