use crate::board::Move;

const TT_BITS: u32 = 20;
const TT_SIZE: usize = 2_usize.pow(TT_BITS);

#[derive(Clone, Copy)]
pub struct TranspositionTable {
    table: [Option<TranspositionEntry>; TT_SIZE],
}
impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            table: [None; TT_SIZE],
        }
    }
    pub fn get_entry(self, hash: u64) -> Option<TranspositionEntry> {
        let entry = self.table[(hash >> (64 - TT_BITS)) as usize];
        if entry?.hash == hash { entry } else { None }
    }
    pub fn insert(mut self, entry: TranspositionEntry) {
        self.table[(entry.hash >> (64 - TT_BITS)) as usize] = Some(entry);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TranspositionEntry {
    pub(crate) best_move: Move,
    pub(crate) depth: u8,
    pub(crate) score: i32,
    pub(crate) hash: u64,
    pub(crate) node_type: Bound,
}
impl TranspositionEntry {
    pub fn is_valid(self, alpha: i32, beta: i32) -> bool {
        match self.node_type {
            Bound::Exact => true,
            Bound::Lower => self.score >= beta,
            Bound::Upper => self.score <= alpha,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Bound {
    Exact,
    Upper,
    Lower,
}
