// Anthropic 格式请求/响应解析器
// 移植自 packages/monitor/src/parsers/anthropic-parser.ts

use std::collections::HashMap;

use crate::monitor::parser::openai::extract_usage;
use crate::monitor::types::{ParsedRequestBody, ParsedResponseBody};

/// 解析 Anthropic 格式请求体
pub fn parse_anthropic_request(body: &serde_json::Value) -> ParsedRequestBody {
    let mut parsed = ParsedRequestBody {
        messages: None,
        prompt: None,
        temperature: None,
        max_tokens: None,
        thinking: None,
        thinking_budget: None,
        reasoning_effort: None,
        reasoning: None,
        extended_thinking: None,
        extra: HashMap::new(),
    };

    if !body.is_object() {
        return parsed;
    }

    let empty_map = serde_json::Map::new();
    let obj = body.as_object().unwrap_or(&empty_map);

    // Anthropic 请求体的标准字段
    parsed.messages = obj.get("messages").and_then(|v| {
        if v.is_array() {
            Some(v.as_array().unwrap().clone())
        } else {
            None
        }
    });

    parsed.temperature = obj.get("temperature").and_then(|v| v.as_f64());

    parsed.max_tokens = obj.get("max_tokens").and_then(|v| v.as_i64());

    parsed.thinking = obj.get("thinking").cloned();
    parsed.thinking_budget = obj.get("thinking_budget").and_then(|v| v.as_i64());

    // 收集其余字段到 extra
    let handled_keys = [
        "messages",
        "prompt",
        "temperature",
        "max_tokens",
        "thinking",
        "thinking_budget",
        "reasoning_effort",
        "reasoning",
        "extended_thinking",
    ];

    for (key, value) in obj {
        if !handled_keys.contains(&key.as_str()) && !value.is_null() {
            parsed.extra.insert(key.clone(), value.clone());
        }
    }

    parsed
}

/// 解析 Anthropic 格式响应体
/// 支持 content blocks (text, thinking, redacted_thinking)
/// Anthropic 用 input_tokens/output_tokens 而非 prompt_tokens/completion_tokens
pub fn parse_anthropic_response(body: &serde_json::Value) -> ParsedResponseBody {
    let mut result = ParsedResponseBody {
        content: None,
        thinking: None,
        choices: None,
        usage: None,
        extra: HashMap::new(),
    };

    if !body.is_object() {
        return result;
    }

    let empty_map = serde_json::Map::new();
    let obj = body.as_object().unwrap_or(&empty_map);

    // Anthropic 响应的核心：content 数组
    if let Some(content_arr) = obj.get("content").and_then(|v| v.as_array()) {
        let mut text_parts: Vec<String> = Vec::new();

        for block in content_arr {
            let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");

            match block_type {
                "thinking" => {
                    if let Some(thinking_text) = block.get("thinking").and_then(|v| v.as_str()) {
                        result.thinking = Some(thinking_text.to_string());
                    }
                }
                "text" => {
                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                        text_parts.push(text.to_string());
                    }
                }
                "redacted_thinking" => {
                    result.thinking = Some("[思考内容已隐藏]".to_string());
                }
                _ => {
                    // 其他类型的 block 忽略
                }
            }
        }

        if !text_parts.is_empty() {
            result.content = Some(text_parts.join("\n"));
        }
    } else if let Some(content_val) = obj.get("content") {
        // 非数组 content（罕见）：尝试从第一个元素提取 text
        if let Some(first) = content_val.as_array().and_then(|a| a.first()) {
            if let Some(text) = first.get("text").and_then(|v| v.as_str()) {
                result.content = Some(text.to_string());
            }
        }
    }

    // 旧版 completion 格式回退
    if result.content.is_none() {
        if let Some(completion) = obj.get("completion").and_then(|v| v.as_str()) {
            result.content = Some(completion.to_string());
        }
    }

    // 提取 usage（复用 openai 模块的 extract_usage，已支持 Anthropic 格式）
    result.usage = extract_usage(obj);

    // 保留其他字段到 extra
    let handled_keys = ["content", "usage", "choices", "thinking"];
    for (key, value) in obj {
        if !handled_keys.contains(&key.as_str()) && !value.is_null() {
            result.extra.insert(key.clone(), value.clone());
        }
    }

    result
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_anthropic_request_basic() {
        let body = json!({
            "model": "claude-3-opus-20240229",
            "messages": [{"role": "user", "content": "Hello"}],
            "max_tokens": 1024,
            "temperature": 0.5,
            "system": "You are a helpful assistant."
        });

        let parsed = parse_anthropic_request(&body);

        assert!(parsed.messages.is_some());
        assert_eq!(parsed.max_tokens, Some(1024));
        assert_eq!(parsed.temperature, Some(0.5));
        // model 和 system 应该在 extra 中
        assert!(parsed.extra.contains_key("model"));
        assert!(parsed.extra.contains_key("system"));
    }

    #[test]
    fn test_parse_anthropic_request_with_thinking() {
        let body = json!({
            "model": "claude-3-5-sonnet-20241022",
            "messages": [{"role": "user", "content": "Think carefully"}],
            "max_tokens": 16000,
            "thinking": {"type": "enabled", "budget_tokens": 10000},
            "thinking_budget": 10000
        });

        let parsed = parse_anthropic_request(&body);

        assert!(parsed.thinking.is_some());
        assert_eq!(parsed.thinking_budget, Some(10000));
    }

    #[test]
    fn test_parse_anthropic_request_empty() {
        let body = json!({});
        let parsed = parse_anthropic_request(&body);
        assert!(parsed.messages.is_none());
    }

    #[test]
    fn test_parse_anthropic_response_text_blocks() {
        let body = json!({
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "Hello there!"}
            ],
            "model": "claude-3-opus-20240229",
            "usage": {
                "input_tokens": 10,
                "output_tokens": 5
            }
        });

        let parsed = parse_anthropic_response(&body);

        assert_eq!(parsed.content, Some("Hello there!".to_string()));
        assert!(parsed.thinking.is_none());

        let usage = parsed.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 5);
        assert_eq!(usage.total_tokens, 15);
    }

    #[test]
    fn test_parse_anthropic_response_with_thinking() {
        let body = json!({
            "id": "msg_456",
            "type": "message",
            "content": [
                {"type": "thinking", "thinking": "Let me analyze this..."},
                {"type": "text", "text": "The answer is 42."},
                {"type": "redacted_thinking"}
            ],
            "model": "claude-3-5-sonnet-20241022",
            "usage": {
                "input_tokens": 20,
                "output_tokens": 30,
                "total_tokens": 50
            }
        });

        let parsed = parse_anthropic_response(&body);

        // 注意：redacted_thinking 会覆盖 thinking 的值
        // 因为遍历顺序是 thinking -> text -> redacted_thinking
        // redacted_thinking 最后出现，会覆盖之前设置的 thinking
        assert_eq!(parsed.thinking, Some("[思考内容已隐藏]".to_string()));
        assert_eq!(parsed.content, Some("The answer is 42.".to_string()));
    }

    #[test]
    fn test_parse_anthropic_response_thinking_before_redacted() {
        // 当 thinking block 在 redacted_thinking 之前时
        // redacted_thinking 会覆盖 thinking
        let body = json!({
            "content": [
                {"type": "thinking", "thinking": "My reasoning process..."},
                {"type": "text", "text": "Here is my answer."}
            ],
            "usage": {
                "input_tokens": 10,
                "output_tokens": 10
            }
        });

        let parsed = parse_anthropic_response(&body);

        assert_eq!(parsed.thinking, Some("My reasoning process...".to_string()));
        assert_eq!(parsed.content, Some("Here is my answer.".to_string()));
    }

    #[test]
    fn test_parse_anthropic_response_multiple_text_blocks() {
        let body = json!({
            "content": [
                {"type": "text", "text": "First part."},
                {"type": "text", "text": "Second part."}
            ]
        });

        let parsed = parse_anthropic_response(&body);

        // 多个 text block 用换行拼接
        assert_eq!(
            parsed.content,
            Some("First part.\nSecond part.".to_string())
        );
    }

    #[test]
    fn test_parse_anthropic_response_completion_fallback() {
        // 旧版 completion 格式
        let body = json!({
            "completion": "This is a completion."
        });

        let parsed = parse_anthropic_response(&body);

        assert_eq!(parsed.content, Some("This is a completion.".to_string()));
    }
}
