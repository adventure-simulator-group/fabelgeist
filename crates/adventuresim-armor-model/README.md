# Armor model

This crate defines the armor recipes and generates their geometry on a compute
device. `ArmorGpu` owns the device context and the kernels every armor family
shares; one instance serves the whole process and any number of threads.

Every part is built the same way. The host lays the part out from the design
alone: how many carrier vertices each shell has, how they connect, and how each
shell is thickened. The device evaluates every carrier point in the part's
frame, the fitter (in the character creator) moves the carriers onto the
wearer, and one shell stage thickens every shell at once. It adds the inner
wall and the boundary returns, keeps outward winding under reflected frames,
and computes the final normals. Nothing is read back until the finished part
is, so a part and all of its morph samples are recorded before one readback.

`DevicePart` is that part under construction. Families record into one:
`record_helmet`, `record_close_helmet` and `record_coif` for helmets,
`record_limb_armor` and `record_extremity_armor` for limbs, hands and feet,
`record_garment_tube`, `record_garment_torso`, `record_fauld`,
`record_tassets` and `record_gorget_plates` for garments. Each reads a part
frame from a device buffer: origin, axes and half extents measured on the
wearer, not body triangles. The close helmet and the coif also read sections
the fitter measured (`CloseHelmetProfile`, `CoifDrapeProfile`). The vambrace
(`gpu::bracer`) and the paired breastplate (`gpu::breastplate`) are fitted
directly to the wearer's selected skin and carry its UVs, skin weights and
morph correspondence. `ArmorGpu::build_in`, `generate_helmet_on` and
`generate_limb_armor_on` build a part placed by host frames, for previews and
tests.

Construction determines the recipe family. A barbute has a continuous bowl and
cheek opening; a burgonet has a separate peak, cheek plates and neck defense.
They share device kernels, but have separate designs because changing a few
bowl dimensions cannot express those structural differences. Morions, kettle
hats, sallets and close helmets likewise expose their own brim, tail, visor,
crest, opening and lower-edge controls. Cap and coif recipes have soft covering
boundaries. Clearance and gauge remain separate from style controls.

Long limb plates expose length, taper and wrap. Elbow and knee cops expose
localized projection and wing dimensions; shoulder, finger and foot defenses
have overlapping lame controls. Gauntlets fit the hand and thumb independently.
Textile and mail envelopes use garment patterns with neck, arm and hem openings,
plus joint-following sleeves and leg coverings. They represent the garment's
volume; individual mail rings and closures are outside this geometry layer.

Metal limb and garment plates, vambraces, helmet crowns and close-helmet visors
share `PlateFluting`. Set the recipe's optional fluting field to `None` (`null`
in JSON) for a plain plate. Count, width as a fraction of pitch, physical relief
depth, pattern spread, lower spread, start/end position and fade length are
independent. More flutes at the same spread produce narrower flutes; changing
width changes the balance of raised metal and intervening smooth land. Width
scales with the wearer, while depth and gauge retain their millimetre values.
Breastplates use the same flute parameters with a torso-specific distribution.

Each construction supplies a surface chart for its relief. The smooth carrier
is fitted to anatomy, then relief is applied to both shell surfaces and their
boundaries are closed. Gauge follows the carrier's extrusion direction; it
does not follow the steep local flute slopes. A body morph retains the selected
design's vertex and triangle correspondence. Changing a design, including its
flute count or pattern, may rebuild topology and requires new morph targets.

Boundary controls are specific to the construction: sallets expose opening
width/sweep and visor side-panel depth, greaves expose ankle extension, and
joint cops expose wing notches. Tassets have independent inner cutaway,
roundness and point controls; gorgets expose collar height and slope, separate
front and rear hem flatness, rear sweep, independent front/rear bib depth and
width, and neck clearance. Gorget fluting occupies the front bib; the shoulder
return remains plain.
The canonical gorget uses [DIA 53.202.2](https://dia.org/collection/gorget/110870)
for its shape. Its 1 mm nominal wall is supported by the measured
[German gorget A-25, about 1550](https://www.allenantiques.com/A-25.html);
the DIA object record does not publish wall thickness. The creator fits these
authored outlines to anatomical sections. Export checks measure intersections
and sampled clearance across body morphs.

Use the character creator's `--write-armor-designs` command to obtain the
current catalog recipe schema. Vambrace and breastplate designs load from their
separate `--bracer-design` and `--breastplate-design` files. See the
[creator authoring guide](../adventuresim-character-creator/README.md#parametric-armor-authoring)
for editing, saving and exporting all three files. Required fields must be
present; obsolete recipe schemas are not accepted.

Every catalog armor item must resolve to an authored recipe. The character
creator rejects an unmapped armor item instead of sending it through the
body-topology clothing shell path. Ordinary non-armor clothing still uses its
separate clothing generator.

Run `cargo test -p adventuresim-armor-model` for the crate's regression tests;
they build parts on the device and need a compute-capable GPU. Run `just
fmt-check` and `just lint` for the repository quality gates. See [armor
references and validation](review/README.md) for source references and
reproducible checks.
