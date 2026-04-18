use crate::types::{Distribution, Value};
use crate::parser::ASTNode;
use crate::Interpreter;
use crate::Error;

/// 内置特殊形式名称
const SPECIAL_FORM_LET: &str = "let";
const SPECIAL_FORM_IF: &str = "if";
const SPECIAL_FORM_DEF: &str = "def";

/// 算术操作符
const OP_ADD: &str = "+";
const OP_SUB: &str = "-";
const OP_MUL: &str = "*";
const OP_DIV: &str = "/";

/// 比较操作符
const OP_GT: &str = ">";
const OP_LT: &str = "<";
const OP_EQ: &str = "=";

/// 逻辑操作符
const OP_AND: &str = "and";
const OP_OR: &str = "or";

/// 其他内置命令
const CMD_CALL: &str = "call";
const CMD_PROMPT: &str = "prompt";
const CMD_PARTIAL: &str = "partial";
const CMD_CONSTRAIN: &str = "constrain";
const CMD_GENERATE: &str = "generate";
const CMD_SET_TEMPERATURE: &str = "set-temperature";
const CMD_REMEMBER: &str = "remember";
const CMD_ATTENTION: &str = "attention";
const CMD_CLASSIFY: &str = "classify";
const CMD_PROB: &str = "prob";
const CMD_ASSERT: &str = "assert";

// -----------------------------------------------------------------------------
// 公共求值入口
// -----------------------------------------------------------------------------

/// 求值单个 AST 节点。
pub fn evaluate(ast: &ASTNode, interpreter: &mut Interpreter) -> Result<Value, Error> {
    match ast {
        ASTNode::Literal(value) => evaluate_literal(value, interpreter),
        ASTNode::Comment(_) => Ok(Value::Maybe),
        ASTNode::Query(query) => evaluate_query(query, interpreter),
        ASTNode::Imperative(action) => evaluate(action, interpreter), // 命令式动作直接递归求值
        ASTNode::List(elements) => evaluate_list(elements, interpreter),
    }
}

/// 处理字面量：若为字符串且环境中有同名绑定则返回绑定值，否则返回字面量本身。
fn evaluate_literal(value: &Value, interpreter: &mut Interpreter) -> Result<Value, Error> {
    if let Value::String(name) = value {
        if let Some(bound) = interpreter.env.get(name) {
            return Ok(bound.clone());
        }
    }
    Ok(value.clone())
}

// -----------------------------------------------------------------------------
// 列表求值（函数调用 / 特殊形式）
// -----------------------------------------------------------------------------

fn evaluate_list(elements: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    if elements.is_empty() {
        return Ok(Value::Maybe);
    }

    let first = &elements[0];

    // 若第一个元素是字符串，则尝试作为特殊形式或内置命令处理
    if let ASTNode::Literal(Value::String(op)) = first {
        return dispatch_special_form(op, &elements[1..], interpreter);
    }

    // 否则先求值第一个元素，再报错（当前实现不支持函数值调用）
    let evaluated_op = evaluate(first, interpreter)?;
    Err(Error::RuntimeError(format!(
        "Cannot call value of type {:?}",
        evaluated_op
    )))
}

/// 根据操作符名称分派到对应的处理函数。
fn dispatch_special_form(op: &str, args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    match op {
        SPECIAL_FORM_LET => eval_let(args, interpreter),
        SPECIAL_FORM_IF => eval_if(args, interpreter),
        SPECIAL_FORM_DEF => eval_def(args, interpreter),

        OP_ADD | OP_SUB | OP_MUL | OP_DIV => eval_arithmetic(op, args, interpreter),
        OP_GT | OP_LT | OP_EQ => eval_comparison(op, args, interpreter),
        OP_AND | OP_OR => eval_logical(op, args, interpreter),

        CMD_CALL => eval_model_call(args, interpreter),
        CMD_PROMPT => eval_prompt_creation(args, interpreter),
        CMD_PARTIAL => eval_partial_application(args, interpreter),
        CMD_CONSTRAIN => eval_constraint(args, interpreter),
        CMD_GENERATE => eval_generate(args, interpreter),
        CMD_SET_TEMPERATURE => eval_set_temperature(args, interpreter),
        CMD_REMEMBER => eval_remember(args, interpreter),
        CMD_ATTENTION => eval_attention(args, interpreter),
        CMD_CLASSIFY => eval_classify(args, interpreter),
        CMD_PROB => eval_prob(args, interpreter),
        CMD_ASSERT => eval_assert(args, interpreter),

        _ => {
            // 尝试从环境查找变量
            interpreter
                .env
                .get(op)
                .cloned()
                .ok_or_else(|| Error::RuntimeError(format!("Unknown operation or variable: {}", op)))
        }
    }
}

// -----------------------------------------------------------------------------
// 特殊形式实现
// -----------------------------------------------------------------------------

fn eval_let(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_arg_count("let", args.len(), 2)?;

    let (bindings_node, body) = (&args[0], &args[1]);

    // 解析绑定列表
    if let ASTNode::List(bindings) = bindings_node {
        for chunk in bindings.chunks(2) {
            if chunk.len() == 2 {
                let name = extract_string_literal(&chunk[0], "let binding name")?;
                let value = evaluate(&chunk[1], interpreter)?;
                interpreter.env.insert(name, value);
            }
        }
    }

    evaluate(body, interpreter)
}

fn eval_if(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_arg_count("if", args.len(), 3)?;

    let (cond, then_branch, else_branch) = (&args[0], &args[1], &args[2]);
    let cond_value = evaluate(cond, interpreter)?;

    let is_truthy = is_truthy(&cond_value);
    if is_truthy {
        evaluate(then_branch, interpreter)
    } else {
        evaluate(else_branch, interpreter)
    }
}

fn eval_def(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_arg_count("def", args.len(), 2)?;

    let name = evaluate(&args[0], interpreter)?;
    let value = evaluate(&args[1], interpreter)?;

    let name_str = extract_string_from_value(&name, "def name")?;
    interpreter.env.insert(name_str, value.clone());
    Ok(value)
}

// -----------------------------------------------------------------------------
// 算术运算
// -----------------------------------------------------------------------------

fn eval_arithmetic(op: &str, args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_arg_count(op, args.len(), 2)?;

    let mut acc = evaluate(&args[0], interpreter)?;
    for arg in &args[1..] {
        let next = evaluate(arg, interpreter)?;
        acc = apply_arithmetic(op, &acc, &next)?;
    }
    Ok(acc)
}

fn apply_arithmetic(op: &str, left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Int(x), Value::Int(y)) => {
            let result = binary_int_op(op, *x, *y)?;
            Ok(Value::Int(result))
        }
        (Value::Float(x), Value::Float(y)) => {
            let result = binary_float_op(op, *x, *y)?;
            Ok(Value::Float(result))
        }
        (Value::Dist(dist), other) | (other, Value::Dist(dist)) => {
            propagate_dist_arithmetic(op, dist, other)
        }
        _ => Err(Error::TypeError(format!(
            "Cannot apply {} to {:?} and {:?}",
            op, left, right
        ))),
    }
}

fn binary_int_op(op: &str, x: i64, y: i64) -> Result<i64, Error> {
    match op {
        OP_ADD => Ok(x + y),
        OP_SUB => Ok(x - y),
        OP_MUL => Ok(x * y),
        OP_DIV => {
            if y == 0 {
                Err(Error::RuntimeError("Division by zero".into()))
            } else {
                Ok(x / y)
            }
        }
        _ => Err(Error::RuntimeError(format!("Unknown operator: {}", op))),
    }
}

fn binary_float_op(op: &str, x: f64, y: f64) -> Result<f64, Error> {
    match op {
        OP_ADD => Ok(x + y),
        OP_SUB => Ok(x - y),
        OP_MUL => Ok(x * y),
        OP_DIV => {
            if y == 0.0 {
                Err(Error::RuntimeError("Division by zero".into()))
            } else {
                Ok(x / y)
            }
        }
        _ => Err(Error::RuntimeError(format!("Unknown operator: {}", op))),
    }
}

fn propagate_dist_arithmetic(op: &str, dist: &Distribution, other: &Value) -> Result<Value, Error> {
    let outcomes = dist
        .outcomes
        .iter()
        .filter_map(|(prob, val)| {
            apply_arithmetic(op, val, other)
                .ok()
                .map(|result| (*prob, result))
        })
        .collect();
    Ok(Value::Dist(Distribution::new(outcomes)))
}

// -----------------------------------------------------------------------------
// 比较运算
// -----------------------------------------------------------------------------

fn eval_comparison(op: &str, args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_arg_count(op, args.len(), 2)?;

    let left = evaluate(&args[0], interpreter)?;
    let right = evaluate(&args[1], interpreter)?;

    let result = match op {
        OP_GT => compare_values(&left, &right) == std::cmp::Ordering::Greater,
        OP_LT => compare_values(&left, &right) == std::cmp::Ordering::Less,
        OP_EQ => crate::types::values_equal(&left, &right),
        _ => return Err(Error::RuntimeError(format!("Unknown comparison: {}", op))),
    };

    Ok(Value::Bool(result))
}

fn compare_values(left: &Value, right: &Value) -> std::cmp::Ordering {
    match (left, right) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Float(x), Value::Float(y)) => {
            if x < y {
                std::cmp::Ordering::Less
            } else if x > y {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        }
        _ => std::cmp::Ordering::Equal, // 默认相等，保守处理
    }
}

// -----------------------------------------------------------------------------
// 逻辑运算
// -----------------------------------------------------------------------------

fn eval_logical(op: &str, args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_arg_count(op, args.len(), 2)?;

    let left = evaluate(&args[0], interpreter)?;
    let right = evaluate(&args[1], interpreter)?;

    let result = match op {
        OP_AND => is_truthy(&left) && is_truthy(&right),
        OP_OR => is_truthy(&left) || is_truthy(&right),
        _ => return Err(Error::RuntimeError(format!("Unknown logical operator: {}", op))),
    };

    Ok(Value::Bool(result))
}

/// 判断值是否为“真”（用于条件分支）。
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Maybe => false,
        Value::Dist(dist) => dist
            .most_likely()
            .map(|v| matches!(v, Value::Bool(true)))
            .unwrap_or(false),
        _ => false,
    }
}

// -----------------------------------------------------------------------------
// 模型调用相关命令
// -----------------------------------------------------------------------------

fn eval_model_call(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_min_arg_count("call", args.len(), 1)?;

    let _model_name = evaluate(&args[0], interpreter)?; // 保留但未使用，按原逻辑
    let prompt_text = args
        .get(1)
        .map(|node| format!("{:?}", node))
        .unwrap_or_default();

    let response = interpreter.model.query(&prompt_text, interpreter.temperature)?;
    Ok(wrap_response_as_distribution(&response))
}

fn eval_prompt_creation(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_min_arg_count("prompt", args.len(), 1)?;

    let template_val = evaluate(&args[0], interpreter)?;
    let template_str = extract_string_from_value(&template_val, "prompt template")?;

    let holes = extract_holes(&template_str);
    Ok(Value::Prompt(crate::types::Prompt {
        template: template_str,
        holes,
        bindings: std::collections::HashMap::new(),
    }))
}

fn eval_partial_application(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_arg_count("partial", args.len(), 3)?;

    let prompt_val = evaluate(&args[0], interpreter)?;
    let key_val = evaluate(&args[1], interpreter)?;
    let value = evaluate(&args[2], interpreter)?;

    let prompt = extract_prompt_from_value(&prompt_val, "partial first argument")?;
    let key = extract_string_from_value(&key_val, "partial key")?;

    Ok(Value::Prompt(prompt.partial_apply(&key, value)))
}

fn eval_constraint(_args: &[ASTNode], _interpreter: &mut Interpreter) -> Result<Value, Error> {
    Ok(Value::Constraint(crate::types::Constraint {
        conditions: vec![],
    }))
}

fn eval_generate(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    let topic = if let Some(node) = args.first() {
        evaluate(node, interpreter)?
    } else {
        Value::String(String::new())
    };

    let prompt = format!("Generate content about: {}", topic);
    let response = interpreter.model.query(&prompt, interpreter.temperature)?;
    Ok(wrap_response_as_distribution(&response))
}

fn eval_set_temperature(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_min_arg_count("set-temperature", args.len(), 1)?;

    let temp_val = evaluate(&args[0], interpreter)?;
    match temp_val {
        Value::Float(t) => interpreter.temperature = t,
        Value::Int(t) => interpreter.temperature = t as f64,
        _ => return Err(Error::RuntimeError("Temperature must be a number".into())),
    }

    Ok(Value::Float(interpreter.temperature))
}

fn eval_remember(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_min_arg_count("remember", args.len(), 1)?;

    let content_val = evaluate(&args[0], interpreter)?;
    let content_str = extract_string_from_value(&content_val, "remembered content")?;

    interpreter.memory.store_fact(&content_str);
    Ok(Value::Bool(true))
}

fn eval_attention(_args: &[ASTNode], _interpreter: &mut Interpreter) -> Result<Value, Error> {
    Ok(Value::Dist(Distribution::new(vec![
        (0.8, Value::Float(0.9)),
        (0.2, Value::Float(0.1)),
    ])))
}

fn eval_classify(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_arg_count("classify", args.len(), 2)?;

    let text = evaluate(&args[0], interpreter)?;
    let labels = evaluate(&args[1], interpreter)?;

    let prompt = format!("Classify: {:?} into labels: {:?}", text, labels);
    let response = interpreter.model.query(&prompt, interpreter.temperature)?;
    Ok(wrap_response_as_distribution(&response))
}

fn eval_prob(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_min_arg_count("prob", args.len(), 1)?;

    let _value = evaluate(&args[0], interpreter)?;
    // 简化实现：返回固定概率值
    Ok(Value::Float(0.5))
}

fn eval_assert(args: &[ASTNode], interpreter: &mut Interpreter) -> Result<Value, Error> {
    ensure_min_arg_count("assert", args.len(), 1)?;

    let cond = evaluate(&args[0], interpreter)?;
    match cond {
        Value::Bool(true) => Ok(Value::Bool(true)),
        Value::Bool(false) => Err(Error::RuntimeError("Assertion failed".into())),
        _ => Ok(Value::Bool(true)), // 对不确定值宽松通过
    }
}

// -----------------------------------------------------------------------------
// 查询与命令式动作（直接求值）
// -----------------------------------------------------------------------------

fn evaluate_query(query: &ASTNode, interpreter: &mut Interpreter) -> Result<Value, Error> {
    let query_text = format!("{:?}", query);
    let response = interpreter.model.query(&query_text, interpreter.temperature)?;
    Ok(wrap_response_as_distribution(&response))
}

// -----------------------------------------------------------------------------
// 辅助函数
// -----------------------------------------------------------------------------

/// 确保参数个数正好为 expected。
fn ensure_arg_count(op: &str, actual: usize, expected: usize) -> Result<(), Error> {
    if actual < expected {
        Err(Error::RuntimeError(format!(
            "{} requires {} arguments, got {}",
            op, expected, actual
        )))
    } else {
        Ok(())
    }
}

/// 确保参数个数至少为 min。
fn ensure_min_arg_count(op: &str, actual: usize, min: usize) -> Result<(), Error> {
    if actual < min {
        Err(Error::RuntimeError(format!(
            "{} requires at least {} arguments, got {}",
            op, min, actual
        )))
    } else {
        Ok(())
    }
}

/// 从 AST 节点中提取字符串字面量。
fn extract_string_literal(node: &ASTNode, context: &str) -> Result<String, Error> {
    if let ASTNode::Literal(Value::String(s)) = node {
        Ok(s.clone())
    } else {
        Err(Error::RuntimeError(format!("{} must be a string", context)))
    }
}

/// 从 Value 中提取字符串。
fn extract_string_from_value(value: &Value, context: &str) -> Result<String, Error> {
    if let Value::String(s) = value {
        Ok(s.clone())
    } else {
        Err(Error::RuntimeError(format!("{} must be a string", context)))
    }
}

/// 从 Value 中提取 Prompt。
fn extract_prompt_from_value(value: &Value, context: &str) -> Result<crate::types::Prompt, Error> {
    if let Value::Prompt(p) = value {
        Ok(p.clone())
    } else {
        Err(Error::RuntimeError(format!("{} must be a prompt", context)))
    }
}

/// 将模型响应字符串包装为确定性分布。
fn wrap_response_as_distribution(response: &str) -> Value {
    Value::Dist(Distribution::new(vec![(
        1.0,
        Value::String(response.to_string()),
    )]))
}

/// 从模板字符串中提取所有 `{{...}}` 内的洞名。
fn extract_holes(template: &str) -> Vec<String> {
    let mut holes = Vec::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' && chars.peek() == Some(&'{') {
            chars.next(); // 跳过第二个 '{'
            let mut hole = String::new();
            while let Some(&c) = chars.peek() {
                if c == '}' && chars.clone().nth(1) == Some('}') {
                    chars.next();
                    chars.next();
                    break;
                }
                hole.push(c);
                chars.next();
            }
            holes.push(hole);
        }
    }

    holes
}
