#!/usr/bin/env node
"use strict";
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const https = require("https");

const PLATFORM_MAP = {
  "linux-x64": "x86_64-unknown-linux-gnu",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
  "win32-x64": "x86_64-pc-windows-msvc",
};

const platform = `${process.platform}-${process.arch}`;
const target = PLATFORM_MAP[platform];

if (!target) {
  console.error(`Unsupported platform: ${platform}. Supported: ${Object.keys(PLATFORM_MAP).join(", ")}`);
  process.exit(1);
}

const pkg = require("../package.json");
const version = pkg.version;
const isWindows = process.platform === "win32";
const ext = isWindows ? "zip" : "tar.gz";
const binName = isWindows ? "hsx.exe" : "hsx";
const repoUrl = "https://github.com/hypersearchx/hypersearchx";
const archiveName = `hypersearchx-${target}.${ext}`;
const downloadUrl = `${repoUrl}/releases/download/v${version}/${archiveName}`;

const binDir = path.join(__dirname, "..", "bin");
fs.mkdirSync(binDir, { recursive: true });

console.log(`Downloading HyperSearchX v${version} for ${platform}...`);
console.log(`URL: ${downloadUrl}`);

try {
  if (ext === "tar.gz") {
    execSync(`curl -fsSL "${downloadUrl}" | tar xz -C "${binDir}"`, { stdio: "inherit" });
  } else {
    const zipPath = path.join(binDir, archiveName);
    execSync(`curl -fsSL -o "${zipPath}" "${downloadUrl}"`, { stdio: "inherit" });
    execSync(`unzip -o "${zipPath}" -d "${binDir}"`, { stdio: "inherit" });
    fs.unlinkSync(zipPath);
  }
  const binPath = path.join(binDir, binName);
  fs.chmodSync(binPath, 0o755);
  console.log(`HyperSearchX installed at ${binPath}`);
  console.log(`Run: hsx --version`);
} catch (err) {
  console.error(`Installation failed: ${err.message}`);
  console.error(`Try installing manually from: ${repoUrl}/releases`);
  process.exit(1);
}
