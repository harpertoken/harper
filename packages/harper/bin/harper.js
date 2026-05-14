#!/usr/bin/env node

const { spawn } = require("node:child_process");
const { existsSync } = require("node:fs");
const path = require("node:path");

const binaryName = process.platform === "win32" ? "harper.exe" : "harper";
const binaryPath = path.join(__dirname, "..", "vendor", binaryName);

if (!existsSync(binaryPath)) {
  console.error(
    "Harper binary is not installed. Reinstall the package or run npm rebuild.",
  );
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), { stdio: "inherit" });

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
