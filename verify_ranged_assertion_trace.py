"""Verify Git-bound source evidence and historical attribution only."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-RANGED-SOURCE-ASSERTION-95E.json'; doc=json.loads((root/name).read_text()); baseline=json.loads((root/'BASELINE-95E-20261110.json').read_text()); progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 d={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); assert p not in d; d[p]=oid
 return d
current=tree('main-advance-95e/current-tree.tsv'); previous=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 path=(root/s['preserved_bytes']).resolve(); assert path.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev
 raw=gzip.decompress(path.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==current[s['path']]==previous[s['path']]; assert s['path'] not in source and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); source[s['path']]=raw.decode()
assert len(source)==5
combat=source['crates/adventuresim-tactical-server/src/combat.rs']; ranged=source['crates/adventuresim-tactical-server/src/combat/ranged.rs']; auth=source['crates/adventuresim-tactical-server/src/combat/authority.rs']; timing=source['crates/adventuresim-tactical-server/src/combat/authority/ranged.rs']; defense=source['crates/adventuresim-tactical-server/src/combat/defense.rs']
assert '#[cfg(test)]\nmod tests' in combat and 'fn ranged_rejections_happen_before_ammo_scan()' in combat and 'include_str!("combat/ranged.rs")' in combat and 'resolver.find("q_ammo.iter().find").expect("ammo scan")' in combat
assert 'q_ammo.iter().find' not in ranged and 'ammunition.iter().find' in ranged
body=ranged.split('pub(super) fn resolve_ranged_attack(',1)[1]; markers=['authority_permits: authority.permits(now)','validate_ranged_intent(facts)','authoritative_line_of_sight(','CombatDuration::from_elapsed','RangedAmmunition::find(validated.attacker(), &q_ammo)','PreparedRangedDefense::prepare(','authority.authorize_shot(validated, now, cooldown)','let Some(ammunition) = ammunition else','ammunition.consume(&mut cmd, &q_ids, &mut consequences, &shot)']; positions=[body.index(x) for x in markers]; assert positions==sorted(positions)
assert 'if !facts.authority_permits {' in auth and 'return Err(RangedIntentRejection::Windup);' in auth and 'let valid = self.permits(now);' in timing and 'self.windup = None;' in timing and 'self.cooldown_until = now + cooldown;' in timing
assert 'fn prepare(' in ranged and 'return None;' in ranged.split("impl<'v> PreparedRangedDefense",1)[1].split('impl RangedAmmunition',1)[0] and 'fn resolve_passive_block(' in defense
h=doc['historical_source']; assert h['checkpoint']=='059'; ledger=json.loads((root/h['ledger']).read_text()); old=ledger['discovery_findings_and_dependencies']['ADLER-L11-RANGED-AUTHORIZATION-SOURCE-TEST']; assert old['accepted_unique_count_change']==0 and old['repair_status'].startswith('notstarted')
roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; manifest=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in manifest['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as tar:
 for x in h['historical_source_roles']:
  r=roles[x['index']]; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashes[x['index']] and hashlib.sha256(tar.extractfile(x['sha256']).read()).hexdigest()==x['sha256']; p=x['source_id'].split(':',1)[1]; sha=hashlib.sha256(source[p].encode()).hexdigest(); assert x['current_sha256']==sha and x['relation']==('byte-match' if x['sha256']==sha else 'different')
entries=baseline['newly_assessed_ledger']['entries']; c=doc['candidates'][0]; matches=[x for x in entries if x['id']==c['id']]; assert progress['new_current_ledger_candidates']==len(entries) and len(matches)==1 and matches[0]['record']==name and c['repair']=='not started' and c['validation']=='not performed' and c['discovery'].startswith('partial')
print('verified five Git-bound sources, staged source order, and separately attributed history')
