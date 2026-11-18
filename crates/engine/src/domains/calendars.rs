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

// ---------------------------------------------------------------- Chinese
//
// The astronomical Chinese calendar (Reingold & Dershowitz, chapter 19):
// months begin on the day of the new moon in Beijing time (UTC+8), the
// winter solstice falls in month 11, and in a year of 13 months the first
// month with no major solar term is the leap month. New moons follow Meeus,
// "Astronomical Algorithms", chapter 49; the Sun's apparent longitude his
// chapter 25 (about 0.01°).

const SYNODIC: f64 = 29.530588861;
const TROPICAL: f64 = 365.242189;
/// R.D. of the epoch, 15 February 2637 BCE (Gregorian).
const CHINESE_EPOCH: i64 = -963099;

/// ΔT = TT − UT in days (Espenak and Meeus polynomials, simplified).
fn delta_t(year: f64) -> f64 {
    let secs = if (2005.0..2050.0).contains(&year) {
        let t = year - 2000.0;
        62.92 + 0.32217 * t + 0.005589 * t * t
    } else if (1986.0..2005.0).contains(&year) {
        let t = year - 2000.0;
        63.86 + 0.3345 * t - 0.060374 * t * t + 0.0017275 * t.powi(3) + 0.000651814 * t.powi(4) + 0.00002373599 * t.powi(5)
    } else if (1961.0..1986.0).contains(&year) {
        let t = year - 1975.0;
        45.45 + 1.067 * t - t * t / 260.0 - t.powi(3) / 718.0
    } else if (1900.0..1961.0).contains(&year) {
        let t = year - 1900.0;
        -2.79 + 1.494119 * t - 0.0598939 * t * t + 0.0061966 * t.powi(3) - 0.000197 * t.powi(4)
    } else {
        let u = (year - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    };
    secs / 86400.0
}

/// Julian Day of an R.D. moment (UT).
fn jd(moment: f64) -> f64 {
    moment + 1721424.5
}

/// Apparent solar longitude in degrees at an R.D. moment (UT).
fn solar_longitude(moment: f64) -> f64 {
    let year = 1.0 + moment / 365.2425;
    let t = (jd(moment) + delta_t(year) - 2451545.0) / 36525.0;
    let rad = |d: f64| d.to_radians();
    let l0 = 280.46646 + 36000.76983 * t + 0.0003032 * t * t;
    let m = 357.52911 + 35999.05029 * t - 0.0001537 * t * t;
    let c = (1.914602 - 0.004817 * t - 0.000014 * t * t) * rad(m).sin() + (0.019993 - 0.000101 * t) * rad(2.0 * m).sin() + 0.000289 * rad(3.0 * m).sin();
    let om = 125.04 - 1934.136 * t;
    (l0 + c - 0.00569 - 0.00478 * rad(om).sin()).rem_euclid(360.0)
}

/// The moment (R.D., UT) of the k-th new moon after January 2000.
fn nth_new_moon(k: f64) -> f64 {
    let t = k / 1236.85;
    let jde = 2451550.09766 + SYNODIC * k + 0.00015437 * t * t - 0.000000150 * t.powi(3) + 0.00000000073 * t.powi(4);
    let e = 1.0 - 0.002516 * t - 0.0000074 * t * t;
    let r = |d: f64| d.to_radians();
    let m = r(2.5534 + 29.10535670 * k - 0.0000014 * t * t - 0.00000011 * t.powi(3));
    let mp = r(201.5643 + 385.81693528 * k + 0.0107582 * t * t + 0.00001238 * t.powi(3) - 0.000000058 * t.powi(4));
    let f = r(160.7108 + 390.67050284 * k - 0.0016118 * t * t - 0.00000227 * t.powi(3) + 0.000000011 * t.powi(4));
    let om = r(124.7746 - 1.56375588 * k + 0.0020672 * t * t + 0.00000215 * t.powi(3));
    let corr = -0.40720 * mp.sin() + 0.17241 * e * m.sin() + 0.01608 * (2.0 * mp).sin() + 0.01039 * (2.0 * f).sin() + 0.00739 * e * (mp - m).sin()
        - 0.00514 * e * (mp + m).sin()
        + 0.00208 * e * e * (2.0 * m).sin()
        - 0.00111 * (mp - 2.0 * f).sin()
        - 0.00057 * (mp + 2.0 * f).sin()
        + 0.00056 * e * (2.0 * mp + m).sin()
        - 0.00042 * (3.0 * mp).sin()
        + 0.00042 * e * (m + 2.0 * f).sin()
        + 0.00038 * e * (m - 2.0 * f).sin()
        - 0.00024 * e * (2.0 * mp - m).sin()
        - 0.00017 * om.sin()
        - 0.00007 * (mp + 2.0 * m).sin()
        + 0.00004 * (2.0 * mp - 2.0 * f).sin()
        + 0.00004 * (3.0 * m).sin()
        + 0.00003 * (mp + m - 2.0 * f).sin()
        + 0.00003 * (2.0 * mp + 2.0 * f).sin()
        - 0.00003 * (mp + m + 2.0 * f).sin()
        + 0.00003 * (mp - m + 2.0 * f).sin()
        - 0.00002 * (mp - m - 2.0 * f).sin()
        - 0.00002 * (3.0 * mp + m).sin()
        + 0.00002 * (4.0 * mp).sin();
    let year = 2000.0 + k / 12.3685;
    jde + corr - delta_t(year) - 1721424.5
}

/// First new moon at or after an R.D. moment.
fn new_moon_at_or_after(moment: f64) -> f64 {
    let mut k = ((moment - nth_new_moon(0.0)) / SYNODIC).floor() - 1.0;
    loop {
        let t = nth_new_moon(k);
        if t >= moment {
            return t;
        }
        k += 1.0;
    }
}

/// Last new moon before an R.D. moment.
fn new_moon_before(moment: f64) -> f64 {
    let mut k = ((moment - nth_new_moon(0.0)) / SYNODIC).floor() + 1.0;
    loop {
        let t = nth_new_moon(k);
        if t < moment {
            return t;
        }
        k -= 1.0;
    }
}

/// Beijing's offset from UT in days: local mean time before 1929, UTC+8 after.
fn china_offset(date: i64) -> f64 {
    if date < 704_008 { 1397.0 / 180.0 / 24.0 } else { 8.0 / 24.0 }
}

fn midnight_in_china(date: i64) -> f64 {
    date as f64 - china_offset(date)
}

fn china_date(moment: f64) -> i64 {
    let d = moment.floor() as i64;
    (moment + china_offset(d)).floor() as i64
}

fn winter_solstice_on_or_before(date: i64) -> i64 {
    // start a little before the estimate and step to the day whose following midnight passes 270°
    let lambda = |d: i64| solar_longitude(midnight_in_china(d + 1));
    let mut d = date - 370;
    // the most recent day where the longitude crosses 270° going forward
    let mut best = d;
    while d <= date {
        let (a, b) = (lambda(d - 1), lambda(d));
        if a <= 270.0 && b > 270.0 && b - a < 5.0 {
            best = d;
        }
        d += 1;
    }
    best
}

fn chinese_new_moon_on_or_after(date: i64) -> i64 {
    china_date(new_moon_at_or_after(midnight_in_china(date)))
}

fn chinese_new_moon_before(date: i64) -> i64 {
    china_date(new_moon_before(midnight_in_china(date)))
}

fn major_solar_term(date: i64) -> i64 {
    let s = solar_longitude(midnight_in_china(date));
    (2 + (s / 30.0).floor() as i64 - 1).rem_euclid(12) + 1
}

fn no_major_solar_term(date: i64) -> bool {
    major_solar_term(date) == major_solar_term(chinese_new_moon_on_or_after(date + 1))
}

fn prior_leap_month(m_start: i64, m: i64) -> bool {
    m >= m_start && (no_major_solar_term(m) || prior_leap_month(m_start, chinese_new_moon_before(m)))
}

fn chinese_new_year_in_sui(date: i64) -> i64 {
    let s1 = winter_solstice_on_or_before(date);
    let s2 = winter_solstice_on_or_before(s1 + 370);
    let m12 = chinese_new_moon_on_or_after(s1 + 1);
    let m13 = chinese_new_moon_on_or_after(m12 + 1);
    let next_m11 = chinese_new_moon_before(s2 + 1);
    if ((next_m11 - m12) as f64 / SYNODIC).round() as i64 == 12 && (no_major_solar_term(m12) || no_major_solar_term(m13)) {
        chinese_new_moon_on_or_after(m13 + 1)
    } else {
        m13
    }
}

fn chinese_new_year_on_or_before(date: i64) -> i64 {
    let ny = chinese_new_year_in_sui(date);
    if date >= ny { ny } else { chinese_new_year_in_sui(date - 180) }
}

/// (cycle, year in cycle 1–60, month 1–12, leap month, day) for an R.D. date.
pub fn chinese_from_fixed(date: i64) -> (i64, i64, i64, bool, i64) {
    let s1 = winter_solstice_on_or_before(date);
    let s2 = winter_solstice_on_or_before(s1 + 370);
    let m12 = chinese_new_moon_on_or_after(s1 + 1);
    let next_m11 = chinese_new_moon_before(s2 + 1);
    let m = chinese_new_moon_before(date + 1);
    let leap_year = ((next_m11 - m12) as f64 / SYNODIC).round() as i64 == 12;
    let month = ((((m - m12) as f64 / SYNODIC).round() as i64 - if leap_year && prior_leap_month(m12, m) { 1 } else { 0 }) - 1).rem_euclid(12) + 1;
    let leap = leap_year && no_major_solar_term(m) && !prior_leap_month(m12, chinese_new_moon_before(m));
    let elapsed = (1.5 - month as f64 / 12.0 + (date - CHINESE_EPOCH) as f64 / TROPICAL).floor() as i64;
    let cycle = (elapsed - 1).div_euclid(60) + 1;
    let year = (elapsed - 1).rem_euclid(60) + 1;
    (cycle, year, month, leap, date - m + 1)
}

pub fn fixed_from_chinese(cycle: i64, year: i64, month: i64, leap: bool, day: i64) -> i64 {
    let mid_year = (CHINESE_EPOCH as f64 + ((cycle - 1) * 60 + year - 1) as f64 * TROPICAL + TROPICAL / 2.0).floor() as i64;
    let new_year = chinese_new_year_on_or_before(mid_year);
    let p = chinese_new_moon_on_or_after(new_year + (month - 1) * 29);
    let d = chinese_from_fixed(p);
    let prior = if month == d.2 && leap == d.3 { p } else { chinese_new_moon_on_or_after(p + 1) };
    prior + day - 1
}

pub const CHINESE_MONTHS: [&str; 12] = ["zhengyue", "eryue", "sanyue", "siyue", "wuyue", "liuyue", "qiyue", "bayue", "jiuyue", "shiyue", "shiyiyue", "shieryue"];
const STEMS: [&str; 10] = ["jia", "yi", "bing", "ding", "wu", "ji", "geng", "xin", "ren", "gui"];
const BRANCHES: [&str; 12] = ["zi", "chou", "yin", "mao", "chen", "si", "wu", "wei", "shen", "you", "xu", "hai"];
const ANIMALS: [&str; 12] = ["Rat", "Ox", "Tiger", "Rabbit", "Dragon", "Snake", "Horse", "Goat", "Monkey", "Rooster", "Dog", "Pig"];

/// The year in the traditional count from the Yellow Emperor (2697 BCE).
fn huangdi_year(cycle: i64, year: i64) -> i64 {
    (cycle - 1) * 60 + year + 60
}

pub fn chinese_text(f: i64) -> String {
    let (c, y, m, leap, d) = chinese_from_fixed(f);
    let i = (y - 1) as usize;
    format!(
        "{d} {}{} ({}month {m}), year {} ({}{}, year of the {})",
        if leap { "run " } else { "" },
        CHINESE_MONTHS[(m - 1) as usize],
        if leap { "leap " } else { "" },
        huangdi_year(c, y),
        STEMS[i % 10],
        BRANCHES[i % 12],
        ANIMALS[i % 12]
    )
}

/// R.D. of Chinese New Year in a Gregorian year.
pub fn chinese_new_year(gy: i32) -> Option<i64> {
    let jun = fixed(NaiveDate::from_ymd_opt(gy, 6, 1)?);
    Some(chinese_new_year_on_or_before(jun))
}

/// Gregorian date of a day in the Chinese calendar, the year counted from the Yellow Emperor.
pub fn fixed_from_chinese_huangdi(year: i64, month: i64, leap: bool, day: i64) -> i64 {
    let elapsed = year - 60;
    fixed_from_chinese((elapsed - 1).div_euclid(60) + 1, (elapsed - 1).rem_euclid(60) + 1, month, leap, day)
}

/// (calendar, date) rows for a Gregorian date.
pub fn rows(d: NaiveDate) -> Vec<(&'static str, String)> {
    let f = fixed(d);
    let mut out = vec![("Julian calendar", julian_text(f)), ("Hebrew calendar", hebrew_text(f)), ("Islamic calendar (tabular)", islamic_text(f))];
    if let Some(p) = persian_text(f) {
        out.push(("Persian calendar", p));
    }
    out.push(("Maya long count", mayan_long_count(f)));
    if (-2000..=3000).contains(&d.year()) {
        out.push(("Chinese calendar", chinese_text(f)));
    }
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
        "chinese" | "chinese lunar" | "lunar" | "chinese lunisolar" | "traditional chinese" | "nongli" => "chinese",
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
        "chinese" => Some(chinese_text(f)),
        _ => None,
    }
}

/// Parse a date written in one of the other calendars, e.g. "25 Shevat 5780",
/// "1 Ramadan 1445", "15 Mehr 1400", "October 5, 1582 Julian".
pub fn parse_other(s: &str) -> Option<(&'static str, NaiveDate)> {
    // "17 jiuyue, 4706 chinese lunar calendar", "1 run siyue 4721"
    let cl = s.trim().to_lowercase();
    let cl = cl.trim_end_matches(" calendar").trim_end_matches(" date");
    let cl = cl.trim_end_matches(" lunar").trim_end_matches(" lunisolar").trim_end_matches(" chinese").trim_end_matches(" (chinese)").trim().trim_end_matches(',');
    let cw: Vec<&str> = cl.split(|c: char| c.is_whitespace() || c == ',').filter(|w| !w.is_empty()).collect();
    if (3..=4).contains(&cw.len()) {
        let leap = cw.len() == 4 && matches!(cw[1], "run" | "leap");
        if let Some(mi) = CHINESE_MONTHS.iter().position(|n| *n == cw[cw.len() - 2]) {
            if let (Ok(d), Ok(y)) = (cw[0].parse::<i64>(), cw[cw.len() - 1].parse::<i64>()) {
                if (1..=30).contains(&d) && (2000..=7000).contains(&y) && (cw.len() == 3 || leap) {
                    return Some(("chinese", from_fixed(fixed_from_chinese_huangdi(y, mi as i64 + 1, leap, d))?));
                }
            }
        }
    }
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
        // Chinese New Year and leap months
        for (y, m, d) in [(2017, 1, 28), (2020, 1, 25), (2023, 1, 22), (2024, 2, 10), (2025, 1, 29), (2026, 2, 17), (1985, 2, 20), (2033, 1, 31), (2034, 2, 19)] {
            assert_eq!(chinese_new_year(y), Some(g(y, m, d)), "{y}");
        }
        assert_eq!(chinese_from_fixed(g(2017, 7, 23)).2, 6);
        assert!(chinese_from_fixed(g(2017, 7, 23)).3, "2017 has a leap sixth month");
        assert!(chinese_from_fixed(g(2020, 5, 23)).3, "2020 has a leap fourth month");
        assert!(chinese_text(g(2024, 2, 10)).starts_with("1 zhengyue (month 1), year 4721 (jiachen, year of the Dragon)"));
        for f in [g(1990, 6, 15), g(2017, 8, 1), g(2023, 4, 1), g(2033, 12, 31)] {
            let (c, y, m, leap, d) = chinese_from_fixed(f);
            assert_eq!(fixed_from_chinese(c, y, m, leap, d), f);
        }
    }
}
