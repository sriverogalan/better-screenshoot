#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";

const [outputPath, version, pubDate, ...entries] = process.argv.slice(2);

if (!outputPath || !version || !pubDate || entries.length === 0 || entries.length % 3 !== 0) {
  console.error(
    "Usage: generate-latest-json.mjs <output.json> <version> <pub_date> <platform> <sigPath> <url> [...]",
  );
  process.exit(1);
}

/** @type {Record<string, { signature: string; url: string }>} */
const platforms = {};

for (let index = 0; index < entries.length; index += 3) {
  const platform = entries[index];
  const sigPath = entries[index + 1];
  const url = entries[index + 2];

  platforms[platform] = {
    signature: readFileSync(sigPath, "utf8").trim(),
    url,
  };
}

const manifest = {
  version,
  notes: `Better Screenshoot ${version}`,
  pub_date: pubDate,
  platforms,
};

writeFileSync(outputPath, `${JSON.stringify(manifest, null, 2)}\n`);
