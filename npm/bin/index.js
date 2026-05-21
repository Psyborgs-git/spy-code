#!/usr/bin/env node

const { spawnSync } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');

const binaryName = os.platform() === 'win32' ? 'spy-code.exe' : 'spy-code';
const binaryPath = path.join(__dirname, binaryName);

// Check if native binary exists. If not, inform the user they need to wait for installation or run install manually.
if (!fs.existsSync(binaryPath)) {
  console.error(`Error: spy-code native binary not found at "${binaryPath}".`);
  console.error(`If you just installed the package, the download might have failed.`);
  console.error(`You can attempt to trigger the download again by running 'npm rebuild spy-code' or running install.js manually.`);
  process.exit(1);
}

// Forward all command line arguments to the native binary
const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  shell: false
});

// Exit with the same code as the native binary
if (result.error) {
  console.error('Error running spy-code binary:', result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 0);
