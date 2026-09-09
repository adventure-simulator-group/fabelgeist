# Runtime final visual regression

Current result: **PASS within the reviewed scope**. See the
[final boot follow-up](#final-boot25-follow-up--pass-within-the-reviewed-scope).
The failed checkpoint below is retained as superseded evidence.

Independent image review of installed armor in the Bevy runtime. Reviewer: final_body_regression. No geometry or implementation edits. Assessment was formed from the images before consulting other reviewer conclusions. Numeric morph/body audits were not used to infer visual correctness.

## Original checkpoint verdict (superseded for boots)

**FAIL — boot/chausse layering remains unresolved in these captures.** The visibly interleaved leather shaft and mail lower leg is a structural crossing, not merely exposed skin or a lighting seam. The dedicated material-separated diagnostic makes this unambiguous. A corrected boot capture must supersede this checkpoint before a whole-runtime pass.

**PASS within the sampled visible torso, sleeve, hand, skirt, thigh and plate-foot regions:** no disappearing armor, detached GPU triangles, long underarm/cuff spikes, obvious inverted panels, or sawtooth hand/foot overlap edges were observed. Plate, mail and padded harnesses remained recognizable simple armor assemblies through idle, walking, raised guard and short/long spine samples. This regional result does not average away the boot failure.

## Findings and limits

- Plate walk and guard maintain breastplate/fauld and limb components. The fauld becomes wavy as the thighs move, but its broad layered bands remain recognizable; no clear cut through the breastplate or thigh shell is visible in the sampled frames.
- Mail guard keeps both bent sleeves and torso visible. The coif/neck interface and short-spine torso retain shallow coarse folds and overlap marks; no detached shards or obvious open backface patches were seen.
- Padded guard has broad rounded sleeve bends and coarse shoulder/elbow folds. The short-spine side view exposes the upper shoulder along the connected neckline; the long-spine pose raises the sleeve ends relative to the wrists. These are visible coverage limitations, not demonstrated panel penetrations.
- Guard front frames 0016/0032 expose a skin-colored groin patch below the lifted hem, between separate leg pieces, especially in mail and padded harnesses. This is a connected opening beyond the skirt edge. It is explicitly retained as a coverage reservation; it is not classified as leggings breaking through an intact skirt surface. The scope is appropriate armor recipes by body region, not sealed coverage at every joint or hem in every pose.
- The mail boot shaft has a conspicuous brown region within the light lower-leg outline even at guard frame 0000. The separately colored boot/mail diagnostic shows their surfaces alternating across a curved boundary on the shaft. This is the blocking layering failure. Padded boots require the same corrected follow-up because the intended fix also changes their overlap envelope.
- Terrain obscures feet during portions of the ordinary walking/guard fixture. Those absent pixels are not attributed to armor. The plate flat-grid front/side frames 0000/0016/0032 expose both sabatons and their open heels: the toe shells follow the feet with smooth outer overlap edges, without isolated skin islands punching through their closed upper surfaces. Connected heel/ankle openings remain visible.
- These are front/side still samples, not exhaustive frame-by-frame playback, all camera angles, control/completeness proof, or a guarantee for unsampled parameter combinations. Dark padded materials limit inspection of very small surface creases. Historical proportion/detail review is recorded separately in the earlier static reports; no requirement for decoration, straps, textures, or an exact replica is added here.

## Exact images actually viewed

Paths below are relative to `target/armor-review/runtime/`. 104 unique images were opened and visually inspected. Padded idle/walk images were reopened in smaller batches after a tool output truncation; they are counted once.

- `plate-runtime-final-walk/steady-walk-2.0-0000-front.png`
- `plate-runtime-final-walk/steady-walk-2.0-0000-side.png`
- `plate-runtime-final-walk/steady-walk-2.0-0016-front.png`
- `plate-runtime-final-walk/steady-walk-2.0-0016-side.png`
- `plate-runtime-final-walk/steady-walk-2.0-0032-front.png`
- `plate-runtime-final-walk/steady-walk-2.0-0032-side.png`
- `mail-runtime-final-guard/raised-guard-forward-0000-front.png`
- `mail-runtime-final-guard/raised-guard-forward-0000-side.png`
- `mail-runtime-final-guard/raised-guard-forward-0016-front.png`
- `mail-runtime-final-guard/raised-guard-forward-0016-side.png`
- `mail-runtime-final-guard/raised-guard-forward-0032-front.png`
- `mail-runtime-final-guard/raised-guard-forward-0032-side.png`
- `mail-runtime-final-guard/raised-guard-forward-0054-front.png`
- `mail-runtime-final-guard/raised-guard-forward-0054-side.png`
- `plate-runtime-final-idle/ordinary-camera-pitch-0000-front.png`
- `plate-runtime-final-idle/ordinary-camera-pitch-0000-side.png`
- `plate-runtime-final-idle/ordinary-camera-pitch-0016-front.png`
- `plate-runtime-final-idle/ordinary-camera-pitch-0016-side.png`
- `plate-runtime-final-idle/ordinary-camera-pitch-0032-front.png`
- `plate-runtime-final-idle/ordinary-camera-pitch-0032-side.png`
- `plate-runtime-final-guard/raised-guard-forward-0000-front.png`
- `plate-runtime-final-guard/raised-guard-forward-0000-side.png`
- `plate-runtime-final-guard/raised-guard-forward-0016-front.png`
- `plate-runtime-final-guard/raised-guard-forward-0016-side.png`
- `plate-runtime-final-guard/raised-guard-forward-0032-front.png`
- `plate-runtime-final-guard/raised-guard-forward-0032-side.png`
- `plate-runtime-final-guard/raised-guard-forward-0054-front.png`
- `plate-runtime-final-guard/raised-guard-forward-0054-side.png`
- `plate-runtime-final-short-spine/ordinary-camera-pitch-0000-front.png`
- `plate-runtime-final-short-spine/ordinary-camera-pitch-0000-side.png`
- `plate-runtime-final-short-spine/ordinary-camera-pitch-0016-front.png`
- `plate-runtime-final-short-spine/ordinary-camera-pitch-0016-side.png`
- `plate-runtime-final-short-spine/ordinary-camera-pitch-0032-front.png`
- `plate-runtime-final-short-spine/ordinary-camera-pitch-0032-side.png`
- `plate-runtime-final-long-spine/ordinary-camera-pitch-0000-front.png`
- `plate-runtime-final-long-spine/ordinary-camera-pitch-0000-side.png`
- `plate-runtime-final-long-spine/ordinary-camera-pitch-0016-front.png`
- `plate-runtime-final-long-spine/ordinary-camera-pitch-0016-side.png`
- `plate-runtime-final-long-spine/ordinary-camera-pitch-0032-front.png`
- `plate-runtime-final-long-spine/ordinary-camera-pitch-0032-side.png`
- `mail-runtime-final-idle/ordinary-camera-pitch-0000-front.png`
- `mail-runtime-final-idle/ordinary-camera-pitch-0000-side.png`
- `mail-runtime-final-idle/ordinary-camera-pitch-0016-front.png`
- `mail-runtime-final-idle/ordinary-camera-pitch-0016-side.png`
- `mail-runtime-final-idle/ordinary-camera-pitch-0032-front.png`
- `mail-runtime-final-idle/ordinary-camera-pitch-0032-side.png`
- `mail-runtime-final-walk/steady-walk-2.0-0000-front.png`
- `mail-runtime-final-walk/steady-walk-2.0-0000-side.png`
- `mail-runtime-final-walk/steady-walk-2.0-0016-front.png`
- `mail-runtime-final-walk/steady-walk-2.0-0016-side.png`
- `mail-runtime-final-walk/steady-walk-2.0-0032-front.png`
- `mail-runtime-final-walk/steady-walk-2.0-0032-side.png`
- `mail-runtime-final-short-spine/ordinary-camera-pitch-0000-front.png`
- `mail-runtime-final-short-spine/ordinary-camera-pitch-0000-side.png`
- `mail-runtime-final-short-spine/ordinary-camera-pitch-0016-front.png`
- `mail-runtime-final-short-spine/ordinary-camera-pitch-0016-side.png`
- `mail-runtime-final-short-spine/ordinary-camera-pitch-0032-front.png`
- `mail-runtime-final-short-spine/ordinary-camera-pitch-0032-side.png`
- `mail-runtime-final-long-spine/ordinary-camera-pitch-0000-front.png`
- `mail-runtime-final-long-spine/ordinary-camera-pitch-0000-side.png`
- `mail-runtime-final-long-spine/ordinary-camera-pitch-0016-front.png`
- `mail-runtime-final-long-spine/ordinary-camera-pitch-0016-side.png`
- `mail-runtime-final-long-spine/ordinary-camera-pitch-0032-front.png`
- `mail-runtime-final-long-spine/ordinary-camera-pitch-0032-side.png`
- `padded-runtime-final-idle/ordinary-camera-pitch-0000-front.png`
- `padded-runtime-final-idle/ordinary-camera-pitch-0000-side.png`
- `padded-runtime-final-idle/ordinary-camera-pitch-0016-front.png`
- `padded-runtime-final-idle/ordinary-camera-pitch-0016-side.png`
- `padded-runtime-final-idle/ordinary-camera-pitch-0032-front.png`
- `padded-runtime-final-idle/ordinary-camera-pitch-0032-side.png`
- `padded-runtime-final-walk/steady-walk-2.0-0000-front.png`
- `padded-runtime-final-walk/steady-walk-2.0-0000-side.png`
- `padded-runtime-final-walk/steady-walk-2.0-0016-front.png`
- `padded-runtime-final-walk/steady-walk-2.0-0016-side.png`
- `padded-runtime-final-walk/steady-walk-2.0-0032-front.png`
- `padded-runtime-final-walk/steady-walk-2.0-0032-side.png`
- `padded-runtime-final-guard/raised-guard-forward-0000-front.png`
- `padded-runtime-final-guard/raised-guard-forward-0000-side.png`
- `padded-runtime-final-guard/raised-guard-forward-0016-front.png`
- `padded-runtime-final-guard/raised-guard-forward-0016-side.png`
- `padded-runtime-final-guard/raised-guard-forward-0032-front.png`
- `padded-runtime-final-guard/raised-guard-forward-0032-side.png`
- `padded-runtime-final-guard/raised-guard-forward-0054-front.png`
- `padded-runtime-final-guard/raised-guard-forward-0054-side.png`
- `padded-runtime-final-short-spine/ordinary-camera-pitch-0000-front.png`
- `padded-runtime-final-short-spine/ordinary-camera-pitch-0000-side.png`
- `padded-runtime-final-short-spine/ordinary-camera-pitch-0016-front.png`
- `padded-runtime-final-short-spine/ordinary-camera-pitch-0016-side.png`
- `padded-runtime-final-short-spine/ordinary-camera-pitch-0032-front.png`
- `padded-runtime-final-short-spine/ordinary-camera-pitch-0032-side.png`
- `padded-runtime-final-long-spine/ordinary-camera-pitch-0000-front.png`
- `padded-runtime-final-long-spine/ordinary-camera-pitch-0000-side.png`
- `padded-runtime-final-long-spine/ordinary-camera-pitch-0016-front.png`
- `padded-runtime-final-long-spine/ordinary-camera-pitch-0016-side.png`
- `padded-runtime-final-long-spine/ordinary-camera-pitch-0032-front.png`
- `padded-runtime-final-long-spine/ordinary-camera-pitch-0032-side.png`
- `boot-mail-interface/combined-front.png`
- `boot-mail-interface/combined-side.png`
- `plate-runtime-final-flat-walk/flat-grid-walk-2.0-0000-front.png`
- `plate-runtime-final-flat-walk/flat-grid-walk-2.0-0000-side.png`
- `plate-runtime-final-flat-walk/flat-grid-walk-2.0-0016-front.png`
- `plate-runtime-final-flat-walk/flat-grid-walk-2.0-0016-side.png`
- `plate-runtime-final-flat-walk/flat-grid-walk-2.0-0032-front.png`
- `plate-runtime-final-flat-walk/flat-grid-walk-2.0-0032-side.png`

## Original follow-up status (superseded below)

Boot remediation is in progress elsewhere. This report has not reviewed or approved the proposed corrected shaft. Append a new image-backed verdict when final corrected captures are available; preserve this failed evidence as superseded rather than silently replacing it.

## Final boot25 follow-up — PASS within the reviewed scope

The corrected installed boot25 meshes resolve the previously recorded boot/chausse crossing in the actual native runtime images inspected below. Both mail and padded leggings enter a continuous leather shaft at its upper rim; the earlier alternating leather/garment regions down the shaft are absent. Idle front/side views expose the boot silhouette clearly. Guard views preserve the same boundary as the legs bend and separate. No new shaft corrugation, narrow spike, detached island, or broken ankle-to-vamp surface is visible.

This conclusion uses 20 newly viewed native images: mail and padded idle frames 0016/0032 front and side, and guard frames 0016/0032/0054 front and side. It is supported separately by 24 static exported-GLB interface views covering neutral, positive, negative and mixed bodies against both garment types. Those static views show a smooth upper shaft and a continuous ankle-to-vamp transition, with broad angular leather folds near the ankle rather than repeated corrugation. Static views are not substituted for native runtime evidence.

**Final bounded visual verdict: PASS.** The original boot failure above is superseded by this corrected image evidence, not erased. The prior regional findings and coverage reservations remain: connected groin openings under lifted skirts, connected neckline/shoulder and plate heel/joint openings, coarse soft-garment folds, and incomplete visibility of lower feet where terrain occludes them. This review does not certify gait mechanics, the numerical foot-dragging check, every animation frame, or unsampled morph/pose combinations. No geometry edits were made.

### Exact follow-up images actually viewed

Paths here are relative to `target/armor-review/`.

- `boot25-interfaces/neutral-mail_chausses/combined-front.png`
- `boot25-interfaces/neutral-mail_chausses/combined-side.png`
- `boot25-interfaces/neutral-mail_chausses/combined-quarter.png`
- `boot25-interfaces/neutral-padded_chausses/combined-front.png`
- `boot25-interfaces/neutral-padded_chausses/combined-side.png`
- `boot25-interfaces/neutral-padded_chausses/combined-quarter.png`
- `boot25-interfaces/positive-mail_chausses/combined-front.png`
- `boot25-interfaces/positive-mail_chausses/combined-side.png`
- `boot25-interfaces/positive-mail_chausses/combined-quarter.png`
- `boot25-interfaces/positive-padded_chausses/combined-front.png`
- `boot25-interfaces/positive-padded_chausses/combined-side.png`
- `boot25-interfaces/positive-padded_chausses/combined-quarter.png`
- `boot25-interfaces/negative-mail_chausses/combined-front.png`
- `boot25-interfaces/negative-mail_chausses/combined-side.png`
- `boot25-interfaces/negative-mail_chausses/combined-quarter.png`
- `boot25-interfaces/negative-padded_chausses/combined-front.png`
- `boot25-interfaces/negative-padded_chausses/combined-side.png`
- `boot25-interfaces/negative-padded_chausses/combined-quarter.png`
- `boot25-interfaces/mixed-mail_chausses/combined-front.png`
- `boot25-interfaces/mixed-mail_chausses/combined-side.png`
- `boot25-interfaces/mixed-mail_chausses/combined-quarter.png`
- `boot25-interfaces/mixed-padded_chausses/combined-front.png`
- `boot25-interfaces/mixed-padded_chausses/combined-side.png`
- `boot25-interfaces/mixed-padded_chausses/combined-quarter.png`
- `runtime/mail-boot25-final-idle/ordinary-camera-pitch-0016-front.png`
- `runtime/mail-boot25-final-idle/ordinary-camera-pitch-0016-side.png`
- `runtime/mail-boot25-final-idle/ordinary-camera-pitch-0032-front.png`
- `runtime/mail-boot25-final-idle/ordinary-camera-pitch-0032-side.png`
- `runtime/mail-boot25-final-guard/raised-guard-forward-0016-front.png`
- `runtime/mail-boot25-final-guard/raised-guard-forward-0016-side.png`
- `runtime/mail-boot25-final-guard/raised-guard-forward-0032-front.png`
- `runtime/mail-boot25-final-guard/raised-guard-forward-0032-side.png`
- `runtime/mail-boot25-final-guard/raised-guard-forward-0054-front.png`
- `runtime/mail-boot25-final-guard/raised-guard-forward-0054-side.png`
- `runtime/padded-boot25-final-idle/ordinary-camera-pitch-0016-front.png`
- `runtime/padded-boot25-final-idle/ordinary-camera-pitch-0016-side.png`
- `runtime/padded-boot25-final-idle/ordinary-camera-pitch-0032-front.png`
- `runtime/padded-boot25-final-idle/ordinary-camera-pitch-0032-side.png`
- `runtime/padded-boot25-final-guard/raised-guard-forward-0016-front.png`
- `runtime/padded-boot25-final-guard/raised-guard-forward-0016-side.png`
- `runtime/padded-boot25-final-guard/raised-guard-forward-0032-front.png`
- `runtime/padded-boot25-final-guard/raised-guard-forward-0032-side.png`
- `runtime/padded-boot25-final-guard/raised-guard-forward-0054-front.png`
- `runtime/padded-boot25-final-guard/raised-guard-forward-0054-side.png`
