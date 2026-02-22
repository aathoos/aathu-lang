//! Lexer test suite.
//!
//! Covers every token kind, all error paths, and real `.aathu` programs.

use crate::{
    error::LexErrorKind,
    token::{Keyword, TokenKind},
    Lexer,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn lex(src: &str) -> Vec<TokenKind> {
    let (tokens, _) = Lexer::new(src).tokenize();
    tokens.into_iter().map(|t| t.kind).collect()
}

fn lex_clean(src: &str) -> Vec<TokenKind> {
    let (tokens, errors) = Lexer::new(src).tokenize();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    tokens.into_iter().map(|t| t.kind).collect()
}

fn errors(src: &str) -> Vec<LexErrorKind> {
    let (_, errs) = Lexer::new(src).tokenize();
    errs.into_iter().map(|e| e.kind).collect()
}

fn kinds_no_newlines_eof(src: &str) -> Vec<TokenKind> {
    lex_clean(src)
        .into_iter()
        .filter(|k| !matches!(k, TokenKind::Newline | TokenKind::Eof))
        .collect()
}

// ---------------------------------------------------------------------------
// Literals
// ---------------------------------------------------------------------------

#[test]
fn int_decimal() {
    assert_eq!(lex_clean("42"), vec![TokenKind::Int(42), TokenKind::Eof]);
}

#[test]
fn int_with_underscores() {
    assert_eq!(
        lex_clean("1_000_000"),
        vec![TokenKind::Int(1_000_000), TokenKind::Eof]
    );
}

#[test]
fn int_hex() {
    assert_eq!(lex_clean("0xFF"), vec![TokenKind::Int(255), TokenKind::Eof]);
    assert_eq!(lex_clean("0XFF"), vec![TokenKind::Int(255), TokenKind::Eof]);
}

#[test]
fn int_binary() {
    assert_eq!(lex_clean("0b1010"), vec![TokenKind::Int(10), TokenKind::Eof]);
    assert_eq!(lex_clean("0B1111"), vec![TokenKind::Int(15), TokenKind::Eof]);
}

#[test]
fn int_octal() {
    assert_eq!(lex_clean("0o17"), vec![TokenKind::Int(15), TokenKind::Eof]);
    assert_eq!(lex_clean("0O777"), vec![TokenKind::Int(511), TokenKind::Eof]);
}

#[test]
fn float_basic() {
    let (tokens, errors) = Lexer::new("3.14").tokenize();
    assert!(errors.is_empty());
    assert_eq!(tokens[0].kind, TokenKind::Float(3.14));
}

#[test]
fn float_with_exponent_full() {
    let (tokens, errors) = Lexer::new("1.5e+3").tokenize();
    assert!(errors.is_empty());
    assert_eq!(tokens[0].kind, TokenKind::Float(1500.0));
}

#[test]
fn float_negative_exponent() {
    let (tokens, errors) = Lexer::new("2.5e-1").tokenize();
    assert!(errors.is_empty());
    assert_eq!(tokens[0].kind, TokenKind::Float(0.25));
}

#[test]
fn range_does_not_split_int() {
    // `0..10` must NOT be parsed as float `0.` + `.10`
    let kinds = kinds_no_newlines_eof("0..10");
    assert_eq!(
        kinds,
        vec![TokenKind::Int(0), TokenKind::DotDot, TokenKind::Int(10)]
    );
}

#[test]
fn string_basic() {
    let kinds = lex_clean(r#""hello""#);
    assert_eq!(kinds[0], TokenKind::Str("hello".to_string()));
}

#[test]
fn string_escapes() {
    // raw source bytes: "\n\t\r\\\""
    let kinds = lex_clean("\"\\n\\t\\r\\\\\\\"\"");
    match &kinds[0] {
        TokenKind::Str(s) => assert_eq!(s, "\n\t\r\\\""),
        other => panic!("expected Str, got {other:?}"),
    }
}

#[test]
fn string_null_byte() {
    let kinds = lex_clean(r#""\0""#);
    match &kinds[0] {
        TokenKind::Str(s) => assert_eq!(s.as_bytes()[0], 0),
        other => panic!("expected Str, got {other:?}"),
    }
}

#[test]
fn string_unicode_escape() {
    let kinds = lex_clean(r#""\u{1F600}""#);
    match &kinds[0] {
        TokenKind::Str(s) => assert_eq!(s, "\u{1F600}"),
        other => panic!("expected Str, got {other:?}"),
    }
}

#[test]
fn bool_literals() {
    let kinds = kinds_no_newlines_eof("true false");
    assert_eq!(kinds, vec![TokenKind::Bool(true), TokenKind::Bool(false)]);
}

#[test]
fn nil_literal() {
    let kinds = kinds_no_newlines_eof("nil");
    assert_eq!(kinds, vec![TokenKind::Nil]);
}

// ---------------------------------------------------------------------------
// Keywords
// ---------------------------------------------------------------------------

#[test]
fn all_keywords() {
    let src = "let fn if else for in return break continue match \
               import export pub mod as struct enum impl trait type \
               async await spawn and or not is";
    let kinds = kinds_no_newlines_eof(src);
    let expected: Vec<TokenKind> = vec![
        TokenKind::Kw(Keyword::Let),
        TokenKind::Kw(Keyword::Fn),
        TokenKind::Kw(Keyword::If),
        TokenKind::Kw(Keyword::Else),
        TokenKind::Kw(Keyword::For),
        TokenKind::Kw(Keyword::In),
        TokenKind::Kw(Keyword::Return),
        TokenKind::Kw(Keyword::Break),
        TokenKind::Kw(Keyword::Continue),
        TokenKind::Kw(Keyword::Match),
        TokenKind::Kw(Keyword::Import),
        TokenKind::Kw(Keyword::Export),
        TokenKind::Kw(Keyword::Pub),
        TokenKind::Kw(Keyword::Mod),
        TokenKind::Kw(Keyword::As),
        TokenKind::Kw(Keyword::Struct),
        TokenKind::Kw(Keyword::Enum),
        TokenKind::Kw(Keyword::Impl),
        TokenKind::Kw(Keyword::Trait),
        TokenKind::Kw(Keyword::Type),
        TokenKind::Kw(Keyword::Async),
        TokenKind::Kw(Keyword::Await),
        TokenKind::Kw(Keyword::Spawn),
        TokenKind::Kw(Keyword::And),
        TokenKind::Kw(Keyword::Or),
        TokenKind::Kw(Keyword::Not),
        TokenKind::Kw(Keyword::Is),
    ];
    assert_eq!(kinds, expected);
}

// ---------------------------------------------------------------------------
// Operators
// ---------------------------------------------------------------------------

#[test]
fn arithmetic_ops() {
    let kinds = kinds_no_newlines_eof("+ - * / %");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
        ]
    );
}

#[test]
fn compound_assign_ops() {
    let kinds = kinds_no_newlines_eof("+= -= *= /= %=");
    assert_eq!(
        kinds,
        vec![
            TokenKind::PlusEq,
            TokenKind::MinusEq,
            TokenKind::StarEq,
            TokenKind::SlashEq,
            TokenKind::PercentEq,
        ]
    );
}

#[test]
fn comparison_ops() {
    let kinds = kinds_no_newlines_eof("== != < > <= >=");
    assert_eq!(
        kinds,
        vec![
            TokenKind::EqEq,
            TokenKind::BangEq,
            TokenKind::Lt,
            TokenKind::Gt,
            TokenKind::LtEq,
            TokenKind::GtEq,
        ]
    );
}

#[test]
fn logical_ops() {
    let kinds = kinds_no_newlines_eof("&& || !");
    assert_eq!(kinds, vec![TokenKind::And, TokenKind::Or, TokenKind::Bang]);
}

#[test]
fn range_ops() {
    let kinds = kinds_no_newlines_eof(".. ..=");
    assert_eq!(kinds, vec![TokenKind::DotDot, TokenKind::DotDotEq]);
}

#[test]
fn arrows() {
    let kinds = kinds_no_newlines_eof("-> =>");
    assert_eq!(kinds, vec![TokenKind::Arrow, TokenKind::FatArrow]);
}

#[test]
fn assignment() {
    let kinds = kinds_no_newlines_eof("= ==");
    assert_eq!(kinds, vec![TokenKind::Eq, TokenKind::EqEq]);
}

// ---------------------------------------------------------------------------
// Delimiters and punctuation
// ---------------------------------------------------------------------------

#[test]
fn delimiters() {
    let kinds = kinds_no_newlines_eof("( ) { } [ ]");
    assert_eq!(
        kinds,
        vec![
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::LBracket,
            TokenKind::RBracket,
        ]
    );
}

#[test]
fn punctuation() {
    let kinds = kinds_no_newlines_eof(", : ; .");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Comma,
            TokenKind::Colon,
            TokenKind::Semi,
            TokenKind::Dot,
        ]
    );
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

#[test]
fn line_comment_skipped() {
    let kinds = kinds_no_newlines_eof("42 // this is a comment\n43");
    assert_eq!(kinds, vec![TokenKind::Int(42), TokenKind::Int(43)]);
}

#[test]
fn block_comment_skipped() {
    let kinds = kinds_no_newlines_eof("1 /* comment */ 2");
    assert_eq!(kinds, vec![TokenKind::Int(1), TokenKind::Int(2)]);
}

#[test]
fn nested_block_comment() {
    let kinds = kinds_no_newlines_eof("1 /* outer /* inner */ outer */ 2");
    assert_eq!(kinds, vec![TokenKind::Int(1), TokenKind::Int(2)]);
}

#[test]
fn block_comment_multiline() {
    let kinds = kinds_no_newlines_eof("a /* hello\nworld */ b");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("a".to_string()),
            TokenKind::Ident("b".to_string()),
        ]
    );
}

// ---------------------------------------------------------------------------
// Newline handling
// ---------------------------------------------------------------------------

#[test]
fn newlines_emitted() {
    let kinds = lex_clean("a\nb");
    assert!(kinds.contains(&TokenKind::Newline));
}

#[test]
fn multiple_newlines_produce_one_each() {
    let count = lex_clean("a\n\nb")
        .iter()
        .filter(|k| **k == TokenKind::Newline)
        .count();
    assert_eq!(count, 2);
}

// ---------------------------------------------------------------------------
// Span and position
// ---------------------------------------------------------------------------

#[test]
fn token_spans() {
    let (tokens, _) = Lexer::new("hello 42").tokenize();
    // Ident("hello") @ 0..5, Int(42) @ 6..8
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 5);
    assert_eq!(tokens[1].span.start, 6);
    assert_eq!(tokens[1].span.end, 8);
}

#[test]
fn token_line_col() {
    let (tokens, _) = Lexer::new("a\nb").tokenize();
    // tokens[0]=Ident("a") line=1 col=1
    // tokens[1]=Newline      line=1 col=2
    // tokens[2]=Ident("b") line=2 col=1
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].col,  1);
    assert_eq!(tokens[2].line, 2);
    assert_eq!(tokens[2].col,  1);
}

// ---------------------------------------------------------------------------
// Error recovery
// ---------------------------------------------------------------------------

#[test]
fn error_unterminated_string() {
    let errs = errors("\"hello");
    assert_eq!(errs, vec![LexErrorKind::UnterminatedString]);
}

#[test]
fn error_newline_in_string() {
    // The lexer emits NewlineInString for the bare \n inside the string,
    // then continues: the trailing  world"  scans as an ident followed by
    // an unterminated string (just `"`), so two errors are the correct
    // error-recovery behaviour.
    let errs = errors("\"hello\nworld\"");
    assert_eq!(
        errs,
        vec![LexErrorKind::NewlineInString, LexErrorKind::UnterminatedString]
    );
}

#[test]
fn error_invalid_escape() {
    let errs = errors(r#""\q""#);
    assert_eq!(errs, vec![LexErrorKind::InvalidEscapeSequence('q')]);
}

#[test]
fn error_unterminated_block_comment() {
    let errs = errors("/* oops");
    assert_eq!(errs, vec![LexErrorKind::UnterminatedBlockComment]);
}

#[test]
fn error_unexpected_char() {
    let errs = errors("@");
    assert_eq!(errs, vec![LexErrorKind::UnexpectedCharacter('@')]);
}

#[test]
fn error_single_ampersand() {
    let errs = errors("&");
    assert_eq!(errs, vec![LexErrorKind::UnexpectedCharacter('&')]);
}

#[test]
fn error_single_pipe() {
    let errs = errors("|");
    assert_eq!(errs, vec![LexErrorKind::UnexpectedCharacter('|')]);
}

#[test]
fn error_recovery_continues() {
    // Unknown char must not halt lexing — the integer after @ must still appear.
    let kinds = lex("@ 42");
    let has_int = kinds.iter().any(|k| *k == TokenKind::Int(42));
    assert!(has_int, "lexer stopped at error instead of recovering");
}

// ---------------------------------------------------------------------------
// Real .aathu program snippets
// ---------------------------------------------------------------------------

#[test]
fn hello_world() {
    let src = r#"fn main() { print("Hello, World!") }"#;
    let (_, errors) = Lexer::new(src).tokenize();
    assert!(errors.is_empty());

    let kinds = kinds_no_newlines_eof(src);
    assert_eq!(
        kinds,
        vec![
            TokenKind::Kw(Keyword::Fn),
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Ident("print".to_string()),
            TokenKind::LParen,
            TokenKind::Str("Hello, World!".to_string()),
            TokenKind::RParen,
            TokenKind::RBrace,
        ]
    );
}

#[test]
fn fibonacci() {
    let src = "fn fib(n) { if n <= 1 { return n } return fib(n-1) + fib(n-2) }\n\
               fn main() { for i in 0..20 { print(fib(i)) } }";
    let (_, errors) = Lexer::new(src).tokenize();
    assert!(errors.is_empty());
}

#[test]
fn control_flow_program() {
    let src = "fn main() {\n\
               let x = 10\n\
               if x > 5 { print(\"big\") } else { print(\"small\") }\n\
               }";
    let (_, errors) = Lexer::new(src).tokenize();
    assert!(errors.is_empty());
}

#[test]
fn basic_fn_and_return() {
    let src = "fn add(a, b) { return a + b }";
    let kinds = kinds_no_newlines_eof(src);
    assert_eq!(
        kinds,
        vec![
            TokenKind::Kw(Keyword::Fn),
            TokenKind::Ident("add".to_string()),
            TokenKind::LParen,
            TokenKind::Ident("a".to_string()),
            TokenKind::Comma,
            TokenKind::Ident("b".to_string()),
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Kw(Keyword::Return),
            TokenKind::Ident("a".to_string()),
            TokenKind::Plus,
            TokenKind::Ident("b".to_string()),
            TokenKind::RBrace,
        ]
    );
}

#[test]
fn arrow_function_syntax() {
    let kinds = kinds_no_newlines_eof("fn(x) => x + 1");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Kw(Keyword::Fn),
            TokenKind::LParen,
            TokenKind::Ident("x".to_string()),
            TokenKind::RParen,
            TokenKind::FatArrow,
            TokenKind::Ident("x".to_string()),
            TokenKind::Plus,
            TokenKind::Int(1),
        ]
    );
}

#[test]
fn for_in_range() {
    let kinds = kinds_no_newlines_eof("for i in 0..10 { }");
    assert!(kinds.contains(&TokenKind::Kw(Keyword::For)));
    assert!(kinds.contains(&TokenKind::Kw(Keyword::In)));
    assert!(kinds.contains(&TokenKind::DotDot));
}

#[test]
fn for_in_inclusive_range() {
    let kinds = kinds_no_newlines_eof("for i in 0..=9 { }");
    assert!(kinds.contains(&TokenKind::DotDotEq));
}

#[test]
fn let_assignment() {
    let kinds = kinds_no_newlines_eof("let x = 42");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Kw(Keyword::Let),
            TokenKind::Ident("x".to_string()),
            TokenKind::Eq,
            TokenKind::Int(42),
        ]
    );
}

#[test]
fn match_expression() {
    let src = "match x { 1 => \"one\" }";
    let kinds = kinds_no_newlines_eof(src);
    assert_eq!(kinds[0], TokenKind::Kw(Keyword::Match));
    assert!(kinds.contains(&TokenKind::FatArrow));
}

#[test]
fn logical_keyword_ops() {
    let kinds = kinds_no_newlines_eof("a and b or not c");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("a".to_string()),
            TokenKind::Kw(Keyword::And),
            TokenKind::Ident("b".to_string()),
            TokenKind::Kw(Keyword::Or),
            TokenKind::Kw(Keyword::Not),
            TokenKind::Ident("c".to_string()),
        ]
    );
}

#[test]
fn async_spawn() {
    let kinds = kinds_no_newlines_eof("async fn worker() { } spawn worker()");
    assert!(kinds.contains(&TokenKind::Kw(Keyword::Async)));
    assert!(kinds.contains(&TokenKind::Kw(Keyword::Spawn)));
}

#[test]
fn struct_definition() {
    let src = "struct Point { x, y }";
    let kinds = kinds_no_newlines_eof(src);
    assert_eq!(kinds[0], TokenKind::Kw(Keyword::Struct));
    assert_eq!(kinds[1], TokenKind::Ident("Point".to_string()));
}

#[test]
fn module_import() {
    let kinds = kinds_no_newlines_eof("import foo as f");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Kw(Keyword::Import),
            TokenKind::Ident("foo".to_string()),
            TokenKind::Kw(Keyword::As),
            TokenKind::Ident("f".to_string()),
        ]
    );
}

#[test]
fn member_access_not_range() {
    let kinds = kinds_no_newlines_eof("obj.field");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("obj".to_string()),
            TokenKind::Dot,
            TokenKind::Ident("field".to_string()),
        ]
    );
}

#[test]
fn underscore_ident() {
    let kinds = kinds_no_newlines_eof("_unused _foo123");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("_unused".to_string()),
            TokenKind::Ident("_foo123".to_string()),
        ]
    );
}

#[test]
fn eof_is_last_token() {
    let tokens = lex("42");
    assert_eq!(*tokens.last().unwrap(), TokenKind::Eof);
}

#[test]
fn empty_source() {
    let kinds = lex_clean("");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}

#[test]
fn only_whitespace() {
    let kinds = lex_clean("   \t  ");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}
