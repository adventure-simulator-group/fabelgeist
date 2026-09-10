# Independent final close-helmet code review

Verdict: no concrete P1/P2 correctness findings in the final targeted source review. No artistic verdict.

Reviewed ShellExtrusion::InPlane and its validation, from_surface thickening, append/transformed/refit preservation, explicit extrusion callers, current close-helmet profile measurement and validation, independent face clearance, current plate seating/projection, creator fitting dispatch, and the level sight-ray regression with central-bridge negative control. Applicable repository/crates/armor-model/creator AGENTS.md instructions remain followed.

For a unit carrier normal n and unit plane normal p, InPlane uses q = n - p(n dot p), then offsets by thickness * q/(q dot q). This keeps the return within the plane and preserves thickness measured along the source vertex normal. Singular directions and invalid plane normals return errors. The plane is stored per carrier shell, rotated with the anatomical frame, and reused during refitting. Normal extrusion remains explicit at the other callers. Connectivity and component ranges do not depend on the fitted profile or extrusion coordinates.

The earlier radial-spacing concern is addressed in source: layered carrier offsets now follow an estimated surface normal, and close-helmet gauge is explicitly restricted to 1–4 mm. Smooth lower visor projection joins the common seating carrier. The skull/temple split and separate face reserve alter coordinate bounds without adding topology-dependent branches. Existing metadata/export handling continues to receive the same component structure and reference-pose hinges.

Limits: no builds/tests or source edits were performed by this reviewer. Test source was inspected; the coordinator's reported sight-ray and eight representative parameter checks were not independently rerun. Local vertex-normal gauge is not an exact continuous thickness guarantee between arbitrary curved triangles, and finite representative checks do not prove every body/control combination. No new concrete failure requiring remediation was identified. Hinge actuation remains outside scope.
