
pub struct ApiClient {
    client: reqwest::Client,
    token: String,
}

use reqwest::blocking::Client;

mod helper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = helper::get_lichess_token();

    let client = Client::new();

    let res = client
        .get("https://lichess.org/api/account")
        .header("Authorization", format!("Bearer {}", token))
        .send()?
        .text()?;

    println!("Response: {}", res);

    Ok(())
}
