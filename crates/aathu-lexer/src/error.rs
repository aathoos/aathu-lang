//! Lexer-specific error types.
//!
//! The lexer never panics or bails out early. Instead it collects `LexError`
//! values and emits an `Unknown` token so the parser can continue and report
//! as many issues as possible in a single pass.

use aathu_core::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub span: Span,
    pub line: u32,
    pub col: u32,
}

impl LexError {
    pub fn new(kind: LexErrorKind, span: Span, line: u32, col: u32) -> Self {
        Self { kind, span, line, col }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexErrorKind {
    /// A string literal was opened but never closed before EOF.
    UnterminatedString,

    /// A block comment `/* … */` was opened but never closed before EOF.
    UnterminatedBlockComment,

    /// A raw newline appeared inside a string literal.
    NewlineInString,

    /// An unrecognised escape sequence inside a string (e.g. `\q`).
    InvalidEscapeSequence(char),

    /// A numeric literal that could not be parsed into a valid number.
    /// Carries the raw text that failed.
    MalformedNumber(String),

    /// A character that does not start any valid token.
    UnexpectedCharacter(char),
}

impl std::fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexErrorKind::UnterminatedString =>
                write!(f, "unterminated string literal"),
            LexErrorKind::UnterminatedBlockComment =>
                write!(f, "unterminated block comment"),
            LexErrorKind::NewlineInString =>
                write!(f, "unexpected newline inside string literal"),
            LexErrorKind::InvalidEscapeSequence(c) =>
                write!(f, "invalid escape sequence '\\{c}'"),
            LexErrorKind::MalformedNumber(s) =>
                write!(f, "malformed numeric literal '{s}'"),
            LexErrorKind::UnexpectedCharacter(c) =>
                write!(f, "unexpected character '{c}'"),
        }
    }
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}] lex error: {}", self.line, self.col, self.kind)
    }
}

impl std::error::Error for LexError {}
