#!/usr/bin/env python3
"""Read-only verification of this additive attribution record."""
import pathlib,json,hashlib
P=pathlib.Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest()
def read(n):
 p=pathlib.PurePosixPath(n);assert not p.is_absolute() and '..' not in p.parts;q=P.joinpath(*p.parts);assert q.is_file() and not q.is_symlink();return q.read_bytes()
def load(n):return json.loads(read(n))
m=load('MANIFEST.json');es={x['path']:x for x in m['files']};actual=set()
for f in P.rglob('*'):
 assert not f.is_symlink()
 if f.is_file() and f!=P/'MANIFEST.json':actual.add(f.relative_to(P).as_posix())
assert actual==set(es)
for n,v in es.items():
 d=read(n);assert H(d)==v['sha256'] and len(d)==v['bytes']
r=load('REPORT.json')
for p in r['packets']:
 d=read(p['manifest_file']);assert H(d)==p['manifest_sha256'];index={x['path']:x for x in json.loads(d)['files']}
 for q in p['members']:
  b=read(q['file']);old=index[q['source_name']];assert H(b)==q['sha256']==old['sha256'] and len(b)==q['bytes']==old['bytes']
assert H(read('main/REPORT-FINAL.json'))==r['subject_report_sha256']
a=load('hair-2668/read-receipt.json');assert a['agent']=='/root/hair_ops'
for path,ranges in [('crates/fabelgeist-shell/src/outer_layer.rs',[[1,116]]),('crates/fabelgeist-shell/src/shell.rs',[[30,55],[75,90],[340,385]]),('crates/fabelgeist-xpbd/src/dynamics/mass.rs',[[1,115]])]:
 assert any(x['path']==path and x['displayed']==ranges for x in a['support'])
a=load('mhr-2658/read-receipt.json');assert a['author']=='/root/mhr_character';assert all(any(x['role']==2658 and x['revision']==rev and x['ranges']==[[1,70],[71,140],[141,196]] for x in a['actual_displays']) for rev in ('historical','current'))
a=load('mhr-2658/support-read-receipt.json');assert a['author']=='/root/mhr_character';assert any(x['path']=='crates/fabelgeist-shell/src/surface_contact.rs' and x['actual_ranges']==[[72,105],[370,445]] for x in a['reads']);assert any(x['path']=='crates/fabelgeist-xpbd/src/particles.rs' and x['actual_ranges']==[[21,34],[87,120],[139,167]] for x in a['reads'])
a=load('mhr-2341-2342/read-receipt.json');assert a['agent']=='/root/mhr_character'
for path,stop in [('crates/fabelgeist-shell/src/shell.rs',408),('crates/fabelgeist-shell/src/surface_contact.rs',623)]:
 for rev in ('6533e444fa37552927070cc8d8190907fd3b413e','95e63628f9d70b8dd6e6fede64334696c6995009'):
  assert any(x['path']==path and x['revision']==rev and x['ranges']==[[1,stop]] for x in a['actual_source_displays'])
print(json.dumps({'status':'PASS','files':len(es),'prior_frozen_manifests':len(r['packets']),'qualification':'Checks retained recorded scopes and hashes only; no reader transfer, complete prior-work census, product or source behavior validation.'},indent=2))
