#!/usr/bin/env bash

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                 🚀 GIT-TREK INSTALLER 🚀                     ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found! Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust detected: $(rustc --version)"
echo ""

# Build
echo "🔨 Building git-trek..."
cargo build --release

# Install
echo "📦 Installing to cargo bin directory..."
cargo install --path .

echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                    ✨ INSTALLATION COMPLETE ✨                ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "🎮 Start trekking with: git-trek"
echo ""
echo "Pro tip: Add this alias to your shell config:"
echo "  alias gt='git-trek'"
echo ""