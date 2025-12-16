# Copyright 2025 harpertoken
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

#!/bin/bash
set -euo pipefail

PREV_TAG=$(git tag --sort=-version:refname | awk "/^$CURR_TAG$/{getline; print; exit}" || echo "")
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

output=$(gh api repos/$GITHUB_REPO/compare/$BASE_REF...$CURR_TAG -q .commits[].author.login 2>/dev/null) || output=""
if [[ $output == \{* ]]; then
  echo '- None' >> RELEASE_NOTES.md
else
  echo "$output" | grep -v '^$' | sort -u | sed 's/^/- @/' >> RELEASE_NOTES.md || echo '- None' >> RELEASE_NOTES.md
fi

if ! tail -n +1 RELEASE_NOTES.md | awk 'BEGIN {p=0} { if (p == 0 && /^### Contributors/) p=1; if (p == 1 && /^### /) p=0; if (p == 1) print }' | grep -q "^- @"; then
  echo '- None' >> RELEASE_NOTES.md
fi

echo '----- RELEASE_NOTES.md -----'
cat RELEASE_NOTES.md
