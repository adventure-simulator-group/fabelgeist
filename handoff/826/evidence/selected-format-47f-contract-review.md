# Selected sampled-format contract

Root `47f37a07687bc47347cedb9e274c2f273f9deae6`; canonical Advect `e2162708fcf3885193db434300ed6c2999ab7607`.

No effective-format getter needed for canonical from_resource -> default sampled
whole Texture2d/Texture3d route. Advect2D has a concrete second-decode and
quantity-controlled velocity defect in both inspected source pins; repair
remains #780-owned.

Texture2d::default / Texture3d::default: No allocation or cached view, zero
extent, default nominal format and empty usage. A descriptor can describe the
nominal format but does not certify a usable view; later missing-state error
remains authoritative.

Texture2d::new -> create; Texture3d::new: Each nonempty allocation owner fixes
usage once. STORAGE_BINDING requires supports_storage && !is_srgb. Allocation
format is linear_counterpart, but cached view explicitly requests nominal
format. Therefore nominal sRGB implies a nominal sRGB sampled view and no
storage usage; nominal non-sRGB cached view is likewise nominal.

Texture2d image factories: from_image chooses a format then create_from_image.
Empty image returns Default; nonempty routes exclusively through create.
Image::packed_for explicitly does not apply a transfer function, preserves
eight-bit encoding (with BGRA reorder), and documents native sRGB conversion. No
alternate Texture2d construction.

Texture2d color factories and arithmetic output factories:
from_color/create_from_color and add_raw/mix_raw all allocate through create and
then perform operations; no alternate cached-format writer. Color clear/value
semantics are separate from the admitted view identity.

Clone, GpuResource conversion, mutable/native ports: Clone copies exact
immutable metadata/Arc identity; GpuResource carries the whole value.
clear_cached_view removes view; clear_native_texture removes allocation while
leaving cached view; neither mutates nominal format/usage. native/cached getters
expose immutable Arc references, not replacement of wrapper state. Native
destroy/device loss can invalidate usability, not alter admitted format
interpretation.

Explicit view_with_format: Returns another Arc view without replacing cached
view or mutating metadata. On a linear allocation carrying storage usage, an
explicit sRGB request may normalize to linear, but this does not change
subsequent default sampling or descriptor-from-whole-resource identity.

Descriptor producer and binding consumer: from_resource takes GpuResource, which
has only Buffer/whole Texture2d/whole Texture3d, never arbitrary TextureView.
Sampled Naga image reflection leaves binding.format=None. Selection therefore
requests texture.format(); cached nominal view is selected; returned native
dimension/device are checked before use. Storage bindings instead carry
Some(actual storage format) and can deliberately select a counterpart; this
separate path must not be described as default sampling.

## Advect correctness finding

Canonical execute receives a usable whole nominal-sRGB quantity Texture2d and a
plain floating-point velocity Texture2d, with a native-valid floating output.
The actual sampled quantity view is sRGB. Native loads already decode RGB into
linear values, but sample_bilinear then applies another sRGB->linear transfer.
That same quantity-controlled branch also transforms velocity when velocity is
linear. Even dt==0 retains the quantity double decode; for nonzero dt the
quantity encoding additionally changes the advection trajectory through
velocity. This is source reasoning, not an authored/executed test case.

wgpu-types29.0.4 texture/format.rs183,193 states Rgba8UnormSrgb/Bgra8UnormSrgb
are converted from sRGB bytes to linear-color float in shader. Actual
cached/sample view contract and Image::packed_for prevent an unstated raw-byte
interpretation here. Advect shared sample_bilinear(texture_2d<f32>) calls
textureLoad, bilinearly mixes the fetched values, then conditionally applies the
transfer using only the quantity descriptor; both velocity and quantity call it.
The 3D branch directly textureLoads both resources and has no additional
transform.

#780 should remove or otherwise correct the second-decode/quantity-flag contract
at the complete Advect generated shader owner, preserving native sampled-view
semantics and existing interpolation/coordinate/dispatch/zero/format errors. No
extra effective-format getter or alternative whole-resource descriptor owner is
justified by this source. If a distinct raw-encoded source policy is intended,
it requires an explicit actual producer/binding policy rather than applying the
quantity flag to both resources.

Carry existing TextureFormat from the canonical whole-resource producer. For a
successfully bound default sampled whole texture, it identifies the selected
view interpretation; no primitive string reclassification or extra stored flag
is needed.

Do not globally rename nominal allocation metadata to universal effective
format: allocation can be linear while its cached sampled view is sRGB; explicit
alternate views and storage bindings have different selection contracts.

ResourceDescriptor variants and public build_pipeline APIs remain manually
constructible. The proof covers descriptors actually produced from canonical
whole resources, not a manually mismatched descriptor and pass parameter. Native
layout/device/binding checks remain authoritative; a getter would not brand a
manual descriptor to a resource.

Default/missing/cleared/error-handle states do not have a successfully usable
sampled view merely because format() returns a value. Preserve existing
MissingView/Uninitialized/WrongDevice and provider-handled validation policy.

No admitted nominal-sRGB plus STORAGE_BINDING whole texture was found. A future
constructor/mutator changing that relationship must update the
producer/selected-view contract then; inventing a hypothetical admission route
is not a reason to add an API now.

Read-only pinned source/Git and SDK documentation only; no source edits, local
project checks/builds/scanners/tests/runtime/generators or new
tests/cases/helpers/assertions.

Source searches are constrained Git provenance/admission inventories, not
executed project scanners. Unsafe memory fabrication is outside admitted safe
routes.

Native format behavior is supported by SDK source, not a runtime/GPU
reproduction. No global GPU validity, browser synchronous validation or post-
destroy usability guarantee.

Root47f is deliberately unintegrated with required canonical descriptor/caller
migration. This contract does not imply compilation or final thirteen-
lens/wholePR/hosted clearance.

Exact file hashes, source search results, native evidence, all admission routes
and owner-limited acceptance criteria are recorded in the accompanying JSON.
