from pathlib import Path
base=Path('crates/fabelgeist-gpu/src/data/gpu/texture')
for shape,dim in [('2d','D2'),('3d','D3'),('cube','Cube')]:
 name={'2d':'Texture2d','3d':'Texture3d','cube':'TextureCube'}[shape]
 p=base/f'texture_{shape}/mod.rs';s=p.read_text()
 for field in ['texture','view','size','format','usage']:
  s=s.replace('    pub '+field+':','    pub(super) '+field+':')
 start=s.index('pub struct '+name);end=s.index('\n}',start)
 s=s[:end]+'\n    pub(super) device: Option<Arc<wgpu::Device>>,'+s[end:]
 s=s.replace('            texture: None,','            device: None,\n            texture: None,',1)
 s=s.replace('            texture: Some(Arc::new(texture)),','            device: Some(context.device.clone()),\n            texture: Some(Arc::new(texture)),',1)
 p.write_text(s)
 p=base/f'texture_{shape}/views.rs';s=p.read_text()
 s=s.replace('        _context: &WgpuContext,','        context: &WgpuContext,').replace('        _ctx: &WgpuContext,','        context: &WgpuContext,').replace('_context','context').replace('_ctx','context')
 # Public SDK boundary projects only the native handle, keeping selection metadata inside owner.
 start=s.index('        if format == self.format')
 s=s[:start]+'''        Ok(self.select_view(context, format)?.view)
    }

    pub(super) fn select_view(
        &self,
        context: &WgpuContext,
        format: TextureFormat,
    ) -> TextureViewResult<super::super::SelectedTextureView> {
'''+s[start:]
 s=s.replace('            return self.view.clone().ok_or(TextureViewError::MissingView);','''            let view = self.view.clone().ok_or(TextureViewError::MissingView)?;
            return Ok(super::super::SelectedTextureView::new(
                view, self.effective_format(format).into(), wgpu::TextureViewDimension::'''+dim+''',
                self.device.clone().ok_or(TextureViewError::Uninitialized)?,
            ));''',1)
 pos=s.index('        let mut requested_format = format;')
 s=s[:pos]+'''        self.ensure_device(context)?;
'''+s[pos:]
 s=s.replace('        Ok(Arc::new(view))','''        Ok(super::super::SelectedTextureView::new(
            Arc::new(view), requested_format.into(), wgpu::TextureViewDimension::'''+dim+''',
            self.device.clone().ok_or(TextureViewError::Uninitialized)?,
        ))''',1)
 # state access and removal methods, shared metadata is immutable to other modules.
 end=s.rfind('\n}')
 extent={'2d':'wgpu::Extent3d { width: self.size.0, height: self.size.1, depth_or_array_layers: 1 }','3d':'wgpu::Extent3d { width: self.size.0, height: self.size.1, depth_or_array_layers: self.size.2 }','cube':'wgpu::Extent3d { width: self.size, height: self.size, depth_or_array_layers: 6 }'}[shape]
 s=s[:end]+'''
    pub fn format(&self) -> TextureFormat { self.format }
    pub fn usage(&self) -> wgpu::TextureUsages { self.usage }
    pub fn extent(&self) -> wgpu::Extent3d { '''+extent+''' }
    pub fn native_texture(&self) -> Option<&Arc<wgpu::Texture>> { self.texture.as_ref() }
    pub fn cached_view(&self) -> Option<&Arc<wgpu::TextureView>> { self.view.as_ref() }

    /// Remove the cached view without admitting a replacement or changing metadata.
    pub fn clear_cached_view(&mut self) { self.view = None; }
    /// Detach the allocation port; a previously cached view remains usable.
    pub fn clear_native_texture(&mut self) { self.texture = None; }

    pub(super) fn effective_format(&self, format: TextureFormat) -> TextureFormat {
        if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            format.linear_counterpart()
        } else { format }
    }

    pub(super) fn ensure_device(&self, context: &WgpuContext) -> TextureViewResult<()> {
        if self.device.as_deref() != Some(context.device.as_ref()) {
            return Err(TextureViewError::WrongDevice);
        }
        Ok(())
    }
'''+s[end:]
 if shape=='cube':
  s=s.replace('            .ok_or_else(|| anyhow::anyhow!("Texture is not initialized"))?;','            .ok_or(TextureViewError::Uninitialized)?;',1)
  needle='        let mut requested_format = format;';pos=s.index(needle,s.index('pub fn face_view_with_format'))
  s=s[:pos]+'''        if let Some(context) = context { self.ensure_device(context)?; }
'''+s[pos:]
  s=s.replace('            view: Some(Arc::new(view)),','            device: self.device.clone(),\n            view: Some(Arc::new(view)),',1)
 p.write_text(s)
# Generic view only originates from correlated whole owner or cube face.
p=base/'texture_view/mod.rs';s=p.read_text()
for field in ['view','texture','size','format','dimension','layer']:
 s=s.replace('    pub '+field+':','    pub(super) '+field+':')
start=s.index('pub struct TextureView');end=s.index('\n}',start)
s=s[:end]+'\n    pub(super) device: Option<Arc<wgpu::Device>>,'+s[end:]
s=s.replace('            view: None,','            device: None,\n            view: None,',1)
s=s.replace('            view: t.view.clone(),','            device: t.device.clone(),\n            view: t.view.clone(),',1).replace('            format: t.format,','            format: t.effective_format(t.format),',1)
start=s.index('impl TextureView {')+len('impl TextureView {')
s=s[:start]+'''
    pub fn format(&self) -> TextureFormat { self.format }
    pub fn dimension(&self) -> wgpu::TextureViewDimension { self.dimension }
    pub fn native_texture(&self) -> Option<&Arc<wgpu::Texture>> { self.texture.as_ref() }
    pub fn cached_view(&self) -> Option<&Arc<wgpu::TextureView>> { self.view.as_ref() }
    pub fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d { width: self.size.0, height: self.size.1, depth_or_array_layers: 1 }
    }
    pub(super) fn selected_view(&self) -> super::SelectedTextureViewResult {
        Ok(super::SelectedTextureView::new(
            self.view.clone().ok_or(super::TextureViewError::MissingView)?,
            self.format.into(), self.dimension,
            self.device.clone().ok_or(super::TextureViewError::Uninitialized)?,
        ))
    }
'''+s[start:]
p.write_text(s)
p=base/'mod.rs';s=p.read_text().replace('mod cube_face;','pub(crate) use selection::{SelectedTextureView, SelectedTextureViewResult};\n\nmod selection;\nmod cube_face;');p.write_text(s)
p=base/'view_error.rs';s=p.read_text().replace('    MissingView,','''    MissingView,
    WrongDevice,
    NotTexture,
    FormatMismatch { expected: wgpu::TextureFormat, actual: wgpu::TextureFormat },
    DimensionMismatch { expected: wgpu::TextureViewDimension, actual: wgpu::TextureViewDimension },''')
s=s.replace('            Self::MissingView => formatter.write_str("Texture has no view"),','''            Self::MissingView => formatter.write_str("Texture has no view"),
            Self::WrongDevice => formatter.write_str("Texture belongs to a different GPU device"),
            Self::NotTexture => formatter.write_str("Parameter is not a texture"),
            Self::FormatMismatch { expected, actual } => write!(formatter, "Texture format mismatch. Expected {expected:?}, got {actual:?}"),
            Self::DimensionMismatch { expected, actual } => write!(formatter, "Texture dimension mismatch. Expected {expected:?}, got {actual:?}"),''')
s=s.replace('            Self::Uninitialized | Self::MissingView => None,','            _ => None,');p.write_text(s)
