# fabelgeist-garment-code

A native Rust port of [GarmentCode](https://github.com/maria-korosteleva/GarmentCode) —
*Programming Parametric Sewing Patterns* (Korosteleva & Sorkine-Hornung, SIGGRAPH Asia 2023;
GarmentCodeData, ECCV 2024).

Give it body measurements and a design, and it produces a sewing pattern: flat
fabric panels, their placement around the body in 3D, and the stitches that join
them — the same `*_specification.json` format the GarmentCodeData dataset uses,
plus SVG renderings.

```bash
cargo run -p fabelgeist-garment-code --bin garment-code -- \
  --body   crates/fabelgeist-garment-code/assets/bodies/mean_all.yaml \
  --design crates/fabelgeist-garment-code/assets/design_params/t-shirt.yaml \
  --out    output
```

## What is in the box

| Module | Ports | Contents |
| --- | --- | --- |
| `curve` | `svgpathtools` (the parts used) | Line / quadratic / cubic Bezier / circular-arc segments: evaluation, arc length, arc-length inversion, cropping, bounding boxes, intersections, SVG path parsing |
| `optimize` | `scipy.optimize.minimize` | BFGS and a box-constrained projected BFGS, both with a strong-Wolfe line search |
| `garment` | `pygarment.garmentcode` | Edges, edge sequences, panels, interfaces, stitching rules, and the operators that cut corners, insert darts and match seams |
| `pattern` | `pygarment.pattern` | The pattern specification, panel ordering, edge-loop normalisation, JSON and SVG output |
| `programs` | `assets/garment_programs` | The garment library: bodices, sleeves, collars, skirts, pants, waistbands, cuffs, and the `MetaGarment` that stacks them |

Every garment element of the reference is available: `Shirt` and `FittedShirt`
uppers; `PencilSkirt`, `Skirt2`, `SkirtCircle`, `AsymmSkirtCircle`,
`SkirtManyPanels`, `SkirtLevels`, `GodetSkirt` and `Pants` lowers; `StraightWB`
and `FittedWB` waistbands; the seven neckline shapes plus `Turtle`,
`SimpleLapel` and `Hood2Panels` collars; the three armhole shapes and the three
cuff styles.

**Out of scope** — the reference's cloth simulation (NVIDIA Warp), its
Maya/Qualoth tooling, and its web configurator. This crate produces patterns,
not draped 3D garments.

## Library use

```rust
use fabelgeist_garment_code::{Body, Design};
use fabelgeist_garment_code::programs::MetaGarment;

let body = Body::load("assets/bodies/mean_all.yaml")?;
let design = Design::load("assets/design_params/t-shirt.yaml")?;

let piece = MetaGarment::new("t-shirt", &body, &design);
let mut pattern = piece.assembly();

pattern.serialize("output", true, "", false)?;
```

## Driving it from a front-end

Three pieces exist for callers that are not the CLI:

* `assets::BODIES` / `assets::DESIGNS` bundle the presets as strings, and
  `Body::from_yaml_str` / `Design::from_yaml_str` load YAML already in memory.
* `Design::params()` returns every parameter in the tree with the schema the
  YAML declares next to it — `type`, `range` and the current value — which is
  what builds a control per parameter without hard-coding any garment.
  `Design::to_yaml()` writes an edited tree back out.
* `PatternSpec::drawing(layout, margin)` returns the laid-out geometry that
  `to_svg` renders: per panel an SVG path, its bounding box, whether it faces
  front, and each edge's own path and midpoint (edge indices match the ones a
  `Stitch` refers to, so a seam overlay can be drawn from them).

The "Garment Pattern" tab of the `tasks` example app is built on exactly these.

## Fidelity to the reference

The port is validated by building the same designs with both implementations
and diffing the resulting specifications. Across a curated sweep of 48 designs
covering every garment element, and a randomised sweep of 58 designs drawn with
the reference's own `DesignSampler`:

* **106 designs, all structurally identical** — same panels, same vertex and
  edge counts, same edge endpoints and labels, same stitches, same panel order —
  bar one randomised design (see below).
* Most designs match **bit for bit**; the rest agree to well under a tenth of a
  millimetre.
* Both implementations reject the same impossible designs.

### Where it does *not* match exactly, and why

Two of the reference's curve fits have objectives with no unique minimum, so the
answer depends on the optimiser's exact path rather than on the geometry:

* `curve_match_tangents` (`operators.py`) asks for a cubic Bezier with
  prescribed endpoint tangents *and* a prescribed length — an under-determined
  family — and regularises it with `max(curvature)` sampled at 70 points, which
  is only piecewise smooth. The reference's own comment calls it unstable.
* `curve_3_points` fits a quadratic control point so the curve passes through a
  target, which is one equation short of determining it.

On the reference's own objective the two implementations trade places: for the
T-shirt's armhole this port scores 0.00721 against scipy's 0.00729, and
restarting scipy from this port's answer improves on scipy's original result. So
neither is "the" minimum — they are different points on a flat valley.

The visible consequences:

* Sleeve and hood panels can differ by a few millimetres, and in the worst
  randomised case by about 1 cm on a hood's straight top edge, where the fitted
  curve's free endpoint slides along its own direction.
* One randomised design in 58 landed on the far side of a 0.05 tolerance in
  `StitchingRule::isMatching`, so one seam was subdivided into three segments
  instead of two. The garment is the same; the seam is described with one extra
  stitch.

Matching exactly would mean reimplementing scipy's L-BFGS-B iterate for iterate,
which buys nothing geometrically.

### Two conventions the reference genuinely mixes

Worth knowing if you compare code side by side:

* Panels serialise their rotation with scipy's **intrinsic** `XYZ` Euler
  convention (`Rx·Ry·Rz`), while `pattern/rotation.py` reads those same angles
  back with the Maya **extrinsic** convention (`Rz·Ry·Rx`). Both are ported as-is
  (`math::Rotation` and `math::euler_xyz_to_r`), because the panel ordering
  depends on the second one.
* `Panel.set_pivot` truncates its shift to whole centimetres (`int()` in
  Python). Panel translations depend on it, so `trunc()` is deliberate.

## Notes on the port

The reference leans on Python's reference semantics: adjacent edges share the
very same `[x, y]` list for their common vertex, and an interface holds the same
`Edge` objects the panel does, so subdividing an edge for a stitch updates both.
That aliasing *is* the design. It is preserved here with `Rc<RefCell<..>>` for
vertices, edges, panels and interfaces, with `Rc::ptr_eq` standing in for
Python's `is`.

No Python, no C: the `svgpathtools`, `scipy` and YAML pieces the reference needs
are all reimplemented in this crate.

## Testing

```bash
cargo test -p fabelgeist-garment-code
```

Unit tests pin numbers taken from the reference libraries — `svgpathtools`
lengths, scipy rotation matrices, the reference's own `Sun` shape vertices, even
its slightly-off-pi semicircle arc angle. `tests/reference_parity.rs` builds five
whole garments and checks panel counts, perimeters, translations, stitch counts
and panel order against the Python output.

## Attribution

GarmentCode is by Maria Korosteleva, Timur Levent Kesdogan, Fabian Kemper,
Stephan Wenninger, Jasmin Koller, Yuhan Zhang, Mario Botsch and Olga
Sorkine-Hornung, and is MIT licensed. The body measurements and design
parameters under `assets/` are copied from that project unchanged; its licence
is kept alongside them as `assets/LICENSE-GarmentCode`.
