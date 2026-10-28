"""Maintain ignored audit checkpoints; this does not execute product code."""
from datetime import datetime, timezone
from pathlib import Path
import json
import shutil
import hashlib
import subprocess
import bookkeeping as b


def register_original(index, ranges, name, purpose):
    source = b.read(b.PUBLISHED/'audit05/original-scope-072.json')['files'][index]
    digest = source['sha256']
    manifest = b.read(b.PUBLISHED.parent/'MANIFEST.json')['entries']
    candidates = [b.PUBLISHED/'continuation-segovb/source-objects'/digest]
    candidates += [b.PUBLISHED.parent/e['path'] for e in manifest
                   if e['path'].endswith(digest)]
    path = next((p for p in candidates if p.is_file()), None)
    provenance = 'Transported exact registered object'
    if path is None:
        if source['snapshot'] != 'pr-working':
            raise RuntimeError('Needs explicit frozen non-pr-working locator')
        repository_path = source['source_id'].split(':', 1)[1]
        locator = '6533e444fa37552927070cc8d8190907fd3b413e:' + repository_path
        result = subprocess.run(['git', 'show', locator], capture_output=True)
        if result.returncode or hashlib.sha256(result.stdout).hexdigest() != digest:
            raise RuntimeError('Exact commit/path does not recover registered bytes')
        path = b.OUT/'source-objects'/digest
        path.parent.mkdir(exist_ok=True)
        path.write_bytes(result.stdout)
        provenance = locator
    if b.sha(path) != digest:
        raise RuntimeError('Registered object hash mismatch')
    record = {'lens': 11, 'original_index': index,
              'source_id': source['source_id'], 'sha256': digest,
              'object_path': str(path), 'ranges': ranges,
              'prospective': True, 'purpose': purpose, 'provenance': provenance}
    b.save(b.OUT/name, record)
    return path


def advance(lens_number, count, state_path, receipt_path, coverage, next_action,
            findings=None, extensions=None, partial=False):
    ledger_path = b.ROOT / 'evidence/continuation-segovb/ledger.json'
    current_path = b.ROOT / 'CURRENT-ADLER.json'
    current = b.read(current_path)
    ledger = b.read(ledger_path)
    if b.sha(ledger_path) != current['ledger_sha256']:
        raise RuntimeError('Ledger differs from current checkpoint; reconcile before editing')
    archive = b.ROOT / 'checkpoint-history'
    archive.mkdir(exist_ok=True)
    for path in (ledger_path, current_path):
        preserved = archive / (b.sha(path) + '.json')
        if not preserved.exists():
            shutil.copyfile(path, preserved)
    entry = next(x for x in ledger['lens_coverage'] if x['number'] == lens_number)
    relative_state = Path(state_path).relative_to(b.ROOT)
    relative_receipt = Path(receipt_path).relative_to(b.ROOT)
    ledger_state = '../../' + relative_state.as_posix()
    ledger_receipt = '../../' + relative_receipt.as_posix()
    entry.update(state=ledger_state, current_successor=ledger_state,
                 current_successor_sha256=b.sha(state_path),
                 saved_coverage=coverage, current_coverage=coverage,
                 latest_receipt=ledger_receipt,
                 incomplete_or_qualified_scope=next_action)
    ledger['audit_progress'].append({
        'kind': 'Adler source discovery WITH findings', 'lens': lens_number,
        'original_accounted': count, 'receipt': ledger_receipt,
        'accepted_unique_count_change': 0})
    local = ledger['adler_continuation']
    local.setdefault('discovery_receipts', []).append(ledger_receipt)
    if extensions:
        local.setdefault('prospective_extensions', []).extend(extensions)
    if findings:
        ledger['discovery_findings_and_dependencies'].update(findings)
    now = datetime.now(timezone.utc).isoformat()
    ledger['adler_continuation']['last_checkpoint_at'] = now
    ledger_path.write_text(json.dumps(ledger, indent=2, ensure_ascii=False)+'\n', encoding='utf-8')
    selected = next(x for x in current['frontiers'] if x['lens'] == lens_number)
    selected.update(original_accounted=count, next_original=count,
                    state=relative_state.as_posix(), state_sha256=b.sha(state_path),
                    latest_receipt=relative_receipt.as_posix(), next_action=next_action)
    selected.pop('partial_receipt', None)
    if partial:
        selected['partial_receipt'] = relative_receipt.as_posix()
    current.update(at=now, ledger_sha256=b.sha(ledger_path),
                   previous_checkpoint_archive='checkpoint-history/'+b.sha(current_path)+'.json')
    current_path.write_text(json.dumps(current, indent=2, ensure_ascii=False)+'\n', encoding='utf-8')
    print(json.dumps({'lens': lens_number, 'source_accounted': count,
                      'next_original': count, 'implementation_gate_complete': False,
                      'ledger_sha256': b.sha(ledger_path)}))
