//! Knowledge domains beyond pure mathematics: units, physical constants,
//! chemistry, geography, dates and times, finance, probability.

pub mod astro;
pub mod chem;
pub mod chemcalc;
pub mod ephem;
pub mod everyday;
pub mod facts;
pub mod formulas;
pub mod geo;
pub mod knowledge;
pub mod lexicon;
pub mod live;
pub mod misc;
pub mod units;

use crate::engine::Handler;

/// Domain handlers in priority order (they run before the general math
/// analyzer, and only claim inputs that use their vocabulary).
pub fn handlers() -> Vec<(&'static str, Handler)> {
    vec![("live", live::handle), ("everyday", everyday::handle), ("knowledge", knowledge::handle), ("misc", misc::handle), ("chem", chem::handle), ("chemcalc", chemcalc::handle), ("formulas", formulas::handle), ("astro", astro::handle), ("geo", geo::handle), ("units", units::handle)]
}
