# City building variety validation

Worktree: `.worktrees/city-variety`, branch `codex/city-variety`.

## Passing checks

- World schema: 57 library tests, including catchment coverage, seed stability,
  shared specialist eligibility and explicit demand overflow.
- Tactical core: 191 library tests, including housing capacity, non-overlapping
  footprints, connected streets, schema validation and equivalent distant recipes.
- Dispatcher: all six settlement-building tests, including dense playable-pad
  validation, population fallback, repeatability, representation-boundary
  invariance and a fully housed 100,000-person city.
- Dispatcher binary: SDK economy projection and child-spawn claim tests.
- Building generator: five focused settlement checks. The preceding broad run
  passed its other 80 checks and exposed a market-hall regression; the repaired
  regression and focused suite then passed. No structural audit was weakened.
- Scoped Clippy with warnings denied: world schema, building generator, tactical
  core and dispatcher, including all targets.
- Formatting, generated SpacetimeDB binding verification and the committed
  scene fixture generator's `--check` mode.

The dispatcher suite took about 528 seconds on this host while other checks ran.
It repeatedly builds full audited recipe palettes. This is not a single-city
load-time measurement; cold recipe compilation remains a cost of the richer
palette. Residential families have twelve variants, service uses two, and each
recipe is validated once per layout rather than once per lot.

## Review artifact

`cargo run -p adventuresim-tactical-core --bin city-layout-report -- 6500 42`
produced 723 buildings with housing capacity 6,505, no unhoused people and no
unplaced service buildings. `scripts/render_city_layout_report.py` renders its
actual lots, streets and inventory to SVG. The review PNG was inspected for
layout and legibility.

The report is a diagram of generated placement, not a screenshot of the 3D game.
Purpose-specific rooms and the parish-church recipe pass geometry/collision
audits. Water wheels, windmill machinery, millraces, operating industrial
machinery and dated institutional placement are not implemented by this change.

## Repository-wide gate

`just lint` passes generated-binding verification, then stops on existing
Rust-quality debt in the armor model and character creator inherited from
`main`. The checker reports no failures in this change's files. Those unrelated
ceilings and source files were not changed to force a passing global result.

On this Windows host, the checks use the bundled Python through `PYTHON_BIN`
and an absolute, worktree-local `ADVENTURESIM_RUNTIME_ROOT` under
`target/isolated-runtime`. No public database was reset or modified.
