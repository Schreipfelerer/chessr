pub const BISHOP_OFFSETS: [i8; 4] = [15, 17, -15, -17];
pub const ROOK_OFFSETS: [i8; 4] = [1, -1, 16, -16];

pub const fn compute_magic<const N: usize>(
    offsets: &[i8; 4],
    blockers: [u64; 64],
    bits: u8,
    magic_table: [u64; 64],
) -> [[u64; N]; 64] {
    let mut table = [[0u64; N]; 64];
    let mut sq = 0u8;
    while sq < 64 {
        table[sq as usize] = compute_magic_square(sq, offsets, blockers, bits, magic_table);
        sq += 1;
    }
    table
}

const fn compute_magic_square<const N: usize>(
    sq: u8,
    offsets: &[i8; 4],
    blockers: [u64; 64],
    bits: u8,
    magic_table: [u64; 64],
) -> [u64; N] {
    let bb = blockers[sq as usize];
    let mut table = [0u64; N];
    let mut current_mask = bb;
    while current_mask != 0 {
        table[magic_index(sq, current_mask, bits, magic_table)] =
            compute_sliding_attacks(sq, current_mask, offsets);
        current_mask = current_mask.wrapping_sub(1) & bb
    }

    table[0] = compute_sliding_attacks(sq, 0u64, offsets);
    table
}

pub const fn compute_sliding_attacks(sq: u8, blockers: u64, offsets: &[i8]) -> u64 {
    let mut bb = 0u64;
    let sq_88 = Sq64(sq).to_sq88();
    let mut i = 0;
    while i < offsets.len() {
        let offset = offsets[i];
        let mut to_sq = sq_88.step(offset);
        while to_sq.is_on_board() {
            let to_sq_64 = to_sq.to_sq64();
            let mask = 1 << to_sq_64.0;
            bb |= mask;
            if blockers & mask != 0 {
                break;
            }
            to_sq = to_sq.step(offset)
        }
        i += 1;
    }
    bb
}

const fn magic_index(sq: u8, bb: u64, bits: u8, magic_table: [u64; 64]) -> usize {
    (bb.wrapping_mul(magic_table[sq as usize]) >> 64 - bits) as usize
}

pub const fn compute_blockers(offsets: &[i8; 4]) -> [u64; 64] {
    let mut table = [0u64; 64];
    let mut sq = 0u8;
    while sq < 64 {
        let sq_88 = Sq64(sq).to_sq88();
        let mut bb = 0u64;
        let mut i = 0;
        while i < 4 {
            let offset = offsets[i];
            let mut to_88 = sq_88.step(offset);
            let mut prev_sq: Option<Sq64> = None;
            while to_88.is_on_board() {
                if let Some(sq) = prev_sq {
                    bb |= 1u64 << sq.0;
                }
                prev_sq = Some(to_88.to_sq64());
                to_88 = to_88.step(offset);
            }
            i += 1;
        }
        table[sq as usize] = bb;
        sq += 1;
    }
    table
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sq64(pub u8);
impl Sq64 {
    #[inline(always)]
    pub const fn to_sq88(self) -> Sq88 {
        Sq88(self.0 + (self.0 & !7))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sq88(pub u8);
impl Sq88 {
    #[inline(always)]
    pub const fn is_on_board(self) -> bool {
        (self.0 & 0x88) == 0
    }

    #[inline(always)]
    pub const fn to_sq64(self) -> Sq64 {
        Sq64((self.0 + (self.0 & 7)) >> 1)
    }
    #[inline(always)]
    pub const fn step(self, offset: i8) -> Sq88 {
        Sq88(((self.0 as i8).wrapping_add(offset)) as u8)
    }
}

pub const fn compute_between() -> [[u64; 64]; 64] {
    let mut table = [[0; 64]; 64];
    let mut a = 0_i8;
    while a < 64 {
        let file_a = a & 7;
        let rank_a = a >> 3;
        let mut b = 0_i8;
        while b < 64 {
            let file_b = b & 7;
            let rank_b = b >> 3;

            let dr = rank_b - rank_a;
            let adr = rank_a.abs_diff(rank_b);
            let df = file_b - file_a;
            let adf = file_a.abs_diff(file_b);
            if (adr == adf || dr == 0 || df == 0) && a != b {
                let sr = dr.signum();
                let sf = df.signum();

                let mut current_f = file_a + sf;
                let mut current_r = rank_a + sr;
                let mut mask = 0u64;

                // Step towards b, filling all squares STRICTLY between a and b
                while current_f != file_b || current_r != rank_b {
                    let sq = (current_r * 8 + current_f) as u8;
                    mask |= 1u64 << sq;

                    current_f += sf;
                    current_r += sr;
                }

                table[a as usize][b as usize] = mask;
            }
            b += 1;
        }
        a += 1;
    }
    table
}
