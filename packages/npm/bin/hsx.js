#!/usr/bin/env node
// Thin launcher: finds the downloaded hsx binary and execs it with all args.
"use strict";

const path = require("path");
const fs = require("fs");
const { spawnSync } = require("child_process");

const IS_WIN = process.platform === "win32";
const BIN_DIR = path.join(__dirname);
const BIN_NAME = IS_WIN ? "hsx.exe" : "hsx";
const BIN_PATH = path.join(BIN_DIR, BIN_NAME);

if (!fs.existsSync(BIN_PATH)) {
  console.error(
    `\n⚠  hsx binary not found at: ${BIN_PATH}\n\n` +
    `This usually means the postinstall script failed. Try:\n\n` +
    `  npm install -g hypersearchx\n\n` +
    `Or install via shell:\n` +
    `  curl -sSf https://install.hypersearchx.zuhabul.com | sh\n`
  );
  process.exit(1);
}

const { status, error } = spawnSync(BIN_PATH, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});

if (error) {
  console.error(`Failed to run hsx: ${error.message}`);
  process.exit(1);
}

process.exit(status ?? 0);
