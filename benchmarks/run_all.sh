#!/usr/bin/env bash
set -euo pipefail

# ── Controlled Benchmark Runner ──
# Runs Rust, Bun/TypeScript, and PHP benchmarks inside Docker containers
# with identical CPU and memory constraints for a fair comparison.
#
# Resource limits (same for all three):
#   CPU:    1 core
#   Memory: 512 MB
#
# Each Dockerfile clones the relevant repo from GitHub, so the comparison
# is reproducible by anyone.
#
# Usage: ./benchmarks/run_all.sh

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RESULTS_DIR="$SCRIPT_DIR/results"

CPU_LIMIT="1"
MEM_LIMIT="512m"

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

step() { echo -e "\n${CYAN}━━━ $1 ━━━${NC}"; }

# ── 1. Rust ──────────────────────────────────────────────────────────────────

step "Building Rust benchmark container"
docker build -t fiscal-bench-rust "$SCRIPT_DIR/rust" 2>&1 | tail -3

step "Running Rust benchmark (--cpus=$CPU_LIMIT --memory=$MEM_LIMIT)"
docker run --rm --cpus="$CPU_LIMIT" --memory="$MEM_LIMIT" \
  fiscal-bench-rust > "$RESULTS_DIR/rust.json"
echo -e "${GREEN}OK${NC} → results/rust.json"

# ── 2. Bun/TypeScript ────────────────────────────────────────────────────────

step "Building Bun benchmark container"
docker build -t fiscal-bench-bun "$SCRIPT_DIR/bun" 2>&1 | tail -3

step "Running Bun benchmark (--cpus=$CPU_LIMIT --memory=$MEM_LIMIT)"
docker run --rm --cpus="$CPU_LIMIT" --memory="$MEM_LIMIT" \
  fiscal-bench-bun > "$RESULTS_DIR/bun.json"
echo -e "${GREEN}OK${NC} → results/bun.json"

# ── 3. PHP ───────────────────────────────────────────────────────────────────

step "Building PHP benchmark container"
docker build -t fiscal-bench-php "$SCRIPT_DIR/php" 2>&1 | tail -3

step "Running PHP benchmark (--cpus=$CPU_LIMIT --memory=$MEM_LIMIT)"
docker run --rm --cpus="$CPU_LIMIT" --memory="$MEM_LIMIT" \
  fiscal-bench-php > "$RESULTS_DIR/php.json"
echo -e "${GREEN}OK${NC} → results/php.json"

# ── Summary ──────────────────────────────────────────────────────────────────

step "Benchmark complete"
echo -e "Resource limits: ${YELLOW}${CPU_LIMIT} CPU, ${MEM_LIMIT} RAM${NC}"
echo ""
echo "Results:"
for f in "$RESULTS_DIR"/*.json; do
  count=$(python3 -c "import json; print(len(json.load(open('$f'))))" 2>/dev/null || echo "?")
  echo "  $(basename $f): $count benchmarks"
done
echo ""
echo -e "Docker images (reproducible by anyone):"
echo "  fiscal-bench-rust  → clones github.com/JoaoHenriqueBarbosa/fiscal-rs"
echo "  fiscal-bench-bun   → clones github.com/JoaoHenriqueBarbosa/FinOpenPOS"
echo "  fiscal-bench-php   → clones github.com/JoaoHenriqueBarbosa/sped-nfe (branch: benchmarks)"
