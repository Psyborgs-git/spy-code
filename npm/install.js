const fs = require('fs');
const path = require('path');
const os = require('os');
const https = require('https');

const pkg = require('./package.json');
const VERSION = pkg.version;
const REPO = 'Psyborgs-git/spy-code';

function getBinaryInfo() {
  const platform = os.platform();
  const arch = os.arch();

  let target = '';
  let exe = '';

  if (platform === 'darwin') {
    if (arch === 'arm64') {
      target = 'darwin-arm64';
    } else {
      target = 'darwin-x64';
    }
  } else if (platform === 'linux') {
    if (arch === 'x64') {
      target = 'linux-x64';
    } else {
      throw new Error(`Unsupported architecture for Linux: ${arch}`);
    }
  } else if (platform === 'win32') {
    if (arch === 'x64') {
      target = 'win32-x64';
      exe = '.exe';
    } else {
      throw new Error(`Unsupported architecture for Windows: ${arch}`);
    }
  } else {
    throw new Error(`Unsupported platform: ${platform}`);
  }

  const binaryName = `spy-code-${target}${exe}`;
  const localName = `spy-code${exe}`;
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${binaryName}`;

  return { url, localName };
}

function download(url, dest, callback) {
  const file = fs.createWriteStream(dest);
  
  const request = https.get(url, (response) => {
    // Handle redirect
    if (response.statusCode === 302 || response.statusCode === 301) {
      download(response.headers.location, dest, callback);
      return;
    }

    if (response.statusCode !== 200) {
      fs.unlink(dest, () => {}); // Delete file on failure
      callback(new Error(`Failed to download: Server returned status code ${response.statusCode}`));
      return;
    }

    response.pipe(file);

    file.on('finish', () => {
      file.close((err) => {
        if (err) {
          callback(err);
        } else {
          callback(null);
        }
      });
    });
  });

  request.on('error', (err) => {
    fs.unlink(dest, () => {});
    callback(err);
  });

  file.on('error', (err) => {
    fs.unlink(dest, () => {});
    callback(err);
  });
}

function main() {
  const binDir = path.join(__dirname, 'bin');
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  let info;
  try {
    info = getBinaryInfo();
  } catch (err) {
    console.error(err.message);
    process.exit(1);
  }

  const destPath = path.join(binDir, info.localName);

  // If a binary is already bundled (e.g. for offline use or local development builds), skip download
  if (fs.existsSync(destPath)) {
    console.log(`Bundled binary already exists at "${destPath}". Skipping download.`);
    // Make sure it is executable
    if (os.platform() !== 'win32') {
      try {
        fs.chmodSync(destPath, 0o755);
      } catch (err) {
        console.warn(`Failed to set executable permission: ${err.message}`);
      }
    }
    process.exit(0);
  }

  console.log(`Downloading spy-code binary v${VERSION} for platform ${os.platform()}-${os.arch()}...`);
  console.log(`URL: ${info.url}`);

  download(info.url, destPath, (err) => {
    if (err) {
      console.error(`Error downloading binary: ${err.message}`);
      console.warn('Postinstall download failed. The CLI wrapper will not function until the binary is provided.');
      console.warn('If you are doing a offline or manual install, place the compiled binary in "npm/bin/spy-code".');
      process.exit(0); // Exit gracefully so npm install doesn't crash completely
    }

    console.log(`Successfully downloaded binary to "${destPath}"`);
    
    // Set executable permissions on Unix platforms
    if (os.platform() !== 'win32') {
      try {
        fs.chmodSync(destPath, 0o755);
        console.log('Set executable permissions (+x).');
      } catch (chmodErr) {
        console.error(`Failed to set executable permissions: ${chmodErr.message}`);
      }
    }
  });
}

main();
