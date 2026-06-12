import { readFileSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const localesDir = join(root, "src/locales");
const trayLocalesDir = join(root, "src-tauri/locales");

function collectKeys(value, prefix = "") {
  const keys = [];
  if (value && typeof value === "object" && !Array.isArray(value)) {
    for (const [key, nested] of Object.entries(value)) {
      const next = prefix ? `${prefix}.${key}` : key;
      if (nested && typeof nested === "object" && !Array.isArray(nested)) {
        keys.push(...collectKeys(nested, next));
      } else {
        keys.push(next);
      }
    }
  }
  return keys;
}

function loadLocale(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function compareLocales(baseName, baseKeys, fileName, dir) {
  const filePath = join(dir, fileName);
  const locale = loadLocale(filePath);
  const keys = collectKeys(locale).sort();
  const missing = baseKeys.filter((key) => !keys.includes(key));
  const extra = keys.filter((key) => !baseKeys.includes(key));

  if (missing.length > 0 || extra.length > 0) {
    console.error(`\n${fileName}:`);
    if (missing.length > 0) {
      console.error(`  Missing keys (${missing.length}):`);
      missing.forEach((key) => console.error(`    - ${key}`));
    }
    if (extra.length > 0) {
      console.error(`  Extra keys (${extra.length}):`);
      extra.forEach((key) => console.error(`    - ${key}`));
    }
    return false;
  }

  return true;
}

function compareTrayLocales() {
  const base = loadLocale(join(trayLocalesDir, "en.json"));
  const baseTrayKeys = collectKeys(base.tray).sort();
  const trayFiles = readdirSync(trayLocalesDir).filter((file) => file.endsWith(".json"));
  let ok = true;

  for (const file of trayFiles) {
    if (file === "en.json") continue;
    const locale = loadLocale(join(trayLocalesDir, file));
    const keys = collectKeys(locale.tray).sort();
    const missing = baseTrayKeys.filter((key) => !keys.includes(key));
    const extra = keys.filter((key) => !baseTrayKeys.includes(key));
    if (missing.length > 0 || extra.length > 0) {
      console.error(`\nTray locale ${file}:`);
      missing.forEach((key) => console.error(`  missing: tray.${key}`));
      extra.forEach((key) => console.error(`  extra: tray.${key}`));
      ok = false;
    }
  }

  return ok;
}

const base = loadLocale(join(localesDir, "en.json"));
const baseKeys = collectKeys(base).sort();
const localeFiles = readdirSync(localesDir).filter((file) => file.endsWith(".json"));

let success = true;
for (const file of localeFiles) {
  if (file === "en.json") continue;
  if (!compareLocales("en", baseKeys, file, localesDir)) {
    success = false;
  }
}

if (!compareTrayLocales()) {
  success = false;
}

if (!success) {
  process.exit(1);
}

console.log(`All ${localeFiles.length} UI locales and tray locales match en.json keys.`);
