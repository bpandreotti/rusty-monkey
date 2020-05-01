use super::*;

fn assert_parse(input: &str, expected: &[&str]) {
    let output = parse(input.into()).expect("Parser error during test");
    assert_eq!(output.len(), expected.len());
    for i in 0..output.len() {
        assert_eq!(format!("{:?}", output[i]), expected[i]);
    }
}

fn assert_parse_fails(input: &str) {
    assert!(parse(input.into()).is_err());
}

#[test]
fn test_literals() {
    let input = r#"
        0;
        17;
        true;
        false;
        nil;
        "brown is dark orange";
        "hello world";
        [];
        [0, false, nil];
        #{
            first : "entry",
            second : 1,
            nil : []
        };
        fn(x, y, z) {
            return x;
        };
    "#;
    let expected = [
        "ExpressionStatement(IntLiteral(0))",
        "ExpressionStatement(IntLiteral(17))",
        "ExpressionStatement(Boolean(true))",
        "ExpressionStatement(Boolean(false))",
        "ExpressionStatement(Nil)",
        "ExpressionStatement(StringLiteral(\"brown is dark orange\"))",
        "ExpressionStatement(StringLiteral(\"hello world\"))",
        "ExpressionStatement(ArrayLiteral([]))",
        "ExpressionStatement(ArrayLiteral([IntLiteral(0), Boolean(false), Nil]))",
        "ExpressionStatement(HashLiteral([(Identifier(\"first\"), StringLiteral(\"entry\")), \
        (Identifier(\"second\"), IntLiteral(1)), (Nil, ArrayLiteral([]))]))",
        "ExpressionStatement(FunctionLiteral { parameters: [\"x\", \"y\", \"z\"], body: \
        [Return(Identifier(\"x\"))] })",
    ];
    assert_parse(input, &expected);
    
    // Testing parser failures:
    // Arrays
    assert_parse_fails("[a, b");
    assert_parse_fails("[nil,]");
    assert_parse_fails("[,true]");

    // Hashes
    assert_parse_fails("#{ a: b,");
    assert_parse_fails("#{ a: b, c: d, }");
    assert_parse_fails("#{ a: b c: d }");
    assert_parse_fails("#{ a: }");
    assert_parse_fails("#{ a }");

    // Functions
    assert_parse_fails("fn() {");
    assert_parse_fails("fn( {}");
    assert_parse_fails("fn(x, y,) {}");
    assert_parse_fails("fn(,) {}");
    assert_parse_fails("fn {}");
    assert_parse_fails("fn()");
}

#[test]
fn test_call_expressions() {
    let input = "foo(); foo(x); foo(x, y, z); fn(x) { x; }(5);";
    let expected = [
        "ExpressionStatement(CallExpression { function: Identifier(\"foo\"), arguments: [] })",
        "ExpressionStatement(CallExpression { function: Identifier(\"foo\"), arguments: \
        [Identifier(\"x\")] })",
        "ExpressionStatement(CallExpression { function: Identifier(\"foo\"), arguments: \
        [Identifier(\"x\"), Identifier(\"y\"), Identifier(\"z\")] })",
        "ExpressionStatement(CallExpression { function: FunctionLiteral { parameters: \
        [\"x\"], body: [ExpressionStatement(Identifier(\"x\"))] }, arguments: \
        [IntLiteral(5)] })",
    ];
    assert_parse(input, &expected);

    assert_parse_fails("foo(x, y,)");
    assert_parse_fails("foo(");
    assert_parse_fails("foo(x y)");
}

#[test]
fn test_index_expression() {
    let input = "
        a[0];
        [nil][0];
    ";
    let expected = [
        "ExpressionStatement(IndexExpression(Identifier(\"a\"), IntLiteral(0)))",
        "ExpressionStatement(IndexExpression(ArrayLiteral([Nil]), IntLiteral(0)))",
    ];
    assert_parse(input, &expected);

    assert_parse_fails("array[]");
    assert_parse_fails("array[i");
    assert_parse_fails("array[only, one, index, man]");
}

#[test]
fn test_let_statements() {
    assert_parse(
        "let a = 1;",
        &["Let((\"a\", IntLiteral(1)))"],
    );
    assert_parse_fails("let 2 = 3;");
    assert_parse_fails("let foo whatever 3;");
    assert_parse_fails("let bar = ;");
    assert_parse_fails("let baz;");
}

#[test]
fn test_return_statements() {
    // Not much to test here, to be honest
    assert_parse("return 0;", &["Return(IntLiteral(0))"]);
    assert_parse("return;", &["Return(Nil)"]);
}

#[test]
fn test_prefix_expressions() {
    let input = "-5; !true; --!!-foo;";
    let expected = [
        "ExpressionStatement(PrefixExpression(Minus, IntLiteral(5)))",
        "ExpressionStatement(PrefixExpression(Bang, Boolean(true)))",
        "ExpressionStatement(PrefixExpression(Minus, PrefixExpression(Minus, PrefixExpression(\
        Bang, PrefixExpression(Bang, PrefixExpression(Minus, Identifier(\"foo\")))))))",
    ];
    assert_parse(input, &expected);

    assert_parse_fails("!;");
    assert_parse_fails("-");
}

#[test]
fn test_infix_expressions() {
    let input = "1 + 2; 4 * 5 - 2 / 3; 1 >= 2 == 2 < 3 != true;";
    let expected = [
        "ExpressionStatement(InfixExpression(IntLiteral(1), Plus, IntLiteral(2)))",
        "ExpressionStatement(InfixExpression(InfixExpression(IntLiteral(4), Asterisk, \
        IntLiteral(5)), Minus, InfixExpression(IntLiteral(2), Slash, IntLiteral(3))))",
        "ExpressionStatement(InfixExpression(InfixExpression(InfixExpression(IntLiteral(1), \
        GreaterEq, IntLiteral(2)), Equals, InfixExpression(IntLiteral(2), LessThan, \
        IntLiteral(3))), NotEquals, Boolean(true)))",
    ];
    assert_parse(input, &expected);

    assert_parse_fails("1 + 2 -");
    assert_parse_fails("1 == + 2");
    assert_parse_fails("> 1 + 2");
}

#[test]
fn test_if_expressions() {
    let input = "
        if 1 { 1 } else { 0 }
        if 2 { 2 }
        if (true) {}
        if nil {} else if nil {} else {}
    ";
    let expected = [
        "ExpressionStatement(IfExpression { condition: IntLiteral(1), consequence: \
        [ExpressionStatement(IntLiteral(1))], alternative: [ExpressionStatement(\
        IntLiteral(0))] })",
        "ExpressionStatement(IfExpression { condition: IntLiteral(2), consequence: \
        [ExpressionStatement(IntLiteral(2))], alternative: [] })",
        "ExpressionStatement(IfExpression { condition: Boolean(true), consequence: \
        [], alternative: [] })",
        "ExpressionStatement(IfExpression { condition: Nil, consequence: [], alternative: \
        [ExpressionStatement(IfExpression { condition: Nil, consequence: [], alternative: [] \
        })] })"
    ];
    assert_parse(input, &expected);

    assert_parse_fails("if true");
    assert_parse_fails("if { return 1; }");
    assert_parse_fails("if true {} else");
}

#[test]
fn test_grouped_expression() {
    let input = "(2 + 3) * (5 + 7); (1 + (1 + (1 + 1)));";
    let expected = [
        "ExpressionStatement(InfixExpression(InfixExpression(IntLiteral(2), Plus, \
        IntLiteral(3)), Asterisk, InfixExpression(IntLiteral(5), Plus, IntLiteral(7))))",
        "ExpressionStatement(InfixExpression(IntLiteral(1), Plus, InfixExpression(\
        IntLiteral(1), Plus, InfixExpression(IntLiteral(1), Plus, IntLiteral(1)))))",
    ];
    assert_parse(input, &expected);

    assert_parse_fails("(1 + 1");
    assert_parse_fails("1 + 1)");
    assert_parse_fails(")(");
}

#[test]
fn test_block_expressions() {
    let input = "
        { let foo = 2; return 1; }
        { return 0; }
        {}
    ";
    let expected = [
        "ExpressionStatement(BlockExpression([Let((\"foo\", IntLiteral(2))), \
        Return(IntLiteral(1))]))",
        "ExpressionStatement(BlockExpression([Return(IntLiteral(0))]))",
        "ExpressionStatement(BlockExpression([]))",
    ];
    assert_parse(input, &expected);

    assert_parse_fails("{ return 0");
}
