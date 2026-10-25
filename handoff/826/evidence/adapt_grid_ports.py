from pathlib import Path
import re,subprocess
p=Path('crates/fabelgeist-compute/src/kernel/mod.rs');s=p.read_text()
s=s.replace('    pub pipeline: ComputePipeline,','    pipeline: ComputePipeline,')
s=s.replace('    pub fn is_cached(&self)', '    pub fn pipeline(&self) -> &ComputePipeline { &self.pipeline }\n\n    pub fn is_cached(&self)')
s=s.replace('pub fn groups_for(&self, items: u32) -> [u32; 3] {\n        [items.div_ceil(self.workgroup_size[0]), 1, 1]','pub fn groups_for(&self, items: u32) -> WorkgroupGrid {\n        WorkgroupGrid::from([items.div_ceil(self.workgroup_size[0]), 1, 1])')
s=s.replace('pub fn groups_folded(&self, items: u32) -> [u32; 3]', 'pub fn groups_folded(&self, items: u32) -> WorkgroupGrid').replace('        [x, groups.div_ceil(x), 1]','        WorkgroupGrid::from([x, groups.div_ceil(x), 1])')
s=s.replace('        groups: [u32; 3],','        groups: WorkgroupGrid,')
s=s.replace('            context, pipeline, parameters, groups[0], groups[1], groups[2],','            context, pipeline, parameters, groups,')
s=s.replace('        if groups.contains(&0) {','        if groups.occupancy() == DispatchOccupancy::Empty {')
s=s.replace('        match &kernel.fast {','        kernel.pipeline.ensure_device(self.context)?;\n        match &kernel.fast {')
start=s.index('                pass.set_pipeline(&prepared.pipeline);');end=s.index('\n            }\n            None =>',start)
s=s[:start]+'                prepared.record(&mut pass, groups)?;'+s[end:]
s=s.replace('                groups[0],\n                groups[1],\n                groups[2],','                groups,')
s=s.replace('    dispatches: usize,','    dispatches: RecordedDispatchCount,').replace('            dispatches: 0,','            dispatches: RecordedDispatchCount::default(),').replace('        self.dispatches += 1;','        self.dispatches.record();').replace('pub fn dispatch_count(&self) -> usize','pub fn dispatch_count(&self) -> RecordedDispatchCount')
s=s.replace('mod fast;','pub use dispatch_count::RecordedDispatchCount;\nmod dispatch_count;\nmod fast;')
s=s.replace('destination: &Buffer, bytes: u64','destination: &Buffer, bytes: BufferByteLength')
s=s.replace('        if bytes > u64::from(source.size) || bytes > u64::from(destination.size) {','        if bytes > source.size || bytes > destination.size {').replace('.copy_buffer_to_buffer(&source.buffer, 0, &destination.buffer, 0, bytes);','.copy_buffer_to_buffer(&source.buffer, 0, &destination.buffer, 0, u64::from(bytes));')
p.write_text(s)
Path('crates/fabelgeist-compute/src/kernel/dispatch_count.rs').write_text(subprocess.check_output(['git','show','71f9cab8a5f97760e2e61ca4c74876b715f8aebc:crates/fabelgeist-compute/src/kernel/dispatch_count.rs'],text=True))
# Convert only the agreed actual native ComputePass receiving calls.
for root in ['crates/fabelgeist-compute/src','crates/fabelgeist-gpu/src','crates/fabelgeist-mesh/src']:
 for p in Path(root).rglob('*.rs'):
  s=p.read_text();edits=[]
  for m in re.finditer(r'ComputePass::(dispatch|record)\s*\(',s):
   start=m.end();depth=0;beg=start;args=[];end=None
   for i in range(start,len(s)):
    c=s[i]
    if c in '([{':depth+=1
    elif c in ')]}':
     if c==')' and depth==0:
      if s[beg:i].strip():args.append(s[beg:i].strip())
      end=i;break
     depth-=1
    elif c==',' and depth==0:
     if s[beg:i].strip():args.append(s[beg:i].strip())
     beg=i+1
   if len(args) == (6 if m.group(1)=='dispatch' else 7):
    args=args[:-3]+['fabelgeist_gpu::data::WorkgroupGrid::from(['+', '.join(args[-3:])+'])']
    if 'crates/fabelgeist-gpu/' in str(p):args[-1]=args[-1].replace('fabelgeist_gpu::data::','crate::data::')
    edits.append((start,end,',\n'.join(args)))
  for a,b,t in reversed(edits):s=s[:a]+t+s[b:]
  if edits:p.write_text(s)
# Literal-grid callers are actual authored/native kernel boundaries. Other #780 producers remain its connected dependency.
for root in ['crates/fabelgeist-bvh/src','crates/fabelgeist-hair/src','crates/fabelgeist-compute/src']:
 for p in Path(root).rglob('*.rs'):
  s=p.read_text();old=s
  # Dispatch calls use simple three-axis arrays with no embedded array literals.
  s=re.sub(r'(batch\.dispatch\([\s\S]*?,\s*&[\w.]+,\s*)(\[[^\[\]]+\])(\s*,?\s*\))',r'\1\2.into()\3',s)
  if p.name=='pipeline.rs' and 'stereo' in str(p):
   s=s.replace('batch.dispatch(&k.depth, &p, groups)','batch.dispatch(&k.depth, &p, groups.into())').replace('batch.dispatch(&k.temporal, &p, groups)','batch.dispatch(&k.temporal, &p, groups.into())')
  if p.name=='prefix_sum.rs':s=s.replace('grid(1))?','grid(1).into())?').replace('grid(blocks))?','grid(blocks).into())?')
  if s!=old:p.write_text(s)
# Byte receiving adapters; producer families remain owned by #780 until its agreed source arrives.
p=Path('crates/fabelgeist-compute/src/sort/mod.rs');s=p.read_text().replace('batch.copy_buffer(&source_keys, keys, needed)','batch.copy_buffer(&source_keys, keys, needed.into())').replace('batch.copy_buffer(&source_values, values, needed)','batch.copy_buffer(&source_values, values, needed.into())');p.write_text(s)
p=Path('crates/fabelgeist-physics/src/contact.rs');s=p.read_text().replace('particles.count() as u64 * 16,','(particles.count() as u64 * 16).into(),');p.write_text(s)
p=Path('crates/fabelgeist-hair/src/gpu/grow.rs');s=p.read_text().replace('&self.positions.own, self.bytes())','&self.positions.own, self.bytes().into())');p.write_text(s)
p=Path('crates/fabelgeist-compute/src/kernel/fast/compile.rs');s=p.read_text().replace('                pub(super) layout: Some','                layout: Some');p.write_text(s)
