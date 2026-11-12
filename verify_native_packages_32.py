#!/usr/bin/env python3
"""Verify selected locked native archive and source identities only."""
import hashlib
import io
import json
import tarfile
import tomllib
from pathlib import Path
R=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
cp=json.loads((R/'DELEGATED-EVIDENCE-CHECKPOINT-32.json').read_text());bs={x['bundle_id']:x for x in cp['bundles']}
with tarfile.open(R/bs['reflect-admission-2563-2564-reconcile']['archive']) as t:
    reg=json.load(t.extractfile('registration.json'));raw=t.extractfile(reg['archive']['path']).read();assert sha(raw)==reg['archive']['sha256']
    for lock in ('sources/6533-Cargo.lock','sources/95e-Cargo.lock'):
        d=tomllib.loads(t.extractfile(lock).read().decode());p=next(x for x in d['package'] if (x['name'],x['version'])==('bevy_reflect_derive','0.19.1'));assert p['checksum']==sha(raw)
    with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as pkg:
        for entry in reg['entries']:
            if entry.get('kind')=='native package role':
                original=pkg.extractfile(entry['member']).read();assert original==t.extractfile(entry['archive']).read() and (sha(original),len(original))==(entry['sha256'],entry['bytes'])
with tarfile.open(R/bs['native-p16-2577-2580-reconcile']['archive']) as t:
    m=next(x for x in bs['native-p16-2577-2580-reconcile']['members'] if x['path']=='native/serde-saphyr-1.2.0.crate');assert m['changed'] and 'Reconstructed public copy' in m['qualification']
    lock=tomllib.loads(t.extractfile('support/current-lock.source').read().decode());p=next(x for x in lock['package'] if (x['name'],x['version'])==('serde-saphyr','1.2.0'));assert p['checksum']==m['original_sha256']
    with tarfile.open(fileobj=io.BytesIO(t.extractfile(m['path']).read()),mode='r:gz') as pkg:
        body=pkg.extractfile('serde-saphyr-1.2.0/src/de/with_deserializer.rs').read();assert body==t.extractfile('source/historical-2577.rs').read() and sha(body)=='c6304415acbe2cb08fe5a730da8625f2c4631d111b7dccde6dad44c2d5e34c95'
print('Verified selected native members against lock-pinned archive identities; saphyr public archive is reconstructed, with no feature resolution or execution.')
