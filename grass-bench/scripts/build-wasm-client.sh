#!/usr/bin/env bash
# Browser build of the bench, ported from skatepark/scripts/build-wasm-client.sh.
#
#   bench/scripts/build-wasm-client.sh [release|dev|webgl2|trace]
#
# Output: bench/web/public (or $OUT_DIR for deploys). Serve it with the
# no-store + version-prefix-stripping server in `just bench-web`.
set -euo pipefail

cd "$(dirname "$0")/.."

wasm_target="wasm32-unknown-unknown"
target_dir="$(cargo metadata --format-version 1 --no-deps 2>/dev/null | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
# Diagnostic features are selected only by this positional mode. Environment
# variables must never turn a regular dev or deploy build into a trace build.
mode="${1:-release}"
case "${mode}" in
    release)
        profile="wasm-release"
        features="webgpu"
        # Deploys also ship the webgl2 engine; index.html probes
        # navigator.gpu at boot and imports whichever build fits.
        webgl2_fallback=1
        ;;
    dev)
        if [[ -n "${OUT_DIR:-}" ]]; then
            echo "dev mode cannot write deploy output (OUT_DIR is set)" >&2
            exit 2
        fi
        profile="wasm-dev"
        features="webgpu"
        ;;
    # dev minus the webgpu feature -> bevy's default webgl2 backend. What Linux
    # Firefox (no WebGPU) would run; for testing custom shaders off WebGPU.
    webgl2)
        if [[ -n "${OUT_DIR:-}" ]]; then
            echo "webgl2 mode cannot write deploy output (OUT_DIR is set)" >&2
            exit 2
        fi
        profile="wasm-dev"
        features="downlevel"
        ;;
    trace)
        if [[ -n "${OUT_DIR:-}" ]]; then
            echo "trace mode cannot write deploy output (OUT_DIR is set)" >&2
            exit 2
        fi
        profile="wasm-trace"
        features="trace,webgpu"
        ;;
    *)
        echo "usage: bench/scripts/build-wasm-client.sh [release|dev|webgl2|trace]" >&2
        exit 2
        ;;
esac
out_dir="${OUT_DIR:-web/public}"
wasm_in="${target_dir}/${wasm_target}/${profile}/gpu-bench.wasm"
wasm_out="${out_dir}/bench_bg.wasm"
required_bindgen="$(awk '/name = "wasm-bindgen"/{p=1} p && /version = /{gsub(/"/, "", $3); print $3; exit}' Cargo.lock)"
local_bindgen="${target_dir}/wasm-bindgen-cli/${required_bindgen}/bin/wasm-bindgen"

if ! rustup target list --installed | grep -qx "${wasm_target}"; then
    rustup target add "${wasm_target}"
fi

if [[ -n "${WASM_BINDGEN:-}" ]]; then
    bindgen_bin="${WASM_BINDGEN}"
elif command -v wasm-bindgen >/dev/null 2>&1 \
    && [[ "$(wasm-bindgen --version | awk '{ print $2 }')" == "${required_bindgen}" ]]; then
    bindgen_bin="wasm-bindgen"
else
    bindgen_bin="${local_bindgen}"
fi

if [[ ! -x "${bindgen_bin}" ]]; then
    cargo install \
        --root "${target_dir}/wasm-bindgen-cli/${required_bindgen}" \
        --version "${required_bindgen}" \
        --locked \
        wasm-bindgen-cli
fi

# Only what the bench loads (assets/ is a symlink to the repo's assets/).
shipped_assets=(
    textures/ground
    textures/oak
    models/oak.glb
    models/fabelgeist.glb
)
asset_files() {
    for a in "${shipped_assets[@]}"; do
        [[ -e "assets/$a" ]] && find "assets/$a" -type f -print0
    done
    find assets/models -maxdepth 1 -name 'armor-*.png' -print0
}

# Content hash of shipped assets → URL prefix baked into the wasm (see
# AssetPlugin in main.rs). A changed asset is a changed URL, so browser caches
# can never serve a stale glb. The dev server strips the prefix.
WEB_ASSET_PREFIX="v$(asset_files | sort -z | xargs -0 md5sum | md5sum | cut -c1-8)"
export WEB_ASSET_PREFIX

feature_args=()
[[ -n "${features}" ]] && feature_args=(--features "${features}")

# simd128: glam gates its SIMD backend on it; measured on skatepark it is both
# faster and smaller. Not +relaxed-simd (non-deterministic FMA contraction).
RUSTFLAGS="${RUSTFLAGS:-} -C target-feature=+simd128" \
    cargo build --profile "${profile}" --target "${wasm_target}" "${feature_args[@]}"

if [[ "${webgl2_fallback:-0}" == "1" ]]; then
    RUSTFLAGS="${RUSTFLAGS:-} -C target-feature=+simd128" \
        cargo build --profile wasm-release-webgl2 --features downlevel --target "${wasm_target}"
fi

rm -rf "${out_dir}"
mkdir -p "${out_dir}/assets"

# wasm-bindgen strips DWARF during its rewrite unless told not to.
bindgen_debug=()
[[ "${profile}" == "wasm-dev" ]] && bindgen_debug=(--keep-debug)

"${bindgen_bin}" \
    --out-dir "${out_dir}" \
    --target web \
    --no-typescript \
    --out-name bench \
    "${bindgen_debug[@]}" \
    "${wasm_in}"

if [[ "${webgl2_fallback:-0}" == "1" ]]; then
    "${bindgen_bin}" \
        --out-dir "${out_dir}" \
        --target web \
        --no-typescript \
        --out-name bench_webgl2 \
        "${target_dir}/${wasm_target}/wasm-release-webgl2/gpu-bench.wasm"
fi

cp web/index.html "${out_dir}/"

while IFS= read -r -d '' f; do
    mkdir -p "${out_dir}/$(dirname "$f")"
    cp "$f" "${out_dir}/$f"
done < <(asset_files)

# wasm-opt is OFF by default: brotli already finds what binaryen rearranges
# (skatepark measured ~6% on the wire for 45 s of build). WASM_OPT=1 to A/B.
if [[ "${WASM_OPT:-0}" != "1" ]]; then
    echo "Skipping wasm-opt (default; WASM_OPT=1 to enable)"
elif command -v wasm-opt >/dev/null 2>&1; then
    wasm-opt --enable-bulk-memory --enable-reference-types --enable-sign-ext \
        --enable-multivalue --enable-mutable-globals --enable-nontrapping-float-to-int \
        -O2 -g "${wasm_out}" -o "${wasm_out}"
else
    echo "wasm-opt not found; skipping optimization"
fi

# Version every URL index.html hands out: assets ride the content-hash prefix
# baked into the wasm, the engine pair gets ?v=<hash of the built wasm>.
build_hash="$(md5sum "${wasm_out}" | cut -c1-8)"
prefetch="$(cd "${out_dir}" && find assets -type f \
    \( -name '*.glb' -o -name '*.png' -o -name '*.ktx2' \) -printf '%s\t%p\n' \
    | sort -rn | head -8 | cut -f2 \
    | sed 's|.*|"./&",|' | tr -d '\n' | sed 's|,$||')"
sed -i \
    -e "s|const WASM_BYTES = 0;|const WASM_BYTES = $(stat -c%s "${wasm_out}");|" \
    -e "s|const PREFETCH = \[\];|const PREFETCH = [${prefetch}];|" \
    -e "s|\./assets/|./${WEB_ASSET_PREFIX}/assets/|g" \
    -e "s|\./bench\.js|./bench.js?v=${build_hash}|g" \
    -e "s|\./bench_bg\.wasm|./bench_bg.wasm?v=${build_hash}|g" \
    "${out_dir}/index.html"

webgl2_wasm="${out_dir}/bench_webgl2_bg.wasm"
if [[ -f "${webgl2_wasm}" ]]; then
    webgl2_hash="$(md5sum "${webgl2_wasm}" | cut -c1-8)"
    sed -i \
        -e "s|const WASM_BYTES_WEBGL2 = 0;|const WASM_BYTES_WEBGL2 = $(stat -c%s "${webgl2_wasm}");|" \
        -e "s|\./bench_webgl2\.js|./bench_webgl2.js?v=${webgl2_hash}|g" \
        -e "s|\./bench_webgl2_bg\.wasm|./bench_webgl2_bg.wasm?v=${webgl2_hash}|g" \
        "${out_dir}/index.html"
fi

# Precompress for a `file_server { precompressed br gzip }` deploy.
if [[ -n "${OUT_DIR:-}" ]]; then
    if ! command -v brotli >/dev/null 2>&1; then
        echo "brotli not found; shipping gzip only (apt install brotli)"
    fi
    while IFS= read -r -d '' f; do
        gzip -9 -kf "${f}"
        if command -v brotli >/dev/null 2>&1; then
            brotli -f -q 11 -o "${f}.br" "${f}"
        fi
    done < <(
        printf '%s\0' "${wasm_out}" "${out_dir}/bench.js"
        [[ -f "${webgl2_wasm}" ]] && printf '%s\0' "${webgl2_wasm}" "${out_dir}/bench_webgl2.js"
        find "${out_dir}/assets" -type f -name '*.glb' -print0
    )
fi

echo "WASM build complete: ${out_dir}/ ($(du -sh "${out_dir}" | cut -f1))"
