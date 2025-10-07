#!/bin/bash
set -euo pipefail

PREV_TAG=$(git describe --tags --abbrev=0 "${CURR_TAG}^" 2>/dev/null || echo "")
ROOT_SHA=$(git rev-list --max-parents=0 HEAD | tail -n1)

if [ -n "$PREV_TAG" ]; then
  BASE_REF="$PREV_TAG"
  RANGE="$PREV_TAG..$CURR_TAG"
else
  BASE_REF="$ROOT_SHA"
  RANGE="$ROOT_SHA..$CURR_TAG"
fi

{
  echo "## Release $CURR_TAG"
  echo
  echo "### Changes"
} > RELEASE_NOTES.md

git log "$RANGE" --pretty=format:'- %s' >> RELEASE_NOTES.md || true

if ! grep -q "^- " RELEASE_NOTES.md; then
  echo '- None' >> RELEASE_NOTES.md
fi

{
  echo
  echo "### Contributors"
} >> RELEASE_NOTES.md

gh api repos/$GITHUB_REPO/compare/$BASE_REF...$CURR_TAG -q .commits[].author.login 2>/dev/null | grep -v '^$' | sort -u | sed 's/^/- @/' >> RELEASE_NOTES.md || true

if ! tail -n +1 RELEASE_NOTES.md | awk 'BEGIN {p=0} { if (p == 0 && /^### Contributors/) p=1; if (p == 1 && /^### /) p=0; if (p == 1) print }' | grep -q "^- @"; then
  echo '- None' >> RELEASE_NOTES.md
fi

echo '----- RELEASE_NOTES.md -----'
cat RELEASE_NOTES.md