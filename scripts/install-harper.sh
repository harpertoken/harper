#!/usr/bin/env sh

set -eu

REPO="${HARPER_INSTALL_REPO:-harpertoken/harper}"
TAG="${HARPER_INSTALL_TAG:-latest}"
INSTALL_DIR="${HARPER_INSTALL_DIR:-$HOME/.local/bin}"
TMP_DIR="${TMPDIR:-/tmp}/harper-install.$$"
DRY_RUN="${HARPER_INSTALL_DRY_RUN:-0}"
GITHUB_TOKEN="${HARPER_INSTALL_GITHUB_TOKEN:-}"

error() {
  printf '%s\n' "$*" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || error "$1 is required"
}

detect_asset() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os:$arch" in
    Darwin:arm64) printf '%s\n' "harper-macos-aarch64.tar.gz" ;;
    Darwin:x86_64) printf '%s\n' "harper-macos-x86_64.tar.gz" ;;
    Linux:aarch64) printf '%s\n' "harper-linux-aarch64.tar.gz" ;;
    Linux:x86_64) printf '%s\n' "harper-linux-x86_64.tar.gz" ;;
    MINGW*:x86_64 | MSYS*:x86_64 | CYGWIN*:x86_64) printf '%s\n' "harper-windows-x86_64.zip" ;;
    *) error "unsupported platform: $os/$arch" ;;
  esac
}

latest_tag() {
  need curl
  need sed
  if [ -n "$GITHUB_TOKEN" ]; then
    curl -fsSL \
      -H "Authorization: Bearer $GITHUB_TOKEN" \
      -H "X-GitHub-Api-Version: 2022-11-28" \
      "https://api.github.com/repos/$REPO/releases?per_page=20"
  else
    curl -fsSL "https://api.github.com/repos/$REPO/releases?per_page=20"
  fi |
    sed -n '
      /"tag_name":[[:space:]]*"harper-[0-9][^"]*"/{
        s/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/
        p
      }
    ' |
    head -n 1
}

extract_archive() {
  archive="$1"
  case "$archive" in
    *.zip)
      need unzip
      unzip -q "$archive" -d "$TMP_DIR/extract"
      ;;
    *.tar.gz)
      need tar
      mkdir -p "$TMP_DIR/extract"
      tar -xzf "$archive" -C "$TMP_DIR/extract"
      ;;
    *) error "unsupported archive: $archive" ;;
  esac
}

cleanup() {
  rm -rf "$TMP_DIR"
}

trap cleanup EXIT INT TERM

asset="$(detect_asset)"
if [ "$TAG" = "latest" ]; then
  TAG="$(latest_tag)"
fi
[ -n "$TAG" ] || error "could not resolve Harper release tag"

archive="$TMP_DIR/$asset"
download_url="https://github.com/$REPO/releases/download/$TAG/$asset"
binary_name="harper"
case "$asset" in
  *.zip) binary_name="harper.exe" ;;
esac

if [ "$DRY_RUN" = "1" ]; then
  printf 'release_tag=%s\n' "$TAG"
  printf 'asset=%s\n' "$asset"
  printf 'install_dir=%s\n' "$INSTALL_DIR"
  printf 'download_url=%s\n' "$download_url"
  exit 0
fi

need curl
mkdir -p "$TMP_DIR" "$INSTALL_DIR"
curl -fL "$download_url" -o "$archive"
extract_archive "$archive"

binary_path="$(find "$TMP_DIR/extract" -type f -name "$binary_name" | head -n 1)"
[ -n "$binary_path" ] || error "Harper binary was not found in $asset"

cp "$binary_path" "$INSTALL_DIR/$binary_name"
chmod +x "$INSTALL_DIR/$binary_name"

printf 'installed Harper %s to %s\n' "$TAG" "$INSTALL_DIR/$binary_name"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) printf 'add %s to PATH if needed\n' "$INSTALL_DIR" ;;
esac
