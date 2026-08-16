mod transposition;
pub use transposition::TranspositionTable;

use crate::board::{BoardState, Move};
use crate::eval::eval;
use crate::movegen::{generate_moves, is_check};
use crate::search::transposition::{Bound, TranspositionEntry};
use std::cmp::max;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

pub fn iterative_deepening(
    board_state: &mut BoardState,
    max_depth: u8,
    time_budget: Option<u64>,
    stop_flag: &Arc<AtomicBool>,
    tt: &mut TranspositionTable,
) -> (Move, u32) {
    let start = Instant::now();
    let moves = generate_moves(board_state, false);
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
            tt,
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
    tt: &mut TranspositionTable,
) -> Option<(Move, i32)> {
    let moves = generate_moves(board_state, false);
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
            tt,
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
    a: i32,
    beta: i32,
    nodes: &mut u32,
    stop_flag: &AtomicBool,
    start: Instant,
    budget_ms: Option<u64>,
    ply: u8,
    tt: &mut TranspositionTable,
) -> Option<i32> {
    if board_state.is_repetition() {
        return Some(0);
    }

    if depth == 0 {
        return quiescence_search(
            board_state,
            4,
            a,
            beta,
            nodes,
            stop_flag,
            start,
            budget_ms,
            ply,
        );
    }
    *nodes += 1;
    if (*nodes).is_multiple_of(2048) {
        if stop_flag.load(Ordering::Relaxed) {
            return None;
        }
        if let Some(b) = budget_ms
            && start.elapsed().as_millis() as u64 + 20 >= b
        {
            stop_flag.store(true, Ordering::Relaxed);
            return None;
        }
    }

    let mut move_hint: Option<Move> = None;
    if let Some(entry) = tt.get_entry(board_state.hash) {
        move_hint = Some(entry.best_move);
        if entry.depth >= depth && entry.is_valid(a, beta) {
            return Some(entry.score);
        }
    }

    let mut alpha = a;

    let mut moves = generate_moves(board_state, false);
    if moves.is_empty() {
        return Some(if is_check(board_state) {
            i32::MAX - ply as i32
        } else {
            0
        });
    }
    if let Some(mh) = move_hint {
        moves.insert(0, mh);
    }
    let mut best_move: Move = *moves.first()?;
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
            tt,
        )
        .map(|e| -e);
        board_state.undo_move(&undo);

        if evaluation? >= beta {
            // Move too good, need to prune
            tt.insert(TranspositionEntry {
                best_move: mv,
                hash: board_state.hash,
                depth: depth,
                score: beta,
                node_type: Bound::Lower,
            });
            return Some(beta);
        }
        if alpha < evaluation? {
            alpha = evaluation?;
            best_move = mv;
        }
    }
    if alpha == a {
        tt.insert(TranspositionEntry {
            best_move: best_move,
            hash: board_state.hash,
            depth: depth,
            score: alpha,
            node_type: Bound::Upper,
        });
    } else {
        tt.insert(TranspositionEntry {
            best_move: best_move,
            hash: board_state.hash,
            depth: depth,
            score: alpha,
            node_type: Bound::Exact,
        });
    }
    Some(alpha)
}

fn quiescence_search(
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
            && start.elapsed().as_millis() as u64 + 20 >= b
        {
            stop_flag.store(true, Ordering::Relaxed);
            return None;
        }
    }

    let stand_pat = eval(board_state);
    if depth == 0 {
        return Some(stand_pat);
    }
    let mut alpha = alpha;

    if stand_pat >= beta {
        return Some(beta);
    }

    if stand_pat > alpha {
        alpha = stand_pat;
    }

    let moves = generate_moves(board_state, true);
    if moves.is_empty() {
        return Some(if is_check(board_state) {
            i32::MAX - ply as i32
        } else {
            return Some(stand_pat);
        });
    }
    for mv in moves {
        let undo = board_state.make_move(mv);
        let evaluation = quiescence_search(
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
