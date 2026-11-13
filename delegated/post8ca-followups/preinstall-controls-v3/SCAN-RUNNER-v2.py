#!/usr/bin/env python3
"""Run the pinned evidence-only preinstallation scanner once with logs."""
from pathlib import Path
import hashlib, json, os, subprocess, sys

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p = Path(__file__).resolve().parent
base = p.parent
paths = {
    'scan': (base / 'post8ca-preinstall-scan-v2/RUN.py', '309188e55860c4893698ea6fdb88bd27adf3ec29fa823bdbd6c36ce0d481ab70'),
    'digest': (base / 'next-15a-digest-corpus-358yvabw/corpus/scan-private-digests.py', 'a2cb3c124802385e0edd4e80731bfeb334b95dcd4721589a87f507208d069917'),
    'pattern': (base / 'checkpoint-after13v-prepared-success-paAYncnb/tar-aware-pattern-scanner.py', '4982a500f168f6f5e5ae82a50b4b7e98edda6e5c622e8dc365d0aad10a08bf0f'),
}
h = lambda b: hashlib.sha256(b).hexdigest()
def verify_inputs():
    for path, expected in paths.values():
        assert path.is_file() and not path.is_symlink() and h(path.read_bytes()) == expected
verify_inputs()
assert {x.name for x in p.iterdir()} == {'RUN.py', 'DERIVATION.json'}
for name in ('tree', 'RESULT.json', 'digest.stdout', 'digest.stderr', 'pattern.stdout', 'pattern.stderr'):
    assert not (base / 'post8ca-preinstall-scan-v2' / name).exists()
assert not (base / 'post8ca-preinstall-scan-v2/tree').exists()
assert not (base / 'post8ca-preinstall-scan-v2/RESULT.json').exists()
env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', PYTHONNOUSERSITE='1')
env.pop('PYTHONOPTIMIZE', None)
result = subprocess.run([sys.executable, '-B', str(paths['scan'][0])],
                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
for name, data in [('STDOUT', result.stdout), ('STDERR', result.stderr),
                   ('EXIT', (str(result.returncode) + '\n').encode())]:
    with (p / name).open('xb') as f:
        f.write(data)
        f.flush()
        os.fsync(f.fileno())
verify_inputs()
receipt = {
    'schema': '726-post8ca-preinstall-scan-run-v2',
    'runner_sha256': h(Path(__file__).read_bytes()),
    'input_script_sha256': {name: expected for name, (_, expected) in paths.items()},
    'exit': result.returncode, 'stdout_sha256': h(result.stdout),
    'stderr_bytes': len(result.stderr), 'stderr_sha256': h(result.stderr),
    'qualification': 'Same-machine evidence-only scan execution with pinned scanner source identities. It does not establish independent execution telemetry, complete private-data coverage, authenticated restoration or product behavior.',
}
with (p / 'RECEIPT.json').open('x') as f:
    json.dump(receipt, f, indent=2)
    f.write('\n')
    f.flush()
    os.fsync(f.fileno())
print(json.dumps({'exit': result.returncode, 'stderr_bytes': len(result.stderr)}))
assert result.returncode == 0 and not result.stderr
