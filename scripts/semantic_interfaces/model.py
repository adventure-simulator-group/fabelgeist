"""Shared census records; wire strings are converted at the report boundary."""

from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
import ast
import hashlib
import json


class InterfaceRule(Enum):
    RAW = "raw-interface"
    UNRESOLVED = "unresolved-interface"
    GENERIC_ERROR = "generic-error"
    OPAQUE = "opaque-source"


@dataclass(frozen=True)
class TypeLeaf:
    rule: InterfaceRule
    spelling: str


@dataclass(frozen=True)
class InterfaceItem:
    parts: tuple[str, ...]

    def child(self, node: ast.AST) -> "InterfaceItem":
        return InterfaceItem(self.parts + (getattr(node, "name", "lambda"),))


@dataclass(frozen=True)
class TokenFingerprint:
    digest: str

    @classmethod
    def from_ast(cls, node: ast.AST) -> "TokenFingerprint":
        text = ast.dump(node, include_attributes=False)
        return cls(hashlib.sha256(text.encode("utf-8")).hexdigest())

    @classmethod
    def from_text(cls, text: str) -> "TokenFingerprint":
        return cls(hashlib.sha256(text.encode("utf-8")).hexdigest())


class CensusError(Exception):
    """A census failed before a complete inventory could be established."""


class SourceReadError(CensusError):
    def __init__(self, path: Path, cause: OSError | UnicodeDecodeError):
        self.path = path
        self.__cause__ = cause
        super().__init__(f"cannot read source {path}: {cause}")


class SourceParseError(CensusError):
    def __init__(self, path: Path, cause: SyntaxError):
        self.path = path
        self.__cause__ = cause
        super().__init__(f"cannot parse source {path}: {cause}")


@dataclass(frozen=True)
class SourceUnit:
    path: Path
    text: str

    @classmethod
    def load(cls, path: Path) -> "SourceUnit":
        try:
            return cls(path, path.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError) as cause:
            raise SourceReadError(path, cause) from cause

    def python_syntax(self) -> ast.Module:
        try:
            return ast.parse(self.text, filename=str(self.path))
        except SyntaxError as cause:
            raise SourceParseError(self.path, cause) from cause


@dataclass(frozen=True)
class Finding:
    path: Path
    item: InterfaceItem
    slot: str
    leaf: TypeLeaf
    signature: str
    line: int
    opaque: TokenFingerprint | None = None

    def as_wire_key(self) -> str:
        return json.dumps([
            str(self.path), "::".join(self.item.parts), self.slot,
            self.leaf.rule.value, self.leaf.spelling, self.signature,
            self.opaque.digest if self.opaque else None,
        ], ensure_ascii=False, separators=(",", ":"))


@dataclass
class Findings:
    entries: list[Finding] = field(default_factory=list)

    def extend(self, other: "Findings") -> None:
        self.entries.extend(other.entries)

    def as_wire_counts(self) -> dict[str, int]:
        counts: dict[str, int] = {}
        for entry in self.entries:
            key = entry.as_wire_key()
            counts[key] = counts.get(key, 0) + 1
        return dict(sorted(counts.items()))
