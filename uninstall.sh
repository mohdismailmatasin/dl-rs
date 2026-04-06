#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="dl-rs"
INSTALL_DIR="/usr/local/bin"

BLUE_BOLD="\033[1;34m"
YELLOW_BOLD="\033[1;33m"
RESET="\033[0m"

if [ -f "${INSTALL_DIR}/${BIN_NAME}" ]; then
    sudo rm "${INSTALL_DIR}/${BIN_NAME}"
    echo -e "${BLUE_BOLD}${BIN_NAME}${RESET} uninstalled from ${YELLOW_BOLD}${INSTALL_DIR}/${BIN_NAME}${RESET}"
else
    echo -e "${BLUE_BOLD}${BIN_NAME}${RESET} is not installed."
fi
