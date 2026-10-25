import json,hashlib,datetime
from pathlib import Path
O=Path(__file__).resolve().parent
p=Path('SNAPSHOTS/pr-working/crates/adventuresim-world-schema/src/calendar.rs')
raw=p.read_bytes();sha='64e57270d86d0c5de29f25c63deb77421e1c0be5eb6ccb4b2e20f805ae9777bc'
assert hashlib.sha256(raw).hexdigest()==sha
assert len(raw.decode('utf8').splitlines())==698
r=dict(id='NATIVE-ADMISSION03-REG-006',at=datetime.datetime.now(datetime.timezone.utc).isoformat(),authority='continuation/root/admission03-discovery-authority-043.json',source_id='pr-working:crates/adventuresim-world-schema/src/calendar.rs',sha256=sha,snapshot='pr-working',actual_EOF=698,material_ranges=[[1,410]],role='Exact canonical StrategicMinute constructors/backing/Serde/default/conversions and directly used day-of-year/day fraction/period/day-of-epoch methods for actual assigned celestial84/sky279–280 and cloud276 receivers.',reason='Root confirms no current03 canonical witness. Already globally scoped source; register bounded definition context before analysis, do not infer signedness/bounds from name.',exclusions='No unrelated clock authority/database/update graph, no coverage for remaining source411–698, no producer/owner/later-lens credit.',resolved_path=str(p).replace('\\','/'),status='registered_before_current03_body_analysis')
with (O/'source-range-registration.jsonl').open('a',encoding='utf8') as f:f.write(json.dumps(r,ensure_ascii=False)+'\n')
print('Registered calendar supporting definition1–410 before analysis.')
