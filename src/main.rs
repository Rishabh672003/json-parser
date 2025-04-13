mod lexer;
mod parser;

fn timeit<T, F: FnOnce() -> T>(label: &str, f: F) -> T {
    use std::time::Instant;
    let start = Instant::now();
    let result = f();
    let end = start.elapsed();
    println!("{label}: {:?}", end);
    result
}

fn main() {
    let mut argv = std::env::args();
    _ = argv.next();
    let file = argv.next().expect("No file was provided");

    let file_content = std::fs::read_to_string(file).unwrap();
    let toks = timeit("Tokenization", || lexer::tokenize(&file_content).unwrap());
    let _ans = timeit("Parsing", || parser::parse(&toks).unwrap());
}

#[cfg(test)]
mod test {
    use crate::lexer::*;
    use crate::parser::*;

    #[test]
    fn tokenize_true() {
        let a = "true";
        let b = tokenize(a).unwrap();
        let c = Token::True;
        assert_eq!(b[0], c)
    }
    #[test]
    fn tokenize_simple_json() {
        let input = r#"{"name": "value"}"#;
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::OpeningCurlyBrace,
                Token::StringLiteral("name".to_string()),
                Token::Colon,
                Token::StringLiteral("value".to_string()),
                Token::ClosingCurlyBrace
            ]
        );
    }
    #[test]
    fn tokenize_complex_json() {
        let input = r#"{"name": "value", "age": 30, "is_student": true}"#;
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::OpeningCurlyBrace,
                Token::StringLiteral("name".to_string()),
                Token::Colon,
                Token::StringLiteral("value".to_string()),
                Token::Comma,
                Token::StringLiteral("age".to_string()),
                Token::Colon,
                Token::Number(30.0),
                Token::Comma,
                Token::StringLiteral("is_student".to_string()),
                Token::Colon,
                Token::True,
                Token::ClosingCurlyBrace
            ]
        );
    }

    #[test]
    fn parse_empty_tokens() {
        let tokens = Vec::<Token>::new();
        let result = parse(&tokens);
        assert!(result.is_err());
    }

    #[test]
    fn parse_true() {
        let a = "true";
        let b = tokenize(a).unwrap();
        let c = parse(&b).unwrap();
        assert_eq!(c.entry, GrammarItem::Json)
    }

    #[test]
    fn parse_simple_json() {
        let tokens = vec![
            Token::OpeningCurlyBrace,
            Token::StringLiteral("name".to_string()),
            Token::Colon,
            Token::StringLiteral("value".to_string()),
            Token::ClosingCurlyBrace,
        ];
        let parse_node = parse(&tokens).unwrap();
        assert_eq!(parse_node.entry, GrammarItem::Json);
        assert_eq!(parse_node.children.len(), 1);
        assert_eq!(parse_node.children[0].entry, GrammarItem::Element);
    }
}
