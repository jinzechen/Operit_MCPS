const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const BIN_PATH = path.join(__dirname, '__BIN__');
const SERVER_NAME = '__NAME__';

try {
  fs.accessSync(BIN_PATH, fs.constants.X_OK);
} catch (_) {
  fs.chmodSync(BIN_PATH, 0o755);
  console.error(`[${SERVER_NAME}] Permissions fixed.`);
}

const child = spawn(BIN_PATH, [], {
  stdio: ['pipe', 'pipe', 'pipe'],
  env: process.env
});

process.stdin.pipe(child.stdin);
child.stdout.pipe(process.stdout);
child.stderr.pipe(process.stderr);
child.on('exit', code => process.exit(code || 0));
