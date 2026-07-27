use chessr::{
    movegen::magic::{BISHOP_BLOCKERS, ROOK_BLOCKERS},
    magic_generator::{BISHOP_OFFSETS, ROOK_OFFSETS, compute_sliding_attacks},
};

struct Xorshift64(u64);
impl Xorshift64 {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn find_magic(
    sq: u8,
    rng: &mut Xorshift64,
    blockers: [u64; 64],
    offsets: &[i8; 4],
    bits: u32,
) -> u64 {
    let mask = blockers[sq as usize];

    // Precompute every (occupancy subset, correct attack bitboard) pair once.
    let mut subsets = Vec::new();
    let mut attacks = Vec::new();
    let mut current = mask;
    loop {
        subsets.push(current);
        attacks.push(compute_sliding_attacks(sq, current, offsets));
        if current == 0 {
            break;
        }
        current = current.wrapping_sub(1) & mask;
    }

    loop {
        // ANDing a few random values biases toward sparse bit patterns,
        // which tend to make better magic candidates.
        let candidate = rng.next_u64() & rng.next_u64() & rng.next_u64();

        let mut used: Vec<Option<u64>> = vec![None; 2_u16.pow(bits) as usize];
        let mut ok = true;
        for i in 0..subsets.len() {
            let idx = (subsets[i].wrapping_mul(candidate) >> (64 - bits)) as usize;
            match used[idx] {
                None => used[idx] = Some(attacks[i]),
                Some(existing) if existing == attacks[i] => {} // harmless collision
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            return candidate;
        }
    }
}

pub fn find_magic_bishops() {
    let mut rng = Xorshift64(0x1234_5678_9abc_def0); // any nonzero seed
    print!("pub const BISHOP_MAGIC: [u64; 64] = [");
    for sq in 0..64 {
        let magic = find_magic(sq, &mut rng, *BISHOP_BLOCKERS, &BISHOP_OFFSETS, 9);
        print!("0x{:016X}, ", magic);
    }
    println!("];");
}
pub fn find_magic_rooks() {
    let mut rng = Xorshift64(0x1234_5678_9abc_def0); // any nonzero seed
    print!("pub const ROOK_MAGIC: [u64; 64] = [");
    for sq in 0..64 {
        let magic = find_magic(sq, &mut rng, *ROOK_BLOCKERS, &ROOK_OFFSETS, 12);
        print!("0x{:016X}, ", magic);
    }
    println!("];");
}

fn main() {
    find_magic_bishops();
    find_magic_rooks();
}
