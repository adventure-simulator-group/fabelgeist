# Baseline metric interpretation

The accompanying baseline-runtime-metrics.csv records the old runtime heuristic
outputs, not physical measurements of museum weapons and not integrated mesh
mass. It is evidence of gameplay behavior before the change.

Authored thickness did not consistently mean maximum mesh thickness. In
particular, Rust Flat blades scaled the requested half-depth by 0.45; an authored
11 mm value could yield only 4.95 mm actual thickness. Fullered and diamond
sections used different conventions. Consequently, a large authored thickness
alone does not establish an oversized visible blade. The full audit must compare
measured section bounds and integrated material volume with museum whole-weapon
mass and visible silhouette. The implementation is correcting the semantics so
an authored thickness means actual maximum section thickness.

Analytical failures are nevertheless decisive for the old gameplay path:
halberd 9.61 kg, war hammer 6.25 kg, hand axe 5.51 kg; knife 0.85 kg and utility
knife 0.65 kg. The corresponding close museum war hammer is 0.992 kg, and the
German/Swiss halberd references are 2.01 and 2.66 kg. Those failures cannot be
accepted by merely relabeling the renderer. The pike is only 3.67 m long and the
Gothic mace 1.37 m long; these are proportion defects independently of density.

Mass integration must account for non-overlapping materials. Leather wrapped
around wood and a steel tang is not a homogeneous leather grip, nor is a steel
sheath around a wood core a solid steel shaft. If the model deliberately omits
hidden tangs or uses component overlap, disclose those modeling approximations
when reporting physical accuracy.
