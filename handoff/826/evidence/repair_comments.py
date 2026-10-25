from pathlib import Path
p=Path('crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs');s=p.read_text();a=s.index('    /// Whether shader, pipeline and texture construction');b=s.index('    pub blocking_validation:',a);s=s[:a]+'''    /// Whether shader, pipeline and texture construction synchronously waits
    /// for native validation results.
    ///
    /// Native contexts created here enable this; borrowed contexts disable it
    /// to avoid blocking their caller on GPU work. A blocking poll waits for
    /// the latest submission observed when polling begins and has no timeout.
    /// When disabled, these constructors install no validation scope, so
    /// native errors follow the device's configured error handling. Browser
    /// builds skip synchronous readback. Shader source is separately checked
    /// by Naga.
'''+s[b:];a=s.index('    /// Wait until everything submitted');b=s.index('    pub async fn submitted_work_done',a);s=s[:a]+'''    /// Wait until a queue marker and all work submitted before it complete.
    ///
    /// Callers can use this for backpressure before submitting more work.
    /// Native builds wait on an explicit submission index; submissions made
    /// after that marker need not finish before this call returns. Browser
    /// builds wait for the queue's completion callback instead.
'''+s[b:];a=s.index('    /// Borrowing a device must not opt into draining');b=s.index('    #[tokio::test]',a);s=s[:a]+'''    /// Borrowed contexts leave synchronous validation to the application.
    ///
    /// Construction paths consult this flag before waiting for native
    /// validation results. Wrapping an application's device must preserve the
    /// nonblocking policy even when the original context permits a wait.
'''+s[b:];s=s.replace('a borrowed device may have someone else submitting to it; draining              its queue would block until the driver gives up','wrapping a device must not enable synchronous validation readback');p.write_text(s)
p=Path('crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs');s=p.read_text();s=s.replace('''        // Reading the error scope means draining the queue, and on a borrowed
        // device -- one an application is submitting to every frame -- that
        // waits for a queue that never empties. The same rule the shader and
        // pipeline paths follow: only a context that owns its device drains it.
''','''        // Borrowed contexts avoid synchronous validation readback; see
        // WgpuContext::blocking_validation.
''');a=s.index("    /// The texture's pixels, read back as an image.");b=s.index('    pub fn write<',a);doc=s[a:b];s=s[:a]+s[b:];p.write_text(s)
p=Path('crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/image.rs');s=p.read_text();s=s.replace('    pub async fn read_image(',doc+'    pub async fn read_image(',1);p.write_text(s)
p=Path('crates/fabelgeist-gpu/src/data/gpu/texture/view_error.rs');s=p.read_text().replace('//! Failures selecting a format view of an existing whole texture.','//! Texture resource admission and whole-texture format-view failures.').replace('/// Failures from the three whole-texture `view_with_format` methods.','/// Missing texture resources and native whole-texture format-view failures.').replace('/// A different-format request has no native texture to view.','/// The operation requires a native texture, but none is present.').replace('/// A same-format request has no cached view to share.','/// The operation requires a cached view, but none is present.');p.write_text(s)
p=Path('crates/fabelgeist-gpu/texture-views.md');s=p.read_text().replace('Public tests cover cache absence, branch priority, native rejection and retained\ncauses. Earlier software Vulkan observations covered view sharing, readback bits and','Public tests cover cache absence, branch priority, native rejection and retained\ncauses. Earlier software Vulkan observations covered view sharing, readback\nbits and');p.write_text(s)
