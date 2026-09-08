"""Capture both text-only shop-sign mounts and fonts using the game's GPU components."""
import argparse
import html
import json
import os
from pathlib import Path
import subprocess

CASES = [
    ('grenze-wall', 'Wall board / Grenze Gotisch', ['--mount', 'wall']),
    ('grenze-projecting', 'Projecting board / Grenze Gotisch', ['--mount', 'projecting']),
    ('fraktur-wall', 'Wall board / UnifrakturCook', ['--mount', 'wall', '--font', 'unifraktur-cook']),
    ('fraktur-projecting', 'Projecting board / UnifrakturCook', ['--mount', 'projecting', '--font', 'unifraktur-cook']),
    ('reverse', 'Opposite reading direction', ['--scene', 'reverse']),
    ('long-name', 'Long name and German characters', ['--scene', 'long-name']),
    ('street', 'Several shops along a street', ['--scene', 'along-street']),
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--skip-build', action='store_true')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    if not args.skip_build:
        subprocess.run(['cargo', 'build', '-p', 'adventuresim-building-generator', '--features', 'viewer', '--bin', 'shop-sign-viewer', '--offline'], cwd=root, check=True)
    executable = root / 'target/debug' / ('shop-sign-viewer.exe' if os.name == 'nt' else 'shop-sign-viewer')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    cards = []
    for name, title, options in CASES:
        path = output / (name + '.png')
        with path.with_suffix('.log').open('w', encoding='utf-8') as log:
            subprocess.run([str(executable), '--output', str(path), *options], cwd=root, check=True, timeout=240,
                           stdout=log, stderr=subprocess.STDOUT,
                           creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        records = json.loads(path.with_suffix('.json').read_text(encoding='utf-8'))
        if not path.exists() or path.stat().st_size == 0 or any(record['audit'] or not record['site_clear'] for record in records):
            raise RuntimeError('Capture or geometry audit failed: ' + name)
        cards.append(f'<article><h2>{html.escape(title)}</h2><a href="{name}.png"><img src="{name}.png" alt="{html.escape(title)}"></a><a href="{name}.json">Names and geometry checks</a></article>')
        print('Captured ' + name, flush=True)
    document = '''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Shop signs</title>
<style>body{margin:0;background:#211c18;color:#f4e9ce;font:16px system-ui}main{max-width:1600px;margin:auto;padding:32px}h1{font-size:40px}section{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px}article{background:#302a24;padding:20px;border-radius:8px}h2{font-size:20px;margin:0 0 15px}img{width:100%;display:block}a{color:#e6c996}article>a:last-child{display:inline-block;margin-top:12px}@media(max-width:800px){section{grid-template-columns:1fr}}</style><main><h1>Text-only shop signs</h1><p>GPU captures using the shared building meshes, sign hardware, and locally bundled blackletter fonts. No illustrations or floating name labels.</p><section>'''
    (output / 'index.html').write_text(document + ''.join(cards) + '</section></main>', encoding='utf-8')
    print(output / 'index.html', flush=True)


if __name__ == '__main__':
    main()
