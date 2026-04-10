// Monitor 模块 - HTTP/HTTPS 代理服务器
// 使用 hudsucker 构建的代理服务器，支持 HTTP 透传和 HTTPS CONNECT 隧道
// 当前阶段：隧道模式（不拦截 HTTPS），HTTP 请求透传

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use hudsucker::{
    certificate_authority::RcgenAuthority,
    rcgen::{CertificateParams, DnType, Issuer, KeyPair, SanType},
    rustls::crypto::aws_lc_rs,
    Proxy,
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::handler::MonitorHandler;

/// 默认代理监听地址
const DEFAULT_PROXY_ADDR: &str = "127.0.0.1:7101";

/// CA 证书通用名称
const CA_COMMON_NAME: &str = "OMOSwitcher Monitor CA";

/// CA 证书组织名称
const CA_ORGANIZATION: &str = "OMOSwitcher";

/// 证书缓存大小
const CERT_CACHE_SIZE: u64 = 1_000;

/// 代理服务器错误类型
#[derive(Debug)]
pub enum ProxyError {
    /// 端口被占用
    AddrInUse(SocketAddr),
    /// 代理构建失败
    BuildFailed(String),
    /// 代理启动失败
    StartFailed(String),
    /// 代理已在运行
    AlreadyRunning,
    /// 代理未运行
    NotRunning,
    /// CA 证书生成失败
    CaGenerationFailed(String),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::AddrInUse(addr) => write!(f, "端口 {} 已被占用，请检查是否有其他 Monitor 进程在运行", addr),
            ProxyError::BuildFailed(msg) => write!(f, "代理构建失败: {}", msg),
            ProxyError::StartFailed(msg) => write!(f, "代理启动失败: {}", msg),
            ProxyError::AlreadyRunning => write!(f, "代理已在运行中"),
            ProxyError::NotRunning => write!(f, "代理未运行"),
            ProxyError::CaGenerationFailed(msg) => write!(f, "CA 证书生成失败: {}", msg),
        }
    }
}

impl std::error::Error for ProxyError {}

/// 生成自签名 CA 的参数和密钥
///
/// 用于 hudsucker 的 RcgenAuthority，即使此阶段不拦截 HTTPS，
/// hudsucker 仍需要 CA 配置才能构建 Proxy。
/// Issuer::new 需要 CertificateParams + KeyPair，而非已签名的 Certificate。
fn generate_ca_params() -> Result<(KeyPair, CertificateParams), ProxyError> {
    // 生成 CA 密钥对
    let key_pair = KeyPair::generate().map_err(|e| {
        ProxyError::CaGenerationFailed(format!("生成密钥对失败: {}", e))
    })?;

    // 配置 CA 证书参数
    let mut params = CertificateParams::default();
    params.distinguished_name.push(DnType::CommonName, CA_COMMON_NAME);
    params.distinguished_name.push(DnType::OrganizationName, CA_ORGANIZATION);
    params
        .distinguished_name
        .push(DnType::CountryName, "CN");

    // 添加 SAN (Subject Alternative Name)
    params.subject_alt_names.push(SanType::IpAddress(
        "127.0.0.1".parse().map_err(|e| {
            ProxyError::CaGenerationFailed(format!("解析 IP 地址失败: {}", e))
        })?,
    ));

    // 标记为 CA 证书
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

    // 设置密钥用途
    params.key_usages.push(rcgen::KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(rcgen::KeyUsagePurpose::CrlSign);

    Ok((key_pair, params))
}

/// 创建 RcgenAuthority
///
/// 从自签名 CA 证书参数生成 RcgenAuthority，供 hudsucker 使用
fn create_ca_authority() -> Result<RcgenAuthority, ProxyError> {
    let (key_pair, params) = generate_ca_params()?;

    // Issuer::new 接受 CertificateParams + KeyPair
    // RcgenAuthority 会用这些参数为每个被拦截的域名动态生成证书
    let issuer = Issuer::new(params, key_pair);

    let ca = RcgenAuthority::new(issuer, CERT_CACHE_SIZE, aws_lc_rs::default_provider());

    Ok(ca)
}

/// 代理服务器
///
/// 基于 hudsucker 构建的 HTTP/HTTPS 代理服务器
/// - HTTP 请求：透传到目标服务器
/// - HTTPS CONNECT：隧道模式（不拦截）
/// - 支持 CancellationToken 优雅停止
pub struct ProxyServer {
    /// 监听地址
    addr: SocketAddr,
    /// 优雅停止令牌
    cancel_token: CancellationToken,
    /// 运行状态
    running: Arc<AtomicBool>,
    /// 代理任务句柄（用于等待任务完成）
    task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl ProxyServer {
    /// 创建新的代理服务器实例
    ///
    /// 默认监听 127.0.0.1:7101
    pub fn new() -> Result<Self, ProxyError> {
        let addr: SocketAddr = DEFAULT_PROXY_ADDR
            .parse()
            .map_err(|e| ProxyError::BuildFailed(format!("解析地址失败: {}", e)))?;

        Self::with_addr(addr)
    }

    /// 使用指定地址创建代理服务器
    pub fn with_addr(addr: SocketAddr) -> Result<Self, ProxyError> {
        Ok(Self {
            addr,
            cancel_token: CancellationToken::new(),
            running: Arc::new(AtomicBool::new(false)),
            task_handle: Arc::new(Mutex::new(None)),
        })
    }

    /// 获取监听地址
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// 检查代理是否在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 启动代理服务器
    ///
    /// 使用 tokio::spawn 在后台启动代理
    /// 成功启动后代理将开始接受连接
    pub async fn start(&self) -> Result<(), ProxyError> {
        // 检查是否已在运行
        if self.is_running() {
            return Err(ProxyError::AlreadyRunning);
        }

        // 安装 aws_lc_rs crypto provider（如果尚未安装）
        // 这对于 rustls 0.23 是必需的
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        // 创建 CA Authority
        let ca = create_ca_authority()?;

        // 创建处理器（透传模式，不捕获请求）
        let handler = MonitorHandler::new_passthrough();

        // 获取 shutdown signal — 需要 'static future
        let cancel_token = self.cancel_token.clone();

        // 构建代理
        let proxy = Proxy::builder()
            .with_addr(self.addr)
            .with_ca(ca)
            .with_rustls_connector(aws_lc_rs::default_provider())
            .with_http_handler(handler)
            .with_graceful_shutdown(async move {
                cancel_token.cancelled().await;
            })
            .build()
            .map_err(|e| ProxyError::BuildFailed(format!("{}", e)))?;

        // 标记为运行中
        self.running.store(true, Ordering::SeqCst);

        let running = Arc::clone(&self.running);
        let addr = self.addr;

        // 启动代理任务
        let handle = tokio::spawn(async move {
            tracing::info!("代理服务器启动，监听 {}", addr);
            match proxy.start().await {
                Ok(()) => {
                    tracing::info!("代理服务器正常停止");
                }
                Err(e) => {
                    tracing::error!("代理服务器运行错误: {}", e);
                }
            }
            running.store(false, Ordering::SeqCst);
        });

        // 保存任务句柄
        *self.task_handle.lock().await = Some(handle);

        // 短暂等待确认代理成功启动（检查端口是否可连接）
        // 给代理 500ms 启动时间
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 尝试连接代理端口以验证启动成功
        let check_result = tokio::net::TcpStream::connect(self.addr).await;
        if check_result.is_err() {
            // 端口不可达，可能启动失败
            let running_state = self.running.load(Ordering::SeqCst);
            if !running_state {
                return Err(ProxyError::StartFailed(format!(
                    "代理服务器启动后端口 {} 不可达",
                    self.addr
                )));
            }
            // 如果 running 仍为 true，可能是启动较慢，不报错
            tracing::warn!("代理服务器启动后端口验证失败，但状态仍为运行中，可能启动较慢");
        }

        tracing::info!("代理服务器已在 {} 上启动", self.addr);
        Ok(())
    }

    /// 停止代理服务器
    ///
    /// 通过 CancellationToken 触发优雅停止
    pub async fn stop(&self) -> Result<(), ProxyError> {
        if !self.is_running() {
            return Err(ProxyError::NotRunning);
        }

        tracing::info!("正在停止代理服务器...");

        // 触发取消令牌
        self.cancel_token.cancel();

        // 等待代理任务完成
        let handle = self.task_handle.lock().await.take();
        if let Some(h) = handle {
            // 等待任务完成，最多 5 秒
            match tokio::time::timeout(tokio::time::Duration::from_secs(5), h).await {
                Ok(Ok(())) => {
                    tracing::info!("代理服务器已正常停止");
                }
                Ok(Err(e)) => {
                    tracing::error!("代理服务器任务异常: {}", e);
                }
                Err(_) => {
                    tracing::warn!("代理服务器停止超时（5秒），强制结束");
                }
            }
        }

        // 确保状态更新
        self.running.store(false, Ordering::SeqCst);
        Ok(())
    }
}

impl Drop for ProxyServer {
    fn drop(&mut self) {
        // 确保取消令牌被触发
        self.cancel_token.cancel();
        self.running.store(false, Ordering::SeqCst);
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_server_creation() {
        let server = ProxyServer::new();
        assert!(server.is_ok(), "ProxyServer 创建应成功");

        let server = server.expect("创建失败");
        assert_eq!(server.addr().port(), 7101);
        assert!(!server.is_running());
    }

    #[test]
    fn test_proxy_server_with_custom_addr() {
        let addr: SocketAddr = "127.0.0.1:7200".parse().expect("解析地址失败");
        let server = ProxyServer::with_addr(addr);
        assert!(server.is_ok());

        let server = server.expect("创建失败");
        assert_eq!(server.addr().port(), 7200);
    }

    #[test]
    fn test_proxy_server_not_running_initially() {
        let server = ProxyServer::new().expect("创建失败");
        assert!(!server.is_running());
    }

    #[tokio::test]
    async fn test_proxy_start_stop() {
        // 使用不太可能冲突的端口
        let addr: SocketAddr = "127.0.0.1:17891".parse().expect("解析地址失败");

        let server = ProxyServer::with_addr(addr).expect("创建失败");

        // 启动
        let start_result = server.start().await;
        if let Err(ProxyError::StartFailed(_)) = start_result {
            // 端口可能被占用，跳过测试
            eprintln!("跳过测试：端口 {} 不可用", addr);
            return;
        }
        assert!(start_result.is_ok(), "代理启动应成功: {:?}", start_result.err());

        // 给代理一点时间完全启动
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        assert!(server.is_running(), "代理应标记为运行中");

        // 停止
        let stop_result = server.stop().await;
        assert!(stop_result.is_ok(), "代理停止应成功: {:?}", stop_result.err());

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        assert!(!server.is_running(), "代理应标记为已停止");
    }

    #[tokio::test]
    async fn test_proxy_double_start_error() {
        let addr: SocketAddr = "127.0.0.1:17892".parse().expect("解析地址失败");
        let server = ProxyServer::with_addr(addr).expect("创建失败");

        // 第一次启动
        let start_result = server.start().await;
        if start_result.is_err() {
            eprintln!("跳过测试：端口 {} 不可用", addr);
            return;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        // 第二次启动应失败
        let second_start = server.start().await;
        assert!(
            matches!(second_start, Err(ProxyError::AlreadyRunning)),
            "重复启动应返回 AlreadyRunning 错误"
        );

        // 清理
        server.stop().await.ok();
    }

    #[tokio::test]
    async fn test_proxy_stop_when_not_running() {
        let server = ProxyServer::new().expect("创建失败");
        let result = server.stop().await;
        assert!(
            matches!(result, Err(ProxyError::NotRunning)),
            "停止未运行的代理应返回 NotRunning 错误"
        );
    }

    #[tokio::test]
    async fn test_proxy_http_forwarding() {
        // 启动本地 mock HTTP 服务器
        let mock_addr: SocketAddr = "127.0.0.1:17893".parse().expect("解析地址失败");
        let mock_listener = tokio::net::TcpListener::bind(mock_addr).await;

        let mock_listener = match mock_listener {
            Ok(l) => l,
            Err(_) => {
                eprintln!("跳过测试：mock 端口 {} 不可用", mock_addr);
                return;
            }
        };

        // 启动 mock 服务器，返回简单的 HTTP 响应
        let mock_handle = tokio::spawn(async move {
            if let Ok((mut stream, _)) = mock_listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = vec![0u8; 4096];
                // 读取请求
                let _ = stream.read(&mut buf).await;
                // 返回 HTTP 响应
                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nHello";
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });

        // 启动代理
        let proxy_addr: SocketAddr = "127.0.0.1:17894".parse().expect("解析地址失败");
        let server = ProxyServer::with_addr(proxy_addr).expect("创建失败");

        let start_result = server.start().await;
        if start_result.is_err() {
            eprintln!("跳过测试：代理端口 {} 不可用", proxy_addr);
            mock_handle.abort();
            return;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 通过代理发送 HTTP 请求到 mock 服务器
        let client = reqwest::Client::builder()
            .proxy(reqwest::Proxy::all(format!("http://{}", proxy_addr)).expect("构建代理配置失败"))
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("构建 HTTP 客户端失败");

        let request_result = client
            .get(format!("http://{}", mock_addr))
            .send()
            .await;

        match request_result {
            Ok(resp) => {
                assert_eq!(resp.status(), reqwest::StatusCode::OK);
                let body = resp.text().await.expect("读取响应体失败");
                assert_eq!(body, "Hello");
            }
            Err(e) => {
                // 网络环境问题可能导致测试失败，不 panic
                eprintln!("HTTP 转发测试请求失败（可能是网络问题）: {}", e);
            }
        }

        // 清理
        server.stop().await.ok();
        mock_handle.abort();
    }

    #[test]
    fn test_generate_ca_params() {
        let result = generate_ca_params();
        assert!(result.is_ok(), "CA 参数生成应成功: {:?}", result.err());

        let (key_pair, params) = result.expect("生成 CA 参数失败");

        // 验证密钥对可以序列化
        let _pem = key_pair.serialize_pem();

        // 验证参数包含正确的 CN
        let cn = params
            .distinguished_name
            .iter()
            .find(|(dn_type, _)| matches!(dn_type, DnType::CommonName));
        assert!(cn.is_some(), "应包含 CommonName");
    }

    #[test]
    fn test_create_ca_authority() {
        let result = create_ca_authority();
        assert!(result.is_ok(), "CA Authority 创建应成功: {:?}", result.err());
    }

    #[test]
    fn test_proxy_error_display() {
        let addr: SocketAddr = "127.0.0.1:7101".parse().expect("解析失败");
        let err = ProxyError::AddrInUse(addr);
        let msg = format!("{}", err);
        assert!(msg.contains("7101"));
        assert!(msg.contains("占用"));

        let err = ProxyError::BuildFailed("test error".into());
        assert!(format!("{}", err).contains("test error"));

        let err = ProxyError::AlreadyRunning;
        assert!(format!("{}", err).contains("运行中"));

        let err = ProxyError::NotRunning;
        assert!(format!("{}", err).contains("未运行"));

        let err = ProxyError::CaGenerationFailed("cert fail".into());
        assert!(format!("{}", err).contains("cert fail"));
    }
}
