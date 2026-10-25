from pathlib import Path
root=Path('crates/fabelgeist-gpu/src/data/gpu/texture')
# Extract whole-view selection into focused shape-owned modules before expanding
# the already oversized allocation/rendering files. No tests are authored.
for shape,typ in [('texture_2d','Texture2d'),('texture_3d','Texture3d'),('texture_cube','TextureCube')]:
    p=root/shape/'mod.rs'; s=p.read_text()
    start=s.index('    pub fn view_with_format(')
    stop=s.index('\n    pub fn ',start+5) if shape!='texture_3d' else s.index('\n}',start)
    method=s[start:stop]
    # Actual native scope retrieval must use the already-owned context policy.
    context='_ctx' if shape=='texture_cube' else '_context'
    if shape=='texture_cube':
        method=method.replace('let error_scope = _ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);','let error_scope = _ctx.blocking_validation.then(|| {\n            _ctx.device.push_error_scope(wgpu::ErrorFilter::Validation)\n        });')
    else:
        method=method.replace('let error_scope = _context\n            .device\n            .push_error_scope(wgpu::ErrorFilter::Validation);','let error_scope = _context.blocking_validation.then(|| {\n            _context.device.push_error_scope(wgpu::ErrorFilter::Validation)\n        });')
    method=method.replace('#[cfg(not(target_arch = "wasm32"))]\n        {\n            let _ = '+context+'.device.poll', '#[cfg(not(target_arch = "wasm32"))]\n        if let Some(error_scope) = error_scope {\n            let _ = '+context+'.device.poll')
    # Remove stale speculative comments while this actual policy is being fixed.
    method=method.replace('            // Optional: log or return a warning if we had a way to do so without noise.\n','')
    (root/shape/'views.rs').write_text('use std::sync::Arc;\n\nuse super::'+typ+';\nuse crate::{\n    data::gpu::texture::{TextureFormat, TextureViewError, TextureViewResult},\n    globals::WgpuContext,\n};\n\nimpl '+typ+' {\n'+method+'\n}\n')
    s=s[:start]+s[stop:]
    s=s.replace('TextureFormat, TextureViewError, TextureViewResult','TextureFormat, TextureViewError')
    s=s.replace('TextureViewError, TextureViewResult,','TextureViewError,')
    s=s.replace('unsafe impl Send for '+typ+' {}\nunsafe impl Sync for '+typ+' {}\n','')
    # Initial allocation failures retain the SDK error through the generic
    # constructor envelope (constructors also return native-size/image failures).
    s=s.replace('anyhow::anyhow!("WGPU '+typ+' Creation Error: {}", err)', 'anyhow::Error::new(err).context("WGPU '+typ+' Creation Error")')
    if shape!='texture_2d':
        s=s.replace('let error_scope = context\n            .device\n            .push_error_scope(wgpu::ErrorFilter::Validation);','let error_scope = context.blocking_validation.then(|| {\n            context.device.push_error_scope(wgpu::ErrorFilter::Validation)\n        });',1)
        s=s.replace('#[cfg(not(target_arch = "wasm32"))]\n        {\n            let _ = context.device.poll', '#[cfg(not(target_arch = "wasm32"))]\n        if let Some(error_scope) = error_scope {\n            let _ = context.device.poll',1)
    # Make import/module order conform to crates/AGENTS.md while touching modules.
    s=s.replace('mod image;\npub use image::*;\n\n','').replace('mod image_data;\n\n','')
    # Inject module declarations after imports, before the struct's derive.
    decl=('mod image;\nmod views;\n\npub use image::*;\n\n' if shape=='texture_2d' else 'mod image_data;\nmod views;\n\n' if shape=='texture_cube' else 'mod views;\n\n')
    # Re-export is an import and must precede module declarations.
    if shape=='texture_2d': decl='pub use image::*;\n\nmod image;\nmod views;\n\n'
    s=s.replace('#[derive(Clone, Debug)]',decl+'#[derive(Clone, Debug)]',1)
    p.write_text(s)
# Existing borrowed-context policy applies to optional-context face creation too.
p=root/'texture_cube/mod.rs'; s=p.read_text()
s=s.replace('let error_scope = context.map(|c| c.device.push_error_scope(wgpu::ErrorFilter::Validation));','let error_scope = context\n            .filter(|context| context.blocking_validation)\n            .map(|context| context.device.push_error_scope(wgpu::ErrorFilter::Validation));')
s=s.replace('anyhow::anyhow!("WGPU TextureCube Face View Error: {}", err)','anyhow::Error::new(err).context("WGPU TextureCube Face View Error")')
s=s.replace('texture: tex.texture.as_ref().unwrap(),','texture: tex.texture.as_ref().ok_or(TextureViewError::Uninitialized)?,')
s=s.replace('.unwrap_or(256.0);','.unwrap_or(super::DEFAULT_TEXTURE_EDGE_PIXELS);')
p.write_text(s)
p=root/'texture_2d/mod.rs'; s=p.read_text()
s=s.replace('Vec2::new(256.0, 256.0)','Vec2::new(super::DEFAULT_TEXTURE_EDGE_PIXELS, super::DEFAULT_TEXTURE_EDGE_PIXELS)')
s=s.replace('texture: tex.texture.as_ref().unwrap(),','texture: tex.texture.as_ref().ok_or(TextureViewError::Uninitialized)?,')
s=s.replace('self.view.as_ref().unwrap(),','self.view.as_ref().ok_or(TextureViewError::MissingView)?,').replace('target.view.as_ref().unwrap(),','target.view.as_ref().ok_or(TextureViewError::MissingView)?,')
p.write_text(s)
p=root/'mod.rs'; s=p.read_text(); s+='\n/// Default edge length, in pixels, for an unspecified texture size.\nconst DEFAULT_TEXTURE_EDGE_PIXELS: f32 = 256.0;\n'; p.write_text(s)
# Existing unchanged consumers must reject missing source textures before
# copying anything into a native command encoder.
p=Path('crates/fabelgeist-compute/src/distance_field.rs'); s=p.read_text()
s=s.replace('use fabelgeist_gpu::data::gpu::texture::Texture3d;','use fabelgeist_gpu::data::gpu::texture::{Texture3d, TextureViewError};')
s=s.replace('texture: io.texture.as_ref().unwrap(),','texture: io.texture.as_ref().ok_or(TextureViewError::Uninitialized)?,').replace('texture: temp_tex.texture.as_ref().unwrap(),','texture: temp_tex.texture.as_ref().ok_or(TextureViewError::Uninitialized)?,')
p.write_text(s)
# Correct the borrowed-device policy description; this is not wiki prose.
p=Path('crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs'); s=p.read_text()
s=s.replace('/// Whether shader and pipeline creation may drain the queue to read back a','/// Whether shader, pipeline and texture view construction may drain the queue to read back a')
s=s.replace('    /// which costs nothing: the naga validator has already run over the\n    /// source by that point and catches essentially everything.\n    ///\n    /// Only the shader and pipeline paths honour this so far. Texture\n    /// creation drains the queue the same way and would hang a borrowed\n    /// device identically -- nothing reaches it on a borrowed context today,\n    /// but that is the place to look if one ever does.','    /// including failures from texture allocation and view construction.\n    /// Shader source is checked separately by the Naga validator; native\n    /// resource validation still belongs to the device.')
p.write_text(s)
