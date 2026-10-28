"""Restore handoff evidence into a new ignored continuation directory only."""
from pathlib import Path
import argparse
import hashlib
import json
import shutil


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify-only', action='store_true')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    repo = root.parent.parent
    manifest = json.loads((root / 'MANIFEST.json').read_text(encoding='utf-8'))
    for entry in manifest['entries']:
        path = (root / entry['path']).resolve()
        if not path.is_relative_to(root) or not path.is_file():
            raise RuntimeError('Invalid or missing manifest entry: ' + entry['path'])
        data = path.read_bytes()
        if len(data) != entry['bytes'] or hashlib.sha256(data).hexdigest() != entry['sha256']:
            raise RuntimeError('Transport hash mismatch: ' + entry['path'])
    print('Verified handoff payload:', len(manifest['entries']), 'files')
    if args.verify_only:
        return
    dest = repo / 'target' / '726-audit-adler-20261027'
    if dest.exists():
        raise RuntimeError('Destination already exists; preserve it and reconcile manually: ' + str(dest))
    dest.mkdir(parents=True)
    for name in ('evidence', 'adler', 'checkpoint-history'):
        shutil.copytree(root / name, dest / name)
    for name in ('CURRENT-ADLER.json', 'EVIDENCE-VERIFICATION-ADLER.json', 'RESUME-ADLER-001.json'):
        shutil.copyfile(root / name, dest / name)
    shutil.copyfile(root / 'CURRENT.json', dest / 'HANDOFF-SEGOVB-CURRENT.json')
    print('Restored evidence only:', dest)
    print('Read CURRENT-ADLER.json and the handoff AUDIT-RESUME.md before continuing.')


if __name__ == '__main__':
    main()
