//! The aathu lexer — full tokenisation pass.
//!
//! [`Lexer::tokenize`] consumes the source string and returns a
//! `(Vec<Token>, Vec<LexError>)` pair.  It never panics or returns early:
//! unrecognised characters produce an [`TokenKind::Unknown`] token and a
//! matching [`LexError`] so the parser can keep going and accumulate as many
//! diagnostics as possible in a single pass.

use aathu_core::span::Span;

use crate::{
    cursor::Cursor,
    error::{LexError, LexErrorKind},
    token::{Keyword, Token, TokenKind},
};

// ---------------------------------------------------------------------------
// Lexer
// ---------------------------------------------------------------------------

pub struct Lexer<'src> {
    cur: Cursor<'src>,
    tokens: Vec<Token>,
    errors: Vec<LexError>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            cur: Cursor::new(source),
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Tokenise the entire source string.
    ///
    /// Returns `(tokens, errors)`. The token stream always ends with a
    /// [`TokenKind::Eof`] sentinel. Errors are non-fatal — every error site
    /// also produces an [`TokenKind::Unknown`] (or a partial) token so the
    /// parser can continue.
    pub fn tokenize(mut self) -> (Vec<Token>, Vec<LexError>) {
        loop {
            let tok = self.next_token();
            let done = tok.kind == TokenKind::Eof;
            self.tokens.push(tok);
            if done {
                break;
            }
        }
        (self.tokens, self.errors)
    }

    // -----------------------------------------------------------------------
    // Core scan loop
    // -----------------------------------------------------------------------

    fn next_token(&mut self) -> Token {
        // Skip horizontal whitespace. Newlines are significant.
        loop {
            match self.cur.peek() {
                Some(' ' | '\t' | '\r') => {
                    self.cur.advance();
                }
                _ => break,
            }
        }

        let start = self.cur.pos();
        let line  = self.cur.line();
        let col   = self.cur.col();

        let ch = match self.cur.advance() {
            None    => return self.make(TokenKind::Eof, start, line, col),
            Some(c) => c,
        };

        let kind = match ch {
            // -------------------------------------------------------------------
            // Newline — significant for automatic statement termination
            // -------------------------------------------------------------------
            '\n' => TokenKind::Newline,

            // -------------------------------------------------------------------
            // Comments
            // -------------------------------------------------------------------
            '/' => match self.cur.peek() {
                Some('/') => {
                    self.cur.advance(); // consume second '/'
                    self.cur.eat_while(|c| c != '\n');
                    return self.next_token();
                }
                Some('*') => {
                    self.cur.advance(); // consume '*'
                    self.scan_block_comment(start, line, col);
                    return self.next_token();
                }
                Some('=') => {
                    self.cur.advance();
                    TokenKind::SlashEq
                }
                _ => TokenKind::Slash,
            },

            // -------------------------------------------------------------------
            // String literal
            // -------------------------------------------------------------------
            '"' => self.scan_string(start, line, col),

            // -------------------------------------------------------------------
            // Numbers
            // -------------------------------------------------------------------
            '0'..='9' => self.scan_number(ch, start, line, col),

            // -------------------------------------------------------------------
            // Identifiers, keywords, booleans, nil
            // -------------------------------------------------------------------
            c if is_ident_start(c) => self.scan_ident(start, line, col),

            // -------------------------------------------------------------------
            // Operators — longest match first
            // -------------------------------------------------------------------

            // Arithmetic / compound-assign
            '+' => {
                if self.cur.eat('=') { TokenKind::PlusEq } else { TokenKind::Plus }
            }
            '-' => match self.cur.peek() {
                Some('=') => { self.cur.advance(); TokenKind::MinusEq }
                Some('>') => { self.cur.advance(); TokenKind::Arrow    }
                _         => TokenKind::Minus,
            },
            '*' => {
                if self.cur.eat('=') { TokenKind::StarEq } else { TokenKind::Star }
            }
            '%' => {
                if self.cur.eat('=') { TokenKind::PercentEq } else { TokenKind::Percent }
            }

            // Assignment / equality / fat-arrow
            '=' => match self.cur.peek() {
                Some('=') => { self.cur.advance(); TokenKind::EqEq     }
                Some('>') => { self.cur.advance(); TokenKind::FatArrow  }
                _         => TokenKind::Eq,
            },

            // Logical
            '!' => {
                if self.cur.eat('=') { TokenKind::BangEq } else { TokenKind::Bang }
            }
            '&' => {
                if self.cur.eat('&') {
                    TokenKind::And
                } else {
                    self.push_error(LexErrorKind::UnexpectedCharacter('&'), start, line, col);
                    TokenKind::Unknown('&')
                }
            }
            '|' => {
                if self.cur.eat('|') {
                    TokenKind::Or
                } else {
                    self.push_error(LexErrorKind::UnexpectedCharacter('|'), start, line, col);
                    TokenKind::Unknown('|')
                }
            }

            // Comparison
            '<' => {
                if self.cur.eat('=') { TokenKind::LtEq } else { TokenKind::Lt }
            }
            '>' => {
                if self.cur.eat('=') { TokenKind::GtEq } else { TokenKind::Gt }
            }

            // Range / dot / member-access
            '.' => match self.cur.peek() {
                Some('.') => {
                    self.cur.advance(); // consume second '.'
                    if self.cur.eat('=') { TokenKind::DotDotEq } else { TokenKind::DotDot }
                }
                _ => TokenKind::Dot,
            },

            // -------------------------------------------------------------------
            // Delimiters
            // -------------------------------------------------------------------
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,

            // -------------------------------------------------------------------
            // Punctuation
            // -------------------------------------------------------------------
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semi,

            // -------------------------------------------------------------------
            // Unknown / error recovery
            // -------------------------------------------------------------------
            c => {
                self.push_error(LexErrorKind::UnexpectedCharacter(c), start, line, col);
                TokenKind::Unknown(c)
            }
        };

        self.make(kind, start, line, col)
    }

    // -----------------------------------------------------------------------
    // Block comment   /* … */   (supports nesting)
    // -----------------------------------------------------------------------

    fn scan_block_comment(&mut self, start: usize, line: u32, col: u32) {
        let mut depth: usize = 1;
        loop {
            match (self.cur.peek(), self.cur.peek2()) {
                (None, _) => {
                    self.push_error(LexErrorKind::UnterminatedBlockComment, start, line, col);
                    return;
                }
                (Some('/'), Some('*')) => {
                    self.cur.advance();
                    self.cur.advance();
                    depth += 1;
                }
                (Some('*'), Some('/')) => {
                    self.cur.advance();
                    self.cur.advance();
                    depth -= 1;
                    if depth == 0 {
                        return;
                    }
                }
                _ => {
                    self.cur.advance();
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // String literal   "…"
    // -----------------------------------------------------------------------

    fn scan_string(&mut self, start: usize, line: u32, col: u32) -> TokenKind {
        let mut value = String::new();

        loop {
            match self.cur.peek() {
                None => {
                    self.push_error(LexErrorKind::UnterminatedString, start, line, col);
                    return TokenKind::Str(value);
                }
                Some('\n') => {
                    self.push_error(LexErrorKind::NewlineInString, start, line, col);
                    // Do NOT consume the newline — it's a significant token.
                    return TokenKind::Str(value);
                }
                Some('"') => {
                    self.cur.advance(); // consume closing '"'
                    return TokenKind::Str(value);
                }
                Some('\\') => {
                    self.cur.advance(); // consume '\'
                    match self.cur.peek() {
                        None => {
                            self.push_error(LexErrorKind::UnterminatedString, start, line, col);
                            return TokenKind::Str(value);
                        }
                        Some(esc) => {
                            self.cur.advance();
                            match esc {
                                'n'  => value.push('\n'),
                                't'  => value.push('\t'),
                                'r'  => value.push('\r'),
                                '"'  => value.push('"'),
                                '\\' => value.push('\\'),
                                '0'  => value.push('\0'),
                                'u'  => {
                                    if let Some(c) = self.scan_unicode_escape(start, line, col) {
                                        value.push(c);
                                    }
                                }
                                c => {
                                    self.push_error(
                                        LexErrorKind::InvalidEscapeSequence(c),
                                        start, line, col,
                                    );
                                    // Emit the raw pair so output isn't totally garbled.
                                    value.push('\\');
                                    value.push(c);
                                }
                            }
                        }
                    }
                }
                Some(c) => {
                    value.push(c);
                    self.cur.advance();
                }
            }
        }
    }

    /// Parse `\u{HHHH}` — called after `\u` has been consumed.
    fn scan_unicode_escape(&mut self, start: usize, line: u32, col: u32) -> Option<char> {
        if !self.cur.eat('{') {
            self.push_error(LexErrorKind::InvalidEscapeSequence('u'), start, line, col);
            return None;
        }
        let hex_start = self.cur.pos();
        self.cur.eat_while(|c| c.is_ascii_hexdigit());
        let hex_str = self.cur.slice_from(hex_start);
        if !self.cur.eat('}') {
            self.push_error(LexErrorKind::InvalidEscapeSequence('u'), start, line, col);
            return None;
        }
        match u32::from_str_radix(hex_str, 16).ok().and_then(char::from_u32) {
            Some(c) => Some(c),
            None => {
                self.push_error(LexErrorKind::InvalidEscapeSequence('u'), start, line, col);
                None
            }
        }
    }

    // -----------------------------------------------------------------------
    // Number literals
    // -----------------------------------------------------------------------

    fn scan_number(&mut self, first: char, start: usize, line: u32, col: u32) -> TokenKind {
        // Detect base prefix: 0x, 0b, 0o
        if first == '0' {
            match self.cur.peek() {
                Some('x' | 'X') => {
                    self.cur.advance();
                    return self.scan_radix_int(16, start, line, col);
                }
                Some('b' | 'B') => {
                    self.cur.advance();
                    return self.scan_radix_int(2, start, line, col);
                }
                Some('o' | 'O') => {
                    self.cur.advance();
                    return self.scan_radix_int(8, start, line, col);
                }
                _ => {} // fall through to decimal / float
            }
        }

        // Decimal integer or float.
        self.cur.eat_while(|c| c.is_ascii_digit() || c == '_');

        // Detect float: digits `.` digits
        // Crucially, do NOT consume `..` for range operators (`0..10`).
        let is_float = self.cur.peek() == Some('.')
            && self.cur.peek2().map(|c| c.is_ascii_digit()).unwrap_or(false);

        if is_float {
            self.cur.advance(); // consume '.'
            self.cur.eat_while(|c| c.is_ascii_digit() || c == '_');

            // Optional exponent: e / E  [+ / -]  digits
            if matches!(self.cur.peek(), Some('e' | 'E')) {
                self.cur.advance();
                if matches!(self.cur.peek(), Some('+' | '-')) {
                    self.cur.advance();
                }
                self.cur.eat_while(|c| c.is_ascii_digit() || c == '_');
            }

            let raw   = self.cur.slice_from(start);
            let clean = raw.replace('_', "");
            match clean.parse::<f64>() {
                Ok(v)  => TokenKind::Float(v),
                Err(_) => {
                    self.push_error(LexErrorKind::MalformedNumber(raw.to_string()), start, line, col);
                    TokenKind::Float(0.0)
                }
            }
        } else {
            let raw   = self.cur.slice_from(start);
            let clean = raw.replace('_', "");
            match clean.parse::<i64>() {
                Ok(v)  => TokenKind::Int(v),
                Err(_) => {
                    self.push_error(LexErrorKind::MalformedNumber(raw.to_string()), start, line, col);
                    TokenKind::Int(0)
                }
            }
        }
    }

    /// Scan a non-decimal integer literal (hex, binary, or octal).
    /// Called after the prefix (`0x`, `0b`, `0o`) has already been consumed.
    fn scan_radix_int(&mut self, radix: u32, start: usize, line: u32, col: u32) -> TokenKind {
        let digit_start = self.cur.pos();
        self.cur.eat_while(|c| c == '_' || c.is_ascii_alphanumeric());
        let raw   = self.cur.slice_from(digit_start);
        let clean = raw.replace('_', "");
        match i64::from_str_radix(&clean, radix) {
            Ok(v)  => TokenKind::Int(v),
            Err(_) => {
                let full = self.cur.slice_from(start);
                self.push_error(LexErrorKind::MalformedNumber(full.to_string()), start, line, col);
                TokenKind::Int(0)
            }
        }
    }

    // -----------------------------------------------------------------------
    // Identifiers, keywords, `true`, `false`, `nil`
    // -----------------------------------------------------------------------

    /// Called after the first character of the identifier has been consumed
    /// (it was matched by `is_ident_start` in the dispatch match, but NOT yet
    /// advanced — we pass `start` pointing before it).
    fn scan_ident(&mut self, start: usize, _line: u32, _col: u32) -> TokenKind {
        // The first character was already consumed by `self.cur.advance()` in
        // `next_token`, so we only need to consume the *rest*.
        self.cur.eat_while(is_ident_continue);
        let text = self.cur.slice_from(start);

        match text {
            "true"  => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            "nil"   => TokenKind::Nil,
            _ => {
                if let Some(kw) = Keyword::from_str(text) {
                    TokenKind::Kw(kw)
                } else {
                    TokenKind::Ident(text.to_string())
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    #[inline]
    fn make(&self, kind: TokenKind, start: usize, line: u32, col: u32) -> Token {
        Token::new(kind, Span::new(start, self.cur.pos()), line, col)
    }

    #[inline]
    fn push_error(&mut self, kind: LexErrorKind, start: usize, line: u32, col: u32) {
        self.errors.push(LexError::new(
            kind,
            Span::new(start, self.cur.pos()),
            line,
            col,
        ));
    }
}

// ---------------------------------------------------------------------------
// Character predicates
// ---------------------------------------------------------------------------

/// Characters that can *start* an identifier or keyword.
#[inline]
fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

/// Characters that can *continue* an identifier or keyword.
#[inline]
fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
