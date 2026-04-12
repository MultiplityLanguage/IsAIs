use isais::{Interpreter, models::MockLLM};

#[test]
fn test_basic_literals() {
    let model = Box::new(MockLLM);
    let mut interpreter = Interpreter::new(model);
    
    // 测试整数
    let result = interpreter.evaluate(":: 42").unwrap();
    assert!(matches!(result, isais::types::Value::Int(42)));
    
    // 测试字符串
    let result = interpreter.evaluate(r#":: "hello""#).unwrap();
    assert!(matches!(result, isais::types::Value::String(s) if s == "hello"));
}

#[test]
fn test_arithmetic() {
    let model = Box::new(MockLLM);
    let mut interpreter = Interpreter::new(model);
    
    let result = interpreter.evaluate(":: (+ 10 20)").unwrap();
    assert!(matches!(result, isais::types::Value::Int(30)));
}

#[test]
fn test_let_binding() {
    let model = Box::new(MockLLM);
    let mut interpreter = Interpreter::new(model);
    
    let code = r#":: (let [x 10] x)"#;
    let result = interpreter.evaluate(code).unwrap();
    assert!(matches!(result, isais::types::Value::Int(10)));
}
