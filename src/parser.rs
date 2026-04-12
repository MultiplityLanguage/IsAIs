use crate::types::Value;
use crate::Error;

/// AST 节点
#[derive(Debug, Clone)]
pub enum ASTNode {
    Literal(Value),
    List(Vec<ASTNode>),
    Query(Box<ASTNode>),      // ::? query
    Imperative(Box<ASTNode>), // ::! action
    Comment(String),          // ::/ comment
}

/// 解析 IsAIs 源代码
pub fn parse(source: &str) -> Result<ASTNode, Error> {
    let tokens = tokenize(source);
    let mut pos = 0;
    parse_expression(&tokens, &mut pos)
}

/// 简单的词法分析器
fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            // 跳过空白和注释
            ' ' | '\t' | '\n' | '\r' => {
                i += 1;
            }
            ';' => {
                // 行注释
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            // 模式标签
            ':' if i + 1 < chars.len() && chars[i + 1] == ':' => {
                i += 2;
                if i < chars.len() {
                    match chars[i] {
                        '?' => {
                            tokens.push(Token::QueryTag);
                            i += 1;
                        }
                        '!' => {
                            tokens.push(Token::ImperativeTag);
                            i += 1;
                        }
                        '/' => {
                            tokens.push(Token::CommentTag);
                            i += 1;
                        }
                        '~' => {
                            tokens.push(Token::DistTag);
                            i += 1;
                        }
                        '@' => {
                            tokens.push(Token::EmbeddingTag);
                            i += 1;
                        }
                        '&' => {
                            tokens.push(Token::ReferenceTag);
                            i += 1;
                        }
                        _ => {
                            tokens.push(Token::CodeTag);
                        }
                    }
                } else {
                    tokens.push(Token::CodeTag);
                }
            }
            // S-表达式括号
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '[' => {
                tokens.push(Token::LBracket);
                i += 1;
            }
            ']' => {
                tokens.push(Token::RBracket);
                i += 1;
            }
            // 字符串
            '"' => {
                i += 1;
                let mut s = String::new();
                while i < chars.len() && chars[i] != '"' {
                    s.push(chars[i]);
                    i += 1;
                }
                i += 1; // 跳过结束引号
                tokens.push(Token::String(s));
            }
            // 数字和其他标识符
            c if c.is_alphanumeric() || c == '-' || c == '.' => {
                let mut s = String::new();
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '.' || chars[i] == '_') {
                    s.push(chars[i]);
                    i += 1;
                }
                tokens.push(Token::Ident(s));
            }
            _ => {
                i += 1;
            }
        }
    }

    tokens
}

#[derive(Debug, Clone)]
enum Token {
    CodeTag,
    QueryTag,
    ImperativeTag,
    CommentTag,
    DistTag,
    EmbeddingTag,
    ReferenceTag,
    LParen,
    RParen,
    LBracket,
    RBracket,
    String(String),
    Ident(String),
}

/// 递归下降解析器
fn parse_expression(tokens: &[Token], pos: &mut usize) -> Result<ASTNode, Error> {
    if *pos >= tokens.len() {
        return Err(Error::ParseError("Unexpected end of input".to_string()));
    }

    match &tokens[*pos] {
        Token::CodeTag => {
            // :: 是默认的代码模式，跳过它并解析下一个表达式
            *pos += 1;
            if *pos >= tokens.len() {
                return Err(Error::ParseError("Expected expression after ::".to_string()));
            }
            parse_expression(tokens, pos)
        }
        Token::QueryTag => {
            *pos += 1;
            let expr = parse_expression(tokens, pos)?;
            Ok(ASTNode::Query(Box::new(expr)))
        }
        Token::ImperativeTag => {
            *pos += 1;
            let expr = parse_expression(tokens, pos)?;
            Ok(ASTNode::Imperative(Box::new(expr)))
        }
        Token::CommentTag => {
            *pos += 1;
            if *pos < tokens.len() {
                if let Token::String(s) = &tokens[*pos] {
                    *pos += 1;
                    Ok(ASTNode::Comment(s.clone()))
                } else {
                    Err(Error::ParseError("Expected string after ::/".to_string()))
                }
            } else {
                Err(Error::ParseError("Expected string after ::/".to_string()))
            }
        }
        Token::LParen => {
            *pos += 1;
            let mut elements = Vec::new();
            while *pos < tokens.len() && !matches!(tokens[*pos], Token::RParen) {
                elements.push(parse_expression(tokens, pos)?);
            }
            if *pos >= tokens.len() {
                return Err(Error::ParseError("Unmatched parenthesis".to_string()));
            }
            *pos += 1; // 跳过 )
            Ok(ASTNode::List(elements))
        }
        Token::LBracket => {
            // 解析向量或列表
            *pos += 1;
            let mut elements = Vec::new();
            while *pos < tokens.len() && !matches!(tokens[*pos], Token::RBracket) {
                elements.push(parse_expression(tokens, pos)?);
            }
            if *pos >= tokens.len() {
                return Err(Error::ParseError("Unmatched bracket".to_string()));
            }
            *pos += 1; // 跳过 ]
            Ok(ASTNode::List(elements))
        }
        Token::String(s) => {
            *pos += 1;
            Ok(ASTNode::Literal(Value::String(s.clone())))
        }
        Token::Ident(s) => {
            *pos += 1;
            // 尝试解析为数字
            if let Ok(n) = s.parse::<i64>() {
                Ok(ASTNode::Literal(Value::Int(n)))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(ASTNode::Literal(Value::Float(f)))
            } else if s == "true" {
                Ok(ASTNode::Literal(Value::Bool(true)))
            } else if s == "false" {
                Ok(ASTNode::Literal(Value::Bool(false)))
            } else if s == "maybe" {
                Ok(ASTNode::Literal(Value::Maybe))
            } else {
                // 作为标识符/变量名
                Ok(ASTNode::Literal(Value::String(s.clone())))
            }
        }
        _ => Err(Error::ParseError(format!("Unexpected token: {:?}", tokens[*pos]))),
    }
}
