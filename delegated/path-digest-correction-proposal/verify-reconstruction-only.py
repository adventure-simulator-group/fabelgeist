#!/usr/bin/env python3
"""Read-only verification of this correction only; not a legacy verifier replacement."""
import argparse,hashlib,json,pathlib
P=pathlib.Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest()
def read(n):
 q=pathlib.PurePosixPath(n);assert not q.is_absolute() and '..' not in q.parts
 p=P.joinpath(*q.parts);assert p.is_file() and not p.is_symlink();return p.read_bytes()
def ptr(doc,s):
 for k in s.split('/')[1:]:
  k=k.replace('~1','/').replace('~0','~');doc=doc[int(k)] if isinstance(doc,list) else doc[k]
 return doc
ap=argparse.ArgumentParser();ap.add_argument('--publication',type=pathlib.Path);a=ap.parse_args()
m=json.loads(read('MANIFEST.json'));listed={x['path']:x for x in m['files']};assert len(listed)==len(m['files'])
actual=set()
for p in P.rglob('*'):
 assert not p.is_symlink()
 if p.is_file() and p!=P/'MANIFEST.json':actual.add(p.relative_to(P).as_posix())
assert actual==set(listed)
for name,r in listed.items():b=read(name);assert (H(b),len(b))==(r['sha256'],r['bytes']),name
cross=json.loads(read('CROSSWALK.json'));old=json.loads(read('OLDER-TEXT-MANIFEST.json'));assert H(read('OLDER-TEXT-MANIFEST.json'))==cross['source_inventory_manifest_sha256'];historic={x['tracked_path']:x for x in old['files']};rows=cross['files'];assert len(rows)==len(historic)==15
fieldnames={'original_path_sha256','path_sha256','original_value_sha256','value_sha256','matched_sha256','matched_prefix_sha256'};marker='<REDACTED_INDIVIDUAL_PATH_DIGEST>'
def check_semantic_fields(obj):
 if isinstance(obj,dict):
  for k,v in obj.items():
   if k in fieldnames:assert v==marker,(k,'a semantically identified digest was not reconstructed')
   check_semantic_fields(v)
 elif isinstance(obj,list):
  for v in obj:check_semantic_fields(v)
count=known=0
for r in rows:
 v=historic[r['source_path']];assert (r['original_file_sha256'],r['original_file_bytes'],r['original_git_blob'])==(v['whole_file_sha256'],v['bytes'],v['git_blob'])
 b=read(r['public_path']);assert (H(b),len(b))==(r['successor_file_sha256'],r['successor_file_bytes']);doc=json.loads(b);check_semantic_fields(doc)
 positions=[]
 for c in r['changes']:
  k=c['public_offset_0based'];assert b[k:k+len(marker)]==marker.encode();assert marker in ptr(doc,c['json_pointer']);assert c['public_marker']==marker;positions.append(k);count+=1;known+=int(c['observed_private_corpus_match'])
 assert positions==sorted(set(positions));assert len([c for c in r['changes'] if c['observed_private_corpus_match']])==v['match_occurrences'];assert sorted({c['original_line_1based'] for c in r['changes'] if c['observed_private_corpus_match']})==v['lines']
assert (count,known)==(795,522)
if a.publication:
 dep=json.loads(read('DEPENDENCY-INVENTORY.json'));checked=0
 for x in dep['references']:
  path=a.publication/x['path'];assert not path.is_symlink();b=path.read_bytes();assert (H(b),len(b))==(x['sha256'],x['bytes']),x['path'];checked+=1
 print('Checked',checked,'observed working-tree text references against their captured whole-file identities; archive and indirect references remain open.')
else:print('Working-tree dependency bytes not supplied; only captured inventory assertions were checked.')
print('PASS: 15 qualified historical whole-file bindings, successor hashes, 795 marker offsets and pointers, 522 matched historical observations. Exact old-to-new replay requires restricted originals; this does not replace legacy verifier checks or prove complete privacy.')
