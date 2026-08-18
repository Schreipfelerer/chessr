use crate::{
    board::{Board, BoardState, Color, Piece},
    eval::pst::{BISHOP_PST, KING_PST, KNIGHT_PST, PAWN_PST, QUEEN_PST, ROOK_PST},
    movegen::BitboardIter,
};
mod pst;

const PAWN_VALUE: i32 = 1000;
const KNIGHT_VALUE: i32 = 3200;
const BISHOP_VALUE: i32 = 3300;
const ROOK_VALUE: i32 = 5000;
const QUEEN_VALUE: i32 = 9000;

const PST_SCALE: i32 = 10;

#[must_use]
pub fn eval(board_state: &BoardState) -> i32 {
    let board = &board_state.board;
    let mut score = count_material(board, Color::White) - count_material(board, Color::Black);
    score += count_pst(board, Color::White) - count_pst(board, Color::Black);

    let perspective = match board_state.state_info.active_color {
        Color::White => 1,
        Color::Black => -1,
    };
    score * perspective
}

fn count_material(board: &Board, color: Color) -> i32 {
    board.get_piece_bitboard(color, Piece::Pawn).count_ones() as i32 * PAWN_VALUE
        + board.get_piece_bitboard(color, Piece::Knight).count_ones() as i32 * KNIGHT_VALUE
        + board.get_piece_bitboard(color, Piece::Bishop).count_ones() as i32 * BISHOP_VALUE
        + board.get_piece_bitboard(color, Piece::Rook).count_ones() as i32 * ROOK_VALUE
        + board.get_piece_bitboard(color, Piece::Queen).count_ones() as i32 * QUEEN_VALUE
}

fn count_pst(board: &Board, color: Color) -> i32 {
    let mirror = |sq: usize| if color == Color::White { sq } else { sq ^ 56 };

    let mut score = 0;
    for (piece, table) in [
        (Piece::Pawn, &PAWN_PST),
        (Piece::Knight, &KNIGHT_PST),
        (Piece::Bishop, &BISHOP_PST),
        (Piece::Rook, &ROOK_PST),
        (Piece::Queen, &QUEEN_PST),
        (Piece::King, &KING_PST),
    ] {
        for sq in BitboardIter(board.get_piece_bitboard(color, piece)) {
            score += table[mirror(sq.ind())];
        }
    }
    score * PST_SCALE
}
