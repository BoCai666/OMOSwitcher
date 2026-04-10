// OpenAI 格式请求/响应解析器
// 移植自 packages/monitor/src/parsers/openai-parser.ts 和 request-capture.ts

use std::collections::HashMap;

use crate::monitor::types::{ParsedRequestBody, ParsedResponseBody, Usage};

/// 从请求体中提取关键字段名列表
/// 对应 request-capture.ts 中的 extractKeyFields
const KEY_FIELDS: &[&str] = &[
    "model",
    "messages",
    "prompt",
    "temperature",
    "max_tokens",
    "max_completion_tokens",
    "top_p",
    "top_k",
    "frequency_penalty",
    "presence_penalty",
    "stop",
    "stream",
    "n",
    "system",
    "tools",
    "tool_choice",
    // 思考相关字段
    "thinking",
    "thinking_budget",
    "reasoning_effort",
    "reasoning",
    "extended_thinking",
];

/// 解析 OpenAI 格式请求体
/// 从 body 中提取关键字段，放入 ParsedRequestBody 的对应字段和 extra 中
pub fn parse_openai_request(body: &serde_json::Value) -> ParsedRequestBody {
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

    // 提取已知的强类型字段
    parsed.messages = obj.get("messages").and_then(|v| {
        if v.is_array() {
            Some(v.as_array().unwrap().clone())
        } else {
            None
        }
    });

    parsed.prompt = obj
        .get("prompt")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    parsed.temperature = obj.get("temperature").and_then(|v| v.as_f64());

    parsed.max_tokens = obj
        .get("max_tokens")
        .or_else(|| obj.get("max_completion_tokens"))
        .and_then(|v| v.as_i64());

    parsed.thinking = obj.get("thinking").cloned();
    parsed.thinking_budget = obj.get("thinking_budget").and_then(|v| v.as_i64());
    parsed.reasoning_effort = obj
        .get("reasoning_effort")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    parsed.reasoning = obj.get("reasoning").cloned();
    parsed.extended_thinking = obj.get("extended_thinking").cloned();

    // 收集其余关键字段到 extra
    let handled_keys = [
        "messages",
        "prompt",
        "temperature",
        "max_tokens",
        "max_completion_tokens",
        "thinking",
        "thinking_budget",
        "reasoning_effort",
        "reasoning",
        "extended_thinking",
    ];

    for &field in KEY_FIELDS {
        if let Some(value) = obj.get(field) {
            // 跳过已处理为强类型的字段
            if !handled_keys.contains(&field) && !value.is_null() {
                parsed.extra.insert(field.to_string(), value.clone());
            }
        }
    }

    parsed
}

/// 解析 OpenAI 格式响应体
/// 支持：
/// - choices[0].message.content → content
/// - choices[0].message.reasoning_content → thinking (DeepSeek R1)
/// - choices[0].text → content (旧版 completions)
/// - usage → Usage
/// - 保留原始 choices
pub fn parse_openai_response(body: &serde_json::Value) -> ParsedResponseBody {
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

    // 提取 choices
    if let Some(choices) = obj.get("choices").and_then(|v| v.as_array()) {
        result.choices = Some(choices.clone());

        if let Some(first_choice) = choices.first() {
            // 尝试从 message 中提取
            if let Some(message) = first_choice.get("message") {
                // DeepSeek R1 格式：reasoning_content 字段
                if let Some(reasoning) = message.get("reasoning_content").and_then(|v| v.as_str()) {
                    result.thinking = Some(reasoning.to_string());
                }

                // 主内容
                if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                    result.content = Some(content.to_string());
                }
            }

            // 旧版 completions 格式：choice.text
            if result.content.is_none() {
                if let Some(text) = first_choice.get("text").and_then(|v| v.as_str()) {
                    result.content = Some(text.to_string());
                }
            }
        }
    }

    // 提取 usage
    result.usage = extract_usage(obj);

    // 保留其他字段到 extra
    let handled_keys = ["choices", "usage", "content", "thinking"];
    for (key, value) in obj {
        if !handled_keys.contains(&key.as_str()) && !value.is_null() {
            result.extra.insert(key.clone(), value.clone());
        }
    }

    result
}

/// 从 JSON 对象中提取 usage 信息
/// 支持多种字段命名格式：
/// - OpenAI: prompt_tokens, completion_tokens, total_tokens
/// - Anthropic: input_tokens, output_tokens
/// - 火山引擎: 根级别 input_tokens, output_tokens
pub fn extract_usage(obj: &serde_json::Map<String, serde_json::Value>) -> Option<Usage> {
    // 从 body.usage 中提取
    if let Some(usage_obj) = obj.get("usage").and_then(|v| v.as_object()) {
        let prompt = usage_obj
            .get("prompt_tokens")
            .or_else(|| usage_obj.get("input_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let completion = usage_obj
            .get("completion_tokens")
            .or_else(|| usage_obj.get("output_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let total = usage_obj
            .get("total_tokens")
            .or_else(|| usage_obj.get("total"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // 如果 total_tokens 为 0 但有 input/output，计算 total
        let total = if total == 0 && (prompt > 0 || completion > 0) {
            prompt + completion
        } else {
            total
        };

        return Some(Usage {
            prompt_tokens: prompt,
            completion_tokens: completion,
            total_tokens: total,
        });
    }

    // 一些提供商在根级别返回 token 使用情况
    let root_input = obj
        .get("input_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let root_output = obj
        .get("output_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let root_total = obj
        .get("total_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    if root_input > 0 || root_output > 0 || root_total > 0 {
        let total = if root_total > 0 {
            root_total
        } else {
            root_input + root_output
        };

        return Some(Usage {
            prompt_tokens: root_input,
            completion_tokens: root_output,
            total_tokens: total,
        });
    }

    None
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_openai_request_basic() {
        let body = json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hello"}],
            "temperature": 0.7,
            "max_tokens": 100,
            "stream": true,
            "tools": [{"type": "function", "function": {"name": "test"}}]
        });

        let parsed = parse_openai_request(&body);

        assert_eq!(parsed.temperature, Some(0.7));
        assert_eq!(parsed.max_tokens, Some(100));
        assert!(parsed.messages.is_some());
        assert_eq!(parsed.messages.as_ref().unwrap().len(), 1);

        // stream 和 tools 应该在 extra 中
        assert!(parsed.extra.contains_key("stream"));
        assert!(parsed.extra.contains_key("tools"));
    }

    #[test]
    fn test_parse_openai_request_with_thinking() {
        let body = json!({
            "model": "deepseek-r1",
            "messages": [{"role": "user", "content": "Think"}],
            "thinking": {"type": "enabled", "budget_tokens": 10000},
            "thinking_budget": 10000
        });

        let parsed = parse_openai_request(&body);

        assert!(parsed.thinking.is_some());
        assert_eq!(parsed.thinking_budget, Some(10000));
        assert_eq!(
            parsed.thinking.as_ref().unwrap().get("type"),
            Some(&json!("enabled"))
        );
    }

    #[test]
    fn test_parse_openai_request_empty_body() {
        let body = json!(null);
        let parsed = parse_openai_request(&body);
        assert!(parsed.messages.is_none());
        assert!(parsed.extra.is_empty());

        let body = json!("not an object");
        let parsed = parse_openai_request(&body);
        assert!(parsed.messages.is_none());
    }

    #[test]
    fn test_parse_openai_response_basic() {
        let body = json!({
            "id": "chatcmpl-123",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! How can I help you?"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 8,
                "total_tokens": 18
            }
        });

        let parsed = parse_openai_response(&body);

        assert_eq!(
            parsed.content,
            Some("Hello! How can I help you?".to_string())
        );
        assert!(parsed.thinking.is_none());
        assert!(parsed.choices.is_some());
        assert!(parsed.usage.is_some());

        let usage = parsed.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 8);
        assert_eq!(usage.total_tokens, 18);
    }

    #[test]
    fn test_parse_openai_response_deepseek_thinking() {
        let body = json!({
            "id": "chatcmpl-456",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "The answer is 42.",
                    "reasoning_content": "Let me think about this step by step..."
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 20,
                "completion_tokens": 30,
                "total_tokens": 50
            }
        });

        let parsed = parse_openai_response(&body);

        assert_eq!(parsed.content, Some("The answer is 42.".to_string()));
        assert_eq!(
            parsed.thinking,
            Some("Let me think about this step by step...".to_string())
        );
    }

    #[test]
    fn test_parse_openai_response_completions_format() {
        // 旧版 completions 格式：使用 text 而非 message
        let body = json!({
            "id": "cmpl-789",
            "choices": [{
                "text": "This is a completion response.",
                "index": 0,
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 5,
                "completion_tokens": 6,
                "total_tokens": 11
            }
        });

        let parsed = parse_openai_response(&body);

        assert_eq!(
            parsed.content,
            Some("This is a completion response.".to_string())
        );
    }

    #[test]
    fn test_parse_openai_response_no_choices() {
        let body = json!({
            "id": "chatcmpl-empty",
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 0,
                "total_tokens": 10
            }
        });

        let parsed = parse_openai_response(&body);

        assert!(parsed.content.is_none());
        assert!(parsed.choices.is_none());
        assert!(parsed.usage.is_some());
    }

    #[test]
    fn test_extract_usage_anthropic_format() {
        let obj = json!({
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50
            }
        })
        .as_object()
        .unwrap()
        .clone();

        let usage = extract_usage(&obj).unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        // total_tokens 应该由计算得出
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_extract_usage_root_level_tokens() {
        let obj = json!({
            "input_tokens": 200,
            "output_tokens": 100,
            "total_tokens": 300
        })
        .as_object()
        .unwrap()
        .clone();

        let usage = extract_usage(&obj).unwrap();
        assert_eq!(usage.prompt_tokens, 200);
        assert_eq!(usage.completion_tokens, 100);
        assert_eq!(usage.total_tokens, 300);
    }

    #[test]
    fn test_extract_usage_no_usage() {
        let obj = json!({"model": "gpt-4"}).as_object().unwrap().clone();

        let usage = extract_usage(&obj);
        assert!(usage.is_none());
    }
}
