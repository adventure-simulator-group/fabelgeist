#!/usr/bin/env python3
"""Verify independent FBX review identities; never execute product source."""
import argparse, datetime, gzip, hashlib, io, json, subprocess, tarfile
from pathlib import Path, PurePosixPath

root=Path(__file__).resolve().parent
parser=argparse.ArgumentParser()
parser.add_argument('--publication',type=Path,required=True)
parser.add_argument('--git-repo',type=Path)
args=parser.parse_args()
publication=args.publication.resolve()
sha=lambda b:hashlib.sha256(b).hexdigest()
oid=lambda kind,b:hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
load=lambda n:json.loads((root/n).read_bytes())
reg=load('REGISTRATION.json');ext=load('EXT-REGISTRATION.json');proof=load('TREE-PROOFS.json');receipt=load('READ-RECEIPT.json');review=load('REVIEW.json')
assert sha((root/'REGISTRATION.json').read_bytes())==receipt['registration_sha256']
assert sha((root/'EXT-REGISTRATION.json').read_bytes())==receipt['extension_registration_sha256']
assert datetime.datetime.fromisoformat(reg['registered_utc'])<datetime.datetime.fromisoformat(receipt['recorded_utc'])
assert datetime.datetime.fromisoformat(ext['registered_utc'])<datetime.datetime.fromisoformat(receipt['recorded_utc'])
for name,info in receipt['displays'].items():
 path=PurePosixPath(name);assert not path.is_absolute() and '..' not in path.parts
 raw=(root/path).read_bytes();assert (len(raw),sha(raw))==(info['bytes'],info['sha256'])
draft=(root/'REVIEWED-COORDINATOR-DRAFT.json').read_bytes()
assert sha(draft)==receipt['reviewed_coordinator_draft']['sha256']==review['reviewed_coordinator_draft_sha256']
assert len(draft)==receipt['reviewed_coordinator_draft']['bytes']

commits={}
for rev,info in proof['commits'].items():
 if 'published_archive_reference' in info:
  ref=info['published_archive_reference'];path=PurePosixPath(ref['archive']);assert not path.is_absolute() and '..' not in path.parts
  archived=(publication/path).read_bytes();assert sha(archived)==ref['archive_sha256']
  with tarfile.open(fileobj=io.BytesIO(archived),mode='r:gz') as tar:
   member=tar.extractfile(ref['member']);assert member is not None;packed=member.read()
  assert sha(packed)==ref['member_gzip_sha256']
 else:
  packed=(root/info['path']).read_bytes();assert sha(packed)==info['gzip_sha256']
 raw=gzip.decompress(packed);assert oid('commit',raw)==rev and sha(raw)==info['raw_sha256'] and len(raw)==info['raw_bytes']
 assert raw.splitlines()[0]==b'tree '+info['root_tree'].encode();commits[rev]=info['root_tree']
trees={}
for h,info in proof['tree_objects'].items():
 packed=(root/info['path']).read_bytes();assert sha(packed)==info['gzip_sha256']
 raw=gzip.decompress(packed);assert oid('tree',raw)==h and sha(raw)==info['raw_sha256'] and len(raw)==info['raw_bytes']
 pos=0;nodes={}
 while pos<len(raw):
  end=raw.index(b'\0',pos);mode,name=raw[pos:end].split(b' ',1);child=raw[end+1:end+21].hex();pos=end+21
  assert name.decode() not in nodes;nodes[name.decode()]=(mode.decode(),child)
 assert pos==len(raw);trees[h]=nodes
bytag={entry['tag']:entry for entry in reg['entries']}
for row in proof['proofs']:
 entry=bytag[row['tag']];current=commits[entry['revision']]
 assert [x['name'] for x in row['chain']]==entry['path'].split('/')
 for part in row['chain']:
  assert part['tree']==current and trees[current][part['name']]==(part['mode'],part['child']);current=part['child']
 assert current==entry['git_blob']==row['git_blob']
 if entry['scope_kind']=='source':
  body=(root/entry['capture']).read_bytes();assert (len(body),sha(body),oid('blob',body),len(body.splitlines()))==(entry['bytes'],entry['sha256'],entry['git_blob'],entry['eof'])
reader=(root/'sources/reader.rs').read_text().splitlines();encoding=(root/'sources/encoding.rs').read_text().splitlines()
a=''.join(f'reader:{n:03}: {reader[n-1]}\n' for lo,hi in ((210,250),(405,450)) for n in range(lo,hi+1))
b=''.join(f'reader:{n:03}: {reader[n-1]}\n' for n in range(451,501))+''.join(f'encoding:{n:03}: {encoding[n-1]}\n' for n in range(1,25))
assert (root/'DISPLAY-A.txt').read_text()==a and (root/'DISPLAY-B.txt').read_text()==b
assert 'not credited as fully displayed' in receipt['partial_display_qualification']
assert review['baseline_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009'

if args.git_repo:
 def show(path):
  return subprocess.check_output(['git','--no-replace-objects','-C',str(args.git_repo),'show','8608c88ad0cd75ca188ce03908c7662c1aa436e8:'+path])
 history={}
 for entry in reg['entries']:
  if entry['scope_kind']=='history':
   raw=show(entry['path']);assert (len(raw),sha(raw),oid('blob',raw))==(entry['bytes'],entry['sha256'],entry['git_blob']);history[entry['tag']]=json.loads(raw)
 for entry in ext['entries']:
  raw=show(entry['path']);assert (len(raw),sha(raw),oid('blob',raw))==(entry['bytes'],entry['sha256'],entry['git_blob']);history[entry['tag']]=json.loads(raw)
 q=history['queue059']['entries'];keys=['contract_id','kind','lens','status','canonical_owner','first_failed_contract_or_unresolved_question','historical_receipt','remaining_evidence','dependencies','closure_condition','body_or_population_credit','runtime_validation','implementation_gate_complete']
 expected={str(i):{k:q[i][k] for k in keys} for i in (152,153)}
 assert json.loads((root/'HISTORY-DISPLAY-A.txt').read_bytes())==expected
 row=q[2942];v=row['granular_contract_views'][0];c=v['contract_update_as_proposed'];keys=('contract_id_or_proposed_new_id','canonical_owner_or_explicit_unresolved_owner_question','first_failed_contract_or_unresolved_question','audit_disposition_for_exact_frozen_scope','accepted_unique_count_change','current_source_reconciled','closure_condition')
 expected={'contract_id':row['contract_id'],'lens':row['lens'],'body_or_population_credit':row['body_or_population_credit'],'implementation_gate_complete':row['implementation_gate_complete'],'remaining_evidence':row['remaining_evidence'],'exact_frozen_scope_dispositions':row['exact_frozen_scope_dispositions'],'view_0_selected_update':{k:c[k] for k in keys},'view_0_attributions':[{'original_index':x['original_index'],'source_id':x['source_id'],'frozen_git_revision':x['frozen_git_revision'],'attribution':x['attribution']} for x in c['exact_source_identities_and_assessed_ranges']],'view_0_all_current_and_all_lens_closure':v['all_current_and_all_lens_closure'],'view_0_new_body_credit':v['new_body_credit']}
 assert json.loads((root/'HISTORY-DISPLAY-B2.txt').read_bytes())==expected
 for name,tag,keys in [
  ('HISTORY-DISPLAY-C.txt','review39',('schema','author','recorded_utc','scope','authority','independent_personal_reads','package_identity','current_assessment')),
  ('HISTORY-DISPLAY-D.txt','review39',('existing_relationships','prior_attribution','historic_recovery_qualifications','open_questions')),
  ('HISTORY-DISPLAY-E.txt','prior95e',('schema','source_revision','scope','observations','reconnected_historical_findings','connected_historical_reason_family','new_current_candidates','historical_attribution_qualification','remaining','repair','validation'))]:
  assert json.loads((root/name).read_bytes())=={k:history[tag][k] for k in keys}
 q=history['live-queue']['entries'];ledger=history['live-ledger']['newly_assessed_ledger']['entries'];ids=['ADLER-L11-FBX-RECORD-EXTENT-ADMISSION','ADLER-L11-FBX-DECODE-BUDGET-ADMISSION','L12-CLOSURE-FBX-NUM-ENUM-NATIVE-ADAPTER'];found=[]
 for ident in ids:
  for i,row in enumerate(ledger):
   if row['id']==ident:found.append({'index':i,'id':ident,'record':row.get('record'),'repair':row.get('repair'),'validation':row.get('validation'),'discovery':row.get('discovery')})
 expected={'snapshot_revision':'8608c88ad0cd75ca188ce03908c7662c1aa436e8','reconciliation':{str(i):q[i] for i in (152,153,2942)},'matching_ledger_entries':found}
 assert json.loads((root/'HISTORY-DISPLAY-F.txt').read_bytes())==expected
 print('Historical selected fields checked against exact 8608 Git blobs; the full truncated first q2942 display was not credited.')
else:
 print('No Git repository supplied; historical selected-field comparisons were not repeated.')
print('Verified Git source proofs, exact registered displays, receipt and draft identities; semantic correctness and product behavior require separate assessment.')
