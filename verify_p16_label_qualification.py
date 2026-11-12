#!/usr/bin/env python3
"""Verify published label evidence and expressly qualified redaction limits."""
import gzip,hashlib,io,json,re,subprocess,tarfile,tempfile
from pathlib import Path
R=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest()
q=json.loads((R/'QUALIFICATION-P16-REGISTRY-LABELS-2702-2704.json').read_bytes())
assert q['original_roles']==[2702,2703,2704]
assert json.loads((R/'DELEGATED-EVIDENCE-CHECKPOINT-23.json').read_bytes())['p16_label_qualification']=='QUALIFICATION-P16-REGISTRY-LABELS-2702-2704.json'
raw=(R/q['archive']).read_bytes();assert (sha(raw),len(raw))==(q['archive_sha256'],q['archive_bytes'])
local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-[^"\'\s]+',re.I)
secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
def verify(body,m,depth=0):
 assert depth<=9 and (sha(body),len(body))==(m['published_sha256'],m['published_bytes'])
 encoding=m['content_encoding'];assert encoding in ('identity','gzip','tar','gzip-tar')
 plain=gzip.decompress(body) if encoding.startswith('gzip') else body
 assert (sha(plain),len(plain))==(m['published_uncompressed_sha256'],m['published_uncompressed_bytes'])
 assert m['changed']==(m['original_sha256']!=m['published_sha256'])
 if not m['changed']:assert (len(body),sha(plain),len(plain),m['total_local_path_redactions'])==(m['original_bytes'],m['original_uncompressed_sha256'],m['original_uncompressed_bytes'],0)
 else:assert 'Reconstructed public copy' in m['qualification'] and m['total_local_path_redactions']>0
 if encoding.endswith('tar'):
  children={x['path']:x for x in m['members']};assert len(children)==len(m['members'])
  with tarfile.open(fileobj=io.BytesIO(plain),mode='r:') as t:
   files=t.getmembers();assert len(files)==len(children) and set(x.name for x in files)==set(children)
   for f in files:
    assert (f.isfile() or f.isdir()) and not f.name.startswith('/') and '..' not in Path(f.name).parts
    assert not local.search(f.name.encode()) and not secret.search(f.name.encode())
    assert all(not local.search(s.encode()) and not secret.search(s.encode()) for s in (f.uname,f.gname))
    assert all(not local.search(str(s).encode()) and not secret.search(str(s).encode()) for s in f.pax_headers.values())
    if f.isfile():verify(t.extractfile(f).read(),children[f.name],depth+1)
    else:assert children[f.name]['entry_type']=='directory' and not children[f.name]['changed']
  assert m['total_local_path_redactions']==sum(x['total_local_path_redactions'] for x in children.values())
 else:
  assert not local.search(plain) and not secret.search(plain)
  assert m['total_local_path_redactions']==m['direct_local_path_redactions'] and plain.count(b'<LOCAL_PATH_')>=m['direct_local_path_redactions']
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
 files=t.getmembers();members={x['path']:x for x in q['members']}
 assert len(files)==len(members)==155 and set(x.name for x in files)==set(members)
 assert all(f.isfile() and not f.name.startswith('/') and '..' not in Path(f.name).parts and (f.uid,f.gid,f.uname,f.gname)==(0,0,'','') for f in files)
 data={f.name:t.extractfile(f).read() for f in files}
for p,b in data.items():verify(b,members[p])
assert {k for k,v in members.items() if v['changed']}=={'freeze.py','history-774/catalog.json.gz','report.json'}
manifest=json.loads(data['BUNDLE-MANIFEST.json']);assert sha(data['BUNDLE-MANIFEST.json'])==q['pre_redaction_manifest_sha256']
assert len(manifest['files'])==manifest['file_count']==154
for x in manifest['files']:
 assert (members[x['path']]['original_sha256'],members[x['path']]['original_bytes'])==(x['sha256'],x['bytes'])
assert members['report.json']['original_sha256']==q['pre_redaction_report_sha256']
assert sha(data['verification.json'])==q['pre_redaction_verification_sha256']
# Reuse the frozen evidence-only verifier with two explicit adaptations for
# catalog path redaction. The original is preserved untouched in the archive.
source=data['verify-evidence.py'].decode()
old="b=(p/x['archive']).read_bytes();b=gzip.decompress(b) if x['encoding']=='gzip' else b;assert (sha(b),blob(b),tree)==(x['sha256'],x['blob'],x['blob'])"
new="""b=(p/x['archive']).read_bytes();b=gzip.decompress(b) if x['encoding']=='gzip' else b
 if x['archive']=='history-774/catalog.json.gz':
  assert tree==x['blob'] and catalog_meta['original_uncompressed_sha256']==x['sha256']
 else:assert (sha(b),blob(b),tree)==(x['sha256'],x['blob'],x['blob'])"""
assert source.count(old)==1;source=source.replace(old,new)
old="saved['original_registry_row']==row and saved['original_catalog_row']==c and"
new="saved['original_registry_row']==row and {k:v for k,v in saved['original_catalog_row'].items() if k!='object'}=={k:v for k,v in c.items() if k!='object'} and"
assert source.count(old)==1;source=source.replace(old,new)
source=source.replace("p=pathlib.Path(__file__).parent;", "catalog_meta="+repr(members['history-774/catalog.json.gz'])+";p=pathlib.Path(__file__).parent;",1)
source=source.replace("Three original 774 roles individually match f82 transported source, 039 preservation and a120 base; 774 catalog and registration Git proofs", "Three roles match f82, 039 and a120; exact registration proofs and a catalog chain qualified for public redaction")
source=source.replace("Exact 774 mixed-label registry/catalog/registration and 059 reviewer/range identities; no normalized-label assertion", "Exact 774 mixed-label registry and registration, reconstructed catalog fields, and 059 reviewer/range identities; no normalized-label assertion")
with tempfile.TemporaryDirectory(prefix='726-p16-label-evidence-') as tmp:
 p=Path(tmp)
 for name,body in data.items():
  dest=p/name;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(body)
 script=p/'verify-published-adaptation.py';script.write_text(source)
 result=subprocess.run(['python3',str(script)],capture_output=True,text=True)
 assert result.returncode==0,(result.stdout,result.stderr)
 print(result.stdout.strip())
print('Published catalog original bytes cannot be independently recomputed after path redaction; the chain ends at the recorded original Git blob and the original uncompressed SHA is retained. Exact originals and pre-redaction verification are local only. No label normalization, reader transfer, product execution or 3821 promotion.')
