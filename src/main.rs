pub mod lexer {
    struct PeekWhile<'a, I, F>
    where
        I: Iterator + 'a,
    {
        iter: &'a mut std::iter::Peekable<I>,
        f: F,
    }

    impl<'a, I, F> Iterator for PeekWhile<'a, I, F>
    where
        I: Iterator + 'a,
        F: for<'b> FnMut(&'b <I as Iterator>::Item) -> bool,
    {
        type Item = <I as Iterator>::Item;
        fn next(&mut self) -> Option<<Self as Iterator>::Item> {
            let &mut PeekWhile {
                ref mut iter,
                ref mut f,
            } = self;
            if iter.peek().map(f).unwrap_or(false) {
                iter.next()
            } else {
                None
            }
        }
    }

    fn peek_while<'a, I, F>(iter: &'a mut std::iter::Peekable<I>, f: F) -> PeekWhile<'a, I, F>
    where
        I: Iterator + 'a,
        F: for<'b> FnMut(&'b <I as Iterator>::Item) -> bool,
    {
        PeekWhile { iter, f }
    }

    #[derive(Debug, PartialEq, PartialOrd)]
    pub enum Token {
        OpeningCurlyBrace,
        ClosingCurlyBrace,
        OpeningSquareBrace,
        ClosingSquareBrace,
        StringLiteral(String),
        Number(f64),
        True,
        False,
        Null,
        Colon,
        Comma,
    }

    pub fn tokenize(stream: &str) -> Result<Vec<Token>, String> {
        let mut toks = Vec::with_capacity(stream.len());
        let mut chars = stream.chars().peekable();
        while let Some(&c) = chars.peek() {
            match c {
                '{' => {
                    toks.push(Token::OpeningCurlyBrace);
                    chars.next();
                }
                '}' => {
                    toks.push(Token::ClosingCurlyBrace);
                    chars.next();
                }
                '"' => {
                    chars.next();
                    let mut v = Vec::new();
                    while let Some(&c) = chars.peek() {
                        if !('\u{0020}'..='\u{10FFFF}').contains(&c) {
                            return Err(format!("Invalid Character: {c}"));
                        }
                        let mut escaping = false;
                        if c == '\\' && v.last() != Some(&'\\') {
                            escaping = true;
                        }
                        if v.last().is_some() && v.last() == Some(&'\\') && escaping {
                            match c {
                                '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u' | '"' => {
                                    v.push(c);
                                    escaping = false;
                                    chars.next();
                                }
                                _ => {
                                    return Err(format!(
                                        "Invalid Escape sequence: {c}, {}",
                                        v.iter().collect::<String>()
                                    ));
                                }
                            }
                        } else if c != '"' || (v.last().is_some() && *v.last().unwrap() == '\\') {
                            v.push(c);
                            escaping = false;
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    toks.push(Token::StringLiteral(v.iter().collect()));
                    v.clear();
                    chars.next();
                }
                ':' => {
                    toks.push(Token::Colon);
                    chars.next();
                }
                '[' => {
                    toks.push(Token::OpeningSquareBrace);
                    chars.next();
                }
                ',' => {
                    toks.push(Token::Comma);
                    chars.next();
                }
                ']' => {
                    toks.push(Token::ClosingSquareBrace);
                    chars.next();
                }
                't' => {
                    let tr = peek_while(&mut chars, |c| c.is_alphabetic()).collect::<String>();
                    if tr == "true" {
                        toks.push(Token::True);
                    } else {
                        return Err(format!("Invalid value: {tr}; Expected: true"));
                    }
                }
                'f' => {
                    let fa = peek_while(&mut chars, |c| c.is_alphabetic()).collect::<String>();
                    if fa == "false" {
                        toks.push(Token::False);
                    } else {
                        return Err(format!("Invalid value: {fa}; Expected: false"));
                    }
                }
                'n' => {
                    let nu = peek_while(&mut chars, |c| c.is_alphabetic()).collect::<String>();
                    if nu == "null" {
                        toks.push(Token::Null);
                    } else {
                        return Err(format!("Invalid value: {nu}; Expected: null"));
                    }
                }
                '-' | '0'..='9' => {
                    let digits = peek_while(&mut chars, |c| {
                        c.is_numeric()
                            || *c == '.'
                            || *c == 'e'
                            || *c == 'E'
                            || *c == '-'
                            || *c == '+'
                    })
                    .collect::<String>();
                    toks.push(Token::Number(
                        digits
                            .parse()
                            .unwrap_or_else(|_| panic!("Parsing to Number failed: {digits}")),
                    ));
                }
                c if c.is_whitespace() => {
                    chars.next();
                }
                _ => return Err(format!("Bare strings are not allowed: {c}")),
                // _ => {
                //     chars.next();
                // }
            }
        }
        Ok(toks)
    }
}

pub mod parser {
    use crate::lexer::Token;

    #[derive(Debug, Clone, PartialEq, PartialOrd)]
    pub enum GrammarItem {
        Json,
        Value,
        Object,
        Member(String),
        Members,
        Array,
        Element,
        Elements,
        Number(f64),
        Bool(bool),
        StrLit(String),
        Null,
    }

    use std::fmt;

    impl fmt::Display for ParseNode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.entry)?;
            write!(f, "{{")?;
            for child in &self.children {
                write!(f, "{} ", child)?;
            }
            write!(f, "}}")?;
            Ok(())
        }
    }

    impl fmt::Display for GrammarItem {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                GrammarItem::Json => write!(f, "Json "),
                GrammarItem::Value => write!(f, "Value "),
                GrammarItem::Object => write!(f, "Object "),
                GrammarItem::Member(name) => write!(f, "Member({}) ", name),
                GrammarItem::Members => write!(f, "Members "),
                GrammarItem::Array => write!(f, "Array "),
                GrammarItem::Element => write!(f, "Element "),
                GrammarItem::Elements => write!(f, "Elements "),
                GrammarItem::Number(num) => write!(f, "Number({}) ", num),
                GrammarItem::Bool(val) => write!(f, "Bool({}) ", val),
                GrammarItem::StrLit(val) => write!(f, "StrLit({}) ", val),
                GrammarItem::Null => write!(f, "Null "),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, PartialOrd)]
    pub struct ParseNode {
        pub entry: GrammarItem,
        pub children: Vec<ParseNode>,
    }

    impl ParseNode {
        pub fn new(entry: GrammarItem) -> ParseNode {
            ParseNode {
                entry,
                children: Vec::new(),
            }
        }
    }

    pub fn parse(toks: &Vec<Token>) -> Result<ParseNode, String> {
        parse_json(toks, 0).and_then(|(n, i)| {
            if i == toks.len() {
                Ok(n)
            } else {
                Err(format!(
                    "Expected end of input, found {:?} at {}",
                    toks[i], i
                ))
            }
        })
    }

    fn parse_json(toks: &Vec<Token>, pos: usize) -> Result<(ParseNode, usize), String> {
        let (parsenode, pos) = parse_element(toks, pos)?;
        let mut node = ParseNode::new(GrammarItem::Json);
        node.children.push(parsenode.clone());
        Ok((node, pos))
    }

    fn parse_element(toks: &Vec<Token>, pos: usize) -> Result<(ParseNode, usize), String> {
        let (parsenode, pos) = parse_value(toks, pos)?;
        let mut node = ParseNode::new(GrammarItem::Element);
        node.children.push(parsenode.clone());
        Ok((node, pos))
    }

    fn parse_value(toks: &Vec<Token>, pos: usize) -> Result<(ParseNode, usize), String> {
        let c = toks
            .get(pos)
            .ok_or_else(|| format!("Expected A value, found None, at position: {pos}"))?;

        match c {
            Token::OpeningCurlyBrace => {
                let (parsenode, pos) = parse_object(toks, pos)?;
                Ok((parsenode, pos))
            }
            Token::OpeningSquareBrace => {
                let (parsenode, pos) = parse_array(toks, pos)?;
                Ok((parsenode, pos))
            }
            Token::StringLiteral(val) => Ok((
                ParseNode::new(GrammarItem::StrLit(val.to_string())),
                pos + 1,
            )),
            Token::Number(number) => Ok((ParseNode::new(GrammarItem::Number(*number)), pos + 1)),
            Token::True => Ok((ParseNode::new(GrammarItem::Bool(true)), pos + 1)),
            Token::False => Ok((ParseNode::new(GrammarItem::Bool(false)), pos + 1)),
            Token::Null => Ok((ParseNode::new(GrammarItem::Null), pos + 1)),
            _ => Err(format!("Invalid token: {:?} at potition: {pos}", c)),
        }
    }

    fn parse_object(toks: &Vec<Token>, pos: usize) -> Result<(ParseNode, usize), String> {
        let mut node = ParseNode::new(GrammarItem::Object);
        if let Some(Token::ClosingCurlyBrace) = toks.get(pos + 1) {
            Ok((node, pos + 2))
        } else {
            let (parsenode, pos) = parse_members(toks, pos + 1)?;
            let Token::ClosingCurlyBrace = toks
                .get(pos)
                .ok_or_else(|| "Unexpected End of input".to_string())?
            else {
                return Err(format!(
                    "invalid token while parsing object at pos: {pos} {:?}",
                    toks.get(pos)
                ));
            };
            node.children.push(parsenode.clone());
            Ok((node, pos + 1))
        }
    }

    fn parse_members(toks: &Vec<Token>, pos: usize) -> Result<(ParseNode, usize), String> {
        let (parsenode, pos) = parse_member(toks, pos)?;
        let mut node = ParseNode::new(GrammarItem::Members);
        node.children.push(parsenode.clone());
        let mut cur_pos = pos;
        while let Some(Token::Comma) = toks.get(cur_pos) {
            let (parsenode, p) = parse_member(toks, cur_pos + 1)?;
            node.children.push(parsenode.clone());
            cur_pos = p;
        }
        Ok((node, cur_pos))
    }

    fn parse_member(toks: &Vec<Token>, pos: usize) -> Result<(ParseNode, usize), String> {
        let Token::StringLiteral(cur_token) = toks
            .get(pos)
            .ok_or_else(|| "Unexpected End of input".to_string())?
        else {
            return Err(format!(
                "invalid token while parsing stringliteral of member at pos: {pos} {:?}",
                toks.get(pos)
            ));
        };
        let pos = pos + 1;
        let Token::Colon = toks
            .get(pos)
            .ok_or_else(|| "Unexpected End of input".to_string())?
        else {
            return Err(format!(
                "invalid token while parsing element of member at pos: {pos} {:?}",
                toks.get(pos)
            ));
        };
        let pos = pos + 1;
        let (parsenode, pos) = parse_element(toks, pos)?;
        let mut node = ParseNode::new(GrammarItem::Member(cur_token.to_owned()));
        node.children.push(parsenode.clone());
        Ok((node, pos))
    }

    fn parse_array(toks: &Vec<Token>, pos: usize) -> Result<(ParseNode, usize), String> {
        let mut node = ParseNode::new(GrammarItem::Array);
        if let Some(Token::ClosingSquareBrace) = toks.get(pos + 1) {
            Ok((node, pos + 2))
        } else {
            let (parsenode, pos) = parse_elements(toks, pos + 1)?;
            let Token::ClosingSquareBrace = toks
                .get(pos)
                .ok_or_else(|| "Unexpected End of input".to_string())?
            else {
                return Err(format!(
                    "invalid token while parsing array at pos: {pos} {:?}",
                    toks.get(pos)
                ));
            };
            node.children.push(parsenode.clone());
            Ok((node, pos + 1))
        }
    }

    fn parse_elements(toks: &Vec<Token>, pos: usize) -> Result<(ParseNode, usize), String> {
        let (parsenode, pos) = parse_element(toks, pos)?;
        let mut node = ParseNode::new(GrammarItem::Elements);
        node.children.push(parsenode.clone());
        let mut cur_pos = pos;
        while let Some(Token::Comma) = toks.get(cur_pos) {
            let (parsenode, p) = parse_element(toks, cur_pos + 1)?;
            node.children.push(parsenode.clone());
            cur_pos = p;
        }
        Ok((node, cur_pos))
    }
}

fn main() {
    let mut argv = std::env::args();
    _ = argv.next();
    let file = argv.next().expect("No file was provided");

    let file_content = std::fs::read_to_string(file).unwrap();
    let toks = lexer::tokenize(&file_content).unwrap();
    println!("{:?}", toks);
    let ans = parser::parse(&toks).unwrap();
    println!("{}", ans);
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
