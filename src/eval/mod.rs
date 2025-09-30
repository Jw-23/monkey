use crate::syntax::*;
use crate::MonkeyError;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Integer(BigInt),
    Float(f64),
    Boolean(bool),
    String(String),
    Function {
        parameters: Vec<String>,
        body: BlockStatement,
        env: Rc<RefCell<Environment>>,
    },
    ReturnValue(Box<Object>),
    Null,
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::Float(fl) => write!(f, "{}", fl),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::String(s) => write!(f, "{}", s),
            Object::Function { parameters, .. } => {
                write!(f, "fn({}) {{ ... }}", parameters.join(", "))
            }
            Object::ReturnValue(obj) => write!(f, "{}", obj),
            Object::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    store: HashMap<String, Object>,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_with_outer(outer: Rc<RefCell<Environment>>) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some(obj) => Some(obj.clone()),
            None => match &self.outer {
                Some(outer) => outer.borrow().get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, val: Object) {
        self.store.insert(name, val);
    }
}

pub struct Evaluator {
    full_text: String,
}

impl Evaluator {
    pub fn new(full_text: String) -> Self {
        Evaluator { full_text }
    }

    fn create_error(&self, message: &str, line: usize, column: usize) -> MonkeyError {
        let lines: Vec<&str> = self.full_text.lines().collect();
        let snippet = if line > 0 && line <= lines.len() {
            let l_idx = line - 1;
            let line_text = lines[l_idx];
            let mut s = format!("{:>4} | {}\n     | ", line, line_text);
            for _ in 0..column.saturating_sub(1) {
                s.push(' ');
            }
            s.push('^');
            s.push_str(" --- here");
            s
        } else {
            format!("     | (line {}, col {})", line, column)
        };

        MonkeyError::ContextualError {
            message: message.to_string(),
            line,
            column,
            snippet,
        }
    }

    pub fn eval_program(&self, program: &Program, env: Rc<RefCell<Environment>>) -> Result<Object, MonkeyError> {
        let mut result = Object::Null;

        for statement in &program.statements {
            result = self.eval_statement(statement, Rc::clone(&env))?;

            if let Object::ReturnValue(val) = result {
                return Ok(*val);
            }
        }

        Ok(result)
    }

    fn eval_statement(&self, stmt: &Statement, env: Rc<RefCell<Environment>>) -> Result<Object, MonkeyError> {
        match &stmt.kind {
            StatementKind::Expression { expression } => self.eval_expression(expression, env),
            StatementKind::Let { name, value } => {
                let val = self.eval_expression(value, Rc::clone(&env))?;
                env.borrow_mut().set(name.clone(), val);
                Ok(Object::Null)
            }
            StatementKind::Return { value } => {
                let val = self.eval_expression(value, env)?;
                Ok(Object::ReturnValue(Box::new(val)))
            }
        }
    }

    fn eval_expression(&self, expr: &Expression, env: Rc<RefCell<Environment>>) -> Result<Object, MonkeyError> {
        match &expr.kind {
            ExpressionKind::IntLiteral(i) => Ok(Object::Integer(i.clone())),
            ExpressionKind::FloatLiteral(f) => Ok(Object::Float(*f)),
            ExpressionKind::BoolLiteral(b) => Ok(Object::Boolean(*b)),
            ExpressionKind::StringLiteral(s) => Ok(Object::String(s.clone())),
            ExpressionKind::Identifier(name) => {
                env.borrow().get(name).ok_or_else(|| {
                    self.create_error(&format!("identifier not found: {}", name), expr.line, expr.column)
                })
            }
            ExpressionKind::Prefix { operator, right } => {
                let right_val = self.eval_expression(right, env)?;
                self.eval_prefix_expression(operator, right_val, expr.line, expr.column)
            }
            ExpressionKind::Infix { left, operator, right } => {
                let left_val = self.eval_expression(left, Rc::clone(&env))?;
                let right_val = self.eval_expression(right, env)?;
                self.eval_infix_expression(operator, left_val, right_val, expr.line, expr.column)
            }
            ExpressionKind::If { condition, consequence, alternative } => {
                let cond_val = self.eval_expression(condition, Rc::clone(&env))?;
                if self.is_truthy(&cond_val) {
                    self.eval_block_statement(consequence, env)
                } else if let Some(alt) = alternative {
                    self.eval_block_statement(alt, env)
                } else {
                    Ok(Object::Null)
                }
            }
            ExpressionKind::Function { parameters, body } => {
                Ok(Object::Function {
                    parameters: parameters.clone(),
                    body: body.clone(),
                    env: Rc::clone(&env),
                })
            }
            ExpressionKind::Call { function, arguments } => {
                let func_obj = self.eval_expression(function, Rc::clone(&env))?;
                let args = arguments.iter()
                    .map(|arg| self.eval_expression(arg, Rc::clone(&env)))
                    .collect::<Result<Vec<Object>, MonkeyError>>()?;
                
                self.apply_function(func_obj, args, expr.line, expr.column)
            }
        }
    }

    fn eval_prefix_expression(&self, op: &PrefixOp, right: Object, line: usize, col: usize) -> Result<Object, MonkeyError> {
        match op {
            PrefixOp::Not => match right {
                Object::Boolean(b) => Ok(Object::Boolean(!b)),
                Object::Null => Ok(Object::Boolean(true)),
                _ => Ok(Object::Boolean(false)),
            },
            PrefixOp::Minus => match right {
                Object::Integer(i) => Ok(Object::Integer(-i)),
                Object::Float(f) => Ok(Object::Float(-f)),
                _ => Err(self.create_error(&format!("unknown operator: -{}", right), line, col)),
            },
        }
    }

    fn eval_infix_expression(&self, op: &InfixOp, left: Object, right: Object, line: usize, col: usize) -> Result<Object, MonkeyError> {
        match (&left, &right) {
            (Object::Integer(l), Object::Integer(r)) => {
                match op {
                    InfixOp::Plus => Ok(Object::Integer(l + r)),
                    InfixOp::Minus => Ok(Object::Integer(l - r)),
                    InfixOp::Multiply => Ok(Object::Integer(l * r)),
                    InfixOp::Divide => Ok(Object::Integer(l / r)),
                    InfixOp::Equal => Ok(Object::Boolean(l == r)),
                    InfixOp::NotEqual => Ok(Object::Boolean(l != r)),
                    InfixOp::LessThan => Ok(Object::Boolean(l < r)),
                    InfixOp::GreaterThan => Ok(Object::Boolean(l > r)),
                }
            }
            (Object::Float(l), Object::Float(r)) => {
                match op {
                    InfixOp::Plus => Ok(Object::Float(l + r)),
                    InfixOp::Minus => Ok(Object::Float(l - r)),
                    InfixOp::Multiply => Ok(Object::Float(l * r)),
                    InfixOp::Divide => Ok(Object::Float(l / r)),
                    InfixOp::Equal => Ok(Object::Boolean((l - r).abs() < f64::EPSILON)),
                    InfixOp::NotEqual => Ok(Object::Boolean((l - r).abs() >= f64::EPSILON)),
                    InfixOp::LessThan => Ok(Object::Boolean(l < r)),
                    InfixOp::GreaterThan => Ok(Object::Boolean(l > r)),
                }
            }
            (Object::Integer(l), Object::Float(r)) => {
                let l_float = l.to_f64().unwrap_or(0.0);
                match op {
                    InfixOp::Plus => Ok(Object::Float(l_float + r)),
                    InfixOp::Minus => Ok(Object::Float(l_float - r)),
                    InfixOp::Multiply => Ok(Object::Float(l_float * r)),
                    InfixOp::Divide => Ok(Object::Float(l_float / r)),
                    InfixOp::Equal => Ok(Object::Boolean((l_float - r).abs() < f64::EPSILON)),
                    InfixOp::NotEqual => Ok(Object::Boolean((l_float - r).abs() >= f64::EPSILON)),
                    InfixOp::LessThan => Ok(Object::Boolean(l_float < *r)),
                    InfixOp::GreaterThan => Ok(Object::Boolean(l_float > *r)),
                }
            }
            (Object::Float(l), Object::Integer(r)) => {
                let r_float = r.to_f64().unwrap_or(0.0);
                match op {
                    InfixOp::Plus => Ok(Object::Float(l + r_float)),
                    InfixOp::Minus => Ok(Object::Float(l - r_float)),
                    InfixOp::Multiply => Ok(Object::Float(l * r_float)),
                    InfixOp::Divide => Ok(Object::Float(l / r_float)),
                    InfixOp::Equal => Ok(Object::Boolean((l - r_float).abs() < f64::EPSILON)),
                    InfixOp::NotEqual => Ok(Object::Boolean((l - r_float).abs() >= f64::EPSILON)),
                    InfixOp::LessThan => Ok(Object::Boolean(*l < r_float)),
                    InfixOp::GreaterThan => Ok(Object::Boolean(*l > r_float)),
                }
            }
            (Object::Boolean(l), Object::Boolean(r)) => {
                match op {
                    InfixOp::Equal => Ok(Object::Boolean(l == r)),
                    InfixOp::NotEqual => Ok(Object::Boolean(l != r)),
                    _ => Err(self.create_error(&format!("unknown operator: {:?} for Boolean", op), line, col)),
                }
            }
            (Object::String(l), Object::String(r)) => {
                match op {
                    InfixOp::Plus => Ok(Object::String(format!("{}{}", l, r))),
                    InfixOp::Equal => Ok(Object::Boolean(l == r)),
                    InfixOp::NotEqual => Ok(Object::Boolean(l != r)),
                    _ => Err(self.create_error(&format!("unknown operator: {:?} for String", op), line, col)),
                }
            }
            (l, r) => Err(self.create_error(&format!("type mismatch: {} {:?} {}", l, op, r), line, col)),
        }
    }

    fn eval_block_statement(&self, block: &BlockStatement, env: Rc<RefCell<Environment>>) -> Result<Object, MonkeyError> {
        let mut result = Object::Null;
        for stmt in &block.statements {
            result = self.eval_statement(stmt, Rc::clone(&env))?;
            if let Object::ReturnValue(_) = result {
                return Ok(result);
            }
        }
        Ok(result)
    }

    fn is_truthy(&self, obj: &Object) -> bool {
        match obj {
            Object::Boolean(b) => *b,
            Object::Null => false,
            _ => true,
        }
    }

    fn apply_function(&self, func: Object, args: Vec<Object>, line: usize, col: usize) -> Result<Object, MonkeyError> {
        if let Object::Function { parameters, body, env } = func {
            if parameters.len() != args.len() {
                return Err(self.create_error(&format!("wrong number of arguments: want={}, got={}", parameters.len(), args.len()), line, col));
            }

            let extended_env = Rc::new(RefCell::new(Environment::new_with_outer(Rc::clone(&env))));
            for (param, arg) in parameters.into_iter().zip(args) {
                extended_env.borrow_mut().set(param, arg);
            }

            let result = self.eval_block_statement(&body, extended_env)?;
            if let Object::ReturnValue(val) = result {
                Ok(*val)
            } else {
                Ok(result)
            }
        } else {
            Err(self.create_error(&format!("not a function: {}", func), line, col))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile;

    fn test_eval(input: &str) -> Object {
        let program = compile(input).expect("compile failed");
        let env = Rc::new(RefCell::new(Environment::new()));
        let evaluator = Evaluator::new(input.to_string());
        evaluator.eval_program(&program, env).expect("eval failed")
    }

    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![
            ("5", Object::Integer(BigInt::from(5))),
            ("10", Object::Integer(BigInt::from(10))),
            ("-5", Object::Integer(BigInt::from(-5))),
            ("-10", Object::Integer(BigInt::from(-10))),
            ("5 + 5 + 5 + 5 - 10", Object::Integer(BigInt::from(10))),
            ("2 * 2 * 2 * 2 * 2", Object::Integer(BigInt::from(32))),
            ("-50 + 100 + -50", Object::Integer(BigInt::from(0))),
            ("5 * 2 + 10", Object::Integer(BigInt::from(20))),
            ("5 + 2 * 10", Object::Integer(BigInt::from(25))),
            ("20 + 2 * -10", Object::Integer(BigInt::from(0))),
            ("50 / 2 * 2 + 10", Object::Integer(BigInt::from(60))),
            ("2 * (5 + 10)", Object::Integer(BigInt::from(30))),
            ("3 * 3 * 3 + 10", Object::Integer(BigInt::from(37))),
            ("3 * (3 * 3) + 10", Object::Integer(BigInt::from(37))),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Object::Integer(BigInt::from(50))),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = vec![
            ("true", Object::Boolean(true)),
            ("false", Object::Boolean(false)),
            ("1 < 2", Object::Boolean(true)),
            ("1 > 2", Object::Boolean(false)),
            ("1 < 1", Object::Boolean(false)),
            ("1 > 1", Object::Boolean(false)),
            ("1 == 1", Object::Boolean(true)),
            ("1 != 1", Object::Boolean(false)),
            ("1 == 2", Object::Boolean(false)),
            ("1 != 2", Object::Boolean(true)),
            ("true == true", Object::Boolean(true)),
            ("false == false", Object::Boolean(true)),
            ("true == false", Object::Boolean(false)),
            ("true != false", Object::Boolean(true)),
            ("false != true", Object::Boolean(true)),
            ("(1 < 2) == true", Object::Boolean(true)),
            ("(1 < 2) == false", Object::Boolean(false)),
            ("(1 > 2) == true", Object::Boolean(false)),
            ("(1 > 2) == false", Object::Boolean(true)),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected);
        }
    }

    #[test]
    fn test_bang_operator() {
        let tests = vec![
            ("!true", Object::Boolean(false)),
            ("!false", Object::Boolean(true)),
            ("!5", Object::Boolean(false)),
            ("!!true", Object::Boolean(true)),
            ("!!false", Object::Boolean(false)),
            ("!!5", Object::Boolean(true)),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected);
        }
    }

    #[test]
    fn test_if_else_expressions() {
        let tests = vec![
            ("if (true) { 10 }", Object::Integer(BigInt::from(10))),
            ("if (false) { 10 }", Object::Null),
            ("if (1) { 10 }", Object::Integer(BigInt::from(10))),
            ("if (1 < 2) { 10 }", Object::Integer(BigInt::from(10))),
            ("if (1 > 2) { 10 }", Object::Null),
            ("if (1 > 2) { 10 } else { 20 }", Object::Integer(BigInt::from(20))),
            ("if (1 < 2) { 10 } else { 20 }", Object::Integer(BigInt::from(10))),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected);
        }
    }

    #[test]
    fn test_return_statements() {
        let tests = vec![
            ("return 10;", Object::Integer(BigInt::from(10))),
            ("return 10; 9;", Object::Integer(BigInt::from(10))),
            ("return 2 * 5; 9;", Object::Integer(BigInt::from(10))),
            ("9; return 2 * 5; 9;", Object::Integer(BigInt::from(10))),
            (
                r#"
                if (10 > 1) {
                    if (10 > 1) {
                        return 10;
                    }
                    return 1;
                }
                "#,
                Object::Integer(BigInt::from(10))),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected);
        }
    }

    #[test]
    fn test_let_statements() {
        let tests = vec![
            ("let a = 5; a;", Object::Integer(BigInt::from(5))),
            ("let a = 5 * 5; a;", Object::Integer(BigInt::from(25))),
            ("let a = 5; let b = a; b;", Object::Integer(BigInt::from(5))),
            ("let a = 5; let b = a; let c = a + b + 5; c;", Object::Integer(BigInt::from(15))),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected);
        }
    }

    #[test]
    fn test_function_application() {
        let tests = vec![
            ("let identity = fn(x) { x; }; identity(5);", Object::Integer(BigInt::from(5))),
            ("let identity = fn(x) { return x; }; identity(5);", Object::Integer(BigInt::from(5))),
            ("let double = fn(x) { x * 2; }; double(5);", Object::Integer(BigInt::from(10))),
            ("let add = fn(x, y) { x + y; }; add(5, 5);", Object::Integer(BigInt::from(10))),
            ("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));", Object::Integer(BigInt::from(20))),
            ("fn(x) { x; }(5)", Object::Integer(BigInt::from(5))),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected);
        }
    }

    #[test]
    fn test_closures() {
        let input = r#"
            let new_adder = fn(x) {
                fn(y) { x + y };
            };
            let add_two = new_adder(2);
            add_two(2);
        "#;
        assert_eq!(test_eval(input), Object::Integer(BigInt::from(4)));
    }

    #[test]
    fn test_string_concatenation() {
        let input = r#""Hello" + " " + "World!""#;
        assert_eq!(test_eval(input), Object::String("Hello World!".to_string()));
    }

    #[test]
    fn test_error_handling() {
        let tests = vec![
            (
                "5 + true;",
                "type mismatch: 5 Plus true",
            ),
            (
                "5 + true; 5;",
                "type mismatch: 5 Plus true",
            ),
            (
                "-true",
                "unknown operator: -true",
            ),
            (
                "true + false;",
                "unknown operator: Plus for Boolean",
            ),
            (
                "5; true + false; 5",
                "unknown operator: Plus for Boolean",
            ),
            (
                "if (10 > 1) { true + false; }",
                "unknown operator: Plus for Boolean",
            ),
            (
                r#"
                if (10 > 1) {
                    if (10 > 1) {
                        return true + false;
                    }
                    return 1;
                }
                "#,
                "unknown operator: Plus for Boolean",
            ),
            (
                "foobar",
                "identifier not found: foobar",
            ),
            (
                r#""Hello" - "World""#,
                "unknown operator: Minus for String",
            ),
        ];

        for (input, expected_message) in tests {
            let program = compile(input).expect("compile failed");
            let env = Rc::new(RefCell::new(Environment::new()));
            let evaluator = Evaluator::new(input.to_string());
            let result = evaluator.eval_program(&program, env);

            match result {
                Err(MonkeyError::ContextualError { message, .. }) => {
                    assert_eq!(message, expected_message);
                }
                _ => panic!("Expected ContextualError with message '{}', got {:?}", expected_message, result),
            }
        }
    }
}