//! The geologic time scale: eons, eras, periods and epochs with their
//! boundaries (International Chronostratigraphic Chart, rounded), the
//! divisions before and after, subdivisions and parents, durations, the
//! main events of each interval, and the major ice ages.

use super::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

#[derive(Clone, Copy, PartialEq)]
enum Rank {
    Eon,
    Era,
    Period,
    Epoch,
}

impl Rank {
    fn word(self) -> &'static str {
        match self {
            Rank::Eon => "eon",
            Rank::Era => "era",
            Rank::Period => "period",
            Rank::Epoch => "epoch",
        }
    }
}

struct Div {
    name: &'static str,
    rank: Rank,
    parent: &'static str,
    start: f64,
    end: f64,
    events: &'static str,
}

use Rank::*;

const fn d(name: &'static str, rank: Rank, parent: &'static str, start: f64, end: f64, events: &'static str) -> Div {
    Div { name, rank, parent, start, end, events }
}

const DIVS: &[Div] = &[
    d("Hadean", Eon, "", 4567.0, 4031.0, "Earth accretes; the Moon forms after a giant impact; oceans condense; the oldest known minerals (Jack Hills zircons, ≈ 4.4 Ga) crystallize"),
    d("Archean", Eon, "", 4031.0, 2500.0, "the first continental crust and the oldest surviving rocks; earliest life (stromatolites by ≈ 3.5 Ga); an atmosphere with almost no free oxygen"),
    d("Proterozoic", Eon, "", 2500.0, 538.8, "the Great Oxidation Event (≈ 2.4 Ga); first eukaryotes; supercontinents Columbia and Rodinia; Snowball Earth glaciations; the Ediacaran biota"),
    d("Phanerozoic", Eon, "", 538.8, 0.0, "abundant animal life with hard parts, beginning with the Cambrian explosion; five major mass extinctions; colonization of the land"),
    d("Eoarchean", Era, "Archean", 4031.0, 3600.0, "oldest preserved rocks (Acasta Gneiss, Isua belt); possible earliest life"),
    d("Paleoarchean", Era, "Archean", 3600.0, 3200.0, "oldest widely accepted stromatolites and microfossils"),
    d("Mesoarchean", Era, "Archean", 3200.0, 2800.0, "growth of cratons; possible first supercontinent (Vaalbara break-up)"),
    d("Neoarchean", Era, "Archean", 2800.0, 2500.0, "oxygenic photosynthesis by cyanobacteria becomes widespread"),
    d("Paleoproterozoic", Era, "Proterozoic", 2500.0, 1600.0, "the Great Oxidation Event and the Huronian glaciation; banded iron formations peak; the supercontinent Columbia (Nuna) assembles; first eukaryotes"),
    d("Mesoproterozoic", Era, "Proterozoic", 1600.0, 1000.0, "the 'boring billion' of stable climate; Columbia breaks up and Rodinia assembles; sexual reproduction and multicellular algae appear"),
    d("Neoproterozoic", Era, "Proterozoic", 1000.0, 538.8, "Rodinia breaks up; Snowball Earth glaciations (Sturtian and Marinoan); the Ediacaran biota, the first large complex organisms"),
    d("Paleozoic", Era, "Phanerozoic", 538.8, 251.902, "begins with the Cambrian explosion; life moves onto land (plants, arthropods, tetrapods); Pangaea assembles; ends with the Permian–Triassic extinction"),
    d("Mesozoic", Era, "Phanerozoic", 251.902, 66.0, "the age of reptiles: dinosaurs, first mammals and birds, flowering plants; Pangaea breaks up; begins after the Permian–Triassic extinction and ends with the Cretaceous–Paleogene extinction (Chicxulub impact)"),
    d("Cenozoic", Era, "Phanerozoic", 66.0, 0.0, "the age of mammals: radiation of mammals and birds, grasslands spread, Antarctic and northern ice sheets form, humans evolve"),
    d("Siderian", Period, "Paleoproterozoic", 2500.0, 2300.0, "banded iron formations peak; the Great Oxidation Event begins; Huronian glaciation"),
    d("Rhyacian", Period, "Paleoproterozoic", 2300.0, 2050.0, "Bushveld igneous complex; Huronian glaciation ends"),
    d("Orosirian", Period, "Paleoproterozoic", 2050.0, 1800.0, "Vredefort and Sudbury impacts; widespread mountain building"),
    d("Statherian", Period, "Paleoproterozoic", 1800.0, 1600.0, "Columbia (Nuna) supercontinent; early eukaryotes"),
    d("Calymmian", Period, "Mesoproterozoic", 1600.0, 1400.0, "platform covers expand on new continental crust"),
    d("Ectasian", Period, "Mesoproterozoic", 1400.0, 1200.0, "platform covers continue to expand"),
    d("Stenian", Period, "Mesoproterozoic", 1200.0, 1000.0, "Grenville orogeny; Rodinia assembles"),
    d("Tonian", Period, "Neoproterozoic", 1000.0, 720.0, "Rodinia begins to break up; early animals possibly evolve"),
    d("Cryogenian", Period, "Neoproterozoic", 720.0, 635.0, "global (Snowball Earth) glaciations: Sturtian and Marinoan"),
    d("Ediacaran", Period, "Neoproterozoic", 635.0, 538.8, "the Ediacaran biota, the first large multicellular organisms; Gaskiers glaciation"),
    d("Cambrian", Period, "Paleozoic", 538.8, 485.4, "the Cambrian explosion: most animal phyla appear; trilobites; Gondwana lies in the south; warm climate with high sea levels"),
    d("Ordovician", Period, "Paleozoic", 485.4, 443.8, "the Great Ordovician Biodiversification Event; first land plants; ends with the Late Ordovician mass extinction during the Hirnantian glaciation"),
    d("Silurian", Period, "Paleozoic", 443.8, 419.2, "recovery from the Ordovician extinction; first vascular land plants and jawed fish; warm climate with high sea levels"),
    d("Devonian", Period, "Paleozoic", 419.2, 358.9, "the age of fishes; first forests and seed plants; first tetrapods; ends with the Late Devonian extinctions"),
    d("Carboniferous", Period, "Paleozoic", 358.9, 298.9, "coal swamp forests; atmospheric oxygen near 35%; giant arthropods; first amniotes (reptiles); the Karoo ice age begins"),
    d("Permian", Period, "Paleozoic", 298.9, 251.902, "Pangaea fully assembled; synapsids and reptiles spread; ends with the Permian–Triassic extinction (Siberian Traps), the largest mass extinction (≈ 90% of marine species)"),
    d("Triassic", Period, "Mesozoic", 251.902, 201.4, "recovery from the end-Permian extinction; first dinosaurs, pterosaurs and mammals; ends with the Triassic–Jurassic extinction (Central Atlantic Magmatic Province)"),
    d("Jurassic", Period, "Mesozoic", 201.4, 145.0, "dinosaurs dominate the land; first birds (Archaeopteryx); Pangaea breaks into Laurasia and Gondwana"),
    d("Cretaceous", Period, "Mesozoic", 145.0, 66.0, "flowering plants spread; very high sea levels and a warm climate; ends with the Cretaceous–Paleogene extinction (Chicxulub impact), which kills the non-avian dinosaurs"),
    d("Paleogene", Period, "Cenozoic", 66.0, 23.03, "mammals radiate; Paleocene–Eocene Thermal Maximum (56 Ma); Antarctic glaciation begins (≈ 34 Ma)"),
    d("Neogene", Period, "Cenozoic", 23.03, 2.58, "grasslands spread; hominins appear; the Isthmus of Panama closes"),
    d("Quaternary", Period, "Cenozoic", 2.58, 0.0, "repeated glacial cycles of the Quaternary ice age; the genus Homo and modern humans; megafaunal extinctions"),
    d("Terreneuvian", Epoch, "Cambrian", 538.8, 521.0, "first small shelly fossils and burrows"),
    d("Cambrian Series 2", Epoch, "Cambrian", 521.0, 509.0, "first trilobites; Chengjiang biota"),
    d("Miaolingian", Epoch, "Cambrian", 509.0, 497.0, "Burgess Shale biota"),
    d("Furongian", Epoch, "Cambrian", 497.0, 485.4, "trilobite extinctions (biomeres)"),
    d("Early Ordovician", Epoch, "Ordovician", 485.4, 470.0, "biodiversification begins"),
    d("Middle Ordovician", Epoch, "Ordovician", 470.0, 458.4, "Ordovician meteor event (L-chondrite parent body break-up)"),
    d("Late Ordovician", Epoch, "Ordovician", 458.4, 443.8, "Hirnantian glaciation and the Late Ordovician mass extinction"),
    d("Llandovery", Epoch, "Silurian", 443.8, 433.4, "recovery of marine life after the end-Ordovician extinction; sea levels rise as ice melts"),
    d("Wenlock", Epoch, "Silurian", 433.4, 427.4, "reef building; Ireviken event"),
    d("Ludlow", Epoch, "Silurian", 427.4, 423.0, "Lau event extinction"),
    d("Pridoli", Epoch, "Silurian", 423.0, 419.2, "early vascular plants (Cooksonia) spread"),
    d("Early Devonian", Epoch, "Devonian", 419.2, 393.3, "early land plants diversify"),
    d("Middle Devonian", Epoch, "Devonian", 393.3, 382.7, "first trees and forests"),
    d("Late Devonian", Epoch, "Devonian", 382.7, 358.9, "first tetrapods; Kellwasser and Hangenberg extinction events"),
    d("Mississippian", Epoch, "Carboniferous", 358.9, 323.2, "extensive shallow seas and limestone deposits (subperiod)"),
    d("Pennsylvanian", Epoch, "Carboniferous", 323.2, 298.9, "coal swamps; first reptiles (subperiod)"),
    d("Cisuralian", Epoch, "Permian", 298.9, 273.01, "Karoo glaciation wanes"),
    d("Guadalupian", Epoch, "Permian", 273.01, 259.51, "ends with the Capitanian extinction (Emeishan Traps)"),
    d("Lopingian", Epoch, "Permian", 259.51, 251.902, "biodiversity collapses at its end in the Permian–Triassic extinction, driven by the Siberian Traps eruptions: about 90% of marine and 70% of land vertebrate species vanish"),
    d("Early Triassic", Epoch, "Triassic", 251.902, 247.2, "slow recovery from the end-Permian extinction; very hot climate"),
    d("Middle Triassic", Epoch, "Triassic", 247.2, 237.0, "marine reptiles and archosaurs diversify"),
    d("Late Triassic", Epoch, "Triassic", 237.0, 201.4, "first dinosaurs and mammals; Carnian Pluvial Episode; ends with the Triassic–Jurassic extinction"),
    d("Early Jurassic", Epoch, "Jurassic", 201.4, 174.7, "dinosaurs diversify after the end-Triassic extinction; Toarcian oceanic anoxic event"),
    d("Middle Jurassic", Epoch, "Jurassic", 174.7, 161.5, "the Atlantic begins to open"),
    d("Late Jurassic", Epoch, "Jurassic", 161.5, 145.0, "giant sauropods; first birds (Archaeopteryx)"),
    d("Early Cretaceous", Epoch, "Cretaceous", 145.0, 100.5, "first flowering plants"),
    d("Late Cretaceous", Epoch, "Cretaceous", 100.5, 66.0, "highest sea levels of the Phanerozoic; ends with the Chicxulub impact"),
    d("Paleocene", Epoch, "Paleogene", 66.0, 56.0, "recovery after the K–Pg extinction; mammals diversify"),
    d("Eocene", Epoch, "Paleogene", 56.0, 33.9, "begins with the Paleocene–Eocene Thermal Maximum; modern mammal orders appear"),
    d("Oligocene", Epoch, "Paleogene", 33.9, 23.03, "Antarctic ice sheet forms; grasslands begin to spread"),
    d("Miocene", Epoch, "Neogene", 23.03, 5.333, "apes diversify; grasslands and kelp forests spread"),
    d("Pliocene", Epoch, "Neogene", 5.333, 2.58, "hominins such as Australopithecus; the Isthmus of Panama forms"),
    d("Pleistocene", Epoch, "Quaternary", 2.58, 0.0117, "repeated glaciations; Homo erectus to Homo sapiens; megafauna"),
    d("Holocene", Epoch, "Quaternary", 0.0117, 0.0, "the present interglacial; agriculture and civilization"),
];

/// (names, display, start Ma, end Ma, note)
const ICE_AGES: &[(&[&str], &str, f64, f64, &str)] = &[
    (&["huronian glaciation", "huronian ice age"], "Huronian glaciation", 2400.0, 2100.0, "possibly global; tied to the Great Oxidation Event removing methane"),
    (&["sturtian glaciation", "sturtian ice age"], "Sturtian glaciation", 717.0, 660.0, "a Snowball Earth episode of the Cryogenian"),
    (&["marinoan glaciation", "marinoan ice age"], "Marinoan glaciation", 650.0, 635.0, "a Snowball Earth episode ending the Cryogenian"),
    (&["andean-saharan glaciation", "andean saharan glaciation", "andean-saharan ice age", "hirnantian glaciation"], "Andean–Saharan glaciation", 460.0, 420.0, "peaks in the Hirnantian with the Late Ordovician extinction"),
    (&["karoo ice age", "karoo glaciation", "late paleozoic ice age"], "Karoo ice age", 360.0, 260.0, "ice sheets on southern Gondwana as Pangaea assembled; continents clustered into Pangaea with Gondwana over the South Pole"),
    (&["quaternary ice age", "quaternary glaciation", "pleistocene ice age", "late cenozoic ice age", "current ice age"], "Quaternary ice age", 2.58, 0.0, "ongoing: about 50 glacial–interglacial cycles, 41-kyr then 100-kyr; global temperatures in glacials ≈ 4–7 °C below today"),
    (&["last glacial period", "last glacial episode", "last ice age", "last glaciation", "last glacial maximum"], "last glacial period", 0.115, 0.0117, "the last glacial maximum was ≈ 26,500–19,000 years ago, with sea level ≈ 120 m lower than today"),
];

fn fmt_ma(ma: f64) -> String {
    if ma == 0.0 {
        "present".to_string()
    } else if ma < 1.0 {
        format!("{} years ago", crate::print::fmt_float((ma * 1e6).round(), 8))
    } else if ma >= 1000.0 {
        format!("{} Ga", crate::print::fmt_float(ma / 1000.0, 5))
    } else {
        format!("{} Ma", crate::print::fmt_float(ma, 6))
    }
}

fn span(ma: f64) -> String {
    if ma < 1.0 {
        format!("{} years", crate::print::fmt_float((ma * 1e6).round(), 8))
    } else {
        format!("{} million years", crate::print::fmt_float((ma * 1000.0).round() / 1000.0, 6))
    }
}

fn find(name: &str) -> Option<&'static Div> {
    let n = name.trim().trim_start_matches("the ");
    let n = re!(r"\s+(eon|era|period|epoch|series|subperiod)$").replace(n, "");
    DIVS.iter().find(|d| d.name.to_lowercase() == n.trim())
}

fn row(d: &Div) -> Vec<Content> {
    vec![text(d.name), text(d.rank.word()), text(fmt_ma(d.start)), text(fmt_ma(d.end)), text(span(d.start - d.end))]
}

fn children(d: &Div) -> Vec<&'static Div> {
    DIVS.iter().filter(|c| c.parent == d.name).collect()
}

fn siblings(d: &Div) -> Vec<&'static Div> {
    // same rank, ordered oldest first
    let mut v: Vec<&Div> = DIVS.iter().filter(|c| c.rank == d.rank).collect();
    v.sort_by(|a, b| b.start.partial_cmp(&a.start).unwrap());
    v
}

fn full(d: &'static Div, title: &str, main: String) -> Answer {
    let mut chain = vec![d.name.to_string()];
    let mut cur = d;
    while let Some(p) = DIVS.iter().find(|p| p.name == cur.parent) {
        chain.push(format!("{} {}", p.name, p.rank.word()));
        cur = p;
    }
    let mut pods = vec![
        pod("Input", "Input interpretation", text(format!("{} {} (geologic time)", d.name, d.rank.word()))),
        primary(pod("Result", title, text(main))),
        pod("Interval", "Time interval", table(Some(vec!["division", "rank", "start", "end", "duration"]), vec![row(d)])),
        pod("Events", "Major events", text(d.events)),
        pod("Hierarchy", "Hierarchy", text(chain.join(" ⊂ "))),
    ];
    let kids = children(d);
    if !kids.is_empty() {
        pods.push(pod("Subdivisions", "Subdivisions", table(Some(vec!["division", "rank", "start", "end", "duration"]), kids.iter().map(|k| row(k)).collect())));
    }
    Answer::new("geologic time", pods)
}

fn ice(l: &str) -> Option<Answer> {
    if re!(r"(major|all|list of|geological time periods of the)\s.*ice ages|^ice ages$|^glaciations$").is_match(l) {
        let rows = ICE_AGES.iter().filter(|x| x.1 != "last glacial period").map(|(_, n, s, e, _)| {
            let per: Vec<&str> = DIVS.iter().filter(|d| d.rank == Period && d.start > *e && d.end < *s).map(|d| d.name).collect();
            vec![text(*n), text(format!("{} – {}", fmt_ma(*s), fmt_ma(*e))), text(per.join(", "))]
        }).collect();
        return Some(Answer::new("ice ages", vec![pod("Input", "Input interpretation", text("major ice ages")), primary(pod("Result", "Major ice ages", table(Some(vec!["ice age", "interval", "geologic periods"]), rows)))]));
    }
    let m = re!(r"^(?:(?:the\s+)?(temperature anomaly|duration|length|time span|map|continental map|when was)\s+(?:during|of|for)?\s*)?(?:the\s+)?(.+?)$").captures(l)?;
    let name = m[2].trim();
    let (_, disp, s, e, note) = ICE_AGES.iter().find(|x| x.0.contains(&name))?;
    let per: Vec<&str> = DIVS.iter().filter(|d| d.rank == Period && d.start > *e && d.end < *s).map(|d| d.name).collect();
    let rows = vec![
        vec![text("interval"), text(format!("{} – {}", fmt_ma(*s), fmt_ma(*e)))],
        vec![text("duration"), text(if *e == 0.0 { format!("{} so far", span(s - e)) } else { span(s - e) })],
        vec![text("geologic periods"), text(per.join(", "))],
        vec![text("notes"), text(*note)],
    ];
    let (title, main) = match m.get(1).map(|x| x.as_str()) {
        Some("temperature anomaly") => ("Climate", note.to_string()),
        Some("duration" | "length" | "time span") => ("Duration", span(s - e)),
        _ => ("Interval", format!("{} – {}", fmt_ma(*s), fmt_ma(*e))),
    };
    Some(Answer::new("ice age", vec![pod("Input", "Input interpretation", text(*disp)), primary(pod("Result", title, text(main))), pod("Details", "Details", table(None, rows))]))
}

/// Approximate atmosphere and climate of the periods, from geochemical models
/// (GEOCARB-type estimates): (period, O₂ vol %, CO₂ ppm, mean surface
/// temperature °C, sea level relative to today).
const PERIOD_CLIMATE: &[(&str, f64, f64, f64, &str)] = &[
    ("Cambrian", 12.5, 4500.0, 21.0, "about 4 m rising to 90 m above today"),
    ("Ordovician", 13.5, 4200.0, 16.0, "about 180 m above today, rising to 220 m, then falling sharply to 140 m in the end-Ordovician glaciation"),
    ("Silurian", 14.0, 4500.0, 17.0, "about 180 m above today, with short-term falls"),
    ("Devonian", 15.0, 2200.0, 20.0, "about 189 m above today, falling to 120 m"),
    ("Carboniferous", 32.5, 800.0, 14.0, "falling from about 120 m above today to near today's level"),
    ("Permian", 23.0, 900.0, 16.0, "about 60 m above today, falling through the period"),
    ("Triassic", 16.0, 1750.0, 17.0, "relatively low, near today's level"),
    ("Jurassic", 26.0, 1950.0, 16.5, "rising through the period, to about 140 m above today"),
    ("Cretaceous", 30.0, 1700.0, 18.0, "high, about 100–250 m above today"),
    ("Paleogene", 26.0, 500.0, 18.0, "falling from about 150 m above today"),
    ("Neogene", 21.5, 280.0, 14.0, "falling toward today's level"),
    ("Quaternary", 20.8, 250.0, 14.0, "swinging about 120 m below today in glacial maxima"),
];

fn period_climate(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:what (?:is|was|were) )?(?:the )?(?:mean |average )?(oxygen|o2|co2|carbon dioxide|sea level|temperature|surface temperature|climate|atmosphere)(?: levels?| contents?| concentrations?)? (?:during|in|of|for) (?:the )?([a-z]+)(?: period)?$").captures(l)?;
    let (name, o2, co2, t, sea) = PERIOD_CLIMATE.iter().find(|x| x.0.eq_ignore_ascii_case(&m[2]))?;
    let (title, main) = match &m[1] {
        "oxygen" | "o2" => ("Mean atmospheric oxygen", format!("about {o2}% by volume (today 20.9%)")),
        "co2" | "carbon dioxide" => ("Mean atmospheric CO₂", format!("about {co2} ppm (about {:.1} × the pre-industrial 280 ppm)", co2 / 280.0)),
        "sea level" => ("Sea level", sea.to_string()),
        "temperature" | "surface temperature" => ("Mean surface temperature", format!("about {t} °C (today about 14 °C)")),
        _ => ("Climate", format!("O₂ about {o2}%, CO₂ about {co2} ppm, mean surface temperature about {t} °C")),
    };
    Some(Answer::new(format!("{name} {}", m[1].to_string()), vec![
        pod("Input", "Input interpretation", text(format!("{name} Period | {}", title.to_lowercase()))),
        primary(pod("Result", title, text(main))),
        pod("Table", "Atmosphere and climate", table(None, vec![
            vec![text("oxygen"), text(format!("about {o2}% by volume"))],
            vec![text("carbon dioxide"), text(format!("about {co2} ppm"))],
            vec![text("mean surface temperature"), text(format!("about {t} °C"))],
            vec![text("sea level"), text(*sea)],
        ])),
        pod("Note", "Note", text("model estimates with large uncertainties (GEOCARB-type reconstructions)")),
    ]))
}

/// "names of the geological rock layers associated with the Quaternary Period and the Ordovician Period"
fn rock_systems(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(?:the )?names? of )?(?:the )?(?:geologic(?:al)? )?(?:rock layers?|rock units?|strata|stratigraphic units?|systems?)(?: (?:associated with|of|for|from|deposited (?:in|during)|formed (?:in|during)))? (?:the )?(.+)$").captures(l)?;
    let parts: Vec<&str> = re!(r"\s*(?:,\s*(?:and\s+)?(?:the\s+)?|\s+and\s+(?:the\s+)?)").split(&m[1]).filter(|p| !p.trim().is_empty()).collect();
    let mut rows = Vec::new();
    let mut names = Vec::new();
    for p in parts {
        let p = p.trim().trim_end_matches(" periods").trim_end_matches(" period");
        let d = find(p).filter(|d| d.rank == Period)?;
        let series: Vec<String> = children(d).iter().map(|c| format!("{} Series", c.name.replace("Early ", "Lower ").replace("Late ", "Upper "))).collect();
        names.push(format!("{} System", d.name));
        rows.push(vec![text(format!("{} Period", d.name)), text(format!("{} System", d.name)), text(series.join(", "))]);
    }
    if rows.is_empty() {
        return None;
    }
    Some(Answer::new("rock systems", vec![
        pod("Input", "Input interpretation", text("rock layers (chronostratigraphic systems) of the periods")),
        primary(pod("Result", "Rock layers", text(names.join(" | ")))),
        pod("Table", "Systems and series", table(Some(vec!["period (time)", "system (rocks)", "series"]), rows)),
        pod("Note", "Note", text("rocks laid down during a period form a system; those of an epoch form a series, and of an age a stage")),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.clean.trim().trim_end_matches(['?', '.']).to_lowercase();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    if let Some(a) = ice(&l).or_else(|| period_climate(&l)).or_else(|| rock_systems(&l)) {
        return Some(a);
    }
    // a moment in deep time: which eon, era, period and epoch
    if let Some(m) = re!(r"^(?:geologic(?:al)? (?:time|period|era|epoch) )?(\d+(?:\.\d+)?) (thousand|million|billion) years ago$").captures(&l) {
        let v: f64 = m[1].parse().ok()?;
        let ma = v * match &m[2] { "thousand" => 1e-3, "million" => 1.0, _ => 1e3 };
        let mut rows = Vec::new();
        let mut names = Vec::new();
        for rank in [Eon, Era, Period, Epoch] {
            if let Some(d) = DIVS.iter().find(|d| d.rank == rank && d.start >= ma && d.end <= ma) {
                rows.push(vec![text(rank.word()), text(d.name), text(format!("{} – {}", fmt_ma(d.start), fmt_ma(d.end)))]);
                names.push(format!("{} {}", d.name, rank.word()));
            }
        }
        if rows.is_empty() {
            return None;
        }
        let main = names.iter().rev().cloned().collect::<Vec<_>>().join(", ");
        return Some(Answer::new(
            "geologic time",
            vec![
                pod("Input", "Input interpretation", text(format!("{} years ago", m[0].trim_end_matches(" years ago").trim_start_matches("geological ").trim_start_matches("geologic ")))),
                primary(pod("Result", "Geologic time", text(main))),
                pod("Divisions", "Divisions containing that time", table(Some(vec!["rank", "name", "interval"]), rows)),
            ],
        ));
    }
    // the eras and what started them
    if re!(r"^(?:events that (?:started|began|ended) the )?geologic(?:al)? (eras|eons|periods|epochs|time scale|timescale)$").is_match(&l) {
        let rank = if l.contains("eon") { Eon } else if l.contains("period") { Period } else if l.contains("epoch") { Epoch } else { Era };
        let rows = DIVS.iter().filter(|d| d.rank == rank).map(|d| vec![text(d.name), text(fmt_ma(d.start)), text(fmt_ma(d.end)), text(d.events)]).collect();
        return Some(Answer::new("geologic time", vec![pod("Input", "Input interpretation", text(format!("geologic {}s", rank.word()))), primary(pod("Result", "Divisions", table(Some(vec!["name", "start", "end", "events"]), rows)))]));
    }
    let pats: [(&Regex, &str); 7] = [
        (re!(r"^how long did (?:the )?(.+?) last$"), "duration"),
        (re!(r"^(?:duration|length) of (?:the )?(.+)$"), "duration"),
        (re!(r"^what came (?:after|next after) (?:the )?(.+)$"), "after"),
        (re!(r"^what came before (?:the )?(.+)$"), "before"),
        (re!(r"^(?:subdivisions|divisions|epochs|periods|eras|stages) (?:of|in) (?:the )?(.+)$"), "children"),
        (re!(r"^(?:the )?(.+?) parent(?: division)?$"), "parent"),
        (re!(r"^(?:what did the world look like(?: during)?|what happened|what events (?:ended|started|began)|(?:important )?(?:geological |biological )?events (?:during|of|in|that ended|that started)|biodiversity during|important (?:geological|biological) events (?:during|of)|life during|when (?:was|did)|when was) (?:the )?(.+?)(?: (?:start|end|begin|happen))?$"), "events"),
    ];
    for (r, kind) in pats {
        let Some(m) = r.captures(&l) else { continue };
        let Some(d) = find(&m[1]) else { continue };
        return Some(match kind {
            "duration" => full(d, "Duration", span(d.start - d.end)),
            "after" | "before" => {
                let sib = siblings(d);
                let i = sib.iter().position(|x| x.name == d.name)?;
                let other = if kind == "after" { sib.get(i + 1) } else { i.checked_sub(1).and_then(|j| sib.get(j)) };
                match other {
                    Some(o) => {
                        let mut a = full(o, if kind == "after" { "Next division" } else { "Previous division" }, format!("{} {} ({} – {})", o.name, o.rank.word(), fmt_ma(o.start), fmt_ma(o.end)));
                        a.pods[0] = pod("Input", "Input interpretation", text(format!("{} immediately after the {} {}", o.rank.word(), d.name, d.rank.word())));
                        if kind == "before" {
                            a.pods[0] = pod("Input", "Input interpretation", text(format!("{} immediately before the {} {}", o.rank.word(), d.name, d.rank.word())));
                        }
                        a
                    }
                    None => full(d, "Result", if kind == "after" { "it extends to the present".into() } else { "it is the first division of its rank".into() }),
                }
            }
            "children" => {
                let kids = children(d);
                if kids.is_empty() {
                    return None;
                }
                full(d, "Subdivisions", kids.iter().map(|k| k.name).collect::<Vec<_>>().join(", "))
            }
            "parent" => {
                let p = DIVS.iter().find(|p| p.name == d.parent)?;
                full(d, "Parent division", format!("{} {}", p.name, p.rank.word()))
            }
            _ => full(d, "Major events", d.events.to_string()),
        });
    }
    // a bare name, with or without its rank
    let m = re!(r"^(?:the )?([a-z ]+?)(?: (eon|era|period|epoch|series|subperiod))?$").captures(&l)?;
    let d = find(&m[1])?;
    if let Some(r) = m.get(2) {
        let ok = r.as_str() == d.rank.word() || (d.rank == Epoch && matches!(r.as_str(), "series" | "subperiod"));
        if !ok {
            return None;
        }
    }
    Some(full(d, "Time interval", format!("{} to {}", fmt_ma(d.start), fmt_ma(d.end))))
}

#[cfg(test)]
mod tests {
    fn ans(s: &str) -> String {
        serde_json::to_string(&crate::engine::query(s).pods).unwrap_or_default()
    }

    #[test]
    fn geologic_time() {
        assert!(ans("What came after the Hadean eon?").contains("Archean"));
        assert!(ans("How long did the Paleoproterozoic Era last?").contains("900 million years"));
        assert!(ans("subdivisions of the Devonian Period").contains("Late Devonian"));
        assert!(ans("Tonian Period parent division").contains("Neoproterozoic"));
        assert!(ans("Llandovery Epoch").contains("443.8 Ma"));
    }
}
