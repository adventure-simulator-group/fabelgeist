import json
from collections import Counter
from pathlib import Path

C = Path(__file__).resolve().parents[1]
A = C / 'root/initial04-invariant-operations'
authority = json.loads((A / 'scope-authority-001.json').read_text(encoding='utf-8'))
print(json.dumps({'authority_keys': list(authority), 'statuses': Counter(r['status'] for r in authority['rows'])}))
print('next_uncompared', json.dumps([r for r in authority['rows'] if r['status'] != 'exact_scoped_current04_operations_manually_compared'][:12]))
for name in ['coverage.jsonl', 'findings.jsonl']:
    p = A / name
    print(name, p.exists())
    if p.exists():
        rows = [json.loads(line) for line in p.read_text(encoding='utf-8').splitlines()]
        print('row_count', len(rows), 'last', json.dumps(rows[-1]))
print('latest_packets', [p.name for p in sorted(A.glob('contract-groups-*.json'))[-5:]])
