use std::io::BufRead;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::board::{BoardState, Color};

fn main() {
    let stdin = std::io::stdin();
    let mut board_state = BoardState::from_fen(START_FEN).unwrap();
    let stop_flag = Arc::new(AtomicBool::new(false));

    for line in stdin.lock().lines().flatten() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("uci") => {
                println!("id name chessr");
                println!("id author <you>");
                println!("uciok");
            }
            Some("isready") => println!("readyok"),
            Some("ucinewgame") => board_state = BoardState::from_fen(START_FEN).unwrap(),
            Some("position") => {
                if let Some(bs) = handle_position(&mut parts) {
                    board_state = bs;
                }
            }
            Some("go") => {
                let params = parse_go(&mut parts);
                // spawn search thread as above
            }
            Some("stop") => stop_flag.store(true, Ordering::Relaxed),
            Some("quit") => break,
            _ => {}
        }
    }
}

fn handle_position(parts: &mut std::str::SplitWhitespace) -> Option<BoardState> {
    let mut board_state = match parts.next()? {
        "startpos" => BoardState::from_fen(START_FEN).ok()?,
        "fen" => {
            // FEN is 6 space-separated fields — collect until "moves" or end
            let mut fen_parts = Vec::new();
            while let Some(tok) = parts.clone().next() {
                if tok == "moves" { break; }
                fen_parts.push(parts.next()?);
            }
            BoardState::from_fen(&fen_parts.join(" ")).ok()?
        }
        _ => return None,
    };

    if parts.next() == Some("moves") {
        for mv_str in parts {
            let Some(mut m) = Move::from_notation(&board_state, mv_str) else { break };
            let legal = generate_moves(&board_state);
            let Some(found) = legal.iter().find(|mov| mov.0 & 0xFFF == m.0 & 0xFFF) else { break };
            m = *found;
            board_state.make_move(m);
        }
    }

    Some(board_state)
}

struct GoParams {
    movetime: Option<u64>,
    wtime: Option<u64>,
    btime: Option<u64>,
    winc: Option<u64>,
    binc: Option<u64>,
    depth: Option<u8>,
    infinite: bool,
}

fn compute_budget_ms(params: &GoParams, side: Color) -> Option<u64> {
    if let Some(mt) = params.movetime {
        return Some(mt);
    }
    let (time, inc) = match side {
        Color::White => (params.wtime, params.winc.unwrap_or(0)),
        Color::Black => (params.btime, params.binc.unwrap_or(0)),
    };
    time.map(|t| (t / 20 + inc / 2).min(t.saturating_sub(50)))
}