pub mod types;
pub mod parser;
pub mod evaluator;
pub mod constraints;
pub mod memory;
pub mod models;

use std::collections::HashMap;

/// IsAIs 解释器主结构
pub struct Interpreter {
    /// 环境变量
    env: HashMap<String, types::Value>,
    /// 记忆存储
    memory: memory::MemoryStore,
    /// 模型接口
    model: Box<dyn models::LLMInterface>,
    /// 生成参数
    temperature: f64,
}

impl Interpreter {
    pub fn new(model: Box<dyn models::LLMInterface>) -> Self {
        Self {
            env: HashMap::new(),
            memory: memory::MemoryStore::new(),
            model,
            temperature: 0.7,
        }
    }

    pub fn evaluate(&mut self, source: &str) -> Result<types::Value, Error> {
        let ast = parser::parse(source)?;
        evaluator::evaluate(&ast, self)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    
    #[error("Model error: {0}")]
    ModelError(String),
}
