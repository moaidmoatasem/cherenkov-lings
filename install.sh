#!/usr/bin/env bash
# ==============================================================================
# cherenkov-lings -- macOS / Linux Global Installer
# ==============================================================================

set -euo pipefail

INSTALL_DIR="${HOME}/.cherenkov-lings/bin"
BINARY_NAME="cherenkov-lings"

echo "[INFO] Installing cherenkov-lings globally..."

# 1. Check for Rust / Cargo
if ! command -v cargo &> /dev/null; then
    echo "[ERROR] Rust/Cargo is not installed. Install it via https://rustup.rs and re-run this script."
    exit 1
fi

# 2. Build optimized release binary
echo "[INFO] Building release binary with Cargo..."
cargo build --release

# 3. Create install directory
mkdir -p "${INSTALL_DIR}"

# 4. Copy binary
cp "target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo "[OK] Binary installed to: ${INSTALL_DIR}/${BINARY_NAME}"

# 5. Check and configure PATH in user shell profile
SHELL_PROFILE=""
case "${SHELL:-}" in
    */zsh)
        SHELL_PROFILE="${HOME}/.zshrc"
        ;;
    *)
        if [ -f "${HOME}/.bashrc" ]; then
            SHELL_PROFILE="${HOME}/.bashrc"
        elif [ -f "${HOME}/.bash_profile" ]; then
            SHELL_PROFILE="${HOME}/.bash_profile"
        fi
        ;;
esac

EXPORT_LINE="export PATH=\"\${HOME}/.cherenkov-lings/bin:\${PATH}\""

if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
    if [ -n "${SHELL_PROFILE}" ]; then
        if ! grep -qs "cherenkov-lings" "${SHELL_PROFILE}"; then
            echo "" >> "${SHELL_PROFILE}"
            echo "# cherenkov-lings CLI" >> "${SHELL_PROFILE}"
            echo "${EXPORT_LINE}" >> "${SHELL_PROFILE}"
            echo "[OK] Added ~/.cherenkov-lings/bin to ${SHELL_PROFILE}"
        fi
    fi
    echo "[NEXT] Run 'source ${SHELL_PROFILE:-~/.bashrc}' or restart your terminal to reload PATH."
else
    echo "[OK] PATH already contains ${INSTALL_DIR}."
fi

echo ""
echo "[DONE] Installation Complete!"
echo "Run 'cherenkov-lings dashboard' or 'cherenkov-lings watch --track=getting-started' to begin."
