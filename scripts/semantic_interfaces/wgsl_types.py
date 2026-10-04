"""WGSL type ownership: aliases cannot turn scalar storage into domain meaning."""

from dataclasses import dataclass, field
import re

from .model import InterfaceRule, TokenFingerprint, TypeLeaf
from .wgsl_tokens import (
    ShaderIdentifier, ShaderSpelling, ShaderSyntax, ShaderTokens,
)


@dataclass
class ShaderTypeLeaves:
    entries: list[TypeLeaf] = field(default_factory=list)


@dataclass
class ShaderAliasVisits:
    active: set[ShaderSpelling] = field(default_factory=set)


@dataclass
class ShaderTypeRegistry:
    _structures: dict[ShaderSpelling, set[TokenFingerprint]] = field(
        default_factory=dict,
    )
    _aliases: dict[ShaderSpelling, dict[ShaderSpelling, ShaderTokens]] = field(
        default_factory=dict,
    )

    def admit(self, tokens: ShaderTokens) -> None:
        cursor = tokens.cursor()
        while keyword := cursor.take():
            if keyword.syntax() not in {ShaderSyntax.STRUCT, ShaderSyntax.ALIAS}:
                continue
            name = cursor.take()
            if (name is None or name.spelling.identifier()
                    is not ShaderIdentifier.NAMED):
                continue
            if keyword.syntax() is ShaderSyntax.STRUCT:
                if cursor.peek_syntax() is not ShaderSyntax.OPEN_BODY:
                    continue
                cursor.take()
                body = cursor.until(ShaderSyntax.CLOSE_BODY)
                if cursor.take() is None:
                    continue
                fingerprint = TokenFingerprint.from_text(str(body.spelling()))
                self._structures.setdefault(name.spelling, set()).add(fingerprint)
            elif cursor.peek_syntax() is ShaderSyntax.ASSIGN:
                cursor.take()
                target = cursor.until(ShaderSyntax.SEMICOLON)
                if cursor.take() is not None:
                    self._aliases.setdefault(name.spelling, {})[
                        target.spelling()
                    ] = target

    def inspect(self, expression: ShaderTokens) -> ShaderTypeLeaves:
        return self._inspect(expression.without_attributes(), ShaderAliasVisits())

    def _inspect(
        self, expression: ShaderTokens, visiting: ShaderAliasVisits,
    ) -> ShaderTypeLeaves:
        spelling = expression.spelling()
        cursor = expression.cursor()
        first = cursor.take()
        if first is None:
            return ShaderTypeLeaves([
                TypeLeaf(InterfaceRule.UNRESOLVED, "missing-type"),
            ])
        name = first.spelling
        if name in visiting.active:
            return ShaderTypeLeaves([
                TypeLeaf(InterfaceRule.UNRESOLVED, f"cyclic-alias:{name}"),
            ])
        targets = self._aliases.get(name)
        structures = self._structures.get(name)
        if (targets and structures) or (targets and len(targets) != 1):
            return ShaderTypeLeaves([
                TypeLeaf(InterfaceRule.UNRESOLVED, f"ambiguous-type:{name}"),
            ])
        if targets and cursor.peek() is None:
            visiting.active.add(name)
            leaves = self._inspect(next(iter(targets.values())), visiting)
            visiting.active.remove(name)
            return leaves
        if structures and cursor.peek() is None:
            if len(structures) == 1:
                return ShaderTypeLeaves()
            return ShaderTypeLeaves([
                TypeLeaf(InterfaceRule.UNRESOLVED, f"ambiguous-struct:{name}"),
            ])
        if cursor.peek() is None:
            if str(name) in {"bool", "i32", "u32", "f32", "f16", "i64", "u64", "f64"}:
                return ShaderTypeLeaves([TypeLeaf(InterfaceRule.RAW, str(name))])
            if re.fullmatch(r"(?:vec[234][fhiu]|mat[234]x[234][fh])", str(name)):
                return ShaderTypeLeaves([TypeLeaf(InterfaceRule.RAW, str(name))])
        if cursor.peek_syntax() is ShaderSyntax.OPEN_TEMPLATE:
            cursor.take()
            contents = cursor.until(ShaderSyntax.CLOSE_TEMPLATE)
            closing = cursor.take()
            if closing is not None and cursor.peek() is None:
                arguments = contents.split(ShaderSyntax.COMMA).entries
                element = None
                scalar_container = (str(name) == "atomic" or re.fullmatch(
                    r"(?:vec[234]|mat[234]x[234])", str(name),
                ) is not None)
                if scalar_container and len(arguments) == 1:
                    leaves = self._inspect(arguments[0], visiting)
                    if leaves.entries:
                        return leaves
                elif str(name) == "array" and len(arguments) in {1, 2}:
                    element = arguments[0]
                elif str(name) == "ptr" and len(arguments) in {2, 3}:
                    address = str(arguments[0].spelling())
                    access = str(arguments[2].spelling()) if len(arguments) == 3 else None
                    if (address in {"function", "private", "workgroup", "uniform", "storage"}
                            and access in {None, "read", "write", "read_write"}):
                        element = arguments[1]
                if element is not None:
                    return self._inspect(element, visiting)
        return ShaderTypeLeaves([
            TypeLeaf(InterfaceRule.UNRESOLVED, f"unknown-type:{spelling}"),
        ])
