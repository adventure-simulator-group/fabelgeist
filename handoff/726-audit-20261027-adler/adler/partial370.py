"""Ignored evidence only: save already personally read lock ranges, no product execution."""
import sys
from pathlib import Path
import bookkeeping as b
import continuation as c
seq,start,end=map(int,sys.argv[1:])
regp=b.OUT/'REGISTRATION-ADLER-1173.json';reg=b.read(regp)
obj=Path(reg['object_path'])
assert b.sha(obj)==reg['sha256'] and len(obj.read_text(encoding='utf-8-sig').splitlines())==7093
assert reg['original_index']==2313 and reg['ranges']==[[1,7093]] and reg['prospective']
cur=b.read(b.ROOT/'CURRENT-ADLER.json');front=next(v for v in cur['frontiers'] if v['lens']==11)
assert front['next_original']==front['original_accounted']==2313
prior=Path(b.ROOT/front['state']);assert b.sha(prior)==front['state_sha256']
if seq==1:
 assert start==1 and prior==b.OUT/'STATE-L11-369.json'
 previous=[]
else:
 old=b.read(b.OUT/f'READ-L11-370-PARTIAL-{seq-1:03}.json')
 assert old['read_through']+1==start
 previous=old['partial_chain']+[dict(receipt=str(b.OUT/f'READ-L11-370-PARTIAL-{seq-1:03}.json'),sha256=b.sha(b.OUT/f'READ-L11-370-PARTIAL-{seq-1:03}.json'))]
assert 1<=start<=end<=7093
rp=b.OUT/f'READ-L11-370-PARTIAL-{seq:03}.json'
sp=b.OUT/f'STATE-L11-370-PARTIAL-{seq:03}.json'
b.save(rp,dict(lens=11,original_index=2313,kind='Independent exact same-lens personally read partial frozen requested source; zero ordered original credit',registration=str(regp),registration_sha256=b.sha(regp),source_id=reg['source_id'],source_sha256=reg['sha256'],registered_scope=reg['ranges'],fresh_reads=[[start,end]],read_through=end,cumulative_read_ranges=[[1,end]],partial_chain=previous,ordered_predecessor=str(b.OUT/'STATE-L11-369.json'),ordered_predecessor_sha256=b.sha(b.OUT/'STATE-L11-369.json'),additional_original_frontier_credit=0,next_original=2313,accepted_unique_count_change=0,current_reconciled=False,validated=False,implementation_gate_complete=False,disposition='Exact dependency package/version/source/checksum and dependency-name records only; native API/source bodies and capability/target resolution not inferred. No builds/tests/scanners/generators/runtime probes or repair. All13 populations/current/all20/aliases/connected acceptance remain mandatory. No whole-file credit until1..7093 complete.',pending_ranges=[[end+1,7093]] if end<7093 else [],truncated_output_credit=False))
b.save(sp,dict(lens=11,original_accounted=2313,original_total=2853,next_original=2313,newly_accounted=[],partial_original=2313,read_through=end,predecessor=str(prior),predecessor_sha256=b.sha(prior),receipt=str(rp),receipt_sha256=b.sha(rp),all_inherited_qualifications_retained=True,implementation_gate_complete=False,accepted_unique_count_change=0))
c.advance(11,2313,sp,rp,f'2313/2853 originals source-accounted; original2313 partial1..{end}/7093, zero new ordered credit',f'Continue registered original2313 REG1173 from{end+1}..7093; aggregate exact partial proofs before final370. Full ordered predecessor369 remains. All13 populations/current/all20/aliases/acceptance incomplete; no product execution/repairs.',partial=True)
now=b.read(b.ROOT/'CURRENT-ADLER.json');f=next(v for v in now['frontiers'] if v['lens']==11)
assert f['next_original']==2313 and f['partial_receipt']==rp.relative_to(b.ROOT).as_posix()
assert f['state_sha256']==b.sha(sp) and now['ledger_sha256']==b.sha(b.ROOT/now['ledger'])
for x in previous: assert b.sha(Path(x['receipt']))==x['sha256']
b.save(b.OUT/f'VERIFY-L11-370-PARTIAL-{seq:03}.json',dict(kind='Ignored partial evidence integrity only; not product verification',receipt=str(rp),receipt_sha256=b.sha(rp),state=str(sp),state_sha256=b.sha(sp),registration=str(regp),registration_sha256=b.sha(regp),source_sha256=b.sha(obj),ledger_sha256=now['ledger_sha256'],read_through=end,additional_original_frontier_credit=0,implementation_gate_complete=False,accepted_unique_count_change=0))
print(dict(partial=seq,original_index=2313,read_through=end,next_original=2313,ordered_credit=0))
