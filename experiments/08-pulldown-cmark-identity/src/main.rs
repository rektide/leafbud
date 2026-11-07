use std::{env, error::Error, fs, process::ExitCode};

use pulldown_identity_experiment::{Mode, parse};

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err("usage: pulldown-identity-experiment <plain|native|attributes|strict> FILE".into());
    }
    let mode = match args[1].as_str() {
        "plain" => Mode::Plain,
        "native" => Mode::Native,
        "attributes" => Mode::Attributes,
        "strict" => Mode::Strict,
        _ => return Err("unknown mode".into()),
    };
    let source = fs::read_to_string(&args[2])?;
    let result = parse(&source, mode);
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(if result.diagnostics.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    })
}
