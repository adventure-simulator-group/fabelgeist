# P06 preliminary entry-point survey

This is an entry-point note, not the complete producer/consumer plan or
implementation. P04 exact-head acceptance remains the active prerequisite.

Verified on 82528917d: SupportElevation is defined once in Tactical Core
city_layout/grounding.rs. It is a finite signed scene support height, with
checked constructor and Serde. ScenePlanPoint is defined in the same crate's
scene_coordinates.rs. Preserve those owners rather than inventing equivalents.

SceneTerrain::height_at and coarse_height_at call surface_at_stride and erase
SurfaceHit.elevation to f32. The detailed sampler retains shared triangle/owned
surface behavior. The sampled and owned geometry paths must both stay covered.

IK terrain_callbacks in foot_clearance.rs currently accept
Fn(Vec2) -> Option<f32>; checked_world_height reconstructs a checked length.
Actual support elevation, absolute ankle target elevation, signed clearance
and relative offsets need separate inspection before changing signatures.
Normal sampling and coarse-detail geometry consumers must be accounted for;
a search limited to the foot-clearance helper would be insufficient.

The next complete survey must include all SceneTerrain query consumers across
Core scene generation/grounding, tactical client, viewers and tactical server,
plus callback signatures, retained fields and existing fixtures. Similar-named
profile and native triangle height_at functions are separate kernels; do not
blindly replace search hits. Existing numeric native triangle boundaries and
LOD/multilevel support selection must retain their operation order and datum.
No P06 source was edited or marked complete by this entry survey.
