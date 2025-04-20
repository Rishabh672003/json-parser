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
pub enum TokenType {
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

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::OpeningCurlyBrace => write!(f, "`{{`"),
            TokenType::ClosingCurlyBrace => write!(f, "`}}`"),
            TokenType::OpeningSquareBrace => write!(f, "`[`"),
            TokenType::ClosingSquareBrace => write!(f, "`]`"),
            TokenType::StringLiteral(_) => write!(f, "StringLiteral"),
            TokenType::Number(_) => write!(f, "Number"),
            TokenType::True => write!(f, "`true`"),
            TokenType::False => write!(f, "`false`"),
            TokenType::Null => write!(f, "`null`"),
            TokenType::Colon => write!(f, "`:`"),
            TokenType::Comma => write!(f, "`,`"),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Token {
    pub tok: TokenType,
    pub line: usize,
    pub bol: usize,
}

pub fn tokenize(stream: &str) -> Result<Vec<Token>, String> {
    use TokenType::*;
    let mut toks = Vec::with_capacity(stream.len());
    let mut chars = stream.chars().peekable();
    let mut line = 1;
    let mut bol = 0;
    while let Some(&c) = chars.peek() {
        bol += 1;
        match c {
            '\n' => {
                line += 1;
                bol = 0;
                chars.next();
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            '{' => {
                toks.push(Token {
                    tok: OpeningCurlyBrace,
                    line,
                    bol,
                });
                chars.next();
            }
            '}' => {
                toks.push(Token {
                    tok: ClosingCurlyBrace,
                    line,
                    bol,
                });
                chars.next();
            }
            '"' => {
                chars.next();
                let mut v = String::with_capacity(32);
                let mut escaping = false;

                while let Some(&c) = chars.peek() {
                    if !('\u{0020}'..='\u{10FFFF}').contains(&c) {
                        return Err(format!(
                            "Invalid Character: {c}, Got String: {}, at Line: {}, Col: {}",
                            v, line, bol
                        ));
                    }
                    chars.next();
                    if escaping {
                        match c {
                            '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u' | '"' => {
                                v.push(c);
                            }
                            _ => {
                                return Err(format!("Invalid Escape sequence: {c}, {}", v));
                            }
                        }
                        escaping = false;
                    } else if c == '\\' {
                        v.push(c);
                        escaping = true;
                    } else if c == '"' {
                        break;
                    } else {
                        v.push(c);
                    }
                }

                if escaping {
                    return Err(format!("Unterminated escape sequence: {}", v));
                }
                let count = v.len();
                toks.push(Token {
                    tok: StringLiteral(v),
                    line,
                    bol,
                });
                bol += count + 1;
            }
            ':' => {
                toks.push(Token {
                    tok: Colon,
                    line,
                    bol,
                });
                chars.next();
            }
            '[' => {
                toks.push(Token {
                    tok: OpeningSquareBrace,
                    line,
                    bol,
                });
                chars.next();
            }
            ',' => {
                toks.push(Token {
                    tok: Comma,
                    line,
                    bol,
                });
                chars.next();
            }
            ']' => {
                toks.push(Token {
                    tok: ClosingSquareBrace,
                    line,
                    bol,
                });
                chars.next();
            }
            't' => {
                let tr = peek_while(&mut chars, |c| c.is_alphabetic()).collect::<String>();
                if tr == "true" {
                    bol += 3;
                    toks.push(Token {
                        tok: True,
                        line,
                        bol,
                    });
                } else {
                    return Err(format!("Invalid value: {tr}; Expected: true"));
                }
            }
            'f' => {
                let fa = peek_while(&mut chars, |c| c.is_alphabetic()).collect::<String>();
                if fa == "false" {
                    bol += 4;
                    toks.push(Token {
                        tok: False,
                        line,
                        bol,
                    });
                } else {
                    return Err(format!("Invalid value: {fa}; Expected: false"));
                }
            }
            'n' => {
                let nu = peek_while(&mut chars, |c| c.is_alphabetic()).collect::<String>();
                if nu == "null" {
                    bol += 3;
                    toks.push(Token {
                        tok: Null,
                        line,
                        bol,
                    });
                } else {
                    return Err(format!("Invalid value: {nu}; Expected: null"));
                }
            }
            '-' | '0'..='9' => {
                let digits = peek_while(&mut chars, |c| {
                    c.is_numeric() || *c == '.' || *c == 'e' || *c == 'E' || *c == '-' || *c == '+'
                })
                .collect::<String>();
                toks.push(Token {
                    tok: Number(
                        digits
                            .parse()
                            .unwrap_or_else(|_| panic!("Parsing to Number failed: {digits}")),
                    ),
                    line,
                    bol,
                });
            }
            _ => return Err(format!("Bare strings are not allowed: {c}")),
        }
    }
    Ok(toks)
}
