//! Other calendars: Hebrew, Islamic (tabular), Julian, Persian (arithmetic
//! 33-year rule) and the Maya long count, all through fixed day numbers
//! (R.D. 1 = January 1, 1 CE, proleptic Gregorian), following the
//! algorithms in Reingold & Dershowitz, "Calendrical Calculations".

use chrono::{Datelike, NaiveDate};

pub fn fixed(d: NaiveDate) -> i64 {
    d.num_days_from_ce() as i64
}

pub fn from_fixed(f: i64) -> Option<NaiveDate> {
    NaiveDate::from_num_days_from_ce_opt(i32::try_from(f).ok()?)
}

fn fdiv(a: i64, b: i64) -> i64 {
    a.div_euclid(b)
}

// ---------------------------------------------------------------- Julian

fn julian_leap(y: i64) -> bool {
    y.rem_euclid(4) == if y > 0 { 0 } else { 3 }
}

pub fn fixed_from_julian(y: i64, m: i64, d: i64) -> i64 {
    let yy = if y < 0 { y + 1 } else { y };
    -2 + 365 * (yy - 1) + fdiv(yy - 1, 4) + fdiv(367 * m - 362, 12) + if m <= 2 { 0 } else if julian_leap(y) { -1 } else { -2 } + d
}

pub fn julian_from_fixed(f: i64) -> (i64, i64, i64) {
    let approx = fdiv(4 * (f + 1) + 1464, 1461);
    let y = if approx <= 0 { approx - 1 } else { approx };
    let prior = f - fixed_from_julian(y, 1, 1);
    let corr = if f < fixed_from_julian(y, 3, 1) { 0 } else if julian_leap(y) { 1 } else { 2 };
    let m = fdiv(12 * (prior + corr) + 373, 367);
    (y, m, f - fixed_from_julian(y, m, 1) + 1)
}

// ---------------------------------------------------------------- Islamic

const ISLAMIC_EPOCH: i64 = 227015; // July 16, 622 (Julian)
pub const ISLAMIC_MONTHS: [&str; 12] = ["Muharram", "Safar", "Rabi' al-awwal", "Rabi' al-thani", "Jumada al-awwal", "Jumada al-thani", "Rajab", "Sha'ban", "Ramadan", "Shawwal", "Dhu al-Qi'dah", "Dhu al-Hijjah"];

pub fn fixed_from_islamic(y: i64, m: i64, d: i64) -> i64 {
    d + 29 * (m - 1) + fdiv(6 * m - 1, 11) + (y - 1) * 354 + fdiv(3 + 11 * y, 30) + ISLAMIC_EPOCH - 1
}

pub fn islamic_from_fixed(f: i64) -> (i64, i64, i64) {
    let y = fdiv(30 * (f - ISLAMIC_EPOCH) + 10646, 10631);
    let prior = f - fixed_from_islamic(y, 1, 1);
    let m = fdiv(11 * prior + 330, 325);
    (y, m, f - fixed_from_islamic(y, m, 1) + 1)
}

// ---------------------------------------------------------------- Hebrew

const HEBREW_EPOCH: i64 = -1373427; // October 7, 3761 BCE (Julian)

fn hebrew_leap(y: i64) -> bool {
    (7 * y + 1).rem_euclid(19) < 7
}

fn last_month(y: i64) -> i64 {
    if hebrew_leap(y) {
        13
    } else {
        12
    }
}

fn elapsed_days(y: i64) -> i64 {
    let months = fdiv(235 * y - 234, 19);
    let parts = 12084 + 13753 * months;
    let days = 29 * months + fdiv(parts, 25920);
    if (3 * (days + 1)).rem_euclid(7) < 3 {
        days + 1
    } else {
        days
    }
}

fn year_correction(y: i64) -> i64 {
    let (a, b, c) = (elapsed_days(y - 1), elapsed_days(y), elapsed_days(y + 1));
    if c - b == 356 {
        2
    } else if b - a == 382 {
        1
    } else {
        0
    }
}

fn hebrew_new_year(y: i64) -> i64 {
    HEBREW_EPOCH + elapsed_days(y) + year_correction(y)
}

fn hebrew_year_days(y: i64) -> i64 {
    hebrew_new_year(y + 1) - hebrew_new_year(y)
}

fn hebrew_month_days(y: i64, m: i64) -> i64 {
    let n = hebrew_year_days(y);
    let short = matches!(m, 2 | 4 | 6 | 10 | 13) || (m == 12 && !hebrew_leap(y)) || (m == 8 && n % 10 != 5) || (m == 9 && n % 10 == 3);
    if short {
        29
    } else {
        30
    }
}

pub fn fixed_from_hebrew(y: i64, m: i64, d: i64) -> i64 {
    let mut f = hebrew_new_year(y) + d - 1;
    if m < 7 {
        f += (7..=last_month(y)).map(|k| hebrew_month_days(y, k)).sum::<i64>();
        f += (1..m).map(|k| hebrew_month_days(y, k)).sum::<i64>();
    } else {
        f += (7..m).map(|k| hebrew_month_days(y, k)).sum::<i64>();
    }
    f
}

pub fn hebrew_from_fixed(f: i64) -> (i64, i64, i64) {
    let approx = fdiv((f - HEBREW_EPOCH) * 98496, 35975351) + 1;
    let mut y = approx - 1;
    while hebrew_new_year(y + 1) <= f {
        y += 1;
    }
    let start = if f < fixed_from_hebrew(y, 1, 1) { 7 } else { 1 };
    let mut m = start;
    for _ in 0..13 {
        if f <= fixed_from_hebrew(y, m, hebrew_month_days(y, m)) {
            break;
        }
        m = if m == last_month(y) { 1 } else { m + 1 };
    }
    (y, m, f - fixed_from_hebrew(y, m, 1) + 1)
}

pub fn hebrew_month_name(y: i64, m: i64) -> &'static str {
    match m {
        1 => "Nisan",
        2 => "Iyar",
        3 => "Sivan",
        4 => "Tammuz",
        5 => "Av",
        6 => "Elul",
        7 => "Tishrei",
        8 => "Cheshvan",
        9 => "Kislev",
        10 => "Tevet",
        11 => "Shevat",
        12 if hebrew_leap(y) => "Adar I",
        12 => "Adar",
        _ => "Adar II",
    }
}

// ---------------------------------------------------------------- Persian

pub const PERSIAN_MONTHS: [&str; 12] = ["Farvardin", "Ordibehesht", "Khordad", "Tir", "Mordad", "Shahrivar", "Mehr", "Aban", "Azar", "Dey", "Bahman", "Esfand"];

/// Persian (Solar Hijri) calendar by Borkowski's algorithm, which matches
/// the official astronomical calendar for years 1 to 3177 AP.
fn jal_cal(jy: i64) -> (i64, i64) {
    const BREAKS: [i64; 20] = [-61, 9, 38, 199, 426, 686, 756, 818, 1111, 1181, 1210, 1635, 2060, 2097, 2192, 2262, 2324, 2394, 2456, 3178];
    let gy = jy + 621;
    let mut leap_j = -14;
    let mut jp = BREAKS[0];
    let mut jump = 0;
    for &jm in &BREAKS[1..] {
        jump = jm - jp;
        if jy < jm {
            break;
        }
        leap_j += jump / 33 * 8 + (jump % 33) / 4;
        jp = jm;
    }
    let mut n = jy - jp;
    leap_j += n / 33 * 8 + ((n % 33) + 3) / 4;
    if jump % 33 == 4 && jump - n == 4 {
        leap_j += 1;
    }
    let leap_g = gy / 4 - (gy / 100 + 1) * 3 / 4 - 150;
    let march = 20 + leap_j - leap_g;
    if jump - n < 6 {
        n = n - jump + (jump + 4) / 33 * 33;
    }
    let mut leap = ((n + 1) % 33 - 1) % 4;
    if leap == -1 {
        leap = 4;
    }
    (leap, march)
}

pub fn fixed_from_persian(y: i64, m: i64, d: i64) -> Option<i64> {
    let (_, march) = jal_cal(y);
    let start = fixed(NaiveDate::from_ymd_opt((y + 621) as i32, 3, march as u32)?);
    Some(start + if m <= 7 { (m - 1) * 31 } else { 186 + (m - 7) * 30 } + d - 1)
}

pub fn persian_from_fixed(f: i64) -> Option<(i64, i64, i64)> {
    let g = from_fixed(f)?;
    let mut y = g.year() as i64 - 621;
    let mut start = fixed_from_persian(y, 1, 1)?;
    if f < start {
        y -= 1;
        start = fixed_from_persian(y, 1, 1)?;
    }
    let k = f - start;
    Some(if k < 186 { (y, k / 31 + 1, k % 31 + 1) } else { (y, (k - 186) / 30 + 7, (k - 186) % 30 + 1) })
}

// ---------------------------------------------------------------- Maya

const MAYAN_EPOCH: i64 = -1137142; // August 11, 3114 BCE (Gregorian)

pub fn mayan_long_count(f: i64) -> String {
    let mut k = f - MAYAN_EPOCH;
    let mut parts = [0i64; 5];
    for (i, n) in [144000, 7200, 360, 20, 1].iter().enumerate() {
        parts[i] = k.div_euclid(*n);
        k = k.rem_euclid(*n);
    }
    parts.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(".")
}

// ---------------------------------------------------------------- display

fn year_era(y: i64, ce: &str, bce: &str) -> String {
    if y > 0 {
        format!("{y} {ce}")
    } else {
        format!("{} {bce}", 1 - y)
    }
}

pub fn hebrew_text(f: i64) -> String {
    let (y, m, d) = hebrew_from_fixed(f);
    format!("{d} {} {y}", hebrew_month_name(y, m))
}

pub fn islamic_text(f: i64) -> String {
    let (y, m, d) = islamic_from_fixed(f);
    format!("{d} {} {}", ISLAMIC_MONTHS[(m - 1) as usize], year_era(y, "AH", "BH"))
}

pub fn julian_text(f: i64) -> String {
    let (y, m, d) = julian_from_fixed(f);
    let month = NaiveDate::from_ymd_opt(2001, m as u32, 1).map(|x| x.format("%B").to_string()).unwrap_or_default();
    format!("{month} {d}, {}", year_era(y, "AD", "BC"))
}

pub fn persian_text(f: i64) -> Option<String> {
    let (y, m, d) = persian_from_fixed(f)?;
    (y > 0).then(|| format!("{d} {} {y} AP", PERSIAN_MONTHS[(m - 1) as usize]))
}

/// (calendar, date) rows for a Gregorian date.
pub fn rows(d: NaiveDate) -> Vec<(&'static str, String)> {
    let f = fixed(d);
    let mut out = vec![("Julian calendar", julian_text(f)), ("Hebrew calendar", hebrew_text(f)), ("Islamic calendar (tabular)", islamic_text(f))];
    if let Some(p) = persian_text(f) {
        out.push(("Persian calendar", p));
    }
    out.push(("Maya long count", mayan_long_count(f)));
    out
}

/// Which calendar a phrase names: "hebrew", "islamic", "julian", "persian", "mayan".
pub fn calendar_name(s: &str) -> Option<&'static str> {
    let s = s.trim().trim_start_matches("the ").trim_end_matches(" calendar").trim_end_matches(" date").trim();
    Some(match s {
        "hebrew" | "jewish" => "hebrew",
        "islamic" | "hijri" | "muslim" | "arabic" => "islamic",
        "julian" | "old style" => "julian",
        "persian" | "iranian" | "solar hijri" | "jalali" | "shamsi" => "persian",
        "mayan" | "maya" | "mayan long count" | "long count" => "mayan",
        _ => return None,
    })
}

pub fn to_calendar(cal: &str, d: NaiveDate) -> Option<String> {
    let f = fixed(d);
    match cal {
        "hebrew" => Some(hebrew_text(f)),
        "islamic" => Some(islamic_text(f)),
        "julian" => Some(julian_text(f)),
        "persian" => persian_text(f),
        "mayan" => Some(mayan_long_count(f)),
        _ => None,
    }
}

/// Parse a date written in one of the other calendars, e.g. "25 Shevat 5780",
/// "1 Ramadan 1445", "15 Mehr 1400", "October 5, 1582 Julian".
pub fn parse_other(s: &str) -> Option<(&'static str, NaiveDate)> {
    let l = s.trim().to_lowercase().replace(['\'', '’', '-'], " ");
    let l = l.split_whitespace().collect::<Vec<_>>().join(" ");
    let words: Vec<&str> = l.split(' ').collect();
    if words.len() < 3 {
        return None;
    }
    let num = |w: &str| w.trim_end_matches(',').trim_end_matches("ah").trim_end_matches("ap").parse::<i64>().ok();
    let (d, y) = (num(words[0])?, num(words[words.len() - 1].trim_end_matches(','))?);
    let month = words[1..words.len() - 1].join(" ");
    let simple = |x: &str| x.to_lowercase().replace(['\'', '-'], " ").split_whitespace().collect::<Vec<_>>().join(" ");
    let hebrew = [("nisan", 1), ("iyar", 2), ("iyyar", 2), ("sivan", 3), ("tammuz", 4), ("tamuz", 4), ("av", 5), ("elul", 6), ("tishrei", 7), ("tishri", 7), ("cheshvan", 8), ("heshvan", 8), ("marheshvan", 8), ("kislev", 9), ("tevet", 10), ("shevat", 11), ("shvat", 11), ("adar", 12), ("adar i", 12), ("adar 1", 12), ("adar ii", 13), ("adar 2", 13)];
    if let Some(&(_, m)) = hebrew.iter().find(|(n, _)| *n == month) {
        let m = if m == 12 && hebrew_leap(y) && month == "adar" { 13 } else { m };
        return Some(("hebrew", from_fixed(fixed_from_hebrew(y, m, d))?));
    }
    if let Some(i) = ISLAMIC_MONTHS.iter().position(|n| simple(n) == month) {
        return Some(("islamic", from_fixed(fixed_from_islamic(y, i as i64 + 1, d))?));
    }
    if let Some(i) = PERSIAN_MONTHS.iter().position(|n| simple(n) == month) {
        return Some(("persian", from_fixed(fixed_from_persian(y, i as i64 + 1, d)?)?));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(y: i32, m: u32, d: u32) -> i64 {
        fixed(NaiveDate::from_ymd_opt(y, m, d).unwrap())
    }

    #[test]
    fn known_dates() {
        // Calendrical Calculations sample dates and published conversions
        assert_eq!(hebrew_text(g(2020, 2, 20)), "25 Shevat 5780");
        assert_eq!(hebrew_text(g(2023, 9, 16)), "1 Tishrei 5784");
        assert_eq!(hebrew_text(g(2024, 3, 11)), "1 Adar II 5784");
        assert_eq!(islamic_text(g(2024, 3, 11)), "1 Ramadan 1445 AH");
        assert_eq!(julian_text(g(1582, 10, 15)), "October 5, 1582 AD");
        assert_eq!(persian_text(g(2024, 3, 20)).unwrap(), "1 Farvardin 1403 AP");
        assert_eq!(persian_text(g(2021, 3, 21)).unwrap(), "1 Farvardin 1400 AP");
        assert_eq!(mayan_long_count(g(2012, 12, 21)), "13.0.0.0.0");
        for f in [g(1, 1, 1), g(1945, 11, 12), g(2096, 7, 21), g(-500, 3, 3)] {
            let (y, m, d) = hebrew_from_fixed(f);
            assert_eq!(fixed_from_hebrew(y, m, d), f);
            let (y, m, d) = islamic_from_fixed(f);
            assert_eq!(fixed_from_islamic(y, m, d), f);
            let (y, m, d) = julian_from_fixed(f);
            assert_eq!(fixed_from_julian(y, m, d), f);
        }
        assert_eq!(parse_other("25 Shevat 5780").unwrap().1, NaiveDate::from_ymd_opt(2020, 2, 20).unwrap());
        assert_eq!(parse_other("1 Ramadan 1445").unwrap().1, NaiveDate::from_ymd_opt(2024, 3, 11).unwrap());
    }
}
