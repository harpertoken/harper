#!/usr/bin/env python3

import argparse
import pathlib
import re


PACKAGE_NAME = "harper"
HOMEPAGE = "https://github.com/harpertoken/harper"
LICENSE = "mit"
DESCRIPTION = "Terminal assistant for code and shell work."

ASSETS = {
    "x86_64-linux": {
        "archive": "harper-linux-x86_64.tar.gz",
        "arch": "x86_64",
    },
    "aarch64-linux": {
        "archive": "harper-linux-aarch64.tar.gz",
        "arch": "aarch64",
    },
}


def version_from_tag(release_tag: str) -> str:
    prefix = "harper-"
    if not release_tag.startswith(prefix):
        raise ValueError(f"release tag must start with '{prefix}': {release_tag}")
    version = release_tag[len(prefix) :]
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?", version):
        raise ValueError(f"release tag does not contain a package version: {release_tag}")
    return version


def asset_url(release_tag: str, archive: str) -> str:
    return f"{HOMEPAGE}/releases/download/{release_tag}/{archive}"


def write_flake(output_dir: pathlib.Path, release_tag: str, version: str, checksums: dict[str, str]) -> None:
    systems = ", ".join(f'"{system}"' for system in ASSETS)
    sources = "\n".join(
        f"""          {system} = pkgs.fetchurl {{
            url = "{asset_url(release_tag, metadata["archive"])}";
            sha256 = "{checksums[system]}";
          }};"""
        for system, metadata in ASSETS.items()
    )

    flake = f"""# Generated package-channel manifest for Harper {release_tag}.
{{
  description = "Harper CLI";

  outputs = {{ self, nixpkgs }}:
    let
      systems = [ {systems} ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {{
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs {{ inherit system; }};
          sources = {{
{sources}
          }};
        in
        {{
          default = pkgs.stdenvNoCC.mkDerivation {{
            pname = "{PACKAGE_NAME}";
            version = "{version}";
            src = sources.${{system}};

            installPhase = ''
              runHook preInstall
              install -Dm755 harper $out/bin/harper
              runHook postInstall
            '';

            meta = with pkgs.lib; {{
              description = "{DESCRIPTION}";
              homepage = "{HOMEPAGE}";
              license = licenses.mit;
              platforms = systems;
              mainProgram = "harper";
            }};
          }};
        }});
    }};
}}
"""

    path = output_dir / "nix" / "flake.nix"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(flake, encoding="utf-8")


def write_pkgbuild(output_dir: pathlib.Path, release_tag: str, version: str, checksums: dict[str, str]) -> None:
    pkgbuild = f"""# Generated package-channel manifest for Harper {release_tag}.
pkgname=harper
pkgver={version}
pkgrel=1
pkgdesc="{DESCRIPTION}"
arch=('x86_64' 'aarch64')
url="{HOMEPAGE}"
license=('MIT')
source_x86_64=("{asset_url(release_tag, ASSETS["x86_64-linux"]["archive"])}")
source_aarch64=("{asset_url(release_tag, ASSETS["aarch64-linux"]["archive"])}")
sha256sums_x86_64=('{checksums["x86_64-linux"]}')
sha256sums_aarch64=('{checksums["aarch64-linux"]}')

package() {{
  install -Dm755 harper "$pkgdir/usr/bin/harper"
}}
"""

    path = output_dir / "arch" / "PKGBUILD"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(pkgbuild, encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate Nix and Arch package-channel manifests for a Harper release."
    )
    parser.add_argument("--release-tag", required=True, help="Release tag like harper-0.20.1")
    parser.add_argument("--linux-x86-64-sha256", required=True, help="SHA256 for harper-linux-x86_64.tar.gz")
    parser.add_argument("--linux-aarch64-sha256", required=True, help="SHA256 for harper-linux-aarch64.tar.gz")
    parser.add_argument("--output-dir", required=True, help="Directory to write manifests into")
    args = parser.parse_args()

    checksums = {
        "x86_64-linux": args.linux_x86_64_sha256.lower(),
        "aarch64-linux": args.linux_aarch64_sha256.lower(),
    }
    for checksum in checksums.values():
        if not re.fullmatch(r"[0-9a-f]{64}", checksum):
            raise SystemExit("sha256 values must be 64 lowercase or uppercase hexadecimal characters")

    try:
        version = version_from_tag(args.release_tag)
    except ValueError as err:
        raise SystemExit(str(err)) from err

    output_dir = pathlib.Path(args.output_dir)
    write_flake(output_dir, args.release_tag, version, checksums)
    write_pkgbuild(output_dir, args.release_tag, version, checksums)
    print(f"generated Nix and Arch package manifests for {args.release_tag} in {output_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
