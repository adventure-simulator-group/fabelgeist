#!/usr/bin/env python3
"""Recheck recorded 13L displays against their bound source bytes, read only."""
import argparse,collections,gzip,hashlib,json,pathlib,tarfile
p=argparse.ArgumentParser();p.add_argument('--stage',required=True,type=pathlib.Path);p.add_argument('--out',required=True,type=pathlib.Path);a=p.parse_args();H=lambda x:hashlib.sha256(x).hexdigest();stage=a.stage;target='5dfe0bfe427cf9089f636ac8e52ae1a7e479e5ce0372e2fa8d884cd3f6c07e15';assert H((stage/'STAGE-MANIFEST.json').read_bytes())==target
cat=json.loads((stage/'CATALOG.json').read_bytes());item=next(x for x in cat['packets'] if x['id']=='central-13l');raw=(stage/item['archive']).read_bytes();assert H(raw)==item['archive_sha256']
with tarfile.open(stage/item['archive'],'r:gz') as t:arc={x.name:t.extractfile(x).read() for x in t if x.isfile()}
source=json.loads(arc['REGISTRATION.json']);items={x['id']:x for x in source['rows']};covered={k:set() for k in items};total=collections.Counter()
for regname,recname,displayname,kind in [('REGISTRATION.json','READ-RECEIPT.json','DISPLAY.txt','base'),('EXT-REGISTRATION.json','EXT-READ-RECEIPT.json','EXT-DISPLAY.txt','extension'),('EXT2-REGISTRATION.json','EXT2-READ-RECEIPT.json','EXT2-DISPLAY.txt','single')]:
 reg=json.loads(arc[regname]);receipt=json.loads(arc[recname]);assert reg['reader']==receipt['reader']=='/root' and reg['registered_utc']<receipt['displayed_utc'] and H(arc[regname])==receipt['registered_sha256'] and len(reg['rows'])==len(receipt['rows']);lines=[]
 if kind!='base':assert reg['prior_registration_sha256']==H(arc['REGISTRATION.json'])
 for row,read in zip(reg['rows'],receipt['rows']):
  assert row['id']==read['id'];obj=items[row['id']];body=arc[obj['stored_copy']];body=gzip.decompress(body) if row['id']=='rust-reference-cast' else body;source_lines=body.decode().splitlines();assert len(source_lines)==read['eof'] and row['planned_display_ranges']==read['displayed_ranges']
  if kind=='base':lines.append('=== '+row['id']+' ===')
  elif kind=='extension':lines.append('=== '+row['id']+' extension ===')
  for lo,hi in row['planned_display_ranges']:
   assert 1<=lo<=hi<=len(source_lines)
   for n in range(lo,hi+1):lines.append(f'{n:4d} {source_lines[n-1]}');covered[row['id']].add(n)
  if 'unread_complement' in read:
   listed={i for lo,hi in read['displayed_ranges'] for i in range(lo,hi+1)};unread={i for lo,hi in read['unread_complement'] for i in range(lo,hi+1)};assert not listed&unread and listed|unread==set(range(1,len(source_lines)+1))
 result=('\n'.join(lines)+'\n').encode();assert result==arc[displayname] and H(result)==receipt['display_sha256'];total[kind]=len(lines)
assessment=json.loads(arc['ASSESSMENT.json']);assert assessment['source_revision']==cat['working_revision'] and assessment['repair']=='not started' and assessment['product_validation']=='not performed'
for ref in assessment['source_references']:
 for lo,hi in ref['lines']:assert set(range(lo,hi+1))<=covered[ref['id']]
assert H((stage/'STAGE-MANIFEST.json').read_bytes())==target
out={'schema':'726-successor-13l-display-check-v1','stage_manifest_sha256':target,'original_display_reader':'/root','registered_source_count':len(items),'display_line_counts':dict(total),'source_citations_within_recorded_ranges':True,'qualification':'Recorded registration, transcript and source bytes reconcile; reproduction does not prove human/model attention or semantic correctness. Historical readers and the separately reviewed wording remain attributed as before. The source is the pinned working revision, not a claim that the working ledger equals any committed baseline.'};assert not a.out.exists();a.out.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'source_count':len(items),'display_lines':dict(total),'cites_in_receipts':True}))
