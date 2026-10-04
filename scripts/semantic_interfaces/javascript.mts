/** Babel's parsed syntax is the boundary; no repository source is executed. */
import { parsers } from "prettier/plugins/babel";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { posix } from "node:path";

interface SyntaxNode {
  type: string;
  start?: number;
  end?: number;
  loc?: { start: { line: number } };
  [key: string]: any;
}

interface SourceWire {
  path: string;
  text: string;
}
interface FindingWire {
  path: string;
  item: string;
  slot: string;
  rule: string;
  spelling: string;
  signature: string;
  line: number;
  opaque: string | null;
}

class InterfaceRule {
  private constructor(readonly wireName: string) {}
  static readonly Raw = new InterfaceRule("raw-interface");
  static readonly Unresolved = new InterfaceRule("unresolved-interface");
  static readonly GenericError = new InterfaceRule("generic-error");
}

class TypeLeaf {
  constructor(
    readonly rule: InterfaceRule,
    readonly spelling: string,
  ) {}
}

class TypeLeaves {
  constructor(readonly entries: TypeLeaf[] = []) {}
  extend(other: TypeLeaves): void {
    this.entries.push(...other.entries);
  }
}

class InterfaceItem {
  constructor(readonly parts: string[] = []) {}
  child(node: SyntaxNode): InterfaceItem {
    return new InterfaceItem([
      ...this.parts,
      node.id?.name ?? node.key?.name ?? "closure",
    ]);
  }
}

class TokenFingerprint {
  private constructor(readonly digest: string) {}
  static fromNode(node: SyntaxNode): TokenFingerprint {
    // JSON's callback parameters are mandated by that external serializer.
    const tokens = JSON.stringify(node, (key: string, value: unknown) =>
      [
        "start",
        "end",
        "loc",
        "extra",
        "comments",
        "leadingComments",
        "trailingComments",
        "innerComments",
        "tokens",
        "__contentEnd",
      ].includes(key)
        ? undefined
        : typeof value === "bigint"
          ? { bigint: value.toString() }
          : value,
    );
    return new TokenFingerprint(
      createHash("sha256").update(tokens).digest("hex"),
    );
  }
}

class SourceParseFailure extends Error {
  constructor(source: SourceWire, cause: unknown) {
    super(`cannot parse ${source.path}`, { cause });
  }
}

class SourceUnit {
  readonly path: string;
  readonly text: string;
  readonly program: SyntaxNode;
  constructor(wire: SourceWire) {
    this.path = wire.path;
    this.text = wire.text;
    try {
      const parser = /\.[cm]?tsx?$/.test(wire.path)
        ? parsers["babel-ts"]
        : parsers.babel;
      const parsed = parser.parse(wire.text, { filepath: wire.path });
      if (parsed.errors?.length) throw parsed.errors[0];
      this.program = parsed.program;
    } catch (cause) {
      throw new SourceParseFailure(wire, cause);
    }
  }
  asSpelling(node: SyntaxNode): string {
    if (node.start === undefined) return node.type;
    return this.text.slice(node.start, node.end).replace(/\s+/g, " ");
  }
}

class SyntaxChildren {
  constructor(readonly nodes: SyntaxNode[]) {}
  static fromNode(node: SyntaxNode): SyntaxChildren {
    const children: SyntaxNode[] = [];
    for (const [key, value] of Object.entries(node)) {
      if (["loc", "comments", "tokens"].includes(key)) continue;
      for (const child of Array.isArray(value) ? value : [value]) {
        if (
          child &&
          typeof child === "object" &&
          typeof child.type === "string"
        )
          children.push(child);
      }
    }
    return new SyntaxChildren(children);
  }
}

class TypeScope {
  readonly aliases = new Map<string, SyntaxNode>();
  readonly nominal = new Set<string>();
  readonly imports = new Map<string, ImportedType>();
  readonly exports = new Map<string, string | ImportedType>();
  readonly exportStars: ImportedType[] = [];
  readonly parameters = new Set<string>();
  constructor(
    readonly parent: TypeScope | null = null,
    readonly source: SourceUnit | null = parent?.source ?? null,
  ) {}
  static fromBody(
    body: SyntaxNode,
    parent: TypeScope | null,
    source: SourceUnit | null = parent?.source ?? null,
  ): TypeScope {
    const scope = new TypeScope(parent, source);
    for (let node of body.body ?? []) {
      if (node.type === "ExportAllDeclaration") {
        scope.exportStars.push(new ImportedType(node.source.value, null));
        continue;
      }
      if (node.type === "ExportNamedDeclaration") {
        for (const exported of node.specifiers ?? []) {
          const name = exported.exported.name ?? exported.exported.value;
          const local = exported.local.name ?? exported.local.value;
          scope.exports.set(
            name,
            node.source ? new ImportedType(node.source.value, local) : local,
          );
        }
        if (node.declaration?.id)
          scope.exports.set(node.declaration.id.name, node.declaration.id.name);
        node = node.declaration;
      }
      if (node?.type === "ExportDefaultDeclaration") {
        if (node.declaration?.id)
          scope.exports.set("default", node.declaration.id.name);
        node = node.declaration;
      }
      if (!node) continue;
      if (node.type === "TSTypeAliasDeclaration")
        scope.aliases.set(node.id.name, node);
      if (
        [
          "ClassDeclaration",
          "TSInterfaceDeclaration",
          "TSEnumDeclaration",
        ].includes(node.type)
      )
        scope.nominal.add(node.id.name);
      if (node.type === "ImportDeclaration") {
        for (const imported of node.specifiers)
          scope.imports.set(
            imported.local.name,
            new ImportedType(
              node.source.value,
              imported.type === "ImportDefaultSpecifier"
                ? "default"
                : imported.type === "ImportNamespaceSpecifier"
                  ? null
                  : (imported.imported.name ?? imported.imported.value),
            ),
          );
      }
    }
    return scope;
  }
}

class ImportedType {
  constructor(
    readonly module: string,
    readonly member: string | null,
  ) {}
}

class TypeTarget {
  constructor(
    readonly scope: TypeScope,
    readonly name: string,
  ) {}
}

class SourceModules {
  readonly scopes = new Map<string, TypeScope>();
  constructor(readonly sources: SourceUnit[]) {
    for (const source of sources) {
      this.scopes.set(
        source.path,
        TypeScope.fromBody(source.program, null, source),
      );
    }
  }
  target(imported: ImportedType, scope: TypeScope): TypeScope | null {
    if (!scope.source || !imported.module.startsWith(".")) return null;
    const base = posix.normalize(
      posix.join(posix.dirname(scope.source.path), imported.module),
    );
    const paths = [
      base,
      ...[".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"].flatMap(
        (extension) => [base + extension, base + "/index" + extension],
      ),
    ];
    if (base.endsWith(".js"))
      paths.push(base.slice(0, -3) + ".ts", base.slice(0, -3) + ".tsx");
    const matches = [...new Set(paths)]
      .map((path) => this.scopes.get(path))
      .filter((scope) => scope !== undefined);
    return matches.length === 1 ? matches[0] : null;
  }
  exported(
    scope: TypeScope,
    name: string,
    visited: Set<string> = new Set(),
  ): TypeTarget | null {
    const key = `${scope.source?.path}:${name}`;
    if (visited.has(key)) return null;
    visited.add(key);
    const value = scope.exports.get(name);
    if (typeof value === "string") return new TypeTarget(scope, value);
    if (value instanceof ImportedType) {
      const target = this.target(value, scope);
      return target && value.member
        ? this.exported(target, value.member, visited)
        : null;
    }
    const found: TypeTarget[] = [];
    for (const star of scope.exportStars) {
      const target = this.target(star, scope);
      const result = target && this.exported(target, name, new Set(visited));
      if (result) found.push(result);
    }
    return found.length === 1 ? found[0] : null;
  }
}

class TypeBindings {
  readonly values = new Map<string, TypeLeaves>();
  readonly aliases = new Set<SyntaxNode>();
  readonly imports = new Set<ImportedType>();
}

const scalarKinds = new Set([
  "TSNumberKeyword",
  "TSStringKeyword",
  "TSBooleanKeyword",
  "TSBigIntKeyword",
  "TSSymbolKeyword",
  "NumberTypeAnnotation",
  "StringTypeAnnotation",
  "BooleanTypeAnnotation",
  "BigIntTypeAnnotation",
  "NumberLiteralTypeAnnotation",
  "StringLiteralTypeAnnotation",
  "BooleanLiteralTypeAnnotation",
]);
const containers = new Set([
  "Array",
  "ReadonlyArray",
  "Promise",
  "Readonly",
  "Partial",
  "Required",
  "Pick",
  "Omit",
  "Record",
  "Map",
  "Set",
  "WeakMap",
  "WeakSet",
  "Iterable",
  "Iterator",
  "Generator",
]);
const genericErrors = new Set([
  "Error",
  "TypeError",
  "RangeError",
  "SyntaxError",
  "AggregateError",
]);

class AnnotationProbe {
  constructor(
    readonly scope: TypeScope,
    readonly modules: SourceModules,
  ) {}
  inspect(node: SyntaxNode): TypeLeaves {
    return this.visit(node, this.scope, new TypeBindings());
  }
  private visit(
    node: SyntaxNode,
    scope: TypeScope,
    bindings: TypeBindings,
  ): TypeLeaves {
    if (scalarKinds.has(node.type))
      return new TypeLeaves([new TypeLeaf(InterfaceRule.Raw, node.type)]);
    if (
      [
        "TSAnyKeyword",
        "TSUnknownKeyword",
        "TSTypeQuery",
        "TSInferType",
        "TSImportType",
        "TSObjectKeyword",
        "TSIntrinsicKeyword",
        "AnyTypeAnnotation",
        "MixedTypeAnnotation",
      ].includes(node.type)
    ) {
      return new TypeLeaves([
        new TypeLeaf(InterfaceRule.Unresolved, node.type),
      ]);
    }
    if (node.type === "TSTypePredicate") {
      const found = new TypeLeaves([
        new TypeLeaf(InterfaceRule.Raw, "type-predicate:boolean"),
      ]);
      if (node.typeAnnotation)
        found.extend(this.visit(node.typeAnnotation, scope, bindings));
      return found;
    }
    if (node.type === "TSLiteralType") {
      return new TypeLeaves([
        new TypeLeaf(InterfaceRule.Raw, `literal:${node.literal.type}`),
      ]);
    }
    if (node.type === "GenericTypeAnnotation") {
      const found = new TypeLeaves([
        new TypeLeaf(InterfaceRule.Unresolved, "flow-generic-type"),
      ]);
      found.extend(this.arguments(node, scope, bindings));
      return found;
    }
    if (node.type === "TSTypeReference") {
      const namespace =
        node.typeName.type === "TSQualifiedName"
          ? node.typeName.left.name
          : null;
      const name = namespace ?? node.typeName.name;
      if (bindings.values.has(name)) return bindings.values.get(name)!;
      for (
        let current: TypeScope | null = scope;
        current;
        current = current.parent
      ) {
        if (current.parameters.has(name))
          return new TypeLeaves([
            new TypeLeaf(InterfaceRule.Unresolved, `generic-parameter:${name}`),
          ]);
        if (current.nominal.has(name))
          return this.arguments(node, scope, bindings);
        if (current.aliases.has(name)) {
          const alias = current.aliases.get(name)!;
          if (bindings.aliases.has(alias))
            return new TypeLeaves([
              new TypeLeaf(InterfaceRule.Unresolved, `recursive-alias:${name}`),
            ]);
          const nested = new TypeBindings();
          for (const active of bindings.aliases) nested.aliases.add(active);
          for (const active of bindings.imports) nested.imports.add(active);
          nested.aliases.add(alias);
          const supplied =
            node.typeParameters?.params ?? node.typeArguments?.params ?? [];
          for (const [index, parameter] of (
            alias.typeParameters?.params ?? []
          ).entries()) {
            const key = parameter.name.name ?? parameter.name;
            const argument = supplied[index];
            const leaves = argument
              ? this.visit(argument, scope, bindings)
              : parameter.default
                ? this.visit(parameter.default, current, nested)
                : new TypeLeaves([
                    new TypeLeaf(
                      InterfaceRule.Unresolved,
                      `generic-parameter:${key}`,
                    ),
                  ]);
            nested.values.set(key, leaves);
          }
          return this.visit(alias.typeAnnotation, current, nested);
        }
        if (current.imports.has(name)) {
          const imported = current.imports.get(name)!;
          const member = namespace ? node.typeName.right.name : imported.member;
          const target = this.modules.target(imported, current);
          const exported =
            target && member ? this.modules.exported(target, member) : null;
          if (exported && !bindings.imports.has(imported)) {
            bindings.imports.add(imported);
            const found = this.visit(
              {
                ...node,
                typeName: { type: "Identifier", name: exported.name },
              },
              exported.scope,
              bindings,
            );
            bindings.imports.delete(imported);
            return found;
          }
          const found = new TypeLeaves([
            new TypeLeaf(
              InterfaceRule.Unresolved,
              `imported-type:${imported.module}:${member ?? name}`,
            ),
          ]);
          found.extend(this.arguments(node, scope, bindings));
          return found;
        }
      }
      const result = this.arguments(node, scope, bindings);
      if (genericErrors.has(name))
        result.entries.push(new TypeLeaf(InterfaceRule.GenericError, name));
      else if (!containers.has(name))
        result.entries.push(
          new TypeLeaf(
            InterfaceRule.Unresolved,
            `unknown-type:${name ?? node.typeName.type}`,
          ),
        );
      return result;
    }
    const result = new TypeLeaves();
    for (const child of SyntaxChildren.fromNode(node).nodes)
      result.extend(this.visit(child, scope, bindings));
    return result;
  }
  private arguments(
    node: SyntaxNode,
    scope: TypeScope,
    bindings: TypeBindings,
  ): TypeLeaves {
    const result = new TypeLeaves();
    for (const argument of node.typeParameters?.params ??
      node.typeArguments?.params ??
      [])
      result.extend(this.visit(argument, scope, bindings));
    return result;
  }
}

class Findings {
  readonly entries: FindingWire[] = [];
}

const functionKinds = new Set([
  "FunctionDeclaration",
  "FunctionExpression",
  "ArrowFunctionExpression",
  "ObjectMethod",
  "ClassMethod",
  "ClassPrivateMethod",
  "TSDeclareFunction",
  "TSMethodSignature",
  "TSFunctionType",
  "TSConstructorType",
  "TSIndexSignature",
  "FunctionTypeAnnotation",
  "TSCallSignatureDeclaration",
  "TSConstructSignatureDeclaration",
]);

class JavascriptScanner {
  readonly findings = new Findings();
  scope: TypeScope;
  constructor(
    readonly source: SourceUnit,
    readonly modules: SourceModules,
  ) {
    this.scope = modules.scopes.get(source.path)!;
  }
  inspect(): Findings {
    this.visit(this.source.program, new InterfaceItem(), null);
    return this.findings;
  }
  private record(
    node: SyntaxNode,
    item: InterfaceItem,
    leaf: TypeLeaf,
    slot: SyntaxNode,
    owner: SyntaxNode,
  ): void {
    this.findings.entries.push({
      path: this.source.path,
      item: item.parts.join("::"),
      slot:
        typeof slot.name === "string"
          ? slot.name
          : (slot.name?.name ?? slot.type),
      rule: leaf.rule.wireName,
      spelling: leaf.spelling,
      signature: this.source.asSpelling(node),
      line: node.loc?.start.line ?? 1,
      opaque:
        leaf.rule === InterfaceRule.Unresolved
          ? TokenFingerprint.fromNode(owner).digest
          : null,
    });
  }
  private annotation(
    node: SyntaxNode,
    item: InterfaceItem,
    owner: SyntaxNode,
  ): void {
    if (node.type === "TSParameterProperty") node = node.parameter;
    const annotation =
      node.typeAnnotation ??
      (node.type === "AssignmentPattern" ? node.left.typeAnnotation : null);
    const leaves = annotation
      ? new AnnotationProbe(this.scope, this.modules).inspect(annotation)
      : new TypeLeaves([
          new TypeLeaf(InterfaceRule.Unresolved, "unannotated-interface"),
        ]);
    for (const leaf of leaves.entries)
      this.record(annotation ?? node, item, leaf, node, owner);
  }
  private parameterBodies(
    node: SyntaxNode,
    item: InterfaceItem,
    owner: SyntaxNode,
  ): void {
    if (functionKinds.has(node.type)) {
      this.visit(node, item, owner);
      return;
    }
    for (const child of SyntaxChildren.fromNode(node).nodes) {
      if (child !== node.typeAnnotation)
        this.parameterBodies(child, item, owner);
    }
  }
  private visit(
    node: SyntaxNode,
    item: InterfaceItem,
    owner: SyntaxNode | null,
  ): void {
    if (functionKinds.has(node.type)) {
      const current = item.child(node);
      const previous = this.scope;
      // Body-local aliases are invisible at the declaration's signature.
      this.scope = new TypeScope(previous);
      for (const parameter of node.typeParameters?.params ?? [])
        this.scope.parameters.add(parameter.name.name ?? parameter.name);
      for (const parameter of node.params ?? node.parameters ?? [])
        this.annotation(parameter, current, node);
      const returnNode: SyntaxNode = {
        type: "return",
        typeAnnotation:
          node.returnType ??
          (node.type === "TSIndexSignature" ? node.typeAnnotation : undefined),
        loc: node.loc,
      };
      if (node.kind !== "constructor")
        this.annotation(returnNode, current, node);
      for (const parameter of node.params ?? node.parameters ?? [])
        this.parameterBodies(parameter, current, node);
      for (const decorator of node.decorators ?? [])
        this.visit(decorator, current, node);
      if (node.body) this.visit(node.body, current, node);
      this.scope = previous;
      return;
    }
    if (
      [
        "ClassDeclaration",
        "ClassExpression",
        "TSInterfaceDeclaration",
      ].includes(node.type)
    ) {
      const previous = this.scope;
      this.scope = TypeScope.fromBody(node.body, this.scope);
      for (const parameter of node.typeParameters?.params ?? [])
        this.scope.parameters.add(parameter.name.name ?? parameter.name);
      for (const child of SyntaxChildren.fromNode(node).nodes)
        this.visit(child, item.child(node), owner);
      this.scope = previous;
      return;
    }
    if (node.type === "BlockStatement") {
      const previous = this.scope;
      this.scope = TypeScope.fromBody(node, this.scope);
      for (const child of SyntaxChildren.fromNode(node).nodes)
        this.visit(child, item, owner);
      this.scope = previous;
      return;
    }
    if (node.type === "ThrowStatement") {
      const raised = node.argument;
      if (
        raised?.type === "NewExpression" &&
        raised.callee?.type === "Identifier"
      ) {
        const reference: SyntaxNode = {
          type: "TSTypeReference",
          typeName: raised.callee,
        };
        for (const leaf of new AnnotationProbe(
          this.scope,
          this.modules,
        ).inspect(reference).entries) {
          this.record(raised.callee, item, leaf, raised.callee, owner ?? node);
        }
      } else if (["StringLiteral", "TemplateLiteral"].includes(raised?.type)) {
        this.record(
          raised,
          item,
          new TypeLeaf(InterfaceRule.GenericError, "thrown-string"),
          raised,
          owner ?? node,
        );
      } else {
        this.record(
          node,
          item,
          new TypeLeaf(InterfaceRule.Unresolved, "throw-expression"),
          { type: "throw" },
          owner ?? node,
        );
      }
    }
    for (const child of SyntaxChildren.fromNode(node).nodes)
      this.visit(child, item, owner);
  }
}

export function inspectSources(wire: SourceWire[]): Findings {
  const findings = new Findings();
  const modules = new SourceModules(
    wire.map((source) => new SourceUnit(source)),
  );
  for (const source of modules.sources)
    findings.entries.push(
      ...new JavascriptScanner(source, modules).inspect().entries,
    );
  return findings;
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  try {
    const wire: SourceWire[] = JSON.parse(readFileSync(0, "utf8"));
    process.stdout.write(JSON.stringify(inspectSources(wire).entries));
  } catch (cause) {
    console.error(cause);
    process.exitCode = 1;
  }
}
