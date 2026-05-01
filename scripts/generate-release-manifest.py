#!/usr/bin/env python3

import argparse
import json
from pathlib import Path


def parse_version(raw: str) -> str:
    raw = raw.strip()
    if raw.startswith("harper v"):
        return raw[len("harper v") :].strip()
    return raw


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate a Harper release manifest.")
    parser.add_argument("--version", required=True)
    parser.add_argument("--published-at", default=None)
    parser.add_argument(
        "--artifact",
        action="append",
        default=[],
        help="Artifact entry in the form target=url=sha256=signature_b64",
    )
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    artifacts = {}
    for entry in args.artifact:
        try:
            target, url, sha256, signature = entry.split("=", 3)
        except ValueError as exc:
            raise SystemExit(
                f"invalid artifact entry '{entry}': expected target=url=sha256=signature_b64"
            ) from exc
        artifacts[target] = {"url": url, "sha256": sha256, "signature": signature}

    manifest = {
        "version": parse_version(args.version),
        "published_at": args.published_at,
        "artifacts": artifacts,
    }

    output = Path(args.output)
    output.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
