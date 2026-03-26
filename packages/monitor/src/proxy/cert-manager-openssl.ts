import { homedir } from 'os';
import { join } from 'path';
import { existsSync, mkdirSync, readFileSync, writeFileSync, unlinkSync } from 'fs';
import { execSync } from 'child_process';
import { randomBytes } from 'crypto';

const CERT_DIR = join(homedir(), '.opencode-monitor', 'certs');
const CA_KEY_FILE = join(CERT_DIR, 'ca.key');
const CA_CERT_FILE = join(CERT_DIR, 'ca.crt');

export interface CertificatePair {
  key: string;
  cert: string;
  ca: string;
}

interface CAInfo {
  keyPem: string;
  certPem: string;
}

// 调试日志函数
function debugLog(label: string, data: any) {
  console.log(`[CertManager Debug] ${label}:`, data);
}

export class CertificateManager {
  private caInfo: CAInfo | null = null;
  private certCache: Map<string, CertificatePair> = new Map();

  constructor() {
    this.ensureCertDirectory();
    this.loadOrCreateCA();
  }

  private ensureCertDirectory(): void {
    if (!existsSync(CERT_DIR)) {
      mkdirSync(CERT_DIR, { recursive: true });
    }
  }

  private loadOrCreateCA(): void {
    if (existsSync(CA_KEY_FILE) && existsSync(CA_CERT_FILE)) {
      try {
        const keyPem = readFileSync(CA_KEY_FILE, 'utf-8');
        const certPem = readFileSync(CA_CERT_FILE, 'utf-8');
        
        this.caInfo = { keyPem, certPem };
        console.log('Loaded existing CA certificate');
        return;
      } catch (error) {
        console.warn('Failed to load CA certificate, regenerating:', error);
      }
    }
    this.generateCA();
  }

  private generateCA(): void {
    console.log('Generating new CA root certificate using OpenSSL...');

    try {
      // 生成 CA 私钥
      execSync(`openssl genrsa -out "${CA_KEY_FILE}" 2048`, { stdio: 'pipe' });
      
      // 生成自签名 CA 证书
      execSync(
        `openssl req -x509 -new -nodes -key "${CA_KEY_FILE}" -sha256 -days 3650 ` +
        `-out "${CA_CERT_FILE}" ` +
        `-subj "/C=CN/O=OpenCode Monitor/CN=OpenCode Monitor CA" ` +
        `-addext "basicConstraints=critical,CA:TRUE" ` +
        `-addext "keyUsage=critical,keyCertSign,cRLSign"`,
        { stdio: 'pipe' }
      );

      const keyPem = readFileSync(CA_KEY_FILE, 'utf-8');
      const certPem = readFileSync(CA_CERT_FILE, 'utf-8');

      this.caInfo = { keyPem, certPem };
      console.log(`CA certificate saved to: ${CA_CERT_FILE}`);
      
      // 验证 CA 证书
      try {
        const verifyOutput = execSync(`openssl verify -CAfile "${CA_CERT_FILE}" "${CA_CERT_FILE}"`, { encoding: 'utf-8' });
        debugLog('CA 证书自签名验证', verifyOutput.trim());
      } catch (e) {
        debugLog('CA 证书验证警告', e);
      }
    } catch (error) {
      console.error('Failed to generate CA certificate:', error);
      throw error;
    }
  }

  private generateDomainCertificate(domain: string): CertificatePair {
    if (!this.caInfo) {
      throw new Error('CA certificate not initialized');
    }

    console.log(`Generating certificate for domain: ${domain} using OpenSSL...`);

    const domainKeyFile = join(CERT_DIR, `${domain}.key`);
    const domainCertFile = join(CERT_DIR, `${domain}.crt`);
    const domainCsrFile = join(CERT_DIR, `${domain}.csr`);

    try {
      // 1. 生成域名私钥
      execSync(`openssl genrsa -out "${domainKeyFile}" 2048`, { stdio: 'pipe' });

      // 2. 创建 CSR (证书签名请求)
      execSync(
        `openssl req -new -key "${domainKeyFile}" -out "${domainCsrFile}" ` +
        `-subj "/C=CN/O=OpenCode Monitor/CN=${domain}"`,
        { stdio: 'pipe' }
      );

      // 3. 创建扩展配置文件
      const extFile = join(CERT_DIR, `${domain}.ext`);
      const extContent = `authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = ${domain}
DNS.2 = *.${domain}
`;
      writeFileSync(extFile, extContent);

      // 4. 使用 CA 签名证书
      execSync(
        `openssl x509 -req -in "${domainCsrFile}" -CA "${CA_CERT_FILE}" -CAkey "${CA_KEY_FILE}" ` +
        `-CAcreateserial -out "${domainCertFile}" -days 365 -sha256 ` +
        `-extfile "${extFile}"`,
        { stdio: 'pipe' }
      );

      // 5. 读取生成的文件
      const keyPem = readFileSync(domainKeyFile, 'utf-8');
      const certPem = readFileSync(domainCertFile, 'utf-8');

      // 6. 清理临时文件
      try {
        unlinkSync(domainKeyFile);
        unlinkSync(domainCertFile);
        unlinkSync(domainCsrFile);
        unlinkSync(extFile);
        unlinkSync(join(CERT_DIR, `${domain}.srl`));
      } catch (e) {
        // 忽略清理错误
      }

      console.log(`Certificate generated for: ${domain}`);
      
      // 验证证书链
      try {
        const fullChain = `${certPem}\n${this.caInfo.certPem}`;
        debugLog(`叶子证书(${domain}) 生成成功`, { keyLength: keyPem.length, certLength: certPem.length });
      } catch (e) {
        debugLog(`叶子证书(${domain}) 验证警告`, e);
      }
      
      return {
        key: keyPem,
        cert: certPem,
        ca: this.caInfo.certPem
      };
    } catch (error) {
      console.error(`Failed to generate certificate for ${domain}:`, error);
      
      // 清理临时文件
      try {
        if (existsSync(domainKeyFile)) unlinkSync(domainKeyFile);
        if (existsSync(domainCertFile)) unlinkSync(domainCertFile);
        if (existsSync(domainCsrFile)) unlinkSync(domainCsrFile);
      } catch (e) {
        // 忽略
      }
      
      throw error;
    }
  }

  public getCertificateForDomain(domain: string): CertificatePair {
    if (this.certCache.has(domain)) {
      debugLog(`使用缓存的证书`, domain);
      return this.certCache.get(domain)!;
    }
    debugLog(`生成新证书`, domain);
    const certPair = this.generateDomainCertificate(domain);
    this.certCache.set(domain, certPair);
    return certPair;
  }

  public getCACertPath(): string {
    return CA_CERT_FILE;
  }

  public getCACertContent(): string {
    if (!this.caInfo) {
      throw new Error('CA certificate not initialized');
    }
    return this.caInfo.certPem;
  }

  public clearCache(): void {
    this.certCache.clear();
    console.log('Certificate cache cleared');
  }

  public getCacheStats(): { size: number; domains: string[] } {
    return {
      size: this.certCache.size,
      domains: Array.from(this.certCache.keys())
    };
  }

  public regenerateCA(): void {
    try {
      if (existsSync(CA_KEY_FILE)) {
        unlinkSync(CA_KEY_FILE);
      }
      if (existsSync(CA_CERT_FILE)) {
        unlinkSync(CA_CERT_FILE);
      }
      console.log('Deleted old CA certificate files');
    } catch (error) {
      console.warn('Failed to delete old CA files:', error);
    }

    this.certCache.clear();
    this.generateCA();
  }
}

export default CertificateManager;
