use crate::lexer::Token;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum GrammarItem<'a> {
    Json,
    Value,
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

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ParseNode<'a> {
    pub entry: GrammarItem<'a>,
    pub children: Vec<ParseNode<'a>>,
}

impl ParseNode<'_> {
    pub fn new(entry: GrammarItem) -> ParseNode {
        ParseNode {
            entry,
            children: Vec::new(),
        }
    }
}

pub fn parse(toks: &[Token]) -> Result<ParseNode, String> {
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
    let c = toks.get(pos);

    match c {
        Some(Token::OpeningCurlyBrace) => {
            let (parsenode, pos) = parse_object(toks, pos)?;
            Ok((parsenode, pos))
        }
        Some(Token::OpeningSquareBrace) => {
            let (parsenode, pos) = parse_array(toks, pos)?;
            Ok((parsenode, pos))
        }
        Some(Token::StringLiteral(val)) => Ok((ParseNode::new(GrammarItem::StrLit(val)), pos + 1)),
        Some(Token::Number(number)) => Ok((ParseNode::new(GrammarItem::Number(*number)), pos + 1)),
        Some(Token::True) => Ok((ParseNode::new(GrammarItem::Bool(true)), pos + 1)),
        Some(Token::False) => Ok((ParseNode::new(GrammarItem::Bool(false)), pos + 1)),
        Some(Token::Null) => Ok((ParseNode::new(GrammarItem::Null), pos + 1)),
        None => Err(format!("Expected a value, found None at position: {pos}")),
        _ => Err(format!("Invalid token: {:?} at potition: {pos}", c)),
    }
}

fn parse_object(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let mut node = ParseNode::new(GrammarItem::Object);
    if let Token::ClosingCurlyBrace = toks[pos + 1] {
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
        node.children.push(parsenode);
        Ok((node, pos + 1))
    }
}

fn parse_members(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let (parsenode, pos) = parse_member(toks, pos)?;
    let mut node = ParseNode::new(GrammarItem::Members);
    node.children.push(parsenode);
    let mut cur_pos = pos;
    while let Some(Token::Comma) = toks.get(cur_pos) {
        let (parsenode, p) = parse_member(toks, cur_pos + 1)?;
        node.children.push(parsenode);
        cur_pos = p;
    }
    Ok((node, cur_pos))
}

fn parse_member(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let Token::StringLiteral(ref cur_token) = toks[pos] else {
        return Err(format!(
            "invalid token while parsing stringliteral of member at pos: {pos} {:?}",
            toks.get(pos)
        ));
    };
    let pos = pos + 1;
    let Token::Colon = toks[pos] else {
        return Err(format!(
            "invalid token while parsing element of member at pos: {pos} {:?}",
            toks.get(pos)
        ));
    };
    let pos = pos + 1;
    let (parsenode, pos) = parse_element(toks, pos)?;
    let mut node = ParseNode::new(GrammarItem::Member(cur_token));
    node.children.push(parsenode);
    Ok((node, pos))
}

fn parse_array(toks: &[Token], pos: usize) -> Result<(ParseNode, usize), String> {
    let mut node = ParseNode::new(GrammarItem::Array);
    if let Token::ClosingSquareBrace = toks[pos + 1] {
        Ok((node, pos + 2))
    } else {
        let (parsenode, pos) = parse_elements(toks, pos + 1)?;
        let Token::ClosingSquareBrace = toks[pos] else {
            return Err(format!(
                "invalid token while parsing array at pos: {pos} {:?}",
                toks.get(pos)
            ));
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
    while let Some(Token::Comma) = toks.get(cur_pos) {
        let (parsenode, p) = parse_element(toks, cur_pos + 1)?;
        node.children.push(parsenode);
        cur_pos = p;
    }
    Ok((node, cur_pos))
}
