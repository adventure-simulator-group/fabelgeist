"""Behavioral coverage for the language census and exact debt ratchet."""

from pathlib import Path
import json
import subprocess
import tempfile
import unittest

from scripts.semantic_interfaces.gate import (
    BaselineDecodeFailure, BaselineFormatFailure, BaselineViolation,
    GateReport, InterfaceDebt,
)
from scripts.semantic_interfaces.model import (
    Findings, InterfaceRule, SourceParseError, SourceUnit,
)
from scripts.semantic_interfaces.python import PythonModules, PythonScanner
from scripts.semantic_interfaces.repository import (
    InlineJavascript, JavascriptInputs, SourceUnits, census, javascript_census,
)


ROOT = Path(__file__).resolve().parents[1]


class PythonInterfacesTests(unittest.TestCase):
    def test_cross_file_imports_reexports_and_relative_aliases_are_resolved(self) -> None:
        sources = [
            SourceUnit(Path("pkg/ids.py"), "Count = int\n"),
            SourceUnit(Path("pkg/exported.py"), "from .ids import Count as Total\n"),
            SourceUnit(Path("pkg/consumer.py"), "from .exported import Total as Number\nimport pkg.ids as ids\ndef sample(value: Number, other: ids.Count) -> None: pass\n"),
        ]
        found = PythonScanner(sources[-1], PythonModules(sources)).inspect()
        self.assertEqual([entry.leaf.spelling for entry in found.entries], ["int", "int"])

    def test_nested_aliases_and_renamed_builtins_cannot_hide_scalars(self) -> None:
        source = SourceUnit(Path("sample.py"), """
from builtins import int as Number
Count = Number
Batch = list[tuple[Count, float]]
def sample(value: Batch) -> str: return "ok"
""")
        found = PythonScanner(source).inspect()
        self.assertEqual([entry.leaf.spelling for entry in found.entries], ["int", "float", "str"])

    def test_renamed_builtin_namespaces_and_imported_packages_are_distinct(self) -> None:
        sources = [
            SourceUnit(Path("pkg/__init__.py"), "Count = bool\n"),
            SourceUnit(Path("pkg/ids.py"), "Count = int\n"),
            SourceUnit(Path("consumer.py"), "import builtins as native\nimport pkg.ids\ndef sample(value: native.str, other: pkg.Count) -> None: pass\n"),
        ]
        found = PythonScanner(sources[-1], PythonModules(sources)).inspect()
        self.assertEqual([row.leaf.spelling for row in found.entries], ["str", "bool"])

    def test_literal_values_cannot_masquerade_as_forward_nominal_annotations(self) -> None:
        source = SourceUnit(Path("sample.py"), "from typing import Literal as State, Annotated\nclass Owned: pass\ndef sample(value: State['Owned', True], other: Annotated[Owned, 'metadata']) -> None: pass\n")
        found = PythonScanner(source).inspect()
        self.assertEqual([row.leaf.spelling for row in found.entries], ["literal-annotation", "literal-annotation"])

    def test_callbacks_inside_annotation_metadata_are_audited(self) -> None:
        source = SourceUnit(Path("sample.py"), "from typing import Annotated\nclass Owned: pass\ndef sample(value: Annotated[Owned, lambda other: other]) -> None: pass\n")
        found = PythonScanner(source).inspect()
        self.assertEqual(len(found.entries), 2)
        self.assertTrue(all(row.item.parts == ("sample", "lambda") for row in found.entries))

    def test_nominal_shadowing_and_implicit_method_receiver_are_distinct(self) -> None:
        source = SourceUnit(Path("sample.py"), """
class int: pass
class Owned:
    def method(self, value: int) -> int:
        def helper(self: float) -> None: pass
""")
        found = PythonScanner(source).inspect()
        self.assertEqual(len(found.entries), 1)
        self.assertEqual(found.entries[0].item.parts, ("Owned", "method", "helper"))
        self.assertEqual(found.entries[0].leaf.spelling, "float")

    def test_untyped_lambdas_and_helper_body_changes_remain_visible(self) -> None:
        previous = PythonScanner(SourceUnit(Path("sample.py"), "def sample(value): return lambda other: other")).inspect()
        changed = PythonScanner(SourceUnit(Path("sample.py"), "def sample(value): return lambda other: other + 1")).inspect()
        self.assertEqual(len(previous.entries), 4)
        self.assertTrue(all(row.leaf.rule is InterfaceRule.UNRESOLVED for row in previous.entries))
        self.assertNotEqual(previous.as_wire_counts(), changed.as_wire_counts())

    def test_callbacks_in_defaults_and_decorators_are_not_skipped(self) -> None:
        source = SourceUnit(Path("sample.py"), "@decorate(lambda value: value)\ndef sample(callback=lambda other: other) -> None: pass\n")
        found = PythonScanner(source).inspect()
        self.assertEqual(len(found.entries), 5)
        self.assertEqual(sum(row.item.parts == ("sample", "lambda") for row in found.entries), 4)

    def test_static_and_explicitly_annotated_receivers_are_audited(self) -> None:
        source = SourceUnit(Path("sample.py"), "@decorate(lambda self: self)\nclass Owned:\n @staticmethod\n def helper(self: int) -> None: pass\n def explicit(self: float) -> None: pass\n")
        found = PythonScanner(source).inspect()
        self.assertEqual(sum(row.leaf.rule is InterfaceRule.UNRESOLVED for row in found.entries), 2)
        self.assertEqual([row.leaf.spelling for row in found.entries if row.leaf.rule is InterfaceRule.RAW], ["int", "float"])

    def test_recursive_alias_and_generic_exception_paths_are_not_silent(self) -> None:
        source = SourceUnit(Path("sample.py"), "Loop = list[Loop]\ndef sample(value: Loop) -> None:\n raise RuntimeError('broken')\n")
        found = PythonScanner(source).inspect()
        self.assertEqual(len(found.entries), 2)
        self.assertEqual(found.entries[0].leaf.rule, InterfaceRule.UNRESOLVED)
        self.assertEqual(found.entries[1].leaf.rule, InterfaceRule.GENERIC_ERROR)

    def test_parse_failure_preserves_the_original_syntax_error(self) -> None:
        source = SourceUnit(Path("broken.py"), "def broken(")
        with self.assertRaises(SourceParseError) as caught:
            PythonScanner(source)
        self.assertIsInstance(caught.exception.__cause__, SyntaxError)

    def test_renamed_generic_errors_and_unresolved_rethrows_are_visible(self) -> None:
        source = SourceUnit(Path("sample.py"), "from builtins import ValueError as Failure\ndef sample() -> None:\n raise Failure('bad')\n raise unknown\n")
        found = PythonScanner(source).inspect()
        self.assertEqual([row.leaf.rule for row in found.entries], [InterfaceRule.GENERIC_ERROR, InterfaceRule.UNRESOLVED])


class JavascriptInterfacesTests(unittest.TestCase):
    def test_relative_imports_reexports_namespace_and_generic_defaults(self) -> None:
        inputs = JavascriptInputs([
            SourceUnit(Path("pkg/ids.ts"), "export type Count<T = number> = T[];"),
            SourceUnit(Path("pkg/exported.ts"), "export { Count as Total } from './ids.js';"),
            SourceUnit(Path("pkg/consumer.ts"), "import {Total as Number} from './exported'; import * as ids from './ids'; function sample(value: Number, other: ids.Count<string>): void {}"),
        ])
        found = javascript_census(inputs, ROOT)
        self.assertEqual(sorted(row.leaf.spelling for row in found.entries), ["TSNumberKeyword", "TSStringKeyword"])

    def test_class_generics_literal_aliases_and_constructor_parameter_properties(self) -> None:
        inputs = JavascriptInputs([SourceUnit(Path("sample.ts"), "type T = number; type State = 'open' | false; class Owned<T> { constructor(readonly value: State) {} sample(value: T): void {} }")])
        found = javascript_census(inputs, ROOT)
        self.assertEqual(sum(row.leaf.rule is InterfaceRule.RAW for row in found.entries), 2)
        self.assertEqual([row.leaf.spelling for row in found.entries if row.leaf.rule is InterfaceRule.UNRESOLVED], ["generic-parameter:T"])

    def test_block_aliases_and_interface_generics_shadow_enclosing_types(self) -> None:
        inputs = JavascriptInputs([SourceUnit(Path("sample.ts"), "class Count {} type T = number; { type Count = boolean; function sample(value: Count): void {} } interface Owned<T> {sample(value: T): void;}")])
        found = javascript_census(inputs, ROOT)
        self.assertEqual([row.leaf.spelling for row in found.entries], ["TSBooleanKeyword", "generic-parameter:T"])

    def test_body_aliases_cannot_shadow_the_enclosing_signature(self) -> None:
        inputs = JavascriptInputs([SourceUnit(Path("sample.ts"), "type Count = number; function sample(value: Count): void {class Count {}}")])
        found = javascript_census(inputs, ROOT)
        self.assertEqual([row.leaf.spelling for row in found.entries], ["TSNumberKeyword"])

    def test_callback_defaults_are_scanned_beyond_the_outer_signature(self) -> None:
        inputs = JavascriptInputs([SourceUnit(Path("sample.js"), "function sample(callback = value => value) {}")])
        found = javascript_census(inputs, ROOT)
        self.assertEqual(len(found.entries), 4)
        self.assertEqual(sum(row.item.parts == ("sample", "closure") for row in found.entries), 2)

    def test_constructor_and_index_signatures_are_audited(self) -> None:
        inputs = JavascriptInputs([SourceUnit(Path("sample.ts"), "class Owned {} type Maker = new (value: number) => Owned; interface Lookup { [key: string]: boolean; }")])
        found = javascript_census(inputs, ROOT)
        self.assertEqual(sorted(row.leaf.spelling for row in found.entries), ["TSBooleanKeyword", "TSNumberKeyword", "TSStringKeyword"])

    def test_generic_objects_and_boolean_type_guards_remain_visible(self) -> None:
        inputs = JavascriptInputs([SourceUnit(Path("sample.ts"), "class Owned {} function sample(value: object): value is Owned {return true;}")])
        found = javascript_census(inputs, ROOT)
        self.assertEqual([row.leaf.spelling for row in found.entries], ["TSObjectKeyword", "type-predicate:boolean"])

    def test_bigint_fingerprints_ignore_locations_but_track_value_changes(self) -> None:
        previous = javascript_census(JavascriptInputs([SourceUnit(Path("sample.js"), "function sample() {return 1n;}")]), ROOT)
        formatted = javascript_census(JavascriptInputs([SourceUnit(Path("sample.js"), "\nfunction sample() { return 1n; }\n")]), ROOT)
        changed = javascript_census(JavascriptInputs([SourceUnit(Path("sample.js"), "function sample() {return 2n;}")]), ROOT)
        self.assertEqual(previous.as_wire_counts(), formatted.as_wire_counts())
        self.assertNotEqual(previous.as_wire_counts(), changed.as_wire_counts())

    def test_generic_alias_defaults_and_nested_callback_types_are_expanded(self) -> None:
        inputs = JavascriptInputs([SourceUnit(Path("sample.ts"), """
type Batch<T = number> = T[];
function sample(value: Batch, callback: (value: bigint) => boolean): void {}
""")])
        found = javascript_census(inputs, ROOT)
        self.assertEqual(sorted(row.leaf.spelling for row in found.entries), ["TSBigIntKeyword", "TSBooleanKeyword", "TSNumberKeyword"])

    def test_untyped_arrows_and_generic_errors_remain_explicit_debt(self) -> None:
        inputs = JavascriptInputs([SourceUnit(Path("sample.js"), "const fn = value => value; function fail() {throw new Error('broken');}")])
        found = javascript_census(inputs, ROOT)
        self.assertEqual(sum(row.leaf.rule is InterfaceRule.GENERIC_ERROR for row in found.entries), 1)
        self.assertEqual(sum(row.leaf.rule is InterfaceRule.UNRESOLVED for row in found.entries), 3)

    def test_nominal_types_are_distinct_from_primitive_aliases(self) -> None:
        inputs = JavascriptInputs([SourceUnit(Path("sample.ts"), "class Owned {} type Count = number; function sample(value: Owned, count: Count): Owned {}")])
        found = javascript_census(inputs, ROOT)
        self.assertEqual(len(found.entries), 1)
        self.assertEqual(found.entries[0].leaf.rule, InterfaceRule.RAW)

    def test_inline_scripts_are_scanned_and_json_payloads_are_not_code(self) -> None:
        source = SourceUnit(Path("page.html"), '<script type="application/json">{"count":1}</script><script>function sample(value) { return value; }</script>')
        parser = InlineJavascript(source)
        parser.feed(source.text)
        self.assertEqual(len(parser.scripts.entries), 1)
        found = javascript_census(parser.scripts, ROOT)
        self.assertEqual(len(found.entries), 2)

    def test_unterminated_inline_scripts_remain_explicit_opaque_debt(self) -> None:
        source = SourceUnit(Path("page.html"), '<script>function sample(value) { return value; }')
        parser = InlineJavascript(source)
        parser.feed(source.text)
        parser.close()
        self.assertEqual([row.leaf.rule for row in parser.events.entries], [InterfaceRule.OPAQUE])

    def test_embedded_compute_shaders_remain_visible_without_a_shader_parser(self) -> None:
        sources = SourceUnits([SourceUnit(Path("kernel.rs"), 'const CODE: &str = r#"@compute @workgroup_size(1) fn main() {}"#;')])
        found = census(sources, ROOT)
        self.assertEqual([row.leaf.rule for row in found.entries], [InterfaceRule.OPAQUE])

    def test_wasm_inline_javascript_interfaces_are_parsed_at_the_actual_attribute(self) -> None:
        sources = SourceUnits([SourceUnit(Path("bridge.rs"), '''
#[wasm_bindgen(inline_js = r#"export function select(value) { return value; }"#)]
extern "C" {}
''')])
        found = census(sources, ROOT)
        self.assertEqual(len(found.entries), 2)
        self.assertTrue(all(row.path == Path("bridge.rs#inline-js-1.js") for row in found.entries))
        self.assertTrue(all(row.item.parts == ("select",) for row in found.entries))
        self.assertTrue(all(row.leaf.rule is InterfaceRule.UNRESOLVED for row in found.entries))

    def test_inline_javascript_arrow_callbacks_do_not_need_a_function_marker(self) -> None:
        sources = SourceUnits([SourceUnit(Path("bridge.rs"), '''
#[wasm_bindgen::wasm_bindgen(inline_js = r"export const select = value => value;")]
extern "C" {}
''')])
        found = census(sources, ROOT)
        self.assertEqual(len(found.entries), 2)
        self.assertTrue(all(row.path == Path("bridge.rs#inline-js-1.js") for row in found.entries))
        self.assertFalse(any(row.leaf.rule is InterfaceRule.OPAQUE for row in found.entries))

    def test_inline_javascript_decodes_escapes_and_preserves_other_embedded_debt(self) -> None:
        sources = SourceUnits([SourceUnit(Path("mixed.rs"), r'''
const SHADER: &str = r"fn shade(x: f32) -> f32 { return x; }";
#[wasm_bindgen(js_name = "fixture)name", /* annotation */ inline_js =
    "export function select(value) { return \"chosen\"; }")]
extern "C" {}
''')])
        found = census(sources, ROOT)
        self.assertEqual(sum(row.leaf.rule is InterfaceRule.OPAQUE for row in found.entries), 1)
        self.assertEqual(sum(row.path == Path("mixed.rs#inline-js-1.js") for row in found.entries), 2)

    def test_comments_and_quoted_attribute_prose_cannot_create_inline_javascript(self) -> None:
        sources = SourceUnits([SourceUnit(Path("prose.rs"), r'''
// #[wasm_bindgen(inline_js =
const DESCRIPTION: &str = "an ordinary description";
const ATTRIBUTE_TEXT: &str = "#[wasm_bindgen(inline_js =";
const FUNCTION_TEXT: &str = "function example(x) { return x; }";
''')])
        found = census(sources, ROOT)
        self.assertEqual([row.leaf.rule for row in found.entries], [InterfaceRule.OPAQUE])
        self.assertEqual(found.entries[0].path, Path("prose.rs"))

    def test_inline_javascript_identity_ignores_unrelated_rust_literals(self) -> None:
        bridge = '#[wasm_bindgen(inline_js = r"export function select(value) { return value; }")]'
        previous = census(SourceUnits([SourceUnit(Path("bridge.rs"), bridge)]), ROOT)
        changed = census(SourceUnits([SourceUnit(Path("bridge.rs"), 'const LABEL: &str = "label";\n' + bridge)]), ROOT)
        self.assertEqual(previous.as_wire_counts(), changed.as_wire_counts())

    def test_unresolved_inline_attributes_and_unterminated_values_remain_opaque(self) -> None:
        sources = SourceUnits([
            SourceUnit(Path("alias.rs"), '#[alias(inline_js = r"export const select = value => value;")]'),
            SourceUnit(Path("bytes.rs"), '#[wasm_bindgen(inline_js = br"export const select = value => value;")]'),
            SourceUnit(Path("ordinary-bytes.rs"), '#[wasm_bindgen(inline_js = b"export const select = value => value;")]'),
            SourceUnit(Path("c-string.rs"), '#[wasm_bindgen(inline_js = c"export const select = value => value;")]'),
            SourceUnit(Path("unterminated.rs"), '#[wasm_bindgen(inline_js = r"export const select = value => value;'),
        ])
        found = census(sources, ROOT)
        self.assertEqual({row.path for row in found.entries}, {source.path for source in sources.entries})
        self.assertTrue(all(row.leaf.rule is InterfaceRule.OPAQUE for row in found.entries))

    def test_helper_only_embedded_functions_are_visible_without_entry_points(self) -> None:
        sources = SourceUnits([
            SourceUnit(Path("scalar.rs"), 'const CODE: &str = r"fn scalar(x: f32) -> f32 { return x; }";'),
            SourceUnit(Path("vector.rs"), 'const CODE: &str = r###"fn vector(x: vec2<f32>) -> vec2<f32> { return x; }"###;'),
            SourceUnit(Path("script.rs"), 'const CODE: &str = r#"function helper(x) { return "text"; }"#;'),
            SourceUnit(Path("ordinary.rs"), 'const TEXT: &str = r#"A function description."#; fn ordinary() {}'),
        ])
        found = census(sources, ROOT)
        self.assertEqual({row.path for row in found.entries
                          if row.leaf.rule is InterfaceRule.OPAQUE}, {
            Path("scalar.rs"), Path("vector.rs"), Path("script.rs"),
        })
        raw = [row for row in found.entries if row.leaf.rule is InterfaceRule.RAW]
        self.assertEqual(len(raw), 4)
        self.assertEqual({row.path for row in raw}, {
            Path("scalar.rs#wgsl-1.wgsl"), Path("vector.rs#wgsl-1.wgsl"),
        })

    def test_quoted_function_templates_and_escaped_headers_cannot_evade_discovery(self) -> None:
        sources = SourceUnits([
            SourceUnit(Path("quoted.rs"), 'const CODE: &str = "fn hook() -> u32 { return 0u; }";'),
            SourceUnit(Path("template.rs"), 'let code = format!("fn {name}(x: Scalar) -> Scalar {{ return x; }}");'),
            SourceUnit(Path("escaped.rs"), r'const CODE: &str = "fn\n hook() -> u32 { return 0u; }";'),
            SourceUnit(Path("prose.rs"), 'const TEXT: &str = "A function description."; fn ordinary() {}'),
        ])
        found = census(sources, ROOT)
        self.assertEqual({row.path for row in found.entries
                          if row.leaf.rule is InterfaceRule.OPAQUE}, {
            Path("quoted.rs"), Path("template.rs"), Path("escaped.rs"),
        })
        parsed = [row for row in found.entries
                  if row.leaf.rule is not InterfaceRule.OPAQUE]
        self.assertEqual(len(parsed), 5)
        self.assertEqual(sum(row.leaf.rule is InterfaceRule.RAW
                             for row in parsed), 2)

    def test_comments_characters_and_raw_quotes_preserve_literal_boundaries(self) -> None:
        sources = SourceUnits([
            SourceUnit(Path("ordinary.rs"), r'''
                // "fn commented() {}" @compute
                /* "fn outer() {}" /* "fn inner() {}" */ */
                const QUOTE: char = '"';
                const ESCAPED: char = '\'';
                const PREFIX: &str = "r";
                const TEXT: &str = r###"A "quoted" description."###;
                fn ordinary() { let value = "text"; }
            '''),
            SourceUnit(Path("embedded.rs"), r'''
                const CODE: &str = r###"function helper() { return "text"; }"###;
                fn ordinary() { let value = "text"; }
            '''),
        ])
        found = census(sources, ROOT)
        self.assertEqual([row.path for row in found.entries], [Path("embedded.rs")])

    def test_rust_unicode_hex_continuation_and_raw_prefixes_expose_headers(self) -> None:
        sources = SourceUnits([
            SourceUnit(Path("unicode.rs"), r'const CODE: &str = "\u{66}\x6e hook() {}";'),
            SourceUnit(Path("continued.rs"), 'const CODE: &str = "f\\\n   n hook() {}";'),
            SourceUnit(Path("bytes.rs"), 'const CODE = br##"fn hook() {}"##;'),
            SourceUnit(Path("c.rs"), 'const CODE = cr##"fn hook() {}"##;'),
        ])
        found = census(sources, ROOT)
        self.assertEqual({row.path for row in found.entries}, {
            Path("unicode.rs"), Path("continued.rs"), Path("bytes.rs"), Path("c.rs"),
        })

    def test_reexport_cycles_and_arbitrary_thrown_values_cannot_hide_debt(self) -> None:
        inputs = JavascriptInputs([
            SourceUnit(Path("a.ts"), "export {Count} from './b';"),
            SourceUnit(Path("b.ts"), "export {Count} from './a';"),
            SourceUnit(Path("consumer.ts"), "import {Count} from './a'; function sample(value: Count): void {throw value;}"),
        ])
        found = javascript_census(inputs, ROOT)
        self.assertEqual(len(found.entries), 2)
        self.assertTrue(all(row.leaf.rule is InterfaceRule.UNRESOLVED for row in found.entries))


class InterfaceRatchetTests(unittest.TestCase):
    def test_malformed_baselines_fail_with_structured_reasons_and_source_causes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "baseline.json"
            path.write_text("[]")
            with self.assertRaises(BaselineFormatFailure) as caught:
                InterfaceDebt.load(path)
            self.assertEqual(caught.exception.violation, BaselineViolation.OBJECT_REQUIRED)
            key = json.dumps(["sample.py", "sample", "return", "raw-interface", "int", "return", None])
            path.write_text(json.dumps({key: True}))
            with self.assertRaises(BaselineFormatFailure) as caught:
                InterfaceDebt.load(path)
            self.assertEqual(caught.exception.violation, BaselineViolation.INVALID_COUNT)
            path.write_text("{")
            with self.assertRaises(BaselineDecodeFailure) as caught:
                InterfaceDebt.load(path)
            self.assertIsInstance(caught.exception.__cause__, json.JSONDecodeError)

    def test_added_doubled_and_removed_findings_all_require_baseline_changes(self) -> None:
        found = PythonScanner(SourceUnit(Path("sample.py"), "def sample(value: int) -> None: pass")).inspect()
        original = InterfaceDebt.from_findings(found)
        self.assertFalse(GateReport.compare(original, original).mismatches)
        self.assertTrue(GateReport.compare(InterfaceDebt({}), original).mismatches)
        doubled = Findings(found.entries + found.entries)
        self.assertTrue(GateReport.compare(original, InterfaceDebt.from_findings(doubled)).mismatches)
        self.assertTrue(GateReport.compare(original, InterfaceDebt({})).mismatches)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "baseline.json"
            path.write_text(original.as_wire())
            self.assertEqual(InterfaceDebt.load(path), original)


class RepositoryDiscoveryTests(unittest.TestCase):
    def test_untracked_sources_are_included_and_generated_and_ignored_files_are_excluded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "--quiet", str(root)], check=True)
            files = {
                "ordinary.py": "def sample(value: int) -> None: pass\n",
                "script.mjs": "const sample = value => value;",
                "crates/separate/src/lib.rs": "fn sample() {}",
                "crates/adventuresim-stdb-client/src/generated.ts": "export type Count = number;",
                "ignored.py": "def ignored(value): pass\n",
                "flake.nix": "{ inputs = {}; }",
                "scripts/tool.ps1": "param([string]$name)\n",
                ".github/workflows/test.yml": "name: test\n",
                "justfile": "check:\n    true\n",
                ".gitignore": "ignored.py\n",
            }
            for relative, source in files.items():
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source)
            found = SourceUnits.from_repository(root)
            self.assertEqual({str(source.path) for source in found.entries}, {
                "ordinary.py", "script.mjs", "crates/separate/src/lib.rs",
                "flake.nix", "scripts/tool.ps1", ".github/workflows/test.yml", "justfile",
            })
            subprocess.run(["git", "-C", str(root), "add", "ordinary.py"], check=True)
            (root / "ordinary.py").unlink()
            self.assertNotIn(Path("ordinary.py"), {source.path for source in SourceUnits.from_repository(root).entries})


if __name__ == "__main__":
    unittest.main()
