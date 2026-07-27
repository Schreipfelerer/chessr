use crate::magic::{BISHOP_BLOCKERS, BISHOP_OFFSETS, compute_sliding_attacks};

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

fn find_magic_bishop(sq: u8, rng: &mut Xorshift64) -> u64 {
    let mask = BISHOP_BLOCKERS[sq as usize];

    // Precompute every (occupancy subset, correct attack bitboard) pair once.
    let mut subsets = Vec::new();
    let mut attacks = Vec::new();
    let mut current = mask;
    loop {
        subsets.push(current);
        attacks.push(compute_sliding_attacks(sq, current, &BISHOP_OFFSETS));
        if current == 0 { break; }
        current = current.wrapping_sub(1) & mask;
    }

    loop {
        // ANDing a few random values biases toward sparse bit patterns,
        // which tend to make better magic candidates.
        let candidate = rng.next_u64() & rng.next_u64() & rng.next_u64();

        let mut used: Vec<Option<u64>> = vec![None; 512];
        let mut ok = true;
        for i in 0..subsets.len() {
            let idx = (subsets[i].wrapping_mul(candidate) >> (64 - 9)) as usize;
            match used[idx] {
                None => used[idx] = Some(attacks[i]),
                Some(existing) if existing == attacks[i] => {} // harmless collision
                _ => { ok = false; break; }
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
        let magic = find_magic_bishop(sq, &mut rng);
        print!("0x{:016X}, ", magic);
    }
    println!("];");
}
