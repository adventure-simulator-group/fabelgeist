"""Verify source and recovered lining evidence without running product code."""
import gzip
import hashlib
import json
from pathlib import Path

root=Path(__file__).resolve().parent
revision='95e63628f9d70b8dd6e6fede64334696c6995009'
name='ASSESSMENT-L01-PLATE-LINING-95E.json'
doc=json.loads((root/name).read_text())
assert doc['lens']==1 and doc['source_revision']==revision

def tree(path):
    result={}
    for line in (root/path).read_text().splitlines():
        header,filename=line.split('\t',1)
        _,kind,oid=header.split()
        assert filename not in result and kind in ('blob','commit')
        result[filename]=oid
    return result

current=tree('main-advance-95e/current-tree.tsv')
previous=tree('main-advance-95e/previous-tree.tsv')
sources={}
for item in doc['current_sources']:
    path=(root/item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding']=='gzip'
    raw=gzip.decompress(path.read_bytes())
    oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
    assert item['revision']==revision and oid==item['git_blob']
    assert current[item['path']]==previous[item['path']]==oid
    assert hashlib.sha256(raw).hexdigest()==item['sha256']
    assert len(raw)==item['bytes'] and len(raw.splitlines())==item['lines']
    assert all(1<=a<=b<=item['lines'] for a,b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']]=raw.decode()
assert len(sources)==9
p='crates/adventuresim-character-creator/src/'
a='crates/fabelgeist-armor/src/'
def must(source,markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: '+marker)
must(sources[p+'lib.rs'], ['pub mod garment;'])
must(sources[p+'garment.rs'], ['pub use lining::{PlateLining, UnderPlate};'])
lining=sources[p+'garment/lining.rs']
must(lining,['pub fn new<\'a>(pieces: impl IntoIterator<Item = &\'a GeneratedArmor>) -> Option<Self>',
    'piece.indices.as_chunks::<3>().0.iter().zip(&piece.faces)',
    'if *face == PlateFace::Inner', 'triangles.push(triangle.map(|i| i + offset));',
    'if triangles.is_empty()', 'return None;',
    'let key = |i: u32| positions[i as usize].to_array().map(f32::to_bits);',
    'for (from, to) in [(a, b), (b, c), (c, a)]',
    'uses.entry(edge(from, to)).or_default()'])
assert lining.index('if triangles.is_empty()') < lining.index('let key = |i: u32|') < lining.index('uses.entry(edge(from, to))')
must(sources[a+'design.rs'], ['pub struct GeneratedArmor', 'pub positions: Vec<[f32; 3]>',
    'pub indices: Vec<u32>', 'pub faces: Vec<crate::PlateFace>'])
must(sources[a+'plate_face.rs'], ['pub enum PlateFace', 'Inner,'])
outfit=sources[p+'outfit.rs']
must(outfit,['PlateLining::new(', 'parametric_equipment::is_rigid(piece)', 'standoff: over + half'])
cache=sources[p+'studio_cache.rs']
must(cache,['fits: HashMap<Key, GeneratedArmor>', 'parametric_equipment::fit(model, generated, piece',
    'PlateLining::new(keys.iter().filter_map(|key| self.fits.get(key)))'])
must(sources[p+'parametric_equipment.rs'], ['pub(super) fn fit(', 'fitted_catalog_item(model, generated, placement'])
must(sources[p+'garment/drape.rs'], ['if let Some(under) = &input.under_plate', 'cloth.tuck_under(&output.positions, under)'])
# One selected local inner triangle with three positions indexes past the end.
triangle=(0,1,3);count=3
edges=((triangle[0],triangle[1]),(triangle[1],triangle[2]),(triangle[2],triangle[0]))
assert all(i<count for i in edges[0]) and edges[1][1]>=count
ledger=json.loads((root/'baseline-059/ledger.json').read_text())
old=ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias']
for reference,index in zip(doc['recovered_historical_receipts'],(183,204)):
    path=(root/reference['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and reference['content_encoding']=='gzip'
    raw=gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest()==reference['sha256']==old[index]['receipt_sha256']
for record,index in zip(doc['historical_records'],(183,204)):
    assert record['id']==old[index]['contract']['contract']
work=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())
for reference in doc['historical_source_roles']:
    role=work['roles'][reference['original_index']]
    assert role['source_id']==reference['source_id'] and role['historical_sha256']==reference['historical_sha256']
    assert reference['byte_identical']==(reference['historical_sha256']==reference['current_sha256'])
base=json.loads((root/'BASELINE-95E-20261110.json').read_text())
progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text())
candidate=doc['candidates'][0];entries=base['newly_assessed_ledger']['entries']
assert sum(x['id']==candidate['id'] and x['record']==name for x in entries)==1
assert progress['new_current_ledger_candidates']==len(entries) and progress['queue_records_semantically_reconciled']==0
assert candidate['discovery'].startswith('partial') and candidate['repair']=='not started' and candidate['validation']=='not performed'
print('verified nine Git-bound sources, two recovered receipts and index bound; no product ran')
