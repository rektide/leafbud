use leafbud_workflow::{
    RemoteOptions,
    config::{self, Config},
    push::Request,
};
use std::{path::Path, process::ExitCode};

pub async fn run(options: RemoteOptions, root: Option<&Path>, flags: &Config, request: Request, json: bool) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let report = options.connect().await?.push(&context, request).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for item in &report.items {
            println!("{} {} {:?}", item.result.outcome, item.path.display(), item.result.effects);
            if let Some(message) = &item.result.message {
                eprintln!("{message}");
            }
        }
        if report.preview {
            println!("preview: no source changes, mint or native writes; push saves without publishing");
        }
    }
    let mut success = 0;
    let mut failed = 0;
    let mut decisions = 0;
    let mut partial = 0;
    for item in &report.items {
        match item.result.outcome {
            "saved" | "unchanged" | "would-save" | "would-bind-and-save" | "would-settle" => success += 1,
            "needs-decision" => decisions += 1,
            "pending" | "partial" | "source-changed" => partial += 1,
            _ => failed += 1,
        }
    }
    Ok(if partial > 0 || (success > 0 && failed + decisions > 0) || (failed > 0 && decisions > 0) {
        ExitCode::from(4)
    } else if decisions > 0 {
        ExitCode::from(3)
    } else if failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
