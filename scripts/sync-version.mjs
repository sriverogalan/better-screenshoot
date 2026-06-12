#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const CHECK_ONLY = process.argv.includes("--check");

const rootPackage = JSON.parse(
  readFileSync(join(ROOT, "package.json"), "utf8"),
);
const version = rootPackage.version;

if (typeof version !== "string" || !/^\d+\.\d+\.\d+/.test(version)) {
  console.error(`Invalid root package.json version: ${String(version)}`);
  process.exit(1);
}

/** @type {Array<{ label: string; path: string; read: () => string; write: (content: string) => void }>} */
const targets = [
  {
    label: "apps/desktop/package.json",
    path: join(ROOT, "apps/desktop/package.json"),
    read() {
      return readFileSync(this.path, "utf8");
    },
    write(content) {
      writeFileSync(this.path, content);
    },
  },
  {
    label: "cli/package.json",
    path: join(ROOT, "cli/package.json"),
    read() {
      return readFileSync(this.path, "utf8");
    },
    write(content) {
      writeFileSync(this.path, content);
    },
  },
  {
    label: "packages/licensing/package.json",
    path: join(ROOT, "packages/licensing/package.json"),
    read() {
      return readFileSync(this.path, "utf8");
    },
    write(content) {
      writeFileSync(this.path, content);
    },
  },
  {
    label: "packages/shared-types/package.json",
    path: join(ROOT, "packages/shared-types/package.json"),
    read() {
      return readFileSync(this.path, "utf8");
    },
    write(content) {
      writeFileSync(this.path, content);
    },
  },
  {
    label: "apps/desktop/src-tauri/tauri.conf.json",
    path: join(ROOT, "apps/desktop/src-tauri/tauri.conf.json"),
    read() {
      return readFileSync(this.path, "utf8");
    },
    write(content) {
      writeFileSync(this.path, content);
    },
  },
  {
    label: "Cargo.toml",
    path: join(ROOT, "Cargo.toml"),
    read() {
      return readFileSync(this.path, "utf8");
    },
    write(content) {
      writeFileSync(this.path, content);
    },
  },
];

function syncJsonPackage(content, targetVersion) {
  const parsed = JSON.parse(content);
  if (parsed.version === targetVersion) {
    return { next: content, changed: false };
  }

  parsed.version = targetVersion;
  return {
    next: `${JSON.stringify(parsed, null, 2)}\n`,
    changed: true,
  };
}

function syncTauriConf(content, targetVersion) {
  const parsed = JSON.parse(content);
  if (parsed.version === targetVersion) {
    return { next: content, changed: false };
  }

  parsed.version = targetVersion;
  return {
    next: `${JSON.stringify(parsed, null, 2)}\n`,
    changed: true,
  };
}

function syncCargoToml(content, targetVersion) {
  const pattern = /(\[workspace\.package\][\s\S]*?^version = )"[^"]+"/m;
  const match = content.match(pattern);

  if (!match) {
    throw new Error("Could not find [workspace.package] version in Cargo.toml");
  }

  const current = match[0].match(/"([^"]+)"/)?.[1];
  if (current === targetVersion) {
    return { next: content, changed: false };
  }

  return {
    next: content.replace(pattern, `$1"${targetVersion}"`),
    changed: true,
  };
}

let changedCount = 0;

for (const target of targets) {
  const current = target.read();
  let result;

  if (target.label.endsWith("tauri.conf.json")) {
    result = syncTauriConf(current, version);
  } else if (target.label === "Cargo.toml") {
    result = syncCargoToml(current, version);
  } else {
    result = syncJsonPackage(current, version);
  }

  if (result.changed) {
    changedCount += 1;
    if (CHECK_ONLY) {
      console.error(`Out of sync: ${target.label} (expected ${version})`);
    } else {
      target.write(result.next);
      console.log(`Updated ${target.label} → ${version}`);
    }
  }
}

if (CHECK_ONLY) {
  if (changedCount > 0) {
    console.error(
      `\n${changedCount} file(s) out of sync. Run: pnpm sync-version`,
    );
    process.exit(1);
  }

  console.log(`All versions match root package.json (${version})`);
  process.exit(0);
}

if (changedCount === 0) {
  console.log(`All versions already match root package.json (${version})`);
} else {
  console.log(`Synced ${changedCount} file(s) to ${version}`);
}
