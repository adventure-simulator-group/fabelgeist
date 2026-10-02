# Physical paint construction

This is the required direction, not a description of completed functionality.
The object must be made from mixed paint applied to a prepared shield. Its
appearance, material consumption and layer thickness must follow from that
construction. The current generator supplies drawing masks, specimen-based
color estimates and approximate surface maps; it does not yet implement this
material process.

## Authoritative material state

The dependency is ingredients, mixed batches, applications, drying or curing,
and light transport through the resulting surface. A texture is a derived
rendering cache. It cannot replace the material state from which it was made.

An ingredient record must identify its composition, units, density and relevant
optical and physical measurements, including their conditions and uncertainty.
Pigment grade and particle size matter alongside pigment identity. A prepared
stock needs its pigment, binder and solvent content; a stock's name and RGB
swatch do not specify these quantities.

A mixed batch owns actual quantities of its constituents. Adding white pigment,
adding binder and adding solvent are distinct operations. Mixing conserves
constituents, and its optical law must be validated for the materials and
concentrations it supports. Selecting a desired color searches these recipes;
the requested color never overrides the resulting material appearance.

An application transfers a quantity from a batch onto a region of the object.
Use the shield's actual curved surface area, not its UV chart area, to determine
consumption and thickness. Record remaining paint and waste. Brushwork affects
where material lands, its thickness and its surface structure. Pattern masks
and the sourced lion SVG determine application regions; they do not assign the
final color of the shield.

Drying and curing update the deposited material according to the binder and
process. Volatile loss must not delete pigment or binder solids. Other mass
exchange, such as oxygen uptake during oil curing, requires an explicit process
model. Wet mixing, subsequent coats on dry paint, and any supported interaction
with partially dry paint must have separate, defined behavior.

Retain the ordered ground, paint, leaf, adhesive and coating layers, with their
composition, coverage, thickness and state. Mixing two paints before application
must remain distinct from applying them as successive coats. Fractional area
coverage, light transmission through a continuous coat, and pigment
concentration are different quantities; an opacity control cannot stand in for
all three.

Light transport must depend on wavelength, absorption, scattering, film
thickness and the layers below. Binder and coating interfaces determine surface
reflection separately. A specimen's measured surface reflection must not be
counted again as an additional gloss lobe. Conversion to display RGB and any
approximation for the game renderer or GLB export belong at the output boundary.
Keep the spectral material state so changing an illuminant does not require
changing a recipe or inventing a new pigment color.

Material costs follow the quantities prepared, applied, retained and wasted.
Preparation, application and repair add their own labor and process costs.
Increasing painted area at fixed film thickness requires more paint; increasing
thickness at fixed area also requires more paint. The color picker must quote
the construction it proposes, including coverage requirements, rather than an
unrelated demonstration batch.

## Optical model and calibration

Finite-thickness spectral Kubelka–Munk transport is a candidate first model for
diffuse paint layers. It is a two-flux approximation, with surface interfaces
and directional effects requiring separate treatment. Within this model,
interface-corrected reflectance of an optically thick layer determines the
ratio of absorption to scattering, not the absolute coefficients needed to
predict hiding power at a stated thickness. Fit known-thickness films
over calibrated dark and light grounds, or use reflection and transmission,
with the measurement geometry and interface contribution accounted for.
[Ronnen Levinson's pigment measurement presentation, pp. 15–29](https://coolcolors.lbl.gov/assets/docs/OtherTalks/Pigment-Talk-2004-04-22a.pdf)
sets out these measurement distinctions.

The current [Reichert dataset subset](references/MEASURED_PAINT.md) contains
finished specimens with gum Arabic on parchment. It lacks the controlled film
thickness and stock yield needed here. The authors document support, binder,
grain-size and application effects, including specular reflection that was
difficult to exclude. Their spectra are useful reference observations, not
intrinsic pigment coefficients transferable directly to a shield.
[Reichert et al., 2025](https://doi.org/10.1007/s00216-025-05948-3).

Search for suitable published calibration data first. Record unsupported
parameters explicitly; do not infer them silently from RGB swatches. Synthetic
coefficients can test the solver's invariants but cannot authenticate a named
historical paint. Validate fitted materials against mixtures, thicknesses and
grounds excluded from fitting, with declared spectral and color tolerances.

[Baxter, Wendt and Lin, *IMPaSTo*, 2004](https://gamma.cs.unc.edu/IMPASTO/publications/Baxter-IMPaSTo_Web-NPAR04.pdf)
provides a useful engineering precedent: conserved paint and pigment quantities
in a height field, wet mixing, and optical composition of dry layers. Its
approximations need evaluation for our materials. A layered height field can
represent thin paint films without generating a mesh for every pigment grain;
the conservation and optical behavior still need to follow from material state.

## Acceptance and implementation order

Start with physical test panels before returning to the lion. Implement the
batch and application state, then finite-thickness layer optics and calibration,
then connect the drawing masks, workshop interface and renderer. Replace the
current authoritative color-to-map path when this model is integrated; do not
retain a parallel compatibility implementation.

The first panel must compare the same paint at different deposited amounts over
known dark and light grounds, a mixture containing white, and distinct ordered
coats. Acceptance requires:

- Zero added paint reproduces the ground. Increasing thickness approaches the
  opaque limit where the material has one.
- Reflected, transmitted and absorbed energy are nonnegative and sum to the
  incident energy within a stated numerical tolerance.
- Splitting a homogeneous coat into identical adjacent portions has no optical
  effect when no additional physical interface is introduced.
- Reordering the representation of an otherwise identical homogeneous batch
  has no effect. Any effects of physical preparation order require an explicit
  process model. Reversing distinct dry coats generally changes the result.
- Remaining material, deposition and recorded waste balance the starting
  batch. Drying and curing account for exchanges with the environment.
- Doubling painted area at the same thickness doubles deposited material.
  Changing bake resolution preserves physical coverage and quantities, with
  optical results converging under refinement.
- Held-out measurements validate the material model. A successful conservation
  test alone does not validate pigment appearance.
- Changing illumination leaves the batch and layer construction unchanged.

After these pass, apply the existing lion masks to the same material system.
Painted shadows and highlights become additional paint applications with their
own mixtures and quantities. Their appearance follows from the resulting coats
under the scene lighting.
