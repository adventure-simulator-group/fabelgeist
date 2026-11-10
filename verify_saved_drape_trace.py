"""Verify saved-drape source, recovered records and reconstructed word layout."""
import base64
import gzip
import hashlib
import json
import struct
from pathlib import Path

root=Path(__file__).resolve().parent
revision='95e63628f9d70b8dd6e6fede64334696c6995009'
name='ASSESSMENT-L01-SAVED-DRAPE-95E.json'
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
assert len(sources)==10
p='crates/adventuresim-character-creator/src/'
def must(source,markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: '+marker)
settled=sources[p+'garment/settled.rs']
must(settled,['#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]',
    'pub struct SettledDrape', '#[serde(with = "packed")]', 'const WORDS: usize = 6;',
    'triangle: words[0]', 'drape.validate()?;', 'binding.triangle < self.body.triangles',
    'pub fn wear(&self, input: &DrapeInput) -> DrapeOutcome',
    'let result = self.fit(input, &mut warnings);', 'body == self.body',
    'map(|binding| array(wearer.place(binding)))',
    'self.frame(binding.triangle, binding.barycentric)', 'self.tree.triangle(triangle)'])
wear=settled[settled.index('pub fn wear(&self, input: &DrapeInput)'):settled.index('/// Garments are saved once')]
assert '.validate()' not in wear
packed=sources[p+'garment/settled/packed.rs']
must(packed,['STANDARD.decode(text).map_err(D::Error::custom)?',
    'if bytes.len() % record != 0', 'words.chunks_exact(T::WORDS).map(T::unpack).collect()'])
must(sources[p+'garment.rs'], ['pub use settled::{BodyTopology, SettledDrape, SettledGarment};',
    'pub use outfit::{OutfitOutcome, drape_outfit};', 'pub settled: Option<std::sync::Arc<SettledDrape>>'])
must(sources[p+'garment/outfit.rs'], ['match input.settled.clone()', 'let outcome = saved.wear(&input);'])
must(sources['crates/fabelgeist-bvh/src/cpu/mod.rs'], ['let [a, b, c] = self.triangles[index as usize];'])
must(sources[p+'wardrobe.rs'], ['wardrobe\n            .validate()', 'garment.validate()?;'])
must(sources[p+'inventory.rs'], ['Self::Settled(garment) => garment', 'item.article.validate()?;'])
must(sources[p+'lib.rs'], ['pub fn validate(&self) -> Result<(), String>', 'self.inventory.validate()'])
must(sources[p+'main.rs'], ['let recipe: CharacterRecipe = serde_json::from_slice',
    'recipe.validate().map_err(anyhow::Error::msg)?;'])
must(sources[p+'garment/settled/tests.rs'], ['loaded.validate().unwrap();', 'fn a_truncated_packed_array_is_rejected()'])
reference=doc['reconstructed_witness'];path=(root/reference['file']).resolve()
assert path.is_relative_to(root) and hashlib.sha256(path.read_bytes()).hexdigest()==reference['sha256']
witness=json.loads(path.read_text())
assert witness['body']=={'vertices':3,'triangles':1}
def words(field,fmt):
    raw=base64.b64decode(witness[field],validate=True)
    assert len(raw)%4==0
    return struct.unpack('<'+str(len(raw)//4)+fmt,raw)
positions=words('positions','f');uv=words('texcoords','f');faces=words('faces','I');bindings=words('bindings','I')
assert len(positions)==9 and len(uv)==6 and faces==(0,1,2) and len(bindings)==18
assert bindings[:6]==(1,0,0,0,0,0) and bindings[0]>=witness['body']['triangles']
assert all(i<witness['body']['vertices'] for i in faces)
body=reference['hypothetical_body'];assert len(body['positions'])==3 and len(body['faces'])==1
assert body['faces'][0]==[0,1,2]
ledger=json.loads((root/'baseline-059/ledger.json').read_text())
old=ledger['segovb_continuation']['lens13']['delegated_findings_pending_primary_provenance_and_alias'][10]
under=ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][183]
assert old['finding']['proposed_id']==doc['historical_records'][0]['id']
assert old['primary_function_body_provenance_pending'] and old['current_reconciled'] is False
assert under['contract']['contract']==doc['historical_records'][1]['id']
expected=(old['receipt_sha256'],old['delegated_receipt']['sha256'],old['finding']['primary_registration']['sha256'],under['receipt_sha256'])
for reference,digest in zip(doc['recovered_historical_receipts'],expected):
    path=(root/reference['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and reference['content_encoding']=='gzip'
    raw=gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest()==reference['sha256']==digest
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
print('verified ten Git-bound sources, four recovered receipts and reconstructed packed words; no product ran')
