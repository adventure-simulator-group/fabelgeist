# Material stage correction: Comment and receiving review

Current `stages.rs` SHA-256 `8aeb6a08f00500f384065827975374e7bfb131187c5dd4ae3513044b88e67be7` matches the requested snapshot.
Saved-before SHA-256 `ccbbca63f8954cde1d15934b057edc40dfe382d36c273fed0acd0316f9a2fe2a`.

Independent exact comparison reconstructed the current whole file from the
saved original, then reversed every whole-item move, the apply/validate method
move and the single Ratio constructor. The inverse equals all original bytes.
All docs/attrs/cfg/visibility remain attached, production/helper/test blocks are
preserved, and the three existing cases and assertion population are unchanged.
No source edit or project execution occurred.

P3 STAGES-C1: existing `start_for` prose says “a material edit only repeats the
finish.” That is broader than the actual `same_setup` comparison, which includes
`selection.fabric`; a fabric change selects Placement. Narrow this to an
appearance-only or mail-appearance edit, with the completed-checkpoint
precondition. This is an existing wording issue, not a behavior regression from
the source-order/Ratio delta.

The typed fixture now constructs the same0.9f32 as canonical Math Ratio at its
literal source. `MailWeave.roughness` receives that type directly. The existing
mail-only appearance case still selects Finish; the actual Finish branch reuses
the settled state and does not call simulate. The earlier solver-rerun claim is
withdrawn, not restored.

This review establishes narrow mechanical source equivalence and comment
attachments. Stage physical units, counts, errors, native solver behavior and
whole-family/global clearance remain open. Exact source/read extents and block
hashes are in the sibling JSON. No tests/cases/helpers/assertions/doctests or
project checks were added or run.
