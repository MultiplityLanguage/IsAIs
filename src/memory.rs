use std::collections::HashMap;

/// 记忆存储
pub struct MemoryStore {
    facts: Vec<String>,
    embeddings: HashMap<String, Vec<f64>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            facts: Vec::new(),
            embeddings: HashMap::new(),
        }
    }

    /// 存储事实
    pub fn store_fact(&mut self, fact: &str) {
        self.facts.push(fact.to_string());
        // 在实际实现中，这里会计算 embedding
    }

    /// 检索相关事实
    pub fn retrieve_facts(&self, _query: &str) -> Vec<String> {
        // 简化实现：返回所有事实
        // 实际应该基于向量相似度检索
        self.facts.clone()
    }

    /// 存储嵌入向量
    pub fn store_embedding(&mut self, key: &str, embedding: Vec<f64>) {
        self.embeddings.insert(key.to_string(), embedding);
    }

    /// 获取嵌入向量
    pub fn get_embedding(&self, key: &str) -> Option<&Vec<f64>> {
        self.embeddings.get(key)
    }
}
