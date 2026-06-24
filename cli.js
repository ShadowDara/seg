#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const path = require("node:path");

const exe =
  process.platform === "win32"
    ? path.join(__dirname, "bin", "seg.exe")
    : path.join(__dirname, "bin", "seg");

const result = spawnSync(
  exe,
  process.argv.slice(2),
  {
    stdio: "inherit"
  }
);

process.exit(result.status ?? 1);
