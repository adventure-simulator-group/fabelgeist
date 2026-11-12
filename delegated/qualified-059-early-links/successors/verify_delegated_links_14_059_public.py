#!/usr/bin/env python3
"""Resolve compact checkpoint 14 references with original-byte qualifications."""
import sys as _cw_sys
_cw_sys.dont_write_bytecode = True
from pathlib import Path as _CW_Path
_CW_DIR = _CW_Path(__file__).resolve().parents[2] / 'qualified-059-successors' / 'successors'
_cw_sys.path.insert(0, str(_CW_DIR))
from qualified_crosswalk import configure as _cw_configure
_CW = _cw_configure(_CW_DIR / 'CROSSWALK-059.json')
import hashlib,json,tarfile
from pathlib import Path
R=_CW.root
load=lambda p:json.loads((R/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
cp13=load('DELEGATED-EVIDENCE-CHECKPOINT-13.json');cp=load('DELEGATED-EVIDENCE-CHECKPOINT-14.json')
by={b['bundle_id']:b for b in cp['bundles']};prior={}
for b in cp13['bundles']:
 for m in b['members']:
  if m['path'] in ('MANIFEST.json','BUNDLE-MANIFEST.json','packet-manifest.json'):
   prior[m['original_sha256']]=b
b=by['catalog-fit-surface'];members={x['path']:x for x in b['members']}
with tarfile.open(R/b['archive']) as t:
 refs=json.load(t.extractfile('prior-references.json'))
 for name,packet in refs['packets'].items():
  h=packet['manifest_sha256'];assert h in prior and members[packet['manifest_copy']]['original_sha256']==h
 for ref in refs['references']:
  h=refs['packets'][ref['packet']]['manifest_sha256'];m=next(x for x in prior[h]['members'] if x['path']==ref['manifest_path'])
  assert m['original_sha256']==ref['sha256'] and m['original_bytes']==ref['bytes']
 assert len(refs['packets'])==2 and len(refs['references'])==6
b=by['p16-catalog-occupancy-surface'];members={x['path']:x for x in b['members']}
with tarfile.open(R/b['archive']) as t:
 refs=json.load(t.extractfile('published-references.json'))['entries']
 for item in refs:
  p=(R/item['repository_relative_path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();raw=_CW.read(item['repository_relative_path']);assert _CW.matches(item['repository_relative_path'],raw,item['sha256'],item['bytes'])
 assert len(refs)==11
b=by['p16-catalog-helpers-hair-ops'];members={x['path']:x for x in b['members']}
with tarfile.open(R/b['archive']) as t:
 history=json.load(t.extractfile('historical-manifest.json'))
 for item in history['exact_original_files']:
  m=members[item['local']];assert m['original_sha256']==item['sha256'] and m['original_bytes']==item['bytes']
 paths=['ASSESSMENT-L03-CATALOG-DIMENSIONS-95E.json','ASSESSMENT-L01-HISTORICAL-937-947.json','ASSESSMENT-EQUIPMENT-PRESENTATION-95E.json','RECOVERY-PHYSICAL-P16-PATCH-20261111.json','recovered/physical-p16-patch-recovery/tracked.patch','recovered/original-registry-source-archive/objects.tar.gz','recovered/original-registry-source-archive/registry.json.gz','recovered/original-registry-source-archive/MANIFEST.json','native-evidence/serde_json-1.0.151/serde_json-1.0.151.crate','native-evidence/serde_core-1.0.229/serde_core-1.0.229.crate']
 exact={sha((R/p).read_bytes()):len((R/p).read_bytes()) for p in paths}
 rec=load('RECOVERY-PHYSICAL-P16-PATCH-20261111.json');exact[rec['live_comparison']['original_report_sha256']]=rec['live_comparison']['original_report_bytes']
 parallel=by['p16-catalog-occupancy-surface']
 for path in ('MANIFEST.json','REPORT.json'):
  m=next(x for x in parallel['members'] if x['path']==path);exact[m['original_sha256']]=m['original_bytes']
 for item in history['hash_links']:
  assert exact.get(item['sha256'])==item['bytes']
 assert len(history['exact_original_files'])==6 and len(history['hash_links'])==13
print('Verified two prior packet links, six selected identities, eleven published-file links and thirteen historical hash links; redacted copies retain original-byte qualifications.')

assert _CW.crosswalk_links_checked == 2
_CW.finish()
