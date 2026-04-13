// Monitor 模块 - 配置管理
// 支持 JSONC 解析（去除注释）、文件监听热更新、默认配置

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
/// 覆盖主流 LLM API 域名，确保所有已知 Provider 都能被捕获
fn default_config() -> MonitorConfig {
    MonitorConfig {
        domains: vec![
            // OpenAI
            DomainConfig {
                domain: "api.openai.com".into(),
                provider: "openai".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // Anthropic
            DomainConfig {
                domain: "api.anthropic.com".into(),
                provider: "anthropic".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // Kimi / Moonshot
            DomainConfig {
                domain: "api.kimi.com".into(),
                provider: "kimi".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            DomainConfig {
                domain: "api.moonshot.cn".into(),
                provider: "kimi".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // 百度千帆
            DomainConfig {
                domain: "qianfan.baidubce.com".into(),
                provider: "qianfan".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // 字节豆包 (Volces/Doubao)
            DomainConfig {
                domain: "ark.cn-beijing.volces.com".into(),
                provider: "volces".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // 无问芯穹 (Infini-AI)
            DomainConfig {
                domain: "cloud.infini-ai.com".into(),
                provider: "infini".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // MiniMax
            DomainConfig {
                domain: "api.minimaxi.com".into(),
                provider: "minimax".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            DomainConfig {
                domain: "api.minimax.chat".into(),
                provider: "minimax".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // 京东云
            DomainConfig {
                domain: "modelservice.jdcloud.com".into(),
                provider: "jdcloud".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // 智谱 AI (Zhipu/GLM)
            DomainConfig {
                domain: "open.bigmodel.cn".into(),
                provider: "zhipuai".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // Google Gemini
            DomainConfig {
                domain: "generativelanguage.googleapis.com".into(),
                provider: "google".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // DeepSeek
            DomainConfig {
                domain: "api.deepseek.com".into(),
                provider: "deepseek".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // Groq
            DomainConfig {
                domain: "api.groq.com".into(),
                provider: "groq".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // Mistral
            DomainConfig {
                domain: "api.mistral.ai".into(),
                provider: "mistral".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // 通义千问 (Qwen/Alibaba)
            DomainConfig {
                domain: "dashscope.aliyuncs.com".into(),
                provider: "qwen".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // 硅基流动 (SiliconFlow)
            DomainConfig {
                domain: "api.siliconflow.cn".into(),
                provider: "siliconflow".into(),
                enabled: true,
                match_type: MatchType::Exact,
            },
            // 零一万物 (Yi)
            DomainConfig {
                domain: "api.lingyiwanwu.com".into(),
                provider: "siliconflow".into(),
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
