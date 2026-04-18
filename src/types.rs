use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// IsAIs 中的值类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    TokenSeq(Vec<String>),
    Vector(Vec<f64>),
    Dist(Distribution),
    Model(ModelRef),
    Prompt(Prompt),
    Constraint(Constraint),
    Maybe, // 表示 0.5 的真值
}

/// 概率分布
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Distribution {
    pub outcomes: Vec<(f64, Value)>, // (probability, value)
}

impl Distribution {
    pub fn new(outcomes: Vec<(f64, Value)>) -> Self {
        Self { outcomes }
    }

    /// 获取最可能的值
    pub fn most_likely(&self) -> Option<&Value> {
        self.outcomes
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, v)| v)
    }

    /// 采样一个值
    pub fn sample(&self) -> &Value {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let r: f64 = rng.r#gen::<f64>();
        let mut cumulative = 0.0;
        
        for (prob, value) in &self.outcomes {
            cumulative += prob;
            if r <= cumulative {
                return value;
            }
        }
        
        &self.outcomes.last().unwrap().1
    }

    /// 获取某个值的概率
    pub fn probability_of(&self, target: &Value) -> f64 {
        self.outcomes
            .iter()
            .find_map(|(prob, val)| {
                if values_equal(val, target) {
                    Some(*prob)
                } else {
                    None
                }
            })
            .unwrap_or(0.0)
    }
}

/// 模型引用
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelRef {
    pub name: String,
    pub config: HashMap<String, Value>,
}

/// Prompt 模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub template: String,
    pub holes: Vec<String>, // {{hole}} 名称
    pub bindings: HashMap<String, Value>,
}

impl Prompt {
    pub fn fill(&self) -> String {
        let mut result = self.template.clone();
        for hole in &self.holes {
            if let Some(value) = self.bindings.get(hole) {
                let replacement = format!("{}", value);
                result = result.replace(&format!("{{{{{}}}}}", hole), &replacement);
            }
        }
        result
    }

    pub fn partial_apply(&self, key: &str, value: Value) -> Prompt {
        let mut new_bindings = self.bindings.clone();
        new_bindings.insert(key.to_string(), value);
        Prompt {
            template: self.template.clone(),
            holes: self.holes.clone(),
            bindings: new_bindings,
        }
    }
}

/// 约束条件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Constraint {
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    Hard(Expression),
    Soft(Expression),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    GreaterThan(Value, Value),
    LessThan(Value, Value),
    Equals(Value, Value),
    Custom(String),
}

/// 判断两个值是否相等（基于余弦相似度用于向量）
pub fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => (x - y).abs() < f64::EPSILON,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Maybe, Value::Bool(b)) => *b == false, // maybe 视为 0.5
        (Value::Bool(b), Value::Maybe) => *b == false,
        (Value::Vector(v1), Value::Vector(v2)) => {
            cosine_similarity(v1, v2) > 0.95
        }
        _ => false,
    }
}

/// 计算余弦相似度
pub fn cosine_similarity(v1: &[f64], v2: &[f64]) -> f64 {
    if v1.len() != v2.len() || v1.is_empty() {
        return 0.0;
    }
    
    let dot_product: f64 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
    let norm1: f64 = v1.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm2: f64 = v2.iter().map(|x| x * x).sum::<f64>().sqrt();
    
    if norm1 == 0.0 || norm2 == 0.0 {
        0.0
    } else {
        dot_product / (norm1 * norm2)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::TokenSeq(tokens) => write!(f, "{:?}", tokens),
            Value::Vector(v) => write!(f, "@[{}]", v.iter().map(|x| format!("{:.2}", x)).collect::<Vec<_>>().join(", ")),
            Value::Dist(dist) => {
                let parts: Vec<String> = dist.outcomes.iter()
                    .map(|(p, v)| format!("{:.2}: {}", p, v))
                    .collect();
                write!(f, "~[{}]", parts.join(", "))
            }
            Value::Model(m) => write!(f, "<model: {}>", m.name),
            Value::Prompt(p) => write!(f, "<prompt: {}>", p.template.chars().take(20).collect::<String>()),
            Value::Constraint(_) => write!(f, "<constraint>"),
            Value::Maybe => write!(f, "maybe"),
        }
    }
}
