#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXECUTABLE_PATH="$SCRIPT_DIR/target/release/llm-cli"
ALIAS_LINE="alias llm-cli='$EXECUTABLE_PATH'"

echo "=== Building llm-cli in release mode ==="
cargo build --release

if [ ! -f "$EXECUTABLE_PATH" ]; then
    echo "ERROR: Release executable not found at $EXECUTABLE_PATH"
    exit 1
fi

echo ""
echo "=== Adding alias to shell profiles ==="

add_alias_to_file() {
    local file="$1"
    if [ ! -f "$file" ]; then
        touch "$file"
        echo "Created $file"
    fi

    if grep -qF "alias llm-cli=" "$file" 2>/dev/null; then
        echo "Alias already exists in $file"
    else
        echo "" >> "$file"
        echo "# llm-cli alias - added by build script" >> "$file"
        echo "$ALIAS_LINE" >> "$file"
        echo "Added alias to $file"
    fi
}

add_alias_to_file "$HOME/.bashrc"
add_alias_to_file "$HOME/.zshrc"

echo ""
echo "=== Done ==="
echo "Executable: $EXECUTABLE_PATH"
echo ""
echo "To use in current shell, run:"
echo "  source ~/.bashrc  # or"
echo "  source ~/.zshrc"
echo ""
echo "New shell sessions will have the alias automatically."
