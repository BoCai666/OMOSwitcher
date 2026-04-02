const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const rootDir = path.join(__dirname, '..');
const binariesRoot = path.join(rootDir, 'src-tauri', 'binaries');
const nodeExe = path.join(binariesRoot, 'node', 'node.exe');
const monitorRoot = path.join(binariesRoot, 'monitor-package');

const requiredPaths = [
  { label: 'embedded Node runtime', path: nodeExe },
  { label: 'monitor package directory', path: monitorRoot },
  { label: 'monitor entrypoint', path: path.join(monitorRoot, 'dist', 'index.js') },
  { label: 'monitor package.json', path: path.join(monitorRoot, 'package.json') },
  { label: 'monitor config.jsonc', path: path.join(monitorRoot, 'config.jsonc') },
  { label: 'monitor node_modules', path: path.join(monitorRoot, 'node_modules') },
  { label: 'better-sqlite3 native build', path: path.join(monitorRoot, 'node_modules', 'better-sqlite3', 'build') },
];

const missing = requiredPaths.filter((entry) => !fs.existsSync(entry.path));

if (missing.length > 0) {
  console.error('❌ monitor-package 校验失败，缺少以下运行时资源：');
  for (const entry of missing) {
    console.error(`  - ${entry.label}: ${entry.path}`);
  }
  process.exit(1);
}

console.log('✅ monitor-package 校验通过');
for (const entry of requiredPaths) {
  console.log(`  - ${entry.label}: ${entry.path}`);
}

const runtimeProbe = spawnSync(
  nodeExe,
  [
    '-e',
    "const Database = require('better-sqlite3'); const db = new Database(':memory:'); db.pragma('journal_mode = WAL'); db.close(); console.log('BETTER_SQLITE_RUNTIME_OK');",
  ],
  {
    cwd: monitorRoot,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  }
);

if (runtimeProbe.status !== 0) {
  console.error('❌ monitor-package 运行时校验失败（better-sqlite3 无法在内嵌 Node 中加载）');
  if (runtimeProbe.stdout) {
    console.error(runtimeProbe.stdout.trim());
  }
  if (runtimeProbe.stderr) {
    console.error(runtimeProbe.stderr.trim());
  }
  process.exit(runtimeProbe.status || 1);
}

console.log(runtimeProbe.stdout.trim());
