//! Keyed MDX reading/writing for the `leafbud-mdx/2` source format: implicit
//! main-page Markdown, explicit named pages, one-time source-key preparation,
//! tracking-only settlement and checked rendering. Plain prose stays Markdown;
//! exact rich runs/whitespace use proven typed components. Updates patch owned
//! fields and the reserved envelope; unrelated author text survives.
//!
//! The format-2 grammar (documented unambiguously):
//!
//! - frontmatter fence first; body follows the closing fence;
//! - top-level Markdown blocks and `<Key id="…" />` markers accumulate, in
//!   order, into the **implicit main page**;
//! - `<Page name="…" kind="linear">…</Page>` components are named page
//!   definitions; they may appear anywhere at top level, never interrupt
//!   main-page order, and contain their own blocks/Key markers;
//! - a Key marker binds to the next block of its container; blocks without a
//!   marker are new and get keys only through preparation (`tidy --keys` or
//!   executed push), never from check/preview;
//! - flat `-`/`1.` lists are one keyed block per item (each item is a native
//!   text block with list facts); task/nested lists are unsupported input.
mod author;
mod keys;
mod names;
mod parse;
mod patch;
mod rekey;
mod render;
mod source;
mod view;

#[cfg(test)]
mod author_tests;
#[cfg(test)]
mod comment_tests;
#[cfg(test)]
mod tests;

use anyhow::Result;
use leafbud_document::*;
use serde_json::Value;

pub use author::{MetadataEdit, edit_metadata};
pub use keys::{KeyPreparation, KeySource, TidyEdit, apply_key_preparation, prepare_keys};
pub use names::patch_names;
pub use parse::{BlockComment, BlockSpan, PROVISIONAL_MAIN_PAGE, Parsed, header, parse, parse_with_spans};
pub use patch::{adopt_published, adopt_remote, patch_tracking, prepare_binding_defaults, settle_release, unbind_deleted};
pub use rekey::rename_key;
pub use render::{apply_markdown_tidy, plan_markdown_tidy, render};
pub use view::render_comparison;

pub(crate) fn json(value: &(impl serde::Serialize + ?Sized)) -> String {
    serde_json::to_string(value).expect("serializable document model")
}

pub(crate) fn frontmatter<T: serde::de::DeserializeOwned>(source: &str) -> Result<T> {
    let options = serde_saphyr::options! {
        duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
        merge_keys: serde_saphyr::MergeKeyPolicy::Error,
        reject_unsupported_tags: true,
        strict_booleans: true,
        no_schema: true,
        budget: serde_saphyr::budget! { max_anchors:0, max_aliases:0 },
    };
    serde_saphyr::from_str_with_options(source, options).map_err(|_| anyhow::anyhow!("invalid/restricted managed frontmatter"))
}

/// Emit ordinary, explicitly typed YAML, quoting every key/string (including
/// UUIDs and @leafbud). No aliases/tags/duplicate keys are generated.
pub(crate) fn yaml(value: &Value, indent: usize, out: &mut String) {
    if let Some(map) = value.as_object() {
        for (key, value) in map {
            let _ = std::fmt::Write::write_fmt(out, format_args!("{}{}:", " ".repeat(indent), json(key)));
            if value.as_object().is_some_and(|v| !v.is_empty()) {
                out.push('\n');
                yaml(value, indent + 2, out);
            } else {
                let _ = std::fmt::Write::write_fmt(out, format_args!(" {}\n", json(value)));
            }
        }
    }
}
