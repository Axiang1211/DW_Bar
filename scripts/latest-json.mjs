// Write out/latest.json for the app updater from the signed update files in `out/`.
// Each platform points at this version's release asset; no signatures → no manifest.
import fs from "node:fs";
import path from "node:path";

const dir = process.argv[2] || "out";
const conf = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
const version = conf.version;
const base = `https://github.com/Axiang1211/DW_Bar/releases/download/v${version}/`;
const files = {
  "windows-x86_64": "DW-Bar-Setup.exe",
  "windows-aarch64": "DW-Bar-Setup-arm64.exe",
  "darwin-aarch64": "DW-Bar.app.tar.gz",
  "darwin-x86_64": "DW-Bar.app.tar.gz",
};
const platforms = {};
for (const [platform, file] of Object.entries(files)) {
  const sig = path.join(dir, file + ".sig");
  if (fs.existsSync(sig)) platforms[platform] = { signature: fs.readFileSync(sig, "utf8").trim(), url: base + file };
}
if (!Object.keys(platforms).length) {
  console.log("No update signatures (signing key not set): skipping latest.json");
  process.exit(0);
}
const manifest = { version, notes: `DW Bar ${version}`, pub_date: new Date().toISOString(), platforms };
fs.writeFileSync(path.join(dir, "latest.json"), JSON.stringify(manifest, null, 2));
console.log(`latest.json: ${Object.keys(platforms).join(", ")}`);
