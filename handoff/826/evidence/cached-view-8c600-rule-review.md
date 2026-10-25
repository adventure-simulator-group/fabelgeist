# Independent cached-view command Rule Review8c600

Reviewed exact8c600f727d1eecf21dd241f02739b825edef5000,
tree1c141f5f1a28cb532cb51b472ba7e8fcc01486ae, against47f37. Both changed files
were read completely, including the existing fixture/From adapters, plus
complete actual2d views and unchanged Blitter. Canonical context/error/selection
relationships were revalidated against exact unchanged source. No new owned Rule
finding was identified in this bounded scope.

Texture2d.blit admits both cached views before actual Device checks, then
observes complete unchanged Blitter construction, encoder and render recording.
TextureView.clear admits missing view before wrong Device and observes
encoder/pass creation. Both use the existing separate finish/submit scopes
afterward, preserving the actual native error source with no added poll or
completion promise. Browser and borrowed policy remains device-handled.

Texture2d.clear_raw reuses its existing as_view/From adapter and Viewclear
operation. From copies the cached Arc, Device and metadata without requiring an
allocation. Clear/blit therefore retain cached-view-without-allocation behavior.
Old2d integer scale arms all equal255.0 and match the View arithmetic/order;
depth exactzeroRGB selection and f32-to-f64 payload remain the same. The
intentional label change is2dclear using existing Viewclear labels. No new
API/helper, compatibility path, owner or duplicate backend was introduced.

The existing2d fixture suffix and whole Blitter/shader source are byteidentical
parent. Production meaningful source changes427->359 in2d and259->268 in View;
both below500, Viewclear77 meaningfullines, all modified functions below100. No
baseline was raised; authoritative hosted reductions/checks remain required. No
tests/cases/helpers/assertions/doctests were authored or executed.

No source edits or project checks/build/lint/tests/scanners/generators/runtime,
push or CI occurred. Canonical formatResult consumers and full
producer/caller/Creator/physical/source/error/lock integration remain incomplete
and required. Global13 waves, final independent3reviews and required hosted
gates on the same final pushed SHA remain open; this is no compilation or
whole-PR clearance.
