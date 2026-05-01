#!/usr/bin/env python3

import argparse
import base64
import pathlib
import subprocess
import tempfile


def extract_public_key_b64(private_key_pem_b64: str) -> str:
    private_key_bytes = base64.b64decode(private_key_pem_b64.strip())
    with tempfile.TemporaryDirectory() as tmpdir:
        private_key_path = pathlib.Path(tmpdir) / "update-signing.pem"
        public_key_der_path = pathlib.Path(tmpdir) / "update-public.der"
        private_key_path.write_bytes(private_key_bytes)

        subprocess.run(
            [
                "openssl",
                "pkey",
                "-in",
                str(private_key_path),
                "-pubout",
                "-outform",
                "DER",
                "-out",
                str(public_key_der_path),
            ],
            check=True,
            capture_output=True,
        )

        public_key_der = public_key_der_path.read_bytes()
        public_key_raw = public_key_der[-32:]
        return base64.b64encode(public_key_raw).decode("utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check that the release signing secret matches the repo-shipped updater public key."
    )
    parser.add_argument("--private-key-b64", required=True)
    parser.add_argument("--public-key-file", required=True)
    args = parser.parse_args()

    expected = pathlib.Path(args.public_key_file).read_text(encoding="utf-8").strip()
    actual = extract_public_key_b64(args.private_key_b64)

    if actual != expected:
        raise SystemExit(
            "update signing key mismatch: workflow secret does not match repo public key"
        )

    print("update signing key matches repo public key")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
