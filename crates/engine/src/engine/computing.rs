//! Everyday computing questions: ROT13 and Caesar shifts, Base64, CRC-32 and
//! Adler-32 checksums, word n-grams, random passwords and password strength,
//! IPv4 addresses and CIDR blocks, RAID capacity, data transfer times,
//! Unicode characters and ranges, well-known ports, display and Wi-Fi
//! standards, emoticons and which keys (and hands) typing a phrase uses.

use super::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

fn fmt(v: f64) -> String {
    crate::stats::fmt(v)
}

fn unquote(s: &str) -> String {
    let t = s.trim();
    let t = t.strip_prefix(['"', '“', '\'']).unwrap_or(t);
    let t = t.strip_suffix(['"', '”', '\'']).unwrap_or(t);
    t.to_string()
}

fn ans(title: &str, input: String, result_title: &str, result: Content, extra: Vec<Pod>) -> Answer {
    let mut pods = vec![pod("Input", "Input interpretation", text(input)), primary(pod("Result", result_title, result))];
    pods.extend(extra);
    Answer::new(title, pods)
}

fn kv(rows: Vec<(&str, String)>) -> Content {
    table(None, rows.into_iter().map(|(a, b)| vec![text(a), text(b)]).collect())
}

// ---------------------------------------------------------------------------
// text transforms and checksums
// ---------------------------------------------------------------------------

fn shift(s: &str, n: u8) -> String {
    s.chars().map(|c| match c {
        'a'..='z' => (((c as u8 - b'a' + n) % 26) + b'a') as char,
        'A'..='Z' => (((c as u8 - b'A' + n) % 26) + b'A') as char,
        _ => c,
    }).collect()
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_encode(d: &[u8]) -> String {
    let mut out = String::new();
    for ch in d.chunks(3) {
        let b = [ch[0], *ch.get(1).unwrap_or(&0), *ch.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        for i in 0..4 {
            if i <= ch.len() {
                out.push(B64[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let s: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if s.len() % 4 != 0 || s.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for ch in s.chunks(4) {
        let mut n = 0u32;
        let mut pad = 0;
        for &c in ch {
            let v = if c == b'=' {
                pad += 1;
                0
            } else {
                B64.iter().position(|&x| x == c)? as u32
            };
            n = (n << 6) | v;
        }
        let bytes = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        out.extend_from_slice(&bytes[..3 - pad]);
    }
    Some(out)
}

fn crc32(d: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in d {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 == 1 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
        }
    }
    !c
}

fn adler32(d: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in d {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn transforms(l: &str, raw: &str) -> Option<Answer> {
    if let Some(m) = re!(r"(?i)^(?:rot\s*-?\s*(\d+)|caesar(?:\s+cipher)?(?:\s+shift)?\s*(\d+)?)\s*(?:of|encode|encrypt)?\s*:?\s+(.+)$").captures(raw) {
        let n: u8 = m.get(1).or(m.get(2)).and_then(|x| x.as_str().parse().ok()).unwrap_or(3);
        if n == 0 || n > 25 {
            return None;
        }
        let t = unquote(&m[3]);
        return Some(ans("cipher", format!("ROT{n} of \"{t}\""), &format!("ROT{n}"), text(shift(&t, n)), vec![pod("Inverse", "Decoding (shift back)", text(shift(&t, 26 - n)))]));
    }
    if let Some(m) = re!(r"(?i)^(?:base\s*64\s+(?:decode|decoding)|decode\s+base\s*64|from\s+base\s*64)\s*:?\s+(.+)$").captures(raw) {
        let d = b64_decode(&unquote(&m[1]))?;
        let s = String::from_utf8(d).ok()?;
        return Some(ans("base64", format!("Base64 decoding of {}", unquote(&m[1])), "Decoded text", text(s), vec![]));
    }
    if let Some(m) = re!(r"(?i)^(?:base\s*64(?:\s+(?:encode|encoding))?(?:\s+of)?|encode\s+(.+?)\s+(?:in|to|as)\s+base\s*64|to\s+base\s*64)\s*:?\s*(.*)$").captures(raw) {
        let t = unquote(m.get(1).map(|x| x.as_str()).filter(|x| !x.is_empty()).unwrap_or(&m[2]));
        if t.is_empty() {
            return None;
        }
        return Some(ans("base64", format!("Base64 encoding of \"{t}\""), "Base64", text(b64_encode(t.as_bytes())), vec![]));
    }
    if let Some(m) = re!(r"(?i)^(crc-?\s*32|adler-?\s*32)\s*(?:checksum|hash)?\s*(?:of)?\s+(.+)$").captures(raw) {
        let t = unquote(&m[2]);
        let lname = m[1].to_lowercase();
        let (name, v) = if lname.starts_with("crc") { ("CRC-32", crc32(t.as_bytes())) } else { ("Adler-32", adler32(t.as_bytes())) };
        return Some(ans("checksum", format!("{name} of \"{t}\""), name, text(format!("{v:08x}")), vec![pod("Dec", "Decimal", text(v.to_string()))]));
    }
    if let Some(m) = re!(r"(?i)^(?:word\s+)?n-?grams?\s+(?:of\s+)?(.+)$").captures(raw) {
        let t = unquote(&m[1]).to_lowercase();
        let words: Vec<String> = re!(r"[a-z0-9']+").find_iter(&t).map(|w| w.as_str().to_string()).collect();
        if words.len() < 2 {
            return None;
        }
        let mut pods = Vec::new();
        for n in 1..=3usize.min(words.len()) {
            let mut counts: Vec<(String, usize)> = Vec::new();
            for w in words.windows(n) {
                let g = w.join(" ");
                match counts.iter_mut().find(|(k, _)| *k == g) {
                    Some(e) => e.1 += 1,
                    None => counts.push((g, 1)),
                }
            }
            counts.sort_by(|a, b| b.1.cmp(&a.1));
            let rowsv = counts.iter().take(15).map(|(g, c)| vec![text(g.clone()), text(c.to_string())]).collect();
            pods.push(pod(&format!("G{n}"), &format!("{n}-grams ({} distinct)", counts.len()), table(Some(vec![&format!("{n}-gram"), "count"]), rowsv)));
        }
        let mut all = vec![pod("Input", "Input interpretation", text(format!("word n-grams of \"{}\"", unquote(&m[1]))))];
        let mut it = pods.into_iter();
        if let Some(p) = it.next() {
            all.push(primary(p));
        }
        all.extend(it);
        return Some(Answer::new("n-grams", all));
    }
    let _ = l;
    None
}

// ---------------------------------------------------------------------------
// passwords
// ---------------------------------------------------------------------------

fn passwords(l: &str, raw: &str) -> Option<Answer> {
    // "generate a 12 character password" → "generate 1 password of length 12"
    let rewritten = re!(r"^(?:generate|make|create|give me)?\s*(?:an?\s+|(\d+)\s+)?(\d+)[\s-]+(?:characters?|chars?|letters?)(?:\s+long)?\s+(?:random\s+)?((?:alphanumeric|numeric|alphabetic|lowercase|uppercase|strong|secure)\s+)?passwords?$").captures(l).map(|m| format!("generate {} {}password of length {}", m.get(1).map_or("1", |x| x.as_str()), m.get(3).map_or("", |x| x.as_str()), &m[2]));
    let l = rewritten.as_deref().unwrap_or(l);
    if let Some(m) = re!(r"^(?:generate\s+)?(?:(\d+)\s+)?(?:random\s+)?(alphanumeric|numeric|alphabetic|lowercase|uppercase|strong|secure|pronounceable)?\s*passwords?\s+(?:of\s+)?(?:length\s+|with\s+)?(\d+)(?:\s+characters?)?$|^(?:generate\s+)?(?:(\d+)\s+)?(?:random\s+)?(alphanumeric|numeric|strong|secure)?\s*passwords?$").captures(l) {
        let count: usize = m.get(1).or(m.get(4)).and_then(|x| x.as_str().parse().ok()).unwrap_or(5).clamp(1, 50);
        let len: usize = m.get(3).and_then(|x| x.as_str().parse().ok()).unwrap_or(12).clamp(4, 128);
        let kind = m.get(2).or(m.get(5)).map(|x| x.as_str()).unwrap_or("strong");
        let set: &[u8] = match kind {
            "alphanumeric" => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
            "numeric" => b"0123456789",
            "alphabetic" => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
            "lowercase" | "pronounceable" => b"abcdefghijklmnopqrstuvwxyz",
            "uppercase" => b"ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            _ => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*-_=+?",
        };
        // seed from the clock and the address of a stack value; good enough for display, not for keys
        let mut x = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(7) ^ (&count as *const usize as u64);
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let pw: Vec<String> = (0..count).map(|_| (0..len).map(|_| set[(next() % set.len() as u64) as usize] as char).collect()).collect();
        let bits = len as f64 * (set.len() as f64).log2();
        return Some(ans("passwords", format!("{count} random {kind} passwords of length {len}"), "Passwords", text(pw.join("\n")), vec![pod("Entropy", "Entropy of each password", text(format!("{} bits ({} possible characters per position)", fmt(bits), set.len()))), pod("Note", "Note", text("Generated locally with a non-cryptographic generator; use a password manager for real secrets."))]));
    }
    if let Some(m) = re!(r"(?i)^(?:password\s+strength|how\s+strong\s+is(?:\s+the\s+password)?|strength\s+of\s+(?:the\s+)?password)\s*(?:of|for)?\s+(.+)$").captures(raw) {
        let p = unquote(&m[1]);
        if p.is_empty() || p.contains(' ') && p.len() > 64 {
            return None;
        }
        let mut pool = 0;
        let mut classes = Vec::new();
        if p.chars().any(|c| c.is_ascii_lowercase()) {
            pool += 26;
            classes.push("lowercase");
        }
        if p.chars().any(|c| c.is_ascii_uppercase()) {
            pool += 26;
            classes.push("uppercase");
        }
        if p.chars().any(|c| c.is_ascii_digit()) {
            pool += 10;
            classes.push("digits");
        }
        if p.chars().any(|c| !c.is_ascii_alphanumeric()) {
            pool += 33;
            classes.push("symbols");
        }
        // weak patterns: covered characters cost ~5 bits per pattern instead of full entropy
        let lower = p.to_lowercase();
        let lb = lower.as_bytes();
        let mut covered = vec![false; lb.len()];
        let mut weak = Vec::new();
        for pat in ["qwerty", "qwertz", "azerty", "asdf", "zxcv", "password", "passw0rd", "letmein", "admin", "welcome", "iloveyou", "monkey", "dragon", "abc", "012", "123", "234", "345", "456", "567", "678", "789", "987", "876", "765", "654", "543", "432", "321"] {
            let mut found = false;
            for (i, _) in lower.match_indices(pat) {
                found = true;
                for c in covered.iter_mut().skip(i).take(pat.len()) {
                    *c = true;
                }
            }
            if found && !weak.iter().any(|w: &&str| w.contains(pat)) {
                weak.push(pat);
            }
        }
        for i in 0..lb.len().saturating_sub(2) {
            if lb[i] == lb[i + 1] && lb[i + 1] == lb[i + 2] {
                covered[i] = true;
                covered[i + 1] = true;
                covered[i + 2] = true;
                if !weak.contains(&"repeated characters") {
                    weak.push("repeated characters");
                }
            }
        }
        let segments = (0..covered.len()).filter(|&i| covered[i] && (i == 0 || !covered[i - 1])).count();
        let free = covered.iter().filter(|c| !**c).count();
        let bits = free as f64 * (pool as f64).log2() + 5.0 * segments as f64;
        let bits = bits.max(1.0);
        let guesses = 2f64.powf(bits);
        let secs = guesses / 2.0 / 1e10;
        let time = if secs < 1.0 { "less than a second".to_string() } else if secs < 3600.0 { format!("{} minutes", fmt(secs / 60.0)) } else if secs < 86400.0 * 365.0 { format!("{} days", fmt(secs / 86400.0)) } else { format!("{} years", fmt(secs / 86400.0 / 365.25)) };
        let rating = match bits as u32 {
            0..=27 => "very weak",
            28..=35 => "weak",
            36..=59 => "reasonable",
            60..=127 => "strong",
            _ => "very strong",
        };
        let mut rowsv = vec![("length", p.chars().count().to_string()), ("character classes", classes.join(", ")), ("estimated entropy", format!("{} bits", fmt(bits))), ("average time to guess at 10¹⁰ guesses/s", time)];
        if !weak.is_empty() {
            rowsv.push(("weak patterns found", weak.join(", ")));
        }
        return Some(ans("password strength", format!("strength of the password \"{p}\""), "Rating", text(rating), vec![pod("Details", "Details", kv(rowsv))]));
    }
    None
}

// ---------------------------------------------------------------------------
// networks, storage, transfer
// ---------------------------------------------------------------------------

fn ipv4(t: &str) -> Option<u32> {
    let parts: Vec<u32> = t.split('.').map(|p| p.parse::<u32>().ok().filter(|v| *v < 256)).collect::<Option<_>>()?;
    (parts.len() == 4).then(|| (parts[0] << 24) | (parts[1] << 16) | (parts[2] << 8) | parts[3])
}

fn ip_str(v: u32) -> String {
    format!("{}.{}.{}.{}", v >> 24, (v >> 16) & 255, (v >> 8) & 255, v & 255)
}

fn ip_kind(v: u32) -> &'static str {
    let a = v >> 24;
    let b = (v >> 16) & 255;
    match (a, b) {
        (10, _) => "private (RFC 1918, 10.0.0.0/8)",
        (172, 16..=31) => "private (RFC 1918, 172.16.0.0/12)",
        (192, 168) => "private (RFC 1918, 192.168.0.0/16)",
        (127, _) => "loopback (127.0.0.0/8)",
        (169, 254) => "link-local (169.254.0.0/16)",
        (100, 64..=127) => "carrier-grade NAT (100.64.0.0/10)",
        (0, _) => "\"this network\" (0.0.0.0/8)",
        (224..=239, _) => "multicast (224.0.0.0/4)",
        (240..=255, _) => "reserved (240.0.0.0/4)",
        _ => "public",
    }
}

fn networks(l: &str, raw: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:ip\s+(?:address\s+)?|cidr\s+)?(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})(?:\s*/\s*(\d{1,2}))?$").captures(l) {
        let ip = ipv4(&m[1])?;
        let class = match ip >> 24 {
            0..=127 => "A",
            128..=191 => "B",
            192..=223 => "C",
            224..=239 => "D (multicast)",
            _ => "E (reserved)",
        };
        let mut rows = vec![("address", ip_str(ip)), ("type", ip_kind(ip).to_string()), ("historical class", class.to_string()), ("binary", format!("{:08b}.{:08b}.{:08b}.{:08b}", ip >> 24, (ip >> 16) & 255, (ip >> 8) & 255, ip & 255)), ("as an integer", ip.to_string()), ("hexadecimal", format!("0x{ip:08X}")), ("reverse DNS name", format!("{}.{}.{}.{}.in-addr.arpa", ip & 255, (ip >> 8) & 255, (ip >> 16) & 255, ip >> 24))];
        if let Some(p) = m.get(2) {
            let p: u32 = p.as_str().parse().ok()?;
            if p > 32 {
                return None;
            }
            let mask = if p == 0 { 0 } else { u32::MAX << (32 - p) };
            let net = ip & mask;
            let bcast = net | !mask;
            let total = 1u64 << (32 - p);
            let usable = match p {
                32 => 1,
                31 => 2,
                _ => total - 2,
            };
            let (first, last) = if p >= 31 { (net, bcast) } else { (net + 1, bcast - 1) };
            rows = vec![("network address", ip_str(net)), ("broadcast address", if p >= 31 { "none".into() } else { ip_str(bcast) }), ("subnet mask", ip_str(mask)), ("wildcard mask", ip_str(!mask)), ("usable host range", format!("{} – {}", ip_str(first), ip_str(last))), ("addresses in block", total.to_string()), ("usable hosts", usable.to_string()), ("type", ip_kind(net).to_string())];
            return Some(ans("CIDR", format!("CIDR block {}/{p}", ip_str(ip)), "Subnet", kv(rows), vec![]));
        }
        return Some(ans("IP address", format!("IPv4 address {}", ip_str(ip)), "Properties", kv(rows), vec![]));
    }
    if l == "cidr calculator" || l == "cidr" || l == "subnet mask table" || l == "cidr table" {
        let rowsv = [8u32, 12, 16, 20, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32].iter().map(|&p| {
            let mask = if p == 0 { 0 } else { u32::MAX << (32 - p) };
            let total = 1u64 << (32 - p);
            vec![text(format!("/{p}")), text(ip_str(mask)), text(total.to_string()), text(match p { 32 => 1, 31 => 2, _ => total - 2 }.to_string())]
        }).collect();
        return Some(ans("CIDR", "CIDR prefix lengths".into(), "Prefix lengths", table(Some(vec!["prefix", "subnet mask", "addresses", "usable hosts"]), rowsv), vec![pod("Hint", "Usage", text("Give an address with a prefix, e.g. 192.168.10.0/22, for its network, broadcast address and host range."))]));
    }
    // RAID
    if let Some(m) = re!(r"^raid\s*-?\s*(0|1|5|6|10|1\+0|50|60)\s*(?:with|,)?\s*(\d+)\s+(?:drives|disks|hdds?|ssds?)\s*(?:of|,|x|×)?\s*(\d+(?:\.\d+)?)\s*(tb|gb)(?:\s+each)?$").captures(l) {
        let level = m[1].replace("1+0", "10");
        let n: u32 = m[2].parse().ok()?;
        let size: f64 = m[3].parse().ok()?;
        let unit = m[4].to_uppercase();
        let (min, usable, tolerance) = match level.as_str() {
            "0" => (2, n as f64, "none: any failure loses the array".to_string()),
            "1" => (2, 1.0, format!("{} drives (all but one)", n - 1)),
            "5" => (3, n as f64 - 1.0, "1 drive".into()),
            "6" => (4, n as f64 - 2.0, "2 drives".into()),
            "10" => (4, n as f64 / 2.0, "1 drive per mirror pair (at least 1, at most half the drives)".into()),
            "50" => (6, n as f64 - 2.0, "1 drive per RAID 5 group (two groups assumed)".into()),
            "60" => (8, n as f64 - 4.0, "2 drives per RAID 6 group (two groups assumed)".into()),
            _ => return None,
        };
        if n < min || (level == "10" && n % 2 == 1) {
            return Some(ans("RAID", format!("RAID {level} with {n} drives"), "Result", text(format!("RAID {level} needs at least {min} drives{}", if level == "10" { " (an even number)" } else { "" })), vec![]));
        }
        let cap = usable * size;
        let rows = vec![("usable capacity", format!("{} {unit}", fmt(cap))), ("raw capacity", format!("{} {unit}", fmt(n as f64 * size))), ("storage efficiency", format!("{}%", fmt(100.0 * usable / n as f64))), ("fault tolerance", tolerance)];
        return Some(ans("RAID", format!("RAID {level} with {n} × {} {unit} drives", fmt(size)), "Usable capacity", text(format!("{} {unit}", fmt(cap))), vec![pod("Details", "Details", kv(rows))]));
    }
    if l == "raid calculator" || l == "raid levels" || l == "raid" {
        let rowsv = [("RAID 0", "2", "n × size", "none", "striping"), ("RAID 1", "2", "size", "n − 1", "mirroring"), ("RAID 5", "3", "(n − 1) × size", "1", "striping with distributed parity"), ("RAID 6", "4", "(n − 2) × size", "2", "striping with double parity"), ("RAID 10", "4", "n/2 × size", "1 per mirror", "stripe of mirrors")].iter().map(|r| vec![text(r.0), text(r.1), text(r.2), text(r.3), text(r.4)]).collect();
        return Some(ans("RAID", "RAID levels".into(), "RAID levels", table(Some(vec!["level", "min drives", "usable capacity", "drive failures tolerated", "method"]), rowsv), vec![pod("Hint", "Usage", text("Ask e.g. \"RAID5 4 drives 1TB\" for a specific array."))]));
    }
    // data transfer time: "10GB, 6Mb/s" (B = bytes, b = bits)
    if let Some(m) = re!(r"(?i)^(?:data\s+)?(?:transfer|download|upload)\s*(?:time)?\s*(?:for|of)?\s*(\d+(?:\.\d+)?)\s*([kKmMgGtTpP]?i?(?:B|b|bytes?|bits?))\s*(?:,|at|over|with|@)\s*(\d+(?:\.\d+)?)\s*([kKmMgGtT]?(?:B|b|bytes?|bits?))\s*(?:/|per\s+)\s*(?:s|sec|second)$|^(?:data\s+)?(?:transfer|download|upload)\s*(?:time)?\s*(?:for|of)?\s*(\d+(?:\.\d+)?)\s*([kKmMgGtTpP]?i?(?:B|b|bytes?|bits?))\s*(?:,|at|over|with|@)\s*(\d+(?:\.\d+)?)\s*([kKmMgGtT])bps$").captures(raw) {
        let g = |i: usize, j: usize| m.get(i).or(m.get(j)).map(|x| x.as_str().to_string());
        let size: f64 = g(1, 5)?.parse().ok()?;
        let su = g(2, 6)?;
        let rate: f64 = g(3, 7)?.parse().ok()?;
        let ru = if m.get(4).is_some() { m[4].to_string() } else { format!("{}b", &m[8]) };
        let factor = |u: &str| -> Option<(f64, bool)> {
            let first = u.chars().next()?;
            let prefix = "kmgtp".find(first.to_ascii_lowercase());
            let rest = if prefix.is_some() { &u[1..] } else { u };
            let (bin, rest) = match rest.strip_prefix('i') {
                Some(r) => (true, r),
                None => (false, rest),
            };
            let mult = match prefix {
                Some(p) if bin => 1024f64.powi(p as i32 + 1),
                Some(p) => 1000f64.powi(p as i32 + 1),
                None => 1.0,
            };
            let bytes = rest == "B" || rest.to_lowercase().starts_with("byte");
            Some((mult, bytes))
        };
        let (sm, sb) = factor(&su)?;
        let (rm, rb) = factor(&ru)?;
        let size_bits = size * sm * if sb { 8.0 } else { 1.0 };
        let rate_bps = rate * rm * if rb { 8.0 } else { 1.0 };
        if rate_bps <= 0.0 {
            return None;
        }
        let secs = size_bits / rate_bps;
        let h = (secs / 3600.0).floor();
        let mi = ((secs - h * 3600.0) / 60.0).floor();
        let sec = secs - h * 3600.0 - mi * 60.0;
        let hms = format!("{}h {}min {}s", h as u64, mi as u64, fmt((sec * 10.0).round() / 10.0));
        return Some(ans("transfer time", format!("time to transfer {} {su} at {} {ru}/s", fmt(size), fmt(rate)), "Transfer time", text(hms), vec![pod("Secs", "In seconds", text(format!("{} s", fmt(secs)))), pod("Note", "Assumptions", text("B = byte, b = bit; k, M, G are decimal prefixes, Ki, Mi, Gi binary; no protocol overhead"))]));
    }
    // well-known ports
    if let Some(m) = re!(r"^(?:tcp\s+|udp\s+)?port\s+(?:number\s+)?(\d{1,5})$").captures(l) {
        let p: u32 = m[1].parse().ok()?;
        let svc = match p {
            20 => "FTP data",
            21 => "FTP control",
            22 => "SSH (secure shell), SCP, SFTP",
            23 => "Telnet",
            25 => "SMTP (mail transfer)",
            53 => "DNS",
            67 => "DHCP server",
            68 => "DHCP client",
            69 => "TFTP",
            80 => "HTTP (World Wide Web)",
            110 => "POP3",
            119 => "NNTP (Usenet)",
            123 => "NTP (time)",
            137..=139 => "NetBIOS",
            143 => "IMAP",
            161 => "SNMP",
            179 => "BGP",
            389 => "LDAP",
            443 => "HTTPS (HTTP over TLS)",
            445 => "SMB (Microsoft file sharing)",
            465 => "SMTP over TLS",
            514 => "syslog",
            587 => "SMTP mail submission",
            636 => "LDAP over TLS",
            853 => "DNS over TLS",
            993 => "IMAP over TLS",
            995 => "POP3 over TLS",
            1194 => "OpenVPN",
            1433 => "Microsoft SQL Server",
            1521 => "Oracle database",
            3306 => "MySQL",
            3389 => "Remote Desktop Protocol",
            5060 => "SIP",
            5432 => "PostgreSQL",
            5900 => "VNC",
            6379 => "Redis",
            8080 => "HTTP alternate",
            8443 => "HTTPS alternate",
            9418 => "Git",
            27017 => "MongoDB",
            _ => "",
        };
        let range = match p {
            0..=1023 => "well-known (system) port",
            1024..=49151 => "registered port",
            49152..=65535 => "dynamic / private port",
            _ => return None,
        };
        let res = if svc.is_empty() { format!("no common service; {range}") } else { svc.to_string() };
        return Some(ans("port", format!("port {p}"), "Service", text(res), vec![pod("Range", "Port range", text(range))]));
    }
    None
}

// ---------------------------------------------------------------------------
// standards: displays and wireless networks
// ---------------------------------------------------------------------------

const DISPLAYS: &[(&str, u32, u32)] = &[
    ("CGA", 320, 200), ("EGA", 640, 350), ("VGA", 640, 480), ("SVGA", 800, 600), ("WVGA", 800, 480), ("XGA", 1024, 768), ("XGA+", 1152, 864), ("WXGA", 1280, 800), ("SXGA", 1280, 1024), ("HD", 1280, 720), ("WXGA+", 1440, 900), ("SXGA+", 1400, 1050), ("HD+", 1600, 900), ("UXGA", 1600, 1200), ("WSXGA+", 1680, 1050), ("FHD", 1920, 1080), ("WUXGA", 1920, 1200), ("QWXGA", 2048, 1152), ("QXGA", 2048, 1536), ("WQHD", 2560, 1440), ("WQXGA", 2560, 1600), ("QSXGA", 2560, 2048), ("UWQHD", 3440, 1440), ("QUXGA", 3200, 2400), ("4K UHD", 3840, 2160), ("DCI 4K", 4096, 2160), ("5K", 5120, 2880), ("8K UHD", 7680, 4320),
];

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn standards(l: &str, raw: &str) -> Option<Answer> {
    let names: Vec<String> = re!(r"\s*(?:,|\band\b|\bvs\.?)\s*").split(l.trim_end_matches(" display").trim_end_matches(" resolution")).map(|s| s.trim().trim_end_matches(" display").to_string()).filter(|s| !s.is_empty()).collect();
    let hits: Vec<&(&str, u32, u32)> = names.iter().filter_map(|n| DISPLAYS.iter().find(|d| d.0.eq_ignore_ascii_case(n) || (n == "full hd" && d.0 == "FHD") || (n == "1080p" && d.0 == "FHD") || (n == "720p" && d.0 == "HD") || (n == "4k" && d.0 == "4K UHD"))).collect();
    if !hits.is_empty() && hits.len() == names.len() {
        let rowsv: Vec<Vec<Content>> = hits.iter().map(|(n, w, h)| {
            let g = gcd(*w, *h);
            vec![text(*n), text(format!("{w} × {h}")), text(format!("{}:{}", w / g, h / g)), text(format!("{} megapixels", fmt((*w as f64) * (*h as f64) / 1e6)))]
        }).collect();
        let title = hits.iter().map(|h| h.0).collect::<Vec<_>>().join(", ");
        return Some(ans("display", format!("{title} display standard"), "Resolution", table(Some(vec!["standard", "pixels", "aspect ratio", "pixel count"]), rowsv), vec![]));
    }
    if let Some(m) = re!(r"^(\d+(?:\.\d+)?)\s*(?:megapixels?|mp)\s*,?\s*(\d+)\s*(?:dpi|ppi)(?:\s+(?:display|print|image))?$").captures(l) {
        let mp: f64 = m[1].parse().ok()?;
        let dpi: f64 = m[2].parse().ok()?;
        let px = mp * 1e6;
        let (w, h) = ((px * 4.0 / 3.0).sqrt(), (px * 3.0 / 4.0).sqrt());
        let rows = vec![("pixels (4:3)", format!("about {} × {}", w.round(), h.round())), ("size at that density (4:3)", format!("{} in × {} in ({} cm × {} cm)", fmt(w / dpi), fmt(h / dpi), fmt(w / dpi * 2.54), fmt(h / dpi * 2.54))), ("area", format!("{} in²", fmt(px / dpi / dpi))), ("size (3:2)", format!("{} in × {} in", fmt((px * 1.5).sqrt() / dpi), fmt((px / 1.5).sqrt() / dpi)))];
        return Some(ans("display", format!("{} megapixel image at {} dpi", fmt(mp), fmt(dpi)), "Physical size", kv(rows), vec![]));
    }
    if let Some(m) = re!(r"^(?:ieee\s+)?802\.11\s*([a-z]{1,2})(?:\s+(?:wireless|wi-?fi|network|standard)s?)*$").captures(l) {
        let (gen, year, band, rate) = match &m[1] {
            "a" => ("", "1999", "5 GHz", "54 Mbit/s"),
            "b" => ("Wi-Fi 1", "1999", "2.4 GHz", "11 Mbit/s"),
            "g" => ("Wi-Fi 3", "2003", "2.4 GHz", "54 Mbit/s"),
            "n" => ("Wi-Fi 4", "2009", "2.4 and 5 GHz", "600 Mbit/s (4 MIMO streams, 40 MHz channels)"),
            "ac" => ("Wi-Fi 5", "2013", "5 GHz", "6.9 Gbit/s (8 streams, 160 MHz channels)"),
            "ax" => ("Wi-Fi 6 / 6E", "2021", "2.4, 5 and 6 GHz", "9.6 Gbit/s"),
            "be" => ("Wi-Fi 7", "2024", "2.4, 5 and 6 GHz", "46 Gbit/s (320 MHz channels)"),
            "ad" => ("WiGig", "2012", "60 GHz", "6.8 Gbit/s"),
            _ => return None,
        };
        let mut rows = vec![("frequency bands", band.to_string()), ("maximum data rate", rate.to_string()), ("standard approved", year.to_string())];
        if !gen.is_empty() {
            rows.insert(0, ("Wi-Fi Alliance name", gen.to_string()));
        }
        return Some(ans("wireless", format!("IEEE 802.11{} wireless networking", &m[1]), "Properties", kv(rows), vec![]));
    }
    let emoticon = match raw.trim() {
        ":-)" | ":)" => Some("smiley: happy, joking"),
        ":-(" | ":(" => Some("frown: sad, unhappy"),
        ";-)" | ";)" => Some("wink: joking, not serious"),
        ":-D" | ":D" => Some("big grin, laughing"),
        ":-P" | ":P" | ":-p" | ":p" => Some("sticking out the tongue: teasing"),
        ":-O" | ":O" | ":-o" => Some("surprise, shock"),
        ":-|" | ":|" => Some("indifference, straight face"),
        ":'(" | ":'-(" => Some("crying"),
        ":-/" | ":/" => Some("skeptical, uneasy"),
        "<3" => Some("heart: love"),
        "</3" => Some("broken heart"),
        "^_^" => Some("happy (Japanese style)"),
        "o_O" | "O_o" => Some("confused, taken aback"),
        "xD" | "XD" => Some("laughing hard"),
        _ => None,
    };
    if let Some(e) = emoticon {
        return Some(ans("emoticon", format!("emoticon {}", raw.trim()), "Meaning", text(e), vec![pod("Note", "Origin", text("The first smiley :-) was proposed by Scott Fahlman on 19 September 1982 on a Carnegie Mellon bulletin board."))]));
    }
    None
}

// ---------------------------------------------------------------------------
// Unicode
// ---------------------------------------------------------------------------

const NAMED: &[(&str, u32)] = &[
    ("em dash", 0x2014), ("en dash", 0x2013), ("hyphen", 0x2010), ("minus sign", 0x2212), ("ellipsis", 0x2026), ("horizontal ellipsis", 0x2026), ("bullet", 0x2022), ("degree sign", 0xB0), ("section sign", 0xA7), ("pilcrow", 0xB6), ("copyright sign", 0xA9), ("registered sign", 0xAE), ("trademark sign", 0x2122), ("euro sign", 0x20AC), ("pound sign", 0xA3), ("yen sign", 0xA5), ("non-breaking space", 0xA0), ("left double quotation mark", 0x201C), ("right double quotation mark", 0x201D), ("interrobang", 0x203D), ("dagger", 0x2020), ("double dagger", 0x2021), ("infinity", 0x221E), ("snowman", 0x2603), ("check mark", 0x2713), ("multiplication sign", 0xD7), ("division sign", 0xF7), ("plus-minus sign", 0xB1), ("micro sign", 0xB5), ("per mille sign", 0x2030), ("zero width space", 0x200B), ("byte order mark", 0xFEFF), ("replacement character", 0xFFFD),
];

fn block_of(c: u32) -> &'static str {
    match c {
        0..=0x7F => "Basic Latin",
        0x80..=0xFF => "Latin-1 Supplement",
        0x100..=0x24F => "Latin Extended",
        0x370..=0x3FF => "Greek and Coptic",
        0x400..=0x4FF => "Cyrillic",
        0x590..=0x5FF => "Hebrew",
        0x600..=0x6FF => "Arabic",
        0x900..=0x97F => "Devanagari",
        0x2000..=0x206F => "General Punctuation",
        0x20A0..=0x20CF => "Currency Symbols",
        0x2100..=0x214F => "Letterlike Symbols",
        0x2190..=0x21FF => "Arrows",
        0x2200..=0x22FF => "Mathematical Operators",
        0x2300..=0x23FF => "Miscellaneous Technical",
        0x2500..=0x257F => "Box Drawing",
        0x25A0..=0x25FF => "Geometric Shapes",
        0x2600..=0x26FF => "Miscellaneous Symbols",
        0x2700..=0x27BF => "Dingbats",
        0x3040..=0x309F => "Hiragana",
        0x30A0..=0x30FF => "Katakana",
        0x4E00..=0x9FFF => "CJK Unified Ideographs",
        0xAC00..=0xD7AF => "Hangul Syllables",
        0x1F300..=0x1F5FF => "Miscellaneous Symbols and Pictographs",
        0x1F600..=0x1F64F => "Emoticons",
        0x1F900..=0x1F9FF => "Supplemental Symbols and Pictographs",
        _ => "",
    }
}

fn char_rows(c: char) -> Vec<(&'static str, String)> {
    let u = c as u32;
    let mut b = [0u8; 4];
    let utf8 = c.encode_utf8(&mut b).bytes().map(|x| format!("{x:02X}")).collect::<Vec<_>>().join(" ");
    let mut w = [0u16; 2];
    let utf16 = c.encode_utf16(&mut w).iter().map(|x| format!("{x:04X}")).collect::<Vec<_>>().join(" ");
    let mut rows = vec![("character", c.to_string()), ("code point", format!("U+{u:04X} (decimal {u})")), ("UTF-8", utf8), ("UTF-16", utf16), ("HTML entity", format!("&#{u}; or &#x{u:X};"))];
    if let Some((n, _)) = NAMED.iter().find(|(_, v)| *v == u) {
        rows.insert(1, ("name", n.to_uppercase()));
    }
    let blk = block_of(u);
    if !blk.is_empty() {
        rows.push(("block", blk.to_string()));
    }
    let cat = if c.is_alphabetic() { if c.is_uppercase() { "uppercase letter" } else if c.is_lowercase() { "lowercase letter" } else { "letter" } } else if c.is_numeric() { "number" } else if c.is_whitespace() { "space separator" } else if c.is_control() { "control character" } else if c.is_ascii_punctuation() || (0x2010..=0x2027).contains(&u) { "punctuation" } else { "symbol" };
    rows.push(("kind", cat.to_string()));
    rows
}

fn unicode(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:unicode|unicode characters?|code points?|u\+)\s*(?:from\s+)?(?:u\+|0x)?([0-9a-f]+)\s*(?:to|through|thru|-|–)\s*(?:u\+|0x)?([0-9a-f]+)$").captures(l) {
        let hex = l.contains("u+") || l.contains("0x") || m[1].chars().any(|c| c.is_ascii_alphabetic());
        let parse = |s: &str| if hex { u32::from_str_radix(s, 16).ok() } else { s.parse().ok() };
        let (a, b) = (parse(&m[1])?, parse(&m[2])?);
        if b < a || b - a > 256 {
            return None;
        }
        let rowsv: Vec<Vec<Content>> = (a..=b).filter_map(char::from_u32).map(|c| vec![text(c.to_string()), text(format!("U+{:04X}", c as u32)), text((c as u32).to_string())]).collect();
        let chars: String = (a..=b).filter_map(char::from_u32).collect();
        return Some(ans("unicode", format!("Unicode characters {} to {}", if hex { format!("U+{a:04X}") } else { a.to_string() }, if hex { format!("U+{b:04X}") } else { b.to_string() }), "Characters", text(chars), vec![pod("Table", "Code points", table(Some(vec!["character", "hex", "decimal"]), rowsv))]));
    }
    let c: Option<char> = if let Some(m) = re!(r"^(?:unicode|unicode character|character|code point|char)\s+(?:u\+|0x)([0-9a-f]{2,6})$|^u\+([0-9a-f]{4,6})$").captures(l) {
        char::from_u32(u32::from_str_radix(m.get(1).or(m.get(2))?.as_str(), 16).ok()?)
    } else if let Some(m) = re!(r"^(?:unicode|unicode character|code point)\s+(\d{1,7})$").captures(l) {
        char::from_u32(m[1].parse().ok()?)
    } else if let Some((_, u)) = NAMED.iter().find(|(n, _)| *n == l || format!("{n} character") == l || format!("unicode {n}") == l || n.replace(" sign", " symbol") == l || format!("{n} symbol") == l) {
        char::from_u32(*u)
    } else if let Some(m) = re!(r"^(?:unicode|code point|utf-?8)\s+(?:of|for)\s+(.+)$").captures(l) {
        let t = unquote(&m[1]);
        let mut it = t.chars();
        match (it.next(), it.next()) {
            (Some(c), None) => Some(c),
            _ => None,
        }
    } else {
        None
    };
    let c = c?;
    Some(ans("unicode", format!("Unicode character U+{:04X}", c as u32), "Character", text(c.to_string()), vec![pod("Props", "Properties", kv(char_rows(c)))]))
}

/// base letter + combining mark name → precomposed character
const DIACRITICS: &[(&str, &str)] = &[
    ("umlaut", "aäAÄeëEËiïIÏoöOÖuüUÜyÿYŸ"),
    ("acute", "aáAÁeéEÉiíIÍoóOÓuúUÚyýYÝcćCĆnńNŃsśSŚzźZŹ"),
    ("grave", "aàAÀeèEÈiìIÌoòOÒuùUÙ"),
    ("circumflex", "aâAÂeêEÊiîIÎoôOÔuûUÛ"),
    ("tilde", "aãAÃnñNÑoõOÕ"),
    ("cedilla", "cçCÇsşSŞ"),
    ("ring", "aåAÅuůUŮ"),
    ("caron", "cčCČeěEĚnňNŇrřRŘsšSŠzžZŽ"),
    ("macron", "aāAĀeēEĒiīIĪoōOŌuūUŪ"),
    ("ogonek", "aąAĄeęEĘ"),
    ("stroke", "lłLŁoøOØdđDĐ"),
];

/// (language, letters beyond the basic Latin alphabet, number of letters, note)
const ALPHABETS: &[(&str, &str, u32, &str)] = &[
    ("french", "é è ê ë à â î ï ô û ù ü ÿ ç œ æ", 26, "accented letters are not counted as separate letters"),
    ("german", "ä ö ü ß", 26, "the umlauts and ß (Eszett) are usually counted in addition to the 26 letters"),
    ("spanish", "ñ á é í ó ú ü", 27, "ñ is a separate letter; accents mark stress"),
    ("portuguese", "á â ã à ç é ê í ó ô õ ú", 26, ""),
    ("italian", "à è é ì í î ò ó ù ú", 21, "j, k, w, x and y appear only in loanwords"),
    ("polish", "ą ć ę ł ń ó ś ź ż", 32, "q, v and x appear only in loanwords"),
    ("turkish", "ç ğ ı İ ö ş ü", 29, "q, w and x are not used"),
    ("swedish", "å ä ö", 29, ""),
    ("finnish", "å ä ö", 29, ""),
    ("norwegian", "æ ø å", 29, ""),
    ("danish", "æ ø å", 29, ""),
    ("icelandic", "á ð é í ó ú ý þ æ ö", 32, "c, q, w and z are not used"),
    ("czech", "á č ď é ě í ň ó ř š ť ú ů ý ž", 42, "ch counts as a single letter"),
    ("romanian", "ă â î ș ț", 31, ""),
    ("hungarian", "á é í ó ö ő ú ü ű", 44, "includes digraphs such as cs, gy, ly, ny, sz, ty, zs and the trigraph dzs"),
    ("dutch", "ij (digraph) and occasional é ë ï ö", 26, ""),
];

fn html_entity(e: &str) -> Option<char> {
    if let Some(d) = e.strip_prefix("#x") {
        return char::from_u32(u32::from_str_radix(d, 16).ok()?);
    }
    if let Some(d) = e.strip_prefix('#') {
        return char::from_u32(d.parse().ok()?);
    }
    Some(match e {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{a0}',
        "copy" => '©',
        "reg" => '®',
        "trade" => '™',
        "deg" => '°',
        "euro" => '€',
        "pound" => '£',
        "yen" => '¥',
        "cent" => '¢',
        "sect" => '§',
        "para" => '¶',
        "middot" => '·',
        "times" => '×',
        "divide" => '÷',
        "plusmn" => '±',
        "micro" => 'µ',
        "mdash" => '—',
        "ndash" => '–',
        "hellip" => '…',
        "laquo" => '«',
        "raquo" => '»',
        "hearts" => '♥',
        "spades" => '♠',
        "clubs" => '♣',
        "diams" => '♦',
        "infin" => '∞',
        "pi" => 'π',
        _ => return None,
    })
}

fn control_name(u: u32) -> Option<&'static str> {
    const NAMES: [&str; 33] = ["NUL (null)", "SOH (start of heading)", "STX (start of text)", "ETX (end of text)", "EOT (end of transmission)", "ENQ (enquiry)", "ACK (acknowledge)", "BEL (bell)", "BS (backspace)", "HT (horizontal tab)", "LF (line feed)", "VT (vertical tab)", "FF (form feed)", "CR (carriage return)", "SO (shift out)", "SI (shift in)", "DLE (data link escape)", "DC1 (device control 1)", "DC2 (device control 2)", "DC3 (device control 3)", "DC4 (device control 4)", "NAK (negative acknowledge)", "SYN (synchronous idle)", "ETB (end of transmission block)", "CAN (cancel)", "EM (end of medium)", "SUB (substitute)", "ESC (escape)", "FS (file separator)", "GS (group separator)", "RS (record separator)", "US (unit separator)", "space"];
    match u {
        0..=32 => Some(NAMES[u as usize]),
        127 => Some("DEL (delete)"),
        _ => None,
    }
}

fn characters(l: &str) -> Option<Answer> {
    // ASCII ranges: "ASCII 32 to 39"
    if let Some(m) = re!(r"^ascii (?:characters? |codes? |table )?(?:from )?(\d{1,3}) ?(?:to|through|thru|-|–) ?(\d{1,3})$").captures(l) {
        let (a, b): (u32, u32) = (m[1].parse().ok()?, m[2].parse().ok()?);
        if a > b || b > 127 {
            return None;
        }
        let t: Vec<Vec<Content>> = (a..=b).map(|u| vec![text(u.to_string()), text(format!("{u:02X}")), text(format!("{u:07b}")), text(control_name(u).map(|x| x.to_string()).unwrap_or_else(|| char::from_u32(u).unwrap().to_string()))]).collect();
        let chars: String = (a..=b).filter(|u| *u > 32 && *u != 127).filter_map(char::from_u32).collect();
        let shown = if a <= 32 && b >= 32 { format!("(space){chars}") } else { chars };
        return Some(ans("ascii", format!("ASCII characters {a} to {b}"), "Characters", text(shown), vec![pod("Table", "Codes", table(Some(vec!["decimal", "hex", "binary", "character"]), t))]));
    }
    // HTML entities: "&#42;", "&amp;"
    if let Some(m) = re!(r"^(?:html (?:entity |character )?)?&(#x[0-9a-f]+|#\d+|[a-z]+);?$").captures(l) {
        let c = html_entity(&m[1])?;
        return Some(ans("html entity", format!("HTML entity &{};", &m[1]), "Character", text(c.to_string()), vec![pod("Props", "Properties", kv(char_rows(c)))]));
    }
    // letters with diacritics: "u umlaut", "e with acute accent"
    if let Some(m) = re!(r"^(?:(lowercase|small|uppercase|capital) )?(?:letter )?([a-z]) (?:with (?:an? )?)?(umlaut|diaeresis|dieresis|trema|acute|grave|circumflex|tilde|cedilla|ring|caron|hacek|háček|macron|ogonek|stroke|slash)(?: accent| mark)?$").captures(l) {
        let mark = match &m[3] {
            "diaeresis" | "dieresis" | "trema" => "umlaut",
            "hacek" | "háček" => "caron",
            "slash" => "stroke",
            x => x,
        };
        let upper = matches!(m.get(1).map(|x| x.as_str()), Some("uppercase" | "capital"));
        let base = if upper { m[2].to_uppercase() } else { m[2].to_string() };
        let table_s: Vec<char> = DIACRITICS.iter().find(|d| d.0 == mark)?.1.chars().collect();
        let c = table_s.chunks(2).find(|p| p[0].to_string() == base).map(|p| p[1])?;
        return Some(ans("character", format!("{base} with {mark}"), "Character", text(c.to_string()), vec![pod("Props", "Properties", kv(char_rows(c)))]));
    }
    // "French letters", "German alphabet"
    if let Some(m) = re!(r"^(?:the )?([a-z]+) (?:letters|alphabet|special characters|special letters|accented letters|accents|diacritics)$").captures(l) {
        let (lang, extra, count, note) = ALPHABETS.iter().find(|a| a.0 == &m[1])?;
        let mut r = vec![("letters beyond A–Z", extra.to_string()), ("number of letters", count.to_string())];
        if !note.is_empty() {
            r.push(("note", note.to_string()));
        }
        let title = format!("{}{} alphabet", lang[..1].to_uppercase(), &lang[1..]);
        return Some(ans("alphabet", title, "Special letters", text(extra.to_string()), vec![pod("Props", "Details", kv(r))]));
    }
    // "letter K"
    if let Some(m) = re!(r"^(?:the )?(?:(lowercase|small|uppercase|capital) )?letter ([a-z])$").captures(l) {
        let lower = matches!(m.get(1).map(|x| x.as_str()), Some("lowercase" | "small"));
        let c = m[2].chars().next()?;
        let c = if lower { c } else { c.to_ascii_uppercase() };
        let idx = (c.to_ascii_lowercase() as u8 - b'a') as usize;
        let morse = crate::domains::everyday::MORSE.iter().find(|(k, _)| *k == c.to_ascii_lowercase()).map(|(_, v)| v.to_string()).unwrap_or_default();
        let mut r = vec![("position in the alphabet", format!("{} of 26", idx + 1)), ("type", if "aeiou".contains(c.to_ascii_lowercase()) { "vowel".into() } else { "consonant".to_string() }), ("uppercase / lowercase", format!("{} / {}", c.to_ascii_uppercase(), c.to_ascii_lowercase())), ("ASCII codes", format!("{} / {}", c.to_ascii_uppercase() as u32, c.to_ascii_lowercase() as u32)), ("NATO phonetic", crate::domains::everyday::NATO[idx].to_string()), ("Morse code", morse)];
        r.extend(char_rows(c).into_iter().filter(|(k, _)| matches!(*k, "code point" | "UTF-8" | "HTML entity")));
        return Some(ans("letter", format!("letter {c}"), "Letter", text(c.to_string()), vec![pod("Props", "Properties", kv(r))]));
    }
    None
}

// ---------------------------------------------------------------------------
// keyboards
// ---------------------------------------------------------------------------

fn keyboards(raw: &str) -> Option<Answer> {
    let m = re!(r#"(?i)^(?:(?:us\s+)?(qwerty|dvorak|us english|english)(?:\s+keyboard)?\s+)?(?:keys\s+(?:used\s+)?(?:for\s+)?typing|typing|keystrokes\s+for)\s+["“](.+)["”]$|^(?:us\s+)?(dvorak|qwerty)(?:\s+keyboard)?\s+["“](.+)["”]$"#).captures(raw.trim())?;
    let layout = m.get(1).or(m.get(3)).map(|x| x.as_str().to_lowercase()).unwrap_or_else(|| "qwerty".into());
    let dvorak = layout == "dvorak";
    let phrase = m.get(2).or(m.get(4))?.as_str();
    let rows_q = ["`1234567890-=", "qwertyuiop[]\\", "asdfghjkl;'", "zxcvbnm,./"];
    let rows_d = ["`1234567890[]", "',.pyfgcrl/=\\", "aoeuidhtns-", ";qjkxbmwvz"];
    let rows = if dvorak { rows_d } else { rows_q };
    let shifted = |c: char| -> char {
        match c {
            '!' => '1', '@' => '2', '#' => '3', '$' => '4', '%' => '5', '^' => '6', '&' => '7', '*' => '8', '(' => '9', ')' => '0', '_' => '-', '+' => '=', '{' => '[', '}' => ']', '|' => '\\', ':' => ';', '"' => '\'', '<' => ',', '>' => '.', '?' => '/', '~' => '`',
            _ => c.to_ascii_lowercase(),
        }
    };
    let (mut left, mut right, mut shift, mut space) = (0, 0, 0, 0);
    let mut row_counts = [0usize; 4];
    let mut keys: Vec<char> = Vec::new();
    for c in phrase.chars() {
        if c == ' ' {
            space += 1;
            continue;
        }
        if c.is_ascii_uppercase() || "!@#$%^&*()_+{}|:\"<>?~".contains(c) {
            shift += 1;
        }
        let k = shifted(c);
        let Some((ri, row)) = rows.iter().enumerate().find(|(_, r)| r.contains(k)) else { continue };
        let col = row.find(k)?;
        // columns 0–5 of the number row and 0–4 of the letter rows belong to the left hand
        let is_left = if ri == 0 { col <= 5 } else { col <= 4 };
        if is_left { left += 1 } else { right += 1 }
        row_counts[ri] += 1;
        if !keys.contains(&k) {
            keys.push(k);
        }
    }
    if left + right == 0 {
        return None;
    }
    let tot = (left + right) as f64;
    let res = vec![
        ("keys pressed", keys.iter().collect::<String>()),
        ("left hand", format!("{left} keystrokes ({}%)", fmt(100.0 * left as f64 / tot))),
        ("right hand", format!("{right} keystrokes ({}%)", fmt(100.0 * right as f64 / tot))),
        ("number row / top / home / bottom", format!("{} / {} / {} / {}", row_counts[0], row_counts[1], row_counts[2], row_counts[3])),
        ("home-row share", format!("{}%", fmt(100.0 * row_counts[2] as f64 / tot))),
        ("shifted characters", shift.to_string()),
        ("spaces", space.to_string()),
    ];
    Some(ans("keyboard", format!("typing \"{phrase}\" on a US {} keyboard", if dvorak { "Dvorak" } else { "QWERTY" }), "Keystrokes", kv(res), vec![]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let raw = q.clean.trim().trim_end_matches(['?']).trim();
    let l = raw.to_lowercase();
    let l = l.trim_end_matches('.').to_string();
    transforms(&l, raw).or_else(|| passwords(&l, raw)).or_else(|| networks(&l, raw)).or_else(|| standards(&l, raw)).or_else(|| characters(&l)).or_else(|| unicode(&l)).or_else(|| keyboards(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksums_and_codes() {
        assert_eq!(crc32(b"The quick brown fox jumps over the lazy dog"), 0x414FA339);
        assert_eq!(adler32(b"Wikipedia"), 0x11E60398);
        assert_eq!(b64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(b64_decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(shift("Hello", 13), "Uryyb");
    }
}
