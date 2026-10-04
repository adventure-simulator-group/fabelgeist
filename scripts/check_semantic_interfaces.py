"""Census and enforce script/browser interface debt without executing sources."""

from pathlib import Path
import argparse
import json
import sys
from collections import Counter

from scripts.semantic_interfaces.gate import GateReport, InterfaceDebt
from scripts.semantic_interfaces.model import CensusError
from scripts.semantic_interfaces.repository import SourceUnits, census


ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("check", "census", "baseline"), nargs="?", default="check")
    args = parser.parse_args()
    try:
        findings = census(SourceUnits.from_repository(ROOT), ROOT)
        actual = InterfaceDebt.from_findings(findings)
        if args.command == "baseline":
            print(actual.as_wire(), end="")
            return
        if args.command == "census":
            for finding in findings.entries:
                print(f"{finding.path}:{finding.line}: {finding.as_wire_key()}")
            return
        expected = InterfaceDebt.load(ROOT / "script-interface-baseline.json")
        report = GateReport.compare(expected, actual)
        for mismatch in report.mismatches:
            print(f"interface debt changed: expected {mismatch.expected}, found {mismatch.actual}: {mismatch.key}", file=sys.stderr)
        if report.mismatches:
            raise SystemExit(1)
        counts = Counter(finding.leaf.rule.value for finding in findings.entries)
        print("Script/browser interface debt unchanged: " + json.dumps(dict(counts), sort_keys=True))
    except CensusError as cause:
        print(str(cause), file=sys.stderr)
        raise SystemExit(1) from cause


if __name__ == "__main__":
    main()
