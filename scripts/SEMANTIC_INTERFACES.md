# Script and browser semantic interfaces

Handwritten interfaces must express domain meaning through their parameter
and return types, including private helpers and test helpers. Reuse the
owning domain's types; aliases and wrappers named after storage types do not
establish a semantic boundary. Raw constructor inputs and externally mandated
adapters still need review. Convert immediately before entering application
code. Errors must retain domain context and underlying causes.

## Census and enforcement

Install Node.js 24, Python 3.12 or later, and the pinned repository dependencies
with `npm ci --ignore-scripts`. Run these commands from the repository root:

```sh
python3 -B -m scripts.check_semantic_interfaces census
python3 -B -m scripts.check_semantic_interfaces check
python3 -B -m unittest scripts.test_semantic_interfaces scripts.test_wgsl_interfaces
```

The census reads tracked and untracked, nonignored sources without executing
them. It scans Python ASTs, JavaScript and TypeScript ASTs, and inline
JavaScript in HTML files and literal `wasm_bindgen(inline_js = ...)` attributes
throughout the repository. Inline Rust bridges receive stable
`#inline-js-N.js` source identities and the same JavaScript AST inspection,
including arrow callbacks without a named function marker. Generated SpacetimeDB
bindings are excluded under the repository generation rule. Separate
workspaces, browser code, scripts, and test helpers receive the same treatment.

Python annotations follow lexical aliases, renamed imports, local nominal
types, project module imports, and nested annotations. Literal values remain
distinct from forward annotations and annotation metadata. Default callbacks
and decorator callbacks are scanned too. TypeScript annotations
also follow generic alias arguments and defaults, relative imports,
re-exports, and namespace imports. Unknown and ambiguous resolution remains
explicit debt. Unannotated interfaces carry AST fingerprints so a body change
requires renewed review. Fingerprints exclude source positions and comments.

`script-interface-baseline.json` records exact debt by source, owning item,
interface slot, rule, spelling, signature, and optional fingerprint. Duplicate
findings have occurrence counts. The gate rejects additions, increases,
changed fingerprints, and stale entries after removals. `just lint` and the
repository quality workflow run this gate and its behavioral tests. The
workflow covers every pull request so source outside `crates/` cannot bypass
enforcement through path filters.

After migrating a coherent family, generate a proposed snapshot:

```sh
python3 -B -m scripts.check_semantic_interfaces baseline \
  > target/script-interface-baseline-proposed.json
```

Apply only reductions attributable to that migration. Review any fingerprint
replacement alongside the owning code; a fingerprint update is not a license
to introduce new unresolved interfaces. Do not replace the complete baseline
to accommodate new debt. Constructor, conversion, and external adapter
findings remain visible until their narrow contracts have been reviewed.

## Limits and outstanding review

This syntax census cannot prove semantic ownership or infer dynamic values.
External library types, Python runtime alias factories, ambiguous module
resolution, inferred signatures, and arbitrary thrown expressions remain
unresolved. A nominal declaration alone does not prove a domain invariant.
Generic error constructors and annotations are recorded; deciding whether
an external error belongs at an adapter requires inspection of its consumer.

Native C/C++, shell, PowerShell, Nix, shader, workflow, and build recipe sources
currently produce opaque entries with complete source fingerprints.
Rust strings containing `fn` or `function` declarations are included when a
shader fragment defines only scalar helpers and has no entry point or device
storage. Discovery includes ordinary strings, formatted function headers, raw
strings, and byte/C string prefixes. The lexical adapter skips line comments,
nested block comments, and character literals, retains raw-string delimiters,
and decodes ordinary Rust escapes, including Unicode, hex, and continued lines.
It does not parse Rust expressions or assemble source split across literals.
Such composition still requires source review and assembled-program analysis.
The lexical context recognizes direct and qualified `wasm_bindgen` attributes,
skips comments and quoted attribute prose, and masks earlier literals while
classifying subsequent attributes. Decoded complete string literals are parsed
as JavaScript. Unknown attribute aliases, unsupported byte/C values, and
unterminated values retain opaque coverage. Mixed files retain opaque debt for
other embedded fragments alongside their parsed bridge interfaces.

WGSL receives a signature inventory in standalone files and discovered Rust
fragments. The scanner retains nested comments, parameter/result attributes,
arrays, vectors, matrices, atomics and pointers. It follows scalar aliases,
records alias declarations separately, and recognizes declared structures as
nominal syntax. Conflicting declarations and alias cycles stay unresolved.
Scalar shorthand types remain primitive interfaces. Device entry points receive
the same findings as helpers; an externally mandated built-in parameter is not
an automatic exception for a handwritten helper.

Fragments receive stable `#wgsl-N.wgsl` identities. Only shader candidates
affect that ordinal. Structure and alias declarations can inform prototypes in the
same Rust host, while conflicting definitions remain ambiguous. This is
conservative lexical signature inspection, not a compiler or an assembly proof.
Unknown types, incomplete or templated function names, unsupported type
arguments and missing annotations remain fingerprinted unresolved findings. Bodies and
composed source still need analysis. Declaration references such as `fn helper`
or `struct Owner {` without a complete prototype or definition remain data;
entry-point/storage markers and full prototypes retain opaque discovery.

The scanner retains each host's opaque fingerprint alongside parsed findings.
Nominal-looking headers alone cannot remove source-assembly or implementation
review. Declared structures do not prove units, invariants or semantic
operations; they require inspection of their producers, consumers and storage adapters.
The parser retains nominal tokens, spellings, delimiters, source lines,
parameters and signatures through its internal operations. `ShaderLine` rejects
nonpositive coordinates with `ShaderLineAdmissionFailure`. Its exact integer
conversion and `ShaderSpelling`'s exact string conversion implement Python's
representation protocols for the finding wire record. The admission error's
constructor receives the rejected SDK integer and retains it privately for
its diagnostic. These three narrow representation/admission interfaces are
boundary findings, not ordinary primitive parser helpers.

The new shader findings must be migrated or precisely justified, never added to
the debt baseline to make the gate pass. WGSL's
[function declarations](https://www.w3.org/TR/WGSL/#function-declaration),
[type specifiers](https://www.w3.org/TR/WGSL/#type-specifier), and
[aliases](https://www.w3.org/TR/WGSL/#type-aliases) provide the grammar
reference.

Other embedded browser or shader markers in literals, inline HTML event
handlers, and unterminated script blocks also remain opaque. These entries
enforce review of source changes; they do not claim that all their signatures
have been parsed or justified. Extending language coverage and resolving these
entries is part of the remaining migration, not a completion exception.

The Rust census and its independent debt baseline are documented in
[the shared semantic type guide](../crates/SEMANTIC_TYPES.md). Neither census
baseline proves that the repository migration is complete. Completion requires
reviewed semantic interfaces, narrowly justified external exceptions, and
passing applicable behavioral and repository checks.
