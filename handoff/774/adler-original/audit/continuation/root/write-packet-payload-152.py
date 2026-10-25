"""Write already authored source comparison notes; no project analysis."""
import json
from pathlib import Path

C = Path(__file__).resolve().parents[1]
destination = C / "root/packet-payload-152.json"
assert not destination.exists()
lengths = {
    "derive.rs": 47, "derived_properties.rs": 22, "design/generated.rs": 47,
    "codec.rs": 111, "evaluation.rs": 59, "evaluation/memo.rs": 106,
    "hash.rs": 34, "lib.rs": 50, "mesh.rs": 145, "validation.rs": 72,
}
roles = []
for name, length in lengths.items():
    for snapshot in ["pr-working", "physical-p16"]:
        roles.append({
            "source_id": snapshot + ":crates/adventuresim-weapon-model/src/" + name,
            "actual_compared_ranges": [[1, length]],
            "complete_scoped_comparison": True,
            "source_evidence": "Original full scoped role verified against actual EOF. Fresh literal complete PR counterpart and actual full empty P16/PR diff personally inspected for each of ten files. Independent current04 same operations comparison within each complete role, not hash-only or peer-verdict credit. Original memo fixtures read only, no execution.",
        })
payload = {
    "packet_number": 152, "source_roles": roles,
    "groups": {
        "design-evaluation": "EvaluatedWeapon and EvaluatedHolder own private design plus derived state; accessors expose immutable design and Copy derived values, so original caller edits cannot stale an accepted evaluation. new evaluates through separate static caches; decode parses transport then uses same new. Actual deriving calls identity plus canonical Construction::new; no bypass inferred from omitting a duplicate validate call. Complete Construction/Recipe admission and f64 result bounds remain separately scoped obligations, not proved by names.",
        "memo-operation": "PhysicalMemo owns mutex VecDeque of full D key and DerivedProperties. Lookups compare full PartialEq design, never user hash only. Constructor runs outside lock, Err inserts nothing; second lock rechecks same key, evicts oldest at exact64 length before append. No path exceeds64 when private entries are respected. Concurrent misses can compute twice but deterministic pure constructor relation is intended; first inserted same design remains coherent under that contract. Construct panic occurs outside lock, though Clone panic during insertion can poison lock. No invented recovery guarantee. Existing successful edit/eviction/failure fixtures actual full106 read only.",
        "physical-derivation": "derive_properties gets Construction physical/frames/source geometry then mesh::derived, whereas rendering generate builds Detail::High model then uses same projection. derive_material_masses narrows each component f64 mass to f32 BEFORE grouping addition, distinct from projecting a total once; preserve exact rounding/order unless intentionally replacing with one canonical checked computation and updating relying quantities. No large finite accepted recipe or ordinary overflow demonstrated until actual manufacturing/geometry bounds are read. Forge quote144 time/capacity expression remains pending, not closed by public DerivedProperties representation.",
        "mesh-computed-results": "native_part converts flat positions/normals by as_chunks3 and ignores any remainder, casts f64 to f32, preserves triangle indices then computes bounds. Accepted canonical flat-array divisibility, finite renderer representability, index membership and matching position/normal counts must come from actual ModelPart producer or explicit native boundary admission; these functions alone do not prove them. bounds empty input yields +/-Inf sentinels. derived scans longitudinalY min/max, head-role IDs and optional spear working-section frames, then narrows physical scalars to f32; missing head yields0 intentionally. Identity normalization occurs in Construction separately, so matching original component IDs against resolved source IDs is an existing connected identity graph obligation, not newly proved bad design here. No arbitrary empty ordinary construction/OOB claim.",
        "meaning-and-ownership": "DerivedProperties public Default/Serde raw record contains masskg, lengthsmetres, signedCOM, inertialkgm2 and dimensionlessbalance. SignedCOM may be negative; headlength0 can intentionally mean no head; balance is not automatically UnitInterval. GeneratedWeapon/Holder expose designhash, mesh parts, bounds, anchors and derived copy as mutable/Serde records, not cryptographic authenticated-state tokens. Their correlation and frame invariants need a canonical accepted product where relying code requires them, not wrapper-only f32 replacements. Holder grip explicitly shares recipe-local frame. Final native scalar/vector extraction is legitimate at renderer/physics ports.",
        "codec-and-errors": "decode checks128KiB input envelope before JSON, rejects unknown envelopefields and mismatched schema/generatoru16 versions, then canonical validate. parse-only internal functions intentionally defer physical construction to Evaluated new; encode validates before serialize, internal evaluated encode avoids repeating construction. No compatibility aliases/fallback/oldversions added. Encode itself does not enforce output bytecap after serialization; actual recipe field/structure limits must prove transportclosure before declaring encode/decode asymmetry on accepted designs. CodecError retains serde_json source and structured version/error vector. Construction(String), HolderConstruction.to_string and holder validation stringify native causes before outer wrapper; preserve actionable native cause at original producer, not merely later contextual text. Existing cause-propagation repair graph, no new unique count.",
        "identity-and-quantity": "identity requires nonemptycatalog ID<=128bytes and recipe.validate, not exact catalog membership or current authority. Holder identity checks recommendedkind, exact scabbard/weapon_loop key, bounded physical parameters and required source grip/blade shape. These relational gates cannot be replaced by a generic positive number alone. Hash uses distinct null-terminated design/holder namespaces, fixed little-endian schema+generator versions, then serialized whole design. Digest.finalize.into into fixed32bytes is legitimate native conversion. Public DesignHash32byte value is stable identity storage, not proof matching arbitrary supplied generated geometry; no unframed pair collision shown.",
        "scope-and-limits": "Twenty distinct exact current04 roles533->553, findings60 unchanged. Imports/reexports/lib declarations never clear child source bodies; all other weapon model, recipe/Construction/holder/native dependency proofs remain pending their original scopes. All230/226/224 and prior qualifications retained. No source edits, new tests/helpers/assertions, project execution or other-lens clearance. All thirteen initial audits then handover163; implementation deferred."
    },
    "findings": []
}
destination.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"payload": str(destination), "authored_source_roles": len(roles)}))
