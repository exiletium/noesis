//! Knowledge domains beyond pure mathematics: units, physical constants,
//! chemistry, geography, dates and times, finance, probability.

pub mod acronyms;
pub mod astro;
pub mod calendars;
pub mod chem;
pub mod chemcalc;
pub mod complexity;
pub mod crystals;
pub mod ephem;
pub mod everyday;
pub mod facts;
pub mod formula_db;
pub mod formulas;
pub mod gauges;
pub mod geo;
pub mod knowledge;
pub mod lexicon;
pub mod music;
pub mod practical;
pub mod school;
pub mod nuclides;
pub mod physics;
pub mod physchem;
pub mod fluids;
pub mod bulk;
pub mod landforms;
pub mod genetics;
pub mod prehistoric;
pub mod people;
pub mod history;
pub mod sports;
pub mod travel;
pub mod airports;
pub mod instruments;
pub mod exercises;
pub mod photography;
pub mod seismology;
pub mod states;
pub mod teeth;
pub mod surfaces;
pub mod heun;
pub mod coulomb;
pub mod oxidation;
pub mod firstlines;
pub mod carbon;
pub mod planetpos;
pub mod seasons;
pub mod lagrange;
pub mod mains;
pub mod orbitgeom;
pub mod reagents;
pub mod packing;
pub mod artworks;
pub mod names;
pub mod garden;
pub mod elemelec;
pub mod miscibility;
pub mod inttables;
pub mod vetanatomy;
pub mod compoundclasses;
pub mod acvoltage;
pub mod ranges;
pub mod boardgames;
pub mod ropes;
pub mod comets;
pub mod labtests;
pub mod textlength;
pub mod formats;
pub mod nations;
pub mod continents;
pub mod wordproblems;
pub mod nutrition;
pub mod algebra_geo;
pub mod factbook;
pub mod cosmos;
pub mod money;
pub mod health;
pub mod applied;
pub mod fun;
pub mod objects;
pub mod clock;
pub mod materials;
pub mod animals;
pub mod myth;
pub mod structures;
pub mod literature;
pub mod anatomy;
pub mod sky;
pub mod technology;
pub mod society;
pub mod stellar;
pub mod earth;
pub mod trades;
pub mod shapes;
pub mod reactions;
pub mod solutions;
pub mod live;
pub mod misc;
pub mod molecule;
pub mod units;

use crate::engine::{Answer, Handler, Query};

/// Named-entity domains only make sense for inputs with a letter or digit;
/// punctuation alone ("...", "?") would otherwise match an empty name.
macro_rules! needs_word {
    ($($name:ident => $h:path),* $(,)?) => {
        $(fn $name(q: &Query) -> Option<Answer> {
            if q.lower.chars().any(char::is_alphanumeric) { $h(q) } else { None }
        })*
    };
}

needs_word!(society_h => society::handle, stellar_h => stellar::handle, animals_h => animals::handle, myth_h => myth::handle, structures_h => structures::handle, literature_h => literature::handle, anatomy_h => anatomy::handle, units_h => units::handle, technology_h => technology::handle, landforms_h => landforms::handle, prehistoric_h => prehistoric::handle, people_h => people::handle, history_h => history::handle, sports_h => sports::handle, travel_h => travel::handle, nutrition_h => nutrition::handle, factbook_h => factbook::handle);

/// Domain handlers in priority order (they run before the general math
/// analyzer, and only claim inputs that use their vocabulary).
pub fn handlers() -> Vec<(&'static str, Handler)> {
    vec![("live", live::handle), ("acronyms", acronyms::handle), ("acvoltage", acvoltage::handle), ("surfaces", surfaces::handle), ("heun", heun::handle), ("coulomb", coulomb::handle), ("oxidation", oxidation::handle), ("firstlines", firstlines::handle), ("carbon", carbon::handle), ("planetpos", planetpos::handle), ("seasons", seasons::handle), ("lagrange", lagrange::handle), ("mains", mains::handle), ("orbitgeom", orbitgeom::handle), ("reagents", reagents::handle), ("artworks", artworks::handle), ("names", names::handle), ("garden", garden::handle), ("genetics", genetics::handle), ("everyday", everyday::handle), ("practical", practical::handle), ("wordproblems", wordproblems::handle), ("school", school::handle), ("music", music::handle), ("ranges", ranges::handle), ("nutrition", nutrition_h), ("algebra_geo", algebra_geo::handle), ("inttables", inttables::handle), ("complexity", complexity::handle), ("boardgames", boardgames::handle), ("comets", comets::handle), ("crystals", crystals::handle), ("knowledge", knowledge::handle), ("society", society_h), ("nuclides", nuclides::handle), ("physics", physics::handle), ("shapes", shapes::handle), ("misc", misc::handle), ("clock", clock::handle), ("stellar", stellar_h), ("sky", sky::handle), ("earth", earth::handle), ("seismology", seismology::handle), ("photography", photography::handle), ("textlength", textlength::handle), ("ropes", ropes::handle), ("trades", trades::handle), ("states", states::handle), ("money", money::handle), ("labtests", labtests::handle), ("health", health::handle), ("applied", applied::handle), ("fun", fun::handle), ("objects", objects::handle), ("packing", packing::handle), ("animals", animals_h), ("prehistoric", prehistoric_h), ("myth", myth_h), ("structures", structures_h), ("literature", literature_h), ("people", people_h), ("history", history_h), ("sports", sports_h), ("travel", travel_h), ("airports", airports::handle), ("exercises", exercises::handle), ("teeth", teeth::handle), ("vetanatomy", vetanatomy::handle), ("anatomy", anatomy_h), ("formats", formats::handle), ("technology", technology_h), ("instruments", instruments::handle), ("landforms", landforms_h), ("compoundclasses", compoundclasses::handle), ("molecule", molecule::handle), ("physchem", physchem::handle), ("fluids", fluids::handle), ("cosmos", cosmos::handle), ("gauges", gauges::handle), ("elemelec", elemelec::handle), ("chem", chem::handle), ("chemcalc", chemcalc::handle), ("miscibility", miscibility::handle), ("solutions", solutions::handle), ("materials", materials::handle), ("bulk", bulk::handle), ("reactions", reactions::handle), ("formulas", formulas::handle), ("formula_db", formula_db::handle), ("astro", astro::handle), ("nations", nations::handle), ("continents", continents::handle), ("geo", geo::handle), ("factbook", factbook_h), ("units", units_h)]
}
