//! Measuring instruments and sensors: what each measures, how it works,
//! other names, and the instruments for a given quantity.
//!
//! "hydrometer", "what does a ruler measure?", "a bubble chamber measures",
//! "alternative names for an hourglass", "device used to measure time",
//! "acceleration measurement devices", "thermometer types",
//! "instances of a particle detector".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names (first is the display name; `~` = not taken alone, it reads first as
/// an ordinary word) | quantities measured | category | how it works
const INSTRUMENTS: &[(&str, &str, &str, &str)] = &[
    ("thermometer", "temperature", "thermometer", "measures temperature, usually from the thermal expansion of a liquid or the change of an electrical property with temperature"),
    ("mercury thermometer;mercury-in-glass thermometer", "temperature", "thermometer", "liquid-in-glass thermometer using the thermal expansion of mercury; usable from about −38 °C (the freezing point of mercury) to about 350 °C"),
    ("alcohol thermometer;spirit thermometer", "temperature", "thermometer", "liquid-in-glass thermometer filled with dyed ethanol; usable down to about −110 °C (ethanol freezes at −114 °C) and up to about 78 °C"),
    ("bimetallic strip thermometer;bimetallic thermometer;dial thermometer;bimetallic strip", "temperature", "thermometer", "two bonded metal strips with different thermal expansion bend as the temperature changes, turning a pointer or switching a contact (thermostats)"),
    ("thermocouple", "temperature", "thermometer", "a junction of two different metals produces a voltage that depends on temperature (Seebeck effect); type K covers about −200 °C to 1,260 °C"),
    ("resistance thermometer;rtd;resistance temperature detector;platinum resistance thermometer;pt100", "temperature", "thermometer", "the electrical resistance of a platinum wire rises with temperature; a Pt100 element has 100 Ω at 0 °C"),
    ("thermistor", "temperature", "thermometer", "semiconductor resistor whose resistance changes strongly with temperature (usually falling as it warms)"),
    ("infrared thermometer;pyrometer;non-contact thermometer;radiation thermometer", "temperature", "thermometer", "measures the thermal radiation emitted by a surface, without touching it"),
    ("gas thermometer;constant-volume gas thermometer", "temperature", "thermometer", "the pressure of a gas held at constant volume is proportional to its absolute temperature"),
    ("liquid crystal thermometer;strip thermometer", "temperature", "thermometer", "cholesteric liquid crystals change colour with temperature"),
    ("barometer", "atmospheric pressure;pressure", "pressure gauge", "measures air pressure, with a column of mercury or a sealed metal capsule (aneroid barometer)"),
    ("aneroid barometer", "atmospheric pressure;pressure", "pressure gauge", "a sealed, partly evacuated metal capsule flexes as the air pressure changes"),
    ("manometer", "pressure;gas pressure", "pressure gauge", "a U-tube of liquid whose difference in levels gives the pressure relative to a reference"),
    ("pressure gauge;bourdon gauge", "pressure", "pressure gauge", "a curved tube (Bourdon tube) straightens under pressure and moves a pointer"),
    ("sphygmomanometer;blood pressure monitor;blood pressure cuff", "blood pressure;pressure", "medical instrument", "an inflatable cuff and a pressure gauge give the systolic and diastolic arterial pressure"),
    ("hygrometer;psychrometer", "humidity", "weather instrument", "measures the water vapour in air, for instance from the difference between a wet-bulb and a dry-bulb thermometer"),
    ("anemometer", "wind speed;speed", "weather instrument", "rotating cups, a propeller, a heated wire or sound pulses give the speed of the wind"),
    ("~wind vane;weather vane;weathervane", "wind direction;direction", "weather instrument", "a pivoted vane turns to point into the wind"),
    ("rain gauge;pluviometer;udometer", "rainfall;precipitation", "weather instrument", "collects rain in a calibrated container"),
    ("seismometer;seismograph", "ground motion;earthquakes;vibration", "geophysical instrument", "a suspended mass stays still while the ground moves, recording the motion"),
    ("accelerometer", "acceleration;vibration", "motion sensor", "a proof mass on a spring (in phones a microscopic MEMS structure) is displaced in proportion to acceleration"),
    ("gravimeter", "gravitational acceleration;acceleration;gravity", "geophysical instrument", "measures the local acceleration due to gravity, from a mass on a spring or a falling test mass"),
    ("gyroscope", "angular velocity;orientation;rotation", "motion sensor", "a spinning rotor or vibrating structure resists changes of orientation, giving the rate of rotation"),
    ("speedometer", "speed", "vehicle instrument", "gives the speed of a vehicle, from the rotation of the wheels or transmission"),
    ("odometer", "distance;distance travelled", "vehicle instrument", "counts wheel revolutions to give the distance travelled"),
    ("tachometer;rev counter", "rotational speed;rpm", "vehicle instrument", "gives the rotational speed of a shaft or engine in revolutions per minute"),
    ("~ruler;rule;measuring stick;yardstick;meter stick", "length;distance", "length measure", "a straight strip marked in units of length"),
    ("tape measure;measuring tape", "length;distance", "length measure", "a flexible ribbon marked in units of length"),
    ("caliper;calipers;vernier caliper;vernier calipers", "length;thickness;diameter", "length measure", "two jaws close on an object; a vernier or dial scale reads to about 0.02 mm"),
    ("micrometer;micrometer screw gauge;screw gauge", "length;thickness;diameter", "length measure", "a calibrated screw closes on the object; reads to about 0.01 mm"),
    ("~level;spirit level;bubble level", "levelness;tilt;angle", "angle measure", "an air bubble in a curved vial of liquid centres when the surface is horizontal"),
    ("inclinometer;clinometer;tilt meter;tiltmeter;tilt sensor;slope meter", "tilt;slope;angle;inclination", "angle measure", "measures the angle of slope or tilt relative to gravity"),
    ("protractor", "angle", "angle measure", "a semicircle or circle marked in degrees"),
    ("theodolite;transit", "angle;horizontal and vertical angles", "surveying instrument", "a telescope on a graduated mount measures horizontal and vertical angles for surveying"),
    ("sextant", "angle;altitude of celestial bodies", "navigation instrument", "mirrors bring a star or the Sun onto the horizon to measure its angle above it, for celestial navigation"),
    ("~compass;magnetic compass", "direction;magnetic north;bearing", "navigation instrument", "a magnetised needle aligns with Earth's magnetic field"),
    ("altimeter", "altitude;height", "navigation instrument", "gives height above a reference, from air pressure (barometric altimeter) or a radio or radar echo"),
    ("~scale;weighing scale;scales;bathroom scale", "mass;weight", "mass measure", "gives the weight of an object, from the deflection of a spring or a strain gauge"),
    ("~balance;beam balance;analytical balance;pan balance", "mass", "mass measure", "compares an unknown mass with known masses, or measures the force needed to balance it"),
    ("spring scale;spring balance;newton meter;newtonmeter;force gauge", "force;weight", "force measure", "a spring stretches in proportion to the force applied (Hooke's law)"),
    ("dynamometer", "force;torque;power", "force measure", "measures force, torque or the power of an engine or motor"),
    ("kibble balance;watt balance", "mass", "mass measure", "balances the weight of a mass against an electromagnetic force; used to realise the kilogram from the Planck constant"),
    ("ammeter", "electric current;current", "electrical instrument", "connected in series, measures the current through a circuit"),
    ("voltmeter", "voltage;potential difference;electric potential", "electrical instrument", "connected in parallel, measures the potential difference between two points"),
    ("ohmmeter", "electrical resistance;resistance", "electrical instrument", "passes a small known current and measures the voltage to give resistance"),
    ("multimeter;multitester;avometer", "voltage;current;resistance", "electrical instrument", "combines a voltmeter, ammeter and ohmmeter, often with more functions"),
    ("galvanometer", "electric current;current", "electrical instrument", "a coil in a magnetic field turns in proportion to a small current"),
    ("electrometer", "electric charge;charge;voltage", "electrical instrument", "measures electric charge or potential difference while drawing almost no current"),
    ("electroscope;gold-leaf electroscope", "electric charge;charge", "electrical instrument", "two thin leaves repel each other when charged, showing the presence and rough size of a charge"),
    ("coulometer;voltameter", "electric charge;charge", "electrical instrument", "measures the charge passed from the amount of substance deposited or released by electrolysis"),
    ("wattmeter", "electric power;power", "electrical instrument", "measures the electric power in a circuit"),
    ("electricity meter;electric meter;kilowatt-hour meter;watt-hour meter", "electrical energy;energy", "electrical instrument", "records the electrical energy delivered to a building, in kilowatt-hours"),
    ("oscilloscope", "voltage;waveform;signal", "electrical instrument", "displays voltage signals against time"),
    ("~clock", "time", "timekeeper", "keeps and shows the time of day"),
    ("~watch;wristwatch", "time", "timekeeper", "a small portable clock"),
    ("stopwatch;stop watch", "time;elapsed time", "timekeeper", "measures elapsed time between start and stop"),
    ("hourglass;sandglass;sand timer;sand clock;egg timer", "time;elapsed time", "timekeeper", "sand flows through a narrow neck between two glass bulbs in a fixed time"),
    ("sundial", "time;time of day", "timekeeper", "the shadow of a gnomon on a marked dial shows the solar time"),
    ("chronometer;marine chronometer", "time", "timekeeper", "a precise clock used to find longitude at sea"),
    ("atomic clock", "time;frequency", "timekeeper", "counts the frequency of an atomic transition (the second is defined by caesium-133 at 9,192,631,770 Hz)"),
    ("hydrometer", "density;specific gravity;relative density", "density measure", "a weighted float sinks to a depth that depends on the density of the liquid; used for wine, beer, batteries and antifreeze"),
    ("pycnometer;specific gravity bottle;density bottle", "density", "density measure", "a flask of precisely known volume, weighed empty and full"),
    ("viscometer;viscosimeter", "viscosity", "fluid instrument", "measures viscosity from flow through a tube, a falling ball or the torque on a rotating spindle"),
    ("flow meter;flowmeter", "flow rate;flow", "fluid instrument", "measures the rate of flow of a liquid or gas"),
    ("ph meter", "ph;acidity", "chemical instrument", "a glass electrode develops a voltage that depends on hydrogen ion activity"),
    ("refractometer", "refractive index;sugar content", "optical instrument", "measures the refractive index of a liquid, used for sugar content (degrees Brix)"),
    ("polarimeter", "optical rotation", "optical instrument", "measures the rotation of polarised light by an optically active substance"),
    ("spectrometer;spectroscope;spectrograph", "spectrum;wavelength", "optical instrument", "splits light into its wavelengths and measures their intensities"),
    ("photometer;light meter;lux meter;illuminance meter", "light intensity;illuminance;light", "optical instrument", "measures the intensity of light"),
    ("interferometer", "wavelength;displacement;distance", "optical instrument", "superposes light waves; the interference pattern reveals tiny differences in path length"),
    ("sound level meter;decibel meter;noise meter", "sound level;sound pressure level;loudness", "acoustic instrument", "a microphone and weighting filters give the sound pressure level in decibels"),
    ("planimeter", "area", "length measure", "tracing the boundary of a figure gives its area"),
    ("pedometer;step counter", "steps;distance", "personal instrument", "counts steps from the motion of the body"),
    ("calorimeter", "heat;energy;enthalpy", "chemical instrument", "measures the heat released or absorbed by a reaction or physical change (in particle physics, the energy of particles)"),
    ("geiger counter;geiger-muller counter;geiger–müller counter;geiger muller counter", "ionizing radiation;radiation;radioactivity", "particle detector", "each ionizing particle sets off an avalanche of ions in a gas-filled tube, producing a click"),
    ("dosimeter", "radiation dose;dose;ionizing radiation", "radiation instrument", "measures the accumulated dose of ionizing radiation"),
    ("bubble chamber", "tracks of charged particles;charged particles;particle tracks", "particle detector", "charged particles leave trails of bubbles in a superheated liquid (usually hydrogen); invented by Donald Glaser in 1952"),
    ("cloud chamber;wilson chamber;wilson cloud chamber", "tracks of ionizing particles;charged particles;particle tracks", "particle detector", "ionizing particles leave trails of droplets in a supersaturated vapour; invented by C. T. R. Wilson in 1911"),
    ("spark chamber", "tracks of charged particles;charged particles;particle tracks", "particle detector", "charged particles trigger sparks between high-voltage plates along their path"),
    ("wire chamber;multiwire proportional chamber;drift chamber", "tracks of charged particles;charged particles;particle tracks", "particle detector", "charged particles ionize a gas and the charge is collected on fine wires, locating the track; developed by Georges Charpak in 1968"),
    ("scintillation counter;scintillator", "ionizing radiation;radiation;particle energy", "particle detector", "a crystal or plastic emits flashes of light when struck, which a photomultiplier counts"),
    ("cherenkov detector", "charged particles;particle velocity", "particle detector", "detects the cone of light emitted by charged particles faster than light in the medium"),
    ("semiconductor detector;silicon detector;silicon strip detector", "ionizing radiation;particle tracks;particle energy", "particle detector", "a particle frees electron–hole pairs in a reverse-biased semiconductor junction"),
    ("crankshaft position sensor;crank sensor;ckp sensor", "crankshaft position;engine speed;rotational speed", "vehicle sensor", "a Hall-effect or magnetic (variable reluctance) sensor reads a toothed wheel on the crankshaft; the engine control unit uses it to time ignition and fuel injection"),
    ("camshaft position sensor;cam sensor", "camshaft position", "vehicle sensor", "reads the camshaft's angle so the engine control unit knows which cylinder is on its compression stroke"),
    ("oxygen sensor;lambda sensor;o2 sensor", "oxygen concentration;air-fuel ratio", "vehicle sensor", "measures oxygen in the exhaust so the engine can hold the air–fuel ratio near stoichiometric"),
];

struct Inst {
    names: Vec<&'static str>,
    bare: bool,
    measures: Vec<&'static str>,
    category: &'static str,
    how: &'static str,
}

fn all() -> &'static Vec<Inst> {
    static D: OnceLock<Vec<Inst>> = OnceLock::new();
    D.get_or_init(|| {
        INSTRUMENTS
            .iter()
            .map(|(n, m, c, h)| Inst { names: n.trim_start_matches('~').split(';').collect(), bare: !n.starts_with('~'), measures: m.split(';').collect(), category: c, how: h })
            .collect()
    })
}

fn find(s: &str) -> Option<&'static Inst> {
    let s = s.trim().trim_start_matches("a ").trim_start_matches("an ").trim_start_matches("the ").trim();
    let s = s.replace("geiger-müller", "geiger-muller");
    all().iter().find(|i| i.names.iter().any(|n| *n == s || format!("{n}s") == s || format!("{n}es") == s))
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn card(i: &Inst, want: Option<&str>) -> Answer {
    let (title, value) = match want {
        Some("measures") => ("Measures", i.measures[0].to_string()),
        Some("names") => ("Alternative names", if i.names.len() > 1 { i.names[1..].join(" | ") } else { "(none in common use)".to_string() }),
        _ => ("Description", format!("{} — measures {}; {}", cap(i.names[0]), i.measures[0], i.how)),
    };
    let mut rows = vec![
        vec![text("measures"), text(i.measures.join(", "))],
        vec![text("type"), text(i.category)],
        vec![text("how it works"), text(i.how)],
    ];
    if i.names.len() > 1 {
        rows.push(vec![text("other names"), text(i.names[1..].join(", "))]);
    }
    Answer::new("measuring instrument", vec![
        input(format!("{}{}", i.names[0], want.map(|w| format!(" | {}", if w == "names" { "alternative names" } else { w })).unwrap_or_default())),
        primary(pod("Result", title, text(value))),
        pod("Table", "Basic information", table(None, rows)),
    ])
}

fn listing(title: String, items: Vec<&'static Inst>) -> Option<Answer> {
    if items.is_empty() {
        return None;
    }
    let rows = items.iter().map(|i| vec![text(i.names[0]), text(i.measures.join(", ")), text(i.how)]).collect();
    Some(Answer::new("measuring instruments", vec![
        input(title),
        primary(pod("Result", "Instruments", text(items.iter().map(|i| i.names[0]).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Details", table(Some(vec!["instrument", "measures", "how it works"]), rows)),
    ]))
}

/// Instruments measuring a quantity ("time", "charges", "acceleration").
fn for_quantity(q: &str) -> Option<Answer> {
    let q = q.trim().trim_start_matches("the ").trim_start_matches("an ").trim_start_matches("a ");
    let q1 = q.trim_end_matches('s');
    if q.len() < 2 {
        return None;
    }
    let exact: Vec<&'static Inst> = all().iter().filter(|i| i.measures.iter().any(|m| *m == q || *m == q1)).collect();
    let items = if exact.is_empty() {
        let w = Regex::new(&format!(r"\b(?:{}|{})\b", regex::escape(q), regex::escape(q1))).ok()?;
        all().iter().filter(|i| i.measures.iter().any(|m| w.is_match(m))).collect()
    } else {
        exact
    };
    listing(format!("instruments that measure {q}"), items)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    // "what does a ruler measure", "a bubble chamber measures", "what is a hydrometer used for"
    if let Some(m) = re!(r"^(?:what (?:does|do) (.+?) measure|(.+?) measures|what (?:is|are) (.+?) used (?:for|to measure)|(.+?) is used to measure what)$").captures(l) {
        let s = m.get(1).or(m.get(2)).or(m.get(3)).or(m.get(4))?.as_str();
        return find(s).map(|i| card(i, Some("measures")));
    }
    // "alternative names for an hourglass", "other names for a seismometer", "synonyms of hourglass"
    if let Some(m) = re!(r"^(?:alternative|alternate|other) names? (?:for|of) (.+)$|^synonyms? (?:for|of) (.+)$").captures(l) {
        let s = m.get(1).or(m.get(2))?.as_str();
        return find(s).map(|i| card(i, Some("names")));
    }
    // "device used to measure time", "how do you measure charges", "acceleration measurement devices"
    if let Some(m) = re!(r"^(?:(?:an? |the )?(?:devices?|instruments?|tools?|equipment|sensors?|meters?) (?:used )?(?:to measure|for measuring|that measures?|which measures?|for) |how (?:do|can|does) (?:you|one|we|i) measure |what (?:measures|is used to measure|device measures|instrument measures) |(?:what|which) (?:instruments?|devices?|tools?|sensors?|meters?) (?:can |are used to |do you use to |is used to )?measures? |measuring )(.+)$").captures(l) {
        return for_quantity(&m[1]);
    }
    if let Some(m) = re!(r"^(.+?) (?:measurement|measuring) (?:devices?|instruments?|tools?|equipment)$|^(.+?) (?:sensors|meters|gauges)$").captures(l) {
        let s = m.get(1).or(m.get(2))?.as_str();
        return for_quantity(s);
    }
    // "thermometer types", "types of thermometers", "instances of a particle detector"
    if let Some(m) = re!(r"^(?:types|kinds|instances|examples|varieties) of (?:an? )?(.+?)s?$|^(.+?) (?:types|kinds)$").captures(l) {
        let s = m.get(1).or(m.get(2))?.as_str().trim_end_matches('s');
        let items: Vec<&'static Inst> = all().iter().filter(|i| i.category == s && i.names[0] != s).collect();
        return listing(format!("types of {s}"), items);
    }
    // a bare instrument: "hydrometer", "crankshaft position sensor"
    let i = find(l)?;
    (i.bare || l.contains(' ')).then(|| card(i, None))
}
