#!/usr/bin/env node

const { createWriteStream, chmodSync, copyFileSync, existsSync, mkdirSync, readdirSync, rmSync } = require("node:fs");
const https = require("node:https");
const { basename, join } = require("node:path");
const { spawnSync } = require("node:child_process");
const packageJson = require("../package.json");

const ROOT = join(__dirname, "..");
const DOWNLOAD_DIR = join(ROOT, ".download");
const VENDOR_DIR = join(ROOT, "vendor");
const RELEASE_TAG = process.env.HARPER_NPM_RELEASE_TAG || `harper-${packageJson.version}`;

const TARGETS = {
  "darwin:arm64": "harper-macos-aarch64.tar.gz",
  "darwin:x64": "harper-macos-x86_64.tar.gz",
  "linux:arm64": "harper-linux-aarch64.tar.gz",
  "linux:x64": "harper-linux-x86_64.tar.gz",
  "win32:x64": "harper-windows-x86_64.zip",
};

function fail(message) {
  console.error(message);
  process.exit(1);
}

function targetAsset() {
  const asset = TARGETS[`${process.platform}:${process.arch}`];
  if (!asset) {
    fail(`Unsupported platform for Harper npm package: ${process.platform}/${process.arch}`);
  }

  return asset;
}

function download(url, destination, redirects = 0) {
  return new Promise((resolve, reject) => {
    https.get(url, (response) => {
      if ([301, 302, 303, 307, 308].includes(response.statusCode)) {
        if (redirects >= 5) {
          reject(new Error("Too many redirects while downloading Harper"));
          return;
        }

        download(response.headers.location, destination, redirects + 1).then(resolve, reject);
        return;
      }

      if (response.statusCode !== 200) {
        reject(new Error(`Download failed with HTTP ${response.statusCode}`));
        return;
      }

      const file = createWriteStream(destination);
      response.pipe(file);
      file.on("finish", () => file.close(resolve));
      file.on("error", reject);
    }).on("error", reject);
  });
}

function extract(archivePath) {
  rmSync(VENDOR_DIR, { recursive: true, force: true });
  mkdirSync(VENDOR_DIR, { recursive: true });

  if (archivePath.endsWith(".zip")) {
    const result = spawnSync(
      "powershell",
      [
        "-NoProfile",
        "-Command",
        "Expand-Archive -LiteralPath $args[0] -DestinationPath $args[1] -Force",
        archivePath,
        VENDOR_DIR,
      ],
      { stdio: "inherit" },
    );
    if (result.status !== 0) {
      fail("Failed to extract Harper Windows archive");
    }
    return;
  }

  const result = spawnSync("tar", ["-xzf", archivePath, "-C", VENDOR_DIR], { stdio: "inherit" });
  if (result.status !== 0) {
    fail("Failed to extract Harper archive");
  }
}

function findBinary(directory) {
  const binaryName = process.platform === "win32" ? "harper.exe" : "harper";
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const currentPath = join(directory, entry.name);
    if (entry.isFile() && entry.name === binaryName) {
      return currentPath;
    }
    if (entry.isDirectory()) {
      const found = findBinary(currentPath);
      if (found) {
        return found;
      }
    }
  }

  return undefined;
}

async function main() {
  if (process.env.HARPER_NPM_SKIP_DOWNLOAD === "1") {
    mkdirSync(VENDOR_DIR, { recursive: true });
    return;
  }

  const asset = targetAsset();
  const archivePath = join(DOWNLOAD_DIR, basename(asset));
  const url = `https://github.com/harpertoken/harper/releases/download/${RELEASE_TAG}/${asset}`;

  mkdirSync(DOWNLOAD_DIR, { recursive: true });
  await download(url, archivePath);
  extract(archivePath);

  const binaryPath = findBinary(VENDOR_DIR);
  if (!binaryPath) {
    fail("Harper binary was not found in the release archive");
  }

  const finalPath = join(VENDOR_DIR, process.platform === "win32" ? "harper.exe" : "harper");
  if (binaryPath !== finalPath) {
    copyFileSync(binaryPath, finalPath);
  }
  chmodSync(finalPath, 0o755);
  rmSync(DOWNLOAD_DIR, { recursive: true, force: true });
}

main().catch((error) => fail(error.message));
