from pathlib import Path
root=Path('crates/fabelgeist-gpu/src/data/gpu/texture')
p=root/'texture_cube/mod.rs';s=p.read_text();a=s.index('    pub fn face(');b=s.index('    pub fn set_side_image(',a);face=s[a:b];s=s[:a]+s[b:];a=s.index('    pub fn view_2d_array(');b=s.index('    pub fn render_face_shader(',a);array=s[a:b];s=s[:a]+s[b:];p.write_text(s)
p=root/'texture_cube/views.rs';v=p.read_text().replace('use std::sync::Arc;','use std::sync::Arc;\n\nuse anyhow::Result;').replace('TextureFormat, TextureViewError, TextureViewResult','CubeFace, TextureFormat, TextureView, TextureViewError, TextureViewResult');v=v.rsplit('\n}',1)[0]+'\n'+face+array+'}\n';p.write_text(v)
# Parent no longer owns direct face/whole view implementation imports.
p=root/'texture_cube/mod.rs';s=p.read_text().replace('CubeFace, Image, TextureFormat, TextureView, TextureViewError,','CubeFace, Image, TextureFormat, TextureView, TextureViewError,');p.write_text(s)
# Narrow the SDK comment to the actual descriptor policy rather than claiming
# the native API can never form any sRGB view of a storage-capable allocation.
for p in [root/'texture_2d/mod.rs',root/'texture_3d/mod.rs',root/'texture_2d/views.rs',root/'texture_3d/views.rs']:
    s=p.read_text().replace('// WebGPU Limitation: If a texture has STORAGE_BINDING, it cannot have an sRGB view.','// The default view inherits storage usage, which excludes sRGB formats.').replace('            // Fallback to linear counterpart to avoid validation error during creation.\n','').replace('            // Fallback to linear counterpart to avoid validation error.\n','');p.write_text(s)
p=Path('crates/fabelgeist-gpu/texture-views.md');s=p.read_text();s=s.replace('method does not change native validation, descriptor inference, cube layers,\nlabels, scope filtering or polling. Cached returns do not add a poll; native\ncreation retains its existing poll before returning.','methods preserve descriptor inference, cube layers, labels and validation\nfiltering. Cached returns do not poll. Native contexts that own their device\nread back the validation scope before returning; borrowed contexts skip that\nblocking readback, as do browser builds. Those contexts report native failures\nthrough the device\'s uncaptured-error handler rather than a synchronous\n`TextureViewError`. Initial texture and cube-face view construction follow the\nsame context policy and retain native causes when a scope is read back.');s=s.replace('Software Vulkan observations cover view sharing, readback bits, and\ncompute-pass propagation with explicitly authored public reflection metadata.\nThey do not establish hardware-wide, browser runtime, performance or calibrated\nrendering behavior. Browser builds retain the existing native scope differences.','Earlier software Vulkan observations covered view sharing, readback bits and\ncompute-pass propagation with authored public reflection metadata. Those\nhistorical observations do not validate the current source. They supply no\nhardware-wide, browser runtime, performance or calibrated rendering verdict.\nCurrent acceptance requires the hosted checks at the final PR revision.');p.write_text(s)
p=Path('.github/workflows/rust-quality.yml');s=p.read_text();anchor='      - name: Test resolved Bevy guard\n';step='''      - name: Test whole-texture format views
        env:
          XDG_RUNTIME_DIR: ${{ runner.temp }}/texture-view-runtime
        run: |
          mkdir -p "$XDG_RUNTIME_DIR" target/texture-view-validation
          chmod 700 "$XDG_RUNTIME_DIR"
          git rev-parse HEAD > target/texture-view-validation/source.txt
          set -o pipefail
          cargo test --locked --package fabelgeist-gpu --test texture_views -- --test-threads=1 --nocapture 2>&1 | tee target/texture-view-validation/integration.log
          cargo test --locked --package fabelgeist-gpu --doc 2>&1 | tee target/texture-view-validation/doctests.log
      - name: Upload texture-view validation evidence
        if: always()
        uses: actions/upload-artifact@v7
        with:
          name: texture-view-validation-${{ github.run_id }}-${{ github.run_attempt }}
          path: target/texture-view-validation/
          if-no-files-found: warn
''';assert s.count(anchor)==1;s=s.replace(anchor,step+anchor);p.write_text(s)
