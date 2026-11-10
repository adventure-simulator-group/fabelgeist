"""Verify two distinct deleted image roles against their pre-deletion Git trees."""
import gzip,hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent; work=json.loads((root/'COVERAGE-L01-ROLES.json').read_text()); reg=json.loads(gzip.decompress((root/work['registry']).read_bytes()))['files']; prog=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); current={line.split('\t',1)[1]:line.split('\t',1)[0].split()[-1] for line in (root/'main-advance-95e/current-tree.tsv').read_text().splitlines()}
for i in (501,2416):
 name=f'ASSESSMENT-L01-DELETED-{i}.json'; doc=json.loads((root/name).read_text()); role=work['roles'][i]; original=reg[i]; assert doc['original_index']==i and doc['source_id']==role['source_id']==original['source_id'] and original['state']==doc['registry_state']=='deleted' and original['sha256'] is None and role['historical_sha256'] is None and not role['historical_object_available'] and role['status']=='deleted-base-trace-open' and role['historical_assessment']==name and doc['base_sha256']==original['base_sha256'] and doc['snapshot_sha256'] is None and name in prog['post_transition_deleted_base_reads']; bp=(root/doc['preserved_base_bytes']).resolve(); assert bp.is_relative_to(root); data=bp.read_bytes(); blob=hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest(); assert hashlib.sha256(data).hexdigest()==doc['base_sha256'] and len(data)==doc['base_bytes'] and blob==doc['base_git_blob'] and data==(root/'source-evidence/belt-worn.png').read_bytes() and original['path'] not in current
 def read(item):
  q=(root/item['preserved_bytes']).resolve(); assert q.is_relative_to(root); raw=gzip.decompress(q.read_bytes()); assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']; return raw
 commit=read(doc['commit_proof']); assert hashlib.sha1(b'commit '+str(len(commit)).encode()+b'\0'+commit).hexdigest()==doc['base_revision']==doc['commit_proof']['git_commit']; expected=commit.split(b'\n',1)[0].decode().split(' ',1)[1]; parts=original['path'].split('/'); assert len(parts)==len(doc['tree_proof'])
 for j,(part,proof) in enumerate(zip(parts,doc['tree_proof'])):
  raw=read(proof); assert hashlib.sha1(b'tree '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==expected==proof['tree'] and part==proof['component']; offset=0; hits=[]
  while offset<len(raw):
   sp=raw.index(b' ',offset); end=raw.index(b'\0',sp); mode=raw[offset:sp].decode(); entry=raw[sp+1:end].decode(); oid=raw[end+1:end+21].hex(); offset=end+21
   if entry==part: hits.append((mode,oid))
  assert len(hits)==1 and hits[0]==(proof['entry_mode'],proof['entry_oid']); mode,expected=hits[0]; assert (mode in ('40000','040000') and proof['entry_kind']=='tree') if j<len(parts)-1 else (mode=='100644' and proof['entry_kind']=='blob' and expected==blob)
print('verified distinct deleted pr-working icon identities and both pre-deletion Git path proofs')
