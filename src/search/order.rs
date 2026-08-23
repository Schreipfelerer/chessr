use arrayvec::ArrayVec;

use crate::board::{BoardState, Move, MoveFlag, Piece};
use crate::eval::{BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};

fn piece_value(p: Piece) -> i32 {
    match p {
        Piece::Pawn => PAWN_VALUE,
        Piece::Knight => KNIGHT_VALUE,
        Piece::Bishop => BISHOP_VALUE,
        Piece::Rook => ROOK_VALUE,
        Piece::Queen => QUEEN_VALUE,
        Piece::King => 0,
    }
}

/// Higher = search first. Non-captures score 0 so `sort_by_key` w/
/// `Reverse` keeps them after captures without needing a special case.
fn mvv_lva_score(board_state: &BoardState, mv: Move) -> i32 {
    let flags = mv.flags();
    if !flags.is_capture() {
        return 0;
    }
    let attacker = board_state.board.get_piece_at(mv.source());
    let victim = if flags == MoveFlag::EnPassant {
        Piece::Pawn // target sq is empty for e.p., captured pawn is elsewhere
    } else {
        board_state.board.get_piece_at(mv.target())
    };
    piece_value(victim) * 16 - piece_value(attacker)
}

/// TT move (if present and legal) goes first; everything after is sorted
/// by MVV-LVA, quiets falling to the back with score 0.
pub fn order_moves(
    board_state: &BoardState,
    moves: &mut ArrayVec<Move, 256>,
    tt_move: Option<Move>,
) {
    let mut start = 0;
    if let Some(mh) = tt_move {
        if let Some(pos) = moves.iter().position(|&m| m == mh) {
            moves.swap(0, pos);
            start = 1;
        }
    }
    moves[start..].sort_by_key(|&mv| std::cmp::Reverse(mvv_lva_score(board_state, mv)));
}
