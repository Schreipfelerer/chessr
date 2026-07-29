use crate::board::{Board, BoardState, Color, Piece};

const PAWN_VALUE: i32 = 1000;
const KNIGHT_VALUE: i32 = 3200;
const BISHOP_VALUE: i32 = 3300;
const ROOK_VALUE: i32 = 5000;
const QUEEN_VALUE: i32 = 9000;
pub fn eval(board_state: &BoardState) -> i32 {
    let board = &board_state.board;
    let score = count_material(board, Color::White) - count_material(board, Color::Black);

    let perspective = if board_state.state_info.active_color == Color::White {
        1
    } else {
        -1
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
