"""WGSL signature tokens, preserving comments, attributes and nested templates."""

from dataclasses import dataclass, field
from enum import Enum
import re

from .model import CensusError, SourceUnit


class ShaderSyntax(Enum):
    FUNCTION = "fn"
    STRUCT = "struct"
    ALIAS = "alias"
    OPEN_PAREN = "("
    CLOSE_PAREN = ")"
    OPEN_TEMPLATE = "<"
    CLOSE_TEMPLATE = ">"
    OPEN_BODY = "{"
    CLOSE_BODY = "}"
    COMMA = ","
    COLON = ":"
    SEMICOLON = ";"
    ASSIGN = "="
    ATTRIBUTE = "@"
    RETURN = "->"
    OTHER = "other"


class ShaderBalance(Enum):
    EMPTY = "empty"
    NESTED = "nested"
    MALFORMED = "malformed"


class ShaderIdentifier(Enum):
    NAMED = "named"
    UNSUPPORTED = "unsupported"


@dataclass(frozen=True)
class ShaderSpelling:
    _text: str

    def __str__(self) -> str:
        return self._text

    def syntax(self) -> ShaderSyntax:
        try:
            return ShaderSyntax(self._text)
        except ValueError:
            return ShaderSyntax.OTHER

    def identifier(self) -> ShaderIdentifier:
        if re.fullmatch(r"[^\W\d]\w*", self._text, flags=re.UNICODE):
            return ShaderIdentifier.NAMED
        return ShaderIdentifier.UNSUPPORTED


@dataclass(frozen=True)
class ShaderLine:
    _line: int

    def __post_init__(self) -> None:
        if self._line < 1:
            raise ShaderLineAdmissionFailure(self._line)

    def __int__(self) -> int:
        return self._line


class ShaderLineAdmissionFailure(CensusError):
    def __init__(self, line: int) -> None:
        self._rejected = line
        super().__init__(f"shader source line must be positive; received {line}")


@dataclass(frozen=True)
class ShaderToken:
    spelling: ShaderSpelling
    line: ShaderLine

    def syntax(self) -> ShaderSyntax:
        return self.spelling.syntax()


@dataclass
class ShaderDelimiters:
    _closings: list[ShaderSyntax] = field(default_factory=list)
    _state: ShaderBalance = ShaderBalance.EMPTY

    def admit(self, syntax: ShaderSyntax) -> None:
        closings = {
            ShaderSyntax.OPEN_PAREN: ShaderSyntax.CLOSE_PAREN,
            ShaderSyntax.OPEN_TEMPLATE: ShaderSyntax.CLOSE_TEMPLATE,
            ShaderSyntax.OPEN_BODY: ShaderSyntax.CLOSE_BODY,
        }
        if syntax in closings:
            self._closings.append(closings[syntax])
        elif syntax in closings.values():
            if not self._closings or self._closings.pop() is not syntax:
                self._state = ShaderBalance.MALFORMED

    def balance(self) -> ShaderBalance:
        if self._state is ShaderBalance.MALFORMED:
            return self._state
        return ShaderBalance.NESTED if self._closings else ShaderBalance.EMPTY


@dataclass(frozen=True)
class ShaderTokens:
    _entries: tuple[ShaderToken, ...] = ()

    @classmethod
    def from_source(cls, source: SourceUnit) -> "ShaderTokens":
        # This lexer admits source text; it never executes or assembles it.
        pattern = re.compile(
            r"//|/\*|\*/|->|[^\W\d]\w*|\d+(?:\.\d*)?|[^\s]",
            re.UNICODE,
        )
        comment_depth = 0
        entries = []
        cursor = 0
        while match := pattern.search(source.text, cursor):
            cursor = match.end()
            spelling = match.group()
            if spelling == "/*":
                comment_depth += 1
            elif spelling == "*/" and comment_depth:
                comment_depth -= 1
            elif not comment_depth and spelling == "//":
                end = source.text.find("\n", cursor)
                cursor = len(source.text) if end < 0 else end + 1
            elif not comment_depth:
                line = ShaderLine(source.text.count("\n", 0, match.start()) + 1)
                entries.append(ShaderToken(ShaderSpelling(spelling), line))
        return cls(tuple(entries))

    def cursor(self) -> "ShaderCursor":
        return ShaderCursor(self)

    def spelling(self) -> ShaderSpelling:
        return ShaderSpelling("".join(str(token.spelling) for token in self._entries))

    def first(self) -> ShaderToken | None:
        return self._entries[0] if self._entries else None

    def split(self, delimiter: ShaderSyntax) -> "ShaderTokenGroups":
        groups = []
        current = []
        stack = ShaderDelimiters()
        for token in self._entries:
            if token.syntax() is delimiter and stack.balance() is ShaderBalance.EMPTY:
                groups.append(ShaderTokens(tuple(current)))
                current = []
            else:
                stack.admit(token.syntax())
                current.append(token)
        if current:
            groups.append(ShaderTokens(tuple(current)))
        return ShaderTokenGroups(tuple(groups))

    def without_attributes(self) -> "ShaderTokens":
        cursor = self.cursor()
        while cursor.peek_syntax() is ShaderSyntax.ATTRIBUTE:
            cursor.take()
            cursor.take()
            if cursor.peek_syntax() is ShaderSyntax.OPEN_PAREN:
                cursor.take()
                cursor.until(ShaderSyntax.CLOSE_PAREN)
                cursor.take()
        return cursor.remaining()


@dataclass(frozen=True)
class ShaderTokenGroups:
    entries: tuple[ShaderTokens, ...]


@dataclass
class ShaderCursor:
    _tokens: ShaderTokens
    _index: int = field(default=0, init=False)

    def peek(self) -> ShaderToken | None:
        if self._index < len(self._tokens._entries):
            return self._tokens._entries[self._index]
        return None

    def peek_syntax(self) -> ShaderSyntax:
        token = self.peek()
        return token.syntax() if token else ShaderSyntax.OTHER

    def take(self) -> ShaderToken | None:
        token = self.peek()
        if token:
            self._index += 1
        return token

    def remaining(self) -> ShaderTokens:
        return ShaderTokens(self._tokens._entries[self._index:])

    def until(self, delimiter: ShaderSyntax) -> ShaderTokens:
        entries = []
        stack = ShaderDelimiters()
        while token := self.peek():
            if token.syntax() is delimiter and stack.balance() is ShaderBalance.EMPTY:
                break
            if stack.balance() is ShaderBalance.MALFORMED:
                break
            entries.append(self.take())
            stack.admit(token.syntax())
        return ShaderTokens(tuple(entries))
