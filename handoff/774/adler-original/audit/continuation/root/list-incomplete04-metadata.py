import json
from pathlib import Path
C = Path(__file__).resolve().parents[1]
a = json.loads((C / 'root/initial04-invariant-operations/scope-authority-001.json').read_text(encoding='utf-8'))
s = json.loads((C.parent / a['scope']['record']).read_text(encoding='utf-8'))
byid = {r['source_id']: r for r in s['files']}
rows = [byid[r['source_id']] for r in a['rows'] if r['status'] != 'exact_scoped_current04_operations_manually_compared']
for r in rows[:65]:
    print(json.dumps({k: v for k, v in r.items() if k in ['source_id','path','actual_line_count','scope_kind','material_ranges','ranges']}))
