"""Signature census for standalone WGSL and unassembled embedded fragments.

Function bodies are not compiled or type-inferred. Host opaque coverage remains
in place: a parsed prototype does not prove source assembly or domain ownership.
"""

from dataclasses import dataclass, field
from pathlib import Path

from .model import (
    Finding, Findings, InterfaceItem, InterfaceRule, SourceUnit,
    TokenFingerprint, TypeLeaf,
)
from .wgsl_tokens import (
    ShaderIdentifier, ShaderLine, ShaderSpelling, ShaderSyntax, ShaderTokens,
)
from .wgsl_types import ShaderTypeRegistry


@dataclass
class ShaderInputs:
    entries: list[SourceUnit] = field(default_factory=list)


@dataclass(frozen=True)
class ShaderParameter:
    name: ShaderSpelling
    annotation: ShaderTokens

    @classmethod
    def from_tokens(cls, tokens: ShaderTokens) -> "ShaderParameter":
        cursor = tokens.without_attributes().cursor()
        name = cursor.until(ShaderSyntax.COLON).spelling()
        if cursor.take() is None:
            return cls(name, ShaderTokens())
        return cls(name, cursor.remaining())


@dataclass(frozen=True)
class ShaderSignature:
    source: SourceUnit
    name: ShaderSpelling
    parameters: tuple[ShaderParameter, ...]
    result: ShaderTokens | None
    line: ShaderLine
    fingerprint: TokenFingerprint

    def finding(self, slot: ShaderSpelling, leaf: TypeLeaf) -> Finding:
        parameters = ",".join(
            f"{parameter.name}:{parameter.annotation.spelling()}"
            for parameter in self.parameters
        )
        result = str(self.result.spelling()) if self.result is not None else "void"
        return Finding(
            self.source.path, InterfaceItem((str(self.name),)), str(slot), leaf,
            f"fn {self.name}({parameters})->{result}", int(self.line),
            self.fingerprint if leaf.rule is InterfaceRule.UNRESOLVED else None,
        )


@dataclass
class ShaderScanner:
    inputs: ShaderInputs

    def inspect(self) -> Findings:
        parsed = []
        registries = {}
        # Embedded identities share declarations only within the same Rust host.
        # Conflicting declarations remain ambiguous instead of choosing one.
        for source in self.inputs.entries:
            tokens = ShaderTokens.from_source(source)
            host = Path(str(source.path).split("#wgsl-", 1)[0])
            registry = registries.setdefault(host, ShaderTypeRegistry())
            registry.admit(tokens)
            parsed.append((source, tokens, registry))
        findings = Findings()
        for source, tokens, registry in parsed:
            cursor = tokens.cursor()
            while keyword := cursor.take():
                if keyword.syntax() is ShaderSyntax.ALIAS:
                    name = cursor.take()
                    if name is not None:
                        findings.entries.append(Finding(
                            source.path, InterfaceItem((str(name.spelling),)),
                            "alias", TypeLeaf(InterfaceRule.UNRESOLVED, "type-alias"),
                            "alias", int(keyword.line),
                            TokenFingerprint.from_text(str(tokens.spelling())),
                        ))
                    continue
                if keyword.syntax() is not ShaderSyntax.FUNCTION:
                    continue
                name = cursor.until(ShaderSyntax.OPEN_PAREN).spelling()
                opening = cursor.take()
                parameters = cursor.until(ShaderSyntax.CLOSE_PAREN)
                closing = cursor.take()
                result = None
                if cursor.peek_syntax() is ShaderSyntax.RETURN:
                    cursor.take()
                    result = cursor.until(ShaderSyntax.OPEN_BODY)
                signature = ShaderSignature(
                    source, name,
                    tuple(ShaderParameter.from_tokens(group) for group in
                          parameters.split(ShaderSyntax.COMMA).entries),
                    result, keyword.line,
                    TokenFingerprint.from_text(str(tokens.spelling())),
                )
                if (opening is None or closing is None
                        or name.identifier() is not ShaderIdentifier.NAMED):
                    findings.entries.append(signature.finding(
                        ShaderSpelling("signature"),
                        TypeLeaf(InterfaceRule.UNRESOLVED, "incomplete-or-templated-function"),
                    ))
                for parameter in signature.parameters:
                    for leaf in registry.inspect(parameter.annotation).entries:
                        findings.entries.append(signature.finding(parameter.name, leaf))
                if result is not None:
                    for leaf in registry.inspect(result).entries:
                        findings.entries.append(signature.finding(
                            ShaderSpelling("return"), leaf,
                        ))
        return findings
