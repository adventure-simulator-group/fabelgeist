# Interior daylight

The tactical presentation derives a diffuse daylight field from each playable
building's room cells, storeys, and exterior windows or unclosed apertures.
Each cell stores six directional intensities. Nearby openings contribute more
light; a bounded isotropic component approximates bounce onto ceilings and
surfaces facing away from windows. Opaque doors contribute no daylight.

The forward PBR shader interpolates samples within the containing room and
storey. Empty cells receive no interior contribution. Albedo, metallic response,
material occlusion, exposure, and fog still apply. Building meshes, furniture,
characters, equipment, and blood-stained character surfaces share the field.
This is estimated diffuse lighting, with no ray tracing, scene captures,
shadow maps, or additional light entities. It does not simulate changing door
positions, detailed furniture occlusion, or specular interreflection.

At most sixteen building fields within 100 metres of the camera are resident.
World bounds reject exterior fragments before local-coordinate calculations;
interior fragments read four neighbouring samples. Static sample data uploads
only when the selected buildings, transforms, or daylight change. Samples use
32 bytes per cell and occupy one shared read-only storage-buffer binding on
both native and WebGPU. Standard materials retain their authored assets in
`InteriorMaterialSource`; rendering uses `InteriorMaterial` and propagates
edits to those source assets.

The gameplay camera applies up to two stops of interior exposure compensation.
The target comes from the room's estimated daylight. Exponential adaptation
takes longer when entering darkness than when returning to brighter surroundings
and is independent of frame rate. The field fades out with daylight and the
authored celestial exposure remains the night baseline. Scene changes reset
adaptation.

Run the focused checks with:

```text
cargo test -p adventuresim-tactical-client --bin adventuresim-tactical-client interior_lighting
cargo check -p adventuresim-tactical-client --bin adventuresim-tactical-client --target wasm32-unknown-unknown
cargo run -p adventuresim-tactical-client --bin tactical-scene-viewer -- --fixture interior-furniture-rooms --profile furnished-room-review --view house --settle-frames 120 --output target/interior-review
```
