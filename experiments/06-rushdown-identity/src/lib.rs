//! Offline Rushdown extension experiment; see REPORT.md for the dialect boundary.

mod extension;
mod projection;

use rushdown::{ast::*, parser, renderer::html, text};
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

pub use projection::{Block, Inline, SourceSegment};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Plain,
    Attributes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Serialize)]
pub struct Parsed {
    #[serde(skip)]
    source: String,
    pub blocks: Vec<Block>,
    pub diagnostics: Vec<Diagnostic>,
    pub html: String,
}

impl Parsed {
    /// No serialization or reconstruction: retain the authoritative source bytes.
    pub fn original_source(&self) -> &str {
        &self.source
    }
}

#[derive(Debug, Clone)]
struct Assignment {
    target: NodeRef,
    key: String,
    span: Span,
}

#[derive(Debug, Default)]
struct State {
    assignments: Vec<Assignment>,
    diagnostics: Vec<Diagnostic>,
}

pub fn parse(source: &str, mode: Mode) -> Parsed {
    let state = Rc::new(RefCell::new(State::default()));
    let parser = parser::Parser::with_extensions(
        // Deliberately disable built-in HTML id attributes and heading slugging.
        parser::Options::default(),
        extension::identity(mode, state.clone()),
    );
    let (arena, root) = parser.parse(&mut text::BasicReader::new(source));
    let mut html = String::new();
    html::Renderer::with_options(html::Options::default())
        .render(&mut html, source, &arena, root)
        .expect("rendering a String cannot fail");
    let state = state.borrow();
    Parsed {
        source: source.into(),
        blocks: arena[root]
            .children(&arena)
            .map(|n| projection::block(&arena, n, source, &state.assignments))
            .collect(),
        diagnostics: state.diagnostics.clone(),
        html,
    }
}
