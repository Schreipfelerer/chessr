use crate::{board::BoardState, movegen::generate::generate_moves};
use std::time::Instant;

pub fn perft(board_state: &mut BoardState, max_depth: u8) {
    for depth in 1..=max_depth {
        let start = Instant::now();
        let moves = number_of_moves(board_state, depth);
        let duration = start.elapsed();

        println!(
            "perft with depth {}: {:?}, moves: {}",
            depth, duration, moves
        );
    }
}

pub fn perft_devide(board_state: &mut BoardState, depth: u8) {
    let mut total = 0;
    for m in generate_moves(board_state) {
        let undo = board_state.make_move(m);
        let moves = number_of_moves(board_state, depth - 1);
        println!("  {}: {} moves", m, moves);
        total += moves;
        board_state.undo_move(undo);
    }
    println!("Total moves: {}", total)
}

pub fn number_of_moves(board_state: &mut BoardState, depth: u8) -> u32 {
    if depth == 0 {
        return 1;
    }
    let mut move_nunmber = 0;
    let moves = generate_moves(board_state);
    if depth == 1 {
        return moves.len() as u32;
    }
    for m in moves {
        let undo = board_state.make_move(m);
        move_nunmber += number_of_moves(board_state, depth - 1);
        board_state.undo_move(undo);
    }
    move_nunmber
}
