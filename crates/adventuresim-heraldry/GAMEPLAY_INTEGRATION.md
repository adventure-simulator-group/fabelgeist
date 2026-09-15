# Heraldic workmanship and gameplay

This is a requirement for future game integration. The standalone generator
currently models an intact decorated surface. Its measured-paint mixer quotes
a chosen batch using user-supplied stock prices; it does not calculate a
whole-object price, production time, damage, or repair.

Material and labor constraints should make historically common, practical
finishes attractive through ordinary player choices. Players should be able
to commission elaborate work when its cost and purpose justify it. The game
should not enforce a preferred appearance through an arbitrary authenticity
bonus, a ban on exceptional techniques, or hidden combat penalties.

G. W. Eve conjectured that ordinary working shields were generally painted
flat, while physical relief was exceptional partly because of its expense
and difficulty of repair. This is a useful economic hypothesis, not a measured
distribution or a price schedule for Germany in 1544. Surviving elaborate
objects establish that techniques existed; they do not establish how common
they were. See the
[bibliographic entry](REFERENCES.md#historical-interpretation-and-economics)
and the [material evidence study](references/PAINTED_SHADING.md).

Costing must distinguish painted tonal modeling from actual raised decoration.
Painting a shadow does not imply carving or molding a raised charge. Likewise,
a genuinely raised charge may use comparatively simple flat colors. The
generator's very thin adhesive and paint relief is not a model of an entire
sculpted animal.

| Work | Production factors to represent | Repair factors to represent |
| --- | --- | --- |
| Flat painted device | Preparation, pigment and binder, painted area, outlines and application time. | Local preparation, repainting lost areas, matching colors and line work. |
| Painted tonal modeling | Additional marks or paint passes, color preparation, painter skill and time. | Restoring the drawing and tonal treatment in the damaged area. |
| Metal leaf and glazes | Leaf and adhesive, preparation, application, burnishing where applicable, drying or curing. | The layers that must be rebuilt, replacement leaf and matching the finish. |
| Raised decoration | Material volume, carving or mold preparation, forming, attachment and finishing. | Shape and attachment damage, access to suitable tools and skills, rebuilding affected layers. |

These are requirements for a process-based estimate, not assigned multipliers.
The [paint catalog](references/PAINT_RECIPES.md) now records pigment and binder
choices. Costing must count materials actually applied, rather than every
entry in the palette. The [measured mixer](references/MEASURED_PAINT.md)
records prepared-stock volume fractions and batch quotes. Its measurements do
not provide dry-film yield, shield coverage, historical prices, labor or
durability. The catalog display swatches provide no ingredient quantities.
Local material supply, craftspeople, workshop access, and time should affect
what can actually be made or restored. Repair should depend on the damage and
the affected construction, rather than always charging for a complete new
device. A mold or repeatable pattern may change the labor economics; more
elaboration should not automatically mean a fixed surcharge everywhere.

The commissioning and repair interfaces must expose the estimated price,
time, required workshop or skill, and the work being proposed. Players should
be able to compare maintaining a finish with simplifying it when appropriate.
Any gameplay-relevant consequences must be visible or intuitive.

Historical prevalence should be a result to compare against evidence after
calibration, not an assumed percentage built into the selection rules.
Quantitative costs, durability, repair times, and relative frequencies require
further evidence; Eve's conjecture alone cannot supply those numbers.
