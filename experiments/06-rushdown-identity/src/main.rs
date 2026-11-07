use rushdown_identity_experiment::{Mode, parse};
use std::{env, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let mode = match args.next().as_deref() {
        Some("attributes") => Mode::Attributes,
        Some("plain") => Mode::Plain,
        _ => return Err("usage: rushdown-identity-experiment <attributes|plain> FILE".into()),
    };
    let path = args.next().ok_or("missing input file")?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let source = fs::read_to_string(path)?;
    let parsed = parse(&source, mode);
    println!("{}", serde_json::to_string_pretty(&parsed)?);
    if !parsed.diagnostics.is_empty() {
        std::process::exit(2);
    }
    Ok(())
}
