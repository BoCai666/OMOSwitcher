// SSE 流式响应解析器
// 增量累积流式响应的 content、thinking 和 usage
// 移植自 packages/monitor/src/proxy/response-capture.ts 中的流式解析逻辑

use std::collections::HashMap;

use crate::monitor::types::{ParsedResponseBody, Usage};

/// SSE 解析器状态
/// 增量累积流式响应的 content、thinking 和 usage
pub struct SseParser {
    /// 累积的文本内容片段
    content_parts: Vec<String>,
    /// 累积的思考内容片段
    thinking_parts: Vec<String>,
    /// 累积的 usage（通常在最后一个 chunk 中出现）
    usage: Option<Usage>,
    /// 模型名称（可能在流式 chunk 中出现）
    model: Option<String>,
    /// 已处理的 chunk 数量
    chunk_count: usize,
    /// 是否已完成（收到 [DONE] 信号）
    done: bool,
    /// 未处理的缓冲区（用于处理跨 chunk 的 SSE 行）
    buffer: String,
}

impl SseParser {
    /// 创建新的 SSE 解析器
    pub fn new() -> Self {
        Self {
            content_parts: Vec::new(),
            thinking_parts: Vec::new(),
            usage: None,
            model: None,
            chunk_count: 0,
            done: false,
            buffer: String::new(),
        }
    }

    /// 处理原始 SSE 文本（可能包含多行）
    /// 输入可以是完整的 SSE chunk 或部分文本
    pub fn feed_chunk(&mut self, chunk: &str) {
        self.buffer.push_str(chunk);

        // 按行分割处理
        while let Some(pos) = self.buffer.find('\n') {
            let line = self.buffer[..pos].trim_end_matches('\r').to_string();
            self.buffer = self.buffer[pos + 1..].to_string();
            self.feed_line(&line);
        }
    }

    /// 处理一个 SSE 数据行
    /// 输入格式: "data: {json}" 或 "data: [DONE]"
    /// 也会处理 "event:" 行（忽略）和空行
    pub fn feed_line(&mut self, line: &str) {
        let trimmed = line.trim();

        // 空行分隔 SSE 事件，忽略
        if trimmed.is_empty() {
            return;
        }

        // 只处理 data: 开头的行
        if !trimmed.starts_with("data:") {
            // 忽略 event:, id:, retry: 等其他 SSE 字段
            return;
        }

        let data = trimmed[5..].trim();

        // 检查 [DONE] 终止信号
        if data == "[DONE]" {
            self.done = true;
            return;
        }

        // 尝试解析 JSON
        let json: serde_json::Value = match serde_json::from_str(data) {
            Ok(v) => v,
            Err(_) => return, // 忽略无法解析的行
        };

        self.chunk_count += 1;

        // 尝试提取 model（某些提供商在 chunk 中返回 model）
        if let Some(model) = json.get("model").and_then(|v| v.as_str()) {
            self.model = Some(model.to_string());
        }

        // 尝试 OpenAI delta 格式
        self.extract_openai_delta(&json);

        // 尝试 Anthropic SSE 格式
        self.extract_anthropic_delta(&json);

        // 提取 usage（通常在最后一个 chunk 中）
        self.extract_usage_from_chunk(&json);
    }

    /// 提取 OpenAI delta 格式的内容
    /// choices[0].delta.content → 累积 content
    /// choices[0].delta.reasoning_content → 累积 thinking (DeepSeek R1)
    fn extract_openai_delta(&mut self, json: &serde_json::Value) {
        let delta = json
            .get("choices")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|c| c.get("delta"));

        if let Some(delta) = delta {
            // 主内容
            if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                if !content.is_empty() {
                    self.content_parts.push(content.to_string());
                }
            }

            // DeepSeek R1 思考内容
            if let Some(reasoning) = delta.get("reasoning_content").and_then(|v| v.as_str()) {
                if !reasoning.is_empty() {
                    self.thinking_parts.push(reasoning.to_string());
                }
            }
        }
    }

    /// 提取 Anthropic SSE 格式的内容
    /// 事件类型 content_block_delta 的 delta.text → 累积 content
    /// 事件类型 content_block_start 的 content_block.thinking → 累积 thinking
    fn extract_anthropic_delta(&mut self, json: &serde_json::Value) {
        let event_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match event_type {
            "content_block_delta" => {
                if let Some(delta) = json.get("delta") {
                    let delta_type = delta.get("type").and_then(|v| v.as_str()).unwrap_or("");

                    match delta_type {
                        "text_delta" => {
                            if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                if !text.is_empty() {
                                    self.content_parts.push(text.to_string());
                                }
                            }
                        }
                        "thinking_delta" => {
                            if let Some(thinking) = delta.get("thinking").and_then(|v| v.as_str()) {
                                if !thinking.is_empty() {
                                    self.thinking_parts.push(thinking.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            "content_block_start" => {
                if let Some(content_block) = json.get("content_block") {
                    let block_type = content_block
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    if block_type == "thinking" {
                        if let Some(thinking) =
                            content_block.get("thinking").and_then(|v| v.as_str())
                        {
                            if !thinking.is_empty() {
                                self.thinking_parts.push(thinking.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// 从 chunk 中提取 usage
    /// OpenAI: choices[0].usage 或根级 usage
    /// Anthropic: message_delta 事件中的 usage
    fn extract_usage_from_chunk(&mut self, json: &serde_json::Value) {
        // OpenAI 格式：根级 usage 或 choices[0].usage
        if let Some(usage_obj) = json.get("usage").and_then(|v| v.as_object()) {
            self.usage = self.parse_usage_object(usage_obj);
            return;
        }

        // OpenAI 格式：choices[0].usage（某些提供商在最后一个 chunk 的 choice 中放 usage）
        if let Some(choice_usage) = json
            .get("choices")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|c| c.get("usage"))
            .and_then(|v| v.as_object())
        {
            self.usage = self.parse_usage_object(choice_usage);
            return;
        }

        // Anthropic 格式：message_delta 事件
        if json.get("type").and_then(|v| v.as_str()) == Some("message_delta") {
            if let Some(usage_obj) = json.get("usage").and_then(|v| v.as_object()) {
                self.usage = self.parse_usage_object(usage_obj);
            }
        }
    }

    /// 解析 usage 对象
    /// 支持多种字段命名格式
    fn parse_usage_object(
        &self,
        usage_obj: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<Usage> {
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
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // 如果有非零值则返回
        if prompt > 0 || completion > 0 || total > 0 {
            let total = if total == 0 {
                prompt + completion
            } else {
                total
            };
            Some(Usage {
                prompt_tokens: prompt,
                completion_tokens: completion,
                total_tokens: total,
            })
        } else {
            None
        }
    }

    /// 获取当前累积的结果
    pub fn get_result(&self) -> ParsedResponseBody {
        let content = if self.content_parts.is_empty() {
            None
        } else {
            Some(self.content_parts.join(""))
        };

        let thinking = if self.thinking_parts.is_empty() {
            None
        } else {
            Some(self.thinking_parts.join(""))
        };

        let mut extra = HashMap::new();
        if let Some(ref model) = self.model {
            extra.insert(
                "model".to_string(),
                serde_json::Value::String(model.clone()),
            );
        }

        ParsedResponseBody {
            content,
            thinking,
            choices: None,
            usage: self.usage.clone(),
            extra,
        }
    }

    /// 是否已完成（收到 [DONE] 信号）
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// 获取已处理的 chunk 数量
    pub fn chunk_count(&self) -> usize {
        self.chunk_count
    }
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sse_single_line_parse() {
        let mut parser = SseParser::new();

        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}");

        assert!(!parser.is_done());
        assert_eq!(parser.chunk_count(), 1);

        let result = parser.get_result();
        assert_eq!(result.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_sse_multiple_chunks_accumulate() {
        let mut parser = SseParser::new();

        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}");
        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\" world\"}}]}");
        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\"!\"}}]}");

        let result = parser.get_result();
        assert_eq!(result.content, Some("Hello world!".to_string()));
        assert_eq!(parser.chunk_count(), 3);
    }

    #[test]
    fn test_sse_done_signal() {
        let mut parser = SseParser::new();

        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}");
        assert!(!parser.is_done());

        parser.feed_line("data: [DONE]");
        assert!(parser.is_done());

        let result = parser.get_result();
        assert_eq!(result.content, Some("Hi".to_string()));
    }

    #[test]
    fn test_sse_deepseek_thinking() {
        let mut parser = SseParser::new();

        parser.feed_line(
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"Let me think\"}}]}",
        );
        parser.feed_line(
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\" step by step\"}}]}",
        );
        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\"The answer is 42.\"}}]}");

        let result = parser.get_result();
        assert_eq!(
            result.thinking,
            Some("Let me think step by step".to_string())
        );
        assert_eq!(result.content, Some("The answer is 42.".to_string()));
    }

    #[test]
    fn test_sse_usage_accumulation() {
        let mut parser = SseParser::new();

        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}");
        parser.feed_line("data: {\"choices\":[{\"delta\":{},\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":5,\"total_tokens\":15}}]}");

        let result = parser.get_result();
        assert!(result.usage.is_some());

        let usage = result.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 5);
        assert_eq!(usage.total_tokens, 15);
    }

    #[test]
    fn test_sse_root_level_usage() {
        let mut parser = SseParser::new();

        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\"test\"}}]}");
        parser.feed_line("data: {\"choices\":[{\"delta\":{}}],\"usage\":{\"prompt_tokens\":20,\"completion_tokens\":10,\"total_tokens\":30}}");

        let result = parser.get_result();
        let usage = result.usage.unwrap();
        assert_eq!(usage.total_tokens, 30);
    }

    #[test]
    fn test_sse_anthropic_format() {
        let mut parser = SseParser::new();

        // Anthropic content_block_delta 事件
        parser.feed_line("data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}");
        parser.feed_line("data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\" there\"}}");

        let result = parser.get_result();
        assert_eq!(result.content, Some("Hello there".to_string()));
    }

    #[test]
    fn test_sse_anthropic_thinking() {
        let mut parser = SseParser::new();

        // Anthropic thinking_delta
        parser.feed_line("data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"Hmm...\"}}");
        // Anthropic content_block_start with thinking
        parser.feed_line("data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"thinking\",\"thinking\":\"Initial thought\"}}");
        // Text delta
        parser.feed_line("data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Answer\"}}");

        let result = parser.get_result();
        assert_eq!(result.thinking, Some("Hmm...Initial thought".to_string()));
        assert_eq!(result.content, Some("Answer".to_string()));
    }

    #[test]
    fn test_sse_feed_chunk_multiline() {
        let mut parser = SseParser::new();

        let chunk = "data: {\"choices\":[{\"delta\":{\"content\":\"A\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"B\"}}]}\n\n";
        parser.feed_chunk(chunk);

        let result = parser.get_result();
        assert_eq!(result.content, Some("AB".to_string()));
    }

    #[test]
    fn test_sse_model_extraction() {
        let mut parser = SseParser::new();

        parser.feed_line(
            "data: {\"model\":\"gpt-4\",\"choices\":[{\"delta\":{\"content\":\"test\"}}]}",
        );

        let result = parser.get_result();
        assert_eq!(result.extra.get("model"), Some(&json!("gpt-4")));
    }

    #[test]
    fn test_sse_empty_content_delta() {
        let mut parser = SseParser::new();

        // 空 content 不应该被累积
        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\"\"}}]}");
        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\"actual\"}}]}");

        let result = parser.get_result();
        assert_eq!(result.content, Some("actual".to_string()));
    }

    #[test]
    fn test_sse_ignore_non_data_lines() {
        let mut parser = SseParser::new();

        parser.feed_line("event: message_start");
        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\"test\"}}]}");
        parser.feed_line("id: some-id");

        let result = parser.get_result();
        assert_eq!(result.content, Some("test".to_string()));
        assert_eq!(parser.chunk_count(), 1);
    }

    #[test]
    fn test_sse_malformed_json_ignored() {
        let mut parser = SseParser::new();

        parser.feed_line("data: not valid json");
        parser.feed_line("data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}");

        let result = parser.get_result();
        assert_eq!(result.content, Some("ok".to_string()));
        assert_eq!(parser.chunk_count(), 1);
    }

    #[test]
    fn test_sse_anthropic_usage() {
        let mut parser = SseParser::new();

        parser.feed_line("data: {\"type\":\"message_delta\",\"usage\":{\"output_tokens\":10}}");

        let result = parser.get_result();
        let usage = result.usage.unwrap();
        assert_eq!(usage.completion_tokens, 10);
    }
}
