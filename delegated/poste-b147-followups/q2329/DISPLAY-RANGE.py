#!/usr/bin/env python3
"""Display prospectively registered captured source ranges."""
import pathlib,json,sys
p=pathlib.Path(__file__).resolve().parent;path=sys.argv[1];a=int(sys.argv[2]);z=int(sys.argv[3]);label=sys.argv[4]
assert any(any(x['path']==path and x['range'][0]<=a<=z<=x['range'][1] for x in json.loads(f.read_text()).get('sources',[])) for f in p.glob('SOURCE*REGISTRATION.json'))
lines=(p/'source-95e'/path).read_text().splitlines();z=min(z,len(lines));s=''.join(f'{i:4d} {lines[i-1]}\n' for i in range(a,z+1));d=p/'displays'/f'{label}-{a}-{z}.txt';d.parent.mkdir(exist_ok=True);d.write_text(s);print(s,end='')
