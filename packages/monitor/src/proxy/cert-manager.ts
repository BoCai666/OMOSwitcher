/**
 * 证书管理器 (node-forge 实现)
 * 
 * 使用纯 JavaScript 实现，无需系统 OpenSSL 依赖
 * 证书存储路径: ~/.config/omoswitcher/monitor/certs/
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync, unlinkSync } from 'fs';
import forge from 'node-forge';
import { randomBytes } from 'crypto';
import { CERTS_DIR, CA_CERT_FILE, CA_KEY_FILE } from '../paths.js';

export interface CertificatePair {
  key: string;
  cert: string;
  ca: string;
}

interface CAInfo {
  key: forge.pki.rsa.PrivateKey;
  cert: forge.pki.Certificate;
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
        const key = forge.pki.privateKeyFromPem(keyPem);
        const cert = forge.pki.certificateFromPem(certPem);

        // 检查证书是否过期
        if (cert.validity.notAfter < new Date()) {
          console.log('[CertManager] CA 证书已过期，重新生成...');
          this.generateCA();
          return;
        }

        // 验证私钥和公钥是否匹配
        try {
          const testData = 'test';
          const signature = key.sign(forge.md.sha256.create().update(testData));
          const publicKey = cert.publicKey as forge.pki.rsa.PublicKey;
          const verified = publicKey.verify(
            forge.md.sha256.create().update(testData).digest().getBytes(),
            signature
          );
          if (!verified) {
            console.warn('[CertManager] CA 私钥和证书不匹配，重新生成...');
            this.generateCA();
            return;
          }
        } catch {
          console.warn('[CertManager] CA 密钥验证失败，重新生成...');
          this.generateCA();
          return;
        }

        this.caInfo = { key, cert, keyPem, certPem };
        console.log('[CertManager] 已加载现有 CA 证书');
        return;
      } catch (error) {
        console.warn('[CertManager] 加载 CA 证书失败，重新生成:', error);
      }
    }
    this.generateCA();
  }

  private generateCA(): void {
    console.log('[CertManager] 生成新的 CA 根证书...');

    const keyPair = forge.pki.rsa.generateKeyPair({ bits: 2048 });
    const cert = forge.pki.createCertificate();
    cert.publicKey = keyPair.publicKey;
    cert.serialNumber = '00' + randomBytes(16).toString('hex');

    const now = new Date();
    cert.validity.notBefore = now;
    cert.validity.notAfter = new Date(now.getTime() + 10 * 365 * 24 * 60 * 60 * 1000); // 10 年

    const subject = [
      { name: 'commonName', value: 'OpenCode Monitor CA' },
      { name: 'organizationName', value: 'OpenCode Monitor' },
      { name: 'countryName', value: 'CN' }
    ];
    cert.setSubject(subject);
    cert.setIssuer(subject);

    // Subject Key Identifier
    const ski = forge.pki.getPublicKeyFingerprint(keyPair.publicKey, {
      type: 'SubjectPublicKeyInfo',
      encoding: 'hex'
    });

    cert.setExtensions([
      { name: 'basicConstraints', cA: true, critical: true },
      { name: 'keyUsage', keyCertSign: true, cRLSign: true, critical: true },
      { name: 'subjectKeyIdentifier', keyIdentifier: forge.util.hexToBytes(ski) }
    ]);

    cert.sign(keyPair.privateKey, forge.md.sha256.create());

    const keyPem = forge.pki.privateKeyToPem(keyPair.privateKey);
    const certPem = forge.pki.certificateToPem(cert);

    this.caInfo = { key: keyPair.privateKey, cert, keyPem, certPem };

    writeFileSync(CA_KEY_FILE, keyPem, { mode: 0o600 });
    writeFileSync(CA_CERT_FILE, certPem, { mode: 0o644 });

    console.log(`[CertManager] CA 证书已保存: ${CA_CERT_FILE}`);
  }

  private generateDomainCertificate(domain: string): CertificatePair {
    if (!this.caInfo) {
      throw new Error('CA 证书未初始化');
    }

    console.log(`[CertManager] 生成域名证书: ${domain}`);

    const keyPair = forge.pki.rsa.generateKeyPair({ bits: 2048 });
    const cert = forge.pki.createCertificate();
    cert.publicKey = keyPair.publicKey;
    cert.serialNumber = '00' + randomBytes(16).toString('hex');

    const now = new Date();
    cert.validity.notBefore = now;
    cert.validity.notAfter = new Date(now.getTime() + 365 * 24 * 60 * 60 * 1000); // 1 年

    cert.setSubject([
      { name: 'commonName', value: domain },
      { name: 'organizationName', value: 'OpenCode Monitor' },
      { name: 'countryName', value: 'CN' }
    ]);
    cert.setIssuer(this.caInfo.cert.subject.attributes);

    // 获取 CA 的 Subject Key Identifier
    const caSkiExt = this.caInfo.cert.getExtension('subjectKeyIdentifier') as {
      keyIdentifier?: string | Uint8Array;
    } | null;
    const caSki = caSkiExt?.keyIdentifier || '';

    // 叶子证书的 Subject Key Identifier
    const leafSki = forge.pki.getPublicKeyFingerprint(keyPair.publicKey, {
      type: 'SubjectPublicKeyInfo',
      encoding: 'hex'
    });

    cert.setExtensions([
      { name: 'basicConstraints', cA: false, critical: true },
      { name: 'keyUsage', digitalSignature: true, keyEncipherment: true, critical: true },
      { name: 'extKeyUsage', serverAuth: true, clientAuth: true },
      {
        name: 'subjectAltName',
        altNames: [
          { type: 2, value: domain },
          { type: 2, value: `*.${domain}` }
        ]
      },
      { name: 'subjectKeyIdentifier', keyIdentifier: forge.util.hexToBytes(leafSki) },
      { name: 'authorityKeyIdentifier', keyIdentifier: caSki }
    ]);

    cert.sign(this.caInfo.key, forge.md.sha256.create());

    const keyPem = forge.pki.privateKeyToPem(keyPair.privateKey);
    const certPem = forge.pki.certificateToPem(cert);

    console.log(`[CertManager] 域名证书已生成: ${domain}`);

    return {
      key: keyPem,
      cert: certPem,
      ca: this.caInfo.certPem
    };
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
    console.log('[CertManager] 证书缓存已清除');
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
      console.log('[CertManager] 已删除旧的 CA 证书文件');
    } catch (error) {
      console.warn('[CertManager] 删除旧 CA 文件失败:', error);
    }

    this.certCache.clear();
    this.generateCA();
  }
}

export default CertificateManager;
