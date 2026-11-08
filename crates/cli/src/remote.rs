//! Rendering only. Credential resolution and reusable operations are in workflow.
use std::fmt::Write as _;

use springleaf_workflow::{LeafletState, RemoteList, RemoteOptions, RemoteShow};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn list(options: RemoteOptions, state: Option<LeafletState>, json: bool) -> Result<()> {
    let list = options.connect().await?.list(state).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&list)?);
    } else {
        print!("{}", list_text(&list));
    }
    if !list.href_payload_mismatch.is_empty() {
        eprintln!(
            "warning: /home href/payload token sets differ ({} IDs); inspect --json",
            list.href_payload_mismatch.len()
        );
    }
    Ok(())
}

pub async fn show(options: RemoteOptions, locator: &str, json: bool) -> Result<()> {
    let show = options.connect().await?.show(locator).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&show)?);
    } else {
        print!("{}", show_text(&show));
    }
    Ok(())
}

/// Remote strings are untrusted terminal input; escape control characters.
fn text(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| if c.is_control() { c.escape_default().collect::<Vec<_>>() } else { vec![c] })
        .collect()
}

fn list_text(list: &RemoteList) -> String {
    let mut out = format!("{:<10} {:<12} {:<40} TOKEN-ID\n", "STATE", "CREATED", "TITLE");
    for item in &list.leaflets {
        let leaflet = &item.leaflet;
        let title = text(leaflet.metadata.title.as_deref().unwrap_or("(untitled)"));
        let title = if title.chars().count() > 38 {
            format!("{}…", title.chars().take(35).collect::<String>())
        } else {
            title
        };
        let date: String = leaflet.created_at.as_deref().unwrap_or("unknown").chars().take(10).collect();
        let contributor = if leaflet.contributor { " *" } else { "" };
        let _ = writeln!(
            out,
            "{:<10} {:<12} {:<40} {}{contributor}",
            item.state.as_str(),
            text(&date),
            title,
            leaflet.metadata.id
        );
    }
    let _ = writeln!(
        out,
        "\n{} shown / {} discovered in /home{}",
        list.count,
        list.discovered_count,
        list.identity.handle.as_ref().map(|h| format!(" as {}", text(h))).unwrap_or_default()
    );
    let _ = writeln!(out, "Coverage: {}", list.coverage.limitation);
    if list.leaflets.iter().any(|l| l.leaflet.contributor) {
        out.push_str("* contributor access\n");
    }
    out
}

fn show_text(show: &RemoteShow) -> String {
    let metadata = &show.data.metadata;
    let mut out = String::new();
    let _ = writeln!(out, "token id:    {}", metadata.id);
    let _ = writeln!(out, "editor URL:  {}", show.editor_url);
    let _ = writeln!(out, "title:       {}", text(metadata.title.as_deref().unwrap_or("(untitled)")));
    if let Some(description) = &metadata.description {
        let _ = writeln!(out, "description: {}", text(description));
    }
    let _ = writeln!(out, "state:       {}", show.state.as_str());
    let _ = writeln!(out, "root entity: {}", text(metadata.root_entity.as_deref().unwrap_or("unknown")));
    for right in &metadata.permission_token_rights {
        let _ = writeln!(
            out,
            "rights:      read={} write={} (reported; not a write probe)",
            boolean(right.read),
            boolean(right.write)
        );
        if let Some(entity_set) = &right.entity_set {
            let _ = writeln!(out, "entity set:  {}", text(entity_set));
        }
    }
    if metadata.permission_token_rights.is_empty() {
        out.push_str("rights:      none reported; editing availability unknown\n");
    }
    if let Some(blocked) = show.data.raw.get("blocked_by_admin").filter(|v| !v.is_null()) {
        let _ = writeln!(out, "blocked:     {}", text(&blocked.to_string()));
    }
    if metadata.leaflets_in_publications.is_empty() {
        out.push_str("publication linkage: none\n");
    }
    for link in &metadata.leaflets_in_publications {
        let _ = writeln!(out, "publication: {}", text(link.publication.as_deref().unwrap_or("unknown")));
        let _ = writeln!(out, "document:    {}", text(link.doc.as_deref().unwrap_or("unpublished")));
        if let Some(at) = &link.scheduled_publish_at {
            let _ = writeln!(out, "scheduled:   {}", text(at));
        }
    }
    for link in &metadata.leaflets_to_documents {
        let _ = writeln!(out, "standalone document: {}", text(link.document.as_deref().unwrap_or("unknown")));
    }
    out.push_str("Metadata inspection only; no editor content fetched. Full RPC data: --json\n");
    out
}

fn boolean(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_text_cannot_inject_control_sequences() {
        assert_eq!(text("hello\n\x1b[31m"), "hello\\n\\u{1b}[31m");
        assert_eq!(boolean(None), "unknown");
    }
}
