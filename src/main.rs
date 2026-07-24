mod helper;

fn main() {
    let token = helper::get_lichess_token();
    println!("Lichess Token: {}", token);
}
