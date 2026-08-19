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

const DOUBLE_PAWN_VALUE: i32 = -40;
const PAST_PAWN_VALUE: i32 = 90;

#[must_use]
pub fn eval(board_state: &BoardState) -> i32 {
    let board = &board_state.board;
    let phase = game_phase(board);
    let mut score = count_pst(board, Color::White) - count_pst(board, Color::Black);
    score +=
        count_pst_phase(board, Color::White, phase) - count_pst_phase(board, Color::Black, phase);
    score += punish_double_pawns(board, Color::White) - punish_double_pawns(board, Color::Black);
    score += score_past_pawns(board, Color::White) - score_past_pawns(board, Color::Black);

    let perspective = match board_state.state_info.active_color {
        Color::White => 1,
        Color::Black => -1,
    };
    score * perspective
}

fn score_past_pawns(board: &Board, color: Color) -> i32 {
    let mut score = 0;
    const PAST_MASK: u64 = 0x0383_8383_8383_8380;
    const PAST_MASK_NO_L: u64 = 0x0303_0303_0303_0300;
    const PAST_MASK_NO_R: u64 = 0x0181_8181_8181_8180;
    let op_pawn_bb = board.get_piece_bitboard(!color, Piece::Pawn);
    for sq in BitboardIter(board.get_piece_bitboard(color, Piece::Pawn)) {
        let mask = match sq.file() {
            0 => PAST_MASK_NO_L << sq.0,
            7 => PAST_MASK_NO_R << sq.0,
            _ => PAST_MASK << sq.0,
        };
        if op_pawn_bb & mask == 0 {
            score += PAST_PAWN_VALUE;
        }
    }
    score
}

// Subtract Value for every double Pawn
fn punish_double_pawns(board: &Board, color: Color) -> i32 {
    let mut score = 0;
    const RANK_MASK: u64 = 0x0101_0101_0101_0101;
    for i in 0..8 {
        if (board.get_piece_bitboard(color, Piece::Pawn) & (RANK_MASK << i * 8)).count_ones() > 1 {
            score += DOUBLE_PAWN_VALUE;
        }
    }
    score
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
