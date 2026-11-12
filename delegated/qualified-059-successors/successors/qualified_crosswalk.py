"""Read-only, path-specific original/public 059 identity crosswalk.

A historical SHA/length describes the original transport object. Public SHA,
length and Git blob describe separately reconstructed bytes. Public checks do
not reconstruct private original paths or certify a byte-exact transformation.
"""
import argparse
import gzip
import hashlib
import json
import os
import stat
from pathlib import Path

sha = lambda b: hashlib.sha256(b).hexdigest()
def git_blob(b):
    return hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()

def _absolute(path):
    return Path(os.path.abspath(os.fspath(path)))

def _assert_components_not_symlinks(path):
    p = _absolute(path)
    current = Path(p.anchor)
    for part in p.parts[1:]:
        current = current / part
        mode = os.lstat(current).st_mode
        if stat.S_ISLNK(mode):
            raise ValueError('symlink component is not permitted')
    return p

def _reject_tree_symlinks(root, *, skip_git):
    # Do not recurse through symlinks; fail instead of following one.
    for here, dirs, files in os.walk(root, topdown=True, followlinks=False):
        if skip_git:
            dirs[:] = [name for name in dirs if name != '.git']
        for name in dirs + files:
            if stat.S_ISLNK(os.lstat(Path(here) / name).st_mode):
                raise ValueError('symlink in evidence tree is not permitted')

class Crosswalk:
    def __init__(self, root, public_dir, config):
        self.root = _assert_components_not_symlinks(root)
        _reject_tree_symlinks(self.root, skip_git=True)
        self.public_dir = _assert_components_not_symlinks(public_dir)
        _reject_tree_symlinks(self.public_dir, skip_git=False)
        config_path = _assert_components_not_symlinks(config)
        config_bytes = config_path.read_bytes()
        assert sha(config_bytes) == 'c6d9ccf4bcfa84ad0eef68e51a5b5a49824b4a1253560b6a0ba22281c485f33d'
        data = json.loads(config_bytes)
        assert data['schema'] == '726-059-qualified-crosswalk-v1'
        rows = data['records']; assert len(rows) == 2
        self.rows = {x['public_path']:x for x in rows}
        assert set(self.rows) == {'baseline-059/queue.json','baseline-059/ledger.json'}
        t = data['transport']; assert t['path'] == 'recovered/original-transport-manifest.json.gz'
        packed = (self.root / t['path']).read_bytes()
        assert sha(packed) == t['stored_sha256']
        raw = gzip.decompress(packed); assert sha(raw) == t['uncompressed_sha256']
        entries = json.loads(raw)['entries']
        self.public = {}
        for name,row in self.rows.items():
            matches = [x for x in entries if x['path'] == row['historical_transport_path']]
            assert len(matches) == 1
            original = matches[0]
            assert (original['sha256'],original['bytes'],original['git_blob']) == (row['original_sha256'],row['original_bytes'],row['original_git_blob'])
            b = (self.public_dir / Path(name).name).read_bytes()
            assert (sha(b),len(b),git_blob(b)) == (row['public_sha256'],row['public_bytes'],row['public_git_blob'])
            assert row['original_sha256'] != row['public_sha256']
            self.public[name] = b
            print('QUALIFIED_ORIGINAL_TO_PUBLIC_CROSSWALK',name,'original_sha256='+row['original_sha256'],'public_sha256='+row['public_sha256'],'not_direct_original_public_byte_match', flush=True)
        self.crosswalk_links_checked = 0

    def read(self, path):
        if path in self.rows:
            return self.public[path]
        return (self.root / path).read_bytes()

    def matches(self, path, actual, expected_sha, expected_bytes=None):
        if path not in self.rows:
            return sha(actual) == expected_sha and (expected_bytes is None or len(actual) == expected_bytes)
        row = self.rows[path]
        if expected_sha != row['original_sha256']:
            return False
        if expected_bytes is not None and expected_bytes != row['original_bytes']:
            return False
        if actual != self.public[path]:
            return False
        self.crosswalk_links_checked += 1
        return True

    def known(self, paths):
        result = {}
        for p in paths:
            raw = self.read(p)
            if p in self.rows:
                row = self.rows[p]
                assert self.matches(p,raw,row['original_sha256'],row['original_bytes'])
                h,n = row['original_sha256'],row['original_bytes']
            else:
                h,n = sha(raw),len(raw)
            if h in result:
                assert result[h] == n
            result[h] = n
        return result

    def finish(self):
        print('QUALIFIED_059_CROSSWALK_RESULT','qualified_path_checks='+str(self.crosswalk_links_checked),'original_sha_and_public_sha_are_distinct','original_byte_transformation_not_proved_by_public_check',flush=True)

def configure(config_path):
    p = argparse.ArgumentParser(description='Read-only evidence check with qualified 059 crosswalk')
    p.add_argument('--publication', type=Path, required=True)
    p.add_argument('--baseline-dir', type=Path, help='Directory containing proposed public queue.json and ledger.json; defaults to publication/baseline-059')
    args = p.parse_args()
    root = _absolute(args.publication)
    public = _absolute(args.baseline_dir) if args.baseline_dir else root / 'baseline-059'
    return Crosswalk(root, public, config_path)
