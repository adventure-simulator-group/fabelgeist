#!/usr/bin/env python3
"""Verify the additive q61 pointer and its unchanged predecessor draft."""
from pathlib import Path
import argparse
import copy
import hashlib
import json
import subprocess
import sys

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--publication', type=Path, required=True)
root = parser.parse_args().publication.resolve()
prior_path = p.parent / 'reconciliation/DRAFT-v4.json'
prior = prior_path.read_bytes()
current = (p / 'DRAFT-v5.json').read_bytes()
sha = lambda data: hashlib.sha256(data).hexdigest()
assert sha(prior) == '08722589f73bfddd7df95e77e0183558b7fb2a9120b1a3bfa19d2e2d1c9854db'
assert sha(current) == '1212f4f29a5dbf2ed6d4faff7788d9df13ed744a69b7c7c1262548d3899d1fef'
c = json.loads((p / 'CORRECTION-v5.json').read_bytes())
assert c['predecessor_sha256'] == sha(prior) and c['successor_sha256'] == sha(current)
assert c['counts_changed'] is False
old = json.loads(prior)
new = json.loads(current)
assert old['status'] == new['status'] == 'draft for independent review; not adopted'
assert new['schema'] == '726-postb111-central-followups-draft-v5'
assert new['predecessor_draft_sha256'] == sha(prior)
expected = copy.deepcopy(old)
expected['schema'] = new['schema']
expected['predecessor_draft_sha256'] = new['predecessor_draft_sha256']
expected['correction_qualification'] = new['correction_qualification']
relation = next(r for r in expected['proposed_ledger_relationships']
                if r['id'] == 'relationship-retained-fit-item-identity-95e')
assert relation['queue'] == [0]
relation['queue'] = [0, 61]
assert expected == new
queue = json.loads(subprocess.check_output(['git', '-C', str(root), 'show',
    new['parent_checkpoint'] + ':RECONCILIATION-059.json']))['entries']
assert queue[61]['historical_lens'] == 11
assert {x.name for x in p.iterdir()} == {'DRAFT-v5.json', 'CORRECTION-v5.json', 'README.md', 'VERIFY.py'}
print('Verified additive q61 navigation and unchanged v4 draft content; '
      'no alias, source credit, closure or product behavior is established.')
