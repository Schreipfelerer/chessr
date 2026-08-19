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

const MATE_SCORE: i32 = 1_000_001_000;
const MATE_THRESHOLD: i32 = 1_000_000_000;

pub fn iterative_deepening(
    board_state: &mut BoardState,
    max_depth: u8,
    budget_ms: Option<u64>,
    stop_flag: &Arc<AtomicBool>,
    tt: &mut TranspositionTable,
) -> (Move, u64) {
    let start = Instant::now();
    let moves = generate_moves(board_state, false);
    let mut best_move = moves[0]; // fallback, always legal
    let mut ctx = SearchCtx {
        nodes: 0,
        stop_flag,
        start,
        budget_ms,
        tt,
    };

    for depth in 1..=max_depth {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        let Some((mv, score)) = search_root(board_state, depth, &mut ctx) else {
            break; // aborted mid-depth, keep previous best_move
        };

        best_move = mv;
        println!(
            "info depth {depth} score cp {score} time {} nodes {} pv {mv}",
            start.elapsed().as_millis(),
            ctx.nodes,
        );
    }

    (best_move, ctx.nodes)
}

fn search_root(
    board_state: &mut BoardState,
    depth: u8,
    ctx: &mut SearchCtx,
) -> Option<(Move, i32)> {
    let moves = generate_moves(board_state, false);
    let mut best_move = moves[0];
    let mut best_score = i32::MIN + 1;

    for mv in moves {
        let undo = board_state.make_move(mv);
        let score = search(board_state, depth - 1, i32::MIN + 1, -best_score, 1, ctx).map(|e| -e);
        board_state.undo_move(&undo);
        if score? > best_score {
            best_score = score?;
            best_move = mv;
        }
    }
    Some((best_move, best_score))
}
struct SearchCtx<'a> {
    nodes: u64,
    stop_flag: &'a AtomicBool,
    start: Instant,
    budget_ms: Option<u64>,
    tt: &'a mut TranspositionTable,
}
impl SearchCtx<'_> {
    /// Returns true if the search should abort (timeout or external stop).
    fn should_stop(&mut self) -> bool {
        self.nodes += 1;
        if !self.nodes.is_multiple_of(2048) {
            return false;
        }
        if self.stop_flag.load(Ordering::Relaxed) {
            return true;
        }
        if let Some(b) = self.budget_ms
            && self.start.elapsed().as_millis() as u64 + 20 >= b
        {
            self.stop_flag.store(true, Ordering::Relaxed);
            return true;
        }
        false
    }
}

fn search(
    board_state: &mut BoardState,
    depth: u8,
    alpha: i32,
    beta: i32,
    ply: u8,
    ctx: &mut SearchCtx,
) -> Option<i32> {
    if board_state.is_repetition() {
        return Some(0);
    }

    if depth == 0 {
        return quiescence_search(board_state, 6, alpha, beta, ply, ctx);
    }

    if ctx.should_stop() {
        return None;
    }

    let mut move_hint: Option<Move> = None;
    if let Some(entry) = ctx.tt.get_entry(board_state.hash) {
        move_hint = entry.best_move;
        if entry.depth >= depth && entry.is_valid(alpha, beta) {
            return Some(entry.get_score(ply));
        }
    }

    let mut alpha = alpha;

    let mut moves = generate_moves(board_state, false);
    if moves.is_empty() {
        return Some(if is_check(board_state) {
            -MATE_SCORE + ply as i32
        } else {
            0
        });
    }
    if let Some(mh) = move_hint {
        moves.insert(0, mh);
    }
    let mut best_move: Option<Move> = None;
    for mv in moves {
        let undo = board_state.make_move(mv);
        let evaluation = search(board_state, depth - 1, -beta, -alpha, ply + 1, ctx).map(|e| -e);
        board_state.undo_move(&undo);

        if evaluation? >= beta {
            // Move too good, need to prune
            ctx.tt.insert(TranspositionEntry::new(
                Some(mv),
                depth,
                beta,
                board_state.hash,
                Bound::Lower,
                ply,
            ));
            return Some(beta);
        }
        if alpha < evaluation? {
            alpha = evaluation?;
            best_move = Some(mv);
        }
    }
    if best_move.is_none() {
        ctx.tt.insert(TranspositionEntry::new(
            best_move,
            depth,
            alpha,
            board_state.hash,
            Bound::Upper,
            ply,
        ));
    } else {
        ctx.tt.insert(TranspositionEntry::new(
            best_move,
            depth,
            alpha,
            board_state.hash,
            Bound::Exact,
            ply,
        ));
    }
    Some(alpha)
}

fn quiescence_search(
    board_state: &mut BoardState,
    depth: u8,
    alpha: i32,
    beta: i32,
    ply: u8,
    ctx: &mut SearchCtx,
) -> Option<i32> {
    if ctx.should_stop() {
        return None;
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
            -MATE_SCORE + ply as i32
        } else {
            return Some(stand_pat);
        });
    }
    for mv in moves {
        let undo = board_state.make_move(mv);
        let evaluation =
            quiescence_search(board_state, depth - 1, -beta, -alpha, ply + 1, ctx).map(|e| -e);
        board_state.undo_move(&undo);

        if evaluation? >= beta {
            // Move too good, need to prune
            return Some(beta);
        }
        alpha = max(alpha, evaluation?);
    }
    Some(alpha)
}
