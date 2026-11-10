"""Verify pinned fraction-admission evidence without executing product code."""
import gzip,hashlib,json,tarfile,tomllib
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-BASIS-WIRE-95E.json'; doc=json.loads((root/name).read_text()); base=json.loads((root/'BASELINE-95E-20261110.json').read_text()); progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 d={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); assert p not in d; d[p]=oid
 return d
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev; raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']]; assert s['path'] not in source and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); source[s['path']]=raw.decode()
assert len(source)==9
unit=source['crates/adventuresim-world-schema/src/basis_points.rs']; inv=source['crates/adventuresim-core/src/investigation.rs']; exposure=source['crates/adventuresim-stdb-module/src/disease/exposure_sources.rs']; disease=source['crates/adventuresim-stdb-module/src/disease.rs']; local=source['crates/adventuresim-core/src/local_problem.rs']; claims=source['crates/adventuresim-stdb-module/src/investigation/claims.rs']; manifest=source['Cargo.toml']; schema=source['crates/adventuresim-world-schema/Cargo.toml']
assert 'features = [\n  "alloc",\n  "derive",' in manifest and 'serde.workspace = true' in schema
assert unit.count('#[serde(transparent)]')==2 and 'pub struct UnitBasisPoints {' in unit and 'pub struct SignedUnitBasisPoints {' in unit and 'basis_points: u16' in unit and 'basis_points: i16' in unit
for marker in ('value <= BASIS_POINTS_PER_WHOLE', 'value >= Self::MIN.basis_points && value <= Self::MAX.basis_points', 'BASIS_POINTS_PER_WHOLE - self.basis_points', 'whole as u64 * self.basis_points as u64', 'f32::from(self.basis_points) / f32::from(BASIS_POINTS_PER_WHOLE)'): assert marker in unit
assert 'pub struct BasisPoints(UnitBasisPoints);' in inv and '#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]' in inv
for kind in ('Observation','Recollection','Claim','Evidence','BeliefRevision','Belief'):
 chunk=inv.split('pub struct '+kind+' {',1)[1].split('\n}',1)[0]; assert 'pub confidence: BasisPoints' in chunk
pipeline=inv.split('pub struct PipelineInput {',1)[1].split('\n}',1)[0]; assert 'BasisPoints' not in pipeline and 'confidence' not in pipeline and 'confidence: input.perception.confidence()' in inv
assert 'serde_json::from_str(&pipeline_json)' in claims and 'inv::PipelineInput' in claims and 'stage_json: serde_json::to_string(&observation)' in claims
assert 'row.mitigation_bps < adventuresim_world_schema::BASIS_POINTS_PER_WHOLE' in exposure and 'UnitBasisPoints::new(row.mitigation_bps)' in exposure and 'problem.mitigation_bps >= adventuresim_world_schema::BASIS_POINTS_PER_WHOLE' in disease and 'UnitBasisPoints::new(problem.mitigation_bps)' in disease and 'mitigation.complement().as_unit_f32()' in local
packages={(p['name'],p['version']):p for p in tomllib.loads(source['Cargo.lock'])['package']}; native={}
for n in doc['native_sources']:
 p=(root/n['package_manifest']).resolve(); assert p.is_relative_to(root); m=json.loads(p.read_text()); archive=p.parent/m['archive']; raw=archive.read_bytes(); assert len(raw)==m['bytes'] and hashlib.sha256(raw).hexdigest()==m['sha256']==packages[(m['package'],m['version'])]['checksum'];
 with tarfile.open(archive,'r:gz') as t: body=t.extractfile(n['archive_member']).read()
 assert len(body)==n['bytes'] and hashlib.sha256(body).hexdigest()==n['sha256'] and any(x['path']==n['archive_member'] and x['sha256']==n['sha256'] for x in m['inspected_members']) and all(1<=a<=b<=len(body.splitlines()) for a,b in n['ranges']); native[n['archive_member']]=body.decode()
assert len(native)==4
derive=native['serde_derive-1.0.229/src/de.rs']; tuple_=native['serde_derive-1.0.229/src/de/tuple.rs']; core=native['serde_core-1.0.229/src/de/impls.rs']; parser=native['serde_json-1.0.151/src/de.rs']
for marker in ('if cont.attrs.transparent() {', 'deserialize_transparent(cont, params)', 'quote!(#member: __transparent)', '|__transparent| #this_value { #(#assign),* }'): assert marker in derive
assert 'quote!(#type_path(__field0))' in tuple_ and 'deserialize_newtype_struct' in tuple_
for marker in ('u16, NonZeroU16, deserialize_u16', 'i16, NonZeroI16, deserialize_i16', 'uint_to_self!(u32:visit_u32 u64:visit_u64)', 'Self::Value::try_from(v as u64)', 'Self::Value::try_from(v as i64)'): assert marker in core
for marker in ('ParserNumber::U64(x) => visitor.visit_u64(x)', 'ParserNumber::I64(x) => visitor.visit_i64(x)', 'deserialize_number!(deserialize_u16)', 'deserialize_number!(deserialize_i16)', 'ParserNumber::U64(significand)', 'visitor.visit_newtype_struct(self)'): assert marker in parser
assert 10001<=65535 and -32768<=-10001<=32767 and 10000*10001//10000==10001
h=doc['historical_source']; ledger=json.loads((root/h['ledger']).read_text()); old=ledger['discovery_findings_and_dependencies']['ADLER-L11-BASIS-POINTS-WIRE-ADMISSION']; assert h['checkpoint']=='059' and old['accepted_unique_count_change']==0 and old['current_reconciled'] is False and 'PL13-BASIS-POINTS-NATIVE-ADMISSION' in json.dumps(ledger)
roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; reg=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in reg['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as tar:
 for x in h['historical_source_roles']:
  r=roles[x['index']]; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashes[x['index']] and hashlib.sha256(tar.extractfile(x['sha256']).read()).hexdigest()==x['sha256']; p=x['source_id'].split(':',1)[1]; sha=hashlib.sha256(source[p].encode()).hexdigest(); assert x['current_sha256']==sha and x['relation']==('byte-match' if x['sha256']==sha else 'different')
entries=base['newly_assessed_ledger']['entries']; c=doc['candidates'][0]; found=[x for x in entries if x['id']==c['id']]; assert progress['new_current_ledger_candidates']==len(entries) and len(found)==1 and found[0]['record']==name and c['repair']=='not started' and c['validation']=='not performed' and c['discovery'].startswith('partial')
print('verified nine Git sources, four lock-bound native members and qualified 059 attribution')
