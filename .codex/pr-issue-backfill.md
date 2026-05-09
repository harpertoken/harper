# PR Issue Backfill Runbook

Use this when merged PRs need tracking issues created after the fact.

## Goal

Find merged PRs that:

- do not have GitHub `closingIssuesReferences`
- do not mention an issue in the PR body, comments, or review comments

Then create a matching issue and comment back on the PR with the created issue URL.

## Safety

- Always produce a dry-run file first.
- Confirm the count with the user before creating issues.
- Write an audit mapping file as each issue is created.
- If the run stops midway, resume from the audit file instead of starting over.

## Query Merged PRs

```sh
gh api graphql --paginate -f query='query($endCursor: String) { repository(owner: "harpertoken", name: "harper") { pullRequests(first: 100, after: $endCursor, states: MERGED, orderBy: {field: CREATED_AT, direction: ASC}) { pageInfo { hasNextPage endCursor } nodes { number title url body closingIssuesReferences(first: 10) { totalCount nodes { number url } } comments(first: 50) { nodes { body } } reviews(first: 20) { nodes { body comments(first: 20) { nodes { body } } } } } } } }' --jq '.data.repository.pullRequests.nodes[] | @base64' > /private/tmp/harper-merged-prs.b64
```

## Build Candidate List

```sh
jq -Rr '@base64d | fromjson | {number,title,url,body,closing:.closingIssuesReferences.totalCount, comments:[.comments.nodes[].body], reviews:[.reviews.nodes[]? | .body, (.comments.nodes[]?.body)]} | select(.closing == 0) | .text = ((.body // "") + "\n" + ((.comments // []) | join("\n")) + "\n" + ((.reviews // []) | join("\n"))) | select((.text | test("(?i)(close[sd]?|fix(e[sd])?|resolve[sd]?|refs?|related to|issue)[: ]+(#|https://github.com/harpertoken/harper/issues/)")) | not) | .kind = (if (.title | test("(?i)(\\[fix\\]|^fix|fix:|bug|security|vulnerab|fail|stabilize|restore|sanitize|harden|correct|prevent)")) then "bug" else "feature" end) | [.number, .kind, .title, .url] | @tsv' /private/tmp/harper-merged-prs.b64 > /private/tmp/harper-prs-to-backfill-issues.tsv
```

Check the count and sample:

```sh
wc -l /private/tmp/harper-prs-to-backfill-issues.tsv
tail -20 /private/tmp/harper-prs-to-backfill-issues.tsv
```

## Build Dry Run

```sh
jq -Rr 'split("\t") | {number:.[0], kind:.[1], title:.[2], url:.[3]} | .issue_title = (if .kind == "bug" then "Backfill bug: " else "Backfill feature: " end) + .title | .labels = (if .kind == "bug" then ["bug"] else ["enhancement"] end) | .body = (if .kind == "bug" then ("## Bug Report\n\n**Description:**\nBackfilled tracking issue for merged PR #" + .number + ": " + .title + "\n\n**Steps to Reproduce:**\n1. See the related PR for the original failure context.\n\n**Expected Behavior:**\nThe behavior fixed by the related PR should be tracked as an issue.\n\n**Actual Behavior:**\nThe merged PR was not associated with a tracked issue.\n\n**Environment:**\n\n* OS:\n* Version:\n* Browser (if applicable):\n* App/Commit version:\n\n**Screenshots / Logs:**\nSee related PR if applicable.\n\n**Additional Context:**\nRelated merged PR: #" + .number + "\nPR URL: " + .url) else ("## Feature Request\n\n**Problem Statement:**\nBackfilled tracking issue for merged PR #" + .number + ": " + .title + "\n\n**Proposed Solution:**\nSee the related merged PR for the implemented solution.\n\n**Alternatives Considered:**\nSee the related PR discussion if applicable.\n\n**Additional Context:**\nRelated merged PR: #" + .number + "\nPR URL: " + .url) end) | {pr_number:(.number|tonumber), title:.issue_title, labels:.labels, body:.body, pr_comment:("Backfilled tracking issue: ISSUE_URL")} | @json' /private/tmp/harper-prs-to-backfill-issues.tsv > /private/tmp/harper-issue-backfill-dry-run.jsonl
```

## Create Issues

Only run this after the user confirms the dry-run count.

```sh
set -euo pipefail
: > /private/tmp/harper-issue-backfill-created.tsv
count=0
while IFS= read -r payload; do
  pr_number=$(jq -r '.pr_number' <<< "$payload")
  issue_payload=$(jq -c '{title, body, labels}' <<< "$payload")
  issue_json=$(gh api repos/harpertoken/harper/issues -X POST --input - <<< "$issue_payload")
  issue_number=$(jq -r '.number' <<< "$issue_json")
  issue_url=$(jq -r '.html_url' <<< "$issue_json")
  jq -n --arg body "Backfilled tracking issue: ${issue_url}" '{body: $body}' \
    | gh api repos/harpertoken/harper/issues/${pr_number}/comments -X POST --input - >/dev/null
  printf '%s\t%s\t%s\n' "$pr_number" "$issue_number" "$issue_url" >> /private/tmp/harper-issue-backfill-created.tsv
  count=$((count + 1))
  printf 'created issue #%s for PR #%s (%s)\n' "$issue_number" "$pr_number" "$count"
done < /private/tmp/harper-issue-backfill-dry-run.jsonl
```

## Verify

```sh
wc -l /private/tmp/harper-issue-backfill-created.tsv
tail -5 /private/tmp/harper-issue-backfill-created.tsv
```
