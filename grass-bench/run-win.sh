#!/usr/bin/env bash
# Stages the Windows build of the bench on the shared drive and runs it from
# WSL through cmd.exe (the Windows renderer, not WSL's).
#
#   bench/run-win.sh                       # interactive, F7 panel
#   BENCH_EXIT_AFTER=20 BENCH_SCREENSHOT=shot.png BENCH_LOG=run.csv bench/run-win.sh
#   BENCH_SETTINGS='{"aa":"Taa","shading":"Custom"}' bench/run-win.sh
#
# Set BUILD=0 to skip the cargo build. Relative BENCH_LOG / BENCH_SCREENSHOT
# paths land in the stage dir.
set -euo pipefail
cd "$(dirname "$0")"

WIN_TARGET=x86_64-pc-windows-gnu
TARGET_DIR=$(pwd)/target
STAGE=/mnt/e/animation-playground-dev/bench

command -v cmd.exe >/dev/null 2>&1 || { echo "cmd.exe not found; run from WSL2."; exit 1; }
if [[ "${BUILD:-1}" == "1" ]]; then
    cargo build --target "$WIN_TARGET" --profile win-dev
fi

cmd.exe /C "taskkill /IM gpu-bench.exe /F >NUL 2>&1" || true
mkdir -p "$STAGE/assets/models" "$STAGE/assets/textures"
cp "$TARGET_DIR/$WIN_TARGET/win-dev/gpu-bench.exe" "$STAGE/"
rsync -a --delete assets/textures/ground/ "$STAGE/assets/textures/ground/"
[[ -d assets/textures/oak ]] && rsync -a --delete assets/textures/oak/ "$STAGE/assets/textures/oak/"
[[ -f assets/models/oak.glb ]] && cp -u assets/models/oak.glb "$STAGE/assets/models/"
cp -u assets/models/fabelgeist.glb "$STAGE/assets/models/"
cp -u assets/models/armor-*.png "$STAGE/assets/models/" 2>/dev/null || true
[[ -f bench.json ]] && cp -u bench.json "$STAGE/" || true

export WSLENV="BENCH_MAP_DEBUG:BENCH_SWITCH:BENCH_LOG:BENCH_EXIT_AFTER:BENCH_SCREENSHOT:BENCH_SCREENSHOT_AT:BENCH_GRASS_TIER_MASK:BENCH_MESH_NO_RANGE:BENCH_SETTINGS:RUST_LOG:${WSLENV:-}"
cd "$STAGE"
# DWM throttles unfocused windows to the refresh rate; bring the bench to the
# front once the window exists so scripted back-to-back runs measure the real
# frame rate.
( for delay in 2 2 2 2 3; do sleep "$delay"; powershell.exe -NoProfile -Command "(New-Object -ComObject WScript.Shell).AppActivate('GPU bench') | Out-Null" >/dev/null 2>&1; done ) &
./gpu-bench.exe
