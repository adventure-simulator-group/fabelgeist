# Candidate 33 installed-runtime helmet visual review

Verdict: **PASS — moderate confidence for the visible coarse helmet at the sampled runtime poses.** No visible wearer penetration or loss of coarse plate coherence is established in the exposed regions. **The nape/shoulder interaction in the turned guard pose remains visually uncertain because the shoulder hides most of that region.**

I actually opened these twelve images:

- `idle/ordinary-camera-pitch-0000-front.png`
- `idle/ordinary-camera-pitch-0000-side.png`
- `idle/ordinary-camera-pitch-0032-front.png`
- `idle/ordinary-camera-pitch-0032-side.png`
- `idle/ordinary-camera-pitch-0064-front.png`
- `idle/ordinary-camera-pitch-0064-side.png`
- `guard/raised-guard-stationary-turn-0000-front.png`
- `guard/raised-guard-stationary-turn-0000-side.png`
- `guard/raised-guard-stationary-turn-0064-front.png`
- `guard/raised-guard-stationary-turn-0064-side.png`
- `guard/raised-guard-stationary-turn-0127-front.png`
- `guard/raised-guard-stationary-turn-0127-side.png`

All paths are relative to `target/helmet-review/`. I also read `runtime-provenance.json`, plus both scenarios' `armor-readiness.json` and `body-proportions.json`. Provenance records installed asset `assets/equipment/procedural/close_helmet--worn.glb`, SHA-256 `16128175cb230e0e4ec547ab0dadccf9a680437019e079f069a3e5deff11d595`. The readiness records report loaded skull, visor, and bevor, each with 130 skin joints and 47 morph targets; both scenarios record the same nonzero morph and body-proportion configuration. I did not independently hash the installed asset.

In all six idle images and the initial guard pair, the rounded skull, enclosing visor, compact jaw, and descending rear plates remain a coherent helmet on the wearer. The front shows skin through the intentional sight openings without a visible skin patch breaking through the face plate or crown. The side silhouettes preserve the skull-to-face and chin-to-neck relationships, and exposed nape plates remain readable over the rear neck/shoulder region. The sampled camera-pitch change does not visibly separate the main plates or distort the enclosing volumes.

In guard frames 64 and 127, the visible skull/visor/jaw remain coherent as the wearer turns and raises the arms. No exposed wearer surface visibly pierces those major helmet surfaces. However, the view labeled `side` now looks largely toward the back of the turned wearer, whose raised shoulder occludes the lower skull and most neck plates. The companion `front` image gives an oblique view of part of the tail, but does not reveal the complete hidden interface. Occlusion is not evidence of penetration and cannot prove its absence. I therefore leave that hidden interface unresolved rather than passing its clearance.

The helmet occupies roughly a few hundred pixels in these gameplay captures, sufficient for the requested gross visual check but insufficient for small seam or clearance judgments. This report does not assess animation quality, unsampled frames, continuous clearance, minor details, other wearers, or all supported morphs. The reported idle hand/chest jitter failure and guard process success are harness outcomes, not evidence for or against this scoped helmet visual verdict. This is not a full historical-reference re-review.
