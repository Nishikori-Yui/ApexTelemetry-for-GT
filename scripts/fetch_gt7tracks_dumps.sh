#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="$ROOT_DIR/data/vendor/GT7Tracks"
REPO_URL="https://github.com/vthinsel/GT7Tracks"

if ! command -v git >/dev/null 2>&1; then
  echo "git is required to download GT7Tracks dumps." >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

git clone --depth 1 --filter=blob:none --sparse "$REPO_URL" "$TMP_DIR/GT7Tracks"

git -C "$TMP_DIR/GT7Tracks" sparse-checkout set dumps

rm -rf "$TARGET_DIR/dumps"
mkdir -p "$TARGET_DIR"
cp -R "$TMP_DIR/GT7Tracks/dumps" "$TARGET_DIR/"

echo "GT7Tracks dumps downloaded to $TARGET_DIR/dumps"
