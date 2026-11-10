"""Verify source-bound timeout accounting and historical attribution only."""
import gzip,hashlib,json,struct,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-ITERATION-TIMEOUT-95E.json'; doc=json.loads((root/name).read_text()); base=json.loads((root/'BASELINE-95E-20261110.json').read_text()); progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 d={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); assert p not in d; d[p]=oid
 return d
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev; raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']]; assert s['path'] not in source and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); source[s['path']]=raw.decode()
assert len(source)==8
it=source['crates/adventuresim-tactical-server/src/iteration.rs']; clock=source['crates/adventuresim-tactical-server/src/iteration/clock.rs']; net=source['crates/adventuresim-tactical-netcode/src/lib.rs']; runner=source['crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/runner.rs']; report=source['crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/mod.rs']; measurement=source['crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/measurement.rs']; shape=source['crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/report.rs']; outcome=source['crates/adventuresim-tactical-server/src/iteration/logging.rs']
for marker in ('const MAX_DUEL_SECONDS: u64 = 180;', '1.0 / TACTICAL_TICK_HZ as f32', 'let maximum_ticks = MAX_DUEL_SECONDS * u64::from(TACTICAL_TICK_HZ);', 'let mut final_tick = maximum_ticks;', 'for tick in 0..maximum_ticks {', 'app.update();', 'final_tick = tick;', 'simulated_ticks: final_tick + 1,', 'TacticalElapsed::from_tick_count(final_tick + 1)'):
 assert marker in it,marker
assert 'pub const TACTICAL_TICK_HZ: u32 = 30;' in net and 'Self(ticks as f32 * TACTICAL_TICK_SECONDS)' in clock
assert 'outcome.simulated_seconds' in runner and 'Measurement::<TacticalSeconds>::from_elapsed' in runner and 'Self::from_native(elapsed.seconds())' in measurement
for marker in ('tactical_total_simulated_seconds: tactical.simulated_seconds','tactical_mean_simulated_seconds: tactical.simulated_seconds.mean(args.tactical_seeds)','per_wall_second(tactical.wall_seconds)'):
 assert marker in report
assert 'pub(super) tactical_total_simulated_seconds:' in shape and 'pub simulated_ticks: u64' in outcome and 'pub simulated_seconds: TacticalElapsed' in outcome
f32=lambda x:struct.unpack('!f',struct.pack('!f',x))[0]; tick=f32(1/30); assert 180*30==5400 and f32(5400*tick)==180.00001525878906 and f32(5401*tick)==180.03334045410156
h=doc['historical_source']; ledger=json.loads((root/h['ledger']).read_text()); old=ledger['discovery_findings_and_dependencies']['ADLER-L11-ITERATION-TIMEOUT-TICK-COUNT']; assert h['checkpoint']=='059' and old['accepted_unique_count_change']==0 and '11520' in json.dumps(old) and '64' in json.dumps(old)
roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; manifest=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in manifest['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as tar:
 for x in h['historical_source_roles']:
  r=roles[x['index']]; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashes[x['index']] and hashlib.sha256(tar.extractfile(x['sha256']).read()).hexdigest()==x['sha256']; p=x['source_id'].split(':',1)[1]; sha=hashlib.sha256(source[p].encode()).hexdigest(); assert x['current_sha256']==sha and x['relation']==('byte-match' if x['sha256']==sha else 'different')
entries=base['newly_assessed_ledger']['entries']; c=doc['candidates'][0]; found=[x for x in entries if x['id']==c['id']]; assert progress['new_current_ledger_candidates']==len(entries) and len(found)==1 and found[0]['record']==name and c['repair']=='not started' and c['validation']=='not performed' and c['discovery'].startswith('partial')
print('verified eight Git sources, 30 Hz source arithmetic and historical 64 Hz attribution')
