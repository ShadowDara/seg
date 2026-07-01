#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const path = require("node:path");
const fs = require("node:fs");

const exe =
  process.platform === "win32"
    ? path.join(__dirname, "bin", "seg.exe")
    : path.join(__dirname, "bin", "seg");

if (!fs.existsSync(exe)) {
  console.error(
    `Error: Binary not found: ${exe}! Did you run the install script?`,
  );
  process.exit(1);
}

const result = spawnSync(exe, process.argv.slice(2), {
  stdio: "inherit",
});

process.exit(result.status ?? 1);
