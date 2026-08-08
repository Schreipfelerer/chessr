use rand::rngs::StdRng;
use rand::{Rng, RngExt, SeedableRng};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZobristKeys {
    // [piece_color][piece_type][square]
    pub pieces: [[[u64; 64]; 6]; 2], 
    pub castling: [u64; 16],
    pub en_passant: [u64; 8],
    pub side_to_move: u64,
}

impl ZobristKeys {
    pub fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(133767);
        let mut p = [[[0_u64; 64]; 6]; 2];
        for color in 0..2{
            for piece in 0..6{
                for sq in 0..64{
                    p[color][piece][sq] = rng.next_u64();
                }
            }
        }

        let mut c = [0_u64; 16];
        rng.fill(&mut c);

        let mut e = [0_u64; 8];
        rng.fill(&mut e);
        ZobristKeys {
            pieces: p,
            castling: c,
            en_passant: e,
            side_to_move: rng.next_u64(),
        }
    }
}

