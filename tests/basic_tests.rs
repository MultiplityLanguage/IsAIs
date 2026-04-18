#[cfg(test)]
mod interpreter_tests {
    use isais::types::Value;
    use isais::{Interpreter, MockLLM};

    /// 创建一个使用 MockLLM 的解释器实例。
    fn setup() -> Interpreter {
        let model = Box::new(MockLLM);
        Interpreter::new(model)
    }

    #[test]
    fn evaluate_basic_literals() {
        let mut interpreter = setup();

        // 整数字面量
        let result = interpreter.evaluate(":: 42").unwrap();
        assert_eq!(result, Value::Int(42));

        // 字符串字面量
        let result = interpreter.evaluate(r#":: "hello""#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn evaluate_arithmetic_expression() {
        let mut interpreter = setup();

        let result = interpreter.evaluate(":: (+ 10 20)").unwrap();
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn evaluate_let_binding() {
        let mut interpreter = setup();

        let code = r#":: (let [x 10] x)"#;
        let result = interpreter.evaluate(code).unwrap();
        assert_eq!(result, Value::Int(10));
    }
}
