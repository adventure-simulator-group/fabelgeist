#!/usr/bin/env python3
"""Replay the ede predecessor against the archived f2ec evidence state."""
from pathlib import Path
import subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
root=Path(__file__).resolve().parent
subprocess.run([sys.executable,'-B',str(root/'delegated/postf2ec-followups/application/PREDECESSOR-REPLAY.py'),'ede',str(root)],check=True)
