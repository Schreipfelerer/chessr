use crate::{
    board::{Board, BoardState, Color, Piece},
    movegen::BitboardIter,
};
mod pst;
use pst::{
    BISHOP_PST, KING_PST_EG, KING_PST_MG, KNIGHT_PST, PAWN_PST_EG, PAWN_PST_MG, QUEEN_PST, ROOK_PST,
};

const PAWN_VALUE: i32 = 1000;
const KNIGHT_VALUE: i32 = 3200;
const BISHOP_VALUE: i32 = 3300;
const ROOK_VALUE: i32 = 5000;
const QUEEN_VALUE: i32 = 9000;

const PST_SCALE: i32 = 10;

// Phase weights
const KNIGHT_PHASE: i32 = 1;
const BISHOP_PHASE: i32 = 1;
const ROOK_PHASE: i32 = 2;
const QUEEN_PHASE: i32 = 4;
const TOTAL_PHASE: i32 = KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

#[must_use]
pub fn eval(board_state: &BoardState) -> i32 {
    let board = &board_state.board;
    let phase = game_phase(board);
    let mut score = count_pst(board, Color::White) - count_pst(board, Color::Black);
    score +=
        count_pst_phase(board, Color::White, phase) - count_pst_phase(board, Color::Black, phase);

    let perspective = match board_state.state_info.active_color {
        Color::White => 1,
        Color::Black => -1,
    };
    score * perspective
}

// Count Piece Sqaure Table Values for all pieces
fn count_pst(board: &Board, color: Color) -> i32 {
    let mirror = |sq: usize| if color == Color::White { sq } else { sq ^ 56 };

    let mut score = 0;
    for (piece, table, value) in [
        (Piece::Knight, &KNIGHT_PST, KNIGHT_VALUE),
        (Piece::Bishop, &BISHOP_PST, BISHOP_VALUE),
        (Piece::Rook, &ROOK_PST, ROOK_VALUE),
        (Piece::Queen, &QUEEN_PST, QUEEN_VALUE),
    ] {
        for sq in BitboardIter(board.get_piece_bitboard(color, piece)) {
            score += table[mirror(sq.ind())] + value;
        }
    }
    score * PST_SCALE
}

// Count Piece Square Table Values for Seperate MG and EG Vaulues
fn count_pst_phase(board: &Board, color: Color, phase: i32) -> i32 {
    let mirror = |sq: usize| if color == Color::White { sq } else { sq ^ 56 };

    let mut mg_score = 0;
    let mut eg_score = 0;
    for (piece, mg_table, eg_table, value) in [
        (Piece::Pawn, &PAWN_PST_MG, &PAWN_PST_EG, PAWN_VALUE),
        (Piece::King, &KING_PST_MG, &KING_PST_EG, 0),
    ] {
        for sq in BitboardIter(board.get_piece_bitboard(color, piece)) {
            mg_score += mg_table[mirror(sq.ind())] + value;
            eg_score += eg_table[mirror(sq.ind())] + value;
        }
    }
    ((mg_score * phase + eg_score * (TOTAL_PHASE - phase)) / TOTAL_PHASE) * PST_SCALE
}

/// 0 (pure endgame, no material) .. TOTAL_PHASE (both sides at full strength)
fn game_phase(board: &Board) -> i32 {
    let count = |c: Color, p: Piece| board.get_piece_bitboard(c, p).count_ones() as i32;
    let phase: i32 = Color::ALL
        .iter()
        .map(|&c| {
            count(c, Piece::Knight) * KNIGHT_PHASE
                + count(c, Piece::Bishop) * BISHOP_PHASE
                + count(c, Piece::Rook) * ROOK_PHASE
                + count(c, Piece::Queen) * QUEEN_PHASE
        })
        .sum();
    phase.min(TOTAL_PHASE) // promotions could theoretically overshoot
}
