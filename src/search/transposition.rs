use crate::{board::Move, search::MATE_THRESHOLD};

const TT_BITS: u32 = 20;
const TT_SIZE: usize = 2_usize.pow(TT_BITS);

pub struct TranspositionTable {
    table: Box<[Option<TranspositionEntry>]>,
}
impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            table: vec![None; TT_SIZE].into_boxed_slice(),
        }
    }
    pub fn get_entry(&self, hash: u64) -> Option<TranspositionEntry> {
        let entry = self.table[(hash >> (64 - TT_BITS)) as usize];
        if entry?.hash == hash { entry } else { None }
    }
    pub fn insert(&mut self, entry: TranspositionEntry) {
        self.table[(entry.hash >> (64 - TT_BITS)) as usize] = Some(entry);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TranspositionEntry {
    pub(crate) best_move: Option<Move>,
    pub(crate) depth: u8,
    score: i32,
    hash: u64,
    node_type: Bound,
}
impl TranspositionEntry {
    pub fn new(
        best_move: Option<Move>,
        depth: u8,
        score: i32,
        hash: u64,
        node_type: Bound,
        ply: u8,
    ) -> Self {
        let score_adjusted = {
            if score > MATE_THRESHOLD {
                score + ply as i32
            } else if score < -MATE_THRESHOLD {
                score - ply as i32
            } else {
                score
            }
        };
        TranspositionEntry {
            best_move,
            depth,
            score: score_adjusted,
            hash,
            node_type,
        }
    }
    pub fn is_valid(&self, alpha: i32, beta: i32) -> bool {
        match self.node_type {
            Bound::Exact => true,
            Bound::Lower => self.score >= beta,
            Bound::Upper => self.score <= alpha,
        }
    }
    pub fn get_score(&self, ply: u8) -> i32 {
        if self.score > MATE_THRESHOLD {
            self.score - ply as i32
        } else if self.score < -MATE_THRESHOLD {
            self.score + ply as i32
        } else {
            self.score
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Bound {
    Exact,
    Upper,
    Lower,
}
