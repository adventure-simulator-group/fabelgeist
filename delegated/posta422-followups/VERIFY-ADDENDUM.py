#!/usr/bin/env python3
"""Verify the a422 evidence package and supplemental custody receipts."""
from pathlib import Path
import argparse,json,subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);a=ap.parse_args();subprocess.run([sys.executable,'-B',str(p/'VERIFY.py'),'--publication',str(a.publication)],check=True);subprocess.run([sys.executable,'-B',str(p/'supplemental-encrypted/VERIFY.py')],check=True);subprocess.run([sys.executable,'-B',str(p/'proposals/VERIFY-STATUS-CORRECTION.py')],check=True)
r=json.loads((p/'SUPPLEMENTAL-RESTORATION.json').read_bytes());assert r['restore_exit']==0 and r['collections']==37 and r['matched_file_bytes_and_modes']==1132 and r['matched_directory_modes']==167 and r['selected_replay_exits']==[0]*9
print('Verified supplemental custody identities and same-machine replay receipt; no semantic closure or product validation is implied.')
