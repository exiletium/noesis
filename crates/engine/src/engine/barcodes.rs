//! Barcode generation: QR codes for text and UPC-A / EAN-13 product codes,
//! drawn as black-and-white module images.
//!
//! "QR code for hello", "QR code for www.wolframalpha.com",
//! "barcode 12345678910", "EAN-13 400638133393".

use super::qr::{self, Ecl};
use super::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

const L_CODES: [&str; 10] = ["0001101", "0011001", "0010011", "0111101", "0100011", "0110001", "0101111", "0111011", "0110111", "0001011"];
const EAN_PARITY: [&str; 10] = ["LLLLLL", "LLGLGG", "LLGGLG", "LLGGGL", "LGLLGG", "LGGLLG", "LGGGLL", "LGLGLG", "LGLGGL", "LGGLGL"];

fn r_code(d: usize) -> String {
    L_CODES[d].chars().map(|c| if c == '0' { '1' } else { '0' }).collect()
}

fn g_code(d: usize) -> String {
    r_code(d).chars().rev().collect()
}

/// GS1 check digit for the given digits (without the check digit).
fn check_digit(ds: &[usize]) -> usize {
    let s: usize = ds.iter().rev().enumerate().map(|(i, d)| if i % 2 == 0 { 3 * d } else { *d }).sum();
    (10 - s % 10) % 10
}

/// The 95-module pattern of an EAN-13 code (UPC-A is EAN-13 with a leading 0).
fn ean13_modules(ds: &[usize; 13]) -> String {
    let mut s = String::from("101");
    let parity = EAN_PARITY[ds[0]].as_bytes();
    for i in 0..6 {
        s += &if parity[i] == b'L' { L_CODES[ds[i + 1]].to_string() } else { g_code(ds[i + 1]) };
    }
    s += "01010";
    for d in &ds[7..13] {
        s += &r_code(*d);
    }
    s + "101"
}

fn bitmap_rows(rows: &[Vec<bool>], quiet: usize) -> Vec<Vec<bool>> {
    let w = rows.first().map_or(0, |r| r.len()) + 2 * quiet;
    let mut out = vec![vec![false; w]; quiet];
    for r in rows {
        let mut row = vec![false; quiet];
        row.extend(r);
        row.extend(vec![false; quiet]);
        out.push(row);
    }
    out.extend(vec![vec![false; w]; quiet]);
    out
}

/// Two text rows per line with half blocks, for terminals.
fn blocks(rows: &[Vec<bool>]) -> String {
    let mut s = String::new();
    for y in (0..rows.len()).step_by(2) {
        for x in 0..rows[y].len() {
            let top = rows[y][x];
            let bot = rows.get(y + 1).is_some_and(|r| r[x]);
            s.push(match (top, bot) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                _ => ' ',
            });
        }
        s.push('\n');
    }
    s.trim_end_matches('\n').to_string()
}

fn qr_code(q: &Query) -> Option<Answer> {
    let raw = q.raw.trim();
    let m = re!(r"(?i)^(?:(?:generate|make|create|draw|show|give)(?: me)? (?:an? |the )?)?qr[ -]?codes?(?: (?:for|of|with|that says|saying|encoding|containing|to))?[: ]\s*(.+)$").captures(raw)?;
    let payload = m[1].trim();
    let payload = payload.strip_prefix('"').and_then(|p| p.strip_suffix('"')).unwrap_or(payload);
    if payload.is_empty() {
        return None;
    }
    let ecl = Ecl::M;
    let code = qr::encode(payload.as_bytes(), ecl, None).or_else(|| qr::encode(payload.as_bytes(), Ecl::L, None))?;
    let rows = bitmap_rows(&code.modules, 4);
    let mut pl = crate::plot::bitmap(&rows);
    pl.x_label = String::new();
    Some(Answer::new(format!("QR code for {payload}"), vec![
        pod("Input", "Input interpretation", text(format!("QR code | \"{payload}\""))),
        primary(pod("Result", "QR code", Content::Plot { plot: Box::new(pl) })),
        pod("Text", "Text rendering", text(blocks(&rows))),
        pod("Properties", "Properties", table(None, vec![
            vec![text("encoded text"), text(payload)],
            vec![text("version"), text(format!("{} ({} × {} modules)", code.version, code.size, code.size))],
            vec![text("error correction"), text(if code.ecl == Ecl::M { "level M (recovers about 15% damage)" } else { "level L (recovers about 7% damage)" })],
            vec![text("encoding mode"), text(format!("byte, {} bytes", payload.len()))],
            vec![text("mask pattern"), text(code.mask.to_string())],
        ])),
    ]))
}

fn product_code(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).trim();
    let m = re!(r"^(?:(?:generate|make|create|draw|show)(?: me)? (?:an? |the )?)?(barcode|bar code|upc|upc-a|upc a|ean|ean-13|ean13|ean 13|gtin|gtin-13)(?: barcode| bar code| code)?(?: (?:for|of|number))? ([\d -]{11,17})$").captures(l)?;
    let digits: Vec<usize> = m[2].chars().filter(|c| c.is_ascii_digit()).map(|c| c as usize - '0' as usize).collect();
    let kind = &m[1];
    let ean_hint = kind.starts_with("ean") || kind.starts_with("gtin");
    // complete the check digit when it is missing; verify it otherwise
    let (full, symbology, given_check) = match digits.len() {
        11 if !ean_hint => {
            let mut d = digits.clone();
            d.push(check_digit(&digits));
            (d, "UPC-A", None)
        }
        12 if !ean_hint => (digits.clone(), "UPC-A", Some(digits[11])),
        12 => {
            let mut d = digits.clone();
            d.push(check_digit(&digits));
            (d, "EAN-13", None)
        }
        13 => (digits.clone(), "EAN-13", Some(digits[12])),
        _ => return None,
    };
    let body = &full[..full.len() - 1];
    let expected = check_digit(body);
    let ean: [usize; 13] = if full.len() == 12 { std::array::from_fn(|i| if i == 0 { 0 } else { full[i - 1] }) } else { full.clone().try_into().ok()? };
    let mods = ean13_modules(&ean);
    let bar: Vec<bool> = mods.chars().map(|c| c == '1').collect();
    let rows = bitmap_rows(&vec![bar.clone(); 40], 9);
    let rows: Vec<Vec<bool>> = rows[9..rows.len() - 9].to_vec();
    let number: String = full.iter().map(|d| d.to_string()).collect();
    let valid = given_check.map_or(true, |c| c == expected);
    let mut props = vec![
        vec![text("symbology"), text(symbology)],
        vec![text("digits"), text(&number)],
        vec![text("check digit"), text(match given_check {
            None => format!("{expected} (computed and appended)"),
            Some(c) if c == expected => format!("{c} (valid)"),
            Some(c) => format!("{c} is wrong: the check digit should be {expected}"),
        })],
        vec![text("width"), text("95 modules (start, 12 digits, centre and end guards)")],
    ];
    if symbology == "UPC-A" {
        props.push(vec![text("as EAN-13"), text(format!("0{number}"))]);
    }
    let pl = crate::plot::bitmap(&rows);
    Some(Answer::new(format!("{symbology} barcode {number}"), vec![
        pod("Input", "Input interpretation", text(format!("{symbology} barcode | {number}"))),
        primary(pod("Result", "Barcode", Content::Plot { plot: Box::new(pl) })),
        pod("Text", "Bar pattern (1 = bar)", text(&mods)),
        pod("Properties", if valid { "Properties" } else { "Properties (invalid check digit)" }, table(None, props)),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    qr_code(q).or_else(|| product_code(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upc_check_and_pattern() {
        // UPC-A 036000291452 (a common textbook example)
        assert_eq!(check_digit(&[0, 3, 6, 0, 0, 0, 2, 9, 1, 4, 5]), 2);
        // EAN-13 4006381333931
        assert_eq!(check_digit(&[4, 0, 0, 6, 3, 8, 1, 3, 3, 3, 9, 3]), 1);
        let m = ean13_modules(&[0, 0, 3, 6, 0, 0, 0, 2, 9, 1, 4, 5, 2]);
        assert_eq!(m.len(), 95);
        assert!(m.starts_with("1010001101"));
    }
}
