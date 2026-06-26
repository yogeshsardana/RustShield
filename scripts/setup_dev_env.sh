#!/usr/bin/env bash
#
# RustShield Development Environment Setup Script
#
# Installs and configures all dependencies needed for RustShield development.
# Supports Ubuntu/Debian and Fedora/RHEL.

set -euo pipefail

# ─── Configuration ──────────────────────────────────────────────────────────
RUST_NIGHTLY="nightly-2026-05-01"
VERUS_REPO="https://github.com/verus-lang/verus.git"
LINUX_REPO="https://github.com/Rust-for-Linux/linux.git"
LINUX_BRANCH="rust-nightly"

# ─── Color Output ──────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'
info()  { echo -e "${BLUE}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
err()   { echo -e "${RED}[ERROR]${NC} $*"; }

# ─── Detect OS ──────────────────────────────────────────────────────────────
detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$ID
    else
        err "Cannot detect OS"
        exit 1
    fi
    info "Detected OS: ${OS} ${VERSION_ID}"
}

# ─── Install System Dependencies ────────────────────────────────────────────
install_system_deps() {
    info "Installing system dependencies..."

    case "${OS}" in
        ubuntu|debian)
            sudo apt-get update
            sudo apt-get install -y \
                build-essential \
                bc \
                bison \
                flex \
                libssl-dev \
                make \
                pkg-config \
                clang \
                llvm \
                libelf-dev \
                libcap-dev \
                linux-tools-common \
                linux-tools-generic \
                python3 \
                python3-pip \
                git
            ;;
        fedora|rhel|centos)
            sudo dnf install -y \
                gcc \
                make \
                bc \
                bison \
                flex \
                openssl-devel \
                elfutils-libelf-devel \
                libcap-devel \
                clang \
                llvm \
                python3 \
                python3-pip \
                git \
                kernel-devel
            ;;
        *)
            err "Unsupported OS: ${OS}"
            exit 1
            ;;
    esac
    ok "System dependencies installed"
}

# ─── Install Rust ───────────────────────────────────────────────────────────
install_rust() {
    if command -v rustup &>/dev/null; then
        info "rustup already installed, updating..."
        rustup update
    else
        info "Installing rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        . "${HOME}/.cargo/env"
    fi

    info "Installing Rust ${RUST_NIGHTLY}..."
    rustup install "${RUST_NIGHTLY}"
    rustup component add \
        rust-src \
        llvm-tools \
        rustc-dev \
        --toolchain "${RUST_NIGHTLY}"
    rustup target add \
        x86_64-unknown-linux-gnu \
        aarch64-unknown-linux-gnu \
        --toolchain "${RUST_NIGHTLY}"

    ok "Rust ${RUST_NIGHTLY} installed"
}

# ─── Install Verus ──────────────────────────────────────────────────────────
install_verus() {
    if [ -d "${HOME}/verus" ]; then
        info "Verus already installed at ${HOME}/verus"
        return
    fi

    info "Cloning Verus..."
    git clone "${VERUS_REPO}" "${HOME}/verus"

    info "Building Verus..."
    cd "${HOME}/verus"
    source tools/activate
    vargo build --release

    # Add to PATH
    echo 'export PATH="$HOME/verus/source/target/release:$PATH"' >> "${HOME}/.bashrc"
    ok "Verus installed at ${HOME}/verus"
}

# ─── Setup RustShield ───────────────────────────────────────────────────────
setup_rustshield() {
    info "Setting up RustShield workspace..."

    # Create rust-toolchain.toml if not present
    if [ ! -f "rust-toolchain.toml" ]; then
        cat > rust-toolchain.toml <<EOF
[toolchain]
channel = "${RUST_NIGHTLY}"
components = ["rust-src", "llvm-tools", "rustc-dev"]
targets = ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"]
EOF
        ok "Created rust-toolchain.toml"
    fi

    # Build workspace
    cargo check --workspace 2>&1 | tail -5
    ok "RustShield workspace builds successfully"
}

# ─── Main ───────────────────────────────────────────────────────────────────
main() {
    echo ""
    echo "RustShield Development Environment Setup"
    echo "════════════════════════════════════════"
    echo ""

    detect_os
    install_system_deps
    install_rust
    install_verus
    setup_rustshield

    echo ""
    echo "════════════════════════════════════════"
    echo "  Setup complete!"
    echo ""
    echo "  Next steps:"
    echo "  1. Clone the Linux kernel with Rust support:"
    echo "     git clone ${LINUX_REPO} --branch ${LINUX_BRANCH} ~/linux"
    echo ""
    echo "  2. Build the rust_driver_hotswap module:"
    echo "     cd kernel/rust_driver_hotswap && make"
    echo ""
    echo "  3. Run the demo:"
    echo "     ./scripts/demo.sh --driver=e1000e"
    echo "════════════════════════════════════════"
}

main
