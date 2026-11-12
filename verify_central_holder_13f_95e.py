#!/usr/bin/env python3
"""Check bounded holder source relationships and the navigation qualification."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent
load=lambda name: json.loads((R/name).read_text())
sha=lambda b: hashlib.sha256(b).hexdigest()
a=load('ASSESSMENT-CENTRAL-HOLDER-13F-95E.json')
assert a['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009'
assert a['new_qualified_ledger_rows']==[] and a['further_evidence_for_existing_rows']==['SEGOVB-L12-HOLDER-ORIGINAL-OPTIONAL-ID-EXPECT']
refs=[]; bodies={}
for tag,archive in [('13F','delegated/central-13f-windows-proof.tar.gz'),('13F-SUPPORT','delegated/central-13f-support-windows-proof.tar.gz')]:
 reg=load('CENTRAL-'+tag+'-REGISTRATION.json')
 refs.extend({'revision':x['revision'],'path':x['path'],'lines':x['planned_display_ranges']} for x in reg['rows'])
 with tarfile.open(R/archive) as t:
  p=json.load(t.extractfile('proof.json'))
  for x in p['paths']:
   b=t.extractfile(p['objects'][x['blob']]['archive']).read();assert sha(b)==x['sha256'];bodies[x['path']]=b.decode().splitlines()
assert refs==a['source_references'] and len(refs)==9
def line(path,n):return bodies['crates/adventuresim-weapon-model/src/'+path][n-1]
assert 'pub id: Option<String>' in line('recipe/mod.rs',157) and 'fn resolved_id' in line('recipe/mod.rs',217)
assert 'component.resolved_id(index)' in line('recipe/validation.rs',152) and 'component.validate()' in line('recipe/validation.rs',164)
assert 'dimensions::check' in line('recipe/validation.rs',197) and 'melee::check' in line('recipe/validation.rs',200)
assert 'recipe.clone()' in line('model/placement.rs',49) and 'component.id = Some(id.clone())' in line('model/placement.rs',110)
assert 'holder_identity' in line('holders.rs',153) and 'Construction::new' in line('holders.rs',154)
assert 'expect("gameplay component ID")' in line('holders.rs',167) and 'expect("gameplay component ID")' in line('holders.rs',183)
assert 'recipe.validate()' in line('model/mod.rs',82) and 'placement::resolve' in line('model/mod.rs',83) and 'construct(' in line('model/mod.rs',93)
assert 'furniture::grip' in line('recipe/validation/dimensions.rs',54) and 'positive(p.length.get())' in line('recipe/validation/dimensions/furniture.rs',169)
nav=load(a['incidental_navigation']); raw=(R/nav['reconstructed_transcript']).read_bytes()
assert sha(raw)==nav['transcript_sha256'] and len(raw.splitlines())==nav['transcript_lines']==38
assert 'not established as pinned 95e bytes' in nav['transcript_qualification'] and 'may not preserve exact original ordering' in nav['transcript_qualification']
q=load('RECONCILIATION-059.json')['entries'][287];assert q['contract_id']=='SEGOVB-L12-HOLDER-ORIGINAL-OPTIONAL-ID-EXPECT' and 'ASSESSMENT-CENTRAL-HOLDER-13F-95E.json' in q['current_reconciliation']
rows=[x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'] if x['id']==q['contract_id']];assert len(rows)==1 and 'ASSESSMENT-CENTRAL-HOLDER-13F-95E.json' in rows[0]['additional_bounded_assessments']
assert a['repair']=='not started' and a['validation']=='not performed'
print('Verified bounded holder source relationship and separately qualified unregistered navigation; no construction or product execution inferred.')
