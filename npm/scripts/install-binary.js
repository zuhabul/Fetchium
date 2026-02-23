#!/usr/bin/env node

'use strict';

const { execSync } = require('child_process');
const fs = require('fs');
const https = require('https');
const os = require('os');
const path = require('path');
const { createWriteStream, mkdirSync } = require('fs');
const { pipeline } = require('stream/promises');
const zlib = require('zlib');
const { createGunzip } = require('zlib');

const REPO = 'hypersearchx/hypersearchx';
const BINARY_NAME = 'hsx';

/**
 * Map Node.js platform/arch to our release artifact names.
 */
function getArtifactName() {
  const platform = os.platform();
  const arch = os.arch();

  const map = {
    'darwin-x64': 'hsx-darwin-x64',
    'darwin-arm64': 'hsx-darwin-arm64',
    'linux-x64': 'hsx-linux-x64',
    'linux-arm64': 'hsx-linux-arm64',
    'win32-x64': 'hsx-win-x64',
  };

  const key = `${platform}-${arch}`;
  const artifact = map[key];

  if (!artifact) {
    console.error(
      `Unsupported platform: ${platform}-${arch}. ` +
        `Supported: ${Object.keys(map).join(', ')}`,
    );
    console.error('You can build from source: cargo install hypersearchx');
    process.exit(1);
  }

  return artifact;
}

/**
 * Get the version from package.json.
 */
function getVersion() {
  const pkg = JSON.parse(
    fs.readFileSync(path.join(__dirname, '..', 'package.json'), 'utf8'),
  );
  return pkg.version;
}

/**
 * Download a file from a URL, following redirects.
 */
function download(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { 'User-Agent': 'hypersearchx-npm' } }, (res) => {
        if (
          res.statusCode >= 300 &&
          res.statusCode < 400 &&
          res.headers.location
        ) {
          // Follow redirect
          return download(res.headers.location).then(resolve).catch(reject);
        }
        if (res.statusCode !== 200) {
          reject(
            new Error(`Download failed: HTTP ${res.statusCode} from ${url}`),
          );
          return;
        }
        resolve(res);
      })
      .on('error', reject);
  });
}

/**
 * Extract a tar.gz archive to a destination directory.
 */
async function extractTarGz(archivePath, destDir) {
  // Use tar command (available on macOS, Linux, and Git Bash on Windows)
  try {
    execSync(`tar xzf "${archivePath}" -C "${destDir}"`, { stdio: 'pipe' });
  } catch (e) {
    throw new Error(`Failed to extract archive: ${e.message}`);
  }
}

/**
 * Main installation logic.
 */
async function install() {
  const artifact = getArtifactName();
  const version = getVersion();
  const tag = `v${version}`;
  const isWindows = os.platform() === 'win32';
  const ext = isWindows ? 'zip' : 'tar.gz';

  const downloadUrl = `https://github.com/${REPO}/releases/download/${tag}/${artifact}.${ext}`;
  const binDir = path.join(__dirname, '..', 'bin');
  const tmpDir = path.join(os.tmpdir(), `hsx-install-${Date.now()}`);

  console.log(`HyperSearchX: Downloading ${artifact} (${tag})...`);
  console.log(`  URL: ${downloadUrl}`);

  mkdirSync(tmpDir, { recursive: true });
  mkdirSync(binDir, { recursive: true });

  const archivePath = path.join(tmpDir, `${artifact}.${ext}`);

  try {
    // Download the archive
    const res = await download(downloadUrl);
    const fileStream = createWriteStream(archivePath);
    await pipeline(res, fileStream);

    // Extract
    if (isWindows) {
      execSync(
        `powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${tmpDir}'"`,
        {
          stdio: 'pipe',
        },
      );
    } else {
      await extractTarGz(archivePath, tmpDir);
    }

    // Copy binary to bin/
    const binaryExt = isWindows ? '.exe' : '';
    const srcBinary = path.join(tmpDir, `${BINARY_NAME}${binaryExt}`);
    const destBinary = path.join(binDir, `${BINARY_NAME}${binaryExt}`);

    if (!fs.existsSync(srcBinary)) {
      throw new Error(`Binary not found in archive at: ${srcBinary}`);
    }

    fs.copyFileSync(srcBinary, destBinary);
    if (!isWindows) {
      fs.chmodSync(destBinary, 0o755);
    }

    console.log(`HyperSearchX: Installed ${BINARY_NAME} to ${destBinary}`);
  } catch (err) {
    console.error(`HyperSearchX: Failed to install binary.`);
    console.error(`  Error: ${err.message}`);
    console.error(``);
    console.error(`  You can install manually:`);
    console.error(`    cargo install hypersearchx`);
    console.error(`  Or download from:`);
    console.error(`    https://github.com/${REPO}/releases`);

    // Don't fail the npm install -- allow graceful degradation
    // The bin stubs will show a helpful error if the binary is missing
    process.exit(0);
  } finally {
    // Clean up temp directory
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch (_) {
      // ignore cleanup errors
    }
  }
}

install();
