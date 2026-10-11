# Focused city installation

Street-scale map views refine the regional terrain with one complete canonical
settlement. The renderer requests the closest observer-admitted settlement
within two kilometres when the vertical view span is at most two kilometres.
Case-site markers never request scenes or reveal actual case locations.

The browser fetches the read-only city endpoint and carries its response as
opaque text. The existing worker pool prepares supported terrain, ground
samples, paving and every exterior building program. It requests no occupied
interiors, furniture, scatter or tactical actors. Native installation admits the
source, known settlement focus, full scene document and exact preparation ticket
before consuming the completed regional-map product.

Building installation queues every primary and distant placement. The existing
GPU city assembler publishes a marker on the exact candidate root. Failure
preserves the previous city buffers; an older owner's readiness flag cannot
acknowledge the new candidate. Successful assembly and render preparation
replace the previous city, independently of actor preparation or replacement.
Status exposes `city_requested`, `city_installation` and `city_visible`.

Canonical playable ground and all configured vista rings use the same producer
as actor scenery. Streets and yards reuse the worker's terrain-clipped batches.
Geographic uploads reverse triangle indices without changing actor mesh
handles. The city root and GPU frame carry local east/up/north metres into the
window's east/up/south frame, including absolute source elevation. Reanchoring
updates the root, one GPU frame buffer and material-coordinate uniforms without
regenerating terrain or buildings. Soil, stone and wear patterns stay fixed in
canonical city coordinates.

The renderer retains the actual displayed ground triangles for camera focus,
pins, selected routes and source connections. Paving and route ribbons clip to
these same faces. Missing city support provides no position; it never
substitutes the ungraded regional elevation under a displayed city.

The initial implementation exchanges whole surfaces. City detail is displayed
only when its configured vista extent covers the entire current regional
window. The settled regional surface stays underneath while fine-ground
pipelines prepare. Broader views or uncovered windows display the regional
surface and retain the hidden city's handles. A future boundary compositor can
extend detail outside this complete-window policy without changing city input,
ground authority or GPU placement.

One displayed city and one temporary installation candidate bound renderer
residency. Cancellation prevents obsolete browser preparations from claiming
residency. Native acknowledgement, rather than command delivery, establishes
the browser's resident city. A matching warm native product reuses its root and
buffers; hiding and returning require no city fetch or regeneration. Failed
detail leaves regional terrain available and waits for an explicit retry.

Initial ground mesh construction and upload still run on the renderer thread.
Traffic masks use a neutral map-owned texture, and exterior shadows are not yet
provided for the map. These are presentation limits, not changes to canonical
placement, grading or actor traffic. Moving immutable ground mesh construction
into the existing worker product and adding staged uploads are available
performance iterations. Neither requires generating all cities on map opening
or creating another renderer.
