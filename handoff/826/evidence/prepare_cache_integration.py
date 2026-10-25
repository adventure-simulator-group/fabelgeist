from pathlib import Path
import hashlib,json,subprocess,re
base=Path('/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/target/pr780-audit/handoff-cache-5c2d287936c62432');m=json.loads((base/'manifest.json').read_text());out=Path('target/pr826-audit/cache-integration-preview');out.mkdir(exist_ok=True);rows=[]
for f in m['files']:
 path=f['path']
 if '/reduce/' in path:continue
 src=base/'source'/path;dest=out/'source'/path;dest.parent.mkdir(parents=True,exist_ok=True)
 current=Path(path).read_text() if Path(path).exists() else ''
 previous=(base/'before'/path).read_text() if f['before_sha256'] is not None else ''
 incoming=src.read_text()
 if hashlib.sha256(incoming.encode()).hexdigest()!=f['sha256']:raise SystemExit('incoming mismatch '+path)
 if previous and hashlib.sha256(previous.encode()).hexdigest()!=f['before_sha256']:raise SystemExit('before mismatch '+path)
 if path.endswith('/kernel/mod.rs'):
  value=current.replace('use std::collections::HashMap;\n','').replace('use std::sync::{Arc, RwLock};','use std::sync::Arc;')
  start='/// Compiles each distinct kernel source once and hands out clones after that.'
  end='/// A command encoder that many kernel dispatches record into before a single'
  replacement=incoming[incoming.index('/// Shares successful kernel builds'):incoming.index(end)]
  value=value[:value.index(start)]+replacement+value[value.index(end):]
  mark='/// The flat item index of an invocation dispatched by'
  endmark='/// A compiled compute kernel:'
  replacement=incoming[incoming.index(mark):incoming.index(endmark)]
  value=value[:value.index(mark)]+replacement+value[value.index(endmark):]
  status='exact authorized KernelCache/import/FOLDED constant-doc fragments; core methods preserved'
 elif path=='crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs':
  value=current.replace('pub use signature::*;','pub use signature::*;\npub use source_cache::ComputeSourceCache;').replace('pub mod signature;','pub mod signature;\nmod source_cache;')
  status='exact authorized module/reexport only'
 elif not previous:
  if current:raise SystemExit('new source already present '+path)
  value=incoming;status='new shared cache owner exact source'
 else:
  temp=out/'merge';temp.mkdir(exist_ok=True)
  for name,text in [('current',current),('before',previous),('incoming',incoming)]:(temp/name).write_text(text)
  r=subprocess.run(['git','merge-file','-p',str(temp/'current'),str(temp/'before'),str(temp/'incoming')],capture_output=True,text=True)
  if r.returncode<0 or r.returncode>127:raise SystemExit(r.stderr)
  value=r.stdout;status='clean source reconciliation' if r.returncode==0 else 'CONFLICT '+str(r.returncode)
 dest.write_text(value);rows.append({'path':path,'status':status,'before_current_sha256':hashlib.sha256(current.encode()).hexdigest(),'preview_sha256':hashlib.sha256(value.encode()).hexdigest()})
(out/'manifest.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows,indent=2))
