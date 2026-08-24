#!/usr/bin/env bash
# ==============================================================================
# cherenkov-lings — macOS / Linux Global Installer
# ==============================================================================

set -euo pipefail

INSTALL_DIR="${HOME}/.cherenkov-lings/bin"
BINARY_NAME="cherenkov-lings"

echo "? Installing cherenkov-lings globally..."

# 1. Check for Rust / Cargo
if ! command -v cargo &> /dev/null; then
    echo "? Rust/Cargo is not installed. Please install Rust via https://rustup.rs first."
    exit 1
fi

# 2. Build optimized release binary
echo "?? Building release binary with Cargo..."
cargo build --release

# 3. Create install directory
mkdir -p "${INSTALL_DIR}"

# 4. Copy binary
cp "target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo "? Binary installed to: ${INSTALL_DIR}/${BINARY_NAME}"

# 5. Check and configure PATH in user shell profile
SHELL_PROFILE=""
if [ -n "${ZSH_VERSION:-}" ] || [ "${SHELL:-}" = "*/zsh" ]; then
    SHELL_PROFILE="${HOME}/.zshrc"
elif [ -f "${HOME}/.bashrc" ]; then
    SHELL_PROFILE="${HOME}/.bashrc"
elif [ -f "${HOME}/.bash_profile" ]; then
    SHELL_PROFILE="${HOME}/.bash_profile"
fi

EXPORT_LINE="export PATH=\"\${HOME}/.cherenkov-lings/bin:\${PATH}\""

if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
    if [ -n "${SHELL_PROFILE}" ]; then
        if ! grep -qs "cherenkov-lings" "${SHELL_PROFILE}"; then
            echo "" >> "${SHELL_PROFILE}"
            echo "# cherenkov-lings CLI" >> "${SHELL_PROFILE}"
            echo "${EXPORT_LINE}" >> "${SHELL_PROFILE}"
            echo "? Added ~/.cherenkov-lings/bin to ${SHELL_PROFILE}"
        fi
    fi
    echo "??  Run 'source ${SHELL_PROFILE:-~/.bashrc}' or restart your terminal to reload PATH."
else
    echo "? PATH already contains ${INSTALL_DIR}."
fi

echo ""
echo "?? Installation Complete!"
echo "Run 'cherenkov-lings dashboard' or 'cherenkov-lings watch --track=foundations' to begin."
