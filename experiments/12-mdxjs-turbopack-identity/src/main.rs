use mdxjs_turbopack_identity_experiment::{inspect, stock};
use std::{env, error::Error, fs, process::ExitCode};

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let arguments: Vec<_> = env::args().skip(1).collect();
    let [mode, path] = arguments.as_slice() else {
        return Err("usage: mdxjs-turbopack-identity-experiment stock|check FILE".into());
    };
    let source = fs::read_to_string(path)?;
    match mode.as_str() {
        "stock" => match stock(&source) {
            Ok(tree) => println!("{}", serde_json::to_string_pretty(&tree)?),
            Err(error) => {
                eprintln!("{error}");
                return Ok(ExitCode::from(2));
            }
        },
        "check" => match inspect(&source) {
            Ok(result) => {
                println!("{}", serde_json::to_string_pretty(&result)?);
                if !result.diagnostics.is_empty() {
                    return Ok(ExitCode::from(2));
                }
            }
            Err(error) => {
                eprintln!("{error}");
                return Ok(ExitCode::from(2));
            }
        },
        _ => return Err("expected stock or check".into()),
    }
    Ok(ExitCode::SUCCESS)
}
