# Candidate 32 focused code review

Verdict: one concrete P2 correctness finding. Earlier artistic acceptance reports were not used as evidence for this candidate. No implementation/docs edits, builds, tests, or artistic assessment performed.

## [P2] Reject sight/breath conflicts before constrained triangulation

Location: crates/adventuresim-armor-model/src/helmets_visor_domain.rs:35–43, the loop validating holes against only the outer shield boundary; related new height/row controls in helmets_close_design.rs.

The new breath-height range permits a breathing row to intersect an eye opening, but validation only checks spacing within the breath pattern, between cheek patterns, and against the outer plate boundary. It never checks eye/breath polygon intersections or their minimum connecting web. These intersecting constraints then reach ConstrainedDelaunayTriangulation::bulk_load_cdt. Spade 2.15.1 explicitly documents a panic when constraint edges overlap; the existing map_err cannot catch it. A user-controlled valid design can therefore panic during creator generation/export instead of returning VisorOpeningSpacing.

Concrete source-derived reproducer: start from CloseHelmetDesign::default(); set breaths to count_per_row=1, rows=2, width=3 mm, length=20 mm, span=20 mm, row_spacing=25 mm, center_offset=30 mm, height=450 permille, inclination=0 degrees; leave rounding and Both sides at their defaults. Every scalar check passes, within-row span is sufficient, and row separation is 25 >= 20 + 3 mm. The upper row center is authored y=32.5 mm, so its vertical sides run through y=27 and y=33 mm at x near +/-30 mm. Those are the two long eye-opening edges for the default 6 mm sight gap centered at y=30. Both breathing rows remain well inside the outer shield boundary. Thus the outer-boundary check does not reject this configuration before crossing constraints are inserted.

Reject intersections/containment and webs below the allowed minimum between every pair of opening polygons before CDT construction. A conflict-aware CDT API can also make the boundary robust, but must reject conflicts rather than silently omit constraints and change the requested holes. Add this configuration as a regression that receives a typed spacing error without panic. This review did not execute the reproducer; the coordinates follow directly from openings(), and the panic contract was verified in the locally installed Spade 2.15.1 source.

## Other reviewed contracts

The independent temple reserve and anatomical maxima preserve the measured lower bounds while the jaw/neck style controls establish minimum widths. These maxima alter coordinates, not row counts. The shield domain depends only on design; its connectivity does not vary with wearer profiles. Nape lames are generated with fixed counts when nape_length is nonzero and appended into the skull component, so a fixed design preserves component ranges across morph endpoints. Setting nape_length to zero is a design/topology change, not a wearer-dependent branch. Hinge axis/origin metadata remains transformed with the head frame and shared by the bevor/visor; it is descriptive, not an implemented articulation. No additional concrete P1/P2 finding was established in these paths.
