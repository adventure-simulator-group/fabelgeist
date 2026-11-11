#!/usr/bin/env python3
"""Verify recovered tree-frame attribution and current working-ID alias."""
import json
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
a=load('RECONCILIATION-VISTA-TREE-FRAME-95E.json');old=a['superseded_working_id'];canonical=a['canonical_id']
assert canonical=='ADLER-L11-VISTA-TREE-PLACEMENT-FRAME' and old=='REVIEW-VISTA-TREE-ORIGIN-FRAME-001'
assert a['historical_revision']=='6533e444fa37552927070cc8d8190907fd3b413e' and a['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009'
historical=load('baseline-059/ledger.json')['discovery_findings_and_dependencies'][canonical]
assert 'tree_root_height' in ' '.join(historical['trace']) and 'local' in ' '.join(historical['trace'])
assert 'source frame' in historical['first_failed_contract'] and 'VistaLod' in historical['canonical_owner'] and historical['accepted_unique_count_change']==0
q=load('baseline-059/queue.json')['entries'][87];assert q['contract_id']==canonical and q['kind']=='retained-finding-or-question'
assessment=load(a['current_evidence']['record']);finding=assessment['conditional_findings'][0];assert finding['id']==old and 'tree_root_height' in finding['trace'] and 'local' in finding['first_failed_contract']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];entries=[x for x in ledger if x['id']==canonical];assert len(entries)==1 and not any(x['id']==old for x in ledger)
x=entries[0];assert x['former_working_ids']==[old] and x['record']==a['current_evidence']['record'] and 'RECONCILIATION-VISTA-TREE-FRAME-95E.json' in x['additional_assessments']
assert load('RECONCILIATION-059.json')['entries'][87]['current_reconciliation'].startswith('partial ') and a['historical_accepted_unique_count_change']==0 and a['repair']=='not started' and a['validation']=='not performed'
print('Verified recovered059 Adler identifier as the single current tree-frame ledger owner; no product execution.')
import gzip,hashlib,tarfile
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def tree_entries(b):
    out,pos={},0
    while pos<len(b):
        end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
    assert pos==len(b);return out
c=a['owned_origin_counterevidence'];proof=load(c['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][proof['revision']]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision'] and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():header,path=line.split('\t',1);tree[path]=header.split()[2]
sources={};w=load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
    for s in c['supporting_sources']:
        b=contents(s['preserved_current_bytes']);old=ar.extractfile(s['sha256']).read();path=s['path'];i=s['original_index'];record=proof['paths'][path]
        assert old==b and sha(b)==s['sha256']==w[i]['historical_sha256'] and oid('blob',b)==s['git_blob']==tree[path] and len(b)==s['bytes'] and len(b.splitlines())==s['lines']
        assert s['historical_personal_ordered_credit']==0 and s['historical_current_identical'] and all(1<=lo<=hi<=s['lines'] for lo,hi in s['current_personal_ranges'])
        assert [e['name'] for e in record['chain']]==path.split('/');current=proof['root_tree']
        for step in record['chain']:
            info=nodes[current];treebody=contents(info['preserved_bytes']);assert oid('tree',treebody)==current==step['tree'] and sha(treebody)==info['sha256'] and len(treebody)==info['bytes'];assert tree_entries(treebody)[step['name']]==(step['mode'],step['child']);current=step['child']
        assert current==record['git_blob']==oid('blob',b) and record['sha256']==sha(b);sources[path]=b
assert [s['original_index'] for s in c['supporting_sources']]==[1823,2699,2698,1964]
presented=sources['crates/adventuresim-tactical-core/src/city_layout/grounding/foundations/source/presented.rs'];supported=sources['crates/adventuresim-tactical-core/src/scene_input/generation/supported.rs'];grounding=sources['crates/adventuresim-tactical-core/src/scene_input/generation/grounding.rs'];vista=sources['crates/adventuresim-tactical-core/src/vista_surface.rs']
assert b'lod.origin_east_metres != 0.0' in presented and b'lod.origin_north_metres != 0.0' in presented
assert b'if let Some(projection) = &self.grounding' in supported and b'GeographicSurface::from_presented_scene' in supported and b'with_property_surface(support)' in supported and b'GeographicSurface::from_presented_scene' in grounding
assert b'native_stitch_vista_height_to_playable_edge' in vista and b'let boundary = local.clamp' in vista
seam='ADLER-L11-VISTA-PLAYABLE-SEAM-FRAME';assert seam in load('baseline-059/ledger.json')['discovery_findings_and_dependencies'];assert load('baseline-059/queue.json')['entries'][112]['contract_id']==seam
assert sum(x['id']==seam and x['record']=='RECONCILIATION-VISTA-TREE-FRAME-95E.json' for x in ledger)==1 and load('RECONCILIATION-059.json')['entries'][112]['current_reconciliation'].startswith('partial ')
assert 'RECONCILIATION-VISTA-TREE-FRAME-95E.json' in next(x for x in ledger if x['id']=='REVIEW-VISTA-OWNED-SURFACE-ORIGIN-FRAME-001')['additional_assessments']
print('Verified current grounded origin exclusion and recovered059 seam attribution; no product execution.')
