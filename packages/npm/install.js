#!/usr/bin/env node
// postinstall.js — Downloads the correct hsx binary from GitHub Releases.
// Runs automatically after `npm install -g hypersearchx`.
"use strict";

const https = require("https");
const fs = require("fs");
const path = require("path");
const os = require("os");
const { execFileSync } = require("child_process");

const PKG = require("./package.json");
const VERSION = PKG.version;
const REPO = "zuhabul/HyperSearchX";
const BIN_DIR = path.join(__dirname, "bin");
const IS_WIN = process.platform === "win32";

// ── Platform detection ────────────────────────────────────────────────────────

function getArtifact() {
  const plat = process.platform;
  const arch = process.arch;
  const map = {
    "linux-x64":    { name: "hsx-linux-x64",    ext: ".tar.gz", bin: "hsx"     },
    "linux-arm64":  { name: "hsx-linux-arm64",   ext: ".tar.gz", bin: "hsx"     },
    "darwin-x64":   { name: "hsx-darwin-x64",    ext: ".tar.gz", bin: "hsx"     },
    "darwin-arm64": { name: "hsx-darwin-arm64",  ext: ".tar.gz", bin: "hsx"     },
    "win32-x64":    { name: "hsx-win-x64",       ext: ".zip",    bin: "hsx.exe" },
  };
  const key = `${plat}-${arch}`;
  const info = map[key];
  if (!info) {
    throw new Error(
      `Unsupported platform: ${key}\n` +
      `Supported: linux-x64, linux-arm64, darwin-x64, darwin-arm64, win32-x64\n` +
      `Build from source: https://github.com/${REPO}#build-from-source`
    );
  }
  return {
    filename: `${info.name}${info.ext}`,
    url: `https://github.com/${REPO}/releases/download/v${VERSION}/${info.name}${info.ext}`,
    binName: info.bin,
    isZip: info.ext === ".zip",
  };
}

// ── Download helper ───────────────────────────────────────────────────────────

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    let redirectCount = 0;

    function request(url) {
      if (++redirectCount > 5) return reject(new Error("Too many redirects"));
      https.get(url, { headers: { "User-Agent": `hypersearchx-npm-installer/${VERSION}` } }, (res) => {
        if ([301, 302, 307, 308].includes(res.statusCode)) {
          return request(res.headers.location);
        }
        if (res.statusCode !== 200) {
          file.destroy();
          return reject(new Error(`HTTP ${res.statusCode} downloading from:\n  ${url}`));
        }
        let downloaded = 0;
        res.on("data", (chunk) => {
          downloaded += chunk.length;
          process.stdout.write(`\r  ${(downloaded / 1024 / 1024).toFixed(1)} MB`);
        });
        res.pipe(file);
        file.on("finish", () => { process.stdout.write("\n"); file.close(resolve); });
        file.on("error", reject);
      }).on("error", reject);
    }

    request(url);
  });
}

// ── Extract helper ────────────────────────────────────────────────────────────

function extract(archive, destDir, isZip) {
  if (isZip) {
    // Windows: use PowerShell's built-in Expand-Archive
    execFileSync("powershell.exe", [
      "-NoProfile", "-NonInteractive", "-Command",
      `Expand-Archive -Force '${archive}' '${destDir}'`,
    ], { stdio: "pipe" });
  } else {
    execFileSync("tar", ["-xzf", archive, "-C", destDir], { stdio: "pipe" });
  }
}

// ── Main ──────────────────────────────────────────────────────────────────────

async function main() {
  let artifact;
  try {
    artifact = getArtifact();
  } catch (err) {
    console.warn(`\n⚠  ${err.message}\n`);
    console.warn("Skipping binary download. You can still install via:");
    console.warn("  curl -sSf https://install.hypersearchx.zuhabul.com | sh");
    return; // Don't fail npm install
  }

  const binPath = path.join(BIN_DIR, artifact.binName);

  // Skip if already installed and version matches
  if (fs.existsSync(binPath)) {
    try {
      const out = execFileSync(binPath, ["--version"], { encoding: "utf8", timeout: 5000 }).trim();
      if (out.includes(VERSION)) {
        console.log(`✓ hsx ${VERSION} already installed`);
        return;
      }
    } catch {}
  }

  if (!fs.existsSync(BIN_DIR)) {
    fs.mkdirSync(BIN_DIR, { recursive: true });
  }

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "hsx-"));
  const tmpArchive = path.join(tmpDir, artifact.filename);

  console.log(`\nDownloading hsx v${VERSION} (${process.platform}/${process.arch})`);
  console.log(`  ${artifact.url}`);

  try {
    await download(artifact.url, tmpArchive);
  } catch (err) {
    console.warn(`\n⚠  Download failed: ${err.message}`);
    console.warn("\nAlternative installation methods:");
    console.warn("  Shell:   curl -sSf https://install.hypersearchx.zuhabul.com | sh");
    console.warn("  Brew:    brew install zuhabul/tap/hsx");
    console.warn("  Binstall: cargo binstall hsx");
    // Don't fail npm install — the CLI wrapper will print a helpful error
    return;
  }

  try {
    process.stdout.write("  Extracting...");
    extract(tmpArchive, tmpDir, artifact.isZip);
    const extracted = path.join(tmpDir, artifact.binName);
    fs.copyFileSync(extracted, binPath);
    if (!IS_WIN) fs.chmodSync(binPath, 0o755);
    process.stdout.write(" done\n");
  } catch (err) {
    console.warn(`\n⚠  Extraction failed: ${err.message}`);
    return;
  } finally {
    try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch {}
  }

  // Verify
  try {
    const ver = execFileSync(binPath, ["--version"], { encoding: "utf8", timeout: 5000 }).trim();
    console.log(`\n✓ ${ver} installed`);
  } catch {
    console.log(`\n✓ hsx v${VERSION} installed`);
  }
  console.log(`  Run: hsx --help`);
  console.log(`  Docs: https://hypersearchx.zuhabul.com/docs\n`);
}

main().catch((err) => {
  // Swallow errors so npm install never fails because of this postinstall
  console.warn(`\n⚠  hsx postinstall warning: ${err.message}`);
  console.warn("  You can install manually: https://hypersearchx.zuhabul.com/docs/self-hosting\n");
  process.exitCode = 0;
});
