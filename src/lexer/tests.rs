use super::token::Token;
use super::*;

// Shortcut to create a `Token::Identifier` from a string literal
macro_rules! iden {
    ($x:expr) => {
        Token::Identifier($x.into())
    };
}

fn assert_lex(input: &str, expected: &[Token]) {
    let mut lex = Lexer::from_string(input.into()).expect("Lexer error during test");
    for ex in expected {
        let got = lex.next_token().expect("Lexer error during test");
        assert_eq!(ex, &got);
    }
}

fn assert_lexer_error(input: &str, expected_error: LexerError) {
    let mut lex = Lexer::from_string(input.into()).unwrap();
    loop {
        match lex.next_token() {
            Ok(Token::EOF) => panic!("No lexer errors encountered"),
            Err(e) => {
                match e.error {
                    ErrorType::Lexer(got) => assert_eq!(expected_error, got),
                    _ => panic!("Wrong error type"),
                }
                return;
            }
            _ => continue,
        }
    }
}

#[test]
fn test_identifiers() {
    let input = "foo bar two_words _ _foo2 back2thefuture 3different_ones olá 統一碼 यूनिकोड";
    let expected = [
        iden!("foo"),
        iden!("bar"),
        iden!("two_words"),
        iden!("_"),
        iden!("_foo2"),
        iden!("back2thefuture"),
        Token::Int(3),
        iden!("different_ones"),
        iden!("olá"),
        iden!("統一碼"),
        iden!("यूनिकोड"),
        Token::EOF,
    ];
    assert_lex(input, &expected);

    // Test keywords
    let input = "fn let true false if else return nil";
    let expected = [
        Token::Function,
        Token::Let,
        Token::True,
        Token::False,
        Token::If,
        Token::Else,
        Token::Return,
        Token::Nil,
        Token::EOF,
    ];
    assert_lex(input, &expected);
}

#[test]
fn test_int_literals() {
    let input = "0 1729 808017424794 1_000_000 1___0____2 _1_000_000";
    let expected = [
        Token::Int(0),
        Token::Int(1729),
        Token::Int(808_017_424_794),
        Token::Int(1_000_000),
        Token::Int(102),
        iden!("_1_000_000"),
        Token::EOF,
    ];
    assert_lex(input, &expected);
}

#[test]
fn test_strings() {
    let input = r#"
        "string"
        "escape sequences: \\ \n \t \r \" "
        "whitespace
        inside strings"
    "#;
    let expected = [
        Token::Str("string".into()),
        Token::Str("escape sequences: \\ \n \t \r \" ".into()),
        Token::Str("whitespace\n        inside strings".into()),
        Token::EOF,
    ];
    assert_lex(input, &expected);
}

#[test]
fn test_operators() {
    let input = "= ! + - * / ^ % < > == != <= >=";
    let expected = [
        Token::Assign,
        Token::Bang,
        Token::Plus,
        Token::Minus,
        Token::Asterisk,
        Token::Slash,
        Token::Exponent,
        Token::Modulo,
        Token::LessThan,
        Token::GreaterThan,
        Token::Equals,
        Token::NotEquals,
        Token::LessEq,
        Token::GreaterEq,
        Token::EOF,
    ];
    assert_lex(input, &expected);
}

#[test]
fn test_delimiters() {
    let input = ", ; : () {} [] #{}";
    let expected = [
        Token::Comma,
        Token::Semicolon,
        Token::Colon,
        Token::OpenParen,
        Token::CloseParen,
        Token::OpenCurlyBrace,
        Token::CloseCurlyBrace,
        Token::OpenSquareBracket,
        Token::CloseSquareBracket,
        Token::OpenHash,
        Token::CloseCurlyBrace,
        Token::EOF,
    ];
    assert_lex(input, &expected);
}

#[test]
fn test_comments() {
    let input = r"
        // comments
        foo // bar
        // Unicode! 中文 Português हिन्दी Français Español
        //
        baz
    ";
    let expected = [iden!("foo"), iden!("baz"), Token::EOF];
    assert_lex(input, &expected);
}

#[test]
fn test_large_program() {
    use Token::*;
    let input = r#"
        let fizzbuzz = fn(i) {
            let result = if i % 3 == 0 {
                "fizz"
            } else {
                ""
            };
            let result = if i % 5 == 0 {
                result + "buzz"
            } else {
                result
            };
            if result != "" {
                puts(result)
            } else {
                puts(i)
            }
        }
        map(fizzbuzz, range(1, 100));
    "#;
    let expected = [
        // Line 1
        Let,
        iden!("fizzbuzz"),
        Assign,
        Function,
        OpenParen,
        iden!("i"),
        CloseParen,
        OpenCurlyBrace,
        // Line 2
        Let,
        iden!("result"),
        Assign,
        If,
        iden!("i"),
        Modulo,
        Int(3),
        Equals,
        Int(0),
        OpenCurlyBrace,
        // Line 3
        Str("fizz".into()),
        // Line 4
        CloseCurlyBrace,
        Else,
        OpenCurlyBrace,
        // Line 5
        Str("".into()),
        // Line 6
        CloseCurlyBrace,
        Semicolon,
        // Line 7
        Let,
        iden!("result"),
        Assign,
        If,
        iden!("i"),
        Modulo,
        Int(5),
        Equals,
        Int(0),
        OpenCurlyBrace,
        // Line 8
        iden!("result"),
        Plus,
        Str("buzz".into()),
        // Line 9
        CloseCurlyBrace,
        Else,
        OpenCurlyBrace,
        // Line 10
        iden!("result"),
        // Line 11
        CloseCurlyBrace,
        Semicolon,
        // Line 12
        If,
        iden!("result"),
        NotEquals,
        Str("".into()),
        OpenCurlyBrace,
        // Line 13
        iden!("puts"),
        OpenParen,
        iden!("result"),
        CloseParen,
        // Line 14
        CloseCurlyBrace,
        Else,
        OpenCurlyBrace,
        // Line 15
        iden!("puts"),
        OpenParen,
        iden!("i"),
        CloseParen,
        // Line 16
        CloseCurlyBrace,
        // Line 17
        CloseCurlyBrace,
        // Line 18
        iden!("map"),
        OpenParen,
        iden!("fizzbuzz"),
        Comma,
        iden!("range"),
        OpenParen,
        Int(1),
        Comma,
        Int(100),
        CloseParen,
        CloseParen,
        Semicolon,
        EOF,
    ];
    assert_lex(input, &expected);
}

#[test]
fn test_lexer_position() {
    let program = "first line\nsecond\n3";
    let expected = [
        // (token, current_position, token_position)
        (iden!("first"), (1, 6), (1, 1)),
        (iden!("line"), (1, 11), (1, 7)),
        (iden!("second"), (2, 7), (2, 1)),
        (Token::Int(3), (3, 2), (3, 1)),
        (Token::EOF, (3, 2), (3, 2)),
    ];

    let mut lex = Lexer::from_string(program.into()).unwrap();
    assert_eq!((1, 1), lex.current_position);
    assert_eq!((0, 0), lex.token_position);

    for (tk, current_pos, token_pos) in &expected {
        let got = lex.next_token().unwrap();
        assert_eq!(tk, &got);
        assert_eq!(current_pos, &lex.current_position);
        assert_eq!(token_pos, &lex.token_position);
    }
}

#[test]
fn test_lexer_errors() {
    assert_lexer_error(
        r#" "some string that doesn't end "#,
        LexerError::UnexpectedEOF,
    );
    assert_lexer_error(
        r#" "i don't know this guy: \w" "#,
        LexerError::UnknownEscapeSequence('w'),
    );
    assert_lexer_error(
        r#" "whats up with this weird symbol:" & "#,
        LexerError::IllegalChar('&'),
    );
}
