use std::cmp::max;
use crate::board::BoardState;
use crate::eval::eval;
use crate::movegen::generate_moves;

pub fn search(board_state: &mut BoardState, depth: u8, alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return eval(board_state);
    }
    let mut alpha = alpha;

    let moves = generate_moves(board_state);
    for mv in moves {
        let undo = board_state.make_move(mv);
        let evaluation = -search(board_state, depth - 1, -beta, -alpha);
        board_state.undo_move(&undo);

        if(evaluation >= beta){ // Move too good, need to prune
            return beta;
        }
        alpha = max(alpha, evaluation)
    }
    alpha
}