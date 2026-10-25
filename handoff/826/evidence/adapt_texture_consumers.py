from pathlib import Path
import re
paths=[p for root in ['crates/fabelgeist-compute/src','crates/fabelgeist-gpu/src/data/gpu'] for p in Path(root).rglob('*.rs') if 'texture/' not in str(p)]
for p in paths:
 s=p.read_text();old=s
 # All t.format and t.size tuple hits were traced to whole texture variants.
 s=re.sub(r'\bt\.format\b(?!\s*\()', 't.format()', s)
 for axis,field in [(0,'width'),(1,'height'),(2,'depth_or_array_layers')]:
  s=s.replace(f't.size.{axis}',f't.extent().{field}')
 if p.name=='distance_field.rs':
  s=s.replace('io.texture.as_ref()','io.native_texture()')
  s=re.sub(r'temp_tex\s*\.texture\s*\.as_ref\(\)','temp_tex.native_texture()',s)
  s=s.replace('io.format,','io.format(),')
  for axis,field in [(0,'width'),(1,'height'),(2,'depth_or_array_layers')]:s=s.replace(f'io.size.{axis}',f'io.extent().{field}')
 if p.name=='distance_field_jfa.rs':
  pos=s.index('        let size = input.size;');s=s[:pos]+s[pos:].replace('let size = input.size;','let size = input.extent();').replace('size.0','size.width').replace('size.1','size.height').replace('size.2','size.depth_or_array_layers')
 if s!=old:p.write_text(s)
p=Path('crates/fabelgeist-gpu/tests/texture_views.rs');s=p.read_text().replace('=> texture.format,','=> texture.format(),').replace('=> texture.view = None,','=> texture.clear_cached_view(),').replace('=> texture.texture = None,','=> texture.clear_native_texture(),');p.write_text(s)
# Existing helper remains distinct in buffer/uniform/sampler semantics, but shares view selection.
p=Path('crates/fabelgeist-gpu/src/data/gpu/bind_group.rs');s=p.read_text().replace('use anyhow::{Result, anyhow};','use anyhow::{Context, Result, anyhow};')
start=s.index('        // 3. Textures');end=s.index('        // 4. Samplers',start)
s=s[:start]+'''        // Select once: effective view format is independent of allocation format.
        let mut texture_views = Vec::new();
        for binding in &bg_reflection.texture_bindings {
            let parameter = parameters.get(&binding.name).ok_or_else(|| anyhow!("{pass_name}: Texture '{}' missing", binding.name))?;
            let selected = parameter.texture_binding(context, binding)
                .map_err(anyhow::Error::new)
                .with_context(|| format!("{pass_name}: Texture '{}'", binding.name))?;
            texture_views.push((binding.binding, selected.view));
        }
        for (binding, view) in &texture_views {
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: *binding,
                resource: wgpu::BindingResource::TextureView(view),
            });
        }

'''+s[end:]
p.write_text(s)
# Readonly shader and reflection receivers preserve existing fast-path selection.
p=Path('crates/fabelgeist-compute/src/kernel/mod.rs');s=p.read_text().replace('self.pipeline.shader.code == other.pipeline.shader.code','self.pipeline.shader().code() == other.pipeline.shader().code()')
start=s.index('        let fast = match pipeline');end=s.index('\n        // The batched layout',start)
s=s[:start]+'''        let fast = fast::FastPath::new(
            context, pipeline.shader().module(), &entry_point, pipeline.reflection(),
        )?;
'''+s[end:]
p.write_text(s)
