"""Exact debt accounting: additions, growth, and stale findings fail the gate."""

from dataclasses import dataclass, field
from pathlib import Path
from enum import Enum
import json
import re

from .model import CensusError, Findings, InterfaceRule


class BaselineDecodeFailure(CensusError):
    def __init__(self, path: Path, cause: Exception):
        self.path = path
        self.__cause__ = cause
        super().__init__(f"cannot decode interface baseline {path}: {cause}")


class BaselineViolation(Enum):
    OBJECT_REQUIRED = "baseline must be an object"
    INVALID_KEY = "signature key must contain six strings and an optional SHA-256 fingerprint"
    UNKNOWN_RULE = "signature key has an unknown rule"
    INVALID_COUNT = "signature count must be a positive integer"


class BaselineFormatFailure(CensusError):
    def __init__(self, path: Path, violation: BaselineViolation):
        self.path = path
        self.violation = violation
        super().__init__(f"invalid interface baseline {path}: {violation.value}")


@dataclass(frozen=True)
class InterfaceDebt:
    counts: dict[str, int]

    @classmethod
    def load(cls, path: Path) -> "InterfaceDebt":
        try:
            counts = json.loads(path.read_text(encoding="utf-8"))
            if not isinstance(counts, dict):
                raise BaselineFormatFailure(path, BaselineViolation.OBJECT_REQUIRED)
            for key, count in counts.items():
                row = json.loads(key)
                if (not isinstance(row, list) or len(row) != 7
                        or any(not isinstance(value, str) for value in row[:6])
                        or (row[6] is not None and (not isinstance(row[6], str)
                            or re.fullmatch(r"[0-9a-f]{64}", row[6]) is None))):
                    raise BaselineFormatFailure(path, BaselineViolation.INVALID_KEY)
                if row[3] not in {rule.value for rule in InterfaceRule}:
                    raise BaselineFormatFailure(path, BaselineViolation.UNKNOWN_RULE)
                if type(count) is not int or count <= 0:
                    raise BaselineFormatFailure(path, BaselineViolation.INVALID_COUNT)
            return cls(counts)
        except (OSError, UnicodeDecodeError, json.JSONDecodeError) as cause:
            raise BaselineDecodeFailure(path, cause) from cause

    @classmethod
    def from_findings(cls, findings: Findings) -> "InterfaceDebt":
        return cls(findings.as_wire_counts())

    def as_wire(self) -> str:
        return json.dumps(self.counts, indent=2, ensure_ascii=False) + "\n"


@dataclass(frozen=True)
class DebtMismatch:
    key: str
    expected: int | None
    actual: int | None


@dataclass
class GateReport:
    mismatches: list[DebtMismatch] = field(default_factory=list)

    @classmethod
    def compare(cls, expected: InterfaceDebt, actual: InterfaceDebt) -> "GateReport":
        report = cls()
        for key in sorted(expected.counts.keys() | actual.counts.keys()):
            old, new = expected.counts.get(key), actual.counts.get(key)
            if old != new:
                report.mismatches.append(DebtMismatch(key, old, new))
        return report
