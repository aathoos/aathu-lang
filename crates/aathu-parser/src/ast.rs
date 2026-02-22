//! Abstract Syntax Tree for aathu.
//!
//! Design notes (mirrors lexer token design):
//! - Dynamically typed — no type annotations.
//! - No `while` / `do-while`; only `for … in`.
//! - Arrow lambdas:  `fn(x) => x + 1`  and block lambdas:  `fn(x) { … }`.
//! - `nil`, `true`, `false` are first-class literals.
//! - Assignment is an expression (`x = 5`, `x += 1`).
//! - Ranges are expressions (`0..10`, `0..=9`).
//! - `if` and `match` can appear as both statements and expressions.

use aathu_core::span::Span;

// ---------------------------------------------------------------------------
// Top-level
// ---------------------------------------------------------------------------

/// A complete parsed source file.
#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    // -----------------------------------------------------------------------
    // Declarations
    // -----------------------------------------------------------------------

    /// `let name = expr`  or  `let name` (uninitialized)
    Let {
        name: String,
        init: Option<Expr>,
    },

    /// Named function:  `fn name(p1, p2) { … }`
    /// With optional `pub` visibility.
    Fn {
        public: bool,
        name: String,
        params: Vec<String>,
        body: Block,
    },

    /// `struct Name { field1, field2 }`
    Struct {
        public: bool,
        name: String,
        fields: Vec<String>,
    },

    /// `enum Name { Variant1, Variant2 }`
    Enum {
        public: bool,
        name: String,
        variants: Vec<EnumVariant>,
    },

    /// `impl Name { … }`  — method block stub.
    Impl {
        name: String,
        body: Vec<Stmt>,
    },

    // -----------------------------------------------------------------------
    // Module system
    // -----------------------------------------------------------------------

    /// `import path::to::mod`  or  `import path as alias`
    Import {
        path: Vec<String>,
        alias: Option<String>,
    },

    /// `export name`
    Export {
        name: String,
    },

    // -----------------------------------------------------------------------
    // Control flow
    // -----------------------------------------------------------------------

    /// `return`  or  `return expr`
    Return(Option<Expr>),

    /// `break`
    Break,

    /// `continue`
    Continue,

    // -----------------------------------------------------------------------
    // Compound statements
    // -----------------------------------------------------------------------

    /// `if cond { … } else { … }` — else branch is optional; handles
    /// `else if` chains by nesting an `If` stmt inside the else block.
    If {
        cond: Expr,
        then: Block,
        else_: Option<ElseBranch>,
    },

    /// `for var in iter { … }`
    For {
        var: String,
        iter: Expr,
        body: Block,
    },

    /// `match scrutinee { pat => expr, … }`
    Match {
        scrutinee: Expr,
        arms: Vec<MatchArm>,
    },

    // -----------------------------------------------------------------------
    // Expression statement
    // -----------------------------------------------------------------------

    /// Any expression used as a statement (calls, assignments, etc.).
    Expr(Expr),
}

// ---------------------------------------------------------------------------
// Blocks
// ---------------------------------------------------------------------------

/// A `{ stmt* }` block; carries its own span.
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Else branch
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum ElseBranch {
    /// `else { … }`
    Block(Block),
    /// `else if cond { … }`
    If(Box<Stmt>),
}

// ---------------------------------------------------------------------------
// Match
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    /// `_` — wildcard
    Wildcard,
    /// A literal value: number, string, bool, nil
    Literal(Expr),
    /// A bare identifier — acts as a catch-all binding
    Ident(String),
}

// ---------------------------------------------------------------------------
// Enum variant (no payload for now)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    // -----------------------------------------------------------------------
    // Literals
    // -----------------------------------------------------------------------
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Nil,

    // -----------------------------------------------------------------------
    // Variable reference
    // -----------------------------------------------------------------------
    Ident(String),

    // -----------------------------------------------------------------------
    // Collection literals
    // -----------------------------------------------------------------------
    /// `[a, b, c]`
    List(Vec<Expr>),
    /// `{ key: val, … }` — map literal using brace syntax
    Map(Vec<(Expr, Expr)>),

    // -----------------------------------------------------------------------
    // Functions
    // -----------------------------------------------------------------------
    /// `fn(p1, p2) => expr`  or  `fn(p1, p2) { … }`
    Lambda {
        params: Vec<String>,
        body: LambdaBody,
    },

    // -----------------------------------------------------------------------
    // Operators
    // -----------------------------------------------------------------------
    /// Binary infix: `a + b`, `a == b`, etc.
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    /// Unary prefix: `-x`, `!x`, `not x`
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },

    // -----------------------------------------------------------------------
    // Assignment
    // -----------------------------------------------------------------------
    /// `x = val`, `x += val`, etc.  `op` is `None` for plain `=`.
    Assign {
        target: Box<Expr>,
        op: Option<AssignOp>,
        value: Box<Expr>,
    },

    // -----------------------------------------------------------------------
    // Range
    // -----------------------------------------------------------------------
    /// `lo..hi` (exclusive) or `lo..=hi` (inclusive)
    Range {
        lo: Box<Expr>,
        hi: Box<Expr>,
        inclusive: bool,
    },

    // -----------------------------------------------------------------------
    // Postfix / access
    // -----------------------------------------------------------------------
    /// `callee(arg, …)`
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    /// `object[index]`
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    /// `object.field`
    Member {
        object: Box<Expr>,
        field: String,
    },

    // -----------------------------------------------------------------------
    // Control-flow expressions
    // -----------------------------------------------------------------------
    /// `if cond { … } else { … }` used as an expression
    If {
        cond: Box<Expr>,
        then: Block,
        else_: Option<Box<Expr>>,
    },

    /// `match scrutinee { pat => expr, … }` used as an expression
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// `{ stmt* }` — block yields the value of the last expression, if any
    Block(Block),

    /// `spawn expr` — launch a concurrent task; yields a task handle
    Spawn(Box<Expr>),
}

// ---------------------------------------------------------------------------
// Lambda body
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum LambdaBody {
    /// `fn(x) => x + 1`  — single expression
    Expr(Box<Expr>),
    /// `fn(x) { … }`  — full block
    Block(Block),
}

// ---------------------------------------------------------------------------
// Operators
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    // Comparison
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    // Logical
    And, Or,
    // Identity check
    Is,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, // `-`
    Not, // `!` or `not`
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignOp {
    Add, // +=
    Sub, // -=
    Mul, // *=
    Div, // /=
    Mod, // %=
}
