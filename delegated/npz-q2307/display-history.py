#!/usr/bin/env python3
"""Display the previously registered exact historical queue entry."""
from pathlib import Path
import sys
p=Path(__file__).resolve().parent;out,lo,hi=sys.argv[1],int(sys.argv[2]),int(sys.argv[3]);name='QUEUE-ENTRY-ORIGINAL.json';lines=(p/name).read_text().splitlines();assert out.startswith('HISTORY-DISPLAY-') and 1<=lo<=hi<=len(lines);s=f'=== {name} {lo}..{hi} ===\n'+'\n'.join(f'{i:4d} | {lines[i-1]}' for i in range(lo,hi+1))+'\n'
with (p/out).open('x') as f:f.write(s)
print(s,end='')
