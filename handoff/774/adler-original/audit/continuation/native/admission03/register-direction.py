import json,datetime,hashlib
from pathlib import Path
O=Path(__file__).resolve().parent
p=Path('CARGO_HOME/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_math-0.19.1/src/direction.rs');raw=p.read_bytes();sha='0aef18fddadf5d5361c4c11f0b9d5561637a7cee6dabde16dfd2816dcf2e521c'
assert hashlib.sha256(raw).hexdigest()==sha;assert len(raw.decode('utf8').splitlines())==1511
r=dict(id='NATIVE-ADMISSION03-REG-007',at=datetime.datetime.now(datetime.timezone.utc).isoformat(),authority='continuation/root/admission03-discovery-authority-043.json',source_id='native-dependency:bevy_math-0.19.1/src/direction.rs',sha256=sha,snapshot='native-dependency',actual_EOF=1511,material_ranges=[[1,170],[550,660]],role='Exact direct Dir2/Dir3 new/new_unchecked admission and invalid direction errors relied upon by assigned P16 orientation18, preserving prior15–47/550–611 roles.',reason='Actual P16 orientation from_vector/from_normalized/spatial methods directly call these already-pinned native constructors. Register bounded needed method context before reading; no wholeSDK exemption.',exclusions='No unrelated operators/rotation/slerp/foreign tests or whole1511 coverage. Native is_normalized predicate downstream is literal firstparty gate; arithmetic04 independent.',resolved_path=str(p).replace('\\','/'),status='registered_before_direct_native_admission_body_analysis')
with (O/'source-range-registration.jsonl').open('a',encoding='utf8') as f:f.write(json.dumps(r,ensure_ascii=False)+'\n')
print('Registered native direction exact bounded constructors.')
