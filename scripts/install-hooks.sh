#!/usr/bin/env bash
set -euo pipefail

# Installs git hooks for the fiscal-rs repository.
# Usage: ./scripts/install-hooks.sh

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOOKS_SRC="$REPO_ROOT/scripts/hooks"
HOOKS_DST="$REPO_ROOT/.git/hooks"

for hook in "$HOOKS_SRC"/*; do
    name="$(basename "$hook")"
    cp "$hook" "$HOOKS_DST/$name"
    chmod +x "$HOOKS_DST/$name"
    echo "Instalado: $name"
done

echo "Hooks instalados com sucesso!"
