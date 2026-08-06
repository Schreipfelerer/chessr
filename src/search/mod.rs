use crate::board::{BoardState, Move};
use crate::eval::eval;
use crate::movegen::{generate_moves, is_check};
use std::cmp::max;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

pub fn iterative_deepening(
    board_state: &mut BoardState,
    max_depth: u8,
    time_budget: Option<u64>,
    stop_flag: &Arc<AtomicBool>,
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

        let Some((mv, score)) = search_root(board_state, depth, &stop_flag, start, time_budget)
        else {
            break; // aborted mid-depth, keep previous best_move
        };

        best_move = mv;
        println!(
            "info depth {depth} score mp {score} time {} nodes 0 pv {mv}",
            start.elapsed().as_millis()
        );
    }

    best_move
}

fn search_root(
    board_state: &mut BoardState,
    depth: u8,
    stop_flag: &AtomicBool,
    start: Instant,
    budget_ms: Option<u64>,
) -> Option<(Move, i32)> {
    let moves = generate_moves(board_state);
    let mut best_move = moves[0];
    let mut best_score = i32::MIN + 1;
    let mut nodes = 0_u64;
    for mv in moves {
        let undo = board_state.make_move(mv);
        let score = search(
            board_state,
            depth - 1,
            i32::MIN + 1,
            -best_score,
            &mut nodes,
            stop_flag,
            start,
            budget_ms,
            0,
        )
        .map(|e| -e);
        board_state.undo_move(&undo);
        if score? > best_score {
            best_score = score?;
            best_move = mv;
        }
    }
    Some((best_move, best_score))
}

fn search(
    board_state: &mut BoardState,
    depth: u8,
    alpha: i32,
    beta: i32,
    nodes: &mut u64,
    stop_flag: &AtomicBool,
    start: Instant,
    budget_ms: Option<u64>,
    ply: u8,
) -> Option<i32> {
    *nodes += 1;
    if *nodes % 2048 == 0 {
        if stop_flag.load(Ordering::Relaxed) {
            return None;
        }
        if let Some(b) = budget_ms {
            if start.elapsed().as_millis() as u64 >= b {
                stop_flag.store(true, Ordering::Relaxed);
                return None;
            }
        }
    }
    if depth == 0 {
        return Some(eval(board_state));
    }
    let mut alpha = alpha;

    let moves = generate_moves(board_state);
    if moves.is_empty() {
        return Some(match is_check(board_state) {
            true => i32::MAX - ply as i32,
            false => 0,
        });
    }
    for mv in moves {
        let undo = board_state.make_move(mv);
        let evaluation = search(
            board_state,
            depth - 1,
            -beta,
            -alpha,
            nodes,
            stop_flag,
            start,
            budget_ms,
            ply + 1,
        )
        .map(|e| -e);
        board_state.undo_move(&undo);

        if evaluation? >= beta {
            // Move too good, need to prune
            return Some(beta);
        }
        alpha = max(alpha, evaluation?)
    }
    Some(alpha)
}
