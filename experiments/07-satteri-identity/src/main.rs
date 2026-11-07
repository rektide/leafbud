use satteri_identity_experiment::{Mode, parse};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mode = match args.get(1).map(String::as_str) {
        Some("plain") => Mode::Plain,
        Some("attributes") => Mode::Attributes,
        Some("mdx") => Mode::Mdx,
        Some("author") => Mode::Author,
        _ => return Err("usage: satteri-identity-experiment plain|attributes|mdx|author FILE".into()),
    };
    let source = std::fs::read_to_string(args.get(2).ok_or("missing FILE")?)?;
    let parsed = parse(&source, mode);
    println!("{}", serde_json::to_string_pretty(&parsed)?);
    if !parsed.diagnostics.is_empty() {
        std::process::exit(2);
    }
    Ok(())
}
