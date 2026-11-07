use std::{env, error::Error, fs, process::ExitCode};

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let mode = args.next().ok_or("usage: experiment check|stock|staged FILE")?;
    let path = args.next().ok_or("missing FILE")?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let source = fs::read_to_string(path)?;
    match mode.as_str() {
        "check" => {
            let parsed = satteri_mdxjs_identity_experiment::check(&source);
            println!("{}", serde_json::to_string_pretty(&parsed)?);
            Ok(if parsed.diagnostics.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(2)
            })
        }
        "stock" => match satteri_mdxjs_identity_experiment::stock(&source) {
            Ok(js) => {
                println!("{js}");
                Ok(ExitCode::SUCCESS)
            }
            Err(error) => {
                eprintln!("{error}");
                Ok(ExitCode::from(2))
            }
        },
        "staged" => match satteri_mdxjs_identity_experiment::staged(&source) {
            Ok(js) => {
                println!("{js}");
                Ok(ExitCode::SUCCESS)
            }
            Err(errors) => {
                println!("{}", serde_json::to_string_pretty(&errors)?);
                Ok(ExitCode::from(2))
            }
        },
        _ => Err("unknown mode; use check|stock|staged".into()),
    }
}
