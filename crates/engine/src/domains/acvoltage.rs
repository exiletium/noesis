//! AC mains voltages: RMS, peak and peak-to-peak values.
//!
//! "110V AC", "AC source 110V", "230 volts AC", "peak voltage of 120 V AC".

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

fn v(x: f64) -> String {
    format!("{} V", fmt_float((x * 100.0).round() / 100.0, 6))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let m = re!(r"^(?:(peak|peak-to-peak|peak to peak|rms|average) (?:voltage )?(?:of )?(?:an? )?)?(?:(?:ac|a\.c\.) (?:source|supply|mains|power|voltage) (?:of )?)?(\d+(?:\.\d+)?)\s*(?:v|volts?)\s*(ac|a\.c\.|rms|mains)?(?:\s+(?:at\s+)?(50|60)\s*hz)?$").captures(l)?;
    // "AC" must be said somewhere
    if !l.contains("ac") && !l.contains("a.c.") && !l.contains("rms") && !l.contains("mains") {
        return None;
    }
    let vr: f64 = m[2].parse().ok()?;
    if vr <= 0.0 || vr > 1e6 {
        return None;
    }
    let f: f64 = match m.get(4) {
        Some(x) => x.as_str().parse().ok()?,
        None if (100.0..=127.0).contains(&vr) => 60.0,
        None => 50.0,
    };
    let vp = vr * std::f64::consts::SQRT_2;
    let avg = 2.0 * vp / std::f64::consts::PI;
    let (title, res) = match m.get(1).map(|x| x.as_str()) {
        Some("peak") => ("Peak voltage", v(vp)),
        Some("peak-to-peak" | "peak to peak") => ("Peak-to-peak voltage", v(2.0 * vp)),
        Some("average") => ("Average rectified voltage", v(avg)),
        _ => ("Result", format!("{} RMS | peak {} | peak-to-peak {}", v(vr), v(vp), v(2.0 * vp))),
    };
    Some(Answer::new("AC voltage", vec![
        pod("Input", "Input interpretation", text(format!("AC voltage source | {} RMS, {} Hz sine", v(vr), fmt_float(f, 3)))),
        primary(pod("Result", title, text(res))),
        pod("Table", "Values", table(None, vec![
            vec![text("RMS voltage"), text(v(vr))],
            vec![text("peak voltage V_p = √2 V_rms"), text(v(vp))],
            vec![text("peak-to-peak voltage"), text(v(2.0 * vp))],
            vec![text("average rectified voltage (2/π) V_p"), text(v(avg))],
            vec![text("frequency"), text(format!("{} Hz (period {} ms)", fmt_float(f, 3), fmt_float((1000.0 / f * 1000.0).round() / 1000.0, 5)))],
            vec![text("angular frequency ω = 2πf"), text(format!("{} rad/s", fmt_float((2.0 * std::f64::consts::PI * f * 100.0).round() / 100.0, 6)))],
        ])),
        pod("Waveform", "Waveform", text(format!("v(t) = {} sin({} t) V", fmt_float((vp * 100.0).round() / 100.0, 6), fmt_float((2.0 * std::f64::consts::PI * f * 100.0).round() / 100.0, 6)))),
        pod("Note", "Note", text("assumes a pure sine wave; 100–127 V mains is usually 60 Hz (the Americas, parts of Japan) and 220–240 V usually 50 Hz")),
    ]))
}
