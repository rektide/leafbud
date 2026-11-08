use leafspring_workflow::{
    RemoteOptions,
    config::{self, Config},
    import::Request,
};
use std::{path::Path, process::ExitCode};

pub async fn run(options: RemoteOptions, root: Option<&Path>, flags: &Config, request: Request, json: bool) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let report = options.connect().await?.import(&context, request).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for item in &report.items {
            if let Some(path) = &item.path {
                println!("{} {}", item.outcome, path.display());
            }
            if let Some(error) = &item.error {
                eprintln!("{}: {error}", item.token);
            }
        }
        println!(
            "{} discovered; {} selected; {} imported; {} existing preserved; {} filtered; {} failed",
            report.discovered, report.selected, report.imported, report.unchanged, report.skipped, report.failed
        );
        println!("Coverage: {}", report.coverage);
    }
    Ok(if report.failed == 0 {
        ExitCode::SUCCESS
    } else if report.imported + report.unchanged > 0 {
        ExitCode::from(4)
    } else {
        ExitCode::FAILURE
    })
}
