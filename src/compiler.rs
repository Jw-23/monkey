use crate::tokenizer::tokenize;
use crate::syntax::parse;
use crate::MonkeyError;
use num_bigint::BigInt;

/// 从源代码字符串直接生成 AST
pub fn compile(source: &str) -> Result<crate::syntax::Program, MonkeyError> {
    // 步骤 1: 词法分析 - 将源代码转换为 Token 流
    let token_iter = tokenize(source);
    
    // 收集tokens
    let tokens: Vec<_> = token_iter
        .collect::<Result<Vec<_>, _>>()?;
    
    if tokens.is_empty() {
        return Err(MonkeyError::ParseError);
    }

    // 步骤 2: 语法分析 - 将 Token 流转换为 AST
    let ast = parse(tokens, source.to_string())?;

    Ok(ast)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{ExpressionKind, StatementKind, InfixOp};

    #[test]
    fn test_compile_simple_expression() {
        let source = "5 + 3 * 2";
        let program = compile(source).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0].kind {
            StatementKind::Expression { expression } => {
                match &expression.kind {
                    ExpressionKind::Infix { left, operator, right: _ } => {
                        assert_eq!(operator, &InfixOp::Plus);
                        match &left.kind {
                            ExpressionKind::IntLiteral(n) => assert_eq!(*n, BigInt::from(5)),
                            _ => panic!("Expected IntLiteral"),
                        }
                    }
                    _ => panic!("Expected infix expression"),
                }
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_compile_let_statement() {
        let source = "let x = 10;";
        let program = compile(source).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0].kind {
            StatementKind::Let { name, value } => {
                assert_eq!(name, "x");
                match &value.kind {
                    ExpressionKind::IntLiteral(n) => assert_eq!(*n, BigInt::from(10)),
                    _ => panic!("Expected IntLiteral"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_compile_function() {
        let source = "fn add(a, b) { return a + b; }";
        let program = compile(source).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0].kind {
            StatementKind::Expression { expression } => {
                match &expression.kind {
                    ExpressionKind::Function { parameters, body } => {
                        assert_eq!(parameters.len(), 2);
                        assert_eq!(parameters[0], "a");
                        assert_eq!(parameters[1], "b");
                        assert_eq!(body.statements.len(), 1);
                    }
                    _ => panic!("Expected function literal"),
                }
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_compile_if_expression() {
        let source = "if (x < 10) { return x; } else { return 10; }";
        let program = compile(source).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0].kind {
            StatementKind::Expression { expression } => {
                match &expression.kind {
                    ExpressionKind::If { condition, consequence, alternative } => {
                        assert!(matches!(condition.kind, ExpressionKind::Infix { .. }));
                        assert_eq!(consequence.statements.len(), 1);
                        assert!(alternative.is_some());
                        assert_eq!(alternative.as_ref().unwrap().statements.len(), 1);
                    }
                    _ => panic!("Expected if expression"),
                }
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_compile_complex_program() {
        let source = r#"
            let x = 5;
            let y = 10;
            let add = fn(a, b) { return a + b; };
            let result = add(x, y);
        "#;
        
        let program = compile(source).unwrap();
        assert_eq!(program.statements.len(), 4);
    }

    #[test]
    fn test_compile_with_string() {
        let source = r#"let message = "Hello, World!";"#;
        let program = compile(source).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0].kind {
            StatementKind::Let { name, value } => {
                assert_eq!(name, "message");
                match &value.kind {
                    ExpressionKind::StringLiteral(s) => assert_eq!(s, "Hello, World!"),
                    _ => panic!("Expected StringLiteral"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }
    #[test]
    fn test_closure(){
        // 正确的闭包泛型标注写法
        let id_str:for<'a> fn(&'a str)->&'a str =  |s: &str| {
            s
        };
        let text="helllo";
        println!("{}",id_str(text));
    }
}
