#!/usr/bin/env bash
set -euo pipefail

REPO="${GITHUB_REPO:-sriverogalan/better-screenshoot}"
BRANCHES=("main" "develop")

if ! command -v gh >/dev/null 2>&1; then
  echo "Install GitHub CLI: https://cli.github.com/"
  exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
  echo "Run: gh auth login"
  exit 1
fi

protection_payload() {
  cat <<'EOF'
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
}

for branch in "${BRANCHES[@]}"; do
  echo "Configuring branch protection for '${branch}' on ${REPO}…"
  gh api \
    --method PUT \
    "repos/${REPO}/branches/${branch}/protection" \
    --input - <<<"$(protection_payload)"
done

echo "Done. Protected branches: ${BRANCHES[*]}"
