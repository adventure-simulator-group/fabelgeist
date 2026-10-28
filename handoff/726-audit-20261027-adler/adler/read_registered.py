"""Display exact prospectively registered source ranges, no product execution."""
import sys
from pathlib import Path
import bookkeeping as b
sys.stdout.reconfigure(encoding='utf-8')
num,start,end=map(int,sys.argv[1:])
r=b.read(b.OUT/f'REGISTRATION-ADLER-{num:03}.json')
if r.get('deleted_original'):
    print('HISTORICAL BASE BODY ONLY; frozen current original is deleted, not live source')
    r=r['base_source']
p=Path(r['object_path'])
assert b.sha(p)==r['sha256']
assert 1 <= start <= end and end-start < 200
assert set(range(start,end+1)) <= set(i for a,z in r['ranges'] for i in range(a,z+1))
lines=p.read_text(encoding='utf-8-sig').splitlines()
assert end <= len(lines)
print(f"REG{num} {r['source_id']} SHA256={r['sha256']} scope={start}..{end} actualEOF={len(lines)}")
for i in range(start,end+1): print(f'{i}: {lines[i-1]}')
