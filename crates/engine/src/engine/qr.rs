//! QR code encoder (ISO/IEC 18004): byte mode, versions 1–10, error
//! correction levels L and M, with automatic mask selection.

/// (total codewords, [L: ecc per block, g1 blocks, g1 data, g2 blocks, g2 data], [M: …])
const VERSIONS: [(usize, [usize; 5], [usize; 5]); 10] = [
    (26, [7, 1, 19, 0, 0], [10, 1, 16, 0, 0]),
    (44, [10, 1, 34, 0, 0], [16, 1, 28, 0, 0]),
    (70, [15, 1, 55, 0, 0], [26, 1, 44, 0, 0]),
    (100, [20, 1, 80, 0, 0], [18, 2, 32, 0, 0]),
    (134, [26, 1, 108, 0, 0], [24, 2, 43, 0, 0]),
    (172, [18, 2, 68, 0, 0], [16, 4, 27, 0, 0]),
    (196, [20, 2, 78, 0, 0], [18, 4, 31, 0, 0]),
    (242, [24, 2, 97, 0, 0], [22, 2, 38, 2, 39]),
    (292, [30, 2, 116, 0, 0], [22, 3, 36, 2, 37]),
    (346, [18, 2, 68, 2, 69], [26, 4, 43, 1, 44]),
];

const ALIGN: [&[usize]; 10] = [&[], &[6, 18], &[6, 22], &[6, 26], &[6, 30], &[6, 34], &[6, 22, 38], &[6, 24, 42], &[6, 26, 46], &[6, 28, 50]];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Ecl {
    L,
    M,
}

pub struct Qr {
    pub version: usize,
    pub size: usize,
    pub ecl: Ecl,
    pub mask: u8,
    /// modules[y][x], true = dark
    pub modules: Vec<Vec<bool>>,
}

fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    let mut r = 0u8;
    while b != 0 {
        if b & 1 != 0 {
            r ^= a;
        }
        let hi = a & 0x80;
        a <<= 1;
        if hi != 0 {
            a ^= 0x1D;
        }
        b >>= 1;
    }
    r
}

/// Reed–Solomon remainder of `data` for a generator of degree `n`.
fn rs_ecc(data: &[u8], n: usize) -> Vec<u8> {
    // generator ∏ (x − α^i), i = 0..n, coefficients highest first (leading 1 dropped)
    let mut g = vec![0u8; n];
    g[n - 1] = 1;
    let mut root = 1u8;
    for _ in 0..n {
        for j in 0..n {
            g[j] = gf_mul(g[j], root);
            if j + 1 < n {
                g[j] ^= g[j + 1];
            }
        }
        root = gf_mul(root, 2);
    }
    let mut r = vec![0u8; n];
    for &b in data {
        let f = b ^ r[0];
        r.remove(0);
        r.push(0);
        for j in 0..n {
            r[j] ^= gf_mul(g[j], f);
        }
    }
    r
}

fn block_spec(v: usize, ecl: Ecl) -> [usize; 5] {
    match ecl {
        Ecl::L => VERSIONS[v - 1].1,
        Ecl::M => VERSIONS[v - 1].2,
    }
}

fn data_capacity(v: usize, ecl: Ecl) -> usize {
    let s = block_spec(v, ecl);
    s[1] * s[2] + s[3] * s[4]
}

struct Builder {
    size: usize,
    m: Vec<Vec<bool>>,
    func: Vec<Vec<bool>>,
}

impl Builder {
    fn set(&mut self, x: usize, y: usize, dark: bool) {
        self.m[y][x] = dark;
        self.func[y][x] = true;
    }

    fn finder(&mut self, cx: i64, cy: i64) {
        for dy in -4i64..=4 {
            for dx in -4i64..=4 {
                let (x, y) = (cx + dx, cy + dy);
                if x < 0 || y < 0 || x >= self.size as i64 || y >= self.size as i64 {
                    continue;
                }
                let d = dx.abs().max(dy.abs());
                self.set(x as usize, y as usize, d != 2 && d != 4);
            }
        }
    }

    fn alignment(&mut self, cx: usize, cy: usize) {
        for dy in -2i64..=2 {
            for dx in -2i64..=2 {
                let d = dx.abs().max(dy.abs());
                self.set((cx as i64 + dx) as usize, (cy as i64 + dy) as usize, d != 1);
            }
        }
    }

    fn format_bits(&mut self, ecl: Ecl, mask: u8) {
        let data = (match ecl { Ecl::L => 1u32, Ecl::M => 0 } << 3) | mask as u32;
        let mut rem = data;
        for _ in 0..10 {
            rem = (rem << 1) ^ ((rem >> 9) * 0x537);
        }
        let bits = ((data << 10) | rem) ^ 0x5412;
        let bit = |i: u32| (bits >> i) & 1 != 0;
        for i in 0..=5 {
            self.set(8, i, bit(i as u32));
        }
        self.set(8, 7, bit(6));
        self.set(8, 8, bit(7));
        self.set(7, 8, bit(8));
        for i in 9..15 {
            self.set(14 - i, 8, bit(i as u32));
        }
        let n = self.size;
        for i in 0..8 {
            self.set(n - 1 - i, 8, bit(i as u32));
        }
        for i in 8..15 {
            self.set(8, n - 15 + i, bit(i as u32));
        }
        self.set(8, n - 8, true);
    }

    fn version_bits(&mut self, v: usize) {
        if v < 7 {
            return;
        }
        let mut rem = v as u32;
        for _ in 0..12 {
            rem = (rem << 1) ^ ((rem >> 11) * 0x1F25);
        }
        let bits = ((v as u32) << 12) | rem;
        for i in 0..18 {
            let b = (bits >> i) & 1 != 0;
            let a = self.size - 11 + i % 3;
            let c = i / 3;
            self.set(a, c, b);
            self.set(c, a, b);
        }
    }
}

fn masked(mask: u8, x: usize, y: usize) -> bool {
    match mask {
        0 => (x + y) % 2 == 0,
        1 => y % 2 == 0,
        2 => x % 3 == 0,
        3 => (x + y) % 3 == 0,
        4 => (x / 3 + y / 2) % 2 == 0,
        5 => x * y % 2 + x * y % 3 == 0,
        6 => (x * y % 2 + x * y % 3) % 2 == 0,
        _ => ((x + y) % 2 + x * y % 3) % 2 == 0,
    }
}

fn penalty(m: &[Vec<bool>]) -> u32 {
    let n = m.len();
    let mut p = 0u32;
    let at = |x: usize, y: usize, t: bool| if t { m[x][y] } else { m[y][x] };
    for t in [false, true] {
        for y in 0..n {
            // runs of five or more
            let mut run = 1;
            for x in 1..n {
                if at(x, y, t) == at(x - 1, y, t) {
                    run += 1;
                } else {
                    if run >= 5 {
                        p += run - 2;
                    }
                    run = 1;
                }
            }
            if run >= 5 {
                p += run - 2;
            }
            // finder-like 1:1:3:1:1 with four light modules on one side
            for x in 0..n.saturating_sub(10) {
                let w: Vec<bool> = (0..11).map(|k| at(x + k, y, t)).collect();
                let a = [true, false, true, true, true, false, true, false, false, false, false];
                let b = [false, false, false, false, true, false, true, true, true, false, true];
                if w == a || w == b {
                    p += 40;
                }
            }
        }
    }
    for y in 0..n - 1 {
        for x in 0..n - 1 {
            let c = m[y][x];
            if m[y][x + 1] == c && m[y + 1][x] == c && m[y + 1][x + 1] == c {
                p += 3;
            }
        }
    }
    let dark = m.iter().flatten().filter(|d| **d).count() as i64;
    let total = (n * n) as i64;
    let k = ((dark * 20 - total * 10).abs() + total - 1) / total - 1;
    p + (k as u32) * 10
}

/// Encode bytes; `mask` forces a mask pattern (0–7), otherwise the lowest penalty wins.
pub fn encode(data: &[u8], ecl: Ecl, mask: Option<u8>) -> Option<Qr> {
    let v = (1..=10).find(|v| {
        let cc = if *v <= 9 { 8 } else { 16 };
        4 + cc + 8 * data.len() <= 8 * data_capacity(*v, ecl)
    })?;
    let cap = data_capacity(v, ecl);
    // bit stream
    let mut bits: Vec<bool> = Vec::new();
    let push = |val: u32, n: usize, bits: &mut Vec<bool>| {
        for i in (0..n).rev() {
            bits.push((val >> i) & 1 != 0);
        }
    };
    push(0b0100, 4, &mut bits);
    push(data.len() as u32, if v <= 9 { 8 } else { 16 }, &mut bits);
    for &b in data {
        push(b as u32, 8, &mut bits);
    }
    let room = cap * 8 - bits.len();
    push(0, room.min(4), &mut bits);
    while bits.len() % 8 != 0 {
        bits.push(false);
    }
    let mut cw: Vec<u8> = bits.chunks(8).map(|c| c.iter().fold(0u8, |a, b| (a << 1) | *b as u8)).collect();
    let mut pad = [0xECu8, 0x11].into_iter().cycle();
    while cw.len() < cap {
        cw.push(pad.next()?);
    }
    // blocks and interleaving
    let s = block_spec(v, ecl);
    let mut blocks: Vec<&[u8]> = Vec::new();
    let mut off = 0;
    for (count, len) in [(s[1], s[2]), (s[3], s[4])] {
        for _ in 0..count {
            blocks.push(&cw[off..off + len]);
            off += len;
        }
    }
    let eccs: Vec<Vec<u8>> = blocks.iter().map(|b| rs_ecc(b, s[0])).collect();
    let mut out = Vec::new();
    let maxlen = blocks.iter().map(|b| b.len()).max()?;
    for i in 0..maxlen {
        for b in &blocks {
            if i < b.len() {
                out.push(b[i]);
            }
        }
    }
    for i in 0..s[0] {
        for e in &eccs {
            out.push(e[i]);
        }
    }
    // function patterns
    let size = 17 + 4 * v;
    let mut b = Builder { size, m: vec![vec![false; size]; size], func: vec![vec![false; size]; size] };
    for i in 0..size {
        b.set(6, i, i % 2 == 0);
        b.set(i, 6, i % 2 == 0);
    }
    b.finder(3, 3);
    b.finder(size as i64 - 4, 3);
    b.finder(3, size as i64 - 4);
    let al = ALIGN[v - 1];
    for (i, &ay) in al.iter().enumerate() {
        for (j, &ax) in al.iter().enumerate() {
            let corner = (i == 0 && j == 0) || (i == 0 && j == al.len() - 1) || (i == al.len() - 1 && j == 0);
            if !corner {
                b.alignment(ax, ay);
            }
        }
    }
    b.format_bits(ecl, 0);
    b.version_bits(v);
    // data placement
    let mut i = 0usize;
    let mut right = size as i64 - 1;
    while right >= 1 {
        if right == 6 {
            right = 5;
        }
        for vert in 0..size {
            for j in 0..2 {
                let x = (right - j) as usize;
                let upward = (right + 1) & 2 == 0;
                let y = if upward { size - 1 - vert } else { vert };
                if !b.func[y][x] && i < out.len() * 8 {
                    b.m[y][x] = (out[i >> 3] >> (7 - (i & 7))) & 1 != 0;
                    i += 1;
                }
            }
        }
        right -= 2;
    }
    let apply = |b: &mut Builder, mk: u8| {
        for y in 0..size {
            for x in 0..size {
                if !b.func[y][x] && masked(mk, x, y) {
                    b.m[y][x] = !b.m[y][x];
                }
            }
        }
    };
    let chosen = match mask {
        Some(mk) => mk,
        None => {
            let mut best = (u32::MAX, 0u8);
            for mk in 0..8u8 {
                apply(&mut b, mk);
                b.format_bits(ecl, mk);
                let pen = penalty(&b.m);
                if pen < best.0 {
                    best = (pen, mk);
                }
                apply(&mut b, mk);
            }
            best.1
        }
    };
    apply(&mut b, chosen);
    b.format_bits(ecl, chosen);
    Some(Qr { version: v, size, ecl, mask: chosen, modules: b.m })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_and_version_words() {
        let n = 21;
        let mut b = Builder { size: n, m: vec![vec![false; n]; n], func: vec![vec![false; n]; n] };
        b.format_bits(Ecl::M, 0);
        // bits 14..0 read along row 8 from the left: (0..6, 7, 8) then column 8 upward
        let row: String = [0, 1, 2, 3, 4, 5, 7, 8].iter().map(|x| if b.m[8][*x] { '1' } else { '0' }).collect::<String>() + &[7usize, 5, 4, 3, 2, 1, 0].iter().map(|y| if b.m[*y][8] { '1' } else { '0' }).collect::<String>();
        assert_eq!(row, "101010000010010");
        b.format_bits(Ecl::L, 0);
        let row: String = [0, 1, 2, 3, 4, 5, 7, 8].iter().map(|x| if b.m[8][*x] { '1' } else { '0' }).collect::<String>() + &[7usize, 5, 4, 3, 2, 1, 0].iter().map(|y| if b.m[*y][8] { '1' } else { '0' }).collect::<String>();
        assert_eq!(row, "111011111000100");
        let mut rem = 7u32;
        for _ in 0..12 {
            rem = (rem << 1) ^ ((rem >> 11) * 0x1F25);
        }
        assert_eq!(format!("{:018b}", (7 << 12) | rem), "000111110010010100");
    }

    #[test]
    fn rs_known_vector() {
        // ISO 18004 annex example "01234567" 1-M data codewords and their ECC
        let data = [0x10, 0x20, 0x0C, 0x56, 0x61, 0x80, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11];
        assert_eq!(rs_ecc(&data, 10), vec![0xA5, 0x24, 0xD4, 0xC1, 0xED, 0x36, 0xC7, 0x87, 0x2C, 0x55]);
    }
}
