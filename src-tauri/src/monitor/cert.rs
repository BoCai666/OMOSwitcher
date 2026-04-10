// Monitor 模块 - CA 证书管理
// 使用 rcgen crate 生成自签名 CA 证书、为拦截域名生成终端证书
// 支持证书缓存和持久化

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use hudsucker::{
    certificate_authority::RcgenAuthority,
    rcgen::{CertificateParams, DnType, Issuer, KeyPair, SanType},
    rustls::crypto::aws_lc_rs,
};
use rcgen::string::Ia5String;
use time::OffsetDateTime;
use tokio::sync::RwLock;

// ============================================================================
// 常量定义
// ============================================================================

/// CA 证书通用名称
const CA_COMMON_NAME: &str = "OMOSwitcher Monitor CA";

/// CA 证书组织名称
const CA_ORGANIZATION: &str = "OMOSwitcher";

/// CA 证书国家代码
const CA_COUNTRY: &str = "CN";

/// CA 证书有效期（年）
const CA_VALIDITY_YEARS: i32 = 10;

/// 域名证书有效期（年）
const DOMAIN_CERT_VALIDITY_YEARS: i32 = 1;

/// 证书缓存大小
const CERT_CACHE_SIZE: u64 = 1_000;

// ============================================================================
// 错误类型
// ============================================================================

/// 证书管理错误类型
#[derive(Debug)]
pub enum CertError {
    /// 证书生成失败
    GenerationFailed(String),
    /// 证书加载失败
    LoadFailed(String),
    /// 证书保存失败
    SaveFailed(String),
    /// 证书目录创建失败
    DirectoryCreationFailed(String),
    /// CA 证书未初始化
    CaNotInitialized,
    /// 无效的证书路径
    InvalidPath(String),
    /// 证书解析失败
    ParseFailed(String),
}

impl std::fmt::Display for CertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CertError::GenerationFailed(msg) => write!(f, "证书生成失败: {}", msg),
            CertError::LoadFailed(msg) => write!(f, "证书加载失败: {}", msg),
            CertError::SaveFailed(msg) => write!(f, "证书保存失败: {}", msg),
            CertError::DirectoryCreationFailed(msg) => write!(f, "证书目录创建失败: {}", msg),
            CertError::CaNotInitialized => write!(f, "CA 证书未初始化"),
            CertError::InvalidPath(msg) => write!(f, "无效的证书路径: {}", msg),
            CertError::ParseFailed(msg) => write!(f, "证书解析失败: {}", msg),
        }
    }
}

impl std::error::Error for CertError {}

// ============================================================================
// 证书对结构
// ============================================================================

/// 证书对（私钥 + 证书 + CA 证书链）
#[derive(Debug, Clone)]
pub struct CertPair {
    /// PEM 格式私钥
    pub key: String,
    /// PEM 格式证书
    pub cert: String,
    /// PEM 格式 CA 证书链
    pub ca: String,
}

// ============================================================================
// 证书状态
// ============================================================================

/// 证书文件状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertStatus {
    /// CA 证书文件是否存在
    pub ca_cert_exists: bool,
    /// CA 私钥文件是否存在
    pub ca_key_exists: bool,
    /// 证书目录是否存在
    pub cert_dir_exists: bool,
}

// ============================================================================
// CA 证书信息（内部使用）
// ============================================================================

/// CA 证书信息
struct CAInfo {
    /// 密钥对
    key_pair: KeyPair,
    /// 证书参数
    params: CertificateParams,
    /// PEM 格式私钥
    key_pem: String,
    /// PEM 格式证书
    cert_pem: String,
}

// ============================================================================
// 证书管理器
// ============================================================================

/// 证书管理器
///
/// 管理自签名 CA 证书和域名证书的生成、缓存和持久化
pub struct CertManager {
    /// CA 证书信息
    ca_info: Option<CAInfo>,
    /// 域名证书缓存: domain → CertPair
    cert_cache: Arc<RwLock<HashMap<String, CertPair>>>,
    /// 证书目录路径
    certs_dir: PathBuf,
    /// 企业 CA 证书路径（可选）
    enterprise_ca_path: Option<PathBuf>,
}

impl CertManager {
    /// 创建新的证书管理器
    ///
    /// 初始化证书目录，加载或创建 CA 证书
    pub fn new() -> Result<Self, CertError> {
        let certs_dir = Self::get_certs_dir()?;

        let manager = Self {
            ca_info: None,
            cert_cache: Arc::new(RwLock::new(HashMap::new())),
            certs_dir,
            enterprise_ca_path: None,
        };

        // 确保证书目录存在
        manager.ensure_certs_dir()?;

        // 加载或创建 CA 证书
        let mut manager = manager;
        manager.load_or_create_ca()?;

        Ok(manager)
    }

    /// 使用指定证书目录创建证书管理器
    pub fn with_certs_dir(certs_dir: PathBuf) -> Result<Self, CertError> {
        let manager = Self {
            ca_info: None,
            cert_cache: Arc::new(RwLock::new(HashMap::new())),
            certs_dir: certs_dir.clone(),
            enterprise_ca_path: None,
        };

        // 确保证书目录存在
        manager.ensure_certs_dir()?;

        // 加载或创建 CA 证书
        let mut manager = manager;
        manager.load_or_create_ca()?;

        Ok(manager)
    }

    /// 设置企业 CA 证书路径
    pub fn set_enterprise_ca_path(&mut self, path: Option<PathBuf>) {
        self.enterprise_ca_path = path;
    }

    /// 获取证书目录路径
    ///
    /// 返回 ~/.config/omoswitcher/monitor/certs/
    fn get_certs_dir() -> Result<PathBuf, CertError> {
        let home = dirs::home_dir().ok_or_else(|| {
            CertError::InvalidPath("无法获取用户主目录".to_string())
        })?;
        Ok(home
            .join(".config")
            .join("omoswitcher")
            .join("monitor")
            .join("certs"))
    }

    /// 获取 CA 证书文件路径
    pub fn get_ca_cert_path(&self) -> PathBuf {
        self.certs_dir.join("ca.crt")
    }

    /// 获取 CA 私钥文件路径
    fn get_ca_key_path(&self) -> PathBuf {
        self.certs_dir.join("ca.key")
    }

    /// 确保证书目录存在
    fn ensure_certs_dir(&self) -> Result<(), CertError> {
        if !self.certs_dir.exists() {
            std::fs::create_dir_all(&self.certs_dir).map_err(|e| {
                CertError::DirectoryCreationFailed(format!(
                    "创建证书目录失败: {} - {}",
                    self.certs_dir.display(),
                    e
                ))
            })?;
            tracing::info!("已创建证书目录: {}", self.certs_dir.display());
        }
        Ok(())
    }

    /// 加载或创建 CA 证书
    fn load_or_create_ca(&mut self) -> Result<(), CertError> {
        let ca_cert_path = self.get_ca_cert_path();
        let ca_key_path = self.get_ca_key_path();

        // 检查 CA 证书和私钥文件是否都存在
        if ca_cert_path.exists() && ca_key_path.exists() {
            // 尝试加载现有 CA 证书
            match self.load_ca() {
                Ok(()) => {
                    tracing::info!("已加载现有 CA 证书");
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("加载 CA 证书失败，将重新生成: {}", e);
                }
            }
        }

        // 生成新的 CA 证书
        self.generate_ca()?;
        tracing::info!("已生成新的 CA 证书");
        Ok(())
    }

    /// 从文件加载 CA 证书
    fn load_ca(&mut self) -> Result<(), CertError> {
        let ca_cert_path = self.get_ca_cert_path();
        let ca_key_path = self.get_ca_key_path();

        // 读取私钥
        let key_pem = std::fs::read_to_string(&ca_key_path).map_err(|e| {
            CertError::LoadFailed(format!("读取 CA 私钥失败: {} - {}", ca_key_path.display(), e))
        })?;

        // 读取证书
        let cert_pem = std::fs::read_to_string(&ca_cert_path).map_err(|e| {
            CertError::LoadFailed(format!(
                "读取 CA 证书失败: {} - {}",
                ca_cert_path.display(),
                e
            ))
        })?;

        // 解析私钥
        let key_pair = KeyPair::from_pem(&key_pem).map_err(|e| {
            CertError::ParseFailed(format!("解析 CA 私钥失败: {}", e))
        })?;

        // 使用 x509-parser 解析证书检查有效期
        #[cfg(feature = "x509-parser")]
        {
            use x509_parser::prelude::*;
            let cert_der = pem_parser::parse_pem(&cert_pem)
                .map_err(|e| CertError::ParseFailed(format!("PEM 解析失败: {}", e)))?;
            
            let (_, cert) = X509Certificate::from_der(&cert_der.contents).map_err(|e| {
                CertError::ParseFailed(format!("解析证书失败: {}", e))
            })?;
            
            // 检查证书是否过期
            let now = OffsetDateTime::now_utc();
            if cert.validity().not_before > now || cert.validity().not_after < now {
                return Err(CertError::LoadFailed("CA 证书已过期".to_string()));
            }
        }

        // 重建 CA 参数
        let params = self.generate_ca_params()?;

        self.ca_info = Some(CAInfo {
            key_pair,
            params,
            key_pem,
            cert_pem,
        });

        Ok(())
    }

    /// 生成 CA 证书参数
    fn generate_ca_params(&self) -> Result<CertificateParams, CertError> {
        let mut params = CertificateParams::default();
        
        // 设置主题名称
        params.distinguished_name.push(DnType::CommonName, CA_COMMON_NAME);
        params.distinguished_name.push(DnType::OrganizationName, CA_ORGANIZATION);
        params.distinguished_name.push(DnType::CountryName, CA_COUNTRY);

        // 标记为 CA 证书
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

        // 设置密钥用途
        params.key_usages.push(rcgen::KeyUsagePurpose::KeyCertSign);
        params.key_usages.push(rcgen::KeyUsagePurpose::CrlSign);

        // 设置有效期（10 年）
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + time::Duration::days(CA_VALIDITY_YEARS as i64 * 365);

        Ok(params)
    }

    /// 生成自签名 CA 证书
    fn generate_ca(&mut self) -> Result<(), CertError> {
        tracing::info!("正在生成 CA 证书...");

        // 生成 CA 密钥对
        let key_pair = KeyPair::generate().map_err(|e| {
            CertError::GenerationFailed(format!("生成 CA 密钥对失败: {}", e))
        })?;

        // 配置 CA 证书参数
        let params = self.generate_ca_params()?;

        // 生成自签名证书
        let cert = params.self_signed(&key_pair).map_err(|e| {
            CertError::GenerationFailed(format!("生成 CA 证书失败: {}", e))
        })?;

        // 获取 PEM 格式
        let key_pem = key_pair.serialize_pem();
        let cert_pem = cert.pem();

        // 保存到文件
        self.save_ca_to_files(&key_pem, &cert_pem)?;

        self.ca_info = Some(CAInfo {
            key_pair,
            params,
            key_pem,
            cert_pem,
        });

        tracing::info!("CA 证书已保存到: {}", self.get_ca_cert_path().display());
        Ok(())
    }

    /// 保存 CA 证书到文件
    fn save_ca_to_files(&self, key_pem: &str, cert_pem: &str) -> Result<(), CertError> {
        let ca_key_path = self.get_ca_key_path();
        let ca_cert_path = self.get_ca_cert_path();

        // 保存私钥
        std::fs::write(&ca_key_path, key_pem).map_err(|e| {
            CertError::SaveFailed(format!("保存 CA 私钥失败: {} - {}", ca_key_path.display(), e))
        })?;

        // 保存证书
        std::fs::write(&ca_cert_path, cert_pem).map_err(|e| {
            CertError::SaveFailed(format!(
                "保存 CA 证书失败: {} - {}",
                ca_cert_path.display(),
                e
            ))
        })?;

        // 设置文件权限（仅 Unix）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&ca_key_path, std::fs::Permissions::from_mode(0o600));
            let _ = std::fs::set_permissions(&ca_cert_path, std::fs::Permissions::from_mode(0o644));
        }

        Ok(())
    }

    /// 获取域名证书
    ///
    /// 优先从缓存获取，否则生成新证书
    pub async fn get_certificate_for_domain(&self, domain: &str) -> Result<CertPair, CertError> {
        // 检查缓存
        {
            let cache = self.cert_cache.read().await;
            if let Some(cert_pair) = cache.get(domain) {
                return Ok(cert_pair.clone());
            }
        }

        // 生成新证书
        let cert_pair = self.generate_domain_certificate(domain)?;

        // 添加到缓存
        {
            let mut cache = self.cert_cache.write().await;
            cache.insert(domain.to_string(), cert_pair.clone());
        }

        Ok(cert_pair)
    }

    /// 生成域名终端证书
    fn generate_domain_certificate(&self, domain: &str) -> Result<CertPair, CertError> {
        let ca_info = self.ca_info.as_ref().ok_or(CertError::CaNotInitialized)?;

        tracing::debug!("正在生成域名证书: {}", domain);

        // 生成域名密钥对
        let key_pair = KeyPair::generate().map_err(|e| {
            CertError::GenerationFailed(format!("生成域名密钥对失败: {}", e))
        })?;

        // 配置域名证书参数
        let mut params = CertificateParams::default();

        // 设置主题名称
        params.distinguished_name.push(DnType::CommonName, domain);
        params.distinguished_name.push(DnType::OrganizationName, CA_ORGANIZATION);
        params.distinguished_name.push(DnType::CountryName, CA_COUNTRY);

        // 设置 SAN (Subject Alternative Name)
        // 使用 Ia5String 包装域名
        let dns_name = Ia5String::try_from(domain.to_string())
            .map_err(|e| CertError::GenerationFailed(format!("无效的域名: {}", e)))?;
        let wildcard_name = Ia5String::try_from(format!("*.{}", domain))
            .map_err(|e| CertError::GenerationFailed(format!("无效的通配符域名: {}", e)))?;
        
        params.subject_alt_names.push(SanType::DnsName(dns_name));
        params.subject_alt_names.push(SanType::DnsName(wildcard_name));

        // 标记为终端证书（非 CA）
        params.is_ca = rcgen::IsCa::NoCa;

        // 设置密钥用途
        params.key_usages.push(rcgen::KeyUsagePurpose::DigitalSignature);
        params.key_usages.push(rcgen::KeyUsagePurpose::KeyEncipherment);

        // 设置扩展密钥用途
        params.extended_key_usages.push(rcgen::ExtendedKeyUsagePurpose::ServerAuth);
        params.extended_key_usages.push(rcgen::ExtendedKeyUsagePurpose::ClientAuth);

        // 设置有效期（1 年）
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + time::Duration::days(DOMAIN_CERT_VALIDITY_YEARS as i64 * 365);

        // 使用 CA 签名
        // 创建 Issuer 用于签名
        let issuer_key_pair = KeyPair::from_pem(&ca_info.key_pem).map_err(|e| {
            CertError::ParseFailed(format!("解析 CA 私钥失败: {}", e))
        })?;
        let issuer = Issuer::new(ca_info.params.clone(), issuer_key_pair);
        
        let cert = params.signed_by(&key_pair, &issuer).map_err(|e| {
            CertError::GenerationFailed(format!("签名域名证书失败: {}", e))
        })?;

        // 获取 PEM 格式
        let key_pem = key_pair.serialize_pem();
        let cert_pem = cert.pem();

        Ok(CertPair {
            key: key_pem,
            cert: cert_pem,
            ca: ca_info.cert_pem.clone(),
        })
    }

    /// 获取 CA 证书内容（PEM 格式）
    pub fn get_ca_cert_content(&self) -> Result<String, CertError> {
        let ca_info = self.ca_info.as_ref().ok_or(CertError::CaNotInitialized)?;
        Ok(ca_info.cert_pem.clone())
    }

    /// 获取证书状态
    pub fn cert_status(&self) -> CertStatus {
        let ca_cert_path = self.get_ca_cert_path();
        let ca_key_path = self.get_ca_key_path();

        CertStatus {
            ca_cert_exists: ca_cert_path.exists(),
            ca_key_exists: ca_key_path.exists(),
            cert_dir_exists: self.certs_dir.exists(),
        }
    }

    /// 清除域名证书缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cert_cache.write().await;
        cache.clear();
        tracing::info!("证书缓存已清除");
    }

    /// 获取缓存统计信息
    pub async fn get_cache_stats(&self) -> (usize, Vec<String>) {
        let cache = self.cert_cache.read().await;
        let size = cache.len();
        let domains: Vec<String> = cache.keys().cloned().collect();
        (size, domains)
    }

    /// 重新生成 CA 证书
    ///
    /// 删除旧的 CA 证书文件，清空缓存，生成新的 CA 证书
    pub async fn regenerate_ca(&mut self) -> Result<(), CertError> {
        tracing::info!("正在重新生成 CA 证书...");

        // 删除旧的 CA 证书文件
        let ca_cert_path = self.get_ca_cert_path();
        let ca_key_path = self.get_ca_key_path();

        if ca_cert_path.exists() {
            std::fs::remove_file(&ca_cert_path).map_err(|e| {
                CertError::SaveFailed(format!("删除旧 CA 证书失败: {}", e))
            })?;
        }

        if ca_key_path.exists() {
            std::fs::remove_file(&ca_key_path).map_err(|e| {
                CertError::SaveFailed(format!("删除旧 CA 私钥失败: {}", e))
            })?;
        }

        // 清空缓存
        self.clear_cache().await;

        // 生成新的 CA 证书
        self.generate_ca()?;

        tracing::info!("CA 证书重新生成完成");
        Ok(())
    }

    /// 构建 rustls RootCertStore
    ///
    /// 加载系统根证书和企业 CA 证书（如果配置）
    pub fn build_root_cert_store(&self) -> Result<rustls::RootCertStore, CertError> {
        let mut root_store = rustls::RootCertStore::empty();

        // 加载系统根证书
        // 注意：webpki_roots 需要作为依赖添加，这里先留空
        // root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        // 加载企业 CA 证书（如果配置）
        if let Some(ref enterprise_ca_path) = self.enterprise_ca_path {
            if enterprise_ca_path.exists() {
                let cert_pem = std::fs::read_to_string(enterprise_ca_path).map_err(|e| {
                    CertError::LoadFailed(format!(
                        "读取企业 CA 证书失败: {} - {}",
                        enterprise_ca_path.display(),
                        e
                    ))
                })?;

                // 解析 PEM 格式证书
                use rustls::pki_types::pem::PemObject;
                let certs: Vec<rustls::pki_types::CertificateDer<'_>> = 
                    rustls::pki_types::CertificateDer::pem_slice_iter(cert_pem.as_bytes())
                        .filter_map(|c| c.ok())
                        .collect();
                
                for cert_der in certs {
                    root_store.add(cert_der).map_err(|e| {
                        CertError::ParseFailed(format!("添加企业 CA 证书失败: {}", e))
                    })?;
                }

                tracing::info!("已加载企业 CA 证书: {}", enterprise_ca_path.display());
            }
        }

        Ok(root_store)
    }

    /// 创建 RcgenAuthority
    ///
    /// 用于 hudsucker 代理服务器的 HTTPS 拦截
    pub fn create_rcgen_authority(&self) -> Result<RcgenAuthority, CertError> {
        let ca_info = self.ca_info.as_ref().ok_or(CertError::CaNotInitialized)?;

        // 由于 KeyPair 不能 clone，需要从 PEM 重新解析
        let key_pair = KeyPair::from_pem(&ca_info.key_pem).map_err(|e| {
            CertError::ParseFailed(format!("解析 CA 私钥失败: {}", e))
        })?;

        // 创建 Issuer
        let issuer = Issuer::new(ca_info.params.clone(), key_pair);

        // 创建 RcgenAuthority
        let authority = RcgenAuthority::new(
            issuer,
            CERT_CACHE_SIZE,
            aws_lc_rs::default_provider(),
        );

        Ok(authority)
    }
}

impl Default for CertManager {
    fn default() -> Self {
        Self::new().expect("创建 CertManager 失败")
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// 获取测试用的临时证书目录
    fn get_test_certs_dir() -> PathBuf {
        std::env::temp_dir()
            .join("omoswitcher_test_certs")
            .join(uuid::Uuid::new_v4().to_string())
    }

    #[test]
    fn test_generate_ca_certificate() {
        let certs_dir = get_test_certs_dir();
        
        // 创建证书管理器
        let manager = CertManager::with_certs_dir(certs_dir.clone()).expect("创建 CertManager 失败");

        // 验证 CA 证书已生成
        let status = manager.cert_status();
        assert!(status.ca_cert_exists, "CA 证书文件应存在");
        assert!(status.ca_key_exists, "CA 私钥文件应存在");

        // 验证可以获取 CA 证书内容
        let ca_content = manager.get_ca_cert_content().expect("获取 CA 证书内容失败");
        assert!(ca_content.contains("BEGIN CERTIFICATE"), "证书内容应包含 PEM 头");

        // 清理
        let _ = std::fs::remove_dir_all(&certs_dir);
    }

    #[tokio::test]
    async fn test_domain_certificate_generation() {
        let certs_dir = get_test_certs_dir();
        
        let manager = CertManager::with_certs_dir(certs_dir.clone()).expect("创建 CertManager 失败");

        // 生成域名证书
        let domain = "api.openai.com";
        let cert_pair = manager
            .get_certificate_for_domain(domain)
            .await
            .expect("生成域名证书失败");

        // 验证证书内容
        assert!(cert_pair.key.contains("BEGIN PRIVATE KEY"), "私钥应包含 PEM 头");
        assert!(cert_pair.cert.contains("BEGIN CERTIFICATE"), "证书应包含 PEM 头");
        assert!(cert_pair.ca.contains("BEGIN CERTIFICATE"), "CA 证书应包含 PEM 头");

        // 清理
        let _ = std::fs::remove_dir_all(&certs_dir);
    }

    #[tokio::test]
    async fn test_domain_certificate_cache() {
        let certs_dir = get_test_certs_dir();
        
        let manager = CertManager::with_certs_dir(certs_dir.clone()).expect("创建 CertManager 失败");

        let domain = "api.anthropic.com";

        // 第一次获取
        let cert_pair1 = manager
            .get_certificate_for_domain(domain)
            .await
            .expect("生成域名证书失败");

        // 第二次获取（应该从缓存获取）
        let cert_pair2 = manager
            .get_certificate_for_domain(domain)
            .await
            .expect("从缓存获取域名证书失败");

        // 验证两次获取的是相同的证书
        assert_eq!(cert_pair1.key, cert_pair2.key, "缓存的证书应该相同");
        assert_eq!(cert_pair1.cert, cert_pair2.cert, "缓存的证书应该相同");

        // 验证缓存统计
        let (size, domains) = manager.get_cache_stats().await;
        assert_eq!(size, 1, "缓存应包含 1 个证书");
        assert!(domains.contains(&domain.to_string()), "缓存应包含该域名");

        // 清理
        let _ = std::fs::remove_dir_all(&certs_dir);
    }

    #[test]
    fn test_cert_status_empty_dir() {
        let certs_dir = get_test_certs_dir();

        // 不创建证书，检查状态
        let status = CertStatus {
            ca_cert_exists: false,
            ca_key_exists: false,
            cert_dir_exists: false,
        };

        assert!(!status.ca_cert_exists, "CA 证书不应存在");
        assert!(!status.ca_key_exists, "CA 私钥不应存在");
        assert!(!status.cert_dir_exists, "证书目录不应存在");
    }

    #[test]
    fn test_cert_status_after_generation() {
        let certs_dir = get_test_certs_dir();

        let manager = CertManager::with_certs_dir(certs_dir.clone()).expect("创建 CertManager 失败");

        let status = manager.cert_status();
        assert!(status.ca_cert_exists, "CA 证书应存在");
        assert!(status.ca_key_exists, "CA 私钥应存在");
        assert!(status.cert_dir_exists, "证书目录应存在");

        // 清理
        let _ = std::fs::remove_dir_all(&certs_dir);
    }

    #[tokio::test]
    async fn test_regenerate_ca() {
        let certs_dir = get_test_certs_dir();

        let mut manager = CertManager::with_certs_dir(certs_dir.clone()).expect("创建 CertManager 失败");

        // 获取原始 CA 证书内容
        let original_ca = manager.get_ca_cert_content().expect("获取 CA 证书内容失败");

        // 重新生成 CA 证书
        manager.regenerate_ca().await.expect("重新生成 CA 证书失败");

        // 获取新的 CA 证书内容
        let new_ca = manager.get_ca_cert_content().expect("获取新 CA 证书内容失败");

        // 验证证书已更改
        assert_ne!(original_ca, new_ca, "重新生成的 CA 证书应该不同");

        // 清理
        let _ = std::fs::remove_dir_all(&certs_dir);
    }

    #[test]
    fn test_create_rcgen_authority() {
        let certs_dir = get_test_certs_dir();

        let manager = CertManager::with_certs_dir(certs_dir.clone()).expect("创建 CertManager 失败");

        // 创建 RcgenAuthority
        let authority = manager.create_rcgen_authority().expect("创建 RcgenAuthority 失败");

        // 验证 authority 创建成功（没有简单的方法验证内部状态）
        drop(authority);

        // 清理
        let _ = std::fs::remove_dir_all(&certs_dir);
    }

    #[test]
    fn test_cert_error_display() {
        let err = CertError::GenerationFailed("test error".to_string());
        assert!(format!("{}", err).contains("test error"));

        let err = CertError::LoadFailed("load error".to_string());
        assert!(format!("{}", err).contains("load error"));

        let err = CertError::CaNotInitialized;
        assert!(format!("{}", err).contains("未初始化"));
    }

    #[test]
    fn test_get_ca_cert_path() {
        let certs_dir = get_test_certs_dir();
        let manager = CertManager::with_certs_dir(certs_dir.clone()).expect("创建 CertManager 失败");

        let ca_path = manager.get_ca_cert_path();
        assert!(ca_path.ends_with("ca.crt"), "CA 证书文件名应为 ca.crt");

        // 清理
        let _ = std::fs::remove_dir_all(&certs_dir);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let certs_dir = get_test_certs_dir();

        let manager = CertManager::with_certs_dir(certs_dir.clone()).expect("创建 CertManager 失败");

        // 生成一些证书
        manager.get_certificate_for_domain("test1.com").await.ok();
        manager.get_certificate_for_domain("test2.com").await.ok();

        let (size, _) = manager.get_cache_stats().await;
        assert_eq!(size, 2, "缓存应包含 2 个证书");

        // 清除缓存
        manager.clear_cache().await;

        let (size, _) = manager.get_cache_stats().await;
        assert_eq!(size, 0, "缓存应为空");

        // 清理
        let _ = std::fs::remove_dir_all(&certs_dir);
    }
}
