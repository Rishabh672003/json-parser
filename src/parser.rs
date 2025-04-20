use crate::lexer::{Token, TokenType};

#[derive(Debug, PartialEq, PartialOrd)]
pub enum GrammarItem<'a> {
    Json,
    Object,
    Member(&'a str),
    Members,
    Array,
    Element,
    Elements,
    Number(f64),
    Bool(bool),
    StrLit(&'a str),
    Null,
}

use std::fmt;

impl fmt::Display for ParseNode<'_> {
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

impl fmt::Display for GrammarItem<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GrammarItem::Json => write!(f, "Json "),
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

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ParseNode<'a> {
    pub entry: GrammarItem<'a>,
    pub children: Vec<ParseNode<'a>>,
}

impl ParseNode<'_> {
    pub fn new(entry: GrammarItem) -> ParseNode {
        ParseNode {
            entry,
            children: Vec::with_capacity(1),
        }
    }
}

#[inline]
fn error_expected(expected: &str, toks: &[Token], pos: usize) -> String {
    format!(
        "ERROR: Expected `{expected}`, Got: {}, at Line: {}, Col: {}",
        toks[pos].tok, toks[pos].line, toks[pos].bol
    )
}

pub fn parse(toks: &[Token]) -> Result<ParseNode, String> {
    parse_json(toks, 0).and_then(|(n, i)| {
        if i == toks.len() {
            Ok(n)
        } else {
            Err(format!(
                "ERROR: Expected end of input, Got: {}, at Line: {}, Col: {}",
                toks[i].tok, toks[i].line, toks[i].bol
            ))
        }
    })
}

fn parse_json(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let (parsenode, pos) = parse_element(toks, pos)?;
    let mut node = ParseNode::new(GrammarItem::Json);
    node.children.push(parsenode);
    Ok((node, pos))
}

fn parse_element(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let (parsenode, pos) = parse_value(toks, pos)?;
    let mut node = ParseNode::new(GrammarItem::Element);
    node.children.push(parsenode);
    Ok((node, pos))
}

fn parse_value(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let c = toks.get(pos).unwrap();

    match &c.tok {
        TokenType::OpeningCurlyBrace => {
            let (parsenode, pos) = parse_object(toks, pos)?;
            Ok((parsenode, pos))
        }
        TokenType::OpeningSquareBrace => {
            let (parsenode, pos) = parse_array(toks, pos)?;
            Ok((parsenode, pos))
        }
        TokenType::StringLiteral(val) => Ok((ParseNode::new(GrammarItem::StrLit(val)), pos + 1)),
        TokenType::Number(number) => Ok((ParseNode::new(GrammarItem::Number(*number)), pos + 1)),
        TokenType::True => Ok((ParseNode::new(GrammarItem::Bool(true)), pos + 1)),
        TokenType::False => Ok((ParseNode::new(GrammarItem::Bool(false)), pos + 1)),
        TokenType::Null => Ok((ParseNode::new(GrammarItem::Null), pos + 1)),
        _ => Err(format!(
            "ERROR: Invalid token, Got: {}, at Line: {}, Col: {}",
            c.tok, c.line, c.bol
        )),
    }
}

fn parse_object(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let mut node = ParseNode::new(GrammarItem::Object);
    if let TokenType::ClosingCurlyBrace = toks[pos + 1].tok {
        Ok((node, pos + 2))
    } else {
        let (parsenode, pos) = parse_members(toks, pos + 1)?;
        let TokenType::ClosingCurlyBrace = toks[pos].tok else {
            match toks.get(pos) {
                Some(val) => match val.tok {
                    TokenType::StringLiteral(_) => {
                        return Err(error_expected(",", toks, pos));
                    }
                    _ => {
                        return Err(format!(
                            "ERROR: Invalid Token, Got: {}, at Line: {}, Col: {}",
                            toks[pos].tok, toks[pos].line, toks[pos].bol
                        ));
                    }
                },
                None => {
                    return Err(error_expected("}}", toks, pos));
                }
            }
        };
        node.children.push(parsenode);
        Ok((node, pos + 1))
    }
}

fn parse_members(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let (parsenode, pos) = parse_member(toks, pos)?;
    let mut node = ParseNode::new(GrammarItem::Members);
    node.children.push(parsenode);
    let mut cur_pos = pos;
    while let TokenType::Comma = toks.get(cur_pos).unwrap().tok {
        let (parsenode, p) = parse_member(toks, cur_pos + 1)?;
        node.children.push(parsenode);
        cur_pos = p;
    }
    Ok((node, cur_pos))
}

fn parse_member(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let TokenType::StringLiteral(ref cur_token) = toks[pos].tok else {
        return Err(error_expected("StringLiteral", toks, pos));
    };

    let pos = pos + 1;
    let TokenType::Colon = toks[pos].tok else {
        return Err(error_expected(":", toks, pos));
    };

    let pos = pos + 1;
    let (parsenode, pos) = parse_element(toks, pos)?;
    let mut node = ParseNode::new(GrammarItem::Member(cur_token));
    node.children.push(parsenode);
    Ok((node, pos))
}

fn parse_array(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let mut node = ParseNode::new(GrammarItem::Array);
    if let TokenType::ClosingSquareBrace = toks[pos + 1].tok {
        Ok((node, pos + 2))
    } else {
        let (parsenode, pos) = parse_elements(toks, pos + 1)?;
        let TokenType::ClosingSquareBrace = toks[pos].tok else {
            match toks.get(pos) {
                Some(val) => match val.tok {
                    TokenType::StringLiteral(_) => {
                        return Err(error_expected(",", toks, pos));
                    }
                    _ => {
                        return Err(error_expected("Token", toks, pos));
                    }
                },
                None => {
                    return Err(error_expected("]", toks, pos));
                }
            }
        };
        node.children.push(parsenode);
        Ok((node, pos + 1))
    }
}

fn parse_elements(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let (parsenode, pos) = parse_element(toks, pos)?;
    let mut node = ParseNode::new(GrammarItem::Elements);
    node.children.push(parsenode);
    let mut cur_pos = pos;
    while let TokenType::Comma = toks.get(cur_pos).unwrap().tok {
        let (parsenode, p) = parse_element(toks, cur_pos + 1)?;
        node.children.push(parsenode);
        cur_pos = p;
    }
    Ok((node, cur_pos))
}
