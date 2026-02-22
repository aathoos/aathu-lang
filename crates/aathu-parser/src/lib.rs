//! aathu parser — converts a token stream into an AST.
//!
//! # Quick start
//!
//! ```rust
//! use aathu_lexer::Lexer;
//! use aathu_parser::Parser;
//!
//! let src = "fn greet(name) { print(\"Hello, \" + name) }";
//! let (tokens, lex_errors) = Lexer::new(src).tokenize();
//! assert!(lex_errors.is_empty());
//! let program = Parser::new(tokens).parse_program().unwrap();
//! assert_eq!(program.stmts.len(), 1);
//! ```

pub mod ast;
pub mod error;
pub mod grammar;
pub mod precedence;

pub use grammar::Parser;
pub use ast::{
    AssignOp, BinOp, Block, ElseBranch, EnumVariant, Expr, ExprKind,
    LambdaBody, MatchArm, Pattern, Program, Stmt, StmtKind, UnaryOp,
};
pub use error::{ParseError, ParseErrorKind};

#[cfg(test)]
mod tests;
