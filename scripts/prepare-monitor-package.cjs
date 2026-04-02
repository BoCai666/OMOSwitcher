/**
 * 准备 monitor-package 目录
 * 将编译后的 monitor 服务复制到 Tauri binaries 目录
 * 
 * 用于 Tauri 打包时内嵌 Node.js 运行
 */

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const rootDir = path.join(__dirname, '..');
const monitorDir = path.join(rootDir, 'packages', 'monitor');
const rootNodeModules = path.join(rootDir, 'node_modules');
const binariesDir = path.join(rootDir, 'src-tauri', 'binaries', 'monitor-package');
const embeddedNodeExe = path.join(rootDir, 'src-tauri', 'binaries', 'node', 'node.exe');

// Monitor 的直接依赖（从 package.json 读取）
const monitorDeps = [
  '@mswjs/interceptors',
  'better-sqlite3',
  'cors',
  'express',
  'http-proxy',
  'node-forge',
  'uuid',
];

console.log('📦 准备 monitor-package...\n');
console.log(`源目录: ${monitorDir}`);
console.log(`目标目录: ${binariesDir}\n`);

// 1. 清理目标目录
if (fs.existsSync(binariesDir)) {
  console.log('🗑️  清理旧目录...');
  fs.rmSync(binariesDir, { recursive: true, force: true });
}

// 2. 创建目标目录
fs.mkdirSync(binariesDir, { recursive: true });

// 3. 复制 dist 目录
const distSrc = path.join(monitorDir, 'dist');
const distDest = path.join(binariesDir, 'dist');
if (fs.existsSync(distSrc)) {
  console.log('📁 复制编译产物: dist/...');
  fs.cpSync(distSrc, distDest, { recursive: true });
} else {
  console.error('❌ dist 目录不存在，请先运行 build');
  process.exit(1);
}

// 4. 复制配置文件
console.log('📄 复制配置文件...');
fs.copyFileSync(
  path.join(monitorDir, 'config.jsonc'),
  path.join(binariesDir, 'config.jsonc')
);
fs.copyFileSync(
  path.join(monitorDir, 'package.json'),
  path.join(binariesDir, 'package.json')
);

// 5. 复制 node_modules（从根目录提取）
console.log('\n📦 复制依赖...');
const nodeModulesDest = path.join(binariesDir, 'node_modules');
fs.mkdirSync(nodeModulesDest, { recursive: true });

let copiedCount = 0;
for (const dep of monitorDeps) {
  const depSrc = path.join(rootNodeModules, dep);
  const depDest = path.join(nodeModulesDest, dep);
  
  if (fs.existsSync(depSrc)) {
    // 创建父目录（处理 @scope/package 格式）
    const parentDir = path.dirname(depDest);
    if (!fs.existsSync(parentDir)) {
      fs.mkdirSync(parentDir, { recursive: true });
    }
    
    fs.cpSync(depSrc, depDest, { recursive: true });
    console.log(`  ✓ ${dep}`);
    copiedCount++;
    
    // 复制该依赖的嵌套 node_modules
    copyNestedDeps(depSrc, nodeModulesDest);
  } else {
    console.warn(`  ⚠ ${dep} 不存在`);
  }
}

// 补充：复制 better-sqlite3 需要的 build 目录（包含 .node 文件）
const betterSqliteBuild = path.join(rootNodeModules, 'better-sqlite3', 'build');
if (fs.existsSync(betterSqliteBuild)) {
  const buildDest = path.join(nodeModulesDest, 'better-sqlite3', 'build');
  if (!fs.existsSync(buildDest)) {
    fs.mkdirSync(buildDest, { recursive: true });
  }
  fs.cpSync(betterSqliteBuild, buildDest, { recursive: true });
  console.log('  ✓ better-sqlite3/build (原生模块)');
}

// 6. 清理不需要的文件
console.log('\n🧹 清理冗余文件...');

// 删除 @types 目录（类型定义）
const typesDir = path.join(nodeModulesDest, '@types');
if (fs.existsSync(typesDir)) {
  fs.rmSync(typesDir, { recursive: true, force: true });
  console.log('  - @types/');
}

// 删除 dist 中的 .d.ts 和 .map 文件
cleanTypeFiles(distDest);
console.log('  - .d.ts 和 .map 文件');

// 删除旧的 wasm 目录（如果存在）
const wasmDir = path.join(distDest, 'wasm');
if (fs.existsSync(wasmDir)) {
  fs.rmSync(wasmDir, { recursive: true, force: true });
  console.log('  - wasm/ (旧 sql.js)');
}

// 删除 node_modules 中的 .d.ts 文件
cleanTypeFilesInNodeModules(nodeModulesDest);

// 7. 统计大小
const totalSize = getDirSize(binariesDir);
console.log(`\n✅ 准备完成！总大小: ${(totalSize / 1024 / 1024).toFixed(2)} MB`);

// 8. 使用内嵌 Node 目标版本重建 better-sqlite3 原生模块
rebuildBetterSqliteForEmbeddedNode();

// 9. 校验关键运行时资源
verifyRequiredArtifacts();
console.log('✅ monitor-package 关键资源校验通过');

// ============ 辅助函数 ============

/**
 * 递归复制依赖的依赖
 */
function copyNestedDeps(depSrc, nodeModulesDest) {
  const nestedNodeModules = path.join(depSrc, 'node_modules');
  if (!fs.existsSync(nestedNodeModules)) return;
  
  const nestedDeps = fs.readdirSync(nestedNodeModules);
  for (const nestedDep of nestedDeps) {
    // 跳过 .bin 等特殊目录
    if (nestedDep.startsWith('.')) continue;
    
    const nestedSrc = path.join(nestedNodeModules, nestedDep);
    const nestedDest = path.join(nodeModulesDest, nestedDep);
    
    // 如果目标不存在，则复制
    if (!fs.existsSync(nestedDest)) {
      // 创建父目录（处理 @scope/package 格式）
      const parentDir = path.dirname(nestedDest);
      if (!fs.existsSync(parentDir)) {
        fs.mkdirSync(parentDir, { recursive: true });
      }
      
      fs.cpSync(nestedSrc, nestedDest, { recursive: true });
      
      // 递归处理
      copyNestedDeps(nestedSrc, nodeModulesDest);
    }
  }
}

/**
 * 递归删除 .d.ts 和 .map 文件
 */
function cleanTypeFiles(dir) {
  if (!fs.existsSync(dir)) return;
  
  const files = fs.readdirSync(dir);
  
  for (const file of files) {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    
    if (stat.isDirectory()) {
      cleanTypeFiles(filePath);
    } else if (file.endsWith('.d.ts') || file.endsWith('.d.ts.map') || file.endsWith('.js.map')) {
      fs.unlinkSync(filePath);
    }
  }
}

/**
 * 清理 node_modules 中的类型文件（保留 README 等）
 */
function cleanTypeFilesInNodeModules(dir) {
  if (!fs.existsSync(dir)) return;
  
  const files = fs.readdirSync(dir);
  
  for (const file of files) {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    
    if (stat.isDirectory()) {
      // 跳过 .bin 目录
      if (file === '.bin') continue;
      cleanTypeFilesInNodeModules(filePath);
    } else if (file.endsWith('.d.ts') || file.endsWith('.d.ts.map') || file.endsWith('.ts.map')) {
      fs.unlinkSync(filePath);
    }
  }
}

/**
 * 计算目录大小
 */
function getDirSize(dir) {
  if (!fs.existsSync(dir)) return 0;
  
  let size = 0;
  const files = fs.readdirSync(dir);
  
  for (const file of files) {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    
    if (stat.isDirectory()) {
      size += getDirSize(filePath);
    } else {
      size += stat.size;
    }
  }
  
  return size;
}

/**
 * 校验关键运行时资源是否已准备完成
 */
function verifyRequiredArtifacts() {
  const requiredPaths = [
    path.join(binariesDir, 'dist', 'index.js'),
    path.join(binariesDir, 'package.json'),
    path.join(binariesDir, 'config.jsonc'),
    path.join(binariesDir, 'node_modules'),
    path.join(binariesDir, 'node_modules', 'better-sqlite3', 'build'),
  ];

  const missing = requiredPaths.filter((targetPath) => !fs.existsSync(targetPath));
  if (missing.length > 0) {
    console.error('\n❌ monitor-package 缺少关键运行时资源：');
    for (const targetPath of missing) {
      console.error(`  - ${targetPath}`);
    }
    process.exit(1);
  }
}

/**
 * 针对内嵌 Node 版本重建 better-sqlite3 原生模块，避免 ABI 不匹配
 */
function rebuildBetterSqliteForEmbeddedNode() {
  if (!fs.existsSync(embeddedNodeExe)) {
    console.error(`\n❌ 内嵌 Node.js 不存在，无法重建 better-sqlite3: ${embeddedNodeExe}`);
    process.exit(1);
  }

  const versionResult = spawnSync(embeddedNodeExe, ['-p', 'process.versions.node'], {
    cwd: binariesDir,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  if (versionResult.status !== 0) {
    console.error('\n❌ 获取内嵌 Node.js 版本失败');
    process.stderr.write(versionResult.stderr || '');
    process.exit(versionResult.status || 1);
  }

  const embeddedNodeVersion = versionResult.stdout.trim();
  const npmCommand = process.platform === 'win32' ? 'npm.cmd' : 'npm';
  const rebuildArgs = [
    'rebuild',
    'better-sqlite3',
    '--foreground-scripts',
    `--target=${embeddedNodeVersion}`,
    '--runtime=node',
    '--dist-url=https://nodejs.org/download/release/',
    '--build-from-source',
  ];

  console.log(`\n🔧 使用内嵌 Node ${embeddedNodeVersion} 重建 better-sqlite3...`);

  const rebuildResult = process.platform === 'win32'
    ? spawnSync(
        'cmd.exe',
        ['/d', '/s', '/c', `${npmCommand} ${rebuildArgs.join(' ')}`],
        {
          cwd: binariesDir,
          stdio: 'inherit',
          env: {
            ...process.env,
            npm_config_target: embeddedNodeVersion,
            npm_config_runtime: 'node',
            npm_config_disturl: 'https://nodejs.org/download/release/',
            npm_config_build_from_source: 'true',
          },
        }
      )
    : spawnSync(npmCommand, rebuildArgs, {
        cwd: binariesDir,
        stdio: 'inherit',
        env: {
          ...process.env,
          npm_config_target: embeddedNodeVersion,
          npm_config_runtime: 'node',
          npm_config_disturl: 'https://nodejs.org/download/release/',
          npm_config_build_from_source: 'true',
        },
      });

  if (rebuildResult.status !== 0) {
    console.error('\n❌ better-sqlite3 重建失败');
    if (rebuildResult.error) {
      console.error(rebuildResult.error);
    }
    process.exit(rebuildResult.status || 1);
  }
}
