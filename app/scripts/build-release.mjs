import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const appDir = join(dirname(fileURLToPath(import.meta.url)), "..");
const tauriDir = join(appDir, "src-tauri");
const keyPath = process.env.LOUNGE_SIGNING_KEY ?? join(homedir(), ".tauri", "lounge.key");
const downloadBase = process.env.LOUNGE_DOWNLOAD_BASE ?? "https://github.com/tguillain/lounge/releases/download";

if (!existsSync(keyPath)) {
  console.error(`No signing key at ${keyPath}. Updates must be signed with the key whose public half is in tauri.conf.json.`);
  process.exit(1);
}

const extraArgs = process.argv.slice(2);
const tauriCli = join(appDir, "node_modules", "@tauri-apps", "cli", "tauri.js");
const signUpdates = JSON.stringify({ bundle: { createUpdaterArtifacts: true } });
const build = spawnSync(process.execPath, [tauriCli, "build", "--config", signUpdates, ...extraArgs], {
  cwd: appDir,
  stdio: "inherit",
  env: {
    ...process.env,
    TAURI_SIGNING_PRIVATE_KEY: readFileSync(keyPath, "utf8").trim(),
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD ?? "",
  },
});
if (build.status !== 0) {
  process.exit(build.status ?? 1);
}

const config = JSON.parse(readFileSync(join(tauriDir, "tauri.conf.json"), "utf8"));
const version = config.version;
const installer = `${config.productName}_${version}_x64-setup.exe`;
const bundleDir = join(tauriDir, "target", "release", "bundle", "nsis");
const signaturePath = join(bundleDir, `${installer}.sig`);

if (!existsSync(signaturePath)) {
  console.error(`The build did not produce ${signaturePath}.`);
  process.exit(1);
}

const manifest = {
  version,
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: readFileSync(signaturePath, "utf8").trim(),
      url: `${downloadBase}/v${version}/${installer}`,
    },
  },
};
const manifestPath = join(bundleDir, "latest.json");
writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`\nUpload both files to the v${version} release:\n  ${join(bundleDir, installer)}\n  ${manifestPath}`);
