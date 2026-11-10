//! leafbud — thin parsing/rendering for native reads/imports, bounded
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
//! - `create FILE --title` writes one unbound local draft (offline, stable
//!   random document UUID, no clobbering) with tri-state publication intent;
//! - missing source keys are assigned on executed `push`/`publish`;
//!   manual local assignment is `tidy FILE --keys` (there is no
//!   `keys assign`), and `keys rename` is the explicit binding-preserving
//!   rename;
//! - `tidy FILE` runs all applicable implemented passes by default (key
//!   preparation, Markdown normalization and unambiguous naming);
//!   `--keys`, `--markdown` and `--from`/`--accept` select their union;
//! - `list` inventories local managed files; `remote list`/`pubs list`
//!   discover remote objects.
//!
//! Other commands report `not implemented` on stderr and exit `1`.
//! Business policy lives in
//! `leafbud-workflow`; this binary stays a thin Clap/rendering layer.
//! The pre-reset stage-5 client (auth status, drafts list/export/author,
//! publish build/run, sync, config) is recoverable at revision
//! `80436b61`.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use leafbud_workflow::{LeafletState, RemoteOptions};

mod author;
mod config;
mod create;
mod diff;
mod import;
mod list;
mod pull;
mod push;
mod recover;
mod remote;
mod status;
mod tidy;
mod watch;

#[derive(Parser)]
#[command(
    name = "leafbud",
    version,
    about = "Local-first MDX authoring for leaflet.pub (keyed editor saves, guarded pull, connected watch, diff and native reads)"
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
    /// auth_token cookie value (LEAFLET_AUTH_TOKEN; else XDG leafbud/auth-token).
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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RecoverTake {
    Remote,
    Local,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect/settle a prior save; explicitly accept remote or replan local only after terminal acknowledgment
    Recover {
        file: PathBuf,
        /// Execute settlement/resolution (read-only preview by default)
        #[arg(long)]
        yes: bool,
        /// New decision for terminal partial work; never overrides uncertainty
        #[arg(long)]
        take: Option<RecoverTake>,
        #[arg(long)]
        json: bool,
    },
    /// Create one local MDX source; no remote creation
    Create {
        file: PathBuf,
        /// Document title (author frontmatter)
        #[arg(short, long)]
        title: String,
        /// Explicit standalone publication intent (overrides --publication)
        #[arg(long, conflicts_with = "publication")]
        standalone: bool,
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
        #[arg(long)]
        json: bool,
    },
    /// Naming, editor and release summaries
    Status {
        files: Vec<PathBuf>,
        /// Report remote state as unknown; local changes relative to stored receipts only
        #[arg(long)]
        offline: bool,
        #[arg(long)]
        json: bool,
    },
    /// Compare readable current local/editor Markdown through a foreground tool
    #[command(
        after_help = "Example: leafbud diff essay.mdx --tool diff --tool-arg=-u --tool-exit diff\nOr configure: leafbud config set diff-program diff\n              leafbud config set diff-args '[\"-u\"]'\n              leafbud config set diff-exit diff\nTool receives ARG... LOCAL.md EDITOR.md; no shell. GUI tools must wait until closed.\nStdio/TTY is inherited; --json is intentionally unsupported. names/release are not implemented."
    )]
    Diff {
        file: PathBuf,
        #[arg(long, default_value = "editor")]
        edge: Edge,
        /// Executable name/path only (not a shell command); overrides diff-program
        #[arg(long, value_name = "PROGRAM")]
        tool: Option<String>,
        /// Literal argument, repeatable; replaces diff-args when supplied
        #[arg(long, value_name = "ARG", allow_hyphen_values = true)]
        tool_arg: Vec<String>,
        /// success: only exit 0; diff: 0 equal/1 different are successful reports
        #[arg(long, value_name = "success|diff")]
        tool_exit: Option<leafbud_workflow::diff::ExitMode>,
    },
    /// Plan/execute identity-preserving editor updates and first binding
    /// (save; publish releases)
    Push {
        #[arg(required_unless_present = "all", conflicts_with = "all")]
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
        #[arg(long)]
        json: bool,
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
    /// Follow durable remote edits into existing bound files (never auto-push)
    Watch {
        #[arg(required_unless_present = "all", conflicts_with = "all")]
        files: Vec<PathBuf>,
        #[arg(long)]
        all: bool,
        /// Versioned JSON Lines transitions and final summary
        #[arg(long)]
        json: bool,
        /// Slow HTTP reconciliation backstop, seconds
        #[arg(long, default_value = "300", value_parser = clap::value_parser!(u64).range(1..))]
        backstop: u64,
        /// Local selection/source inventory rescan, seconds
        #[arg(long, default_value = "5", value_parser = clap::value_parser!(u64).range(1..))]
        rescan: u64,
        /// Observe advisory passive presence; never infer permission or idleness
        #[arg(long)]
        presence: bool,
        /// Explicit public Supabase origin (else bounded same-origin discovery)
        #[arg(long)]
        project_url: Option<String>,
        /// Public anon JWT; alternatively LEAFBUD_REALTIME_ANON_KEY
        #[arg(long)]
        anon_key: Option<String>,
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
    #[command(group(clap::ArgGroup::new("edit_fields").required(true).multiple(true).args(["title", "description", "tag", "clear_tags", "published_at", "now"])))]
    Edit {
        file: PathBuf,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        /// Replace the entire ordered tag list (repeat for each tag)
        #[arg(long, value_name = "TAG", conflicts_with = "clear_tags")]
        tag: Vec<String>,
        /// Explicitly replace tags with [] (not additive/removal by name)
        #[arg(long)]
        clear_tags: bool,
        #[arg(long, value_name = "TIME")]
        published_at: Option<String>,
        #[arg(long, conflicts_with = "published_at")]
        now: bool,
    },
    /// Run all applicable implemented local tidying/normalization passes
    /// (keys, Markdown and unambiguous names; selectors narrow)
    Tidy {
        /// Managed MDX files to tidy
        #[arg(required_unless_present = "all", conflicts_with = "all")]
        files: Vec<PathBuf>,
        /// Tidy every managed file in the workspace
        #[arg(long)]
        all: bool,
        /// Select Markdown body normalization
        #[arg(long)]
        markdown: bool,
        /// Select missing source identity assignment
        #[arg(long)]
        keys: bool,
        /// Naming: reconcile from the actual filename or author path intent
        #[arg(long, value_name = "FROM", conflicts_with = "accept")]
        from: Option<NameFrom>,
        /// Naming: witness the present filename/path pair as intentional
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
        key: leafbud_workflow::config::Key,
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
    let mut flags = leafbud_workflow::config::Config {
        handle: cli.handle,
        pds: cli.pds,
        publication: cli.publication.clone(),
        ..Default::default()
    };
    if let Command::List { state, json } = &cli.command {
        return match list::run(
            cli.root.as_deref(),
            &flags,
            leafbud_workflow::list::Request {
                state: state.map(|s| match s {
                    State::Draft => LeafletState::Draft,
                    State::Published => LeafletState::Published,
                }),
            },
            *json,
        ) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("leafbud: {e}");
                ExitCode::FAILURE
            }
        };
    }
    let author_operation = match &cli.command {
        Command::Edit {
            file,
            title,
            description,
            tag,
            clear_tags,
            published_at,
            now,
        } => Some((
            file,
            author::Operation::Edit(leafbud_workflow::author::MetadataEdit {
                title: title.clone(),
                description: description.clone(),
                tags: if *clear_tags || !tag.is_empty() { Some(tag.clone()) } else { None },
                published_at: if *now {
                    Some(jiff::Timestamp::now().to_string())
                } else {
                    published_at.clone()
                },
            }),
        )),
        Command::Keys {
            command: KeysCommand::Rename { file, old, new },
        } => Some((file, author::Operation::Rename(old, new))),
        _ => None,
    };
    if let Some((file, operation)) = author_operation {
        return match author::run(cli.root.as_deref(), &flags, file, operation) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("leafbud: {e}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Recover { file, yes, take, json } = &cli.command {
        return match recover::run(
            options,
            cli.root.as_deref(),
            &flags,
            file,
            leafbud_workflow::native::RecoverOptions {
                yes: *yes,
                take: take.map(|t| match t {
                    RecoverTake::Remote => leafbud_workflow::native::ResolutionDirection::Remote,
                    RecoverTake::Local => leafbud_workflow::native::ResolutionDirection::Local,
                }),
            },
            *json,
        )
        .await
        {
            Ok(code) => code,
            Err(error) => {
                eprintln!("leafbud: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Push {
        files,
        all,
        state,
        yes,
        take,
        recreate,
        recreate_body,
        json,
    } = &cli.command
    {
        if *recreate_body {
            eprintln!("leafbud: whole-body recreation is not implemented; use a supported --recreate KEY");
            return ExitCode::FAILURE;
        }
        return match push::run(
            options,
            cli.root.as_deref(),
            &flags,
            leafbud_workflow::push::Request {
                files: files.clone(),
                all: *all,
                state: state.map(|s| match s {
                    State::Draft => LeafletState::Draft,
                    State::Published => LeafletState::Published,
                }),
                yes: *yes,
                take_local: take.is_some(),
                recreate: recreate.iter().cloned().collect(),
            },
            *json,
        )
        .await
        {
            Ok(code) => code,
            Err(error) => {
                eprintln!("leafbud: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Watch {
        files,
        all,
        json,
        backstop,
        rescan,
        presence,
        project_url,
        anon_key,
    } = &cli.command
    {
        return match watch::run(
            options,
            cli.root.as_deref(),
            &flags,
            leafbud_workflow::watch::Request {
                files: files.clone(),
                all: *all,
            },
            watch::Settings {
                json: *json,
                backstop: *backstop,
                rescan: *rescan,
                presence: *presence,
                project_url: project_url.clone(),
                anon_key: anon_key.clone(),
            },
        )
        .await
        {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("leafbud: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Diff {
        file,
        edge,
        tool,
        tool_arg,
        tool_exit,
    } = &cli.command
    {
        if *edge != Edge::Editor {
            eprintln!("leafbud: diff --edge names/release is not implemented; only current local/editor content is compared");
            return ExitCode::FAILURE;
        }
        flags.diff_program = tool.clone();
        flags.diff_args = (!tool_arg.is_empty()).then(|| tool_arg.clone());
        flags.diff_exit = *tool_exit;
        return match diff::run(options, cli.root.as_deref(), &flags, file).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("leafbud: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Status { files, offline, json } = &cli.command {
        return match status::run(
            options,
            cli.root.as_deref(),
            &flags,
            leafbud_workflow::status::Request { files: files.clone() },
            *offline,
            *json,
        )
        .await
        {
            Ok(code) => code,
            Err(error) => {
                eprintln!("leafbud: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Import { locator, to, all, state, json } = &cli.command {
        return match import::run(
            options,
            cli.root.as_deref(),
            &flags,
            leafbud_workflow::import::Request {
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
                eprintln!("leafbud: {error}");
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
            leafbud_workflow::pull::Request {
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
                eprintln!("leafbud: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Create { file, title, standalone } = &cli.command {
        let publication = match (&cli.publication, standalone) {
            (Some(uri), _) => leafbud_document::Publication::At(uri.clone()),
            (None, true) => leafbud_document::Publication::Standalone,
            (None, false) => leafbud_document::Publication::Inherit,
        };
        return match create::run(
            cli.root.as_deref(),
            &flags,
            leafbud_workflow::create::Request {
                file: file.clone(),
                title: title.clone(),
                publication,
            },
        ) {
            Ok(code) => code,
            Err(error) => {
                eprintln!("leafbud: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Tidy {
        files,
        all,
        markdown,
        keys,
        from,
        accept,
        dry_run,
        json,
    } = &cli.command
    {
        return match tidy::run(
            cli.root.as_deref(),
            &flags,
            leafbud_workflow::tidy::Request {
                all: *all,
                files: files.clone(),
                dry_run: *dry_run,
                keys: *keys,
                markdown: *markdown,
                from: from.map(|from| match from {
                    NameFrom::Filename => leafbud_workflow::names::Direction::Filename,
                    NameFrom::Path => leafbud_workflow::names::Direction::Path,
                }),
                accept: *accept,
            },
            *json,
        ) {
            Ok(code) => code,
            Err(error) => {
                eprintln!("leafbud: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Command::Check { files } = &cli.command {
        let context = match leafbud_workflow::config::load(cli.root.as_deref(), &flags, None) {
            Ok(context) => context,
            Err(e) => {
                eprintln!("leafbud: {e}");
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
            match leafbud_workflow::import::check(&path) {
                Ok(outcome) => {
                    let preparation = if outcome.needs_preparation > 0 {
                        format!("; {} objects need key preparation (tidy --keys)", outcome.needs_preparation)
                    } else {
                        String::new()
                    };
                    let changes: String = match outcome.local_change {
                        Some(true) => "local changes: true".into(),
                        Some(false) => "local changes: false".into(),
                        None => "no receipts yet".into(),
                    };
                    println!("valid {} ({} blocks; {changes}{preparation})", path.display(), outcome.blocks);
                }
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
            eprintln!("leafbud: {error}");
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
        "leafbud: '{name}' is not implemented — this is the post-reset scaffold; \
         see design/authoring-interface0.gpt6a.md and design/scaffold-reset0.glm53m.md"
    );
    ExitCode::FAILURE
}

/// Render the full dotted command path for diagnostics.
fn command_name(command: &Command) -> String {
    match command {
        Command::Recover { .. } => "recover".into(),
        Command::Create { .. } => "create".into(),
        Command::Check { .. } => "check".into(),
        Command::List { .. } => "list".into(),
        Command::Status { .. } => "status".into(),
        Command::Diff { .. } => "diff".into(),
        Command::Push { .. } => "push".into(),
        Command::Pull { .. } => "pull".into(),
        Command::Watch { .. } => "watch".into(),
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
