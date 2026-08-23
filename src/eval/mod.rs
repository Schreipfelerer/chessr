use crate::{
    board::{Board, BoardState, Color, Piece, Sq64},
    movegen::BitboardIter,
};
mod pst;
use pst::{
    BISHOP_PST, KING_PST_EG, KING_PST_MG, KNIGHT_PST, PAWN_PST_EG, PAWN_PST_MG, QUEEN_PST, ROOK_PST,
};
mod mobility;
use mobility::mobility;

pub(crate) const PAWN_VALUE: i32 = 100;
pub(crate) const KNIGHT_VALUE: i32 = 315;
pub(crate) const BISHOP_VALUE: i32 = 330;
pub(crate) const ROOK_VALUE: i32 = 500;
pub(crate) const QUEEN_VALUE: i32 = 900;

// Phase weights
const KNIGHT_PHASE: i32 = 1;
const BISHOP_PHASE: i32 = 1;
const ROOK_PHASE: i32 = 2;
const QUEEN_PHASE: i32 = 4;
const TOTAL_PHASE: i32 = KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

const DOUBLE_PAWN_VALUE: i32 = -40;
const PASSED_PAWN_BY_RANK: [i32; 8] = [0, 5, 10, 20, 35, 60, 100, 0];
const BISHOP_PAIR_VALUE: i32 = 30;

#[must_use]
pub fn eval(board_state: &BoardState) -> i32 {
    let board = &board_state.board;
    let phase = game_phase(board);
    let mut score = 0;
    for c in Color::ALL {
        let mut color_score = count_pst(board, c);
        color_score += count_pst_phase(board, c, phase);
        color_score += punish_double_pawns(board, c);
        color_score += score_past_pawns(board, c);
        color_score += bishop_pair(board, c);
        color_score += mobility(board, c);

        if c == board_state.state_info.active_color {
            score += color_score;
        } else {
            score -= color_score;
        }
    }
    score
}

fn bishop_pair(board: &Board, color: Color) -> i32 {
    if board.get_piece_bitboard(color, Piece::Bishop).count_ones() >= 2 {
        BISHOP_PAIR_VALUE
    } else {
        0
    }
}

fn score_past_pawns(board: &Board, color: Color) -> i32 {
    let mut score = 0;
    let op_pawn_bb = board.get_piece_bitboard(!color, Piece::Pawn);
    for sq in BitboardIter(board.get_piece_bitboard(color, Piece::Pawn)) {
        if op_pawn_bb & passed_pawn_mask(sq, color) == 0 {
            // Distance advanced toward promotion, from `color`'s perspective
            let advance = match color {
                Color::White => sq.rank(),
                Color::Black => 7 - sq.rank(),
            };
            score += PASSED_PAWN_BY_RANK[advance as usize];
        }
    }
    score
}

/// Squares an enemy pawn could occupy to block `sq` from being a passed
/// pawn: its own file plus both neighbors, on every rank strictly ahead
/// of `sq` in `color`'s direction of travel.
fn passed_pawn_mask(sq: Sq64, color: Color) -> u64 {
    const FILE_A: u64 = 0x0101_0101_0101_0101;

    let file = sq.file();
    let mut file_band = FILE_A << file;
    if file > 0 {
        file_band |= FILE_A << (file - 1);
    }
    if file < 7 {
        file_band |= FILE_A << (file + 1);
    }

    let rank = sq.rank();
    let ahead_ranks = match color {
        Color::White => !0u64 << ((rank + 1) * 8),
        Color::Black => !0u64 >> ((8 - rank) * 8),
    };

    file_band & ahead_ranks
}

// Subtract Value for every double Pawn
fn punish_double_pawns(board: &Board, color: Color) -> i32 {
    const RANK_MASK: u64 = 0x0101_0101_0101_0101;
    let mut score = 0;
    for i in 0..8 {
        if (board.get_piece_bitboard(color, Piece::Pawn) & (RANK_MASK << i)).count_ones() > 1
        {
            score += DOUBLE_PAWN_VALUE;
        }
    }
    score
}

// Count Piece Sqaure Table Values for all pieces
fn count_pst(board: &Board, color: Color) -> i32 {
    let mirror = |sq: usize| if color == Color::White { sq } else { sq ^ 0o70 };

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
    score
}

// Count Piece Square Table Values for Seperate MG and EG Vaulues
fn count_pst_phase(board: &Board, color: Color, phase: i32) -> i32 {
    let mirror = |sq: usize| if color == Color::White { sq } else { sq ^ 0o70 };

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
    (mg_score * phase + eg_score * (TOTAL_PHASE - phase)) / TOTAL_PHASE
}

/// 0 (pure endgame, no material) .. `TOTAL_PHASE` (both sides at full strength)
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
