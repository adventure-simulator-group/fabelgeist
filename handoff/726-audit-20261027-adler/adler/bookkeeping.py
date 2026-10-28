"""Evidence bookkeeping only. Never runs or changes project source."""
from pathlib import Path
import hashlib
import json
from datetime import datetime, timezone

ROOT = Path('target/726-audit-adler-20261027')
PUBLISHED = Path('handoff/726-audit-20261027-adler/evidence')
OUT = ROOT / 'adler'
LENS = PUBLISHED / 'continuation-segovb/audit-resume-20261027/lens11'

def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def save(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        raise RuntimeError(f'Append-only receipt already exists: {path}')
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False)+'\n', encoding='utf-8')

def resolve(value):
    prefix = 'target/recovery-20261025/'
    if value.startswith(prefix):
        return PUBLISHED / value[len(prefix):]
    return LENS / value

def verify_refs(value, result):
    if isinstance(value, dict):
        if 'sha256' in value and ('path' in value or 'object_path' in value):
            path = resolve(value.get('object_path', value.get('path')))
            actual = sha(path)
            if actual != value['sha256']:
                raise RuntimeError(f'Hash mismatch: {path}')
            result.append({'path': str(path), 'sha256': actual})
        for child in value.values():
            verify_refs(child, result)
    elif isinstance(value, list):
        for child in value:
            verify_refs(child, result)

def adopt11():
    checkpoint = read(LENS/'HANDOFF-paused-20261027-001.json')
    registry = read(PUBLISHED/'audit05/original-scope-072.json')['files']
    packets = []
    next_index = 1195
    for entry in checkpoint['complete_pending_adoption']:
        refs = []
        verify_refs(entry, refs)
        number = entry['batch']
        receipt = read(LENS/f'batch-{number:03}-receipt.json')
        state = read(LENS/f'STATE-proposal-{number:03}.json')
        verify_refs(receipt, refs)
        verify_refs(state.get('prior_proposal', {}), refs)
        for source in receipt['originals']:
            index = source['original_index']
            if index != next_index:
                raise RuntimeError(f'Non-contiguous original {index}, expected {next_index}')
            registered = registry[index]
            if registered['source_id'] != source['source_id'] or registered['sha256'] != source['sha256']:
                raise RuntimeError(f'Original identity mismatch: {index}')
            consumed = list(source.get('actual_read_ranges', []))
            for reused in source.get('personal_same_lens_reuse', []):
                consumed.extend(reused['ranges'])
            for first, last in source['ranges']:
                position = first
                for start, end in sorted(consumed):
                    if start <= position <= end:
                        position = end + 1
                if position <= last:
                    raise RuntimeError(f'Unaccounted registered range: {index}:{position}-{last}')
            next_index += 1
        if state['original_accounted'] != next_index or state['next_original'] != next_index:
            raise RuntimeError('Proposal frontier does not match receipts')
        packets.append({'batch': number, 'receipt': entry['receipt'], 'state': entry['state'],
                        'verified_references': refs, 'next_original': next_index})
    adoption = {
        'at': datetime.now(timezone.utc).isoformat(), 'lens': 11,
        'kind': 'Root adoption of inherited same-lens evidence; not a new personal body read',
        'packets': packets, 'prior_original_accounted': 1195,
        'original_accounted': next_index, 'next_original': next_index,
        'basis': [
            'Reviewed all six semantic inventories, receiving traces, reuse records and limits.',
            'Exact receipt, source-object and prior-state references verified; ordered original identities match the immutable registry.',
            'Adopted source discovery WITH findings only. Historical author body reads remain attributed to their receipts.',
            'Gateway-sufficient authorization rejects missing-check suspicion. Directed affinity, unordered familiarity and separate personal/shared clocks remain distinct.',
            'Party approval binding and six-rating create/update mismatch remain frozen qualified source findings, not current-runtime or accepted-unique claims.',
            'Roster-count narrowing suspicion rejected only for inspected capped path; ammunition availability and report-overwrite concerns remain API questions.',
        ],
        'qualification_authorities': {
            'state': str(LENS/'STATE-proposal-107.json'), 'sha256': sha(LENS/'STATE-proposal-107.json'),
            'checkpoint': str(LENS/'HANDOFF-paused-20261027-001.json'),
            'checkpoint_sha256': sha(LENS/'HANDOFF-paused-20261027-001.json'),
        },
        'partial108': 'Original1207 lines1..2160 previously read,2161..2815 pending;1208 full1..29 unread. No ordered credit.',
        'accepted_unique_count_change': 0, 'implementation_gate_complete': False,
        'current_reconciled': False, 'validated': False,
        'remaining': 'All inherited overlays, merged, shared, prospective extensions, current source, twenty families, aliases and connected acceptance remain separate and incomplete.',
    }
    save(OUT/'ADOPTION-070-lens11.json', adoption)
    print(json.dumps({'adopted_packets': [p['batch'] for p in packets], 'lens':11, 'next_original':next_index}))

if __name__ == '__main__':
    adopt11()
