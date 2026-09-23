# WebGL2 grass tests

Build the static site from the repository root:

```bash
python3 scripts/build_grass_demo.py --output target/grass-demo/site
python3 -m http.server 8080 --directory target/grass-demo/site
```

Open one of these paths:

- `/instanced-cpu-culled/`: Instanced, CPU culled.
- `/no-instancing-mesh-chunks/`: No-instancing, Mesh chunks, with analytic
  affector bending.
- `/no-instancing-displacement/`: No-instancing, Displacement, sampling
  displacement and trail maps.
- `/no-instancing-textured-sprites/`: No-instancing, Textured sprites (curved).
  This is `CardsCurved`, the final mode in the full benchmark list.

These pages always load the WebGL2 engine, even on a WebGPU browser. They
lock the mode and start at 12% density and 18 m range, with no MSAA, shadows,
trees, or characters. The small panel controls density, range, camera orbit,
and (on the displacement page) trails. The main `/` page opens the
Instanced, CPU culled test. `/instanced-no-instancing-advanced/` retains the
full benchmark panel and requires
WebGPU, including for the original tree LOD and GPU-compute paths. Browsers
without WebGPU see a message pointing back to the laptop tests.
All demo labels and routes start with Instanced or No-instancing. The mixed
benchmark is named Instanced / No-instancing, Advanced, and is separate from
the two navigation groups. `/instanced-eidolon/` opens Instanced, Eidolon
with the full controls available and requires WebGPU. All six pages share
the same engines and assets.

## Renderer contract

WebGL2 has no storage buffers or compute shaders. The `downlevel` Cargo
feature replaces affector storage bindings with fixed uniform arrays and
omits the eidolon compute plugin. Object parameters (128 entries) and sprite
parameters (32 entries) share one uniform binding on both builds, keeping
the material within the minimum 11 uniform bindings per shader stage.
Repeated meshes and respawns reuse identical parameter records. Normal builds
retain the shared affector storage buffer and update it in place without
changing material assets.

Displacement and trail maps use `Rgba8UnormSrgb` without HDR on downlevel builds.
Signed horizontal offsets are encoded as `(metres * 32 + 128) / 255`, giving
approximately +/-4 m range with 8-bit sRGB quantization. Hardware converts
between linear shader values and sRGB storage on writes and reads. Excessive
overlapping pushes saturate; the stripped tests use two affectors. Crush and
scorch also use 8-bit channels. Normal builds retain `Rgba16Float` maps. Wind remains
analytic per blade in both builds.

`feed_instanced_grass` updates only the selected instanced path. Map cameras
run only in the displacement mode; the trail camera additionally requires
trails to be enabled. Downlevel mesh chunks use the CPU streaming window as
a hard range boundary, avoiding Bevy 0.19's inconsistent dither-range uniform
layout. Normal builds retain the dithered range fade. Chunk origins sit at
terrain height so short-range distance tests measure from the grass.

## Local verification

Run `python3 grass-bench/scripts/smoke-downlevel.py` from the repository
root for bounded runs of the original three routes, validation logs, and
screenshots.

From `grass-bench/`, run each route interactively under desktop WebGL2 limits:

```bash
BENCH_DOWNLEVEL=1 BENCH_DEMO=instanced-cpu-culled cargo run --features downlevel
BENCH_DOWNLEVEL=1 BENCH_DEMO=no-instancing-mesh-chunks cargo run --features downlevel
BENCH_DOWNLEVEL=1 BENCH_DEMO=no-instancing-displacement cargo run --features downlevel
BENCH_DOWNLEVEL=1 BENCH_DEMO=no-instancing-textured-sprites cargo run --features downlevel
```

The Cargo feature is required: the environment variable constrains the
renderer but cannot remove compiled storage bindings. Desktop constrained
limits do not emulate every WebGL capability or GLSL translation detail;
also test the built pages in a browser. Success requires no render validation
errors and visible, animated grass in all four WebGL2 modes.

For bounded native runs, add `BENCH_EXIT_AFTER=20`, `BENCH_SCREENSHOT_AT=12`,
and `BENCH_SCREENSHOT=/absolute/path/to/target/screenshot.png`. Keep logs and
screenshots under ignored `target/` directories. Deployment remains a
separate step after reviewing the local runs.
