#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO="${GITHUB_REPO:-sriverogalan/better-screenshoot}"
KEY_PATH="${TAURI_SIGNING_PRIVATE_KEY_PATH:-$ROOT/.tauri/better-screenshoot.key}"
STAGING="$ROOT/.release-staging"
ALL_ARCH=false
DRAFT=true
TAG=""

usage() {
  cat <<'EOF'
Build and publish a macOS release locally (signed updater artifacts + .dmg).

Usage:
  ./scripts/release-mac.sh <tag> [options]

Arguments:
  tag                 Git tag, e.g. v0.2.1 (must match tauri.conf.json version)

Options:
  --all-arch          Build Apple Silicon and Intel (default: native arch only)
  --publish           Publish the GitHub release immediately (default: draft)
  -h, --help          Show this help

Examples:
  ./scripts/release-mac.sh v0.2.1
  ./scripts/release-mac.sh v0.2.1 --all-arch --publish

Requires: git checkout on main, pnpm, gh, rustup targets, signing key at .tauri/better-screenshoot.key
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all-arch)
      ALL_ARCH=true
      shift
      ;;
    --publish)
      DRAFT=false
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    -*)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
    *)
      if [[ -n "$TAG" ]]; then
        echo "Unexpected argument: $1" >&2
        exit 1
      fi
      TAG="$1"
      shift
      ;;
  esac
done

if [[ -z "$TAG" ]]; then
  echo "Missing release tag." >&2
  usage >&2
  exit 1
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS releases must be built on a Mac." >&2
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "Install GitHub CLI: https://cli.github.com/" >&2
  exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
  echo "Run: gh auth login" >&2
  exit 1
fi

if [[ ! -f "$KEY_PATH" ]]; then
  echo "Signing key not found: $KEY_PATH" >&2
  echo "Generate one with:" >&2
  echo "  pnpm --filter @better-screenshoot/desktop tauri signer generate --ci -w .tauri/better-screenshoot.key" >&2
  exit 1
fi

if ! git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Not a git repository." >&2
  exit 1
fi

BRANCH="$(git -C "$ROOT" rev-parse --abbrev-ref HEAD)"
if [[ "$BRANCH" != "main" ]]; then
  echo "Releases must be cut from main (current branch: $BRANCH)." >&2
  echo "Run: git checkout main && git pull origin main" >&2
  exit 1
fi

VERSION="${TAG#v}"
CONF_VERSION="$(node -pe "JSON.parse(require('fs').readFileSync('$ROOT/apps/desktop/src-tauri/tauri.conf.json','utf8')).version")"

if [[ "$CONF_VERSION" != "$VERSION" ]]; then
  echo "Version mismatch: tag $TAG but tauri.conf.json is $CONF_VERSION" >&2
  echo "Sync version in Cargo.toml, package.json files, and tauri.conf.json first." >&2
  exit 1
fi

if [[ "$ALL_ARCH" == true ]]; then
  TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
  rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null 2>&1 || true
else
  case "$(uname -m)" in
    arm64) TARGETS=(aarch64-apple-darwin) ;;
    x86_64) TARGETS=(x86_64-apple-darwin) ;;
    *)
      echo "Unsupported architecture: $(uname -m)" >&2
      exit 1
      ;;
  esac
fi

export CARGO_TARGET_DIR="$ROOT/apps/desktop/src-tauri/target"
export TAURI_SIGNING_PRIVATE_KEY="$(cat "$KEY_PATH")"
export CI=true

rm -rf "$STAGING"
mkdir -p "$STAGING"

echo "Building Better Screenshoot $TAG for: ${TARGETS[*]}"

cd "$ROOT"
pnpm install

LATEST_ARGS=()
SHA256_ARM64=""
SHA256_X64=""

for target in "${TARGETS[@]}"; do
  echo "→ tauri build --target $target"
  pnpm --filter @better-screenshoot/desktop tauri build --target "$target"

  bundle_root="$ROOT/apps/desktop/src-tauri/target/$target/release/bundle"
  tarball="$(find "$bundle_root/macos" -maxdepth 1 -name '*.tar.gz' ! -name '*.sig' | head -n 1)"
  sig_file="${tarball}.sig"
  dmg_file="$(find "$bundle_root/dmg" -maxdepth 1 -name '*.dmg' | head -n 1)"

  if [[ -z "$tarball" || ! -f "$sig_file" || -z "$dmg_file" ]]; then
    echo "Missing updater or dmg artifacts for $target" >&2
    exit 1
  fi

  case "$target" in
    aarch64-apple-darwin)
      platform="darwin-aarch64"
      suffix="aarch64"
      ;;
    x86_64-apple-darwin)
      platform="darwin-x86_64"
      suffix="x64"
      ;;
    *)
      echo "Unknown target: $target" >&2
      exit 1
      ;;
  esac

  staged_tarball="$STAGING/Better-Screenshoot_${VERSION}_${suffix}.app.tar.gz"
  staged_sig="$STAGING/Better-Screenshoot_${VERSION}_${suffix}.app.tar.gz.sig"
  staged_dmg="$STAGING/Better-Screenshoot_${VERSION}_${suffix}.dmg"

  cp "$tarball" "$staged_tarball"
  cp "$sig_file" "$staged_sig"
  cp "$dmg_file" "$staged_dmg"

  _sha256="$(shasum -a 256 "$staged_dmg" | awk '{print $1}')"
  case "$target" in
    aarch64-apple-darwin) SHA256_ARM64="$_sha256" ;;
    x86_64-apple-darwin)  SHA256_X64="$_sha256" ;;
  esac

  tarball_name="$(basename "$staged_tarball")"
  LATEST_ARGS+=("$platform" "$staged_sig" "https://github.com/$REPO/releases/download/$TAG/$tarball_name")
done

echo ""
echo "━━━ Cask bump — homebrew-better-screenshoot/Casks/better-screenshoot.rb ━━━"
echo "  version \"$VERSION\""
[[ -n "$SHA256_ARM64" ]] && echo "  on_arm   → sha256 \"$SHA256_ARM64\"" || echo "  WARNING: SHA256 not computed for arm64 (arch not built)"
[[ -n "$SHA256_X64"   ]] && echo "  on_intel → sha256 \"$SHA256_X64\""   || echo "  WARNING: SHA256 not computed for x86_64 (arch not built)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

PUB_DATE="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
LATEST_JSON="$STAGING/latest.json"
node "$ROOT/scripts/generate-latest-json.mjs" "$LATEST_JSON" "$VERSION" "$PUB_DATE" "${LATEST_ARGS[@]}"

RELEASE_ARGS=(release create "$TAG" --repo "$REPO" --title "Better Screenshoot $TAG")
if [[ "$DRAFT" == true ]]; then
  RELEASE_ARGS+=(--draft)
fi

INSTALL_NOTES_FILE="$ROOT/docs/release-install-notes.md"
if [[ ! -f "$INSTALL_NOTES_FILE" ]]; then
  echo "Missing release notes template: $INSTALL_NOTES_FILE" >&2
  exit 1
fi

RELEASE_ARGS+=(--notes-file "$INSTALL_NOTES_FILE")

if gh release view "$TAG" --repo "$REPO" >/dev/null 2>&1; then
  echo "Release $TAG already exists — uploading assets"
  gh release upload "$TAG" --repo "$REPO" --clobber "$STAGING"/*
else
  gh "${RELEASE_ARGS[@]}" "$STAGING"/*
fi

echo ""
echo "Done. Release: https://github.com/$REPO/releases/tag/$TAG"
if [[ "$DRAFT" == true ]]; then
  echo "Review the draft on GitHub and publish when ready."
fi

rm -rf "$STAGING"
