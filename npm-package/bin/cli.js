#!/usr/bin/env node

const { execFileSync } = require("child_process");
const path = require("path");
const fs = require("fs");

function getBinaryName() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "linux" && arch === "x64") {
    return "zoriqa-linux-x64";
  }

  if (platform === "darwin" && arch === "x64") {
    return "zoriqa-macos-x64";
  }

  if (platform === "darwin" && arch === "arm64") {
    return "zoriqa-macos-arm64";
  }

  if (platform === "win32" && arch === "x64") {
    return "zoriqa-windows-x64.exe";
  }

  console.error(`Zoriqa is not available for ${platform}-${arch}.`);
  console.error("Supported platforms: linux-x64, macos-x64, macos-arm64, windows-x64");
  process.exit(1);
}

const binaryPath = path.join(__dirname, getBinaryName());

if (!fs.existsSync(binaryPath)) {
  console.error(`Zoriqa binary not found: ${binaryPath}`);
  console.error("The npm package is incomplete. Run the GitHub Actions release first to generate the binaries.");
  process.exit(1);
}

try {
  execFileSync(binaryPath, process.argv.slice(2), {
    stdio: "inherit"
  });
} catch (err) {
  if (typeof err.status === "number") {
    process.exit(err.status);
  }
  console.error(err.message);
  process.exit(1);
}