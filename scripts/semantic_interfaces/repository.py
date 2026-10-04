"""Repository discovery and parser adapters, including opaque language debt."""

from dataclasses import dataclass, field
from html.parser import HTMLParser
from pathlib import Path
import json
import re
import subprocess

from .model import (
    CensusError, Finding, Findings, InterfaceItem, InterfaceRule, SourceUnit,
    TokenFingerprint, TypeLeaf,
)
from .python import PythonModules, PythonScanner
from .rust_literals import RustLiteralRole, RustStringLiterals
from .wgsl import ShaderInputs, ShaderScanner


SCRIPT_SUFFIXES = {".js", ".mjs", ".cjs", ".jsx", ".ts", ".mts", ".cts", ".tsx"}
OPAQUE_SUFFIXES = {".wgsl", ".glsl", ".vert", ".frag", ".cpp", ".cxx", ".cc", ".c", ".h", ".hpp", ".sh", ".bash", ".ps1", ".nix"}
OPAQUE_NAMES = {"justfile", "Dockerfile"}
EMBEDDED_MARKERS = {"<script", "@compute", "@vertex", "@fragment", "var<storage", "var<uniform"}
EMBEDDED_FUNCTION = re.compile(r"\b(?:fn|function)\s+(?:[A-Za-z_]\w*|\{[^{}]*\})\s*(?:<[^;{}]*>)?\s*\(")
SHADER_DECLARATION = re.compile(
    r"\b(?:fn\s+(?:[A-Za-z_]\w*|\{[^{}]*\})\s*\("
    r"|struct\s+\w+\s*\{[^{}]*\}|alias\s+\w+\s*=[^;{}]*;)"
)
GENERATED_PREFIX = "crates/adventuresim-stdb-client/src/"


class DiscoveryFailure(CensusError):
    def __init__(self, root: Path, cause: subprocess.CalledProcessError | OSError):
        self.root = root
        self.__cause__ = cause
        super().__init__(f"cannot inventory repository {root}: {cause}")


class JavascriptToolFailure(CensusError):
    def __init__(self, cause: subprocess.CalledProcessError | OSError):
        self.__cause__ = cause
        super().__init__(f"JavaScript census failed: {getattr(cause, 'stderr', None) or cause}")


class JavascriptOutputFailure(CensusError):
    def __init__(self, cause: Exception):
        self.__cause__ = cause
        super().__init__(f"cannot decode JavaScript census: {cause}")


@dataclass
class SourceUnits:
    entries: list[SourceUnit] = field(default_factory=list)

    @classmethod
    def from_repository(cls, root: Path) -> "SourceUnits":
        try:
            result = subprocess.run(
                ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
                cwd=root, capture_output=True, check=True,
            )
        except (subprocess.CalledProcessError, OSError) as cause:
            raise DiscoveryFailure(root, cause) from cause
        paths = sorted(set(result.stdout.decode("utf-8").split("\0")) - {""})
        units = cls()
        for relative in paths:
            if relative.startswith(GENERATED_PREFIX):
                continue
            path = Path(relative)
            if (path.suffix in SCRIPT_SUFFIXES | OPAQUE_SUFFIXES | {".py", ".html", ".rs"}
                    or path.name in OPAQUE_NAMES
                    or (relative.startswith(".github/workflows/") and path.suffix in {".yml", ".yaml"})):
                absolute = root / path
                # git ls-files also reports tracked files removed in the
                # checkout; deletion is legitimate and ratchets their debt.
                if absolute.is_file():
                    loaded = SourceUnit.load(absolute)
                    units.entries.append(SourceUnit(path, loaded.text))
        return units


@dataclass
class JavascriptInputs:
    entries: list[SourceUnit] = field(default_factory=list)


class InlineJavascript(HTMLParser):
    """HTML callbacks have signatures mandated by the standard parser."""

    def __init__(self, source: SourceUnit):
        super().__init__(convert_charrefs=False)
        self.source = source
        self.scripts = JavascriptInputs()
        self.active: list[str] | None = None
        self.ordinal = 0
        self.events = Findings()

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attributes = dict(attrs)
        if tag == "script" and "src" not in attributes and attributes.get("type", "") in {
            "", "module", "text/javascript", "application/javascript",
        }:
            self.active = []
        if any(name.startswith("on") for name in attributes):
            self.events.entries.append(opaque_finding(self.source))

    def handle_data(self, data: str) -> None:
        if self.active is not None:
            self.active.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag == "script" and self.active is not None:
            self._finish_script()

    def close(self) -> None:
        super().close()
        if self.active is not None:
            # HTMLParser can retain an unterminated CDATA script in its
            # internal buffer. Keep the source visible even without an AST.
            self.events.entries.append(opaque_finding(self.source))
            self._finish_script()

    def _finish_script(self) -> None:
        self.ordinal += 1
        self.scripts.entries.append(SourceUnit(
            Path(f"{self.source.path}#script-{self.ordinal}.js"), "".join(self.active or []),
        ))
        self.active = None


def opaque_finding(source: SourceUnit) -> Finding:
    return Finding(
        source.path, InterfaceItem(("unparsed-language",)), "source",
        TypeLeaf(InterfaceRule.OPAQUE, source.path.suffix), "opaque",
        1, TokenFingerprint.from_text(source.text),
    )


def javascript_census(inputs: JavascriptInputs, root: Path) -> Findings:
    if not inputs.entries:
        return Findings()
    payload = [{"path": str(source.path), "text": source.text} for source in inputs.entries]
    try:
        result = subprocess.run(
            ["node", "--experimental-transform-types", str(root / "scripts/semantic_interfaces/javascript.mts")],
            input=json.dumps(payload), text=True, capture_output=True, cwd=root, check=True,
        )
    except (subprocess.CalledProcessError, OSError) as cause:
        raise JavascriptToolFailure(cause) from cause
    findings = Findings()
    try:
        records = json.loads(result.stdout)
        for record in records:
            findings.entries.append(Finding(
                Path(record["path"]), InterfaceItem(tuple(record["item"].split("::"))),
                record["slot"], TypeLeaf(InterfaceRule(record["rule"]), record["spelling"]),
                record["signature"], record["line"],
                TokenFingerprint(record["opaque"]) if record["opaque"] else None,
            ))
    except (ValueError, TypeError, KeyError) as cause:
        raise JavascriptOutputFailure(cause) from cause
    return findings


def census(sources: SourceUnits, root: Path) -> Findings:
    findings = Findings()
    javascript = JavascriptInputs()
    shaders = ShaderInputs()
    modules = PythonModules([source for source in sources.entries if source.path.suffix == ".py"])
    for source in sources.entries:
        if source.path.suffix == ".py":
            findings.extend(PythonScanner(source, modules).inspect())
        elif source.path.suffix in SCRIPT_SUFFIXES:
            javascript.entries.append(source)
        elif source.path.suffix == ".html":
            parser = InlineJavascript(source)
            parser.feed(source.text)
            parser.close()
            javascript.entries.extend(parser.scripts.entries)
            findings.extend(parser.events)
        elif source.path.suffix == ".rs":
            literals = RustStringLiterals.from_source(source)
            opaque = False
            shader_ordinal = 0
            for literal in literals.entries:
                if literal.role is RustLiteralRole.INLINE_JAVASCRIPT:
                    javascript.entries.append(literal.source)
                elif (literal.role is RustLiteralRole.UNRESOLVED_INLINE_JAVASCRIPT
                        or EMBEDDED_FUNCTION.search(literal.source.text) is not None
                        or any(marker in literal.source.text for marker in EMBEDDED_MARKERS)):
                    opaque = True
                if (literal.role is RustLiteralRole.OTHER
                        and (SHADER_DECLARATION.search(literal.source.text)
                             or any(marker in literal.source.text for marker in
                                    {"@compute", "@vertex", "@fragment", "var<storage", "var<uniform"}))):
                    shader_ordinal += 1
                    shaders.entries.append(SourceUnit(
                        Path(f"{source.path}#wgsl-{shader_ordinal}.wgsl"),
                        literal.source.text,
                    ))
                    opaque = True
            if opaque:
                findings.entries.append(opaque_finding(source))
        elif (source.path.suffix in OPAQUE_SUFFIXES or source.path.name in OPAQUE_NAMES
                or (source.path.parts[:2] == (".github", "workflows") and source.path.suffix in {".yml", ".yaml"})):
            findings.entries.append(opaque_finding(source))
            if source.path.suffix == ".wgsl":
                shaders.entries.append(source)
    findings.extend(ShaderScanner(shaders).inspect())
    findings.extend(javascript_census(javascript, root))
    return findings
