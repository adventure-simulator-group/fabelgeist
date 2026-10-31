"""Verify or restore exact audit evidence into a fresh ignored target directory.

Transport only. No product checkout, helper execution, or audit read credit.
Requires Python 3.9+ and Git. Inspect this file before use.
"""
import argparse
from collections import defaultdict
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tempfile


def sha256(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def safe_relative(value):
    p = PurePosixPath(value)
    if (not value or p.is_absolute() or '..' in p.parts or '\\' in value
            or ':' in value or p.as_posix() != value):
        raise RuntimeError('Invalid transport path: ' + value)
    return p


def verify_file(root, entry):
    rel = safe_relative(entry['path'])
    path = root.joinpath(*rel.parts)
    if path.is_symlink() or not path.is_file():
        raise RuntimeError('Missing/unsafe transport file: ' + entry['path'])
    if (not path.resolve().is_relative_to(root.resolve())
            or path.stat().st_size != entry['bytes']
            or sha256(path) != entry['sha256']):
        raise RuntimeError('Transport bytes mismatch: ' + entry['path'])
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify-only', action='store_true')
    parser.add_argument('--destination', default='target/726-audit-adler-20261027')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    repo = Path.cwd().resolve()
    if subprocess.check_output(['git', 'rev-parse', '--show-toplevel'],
                               text=True).strip() != str(repo):
        raise RuntimeError('Run from the intended repository root')
    direct = json.loads((root / 'PUBLICATION-MANIFEST.json').read_text())
    for entry in direct['entries']:
        verify_file(root, entry)
    manifest = json.loads((root / 'MANIFEST.json').read_text())
    groups = defaultdict(list)
    names = set()
    for entry in manifest['entries']:
        safe_relative(entry['path'])
        if entry['path'] in names:
            raise RuntimeError('Duplicate payload path')
        names.add(entry['path'])
        if (not re.fullmatch('[0-9a-f]{40}', entry['git_blob'])
                or not re.fullmatch('[0-9a-f]{64}', entry['sha256'])
                or not isinstance(entry['bytes'], int) or entry['bytes'] < 0):
            raise RuntimeError('Invalid payload identity')
        groups[entry['git_blob']].append(entry)
    if len(names) != manifest['file_count']:
        raise RuntimeError('Payload count mismatch')
    target = repo / 'target'
    if target.is_symlink() or target.resolve() != target:
        raise RuntimeError('Repository target/ must not be a symlink')
    target.mkdir(exist_ok=True)
    requested = repo / args.destination
    if not args.verify_only and (requested.exists() or requested.is_symlink()):
        raise RuntimeError('Refusing existing continuation: ' + str(requested))
    dest = requested.resolve()
    if not dest.is_relative_to(target.resolve()) or dest == target.resolve():
        raise RuntimeError('Destination must be inside this repository target/')
    ignored = subprocess.run(['git', 'check-ignore', '--quiet', '--',
                              str(dest.relative_to(repo))]).returncode == 0
    if not ignored:
        raise RuntimeError('Destination must be ignored by Git')
    if not args.verify_only and dest.exists():
        raise RuntimeError('Refusing existing continuation: ' + str(dest))
    parts = [verify_file(root, entry) for entry in manifest['bundle_parts']]
    # Temporary bare Git store is ignored; no existing Git database is changed.
    with tempfile.TemporaryDirectory(prefix='726-transport-', dir=target) as tmp:
        tmp = Path(tmp)
        bundle = tmp / 'payload.bundle'
        with bundle.open('xb') as output:
            for part in parts:
                with part.open('rb') as source:
                    shutil.copyfileobj(source, output, 1024 * 1024)
        if sha256(bundle) != manifest['bundle_sha256']:
            raise RuntimeError('Reassembled bundle hash mismatch')
        store = tmp / 'payload.git'
        subprocess.run(['git', '-c', 'core.hooksPath=/dev/null', 'clone',
                        '--bare', '--quiet', '--no-hardlinks', str(bundle),
                        str(store)], check=True)
        commit = subprocess.check_output(['git', '--git-dir=' + str(store),
                  'rev-parse', 'refs/heads/evidence-only'], text=True).strip()
        if commit != manifest['payload_commit']:
            raise RuntimeError('Unexpected evidence transport commit')
        batch = subprocess.Popen(['git', '--git-dir=' + str(store),
                                  'cat-file', '--batch'], stdin=subprocess.PIPE,
                                 stdout=subprocess.PIPE)
        if not args.verify_only:
            # Failures leave a named partial directory for investigation.
            # It must never be treated as a successful continuation.
            dest.mkdir(parents=True, exist_ok=False)
            (dest / 'RESTORATION-INCOMPLETE').write_text(
                'Do not continue: lossless restoration has not completed.\n')
        verified = 0
        try:
            for blob, entries in groups.items():
                entry = entries[0]
                if any((e['bytes'], e['sha256']) !=
                       (entry['bytes'], entry['sha256']) for e in entries):
                    raise RuntimeError('Inconsistent shared Git blob identity')
                batch.stdin.write((blob + '\n').encode())
                batch.stdin.flush()
                header = batch.stdout.readline().decode().strip().split()
                if header != [blob, 'blob', str(entry['bytes'])]:
                    raise RuntimeError('Git blob header mismatch')
                data = batch.stdout.read(entry['bytes'])
                if (len(data) != entry['bytes'] or batch.stdout.read(1) != b'\n'
                        or hashlib.sha256(data).hexdigest() != entry['sha256']):
                    raise RuntimeError('Payload hash mismatch: ' + entry['path'])
                if not args.verify_only:
                    for e in entries:
                        p = dest.joinpath(*safe_relative(e['path']).parts)
                        p.parent.mkdir(parents=True, exist_ok=True)
                        # Independent copies: editing a live ledger must not
                        # mutate a historical receipt through a hard link.
                        with p.open('xb') as output:
                            output.write(data)
                verified += len(entries)
                if verified % 2000 < len(entries):
                    print('Verified payload files:', verified, flush=True)
        finally:
            batch.stdin.close()
            batch.stdout.close()
            code = batch.wait()
        if code != 0 or verified != manifest['file_count']:
            raise RuntimeError('Incomplete payload verification')
    if not args.verify_only:
        (dest / 'RESTORATION-INCOMPLETE').unlink()
        print('Restored exact evidence:', dest)
        if args.destination != 'target/726-audit-adler-20261027':
            print('Alternate destination is for preservation/review. Historical')
            print('helpers retain their fixed paths and require separate review.')
    print('Verified', verified, 'logical files; body/read credit: zero')
    print('Read central CURRENT.json, AUDIT-RESUME.md, and SOURCE-LOCATORS.json.')


if __name__ == '__main__':
    main()
