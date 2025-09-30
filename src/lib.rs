mod tokenizer;
mod logic;
mod syntax;
mod compiler;
// 解释器模块
pub mod eval;
pub use tokenizer::tokenize;
pub use syntax::{parse, Expression, Statement, Program};
pub use compiler::compile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorContext {
    pub line: usize,
    pub column: usize,
    pub snippet: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum MonkeyError {
    #[error("Parse error at line {line}, col {column}: {message}\n\n{snippet}\n")]
    ContextualError {
        message: String,
        line: usize,
        column: usize,
        snippet: String,
    },
    #[error("failed to translate string to tokens")]
    ParseError,
    #[error("no parser maches input")]
    EOFParserSequence
}
