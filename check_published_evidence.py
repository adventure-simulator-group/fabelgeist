"""Run this checkpoint's source and recovery verifiers, never product checks."""
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
# These establish the recovered state and both Git-tree inventories before
# checks that bind source bytes to them.
first = [
    'verify_recovered.py',
    'verify_selected_states.py',
    'verify_lock_inventory.py',
    'verify_delta_sources.py',
    'verify_role_worklist.py',
]
optional = {'verify_local_checkpoint_059.py'}
remaining = sorted(path.name for path in root.glob('verify_*.py')
                   if path.name not in first and path.name not in optional)
native = sorted(path.relative_to(root).as_posix()
                for path in (root / 'native-evidence').glob('*/verify.py'))
for name in first + remaining + native:
    print('CHECK', name, flush=True)
    subprocess.run([sys.executable, str(root / name)], cwd=root, check=True)
print('Published evidence checks passed; no product build, test or probe ran.')
