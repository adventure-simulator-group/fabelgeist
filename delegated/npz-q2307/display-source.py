#!/usr/bin/env python3
"""Display only prospectively registered source ranges; no product execution."""
from pathlib import Path
import json,sys
p=Path(__file__).resolve().parent;out=sys.argv[1];assert out.startswith('SOURCE-DISPLAY-') and '/' not in out
reg=json.loads((p/'SOURCE-REGISTRATION.json').read_text());parts=[]
for arg in sys.argv[2:]:
 name,a,b=arg.split(':');lo=int(a);hi=int(b);rows=[r for r in reg['rows'] if r['path'].endswith('/'+name)];assert len(rows)==1;row=rows[0]
 assert any(x<=lo<=hi<=y for x,y in row['registered_ranges']);lines=(p/row['source_file']).read_text().splitlines();parts.append(f"=== {row['path']} {lo}..{hi} ===");parts.extend(f'{i:4d} | {lines[i-1]}' for i in range(lo,hi+1))
s='\n'.join(parts)+'\n'
with (p/out).open('x') as f:f.write(s)
print(s,end='')
