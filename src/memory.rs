use std::collections::HashMap;

/// 记忆存储结构，用于保存事实（文本）及其对应的嵌入向量。
///
/// 当前实现为简化版，未来可扩展为基于向量相似度的检索。
#[derive(Debug, Default)]
pub struct MemoryStore {
    /// 存储的事实文本列表。
    facts: Vec<String>,
    /// 文本到嵌入向量的映射。
    embeddings: HashMap<String, Vec<f64>>,
}

impl MemoryStore {
    /// 创建一个空的 `MemoryStore`。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 存储一条事实。
    ///
    /// # 注意
    /// 当前仅将文本追加到列表中，未计算嵌入向量。在实际实现中应同步生成并存储 embedding。
    pub fn store_fact(&mut self, fact: impl Into<String>) {
        self.facts.push(fact.into());
        // TODO: 计算并存储 embedding
    }

    /// 检索与查询相关的事实。
    ///
    /// # 当前实现
    /// 简化版直接返回所有已存储事实的克隆。实际应用应基于向量相似度（如余弦距离）进行语义检索。
    #[must_use]
    pub fn retrieve_facts(&self, _query: &str) -> Vec<String> {
        self.facts.clone()
    }

    /// 存储指定键的嵌入向量。
    pub fn store_embedding(&mut self, key: impl Into<String>, embedding: Vec<f64>) {
        self.embeddings.insert(key.into(), embedding);
    }

    /// 获取指定键的嵌入向量（如果存在）。
    #[must_use]
    pub fn get_embedding(&self, key: &str) -> Option<&Vec<f64>> {
        self.embeddings.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_retrieve_fact() {
        let mut store = MemoryStore::new();
        store.store_fact("Rust 是一门系统编程语言");
        store.store_fact("V 语言编译速度快");

        let facts = store.retrieve_facts("");
        assert_eq!(facts.len(), 2);
        assert!(facts.contains(&"Rust 是一门系统编程语言".to_string()));
    }

    #[test]
    fn store_and_get_embedding() {
        let mut store = MemoryStore::new();
        let embedding = vec![0.1, 0.2, 0.3];
        store.store_embedding("rust", embedding.clone());

        assert_eq!(store.get_embedding("rust"), Some(&embedding));
        assert_eq!(store.get_embedding("nonexistent"), None);
    }
}
