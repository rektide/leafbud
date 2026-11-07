//! Markdown → leaflet page-blocks converter, at social-cli's v0 subset.
//!
//! Port of `markdownToBlocks` + `parseInlineWithFacets` from
//! `~/archive/letta-ai/social-cli/src/commands/publish.ts` (the reference
//! implementation named by `design/springleaf.glm53m.md`), with the facet
//! math proven by `experiments/03-post-builder/`:
//!
//! - **Block level**: split on blank lines (one or more newlines with only
//!   whitespace between). Single-line ATX headings (`#{1,6}` + whitespace +
//!   text, trailing `#`s stripped) become `pub.leaflet.blocks.header`;
//!   everything else becomes a `pub.leaflet.blocks.text` paragraph with
//!   internal newlines collapsed to a single space.
//! - **Inline**: `[text](url)`, `**bold**`/`__bold__`, `*italic*`/`_italic_`
//!   (word-boundary guarded), `` `code` ``, and `**[text](url)**`
//!   (bold+link, both features on one span) become facets.
//!   `\<https?|mailto:…\>` autolinks normalize to bare URLs. Backslash
//!   escapes (`\*`, `\_`, `` \` ``, `\[`, `\]`, `\(`, `\)`, …) yield the
//!   literal character and never open a token — a deliberate divergence
//!   from social-cli, which strips escapes in a pre-pass and then
//!   re-interprets the exposed markers (`\*x\*` becomes italic there;
//!   here it stays literal, per CommonMark). Unclosed markers stay
//!   literal. Emphasis never nests inside link text (social-cli parity).
//! - **Byte offsets**: facet indices count **UTF-8 bytes** of the emitted
//!   plaintext, never chars or UTF-16 units (experiment 03 §4: the sample's
//!   facets sit at byte 73 but char 67 — six non-ASCII bytes earlier in the
//!   line; `plaintext.as_bytes()[start..end]` must round-trip to the marked
//!   span, which the tests assert).
//!
//! Deferred, deliberately (documented divergence from social-cli, which
//! uploads blobs):
//!
//! - **Images** (`![alt](path)` as a whole paragraph): need
//!   `uploadBlob` + the top-level `blobs[]` mirroring question (Blob-GC
//!   gate, design §Risks). The converter refuses them with
//!   [`MarkdownError::Unsupported`] rather than uploading or silently
//!   dropping.
//! - **Lists and blockquotes**: passed through as plain text paragraphs,
//!   markers intact (exactly what social-cli's v0 does with them).

use jacquard_common::DefaultStr;

use crate::pub_leaflet::blocks::header::Header;
use crate::pub_leaflet::blocks::text::Text;
use crate::pub_leaflet::pages::linear_document::{Block, BlockBlock};
use crate::pub_leaflet::richtext::facet::{ByteSlice, Facet, FacetFeaturesItem};

/// Converter errors. Only the unsupported-construct case exists today.
#[derive(Debug, thiserror::Error)]
pub enum MarkdownError {
    #[error("markdown line {line}: {construct} is not supported yet — {detail}")]
    Unsupported {
        line: usize,
        construct: &'static str,
        detail: &'static str,
    },
}

/// Convert a markdown body into `pub.leaflet.pages.linearDocument` blocks.
pub fn convert_markdown(markdown: &str) -> Result<Vec<Block<DefaultStr>>, MarkdownError> {
    let mut blocks = Vec::new();
    let mut paragraph: Vec<(usize, &str)> = Vec::new(); // (1-based line no, line)
    for (idx, line) in markdown.lines().enumerate() {
        if line.trim().is_empty() {
            flush_paragraph(&mut paragraph, &mut blocks)?;
        } else {
            paragraph.push((idx + 1, line));
        }
    }
    flush_paragraph(&mut paragraph, &mut blocks)?;
    Ok(blocks)
}

/// Turn the collected paragraph lines into one block (empty when the
/// paragraph is only whitespace). `paragraph[0].0` is its first line
/// number, for error messages.
fn flush_paragraph(paragraph: &mut Vec<(usize, &str)>, blocks: &mut Vec<Block<DefaultStr>>) -> Result<(), MarkdownError> {
    if paragraph.is_empty() {
        return Ok(());
    }
    let first_line = paragraph[0].0;
    let joined = paragraph.iter().map(|(_, l)| *l).collect::<Vec<_>>().join("\n");
    paragraph.clear();
    let para = joined.trim();
    if !para.is_empty() {
        blocks.push(paragraph_block(para, first_line)?);
    }
    Ok(())
}

fn paragraph_block(para: &str, line: usize) -> Result<Block<DefaultStr>, MarkdownError> {
    // Standalone image: deferred behind the blob-GC gate (see module docs).
    if is_standalone_image(para) {
        return Err(MarkdownError::Unsupported {
            line,
            construct: "image",
            detail: "needs uploadBlob + the blobs[] GC-mirroring question answered; refused rather than dropped",
        });
    }
    // ATX heading: single line, 1–6 hashes + whitespace + non-empty text
    // (trailing `#`s stripped after trailing whitespace, mirroring
    // social-cli's regex).
    if !para.contains('\n')
        && let Some((level, text)) = parse_atx_heading(para)
    {
        let parsed = parse_inline(text);
        let header = Header {
            level: Some(level as i64),
            plaintext: parsed.plaintext.into(),
            facets: non_empty_facets(parsed.facets),
            extra_data: None,
        };
        return Ok(Block {
            alignment: None,
            block: BlockBlock::Header(Box::new(header)),
            extra_data: None,
        });
    }
    // Text paragraph: collapse internal newlines to single spaces.
    let flat = collapse_internal_newlines(para);
    let parsed = parse_inline(&flat);
    let text = Text {
        facets: non_empty_facets(parsed.facets),
        plaintext: parsed.plaintext.into(),
        text_size: None,
        extra_data: None,
    };
    Ok(Block {
        alignment: None,
        block: BlockBlock::Text(Box::new(text)),
        extra_data: None,
    })
}

/// `![alt](anything)` spanning the whole paragraph (social-cli's
/// standalone-image shape).
fn is_standalone_image(para: &str) -> bool {
    let rest = match para.strip_prefix("![") {
        Some(r) => r,
        None => return false,
    };
    match rest.find("](") {
        Some(i) => rest[i + 2..].strip_suffix(')').is_some_and(|u| !u.contains(')')),
        None => false,
    }
}

/// `#{1,6}` + whitespace + text, with trailing `#`s and whitespace
/// stripped. Content must stay non-empty after stripping.
fn parse_atx_heading(para: &str) -> Option<(usize, &str)> {
    let hashes = para.chars().take_while(|&c| c == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let after = &para[hashes..];
    let content = after.strip_prefix([' ', '\t'])?;
    // No whitespace requirement after a single hash run edge: social-cli
    // requires `\s+` (at least one); strip_prefix handles exactly one, so
    // reject the empty remainder for the multi-space case lazily below.
    let trimmed = content.trim_end();
    let stripped = trimmed.trim_end_matches('#');
    let text = stripped.trim_end();
    if text.is_empty() {
        return None;
    }
    Some((hashes, text))
}

/// Internal whitespace runs that contain a newline collapse to a single
/// space (social-cli: `para.replace(/\s*\n\s*/g, " ")`); whitespace runs
/// without a newline stay exactly as written.
fn collapse_internal_newlines(para: &str) -> String {
    let mut out = String::with_capacity(para.len());
    let mut run = String::new();
    let mut run_has_newline = false;
    let flush_run = |out: &mut String, run: &mut String, has_newline: &mut bool| {
        if *has_newline {
            out.push(' ');
        } else {
            out.push_str(run);
        }
        run.clear();
        *has_newline = false;
    };
    for ch in para.chars() {
        if ch.is_whitespace() {
            run.push(ch);
            if ch == '\n' {
                run_has_newline = true;
            }
        } else {
            flush_run(&mut out, &mut run, &mut run_has_newline);
            out.push(ch);
        }
    }
    // The paragraph is pre-trimmed, so a trailing run (if any) is dropped
    // rather than re-emitted.
    out
}

// ---------------------------------------------------------------------------
// Inline parsing (social-cli parseInlineWithFacets, token priority order)
// ---------------------------------------------------------------------------

/// One facet feature in converter-neutral form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Feature {
    Link(String),
    Bold,
    Italic,
    Code,
}

/// A parsed inline result: plaintext plus byte-spanned facet features.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Parsed {
    pub plaintext: String,
    pub facets: Vec<(usize, usize, Vec<Feature>)>, // (byteStart, byteEnd, features)
}

/// JS `\w`: `[A-Za-z0-9_]`.
fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Normalize autolinks and escapes (no facets), then tokenize.
pub fn parse_inline(input: &str) -> Parsed {
    let normalized = normalize(input);
    tokenize(&normalized)
}

/// social-cli's escapable set (`\\ `*_{}[]()#+-.!`).
const ESCAPABLE: &[char] = &['\\', '*', '_', '`', '{', '}', '[', ']', '(', ')', '#', '+', '-', '.', '!'];

/// Autolinks normalize to bare URLs (no facets); escapes are handled at
/// tokenize time so they can never open a token.
fn normalize(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '<'
            && let Some((inner, consumed)) = autolink_at(&chars, i)
        {
            out.push_str(&inner);
            i += consumed;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Autolink at `i` (a `<`): returns (inner, chars consumed including both
/// brackets). Only `https:`, `http:`, `mailto:` prefixes qualify.
fn autolink_at(chars: &[char], i: usize) -> Option<(String, usize)> {
    let mut j = i + 1;
    let mut inner = String::new();
    while j < chars.len() && chars[j] != '>' {
        inner.push(chars[j]);
        j += 1;
    }
    if j >= chars.len() || inner.is_empty() {
        return None;
    }
    if !inner.starts_with("https:") && !inner.starts_with("http:") && !inner.starts_with("mailto:") {
        return None;
    }
    Some((inner, j - i + 1))
}

/// Tokenize normalized text with social-cli's priority: boldLink, bold,
/// link, italic, code, then one plain char. Facet spans count plaintext
/// UTF-8 bytes (`String::len`).
fn tokenize(input: &str) -> Parsed {
    let s: Vec<char> = input.chars().collect();
    let mut plaintext = String::new();
    let mut facets: Vec<(usize, usize, Vec<Feature>)> = Vec::new();
    let mut i = 0usize;

    macro_rules! emit {
        ($text:expr, $features:expr) => {{
            let text: &str = $text;
            let start = plaintext.len();
            plaintext.push_str(text);
            let end = plaintext.len();
            let features: Vec<Feature> = $features;
            if !features.is_empty() {
                facets.push((start, end, features));
            }
        }};
    }

    while i < s.len() {
        // 1. **[text](url)** — bold + link on the same span.
        if s[i] == '*'
            && s.get(i + 1) == Some(&'*')
            && s.get(i + 2) == Some(&'[')
            && let Some((text, uri, next)) = parse_link(&s, i + 2)
            && next + 1 < s.len()
            && s[next] == '*'
            && s[next + 1] == '*'
        {
            emit!(&text, vec![Feature::Bold, Feature::Link(uri)]);
            i = next + 2;
            continue;
        }
        // 2. **text** / __text__ — bold.
        if (s[i] == '*' && s.get(i + 1) == Some(&'*') || s[i] == '_' && s.get(i + 1) == Some(&'_'))
            && let Some(marker) = s.get(i).copied()
            && let Some((text, next)) = parse_delimited(&s, i, marker, 2)
        {
            emit!(&text, vec![Feature::Bold]);
            i = next;
            continue;
        }
        // 3. [text](url) — link.
        if s[i] == '['
            && let Some((text, uri, next)) = parse_link(&s, i)
        {
            emit!(&text, vec![Feature::Link(uri)]);
            i = next;
            continue;
        }
        // 4. *text* / _text_ — italic, word-boundary guarded
        //    (not adjacent to a word char or the same marker; the string
        //    start counts as a pass, like JS lookbehind).
        if (s[i] == '*' || s[i] == '_')
            && (i == 0 || {
                let prev = s[i - 1];
                !(is_word(prev) || prev == s[i])
            })
            && s.get(i + 1).is_some_and(|&c| c != s[i])
            && let Some(marker) = s.get(i).copied()
            && let Some((text, next)) = parse_delimited(&s, i, marker, 1)
            && s.get(next).is_none_or(|&c| !(is_word(c) || c == marker))
        {
            emit!(&text, vec![Feature::Italic]);
            i = next;
            continue;
        }
        // 5. `text` — code.
        if s[i] == '`'
            && let Some((text, next)) = parse_delimited(&s, i, '`', 1)
        {
            emit!(&text, vec![Feature::Code]);
            i = next;
            continue;
        }
        // 6. Plain char — or an escape pair: the literal character, never
        //    a token opener (see module docs for the divergence note).
        if s[i] == '\\'
            && let Some(&next) = s.get(i + 1)
            && ESCAPABLE.contains(&next)
        {
            plaintext.push(next);
            i += 2;
            continue;
        }
        plaintext.push(s[i]);
        i += 1;
    }
    Parsed { plaintext, facets }
}

/// True when `j` starts an escape pair (the backslash escapes the char
/// after it).
fn is_escape_at(s: &[char], j: usize) -> bool {
    s[j] == '\\' && s.get(j + 1).is_some_and(|c| ESCAPABLE.contains(c))
}

/// Parse `[text](url)` starting at the `[` at `i`. Returns
/// (text, uri, index after the closing paren). Escape pairs inside text or
/// uri yield the escaped char literally.
fn parse_link(s: &[char], i: usize) -> Option<(String, String, usize)> {
    let mut j = i + 1;
    let mut text = String::new();
    while j < s.len() && s[j] != ']' {
        if is_escape_at(s, j) {
            text.push(s[j + 1]);
            j += 2;
        } else {
            text.push(s[j]);
            j += 1;
        }
    }
    if j >= s.len() || text.is_empty() || s.get(j + 1) != Some(&'(') {
        return None;
    }
    let mut k = j + 2;
    let mut uri = String::new();
    while k < s.len() && s[k] != ')' {
        if is_escape_at(s, k) {
            uri.push(s[k + 1]);
            k += 2;
        } else {
            uri.push(s[k]);
            k += 1;
        }
    }
    if k >= s.len() || uri.is_empty() {
        return None;
    }
    Some((text, uri, k + 1))
}

/// Parse a matching `marker.repeat(count)…marker.repeat(count)` span at `i`
/// (content excludes the marker char entirely, mirroring social-cli's
/// character classes; escape pairs inside content are unescaped). Returns
/// (content, index after the closing run).
fn parse_delimited(s: &[char], i: usize, marker: char, count: usize) -> Option<(String, usize)> {
    let mut j = i + count;
    let mut content = String::new();
    while j < s.len() && s[j] != marker {
        if is_escape_at(s, j) {
            content.push(s[j + 1]);
            j += 2;
        } else {
            content.push(s[j]);
            j += 1;
        }
    }
    if content.is_empty() {
        return None;
    }
    // Closing run: exactly `count` markers (a longer run still matches at
    // its start, like the regex would).
    for offset in 0..count {
        if s.get(j + offset) != Some(&marker) {
            return None;
        }
    }
    Some((content, j + count))
}

/// Convert parsed facets to generated wire facets (or `None` when empty).
fn non_empty_facets(facets: Vec<(usize, usize, Vec<Feature>)>) -> Option<Vec<Facet<DefaultStr>>> {
    if facets.is_empty() {
        return None;
    }
    Some(
        facets
            .into_iter()
            .map(|(start, end, features)| Facet {
                features: features
                    .into_iter()
                    .map(|f| match f {
                        Feature::Link(uri) => FacetFeaturesItem::Link(Box::new(crate::pub_leaflet::richtext::facet::Link {
                            uri: uri.into(),
                            extra_data: None,
                        })),
                        Feature::Bold => FacetFeaturesItem::Bold(Box::new(crate::pub_leaflet::richtext::facet::Bold::default())),
                        Feature::Italic => FacetFeaturesItem::Italic(Box::new(crate::pub_leaflet::richtext::facet::Italic::default())),
                        Feature::Code => FacetFeaturesItem::Code(Box::new(crate::pub_leaflet::richtext::facet::Code::default())),
                    })
                    .collect(),
                index: ByteSlice {
                    byte_start: start as i64,
                    byte_end: end as i64,
                    extra_data: None,
                },
                extra_data: None,
            })
            .collect(),
    )
}

/// The document-level `textContent` field: block plaintexts joined with
/// newlines, empty entries dropped (social-cli `blocksToTextContent`,
/// minus the image-alt case — this converter has no image blocks).
pub fn blocks_to_text_content(blocks: &[Block<DefaultStr>]) -> String {
    blocks
        .iter()
        .filter_map(|b| match &b.block {
            BlockBlock::Text(t) => Some(t.plaintext.as_str().to_string()),
            BlockBlock::Header(h) => Some(h.plaintext.as_str().to_string()),
            _ => None,
        })
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Experiment 03 §4's facet-bearing paragraph, verbatim: five
    /// non-ASCII code points before the first facet force byte-offset
    /// arithmetic. The expected spans are the experiment's independently
    /// computed table.
    #[test]
    fn facet_byte_offsets_match_experiment03() {
        let para = "Émigré — the cafés of Montréal sit before the facet below: see the [Leaflet homepage](https://leaflet.pub) for the renderer. **Bold claim**, *italic aside*, and `inline code` finish the sweep.";
        let parsed = parse_inline(para);
        assert_eq!(
            parsed.plaintext,
            "Émigré — the cafés of Montréal sit before the facet below: see the Leaflet homepage for the renderer. Bold claim, italic aside, and inline code finish the sweep."
        );
        let expect: Vec<(usize, usize, Vec<Feature>)> = vec![
            (73, 89, vec![Feature::Link("https://leaflet.pub".into())]),
            (108, 118, vec![Feature::Bold]),
            (120, 132, vec![Feature::Italic]),
            (138, 149, vec![Feature::Code]),
        ];
        assert_eq!(parsed.facets, expect);

        // Byte-vs-char proof (experiment 03 §4): the link sits at char
        // index 67 but byte offset 73.
        let needle = "Leaflet homepage";
        let byte_idx = parsed.plaintext.find(needle).unwrap();
        let char_idx = parsed.plaintext[..byte_idx].chars().count();
        assert_eq!(char_idx, 67, "sample char index is stable");
        assert_eq!(byte_idx, 73);
        assert!(expect[0].0 > char_idx, "byte offset must exceed char index");

        // Round-trip: slicing plaintext bytes at each span decodes to the
        // marked feature text.
        let bytes = parsed.plaintext.as_bytes();
        assert_eq!(&bytes[73..89], b"Leaflet homepage");
        assert_eq!(&bytes[108..118], b"Bold claim");
        assert_eq!(&bytes[120..132], b"italic aside");
        assert_eq!(&bytes[138..149], b"inline code");
    }

    #[test]
    fn headings_h1_to_h6_and_trailing_hashes() {
        let blocks = convert_markdown("# A\n\n## B\n\n### C\n\n#### D\n\n##### E\n\n###### F\n\n## Closing hashes ##").unwrap();
        let levels: Vec<(Option<i64>, &str)> = blocks
            .iter()
            .filter_map(|b| match &b.block {
                BlockBlock::Header(h) => Some((h.level, h.plaintext.as_ref())),
                _ => None,
            })
            .collect();
        assert_eq!(
            levels,
            vec![
                (Some(1), "A"),
                (Some(2), "B"),
                (Some(3), "C"),
                (Some(4), "D"),
                (Some(5), "E"),
                (Some(6), "F"),
                (Some(2), "Closing hashes"),
            ]
        );
        // 7 hashes and no-space `#tag` are paragraphs, not headings.
        let blocks = convert_markdown("####### seven\n\n#tag").unwrap();
        assert!(blocks.iter().all(|b| matches!(b.block, BlockBlock::Text(_))));
    }

    #[test]
    fn paragraphs_group_on_blank_lines_and_flatten_newlines() {
        let blocks = convert_markdown("first para\ncontinues here\n\n\nsecond   \n  para\n\n\n\ntail").unwrap();
        let texts: Vec<&str> = blocks
            .iter()
            .filter_map(|b| match &b.block {
                BlockBlock::Text(t) => Some(t.plaintext.as_ref()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, vec!["first para continues here", "second para", "tail"]);
    }

    #[test]
    fn bold_link_combined_span() {
        let parsed = parse_inline("see **[the docs](https://example.com)** now");
        assert_eq!(parsed.plaintext, "see the docs now");
        assert_eq!(parsed.facets, vec![(4, 12, vec![Feature::Bold, Feature::Link("https://example.com".into())])]);
    }

    #[test]
    fn italic_word_boundaries() {
        // Interior asterisks inside a word: no italic (lookbehind).
        assert_eq!(parse_inline("a*b*c").facets, vec![]);
        // snake_case_words do not become italic.
        assert_eq!(parse_inline("snake_case_word").facets, vec![]);
        // Standalone emphasis works with both markers, including at the
        // very start of the string (JS lookbehind passes at position 0).
        assert_eq!(parse_inline("*i*").facets, vec![(0, 1, vec![Feature::Italic])]);
        assert_eq!(parse_inline("__b__").facets, vec![(0, 1, vec![Feature::Bold])]);
        // Bold followed directly by italic with no separator: the italic's
        // preceding `*` blocks it (social-cli lookbehind semantics).
        let parsed = parse_inline("**b***i*");
        assert_eq!(parsed.plaintext, "b*i*");
        assert_eq!(parsed.facets, vec![(0, 1, vec![Feature::Bold])]);
        // Spaced single markers DO italicize in social-cli (lazy content
        // " b ", boundary chars are spaces) — markers leave doubled spaces.
        let parsed = parse_inline("a * b * c");
        assert_eq!(parsed.plaintext, "a  b  c");
        assert_eq!(parsed.facets, vec![(2, 5, vec![Feature::Italic])]);
    }

    #[test]
    fn unclosed_and_empty_markers_stay_literal() {
        assert_eq!(parse_inline("**bold").plaintext, "**bold");
        assert_eq!(parse_inline("*").plaintext, "*");
        assert_eq!(parse_inline("****").plaintext, "****");
        assert_eq!(parse_inline("[]()").plaintext, "[]()");
        assert_eq!(parse_inline("[only").plaintext, "[only");
        assert_eq!(parse_inline("a (b) c").plaintext, "a (b) c");
    }

    #[test]
    fn escapes_and_autolinks_normalize() {
        // Escapes yield literals and never open tokens (divergence from
        // social-cli's pre-pass, which would re-italicize; see module docs).
        let parsed = parse_inline("\\*not italic\\*");
        assert_eq!(parsed.plaintext, "*not italic*");
        assert!(parsed.facets.is_empty());
        assert_eq!(parse_inline("\\\\ backslash").plaintext, "\\ backslash");
        assert_eq!(parse_inline("a\\.b").plaintext, "a.b");
        assert_eq!(parse_inline("\\`code\\`").plaintext, "`code`");
        // Escapes inside link text and urls are unescaped.
        let parsed = parse_inline("[a\\]b](https://x.example/\\(1\\))");
        assert_eq!(parsed.plaintext, "a]b");
        assert_eq!(parsed.facets, vec![(0, 3, vec![Feature::Link("https://x.example/(1)".into())])]);
        // Autolinks.
        assert_eq!(parse_inline("<https://leaflet.pub>").plaintext, "https://leaflet.pub");
        assert_eq!(parse_inline("<mailto:x@y.example>").plaintext, "mailto:x@y.example");
        // Non-scheme brackets stay literal; autolinks produce no facets.
        assert_eq!(parse_inline("<not a link>").plaintext, "<not a link>");
        assert!(parse_inline("<https://例え.leaflet.pub/path>").facets.is_empty());
    }

    #[test]
    fn images_are_refused_with_line_numbers() {
        let err = convert_markdown("para\n\n![alt text](./img.png)\n\ntail").unwrap_err();
        assert!(
            matches!(
                err,
                MarkdownError::Unsupported {
                    line: 3,
                    construct: "image",
                    ..
                }
            ),
            "{err}"
        );
    }

    #[test]
    fn lists_and_quotes_pass_through_as_text() {
        // Documented deferral: no list/quote blocks in the v0 subset.
        let blocks = convert_markdown("- one\n- two\n\n> quoted").unwrap();
        let texts: Vec<&str> = blocks
            .iter()
            .filter_map(|b| match &b.block {
                BlockBlock::Text(t) => Some(t.plaintext.as_ref()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, vec!["- one - two", "> quoted"]);
    }

    #[test]
    fn text_content_joins_blocks_with_newlines() {
        let blocks = convert_markdown("# Title\n\nPara one.\n\nPara two with [link](https://x.example).").unwrap();
        assert_eq!(blocks_to_text_content(&blocks), "Title\nPara one.\nPara two with link.");
    }

    #[test]
    fn non_ascii_facet_inside_multibyte_content() {
        // A second byte-offset trap: the facet's own text is multi-byte.
        let parsed = parse_inline("префикс: *акцéнт* хвост");
        assert_eq!(parsed.plaintext, "префикс: акцéнт хвост");
        let (start, end, features) = &parsed.facets[0];
        // "префикс: " is 7 two-byte letters + ':' + ' ' = 16 bytes;
        // "акцéнт" is 6 two-byte letters = 12 bytes.
        assert_eq!((*start, *end), (16, 28));
        let bytes = parsed.plaintext.as_bytes();
        let sliced = std::str::from_utf8(&bytes[*start..*end]).unwrap();
        assert_eq!(sliced, "акцéнт");
        assert_eq!(*features, vec![Feature::Italic]);
    }
}
