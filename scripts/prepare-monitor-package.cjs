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

// 5. 复制 node_modules（从根目录提取，含所有传递依赖）
console.log('\n📦 收集并复制依赖...');
const nodeModulesDest = path.join(binariesDir, 'node_modules');
fs.mkdirSync(nodeModulesDest, { recursive: true });

// 递归收集所有传递依赖
const allDeps = new Set();
function collectTransitiveDeps(name) {
  if (allDeps.has(name)) return;
  allDeps.add(name);

  const pkgPath = path.join(rootNodeModules, name, 'package.json');
  if (!fs.existsSync(pkgPath)) return;

  try {
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
    const deps = { ...(pkg.dependencies || {}), ...(pkg.optionalDependencies || {}) };
    for (const dep of Object.keys(deps)) {
      collectTransitiveDeps(dep);
    }
  } catch (_e) {
    // 忽略无法解析的包
  }
}

for (const dep of monitorDeps) {
  collectTransitiveDeps(dep);
}

console.log(`  共收集 ${allDeps.size} 个传递依赖`);

let copiedCount = 0;
let missingCount = 0;
for (const dep of allDeps) {
  const depSrc = path.join(rootNodeModules, dep);
  const depDest = path.join(nodeModulesDest, dep);

  if (fs.existsSync(depSrc)) {
    // 创建父目录（处理 @scope/package 格式）
    const parentDir = path.dirname(depDest);
    if (!fs.existsSync(parentDir)) {
      fs.mkdirSync(parentDir, { recursive: true });
    }

    fs.cpSync(depSrc, depDest, { recursive: true });
    copiedCount++;
  } else {
    console.warn(`  ⚠ 依赖缺失: ${dep}`);
    missingCount++;
  }
}

console.log(`  ✓ 已复制 ${copiedCount} 个依赖`);
if (missingCount > 0) {
  console.warn(`  ⚠ ${missingCount} 个依赖在根 node_modules 中未找到`);
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

// 删除 dist 中的 monitor.exe（旧 SEA/打包残留，运行时使用内嵌 node.exe + index.js）
const monitorExe = path.join(distDest, 'monitor.exe');
if (fs.existsSync(monitorExe)) {
  fs.rmSync(monitorExe, { force: true });
  console.log('  - dist/monitor.exe (旧打包残留)');
}

// 7. 统计大小（rebuild 前）
const sizeBeforeRebuild = getDirSize(binariesDir);
console.log(`\n📊 rebuild 前大小: ${(sizeBeforeRebuild / 1024 / 1024).toFixed(2)} MB`);

// 8. 使用内嵌 Node 目标版本重建 better-sqlite3 原生模块
rebuildBetterSqliteForEmbeddedNode();

// 9. 精简 better-sqlite3 原生模块：删除编译中间产物，只保留运行时必需的 .node 文件
// 必须在 rebuild 之后执行，否则 rebuild 会重新生成中间产物
cleanBetterSqlite3BuildArtifacts(nodeModulesDest);

// 10. 校验关键运行时资源
verifyRequiredArtifacts();
console.log('✅ monitor-package 关键资源校验通过');

// 11. 最终统计
const totalSize = getDirSize(binariesDir);
const saved = sizeBeforeRebuild > 0 ? ((sizeBeforeRebuild - totalSize) / 1024 / 1024).toFixed(2) : '?';
console.log(`\n✅ 准备完成！最终大小: ${(totalSize / 1024 / 1024).toFixed(2)} MB（精简 ${saved} MB）`);

// ============ 辅助函数 ============

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

/**
 * 精简 better-sqlite3 原生模块：删除编译中间产物，只保留运行时必需的 .node 文件
 * 运行时只需 build/Release/better_sqlite3.node，其余都是编译/调试产物
 */
function cleanBetterSqlite3BuildArtifacts(nodeModulesDir) {
  const bs3Dir = path.join(nodeModulesDir, 'better-sqlite3');
  if (!fs.existsSync(bs3Dir)) return;

  console.log('\n🧹 精简 better-sqlite3 编译产物...');

  // 删除 deps/ 目录（SQLite 源码，编译完成不再需要）
  const depsDir = path.join(bs3Dir, 'deps');
  if (fs.existsSync(depsDir)) {
    fs.rmSync(depsDir, { recursive: true, force: true });
    console.log('  - better-sqlite3/deps/ (SQLite 源码)');
  }

  // 删除 src/ 目录（C++ 源码，编译完成不再需要）
  const srcDir = path.join(bs3Dir, 'src');
  if (fs.existsSync(srcDir)) {
    fs.rmSync(srcDir, { recursive: true, force: true });
    console.log('  - better-sqlite3/src/ (C++ 源码)');
  }

  // 精简 build/ 目录：删除编译中间产物，只保留 Release/better_sqlite3.node
  const buildDir = path.join(bs3Dir, 'build');
  if (fs.existsSync(buildDir)) {
    // 删除 build/deps/（编译中间产物）
    const buildDeps = path.join(buildDir, 'deps');
    if (fs.existsSync(buildDeps)) {
      fs.rmSync(buildDeps, { recursive: true, force: true });
      console.log('  - better-sqlite3/build/deps/ (编译中间产物)');
    }

    // 精简 build/Release/：只保留 .node 文件
    const releaseDir = path.join(buildDir, 'Release');
    if (fs.existsSync(releaseDir)) {
      const releaseItems = fs.readdirSync(releaseDir);
      for (const item of releaseItems) {
        const itemPath = path.join(releaseDir, item);
        const stat = fs.statSync(itemPath);

        if (stat.isDirectory()) {
          // 删除子目录（如 obj/）
          fs.rmSync(itemPath, { recursive: true, force: true });
          console.log(`  - better-sqlite3/build/Release/${item}/ (编译中间产物)`);
        } else if (!item.endsWith('.node')) {
          // 删除非 .node 文件（.pdb, .ilk, .lib, .exp 等）
          fs.rmSync(itemPath, { force: true });
          console.log(`  - better-sqlite3/build/Release/${item} (调试/链接产物)`);
        }
      }
    }

    // 删除 build 根目录下的非必要文件（.vcxproj, .sln 等）
    const buildItems = fs.readdirSync(buildDir);
    for (const item of buildItems) {
      if (item === 'Release') continue; // 保留 Release 目录
      const itemPath = path.join(buildDir, item);
      const stat = fs.statSync(itemPath);
      if (!stat.isDirectory()) {
        fs.rmSync(itemPath, { force: true });
        console.log(`  - better-sqlite3/build/${item} (构建配置文件)`);
      }
    }
  }
}
