use std::{env, error::Error, fs, process::ExitCode};

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 || !matches!(args[1].as_str(), "stock" | "check") {
        return Err("usage: mdxjs-identity-experiment stock|check FILE.mdx".into());
    }
    let source = fs::read_to_string(&args[2])?;
    if args[1] == "check" {
        return match mdxjs_identity_experiment::parse(&source) {
            Ok(parsed) => {
                println!("{}", serde_json::to_string_pretty(&parsed)?);
                Ok(if parsed.diagnostics.is_empty() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(2)
                })
            }
            Err(error) => {
                eprintln!("{error}");
                Ok(ExitCode::from(2))
            }
        };
    }
    match mdxjs_identity_experiment::stock(&source) {
        Ok(tree) => {
            println!("{}", serde_json::to_string_pretty(&tree)?);
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => {
            eprintln!("{error}");
            Ok(ExitCode::from(2))
        }
    }
}
