//! Knowledge domains beyond pure mathematics: units, physical constants,
//! chemistry, geography, dates and times, finance, probability.

pub mod astro;
pub mod calendars;
pub mod chem;
pub mod chemcalc;
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
pub mod nutrition;
pub mod algebra_geo;
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

needs_word!(society_h => society::handle, stellar_h => stellar::handle, animals_h => animals::handle, myth_h => myth::handle, structures_h => structures::handle, literature_h => literature::handle, anatomy_h => anatomy::handle, units_h => units::handle, technology_h => technology::handle, landforms_h => landforms::handle, prehistoric_h => prehistoric::handle, people_h => people::handle, history_h => history::handle, sports_h => sports::handle, travel_h => travel::handle, nutrition_h => nutrition::handle);

/// Domain handlers in priority order (they run before the general math
/// analyzer, and only claim inputs that use their vocabulary).
pub fn handlers() -> Vec<(&'static str, Handler)> {
    vec![("live", live::handle), ("genetics", genetics::handle), ("everyday", everyday::handle), ("practical", practical::handle), ("school", school::handle), ("music", music::handle), ("nutrition", nutrition_h), ("algebra_geo", algebra_geo::handle), ("knowledge", knowledge::handle), ("society", society_h), ("nuclides", nuclides::handle), ("physics", physics::handle), ("shapes", shapes::handle), ("misc", misc::handle), ("clock", clock::handle), ("stellar", stellar_h), ("sky", sky::handle), ("earth", earth::handle), ("trades", trades::handle), ("money", money::handle), ("health", health::handle), ("applied", applied::handle), ("fun", fun::handle), ("objects", objects::handle), ("animals", animals_h), ("prehistoric", prehistoric_h), ("myth", myth_h), ("structures", structures_h), ("literature", literature_h), ("people", people_h), ("history", history_h), ("sports", sports_h), ("travel", travel_h), ("anatomy", anatomy_h), ("technology", technology_h), ("landforms", landforms_h), ("molecule", molecule::handle), ("physchem", physchem::handle), ("fluids", fluids::handle), ("cosmos", cosmos::handle), ("gauges", gauges::handle), ("chem", chem::handle), ("chemcalc", chemcalc::handle), ("solutions", solutions::handle), ("materials", materials::handle), ("bulk", bulk::handle), ("reactions", reactions::handle), ("formulas", formulas::handle), ("formula_db", formula_db::handle), ("astro", astro::handle), ("geo", geo::handle), ("units", units_h)]
}
