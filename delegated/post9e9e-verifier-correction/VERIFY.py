#!/usr/bin/env python3
"""Check the two separately preserved verifier-failure replays."""
import argparse
import hashlib
import json
import sys
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
P = Path(__file__).resolve().parent
H = lambda data: hashlib.sha256(data).hexdigest()
parser = argparse.ArgumentParser()
parser.add_argument('--key', type=Path, help='separately held 32-byte AES key')
args = parser.parse_args()
cases = [
    ('RECORD.json', 'initial-verifier.py.txt', 'initial_verifier_sha256',
     'failure-pathsafe.log', 'failure-original.aesgcm'),
    ('SECOND-RECORD.json', 'second-verifier.py.txt', 'verifier_sha256',
     'second-failure-pathsafe.log', 'second-failure-original.aesgcm'),
]
for record_name, script_name, script_key, log_name, cipher_name in cases:
    record = json.loads((P / record_name).read_bytes())
    script = (P / script_name).read_bytes()
    safe_log = (P / log_name).read_bytes()
    blob = (P / cipher_name).read_bytes()
    assert H(script) == record[script_key]
    assert H(safe_log) == record['pathsafe_log_sha256']
    assert (len(blob), H(blob)) == (record['ciphertext_bytes'], record['ciphertext_sha256'])
    assert record['exit'] != 0 and record['associated_data'].startswith('726-audit-private-evidence:')
if not args.key:
    print('Verified path-safe replay and ciphertext identities; NOT REPEATED: authentication of exact raw logs requires the separately held key.')
    raise SystemExit(0)
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
key = args.key.read_bytes()
assert len(key) == 32
for record_name, _, _, _, cipher_name in cases:
    record = json.loads((P / record_name).read_bytes())
    blob = (P / cipher_name).read_bytes()
    plain = AESGCM(key).decrypt(blob[:12], blob[12:], record['associated_data'].encode())
    assert (len(plain), H(plain)) == (record['replayed_raw_log_bytes'], record['replayed_raw_log_sha256'])
print('Authenticated both exact raw replay logs in memory; no files were restored.')
print('Same-machine key custody and independent recovery remain separately qualified.')
