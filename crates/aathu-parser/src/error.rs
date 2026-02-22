//! Parse-error types.
//!
//! The parser stops at the first error and returns it. Error recovery is
//! left to a future pass; having precise span information is more valuable
//! at this stage than attempting to continue with a broken token stream.

use aathu_core::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
    pub line: u32,
    pub col: u32,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: Span, line: u32, col: u32) -> Self {
        Self { kind, span, line, col }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    /// Expected a specific token, found something else.
    /// `expected` is a human-readable description, `found` is the token text.
    UnexpectedToken { expected: String, found: String },

    /// Source ended before the construct was complete.
    UnexpectedEof { expected: String },

    /// Left-hand side of an assignment is not a valid target
    /// (must be an identifier, index expression, or member expression).
    InvalidAssignTarget,

    /// A `match` arm was missing the `=>` separator.
    MissingFatArrow,

    /// An `import` path contained an invalid segment.
    InvalidImportPath,

    /// A function parameter list contained a non-identifier.
    InvalidParam,

    /// An integer / float literal overflowed the target type.
    LiteralOverflow(String),
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorKind::UnexpectedToken { expected, found } =>
                write!(f, "expected {expected}, found `{found}`"),
            ParseErrorKind::UnexpectedEof { expected } =>
                write!(f, "unexpected end of file, expected {expected}"),
            ParseErrorKind::InvalidAssignTarget =>
                write!(f, "invalid assignment target"),
            ParseErrorKind::MissingFatArrow =>
                write!(f, "expected `=>` after match pattern"),
            ParseErrorKind::InvalidImportPath =>
                write!(f, "invalid import path"),
            ParseErrorKind::InvalidParam =>
                write!(f, "function parameter must be an identifier"),
            ParseErrorKind::LiteralOverflow(s) =>
                write!(f, "literal `{s}` overflows its type"),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}] parse error: {}", self.line, self.col, self.kind)
    }
}

impl std::error::Error for ParseError {}
