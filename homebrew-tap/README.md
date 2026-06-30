# homebrew-better-screenshoot

Homebrew tap for [Better Screenshoot](https://github.com/sriverogalan/better-screenshoot).

## Install

```sh
brew install --cask sriverogalan/better-screenshoot/better-screenshoot
```

## Update

```sh
brew upgrade --cask better-screenshoot
```

## Maintainer release checklist

After running `scripts/release-mac.sh` in the main repo:

1. Copy the `version` and `sha256` values from the cask bump block printed by the script.
2. Update `Casks/better-screenshoot.rb` with the new `version` and `sha256` values, then commit and push to this repo.
3. Publish the GitHub release draft on `sriverogalan/better-screenshoot`.
