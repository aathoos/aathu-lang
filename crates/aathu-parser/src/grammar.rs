//! Recursive-descent + Pratt expression parser for aathu.
//!
//! ## Statement termination
//! Newlines act as statement separators (like Go / Python).  Inside matched
//! delimiters `(…)`, `[…]`, `{…}` and after infix operators, newlines are
//! explicitly skipped so multi-token constructs can span lines freely.
//!
//! ## Expression precedence (Pratt binding powers, left / right)
//! ```text
//! = += -= *= /= %=     →  2 / 1   (right-assoc)
//! || or                →  4 / 5
//! && and               →  6 / 7
//! == != is             →  8 / 9
//! < > <= >=            → 10 / 11
//! .. ..=               → 12 / 13
//! + -                  → 14 / 15
//! * / %                → 16 / 17
//! unary - ! not        → prefix 19
//! call () [] .         → 20 (postfix, handled in infix loop)
//! ```

use aathu_core::span::Span;
use aathu_lexer::token::{Keyword, Token, TokenKind};

use crate::{
    ast::*,
    error::{ParseError, ParseErrorKind},
};

type PResult<T> = Result<T, ParseError>;

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Build a parser from a token stream (including `Newline` and `Eof`).
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // -----------------------------------------------------------------------
    // Entry point
    // -----------------------------------------------------------------------

    pub fn parse_program(mut self) -> PResult<Program> {
        let start = self.span().start;
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.is_eof() {
            stmts.push(self.parse_stmt()?);
            self.eat_stmt_end();
        }
        let end = self.span().end;
        Ok(Program { stmts, span: Span::new(start, end) })
    }

    // -----------------------------------------------------------------------
    // Token stream navigation
    // -----------------------------------------------------------------------

    /// Peek at the raw next token (includes `Newline`).
    #[inline]
    fn peek(&self) -> &TokenKind {
        self.tokens.get(self.pos).map(|t| &t.kind).unwrap_or(&TokenKind::Eof)
    }

    /// Peek past any number of newlines (for infix lookahead after operators).
    fn peek_skip_nl(&self) -> &TokenKind {
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::Newline => i += 1,
                k => return k,
            }
        }
        &TokenKind::Eof
    }

    /// Current token's span (or a zero-width span at EOF).
    fn span(&self) -> Span {
        self.tokens.get(self.pos).map(|t| t.span).unwrap_or(Span::new(0, 0))
    }

    /// The span of the most-recently consumed token.
    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::new(0, 0)
        }
    }

    fn line(&self) -> u32 {
        self.tokens.get(self.pos).map(|t| t.line).unwrap_or(1)
    }

    fn col(&self) -> u32 {
        self.tokens.get(self.pos).map(|t| t.col).unwrap_or(1)
    }

    /// Consume and return the current token.
    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or_else(|| Token {
            kind: TokenKind::Eof,
            span: Span::new(0, 0),
            line: 0,
            col: 0,
        });
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek() == kind
    }

    fn check_kw(&self, kw: Keyword) -> bool {
        self.peek() == &TokenKind::Kw(kw)
    }

    /// Consume the current token if it matches `kind`. Returns true on match.
    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.peek() == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_kw(&mut self, kw: Keyword) -> bool {
        self.eat(&TokenKind::Kw(kw))
    }

    /// Consume the current token; error if it doesn't match `kind`.
    fn expect(&mut self, kind: &TokenKind) -> PResult<Token> {
        if self.peek() == kind {
            Ok(self.advance())
        } else {
            let found = format!("{}", self.peek());
            let expected = format!("`{kind}`");
            Err(self.err(ParseErrorKind::UnexpectedToken { expected, found }))
        }
    }

    fn expect_kw(&mut self, kw: Keyword) -> PResult<Token> {
        self.expect(&TokenKind::Kw(kw))
    }

    /// Consume the next token as an identifier, return the name string.
    fn expect_ident(&mut self) -> PResult<String> {
        match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.advance();
                Ok(name)
            }
            _ => {
                let found = format!("{}", self.peek());
                Err(self.err(ParseErrorKind::UnexpectedToken {
                    expected: "identifier".into(),
                    found,
                }))
            }
        }
    }

    /// Skip all consecutive `Newline` tokens.
    fn skip_newlines(&mut self) {
        while self.check(&TokenKind::Newline) {
            self.advance();
        }
    }

    /// Consume trailing newlines / semicolons after a statement.
    fn eat_stmt_end(&mut self) {
        loop {
            match self.peek() {
                TokenKind::Newline | TokenKind::Semi => { self.advance(); }
                _ => break,
            }
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    /// Build an error at the current position.
    fn err(&self, kind: ParseErrorKind) -> ParseError {
        ParseError::new(kind, self.span(), self.line(), self.col())
    }

    fn unexpected(&self, expected: &str) -> ParseError {
        if self.is_eof() {
            self.err(ParseErrorKind::UnexpectedEof { expected: expected.into() })
        } else {
            let found = format!("{}", self.peek());
            self.err(ParseErrorKind::UnexpectedToken {
                expected: expected.into(),
                found,
            })
        }
    }

    // -----------------------------------------------------------------------
    // Statements
    // -----------------------------------------------------------------------

    fn parse_stmt(&mut self) -> PResult<Stmt> {
        let start = self.span().start;
        let line  = self.line();
        let col   = self.col();

        // Visibility prefix
        let public = self.eat_kw(Keyword::Pub);

        let kind = match self.peek().clone() {
            TokenKind::Kw(Keyword::Let)    => self.parse_let(public)?,

            // `fn name(…)` → named function declaration.
            // `fn(…)`      → lambda expression-statement (no name follows).
            TokenKind::Kw(Keyword::Fn) => {
                let next = self.tokens.get(self.pos + 1).map(|t| &t.kind);
                if matches!(next, Some(TokenKind::Ident(_))) {
                    self.parse_fn(public)?
                } else {
                    if public { return Err(self.unexpected("declaration after `pub`")); }
                    StmtKind::Expr(self.parse_expr()?)
                }
            }

            // `async fn name(…)` — `async` is consumed; stub (not yet stored).
            TokenKind::Kw(Keyword::Async) => {
                self.advance(); // consume `async`
                self.expect_kw(Keyword::Fn)?;
                let name   = self.expect_ident()?;
                let params = self.parse_param_list()?;
                let body   = self.parse_block()?;
                StmtKind::Fn { public, name, params, body }
            }

            TokenKind::Kw(Keyword::Struct) => self.parse_struct(public)?,
            TokenKind::Kw(Keyword::Enum)   => self.parse_enum(public)?,
            TokenKind::Kw(Keyword::Impl)   => {
                if public {
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedToken {
                            expected: "declaration".into(),
                            found: "pub impl".into(),
                        },
                        Span::new(start, self.span().end),
                        line, col,
                    ));
                }
                self.parse_impl()?
            }
            TokenKind::Kw(Keyword::Import) => {
                if public { return Err(self.unexpected("declaration after `pub`")); }
                self.parse_import()?
            }
            TokenKind::Kw(Keyword::Export) => {
                if public { return Err(self.unexpected("declaration after `pub`")); }
                self.parse_export()?
            }
            TokenKind::Kw(Keyword::Return)   => self.parse_return(public)?,
            TokenKind::Kw(Keyword::Break)    => {
                self.advance(); StmtKind::Break
            }
            TokenKind::Kw(Keyword::Continue) => {
                self.advance(); StmtKind::Continue
            }
            TokenKind::Kw(Keyword::If)    => self.parse_if_stmt(public)?,
            TokenKind::Kw(Keyword::For)   => self.parse_for(public)?,
            TokenKind::Kw(Keyword::Match) => self.parse_match_stmt(public)?,
            _ => {
                if public {
                    return Err(self.unexpected("declaration after `pub`"));
                }
                let expr = self.parse_expr()?;
                StmtKind::Expr(expr)
            }
        };

        Ok(Stmt { kind, span: Span::new(start, self.prev_span().end) })
    }

    // let name [= expr]
    fn parse_let(&mut self, _public: bool) -> PResult<StmtKind> {
        self.advance(); // consume `let`
        let name = self.expect_ident()?;
        let init = if self.eat(&TokenKind::Eq) {
            self.skip_newlines();
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(StmtKind::Let { name, init })
    }

    // fn name(params) { body }   — named function statement
    fn parse_fn(&mut self, public: bool) -> PResult<StmtKind> {
        self.advance(); // consume `fn`
        let name   = self.expect_ident()?;
        let params = self.parse_param_list()?;
        let body   = self.parse_block()?;
        Ok(StmtKind::Fn { public, name, params, body })
    }

    // struct Name { field, … }
    fn parse_struct(&mut self, public: bool) -> PResult<StmtKind> {
        self.advance(); // consume `struct`
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        self.skip_newlines();
        while !self.check(&TokenKind::RBrace) && !self.is_eof() {
            fields.push(self.expect_ident()?);
            self.skip_newlines();
            if !self.eat(&TokenKind::Comma) {
                self.skip_newlines();
                break;
            }
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(StmtKind::Struct { public, name, fields })
    }

    // enum Name { Variant, … }
    fn parse_enum(&mut self, public: bool) -> PResult<StmtKind> {
        self.advance(); // consume `enum`
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut variants = Vec::new();
        self.skip_newlines();
        while !self.check(&TokenKind::RBrace) && !self.is_eof() {
            let vstart = self.span().start;
            let vname  = self.expect_ident()?;
            variants.push(EnumVariant {
                name: vname,
                span: Span::new(vstart, self.prev_span().end),
            });
            self.skip_newlines();
            if !self.eat(&TokenKind::Comma) {
                self.skip_newlines();
                break;
            }
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(StmtKind::Enum { public, name, variants })
    }

    // impl Name { stmt* }
    fn parse_impl(&mut self) -> PResult<StmtKind> {
        self.advance(); // consume `impl`
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut body = Vec::new();
        self.skip_newlines();
        while !self.check(&TokenKind::RBrace) && !self.is_eof() {
            body.push(self.parse_stmt()?);
            self.eat_stmt_end();
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(StmtKind::Impl { name, body })
    }

    // import seg::seg [as alias]
    fn parse_import(&mut self) -> PResult<StmtKind> {
        self.advance(); // consume `import`
        let mut path = Vec::new();
        path.push(self.expect_ident()?);
        while self.eat(&TokenKind::Colon) {
            // Accept `::` — two colons in sequence
            self.expect(&TokenKind::Colon)?;
            path.push(self.expect_ident()?);
        }
        // Also accept dot-separated: `import foo.bar`
        while self.eat(&TokenKind::Dot) {
            path.push(self.expect_ident()?);
        }
        let alias = if self.eat_kw(Keyword::As) {
            Some(self.expect_ident()?)
        } else {
            None
        };
        Ok(StmtKind::Import { path, alias })
    }

    // export name
    fn parse_export(&mut self) -> PResult<StmtKind> {
        self.advance(); // consume `export`
        let name = self.expect_ident()?;
        Ok(StmtKind::Export { name })
    }

    // return [expr]
    fn parse_return(&mut self, _public: bool) -> PResult<StmtKind> {
        self.advance(); // consume `return`
        // If the next token (raw, including newlines) terminates the statement,
        // there's no return value.
        let value = match self.peek() {
            TokenKind::Newline | TokenKind::Semi | TokenKind::RBrace | TokenKind::Eof => None,
            _ => Some(self.parse_expr()?),
        };
        Ok(StmtKind::Return(value))
    }

    // if cond { then } [else { … } | else if …]
    fn parse_if_stmt(&mut self, _public: bool) -> PResult<StmtKind> {
        self.advance(); // consume `if`
        self.skip_newlines();
        let cond = self.parse_expr()?;
        let then = self.parse_block()?;
        let else_ = if self.eat_kw(Keyword::Else) {
            self.skip_newlines();
            if self.check_kw(Keyword::If) {
                // else if — parse a fresh if-stmt and wrap it
                let inner = self.parse_stmt()?;
                Some(ElseBranch::If(Box::new(inner)))
            } else {
                Some(ElseBranch::Block(self.parse_block()?))
            }
        } else {
            None
        };
        Ok(StmtKind::If { cond, then, else_ })
    }

    // for var in iter { body }
    fn parse_for(&mut self, _public: bool) -> PResult<StmtKind> {
        self.advance(); // consume `for`
        let var = self.expect_ident()?;
        self.expect_kw(Keyword::In)?;
        self.skip_newlines();
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(StmtKind::For { var, iter, body })
    }

    // match scrutinee { pat => expr, … }
    fn parse_match_stmt(&mut self, _public: bool) -> PResult<StmtKind> {
        let (scrutinee, arms) = self.parse_match_inner()?;
        Ok(StmtKind::Match { scrutinee, arms })
    }

    // -----------------------------------------------------------------------
    // Block
    // -----------------------------------------------------------------------

    fn parse_block(&mut self) -> PResult<Block> {
        let start = self.span().start;
        self.expect(&TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.check(&TokenKind::RBrace) && !self.is_eof() {
            stmts.push(self.parse_stmt()?);
            self.eat_stmt_end();
        }
        let end_tok = self.expect(&TokenKind::RBrace)?;
        Ok(Block { stmts, span: Span::new(start, end_tok.span.end) })
    }

    // -----------------------------------------------------------------------
    // Parameter list  (params)
    // -----------------------------------------------------------------------

    fn parse_param_list(&mut self) -> PResult<Vec<String>> {
        self.expect(&TokenKind::LParen)?;
        let mut params = Vec::new();
        self.skip_newlines();
        while !self.check(&TokenKind::RParen) && !self.is_eof() {
            match self.peek().clone() {
                TokenKind::Ident(name) => {
                    self.advance();
                    params.push(name);
                }
                _ => {
                    return Err(self.err(ParseErrorKind::InvalidParam));
                }
            }
            self.skip_newlines();
            if !self.eat(&TokenKind::Comma) {
                break;
            }
            self.skip_newlines();
        }
        self.expect(&TokenKind::RParen)?;
        Ok(params)
    }

    // -----------------------------------------------------------------------
    // Expressions — Pratt parser
    // -----------------------------------------------------------------------

    pub fn parse_expr(&mut self) -> PResult<Expr> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> PResult<Expr> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Determine if there's an infix/postfix operator next.
            // For postfix ops (`.`, `(`, `[`), they must be on the SAME token
            // position (no newline gap) — preserves statement separation.
            // For binary infix ops, we skip newlines so expressions can span
            // lines after operators.
            let next = self.peek();
            let (l_bp, r_bp) = match infix_bp(next) {
                Some(bp) => bp,
                None => {
                    // Skip newlines and retry ONLY for non-postfix infix ops.
                    // Postfix (., (, [) do NOT get this treatment.
                    let next_skip = self.peek_skip_nl();
                    match infix_bp(next_skip) {
                        Some(bp) if !is_postfix_op(next_skip) => {
                            self.skip_newlines();
                            bp
                        }
                        _ => break,
                    }
                }
            };

            if l_bp <= min_bp {
                break;
            }

            let op_tok = self.advance();
            let lhs_start = lhs.span.start;

            lhs = match op_tok.kind.clone() {
                // --- Postfix: call ---
                TokenKind::LParen => {
                    let args = self.parse_arg_list()?;
                    let span = Span::new(lhs_start, self.prev_span().end);
                    Expr { kind: ExprKind::Call { callee: Box::new(lhs), args }, span }
                }

                // --- Postfix: index ---
                TokenKind::LBracket => {
                    self.skip_newlines();
                    let index = self.parse_expr_bp(0)?;
                    self.skip_newlines();
                    self.expect(&TokenKind::RBracket)?;
                    let span = Span::new(lhs_start, self.prev_span().end);
                    Expr {
                        kind: ExprKind::Index { object: Box::new(lhs), index: Box::new(index) },
                        span,
                    }
                }

                // --- Postfix: member ---
                TokenKind::Dot => {
                    let field = self.expect_ident()?;
                    let span  = Span::new(lhs_start, self.prev_span().end);
                    Expr {
                        kind: ExprKind::Member { object: Box::new(lhs), field },
                        span,
                    }
                }

                // --- Range ---
                TokenKind::DotDot => {
                    self.skip_newlines();
                    let rhs  = self.parse_expr_bp(r_bp)?;
                    let span = Span::new(lhs_start, rhs.span.end);
                    Expr {
                        kind: ExprKind::Range { lo: Box::new(lhs), hi: Box::new(rhs), inclusive: false },
                        span,
                    }
                }
                TokenKind::DotDotEq => {
                    self.skip_newlines();
                    let rhs  = self.parse_expr_bp(r_bp)?;
                    let span = Span::new(lhs_start, rhs.span.end);
                    Expr {
                        kind: ExprKind::Range { lo: Box::new(lhs), hi: Box::new(rhs), inclusive: true },
                        span,
                    }
                }

                // --- Assignment ---
                TokenKind::Eq => {
                    validate_assign_target(&lhs)?;
                    self.skip_newlines();
                    let rhs  = self.parse_expr_bp(r_bp)?;
                    let span = Span::new(lhs_start, rhs.span.end);
                    Expr {
                        kind: ExprKind::Assign { target: Box::new(lhs), op: None, value: Box::new(rhs) },
                        span,
                    }
                }
                TokenKind::PlusEq => {
                    validate_assign_target(&lhs)?;
                    self.skip_newlines();
                    let rhs  = self.parse_expr_bp(r_bp)?;
                    let span = Span::new(lhs_start, rhs.span.end);
                    Expr {
                        kind: ExprKind::Assign { target: Box::new(lhs), op: Some(AssignOp::Add), value: Box::new(rhs) },
                        span,
                    }
                }
                TokenKind::MinusEq => {
                    validate_assign_target(&lhs)?;
                    self.skip_newlines();
                    let rhs  = self.parse_expr_bp(r_bp)?;
                    let span = Span::new(lhs_start, rhs.span.end);
                    Expr {
                        kind: ExprKind::Assign { target: Box::new(lhs), op: Some(AssignOp::Sub), value: Box::new(rhs) },
                        span,
                    }
                }
                TokenKind::StarEq => {
                    validate_assign_target(&lhs)?;
                    self.skip_newlines();
                    let rhs  = self.parse_expr_bp(r_bp)?;
                    let span = Span::new(lhs_start, rhs.span.end);
                    Expr {
                        kind: ExprKind::Assign { target: Box::new(lhs), op: Some(AssignOp::Mul), value: Box::new(rhs) },
                        span,
                    }
                }
                TokenKind::SlashEq => {
                    validate_assign_target(&lhs)?;
                    self.skip_newlines();
                    let rhs  = self.parse_expr_bp(r_bp)?;
                    let span = Span::new(lhs_start, rhs.span.end);
                    Expr {
                        kind: ExprKind::Assign { target: Box::new(lhs), op: Some(AssignOp::Div), value: Box::new(rhs) },
                        span,
                    }
                }
                TokenKind::PercentEq => {
                    validate_assign_target(&lhs)?;
                    self.skip_newlines();
                    let rhs  = self.parse_expr_bp(r_bp)?;
                    let span = Span::new(lhs_start, rhs.span.end);
                    Expr {
                        kind: ExprKind::Assign { target: Box::new(lhs), op: Some(AssignOp::Mod), value: Box::new(rhs) },
                        span,
                    }
                }

                // --- Binary operators ---
                ref tok => {
                    let op = token_to_binop(tok).ok_or_else(|| {
                        self.err(ParseErrorKind::UnexpectedToken {
                            expected: "binary operator".into(),
                            found: format!("{tok}"),
                        })
                    })?;
                    self.skip_newlines();
                    let rhs  = self.parse_expr_bp(r_bp)?;
                    let span = Span::new(lhs_start, rhs.span.end);
                    Expr {
                        kind: ExprKind::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                        span,
                    }
                }
            };
        }

        Ok(lhs)
    }

    // -----------------------------------------------------------------------
    // Prefix / primary expressions
    // -----------------------------------------------------------------------

    fn parse_prefix(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        let line  = self.line();
        let col   = self.col();

        match self.peek().clone() {
            // --- Literals ---
            TokenKind::Int(n) => {
                self.advance();
                Ok(Expr { kind: ExprKind::Int(n), span: self.prev_span() })
            }
            TokenKind::Float(f) => {
                self.advance();
                Ok(Expr { kind: ExprKind::Float(f), span: self.prev_span() })
            }
            TokenKind::Str(s) => {
                self.advance();
                Ok(Expr { kind: ExprKind::Str(s), span: self.prev_span() })
            }
            TokenKind::Bool(b) => {
                self.advance();
                Ok(Expr { kind: ExprKind::Bool(b), span: self.prev_span() })
            }
            TokenKind::Nil => {
                self.advance();
                Ok(Expr { kind: ExprKind::Nil, span: self.prev_span() })
            }

            // --- Identifier ---
            TokenKind::Ident(name) => {
                self.advance();
                Ok(Expr { kind: ExprKind::Ident(name), span: self.prev_span() })
            }

            // --- Grouped expression ---
            TokenKind::LParen => {
                self.advance(); // consume `(`
                self.skip_newlines();
                let inner = self.parse_expr()?;
                self.skip_newlines();
                self.expect(&TokenKind::RParen)?;
                Ok(inner)
            }

            // --- List literal  [a, b, c] ---
            TokenKind::LBracket => self.parse_list_lit(start),

            // --- Lambda  fn(params) => expr  |  fn(params) { … }
            //     Named fn is handled as a statement, so here `fn` can only
            //     be a lambda (next token is `(`).
            TokenKind::Kw(Keyword::Fn) => self.parse_lambda_expr(start),

            // --- Block expression  { … } ---
            TokenKind::LBrace => {
                let block = self.parse_block()?;
                let span  = block.span;
                Ok(Expr { kind: ExprKind::Block(block), span })
            }

            // --- If expression ---
            TokenKind::Kw(Keyword::If) => self.parse_if_expr(start),

            // --- Match expression ---
            TokenKind::Kw(Keyword::Match) => self.parse_match_expr(start),

            // --- spawn expr ---
            TokenKind::Kw(Keyword::Spawn) => {
                self.advance();
                self.skip_newlines();
                let inner = self.parse_expr_bp(19)?;
                let span  = Span::new(start, inner.span.end);
                Ok(Expr { kind: ExprKind::Spawn(Box::new(inner)), span })
            }

            // --- Unary operators ---
            TokenKind::Minus => {
                self.advance();
                self.skip_newlines();
                let operand = self.parse_expr_bp(19)?;
                let span    = Span::new(start, operand.span.end);
                Ok(Expr { kind: ExprKind::UnaryOp { op: UnaryOp::Neg, operand: Box::new(operand) }, span })
            }
            TokenKind::Bang | TokenKind::Kw(Keyword::Not) => {
                self.advance();
                self.skip_newlines();
                let operand = self.parse_expr_bp(19)?;
                let span    = Span::new(start, operand.span.end);
                Ok(Expr { kind: ExprKind::UnaryOp { op: UnaryOp::Not, operand: Box::new(operand) }, span })
            }

            _ => {
                Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken {
                        expected: "expression".into(),
                        found: format!("{}", self.peek()),
                    },
                    self.span(), line, col,
                ))
            }
        }
    }

    // -----------------------------------------------------------------------
    // Sub-expression parsers
    // -----------------------------------------------------------------------

    // Lambda: fn(params) => expr   or   fn(params) { … }
    fn parse_lambda_expr(&mut self, start: usize) -> PResult<Expr> {
        self.advance(); // consume `fn`
        let params = self.parse_param_list()?;
        let body = if self.eat(&TokenKind::FatArrow) {
            self.skip_newlines();
            let e = self.parse_expr()?;
            LambdaBody::Expr(Box::new(e))
        } else if self.check(&TokenKind::LBrace) {
            LambdaBody::Block(self.parse_block()?)
        } else {
            return Err(self.unexpected("`=>` or `{` after lambda params"));
        };
        let span = Span::new(start, self.prev_span().end);
        Ok(Expr { kind: ExprKind::Lambda { params, body }, span })
    }

    // List literal: [expr, …]
    fn parse_list_lit(&mut self, start: usize) -> PResult<Expr> {
        self.advance(); // consume `[`
        let mut items = Vec::new();
        self.skip_newlines();
        while !self.check(&TokenKind::RBracket) && !self.is_eof() {
            items.push(self.parse_expr()?);
            self.skip_newlines();
            if !self.eat(&TokenKind::Comma) {
                break;
            }
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBracket)?;
        let span = Span::new(start, self.prev_span().end);
        Ok(Expr { kind: ExprKind::List(items), span })
    }

    // If expression: if cond { then } [else …]
    fn parse_if_expr(&mut self, start: usize) -> PResult<Expr> {
        self.advance(); // consume `if`
        self.skip_newlines();
        let cond = self.parse_expr()?;
        let then = self.parse_block()?;
        let else_ = if self.eat_kw(Keyword::Else) {
            self.skip_newlines();
            if self.check_kw(Keyword::If) {
                // else if — recurse as expression
                let inner_start = self.span().start;
                let inner = self.parse_if_expr(inner_start)?;
                Some(Box::new(inner))
            } else {
                let blk  = self.parse_block()?;
                let span = blk.span;
                Some(Box::new(Expr { kind: ExprKind::Block(blk), span }))
            }
        } else {
            None
        };
        let span = Span::new(start, self.prev_span().end);
        Ok(Expr { kind: ExprKind::If { cond: Box::new(cond), then, else_ }, span })
    }

    // Match expression: match scrutinee { arm, … }
    fn parse_match_expr(&mut self, start: usize) -> PResult<Expr> {
        let (scrutinee, arms) = self.parse_match_inner()?;
        let span = Span::new(start, self.prev_span().end);
        Ok(Expr {
            kind: ExprKind::Match { scrutinee: Box::new(scrutinee), arms },
            span,
        })
    }

    // Shared match body: `match scrutinee { arms }`
    fn parse_match_inner(&mut self) -> PResult<(Expr, Vec<MatchArm>)> {
        self.advance(); // consume `match`
        self.skip_newlines();
        let scrutinee = self.parse_expr()?;
        self.expect(&TokenKind::LBrace)?;
        let mut arms = Vec::new();
        self.skip_newlines();
        while !self.check(&TokenKind::RBrace) && !self.is_eof() {
            arms.push(self.parse_match_arm()?);
            self.skip_newlines();
            // Optional trailing comma after arm body
            self.eat(&TokenKind::Comma);
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace)?;
        Ok((scrutinee, arms))
    }

    fn parse_match_arm(&mut self) -> PResult<MatchArm> {
        let start   = self.span().start;
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::FatArrow)?;
        self.skip_newlines();
        let body = self.parse_expr()?;
        Ok(MatchArm { pattern, body, span: Span::new(start, self.prev_span().end) })
    }

    fn parse_pattern(&mut self) -> PResult<Pattern> {
        match self.peek().clone() {
            TokenKind::Ident(name) if name == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenKind::Ident(name) => {
                self.advance();
                Ok(Pattern::Ident(name))
            }
            TokenKind::Int(_)
            | TokenKind::Float(_)
            | TokenKind::Str(_)
            | TokenKind::Bool(_)
            | TokenKind::Nil => {
                let expr = self.parse_prefix()?;
                Ok(Pattern::Literal(expr))
            }
            _ => Err(self.unexpected("pattern")),
        }
    }

    // Call argument list: (expr, …)  — '(' already NOT consumed here
    fn parse_arg_list(&mut self) -> PResult<Vec<Expr>> {
        // '(' was already consumed by the infix loop
        let mut args = Vec::new();
        self.skip_newlines();
        while !self.check(&TokenKind::RParen) && !self.is_eof() {
            args.push(self.parse_expr()?);
            self.skip_newlines();
            if !self.eat(&TokenKind::Comma) {
                break;
            }
            self.skip_newlines();
        }
        self.expect(&TokenKind::RParen)?;
        Ok(args)
    }
}

// ---------------------------------------------------------------------------
// Pratt binding-power table
// ---------------------------------------------------------------------------

/// Returns `(left_bp, right_bp)` for infix / postfix operators.
/// Returns `None` if the token is not an infix / postfix operator.
fn infix_bp(kind: &TokenKind) -> Option<(u8, u8)> {
    Some(match kind {
        // Assignment — right-associative
        TokenKind::Eq
        | TokenKind::PlusEq
        | TokenKind::MinusEq
        | TokenKind::StarEq
        | TokenKind::SlashEq
        | TokenKind::PercentEq => (2, 1),

        // Logical or
        TokenKind::Or | TokenKind::Kw(Keyword::Or) => (4, 5),

        // Logical and
        TokenKind::And | TokenKind::Kw(Keyword::And) => (6, 7),

        // Equality / identity
        TokenKind::EqEq | TokenKind::BangEq | TokenKind::Kw(Keyword::Is) => (8, 9),

        // Comparison
        TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => (10, 11),

        // Range
        TokenKind::DotDot | TokenKind::DotDotEq => (12, 13),

        // Additive
        TokenKind::Plus | TokenKind::Minus => (14, 15),

        // Multiplicative
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => (16, 17),

        // Postfix — call, index, member (highest)
        TokenKind::LParen | TokenKind::LBracket | TokenKind::Dot => (20, 0),

        _ => return None,
    })
}

fn is_postfix_op(kind: &TokenKind) -> bool {
    matches!(kind, TokenKind::LParen | TokenKind::LBracket | TokenKind::Dot)
}

// ---------------------------------------------------------------------------
// Token → AST operator conversion
// ---------------------------------------------------------------------------

fn token_to_binop(kind: &TokenKind) -> Option<BinOp> {
    Some(match kind {
        TokenKind::Plus    => BinOp::Add,
        TokenKind::Minus   => BinOp::Sub,
        TokenKind::Star    => BinOp::Mul,
        TokenKind::Slash   => BinOp::Div,
        TokenKind::Percent => BinOp::Mod,
        TokenKind::EqEq    => BinOp::Eq,
        TokenKind::BangEq  => BinOp::NotEq,
        TokenKind::Lt      => BinOp::Lt,
        TokenKind::Gt      => BinOp::Gt,
        TokenKind::LtEq    => BinOp::LtEq,
        TokenKind::GtEq    => BinOp::GtEq,
        TokenKind::And | TokenKind::Kw(Keyword::And) => BinOp::And,
        TokenKind::Or  | TokenKind::Kw(Keyword::Or)  => BinOp::Or,
        TokenKind::Kw(Keyword::Is) => BinOp::Is,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Assignment-target validation
// ---------------------------------------------------------------------------

fn validate_assign_target(expr: &Expr) -> PResult<()> {
    match &expr.kind {
        ExprKind::Ident(_) | ExprKind::Member { .. } | ExprKind::Index { .. } => Ok(()),
        _ => Err(ParseError::new(
            ParseErrorKind::InvalidAssignTarget,
            expr.span,
            0, 0,
        )),
    }
}
