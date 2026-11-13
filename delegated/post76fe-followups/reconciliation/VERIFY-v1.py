#!/usr/bin/env python3
"""Verify report members and selected historic evidence without product code."""
from pathlib import Path
import argparse,hashlib,io,json,subprocess,tarfile
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);args=ap.parse_args();root=args.publication.resolve();sha=lambda b:hashlib.sha256(b).hexdigest();git=lambda spec:subprocess.check_output(['git','-C',str(root),'show',spec]);parent='76fe53635e2bf1e3e08bbfaa364314f5e7888bf8';mapping=json.loads((p/'SOURCE-MAP.json').read_bytes());history=json.loads((p/'HISTORY-SELECTION.json').read_bytes());assert mapping['parent_checkpoint']==history['parent_checkpoint']==parent
assert len(mapping['reports'])==17 and len({r['label'] for r in mapping['reports']})==17
for row in mapping['reports']:
 path=row['archive'];prefix='delegated/post76fe-followups/'
 if path.startswith(prefix):
  rel=Path(path[len(prefix):]);assert not rel.is_absolute() and '..' not in rel.parts;file=p.parent/rel;assert file.is_file() and not file.is_symlink() and file.resolve().is_relative_to(p.parent);raw=file.read_bytes()
 else:raw=git(parent+':'+path)
 assert sha(raw)==row['archive_sha256']
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as tar:
  members=[m for m in tar if m.name==row['member']];assert len(members)==1 and members[0].isfile();data=tar.extractfile(members[0]).read()
 assert (len(data),sha(data))==(row['member_bytes'],row['member_sha256'])
assert len(mapping['prior_original_transports'])==6
for r in mapping['prior_original_transports']:git(parent+':'+r['archive'])
original=git(history['original_059_commit']+':handoff/726-audit-20261030-adler/workflow-20261030/CLOSURE-QUEUE-100.json');reconstructed=git(parent+':baseline-059/queue.json');wrappers=json.loads(git(parent+':RECONCILIATION-059.json'))['entries'];assert sha(original)==history['original_queue_sha256'] and sha(reconstructed)==history['reconstructed_queue_sha256'] and original!=reconstructed and not history['whole_stream_bytes_equal'];a=json.loads(original)['entries'];b=json.loads(reconstructed)['entries'];assert len(a)==len(b)==len(wrappers)==3713 and len(history['rows'])==21
for row in history['rows']:
 i=row['index'];assert a[i]==b[i] and row['original_and_reconstructed_equal_as_parsed'];status=row['original_status'];assert status['present']==('status' in a[i]);assert not status['present'] or status['value']==a[i]['status'];w=wrappers[i];assert (row['contract_id'],row['historical_lens'],row['wrapper_historical_status'],row['wrapper_current_reconciliation'])==(w['contract_id'],w['historical_lens'],w['historical_status'],w['current_reconciliation']) and w['current_reconciliation']=='unassessed'
print('Verified 17 public report members, six prior transports and 21 selected original/reconstructed history relationships. Historical Git objects and separately held restricted evidence remain dependencies; source semantics, independent custody, checkpoint3821 recovery and product behavior are not established.')
