use std::io::{self, BufRead};

use chessr::{
    board::{BoardState, Move, Undo},
    movegen::{generate_moves, perft, perft_devide},
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

        let mut parts = input.split_whitespace();
        let Some(command) = parts.next() else {
            continue;
        };

        match command {
            "move" | "m" => {
                if let Some(move_str) = parts.next() {
                    let Some(mut m) = Move::from_notation(&game_state, move_str) else {
                        println!("Invalid Move Notation");
                        continue;
                    };

                    let legal_moves = generate_moves(&game_state);
                    let mut matching = legal_moves
                        .iter()
                        .filter(|mov| mov.0 & 0xFFF == m.0 & 0xFFF);
                    m = match matching.next() {
                        // TODO differention between Promotions
                        None => {
                            println!("Move not legal {m}");
                            continue;
                        }
                        Some(mov) => *mov,
                    };

                    undo = Some(game_state.make_move(m));
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
                let Some(depth_str) = parts.next() else {
                    continue;
                };
                let Ok(depth) = depth_str.parse() else {
                    continue;
                };
                perft(&mut game_state, depth);
                break;
            }
            "perftd" => {
                let Some(depth_str) = parts.next() else {
                    continue;
                };
                let Ok(depth) = depth_str.parse() else {
                    continue;
                };
                perft_devide(&mut game_state, depth);
            }
            "undo" => match undo {
                None => println!("Nothing to Undo"),
                Some(u) => {
                    game_state.undo_move(&u);
                    undo = None;
                    println!("{}", game_state.board);
                }
            },
            "quit" | "exit" => break,
            _ => println!("Unknown command: {command}"),
        }
    }
}
