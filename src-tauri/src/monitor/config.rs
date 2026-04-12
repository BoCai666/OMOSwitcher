// Monitor 模块 - 配置管理
// 支持 JSONC 解析（去除注释）、文件监听热更新、默认配置
// 注意：文件监听功能尚未被主流程集成调用，保留供后续集成使用

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use notify::RecommendedWatcher;
use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, RecommendedCache};

use crate::monitor::types::{
    DomainConfig, MatchType, ModelPricingConfig, MonitorConfig, PortConfig, PricingConfig,
};

/// 配置变更回调类型
pub type ConfigChangeCallback = Box<dyn Fn(&MonitorConfig) + Send + 'static>;

/// 配置管理器
/// 支持 JSONC 解析、文件监听热更新、默认配置
pub struct ConfigManager {
    config: Arc<Mutex<MonitorConfig>>,
    config_path: PathBuf,
    callbacks: Arc<Mutex<Vec<ConfigChangeCallback>>>,
    /// debouncer handle for stopping file watch
    _debouncer: Option<notify_debouncer_full::Debouncer<RecommendedWatcher, RecommendedCache>>,
}

/// 去除 JSONC 注释
/// 处理 // 单行注释和 /* */ 多行注释
/// 注意：不会去除字符串字面量内的注释符号
fn strip_jsonc_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        // 处理转义字符
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_string {
            result.push(ch);
            escape_next = true;
            continue;
        }

        // 处理字符串边界
        if ch == '"' && !in_string {
            in_string = true;
            result.push(ch);
            continue;
        }
        if ch == '"' && in_string {
            in_string = false;
            result.push(ch);
            continue;
        }

        // 如果在字符串内，直接添加字符
        if in_string {
            result.push(ch);
            continue;
        }

        // 处理单行注释 //
        if ch == '/' {
            match chars.peek() {
                Some('/') => {
                    // 跳过单行注释直到行尾
                    chars.next(); // 消费第二个 /
                    while let Some(next_ch) = chars.next() {
                        if next_ch == '\n' {
                            result.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    // 跳过多行注释
                    chars.next(); // 消费 *
                    while let Some(next_ch) = chars.next() {
                        if next_ch == '*' {
                            if let Some('/') = chars.peek() {
                                chars.next(); // 消费 /
                                break;
                            }
                        }
                    }
                    continue;
                }
                _ => {
                    result.push(ch);
                    continue;
                }
            }
        }

        result.push(ch);
    }

    result
}

/// 获取默认配置
/// 与 config.jsonc 保持一致：8 个域名、14 个模型定价、端口 7100/7101
fn default_config() -> MonitorConfig {
    MonitorConfig {
        domains: vec![
            DomainConfig {
                domain: "api.openai.com".into(),
                provider: "OpenAI".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            DomainConfig {
                domain: "qianfan.baidubce.com".into(),
                provider: "qianfan".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            DomainConfig {
                domain: "api.anthropic.com".into(),
                provider: "Anthropic".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            DomainConfig {
                domain: "api.kimi.com".into(),
                provider: "kimi".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            DomainConfig {
                domain: "ark.cn-beijing.volces.com".into(),
                provider: "volces".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            DomainConfig {
                domain: "cloud.infini-ai.com".into(),
                provider: "infini".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            DomainConfig {
                domain: "api.minimaxi.com".into(),
                provider: "minimax".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            DomainConfig {
                domain: "modelservice.jdcloud.com".into(),
                provider: "jdcloud".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
        ],
        pricing: PricingConfig {
            match_strategy: "prefix".into(),
            models: vec![
                ModelPricingConfig {
                    model: "gpt-4".into(),
                    input: 30.0,
                    output: 60.0,
                },
                ModelPricingConfig {
                    model: "gpt-4-turbo".into(),
                    input: 10.0,
                    output: 30.0,
                },
                ModelPricingConfig {
                    model: "gpt-3.5-turbo".into(),
                    input: 0.5,
                    output: 1.5,
                },
                ModelPricingConfig {
                    model: "gpt-4o".into(),
                    input: 5.0,
                    output: 15.0,
                },
                ModelPricingConfig {
                    model: "gpt-4o-mini".into(),
                    input: 0.15,
                    output: 0.6,
                },
                ModelPricingConfig {
                    model: "claude-3-opus".into(),
                    input: 15.0,
                    output: 75.0,
                },
                ModelPricingConfig {
                    model: "claude-3-sonnet".into(),
                    input: 3.0,
                    output: 15.0,
                },
                ModelPricingConfig {
                    model: "doubao".into(),
                    input: 0.1,
                    output: 0.2,
                },
                ModelPricingConfig {
                    model: "kimi".into(),
                    input: 1.0,
                    output: 3.0,
                },
                ModelPricingConfig {
                    model: "kimi-k2.5".into(),
                    input: 0.6,
                    output: 3.0,
                },
                ModelPricingConfig {
                    model: "minimax-m2.5".into(),
                    input: 0.3,
                    output: 1.2,
                },
                ModelPricingConfig {
                    model: "glm-4.7".into(),
                    input: 0.6,
                    output: 2.2,
                },
                ModelPricingConfig {
                    model: "glm-5".into(),
                    input: 0.5,
                    output: 2.25,
                },
                ModelPricingConfig {
                    model: "doubao-seed-2.0-code".into(),
                    input: 0.2,
                    output: 1.0,
                },
            ],
        },
        ports: PortConfig {
            web: 7100,
            proxy: 7101,
        },
    }
}

/// 从 JSONC 字符串解析配置
fn parse_jsonc(jsonc: &str) -> Result<MonitorConfig, String> {
    let json_str = strip_jsonc_comments(jsonc);
    serde_json::from_str(&json_str).map_err(|e| format!("JSON 解析失败: {}", e))
}

impl ConfigManager {
    /// 创建新的配置管理器
    /// 从指定路径加载配置，如果文件不存在则使用默认配置
    pub fn new(config_path: &Path) -> Result<Self, String> {
        let config = if config_path.exists() {
            let content = std::fs::read_to_string(config_path)
                .map_err(|e| format!("读取配置文件失败: {}", e))?;
            parse_jsonc(&content)?
        } else {
            default_config()
        };

        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            config_path: config_path.to_path_buf(),
            callbacks: Arc::new(Mutex::new(Vec::new())),
            _debouncer: None,
        })
    }

    /// 获取当前配置的快照
    pub fn get_config(&self) -> MonitorConfig {
        self.config.lock().unwrap().clone()
    }

    /// 注册配置变更回调
    pub fn on_change(&self, callback: ConfigChangeCallback) {
        self.callbacks.lock().unwrap().push(callback);
    }

    /// 启动文件监听（热更新）
    /// 配置文件变更时自动重载并触发回调
    pub fn start_watching(&mut self) -> Result<(), String> {
        let config_path = self.config_path.clone();
        let config = Arc::clone(&self.config);
        let callbacks = Arc::clone(&self.callbacks);
        let watch_path = config_path.clone();

        // 创建防抖文件监听器（2 秒防抖）
        let mut debouncer = new_debouncer(
            std::time::Duration::from_secs(2),
            None,
            move |res: Result<Vec<notify_debouncer_full::DebouncedEvent>, Vec<notify::Error>>| {
                if let Ok(events) = res {
                    for event in events {
                        // 检查是否为文件修改事件
                        let is_modify = matches!(
                            event.kind,
                            notify::event::EventKind::Modify(_)
                                | notify::event::EventKind::Create(_)
                                | notify::event::EventKind::Remove(_)
                        );

                        if is_modify {
                            // 重新加载配置
                            if let Ok(new_config) = load_config_from_path(&config_path) {
                                *config.lock().unwrap() = new_config.clone();
                                // 触发所有回调
                                for callback in callbacks.lock().unwrap().iter() {
                                    callback(&new_config);
                                }
                            }
                        }
                    }
                }
            },
        )
        .map_err(|e| format!("创建文件监听器失败: {}", e))?;

        // 开始监听配置文件
        debouncer
            .watch(&watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| format!("监听配置文件失败: {}", e))?;

        self._debouncer = Some(debouncer);
        Ok(())
    }

    /// 停止文件监听
    pub fn stop_watching(&mut self) {
        self._debouncer = None;
    }

    /// 重新加载配置
    pub fn reload(&self) -> Result<(), String> {
        let new_config = load_config_from_path(&self.config_path)?;
        *self.config.lock().unwrap() = new_config.clone();

        // 触发所有回调
        for callback in self.callbacks.lock().unwrap().iter() {
            callback(&new_config);
        }

        Ok(())
    }
}

/// 从指定路径加载配置
fn load_config_from_path(path: &Path) -> Result<MonitorConfig, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("读取配置文件失败: {}", e))?;
    parse_jsonc(&content)
}

/// 获取默认配置路径
/// ~/.config/omoswitcher/monitor/config.jsonc
pub fn default_config_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home
        .join(".config")
        .join("omoswitcher")
        .join("monitor")
        .join("config.jsonc"))
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = default_config();

        // 验证 8 个域名
        assert_eq!(config.domains.len(), 8);
        assert!(config.domains.iter().any(|d| d.domain == "api.openai.com"));
        assert!(config
            .domains
            .iter()
            .any(|d| d.domain == "api.anthropic.com"));
        assert!(config.domains.iter().any(|d| d.domain == "api.kimi.com"));

        // 验证 14 个模型定价
        assert_eq!(config.pricing.models.len(), 14);
        assert!(config.pricing.models.iter().any(|m| m.model == "gpt-4o"));
        assert!(config
            .pricing
            .models
            .iter()
            .any(|m| m.model == "claude-3-opus"));

        // 验证端口
        assert_eq!(config.ports.web, 7100);
        assert_eq!(config.ports.proxy, 7101);
    }

    #[test]
    fn test_jsonc_strip_comments() {
        let jsonc = r#"
        // 这是单行注释
        {
            // 域名配置
            "name": "test", /* 多行
            注释 */
            "value": 123 // 尾部注释
        }
        "#;

        let result = strip_jsonc_comments(jsonc);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["name"], "test");
        assert_eq!(parsed["value"], 123);

        // 验证注释被正确去除，不影响解析
        assert!(!result.contains("// 这是单行注释"));
        assert!(!result.contains("/* 多行"));
        assert!(!result.contains("*/"));
    }

    #[test]
    fn test_jsonc_string_preservation() {
        // 验证字符串内的 // 不会被误认为注释
        let jsonc = r#"
        {
            "url": "https://api.openai.com/v1/chat/completions",
            "path": "C:\\Users\\test\\file.txt"
        }
        "#;

        let result = strip_jsonc_comments(jsonc);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["url"], "https://api.openai.com/v1/chat/completions");
        assert_eq!(parsed["path"], "C:\\Users\\test\\file.txt");
    }

    #[test]
    fn test_parse_config_from_jsonc() {
        let jsonc = r#"
        {
            "domains": [
                {
                    "domain": "api.test.com",
                    "provider": "TestProvider",
                    "enabled": true
                }
            ],
            "pricing": {
                "matchStrategy": "exact",
                "models": [
                    { "model": "test-model", "input": 1.0, "output": 2.0 }
                ]
            },
            "ports": { "web": 8000, "proxy": 8001 }
        }
        "#;

        let config = parse_jsonc(jsonc).unwrap();

        assert_eq!(config.domains.len(), 1);
        assert_eq!(config.domains[0].domain, "api.test.com");
        assert_eq!(config.domains[0].provider, "TestProvider");
        assert!(config.domains[0].enabled);

        assert_eq!(config.pricing.match_strategy, "exact");
        assert_eq!(config.pricing.models.len(), 1);
        assert_eq!(config.pricing.models[0].model, "test-model");

        assert_eq!(config.ports.web, 8000);
        assert_eq!(config.ports.proxy, 8001);
    }

    #[test]
    fn test_config_roundtrip() {
        let config = default_config();

        // 序列化
        let json = serde_json::to_string(&config).unwrap();

        // 反序列化
        let deserialized: MonitorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.domains.len(), deserialized.domains.len());
        assert_eq!(
            config.pricing.models.len(),
            deserialized.pricing.models.len()
        );
        assert_eq!(config.ports.web, deserialized.ports.web);
        assert_eq!(config.ports.proxy, deserialized.ports.proxy);
    }

    #[test]
    fn test_config_missing_fields_use_defaults() {
        // 只有部分字段的配置 - ports 字段仍然必须提供
        let jsonc = r#"
        {
            "domains": [
                { "domain": "api.sparse.com", "provider": "Sparse", "enabled": false }
            ],
            "pricing": {
                "matchStrategy": "prefix",
                "models": [
                    { "model": "sparse-model", "input": 0.5, "output": 1.0 }
                ]
            },
            "ports": { "web": 7100, "proxy": 7101 }
        }
        "#;

        let config = parse_jsonc(jsonc).unwrap();

        // domains 被正确解析
        assert_eq!(config.domains.len(), 1);
        assert_eq!(config.domains[0].domain, "api.sparse.com");
        assert_eq!(config.domains[0].enabled, false);

        // pricing.matchStrategy 被正确解析
        assert_eq!(config.pricing.match_strategy, "prefix");
    }

    #[test]
    fn test_multiline_comment_removal() {
        let jsonc = r#"
        {
            /* 这是一个
               多行
               注释 */
            "key": "value"
        }
        "#;

        let result = strip_jsonc_comments(jsonc);
        assert!(!result.contains("这是一个"));
        assert!(!result.contains("多行"));
        assert!(!result.contains("注释"));

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_config_manager_get_config() {
        // 使用临时文件测试
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_config.jsonc");

        // 写入配置
        let jsonc = r#"
        {
            "domains": [
                { "domain": "api.manager.com", "provider": "Manager", "enabled": true }
            ],
            "pricing": {
                "matchStrategy": "prefix",
                "models": [
                    { "model": "manager-model", "input": 0.1, "output": 0.2 }
                ]
            },
            "ports": { "web": 9000, "proxy": 9001 }
        }
        "#;
        std::fs::write(&config_path, jsonc).unwrap();

        let manager = ConfigManager::new(&config_path).unwrap();
        let config = manager.get_config();

        assert_eq!(config.domains.len(), 1);
        assert_eq!(config.domains[0].domain, "api.manager.com");
        assert_eq!(config.ports.web, 9000);

        // 清理
        std::fs::remove_file(&config_path).ok();
    }

    #[test]
    fn test_config_manager_default_when_file_not_exists() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("nonexistent_config.jsonc");

        // 确保文件不存在
        if config_path.exists() {
            std::fs::remove_file(&config_path).ok();
        }

        let manager = ConfigManager::new(&config_path).unwrap();
        let config = manager.get_config();

        // 应该返回默认配置
        assert_eq!(config.domains.len(), 8);
        assert_eq!(config.ports.web, 7100);
    }

    #[test]
    fn test_config_manager_on_change_callback() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_callback.jsonc");

        let jsonc = r#"
        {
            "domains": [
                { "domain": "api.callback.com", "provider": "Callback", "enabled": true }
            ],
            "pricing": {
                "matchStrategy": "prefix",
                "models": []
            },
            "ports": { "web": 9000, "proxy": 9001 }
        }
        "#;
        std::fs::write(&config_path, jsonc).unwrap();

        let manager = ConfigManager::new(&config_path).unwrap();

        // 注册回调
        let callback_called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let callback_called_clone = std::sync::Arc::clone(&callback_called);

        manager.on_change(Box::new(move |_config| {
            *callback_called_clone.lock().unwrap() = true;
        }));

        // 触发 reload
        manager.reload().unwrap();

        assert!(*callback_called.lock().unwrap());

        // 清理
        std::fs::remove_file(&config_path).ok();
    }
}
