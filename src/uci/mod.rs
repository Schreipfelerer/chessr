use crate::board::{BoardState, Color};
use crate::movegen::{generate_moves, perft, perft_devide};
use crate::search::{TranspositionTable, iterative_deepening};
use crate::bench::bench;
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;


pub fn uci_loop() {
    let stdin = std::io::stdin();
    let mut board_state = BoardState::start_pos();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let mut search_handle: Option<JoinHandle<()>> = None;
    let mut tt = TranspositionTable::new();

    for line in stdin.lock().lines().flatten() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("uci") => {
                println!("id name chessr");
                println!("id author thabo");
                println!("uciok");
            }
            Some("isready") => println!("readyok"),
            Some("ucinewgame") => board_state = BoardState::start_pos(),
            Some("position") => {
                if let Some(bs) = handle_position(&mut parts) {
                    board_state = bs;
                }
            }
            Some("go") => {
                // TODO perft
                if let Some(params) = parse_go(&mut parts) {
                    let time_budget =
                        compute_budget_ms(&params, board_state.state_info.active_color);

                    // A new `go` implies any previous search is done with.
                    // Stop it and join before starting the next one so two
                    // searches never run concurrently against stdout/state.
                    if let Some(handle) = search_handle.take() {
                        stop_flag.store(true, Ordering::Relaxed);
                        let _ = handle.join();
                    }
                    stop_flag.store(false, Ordering::Relaxed);

                    let mut thread_board = board_state.clone();
                    let thread_stop = Arc::clone(&stop_flag);
                    let depth = params.depth.unwrap_or(255);

                    search_handle = Some(std::thread::spawn(move || {
                        let (m, _nodes) = iterative_deepening(
                            &mut thread_board,
                            depth,
                            time_budget,
                            &thread_stop,
                            &mut tt,
                        );
                        println!("bestmove {m}");
                        let _ = io::stdout().flush();
                    }));
                } else {
                    println!("Malformed go params");
                }
            }
            Some("d") => println!("{}", board_state.board),
            Some("stop") => stop_flag.store(true, Ordering::Relaxed),
            Some("quit") => break,
            Some("bench") => {
                let depth = parts.next().and_then(|s| s.parse().ok()).unwrap_or(6);
                bench(depth);
            }
            Some("perft") => {
                if let Some(depth) = parts.next().and_then(|s| s.parse().ok()) {
                    perft(&mut board_state, depth);
                }
            }
            Some("perftd") => {
                if let Some(depth) = parts.next().and_then(|s| s.parse().ok()) {
                    perft_devide(&mut board_state, depth);
                }
            }
            _ => {}
        }
        let _ = io::stdout().flush();
    }
}

fn handle_position(parts: &mut std::str::SplitWhitespace) -> Option<BoardState> {
    let mut board_state = match parts.next()? {
        "startpos" => BoardState::start_pos(),
        "fen" => {
            // FEN is 6 space-separated fields — collect until "moves" or end
            let mut fen_parts = Vec::new();
            while let Some(tok) = parts.clone().next() {
                if tok == "moves" {
                    break;
                }
                fen_parts.push(parts.next()?);
            }
            BoardState::from_fen(&fen_parts.join(" ")).ok()?
        }
        _ => return None,
    };

    if parts.next() == Some("moves") {
        for mv_str in parts {
            let legal = generate_moves(&board_state, false);
            if let Some(found) = legal.iter().find(|mov| *mov.to_string() == *mv_str) {
                board_state.make_move(*found);
            } else {
                break;
            }
        }
    }

    Some(board_state)
}

fn parse_go(parts: &mut std::str::SplitWhitespace) -> Option<GoParams> {
    let mut params = GoParams {
        movetime: None,
        wtime: None,
        btime: None,
        winc: None,
        binc: None,
        depth: None,
        infinite: false,
    };

    while let Some(part) = parts.next() {
        match part {
            "wtime" => params.wtime = parts.next()?.parse().ok(),
            "btime" => params.btime = parts.next()?.parse().ok(),
            "winc" => params.winc = parts.next()?.parse().ok(),
            "binc" => params.binc = parts.next()?.parse().ok(),
            "depth" => params.depth = parts.next()?.parse().ok(),
            "movetime" => params.movetime = parts.next()?.parse().ok(),
            "infinite" => params.infinite = true,
            _ => (),
        }
    }
    Some(params)
}

// Time in ms
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
