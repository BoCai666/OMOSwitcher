// 成本计算器
// 移植自 packages/monitor/src/parsers/cost-calculator.ts
// 定价单位：美元/1M tokens
// 注意：尚未被 handler 集成调用，保留供后续集成使用

#![allow(dead_code)]

use crate::monitor::types::{ModelPricingConfig, PricingConfig};

/// 计算成本
/// input_cost = (prompt_tokens / 1_000_000) * input_price
/// output_cost = (completion_tokens / 1_000_000) * output_price
/// total = input_cost + output_cost
///
/// 返回值单位为美元
pub fn calculate_cost(
    model: &str,
    prompt_tokens: i64,
    completion_tokens: i64,
    pricing: &PricingConfig,
) -> f64 {
    let pricing_entry = find_pricing(model, pricing);

    let (input_price, output_price) = match pricing_entry {
        Some(p) => (p.input, p.output),
        None => {
            // 未找到定价时使用默认定价（1 美元/1M tokens 输入，2 美元/1M tokens 输出）
            (1.0, 2.0)
        }
    };

    let input_cost = (prompt_tokens as f64 / 1_000_000.0) * input_price;
    let output_cost = (completion_tokens as f64 / 1_000_000.0) * output_price;
    input_cost + output_cost
}

/// 查找模型定价
/// 1. 先尝试精确匹配
/// 2. 如果 match_strategy == "prefix"，再尝试前缀匹配
pub fn find_pricing<'a>(model: &str, pricing: &'a PricingConfig) -> Option<&'a ModelPricingConfig> {
    // 首先尝试精确匹配
    for model_config in &pricing.models {
        if model_config.model == model {
            return Some(model_config);
        }
    }

    // 如果匹配策略是 prefix，尝试前缀匹配
    if pricing.match_strategy == "prefix" {
        for model_config in &pricing.models {
            if model.starts_with(&model_config.model) {
                return Some(model_config);
            }
        }
    }

    None
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pricing_config(match_strategy: &str, models: Vec<(&str, f64, f64)>) -> PricingConfig {
        PricingConfig {
            match_strategy: match_strategy.to_string(),
            models: models
                .into_iter()
                .map(|(m, i, o)| ModelPricingConfig {
                    model: m.to_string(),
                    input: i,
                    output: o,
                })
                .collect(),
        }
    }

    #[test]
    fn test_exact_match() {
        let config = make_pricing_config(
            "exact",
            vec![("gpt-4", 30.0, 60.0), ("gpt-4-turbo", 10.0, 30.0)],
        );

        let result = find_pricing("gpt-4", &config);
        assert!(result.is_some());
        let p = result.unwrap();
        assert_eq!(p.input, 30.0);
        assert_eq!(p.output, 60.0);

        // 精确匹配时前缀不应该匹配
        let result = find_pricing("gpt-4-0613", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_prefix_match() {
        let config = make_pricing_config(
            "prefix",
            vec![("gpt-4", 30.0, 60.0), ("gpt-4-turbo", 10.0, 30.0)],
        );

        // 精确匹配应该优先
        let result = find_pricing("gpt-4-turbo", &config);
        assert!(result.is_some());
        assert_eq!(result.unwrap().input, 10.0);

        // 前缀匹配：按列表顺序首次匹配
        let result = find_pricing("gpt-4-0613", &config);
        assert!(result.is_some());
        assert_eq!(result.unwrap().input, 30.0);

        // 注意：首次匹配语义下，gpt-4-turbo-preview 会匹配 gpt-4（先出现）
        // 如果需要匹配更具体的前缀，应将更长的前缀放在列表前面
        let result = find_pricing("gpt-4-turbo-preview", &config);
        assert!(result.is_some());
        assert_eq!(result.unwrap().input, 30.0);
    }

    #[test]
    fn test_no_match_returns_none() {
        let config = make_pricing_config("prefix", vec![("gpt-4", 30.0, 60.0)]);

        let result = find_pricing("claude-3-opus", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_calculate_cost_basic() {
        let config = make_pricing_config("prefix", vec![("gpt-4", 30.0, 60.0)]);

        // gpt-4: 输入 30$/1M tokens, 输出 60$/1M tokens
        // 1000 输入 tokens + 500 输出 tokens
        let cost = calculate_cost("gpt-4", 1000, 500, &config);
        let expected = (1000.0 / 1_000_000.0) * 30.0 + (500.0 / 1_000_000.0) * 60.0;
        assert!((cost - expected).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_cost_no_match_uses_default() {
        let config = make_pricing_config("exact", vec![("gpt-4", 30.0, 60.0)]);

        // 未匹配的模型使用默认定价 (1.0, 2.0)
        let cost = calculate_cost("unknown-model", 1000, 500, &config);
        let expected = (1000.0 / 1_000_000.0) * 1.0 + (500.0 / 1_000_000.0) * 2.0;
        assert!((cost - expected).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_cost_zero_tokens() {
        let config = make_pricing_config("prefix", vec![("gpt-4", 30.0, 60.0)]);

        let cost = calculate_cost("gpt-4", 0, 0, &config);
        assert!((cost - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_cost_large_tokens() {
        let config = make_pricing_config("prefix", vec![("gpt-4", 30.0, 60.0)]);

        // 1M 输入 + 1M 输出
        let cost = calculate_cost("gpt-4", 1_000_000, 1_000_000, &config);
        let expected = 30.0 + 60.0; // 30$ + 60$ = 90$
        assert!((cost - expected).abs() < 1e-10);
    }

    #[test]
    fn test_prefix_match_longest_prefix_priority() {
        // 注意：当前实现是按顺序匹配，先匹配到的就返回
        // 确保 gpt-4-turbo 不会被 gpt-4 抢先匹配（精确匹配优先）
        let config = make_pricing_config(
            "prefix",
            vec![("gpt-4-turbo", 10.0, 30.0), ("gpt-4", 30.0, 60.0)],
        );

        // gpt-4-turbo 应该精确匹配自身
        let result = find_pricing("gpt-4-turbo", &config);
        assert!(result.is_some());
        assert_eq!(result.unwrap().input, 10.0);

        // gpt-4-turbo-preview 应该匹配 gpt-4-turbo（列表中先出现）
        let result = find_pricing("gpt-4-turbo-preview", &config);
        assert!(result.is_some());
        assert_eq!(result.unwrap().input, 10.0);

        // gpt-4-0613 匹配 gpt-4
        let result = find_pricing("gpt-4-0613", &config);
        assert!(result.is_some());
        assert_eq!(result.unwrap().input, 30.0);
    }
}
