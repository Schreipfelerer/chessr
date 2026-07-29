use crate::board::{Board, Color, Piece, Sq64};
use crate::movegen::r#const::{BETWEEN, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, get_bishop_moves, get_rook_moves};

/// Finds pinned friendly pieces and the squares each pinned piece may move along.
pub(crate) fn compute_pins(board: &Board, c: Color) -> (u64, [u64; 64]) {
    let mut pins = [u64::MAX; 64];
    let mut pinned_sqs = 0;
    let sq = board.find_king(c);
    //Rooks
    let bb = (board.get_piece_bitboard(!c, Piece::Rook)
        | board.get_piece_bitboard(!c, Piece::Queen))
        & get_rook_moves(sq, board.occ_enemy(c));
    push_pins(board, c, sq, &mut pins, &mut pinned_sqs, bb);
    //Bishops
    let bb = (board.get_piece_bitboard(!c, Piece::Bishop)
        | board.get_piece_bitboard(!c, Piece::Queen))
        & get_bishop_moves(sq, board.occ_enemy(c));
    push_pins(board, c, sq, &mut pins, &mut pinned_sqs, bb);
    (pinned_sqs, pins)
}

/// Records pins caused by enemy sliding pieces aligned with the king.
fn push_pins(
    board: &Board,
    c: Color,
    king_sq: Sq64,
    pins: &mut [u64; 64],
    pinned_sqs: &mut u64,
    bb: u64,
) {
    for pinned_by in BitboardIter(bb) {
        let path = BETWEEN[king_sq.ind()][pinned_by.ind()];
        let path_blockers = path & board.occ_friendly(c);
        if path_blockers.is_power_of_two() {
            // Exactly 1 one in the bitmap
            // Found pin
            *pinned_sqs |= path_blockers;
            pins[path_blockers.trailing_zeros() as usize] = path | pinned_by.mask();
        }
    }
}

/// Returns the enemy pieces currently attacking the active side's king.
pub fn compute_checkers(board: &Board, c: Color) -> u64 {
    let mut bb = 0u64;
    let king_sq = board.find_king(c);
    let co = !c;
    bb |= PAWN_ATTACKS[c as usize][king_sq.ind()] & board.get_piece_bitboard(co, Piece::Pawn);
    bb |= KNIGHT_ATTACKS[king_sq.ind()] & board.get_piece_bitboard(co, Piece::Knight);
    bb |= get_bishop_moves(king_sq, board.occ())
        & (board.get_piece_bitboard(co, Piece::Bishop)
            | board.get_piece_bitboard(co, Piece::Queen));
    bb |= get_rook_moves(king_sq, board.occ())
        & (board.get_piece_bitboard(co, Piece::Rook) | board.get_piece_bitboard(co, Piece::Queen));
    bb
}

/// Checks whether a square is attacked by a given side.
pub fn is_attacked(board: &Board, sq: Sq64, by_color: Color, occ_no_king: u64) -> bool {
    (KNIGHT_ATTACKS[sq.ind()] & board.get_piece_bitboard(by_color, Piece::Knight)) != 0
        || (KING_ATTACKS[sq.ind()] & board.get_piece_bitboard(by_color, Piece::King)) != 0
        || (PAWN_ATTACKS[!by_color as usize][sq.ind()]
            & board.get_piece_bitboard(by_color, Piece::Pawn))
            != 0
        || (get_bishop_moves(sq, occ_no_king)
            & (board.get_piece_bitboard(by_color, Piece::Bishop)
                | board.get_piece_bitboard(by_color, Piece::Queen)))
            != 0
        || (get_rook_moves(sq, occ_no_king)
            & (board.get_piece_bitboard(by_color, Piece::Rook)
                | board.get_piece_bitboard(by_color, Piece::Queen)))
            != 0
}

pub struct BitboardIter(pub u64);

impl Iterator for BitboardIter {
    type Item = Sq64;
    #[inline(always)]
    /// Returns the next set square in the bitboard.
    fn next(&mut self) -> Option<Sq64> {
        if self.0 == 0 {
            return None;
        }
        let sq = self.0.trailing_zeros() as u8;
        self.0 &= self.0 - 1;
        Some(Sq64(sq))
    }
}