# Procedural texture review

Recipes produce deterministic, repeating material maps. Hewn oak shares a growth
coordinate between latewood bands and interrupted fibers; branch intersections
deflect both around the knot. Dressed stone retains its ashlar bond while metric
edge fractures change the stone boundary, bevels round the exposed edge, and
sparse cavities interrupt the dressed face. Mortar has its own continuous
aggregate and trowel fields. Feature masks drive the corresponding color and
roughness changes.

Hewn oak uses a 1024-pixel tile over 2 metres. Dressed stone uses a 2048-pixel tile
over 7.2 metres. Their nominal texel spacings are 1.95 mm and 3.52 mm respectively.
Stone cavities are deliberately sparse and large enough to resolve at that
density. The four RGBA maps, including mips, occupy approximately 21.3 MiB for
oak and 85.3 MiB for stone before GPU compression. This is four times their
previous texture allocation.

## Export and compare

Run these from the repository root. Before changing a recipe, export its current
maps into `before`; after the change, export into `after`:

```text
cargo run -p adventuresim-procedural-textures --bin procedural-texture-lab -- export hewn-oak --output target/procedural-texture-lab/before
cargo run -p adventuresim-procedural-textures --bin procedural-texture-lab -- export hewn-oak --output target/procedural-texture-lab/after
cargo run -p adventuresim-procedural-textures --bin procedural-texture-lab -- compare hewn-oak --directory target/procedural-texture-lab --output target/procedural-texture-lab/hewn-oak-detail.png
cargo run -p adventuresim-procedural-textures --bin procedural-texture-lab -- compare hewn-oak --directory target/procedural-texture-lab --output target/procedural-texture-lab/hewn-oak-overview.png --overview
```

Use `dressed-stone` for the masonry comparison. The headless comparison requires
a graphics adapter and captures the same crop, camera, lighting, and material
settings for both exports. It uses Bevy `StandardMaterial` with color, normal,
and packed AO/roughness/metallic maps, without height displacement. PNG exports
contain the base texture level, so these captures inspect surface construction;
they do not replace gameplay-distance and mip-transition review in the tactical
renderer. All generated evidence stays under `target/`.

Run `cargo test -p adventuresim-procedural-textures` for recipe repeatability,
tiling, feature scale, channel packing, and complete mip chains. Additional
behavioral tests check grain deflection, chipped boundary area, cavity coverage,
and mortar variation.
