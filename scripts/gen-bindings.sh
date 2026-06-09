#!/usr/bin/env bash
# 生成移动端 UniFFI 绑定（Swift / Kotlin）。
#
# 用法:
#   scripts/gen-bindings.sh swift   ./bindings/swift
#   scripts/gen-bindings.sh kotlin  ./bindings/kotlin
#
# 前置：先 `cargo build -p ai-pocket-ffi` 得到 cdylib。
set -euo pipefail

LANG_TARGET="${1:-swift}"
OUT_DIR="${2:-bindings/${LANG_TARGET}}"

# 仓库根目录（脚本位于 <root>/scripts/）。
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# 动态库后缀按平台区分。
case "$(uname -s)" in
  Darwin) LIB="libai_pocket_ffi.dylib" ;;
  Linux)  LIB="libai_pocket_ffi.so" ;;
  *)      LIB="ai_pocket_ffi.dll" ;;
esac

LIB_PATH="target/debug/${LIB}"

echo "==> building ai-pocket-ffi cdylib"
cargo build -p ai-pocket-ffi

if [[ ! -f "$LIB_PATH" ]]; then
  echo "error: built library not found at $LIB_PATH" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

echo "==> generating ${LANG_TARGET} bindings into ${OUT_DIR}"
cargo run -p ai-pocket-ffi --bin uniffi-bindgen -- \
  generate \
  --library "$LIB_PATH" \
  --language "$LANG_TARGET" \
  --config crates/ffi/uniffi.toml \
  --out-dir "$OUT_DIR"

echo "==> done: $OUT_DIR"
