use crate::MonkeyError;
use num_bigint::BigInt;
use num_traits::Num;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Illegal,
    EOF,

    // 标识符与字面量
    Ident(String),
    Int(BigInt),
    Float(f64),
    String(String),

    // 操作符
    Assign,   // =
    Plus,     // +
    Minus,    // -
    Bang,     // !
    Asterisk, // *
    Slash,    // /
    LT,       // <
    GT,       // >
    EQ,       // ==
    NotEQ,    // !=

    // 分隔符
    Comma,
    Semicolon,
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }

    // 关键字
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}

fn skip_whitespace(input: &str) -> &str {
    input.trim_start()
}

macro_rules! parse_symbol {
    ($name:ident, $sym:expr, $token:expr) => {
        pub fn $name(input: &str) -> Result<(&str, TokenKind), MonkeyError> {
            let input = skip_whitespace(input);
            if input.starts_with($sym) {
                Ok((&input[$sym.len()..], $token))
            } else {
                Err(MonkeyError::ParseError)
            }
        }
    };
}

parse_symbol!(parse_eq, "==", TokenKind::EQ);
parse_symbol!(parse_not_eq, "!=", TokenKind::NotEQ);
parse_symbol!(parse_assign, "=", TokenKind::Assign);
parse_symbol!(parse_plus, "+", TokenKind::Plus);
parse_symbol!(parse_minus, "-", TokenKind::Minus);
parse_symbol!(parse_bang, "!", TokenKind::Bang);
parse_symbol!(parse_asterisk, "*", TokenKind::Asterisk);
parse_symbol!(parse_slash, "/", TokenKind::Slash);
parse_symbol!(parse_lt, "<", TokenKind::LT);
parse_symbol!(parse_gt, ">", TokenKind::GT);
parse_symbol!(parse_comma, ",", TokenKind::Comma);
parse_symbol!(parse_semicolon, ";", TokenKind::Semicolon);
parse_symbol!(parse_lparen, "(", TokenKind::LParen);
parse_symbol!(parse_rparen, ")", TokenKind::RParen);
parse_symbol!(parse_lbrace, "{", TokenKind::LBrace);
parse_symbol!(parse_rbrace, "}", TokenKind::RBrace);

macro_rules! parse_keyword {
    ($name:ident, $word:expr, $token:expr) => {
        pub fn $name(input: &str) -> Result<(&str, TokenKind), MonkeyError> {
            let input = skip_whitespace(input);
            if input.starts_with($word) {
                let rest = &input[$word.len()..];
                if rest.chars().next().map_or(true, |c| !c.is_alphanumeric() && c != '_') {
                    return Ok((rest, $token));
                }
            }
            Err(MonkeyError::ParseError)
        }
    };
}

parse_keyword!(parse_function, "fn", TokenKind::Function);
parse_keyword!(parse_let, "let", TokenKind::Let);
parse_keyword!(parse_true, "true", TokenKind::True);
parse_keyword!(parse_false, "false", TokenKind::False);
parse_keyword!(parse_if, "if", TokenKind::If);
parse_keyword!(parse_else, "else", TokenKind::Else);
parse_keyword!(parse_return, "return", TokenKind::Return);

pub fn parse_int(input: &str) -> Result<(&str, TokenKind), MonkeyError> {
    let input = skip_whitespace(input);
    let mut num_str = String::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num_str.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if num_str.is_empty() {
        return Err(MonkeyError::ParseError);
    }

    let val = BigInt::from_str_radix(&num_str, 10).map_err(|_| MonkeyError::ParseError)?;
    Ok((&input[num_str.len()..], TokenKind::Int(val)))
}

pub fn parse_float(input: &str) -> Result<(&str, TokenKind), MonkeyError> {
    let input = skip_whitespace(input);
    let mut num_str = String::new();
    let mut chars = input.chars().peekable();
    let mut has_dot = false;

    // 解析整数部分
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num_str.push(c);
            chars.next();
        } else if c == '.' && !has_dot {
            // 检查小数点后是否有数字
            let mut temp_chars = chars.clone();
            temp_chars.next(); // 跳过小数点
            if let Some(&next_c) = temp_chars.peek() {
                if next_c.is_ascii_digit() {
                    has_dot = true;
                    num_str.push(c);
                    chars.next();
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if num_str.is_empty() || !has_dot {
        return Err(MonkeyError::ParseError);
    }

    let val = num_str.parse::<f64>().map_err(|_| MonkeyError::ParseError)?;
    Ok((&input[num_str.len()..], TokenKind::Float(val)))
}

pub fn parse_string(input: &str) -> Result<(&str, TokenKind), MonkeyError> {
    let input = skip_whitespace(input);
    if !input.starts_with('"') {
        return Err(MonkeyError::ParseError);
    }

    let mut content = String::new();
    let mut chars = input.chars().skip(1); // skip opening quote

    while let Some(c) = chars.next() {
        if c == '"' {
            let consumed = content.len() + 2;
            return Ok((&input[consumed..], TokenKind::String(content)));
        }
        content.push(c);
    }

    Err(MonkeyError::ParseError)
}

pub fn parse_ident(input: &str) -> Result<(&str, TokenKind), MonkeyError> {
    let input = skip_whitespace(input);
    let mut ident = String::new();
    let mut chars = input.chars().peekable();

    if let Some(&c) = chars.peek() {
        if c.is_alphabetic() || c == '_' {
            ident.push(c);
            chars.next();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    ident.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            return Ok((&input[ident.len()..], TokenKind::Ident(ident)));
        }
    }

    Err(MonkeyError::ParseError)
}