use crate::Error;

/// LLM 接口 trait
pub trait LLMInterface {
    /// 查询模型
    fn query(&self, prompt: &str, temperature: f64) -> Result<String, Error>;
    
    /// 获取注意力权重（可选）
    fn get_attention_weights(&self, _tokens: &[String]) -> Result<Vec<f64>, Error> {
        Err(Error::ModelError("Not implemented".to_string()))
    }
}

/// 模拟的 LLM 实现（用于测试）
pub struct MockLLM;

impl LLMInterface for MockLLM {
    fn query(&self, prompt: &str, _temperature: f64) -> Result<String, Error> {
        // 模拟响应
        Ok(format!("Mock response to: {}", prompt))
    }
}

/// GPT-4 模型实现（需要 API key）
pub struct GPT4Model {
    #[allow(dead_code)]
    api_key: String,
}

impl GPT4Model {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

impl LLMInterface for GPT4Model {
    fn query(&self, _prompt: &str, _temperature: f64) -> Result<String, Error> {
        // 实际实现会调用 OpenAI API
        // 这里仅作示例
        Err(Error::ModelError("API call not implemented".to_string()))
    }
}
