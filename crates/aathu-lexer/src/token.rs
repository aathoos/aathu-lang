//! Token definitions for the aathu lexer.
//!
//! Design notes:
//! - Dynamically typed — no type-annotation tokens needed at this stage.
//! - No `while` / `do-while`; only `for … in` loops.
//! - Arrow functions:  `fn(x) => x + 1`  or  `fn(x) { … }`.
//! - `nil` is a first-class literal value.
//! - `true` / `false` are lexed directly into `Bool` literals, not keywords.

use aathu_core::span::Span;

// ---------------------------------------------------------------------------
// Token
// ---------------------------------------------------------------------------

/// A single lexical unit with its position in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    /// Byte-offset span in the original source string.
    pub span: Span,
    /// 1-based source line number.
    pub line: u32,
    /// 1-based column (UTF-8 character count from the start of the line).
    pub col: u32,
}

impl Token {
    #[inline]
    pub fn new(kind: TokenKind, span: Span, line: u32, col: u32) -> Self {
        Self { kind, span, line, col }
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }

    #[inline]
    pub fn is_newline(&self) -> bool {
        self.kind == TokenKind::Newline
    }
}

// ---------------------------------------------------------------------------
// TokenKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // -----------------------------------------------------------------------
    // Literals
    // -----------------------------------------------------------------------

    /// Integer literal — decimal, hex (`0x…`), binary (`0b…`), octal (`0o…`).
    /// Underscores are allowed as separators: `1_000_000`.
    Int(i64),

    /// Floating-point literal: `3.14`, `0.5`, `1e10`, `2.5e-3`.
    Float(f64),

    /// String literal with escape processing: `"hello\nworld"`.
    Str(String),

    /// Boolean literal produced directly by the lexer for `true` / `false`.
    Bool(bool),

    /// Nil literal produced by the lexer for the `nil` keyword.
    Nil,

    // -----------------------------------------------------------------------
    // Identifier / keyword
    // -----------------------------------------------------------------------

    /// User-defined name: `foo`, `_bar`, `myVar`.
    Ident(String),

    /// Reserved language keyword.
    Kw(Keyword),

    // -----------------------------------------------------------------------
    // Arithmetic operators
    // -----------------------------------------------------------------------
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Percent, // %

    // -----------------------------------------------------------------------
    // Compound-assignment operators
    // -----------------------------------------------------------------------
    PlusEq,    // +=
    MinusEq,   // -=
    StarEq,    // *=
    SlashEq,   // /=
    PercentEq, // %=

    // -----------------------------------------------------------------------
    // Assignment
    // -----------------------------------------------------------------------
    Eq, // =

    // -----------------------------------------------------------------------
    // Comparison operators
    // -----------------------------------------------------------------------
    EqEq,   // ==
    BangEq, // !=
    Lt,     // <
    Gt,     // >
    LtEq,   // <=
    GtEq,   // >=

    // -----------------------------------------------------------------------
    // Logical operators (symbol form)
    // -----------------------------------------------------------------------
    And,  // &&
    Or,   // ||
    Bang, // !

    // -----------------------------------------------------------------------
    // Range operators
    // -----------------------------------------------------------------------
    DotDot,   // ..   exclusive range  0..10
    DotDotEq, // ..=  inclusive range  0..=10

    // -----------------------------------------------------------------------
    // Arrows
    // -----------------------------------------------------------------------
    Arrow,    // ->   (reserved for return-type hint)
    FatArrow, // =>   (arrow-function body / match arm)

    // -----------------------------------------------------------------------
    // Delimiters
    // -----------------------------------------------------------------------
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    // -----------------------------------------------------------------------
    // Punctuation
    // -----------------------------------------------------------------------
    Comma,    // ,
    Colon,    // :
    Semi,     // ;
    Dot,      // .   member access: `obj.field`

    // -----------------------------------------------------------------------
    // Special
    // -----------------------------------------------------------------------

    /// Significant newline — used for automatic statement termination.
    Newline,

    /// End-of-file sentinel; always the last token in the stream.
    Eof,

    /// Unrecognised character — emitted for error recovery so the parser can
    /// continue and collect as many errors as possible in one pass.
    Unknown(char),
}

// ---------------------------------------------------------------------------
// Keywords
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Keyword {
    // Variable & function
    Let,
    Fn,

    // Control flow
    If,
    Else,
    For,
    In,
    Return,
    Break,
    Continue,

    // Pattern matching
    Match,

    // Modules
    Import,
    Export,
    Pub,
    Mod,
    As,

    // Type definitions (reserved for later phases)
    Struct,
    Enum,
    Impl,
    Trait,
    Type,

    // Concurrency
    Async,
    Await,
    Spawn,

    // Keyword operators (English aliases for symbol operators)
    And,  // `and`  ≡  &&
    Or,   // `or`   ≡  ||
    Not,  // `not`  ≡  !
    Is,   // `is`   — type-check placeholder
}

impl Keyword {
    /// Try to convert a raw identifier string to a keyword.
    /// Returns `None` for non-keyword strings.
    pub fn from_str(s: &str) -> Option<Keyword> {
        Some(match s {
            "let"      => Keyword::Let,
            "fn"       => Keyword::Fn,
            "if"       => Keyword::If,
            "else"     => Keyword::Else,
            "for"      => Keyword::For,
            "in"       => Keyword::In,
            "return"   => Keyword::Return,
            "break"    => Keyword::Break,
            "continue" => Keyword::Continue,
            "match"    => Keyword::Match,
            "import"   => Keyword::Import,
            "export"   => Keyword::Export,
            "pub"      => Keyword::Pub,
            "mod"      => Keyword::Mod,
            "as"       => Keyword::As,
            "struct"   => Keyword::Struct,
            "enum"     => Keyword::Enum,
            "impl"     => Keyword::Impl,
            "trait"    => Keyword::Trait,
            "type"     => Keyword::Type,
            "async"    => Keyword::Async,
            "await"    => Keyword::Await,
            "spawn"    => Keyword::Spawn,
            "and"      => Keyword::And,
            "or"       => Keyword::Or,
            "not"      => Keyword::Not,
            "is"       => Keyword::Is,
            _          => return None,
        })
    }

    /// Returns the canonical source spelling of this keyword.
    pub fn as_str(&self) -> &'static str {
        match self {
            Keyword::Let      => "let",
            Keyword::Fn       => "fn",
            Keyword::If       => "if",
            Keyword::Else     => "else",
            Keyword::For      => "for",
            Keyword::In       => "in",
            Keyword::Return   => "return",
            Keyword::Break    => "break",
            Keyword::Continue => "continue",
            Keyword::Match    => "match",
            Keyword::Import   => "import",
            Keyword::Export   => "export",
            Keyword::Pub      => "pub",
            Keyword::Mod      => "mod",
            Keyword::As       => "as",
            Keyword::Struct   => "struct",
            Keyword::Enum     => "enum",
            Keyword::Impl     => "impl",
            Keyword::Trait    => "trait",
            Keyword::Type     => "type",
            Keyword::Async    => "async",
            Keyword::Await    => "await",
            Keyword::Spawn    => "spawn",
            Keyword::And      => "and",
            Keyword::Or       => "or",
            Keyword::Not      => "not",
            Keyword::Is       => "is",
        }
    }
}

// ---------------------------------------------------------------------------
// Display impls
// ---------------------------------------------------------------------------

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Int(n)     => write!(f, "{n}"),
            TokenKind::Float(n)   => write!(f, "{n}"),
            TokenKind::Str(s)     => write!(f, "\"{s}\""),
            TokenKind::Bool(b)    => write!(f, "{b}"),
            TokenKind::Nil        => write!(f, "nil"),
            TokenKind::Ident(s)   => write!(f, "{s}"),
            TokenKind::Kw(kw)     => write!(f, "{kw}"),
            TokenKind::Plus       => write!(f, "+"),
            TokenKind::Minus      => write!(f, "-"),
            TokenKind::Star       => write!(f, "*"),
            TokenKind::Slash      => write!(f, "/"),
            TokenKind::Percent    => write!(f, "%"),
            TokenKind::PlusEq     => write!(f, "+="),
            TokenKind::MinusEq    => write!(f, "-="),
            TokenKind::StarEq     => write!(f, "*="),
            TokenKind::SlashEq    => write!(f, "/="),
            TokenKind::PercentEq  => write!(f, "%="),
            TokenKind::Eq         => write!(f, "="),
            TokenKind::EqEq       => write!(f, "=="),
            TokenKind::BangEq     => write!(f, "!="),
            TokenKind::Lt         => write!(f, "<"),
            TokenKind::Gt         => write!(f, ">"),
            TokenKind::LtEq       => write!(f, "<="),
            TokenKind::GtEq       => write!(f, ">="),
            TokenKind::And        => write!(f, "&&"),
            TokenKind::Or         => write!(f, "||"),
            TokenKind::Bang       => write!(f, "!"),
            TokenKind::DotDot     => write!(f, ".."),
            TokenKind::DotDotEq   => write!(f, "..="),
            TokenKind::Arrow      => write!(f, "->"),
            TokenKind::FatArrow   => write!(f, "=>"),
            TokenKind::LParen     => write!(f, "("),
            TokenKind::RParen     => write!(f, ")"),
            TokenKind::LBrace     => write!(f, "{{"),
            TokenKind::RBrace     => write!(f, "}}"),
            TokenKind::LBracket   => write!(f, "["),
            TokenKind::RBracket   => write!(f, "]"),
            TokenKind::Comma      => write!(f, ","),
            TokenKind::Colon      => write!(f, ":"),
            TokenKind::Semi       => write!(f, ";"),
            TokenKind::Dot        => write!(f, "."),
            TokenKind::Newline    => write!(f, "<newline>"),
            TokenKind::Eof        => write!(f, "<eof>"),
            TokenKind::Unknown(c) => write!(f, "<unknown {c:?}>"),
        }
    }
}
