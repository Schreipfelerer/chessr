mod board;
use board::BoardState;

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main(){
    let board = BoardState::from_fen(START_FEN).unwrap().get_board();
    println!("{board}");
}
