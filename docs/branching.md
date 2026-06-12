# Branch rules

## Main branches

| Branch | Purpose |
|---|---|
| `main` | Production. Always deployable. Releases are cut from here. |
| `develop` | Integration branch. Day-to-day work merges here first. |
| `feat/*` | New features |
| `fix/*` | Bug fixes |
| `chore/*` | CI, dependencies, docs, refactors without behavior changes |
| `release/*` | Version preparation (optional) |

## Workflow

1. Create a branch from up-to-date `main`:
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feat/my-change
   ```
2. Small, descriptive commits (Conventional Commits: `feat:`, `fix:`, `chore:`…).
3. Open a PR to `develop` (or `main` for hotfixes). Title: `[better-screenshoot] Clear description`.
4. Wait for green CI (`sync-version`, `frontend`, `rust`).
5. Merge (squash recommended for a clean history).
6. Open a PR from `develop` → `main` when ready to release.

## `main` protection

Configure these rules in GitHub (Settings → Branches → Add rule):

- **Branch name patterns:** `main`, `develop`
- Require a pull request before merging
- Require status checks to pass: `ci-success`
- Do not allow bypassing the above settings
- Restrict pushes (optional): admins only or no direct pushes

### Apply with script

After `gh auth login`:

```bash
./scripts/setup-branch-protection.sh
```

## Releases and versions

- Bump the version in the **root** `package.json` only.
- On pull requests, CI runs `pnpm sync-version` and commits the matching updates to `Cargo.toml`, `tauri.conf.json`, and workspace `package.json` files.
- On `main`, CI fails if any of those files drift from the root version.
- Tags follow semver: `v0.2.0`, `v0.2.1`, `v1.0.0`.
- **Tag only from `main`**, after merging the release PR.

```bash
git checkout main
git pull origin main
git tag v0.2.0
git push origin v0.2.0
pnpm release:mac v0.2.0 --all-arch
```

The release script only runs on `main`. It builds and signs on your Mac, then uploads a draft release with `.dmg` installers and updater artifacts. Publish the draft on GitHub when ready.

## What not to do

- Don't push directly to `main` (except agreed hotfixes).
- Don't create tags from feature branches.
- Don't mix version bumps with unrelated feature changes.
