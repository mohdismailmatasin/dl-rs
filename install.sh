#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="dl-rs"
INSTALL_DIR="${HOME}/.local/bin"
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Building ${BIN_NAME}..."
cargo build --release --manifest-path "${PROJECT_DIR}/Cargo.toml"

mkdir -p "${INSTALL_DIR}"
cp "${PROJECT_DIR}/target/release/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
chmod +x "${INSTALL_DIR}/${BIN_NAME}"

echo "${BIN_NAME} installed to ${INSTALL_DIR}/${BIN_NAME}"
echo "Ensure ${INSTALL_DIR} is in your PATH."
