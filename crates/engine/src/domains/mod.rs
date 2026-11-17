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

use crate::engine::Handler;

/// Domain handlers in priority order (they run before the general math
/// analyzer, and only claim inputs that use their vocabulary).
pub fn handlers() -> Vec<(&'static str, Handler)> {
    vec![("live", live::handle), ("everyday", everyday::handle), ("practical", practical::handle), ("school", school::handle), ("music", music::handle), ("knowledge", knowledge::handle), ("society", society::handle), ("nuclides", nuclides::handle), ("physics", physics::handle), ("shapes", shapes::handle), ("misc", misc::handle), ("clock", clock::handle), ("stellar", stellar::handle), ("sky", sky::handle), ("earth", earth::handle), ("trades", trades::handle), ("money", money::handle), ("health", health::handle), ("applied", applied::handle), ("fun", fun::handle), ("objects", objects::handle), ("animals", animals::handle), ("myth", myth::handle), ("structures", structures::handle), ("literature", literature::handle), ("anatomy", anatomy::handle), ("molecule", molecule::handle), ("physchem", physchem::handle), ("cosmos", cosmos::handle), ("gauges", gauges::handle), ("chem", chem::handle), ("chemcalc", chemcalc::handle), ("solutions", solutions::handle), ("materials", materials::handle), ("reactions", reactions::handle), ("formulas", formulas::handle), ("formula_db", formula_db::handle), ("astro", astro::handle), ("geo", geo::handle), ("units", units::handle)]
}
