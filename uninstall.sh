#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="dl-rs"
INSTALL_DIR="${HOME}/.local/bin"

if [ -f "${INSTALL_DIR}/${BIN_NAME}" ]; then
    rm "${INSTALL_DIR}/${BIN_NAME}"
    echo "${BIN_NAME} uninstalled from ${INSTALL_DIR}/${BIN_NAME}"
else
    echo "${BIN_NAME} is not installed."
fi
