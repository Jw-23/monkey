use crate::tokenizer::token::{Token, TokenKind};
use crate::MonkeyError;
use num_bigint::BigInt;

// AST 节点定义

#[derive(Debug, PartialEq, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionKind {
    // 字面量
    IntLiteral(BigInt),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Identifier(String),

    // 前缀表达式 (!, -)
    Prefix {
        operator: PrefixOp,
        right: Box<Expression>,
    },

    // 中缀表达式 (+, -, *, /, <, >, ==, !=)
    Infix {
        left: Box<Expression>,
        operator: InfixOp,
        right: Box<Expression>,
    },

    // if 表达式
    If {
        condition: Box<Expression>,
        consequence: BlockStatement,
        alternative: Option<BlockStatement>,
    },

    // 函数字面量
    Function {
        parameters: Vec<String>,
        body: BlockStatement,
    },

    // 函数调用
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrefixOp {
    Not,   // !
    Minus, // -
}

#[derive(Debug, PartialEq, Clone)]
pub enum InfixOp {
    Plus,     // +
    Minus,    // -
    Multiply, // *
    Divide,   // /
    Equal,    // ==
    NotEqual, // !=
    LessThan, // <
    GreaterThan, // >
}

#[derive(Debug, PartialEq, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementKind {
    Let {
        name: String,
        value: Expression,
    },
    Return {
        value: Expression,
    },
    Expression {
        expression: Expression,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

// 运算符优先级
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
enum Precedence {
    Lowest,
    Equals,      // ==, !=
    LessGreater, // <, >
    Sum,         // +, -
    Product,     // *, /
    Prefix,      // -x, !x
    Call,        // function(x)
}

impl Token {
    fn precedence(&self) -> Precedence {
        match &self.kind {
            TokenKind::EQ | TokenKind::NotEQ => Precedence::Equals,
            TokenKind::LT | TokenKind::GT => Precedence::LessGreater,
            TokenKind::Plus | TokenKind::Minus => Precedence::Sum,
            TokenKind::Asterisk | TokenKind::Slash => Precedence::Product,
            TokenKind::LParen => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }
}

// Parser 结构
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    full_text: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, full_text: String) -> Self {
        Parser {
            tokens,
            position: 0,
            full_text,
        }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    #[allow(dead_code)]
    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn current_kind(&self) -> Option<&TokenKind> {
        self.current_token().map(|t| &t.kind)
    }
    #[allow(dead_code)]
    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek_token().map(|t| &t.kind)
    }

    fn create_error(&self, message: &str) -> MonkeyError {
        if let Some(token) = self.current_token() {
            let lines: Vec<&str> = self.full_text.lines().collect();
            let snippet = if token.line > 0 && token.line <= lines.len() {
                let l_idx = token.line - 1;
                let line_text = lines[l_idx];
                let mut s = format!("{:>4} | {}\n     | ", token.line, line_text);
                for _ in 0..token.column.saturating_sub(1) {
                    s.push(' ');
                }
                s.push('^');
                s.push_str(" --- here");
                s
            } else {
                format!("     | (end of file at line {}, col {})", token.line, token.column)
            };

            MonkeyError::ContextualError {
                message: message.to_string(),
                line: token.line,
                column: token.column,
                snippet,
            }
        } else {
            MonkeyError::ParseError
        }
    }

    fn expect_kind(&mut self, expected: TokenKind) -> Result<(), MonkeyError> {
        match self.current_kind() {
            Some(k) if k == &expected => {
                self.advance();
                Ok(())
            }
            _ => Err(self.create_error(&format!("Expected {:?}, but found {:?}", expected, self.current_kind()))),
        }
    }

    // 解析程序
    pub fn parse_program(&mut self) -> Result<Program, MonkeyError> {
        let mut statements = Vec::new();

        while self.current_token().is_some() {
            if let Some(TokenKind::EOF) = self.current_kind() {
                break;
            }
            if let Some(TokenKind::Illegal) = self.current_kind() {
                return Err(self.create_error("Illegal token"));
            }
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }

        Ok(Program { statements })
    }

    // 解析语句
    fn parse_statement(&mut self) -> Result<Statement, MonkeyError> {
        match self.current_kind() {
            Some(TokenKind::Let) => self.parse_let_statement(),
            Some(TokenKind::Return) => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    // 解析 let 语句
    fn parse_let_statement(&mut self) -> Result<Statement, MonkeyError> {
        let (line, column) = self.current_token().map(|t| (t.line, t.column)).unwrap_or((0, 0));
        self.advance(); // 跳过 'let'

        let name = match self.current_kind() {
            Some(TokenKind::Ident(s)) => s.clone(),
            _ => return Err(self.create_error("Expected identifier after let")),
        };
        self.advance();

        self.expect_kind(TokenKind::Assign)?;

        let value = self.parse_expression(Precedence::Lowest)?;

        if let Some(TokenKind::Semicolon) = self.current_kind() {
            self.advance();
        }

        Ok(Statement {
            kind: StatementKind::Let { name, value },
            line,
            column,
        })
    }

    // 解析 return 语句
    fn parse_return_statement(&mut self) -> Result<Statement, MonkeyError> {
        let (line, column) = self.current_token().map(|t| (t.line, t.column)).unwrap_or((0, 0));
        self.advance(); // 跳过 'return'

        let value = self.parse_expression(Precedence::Lowest)?;

        if let Some(TokenKind::Semicolon) = self.current_kind() {
            self.advance();
        }

        Ok(Statement {
            kind: StatementKind::Return { value },
            line,
            column,
        })
    }

    // 解析表达式语句
    fn parse_expression_statement(&mut self) -> Result<Statement, MonkeyError> {
        let (line, column) = self.current_token().map(|t| (t.line, t.column)).unwrap_or((0, 0));
        let expression = self.parse_expression(Precedence::Lowest)?;

        if let Some(TokenKind::Semicolon) = self.current_kind() {
            self.advance();
        }

        Ok(Statement {
            kind: StatementKind::Expression { expression },
            line,
            column,
        })
    }

    // 解析表达式 (使用 Pratt 解析算法)
    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, MonkeyError> {
        let mut left = self.parse_prefix_expression()?;

        while self.current_kind().is_some()
            && self.current_kind() != Some(&TokenKind::Semicolon)
            && precedence < self.current_token().unwrap().precedence()
        {
            left = self.parse_infix_expression(left)?;
        }

        Ok(left)
    }

    // 解析前缀表达式
    fn parse_prefix_expression(&mut self) -> Result<Expression, MonkeyError> {
        let (line, column) = self.current_token().map(|t| (t.line, t.column)).unwrap_or((0, 0));
        match self.current_kind() {
            Some(TokenKind::Int(n)) => {
                let val = n.clone();
                self.advance();
                Ok(Expression {
                    kind: ExpressionKind::IntLiteral(val),
                    line,
                    column,
                })
            }
            Some(TokenKind::Float(f)) => {
                let val = *f;
                self.advance();
                Ok(Expression {
                    kind: ExpressionKind::FloatLiteral(val),
                    line,
                    column,
                })
            }
            Some(TokenKind::String(s)) => {
                let val = s.clone();
                self.advance();
                Ok(Expression {
                    kind: ExpressionKind::StringLiteral(val),
                    line,
                    column,
                })
            }
            Some(TokenKind::True) => {
                self.advance();
                Ok(Expression {
                    kind: ExpressionKind::BoolLiteral(true),
                    line,
                    column,
                })
            }
            Some(TokenKind::False) => {
                self.advance();
                Ok(Expression {
                    kind: ExpressionKind::BoolLiteral(false),
                    line,
                    column,
                })
            }
            Some(TokenKind::Ident(s)) => {
                let name = s.clone();
                self.advance();
                Ok(Expression {
                    kind: ExpressionKind::Identifier(name),
                    line,
                    column,
                })
            }
            Some(TokenKind::Bang) => {
                self.advance();
                let right = self.parse_expression(Precedence::Prefix)?;
                Ok(Expression {
                    kind: ExpressionKind::Prefix {
                        operator: PrefixOp::Not,
                        right: Box::new(right),
                    },
                    line,
                    column,
                })
            }
            Some(TokenKind::Minus) => {
                self.advance();
                let right = self.parse_expression(Precedence::Prefix)?;
                Ok(Expression {
                    kind: ExpressionKind::Prefix {
                        operator: PrefixOp::Minus,
                        right: Box::new(right),
                    },
                    line,
                    column,
                })
            }
            Some(TokenKind::LParen) => {
                self.advance();
                let expr = self.parse_expression(Precedence::Lowest)?;
                self.expect_kind(TokenKind::RParen)?;
                Ok(expr)
            }
            Some(TokenKind::If) => self.parse_if_expression(),
            Some(TokenKind::Function) => self.parse_function_literal(),
            _ => Err(self.create_error("No prefix parse function for token")),
        }
    }

    // 解析中缀表达式
    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression, MonkeyError> {
        let (line, column) = self.current_token().map(|t| (t.line, t.column)).unwrap_or((0, 0));
        match self.current_kind() {
            Some(TokenKind::Plus) => {
                let precedence = self.current_token().unwrap().precedence();
                self.advance();
                let right = self.parse_expression(precedence)?;
                Ok(Expression {
                    kind: ExpressionKind::Infix {
                        left: Box::new(left),
                        operator: InfixOp::Plus,
                        right: Box::new(right),
                    },
                    line,
                    column,
                })
            }
            Some(TokenKind::Minus) => {
                let precedence = self.current_token().unwrap().precedence();
                self.advance();
                let right = self.parse_expression(precedence)?;
                Ok(Expression {
                    kind: ExpressionKind::Infix {
                        left: Box::new(left),
                        operator: InfixOp::Minus,
                        right: Box::new(right),
                    },
                    line,
                    column,
                })
            }
            Some(TokenKind::Asterisk) => {
                let precedence = self.current_token().unwrap().precedence();
                self.advance();
                let right = self.parse_expression(precedence)?;
                Ok(Expression {
                    kind: ExpressionKind::Infix {
                        left: Box::new(left),
                        operator: InfixOp::Multiply,
                        right: Box::new(right),
                    },
                    line,
                    column,
                })
            }
            Some(TokenKind::Slash) => {
                let precedence = self.current_token().unwrap().precedence();
                self.advance();
                let right = self.parse_expression(precedence)?;
                Ok(Expression {
                    kind: ExpressionKind::Infix {
                        left: Box::new(left),
                        operator: InfixOp::Divide,
                        right: Box::new(right),
                    },
                    line,
                    column,
                })
            }
            Some(TokenKind::EQ) => {
                let precedence = self.current_token().unwrap().precedence();
                self.advance();
                let right = self.parse_expression(precedence)?;
                Ok(Expression {
                    kind: ExpressionKind::Infix {
                        left: Box::new(left),
                        operator: InfixOp::Equal,
                        right: Box::new(right),
                    },
                    line,
                    column,
                })
            }
            Some(TokenKind::NotEQ) => {
                let precedence = self.current_token().unwrap().precedence();
                self.advance();
                let right = self.parse_expression(precedence)?;
                Ok(Expression {
                    kind: ExpressionKind::Infix {
                        left: Box::new(left),
                        operator: InfixOp::NotEqual,
                        right: Box::new(right),
                    },
                    line,
                    column,
                })
            }
            Some(TokenKind::LT) => {
                let precedence = self.current_token().unwrap().precedence();
                self.advance();
                let right = self.parse_expression(precedence)?;
                Ok(Expression {
                    kind: ExpressionKind::Infix {
                        left: Box::new(left),
                        operator: InfixOp::LessThan,
                        right: Box::new(right),
                    },
                    line,
                    column,
                })
            }
            Some(TokenKind::GT) => {
                let precedence = self.current_token().unwrap().precedence();
                self.advance();
                let right = self.parse_expression(precedence)?;
                Ok(Expression {
                    kind: ExpressionKind::Infix {
                        left: Box::new(left),
                        operator: InfixOp::GreaterThan,
                        right: Box::new(right),
                    },
                    line,
                    column,
                })
            }
            Some(TokenKind::LParen) => self.parse_call_expression(left),
            _ => Err(self.create_error("No infix parse function for token")),
        }
    }

    // 解析 if 表达式
    fn parse_if_expression(&mut self) -> Result<Expression, MonkeyError> {
        let (line, column) = self.current_token().map(|t| (t.line, t.column)).unwrap_or((0, 0));
        self.advance(); // 跳过 'if'

        self.expect_kind(TokenKind::LParen)?;
        let condition = self.parse_expression(Precedence::Lowest)?;
        self.expect_kind(TokenKind::RParen)?;

        let consequence = self.parse_block_statement()?;

        let alternative = if let Some(TokenKind::Else) = self.current_kind() {
            self.advance();
            // 支持 else if ...
            if let Some(TokenKind::If) = self.current_kind() {
                let (if_line, if_col) = self.current_token().map(|t| (t.line, t.column)).unwrap_or((0, 0));
                let expr = self.parse_if_expression()?;
                Some(BlockStatement {
                    statements: vec![Statement {
                        kind: StatementKind::Expression { expression: expr },
                        line: if_line,
                        column: if_col,
                    }],
                    line: if_line,
                    column: if_col,
                })
            } else {
                Some(self.parse_block_statement()?)
            }
        } else {
            None
        };

        Ok(Expression {
            kind: ExpressionKind::If {
                condition: Box::new(condition),
                consequence,
                alternative,
            },
            line,
            column,
        })
    }

    // 解析函数字面量
    fn parse_function_literal(&mut self) -> Result<Expression, MonkeyError> {
        let (line, column) = self.current_token().map(|t| (t.line, t.column)).unwrap_or((0, 0));
        self.advance(); // 跳过 'fn'

        // 识别可能的函数名并跳过它（为了支持 fn add(a, b) { ... } 这种语法）
        if let Some(TokenKind::Ident(_)) = self.current_kind() {
            self.advance();
        }

        self.expect_kind(TokenKind::LParen)?;
        let parameters = self.parse_function_parameters()?;
        self.expect_kind(TokenKind::RParen)?;

        let body = self.parse_block_statement()?;

        Ok(Expression {
            kind: ExpressionKind::Function { parameters, body },
            line,
            column,
        })
    }

    // 解析函数参数
    fn parse_function_parameters(&mut self) -> Result<Vec<String>, MonkeyError> {
        let mut params = Vec::new();

        if let Some(TokenKind::RParen) = self.current_kind() {
            return Ok(params);
        }

        match self.current_kind() {
            Some(TokenKind::Ident(s)) => {
                params.push(s.clone());
                self.advance();
            }
            _ => return Err(self.create_error("Expected identifier in function parameters")),
        }

        while let Some(TokenKind::Comma) = self.current_kind() {
            self.advance();
            match self.current_kind() {
                Some(TokenKind::Ident(s)) => {
                    params.push(s.clone());
                    self.advance();
                }
                _ => return Err(self.create_error("Expected identifier after comma in parameters")),
            }
        }

        Ok(params)
    }

    // 解析函数调用表达式
    fn parse_call_expression(&mut self, function: Expression) -> Result<Expression, MonkeyError> {
        let line = function.line;
        let column = function.column;
        self.advance(); // 跳过 '('
        let arguments = self.parse_call_arguments()?;
        if self.current_kind() != Some(&TokenKind::RParen) {
            return Err(self.create_error("Expected ')' after call arguments"));
        }
        self.advance(); // 跳过 ')'

        Ok(Expression {
            kind: ExpressionKind::Call {
                function: Box::new(function),
                arguments,
            },
            line,
            column,
        })
    }

    // 解析函数调用参数
    fn parse_call_arguments(&mut self) -> Result<Vec<Expression>, MonkeyError> {
        let mut args = Vec::new();

        if let Some(TokenKind::RParen) = self.current_kind() {
            return Ok(args);
        }

        args.push(self.parse_expression(Precedence::Lowest)?);

        while let Some(TokenKind::Comma) = self.current_kind() {
            self.advance();
            args.push(self.parse_expression(Precedence::Lowest)?);
        }

        Ok(args)
    }

    // 解析代码块
    fn parse_block_statement(&mut self) -> Result<BlockStatement, MonkeyError> {
        let (line, column) = self.current_token().map(|t| (t.line, t.column)).unwrap_or((0, 0));
        self.expect_kind(TokenKind::LBrace)?;

        let mut statements = Vec::new();

        while self.current_kind().is_some()
            && self.current_kind() != Some(&TokenKind::RBrace)
        {
            statements.push(self.parse_statement()?);
        }

        self.expect_kind(TokenKind::RBrace)?;

        Ok(BlockStatement { statements, line, column })
    }
}

// 便捷函数：从 Token 流直接解析为 AST
pub fn parse(tokens: Vec<Token>, full_text: String) -> Result<Program, MonkeyError> {
    let mut parser = Parser::new(tokens, full_text);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_token(kind: TokenKind) -> Token {
        Token { kind, line: 1, column: 1 }
    }

    #[test]
    fn test_parse_let_statement() {
        let tokens = vec![
            test_token(TokenKind::Let),
            test_token(TokenKind::Ident("x".to_string())),
            test_token(TokenKind::Assign),
            test_token(TokenKind::Int(BigInt::from(5))),
            test_token(TokenKind::Semicolon),
        ];

        let program = parse(tokens, String::new()).unwrap();
        assert_eq!(program.statements.len(), 1);

        match &program.statements[0].kind {
            StatementKind::Let { name, value } => {
                assert_eq!(name, "x");
                match &value.kind {
                    ExpressionKind::IntLiteral(n) => assert_eq!(*n, BigInt::from(5)),
                    _ => panic!("Expected IntLiteral"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_parse_return_statement() {
        let tokens = vec![
            test_token(TokenKind::Return),
            test_token(TokenKind::Int(BigInt::from(10))),
            test_token(TokenKind::Semicolon),
        ];

        let program = parse(tokens, String::new()).unwrap();
        assert_eq!(program.statements.len(), 1);

        match &program.statements[0].kind {
            StatementKind::Return { value } => {
                match &value.kind {
                    ExpressionKind::IntLiteral(n) => assert_eq!(*n, BigInt::from(10)),
                    _ => panic!("Expected IntLiteral"),
                }
            }
            _ => panic!("Expected return statement"),
        }
    }

    #[test]
    fn test_parse_infix_expression() {
        let tokens = vec![
            test_token(TokenKind::Int(BigInt::from(5))),
            test_token(TokenKind::Plus),
            test_token(TokenKind::Int(BigInt::from(3))),
            test_token(TokenKind::Asterisk),
            test_token(TokenKind::Int(BigInt::from(2))),
        ];

        let program = parse(tokens, String::new()).unwrap();
        assert_eq!(program.statements.len(), 1);

        match &program.statements[0].kind {
            StatementKind::Expression { expression } => {
                match &expression.kind {
                    ExpressionKind::Infix { left, operator, right } => {
                        assert_eq!(operator, &InfixOp::Plus);
                        match &left.kind {
                            ExpressionKind::IntLiteral(n) => assert_eq!(*n, BigInt::from(5)),
                            _ => panic!("Expected IntLiteral"),
                        }
                        // 右边应该是 3 * 2
                        match &right.kind {
                            ExpressionKind::Infix { left: l2, operator: op2, right: r2 } => {
                                assert_eq!(op2, &InfixOp::Multiply);
                                match &l2.kind {
                                    ExpressionKind::IntLiteral(n) => assert_eq!(*n, BigInt::from(3)),
                                    _ => panic!("Expected IntLiteral"),
                                }
                                match &r2.kind {
                                    ExpressionKind::IntLiteral(n) => assert_eq!(*n, BigInt::from(2)),
                                    _ => panic!("Expected IntLiteral"),
                                }
                            }
                            _ => panic!("Expected infix expression"),
                        }
                    }
                    _ => panic!("Expected infix expression"),
                }
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_function_literal() {
        let tokens = vec![
            test_token(TokenKind::Function),
            test_token(TokenKind::LParen),
            test_token(TokenKind::Ident("a".to_string())),
            test_token(TokenKind::Comma),
            test_token(TokenKind::Ident("b".to_string())),
            test_token(TokenKind::RParen),
            test_token(TokenKind::LBrace),
            test_token(TokenKind::Return),
            test_token(TokenKind::Ident("a".to_string())),
            test_token(TokenKind::Plus),
            test_token(TokenKind::Ident("b".to_string())),
            test_token(TokenKind::Semicolon),
            test_token(TokenKind::RBrace),
        ];

        let mut parser = Parser::new(tokens, String::new());
        let program = parser.parse_program();
        if let Err(e) = &program {
            panic!("Parse failed: {:?}", e);
        }
        let program = program.unwrap();
        assert_eq!(program.statements.len(), 1);

        match &program.statements[0].kind {
            StatementKind::Expression { expression } => {
                match &expression.kind {
                    ExpressionKind::Function { parameters, body } => {
                        assert_eq!(parameters, &vec!["a".to_string(), "b".to_string()]);
                        assert_eq!(body.statements.len(), 1);
                    }
                    _ => panic!("Expected function literal, got {:?}", expression),
                }
            }
            _ => panic!("Expected expression statement"),
        }
    }
}