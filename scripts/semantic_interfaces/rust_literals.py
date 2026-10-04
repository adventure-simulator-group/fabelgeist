"""Rust string discovery that preserves literal and comment boundaries.

This is a lexical adapter, not a Rust or embedded-language parser. It skips
line comments, nested block comments, and character literals before admitting
ordinary and raw string bodies. Escapes are decoded using Rust's spelling so
escaped function headers remain visible to the embedded-language inventory.
"""

from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
import re

from .model import SourceUnit


TOKEN_START = re.compile(
    r"""//|/\*|\b(?:b|c)?r(?P<hashes>\#*)"|"|'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^\\'\n])'""",
    re.DOTALL,
)
COMMENT_DELIMITER = re.compile(r"/\*|\*/")
INLINE_JAVASCRIPT_ATTRIBUTE = re.compile(
    r"#\s*\[\s*(?:wasm_bindgen\s*::\s*)?wasm_bindgen\s*\("
    r"[^)\]]*\binline_js\s*=\s*$",
    re.DOTALL,
)
UNRESOLVED_INLINE_JAVASCRIPT_ATTRIBUTE = re.compile(
    r"#\s*\[\s*(?:[A-Za-z_]\w*\s*::\s*)*[A-Za-z_]\w*\s*\("
    r"[^)\]]*\binline_js\s*=\s*(?:b|c)?$",
    re.DOTALL,
)


class RustLiteralRole(Enum):
    OTHER = "other"
    INLINE_JAVASCRIPT = "inline-javascript"
    UNRESOLVED_INLINE_JAVASCRIPT = "unresolved-inline-javascript"

    @classmethod
    def from_context(cls, source: SourceUnit) -> "RustLiteralRole":
        if INLINE_JAVASCRIPT_ATTRIBUTE.search(source.text):
            return cls.INLINE_JAVASCRIPT
        if UNRESOLVED_INLINE_JAVASCRIPT_ATTRIBUTE.search(source.text):
            return cls.UNRESOLVED_INLINE_JAVASCRIPT
        return cls.OTHER


@dataclass(frozen=True)
class RustStringLiteral:
    source: SourceUnit
    role: RustLiteralRole


@dataclass
class RustStringLiterals:
    entries: list[RustStringLiteral] = field(default_factory=list)

    @classmethod
    def from_source(cls, source: SourceUnit) -> "RustStringLiterals":
        literals = cls()
        text = source.text
        cursor = 0
        syntax_parts = []
        inline_javascript_ordinal = 0
        while token := TOKEN_START.search(text, cursor):
            syntax_parts.append(text[cursor:token.start()])
            cursor = token.end()
            spelling = token.group()
            if spelling == "//":
                end = text.find("\n", cursor)
                cursor = len(text) if end < 0 else end + 1
                syntax_parts.append(" ")
            elif spelling == "/*":
                depth = 1
                while depth and (delimiter := COMMENT_DELIMITER.search(text, cursor)):
                    depth += 1 if delimiter.group() == "/*" else -1
                    cursor = delimiter.end()
                if depth:
                    cursor = len(text)
                syntax_parts.append(" ")
            elif spelling.startswith("'"):
                syntax_parts.append("'_' ")
                continue
            elif token.group("hashes") is not None:
                terminator = '"' + token.group("hashes")
                end = text.find(terminator, cursor)
                role = RustLiteralRole.from_context(SourceUnit(source.path, "".join(syntax_parts)))
                if role is RustLiteralRole.INLINE_JAVASCRIPT and (end < 0 or not spelling.startswith("r")):
                    role = RustLiteralRole.UNRESOLVED_INLINE_JAVASCRIPT
                if end < 0:
                    end = len(text)
                path = source.path
                if role is RustLiteralRole.INLINE_JAVASCRIPT:
                    inline_javascript_ordinal += 1
                    path = Path(f"{source.path}#inline-js-{inline_javascript_ordinal}.js")
                literals.entries.append(RustStringLiteral(SourceUnit(path, text[cursor:end]), role))
                cursor = end + len(terminator)
                syntax_parts.append('"" ')
            else:
                body = []
                terminated = False
                while cursor < len(text):
                    character = text[cursor]
                    cursor += 1
                    if character == '"':
                        terminated = True
                        break
                    if character != "\\" or cursor == len(text):
                        body.append(character)
                        continue
                    escape = text[cursor]
                    cursor += 1
                    if escape == "u" and text[cursor:cursor + 1] == "{":
                        end = text.find("}", cursor + 1)
                        if end >= 0:
                            digits = text[cursor + 1:end].replace("_", "")
                            if re.fullmatch(r"[0-9a-fA-F]{1,6}", digits):
                                codepoint = int(digits, 16)
                                if codepoint <= 0x10FFFF:
                                    body.append(chr(codepoint))
                                    cursor = end + 1
                                    continue
                    elif escape == "x":
                        digits = text[cursor:cursor + 2]
                        if re.fullmatch(r"[0-9a-fA-F]{2}", digits):
                            body.append(chr(int(digits, 16)))
                            cursor += 2
                            continue
                    elif escape in {"\n", "\r"}:
                        while cursor < len(text) and text[cursor].isspace():
                            cursor += 1
                        continue
                    body.append({"n": "\n", "r": "\r", "t": "\t", "0": "\0"}.get(escape, escape))
                role = RustLiteralRole.from_context(SourceUnit(source.path, "".join(syntax_parts)))
                if role is RustLiteralRole.INLINE_JAVASCRIPT and not terminated:
                    role = RustLiteralRole.UNRESOLVED_INLINE_JAVASCRIPT
                path = source.path
                if role is RustLiteralRole.INLINE_JAVASCRIPT:
                    inline_javascript_ordinal += 1
                    path = Path(f"{source.path}#inline-js-{inline_javascript_ordinal}.js")
                literals.entries.append(RustStringLiteral(SourceUnit(path, "".join(body)), role))
                syntax_parts.append('"" ')
        return literals
