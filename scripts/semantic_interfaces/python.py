"""Python AST census, including lexical aliases and unresolved dynamic ports."""

from dataclasses import dataclass, field
import ast
from pathlib import Path
from enum import Enum

from .model import (
    Finding, Findings, InterfaceItem, InterfaceRule, SourceUnit,
    TokenFingerprint, TypeLeaf,
)


PRIMITIVES = {"int", "float", "bool", "str", "bytes", "bytearray", "complex"}
GENERIC_ERRORS = {
    "Exception", "BaseException", "RuntimeError", "ValueError", "TypeError",
    "AssertionError", "NotImplementedError", "OSError", "IOError",
}
CONTAINERS = {
    "list", "tuple", "dict", "set", "frozenset", "List", "Tuple", "Dict",
    "Set", "FrozenSet", "Sequence", "Iterable", "Iterator", "Optional",
    "Union", "Callable", "Mapping", "MutableMapping", "Generator", "Type",
    "type", "Literal", "Annotated", "ClassVar", "Final",
}


@dataclass
class TypeLeaves:
    entries: list[TypeLeaf] = field(default_factory=list)


@dataclass(frozen=True)
class ImportBinding:
    module: tuple[str, ...]
    members: tuple[str, ...]
    relative_level: int


@dataclass
class PythonScope:
    parent: "PythonScope | None" = None
    aliases: dict[str, ast.expr] = field(default_factory=dict)
    nominal: set[str] = field(default_factory=set)
    imports: dict[str, ImportBinding] = field(default_factory=dict)
    path: Path | None = None

    @classmethod
    def from_body(cls, body: list[ast.stmt], parent: "PythonScope | None") -> "PythonScope":
        scope = cls(parent)
        for node in body:
            if isinstance(node, ast.ClassDef):
                scope.nominal.add(node.name)
            elif isinstance(node, ast.Assign) and len(node.targets) == 1:
                target = node.targets[0]
                if isinstance(target, ast.Name):
                    # Whether an assignment is a type alias is determined
                    # when its name occurs in an annotation, not by guessing
                    # from capitalization or ignoring older Python syntax.
                    scope.aliases[target.id] = node.value
            elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
                if node.value is not None:
                    scope.aliases[node.target.id] = node.value
            elif isinstance(node, ast.TypeAlias):
                scope.aliases[node.name.id] = node.value
            elif isinstance(node, ast.ImportFrom):
                for imported in node.names:
                    scope.imports[imported.asname or imported.name] = ImportBinding(
                        tuple((node.module or "").split(".")), (imported.name,), node.level,
                    )
            elif isinstance(node, ast.Import):
                for imported in node.names:
                    scope.imports[imported.asname or imported.name.split(".")[0]] = ImportBinding(
                        tuple(imported.name.split(".") if imported.asname else [imported.name.split(".")[0]]), (), 0,
                    )
        return scope


class PythonModules:
    def __init__(self, sources: list[SourceUnit]):
        self.scopes: dict[Path, PythonScope] = {}
        for source in sources:
            syntax = source.python_syntax()
            scope = PythonScope.from_body(syntax.body, None)
            scope.path = source.path
            self.scopes[source.path] = scope

    def target(self, imported: ImportBinding, context: PythonScope) -> PythonScope | None:
        root = context
        while root.parent is not None:
            root = root.parent
        if imported.relative_level and root.path is not None:
            base = root.path.parent
            for _ in range(imported.relative_level - 1):
                base = base.parent
            module = base.joinpath(*imported.module)
        else:
            module = Path(*imported.module)
        candidates = [module.with_suffix(".py"), module / "__init__.py"]
        for path in candidates:
            if path in self.scopes:
                return self.scopes[path]
        # Python scripts commonly add their directory to sys.path. Resolve
        # only a unique repository module suffix; ambiguity stays visible.
        matches = [scope for path, scope in self.scopes.items() if any(
            path.parts[-len(candidate.parts):] == candidate.parts for candidate in candidates
        )]
        return matches[0] if len(matches) == 1 else None


@dataclass
class AliasVisit:
    active: set[tuple[int, str]] = field(default_factory=set)


class AnnotationContainer(Enum):
    LITERAL = "Literal"
    ANNOTATED = "Annotated"


class AnnotationProbe:
    def __init__(self, scope: PythonScope, modules: PythonModules):
        self.scope = scope
        self.modules = modules

    def inspect(self, node: ast.expr) -> TypeLeaves:
        return self._visit(node, self.scope, AliasVisit())

    def _container(self, node: ast.expr, scope: PythonScope, visiting: AliasVisit) -> AnnotationContainer | None:
        if isinstance(node, ast.Name):
            current = scope
            while current is not None:
                if node.id in current.nominal:
                    return None
                imported = current.imports.get(node.id)
                if imported is not None:
                    if imported.module[0] == "typing" and imported.members:
                        return next((kind for kind in AnnotationContainer if kind.value == imported.members[-1]), None)
                    return None
                if node.id in current.aliases:
                    key = (id(current), node.id)
                    if key in visiting.active:
                        return None
                    visiting.active.add(key)
                    result = self._container(current.aliases[node.id], current, visiting)
                    visiting.active.remove(key)
                    return result
                current = current.parent
            return next((kind for kind in AnnotationContainer if kind.value == node.id), None)
        if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name):
            current = scope
            while current is not None:
                imported = current.imports.get(node.value.id)
                if imported is not None and imported.module == ("typing",) and not imported.members:
                    return next((kind for kind in AnnotationContainer if kind.value == node.attr), None)
                if node.value.id in current.nominal or node.value.id in current.aliases:
                    break
                current = current.parent
        return None

    def _visit(self, node: ast.expr, scope: PythonScope, visiting: AliasVisit) -> TypeLeaves:
        if isinstance(node, ast.Constant):
            if node.value is None:
                return TypeLeaves()
            if isinstance(node.value, str):
                try:
                    return self._visit(ast.parse(node.value, mode="eval").body, scope, visiting)
                except SyntaxError:
                    return TypeLeaves([TypeLeaf(InterfaceRule.UNRESOLVED, "forward-annotation")])
            return TypeLeaves([TypeLeaf(InterfaceRule.RAW, "literal-annotation")])
        if isinstance(node, ast.Name):
            current: PythonScope | None = scope
            while current is not None:
                if node.id in current.nominal:
                    return TypeLeaves()
                if node.id in current.aliases:
                    key = (id(current), node.id)
                    if key in visiting.active:
                        return TypeLeaves([TypeLeaf(InterfaceRule.UNRESOLVED, f"recursive-alias:{node.id}")])
                    visiting.active.add(key)
                    result = self._visit(current.aliases[node.id], current, visiting)
                    visiting.active.remove(key)
                    return result
                if node.id in current.imports:
                    imported = current.imports[node.id]
                    if imported.module[0] in {"builtins", "typing", "collections"} and imported.members:
                        return self._builtin(ast.Name(id=imported.members[-1]))
                    key = (id(current), node.id)
                    if key in visiting.active:
                        return TypeLeaves([TypeLeaf(InterfaceRule.UNRESOLVED, f"recursive-import:{node.id}")])
                    target = self.modules.target(imported, current)
                    if target is not None and len(imported.members) == 1:
                        visiting.active.add(key)
                        result = self._visit(ast.Name(id=imported.members[0]), target, visiting)
                        visiting.active.remove(key)
                        return result
                    return TypeLeaves([TypeLeaf(InterfaceRule.UNRESOLVED, "imported-type:" + ".".join(imported.module + imported.members))])
                current = current.parent
            return self._builtin(node)
        if isinstance(node, ast.Attribute):
            if isinstance(node.value, ast.Name):
                current = scope
                while current is not None:
                    imported = current.imports.get(node.value.id)
                    if imported is not None and not imported.members:
                        if imported.module[0] in {"builtins", "typing", "collections"}:
                            return self._builtin(ast.Name(id=node.attr))
                        target = self.modules.target(imported, current)
                        if target is not None:
                            return self._visit(ast.Name(id=node.attr), target, visiting)
                    if node.value.id in current.nominal or node.value.id in current.aliases:
                        break
                    current = current.parent
            return TypeLeaves([TypeLeaf(InterfaceRule.UNRESOLVED, f"qualified-type:{ast.unparse(node)}")])
        if isinstance(node, ast.Subscript):
            container = self._container(node.value, scope, visiting)
            arguments = node.slice.elts if isinstance(node.slice, ast.Tuple) else [node.slice]
            if container is AnnotationContainer.LITERAL:
                return TypeLeaves([
                    TypeLeaf(InterfaceRule.RAW, "literal-annotation") if isinstance(value, ast.Constant)
                    else TypeLeaf(InterfaceRule.UNRESOLVED, "dynamic-literal")
                    for value in arguments
                ])
            if container is AnnotationContainer.ANNOTATED:
                return self._visit(arguments[0], scope, visiting)
            result = self._visit(node.value, scope, visiting)
            result.entries.extend(self._visit(node.slice, scope, visiting).entries)
            return result
        if isinstance(node, ast.Tuple | ast.List | ast.BinOp):
            result = TypeLeaves()
            for child in ast.iter_child_nodes(node):
                if isinstance(child, ast.expr):
                    result.entries.extend(self._visit(child, scope, visiting).entries)
            return result
        return TypeLeaves([TypeLeaf(InterfaceRule.UNRESOLVED, f"dynamic-annotation:{ast.unparse(node)}")])

    def _builtin(self, node: ast.Name) -> TypeLeaves:
        if node.id in PRIMITIVES:
            return TypeLeaves([TypeLeaf(InterfaceRule.RAW, node.id)])
        if node.id in GENERIC_ERRORS:
            return TypeLeaves([TypeLeaf(InterfaceRule.GENERIC_ERROR, node.id)])
        if node.id in CONTAINERS or node.id in {"None", "Self"}:
            return TypeLeaves()
        return TypeLeaves([TypeLeaf(InterfaceRule.UNRESOLVED, "unknown-type:" + node.id)])


class PythonScanner(ast.NodeVisitor):
    def __init__(self, source: SourceUnit, modules: PythonModules | None = None):
        self.source = source
        self.modules = modules or PythonModules([source])
        self.syntax = source.python_syntax()
        self.scope = self.modules.scopes[source.path]
        self.item = InterfaceItem(())
        self.owner: ast.ClassDef | None = None
        self.function: ast.AST | None = None
        self.findings = Findings()

    def inspect(self) -> Findings:
        self.visit(self.syntax)
        return self.findings

    def _record(self, node: ast.AST, leaf: TypeLeaf, slot: ast.AST) -> None:
        if isinstance(slot, ast.FunctionDef | ast.AsyncFunctionDef | ast.Lambda):
            signature = "return"
        elif isinstance(slot, ast.Raise):
            signature = "raise"
        else:
            signature = ast.unparse(slot)
        opaque = TokenFingerprint.from_ast(self.function or node) if leaf.rule is InterfaceRule.UNRESOLVED else None
        self.findings.entries.append(Finding(
            self.source.path, self.item, signature, leaf, signature,
            getattr(node, "lineno", 1), opaque,
        ))

    def _annotation(self, node: ast.AST, annotation: ast.expr | None) -> None:
        if annotation is None:
            self._record(node, TypeLeaf(InterfaceRule.UNRESOLVED, "unannotated-interface"), node)
            return
        for leaf in AnnotationProbe(self.scope, self.modules).inspect(annotation).entries:
            self._record(node, leaf, annotation)
        self.visit(annotation)

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        previous = self.scope, self.item, self.owner
        self.scope = PythonScope.from_body(node.body, self.scope)
        self.item = self.item.child(node)
        self.owner = None
        for expression in node.bases + [keyword.value for keyword in node.keywords] + node.decorator_list:
            self.visit(expression)
        self.owner = node
        for statement in node.body:
            self.visit(statement)
        self.scope, self.item, self.owner = previous

    def _function(self, node: ast.FunctionDef | ast.AsyncFunctionDef | ast.Lambda) -> None:
        previous = self.scope, self.item, self.function, self.owner
        self.item = self.item.child(node)
        self.function = node
        args = node.args
        static = any(
            (isinstance(decorator, ast.Name) and decorator.id == "staticmethod")
            or (isinstance(decorator, ast.Attribute) and decorator.attr == "staticmethod")
            for decorator in getattr(node, "decorator_list", [])
        )
        for index, argument in enumerate(args.posonlyargs + args.args):
            if (index == 0 and self.owner is not None and not static
                    and isinstance(node, ast.FunctionDef | ast.AsyncFunctionDef)
                    and argument.annotation is None and argument.arg in {"self", "cls"}):
                continue
            self._annotation(argument, argument.annotation)
        for argument in args.kwonlyargs:
            self._annotation(argument, argument.annotation)
        for argument in (args.vararg, args.kwarg):
            if argument is not None:
                self._annotation(argument, argument.annotation)
        self._annotation(node, getattr(node, "returns", None))
        self.owner = None
        for expression in args.defaults + [value for value in args.kw_defaults if value is not None] + getattr(node, "decorator_list", []):
            self.visit(expression)
        if isinstance(node, ast.Lambda):
            self.visit(node.body)
        else:
            self.scope = PythonScope.from_body(node.body, self.scope)
            for statement in node.body:
                self.visit(statement)
        self.scope, self.item, self.function, self.owner = previous

    visit_FunctionDef = _function
    visit_AsyncFunctionDef = _function
    visit_Lambda = _function

    def visit_Raise(self, node: ast.Raise) -> None:
        raised = node.exc.func if isinstance(node.exc, ast.Call) else node.exc
        if raised is not None:
            for leaf in AnnotationProbe(self.scope, self.modules).inspect(raised).entries:
                self._record(node, leaf, node)
        self.generic_visit(node)
