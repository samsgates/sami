use serde_json::{json, Value};
use std::io::{self, Read};
use std::process::ExitCode;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let operation = std::env::args()
        .nth(1)
        .ok_or("usage: cargo run -p sami-research --example research -- OPERATION < input.json")?;
    let mut data = String::new();
    io::stdin()
        .take(2 * 1024 * 1024 + 1)
        .read_to_string(&mut data)?;
    if data.len() > 2 * 1024 * 1024 {
        return Err("input exceeds 2 MiB".into());
    }
    let input: Value = serde_json::from_str(&data)?;
    let result = sami_research::dispatch(&operation, &input)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", json!({"error":error.to_string()}));
            ExitCode::FAILURE
        }
    }
}
