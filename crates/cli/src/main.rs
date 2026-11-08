//! springleaf — thin parsing/rendering for native reads/imports, bounded
//! offline check, and layered configuration.
//!
//! The command tree below mirrors the **accepted** authoring interface
//! ([`design/authoring-interface0.gpt6a.md`](../../design/authoring-interface0.gpt6a.md) §1)
//! so this skeleton cannot drift from the destination surface:
//!
//! - root document verbs — `create`, `check`, `list`, `status`, `diff`,
//!   `push`, `pull`, `publish`, `import`, `edit`, `tidy`, `export`;
//! - **push saves the editor; publish releases; pull reads the working
//!   editor** — same meanings for draft and published files;
//! - missing source keys are assigned on executed `push`/`publish`;
//!   manual local assignment is `tidy FILE --keys` (there is no
//!   `keys assign`), and `keys rename` is the explicit binding-preserving
//!   rename — key assignment is not implemented yet and requesting the
//!   selector fails honestly;
//! - `tidy FILE` runs all applicable implemented passes by default (today:
//!   Markdown body normalization); `--markdown` selects it alone, and
//!   `--keys`/`--from`/`--accept` are rejected until implemented;
//! - `list` inventories local managed files; `remote list`/`pubs list`
//!   discover remote objects.
//!
//! Other commands report `not implemented` on stderr and exit `1`.
//! Business policy lives in
//! `springleaf-workflow`; this binary stays a thin Clap/rendering layer.
//! The pre-reset stage-5 client (auth status, drafts list/export/author,
//! publish build/run, sync, config) is recoverable at revision
//! `80436b61`.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use springleaf_workflow::{LeafletState, RemoteOptions};

mod config;
mod import;
mod pull;
mod remote;
mod tidy;

#[derive(Parser)]
#[command(
    name = "springleaf",
    version,
    about = "Local-first MDX authoring for leaflet.pub (native reads, draft import, guarded pull, check and config)"
)]
struct Cli {
    /// Document workspace: explicit root > global workspace setting > cwd
    #[arg(long, global = true, value_name = "DIR")]
    root: Option<PathBuf>,
    /// Per-invocation authoring identity default (not native cookie auth)
    #[arg(long, global = true)]
    handle: Option<String>,
    #[arg(long, global = true)]
    pds: Option<String>,
    /// Default for unbound documents only; never retargets imported bindings
    #[arg(long, global = true)]
    publication: Option<String>,
    /// Leaflet origin (LEAFLET_BASE_URL; default https://leaflet.pub)
    #[arg(long, global = true, value_name = "URL")]
    base_url: Option<String>,
    /// auth_token cookie value (LEAFLET_AUTH_TOKEN; else XDG springleaf/auth-token).
    /// Prefer env/file to avoid shell history/process-list exposure.
    #[arg(long, global = true, value_name = "TOKEN")]
    token: Option<String>,
    #[command(subcommand)]
    command: Command,
}

/// Draft/published lifecycle state filter (bulk operations, listings).
#[derive(Debug, Clone, Copy, ValueEnum)]
enum State {
    Draft,
    Published,
}

/// Which pair of endpoints `diff` compares; `editor` is the default.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
enum Edge {
    Editor,
    Names,
    Release,
}

/// Export representation selector.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExportFormat {
    Mdx,
    Markdown,
    RecordJson,
}

/// Direction selector for `tidy`'s naming reconciliation.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum NameFrom {
    Filename,
    Path,
}

/// `push`/`publish` deliberate-content-replacement side (accepted surface:
/// `--take local`).
#[derive(Debug, Clone, Copy, ValueEnum)]
enum TakeSide {
    Local,
}

/// `pull` deliberate-content-replacement side (accepted surface:
/// `--take remote`).
#[derive(Debug, Clone, Copy, ValueEnum)]
enum TakeFrom {
    Remote,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create one local MDX source; no remote creation
    Create {
        file: PathBuf,
        /// Document title (author frontmatter)
        #[arg(short, long)]
        title: String,
    },
    /// Offline syntax, component, reference and identity validation (import profile)
    Check {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Inventory local managed files in the selected workspace
    List {
        #[arg(long)]
        state: Option<State>,
    },
    /// Naming, editor and release summaries
    Status {
        files: Vec<PathBuf>,
        /// Report remote state as unknown; local changes relative to stored receipts only
        #[arg(long)]
        offline: bool,
    },
    /// Compare current endpoints (editor is the default)
    Diff {
        file: PathBuf,
        #[arg(long, default_value = "editor")]
        edge: Edge,
    },
    /// Plan/execute identity-preserving editor updates and first binding
    /// (save; publish releases)
    Push {
        files: Vec<PathBuf>,
        #[arg(long)]
        all: bool,
        #[arg(long, requires = "all")]
        state: Option<State>,
        /// Execute the displayed plan (preview by default)
        #[arg(long)]
        yes: bool,
        /// Deliberately replace differing current remote content (local wins)
        #[arg(long, value_name = "TAKE")]
        take: Option<TakeSide>,
        /// Explicitly recreate the named source key (repeatable)
        #[arg(long = "recreate", value_name = "KEY")]
        recreate: Vec<String>,
        /// Explicitly recreate the document body
        #[arg(long)]
        recreate_body: bool,
    },
    /// Guarded refresh of bound files from their current editors (local write)
    Pull {
        /// Existing bound local MDX files (never remote discovery; use import)
        #[arg(required_unless_present = "all", conflicts_with = "all")]
        files: Vec<PathBuf>,
        /// Refresh every managed file in the workspace
        #[arg(long)]
        all: bool,
        #[arg(long, requires = "all")]
        state: Option<State>,
        /// Deliberately adopt differing current remote content (remote wins)
        #[arg(long, value_name = "TAKE")]
        take: Option<TakeFrom>,
        /// Preview outcomes and classifications without writing anything
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Plan/execute editor synchronization and native release
    /// (saves as needed, then releases)
    Publish {
        files: Vec<PathBuf>,
        #[arg(long)]
        all: bool,
        #[arg(long, requires = "all")]
        state: Option<State>,
        #[arg(long)]
        yes: bool,
        /// Deliberately replace differing current remote content in the editor-save phase (local wins)
        #[arg(long, value_name = "TAKE")]
        take: Option<TakeSide>,
    },
    /// Adopt a remote document into one bound MDX file
    Import {
        /// Native editor URL, permission-token UUID, or document AT URI
        #[arg(conflicts_with = "all", required_unless_present = "all")]
        locator: Option<String>,
        #[arg(long, value_name = "FILE", conflicts_with = "all")]
        to: Option<PathBuf>,
        /// Discover and adopt remote documents; report per-item outcomes
        #[arg(long)]
        all: bool,
        #[arg(long, requires = "all")]
        state: Option<State>,
        #[arg(long)]
        json: bool,
    },
    /// Explicit local metadata edits (date alternatives are mutually exclusive)
    Edit {
        file: PathBuf,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, value_name = "TIME")]
        published_at: Option<String>,
        #[arg(long, conflicts_with = "published_at")]
        now: bool,
    },
    /// Run all applicable implemented local tidying/normalization passes
    /// (implemented today: Markdown body normalization; selectors narrow)
    Tidy {
        /// Managed MDX files to tidy
        #[arg(required_unless_present = "all", conflicts_with = "all")]
        files: Vec<PathBuf>,
        /// Tidy every managed file in the workspace
        #[arg(long)]
        all: bool,
        /// Markdown body normalization only (the default without selectors)
        #[arg(long)]
        markdown: bool,
        /// Assign missing source identities only (not implemented yet)
        #[arg(long)]
        keys: bool,
        /// Naming: the filename follows the path intent (not implemented yet)
        #[arg(long, value_name = "FROM", conflicts_with = "accept")]
        from: Option<NameFrom>,
        /// Naming: witness the present filename/path pair as intentional (not implemented yet)
        #[arg(long)]
        accept: bool,
        /// Preview outcomes and edits without writing anything
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Emit a representation with explicit loss reporting (stdout unless --to)
    Export {
        file: PathBuf,
        #[arg(long, default_value = "mdx")]
        format: ExportFormat,
        #[arg(long, value_name = "FILE")]
        to: Option<PathBuf>,
        #[arg(long)]
        overwrite: bool,
        /// Approve specifically reported representation loss
        #[arg(long)]
        force_downconvert: bool,
    },
    /// Occasional identity operations
    Keys {
        #[command(subcommand)]
        command: KeysCommand,
    },
    /// Remote Leaflet discovery and inspection (distinct from local `list`)
    Remote {
        #[command(subcommand)]
        command: RemoteCommand,
    },
    /// Publication-container discovery (publications, not documents)
    Pubs {
        #[command(subcommand)]
        command: PubsCommand,
    },
    /// Auth diagnostics
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Layered configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Subcommand)]
enum KeysCommand {
    /// Rename a source key and its references/binding together, retaining remote identity
    Rename { file: PathBuf, old: String, new: String },
}

#[derive(Debug, Subcommand)]
enum RemoteCommand {
    /// Discover remote Leaflets/posts
    List {
        #[arg(long)]
        state: Option<State>,
        /// Versioned JSON including raw per-Leaflet rows and discovery coverage
        #[arg(long)]
        json: bool,
    },
    /// Inspect one remote object and its native editing availability
    Show {
        /// Same-origin native editor URL or permission-token UUID (AT URI not yet supported)
        locator: String,
        /// Versioned JSON including raw get_leaflet_data result and unknown fields
        #[arg(long)]
        json: bool,
    },
    /// Retained additive plumbing; never the ordinary update path
    Append {
        locator: String,
        #[arg(long, value_name = "FILE")]
        from: PathBuf,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
enum PubsCommand {
    /// List publication containers
    List,
}

#[derive(Debug, Subcommand)]
enum AuthCommand {
    /// Report current identity/auth state
    Status,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Show the effective layered config and provenance
    Show {
        #[arg(long)]
        json: bool,
    },
    /// Write one key to the layered config
    Set {
        key: springleaf_workflow::config::Key,
        value: String,
        #[arg(long)]
        workspace_file: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let options = RemoteOptions {
        base_url: cli.base_url,
        token: cli.token,
    };
    let flags = springleaf_workflow::config::Config {
        handle: cli.handle,
        pds: cli.pds,
        publication: cli.publication,
        ..Default::default()
    };
    if let Command::Import { locator, to, all, state, json } = &cli.command {
        return match import::run(
            options,
            cli.root.as_deref(),
            &flags,
            springleaf_workflow::import::Request {
                locator: locator.clone(),
                to: to.clone(),
                all: *all,
                state: state.map(|s| match s {
                    State::Draft => LeafletState::Draft,
                    State::Published => LeafletState::Published,
                }),
            },
            *json,
        )
        .await
        {
            Ok(code) => code,
            Err(error) => {
                eprintln!("springleaf: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Pull {
        files,
        all,
        state,
        take,
        dry_run,
        json,
    } = &cli.command
    {
        return match pull::run(
            options,
            cli.root.as_deref(),
            &flags,
            springleaf_workflow::pull::Request {
                all: *all,
                files: files.clone(),
                state: state.map(|s| match s {
                    State::Draft => LeafletState::Draft,
                    State::Published => LeafletState::Published,
                }),
                take_remote: take.is_some(),
                dry_run: *dry_run,
            },
            *json,
        )
        .await
        {
            Ok(code) => code,
            Err(error) => {
                eprintln!("springleaf: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Tidy {
        files,
        all,
        markdown: _,
        keys,
        from,
        accept,
        dry_run,
        json,
    } = &cli.command
    {
        // Honest rejection of the accepted-but-unimplemented selectors: this
        // build only normalizes Markdown bodies and never mints keys or
        // settles naming silently.
        if *keys {
            eprintln!(
                "springleaf: tidy --keys (missing source identity assignment) is not implemented yet; \
                 this build only normalizes Markdown bodies"
            );
            return ExitCode::FAILURE;
        }
        if from.is_some() || *accept {
            eprintln!(
                "springleaf: tidy --from/--accept (naming reconciliation) is not implemented yet; \
                 this build only normalizes Markdown bodies"
            );
            return ExitCode::FAILURE;
        }
        return match tidy::run(
            cli.root.as_deref(),
            &flags,
            springleaf_workflow::tidy::Request {
                all: *all,
                files: files.clone(),
                dry_run: *dry_run,
            },
            *json,
        ) {
            Ok(code) => code,
            Err(error) => {
                eprintln!("springleaf: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Check { files } = &cli.command {
        let context = match springleaf_workflow::config::load(cli.root.as_deref(), &flags, None) {
            Ok(context) => context,
            Err(e) => {
                eprintln!("springleaf: {e}");
                return ExitCode::FAILURE;
            }
        };
        let mut failed = 0;
        for file in files {
            let path = if file.is_absolute() {
                file.clone()
            } else {
                context.workspace_root.join(file)
            };
            match springleaf_workflow::import::check(&path) {
                Ok((blocks, changed)) => println!("valid {} ({blocks} blocks; local changes: {changed})", path.display()),
                Err(error) => {
                    failed += 1;
                    eprintln!("{}: {error:#}", path.display());
                }
            }
        }
        return if failed == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }
    let result = match &cli.command {
        Command::Remote {
            command: RemoteCommand::List { state, json },
        } => {
            remote::list(
                options,
                state.map(|s| match s {
                    State::Draft => LeafletState::Draft,
                    State::Published => LeafletState::Published,
                }),
                *json,
            )
            .await
        }
        Command::Remote {
            command: RemoteCommand::Show { locator, json },
        } => remote::show(options, locator, *json).await,
        Command::Config {
            command: ConfigCommand::Show { json },
        } => config::show(cli.root.as_deref(), &flags, *json),
        Command::Config {
            command: ConfigCommand::Set { key, value, workspace_file },
        } => config::set(*key, value, *workspace_file, cli.root.as_deref()),
        _ => return not_implemented(&cli.command),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("springleaf: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Every scaffold arm lands here: an honest operational failure that
/// never implies the operation ran. Exit code 1 per the accepted
/// contract (0 report/plan/no-op, 1 operational failure, 2 usage error,
/// 3 needs-decision, 4 partial/diverged).
fn not_implemented(command: &Command) -> ExitCode {
    let name = command_name(command);
    eprintln!(
        "springleaf: '{name}' is not implemented — this is the post-reset scaffold; \
         see design/authoring-interface0.gpt6a.md and design/scaffold-reset0.glm53m.md"
    );
    ExitCode::FAILURE
}

/// Render the full dotted command path for diagnostics.
fn command_name(command: &Command) -> String {
    match command {
        Command::Create { .. } => "create".into(),
        Command::Check { .. } => "check".into(),
        Command::List { .. } => "list".into(),
        Command::Status { .. } => "status".into(),
        Command::Diff { .. } => "diff".into(),
        Command::Push { .. } => "push".into(),
        Command::Pull { .. } => "pull".into(),
        Command::Publish { .. } => "publish".into(),
        Command::Import { .. } => "import".into(),
        Command::Edit { .. } => "edit".into(),
        Command::Tidy { .. } => "tidy".into(),
        Command::Export { .. } => "export".into(),
        Command::Keys { command } => match command {
            KeysCommand::Rename { .. } => "keys rename".into(),
        },
        Command::Remote { command } => match command {
            RemoteCommand::List { .. } => "remote list".into(),
            RemoteCommand::Show { .. } => "remote show".into(),
            RemoteCommand::Append { .. } => "remote append".into(),
        },
        Command::Pubs { command } => match command {
            PubsCommand::List => "pubs list".into(),
        },
        Command::Auth { command } => match command {
            AuthCommand::Status => "auth status".into(),
        },
        Command::Config { command } => match command {
            ConfigCommand::Show { .. } => "config show".into(),
            ConfigCommand::Set { .. } => "config set".into(),
        },
    }
}
