#!/usr/bin/env bash
set -euo pipefail

REPO="${GITHUB_REPO:-sriverogalan/better-screenshoot}"
BRANCH="${1:-main}"

if ! command -v gh >/dev/null 2>&1; then
  echo "Instala GitHub CLI: https://cli.github.com/"
  exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
  echo "Ejecuta primero: gh auth login"
  exit 1
fi

echo "Configurando protección de rama '${BRANCH}' en ${REPO}…"

gh api \
  --method PUT \
  "repos/${REPO}/branches/${BRANCH}/protection" \
  --input - <<EOF
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["ci-success"]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "required_approving_review_count": 0,
    "dismiss_stale_reviews": true
  },
  "restrictions": null,
  "required_linear_history": false,
  "allow_force_pushes": false,
  "allow_deletions": false
}
EOF

echo "Listo. Revisa en GitHub → Settings → Branches → ${BRANCH}."
