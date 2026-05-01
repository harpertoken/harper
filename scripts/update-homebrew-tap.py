#!/usr/bin/env python3

import argparse
import pathlib
import re
import sys


def replace_once(content: str, pattern: str, replacement: str) -> str:
    updated, count = re.subn(pattern, replacement, content, count=1, flags=re.MULTILINE)
    if count != 1:
        raise ValueError(f"pattern not found exactly once: {pattern}")
    return updated


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Update harper-ai Homebrew formula to a released Harper version."
    )
    parser.add_argument("--release-tag", required=True, help="Release tag like harper-0.17.1")
    parser.add_argument("--sha256", required=True, help="SHA256 of the release source tarball")
    parser.add_argument("--formula-path", required=True, help="Path to Formula/harper-ai.rb")
    args = parser.parse_args()

    release_tag = args.release_tag
    prefix = "harper-"
    if not release_tag.startswith(prefix):
        raise SystemExit(f"release tag must start with '{prefix}': {release_tag}")

    version = release_tag[len(prefix) :]
    tarball_url = (
        f"https://github.com/harpertoken/harper/archive/refs/tags/{release_tag}.tar.gz"
    )

    formula_path = pathlib.Path(args.formula_path)
    content = formula_path.read_text(encoding="utf-8")

    if f'version "{version}"' in content and tarball_url in content and args.sha256 in content:
        print("formula already matches requested release")
        return 0

    content = replace_once(content, r'^  version ".*"$', f'  version "{version}"')
    content = replace_once(
        content,
        r'^      url "https://github\.com/harpertoken/harper/archive/refs/tags/.*\.tar\.gz"$',
        f'      url "{tarball_url}"',
    )
    content = replace_once(
        content,
        r'^      sha256 "[0-9a-f]{64}"$',
        f'      sha256 "{args.sha256}"',
    )
    content = replace_once(
        content,
        r'^      url "https://github\.com/harpertoken/harper/archive/refs/tags/.*\.tar\.gz"$',
        f'      url "{tarball_url}"',
    )
    content = replace_once(
        content,
        r'^      sha256 "[0-9a-f]{64}"$',
        f'      sha256 "{args.sha256}"',
    )

    formula_path.write_text(content, encoding="utf-8")
    print(f"updated {formula_path} to {release_tag}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as err:
        print(err, file=sys.stderr)
        raise SystemExit(1)
