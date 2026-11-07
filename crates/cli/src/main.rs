//! springleaf — a leaflet.pub client for agents and the CLI (stage 2).
//!
//! Commands: `auth status`, `drafts list [--json] [--fetch]`,
//! `drafts show <token-id> [--json]`. Read-only against production.
//!
//! Auth model: the `auth_token` cookie value resolved from `--token` >
//! `$LEAFLET_AUTH_TOKEN` > `~/.config/leaflet/auth-token`. The token is
//! never echoed, logged, or embedded in error messages — including at
//! debug/trace levels.
//!
//! Exit codes: 0 ok, 1 operational error (including anonymous `auth
//! status`), 2 usage error (clap).

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use springleaf_drafts::{DEFAULT_BASE_URL, Error as DraftsError, LeafletClient, LeafletData, LeafletList, LeafletState, classify};
use std::fmt::Write as _;
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "springleaf",
    version,
    about = "A leaflet.pub client for agents and the CLI",
    after_help = "The auth token is the auth_token cookie value: --token, $LEAFLET_AUTH_TOKEN, or ~/.config/leaflet/auth-token. It is never printed or logged."
)]
struct Cli {
    /// Leaflet base URL (default https://leaflet.pub; env LEAFLET_BASE_URL)
    #[arg(long, global = true, value_name = "URL")]
    base_url: Option<String>,

    /// Auth token (auth_token cookie value; env LEAFLET_AUTH_TOKEN; else
    /// ~/.config/leaflet/auth-token). Never printed or logged.
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
    // Resolution order: flag > env > default / token file. The token value
    // is never echoed, logged, or embedded in error text.
    let base_url = cli
        .base_url
        .clone()
        .or_else(|| std::env::var("LEAFLET_BASE_URL").ok())
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
    let flag_token = cli.token.clone().or_else(|| std::env::var("LEAFLET_AUTH_TOKEN").ok());
    let token = resolve_token(flag_token.as_deref())?;
    let client = LeafletClient::new(&base_url, token).with_context(|| format!("invalid base url {base_url:?}"))?;
    match cli.command {
        Command::Auth(AuthArgs { cmd: AuthCommand::Status }) => auth_status(&client).await,
        Command::Drafts(DraftsArgs {
            cmd: DraftsCommand::List { json, fetch },
        }) => drafts_list(&client, json, fetch).await,
        Command::Drafts(DraftsArgs {
            cmd: DraftsCommand::Show { token_id, json },
        }) => drafts_show(&client, &token_id, json).await,
    }
}

/// Token resolution: --token > $LEAFLET_AUTH_TOKEN > ~/.config/leaflet/auth-token.
/// The value is trimmed and returned opaque; it is never printed, logged, or
/// embedded in errors.
fn resolve_token(flag_or_env: Option<&str>) -> Result<Option<String>> {
    if let Some(token) = flag_or_env.map(str::trim).filter(|t| !t.is_empty()) {
        return Ok(Some(token.to_string()));
    }
    let home = std::env::var_os("HOME").context("token file: cannot locate ~/.config/leaflet/auth-token ($HOME unset)")?;
    let path = std::path::Path::new(&home).join(".config/leaflet/auth-token");
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
                 ~/.config/leaflet/auth-token"
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
}
