use crate::Token;
use crate::TokenType;

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut line = 1;

    while let Some(c) = chars.next() {
        match c {
            ' ' | '\r' | '\t' => continue,
            '\n' => line += 1,
            'f' if matches_keyword(&mut chars, "unc") => tokens.push(Token { typ: TokenType::Func, lexeme: "func".to_string() }),
            'l' if matches_keyword(&mut chars, "et") => tokens.push(Token { typ: TokenType::Let, lexeme: "let".to_string() }),
            'i' if matches_keyword(&mut chars, "f") => tokens.push(Token { typ: TokenType::If, lexeme: "if".to_string() }),
            'e' if matches_keyword(&mut chars, "lse") => tokens.push(Token { typ: TokenType::Else, lexeme: "else".to_string() }),
            'w' if matches_keyword(&mut chars, "hile") => tokens.push(Token { typ: TokenType::While, lexeme: "while".to_string() }),
            'f' if matches_keyword(&mut chars, "or") => tokens.push(Token { typ: TokenType::For, lexeme: "for".to_string() }),
            'r' if matches_keyword(&mut chars, "eturn") => tokens.push(Token { typ: TokenType::Return, lexeme: "return".to_string() }),
            'w' if matches_keyword(&mut chars, "rite") => tokens.push(Token { typ: TokenType::Write, lexeme: "write".to_string() }),
            't' if matches_keyword(&mut chars, "rue") => tokens.push(Token { typ: TokenType::True, lexeme: "true".to_string() }),
            'f' if matches_keyword(&mut chars, "alse") => tokens.push(Token { typ: TokenType::False, lexeme: "false".to_string() }),
            '+' => tokens.push(Token { typ: TokenType::Plus, lexeme: "+".to_string() }),
            '-' => {
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token { typ: TokenType::Arrow, lexeme: "->".to_string() });
                } else {
                    tokens.push(Token { typ: TokenType::Minus, lexeme: "-".to_string() });
                }
            },
            '*' => tokens.push(Token { typ: TokenType::Star, lexeme: "*".to_string() }),
            '/' => tokens.push(Token { typ: TokenType::Slash, lexeme: "/".to_string() }),
            '%' => tokens.push(Token { typ: TokenType::Mod, lexeme: "%".to_string() }),
            '=' if chars.peek() == Some(&'=') => {
                chars.next();
                tokens.push(Token { typ: TokenType::EqualEqual, lexeme: "==".to_string() });
            }
            '!' if chars.peek() == Some(&'=') => {
                chars.next();
                tokens.push(Token { typ: TokenType::BangEqual, lexeme: "!=".to_string() });
            } else {
                tokens.push(Token { typ: TokenType::Bang, lexeme: "!".to_string() });
            }
            '<' if chars.peek() == Some(&'=') => {
                chars.next();
                tokens.push(Token { typ: TokenType::LessEqual, lexeme: "<=".to_string() });
            } else {
                tokens.push(Token { typ: TokenType::Less, lexeme: "<".to_string() });
            }
            '>' if chars.peek() == Some(&'=') => {
                chars.next();
                tokens.push(Token { typ: TokenType::GreaterEqual, lexeme: ">=".to_string() });
            } else {
                tokens.push(Token { typ: TokenType::Greater, lexeme: ">".to_string() });
            }
            '&' if chars.peek() == Some(&'&') => {
                chars.next();
                tokens.push(Token { typ: TokenType::And, lexeme: "&&".to_string() });
            }
            '|' if chars.peek() == Some(&'|') => {
                chars.next();
                tokens.push(Token { typ: TokenType::Or, lexeme: "||".to_string() });
            }
            '[' => tokens.push(Token { typ: TokenType::LeftBracket, lexeme: "[".to_string() }),
            ']' => tokens.push(Token { typ: TokenType::RightBracket, lexeme: "]".to_string() }),
            '(' => tokens.push(Token { typ: TokenType::LeftParen, lexeme: "(".to_string() }),
            ')' => tokens.push(Token { typ: TokenType::RightParen, lexeme: ")".to_string() }),
            '{' => tokens.push(Token { typ: TokenType::LeftBrace, lexeme: "{".to_string() }),
            '}' => tokens.push(Token { typ: TokenType::RightBrace, lexeme: "}".to_string() }),
            ':' => tokens.push(Token { typ: TokenType::Colon, lexeme: ":".to_string() }),
            '=' => tokens.push(Token { typ: TokenType::Equals, lexeme: "=".to_string() }),
            ',' => tokens.push(Token { typ: TokenType::Comma, lexeme: ",".to_string() }),
            '"' => {
                let mut string = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '"' { break; }
                    string.push(ch);
                    if ch == '\n' { line += 1; }
                }
                tokens.push(Token { typ: TokenType::String, lexeme: string });
            }
            '0'..='9' => {
                let mut num = String::new();
                num.push(c);
                let mut is_float = false;
                while let Some(&next) = chars.peek() {
                    if next.is_digit(10) {
                        num.push(chars.next().unwrap());
                    } else if next == '.' && !is_float {
                        is_float = true;
                        num.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if is_float {
                    tokens.push(Token { typ: TokenType::Float, lexeme: num });
                } else {
                    tokens.push(Token { typ: TokenType::Number, lexeme: num });
                }
            }
            _ if c.is_alphabetic() || c == '_' => {
                let mut id = String::new();
                id.push(c);
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        id.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                let typ = match id.as_str() {
                    "int" => TokenType::IntType,
                    "float" => TokenType::FloatType,
                    "bool" => TokenType::BoolType,
                    "string" => TokenType::StringType,
                    _ => TokenType::Identifier,
                };
                tokens.push(Token { typ, lexeme: id });
            }
            _ => {}, // Ignore or error
        }
    }
    tokens.push(Token { typ: TokenType::Eof, lexeme: "".to_string() });
    tokens
}

fn matches_keyword(chars: &mut std::iter::Peekable<std::str::Chars>, keyword: &str) -> bool {
    for ch in keyword.chars() {
        if chars.peek() == Some(&ch) {
            chars.next();
        } else {
            return false;
        }
    }
    true
}
