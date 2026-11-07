use std::{env, error::Error, fs, process::ExitCode};

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 || args[1] != "stock" {
        return Err("usage: mdxjs-identity-experiment stock FILE.mdx".into());
    }
    let source = fs::read_to_string(&args[2])?;
    match mdxjs::mdast_util_from_mdx(&source, &mdxjs::Options::default()) {
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
