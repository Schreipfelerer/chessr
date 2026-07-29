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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub fn iterative_deepening(
    board_state: &mut BoardState,
    max_depth: u8,
    time_budget: Option<u64>,
    stop_flag: Arc<AtomicBool>,
) -> Move {
    let start = Instant::now();
    let moves = generate_moves(board_state);
    let mut best_move = moves[0]; // fallback, always legal

    for depth in 1..=max_depth {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }
        if let Some(budget) = time_budget {
            if start.elapsed().as_millis() as u64 >= budget {
                break;
            }
        }

        let Some((mv, score)) = search_root(board_state, depth, &stop_flag, start, time_budget) else {
            break; // aborted mid-depth, keep previous best_move
        };

        best_move = mv;
        println!(
            "info depth {depth} score cp {score} time {} nodes 0 pv {mv}",
            start.elapsed().as_millis()
        );
    }

    best_move
}

fn search(board_state: &mut BoardState, depth: u8, alpha: i32, beta: i32,
          nodes: &mut u64, stop_flag: &AtomicBool, start: Instant, budget: Option<u64>) -> Option<i32> {
    *nodes += 1;
    if *nodes % 2048 == 0 {
        if stop_flag.load(Ordering::Relaxed) {
            return None;
        }
        if let Some(b) = budget {
            if start.elapsed().as_millis() as u64 >= b {
                stop_flag.store(true, Ordering::Relaxed);
                return None;
            }
        }
    }
    // ... existing logic, propagating `?` on recursive calls
}

pub fn search_root(board_state: &mut BoardState, depth: u8) -> (Move, i32) {
    let moves = generate_moves(board_state);
    let mut best_move = moves[0];
    let mut best_score = i32::MIN;
    for mv in moves {
        let undo = board_state.make_move(mv);
        let score = -search(board_state, depth - 1, i32::MIN + 1, -best_score);
        board_state.undo_move(&undo);
        if score > best_score {
            best_score = score;
            best_move = mv;
        }
    }
    (best_move, best_score)
}