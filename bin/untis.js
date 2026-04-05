#!/usr/bin/env node

const { spawn } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const binaryName = process.platform === "win32" ? "untis.exe" : "untis";
const binaryPath = path.join(__dirname, "..", "scripts", "native", binaryName);

if (!fs.existsSync(binaryPath)) {
  console.error(
    "The Untis binary is missing. Reinstall the package or run `npm rebuild -g untis`."
  );
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: "inherit"
});

child.on("error", (error) => {
  console.error(error.message);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }

  process.exit(code ?? 1);
});
