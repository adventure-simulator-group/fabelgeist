"""Verify revision-bound terrain aggregation evidence, not product behavior."""
import gzip,hashlib,json,tarfile
from fractions import Fraction
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-TERRAIN-HOURS-95E.json'; doc=json.loads((root/name).read_text()); base=json.loads((root/'BASELINE-95E-20261110.json').read_text()); progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 out={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); assert p not in out; out[p]=oid
 return out
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev; raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']]; assert s['path'] not in source and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); source[s['path']]=raw.decode()
assert len(source)==5
web=source['crates/strategic-web/src/templates/settlement/character_skills.rs']; core=source['crates/adventuresim-core/src/skill.rs']; rank=source['crates/strategic-web/src/templates/settlement/character_skills/rank.rs']; js=source['crates/strategic-web/static/tooltips.js']; details=source['crates/strategic-web/src/templates/settlement/character_details.rs']
segment=web[web.index('fn terrain_skill_rows('):web.index('\nfn language_skill_rows(',web.index('fn terrain_skill_rows('))]
for label in ('Plains','Forest','Hills','Wetlands','Urban','Snow'): assert f'"{label}"' in segment
for marker in ('let average_hours = entries','/ 6.0;','let average_effective_hours = entries','effective_skill_hours(entry.2)','/ 5.0;','&SkillTooltip::aggregate(','average_hours,\n                        average_effective_hours,'): assert marker in segment
for marker in ('fn terrain_family_rank(', 'correlated_hours: (effective_hours - trained_hours).max(0.0)', 'self.trained_hours + self.correlated_hours', 'Skill::TerrainSnow => skills.terrain_snow_hours', 'fn terrain_rows_render_intuitive_cross_habitat_projection()'): assert marker in web
for marker in ('Self::TerrainPlains => &[','(Self::TerrainForest, 0.20)','Self::TerrainSnow => &[','fn effective_skill_hours(&self, skill: Skill)','let transferred = if skill.is_trained()','direct + transferred'): assert marker in core
for marker in ('tooltip.accessible_description()', 'serde_json::to_string(tooltip)', 'data-strategic-tooltip=(tooltip_description)', 'data-skill-tooltip=[tooltip_json]'): assert marker in rank
for marker in ('Number(skill.trained_hours) + Number(skill.correlated_hours)','Number(skill.correlated_hours).toFixed(1)','if (renderSkillTooltip(target)) return;'): assert marker in js
assert 'party_skills_rail(' in details and '@if let Some(skills) = skills' in web
h=doc['historical_source']; old=json.loads((root/h['ledger']).read_text())['discovery_findings_and_dependencies'][h['contract_id']]; assert h['checkpoint']=='059' and old['accepted_unique_count_change']==0 and old['current_reconciled'] is False and h['historical_source_role']['sha256'] in old['source_key']; r=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles'][465]; x=h['historical_source_role']; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashlib.sha256(web.encode()).hexdigest() and x['relation']=='byte-match'
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as tar: assert hashlib.sha256(tar.extractfile(x['sha256']).read()).hexdigest()==x['sha256']
direct=Fraction(1000,6); effective_sum=Fraction(1000)+5*Fraction(1000,5); assert effective_sum==2000 and effective_sum/5==400 and effective_sum/6==Fraction(1000,3) and effective_sum/5-direct==Fraction(700,3)
c=doc['candidates'][0]; entries=base['newly_assessed_ledger']['entries']; found=[x for x in entries if x['id']==c['id']]; assert len(found)==1 and found[0]['record']==name and progress['new_current_ledger_candidates']==len(entries) and c['discovery'].startswith('partial') and c['repair']=='not started' and c['validation']=='not performed'
print('verified five Git sources, source arithmetic and qualified 059 attribution')
