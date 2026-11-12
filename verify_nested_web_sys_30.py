#!/usr/bin/env python3
"""Verify archived lock and web-sys identities, without feature resolution."""
import hashlib
import io
import json
import tarfile
import tomllib
from pathlib import Path
R = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
cp = json.loads((R / 'DELEGATED-EVIDENCE-CHECKPOINT-30.json').read_text())
b = next(x for x in cp['bundles'] if x['bundle_id'] == 'nested-locks-680-2313')
with tarfile.open(R / b['archive']) as t:
    reg = json.load(t.extractfile('native-registration.json'))
    packages = {}
    for p in reg['packages']:
        raw = t.extractfile(p['archive']).read()
        assert (sha(raw), len(raw)) == (p['sha256'], p['bytes'])
        packages[p['version']] = raw
    assert set(packages) == {'0.3.103', '0.3.105'}
    for path, version in [('source/root-current.lock', '0.3.103'), ('source/680-historical.lock', '0.3.105'), ('source/680-current.lock', '0.3.105'), ('source/2313-historical.lock', '0.3.105'), ('source/2313-current.lock', '0.3.105')]:
        lock = tomllib.loads(t.extractfile(path).read().decode())
        rows = [p for p in lock['package'] if p['name'] == 'web-sys' and p['version'] == version]
        assert len(rows) == 1 and rows[0]['checksum'] == sha(packages[version])
    canvases = []
    for version, package in packages.items():
        with tarfile.open(fileobj=io.BytesIO(package), mode='r:gz') as archive:
            path = f'web-sys-{version}/src/features/gen_OffscreenCanvas.rs'
            canvas = archive.extractfile(path).read()
            saved = t.extractfile(f'source/web-sys-{version}-canvas.rs').read()
            assert canvas == saved and sha(canvas) == '254a020b8c525a480fcc0bab0a517eb333bd273075298553559b548ef8b83c50'
            canvases.append(canvas)
    assert canvases[0] == canvases[1]
print('Verified two lock-pinned web-sys archives and byte-identical selected members; no feature selection, browser behavior or source-read transfer inferred.')
