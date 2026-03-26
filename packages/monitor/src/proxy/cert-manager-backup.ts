import { homedir } from 'os';
import { join } from 'path';
import { existsSync, mkdirSync, readFileSync, writeFileSync, unlinkSync } from 'fs';
import forge from 'node-forge';
import { randomBytes, createHash } from 'crypto';

const CERT_DIR = join(homedir(), '.opencode-monitor', 'certs');
const CA_KEY_FILE = join(CERT_DIR, 'ca.key');
const CA_CERT_FILE = join(CERT_DIR, 'ca.crt');

export interface CertificatePair {
  key: string;
  cert: string;
  ca: string;  // 包含 CA 证书以形成完整链
}

interface CAInfo {
  key: forge.pki.rsa.PrivateKey;
  cert: forge.pki.Certificate;
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
        const key = forge.pki.privateKeyFromPem(keyPem);
        const cert = forge.pki.certificateFromPem(certPem);
        
        debugLog('加载的 CA 证书序列号', cert.serialNumber);
        debugLog('加载的 CA 证书主题', cert.subject.attributes.map((a: any) => `${a.name}=${a.value}`).join(', '));
        
        const now = new Date();
        if (cert.validity.notAfter < now) {
          console.log('CA certificate expired, regenerating...');
          this.generateCA();
          return;
        }

        // 验证私钥和公钥是否匹配
        const publicKeyFromCert = cert.publicKey as forge.pki.rsa.PublicKey;
        const testData = 'test';
        try {
          const signature = key.sign(forge.md.sha256.create().update(testData));
          const verified = publicKeyFromCert.verify(
            forge.md.sha256.create().update(testData).digest().getBytes(),
            signature
          );
          debugLog('CA 私钥/公钥匹配验证', verified ? '成功' : '失败');
          if (!verified) {
            console.warn('CA 私钥和证书中的公钥不匹配，重新生成 CA');
            this.generateCA();
            return;
          }
        } catch (e) {
          console.warn('CA 密钥验证失败，重新生成 CA:', e);
          this.generateCA();
          return;
        }

        this.caInfo = { key, cert, keyPem, certPem };
        console.log('Loaded existing CA certificate');
        return;
      } catch (error) {
        console.warn('Failed to load CA certificate, regenerating:', error);
      }
    }
    this.generateCA();
  }

  private generateCA(): void {
    console.log('Generating new CA root certificate...');

    const keyPair = forge.pki.rsa.generateKeyPair({ bits: 2048 });
    const cert = forge.pki.createCertificate();
    cert.publicKey = keyPair.publicKey;
    cert.serialNumber = '00' + randomBytes(16).toString('hex');

    debugLog('新生成的 CA 序列号', cert.serialNumber);

    const now = new Date();
    cert.validity.notBefore = now;
    cert.validity.notAfter = new Date(now.getTime() + 10 * 365 * 24 * 60 * 60 * 1000);

    cert.setSubject([
      { name: 'commonName', value: 'OpenCode Monitor CA' },
      { name: 'organizationName', value: 'OpenCode Monitor' },
      { name: 'countryName', value: 'CN' }
    ]);

    cert.setIssuer([
      { name: 'commonName', value: 'OpenCode Monitor CA' },
      { name: 'organizationName', value: 'OpenCode Monitor' },
      { name: 'countryName', value: 'CN' }
    ]);

    // 显式计算 Subject Key Identifier
    const subjectKeyIdentifier = forge.pki.getPublicKeyFingerprint(keyPair.publicKey, {
      type: 'SubjectPublicKeyInfo',
      encoding: 'hex'
    });

    // 转换为 bytes 格式用于 node-forge
    const subjectKeyIdentifierBytes = forge.util.hexToBytes(subjectKeyIdentifier);
    debugLog('CA SubjectKeyIdentifier (hex)', subjectKeyIdentifier);
    debugLog('CA SubjectKeyIdentifier 长度', subjectKeyIdentifierBytes.length);

    // Add CA extensions
    cert.setExtensions([
      {
        name: 'basicConstraints',
        cA: true,
        critical: true
      },
      {
        name: 'keyUsage',
        keyCertSign: true,
        cRLSign: true,
        critical: true
      },
      {
        name: 'subjectKeyIdentifier',
        keyIdentifier: subjectKeyIdentifierBytes
      }
    ]);

    // 签名前记录信息
    debugLog('签名前 - 颁发者', cert.issuer.attributes.map((a: any) => `${a.name}=${a.value}`).join(', '));
    debugLog('签名前 - 主题', cert.subject.attributes.map((a: any) => `${a.name}=${a.value}`).join(', '));
    debugLog('签名前 - 公钥算法', (cert.publicKey as any).algorithm || 'unknown');

    cert.sign(keyPair.privateKey, forge.md.sha256.create());

    // 签名后验证
    try {
      const verified = cert.verify(cert);
      debugLog('CA 自签名验证', verified ? '成功' : '失败');
    } catch (e) {
      debugLog('CA 自签名验证错误', e);
    }

    const keyPem = forge.pki.privateKeyToPem(keyPair.privateKey);
    const certPem = forge.pki.certificateToPem(cert);

    this.caInfo = { key: keyPair.privateKey, cert, keyPem, certPem };

    writeFileSync(CA_KEY_FILE, keyPem, { mode: 0o600 });
    writeFileSync(CA_CERT_FILE, certPem, { mode: 0o644 });

    console.log(`CA certificate saved to: ${CA_CERT_FILE}`);
  }

  private generateDomainCertificate(domain: string): CertificatePair {
    if (!this.caInfo) {
      throw new Error('CA certificate not initialized');
    }

    console.log(`Generating certificate for domain: ${domain}`);

    const keyPair = forge.pki.rsa.generateKeyPair({ bits: 2048 });
    const cert = forge.pki.createCertificate();
    cert.publicKey = keyPair.publicKey;
    cert.serialNumber = '00' + randomBytes(16).toString('hex');

    debugLog(`叶子证书(${domain}) 序列号`, cert.serialNumber);

    const now = new Date();
    cert.validity.notBefore = now;
    cert.validity.notAfter = new Date(now.getTime() + 365 * 24 * 60 * 60 * 1000);

    cert.setSubject([
      { name: 'commonName', value: domain },
      { name: 'organizationName', value: 'OpenCode Monitor' },
      { name: 'countryName', value: 'CN' }
    ]);

    cert.setIssuer(this.caInfo.cert.subject.attributes);

    debugLog(`叶子证书(${domain}) 颁发者`, cert.issuer.attributes.map((a: any) => `${a.name}=${a.value}`).join(', '));
    debugLog(`叶子证书(${domain}) 主题`, cert.subject.attributes.map((a: any) => `${a.name}=${a.value}`).join(', '));

    // 显式计算叶子证书的 Subject Key Identifier
    const leafSKIHex = forge.pki.getPublicKeyFingerprint(keyPair.publicKey, {
      type: 'SubjectPublicKeyInfo',
      encoding: 'hex'
    });
    const leafSubjectKeyIdentifier = forge.util.hexToBytes(leafSKIHex);

    // 获取 CA 证书的 Subject Key Identifier
    const caSKIExt = this.caInfo.cert.getExtension('subjectKeyIdentifier') as { 
      keyIdentifier?: string | Uint8Array;
      id?: string | Uint8Array;
    } | null;
    debugLog(`CA SKI 扩展`, caSKIExt ? '存在' : '不存在');
    debugLog(`CA SKI 扩展详情`, caSKIExt);
    
    // node-forge 可能以不同格式存储 keyIdentifier
    let caKeyIdentifier: string | Uint8Array | undefined = caSKIExt?.keyIdentifier || caSKIExt?.id;
    
    if (!caKeyIdentifier) {
      throw new Error('CA certificate missing subjectKeyIdentifier extension');
    }
    
    // 转换为 bytes 格式
    let caKeyIdentifierBytes: string;
    if (typeof caKeyIdentifier === 'string') {
      caKeyIdentifierBytes = caKeyIdentifier;
    } else if (caKeyIdentifier instanceof Uint8Array) {
      // 如果是 Uint8Array，转换为 forge bytes
      caKeyIdentifierBytes = String.fromCharCode(...caKeyIdentifier);
    } else {
      caKeyIdentifierBytes = String(caKeyIdentifier);
    }
    
    debugLog(`CA SKI 长度`, caKeyIdentifierBytes.length);
    debugLog(`CA SKI (hex)`, forge.util.bytesToHex(caKeyIdentifierBytes));

    // Add extensions - required for modern HTTPS clients
    cert.setExtensions([
      {
        name: 'basicConstraints',
        cA: false,
        critical: true
      },
      {
        name: 'keyUsage',
        digitalSignature: true,
        keyEncipherment: true,
        critical: true
      },
      {
        name: 'extKeyUsage',
        serverAuth: true,
        clientAuth: true
      },
      {
        name: 'subjectAltName',
        altNames: [
          { type: 2, value: domain },  // DNS name
          { type: 2, value: `*.${domain}` }  // Wildcard
        ]
      },
      {
        name: 'subjectKeyIdentifier',
        keyIdentifier: leafSubjectKeyIdentifier
      },
      {
        name: 'authorityKeyIdentifier',
        keyIdentifier: caKeyIdentifierBytes,
        authorityCertIssuer: true,
        authoritySerialNumber: true
      }
    ]);

    // 签名前保存公钥指纹用于验证
    const publicKeyBeforeSign = forge.pki.getPublicKeyFingerprint(keyPair.publicKey, {
      type: 'SubjectPublicKeyInfo',
      encoding: 'hex'
    });
    debugLog(`叶子证书(${domain}) 签名前公钥指纹`, publicKeyBeforeSign);
    
    // Sign certificate with CA private key
    cert.sign(this.caInfo.key, forge.md.sha256.create());
    
    // 签名后验证公钥是否改变
    const publicKeyAfterSign = forge.pki.getPublicKeyFingerprint(cert.publicKey, {
      type: 'SubjectPublicKeyInfo',
      encoding: 'hex'
    });
    debugLog(`叶子证书(${domain}) 签名后公钥指纹`, publicKeyAfterSign);
    debugLog(`叶子证书(${domain}) 公钥是否改变`, publicKeyBeforeSign !== publicKeyAfterSign ? '❌ 是' : '✅ 否');

    // 签名后验证证书链
    try {
      // 验证叶子证书的颁发者是否与 CA 主题匹配
      const issuerMatches = JSON.stringify(cert.issuer.attributes) === JSON.stringify(this.caInfo.cert.subject.attributes);
      debugLog(`叶子证书(${domain}) 颁发者匹配`, issuerMatches);

      // 尝试使用 CA 公钥验证签名
      const certSignature = cert.signature;
      const certTBS = cert.tbsCertificate;
      
      // 注意：forge 的验证可能需要不同的方式
      // 这里我们只检查基本信息
      debugLog(`叶子证书(${domain}) 签名算法`, (cert as any).signatureAlgorithm || 'unknown');
      debugLog(`叶子证书(${domain}) 签名长度`, certSignature ? certSignature.length : 0);
      
    } catch (e) {
      debugLog(`叶子证书(${domain}) 签名验证错误`, e);
    }

    // 导出 PEM 格式
    const keyPem = forge.pki.privateKeyToPem(keyPair.privateKey);
    const certPem = forge.pki.certificateToPem(cert);

    // 验证私钥和证书匹配（使用原始对象）
    try {
      const testData = 'test';
      const md = forge.md.sha256.create();
      md.update(testData);
      const signature = keyPair.privateKey.sign(md);
      
      const certPublicKey = cert.publicKey as forge.pki.rsa.PublicKey;
      const verified = certPublicKey.verify(md.digest().getBytes(), signature);
      
      debugLog(`叶子证书(${domain}) 私钥/证书匹配验证（原始对象）`, verified ? '✅ 通过' : '❌ 失败');
    } catch (e) {
      debugLog(`叶子证书(${domain}) 私钥/证书验证错误（原始对象）`, e);
    }
    
    // 验证 PEM 可以正确解析且私钥/证书仍然匹配
    try {
      const parsedKey = forge.pki.privateKeyFromPem(keyPem);
      const parsedCert = forge.pki.certificateFromPem(certPem);
      
      const testData = 'test-pem';
      const md = forge.md.sha256.create();
      md.update(testData);
      const signature = parsedKey.sign(md);
      
      const verified = (parsedCert.publicKey as forge.pki.rsa.PublicKey).verify(md.digest().getBytes(), signature);
      
      debugLog(`叶子证书(${domain}) 私钥/证书匹配验证（PEM解析后）`, verified ? '✅ 通过' : '❌ 失败');
      
      if (!verified) {
        console.error(`❌ 叶子证书(${domain}) PEM 导出后私钥和证书不匹配！`);
        console.error(`   这可能是 node-forge PEM 导出的问题`);
      }
    } catch (e) {
      debugLog(`叶子证书(${domain}) PEM 解析验证错误`, e);
    }

    console.log(`Certificate generated for: ${domain}`);
    
    // 返回包含完整证书链的证书对
    return { 
      key: keyPem, 
      cert: certPem,
      ca: this.caInfo.certPem  // 包含 CA 证书形成完整链
    };
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
    // 删除现有的 CA 证书和私钥
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

    // 清除证书缓存
    this.certCache.clear();

    // 重新生成 CA
    this.generateCA();
  }

  // 诊断方法：验证证书链
  public verifyCertificateChain(domain: string): boolean {
    if (!this.caInfo) {
      console.error('CA not initialized');
      return false;
    }

    try {
      const certPair = this.getCertificateForDomain(domain);
      const leafCert = forge.pki.certificateFromPem(certPair.cert);
      
      debugLog(`验证证书链 - 叶子证书主题`, leafCert.subject.getField('CN')?.value);
      debugLog(`验证证书链 - 叶子证书颁发者`, leafCert.issuer.getField('CN')?.value);
      debugLog(`验证证书链 - CA 主题`, this.caInfo.cert.subject.getField('CN')?.value);
      
      // 检查颁发者是否匹配
      const issuerMatches = leafCert.issuer.attributes.every((attr: any) => {
        const caAttr = this.caInfo!.cert.subject.getField(attr.name);
        return caAttr && caAttr.value === attr.value;
      });
      
      debugLog(`验证证书链 - 颁发者匹配`, issuerMatches);
      
      // 尝试验证签名
      try {
        const verified = leafCert.verify(this.caInfo.cert);
        debugLog(`验证证书链 - 签名验证`, verified);
        return verified;
      } catch (e) {
        debugLog(`验证证书链 - 签名验证错误`, e);
        return false;
      }
    } catch (e) {
      debugLog(`验证证书链 - 错误`, e);
      return false;
    }
  }
}

export default CertificateManager;
