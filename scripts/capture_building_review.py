"""Run the production tactical capture harness and present its verified output."""
import argparse
import html
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import shutil
import tempfile
from capture_tactical_scenes import source_identity


def capture(profile, title, *, evidence_name="building-presentation.json", review_input="input.review.json"):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True, help="Fresh capture directory")
    parser.add_argument("--skip-build", action="store_true")
    parser.add_argument("--scene-input", type=Path, help="Explicit deterministic input variant")
    parser.add_argument("--view", action="append", default=[], help="Capture a named view (repeatable)")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    if not args.skip_build:
        subprocess.run(["cargo", "build", "-p", "adventuresim-tactical-client", "--bin", "tactical-scene-viewer", "--offline"], cwd=root, check=True)
    executable = root / "target/debug" / ("tactical-scene-viewer.exe" if os.name == "nt" else "tactical-scene-viewer")
    output = args.output.resolve()
    if output.exists():
        parser.error("--output must be a fresh directory")
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="building-review-", dir=output.parent) as runtime:
        runner = Path(runtime) / executable.name
        shutil.copy2(executable, runner)
        selector = ["--scene-input", str(args.scene_input.resolve())] if args.scene_input else ["--fixture", profile]
        command = [str(runner), *selector, "--profile", profile, "--output", str(output)]
        for view in args.view:
            command += ["--view", view]
        provenance = source_identity()
        with runner.open("rb") as binary:
            provenance["executable_sha256"] = hashlib.file_digest(binary, "sha256").hexdigest()
        environment = dict(os.environ, CAPTURE_SOURCE_IDENTITY=provenance["identity"])
        with output.with_suffix(".log").open("w", encoding="utf-8") as log:
            subprocess.run(command, cwd=root, env=environment, check=True, timeout=1800, stdout=log, stderr=subprocess.STDOUT,
                           creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0)
    for line in output.with_suffix(".log").read_text(encoding="utf-8", errors="replace").splitlines():
        if re.search(r"\bERROR\b|panicked at", line):
            raise RuntimeError("Production renderer reported an error: " + line)
    manifest = json.loads((output / "manifest.json").read_text(encoding="utf-8"))
    evidence = json.loads((output / evidence_name).read_text(encoding="utf-8"))
    if not manifest["validation"]["passed"] or not evidence["material_bindings_verified"]:
        raise RuntimeError("Production presentation validation failed")
    (output / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n", encoding="utf-8")
    if len(evidence["captured_views_ready"]) != len(manifest["captures"]) + 1:
        raise RuntimeError("Not every captured view passed the production asset gate")
    cards = []
    for record in manifest["captures"]:
        path = output / record["screenshot"]
        if not path.is_file() or path.stat().st_size == 0:
            raise RuntimeError(f"Missing screenshot: {path}")
        name = html.escape(record["screenshot"], quote=True)
        label = html.escape(record["label"])
        cards.append(f'<article><h2>{label}</h2><a href="{name}"><img src="{name}" alt="{label}" loading="lazy"></a></article>')
    document = f'''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{html.escape(title)}</title>
<style>body{{margin:0;background:#211c18;color:#f4e9ce;font:16px system-ui}}main{{max-width:1600px;margin:auto;padding:32px}}h1{{font-size:36px}}section{{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px}}article{{background:#302a24;padding:16px;border-radius:8px}}h2{{font-size:20px}}img{{width:100%;display:block}}a{{color:#e6c996}}@media(max-width:800px){{section{{grid-template-columns:1fr}}}}</style><main><h1>{html.escape(title)}</h1><p>Captured through the production tactical renderer. Native {html.escape(evidence['backend'])}, {html.escape(evidence['adapter'])}.</p><p><a href="manifest.json">Capture and lighting checks</a> · <a href="{html.escape(evidence_name, quote=True)}">Production material checks</a> · <a href="{html.escape(review_input, quote=True)}">Review inputs</a></p><section>'''
    (output / "index.html").write_text(document + "".join(cards) + "</section></main></html>", encoding="utf-8")
    print(output / "index.html", flush=True)
