#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="dl-rs"
INSTALL_DIR="/usr/local/bin"
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

BLUE_BOLD="\033[1;34m"
YELLOW_BOLD="\033[1;33m"
RESET="\033[0m"

echo "Building ${BIN_NAME}..."
cargo build --release --manifest-path "${PROJECT_DIR}/Cargo.toml"

sudo mkdir -p "${INSTALL_DIR}"
sudo cp "${PROJECT_DIR}/target/release/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
sudo chmod +x "${INSTALL_DIR}/${BIN_NAME}"

echo -e "${BLUE_BOLD}${BIN_NAME}${RESET} installed to ${YELLOW_BOLD}${INSTALL_DIR}/${BIN_NAME}${RESET}"
