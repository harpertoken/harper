---
name: pr
description: Pull request workflow
---

# Pull Request Workflow

See AGENTS.md for full PR guidelines.

## Before Creating PR
- `git diff` - review changes
- `git log --oneline -5` - review commits
- Ensure all checks pass

## Create PR
```bash
gh pr create --title "[scope] description" --body "## Summary"
```

## PR Title Format
- `[feat]` - new feature
- `[fix]` - bug fix
- `[docs]` - documentation
- `[chore]` - maintenance
- `[refactor]` - code refactoring
- `[test]` - tests

## After PR
- `gh pr status` - check status
- Address review comments
- Squash commits if needed