---
name: commit
description: Git commit workflow
---

# Git Commit Workflow

See AGENTS.md for commit guidelines.

## Before Committing
- `git status` - review changes
- `git diff` - see unstaged changes
- `git diff --staged` - see staged changes

## Stage Files
- `git add <file>` - stage specific file
- `git add .` - stage all changes
- `git add -p` - stage interactively

## Commit
```bash
git commit -m "type: description"
```

## Commit Types
- `[feat]` - new feature
- `[fix]` - bug fix
- `[docs]` - documentation
- `[chore]` - maintenance
- `[refactor]` - code refactoring
- `[test]` - tests
- `[perf]` - performance
- `[style]` - formatting

## Amend Last Commit
- `git commit --amend`
- Only amend unpushed commits

## Tips
- Keep commits atomic
- Write descriptive messages
- First line: 72 chars max
