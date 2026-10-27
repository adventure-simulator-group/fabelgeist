"""Ignored frozen-source provenance bookkeeping only, no SDK/project execution."""
import hashlib
import json
from pathlib import Path
import subprocess

r = Path(__file__).parent
c = json.loads((r/'catalog.json').read_text())['rows']
s = json.loads((r.parent/'audit05/original-scope-072.json').read_text(encoding='utf-8-sig'))['files']
receipts = []
details = {
    2476: ([[100,180]], 'Personally read SeedableNoise/ScalableNoise/Sampleable reached declarations and reciprocal period/frequency default methods. No handwritten structural trait implementation or codec in reached scope.', 'No derivability candidate. Trait API/default numerical behavior is native owner contract, not boilerplate; no whole SDK, unusual-input/numeric/runtime acceptance.'),
    2477: ([[1,195]], 'Personally read exact timestamp1-195 from audit11 source object only (no verdict), independently hash-verified; immutable recovered Git object verified identical. SATS structural traits already derive; Add<TimeDuration> calls checked_add then unwrap, and Add<Duration> declaration starts at195. Native timestamp/system/chrono and checked arithmetic methods read.', 'No derivability candidate. Preserve native checked timestamp/duration semantic operation and SATS codec; no tail195+ Add body, Display/other impls outside range, SDK arithmetic correctness or runtime acceptance.')
}
for i,(ranges,inventory,disposition) in details.items():
    h = c[i]['expected_sha256']
    ref = 'origin/codex/774-audit-handoff-20261025:handoff/774/audit/reconcile-adler-20261023/recovered-sources/objects/' + h
    d = subprocess.check_output(['git','show',ref])
    assert hashlib.sha256(d).hexdigest() == h
    assert len(d.decode().splitlines()) == c[i]['line_count']
    (r/'successor-source-objects'/h).write_bytes(d)
    receipts.append(dict(scope_index=i,source_id=c[i]['source_id'],sha256=h,ranges=ranges,
                         scope_kind=s[i]['scope_kind'],scope_limits=s[i]['scope_limits'],
                         source_basis='Fresh personal exact registered SDK ranges; recovered Git hash plus native source catalogue identity, no other-lens verdict credit.',
                         immutable_ref=ref,trait_inventory_and_semantics=inventory,
                         disposition=disposition,new_finding=False,checks='not run'))
packet = r/'successor-280.json'
assert not packet.exists()
packet.write_text(json.dumps(dict(id='ADLER-CONT10-SUCCESSOR-280',lens='derivability',receipts=receipts,clear=False),indent=2)+'\n')
state=json.loads((r/'STATE.json').read_text())
prior=set(state['new_original_indices'])
assert not prior.intersection(details)
prior.update(details)
inv=json.loads((r/'inherited-inventory.json').read_text())['rows']
done=prior|{x['scope_index'] for x in inv if x['status']!='PENDING_OWN_LENS10_SOURCE_ANALYSIS'}
state.update(new_original_indices=sorted(prior),original_accounted=len(done),next_original=next(i for i in range(state['original_total']) if i not in done),latest_packet=packet.name)
(r/'STATE.json').write_text(json.dumps(state,indent=2)+'\n')
print('Saved SDK roles; accounted',len(done),'next',state['next_original'])
