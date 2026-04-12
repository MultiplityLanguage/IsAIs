use crate::types::{Value, Distribution};
use crate::parser::ASTNode;
use crate::Interpreter;
use crate::Error;

/// 求值 AST 节点
pub fn evaluate(ast: &ASTNode, interpreter: &mut Interpreter) -> Result<Value, Error> {
    match ast {
        ASTNode::Literal(value) => {
            // 如果是字符串，尝试从环境中查找变量
            if let Value::String(name) = value {
                if let Some(bound_value) = interpreter.env.get(name) {
                    Ok(bound_value.clone())
                } else {
                    Ok(value.clone())
                }
            } else {
                Ok(value.clone())
            }
        }
        
        ASTNode::Comment(_) => {
            // 注释不产生值
            Ok(Value::Maybe)
        }
        
        ASTNode::Query(query) => {
            evaluate_query(query, interpreter)
        }
        
        ASTNode::Imperative(action) => {
            evaluate_imperative(action, interpreter)
        }
        
        ASTNode::List(elements) => {
            evaluate_list(elements, interpreter)
        }
    }
}

/// 求值列表（函数调用或特殊形式）
fn evaluate_list(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.is_empty() {
        return Ok(Value::Maybe);
    }

    // 直接检查第一个元素，不求值
    match &elements[0] {
        ASTNode::Literal(Value::String(op)) => {
            // 首先检查是否是内置操作
            match op.as_str() {
                "let" => evaluate_let(&elements[1..], interpreter),
                "if" => evaluate_if(&elements[1..], interpreter),
                "def" => evaluate_def(&elements[1..], interpreter),
                "+" | "-" | "*" | "/" => evaluate_arithmetic(op, &elements[1..], interpreter),
                ">" | "<" | "=" => evaluate_comparison(op, &elements[1..], interpreter),
                "and" | "or" => evaluate_logical(op, &elements[1..], interpreter),
                "call" => evaluate_model_call(&elements[1..], interpreter),
                "prompt" => evaluate_prompt_creation(&elements[1..], interpreter),
                "partial" => evaluate_partial_application(&elements[1..], interpreter),
                "constrain" => evaluate_constraint(&elements[1..], interpreter),
                "generate" => evaluate_generate(&elements[1..], interpreter),
                "set-temperature" => evaluate_set_temperature(&elements[1..], interpreter),
                "remember" => evaluate_remember(&elements[1..], interpreter),
                "attention" => evaluate_attention(&elements[1..], interpreter),
                "classify" => evaluate_classify(&elements[1..], interpreter),
                "prob" => evaluate_prob(&elements[1..], interpreter),
                "assert" => evaluate_assert(&elements[1..], interpreter),
                _ => {
                    // 如果不是内置操作，尝试从环境中查找变量
                    if let Some(value) = interpreter.env.get(op) {
                        Ok(value.clone())
                    } else {
                        Err(Error::RuntimeError(format!("Unknown operation or variable: {}", op)))
                    }
                }
            }
        }
        _ => {
            // 如果第一个元素不是字符串，求值它
            let operator = evaluate(&elements[0], interpreter)?;
            Err(Error::RuntimeError(format!("Cannot call {:?}", operator)))
        }
    }
}

/// 求值 let 绑定
fn evaluate_let(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.len() < 2 {
        return Err(Error::RuntimeError("let requires bindings and body".to_string()));
    }

    // 解析绑定
    if let ASTNode::List(bindings) = &elements[0] {
        for binding in bindings.chunks(2) {
            if binding.len() == 2 {
                // 直接获取变量名字符串，不求值
                let name = if let ASTNode::Literal(Value::String(n)) = &binding[0] {
                    n.clone()
                } else {
                    return Err(Error::RuntimeError("let binding name must be a string".to_string()));
                };
                
                let value = evaluate(&binding[1], interpreter)?;
                interpreter.env.insert(name, value);
            }
        }
    }

    // 求值主体
    evaluate(&elements[1], interpreter)
}

/// 求值条件表达式
fn evaluate_if(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.len() < 3 {
        return Err(Error::RuntimeError("if requires condition, then-branch, else-branch".to_string()));
    }

    let condition = evaluate(&elements[0], interpreter)?;
    
    let is_true = match condition {
        Value::Bool(b) => b,
        Value::Maybe => false,
        Value::Dist(dist) => {
            // 如果分布中最可能的值是 true
            dist.most_likely().map_or(false, |v| {
                matches!(v, Value::Bool(true))
            })
        }
        _ => false,
    };

    if is_true {
        evaluate(&elements[1], interpreter)
    } else {
        evaluate(&elements[2], interpreter)
    }
}

/// 求值定义
fn evaluate_def(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.len() < 2 {
        return Err(Error::RuntimeError("def requires name and value".to_string()));
    }

    let name = evaluate(&elements[0], interpreter)?;
    let value = evaluate(&elements[1], interpreter)?;

    if let Value::String(n) = name {
        interpreter.env.insert(n, value.clone());
        Ok(value)
    } else {
        Err(Error::RuntimeError("def name must be a string".to_string()))
    }
}

/// 求值算术运算
fn evaluate_arithmetic(op: &str, elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.len() < 2 {
        return Err(Error::RuntimeError(format!("{} requires at least 2 operands", op)));
    }

    let mut result = evaluate(&elements[0], interpreter)?;
    
    for elem in &elements[1..] {
        let operand = evaluate(elem, interpreter)?;
        result = apply_arithmetic(op, &result, &operand)?;
    }

    Ok(result)
}

fn apply_arithmetic(op: &str, a: &Value, b: &Value) -> Result<Value, Error> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => {
            let result = match op {
                "+" => x + y,
                "-" => x - y,
                "*" => x * y,
                "/" => {
                    if *y == 0 {
                        return Err(Error::RuntimeError("Division by zero".to_string()));
                    }
                    x / y
                }
                _ => return Err(Error::RuntimeError(format!("Unknown operator: {}", op))),
            };
            Ok(Value::Int(result))
        }
        (Value::Float(x), Value::Float(y)) => {
            let result = match op {
                "+" => x + y,
                "-" => x - y,
                "*" => x * y,
                "/" => {
                    if *y == 0.0 {
                        return Err(Error::RuntimeError("Division by zero".to_string()));
                    }
                    x / y
                }
                _ => return Err(Error::RuntimeError(format!("Unknown operator: {}", op))),
            };
            Ok(Value::Float(result))
        }
        // 处理不确定性传播
        (Value::Dist(dist_a), _) => {
            let outcomes: Vec<(f64, Value)> = dist_a.outcomes.iter().map(|(p, v)| {
                apply_arithmetic(op, v, b).map(|result| (*p, result)).unwrap_or((*p, v.clone()))
            }).collect();
            Ok(Value::Dist(Distribution::new(outcomes)))
        }
        (_, Value::Dist(dist_b)) => {
            let outcomes: Vec<(f64, Value)> = dist_b.outcomes.iter().map(|(p, v)| {
                apply_arithmetic(op, a, v).map(|result| (*p, result)).unwrap_or((*p, v.clone()))
            }).collect();
            Ok(Value::Dist(Distribution::new(outcomes)))
        }
        _ => Err(Error::TypeError(format!("Cannot apply {} to {:?} and {:?}", op, a, b))),
    }
}

/// 求值比较运算
fn evaluate_comparison(op: &str, elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.len() < 2 {
        return Err(Error::RuntimeError(format!("{} requires 2 operands", op)));
    }

    let a = evaluate(&elements[0], interpreter)?;
    let b = evaluate(&elements[1], interpreter)?;

    let result = match op {
        ">" => compare_values(&a, &b) > 0,
        "<" => compare_values(&a, &b) < 0,
        "=" => crate::types::values_equal(&a, &b),
        _ => return Err(Error::RuntimeError(format!("Unknown comparison: {}", op))),
    };

    Ok(Value::Bool(result))
}

fn compare_values(a: &Value, b: &Value) -> i32 {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y) as i32,
        (Value::Float(x), Value::Float(y)) => {
            if x < y { -1 } else if x > y { 1 } else { 0 }
        }
        _ => 0,
    }
}

/// 求值逻辑运算
fn evaluate_logical(op: &str, elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.len() < 2 {
        return Err(Error::RuntimeError(format!("{} requires 2 operands", op)));
    }

    let a = evaluate(&elements[0], interpreter)?;
    let b = evaluate(&elements[1], interpreter)?;

    let result = match op {
        "and" => {
            matches!(a, Value::Bool(true) | Value::Maybe) && matches!(b, Value::Bool(true) | Value::Maybe)
        }
        "or" => {
            matches!(a, Value::Bool(true)) || matches!(b, Value::Bool(true))
        }
        _ => return Err(Error::RuntimeError(format!("Unknown logical operator: {}", op))),
    };

    Ok(Value::Bool(result))
}

/// 求值查询
fn evaluate_query(query: &ASTNode, interpreter: &mut Interpreter) -> Result<Value, Error> {
    // 构造查询 prompt
    let query_text = format!("{:?}", query);
    
    // 调用模型获取答案
    let response = interpreter.model.query(&query_text, interpreter.temperature)?;
    
    // 将响应转换为分布
    Ok(parse_model_response(&response))
}

/// 求值命令式动作
fn evaluate_imperative(action: &ASTNode, interpreter: &mut Interpreter) -> Result<Value, Error> {
    evaluate(action, interpreter)
}

/// 求值模型调用
fn evaluate_model_call(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.is_empty() {
        return Err(Error::RuntimeError("call requires a model".to_string()));
    }

    let _model_name = evaluate(&elements[0], interpreter)?;
    
    // 这里简化处理，实际应该从环境中查找模型
    let prompt_text = if elements.len() > 1 {
        format!("{:?}", &elements[1])
    } else {
        String::new()
    };
    
    let response = interpreter.model.query(&prompt_text, interpreter.temperature)?;
    Ok(parse_model_response(&response))
}

/// 求值 Prompt 创建
fn evaluate_prompt_creation(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.is_empty() {
        return Err(Error::RuntimeError("prompt requires a template".to_string()));
    }

    let template = evaluate(&elements[0], interpreter)?;
    
    if let Value::String(tmpl) = template {
        // 提取 holes ({{...}})
        let holes = extract_holes(&tmpl);
        
        Ok(Value::Prompt(crate::types::Prompt {
            template: tmpl,
            holes,
            bindings: std::collections::HashMap::new(),
        }))
    } else {
        Err(Error::RuntimeError("Prompt template must be a string".to_string()))
    }
}

fn extract_holes(template: &str) -> Vec<String> {
    let mut holes = Vec::new();
    let mut chars = template.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '{' {
            if let Some('{') = chars.peek() {
                chars.next();
                let mut hole = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '}' {
                        chars.next();
                        if let Some('}') = chars.peek() {
                            chars.next();
                            break;
                        }
                    }
                    hole.push(c);
                    chars.next();
                }
                holes.push(hole);
            }
        }
    }
    
    holes
}

/// 求值部分应用
fn evaluate_partial_application(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.len() < 3 {
        return Err(Error::RuntimeError("partial requires prompt, key, and value".to_string()));
    }

    let prompt_val = evaluate(&elements[0], interpreter)?;
    
    if let Value::Prompt(prompt) = prompt_val {
        let key = evaluate(&elements[1], interpreter)?;
        let value = evaluate(&elements[2], interpreter)?;
        
        if let Value::String(k) = key {
            Ok(Value::Prompt(prompt.partial_apply(&k, value)))
        } else {
            Err(Error::RuntimeError("Key must be a string".to_string()))
        }
    } else {
        Err(Error::RuntimeError("First argument must be a prompt".to_string()))
    }
}

/// 求值约束
fn evaluate_constraint(_elements: &[ASTNode], _interpreter: &mut Interpreter) -> Result<Value, Error> {
    // 简化实现：返回约束对象
    Ok(Value::Constraint(crate::types::Constraint {
        conditions: vec![],
    }))
}

/// 求值生成
fn evaluate_generate(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    let topic = if !elements.is_empty() {
        evaluate(&elements[0], interpreter)?
    } else {
        Value::String("".to_string())
    };

    let prompt = format!("Generate content about: {}", topic);
    let response = interpreter.model.query(&prompt, interpreter.temperature)?;
    
    Ok(parse_model_response(&response))
}

/// 求值设置温度
fn evaluate_set_temperature(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.is_empty() {
        return Err(Error::RuntimeError("set-temperature requires a value".to_string()));
    }

    let temp = evaluate(&elements[0], interpreter)?;
    
    match temp {
        Value::Float(t) => {
            interpreter.temperature = t;
            Ok(Value::Float(interpreter.temperature))
        }
        Value::Int(t) => {
            interpreter.temperature = t as f64;
            Ok(Value::Float(interpreter.temperature))
        }
        _ => Err(Error::RuntimeError("Temperature must be a number".to_string())),
    }
}

/// 求值记忆
fn evaluate_remember(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.is_empty() {
        return Err(Error::RuntimeError("remember requires content".to_string()));
    }

    let content = evaluate(&elements[0], interpreter)?;
    
    if let Value::String(text) = content {
        interpreter.memory.store_fact(&text);
        Ok(Value::Bool(true))
    } else {
        Err(Error::RuntimeError("Remembered content must be a string".to_string()))
    }
}

/// 求值注意力（简化版）
fn evaluate_attention(_elements: &[ASTNode], _interpreter: &mut Interpreter) -> Result<Value, Error> {
    // 简化实现：返回一个模拟的注意力权重分布
    Ok(Value::Dist(Distribution::new(vec![
        (0.8, Value::Float(0.9)),
        (0.2, Value::Float(0.1)),
    ])))
}

/// 求值分类
fn evaluate_classify(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.len() < 2 {
        return Err(Error::RuntimeError("classify requires text and labels".to_string()));
    }

    let text = evaluate(&elements[0], interpreter)?;
    let labels = evaluate(&elements[1], interpreter)?;
    
    // 简化实现：调用模型进行分类
    let prompt = format!("Classify: {:?} into labels: {:?}", text, labels);
    let response = interpreter.model.query(&prompt, interpreter.temperature)?;
    
    Ok(parse_model_response(&response))
}

/// 求值概率查询
fn evaluate_prob(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.is_empty() {
        return Err(Error::RuntimeError("prob requires a value".to_string()));
    }

    let _value = evaluate(&elements[0], interpreter)?;
    
    // 如果当前上下文中有分布，返回该值的概率
    // 这里简化实现
    Ok(Value::Float(0.5))
}

/// 求值断言
fn evaluate_assert(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.is_empty() {
        return Err(Error::RuntimeError("assert requires a condition".to_string()));
    }

    let condition = evaluate(&elements[0], interpreter)?;
    
    match condition {
        Value::Bool(true) => Ok(Value::Bool(true)),
        Value::Bool(false) => Err(Error::RuntimeError("Assertion failed".to_string())),
        _ => Ok(Value::Bool(true)), // 对于不确定值，假定通过
    }
}

/// 解析模型响应为分布
fn parse_model_response(response: &str) -> Value {
    // 简化实现：将响应包装为确定性分布
    Value::Dist(Distribution::new(vec![
        (1.0, Value::String(response.to_string())),
    ]))
}
