# Dress construction types and failures

Preview materials and exported surface maps share
`surface_alpha::SurfaceAlpha`. `TextileMaps` carries this enum directly;
`SurfaceTextures` retains it when packaging the maps. Preview and export reuse
the shared alpha owner's cutout threshold. Export formats the enum as glTF alpha
modes at its serialization boundary.

Skin transfer returns `skin_transfer::SkinTransferError` through the garment,
cloth-source and panel-skin producers. Callers can distinguish malformed native
arrays, invalid queries, missing landmarks and unusable influences without
matching display text. Rig lookup and packed-skin failures retain their original
typed causes.

Sewn charts, skirt joins, panel normals and closure construction have separate
error owners. Opening sampling and fastener geometry share
`cloth_closure_error::ClothClosureError`; lacing support returns the shared
`Metres` unit before its consumer enters native geometry arithmetic. Panel
normal failures retain the affected vertex or triangle.

Skirt joining classifies its own admission failures and retains causes from
surface cutting, mesh partitioning and chart normalization. Those existing
dependencies and the wider equipment orchestration still use generic errors;
this increment does not complete the codebase-wide migration in issue #726.

Positions, face ordinals, packed skin and barycentric arrays remain native
representations at the documented numerical and GPU boundaries. Body positions
use metres in the canonical unposed frame with Y up. Error classification and
alpha transport do not change fitting equations, operation order, mesh layouts
or serialized garment designs.
