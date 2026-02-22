//! aathu lexer — tokenises aathu source text into a flat token stream.
//!
//! # Quick start
//!
//! ```rust
//! use aathu_lexer::Lexer;
//!
//! let src = r#"fn greet(name) { print("Hello, " + name) }"#;
//! let (tokens, errors) = Lexer::new(src).tokenize();
//! assert!(errors.is_empty());
//! ```

pub mod cursor;
pub mod error;
pub mod lexer;
pub mod token;

pub use lexer::Lexer;
pub use token::{Keyword, Token, TokenKind};
pub use error::{LexError, LexErrorKind};

#[cfg(test)]
mod tests;
