#!/usr/bin/env bash
# Runs knob presets on Windows back to back (SMOKE_SECS each, default 20) and
# prints, per preset, the grass census, any error, and the median of the 2 s
# frame-time samples after warm-up (t >= 7 s) with the GPU passes of the last
# sample. Screenshots land in the stage dir as smoke_<n>.png, CSVs as
# smoke_<n>.csv.
#
#   bench/scripts/smoke-win.sh                       # the default presets
#   bench/scripts/smoke-win.sh '{"instancing":"Simple"}' '{"instancing":"Eidolon"}'
set -uo pipefail
cd "$(dirname "$0")/.."
STAGE=/mnt/e/animation-playground-dev/bench

presets=(
    '{"tree_model":"Oak","instancing":"Simple","shading":"Standard"}'
    '{"tree_model":"Oak","instancing":"Eidolon","shading":"Standard"}'
    '{"tree_model":"Oak","instancing":"Simple","shading":"Custom","foliage_alpha":"OpaqueDiscard"}'
    '{"tree_model":"Oak","instancing":"Simple","shading":"LineBoil","aa":"Taa","depth_prepass":true,"occlusion_culling":true}'
)
if [[ $# -gt 0 ]]; then
    presets=("$@")
fi

n=0
for cfg in "${presets[@]}"; do
    n=$((n + 1))
    echo "=== [$n] $cfg"
    # A launch occasionally yields no samples (window/focus race with the
    # previous instance shutting down); one retry covers it.
    for attempt in 1 2; do
        rm -f "$STAGE/smoke_$n.csv"
        sleep 2
        BUILD=0 BENCH_SETTINGS="$cfg" BENCH_EXIT_AFTER="${SMOKE_SECS:-20}" \
            BENCH_SCREENSHOT="smoke_$n.png" BENCH_LOG="smoke_$n.csv" \
            RUST_LOG=info,wgpu=warn,naga=warn ./run-win.sh 2>&1 \
            | grep -E 'grass (near|far|vista|near_edge)|grass: |error|panic|ERROR' \
            | sed 's/\x1b\[[0-9;]*m//g' | sed 's/^.*INFO gpu_bench::grass: //' | cut -c1-160 | head -8
        summary=$(python3 scripts/summarize-csv.py "$STAGE/smoke_$n.csv")
        echo "$summary"
        [[ "$summary" == *"no samples"* || "$summary" == *"no csv"* ]] || break
        echo "  (retrying)"
    done
done
