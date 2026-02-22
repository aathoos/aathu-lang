//! Parser test suite.
//!
//! Tests are grouped by construct. Each test tokenises a source snippet with
//! the lexer and then parses it, asserting on the resulting AST shape.

use aathu_lexer::Lexer;
use crate::{
    ast::*,
    error::ParseErrorKind,
    Parser,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse(src: &str) -> Program {
    let (tokens, lex_errs) = Lexer::new(src).tokenize();
    assert!(lex_errs.is_empty(), "lex errors in test input: {lex_errs:?}");
    Parser::new(tokens).parse_program().expect("parse failed")
}

fn parse_err(src: &str) -> ParseErrorKind {
    let (tokens, _) = Lexer::new(src).tokenize();
    Parser::new(tokens)
        .parse_program()
        .expect_err("expected parse error")
        .kind
}

fn parse_expr_src(src: &str) -> ExprKind {
    let prog = parse(src);
    match prog.stmts.into_iter().next().unwrap().kind {
        StmtKind::Expr(e) => e.kind,
        other => panic!("expected expr stmt, got {other:?}"),
    }
}

fn only_stmt(src: &str) -> StmtKind {
    let prog = parse(src);
    assert_eq!(prog.stmts.len(), 1);
    prog.stmts.into_iter().next().unwrap().kind
}

// ---------------------------------------------------------------------------
// Literals
// ---------------------------------------------------------------------------

#[test]
fn literal_int() {
    assert!(matches!(parse_expr_src("42"), ExprKind::Int(42)));
}

#[test]
fn literal_float() {
    assert!(matches!(parse_expr_src("3.14"), ExprKind::Float(_)));
}

#[test]
fn literal_string() {
    match parse_expr_src(r#""hello""#) {
        ExprKind::Str(s) => assert_eq!(s, "hello"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn literal_bool_true() {
    assert!(matches!(parse_expr_src("true"), ExprKind::Bool(true)));
}

#[test]
fn literal_bool_false() {
    assert!(matches!(parse_expr_src("false"), ExprKind::Bool(false)));
}

#[test]
fn literal_nil() {
    assert!(matches!(parse_expr_src("nil"), ExprKind::Nil));
}

// ---------------------------------------------------------------------------
// Arithmetic expressions
// ---------------------------------------------------------------------------

#[test]
fn binop_add() {
    match parse_expr_src("1 + 2") {
        ExprKind::BinOp { op: BinOp::Add, .. } => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn binop_precedence_mul_over_add() {
    // `1 + 2 * 3` must parse as `1 + (2 * 3)`
    match parse_expr_src("1 + 2 * 3") {
        ExprKind::BinOp { op: BinOp::Add, lhs, rhs } => {
            assert!(matches!(lhs.kind, ExprKind::Int(1)));
            assert!(matches!(rhs.kind, ExprKind::BinOp { op: BinOp::Mul, .. }));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn binop_left_assoc() {
    // `1 - 2 - 3` must parse as `(1 - 2) - 3`
    match parse_expr_src("1 - 2 - 3") {
        ExprKind::BinOp { op: BinOp::Sub, lhs, rhs } => {
            assert!(matches!(lhs.kind, ExprKind::BinOp { op: BinOp::Sub, .. }));
            assert!(matches!(rhs.kind, ExprKind::Int(3)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn unary_neg() {
    match parse_expr_src("-5") {
        ExprKind::UnaryOp { op: UnaryOp::Neg, operand } => {
            assert!(matches!(operand.kind, ExprKind::Int(5)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn unary_not() {
    match parse_expr_src("!true") {
        ExprKind::UnaryOp { op: UnaryOp::Not, operand } => {
            assert!(matches!(operand.kind, ExprKind::Bool(true)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn unary_not_keyword() {
    match parse_expr_src("not false") {
        ExprKind::UnaryOp { op: UnaryOp::Not, operand } => {
            assert!(matches!(operand.kind, ExprKind::Bool(false)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn grouped_expr() {
    // `(1 + 2) * 3` — grouping overrides precedence
    match parse_expr_src("(1 + 2) * 3") {
        ExprKind::BinOp { op: BinOp::Mul, lhs, .. } => {
            assert!(matches!(lhs.kind, ExprKind::BinOp { op: BinOp::Add, .. }));
        }
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Comparison and logical operators
// ---------------------------------------------------------------------------

#[test]
fn comparison_ops() {
    assert!(matches!(parse_expr_src("a == b"), ExprKind::BinOp { op: BinOp::Eq, .. }));
    assert!(matches!(parse_expr_src("a != b"), ExprKind::BinOp { op: BinOp::NotEq, .. }));
    assert!(matches!(parse_expr_src("a < b"),  ExprKind::BinOp { op: BinOp::Lt, .. }));
    assert!(matches!(parse_expr_src("a > b"),  ExprKind::BinOp { op: BinOp::Gt, .. }));
    assert!(matches!(parse_expr_src("a <= b"), ExprKind::BinOp { op: BinOp::LtEq, .. }));
    assert!(matches!(parse_expr_src("a >= b"), ExprKind::BinOp { op: BinOp::GtEq, .. }));
}

#[test]
fn logical_and_or() {
    assert!(matches!(parse_expr_src("a && b"), ExprKind::BinOp { op: BinOp::And, .. }));
    assert!(matches!(parse_expr_src("a || b"), ExprKind::BinOp { op: BinOp::Or, .. }));
    assert!(matches!(parse_expr_src("a and b"), ExprKind::BinOp { op: BinOp::And, .. }));
    assert!(matches!(parse_expr_src("a or b"),  ExprKind::BinOp { op: BinOp::Or, .. }));
}

#[test]
fn is_operator() {
    assert!(matches!(parse_expr_src("x is nil"), ExprKind::BinOp { op: BinOp::Is, .. }));
}

// ---------------------------------------------------------------------------
// Range expressions
// ---------------------------------------------------------------------------

#[test]
fn range_exclusive() {
    match parse_expr_src("0..10") {
        ExprKind::Range { inclusive: false, .. } => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn range_inclusive() {
    match parse_expr_src("0..=9") {
        ExprKind::Range { inclusive: true, .. } => {}
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Assignment
// ---------------------------------------------------------------------------

#[test]
fn simple_assign() {
    match parse_expr_src("x = 5") {
        ExprKind::Assign { op: None, .. } => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn compound_assign_add() {
    match parse_expr_src("x += 1") {
        ExprKind::Assign { op: Some(AssignOp::Add), .. } => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn compound_assign_all() {
    assert!(matches!(parse_expr_src("x -= 1"), ExprKind::Assign { op: Some(AssignOp::Sub), .. }));
    assert!(matches!(parse_expr_src("x *= 2"), ExprKind::Assign { op: Some(AssignOp::Mul), .. }));
    assert!(matches!(parse_expr_src("x /= 2"), ExprKind::Assign { op: Some(AssignOp::Div), .. }));
    assert!(matches!(parse_expr_src("x %= 3"), ExprKind::Assign { op: Some(AssignOp::Mod), .. }));
}

#[test]
fn assign_right_assoc() {
    // `a = b = 5` must parse as `a = (b = 5)`
    match parse_expr_src("a = b = 5") {
        ExprKind::Assign { target, value, .. } => {
            assert!(matches!(target.kind, ExprKind::Ident(ref s) if s == "a"));
            assert!(matches!(value.kind, ExprKind::Assign { .. }));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn error_invalid_assign_target() {
    match parse_err("1 + 2 = 3") {
        ParseErrorKind::InvalidAssignTarget => {}
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Postfix: call, index, member
// ---------------------------------------------------------------------------

#[test]
fn call_no_args() {
    match parse_expr_src("foo()") {
        ExprKind::Call { callee, args } => {
            assert!(matches!(callee.kind, ExprKind::Ident(ref s) if s == "foo"));
            assert!(args.is_empty());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn call_with_args() {
    match parse_expr_src("add(1, 2)") {
        ExprKind::Call { args, .. } => {
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0].kind, ExprKind::Int(1)));
            assert!(matches!(args[1].kind, ExprKind::Int(2)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn call_chained() {
    // `f()()` — chained calls
    match parse_expr_src("f()()") {
        ExprKind::Call { callee, .. } => {
            assert!(matches!(callee.kind, ExprKind::Call { .. }));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn index_expr() {
    match parse_expr_src("arr[0]") {
        ExprKind::Index { object, index } => {
            assert!(matches!(object.kind, ExprKind::Ident(ref s) if s == "arr"));
            assert!(matches!(index.kind, ExprKind::Int(0)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn member_expr() {
    match parse_expr_src("obj.field") {
        ExprKind::Member { object, field } => {
            assert!(matches!(object.kind, ExprKind::Ident(ref s) if s == "obj"));
            assert_eq!(field, "field");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn member_call_chain() {
    // `obj.method(1).field`
    match parse_expr_src("obj.method(1).field") {
        ExprKind::Member { object, field } => {
            assert_eq!(field, "field");
            assert!(matches!(object.kind, ExprKind::Call { .. }));
        }
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Collection literals
// ---------------------------------------------------------------------------

#[test]
fn list_empty() {
    match parse_expr_src("[]") {
        ExprKind::List(items) => assert!(items.is_empty()),
        other => panic!("{other:?}"),
    }
}

#[test]
fn list_with_items() {
    match parse_expr_src("[1, 2, 3]") {
        ExprKind::List(items) => assert_eq!(items.len(), 3),
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Lambda expressions
// ---------------------------------------------------------------------------

#[test]
fn lambda_arrow() {
    match parse_expr_src("fn(x) => x + 1") {
        ExprKind::Lambda { params, body: LambdaBody::Expr(_) } => {
            assert_eq!(params, vec!["x"]);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn lambda_block() {
    match parse_expr_src("fn(x) { return x + 1 }") {
        ExprKind::Lambda { params, body: LambdaBody::Block(_) } => {
            assert_eq!(params, vec!["x"]);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn lambda_no_params() {
    match parse_expr_src("fn() => 42") {
        ExprKind::Lambda { params, body: LambdaBody::Expr(_) } => {
            assert!(params.is_empty());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn lambda_multi_params() {
    match parse_expr_src("fn(a, b, c) => a + b + c") {
        ExprKind::Lambda { params, .. } => {
            assert_eq!(params, vec!["a", "b", "c"]);
        }
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Statements — let
// ---------------------------------------------------------------------------

#[test]
fn let_with_init() {
    match only_stmt("let x = 42") {
        StmtKind::Let { name, init: Some(expr) } => {
            assert_eq!(name, "x");
            assert!(matches!(expr.kind, ExprKind::Int(42)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn let_no_init() {
    match only_stmt("let x") {
        StmtKind::Let { name, init: None } => assert_eq!(name, "x"),
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Statements — fn
// ---------------------------------------------------------------------------

#[test]
fn fn_no_params() {
    match only_stmt("fn hello() { }") {
        StmtKind::Fn { name, params, .. } => {
            assert_eq!(name, "hello");
            assert!(params.is_empty());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn fn_with_params() {
    match only_stmt("fn add(a, b) { return a + b }") {
        StmtKind::Fn { name, params, body, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params, vec!["a", "b"]);
            assert_eq!(body.stmts.len(), 1);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn fn_public() {
    match only_stmt("pub fn greet(name) { }") {
        StmtKind::Fn { public: true, name, .. } => assert_eq!(name, "greet"),
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Statements — return / break / continue
// ---------------------------------------------------------------------------

#[test]
fn return_no_value() {
    match only_stmt("fn f() { return }") {
        StmtKind::Fn { body, .. } => {
            assert!(matches!(body.stmts[0].kind, StmtKind::Return(None)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn return_with_value() {
    match only_stmt("fn f() { return 42 }") {
        StmtKind::Fn { body, .. } => {
            match &body.stmts[0].kind {
                StmtKind::Return(Some(e)) => assert!(matches!(e.kind, ExprKind::Int(42))),
                other => panic!("{other:?}"),
            }
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn break_stmt() {
    match only_stmt("fn f() { break }") {
        StmtKind::Fn { body, .. } => {
            assert!(matches!(body.stmts[0].kind, StmtKind::Break));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn continue_stmt() {
    match only_stmt("fn f() { continue }") {
        StmtKind::Fn { body, .. } => {
            assert!(matches!(body.stmts[0].kind, StmtKind::Continue));
        }
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Statements — if / else
// ---------------------------------------------------------------------------

#[test]
fn if_no_else() {
    match only_stmt("if x > 0 { print(x) }") {
        StmtKind::If { else_: None, .. } => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn if_else() {
    match only_stmt("if x > 0 { print(x) } else { print(0) }") {
        StmtKind::If { else_: Some(ElseBranch::Block(_)), .. } => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn if_else_if() {
    match only_stmt("if x > 0 { } else if x < 0 { } else { }") {
        StmtKind::If { else_: Some(ElseBranch::If(_)), .. } => {}
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Statements — for
// ---------------------------------------------------------------------------

#[test]
fn for_exclusive_range() {
    match only_stmt("for i in 0..10 { print(i) }") {
        StmtKind::For { var, iter, body } => {
            assert_eq!(var, "i");
            assert!(matches!(iter.kind, ExprKind::Range { inclusive: false, .. }));
            assert!(!body.stmts.is_empty());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn for_inclusive_range() {
    match only_stmt("for i in 0..=9 { }") {
        StmtKind::For { iter, .. } => {
            assert!(matches!(iter.kind, ExprKind::Range { inclusive: true, .. }));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn for_over_ident() {
    match only_stmt("for item in list { }") {
        StmtKind::For { var, iter, .. } => {
            assert_eq!(var, "item");
            assert!(matches!(iter.kind, ExprKind::Ident(ref s) if s == "list"));
        }
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Statements — match
// ---------------------------------------------------------------------------

#[test]
fn match_basic() {
    let src = "match x { 1 => \"one\" 2 => \"two\" _ => \"other\" }";
    match only_stmt(src) {
        StmtKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 3);
            assert!(matches!(arms[0].pattern, Pattern::Literal(_)));
            assert!(matches!(arms[2].pattern, Pattern::Wildcard));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn match_ident_pattern() {
    let src = "match val { x => x + 1 }";
    match only_stmt(src) {
        StmtKind::Match { arms, .. } => {
            assert!(matches!(arms[0].pattern, Pattern::Ident(ref s) if s == "x"));
        }
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Statements — struct / enum / impl
// ---------------------------------------------------------------------------

#[test]
fn struct_def() {
    match only_stmt("struct Point { x, y }") {
        StmtKind::Struct { name, fields, .. } => {
            assert_eq!(name, "Point");
            assert_eq!(fields, vec!["x", "y"]);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn enum_def() {
    match only_stmt("enum Dir { North, South, East, West }") {
        StmtKind::Enum { name, variants, .. } => {
            assert_eq!(name, "Dir");
            assert_eq!(variants.len(), 4);
            assert_eq!(variants[0].name, "North");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn impl_block() {
    match only_stmt("impl Foo { fn bar() { } }") {
        StmtKind::Impl { name, body } => {
            assert_eq!(name, "Foo");
            assert_eq!(body.len(), 1);
        }
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Statements — import / export
// ---------------------------------------------------------------------------

#[test]
fn import_simple() {
    match only_stmt("import foo") {
        StmtKind::Import { path, alias: None } => assert_eq!(path, vec!["foo"]),
        other => panic!("{other:?}"),
    }
}

#[test]
fn import_with_alias() {
    match only_stmt("import foo as f") {
        StmtKind::Import { path, alias: Some(a) } => {
            assert_eq!(path, vec!["foo"]);
            assert_eq!(a, "f");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn export_stmt() {
    match only_stmt("export greet") {
        StmtKind::Export { name } => assert_eq!(name, "greet"),
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// If / match as expressions
// ---------------------------------------------------------------------------

#[test]
fn if_as_expr() {
    // `if` in expression position (rhs of let binding)
    match only_stmt("let r = if x > 0 { 1 } else { 0 }") {
        StmtKind::Let { init: Some(expr), .. } => {
            assert!(matches!(expr.kind, ExprKind::If { .. }));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn match_as_expr() {
    // `match` in expression position (rhs of let binding)
    let src = r#"let r = match n { 0 => "zero" _ => "nonzero" }"#;
    match only_stmt(src) {
        StmtKind::Let { init: Some(expr), .. } => {
            match expr.kind {
                ExprKind::Match { arms, .. } => assert_eq!(arms.len(), 2),
                other => panic!("{other:?}"),
            }
        }
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Real .aathu program snippets
// ---------------------------------------------------------------------------

#[test]
fn hello_world() {
    let src = r#"fn main() { print("Hello, World!") }"#;
    let prog = parse(src);
    assert_eq!(prog.stmts.len(), 1);
    match &prog.stmts[0].kind {
        StmtKind::Fn { name, body, .. } => {
            assert_eq!(name, "main");
            assert_eq!(body.stmts.len(), 1);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn fibonacci() {
    let src = "fn fib(n) { if n <= 1 { return n } return fib(n-1) + fib(n-2) }\nfn main() { for i in 0..20 { print(fib(i)) } }";
    let prog = parse(src);
    assert_eq!(prog.stmts.len(), 2);
}

#[test]
fn control_flow() {
    let src = "fn main() {\nlet x = 10\nif x > 5 { print(\"big\") } else { print(\"small\") }\n}";
    let prog = parse(src);
    assert_eq!(prog.stmts.len(), 1);
    match &prog.stmts[0].kind {
        StmtKind::Fn { body, .. } => assert_eq!(body.stmts.len(), 2),
        other => panic!("{other:?}"),
    }
}

#[test]
fn multiline_program() {
    let src = r#"
fn greet(name) {
    let msg = "Hello, " + name
    print(msg)
}

fn main() {
    greet("world")
    for i in 0..5 {
        print(i)
    }
}
"#;
    let prog = parse(src);
    assert_eq!(prog.stmts.len(), 2);
}

#[test]
fn async_spawn_stmt() {
    let src = "async fn worker() { }\nspawn worker()";
    let prog = parse(src);
    // async fn → StmtKind::Fn; spawn worker() → StmtKind::Expr(Call)
    assert_eq!(prog.stmts.len(), 2);
    assert!(matches!(prog.stmts[0].kind, StmtKind::Fn { .. }));
}

#[test]
fn block_expr() {
    match parse_expr_src("{ let x = 1\nx }") {
        ExprKind::Block(block) => assert_eq!(block.stmts.len(), 2),
        other => panic!("{other:?}"),
    }
}

#[test]
fn nested_calls() {
    let src = "print(to_str(42))";
    match parse_expr_src(src) {
        ExprKind::Call { args, .. } => {
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0].kind, ExprKind::Call { .. }));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn complex_expression() {
    // `a.b(1, 2)[0] + c * -d`
    let src = "a.b(1, 2)[0] + c * -d";
    // just verify it parses without error
    parse(src);
}

#[test]
fn multiple_stmts_semicolon() {
    let prog = parse("let a = 1; let b = 2; let c = 3");
    assert_eq!(prog.stmts.len(), 3);
}

#[test]
fn multiple_stmts_newline() {
    let prog = parse("let a = 1\nlet b = 2\nlet c = 3");
    assert_eq!(prog.stmts.len(), 3);
}

#[test]
fn empty_program() {
    let prog = parse("");
    assert!(prog.stmts.is_empty());
}
