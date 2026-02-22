//! Character-level cursor used by the lexer.
//!
//! Provides single- and two-character lookahead, consuming advance with
//! automatic line / column tracking, `eat` / `eat_while` helpers, and
//! source-slice access.

pub struct Cursor<'src> {
    source: &'src str,
    /// Current byte offset into `source`.
    pos: usize,
    /// 1-based line number.
    line: u32,
    /// 1-based column (Unicode char count since last newline).
    col: u32,
}

impl<'src> Cursor<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { source, pos: 0, line: 1, col: 1 }
    }

    // -----------------------------------------------------------------------
    // Position
    // -----------------------------------------------------------------------

    #[inline] pub fn pos(&self)  -> usize { self.pos }
    #[inline] pub fn line(&self) -> u32   { self.line }
    #[inline] pub fn col(&self)  -> u32   { self.col }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    // -----------------------------------------------------------------------
    // Non-consuming lookahead
    // -----------------------------------------------------------------------

    /// Next character without consuming.
    #[inline]
    pub fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    /// Character after the next one without consuming.
    #[inline]
    pub fn peek2(&self) -> Option<char> {
        let mut it = self.source[self.pos..].chars();
        it.next()?;
        it.next()
    }

    /// `true` if the next char equals `expected` (non-consuming).
    #[inline]
    pub fn check(&self, expected: char) -> bool {
        self.peek() == Some(expected)
    }

    // -----------------------------------------------------------------------
    // Consuming
    // -----------------------------------------------------------------------

    /// Consume and return the next character, tracking line / column.
    pub fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    /// Consume the next character only if it matches `expected`.
    /// Returns `true` on a match.
    #[inline]
    pub fn eat(&mut self, expected: char) -> bool {
        if self.check(expected) { self.advance(); true } else { false }
    }

    /// Consume characters while `pred` returns `true`.
    pub fn eat_while(&mut self, pred: impl Fn(char) -> bool) {
        while self.peek().map(|c| pred(c)).unwrap_or(false) {
            self.advance();
        }
    }

    // -----------------------------------------------------------------------
    // Source slicing
    // -----------------------------------------------------------------------

    /// The source slice from byte offset `start` to the current position.
    #[inline]
    pub fn slice_from(&self, start: usize) -> &'src str {
        &self.source[start..self.pos]
    }

    /// The complete source string.
    #[inline]
    pub fn source(&self) -> &'src str {
        self.source
    }
}
