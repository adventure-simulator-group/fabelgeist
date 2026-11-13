#!/usr/bin/env python3
"""Verify late q0 transport identities and its unassessed queue status."""
from pathlib import Path
import argparse,hashlib,json,subprocess,sys,tarfile
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);root=ap.parse_args().publication.resolve();t=p.parent/'late-transport';subprocess.run([sys.executable,'-B',str(t/'VERIFY.py')],check=True);m=json.loads((t/'MANIFEST.json').read_bytes());bundles={x['label']:x for x in m['bundles']};assert bundles['q0-behavior-v1']['source_manifest_sha256']=='70012be8c55e22ea93af963b63cd3406a0cda87b51320d4630d0cc741e530b08' and bundles['q0-behavior-v2']['source_manifest_sha256']=='d3d4c911d65bcdeff79b48f21c16b2749feb516e4be39cb19fdef0c24edea214'
with tarfile.open(t/'q0-behavior-v2.tar.gz','r:gz') as a:c=json.load(a.extractfile('q0-behavior-v2/CORRECTION-v2.json'))
assert c['predecessor_manifest_sha256']==bundles['q0-behavior-v1']['source_manifest_sha256'];q=json.loads((root/'RECONCILIATION-059.json').read_bytes())['entries'][0];assert q['contract_id']=='ADLER-CONT08-CANDIDATE-001' and q['historical_lens']==8 and q['current_reconciliation']=='unassessed'
print('Verified late q0 transport and correction binding; source assessment remains pending central reconciliation.')
