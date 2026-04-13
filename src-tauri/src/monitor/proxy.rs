// Monitor 模块 - HTTP/HTTPS 代理服务器
// 使用 hudsucker 构建的代理服务器，支持 HTTP 透传和 HTTPS CONNECT 隧道
// 支持完整 MITM 拦截模式
// 注意：部分功能（临时 CA、透传模式、默认构造）保留供测试使用

#![allow(dead_code)]

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

/// 生成临时自签名 CA 的参数和密钥（用于测试）
///
/// 生产环境应使用 CertManager 提供的持久化 CA
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

/// 创建临时 RcgenAuthority（用于测试）
///
/// 生产环境应使用 CertManager::create_rcgen_authority()
pub fn create_temp_ca_authority() -> Result<RcgenAuthority, ProxyError> {
    let (key_pair, params) = generate_ca_params()?;

    let issuer = Issuer::new(params, key_pair);
    let ca = RcgenAuthority::new(issuer, CERT_CACHE_SIZE, aws_lc_rs::default_provider());

    Ok(ca)
}

/// 代理服务器
///
/// 基于 hudsucker 构建的 HTTP/HTTPS 代理服务器
/// - HTTP 请求：根据 handler 决定是否拦截
/// - HTTPS CONNECT：隧道模式
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

    /// 启动代理服务器（完整模式）
    ///
    /// 使用外部提供的 handler 和 CA，支持请求捕获
    pub async fn start(
        &self,
        handler: MonitorHandler,
        ca: RcgenAuthority,
    ) -> Result<(), ProxyError> {
        // 检查是否已在运行
        if self.is_running() {
            return Err(ProxyError::AlreadyRunning);
        }

        // 安装 aws_lc_rs crypto provider（如果尚未安装）
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        // 获取 shutdown signal
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
            tracing::info!("代理服务器启动（完整模式），监听 {}", addr);
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

        // 等待确认代理成功启动
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 验证端口
        let check_result = tokio::net::TcpStream::connect(self.addr).await;
        if check_result.is_err() {
            let running_state = self.running.load(Ordering::SeqCst);
            if !running_state {
                return Err(ProxyError::StartFailed(format!(
                    "代理服务器启动后端口 {} 不可达",
                    self.addr
                )));
            }
            tracing::warn!("代理服务器启动后端口验证失败，但状态仍为运行中，可能启动较慢");
        }

        tracing::info!("代理服务器已在 {} 上启动（完整模式）", self.addr);
        Ok(())
    }

    /// 启动代理服务器（透传模式，用于测试）
    ///
    /// 不捕获任何请求，所有流量直接转发
    pub async fn start_passthrough(&self) -> Result<(), ProxyError> {
        // 检查是否已在运行
        if self.is_running() {
            return Err(ProxyError::AlreadyRunning);
        }

        // 安装 aws_lc_rs crypto provider
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        // 创建临时 CA 和透传 handler
        let ca = create_temp_ca_authority()?;
        let handler = MonitorHandler::new_passthrough();

        // 获取 shutdown signal
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
            tracing::info!("代理服务器启动（透传模式），监听 {}", addr);
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

        // 等待确认代理成功启动
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 验证端口
        let check_result = tokio::net::TcpStream::connect(self.addr).await;
        if check_result.is_err() {
            let running_state = self.running.load(Ordering::SeqCst);
            if !running_state {
                return Err(ProxyError::StartFailed(format!(
                    "代理服务器启动后端口 {} 不可达",
                    self.addr
                )));
            }
            tracing::warn!("代理服务器启动后端口验证失败，但状态仍为运行中，可能启动较慢");
        }

        tracing::info!("代理服务器已在 {} 上启动（透传模式）", self.addr);
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
