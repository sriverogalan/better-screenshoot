# Branch rules

## Main branches

| Branch | Purpose |
|---|---|
| `main` | Production. Always deployable. Code enters only via PR. |
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
3. Open a PR to `main`. Title: `[better-screenshoot] Clear description`.
4. Wait for green CI (`frontend`, `rust`, `tauri-build`).
5. Merge (squash recommended for a clean history).

## `main` protection

Configure these rules in GitHub (Settings → Branches → Add rule):

- **Branch name pattern:** `main`
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

- Version lives in `Cargo.toml` (workspace), root `package.json`, and `apps/desktop/src-tauri/tauri.conf.json`.
- When preparing a release, sync the version across all those files.
- Tags follow semver: `v0.2.0`, `v0.2.1`, `v1.0.0`.
- **Tag only from `main`**, after merging the release PR.

```bash
git checkout main
git pull origin main
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions will publish a draft release with `.dmg` installers for Apple Silicon and Intel.

## What not to do

- Don't push directly to `main` (except agreed hotfixes).
- Don't create tags from feature branches.
- Don't mix version bumps with unrelated feature changes.
