use std::io::{self, BufRead};

use chessr::{
    board::{Move, Undo, BoardState},
    movegen::{generate_moves, is_square_attacked_by, perft, perft_devide},
};

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut game_state = BoardState::from_fen(START_FEN).unwrap();
    let mut undo: Option<Undo> = None;
    println!("{}", game_state.board);

    loop {
        let mut input = String::new();
        if handle.read_line(&mut input).is_err() || input.trim().is_empty() {
            continue;
        }

        let mut parts = input.trim().split_whitespace();
        let command = match parts.next() {
            Some(cmd) => cmd,
            None => continue,
        };

        match command {
            "move" | "m" => {
                if let Some(move_str) = parts.next() {
                    let mut m = match Move::from_notation(&game_state, move_str) {
                        Some(mov) => mov,
                        None => {
                            println!("Invalid Move Notation");
                            continue;
                        }
                    };

                    let legal_moves = generate_moves(&game_state);
                    let mut matching = legal_moves
                        .iter()
                        .filter(|mov| mov.0 & 0xFFF == m.0 & 0xFFF);
                    m = match matching.next() {
                        // TODO differention between Promotions
                        None => {
                            println!("Move not legal {}", m);
                            continue;
                        }
                        Some(mov) => *mov,
                    };

                    let potential_undo = game_state.make_move(m);
                    if is_square_attacked_by(
                        game_state
                            .board
                            .find_king(game_state.state_info.active_color().flip()),
                        game_state.state_info.active_color(),
                        &game_state.board,
                    ) {
                        println!("That would leave you in check");
                        game_state.undo_move(potential_undo);
                        continue;
                    }

                    undo = Some(potential_undo);
                    println!("{}", game_state.board);
                }
            }
            "fen" => {
                let fen_str: String = parts.collect::<Vec<&str>>().join(" ");
                let result = BoardState::from_fen(&fen_str);
                match result {
                    Ok(gs) => game_state = gs,
                    Err(_) => println!("Error while parsing fen {:?}", result),
                }
                println!("{}", game_state.board);
            }
            "perft" => {
                let depth_str = match parts.next() {
                    Some(dep) => dep,
                    None => continue,
                };
                let depth = match depth_str.parse() {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                perft(&mut game_state, depth);
            }
            "perftd" => {
                let depth_str = match parts.next() {
                    Some(dep) => dep,
                    None => continue,
                };
                let depth = match depth_str.parse() {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                perft_devide(&mut game_state, depth);
            }
            "undo" => match undo {
                None => println!("Nothing to Undo"),
                Some(u) => {
                    game_state.undo_move(u);
                    undo = None;
                    println!("{}", game_state.board);
                }
            },
            "quit" | "exit" => break,
            _ => println!("Unknown command: {}", command),
        }
    }
}
