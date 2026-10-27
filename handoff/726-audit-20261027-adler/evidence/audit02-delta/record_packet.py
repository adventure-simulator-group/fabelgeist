"""Append-only ignored own02 evidence bookkeeping, no project execution."""
import json
from pathlib import Path
import sys

root = Path(__file__).parent
number, low, high = map(int, sys.argv[1:4])
stem = f'{number:03}'
dispositions = json.loads((root / f'dispositions-{stem}.json').read_text())
sources = json.loads((root / 'resolved-source-map.json').read_text())
selected = [row for row in sources if low <= row['index'] <= high]
coverage = root / 'coverage.jsonl'
prior = [json.loads(line) for line in coverage.read_text().splitlines()] if coverage.exists() else []
done = {row['index'] for row in prior}
assert not done.intersection(range(low, high + 1))
assert len(selected) == high - low + 1
packet = dict(id=f'ADLER-OWN02-MATERIAL-{stem}', lens='02 existing type owners',
              sources=[dict(row, disposition=dispositions[str(row['index'])],
                            method='FRESH_PERSONAL_EXACT_REGISTERED_RANGE_OWNER_COMPARISON')
                       for row in selected], new_unique_findings_claimed=0,
              execution='Read-only source evidence; no project checks, source edits or Git changes')
notes = root / f'notes-{stem}.json'
if notes.exists():
    packet.update(json.loads(notes.read_text()))
path = root / f'contract-groups-{stem}.json'
assert not path.exists()
path.write_text(json.dumps(packet, indent=2) + '\n')
with coverage.open('a') as stream:
    for row in selected:
        stream.write(json.dumps(dict(row, packet=path.name,
                                    disposition=dispositions[str(row['index'])],
                                    method='FRESH_PERSONAL_EXACT_REGISTERED_RANGE_OWNER_COMPARISON')) + '\n')
done.update(range(low, high + 1))
remaining = [index for index in range(62, 175) if index not in done]
progress = dict(lens='02', preserved_baseline='inherited-frontier.json',
                completed_material_indices=sorted(done), completed_delta_count=len(done),
                total_delta_count=113, first_unfinished_material=remaining[0] if remaining else None,
                initial_delta_complete=not remaining, final_clearance=False,
                latest_packet=path.name)
(root / f'progress-{stem}.json').write_text(json.dumps(progress, indent=2) + '\n')
print('Saved', len(selected), 'fresh02 entries; completed', len(done), 'of113; next', progress['first_unfinished_material'])
