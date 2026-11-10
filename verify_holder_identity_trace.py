"""Verify the preserved holder identity source chain and historical attribution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-HOLDER-IDENTITY-95E.json'; doc=json.loads((root/name).read_text()); base=json.loads((root/'BASELINE-95E-20261110.json').read_text()); progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 d={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); assert p not in d; d[p]=oid
 return d
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev; raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']]; assert s['path'] not in source and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); source[s['path']]=raw.decode()
assert len(source)==17
prefix='crates/adventuresim-weapon-model/src/'
recipe=source[prefix+'recipe/mod.rs']; rv=source[prefix+'recipe/validation.rs']; validation=source[prefix+'validation.rs']; model=source[prefix+'model/mod.rs']; placement=source[prefix+'model/placement.rs']; holders=source[prefix+'holders.rs']; codec=source[prefix+'codec.rs']; evaluation=source[prefix+'evaluation.rs']; derived=source[prefix+'derive.rs']; catalog=source[prefix+'catalog.rs']; browser=source['crates/adventuresim-weapon-model-browser/src/lib.rs']; smith=source['crates/adventuresim-stdb-module/src/smithing.rs']; instance=source['crates/adventuresim-stdb-module/src/weapon_instance.rs']; instance_eval=source['crates/adventuresim-stdb-module/src/weapon_instance/evaluation.rs']; carry=source['crates/adventuresim-stdb-module/src/character/weapon_carry.rs']
for marker in ('pub id: Option<String>', 'self.id\n            .clone()\n            .or_else(|| self.label.clone())\n            .unwrap_or_else(|| format!("component-{index}"))'): assert marker in recipe
assert 'let id = component.resolved_id(index);' in rv and 'identifiers.insert(id)' in rv
for marker in ('identity(&design.fitted_weapon)?;', 'part.role == Some(ComponentRole::Grip)', 'matches!(part.shape, crate::recipe::Shape::LoftedBlade(_))', 'HolderConstruction::new(design)'): assert marker in validation
assert 'recipe.validate()' in model and 'placement::resolve(recipe)?' in model
for marker in ('let mut recipe = recipe.clone();', 'let id = component.resolved_id(index);', 'component.id = Some(id.clone());'): assert marker in placement
assert holders.count('component.id.as_deref().expect("gameplay component ID")')==2 and 'find(|component| component.role == Some(ComponentRole::Grip))' in holders and 'matches!(&component.shape, crate::recipe::Shape::LoftedBlade(_))' in holders
assert 'validate_holder(design)' in codec and 'validate_holder(&design)' in codec and 'derive_holder_properties(&design)' in evaluation and 'HolderConstruction::new(design)' in derived
assert 'default_holder_design' in catalog and 'fitted_weapon: weapon.clone()' in catalog and 'Self::GenerateHolder { design } => serde_json::to_value(generate_holder(&design)?)?' in browser and 'pub fn weapon_model_request' in browser
assert 'let design = decode(&recipe)' in smith and 'replace_design(ctx, object.id, &design)' in smith and 'default_holder_design(weapon_design)' in instance and 'EvaluatedHolder::new(design.clone())' in instance_eval and 'fit_personal_holder(ctx, character_id, holder, weapon.id)' in carry
weapon=next(x for x in json.loads(source['crates/adventuresim-weapon-model/catalog/gameplay.json']) if x['catalog_id']=='longsword'); blade=next(x for x in weapon['recipe']['components'] if x['kind']=='loftedBlade'); assert blade['id']==blade['label']=='blade'; assert 'longsword' in catalog and 'WeaponHolderKind::BladeSheath' in catalog
h=doc['historical_source']; ledger=json.loads((root/h['ledger']).read_text()); old=ledger['discovery_findings_and_dependencies']['SEGOVB-L12-HOLDER-ORIGINAL-OPTIONAL-ID-EXPECT']; assert h['checkpoint']=='059' and old['accepted_unique_count_change']==0 and old['current_source_reconciled'] is False and 'PL13-HOLDER-OPTIONAL-COMPONENT-ID' in json.dumps(ledger)
roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; manifest=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in manifest['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as tar:
 for x in h['historical_source_roles']:
  r=roles[x['index']]; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashes[x['index']] and hashlib.sha256(tar.extractfile(x['sha256']).read()).hexdigest()==x['sha256']; p=x['source_id'].split(':',1)[1]; sha=hashlib.sha256(source[p].encode()).hexdigest(); assert x['current_sha256']==sha and x['relation']==('byte-match' if x['sha256']==sha else 'different')
entries=base['newly_assessed_ledger']['entries']; c=doc['candidates'][0]; found=[x for x in entries if x['id']==c['id']]; assert progress['new_current_ledger_candidates']==len(entries) and len(found)==1 and found[0]['record']==name and c['repair']=='not started' and c['validation']=='not performed' and c['discovery'].startswith('partial')
print('verified 17 Git sources, conditional holder identity chain and historical 059 attribution')
