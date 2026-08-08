use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

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
        
        ZobristKeys {
            pieces: [[[rng.next_u64(); 64]; 6]; 2],
            castling: [rng.next_u64(); 16],
            en_passant: [rng.next_u64(); 8],
            side_to_move: rng.next_u64(),
        }
    }
}

