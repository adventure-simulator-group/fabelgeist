# The seven grass modes

One scene, one camera, one placement lattice, seven ways of getting grass onto
the screen. The `Instancing` knob switches between them live and each one owns
its passes, so the overlay's timings belong to the mode that is running and to
nothing else. This is what actually differs between them.

## What is held constant

- **The patch.** The same terrain heightfield, the same 128 m square, the same
  slope and disc gating on every placement.
- **The lattice.** The first three modes place *identical* tufts: same seeds,
  same four distance tiers (`near`, `near_edge`, `far`, `vista` at 12, 12, 4
  and 2 tufts per cell side, 8, 6, 10 and 12 blades per tuft side), same fade
  bands, same crossfade dither. Only the machinery around them changes.
- **The knobs.** Density, range, anti-aliasing, shadows, prepass and foliage
  alpha apply to whichever mode is up, so a number moved by a knob moved for
  the same reason in every mode.

## At a glance

| Mode | Geometry per plant | What is uploaded per frame | What culls it | Draws |
| --- | --- | --- | --- | --- |
| bevy_eidolon | tuft mesh, instanced | nothing | GPU compute, per instance | one indirect per tier |
| Simple (chunked) | tuft mesh, instanced | nothing | bevy CPU, per chunk | one per visible chunk |
| Simple, CPU-culled | tuft mesh, instanced | the surviving instances | hand-rolled CPU, per tuft | one per tier |
| Mesh chunks | blades baked into a chunk mesh | nothing | bevy CPU, per chunk | one per visible chunk |
| Mesh chunks + map | same, plus a map fetch | nothing (one small pass) | bevy CPU, per chunk | one per chunk + 2 map passes |
| Sprite cards | one textured triangle or quad | nothing | bevy CPU, per chunk | one per visible chunk |
| Sprite cards, curved | the same card, bent in the fragment | nothing | bevy CPU, per chunk | one per visible chunk |

## The three instanced paths

These three draw the *same blades from the same buffers*. What separates them
is who decides which instances survive, and what that decision costs.

### bevy_eidolon — the game's path

One batch entity per tier holding every tuft of the whole patch, resident on
the GPU and never re-uploaded. A compute pass culls per instance each frame
(frustum plus the tier's fade band) and writes an indirect draw. The CPU's
per-frame work is close to zero regardless of how much grass exists; the cost
is the compute dispatch and whatever the indirect draws cost.

*The question it answers: what does the shipping renderer cost?*

### Simple (chunked) — no compute, let bevy do it

The same tufts, split into 8 m chunks. Each chunk is an ordinary `Mesh3d`
entity with a fitted AABB, a per-chunk instance buffer uploaded once, and a
`VisibilityRange`; bevy's own CPU frustum and range culling decides what
draws, and each survivor is one `draw_indexed`. No compute pass at all.

The per-instance tier crossfade is still the game's complementary dither
computed in the vertex shader, not bevy's per-entity dither, so the *picture*
matches eidolon exactly — which is the point of having it.

*The question it answers: is the compute cull worth it, or does chunk-granular
CPU culling and a pile of draw calls get there?*

### Simple, CPU-culled — one buffer per tier, rewritten in place

One persistent instance buffer per tier, reserved once at the tier's full
count and refilled every frame with the tufts that survive a hand-rolled CPU
test: chunk AABBs against the band and frustum first, and only tufts inside a
*straddling* chunk get tested individually. One draw per tier, no compute.

This is the `particles_and_trails` buffer discipline applied to instances: the
buffer is never recreated, only written. The price is on the CPU and on the
bus — the survivors are cloned into the render world every frame, tens of
thousands of 32-byte instances.

*The question it answers: how far does tight CPU culling plus a streaming
upload get you, and where does the copying start to hurt?*

## The two mesh-chunk paths

No instancing at all. Every 8 m chunk near the camera is baked into one
ordinary mesh (a few per frame, the rest from a cache) and drawn through the
custom material like any other opaque object, so bevy culls, fades, shadows
and prepasses it with no special handling anywhere.

Its tuft is a reduced budget — 6 tufts per cell side, 4x4 blades, 3 ribbon
rows — not the game's 64-blade tuft. **Compare these two to the instanced
modes by blade count, not tuft count.**

### Mesh chunks

Wind and the affector array both run in the custom material's vertex shader,
behind a per-object flag: every vertex loops over the affectors. The mode's
cost is vertex work and memory — baked chunks are much bigger than an instance
buffer, and the bake itself streams in as you move.

*The question it answers: what do you pay for giving up instancing entirely,
and what do you get back in simplicity?*

### Mesh chunks + displacement map

The same geometry, but the affectors no longer reach the grass shader. A 2D
camera on its own render layer draws one quad into a small `Rgba16Float`
target every frame, holding every affector's push per texel; each grass vertex
then samples it once. Wind stays analytic per blade — in the map it smeared
under TAA.

The affector *count* never reaches the shader, so sixteen affectors cost what
one costs. What you pay instead is two small 2D passes, listed separately in
the overlay, and a texture fetch per vertex. `Map resolution` sets the texel
count over the 128 m square.

A second map accumulates trails: a ping-pong pair of images where each frame
reads the previous one, relaxes it toward standing grass over the regrow time
(0 = never), takes the greater of that and this frame's footprints, and writes
the result with blending off. Because it is a running maximum rather than a
blend into a persistent target, a trail keeps exactly the depth it was stamped
with for exactly as long as the knob says.

*The question it answers: is it cheaper to rasterise interaction once into a
texture than to evaluate it per vertex — and what does the extra pass cost?*

## The two card paths

Mesh chunks again, but each plant is a single textured card instead of a tuft
of blades: 24 plants per square metre at density 1. The mesh stores almost
nothing per plant — the root (repeated on every vertex), the facing normal and
(corner, hash). The vertex shader picks the family from a value noise over the
world position, the sprite and size from hashes, and reads the corner offset
and atlas UV from a sprite table in a storage buffer. Nothing is uploaded
after a chunk is baked.

These are the modes where the *foliage alpha* knob matters most: alpha to
coverage, blend, mask and opaque+discard all change the cost, because cards
are mostly transparent pixels.

### Sprite cards

17 photographed plants in three families (grass, clover, dandelion), each
fitted with the tightest apex-down triangle the packer could put over it — or
a quad, behind a knob. The atlas ships as three images for the mip knob: mip 0
only, a plain box chain, and a coverage-preserving chain that rescales each
level's alpha per sprite cell so cut-outs do not evaporate with distance.

*The question it answers: how far can you get on overdraw and texture instead
of geometry?*

### Sprite cards, curved in the fragment

Deliberately the same mode with one thing changed. Same placement, same
meshes, same streaming, same material, same code path — it shares `cards.rs`,
one `CardSet` each. What differs is the fragment: it bends the sprite inside
its own card, and the atlas it samples.

The card covers exactly one cell of a uniform-grid atlas, so a fragment finds
its own cell from the interpolated atlas uv alone (`floor(uv / cell)`) with
nothing passed down from the vertex shader. It then reads the sprite at a
point that leans with the wind: a cantilever `t*t` profile scaled by how much
of the wind crosses the card, a tip-weighted flutter, and the vertical squeeze
a leaning blade owes its own arc length. The rasteriser supplies a linear
parameter for free; the shape comes from evaluating a nonlinear function of it
per pixel. Nothing moves in the vertex shader, and the depth prepass applies
the same bend so it cuts the same pixels.

Its atlas is ten Kenney sprites in one row of 512 px cells, tinted by multiply
(the art is white-to-grey, so a multiply keeps every gradient and cannot
brighten past white) and sized by file order, 0.38 m tufts up to 1.08 m
sprigs. Each sprite sits centred in its cell with a wide transparent margin,
which is the room the bend leans into — past about a quarter of the cell the
tip walks off the sprite and is clipped rather than bent, which is where the
`Curve` knob's range comes from.

Two honest differences from the straight cards, both of which show up in the
numbers: it always uses quads covering the whole cell (the bend needs the
margin), so there is more transparent overdraw per plant; and its mesh carries
one extra vertex attribute, `uv_b = (hash, yaw)`, because the fragment needs a
per-plant phase and the card's facing, and the depth prepass keeps no normal
to take them from.

*The question it answers: does curving foliage per pixel look like curved
foliage — and is it cheaper than the geometry that would do it properly?*

## Reading the numbers

- **Eidolon, Simple and Simple CPU-culled are directly comparable.** Identical
  geometry, identical shading, identical picture. Differences are purely the
  culling and submission strategy.
- **The mesh-chunk modes are not tuft-for-tuft comparable** with those three —
  different tuft budget. The log line prints resident chunks and vertex counts
  each few seconds; compare blades.
- **The card modes are a different question entirely.** Not blades at all:
  they trade geometry for overdraw and texture bandwidth, so they move with
  the alpha, mip and anisotropy knobs rather than with density alone.
- **Mesh chunks + map carries two extra 2D passes** in its own timings. That
  is deliberate: the map camera is active only in that mode, so its cost lands
  in that mode's column and in no other.
- Both card modes and both mesh-chunk modes stream and cache chunks, so give
  them a few seconds after a knob change before reading a number.

## Where they live

```
src/grass/eidolon.rs        bevy_eidolon path (the game's renderer)
src/grass/simple.rs         chunked instancing, bevy culling, per-chunk draws
src/grass/culled.rs         per-tier persistent buffer, CPU cull, one draw
src/grass/mesh_chunks.rs    baked chunk meshes, both mesh-chunk modes
src/grass/cards.rs          both card modes, one CardSet each
src/displacement.rs         the displacement and trail maps
src/shaders/custom.wgsl     the material the mesh-chunk and card modes share
src/grass/shaders/          the material the three instanced modes share
```
