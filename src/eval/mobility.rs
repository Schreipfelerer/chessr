use crate::{board::{Board, Color, Piece}, movegen::{BitboardIter, r#const::{KNIGHT_ATTACKS, get_bishop_moves, get_rook_moves}}};

const KNIGHT_MOBILITY: i32 = 4;
const BISHOP_MOBILITY: i32 = 3;
const ROOK_MOBILITY: i32 = 2;
const QUEEN_MOBILITY: i32 = 1;

pub fn mobility(board: &Board, color: Color) -> i32 {
    let own = board.occ_friendly(color);
    let occ = board.occ();
    let mut score = 0;

    for sq in BitboardIter(board.get_piece_bitboard(color, Piece::Knight)) {
        score += (KNIGHT_ATTACKS[sq.ind()] & !own).count_ones() as i32 * KNIGHT_MOBILITY;
    }
    for sq in BitboardIter(board.get_piece_bitboard(color, Piece::Bishop)) {
        score += (get_bishop_moves(sq, occ) & !own).count_ones() as i32 * BISHOP_MOBILITY;
    }
    for sq in BitboardIter(board.get_piece_bitboard(color, Piece::Rook)) {
        score += (get_rook_moves(sq, occ) & !own).count_ones() as i32 * ROOK_MOBILITY;
    }
    for sq in BitboardIter(board.get_piece_bitboard(color, Piece::Queen)) {
        let attacks = get_bishop_moves(sq, occ) | get_rook_moves(sq, occ);
        score += (attacks & !own).count_ones() as i32 * QUEEN_MOBILITY;
    }
    score
}
