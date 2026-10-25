"""Ignored append-only audit metadata; never runs project code."""
import json
from pathlib import Path

B = Path(__file__).resolve().parents[2]
D = Path(__file__).resolve().parent
Q = json.loads((B / 'camera/owner-fit-delta/optional-rig-supplier-queue-011.json').read_text(encoding='utf-8-sig'))

def append(name, row):
    with (D / (name + '.jsonl')).open('a', encoding='utf-8') as f:
        f.write(json.dumps(row, ensure_ascii=False) + '\n')

def cover(index, comparison, witnesses, fresh=None, findings=None):
    r = next(r for r in Q['sources'] if r['index'] == index)
    s = r['source_scope']
    observations = []
    seen = set()
    for values in [s.get('producer_reached_contract_observations', []), s.get('expanded_producer_contract_observations', []), r.get('producer_role_observations', [])]:
        for o in values:
            identity = (o.get('record'), o.get('receipt'), o.get('json_pointer'))
            if identity not in seen:
                seen.add(identity)
                observations.append({k:o[k] for k in ['record','receipt','json_pointer','source_id_as_recorded','read_ranges_as_recorded','ranges_as_recorded'] if k in o})
    append('coverage', {
        'id': f'RIG-CAMERA-OWNER-{index:03}', 'queue_index':index,
        'source_id':s['source_id'], 'sha256':s['sha256'],
        'queue_ref':f'continuation/camera/owner-fit-delta/optional-rig-supplier-queue-011.json#/sources/{Q["sources"].index(r)}',
        'old_owner_proofs_preserved':s.get('owner_contract_proofs', []),
        'all_producer_observation_refs':observations,
        'actual_semantic_comparison':comparison, 'contract_witnesses':witnesses,
        'fresh_source_read_ranges':fresh or [], 'findings':findings or [],
        'status':'accounted_exact_roles_with_findings_preserved',
        'limits':'Supplier evidence only. Explicit semantic carry only for compared same contracts; no whole-file expansion or other-lens/source/repair acceptance. Prior full actual body proofs are consumed as body evidence, not verdict/hash equivalence.'
    })

def progress(identifier, note):
    rows=[json.loads(l) for l in (D/'coverage.jsonl').read_text(encoding='utf-8').splitlines()]
    done={r['queue_index'] for r in rows}
    append('progress',{'id':identifier,'completed':len(done),'remaining':len(Q['sources'])-len(done),'first_unfinished_index':next((r['index'] for r in Q['sources'] if r['index'] not in done),None),'note':note,'no_overall_lens_clearance':True})
