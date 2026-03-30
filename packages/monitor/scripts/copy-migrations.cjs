const fs = require('fs');
const path = require('path');

fs.mkdirSync('dist/db/migrations', { recursive: true });
fs.cpSync('src/db/migrations', 'dist/db/migrations', { recursive: true });
console.log('Copied migrations to dist/db/migrations');
