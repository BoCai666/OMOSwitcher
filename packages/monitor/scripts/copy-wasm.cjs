const fs = require('fs');
const path = require('path');

fs.mkdirSync('dist/wasm', { recursive: true });

// 尝试多个可能的位置
const possiblePaths = [
  'node_modules/sql.js/dist/sql-wasm.wasm',
  '../../node_modules/sql.js/dist/sql-wasm.wasm'
];

let copied = false;
for (const wasmPath of possiblePaths) {
  try {
    if (fs.existsSync(wasmPath)) {
      fs.cpSync(wasmPath, 'dist/wasm/sql-wasm.wasm');
      console.log('Copied sql-wasm.wasm to dist/wasm from ' + wasmPath);
      copied = true;
      break;
    }
  } catch (e) {
    // 继续尝试下一个路径
  }
}

if (!copied) {
  console.log('WASM file not found. Checked paths:');
  possiblePaths.forEach(p => console.log('  - ' + p));
}
