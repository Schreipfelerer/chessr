use crate::board::BoardState;
use crate::search::{TranspositionTable, iterative_deepening};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

/// Fixed suite for repeatable search benchmarking
const BENCH_FENS: &[&str] = &[
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
];

/// Runs a fixed-depth search over the fixed suite and reports aggregate
/// nodes/NPS.
pub fn bench(depth: u8) {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let mut total_nodes = 0u64;
    let start = Instant::now();
    let mut tt = TranspositionTable::new();

    for fen in BENCH_FENS {
        let mut board_state = BoardState::from_fen(fen).unwrap();
        let (_mv, nodes) = iterative_deepening(&mut board_state, depth, None, &stop_flag, &mut tt);
        total_nodes += nodes;
    }

    let elapsed = start.elapsed();
    let nps = (total_nodes as f64 / elapsed.as_secs_f64().max(1e-9)) as u64;

    println!(
        "Bench: {} nodes, {:.2}s, {} nps",
        format_thousands(total_nodes),
        elapsed.as_secs_f64(),
        format_thousands(nps),
    );
}

fn format_thousands(n: u64) -> String { 
    let s = n.to_string();
    s.as_bytes()
        .rchunks(3)
        .rev()
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join(".")
}
