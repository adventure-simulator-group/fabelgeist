"""Capture audited working buildings with the native GPU viewer and write a review gallery."""
import argparse
import html
import json
import os
from pathlib import Path
import subprocess

KINDS = ("barn", "stable", "granary", "smithy", "bakehouse", "market-hall")
VIEWS = {"exterior": [], "cutaway": ["--cutaway"], "shell": ["--representation", "shell"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True, help="Fresh capture directory")
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--size", choices=("small", "medium", "large"), default="medium")
    parser.add_argument("--skip-build", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    if not args.skip_build:
        subprocess.run(["cargo", "build", "-p", "adventuresim-building-generator", "--features", "viewer", "--bin", "workplace-viewer", "--offline"], cwd=root, check=True)
    executable = root / "target/debug" / ("workplace-viewer.exe" if os.name == "nt" else "workplace-viewer")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    for kind in KINDS:
        for view, options in VIEWS.items():
            path = output / f"{kind}-{view}.png"
            command = [str(executable), "--kind", kind, "--size", args.size, "--seed", str(args.seed), "--output", str(path), *options]
            with path.with_suffix(".log").open("w", encoding="utf-8") as log:
                subprocess.run(command, cwd=root, check=True, timeout=180, stdout=log, stderr=subprocess.STDOUT,
                               creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0)
            if not path.is_file() or path.stat().st_size == 0:
                raise RuntimeError(f"GPU capture did not produce {path}")
            record = json.loads(path.with_suffix(".json").read_text(encoding="utf-8"))
            if record["audit"]:
                raise RuntimeError(f"Structural audit failed for {kind}")
            print(f"Captured {kind}: {view}", flush=True)
    write_gallery(output, args.size, args.seed)
    print(output / "index.html", flush=True)


def write_gallery(output, size, seed):
    cards = []
    for kind in KINDS:
        title = html.escape(kind.replace("-", " ").title())
        cards.append(f'<article><h2>{title}</h2><a class="capture" href="{kind}-exterior.png"><img data-kind="{kind}" src="{kind}-exterior.png" alt="{title}"></a><a href="{kind}-exterior.json">Recipe and audit</a></article>')
    document = '''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Working buildings</title><style>
body{margin:0;background:#edf0ed;color:#19251d;font:16px system-ui}main{max-width:1500px;margin:auto;padding:36px}h1{font-size:40px;margin:0}p{line-height:1.6}nav{display:flex;gap:12px;margin:25px 0}button{font:inherit;border:1px solid #809182;border-radius:6px;padding:10px 22px;background:white;cursor:pointer}button.active{background:#243d2b;color:white}section{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px}article{background:white;border:1px solid #ccd3ca;border-radius:10px;overflow:hidden;padding:20px}h2{margin:0 0 12px}img{display:block;width:100%;height:auto}a{color:#335d3d}article>a:last-child{display:inline-block;margin-top:10px}@media(max-width:800px){section{grid-template-columns:1fr}main{padding:18px}}
</style><main><h1>Working buildings</h1><p>GPU captures of generated geometry using the shared production texture recipes. All structural audits pass. Cutaways hide surfaces only for inspection; distant views retain the major architectural features.</p>'''
    document += f'<p>Size: {html.escape(size)}. Seed: {seed}.</p><nav><button class="active" data-view="exterior">Exterior</button><button data-view="cutaway">Cutaway</button><button data-view="shell">Distant geometry</button></nav><section>'
    document += "".join(cards) + '''</section></main><script>
for(const button of document.querySelectorAll('button'))button.addEventListener('click',()=>{for(const other of document.querySelectorAll('button'))other.classList.toggle('active',other===button);for(const image of document.querySelectorAll('img')){image.src=`${image.dataset.kind}-${button.dataset.view}.png`;image.parentElement.href=image.src;}});
</script></html>'''
    (output / "index.html").write_text(document, encoding="utf-8")


if __name__ == "__main__":
    main()
