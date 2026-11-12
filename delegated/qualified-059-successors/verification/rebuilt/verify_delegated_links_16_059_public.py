#!/usr/bin/env python3
"""Verify compact checkpoint16 references without transferring prior credit."""
import sys as _cw_sys
_cw_sys.dont_write_bytecode = True
from pathlib import Path as _CW_Path
from qualified_crosswalk import configure as _cw_configure
_CW = _cw_configure(_CW_Path(__file__).parent / 'CROSSWALK-059.json')
import gzip,hashlib,json,tarfile
from pathlib import Path
R=_CW.root;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-16.json');by={b['bundle_id']:b for b in cp['bundles']};prior=[]
def flatten(m):
 yield m
 for x in m.get('members',[]):yield from flatten(x)
for n in (13,14,15,16):
 for b in load(f'DELEGATED-EVIDENCE-CHECKPOINT-{n}.json')['bundles']:
  for m in b['members']:prior.extend(flatten(m))
b=by['p16-runtime-determinism']
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('evidence-links.json'))
 for x in d['published']:
  p=(R/x['path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();raw=_CW.read(x['path']);assert _CW.matches(x['path'],raw,x['sha256'],x['bytes'])
 for x in d['prior_packet_records']:assert any(m.get('original_sha256')==x['sha256'] and m.get('original_bytes')==x['bytes'] for m in prior)
 assert len(d['published'])==13 and len(d['prior_packet_records'])==6
transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']}
b=by['puzzle-choices'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('historical-index.json'))
 for x in d['records']:
  m=members[x['file']]
  if m['content_encoding']=='gzip':
   assert m['original_uncompressed_sha256']==x['sha256'] and m['original_uncompressed_bytes']==x['bytes']
  else:
   assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
  if x['kind']=='original-059':
   old=by_path[x['transport_path']];assert old['sha256']==x['sha256'] and old['bytes']==x['bytes'] and old['git_blob']==x['git_blob']
 assert len(d['records'])==15
b=by['garment-underlayer-hair-ops'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('historical-manifest.json'))
 for x in d['exact_original_files']:
  m=members[x['local']];assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
 paths=['recovered/original-registry-source-archive/objects.tar.gz','recovered/original-registry-source-archive/registry.json.gz','recovered/original-registry-source-archive/MANIFEST.json','baseline-059/queue.json','ASSESSMENT-L01-SURFACE-CUT-95E.json','ASSESSMENT-GARMENT-DESIGN-95E.json','ASSESSMENT-GEOMETRY-OWNERS-95E.json'];known=_CW.known(paths)
 for x in d['hash_links']:assert known[x['sha256']]==x['bytes']
 assert len(d['exact_original_files'])==20 and len(d['hash_links'])==7
b=by['trunk-hose-lifecycle-sampling-hair-ops'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('historical-manifest.json'))
 for x in d['exact_original_files']:
  m=members[x['local']];assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
 paths=['recovered/original-registry-source-archive/objects.tar.gz','recovered/original-registry-source-archive/registry.json.gz','recovered/original-registry-source-archive/MANIFEST.json','baseline-059/queue.json','delegated/garment-underlayer-hair-ops-evidence.tar.gz','delegated/garment-underlayer-hair-ops-report.json','ASSESSMENT-WORLD-EVENT-LIFECYCLE-95E.json','ASSESSMENT-L01-BASIS-WIRE-95E.json','ASSESSMENT-L01-SURFACE-CUT-95E.json','RECOVERY-PHYSICAL-P16-PATCH-20261111.json'];known=_CW.known(paths)
 for x in d['hash_links']:assert known[x['sha256']]==x['bytes']
 prev=by['garment-underlayer-hair-ops'];assert prev['archive_sha256']==d['prior_published_packet']['sha256'];pm={m['path']:m for m in prev['members']}
 for x in d['prior_published_packet']['members']:
  m=pm[x['member']];assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
 assert len(d['exact_original_files'])==8 and len(d['hash_links'])==10
b=by['serde-tree-growth'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('evidence-links.json'))
 for x in d['published']:
  p=(R/x['path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();raw=_CW.read(x['path']);assert _CW.matches(x['path'],raw,x['sha256'],x['bytes'])
 for packet in d['prior_packets']:
  for x in packet['records']:assert any(m.get('original_sha256')==x['sha256'] and m.get('original_bytes')==x['bytes'] for m in prior)
 derive=json.load(t.extractfile('derive-eof-qualification.json'));lex=json.load(t.extractfile('lexical-eof-qualification.json'))
 assert derive['requested_range']==[1,110] and derive['actual_displayed_range']==[1,106] and derive['nonexistent_requested_range']==[107,110]
 assert lex['requested_range']==[1,125] and lex['existing_displayed_range']==[1,83] and lex['nonexistent_requested_range']==[84,125]
 assert members[derive['initial_receipt']]['original_sha256']==derive['initial_receipt_sha256']
 for x in (derive,lex):
  raw=t.extractfile(x['source_archive']).read();assert sha(raw)==x['source_sha256'] and len(raw.splitlines())==x['actual_eof']
 assert len(d['published'])==12 and len(d['prior_packets'])==4
b=by['tree-root-plan-profile'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('historical-index.json'))
 for x in d['records']:
  m=members[x['file']]
  if m['content_encoding']=='gzip':assert m['original_uncompressed_sha256']==x['sha256'] and m['original_uncompressed_bytes']==x['bytes']
  else:assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
  if x['kind']=='original-059':
   old=by_path[x['transport_path']];assert old['sha256']==x['sha256'] and old['bytes']==x['bytes'] and old['git_blob']==x['git_blob']
 assert len(d['records'])==21
print('Verified checkpoint16 published, prior-packet and original historical identities; retained reconstructions do not transfer author or read credit.')

_CW.finish()
