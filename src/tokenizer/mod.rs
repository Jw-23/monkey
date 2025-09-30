use crate::MonkeyError;
use crate::logic::sequence;
use num_bigint::BigInt;
pub mod token;
pub use token::*;

#[derive(Clone, Copy)]
struct Input<'a> {
    text: &'a str,
    line: usize,
    column: usize,
}

impl<'a> Input<'a> {
    fn skip_whitespace(self) -> Self {
        let text = self.text;
        let mut line = self.line;
        let mut column = self.column;
        
        let trimmed = text.trim_start();
        let skipped = &text[..text.len() - trimmed.len()];
        
        for c in skipped.chars() {
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        
        Input { text: trimmed, line, column }
    }
}





/// 闭包包装器标本
/// 使用 for<'a> (HRTB) 来显式标注输入与输出的生命周期绑定
fn wrap_parser_template<F>(inner: F) -> impl for<'a> Fn(Input<'a>) -> Result<(Input<'a>, Token), MonkeyError> + Clone + Copy
where
    F: for<'b> Fn(&'b str) -> Result<(&'b str, TokenKind), MonkeyError> + Clone + Copy,
{
    move |input: Input| {
        let input_after_ws = input.skip_whitespace();
        
        // 调用传入的解析闭包，它返回的 &str 生命周期必须与输入的 text 一致
        let (next_text, kind) = inner(input_after_ws.text)?;
        
        let consumed = &input_after_ws.text[..input_after_ws.text.len() - next_text.len()];
        let mut next_line = input_after_ws.line;
        let mut next_column = input_after_ws.column;
        
        for c in consumed.chars() {
            if c == '\n' {
                next_line += 1;
                next_column = 1;
            } else {
                next_column += 1;
            }
        }
        
        Ok((
            Input { text: next_text, line: next_line, column: next_column },
            Token { kind, line: input_after_ws.line, column: input_after_ws.column }
        ))
    }
}

// 定义一个宏来生成具名包装函数，避免闭包带来的生命周期推导问题
/* macro_rules! define_token_parser {
    ($name:ident, $inner:ident) => {
        fn $name(input: Input) -> Result<(Input, Token), MonkeyError> {
            let input_after_ws = input.skip_whitespace();
            let (next_text, kind) = $inner(input_after_ws.text)?;
            
            let consumed = &input_after_ws.text[..input_after_ws.text.len() - next_text.len()];
            let mut next_line = input_after_ws.line;
            let mut next_column = input_after_ws.column;
            
            for c in consumed.chars() {
                if c == '\n' {
                    next_line += 1;
                    next_column = 1;
                } else {
                    next_column += 1;
                }
            }
            
            Ok((
                Input { text: next_text, line: next_line, column: next_column },
                Token { kind, line: input_after_ws.line, column: input_after_ws.column }
            ))
        }
    };
} */
/* 使用过程宏
define_token_parser!(token_eq, parse_eq);
define_token_parser!(token_not_eq, parse_not_eq);
define_token_parser!(token_assign, parse_assign);
define_token_parser!(token_plus, parse_plus);
define_token_parser!(token_minus, parse_minus);
define_token_parser!(token_bang, parse_bang);
define_token_parser!(token_asterisk, parse_asterisk);
define_token_parser!(token_slash, parse_slash);
define_token_parser!(token_lt, parse_lt);
define_token_parser!(token_gt, parse_gt);
define_token_parser!(token_comma, parse_comma);
define_token_parser!(token_semicolon, parse_semicolon);
define_token_parser!(token_lparen, parse_lparen);
define_token_parser!(token_rparen, parse_rparen);
define_token_parser!(token_lbrace, parse_lbrace);
define_token_parser!(token_rbrace, parse_rbrace);
define_token_parser!(token_function, parse_function);
define_token_parser!(token_let, parse_let);
define_token_parser!(token_true, parse_true);
define_token_parser!(token_false, parse_false);
define_token_parser!(token_if, parse_if);
define_token_parser!(token_else, parse_else);
define_token_parser!(token_return, parse_return);
define_token_parser!(token_float, parse_float);
define_token_parser!(token_int, parse_int);
define_token_parser!(token_string, parse_string);
define_token_parser!(token_ident, parse_ident); */

fn token_illegal(input: Input) -> Result<(Input, Token), MonkeyError> {
    let input_after_ws = input.skip_whitespace();
    let mut chars = input_after_ws.text.chars();
    if let Some(c) = chars.next() {
        let next_text = chars.as_str();
        let mut next_line = input_after_ws.line;
        let mut next_column = input_after_ws.column;
        
        if c == '\n' {
            next_line += 1;
            next_column = 1;
        } else {
            next_column += 1;
        }
        
        Ok((
            Input { text: next_text, line: next_line, column: next_column },
            Token { kind: TokenKind::Illegal, line: input_after_ws.line, column: input_after_ws.column }
        ))
    } else {
        Err(MonkeyError::ParseError)
    }
}

pub fn tokenize(input: &str) -> Box<dyn Iterator<Item = Result<Token, MonkeyError>> + '_> {
    let input_state = Input { text: input, line: 1, column: 1 };

    
    let p = sequence((
        wrap_parser_template(parse_eq), wrap_parser_template(parse_not_eq), wrap_parser_template(parse_assign),
        wrap_parser_template(parse_plus), wrap_parser_template(parse_minus), wrap_parser_template(parse_bang),
        wrap_parser_template(parse_asterisk), wrap_parser_template(parse_slash), wrap_parser_template(parse_lt),
        wrap_parser_template(parse_gt), wrap_parser_template(parse_comma), wrap_parser_template(parse_semicolon),
        wrap_parser_template(parse_lparen), wrap_parser_template(parse_rparen), wrap_parser_template(parse_lbrace),
        wrap_parser_template(parse_rbrace), wrap_parser_template(parse_function), wrap_parser_template(parse_let),
        wrap_parser_template(parse_true), wrap_parser_template(parse_false), wrap_parser_template(parse_if),
        wrap_parser_template(parse_else), wrap_parser_template(parse_return), wrap_parser_template(parse_float),
        wrap_parser_template(parse_int), wrap_parser_template(parse_string), wrap_parser_template(parse_ident),
        token_illegal
    ));
   

    /* let p = sequence((
        token_eq, token_not_eq, token_assign, token_plus, token_minus,
        token_bang, token_asterisk, token_slash, token_lt, token_gt,
        token_comma, token_semicolon, token_lparen, token_rparen,
        token_lbrace, token_rbrace, token_function, token_let,
        token_true, token_false, token_if, token_else, token_return,
        token_float, token_int, token_string, token_ident,
        token_illegal
    )); */

    // 使用 take_while 过滤掉最后因为无法匹配（到达末尾）而产生的 EOFParserSequence 错误
    Box::new(p(input_state).take_while(|res| {
        !matches!(res, Err(MonkeyError::EOFParserSequence))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = "let x = 5; fn add(a, b) { return a + b; }";
        let mut tokens = tokenize(input);

        assert_eq!(tokens.next().unwrap().unwrap().kind, TokenKind::Let);
        assert_eq!(tokens.next().unwrap().unwrap().kind, TokenKind::Ident("x".to_string()));
        assert_eq!(tokens.next().unwrap().unwrap().kind, TokenKind::Assign);
        assert_eq!(tokens.next().unwrap().unwrap().kind, TokenKind::Int(BigInt::from(5)));
        assert_eq!(tokens.next().unwrap().unwrap().kind, TokenKind::Semicolon);
    }
}