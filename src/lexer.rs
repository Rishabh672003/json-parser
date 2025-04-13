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
                c if c.is_whitespace() => {
                    chars.next();
                    continue;
                }
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
                    let mut v = String::with_capacity(32);
                    let mut escaping = false;

                    while let Some(&c) = chars.peek() {
                        if !('\u{0020}'..='\u{10FFFF}').contains(&c) {
                            return Err(format!("Invalid Character: {c}, {}", v));
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
                    toks.push(Token::StringLiteral(v));
                }
                // jsonc styled comments
                '/' => {
                    chars.next();
                    if chars.peek().is_some_and(|x| *x == '/') {
                        chars.next();
                        while let Some(&c) = chars.peek() {
                            if c == '\n' {
                                break;
                            }
                            chars.next();
                        }
                    } else if chars.peek().is_some_and(|x| *x == '*') {
                        let mut asterisk = false;
                        while let Some(&c) = chars.peek() {
                            if c == '*' {
                                asterisk = true;
                            }
                            if asterisk && c == '/'{
                                break;
                            }
                            chars.next();
                        }
                    } else {
                        return Err(format!("Invalid comment: {}", c));
                    }
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
                _ => return Err(format!("Bare strings are not allowed: {c}")),
            }
        }
        Ok(toks)
    }
