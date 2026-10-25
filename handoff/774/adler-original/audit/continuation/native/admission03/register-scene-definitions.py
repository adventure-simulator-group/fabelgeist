import json, hashlib
from pathlib import Path
from datetime import datetime, timezone
B=Path('audit')
O=B/'continuation/native/admission03'
specs=[(4,'crates/adventuresim-tactical-core/src/scene_coordinates.rs','1aae0147a71b5bf6e93b486a289dc31681517aef7b8638f8307bc221a3accab1',122,[[1,122]]),(5,'crates/adventuresim-tactical-core/src/city_layout/grounding.rs','8d0975e7d71d04bd596bc3ceb324e894b59c7b9c68eff57d6b59420f47029f9c',407,[[1,120]])]
rows=[]
for n,p,sha,eof,ranges in specs:
    path=Path('SNAPSHOTS/pr-working')/p
    raw=path.read_bytes()
    assert hashlib.sha256(raw).hexdigest()==sha
    assert len(raw.decode('utf8').splitlines())==eof
    rows.append(dict(id=f'NATIVE-ADMISSION03-REG-{n:03}',at=datetime.now(timezone.utc).isoformat(),authority='continuation/root/admission03-discovery-authority-043.json',source_id='pr-working:'+p,sha256=sha,snapshot='pr-working',actual_EOF=eof,material_ranges=ranges,role='Direct canonical ScenePlanPoint/SupportElevation definition: constructor, visibility, raw backing and Serde/reflection admission routes used by exact assigned IK terrain helpers.',reason='Root confirms no current03 canonical wrapper witness. Existing global source; registration precedes fresh bounded body analysis. Generic geometry proof does not automatically establish wrapper admission.',exclusions='No unrelated terrain/building operations, no producer/owner/later-lens credit; supporting definitions are not assigned443 coverage.',resolved_path=str(path).replace('\\','/'),status='registered_before_new_admission_body_analysis'))
with (O/'source-range-registration.jsonl').open('a',encoding='utf8') as f:
    for row in rows: f.write(json.dumps(row,ensure_ascii=False)+'\n')
print('Registered two exact supporting definitions before body analysis.')
