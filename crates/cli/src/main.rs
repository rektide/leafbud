//! springleaf — a leaflet.pub client for agents and the CLI (stage 2).
//!
//! Commands: `auth status`, `drafts list [--json] [--fetch]`,
//! `drafts show <token-id> [--json]`, `drafts export <token-id>`
//! (read-only pull → canonical record in drafts/), `pubs list`,
//! `publish build/run`, `sync`.
//!
//! Auth model: the `auth_token` cookie value resolved from `--token` >
//! `$LEAFLET_AUTH_TOKEN` > `$XDG_CONFIG_HOME/springleaf/auth-token`
//! (else `~/.config/springleaf/auth-token`). The token is
//! never echoed, logged, or embedded in error messages — including at
//! debug/trace levels.
//!
//! Exit codes: 0 ok, 1 operational error (including anonymous `auth
//! status`), 2 usage error (clap).

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use springleaf_drafts::{DEFAULT_BASE_URL, Error as DraftsError, LeafletClient, LeafletData, LeafletList, LeafletState, classify};
use std::fmt::Write as _;
use uuid::Uuid;

mod author;
mod export;
mod publish;
mod sync;
mod workspace;

#[derive(Parser)]
#[command(
    name = "springleaf",
    version,
    about = "A leaflet.pub client for agents and the CLI",
    after_help = "The auth token is the auth_token cookie value: --token, $LEAFLET_AUTH_TOKEN, or ~/.config/springleaf/auth-token. It is never printed or logged."
)]
struct Cli {
    /// Leaflet base URL (default https://leaflet.pub; env LEAFLET_BASE_URL)
    #[arg(long, global = true, value_name = "URL")]
    base_url: Option<String>,

    /// Auth token (auth_token cookie value; env LEAFLET_AUTH_TOKEN; else
    /// ~/.config/springleaf/auth-token). Never printed or logged.
    #[arg(long, global = true, value_name = "TOKEN")]
    token: Option<String>,

    /// Verbose diagnostics on stderr (never includes the auth token)
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check authentication against /home
    Auth(AuthArgs),
    /// Leaflet (draft) operations
    Drafts(DraftsArgs),
    /// Publication operations (public PDS reads, no auth)
    Pubs(PubsArgs),
    /// Build and publish canonical records (build is offline-safe; run writes)
    Publish(PublishArgs),
    /// Reconcile posted/ against the PDS (report-only by default)
    Sync(SyncArgs),
}

#[derive(Args)]
struct AuthArgs {
    #[command(subcommand)]
    cmd: AuthCommand,
}

#[derive(Subcommand)]
enum AuthCommand {
    /// GET /home and report identity (did/handle) or anonymous
    Status,
}

#[derive(Args)]
struct DraftsArgs {
    #[command(subcommand)]
    cmd: DraftsCommand,
}

#[derive(Subcommand)]
enum DraftsCommand {
    /// List leaflets from the /home RSC payload (schema v1)
    List {
        /// Emit the v1 JSON schema verbatim (experiments/01 §5)
        #[arg(long)]
        json: bool,
        /// Enrich each leaflet via get_leaflet_data (N+1 requests)
        #[arg(long)]
        fetch: bool,
    },
    /// Show one leaflet via get_leaflet_data
    Show {
        /// Permission token id (uuid)
        token_id: Uuid,
        /// Emit the raw get_leaflet_data result JSON
        #[arg(long)]
        json: bool,
    },
    /// Pull one editor draft (read-only) and write its canonical record
    /// to drafts/<slug>.json — the extraction half of
    /// design/draft-extraction.glm53m.md. Never writes to leaflet.pub.
    Export {
        /// Permission token id (uuid)
        token_id: Uuid,
        /// Workspace root (default: `workspace` from the XDG config
        /// springleaf.toml, else cwd)
        #[arg(long, value_name = "DIR")]
        root: Option<std::path::PathBuf>,
        /// Slug override (drives the filename and the record path);
        /// default: slugify of the derived title
        #[arg(long, value_name = "SLUG")]
        slug: Option<String>,
        /// Replace an existing drafts/<slug>.json
        #[arg(long)]
        overwrite: bool,
        /// Emit the v1 JSON schema (slug/path/blocks/warns)
        #[arg(long)]
        json: bool,
    },
    /// Author a canonical record into an EXISTING editor draft via a
    /// Replicache push — the reverse half of
    /// design/draft-extraction.glm53m.md. Dry-run by default (prints
    /// the mutation batch, exit 2); --yes executes the push.
    Author {
        /// Permission token id (uuid) of the target draft
        token_id: Uuid,
        /// The record to author: a drafts/<slug>.json / posted/<slug>.json
        /// slug or an existing record file path
        #[arg(long, value_name = "SLUG-OR-FILE")]
        from: String,
        /// Workspace root (default: `workspace` from the XDG config
        /// springleaf.toml, else cwd)
        #[arg(long, value_name = "DIR")]
        root: Option<std::path::PathBuf>,
        /// Execute the push (required; without it the command only
        /// prints the exact mutation batch and exits 2)
        #[arg(long)]
        yes: bool,
        /// Emit the v1 JSON schema (target/batch/warns/pushed)
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args)]
struct PubsArgs {
    #[command(subcommand)]
    cmd: PubsCommand,
}

#[derive(Subcommand)]
enum PubsCommand {
    /// List publications from listRecords (public read, no auth)
    List {
        /// Emit the v1 JSON schema
        #[arg(long)]
        json: bool,
        /// Handle (or full DID) whose publications to list; default:
        /// `handle` from springleaf.toml in the XDG config dir, if any
        #[arg(long, value_name = "HANDLE")]
        handle: Option<String>,
    },
}

#[derive(Args)]
struct PublishArgs {
    #[command(subcommand)]
    cmd: PublishCommand,
}

#[derive(Subcommand)]
enum PublishCommand {
    /// Build the canonical record into drafts/<slug>.json (always safe)
    Build {
        /// Markdown source (frontmatter + body)
        file: std::path::PathBuf,
        /// Target publication at-uri; default: `publication` from the
        /// workspace springleaf.toml. Passing the flag also enables one
        /// public getRecord to derive the publication-domain URL.
        #[arg(long, value_name = "AT_URI")]
        publication: Option<String>,
        /// Workspace root (default: `workspace` from the XDG config
        /// springleaf.toml, else cwd)
        #[arg(long, value_name = "DIR")]
        root: Option<std::path::PathBuf>,
        /// Replace an existing drafts/<slug>.json
        #[arg(long)]
        overwrite: bool,
        /// Emit the v1 JSON schema (record + rkey + url + draft path)
        #[arg(long)]
        json: bool,
    },
    /// putRecord the draft (upsert), move it to posted/, write the
    /// sidecar — refuses without --yes (exit 2)
    Run {
        /// Slug (drafts/<slug>.json or posted/<slug>.json) or a record file path
        slug_or_file: String,
        /// Execute the putRecord (required; without it the command only
        /// prints the exact record + target and exits 2)
        #[arg(long)]
        yes: bool,
        #[arg(long, value_name = "DIR")]
        root: Option<std::path::PathBuf>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args)]
struct SyncArgs {
    /// Adopt remote records into posted/ (a local write, never remote)
    #[arg(long)]
    pull: bool,
    /// Re-put eligible local records (refuses without --yes)
    #[arg(long)]
    force: bool,
    /// Execute --force re-puts (requires app-password env)
    #[arg(long, requires = "force")]
    yes: bool,
    /// Emit the v1 JSON report schema
    #[arg(long)]
    json: bool,
    /// Handle (or full DID) to sync against; default: workspace toml,
    /// XDG config toml, else $ATPROTO_HANDLE
    #[arg(long, value_name = "HANDLE")]
    handle: Option<String>,
    #[arg(long, value_name = "DIR")]
    root: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    let code = match run(cli).await {
        Ok(code) => code,
        Err(err) => {
            // anyhow chain on stderr; the token never appears in here.
            eprintln!("error: {err:#}");
            1
        }
    };
    std::process::exit(code);
}

async fn run(cli: Cli) -> Result<i32> {
    // Public/PDS-only commands dispatch without the drafts client (and
    // its token resolution): `pubs list`, `publish`, and `sync` need no
    // leaflet cookie.
    match cli.command {
        Command::Pubs(PubsArgs {
            cmd: PubsCommand::List { json, handle },
        }) => pubs_list(json, handle).await,
        Command::Publish(PublishArgs {
            cmd:
                PublishCommand::Build {
                    file,
                    publication,
                    root,
                    overwrite,
                    json,
                },
        }) => publish::publish_build_cmd(&file, publication, root, overwrite, json).await,
        Command::Publish(PublishArgs {
            cmd: PublishCommand::Run { slug_or_file, yes, root, json },
        }) => publish::publish_run_cmd(&slug_or_file, root, yes, json, None, None).await,
        Command::Sync(SyncArgs {
            pull,
            force,
            yes,
            json,
            handle,
            root,
        }) => {
            sync::sync_cmd(sync::SyncOptions {
                handle,
                root,
                pull,
                force,
                yes,
                json,
                target: None,
                connect: None,
            })
            .await
        }
        auth_or_drafts => {
            // Resolution order: flag > env > default / token file. The
            // token value is never echoed, logged, or embedded in error
            // text.
            let base_url = cli
                .base_url
                .clone()
                .or_else(|| std::env::var("LEAFLET_BASE_URL").ok())
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
            let flag_token = cli.token.clone().or_else(|| std::env::var("LEAFLET_AUTH_TOKEN").ok());
            let token = resolve_token(flag_token.as_deref())?;
            let client = LeafletClient::new(&base_url, token).with_context(|| format!("invalid base url {base_url:?}"))?;
            match auth_or_drafts {
                Command::Auth(AuthArgs { cmd: AuthCommand::Status }) => auth_status(&client).await,
                Command::Drafts(DraftsArgs {
                    cmd: DraftsCommand::List { json, fetch },
                }) => drafts_list(&client, json, fetch).await,
                Command::Drafts(DraftsArgs {
                    cmd: DraftsCommand::Show { token_id, json },
                }) => drafts_show(&client, &token_id, json).await,
                Command::Drafts(DraftsArgs {
                    cmd:
                        DraftsCommand::Export {
                            token_id,
                            root,
                            slug,
                            overwrite,
                            json,
                        },
                }) => export::drafts_export_cmd(&client, &token_id.to_string(), root, slug, overwrite, json).await,
                Command::Drafts(DraftsArgs {
                    cmd:
                        DraftsCommand::Author {
                            token_id,
                            from,
                            root,
                            yes,
                            json,
                        },
                }) => author::drafts_author_cmd(&client, &token_id.to_string(), &from, root, yes, json, None).await,
                _ => unreachable!("pubs/publish/sync dispatched above"),
            }
        }
    }
}

/// Token resolution: --token > $LEAFLET_AUTH_TOKEN >
/// $XDG_CONFIG_HOME/springleaf/auth-token. The value is trimmed and
/// returned opaque; it is never printed, logged, or embedded in errors.
fn resolve_token(flag_or_env: Option<&str>) -> Result<Option<String>> {
    if let Some(token) = flag_or_env.map(str::trim).filter(|t| !t.is_empty()) {
        return Ok(Some(token.to_string()));
    }
    let path = workspace::xdg_config_dir()
        .context("token file: locating the springleaf XDG config dir")?
        .join("auth-token");
    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            let token = contents.trim();
            if token.is_empty() {
                anyhow::bail!("token file {} is empty", path.display());
            }
            Ok(Some(token.to_string()))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(anyhow::Error::new(err).context(format!("reading token file {}", path.display()))),
    }
}

fn init_tracing(verbose: bool) {
    use tracing_subscriber::EnvFilter;
    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();
}

/// `auth status`: GET /home, report the identity from the flight payload.
/// Anonymous (no identity payload reachable) is a clean message + exit 1.
async fn auth_status(client: &LeafletClient) -> Result<i32> {
    let html = client.get_home().await.context("fetching /home")?;
    match springleaf_drafts::identity_from_home(&html)? {
        Some(identity) => {
            let did = identity.did.as_deref().unwrap_or("(did absent in payload)");
            match identity.handle.as_deref() {
                Some(handle) => println!("authenticated as {handle} ({did})"),
                None => println!("authenticated ({did}; no handle in payload)"),
            }
            Ok(0)
        }
        None => {
            println!("anonymous (no identity payload in /home; token missing or invalid)");
            Ok(1)
        }
    }
}

/// `drafts list`: default human table; --json emits schema v1 verbatim;
/// --fetch enriches per id via get_leaflet_data (source home-rsc+rpc).
async fn drafts_list(client: &LeafletClient, json: bool, fetch: bool) -> Result<i32> {
    let mut list = client
        .list_leaflets()
        .await
        .map_err(|err| match err {
            DraftsError::NoHomePayload => anyhow::anyhow!(
                "{err}\nhint: `springleaf auth status` checks the token; \
                 it is read from --token, $LEAFLET_AUTH_TOKEN, or \
                 ~/.config/springleaf/auth-token"
            ),
            other => anyhow::Error::new(other),
        })
        .context("listing leaflets from /home")?;
    if fetch {
        client.enrich_leaflets(&mut list).await.context("enriching leaflets via get_leaflet_data")?;
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&list)?);
    } else {
        print!("{}", human_table(&list));
    }
    Ok(0)
}

/// Human table per experiments/01 §5: STATE | CREATED | TITLE (38 chars,
/// ellipsized) | TOKEN-ID | PUBLISHED-URL, `*` marking contributor rows,
/// count/handle footer, mismatch warning line.
fn human_table(list: &LeafletList) -> String {
    let id_width = list.leaflets.iter().map(|l| l.token_id.len()).max().unwrap_or(36).max(36);
    let mut out = String::new();
    let title_width = 40;
    out.push_str(&format!(
        "{:<15} {:<12} {:<title_width$} {:<id_width$} PUBLISHED-URL\n",
        "STATE", "CREATED", "TITLE", "TOKEN-ID",
    ));
    for leaflet in &list.leaflets {
        let date = leaflet.created_at.as_deref().unwrap_or("");
        let date = &date[..date.len().min(10)];
        let title = leaflet.title.as_deref().unwrap_or("Untitled");
        let title = if title.chars().count() > 38 {
            format!("{}…", title.chars().take(35).collect::<String>())
        } else {
            title.to_string()
        };
        let flag = if leaflet.contributor { "*" } else { "" };
        out.push_str(&format!(
            "{:<15} {:<12} {:<title_width$} {:<id_width$} {}{}\n",
            leaflet.state.as_str(),
            date,
            title,
            leaflet.token_id,
            leaflet.published_url.as_deref().unwrap_or(""),
            flag,
        ));
    }
    let published = list.leaflets.iter().filter(|l| l.state == LeafletState::Published).count();
    let mut footer = format!(
        "\n{} leaflets ({} published, {} draft/unpublished)",
        list.count,
        published,
        list.count - published
    );
    if let Some(handle) = &list.identity.handle {
        footer.push_str(&format!(" as {handle}"));
    }
    out.push_str(&footer);
    out.push('\n');
    if !list.href_payload_mismatch.is_empty() {
        out.push_str(&format!("warning: href/payload id mismatch: {:?}\n", list.href_payload_mismatch));
    }
    out
}

/// `drafts show`: get_leaflet_data rendered; --json emits the raw result.
async fn drafts_show(client: &LeafletClient, token_id: &Uuid, json: bool) -> Result<i32> {
    let id = token_id.to_string();
    if json {
        let raw = client
            .get_leaflet_data_raw(&id)
            .await
            .with_context(|| format!("fetching leaflet data for {id}"))?;
        println!("{}", serde_json::to_string_pretty(&raw)?);
        return Ok(0);
    }
    let data = client.get_leaflet_data(&id).await.with_context(|| format!("fetching leaflet data for {id}"))?;
    print!("{}", show_text(&data));
    Ok(0)
}

/// The `drafts show` rendering (returned as a string so tests can assert on
/// it; the command prints it verbatim).
fn show_text(data: &LeafletData) -> String {
    let mut out = String::new();
    let classified = classify(&data.leaflets_in_publications);
    let _ = writeln!(out, "token id:      {}", data.id.as_deref().unwrap_or("?"));
    match &data.title {
        Some(title) if !title.is_empty() => {
            let _ = writeln!(out, "title:         {title}");
        }
        _ => {
            let _ = writeln!(out, "title:         (untitled)");
        }
    }
    if let Some(description) = &data.description {
        let _ = writeln!(out, "description:   {description}");
    }
    if let Some(root_entity) = &data.root_entity {
        let _ = writeln!(out, "root entity:   {root_entity}");
    }
    let _ = writeln!(out, "state:         {}", classified.state.as_str());
    if data.blocked_by_admin.as_ref().is_some_and(|v| !v.is_null()) {
        let _ = writeln!(out, "blocked:       {}", data.blocked_by_admin.as_ref().unwrap());
    }
    for right in &data.permission_token_rights {
        let _ = writeln!(
            out,
            "rights:        read={} write={}",
            right.read.unwrap_or(false),
            right.write.unwrap_or(false)
        );
        if let Some(entity_set) = &right.entity_set {
            let _ = writeln!(out, "entity set:    {entity_set}");
        }
    }
    let _ = writeln!(out);
    if data.leaflets_in_publications.is_empty() {
        let _ = writeln!(out, "publication linkage: none");
    } else {
        let _ = writeln!(out, "publication linkage ({}):", data.leaflets_in_publications.len());
        for (i, link) in data.leaflets_in_publications.iter().enumerate() {
            let publication = link.publications.as_ref().and_then(|p| p.record.as_ref());
            match publication {
                Some(record) => {
                    let name = record.name.as_deref().unwrap_or("(unnamed)");
                    let _ = writeln!(out, "  [{}] {name}", i + 1);
                    if let Some(uri) = &link.publication {
                        let _ = writeln!(out, "      publication: {uri}");
                    }
                    if let Some(url) = &record.url {
                        let _ = writeln!(out, "      publication url: {url}");
                    }
                }
                None => {
                    let _ = writeln!(out, "  [{}] {}", i + 1, link.publication.as_deref().unwrap_or("?"));
                }
            }
            if link.doc.is_some() {
                if let Some(doc) = link.doc.as_deref() {
                    let _ = writeln!(out, "      doc: {doc}");
                }
                let record = link.documents.as_ref().and_then(|d| d.data.as_ref());
                if let Some(title) = record.and_then(|d| d.title.as_deref()) {
                    let _ = writeln!(out, "      doc title: {title}");
                }
                if let Some(at) = record.and_then(|d| d.published_at.as_deref()) {
                    let _ = writeln!(out, "      published at: {at}");
                }
            } else {
                let _ = writeln!(out, "      added to publication, not published");
                if let Some(at) = &link.scheduled_publish_at {
                    let _ = writeln!(out, "      scheduled publish at: {at}");
                }
                if let Some(extra) = &link.scheduled_publish_data {
                    let _ = writeln!(out, "      scheduled publish data: {extra}");
                }
            }
            if let Some(title) = &link.title {
                let _ = writeln!(out, "      linkage title: {title}");
            }
        }
    }
    if data.reply_drafts.as_ref().is_some_and(|v| !v.is_null()) {
        let _ = writeln!(out, "reply drafts:  present (see --json)");
    }
    out
}

// ---------------------------------------------------------------------------
// pubs list — public PDS reads via springleaf-atproto (no leaflet cookie)
// ---------------------------------------------------------------------------

/// One publication row in the `pubs list` output.
#[derive(Debug, Clone, Serialize)]
struct PublicationSummary {
    collection: String,
    rkey: String,
    uri: String,
    cid: Option<String>,
    name: Option<String>,
    url: Option<String>,
}

/// The `pubs list --json` schema (version 1).
#[derive(Debug, Serialize)]
struct PubsList {
    version: u8,
    identity: PubsIdentity,
    count: usize,
    publications: Vec<PublicationSummary>,
}

#[derive(Debug, Serialize)]
struct PubsIdentity {
    handle: Option<String>,
    did: String,
    pds: String,
}

/// `pubs list`: resolve the handle (or DID) → PDS, then `listRecords`
/// both publication collections to exhaustion (public, unauthenticated).
async fn pubs_list(json: bool, handle: Option<String>) -> Result<i32> {
    let input = match handle.or_else(default_handle_from_config) {
        Some(h) => h,
        None => {
            eprintln!(
                "error: no handle: pass --handle <handle> (or set `handle` in {})",
                config_toml_path()?.display()
            );
            return Ok(2);
        }
    };

    let resolver = springleaf_atproto::Resolver::new();
    let identity = resolver.resolve(&input).await.with_context(|| format!("resolving identity for {input}"))?;
    if identity.handle_binding_verified == Some(false) {
        eprintln!("warning: DID document does not list at://{input} in alsoKnownAs (unidirectional binding)");
    }
    let client = springleaf_atproto::XrpcClient::new(&identity.pds_endpoint).with_context(|| format!("invalid PDS endpoint {:?}", identity.pds_endpoint))?;
    let did = identity.did.to_string();

    let mut publications = Vec::new();
    for collection in ["site.standard.publication", "pub.leaflet.publication"] {
        let records = client
            .list_all_records(&did, collection)
            .await
            .with_context(|| format!("listRecords {collection}"))?;
        for record in records {
            publications.push(summarize_publication(collection, &record));
        }
    }

    let list = PubsList {
        version: 1,
        identity: PubsIdentity {
            handle: springleaf_atproto::Handle::parse(&input).ok().map(|h| h.to_string()),
            did,
            pds: identity.pds_endpoint,
        },
        count: publications.len(),
        publications,
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&list)?);
    } else {
        print!("{}", pubs_table(&list));
    }
    Ok(0)
}

/// Decode one raw record into a summary. Third-party publications are not
/// guaranteed lexicon-clean (experiment 03 §2: blento's `preferences` is
/// an open object that does not fit the generated type), so the generated
/// types are tried first and raw field extraction is the fallback — the
/// generated-type compatibility is exactly why both paths stay.
fn summarize_publication(collection: &str, record: &springleaf_atproto::RawRecord) -> PublicationSummary {
    use springleaf_leaflet::DefaultStr;
    let typed = if collection == "site.standard.publication" {
        serde_json::from_value::<springleaf_leaflet::site_standard::publication::Publication<DefaultStr>>(record.value.clone())
            .ok()
            .map(|p| (p.name.as_str().to_string(), Some(p.url.as_str().to_string())))
    } else {
        serde_json::from_value::<springleaf_leaflet::pub_leaflet::publication::Publication<DefaultStr>>(record.value.clone())
            .ok()
            .map(|p| (p.name.as_str().to_string(), None))
    };
    let (name, url) = match typed {
        Some(pair) => pair,
        None => (
            record.value.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            record.value.get("url").and_then(|v| v.as_str()).map(str::to_string),
        ),
    };
    PublicationSummary {
        collection: collection.to_string(),
        rkey: record.uri.rsplit('/').next().unwrap_or_default().to_string(),
        uri: record.uri.clone(),
        cid: Some(record.cid.clone()),
        name: (!name.is_empty()).then_some(name),
        url,
    }
}

/// Human table: COLLECTION | RKEY | NAME (38 chars, ellipsized) | URL,
/// footer with counts and identity.
fn pubs_table(list: &PubsList) -> String {
    let mut out = String::new();
    let name_width = 40;
    let collection_width = list.publications.iter().map(|p| p.collection.len()).max().unwrap_or(21).max("COLLECTION".len());
    let rkey_width = list.publications.iter().map(|p| p.rkey.len()).max().unwrap_or(4).max("RKEY".len());
    let _ = writeln!(out, "{:<collection_width$} {:<rkey_width$} {:<name_width$} URL", "COLLECTION", "RKEY", "NAME",);
    for p in &list.publications {
        let name = p.name.as_deref().unwrap_or("(unnamed)");
        let name = if name.chars().count() > 38 {
            format!("{}…", name.chars().take(35).collect::<String>())
        } else {
            name.to_string()
        };
        let _ = writeln!(
            out,
            "{:<collection_width$} {:<rkey_width$} {:<name_width$} {}",
            p.collection,
            p.rkey,
            name,
            p.url.as_deref().unwrap_or(""),
        );
    }
    let standard = list.publications.iter().filter(|p| p.collection == "site.standard.publication").count();
    let legacy = list.count - standard;
    let who = list
        .identity
        .handle
        .as_deref()
        .map(|h| format!(" as {h} ({})", list.identity.did))
        .unwrap_or_else(|| format!(" as {}", list.identity.did));
    let _ = writeln!(
        out,
        "\n{} publications ({} site.standard, {} pub.leaflet legacy){who}",
        list.count, standard, legacy,
    );
    out
}

/// Path of the optional identity-defaults config (see `workspace`).
fn config_toml_path() -> Result<std::path::PathBuf> {
    workspace::xdg_config_toml_path()
}

/// Default handle from the config file, if present.
fn default_handle_from_config() -> Option<String> {
    workspace::toml_value_at(&config_toml_path().ok()?, "handle")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn synthetic_list() -> LeafletList {
        springleaf_drafts::parse_home_leaflets(
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../experiments/01-draft-list/fixture-home.html")),
            "home-rsc",
        )
        .unwrap()
    }

    #[test]
    fn human_table_matches_report_shape() {
        let table = human_table(&synthetic_list());
        let lines: Vec<&str> = table.lines().collect();
        // 15/12/40/36-column layout per experiments/01 §5 (ids are 36 wide).
        assert_eq!(
            lines[0],
            "STATE           CREATED      TITLE                                    \
             TOKEN-ID                             PUBLISHED-URL"
        );
        // legacy v4 row sorts last by row created_at
        assert!(lines[5].starts_with("draft           2025-11-24"));
        assert!(lines[5].contains("12345678-1234-4abc-9def-123456789abc"));
        // contributor row carries the `*` marker
        assert!(lines[2].ends_with('*'), "{}", lines[2]);
        // long title truncated to 35 chars + ellipsis inside a 40-wide column
        assert!(lines[1].contains("A synthetic draft title long enough…"), "{}", lines[1]);
        assert!(
            lines
                .last()
                .unwrap()
                .contains("5 leaflets (2 published, 3 draft/unpublished) as example.bsky.social")
        );
        // no mismatch warning on the fixture
        assert!(!table.contains("warning: href/payload"));
    }

    #[test]
    fn show_rendering_from_experiment02_fixture_shape() {
        let envelope = json!({
            "result": {"error": null, "data": {
                "id": "01999000-0000-7aaa-8000-000000000011",
                "root_entity": "01999000-0000-7aaa-8000-000000000012",
                "title": null, "description": null, "blocked_by_admin": null,
                "permission_token_rights": [
                    {"token": "01999000-0000-7aaa-8000-000000000011",
                     "entity_set": "01999000-0000-7aaa-8000-000000000013",
                     "read": true, "write": true}
                ],
                "leaflets_in_publications": [
                    {"leaflet": "01999000-0000-7aaa-8000-000000000011",
                     "publication": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/pub.leaflet.publication/3kz7examplepub",
                     "title": "Example linked draft",
                     "doc": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.document/3kz7exampledoc",
                     "documents": {"data": {"path": "example-linked",
                         "title": "Example linked draft",
                         "publishedAt": "2026-02-03T05:06:07.123456Z"}},
                     "publications": {"record": {"name": "Example Pub", "url": null}}}
                ],
                "leaflets_to_documents": [], "publications": [],
                "custom_domain_routes": [], "publication_scheduled_posts": null,
                "reply_drafts": null
            }}
        });
        let data: LeafletData = serde_json::from_value(envelope["result"]["data"].clone()).unwrap();
        let text = show_text(&data);
        assert!(text.contains("token id:      01999000-0000-7aaa-8000-000000000011"));
        assert!(text.contains("state:         published"));
        assert!(text.contains("rights:        read=true write=true"));
        assert!(text.contains("entity set:    01999000-0000-7aaa-8000-000000000013"));
        assert!(text.contains("doc: at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.document/3kz7exampledoc"));
        assert!(text.contains("published at: 2026-02-03T05:06:07.123456Z"));
    }

    #[test]
    fn show_rendering_empty_draft() {
        let envelope = json!({
            "result": {"error": null, "data": {
                "id": "01999000-0000-7aaa-8000-000000000001",
                "root_entity": "01999000-0000-7aaa-8000-000000000002",
                "title": null, "description": null, "blocked_by_admin": null,
                "permission_token_rights": [
                    {"token": "01999000-0000-7aaa-8000-000000000001",
                     "entity_set": "01999000-0000-7aaa-8000-000000000003",
                     "read": true, "write": true}
                ],
                "leaflets_in_publications": [], "leaflets_to_documents": [],
                "publications": [], "custom_domain_routes": [],
                "publication_scheduled_posts": null, "reply_drafts": null
            }}
        });
        let data: LeafletData = serde_json::from_value(envelope["result"]["data"].clone()).unwrap();
        let text = show_text(&data);
        assert!(text.contains("title:         (untitled)"));
        assert!(text.contains("state:         draft"));
        assert!(text.contains("publication linkage: none"));
        assert!(!text.contains("reply drafts"));
    }

    // ── pubs list ─────────────────────────────────────────────────────────

    fn raw_record(collection: &str, rkey: &str, value: serde_json::Value) -> springleaf_atproto::RawRecord {
        springleaf_atproto::RawRecord {
            uri: format!("at://did:plc:zjbq26wybii5ojoypkso2mso/{collection}/{rkey}"),
            cid: format!("bafyrei-{rkey}"),
            value,
        }
    }

    fn pubs_list_from(records: &[springleaf_atproto::RawRecord]) -> PubsList {
        PubsList {
            version: 1,
            identity: PubsIdentity {
                handle: Some("jauntywunderkind.bsky.social".into()),
                did: "did:plc:zjbq26wybii5ojoypkso2mso".into(),
                pds: "https://puffball.us-east.host.bsky.network".into(),
            },
            count: records.len(),
            publications: records
                .iter()
                .map(|r| {
                    summarize_publication(
                        if r.uri.contains("pub.leaflet.publication") {
                            "pub.leaflet.publication"
                        } else {
                            "site.standard.publication"
                        },
                        r,
                    )
                })
                .collect(),
        }
    }

    /// Experiment 03 §2's inventory shapes: a lexicon-clean publication,
    /// a third-party publication with an open-object `preferences` that
    /// does not fit the generated type (blento), and one with a tolerated
    /// extra field (skyreader).
    #[test]
    fn pubs_summaries_decode_typed_and_fallback() {
        let records = vec![
            raw_record(
                "site.standard.publication",
                "3molsbwkh522o",
                json!({
                    "$type": "site.standard.publication",
                    "name": "JauntyWunderKind's Technautical Expeditions",
                    "url": "https://technautical.leaflet.pub"
                }),
            ),
            raw_record(
                "site.standard.publication",
                "blento.self",
                json!({
                    "$type": "site.standard.publication",
                    "name": "JauntyWunderKind",
                    "url": "https://blento.app/jauntywk.bsky.social",
                    "preferences": {"baseColor": "#ffffff", "accentColor": "#000000"}
                }),
            ),
            raw_record(
                "site.standard.publication",
                "skyreader-links",
                json!({
                    "$type": "site.standard.publication",
                    "name": "JauntyWunderKind's links",
                    "url": "https://linkblogs.skyreader.app/did:plc:zjbq26wybii5ojoypkso2mso/",
                    "skyreaderLinkblog": {"syndicate": true}
                }),
            ),
            raw_record(
                "pub.leaflet.publication",
                "legacy1",
                json!({
                    "$type": "pub.leaflet.publication", "name": "Legacy shape"
                }),
            ),
        ];
        let list = pubs_list_from(&records);
        assert_eq!(list.count, 4);
        let technautical = &list.publications[0];
        assert_eq!(technautical.rkey, "3molsbwkh522o");
        assert_eq!(technautical.name.as_deref(), Some("JauntyWunderKind's Technautical Expeditions"));
        assert_eq!(technautical.url.as_deref(), Some("https://technautical.leaflet.pub"));
        // blento: preferences shape does not fit the generated type — the
        // raw fallback still extracts name/url.
        let blento = &list.publications[1];
        assert_eq!(blento.name.as_deref(), Some("JauntyWunderKind"));
        assert_eq!(blento.url.as_deref(), Some("https://blento.app/jauntywk.bsky.social"));
        // skyreader: unknown extra field tolerated by extra_data (typed).
        let skyreader = &list.publications[2];
        assert_eq!(skyreader.name.as_deref(), Some("JauntyWunderKind's links"));
        // legacy collection: no url field in the lexicon.
        let legacy = &list.publications[3];
        assert_eq!(legacy.name.as_deref(), Some("Legacy shape"));
        assert_eq!(legacy.url, None);
    }

    #[test]
    fn pubs_human_table_shape() {
        let records = vec![
            raw_record(
                "site.standard.publication",
                "3molsbwkh522o",
                json!({
                    "name": "JauntyWunderKind's Technautical Expeditions", "url": "https://technautical.leaflet.pub"
                }),
            ),
            raw_record(
                "site.standard.publication",
                "blento.self",
                json!({
                    "name": "JauntyWunderKind", "url": "https://blento.app/jauntywk.bsky.social",
                    "preferences": {"baseColor": "#fff"}
                }),
            ),
        ];
        let table = pubs_table(&pubs_list_from(&records));
        let lines: Vec<&str> = table.lines().collect();
        assert!(lines[0].starts_with("COLLECTION"), "{}", lines[0]);
        assert!(lines[1].contains("3molsbwkh522o"));
        // 42-char name ellipsized to 35 chars + … inside the 40-wide column
        assert!(lines[1].contains("JauntyWunderKind's Technautical Exp…"), "{}", lines[1]);
        assert!(lines[1].contains("https://technautical.leaflet.pub"));
        assert!(
            table
                .lines()
                .last()
                .unwrap()
                .contains("2 publications (2 site.standard, 0 pub.leaflet legacy) as jauntywunderkind.bsky.social")
        );
    }

    // (`handle` toml parsing is covered in `workspace`'s tests.)
}
