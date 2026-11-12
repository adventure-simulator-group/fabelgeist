#!/usr/bin/env python3
"""Display only prospectively registered captured source ranges."""
import pathlib,json,sys
p=pathlib.Path(__file__).resolve().parent; path=sys.argv[1]; a=int(sys.argv[2]); b=int(sys.argv[3]); label=sys.argv[4]
regs=list(p.glob('SOURCE*REGISTRATION.json'));assert any(any(x['path']==path and x['range'][0]<=a and x['range'][1]>=b for x in json.loads(f.read_text()).get('sources',[])) for f in regs)
lines=(p/'source-95e'/path).read_text().splitlines(); b=min(b,len(lines));out=''.join(f'{i:4d} {lines[i-1]}\n' for i in range(a,b+1));dst=p/'displays'/f'{label}-{a}-{b}.txt';dst.parent.mkdir(exist_ok=True);dst.write_text(out);print(out,end='')
