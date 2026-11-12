#!/usr/bin/env python3
"""Verify bounded old/current source and connected chart claim without geometry execution."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();a=load('ASSESSMENT-CENTRAL-BUFFE-13J-95E.json');refs=[];bodies={}
for tag in ('13J','13J-SUPPORT','13J-WEB','13J-TRACE','13J-HEAD'):
 reg=load('CENTRAL-'+tag+'-REGISTRATION.json');refs.extend({'revision':x['revision'],'path':x['path'],'lines':x['planned_display_ranges']} for x in reg['rows'])
 with tarfile.open(R/('delegated/central-'+tag.lower()+'-windows-proof.tar.gz')) as t:
  p=json.load(t.extractfile('proof.json'))
  for x in p['paths']:
   raw=t.extractfile(p['objects'][x['blob']]['archive']).read();assert sha(raw)==x['sha256'];bodies[(x['revision'],x['path'])]=raw
assert refs==[{k:x[k] for k in ('revision','path','lines')} for x in a['source_references']] and len(refs)==15
for i in (0,2):assert a['source_references'][i]['revision_role']=='historical object transport, not the old project head' and a['source_references'][i]['recorded_historical_head']=='6533e444fa37552927070cc8d8190907fd3b413e'
cur=a['source_revision'];old='6533e444fa37552927070cc8d8190907fd3b413e';tr='f82adcabd9d89c4fa2dd7f99253a45297a0681ee';prefix='handoff/774/audit/reconcile-adler-20261023/recovered-sources/objects/'
for path,h in [('crates/fabelgeist-armor/src/visor_breaths.rs','bf67732e56b2c3559623ec840f391fcfcaf6a1eaeaf35ecc9d7a35988d69f59b'),('crates/fabelgeist-armor/src/helmets_buffe_courses.rs','53819c919b26e8d78494cceda4e24c061f1c136148d78f3436b293d368b15746')]:assert bodies[(tr,prefix+h)]==bodies[(old,path)]==bodies[(cur,path)] and sha(bodies[(cur,path)])==h
def line(path,n):return bodies[(cur,'crates/fabelgeist-armor/src/'+path)].decode().splitlines()[n-1]
assert 'upper_boundary: Permille(700)' in line('helmets_buffe_courses.rs',25) and 'overlap: Millimeters(5)' in line('helmets_buffe_courses.rs',26)
assert 'length: Millimeters(12)' in line('visor_breaths.rs',59) and 'inclination: SlotInclination(0)' in line('visor_breaths.rs',63) and 'b.count_per_row == 0' in line('visor_breaths.rs',86) and 'f64::from(b.height.0) * chart_height' in line('visor_breaths.rs',132)
assert 'MINIMUM_WEB_MM: f64 = 3.0' in line('pierced_plate_domain.rs',6) and '!inside(*p, outer)' in line('pierced_plate_domain.rs',23) and 'VisorOpeningSpacing' in line('pierced_plate_domain.rs',25)
assert 'courses.validate()' in line('helmets_buffe.rs',57) and 'breaths.validate(50..=950)' in line('helmets_buffe.rs',60)
assert 'design.validate()' in line('gpu/helmet/mod.rs',28) and 'CoordKernel::new' in line('gpu/helmet/mod.rs',29) and 'HelmetDesign::Burgonet' in line('gpu/helmet/parts.rs',129) and 'buffe.validate()' in line('helmets.rs',354)
assert 'buffe(b, gauge)' in line('gpu/helmet/burgonet.rs',173) and 'b.breaths.filter' in line('gpu/helmet/burgonet.rs',138) and 'b.courses.map_or(1' in line('gpu/helmet/burgonet.rs',139) and 'BUFFE_CHART_HEIGHT_MM *' in line('gpu/helmet/burgonet.rs',152) and 'breaths.openings(BUFFE_CHART_HEIGHT_MM' in line('gpu/helmet/burgonet.rs',268) and 'PiercedDomain::new' in line('gpu/helmet/burgonet.rs',269)
assert 'Self::BakeSource => 1' in line('sampling.rs',41) and '.max(minimum).min(source)' in line('sampling.rs',46)
rows=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];r=next(x for x in rows if x['id']=='REVIEW-BUFFE-BREATH-COURSE-CONTEXT-95E');assert 'ASSESSMENT-CENTRAL-BUFFE-13J-95E.json' in r['additional_bounded_assessments'] and a['new_qualified_ledger_rows']==[]
q=load('RECONCILIATION-059.json')['entries'][3141];assert q['contract_id']=='REVIEW-L3-HELMET-CARRIER-TOPOLOGY-ADMISSION' and 'not an alias' in q['current_reconciliation']
for i in (2613,2614):r=load('COVERAGE-L01-ROLES.json')['roles'][i];assert r['snapshot']=='pr-working' and r['status']=='unassessed'
assert a['repair']=='not started' and a['validation']=='not performed' and load('CONTINUATION-PROGRESS-95E.json')['queue_records_semantically_reconciled']==0
print('Verified separately attributed buffe sources and conditional chart boundary; no geometry, device, product execution or alias closure inferred.')
