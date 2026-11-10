"""Verify source-bound cumulative leisure arithmetic and qualified attribution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-DAILY-SHARE-95E.json'; doc=json.loads((root/name).read_text()); base=json.loads((root/'BASELINE-95E-20261110.json').read_text()); progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 d={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); assert p not in d; d[p]=oid
 return d
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev; raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']]; assert s['path'] not in source and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); source[s['path']]=raw.decode()
assert len(source)==6
calendar=source['crates/adventuresim-world-schema/src/calendar.rs']; schedule=source['crates/adventuresim-core/src/strategic_schedule.rs']; allocation=source['crates/adventuresim-core/src/strategic_schedule/allocation.rs']; clock=source['crates/adventuresim-core/src/strategic_time.rs']; rest=source['crates/adventuresim-stdb-module/src/time/settlement_rest.rs']; surgery=source['crates/adventuresim-stdb-module/src/surgery.rs']
for marker in ('pub const MINUTES_PER_DAY: u64 = 24 * 60;', 'pub const MAX: Self = Self::new(u64::MAX);', 'self.minutes.saturating_mul(share_minutes) / MINUTES_PER_DAY', 'if elapsed_years > (i32::MAX - WORLD_START_YEAR.year) as u64'): assert marker in calendar
for marker in ('MINUTES_PER_DAY.saturating_sub(schedule.allocated_minutes())', '.cumulative_daily_share_minutes(leisure)', '.saturating_sub(interval_start_minute.cumulative_daily_share_minutes(leisure))'): assert marker in schedule
assert 'pub labor: u16' in allocation and 'pub fn allocated_minutes(&self) -> u64' in allocation and 'if schedule.allocated_minutes() > MINUTES_PER_DAY' in allocation and 'pub const MAX_SETTLEMENT_REST_MINUTES: u64 = MINUTES_PER_YEAR;' in clock
for marker in ('validate_settlement_rest_minutes(requested_minutes)?', 'restorative_leisure_minutes(', 'InjuryRecoveryMinutes::new(requested_recovery)', 'InjuryRecoveryMinutes::new(recovery_elapsed)', 'recovery_elapsed.saturating_sub(convalescing)', 'checked_add_minutes(elapsed)'): assert marker in rest
for marker in ('pub(crate) fn preview_injury_boundary(', 'pub(crate) fn settle_injuries(', 'recovery.get().min(requested) as f32', 'recovery.get().min(interval.elapsed) as f32'): assert marker in surgery
M=(1<<64)-1; share=720; start=M//share+1; end=start+1440; assert start==25620477880152156 and end<=M; actual=min(end*share,M)//1440-min(start*share,M)//1440; exact=end*share//1440-start*share//1440; assert actual==0 and exact==720
h=doc['historical_source']; ledger=json.loads((root/h['ledger']).read_text()); old=ledger['discovery_findings_and_dependencies']['ADLER-L11-CALENDAR-DAILY-SHARE-ARITHMETIC']; assert h['checkpoint']=='059' and old['accepted_unique_count_change']==0 and old['current_reconciled'] is False
roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; reg=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in reg['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as tar:
 for x in h['historical_source_roles']:
  r=roles[x['index']]; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashes[x['index']] and hashlib.sha256(tar.extractfile(x['sha256']).read()).hexdigest()==x['sha256']; p=x['source_id'].split(':',1)[1]; sha=hashlib.sha256(source[p].encode()).hexdigest(); assert x['current_sha256']==sha and x['relation']==('byte-match' if x['sha256']==sha else 'different')
entries=base['newly_assessed_ledger']['entries']; c=doc['candidates'][0]; found=[x for x in entries if x['id']==c['id']]; assert progress['new_current_ledger_candidates']==len(entries) and len(found)==1 and found[0]['record']==name and c['repair']=='not started' and c['validation']=='not performed' and c['discovery'].startswith('partial')
print('verified six Git sources, extreme source arithmetic and qualified 059 attribution')
