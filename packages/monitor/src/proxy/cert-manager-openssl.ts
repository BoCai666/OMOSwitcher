/**
 * 证书管理器 - OpenSSL 实现
 * 
 * 使用系统 OpenSSL 生成证书（需要系统安装 OpenSSL）
 * 证书存储路径: ~/.config/omoswitcher/monitor/certs/
 */

import { join } from 'path';
import { existsSync, mkdirSync, readFileSync, writeFileSync, unlinkSync } from 'fs';
import { execSync } from 'child_process';
import { randomBytes } from 'crypto';
import { CERTS_DIR, CA_CERT_FILE, CA_KEY_FILE } from '../paths.js';

export interface CertificatePair {
  key: string;
  cert: string;
  ca: string;
}

interface CAInfo {
  keyPem: string;
  certPem: string;
}

export class CertificateManager {
  private caInfo: CAInfo | null = null;
  private certCache: Map<string, CertificatePair> = new Map();

  constructor() {
    this.ensureCertDirectory();
    this.loadOrCreateCA();
  }

  private ensureCertDirectory(): void {
    if (!existsSync(CERTS_DIR)) {
      mkdirSync(CERTS_DIR, { recursive: true });
    }
  }

  private loadOrCreateCA(): void {
    if (existsSync(CA_KEY_FILE) && existsSync(CA_CERT_FILE)) {
      try {
        const keyPem = readFileSync(CA_KEY_FILE, 'utf-8');
        const certPem = readFileSync(CA_CERT_FILE, 'utf-8');

        this.caInfo = { keyPem, certPem };
        console.log('[CertManager-OpenSSL] 已加载现有 CA 证书');
        return;
      } catch (error) {
        console.warn('[CertManager-OpenSSL] 加载 CA 证书失败，重新生成:', error);
      }
    }
    this.generateCA();
  }

  private generateCA(): void {
    console.log('[CertManager-OpenSSL] 使用 OpenSSL 生成新的 CA 根证书...');

    try {
      // 生成 CA 私钥
      execSync(`openssl genrsa -out "${CA_KEY_FILE}" 2048`, { stdio: 'pipe' });

      // 创建 CA 扩展配置文件
      const caExtFile = join(CERTS_DIR, 'ca.cnf');
      const caExtContent = `[req]
distinguished_name = req_distinguished_name
x509_extensions = v3_ca
[req_distinguished_name]
[v3_ca]
basicConstraints = critical,CA:TRUE
keyUsage = critical,keyCertSign,cRLSign
subjectKeyIdentifier = hash
extendedKeyUsage = serverAuth
`;
      writeFileSync(caExtFile, caExtContent);

      // 生成自签名 CA 证书
      execSync(
        `openssl req -x509 -sha256 -new -nodes -key "${CA_KEY_FILE}" -days 3650 ` +
        `-out "${CA_CERT_FILE}" ` +
        `-subj "/C=CN/O=OpenCode Monitor/CN=OpenCode Monitor CA" ` +
        `-config "${caExtFile}"`,
        { stdio: 'pipe' }
      );

      // 清理临时文件
      try {
        unlinkSync(caExtFile);
        const srlFile = join(CERTS_DIR, 'ca.srl');
        if (existsSync(srlFile)) unlinkSync(srlFile);
      } catch {
        // 忽略
      }

      const keyPem = readFileSync(CA_KEY_FILE, 'utf-8');
      const certPem = readFileSync(CA_CERT_FILE, 'utf-8');

      this.caInfo = { keyPem, certPem };
      console.log(`[CertManager-OpenSSL] CA 证书已保存: ${CA_CERT_FILE}`);
    } catch (error) {
      console.error('[CertManager-OpenSSL] 生成 CA 证书失败:', error);
      throw error;
    }
  }

  private generateDomainCertificate(domain: string): CertificatePair {
    if (!this.caInfo) {
      throw new Error('CA 证书未初始化');
    }

    console.log(`[CertManager-OpenSSL] 使用 OpenSSL 生成域名证书: ${domain}`);

    const domainKeyFile = join(CERTS_DIR, `${domain}.key`);
    const domainCertFile = join(CERTS_DIR, `${domain}.crt`);
    const domainCsrFile = join(CERTS_DIR, `${domain}.csr`);

    try {
      // 1. 生成域名私钥
      execSync(`openssl genrsa -out "${domainKeyFile}" 2048`, { stdio: 'pipe' });

      // 2. 创建 CSR
      execSync(
        `openssl req -new -key "${domainKeyFile}" -out "${domainCsrFile}" ` +
        `-subj "/C=CN/O=OpenCode Monitor/CN=${domain}"`,
        { stdio: 'pipe' }
      );

      // 3. 创建扩展配置
      const extFile = join(CERTS_DIR, `${domain}.cnf`);
      const extContent = `[v3_req]
basicConstraints = CA:FALSE
keyUsage = digitalSignature,keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = DNS:${domain}
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid,issuer
`;
      writeFileSync(extFile, extContent);

      // 4. 使用 CA 签名
      execSync(
        `openssl x509 -req -sha256 -in "${domainCsrFile}" -CA "${CA_CERT_FILE}" -CAkey "${CA_KEY_FILE}" ` +
        `-CAcreateserial -out "${domainCertFile}" -days 365 ` +
        `-extfile "${extFile}" -extensions v3_req`,
        { stdio: 'pipe' }
      );

      // 5. 读取证书
      const keyPem = readFileSync(domainKeyFile, 'utf-8');
      const certPem = readFileSync(domainCertFile, 'utf-8');

      // 6. 清理临时文件
      try {
        unlinkSync(domainKeyFile);
        unlinkSync(domainCertFile);
        unlinkSync(domainCsrFile);
        unlinkSync(extFile);
        unlinkSync(join(CERTS_DIR, `${domain}.srl`));
      } catch {
        // 忽略
      }

      console.log(`[CertManager-OpenSSL] 域名证书已生成: ${domain}`);

      return {
        key: keyPem,
        cert: certPem,
        ca: this.caInfo.certPem
      };
    } catch (error) {
      console.error(`[CertManager-OpenSSL] 生成域名证书失败 ${domain}:`, error);
      // 清理临时文件
      try {
        if (existsSync(domainKeyFile)) unlinkSync(domainKeyFile);
        if (existsSync(domainCertFile)) unlinkSync(domainCertFile);
        if (existsSync(domainCsrFile)) unlinkSync(domainCsrFile);
      } catch {
        // 忽略
      }
      throw error;
    }
  }

  public getCertificateForDomain(domain: string): CertificatePair {
    if (this.certCache.has(domain)) {
      return this.certCache.get(domain)!;
    }
    const certPair = this.generateDomainCertificate(domain);
    this.certCache.set(domain, certPair);
    return certPair;
  }

  public getCACertPath(): string {
    return CA_CERT_FILE;
  }

  public getCACertContent(): string {
    if (!this.caInfo) {
      throw new Error('CA 证书未初始化');
    }
    return this.caInfo.certPem;
  }

  public clearCache(): void {
    this.certCache.clear();
    console.log('[CertManager-OpenSSL] 证书缓存已清除');
  }

  public getCacheStats(): { size: number; domains: string[] } {
    return {
      size: this.certCache.size,
      domains: Array.from(this.certCache.keys())
    };
  }

  public regenerateCA(): void {
    try {
      if (existsSync(CA_KEY_FILE)) unlinkSync(CA_KEY_FILE);
      if (existsSync(CA_CERT_FILE)) unlinkSync(CA_CERT_FILE);
      console.log('[CertManager-OpenSSL] 已删除旧的 CA 证书文件');
    } catch (error) {
      console.warn('[CertManager-OpenSSL] 删除旧 CA 文件失败:', error);
    }

    this.certCache.clear();
    this.generateCA();
  }
}

export default CertificateManager;
