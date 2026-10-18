#!/usr/bin/env node
/**
 * Build latest.json for Tauri updater (GitHub Releases static manifest).
 * Usage: node scripts/generate-latest-json.mjs <version> <tag> <assetsDir>
 * Example: node scripts/generate-latest-json.mjs 0.1.0 v0.1.0 ./release-assets
 */
import fs from "node:fs";
import path from "node:path";

const version = process.argv[2];
const tag = process.argv[3];
const assetsDir = process.argv[4] || "./release-assets";
const repo = process.env.GITHUB_REPOSITORY || "alfodri/HR-APP";

if (!version || !tag) {
  console.error("Usage: generate-latest-json.mjs <version> <tag> [assetsDir]");
  process.exit(1);
}

const platforms = {};
const files = fs.readdirSync(assetsDir);

for (const file of files) {
  if (!file.endsWith(".sig")) continue;
  const bundle = file.slice(0, -4);
  const sigPath = path.join(assetsDir, file);
  const signature = fs.readFileSync(sigPath, "utf8").trim();
  const url = `https://github.com/${repo}/releases/download/${tag}/${encodeURIComponent(bundle)}`;

  let key = null;
  if (bundle.endsWith(".AppImage")) {
    key = "linux-x86_64";
  } else if (bundle.endsWith("-setup.exe") || bundle.endsWith(".exe")) {
    key = "windows-x86_64";
  }

  if (key && !platforms[key]) {
    platforms[key] = { url, signature };
  }
}

if (!platforms["linux-x86_64"] && !platforms["windows-x86_64"]) {
  console.error("No signed AppImage or NSIS .exe found in", assetsDir);
  console.error("Files:", files.join(", "));
  process.exit(1);
}

const manifest = {
  version,
  notes: `HR App ${version}`,
  pub_date: new Date().toISOString(),
  platforms,
};

const out = path.join(assetsDir, "latest.json");
fs.writeFileSync(out, JSON.stringify(manifest, null, 2));
console.log("Wrote", out);
console.log(JSON.stringify(manifest, null, 2));
