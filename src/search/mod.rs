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
) -> (Move, u32) {
    let start = Instant::now();
    let moves = generate_moves(board_state);
    let mut best_move = moves[0]; // fallback, always legal
    let mut nodes = 0u32;

    for depth in 1..=max_depth {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        let Some((mv, score)) = search_root(
            board_state,
            depth,
            stop_flag,
            start,
            time_budget,
            &mut nodes,
        ) else {
            break; // aborted mid-depth, keep previous best_move
        };

        best_move = mv;
        println!(
            "info depth {depth} score cp {} time {} nodes {nodes} pv {mv}",
            score / 10,
            start.elapsed().as_millis(),
        );
    }

    (best_move, nodes)
}

fn search_root(
    board_state: &mut BoardState,
    depth: u8,
    stop_flag: &AtomicBool,
    start: Instant,
    budget_ms: Option<u64>,
    nodes: &mut u32,
) -> Option<(Move, i32)> {
    let moves = generate_moves(board_state);
    let mut best_move = moves[0];
    let mut best_score = i32::MIN + 1;
    for mv in moves {
        let undo = board_state.make_move(mv);
        let score = search(
            board_state,
            depth - 1,
            i32::MIN + 1,
            -best_score,
            nodes,
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
    nodes: &mut u32,
    stop_flag: &AtomicBool,
    start: Instant,
    budget_ms: Option<u64>,
    ply: u8,
) -> Option<i32> {
    *nodes += 1;
    if (*nodes).is_multiple_of(2048) {
        if stop_flag.load(Ordering::Relaxed) {
            return None;
        }
        if let Some(b) = budget_ms
            && start.elapsed().as_millis() as u64 + 10 >= b
        {
            stop_flag.store(true, Ordering::Relaxed);
            return None;
        }
    }
    if depth == 0 {
        return Some(eval(board_state));
    }
    let mut alpha = alpha;

    let moves = generate_moves(board_state);
    if moves.is_empty() {
        return Some(if is_check(board_state) {
            i32::MAX - ply as i32
        } else {
            0
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
        alpha = max(alpha, evaluation?);
    }
    Some(alpha)
}
