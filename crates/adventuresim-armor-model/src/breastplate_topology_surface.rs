//! Fixed physical-metric topology for the surface-first breastplate.

use std::collections::{BTreeMap, BTreeSet};

use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};

const BOUNDARY_VERTEX_COUNT: usize = 192;
const INTERIOR_SPACING: f32 = 0.0085;

#[path = "breastplate_metric_panel.rs"]
mod metric_panel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BreastplateBoundaryEdge {
    Neck,
    RightShoulder,
    RightArmhole,
    RightSide,
    Waist,
    LeftSide,
    LeftArmhole,
    LeftShoulder,
}

impl BreastplateBoundaryEdge {
    pub const ALL: [Self; 8] = [
        Self::Neck,
        Self::RightShoulder,
        Self::RightArmhole,
        Self::RightSide,
        Self::Waist,
        Self::LeftSide,
        Self::LeftArmhole,
        Self::LeftShoulder,
    ];
    const fn segments(self) -> usize {
        match self {
            Self::Neck => 28,
            Self::RightShoulder | Self::LeftShoulder => 12,
            Self::RightArmhole | Self::LeftArmhole => 28,
            Self::RightSide | Self::LeftSide => 28,
            Self::Waist => 28,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanonicalBoundaryParameters {
    pub neck_width_scale: f32,
    pub armhole_cut: f32,
    pub waist_width_scale: f32,
}
impl Default for CanonicalBoundaryParameters {
    fn default() -> Self {
        Self {
            neck_width_scale: 1.0,
            armhole_cut: 0.0,
            waist_width_scale: 1.0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticBoundaryRange {
    pub edge: BreastplateBoundaryEdge,
    pub start: usize,
    pub segments: usize,
}

#[derive(Clone, Debug)]
pub struct CanonicalBreastplateTopology {
    canonical_positions: Vec<[f32; 2]>,
    chart_positions: Vec<[f32; 2]>,
    indices: Vec<u32>,
    boundary_vertices: Vec<u32>,
    semantic_ranges: Vec<SemanticBoundaryRange>,
    harmonic_neighbors: Vec<Vec<(usize, f32)>>,
    local_ring_stations: Vec<usize>,
    structured_spoke_caps: Vec<StructuredSpokeCap>,
    metric_panel: Option<metric_panel::MetricPanel>,
}

#[derive(Clone, Debug)]
struct StructuredSpokeCap {
    source: [[u32; 3]; 2],
    rows: Vec<Vec<u32>>,
    samples: Vec<StructuredSpokeSample>,
}

#[derive(Clone, Copy, Debug)]
struct StructuredSpokeSample {
    vertex: u32,
    tangent: f32,
    radial: f32,
}

impl CanonicalBreastplateTopology {
    pub fn generate() -> Self {
        let (boundary, semantic_ranges) = canonical_boundary(Default::default());
        Self::from_boundary_and_ranges(boundary, semantic_ranges, &[])
    }

    pub fn from_physical_boundary(boundary: &[[f32; 2]]) -> Self {
        Self::from_physical_boundary_with_candidates(boundary, &[])
    }

    pub fn from_physical_boundary_with_candidates(
        boundary: &[[f32; 2]],
        extra_candidates: &[[f32; 2]],
    ) -> Self {
        assert_eq!(boundary.len(), BOUNDARY_VERTEX_COUNT);
        Self::from_boundary_and_ranges(boundary.to_vec(), Self::semantic_layout(), extra_candidates)
    }

    pub fn from_metric_chart_with_candidates(
        chart_boundary: &[[f32; 2]],
        parameter_boundary: &[[f32; 2]],
        extra_chart_candidates: &[[f32; 2]],
        wrap_cap_along: f32,
        refine_side_curvature: bool,
        chart_to_parameter: impl Fn([f32; 2]) -> [f32; 2],
    ) -> Self {
        assert_eq!(chart_boundary.len(), BOUNDARY_VERTEX_COUNT);
        assert_eq!(parameter_boundary.len(), BOUNDARY_VERTEX_COUNT);
        let inner_boundary = semantic_panel_inner_boundary(chart_boundary);
        let junctions = semantic_cap_indices();
        let cap_groups = semantic_cap_groups();
        let orientation = inner_boundary
            .iter()
            .copied()
            .zip(inner_boundary.iter().copied().cycle().skip(1))
            .take(inner_boundary.len())
            .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
            .sum::<f32>()
            .signum();
        let cap_points_by_group = cap_groups
            .iter()
            .map(|[start, end]| {
                let anchor = (start + BOUNDARY_VERTEX_COUNT - 1) % BOUNDARY_VERTEX_COUNT;
                let exit = (end + 1) % BOUNDARY_VERTEX_COUNT;
                let a = inner_boundary[anchor];
                let b = inner_boundary[exit];
                let chord = sub2(b, a);
                let chord_length = dot2(chord, chord).sqrt().max(1e-8);
                let inward = [
                    -chord[1] / chord_length * orientation,
                    chord[0] / chord_length * orientation,
                ];
                let mut height = chord_length
                    * if *start == 180 || *start == 39 {
                        // These two-vertex shoulder/armscye caps fan from a
                        // one-station anchor edge.  Sizing their altitude from
                        // the full three-station chord made the anchor triangle
                        // long and thin as the authored bridge handle rose.
                        0.495
                    } else if *start == 29 {
                        0.475
                    } else {
                        0.48
                    };
                let shoulder_turn = matches!((*start, *end), (39, 40) | (180, 181));
                let count = if shoulder_turn { 2 } else { 1 };
                loop {
                    let candidates = (0..count)
                        .map(|sample| {
                            let along = if count == 1 {
                                if *start == 0 {
                                    wrap_cap_along.clamp(0.20, 0.80)
                                } else {
                                    0.50
                                }
                            } else {
                                (sample + 1) as f32 / (count + 1) as f32
                            };
                            let chord_point =
                                [a[0] + (b[0] - a[0]) * along, a[1] + (b[1] - a[1]) * along];
                            add2(chord_point, [inward[0] * height, inward[1] * height])
                        })
                        .collect::<Vec<_>>();
                    if candidates
                        .iter()
                        .all(|candidate| point_in_polygon(*candidate, chart_boundary))
                        || height <= chord_length * 0.08
                    {
                        break candidates;
                    }
                    height *= 0.72;
                }
            })
            .collect::<Vec<_>>();
        let cap_offsets = cap_points_by_group
            .iter()
            .scan(0usize, |offset, points| {
                let current = *offset;
                *offset += points.len();
                Some(current)
            })
            .collect::<Vec<_>>();
        let cap_points = cap_points_by_group
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        // The first rail remains the complete semantic boundary layer. A
        // second physical-width rail exists only on the high-curvature neck
        // and side spans; it collapses onto the first rail at both endpoints.
        // The CDT follows that deeper rail, while explicit quads/fans fill the
        // annulus without duplicate vertices or T-junctions.
        // Carry the lateral ring through the lower armscye-to-side transition.
        // The former side-only spans began at station 69 / ended at 151, so
        // the final six armscye stations crossed the complete physical-width
        // inset in one radial cell.  The exported matched mesh concentrated
        // 38--40 degrees there even though both adjacent fields were smooth.
        // Extending the same tapered ring is a sampling correction only: the
        // semantic perimeter and evaluated surface remain unchanged.
        let local_ring_spans = [(3, 25, 2.8), (55, 95, 2.3), (125, 165, 2.3)]
            .into_iter()
            .filter(|(start, _, _)| *start == 3 || refine_side_curvature)
            .collect::<Vec<_>>();
        let second_boundary = semantic_panel_inner_boundary(&inner_boundary);
        let mut deep_offset_by_station = vec![None; BOUNDARY_VERTEX_COUNT];
        let mut deep_points = Vec::<[f32; 2]>::new();
        let mut local_ring_stations = Vec::<usize>::new();
        for &(start, end, taper_stations) in &local_ring_spans {
            // The extra side layer resolves the legacy graph field's tight
            // lateral bend. A smooth angular loft has no such feature; its
            // tiny terminal taper becomes a sliver at the coronal metric.
            // Keep the full semantic first strip and neck refinement, but
            // join the CDT directly to the side strip for that surface family.
            for station in start + 1..end {
                // The high-curvature neck needs about three physical stations to
                // grow the second ring without stretching its first/last
                // cells on the broad design extreme.  The longer side spans
                // retain the proven two-station collapse: using three there
                // left a one-third-width terminal fan below ten degrees on
                // the exact matched endpoint.
                let taper = ((station - start) as f32 / taper_stations)
                    .min((end - station) as f32 / taper_stations)
                    .min(1.0);
                deep_offset_by_station[station] = Some(deep_points.len());
                local_ring_stations.push(station);
                deep_points.push(add2(
                    inner_boundary[station],
                    [
                        (second_boundary[station][0] - inner_boundary[station][0]) * taper,
                        (second_boundary[station][1] - inner_boundary[station][1]) * taper,
                    ],
                ));
            }
        }
        // At the bottoms of the U-neck and waist curve, a single same-station
        // spoke makes both neighboring strip cells hinge around one edge.  On
        // the fair 3D crown that concentrates the curve's turn into a
        // 40--55-degree dihedral even though the surrounding cells are sound.
        // Replace each such hinge with a curvature-sized structured cap.  The
        // interior rows have seven tangential samples and several radial
        // layers, so neither the U turn nor the rail-to-rail turn is forced
        // through one short diagonal.  Every sample is evaluated from the
        // same garment field on every morph; no authored perimeter point or
        // surface value is changed.
        let inner_vertex_index = |station: usize| (BOUNDARY_VERTEX_COUNT + station) as u32;
        let deep_vertex_index = |station: usize| {
            deep_offset_by_station[station]
                .map(|offset| (BOUNDARY_VERTEX_COUNT * 2 + cap_points.len() + offset) as u32)
        };
        let mut spoke_sources = vec![
            (
                [
                    [13, 14, 15],
                    [
                        inner_vertex_index(13),
                        inner_vertex_index(14),
                        inner_vertex_index(15),
                    ],
                ],
                4_usize,
            ),
            (
                [
                    [109, 110, 111],
                    [
                        inner_vertex_index(109),
                        inner_vertex_index(110),
                        inner_vertex_index(111),
                    ],
                ],
                4_usize,
            ),
        ];
        if let (Some(previous), Some(center), Some(next)) = (
            deep_vertex_index(13),
            deep_vertex_index(14),
            deep_vertex_index(15),
        ) {
            spoke_sources.push((
                [
                    [
                        inner_vertex_index(13),
                        inner_vertex_index(14),
                        inner_vertex_index(15),
                    ],
                    [previous, center, next],
                ],
                4,
            ));
        }
        let structured_spoke_base =
            BOUNDARY_VERTEX_COUNT * 2 + cap_points.len() + deep_points.len();
        let mut structured_spoke_caps = Vec::<StructuredSpokeCap>::new();
        let mut structured_spoke_points = Vec::<[f32; 2]>::new();
        for (source, radial_layer_cap) in spoke_sources {
            let source_point = |rail: usize, station: usize| {
                chart_positions_for_source(
                    source[rail][station] as usize,
                    chart_boundary,
                    &inner_boundary,
                    &cap_points,
                    &deep_points,
                )
            };
            let radial_span = (0..3)
                .map(|station| distance(source_point(0, station), source_point(1, station)))
                .sum::<f32>()
                / 3.0;
            let tangent_span = (0..2)
                .flat_map(|rail| {
                    (0..2).map(move |station| {
                        distance(source_point(rail, station), source_point(rail, station + 1))
                    })
                })
                .sum::<f32>()
                / 4.0;
            // Five physical cells span the two-station turn.  Bias the first
            // and last interior samples toward their endpoint spokes: the
            // old .4/.8 pair gave the terminal zipper triangle a .4-cell base
            // against a .8-cell diagonal and aspect 9.99 on the broad design.
            // .2/.8 grows that base to .6 without changing row count, seams,
            // or the manifold side-fan construction.
            let tangent_cell = tangent_span / 2.5;
            let radial_layers = ((radial_span / tangent_cell.max(1e-6)).round() as usize)
                .saturating_sub(1)
                .clamp(1, radial_layer_cap);
            let interpolate = |tangent: f32, radial: f32| {
                let first = tangent.floor().min(1.0) as usize;
                let second = first + 1;
                let along = tangent - first as f32;
                let rail_point = |rail: usize| {
                    let a = chart_positions_for_source(
                        source[rail][first] as usize,
                        chart_boundary,
                        &inner_boundary,
                        &cap_points,
                        &deep_points,
                    );
                    let b = chart_positions_for_source(
                        source[rail][second] as usize,
                        chart_boundary,
                        &inner_boundary,
                        &cap_points,
                        &deep_points,
                    );
                    add2(a, scale2(sub2(b, a), along))
                };
                let outer = rail_point(0);
                let inner = rail_point(1);
                add2(outer, scale2(sub2(inner, outer), radial))
            };
            let mut rows = vec![source[0].to_vec()];
            let mut samples = Vec::<StructuredSpokeSample>::new();
            for layer in 1..=radial_layers {
                let radial = layer as f32 / (radial_layers + 1) as f32;
                let mut row = Vec::<u32>::new();
                for tangent in [0.2_f32, 0.8, 1.2, 1.8] {
                    let vertex = (structured_spoke_base + structured_spoke_points.len()) as u32;
                    structured_spoke_points.push(interpolate(tangent, radial));
                    samples.push(StructuredSpokeSample {
                        vertex,
                        tangent,
                        radial,
                    });
                    row.push(vertex);
                }
                rows.push(row);
            }
            rows.push(source[1].to_vec());
            structured_spoke_caps.push(StructuredSpokeCap {
                source,
                rows,
                samples,
            });
        }
        #[derive(Clone, Copy)]
        enum CdtSource {
            Inner(usize),
            Cap(usize),
            Deep(usize),
        }
        let mut cdt_sources = Vec::<CdtSource>::new();
        let mut cdt_boundary = Vec::<[f32; 2]>::new();
        for (index, inner_point) in inner_boundary
            .iter()
            .enumerate()
            .take(BOUNDARY_VERTEX_COUNT)
        {
            if !junctions.contains(&index) {
                if let Some(offset) = deep_offset_by_station[index] {
                    cdt_sources.push(CdtSource::Deep(offset));
                    cdt_boundary.push(deep_points[offset]);
                } else {
                    cdt_sources.push(CdtSource::Inner(index));
                    cdt_boundary.push(*inner_point);
                }
            }
            for (group_index, [start, _]) in cap_groups.iter().enumerate() {
                let anchor = (start + BOUNDARY_VERTEX_COUNT - 1) % BOUNDARY_VERTEX_COUNT;
                if index == anchor {
                    for (point_index, point) in cap_points_by_group[group_index].iter().enumerate()
                    {
                        cdt_sources.push(CdtSource::Cap(cap_offsets[group_index] + point_index));
                        cdt_boundary.push(*point);
                    }
                }
            }
        }
        assert!(polygon_is_well_separated(&cdt_boundary));
        let mut candidates = cdt_boundary
            .iter()
            .map(|point| Point2::new(point[0] as f64, point[1] as f64))
            .collect::<Vec<_>>();
        let filtered_extras = extra_chart_candidates
            .iter()
            .filter_map(|point| {
                let margin = cdt_boundary
                    .iter()
                    .copied()
                    .zip(cdt_boundary.iter().copied().cycle().skip(1))
                    .take(cdt_boundary.len())
                    .map(|(a, b)| point_segment_distance(*point, a, b))
                    .fold(f32::INFINITY, f32::min);
                (point_in_polygon(*point, &cdt_boundary) && margin > 1e-5)
                    .then_some(Point2::new(point[0] as f64, point[1] as f64))
            })
            .collect::<Vec<_>>();
        candidates.extend(filtered_extras);
        add_staggered_interior_points(&mut candidates, &cdt_boundary);
        let constraints = (0..cdt_boundary.len())
            .map(|index| [index, (index + 1) % cdt_boundary.len()])
            .collect::<Vec<_>>();
        let (cdt_positions, cdt_triangles) =
            triangulate_candidates(&candidates, &constraints, &cdt_boundary);
        let mut chart_positions = chart_boundary.to_vec();
        chart_positions.extend(inner_boundary.iter().copied());
        chart_positions.extend(cap_points.iter().copied());
        chart_positions.extend(deep_points.iter().copied());
        chart_positions.extend(structured_spoke_points.iter().copied());
        chart_positions.extend(cdt_positions[cdt_boundary.len()..].iter().copied());
        let mut triangles = Vec::<[u32; 3]>::with_capacity(
            BOUNDARY_VERTEX_COUNT * 2
                + deep_points.len() * 2
                + junctions.len()
                + cap_groups.len()
                + cdt_triangles.len(),
        );
        for index in 0..BOUNDARY_VERTEX_COUNT {
            let next = (index + 1) % BOUNDARY_VERTEX_COUNT;
            if matches!(index, 13 | 14 | 109 | 110) {
                continue;
            }
            let rail = (BOUNDARY_VERTEX_COUNT + index) as u32;
            let next_rail = (BOUNDARY_VERTEX_COUNT + next) as u32;
            let forward = [
                [index as u32, next as u32, rail],
                [next as u32, next_rail, rail],
            ];
            let reverse = [
                [index as u32, next as u32, next_rail],
                [index as u32, next_rail, rail],
            ];
            let score = |faces: [[u32; 3]; 2]| {
                faces
                    .into_iter()
                    .map(|face| chart_triangle_score(&chart_positions, face))
                    .fold(f32::INFINITY, f32::min)
            };
            triangles.extend(if score(reverse) > score(forward) {
                reverse
            } else {
                forward
            });
        }
        let inner_vertex = |station: usize| (BOUNDARY_VERTEX_COUNT + station) as u32;
        let deep_vertex = |station: usize| {
            deep_offset_by_station[station]
                .map(|offset| (BOUNDARY_VERTEX_COUNT * 2 + cap_points.len() + offset) as u32)
                .unwrap_or_else(|| inner_vertex(station))
        };
        for &(start, end, _) in &local_ring_spans {
            for station in start..end {
                let next = station + 1;
                if matches!(station, 13 | 14) {
                    continue;
                }
                let outer = inner_vertex(station);
                let next_outer = inner_vertex(next);
                let inner = deep_vertex(station);
                let next_inner = deep_vertex(next);
                if outer == inner {
                    triangles.push([outer, next_outer, next_inner]);
                } else if next_outer == next_inner {
                    triangles.push([outer, next_outer, inner]);
                } else {
                    let forward = [[outer, next_outer, inner], [next_outer, next_inner, inner]];
                    let reverse = [[outer, next_outer, next_inner], [outer, next_inner, inner]];
                    let score = |faces: [[u32; 3]; 2]| {
                        faces
                            .into_iter()
                            .map(|face| chart_triangle_score(&chart_positions, face))
                            .fold(f32::INFINITY, f32::min)
                    };
                    triangles.extend(if score(reverse) > score(forward) {
                        reverse
                    } else {
                        forward
                    });
                }
            }
        }
        for cap in &structured_spoke_caps {
            for pair in cap.rows.windows(2) {
                let outer = &pair[0];
                let inner = &pair[1];
                let mut outer_index = 0_usize;
                let mut inner_index = 0_usize;
                while outer_index + 1 < outer.len() || inner_index + 1 < inner.len() {
                    let next_outer = if outer_index + 1 < outer.len() {
                        (outer_index + 1) as f32 / (outer.len() - 1) as f32
                    } else {
                        f32::INFINITY
                    };
                    let next_inner = if inner_index + 1 < inner.len() {
                        (inner_index + 1) as f32 / (inner.len() - 1) as f32
                    } else {
                        f32::INFINITY
                    };
                    if next_outer <= next_inner {
                        triangles.push([
                            outer[outer_index],
                            outer[outer_index + 1],
                            inner[inner_index],
                        ]);
                        outer_index += 1;
                    } else {
                        triangles.push([
                            outer[outer_index],
                            inner[inner_index + 1],
                            inner[inner_index],
                        ]);
                        inner_index += 1;
                    }
                }
            }
            let outer = &cap.rows[0];
            let inner = cap.rows.last().expect("structured cap has inner row");
            let interior = &cap.rows[1..cap.rows.len() - 1];
            let mut left_chain = interior.iter().map(|row| row[0]).collect::<Vec<_>>();
            left_chain.push(inner[0]);
            for pair in left_chain.windows(2) {
                triangles.push([outer[0], pair[0], pair[1]]);
            }
            let mut right_chain = interior
                .iter()
                .map(|row| *row.last().expect("structured row is nonempty"))
                .collect::<Vec<_>>();
            right_chain.push(*inner.last().expect("structured cap has inner endpoint"));
            for pair in right_chain.windows(2) {
                triangles.push([
                    *outer.last().expect("structured cap has outer endpoint"),
                    pair[1],
                    pair[0],
                ]);
            }
        }
        for (group_index, [start, end]) in cap_groups.iter().copied().enumerate() {
            let anchor = (start + BOUNDARY_VERTEX_COUNT - 1) % BOUNDARY_VERTEX_COUNT;
            let cap_base = BOUNDARY_VERTEX_COUNT * 2 + cap_offsets[group_index];
            if cap_points_by_group[group_index].len() == 2 && end == start + 1 {
                // A two-column structured corner patch replaces the old
                // one-apex fan at each shoulder/armscye reflex. Its boundary
                // follows the same physical-arc stations as the semantic rail,
                // so increasing bridge curvature no longer squeezes all cells
                // through one extraordinary vertex.
                let exit = (end + 1) % BOUNDARY_VERTEX_COUNT;
                let inner = |station: usize| (BOUNDARY_VERTEX_COUNT + station) as u32;
                let cap0 = cap_base as u32;
                let cap1 = (cap_base + 1) as u32;
                triangles.push([inner(anchor), inner(start), cap0]);
                let forward = [[inner(start), inner(end), cap0], [inner(end), cap1, cap0]];
                let reverse = [[inner(start), inner(end), cap1], [inner(start), cap1, cap0]];
                let score = |faces: [[u32; 3]; 2]| {
                    faces
                        .into_iter()
                        .map(|face| chart_triangle_score(&chart_positions, face))
                        .fold(f32::INFINITY, f32::min)
                };
                triangles.extend(if score(reverse) > score(forward) {
                    reverse
                } else {
                    forward
                });
                triangles.push([inner(end), inner(exit), cap1]);
            } else {
                let cap = cap_base as u32;
                for step in 0..=(end + BOUNDARY_VERTEX_COUNT - start) % BOUNDARY_VERTEX_COUNT + 1 {
                    let station = (anchor + step) % BOUNDARY_VERTEX_COUNT;
                    let next = (station + 1) % BOUNDARY_VERTEX_COUNT;
                    triangles.push([
                        (BOUNDARY_VERTEX_COUNT + station) as u32,
                        (BOUNDARY_VERTEX_COUNT + next) as u32,
                        cap,
                    ]);
                }
            }
        }
        triangles.extend(cdt_triangles.into_iter().map(|face| {
            face.map(|index| {
                let index = index as usize;
                if index < cdt_boundary.len() {
                    match cdt_sources[index] {
                        CdtSource::Inner(station) => inner_vertex(station),
                        CdtSource::Cap(offset) => (BOUNDARY_VERTEX_COUNT * 2 + offset) as u32,
                        CdtSource::Deep(offset) => {
                            (BOUNDARY_VERTEX_COUNT * 2 + cap_points.len() + offset) as u32
                        }
                    }
                } else {
                    (BOUNDARY_VERTEX_COUNT * 2
                        + cap_points.len()
                        + deep_points.len()
                        + structured_spoke_points.len()
                        + index
                        - cdt_boundary.len()) as u32
                }
            })
        }));
        for face in &mut triangles {
            if signed_triangle_area(
                chart_positions[face[0] as usize],
                chart_positions[face[1] as usize],
                chart_positions[face[2] as usize],
            ) < 0.0
            {
                face.swap(1, 2);
            }
        }
        orient_triangles_consistently(&mut triangles, &chart_positions);
        let mut edge_incidence = BTreeMap::<(u32, u32), usize>::new();
        for face in &triangles {
            for [a, b] in [[face[0], face[1]], [face[1], face[2]], [face[2], face[0]]] {
                *edge_incidence.entry((a.min(b), a.max(b))).or_default() += 1;
            }
        }
        let invalid_edges = edge_incidence
            .iter()
            .filter_map(|(&(a, b), &incidence)| {
                let expected_boundary = a < BOUNDARY_VERTEX_COUNT as u32
                    && b < BOUNDARY_VERTEX_COUNT as u32
                    && ((a + 1) % BOUNDARY_VERTEX_COUNT as u32 == b
                        || (b + 1) % BOUNDARY_VERTEX_COUNT as u32 == a);
                (incidence != if expected_boundary { 1 } else { 2 }).then_some((
                    a,
                    b,
                    incidence,
                    expected_boundary,
                ))
            })
            .collect::<Vec<_>>();
        assert!(
            invalid_edges.is_empty(),
            "invalid structured-panel edge incidence: {invalid_edges:?}"
        );
        let mut canonical_positions = chart_positions
            .iter()
            .copied()
            .map(chart_to_parameter)
            .collect::<Vec<_>>();
        canonical_positions[..BOUNDARY_VERTEX_COUNT].copy_from_slice(parameter_boundary);
        let harmonic_neighbors = harmonic_neighbors(&canonical_positions, &triangles);
        Self {
            canonical_positions,
            chart_positions,
            indices: triangles.into_iter().flatten().collect(),
            boundary_vertices: (0..BOUNDARY_VERTEX_COUNT as u32).collect(),
            semantic_ranges: Self::semantic_layout(),
            harmonic_neighbors,
            local_ring_stations,
            structured_spoke_caps,
            metric_panel: None,
        }
    }

    pub fn semantic_layout() -> Vec<SemanticBoundaryRange> {
        let mut start = 0;
        BreastplateBoundaryEdge::ALL
            .into_iter()
            .map(|edge| {
                let range = SemanticBoundaryRange {
                    edge,
                    start,
                    segments: edge.segments(),
                };
                start += range.segments;
                range
            })
            .collect()
    }

    fn from_boundary_and_ranges(
        boundary: Vec<[f32; 2]>,
        semantic_ranges: Vec<SemanticBoundaryRange>,
        extra_candidates: &[[f32; 2]],
    ) -> Self {
        let mut candidates = boundary
            .iter()
            .map(|p| Point2::new(p[0] as f64, p[1] as f64))
            .collect::<Vec<_>>();
        candidates.extend(
            extra_candidates
                .iter()
                .map(|point| Point2::new(point[0] as f64, point[1] as f64)),
        );
        add_staggered_interior_points(&mut candidates, &boundary);
        let constraints = (0..BOUNDARY_VERTEX_COUNT)
            .map(|i| [i, (i + 1) % BOUNDARY_VERTEX_COUNT])
            .collect::<Vec<_>>();
        let (canonical_positions, triangles) =
            triangulate_candidates(&candidates, &constraints, &boundary);
        let harmonic_neighbors = harmonic_neighbors(&canonical_positions, &triangles);
        let chart_positions = canonical_positions.clone();
        Self {
            canonical_positions,
            chart_positions,
            indices: triangles.into_iter().flatten().collect(),
            boundary_vertices: (0..BOUNDARY_VERTEX_COUNT as u32).collect(),
            semantic_ranges,
            harmonic_neighbors,
            local_ring_stations: Vec::new(),
            structured_spoke_caps: Vec::new(),
            metric_panel: None,
        }
    }
    pub fn canonical_positions(&self) -> &[[f32; 2]] {
        &self.canonical_positions
    }
    pub fn chart_positions(&self) -> &[[f32; 2]] {
        &self.chart_positions
    }
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
    pub fn boundary_vertices(&self) -> &[u32] {
        &self.boundary_vertices
    }

    pub fn is_structured_spoke_sample(&self, vertex: usize) -> bool {
        self.structured_spoke_caps.iter().any(|cap| {
            cap.samples
                .iter()
                .any(|sample| sample.vertex as usize == vertex)
        })
    }

    fn refresh_structured_samples(
        &mut self,
        before_chart: &[[f32; 2]],
        before_canonical: &[[f32; 2]],
    ) {
        // Construction emits derived sites in dependency order. Update only
        // descendants of moved sources, preserving unrelated chart mappings.
        for cap in &self.structured_spoke_caps {
            for sample in &cap.samples {
                let index = sample.vertex as usize;
                debug_assert!(
                    cap.source
                        .iter()
                        .flatten()
                        .all(|&source| source < sample.vertex)
                );
                if cap.source.iter().flatten().any(|&source| {
                    before_chart[source as usize] != self.chart_positions[source as usize]
                }) {
                    self.chart_positions[index] =
                        structured_sample_position(&self.chart_positions, cap.source, *sample);
                }
                if cap.source.iter().flatten().any(|&source| {
                    before_canonical[source as usize] != self.canonical_positions[source as usize]
                }) {
                    self.canonical_positions[index] =
                        structured_sample_position(&self.canonical_positions, cap.source, *sample);
                }
            }
        }
    }

    pub fn inner_rail_vertex(&self, station: usize) -> Option<u32> {
        if self.metric_panel.is_some() {
            return None;
        }
        (station < self.boundary_vertices.len())
            .then_some((self.boundary_vertices.len() + station) as u32)
    }

    pub fn cap_vertex(&self, offset: usize) -> Option<u32> {
        if self.metric_panel.is_some() {
            return None;
        }
        (offset < semantic_cap_vertex_count())
            .then_some((self.boundary_vertices.len() * 2 + offset) as u32)
    }

    pub fn semantic_ranges(&self) -> &[SemanticBoundaryRange] {
        &self.semantic_ranges
    }
    pub fn parameter_positions(&self, p: CanonicalBoundaryParameters) -> Vec<[f32; 2]> {
        let (b, r) = canonical_boundary(p);
        debug_assert_eq!(r, self.semantic_ranges);
        harmonic_extension(&b, &self.harmonic_neighbors)
    }
    pub fn semantic_domain_from_boundary(&self, boundary: &[[f32; 2]]) -> Vec<[f32; 2]> {
        self.semantic_domain_from_boundary_mapped(boundary, |point| point)
    }
    pub fn semantic_domain_from_boundary_mapped(
        &self,
        boundary: &[[f32; 2]],
        map: impl Fn([f32; 2]) -> [f32; 2],
    ) -> Vec<[f32; 2]> {
        assert_eq!(boundary.len(), self.boundary_vertices.len());
        let count = boundary.len();
        assert!(self.canonical_positions.len() >= count * 2);
        if boundary
            .iter()
            .zip(&self.canonical_positions)
            .all(|(target, reference)| target == reference)
        {
            return self.canonical_positions.clone();
        }
        let orientation = |polygon: &[[f32; 2]]| {
            polygon
                .iter()
                .copied()
                .zip(polygon.iter().copied().cycle().skip(1))
                .take(polygon.len())
                .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
                .sum::<f32>()
                .signum()
        };
        let reference_orientation = orientation(&self.canonical_positions[..count]);
        let target_orientation = orientation(boundary);
        let mut pinned = boundary.to_vec();
        for index in 0..count {
            let previous = (index + count - 1) % count;
            let next = (index + 1) % count;
            let reference_tangent = sub2(
                self.canonical_positions[next],
                self.canonical_positions[previous],
            );
            let target_tangent = sub2(boundary[next], boundary[previous]);
            let reference_length = dot2(reference_tangent, reference_tangent).sqrt().max(1e-7);
            let target_length = dot2(target_tangent, target_tangent).sqrt().max(1e-7);
            let reference_unit = [
                reference_tangent[0] / reference_length,
                reference_tangent[1] / reference_length,
            ];
            let target_unit = [
                target_tangent[0] / target_length,
                target_tangent[1] / target_length,
            ];
            let reference_inward = [
                -reference_unit[1] * reference_orientation,
                reference_unit[0] * reference_orientation,
            ];
            let target_inward = [
                -target_unit[1] * target_orientation,
                target_unit[0] * target_orientation,
            ];
            let offset = sub2(
                self.canonical_positions[count + index],
                self.canonical_positions[index],
            );
            let tangent_offset = dot2(offset, reference_unit);
            let inward_offset = dot2(offset, reference_inward);
            let scale = target_length / reference_length;
            pinned.push(add2(
                boundary[index],
                [
                    (target_unit[0] * tangent_offset + target_inward[0] * inward_offset) * scale,
                    (target_unit[1] * tangent_offset + target_inward[1] * inward_offset) * scale,
                ],
            ));
        }
        // Preserve the authored station correspondence on every wearer. A
        // reflex semantic corner can make a tangent-transformed inner station
        // overtake its neighbor, twisting the strip even though both boundary
        // cycles are individually simple. Contract only stations belonging to
        // invalid quads toward their matching outer samples until one legal,
        // consistently oriented diagonal exists for every strip cell.
        for iteration in 0..32 {
            let mapped_outer = boundary.iter().copied().map(&map).collect::<Vec<_>>();
            let mapped_rail = pinned[count..count * 2]
                .iter()
                .copied()
                .map(&map)
                .collect::<Vec<_>>();
            let invalid = invalid_strip_stations(&mapped_outer, &mapped_rail);
            if invalid.is_empty() {
                break;
            }
            for station in invalid {
                pinned[count + station] =
                    local_inset_point(boundary, station, 0.18 * 0.86_f32.powi(iteration));
            }
        }
        let mapped_outer = boundary.iter().copied().map(&map).collect::<Vec<_>>();
        let mapped_rail = pinned[count..count * 2]
            .iter()
            .copied()
            .map(&map)
            .collect::<Vec<_>>();
        let invalid = invalid_strip_stations(&mapped_outer, &mapped_rail);
        assert!(
            invalid.is_empty(),
            "unresolved mapped strip stations {invalid:?}"
        );
        let mut fixed = vec![None; self.harmonic_neighbors.len()];
        for (index, point) in pinned.iter().copied().enumerate() {
            fixed[index] = Some(point);
        }
        let deep_base = count * 2 + semantic_cap_vertex_count();
        for (offset, &station) in self.local_ring_stations.iter().enumerate() {
            let reference_inner = self.canonical_positions[count + station];
            let reference_deep = self.canonical_positions[deep_base + offset];
            let previous = (station + count - 1) % count;
            let next = (station + 1) % count;
            let reference_tangent = sub2(
                self.canonical_positions[count + next],
                self.canonical_positions[count + previous],
            );
            let target_tangent = sub2(pinned[count + next], pinned[count + previous]);
            let reference_length = dot2(reference_tangent, reference_tangent).sqrt().max(1e-7);
            let target_length = dot2(target_tangent, target_tangent).sqrt().max(1e-7);
            let reference_unit = [
                reference_tangent[0] / reference_length,
                reference_tangent[1] / reference_length,
            ];
            let target_unit = [
                target_tangent[0] / target_length,
                target_tangent[1] / target_length,
            ];
            let reference_inward = [
                -reference_unit[1] * reference_orientation,
                reference_unit[0] * reference_orientation,
            ];
            let target_inward = [
                -target_unit[1] * target_orientation,
                target_unit[0] * target_orientation,
            ];
            let local_offset = sub2(reference_deep, reference_inner);
            let tangent_offset = dot2(local_offset, reference_unit);
            let inward_offset = dot2(local_offset, reference_inward);
            let scale = target_length / reference_length;
            fixed[deep_base + offset] = Some(add2(
                pinned[count + station],
                [
                    (target_unit[0] * tangent_offset + target_inward[0] * inward_offset) * scale,
                    (target_unit[1] * tangent_offset + target_inward[1] * inward_offset) * scale,
                ],
            ));
        }
        for cap in &self.structured_spoke_caps {
            for sample in &cap.samples {
                let first = sample.tangent.floor().min(1.0) as usize;
                let second = first + 1;
                let along = sample.tangent - first as f32;
                let rail_point = |rail: usize| {
                    let a = fixed[cap.source[rail][first] as usize]
                        .expect("structured cap source is pinned");
                    let b = fixed[cap.source[rail][second] as usize]
                        .expect("structured cap source is pinned");
                    add2(a, scale2(sub2(b, a), along))
                };
                let outer = rail_point(0);
                let inner = rail_point(1);
                fixed[sample.vertex as usize] =
                    Some(add2(outer, scale2(sub2(inner, outer), sample.radial)));
            }
        }
        harmonic_extension_with_fixed(&fixed, &self.harmonic_neighbors, &self.canonical_positions)
    }
    pub fn reference_hash(&self) -> u64 {
        let mut h = 0xcbf29ce484222325u64;
        for p in &self.canonical_positions {
            for v in p {
                for b in v.to_bits().to_le_bytes() {
                    h ^= b as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }
            }
        }
        for i in &self.indices {
            for b in i.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
        }
        h
    }

    /// Move one non-landmark inner-rail sample in the physical metric chart.
    ///
    /// The explicit semantic strip owns two copies of every perimeter station:
    /// the authored outer curve and a derived inner rail.  Reflex joins can make
    /// one inner cap triangle slightly anisotropic after the chart is evaluated
    /// on a curved 3D wearer.  This bounded operation leaves the semantic outer
    /// curve, station ordering, indices, and vertex identities unchanged; it
    /// only relaxes the derived rail sample before the harmonic interior is
    /// evaluated.
    pub fn relax_inner_rail_station(
        &mut self,
        station: usize,
        tangent_fraction: f32,
        normal_fraction: f32,
    ) -> bool {
        let count = self.boundary_vertices.len();
        if station >= count {
            return false;
        }
        let index = count + station;
        let previous = count + (station + count - 1) % count;
        let next = count + (station + 1) % count;
        let before_chart = self.chart_positions.clone();
        let before_canonical = self.canonical_positions.clone();
        let tangent = sub2(before_chart[next], before_chart[previous]);
        let tangent_length = dot2(tangent, tangent).sqrt().max(1e-8);
        let tangent_unit = [tangent[0] / tangent_length, tangent[1] / tangent_length];
        let orientation = before_chart[..count]
            .iter()
            .copied()
            .zip(before_chart[..count].iter().copied().cycle().skip(1))
            .take(count)
            .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
            .sum::<f32>()
            .signum();
        let normal_unit = [
            -tangent_unit[1] * orientation,
            tangent_unit[0] * orientation,
        ];
        let local_scale = distance(before_chart[index], before_chart[previous])
            .max(distance(before_chart[index], before_chart[next]));
        let delta = [
            local_scale * (tangent_fraction * tangent_unit[0] + normal_fraction * normal_unit[0]),
            local_scale * (tangent_fraction * tangent_unit[1] + normal_fraction * normal_unit[1]),
        ];
        self.chart_positions[index] = add2(before_chart[index], delta);

        // The metric chart uses physical transverse arc and physical height;
        // the garment domain uses normalized lateral position and the same
        // physical height.  Estimate the local transverse Jacobian from the
        // two neighboring rail secants so the relaxation remains meaningful
        // without retaining a runtime image/body-dependent inverse mapper.
        let chart_dx = before_chart[next][0] - before_chart[previous][0];
        let parameter_dx = before_canonical[next][0] - before_canonical[previous][0];
        let lateral_jacobian = if chart_dx.abs() > 1e-7 {
            parameter_dx / chart_dx
        } else {
            1.0
        };
        self.canonical_positions[index] = [
            before_canonical[index][0] + delta[0] * lateral_jacobian,
            before_canonical[index][1] + delta[1],
        ];
        self.refresh_structured_samples(&before_chart, &before_canonical);

        let valid =
            strip_quads_are_injective(
                &self.chart_positions[..count],
                &self.chart_positions[count..count * 2],
            ) && moved_faces_are_valid(&before_canonical, &self.canonical_positions, &self.indices);
        if !valid {
            self.chart_positions = before_chart;
            self.canonical_positions = before_canonical;
            return false;
        }
        let triangles = self.indices.as_chunks::<3>().0.to_vec();
        self.harmonic_neighbors = harmonic_neighbors(&self.canonical_positions, &triangles);
        true
    }

    /// Relax one derived reflex-cap vertex between its adjacent inner-rail
    /// stations. The cap is not an authored landmark: it exists only to make
    /// the concave garment corner manifold, so a bounded physical-metric move
    /// may improve cross-morph triangle quality without changing semantics.
    pub fn relax_cap_vertex(
        &mut self,
        cap_offset: usize,
        first_station: usize,
        last_station: usize,
        tangent_fraction: f32,
        normal_fraction: f32,
    ) -> bool {
        let count = self.boundary_vertices.len();
        let index = count * 2 + cap_offset;
        if index >= self.canonical_positions.len()
            || first_station >= count
            || last_station >= count
        {
            return false;
        }
        let previous = count + (first_station + count - 1) % count;
        let next = count + (last_station + 1) % count;
        let before_chart = self.chart_positions.clone();
        let before_canonical = self.canonical_positions.clone();
        let tangent = sub2(before_chart[next], before_chart[previous]);
        let tangent_length = dot2(tangent, tangent).sqrt().max(1e-8);
        let tangent_unit = [tangent[0] / tangent_length, tangent[1] / tangent_length];
        let orientation = before_chart[..count]
            .iter()
            .copied()
            .zip(before_chart[..count].iter().copied().cycle().skip(1))
            .take(count)
            .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
            .sum::<f32>()
            .signum();
        let normal_unit = [
            -tangent_unit[1] * orientation,
            tangent_unit[0] * orientation,
        ];
        let local_scale = distance(before_chart[index], before_chart[previous])
            .max(distance(before_chart[index], before_chart[next]));
        let delta = [
            local_scale * (tangent_fraction * tangent_unit[0] + normal_fraction * normal_unit[0]),
            local_scale * (tangent_fraction * tangent_unit[1] + normal_fraction * normal_unit[1]),
        ];
        self.chart_positions[index] = add2(before_chart[index], delta);
        let chart_dx = before_chart[next][0] - before_chart[previous][0];
        let parameter_dx = before_canonical[next][0] - before_canonical[previous][0];
        let lateral_jacobian = if chart_dx.abs() > 1e-7 {
            parameter_dx / chart_dx
        } else {
            1.0
        };
        self.canonical_positions[index] = [
            before_canonical[index][0] + delta[0] * lateral_jacobian,
            before_canonical[index][1] + delta[1],
        ];
        self.refresh_structured_samples(&before_chart, &before_canonical);
        let valid =
            moved_faces_are_valid(&before_canonical, &self.canonical_positions, &self.indices);
        if !valid {
            self.chart_positions = before_chart;
            self.canonical_positions = before_canonical;
            return false;
        }
        let triangles = self.indices.as_chunks::<3>().0.to_vec();
        self.harmonic_neighbors = harmonic_neighbors(&self.canonical_positions, &triangles);
        true
    }

    /// Relax one non-semantic CDT/Steiner vertex in the physical chart while
    /// preserving every incident face orientation and the shared topology.
    pub fn relax_free_vertex(
        &mut self,
        index: usize,
        chart_x_fraction: f32,
        chart_y_fraction: f32,
    ) -> bool {
        let count = self.boundary_vertices.len();
        if (if self.metric_panel.is_some() {
            index < count
                || self
                    .metric_panel
                    .as_ref()
                    .unwrap()
                    .pins
                    .iter()
                    .any(|(id, _)| *id == index)
        } else {
            index < count * 2 + semantic_cap_vertex_count()
        }) || index >= self.canonical_positions.len()
            || self.is_structured_spoke_sample(index)
        {
            return false;
        }
        let before_chart = self.chart_positions.clone();
        let before_canonical = self.canonical_positions.clone();
        let neighbors = self.harmonic_neighbors[index]
            .iter()
            .map(|(neighbor, _)| *neighbor)
            .collect::<Vec<_>>();
        if neighbors.len() < 3 {
            return false;
        }
        let local_scale = neighbors
            .iter()
            .map(|neighbor| distance(before_chart[index], before_chart[*neighbor]))
            .sum::<f32>()
            / neighbors.len() as f32;
        let chart_dx = neighbors
            .iter()
            .map(|neighbor| (before_chart[*neighbor][0] - before_chart[index][0]).abs())
            .fold(0.0_f32, f32::max);
        let parameter_dx = neighbors
            .iter()
            .map(|neighbor| (before_canonical[*neighbor][0] - before_canonical[index][0]).abs())
            .fold(0.0_f32, f32::max);
        let lateral_jacobian = if chart_dx > 1e-7 {
            parameter_dx / chart_dx
        } else {
            1.0
        };
        let requested_delta = [
            local_scale * chart_x_fraction,
            local_scale * chart_y_fraction,
        ];
        let mut accepted = false;
        for halving in 0..=10 {
            self.chart_positions.clone_from(&before_chart);
            self.canonical_positions.clone_from(&before_canonical);
            let fraction = 0.5_f32.powi(halving);
            let delta = [requested_delta[0] * fraction, requested_delta[1] * fraction];
            self.chart_positions[index] = add2(before_chart[index], delta);
            self.canonical_positions[index] = [
                before_canonical[index][0] + delta[0] * lateral_jacobian,
                before_canonical[index][1] + delta[1],
            ];
            self.refresh_structured_samples(&before_chart, &before_canonical);
            accepted =
                moved_faces_are_valid(&before_canonical, &self.canonical_positions, &self.indices);
            if accepted {
                break;
            }
        }
        if !accepted {
            self.chart_positions = before_chart;
            self.canonical_positions = before_canonical;
            return false;
        }
        let triangles = self.indices.as_chunks::<3>().0.to_vec();
        self.harmonic_neighbors = harmonic_neighbors(&self.canonical_positions, &triangles);
        true
    }

    /// Move a quality-only CDT site without refitting the analytic garment
    /// field. Rebuild correspondence weights on the moved reference chart so
    /// subsequent morph displacements retain linear precision.
    pub fn relax_free_vertex_preserving_embedding(
        &mut self,
        index: usize,
        chart_x_fraction: f32,
        chart_y_fraction: f32,
    ) -> bool {
        self.relax_free_vertex(index, chart_x_fraction, chart_y_fraction)
    }

    /// Adjust a derived inner boundary layer without refitting the authored
    /// surface; correspondence weights follow the updated reference chart.
    pub fn relax_inner_rail_station_preserving_embedding(
        &mut self,
        station: usize,
        tangent_fraction: f32,
        normal_fraction: f32,
    ) -> bool {
        self.relax_inner_rail_station(station, tangent_fraction, normal_fraction)
    }

    /// Deterministic shared-connectivity refinement scored against every
    /// supplied physical morph surface.
    pub fn optimize_physical_edges(&mut self, surfaces: &[Vec<[f32; 3]>]) -> usize {
        let mut triangles = self.indices.as_chunks::<3>().0.to_vec();
        debug_assert!(inconsistent_oriented_edges(&triangles).is_empty());
        let mut accepted = 0;
        for _ in 0..24 {
            let mut edges = BTreeMap::<(u32, u32), Vec<(usize, u32)>>::new();
            for (face_index, face) in triangles.iter().enumerate() {
                for (a, b, opposite) in [
                    (face[0], face[1], face[2]),
                    (face[1], face[2], face[0]),
                    (face[2], face[0], face[1]),
                ] {
                    edges
                        .entry((a.min(b), a.max(b)))
                        .or_default()
                        .push((face_index, opposite));
                }
            }
            let mut existing_edges = edges.keys().copied().collect::<BTreeSet<_>>();
            let mut used = BTreeSet::new();
            let mut changed = 0;
            for ((a, b), adjacent) in edges {
                if adjacent.len() != 2
                    || used.contains(&adjacent[0].0)
                    || used.contains(&adjacent[1].0)
                {
                    continue;
                }
                let boundary = self.boundary_vertices.len() as u32;
                // Lock only actual garment-panel constraints.  The former
                // `a < 2 * boundary || b < 2 * boundary` guard also locked
                // every unconstrained spoke from the inner rail into the CDT.
                // Those spokes are precisely where a short shoulder sample
                // can otherwise leave an unfixable skinny triangle.
                let consecutive = |left: u32, right: u32| {
                    (left + 1) % boundary == right || (right + 1) % boundary == left
                };
                let a_outer = a < boundary;
                let b_outer = b < boundary;
                let a_inner = (boundary..boundary * 2).contains(&a);
                let b_inner = (boundary..boundary * 2).contains(&b);
                let deep_start = boundary * 2 + semantic_cap_vertex_count() as u32;
                let deep_end = deep_start + self.local_ring_stations.len() as u32;
                let a_deep = (deep_start..deep_end).contains(&a);
                let b_deep = (deep_start..deep_end).contains(&b);
                let structured_cap_vertex = |vertex: u32| {
                    self.structured_spoke_caps
                        .iter()
                        .any(|cap| cap.samples.iter().any(|sample| sample.vertex == vertex))
                };
                if self.metric_panel.is_some() {
                    if a_outer && b_outer && consecutive(a, b) {
                        continue;
                    }
                } else {
                    if (a_outer && b_outer && consecutive(a, b))
                        || (a_inner && b_inner && consecutive(a - boundary, b - boundary))
                        || (a_deep && b_deep)
                        || structured_cap_vertex(a)
                        || structured_cap_vertex(b)
                    {
                        continue;
                    }
                    if (a_inner && b_deep) || (a_deep && b_inner) {
                        let (inner_station, deep_vertex) = if a_inner {
                            (a - boundary, b)
                        } else {
                            (b - boundary, a)
                        };
                        let deep_station =
                            self.local_ring_stations[(deep_vertex - deep_start) as usize] as u32;
                        // Same-station spokes and the deep-to-deep rail are the
                        // semantic strip constraints.  An inner station joined to
                        // the neighboring deep station is only a quad diagonal;
                        // keeping it locked made the first tapered neck cell a
                        // 9.44-degree sliver on the broad acceptance design even
                        // though the other diagonal was legal.  Let the existing
                        // full-surface lexicographic optimizer choose these local
                        // diagonals across every accepted shape.
                        if inner_station == deep_station
                            || !consecutive(inner_station, deep_station)
                        {
                            continue;
                        }
                    }
                    if (a_outer && b_inner) || (b_outer && a_inner) {
                        let (outer, inner) = if a_outer {
                            (a, b - boundary)
                        } else {
                            (b, a - boundary)
                        };
                        // Same-station spokes are strip rails and remain fixed;
                        // adjacent-station spokes are quad diagonals and may flip.
                        if outer == inner || !consecutive(outer, inner) {
                            continue;
                        }
                    } else if a_outer || b_outer {
                        // Do not connect the authored outer silhouette directly
                        // to cap/CDT vertices across its explicit boundary strip.
                        continue;
                    }
                }
                let (left, c) = adjacent[0];
                let (right, d) = adjacent[1];
                if c == d || existing_edges.contains(&(c.min(d), c.max(d))) {
                    continue;
                }
                // A legal diagonal flip requires the old edge's opposite
                // vertices to lie on opposite sides of the proposed edge.
                // Merely checking each resulting triangle for non-zero area
                // admits reflex/overlapping pairs; orienting those triangles
                // independently then gives their shared edge the same
                // traversal direction and creates a locally non-orientable
                // garment panel.
                let c_point = self.canonical_positions[c as usize];
                let d_point = self.canonical_positions[d as usize];
                let a_side =
                    signed_triangle_area(c_point, d_point, self.canonical_positions[a as usize]);
                let b_side =
                    signed_triangle_area(c_point, d_point, self.canonical_positions[b as usize]);
                if a_side.abs() <= 1e-10
                    || b_side.abs() <= 1e-10
                    || a_side.signum() == b_side.signum()
                {
                    continue;
                }
                let mut proposed = [[c, d, a], [d, c, b]];
                if proposed.iter().any(|face| {
                    signed_triangle_area(
                        self.canonical_positions[face[0] as usize],
                        self.canonical_positions[face[1] as usize],
                        self.canonical_positions[face[2] as usize],
                    )
                    .abs()
                        <= 1e-10
                        || surfaces.iter().any(|surface| {
                            let [p0, p1, p2] = face.map(|index| surface[index as usize]);
                            (p1[0] - p0[0]) * (p2[1] - p0[1]) - (p1[1] - p0[1]) * (p2[0] - p0[0])
                                <= 1e-8
                        })
                }) {
                    continue;
                }
                for face in &mut proposed {
                    if signed_triangle_area(
                        self.canonical_positions[face[0] as usize],
                        self.canonical_positions[face[1] as usize],
                        self.canonical_positions[face[2] as usize],
                    ) < 0.0
                    {
                        face.swap(1, 2);
                    }
                }
                let score = |pair: [[u32; 3]; 2]| {
                    surfaces
                        .iter()
                        .fold((180.0_f32, 0.0_f32), |score, surface| {
                            pair.iter().fold(score, |score, face| {
                                let p = face.map(|index| surface[index as usize]);
                                let edge = |x: [f32; 3], y: [f32; 3]| {
                                    ((x[0] - y[0]).powi(2)
                                        + (x[1] - y[1]).powi(2)
                                        + (x[2] - y[2]).powi(2))
                                    .sqrt()
                                };
                                let e = [edge(p[1], p[2]), edge(p[2], p[0]), edge(p[0], p[1])];
                                let s = (e[0] + e[1] + e[2]) * 0.5;
                                let area =
                                    (s * (s - e[0]) * (s - e[1]) * (s - e[2])).max(0.0).sqrt();
                                let aspect = e.iter().copied().fold(0.0_f32, f32::max) * s
                                    / (2.0 * area).max(1e-10);
                                let angle = (0..3)
                                    .map(|i| {
                                        ((e[(i + 1) % 3].powi(2) + e[(i + 2) % 3].powi(2)
                                            - e[i].powi(2))
                                            / (2.0 * e[(i + 1) % 3] * e[(i + 2) % 3]).max(1e-10))
                                        .clamp(-1.0, 1.0)
                                        .acos()
                                        .to_degrees()
                                    })
                                    .fold(180.0_f32, f32::min);
                                (score.0.min(angle), score.1.max(aspect))
                            })
                        })
                };
                let old = score([triangles[left], triangles[right]]);
                let new = score(proposed);
                let violation = |quality: (f32, f32)| {
                    (10.0 - quality.0).max(0.0) * 100.0 + (quality.1 - 9.0).max(0.0) * 10.0
                };
                let old_violation = violation(old);
                let new_violation = violation(new);
                if new_violation < old_violation - 1e-5
                    || ((new_violation - old_violation).abs() <= 1e-5
                        && (new.0 > old.0 + 1e-5
                            || ((new.0 - old.0).abs() <= 1e-5 && new.1 < old.1 - 1e-5)))
                {
                    triangles[left] = proposed[0];
                    triangles[right] = proposed[1];
                    existing_edges.insert((c.min(d), c.max(d)));
                    used.insert(left);
                    used.insert(right);
                    changed += 1;
                    accepted += 1;
                }
            }
            if changed == 0 {
                break;
            }
        }
        orient_triangles_consistently(&mut triangles, &self.canonical_positions);
        debug_assert!(
            inconsistent_oriented_edges(&triangles).is_empty(),
            "optimized inconsistent edges: {:?}",
            inconsistent_oriented_edges(&triangles)
        );
        self.indices = triangles.into_iter().flatten().collect();
        accepted
    }
}

fn structured_sample_position(
    points: &[[f32; 2]],
    source: [[u32; 3]; 2],
    sample: StructuredSpokeSample,
) -> [f32; 2] {
    let first = sample.tangent.floor().min(1.0) as usize;
    let second = first + 1;
    let along = sample.tangent - first as f32;
    let rail_point = |rail: usize| {
        let a = points[source[rail][first] as usize];
        let b = points[source[rail][second] as usize];
        add2(a, scale2(sub2(b, a), along))
    };
    let outer = rail_point(0);
    add2(outer, scale2(sub2(rail_point(1), outer), sample.radial))
}

fn moved_faces_are_valid(before: &[[f32; 2]], after: &[[f32; 2]], indices: &[u32]) -> bool {
    indices.as_chunks::<3>().0.iter().all(|face| {
        if face
            .iter()
            .all(|&index| before[index as usize] == after[index as usize])
        {
            return true;
        }
        let [a, b, c] = face.map(|index| before[index as usize]);
        let old = signed_triangle_area(a, b, c);
        let [a, b, c] = face.map(|index| after[index as usize]);
        let new = signed_triangle_area(a, b, c);
        old * new > 0.0 && new.abs() > 1e-10
    })
}

fn local_inset_point(boundary: &[[f32; 2]], station: usize, scale: f32) -> [f32; 2] {
    let count = boundary.len();
    let point = boundary[station];
    let previous = boundary[(station + count - 1) % count];
    let next = boundary[(station + 1) % count];
    let incoming = sub2(point, previous);
    let outgoing = sub2(next, point);
    let incoming_length = dot2(incoming, incoming).sqrt().max(1e-8);
    let outgoing_length = dot2(outgoing, outgoing).sqrt().max(1e-8);
    let orientation = polygon_orientation(boundary);
    let incoming_normal = [
        -incoming[1] / incoming_length * orientation,
        incoming[0] / incoming_length * orientation,
    ];
    let outgoing_normal = [
        -outgoing[1] / outgoing_length * orientation,
        outgoing[0] / outgoing_length * orientation,
    ];
    let inward = add2(incoming_normal, outgoing_normal);
    let inward_length = dot2(inward, inward).sqrt().max(0.35);
    let inset = incoming_length.min(outgoing_length) * scale;
    [
        point[0] + inward[0] / inward_length * inset,
        point[1] + inward[1] / inward_length * inset,
    ]
}

fn orient_triangles_consistently(triangles: &mut [[u32; 3]], positions: &[[f32; 2]]) {
    let mut edge_faces = BTreeMap::<(u32, u32), Vec<usize>>::new();
    for (face_index, face) in triangles.iter().enumerate() {
        for (from, to) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            edge_faces
                .entry((from.min(to), from.max(to)))
                .or_default()
                .push(face_index);
        }
    }
    debug_assert!(edge_faces.values().all(|faces| faces.len() <= 2));
    let mut visited = vec![false; triangles.len()];
    for seed in 0..triangles.len() {
        if visited[seed] {
            continue;
        }
        visited[seed] = true;
        let mut pending = vec![seed];
        while let Some(face_index) = pending.pop() {
            let face = triangles[face_index];
            for (from, to) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
                let key = (from.min(to), from.max(to));
                for &neighbor in &edge_faces[&key] {
                    if neighbor == face_index || visited[neighbor] {
                        continue;
                    }
                    if directed_edge(triangles[neighbor], from, to) {
                        triangles[neighbor].swap(1, 2);
                    }
                    visited[neighbor] = true;
                    pending.push(neighbor);
                }
            }
        }
    }
    let signed_area = triangles
        .iter()
        .map(|face| {
            signed_triangle_area(
                positions[face[0] as usize],
                positions[face[1] as usize],
                positions[face[2] as usize],
            )
        })
        .sum::<f32>();
    if signed_area < 0.0 {
        for face in triangles {
            face.swap(1, 2);
        }
    }
}

fn directed_edge(face: [u32; 3], from: u32, to: u32) -> bool {
    [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])].contains(&(from, to))
}

fn inconsistent_oriented_edges(triangles: &[[u32; 3]]) -> Vec<((u32, u32), Vec<(u32, u32)>)> {
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for face in triangles {
        for (from, to) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            edges
                .entry((from.min(to), from.max(to)))
                .or_default()
                .push((from, to));
        }
    }
    edges
        .into_iter()
        .filter(|(_, uses)| {
            uses.len() > 2
                || (uses.len() == 2 && !(uses[0].0 == uses[1].1 && uses[0].1 == uses[1].0))
        })
        .collect()
}

/// One inner rail station per semantic boundary station. Corresponding outer
/// and inner samples form explicit garment-pattern quads; only the smoother
/// inner cycle is triangulated.
fn semantic_panel_inner_boundary(boundary: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let twice_area = boundary
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let next = boundary[(index + 1) % boundary.len()];
            point[0] * next[1] - next[0] * point[1]
        })
        .sum::<f32>();
    let orientation = twice_area.signum();
    let build = |height_scales: &[f32]| {
        boundary
            .iter()
            .enumerate()
            .map(|(index, point)| {
                let previous = boundary[(index + boundary.len() - 1) % boundary.len()];
                let next = boundary[(index + 1) % boundary.len()];
                let incoming = [point[0] - previous[0], point[1] - previous[1]];
                let outgoing = [next[0] - point[0], next[1] - point[1]];
                let incoming_length = (incoming[0] * incoming[0] + incoming[1] * incoming[1])
                    .sqrt()
                    .max(1e-8);
                let outgoing_length = (outgoing[0] * outgoing[0] + outgoing[1] * outgoing[1])
                    .sqrt()
                    .max(1e-8);
                let incoming_normal = [
                    -incoming[1] / incoming_length * orientation,
                    incoming[0] / incoming_length * orientation,
                ];
                let outgoing_normal = [
                    -outgoing[1] / outgoing_length * orientation,
                    outgoing[0] / outgoing_length * orientation,
                ];
                let normal_sum = [
                    incoming_normal[0] + outgoing_normal[0],
                    incoming_normal[1] + outgoing_normal[1],
                ];
                let normal_length = (normal_sum[0] * normal_sum[0] + normal_sum[1] * normal_sum[1])
                    .sqrt()
                    .max(0.35);
                let inward = [normal_sum[0] / normal_length, normal_sum[1] / normal_length];
                let inset = incoming_length.min(outgoing_length) * height_scales[index];
                [point[0] + inward[0] * inset, point[1] + inward[1] * inset]
            })
            .collect::<Vec<_>>()
    };
    // Maximize each local strip width independently: a single global width was
    // dictated by the reflex shoulder junction and collapsed otherwise healthy
    // quads around the entire perimeter. Corresponding station normals keep
    // this local adaptation continuous in the same semantic ordering.
    let mut scales = vec![0.86; boundary.len()];
    for index in 0..boundary.len() {
        let shoulder_turn = (32..=46).contains(&index) || (174..=188).contains(&index);
        let regular = [0.86, 0.72, 0.58, 0.46, 0.34, 0.24, 0.16, 0.10, 0.06];
        let corner = [0.34, 0.30, 0.26, 0.22, 0.18, 0.14, 0.10, 0.08, 0.06];
        for &scale in if shoulder_turn {
            &corner[..]
        } else {
            &regular[..]
        } {
            scales[index] = scale;
            if point_in_polygon(build(&scales)[index], boundary) {
                break;
            }
        }
    }
    for _ in 0..12 {
        let mut rail = build(&scales);
        if polygon_is_simple(&rail) && strip_quads_are_injective(boundary, &rail) {
            optimize_semantic_corner_caps(boundary, &mut rail);
            optimize_regular_junction_rails(boundary, &mut rail);
            if strip_quads_are_injective(boundary, &rail) {
                return rail;
            }
        }
        for scale in &mut scales {
            *scale *= 0.82;
        }
    }
    panic!("semantic inner rail must be simple")
}

fn optimize_regular_junction_rails(outer: &[[f32; 2]], rail: &mut [[f32; 2]]) {
    for index in [0usize, 28, 41, 68, 96, 124, 152, 179] {
        let previous = (index + outer.len() - 1) % outer.len();
        let next = (index + 1) % outer.len();
        let local_scale =
            distance(outer[previous], outer[index]).max(distance(outer[index], outer[next]));
        let center = rail[index];
        let mut best = center;
        let mut best_score = regular_station_score(outer, rail, index);
        for yi in -10..=10 {
            for xi in -10..=10 {
                let candidate = [
                    center[0] + xi as f32 * local_scale * 0.04,
                    center[1] + yi as f32 * local_scale * 0.04,
                ];
                if !point_in_polygon(candidate, outer) {
                    continue;
                }
                rail[index] = candidate;
                let reduced = rail
                    .iter()
                    .enumerate()
                    .filter_map(|(station, point)| {
                        (!semantic_cap_indices().contains(&station)).then_some(*point)
                    })
                    .collect::<Vec<_>>();
                if polygon_is_simple(rail)
                    && polygon_is_well_separated(&reduced)
                    && strip_quads_are_injective(outer, rail)
                {
                    let score = regular_station_score(outer, rail, index);
                    if score > best_score {
                        best = candidate;
                        best_score = score;
                    }
                }
            }
        }
        rail[index] = best;
    }
}

fn regular_station_score(outer: &[[f32; 2]], rail: &[[f32; 2]], index: usize) -> f32 {
    let previous = (index + outer.len() - 1) % outer.len();
    let next = (index + 1) % outer.len();
    let orientation = polygon_orientation(outer);
    [
        [outer[previous], outer[index], rail[previous]],
        [outer[index], rail[index], rail[previous]],
        [outer[index], outer[next], rail[index]],
        [outer[next], rail[next], rail[index]],
        [rail[previous], rail[index], rail[next]],
    ]
    .into_iter()
    .map(|triangle| {
        let area = signed_triangle_area(triangle[0], triangle[1], triangle[2]) * orientation;
        if area <= 1e-12 {
            return 0.0;
        }
        let max_edge = distance(triangle[0], triangle[1])
            .max(distance(triangle[1], triangle[2]))
            .max(distance(triangle[2], triangle[0]));
        area / (max_edge * max_edge).max(1e-12)
    })
    .fold(f32::INFINITY, f32::min)
}

fn optimize_semantic_corner_caps(outer: &[[f32; 2]], rail: &mut [[f32; 2]]) {
    for group in semantic_cap_groups() {
        for _ in 0..4 {
            for index in group[0]..=group[1] {
                let previous = (index + outer.len() - 1) % outer.len();
                let next = (index + 1) % outer.len();
                let local_scale = distance(outer[previous], outer[index])
                    .max(distance(outer[index], outer[next]));
                let center = rail[index];
                let mut best = center;
                let mut best_score = cap_group_score(outer, rail, group);
                for yi in -16..=16 {
                    for xi in -16..=16 {
                        let candidate = [
                            center[0] + xi as f32 * local_scale * 0.04,
                            center[1] + yi as f32 * local_scale * 0.04,
                        ];
                        if !point_in_polygon(candidate, outer) {
                            continue;
                        }
                        let old = rail[index];
                        rail[index] = candidate;
                        if polygon_is_simple(rail) && strip_quads_are_injective(outer, rail) {
                            let score = cap_group_score(outer, rail, group);
                            if score > best_score {
                                best = candidate;
                                best_score = score;
                            }
                        }
                        rail[index] = old;
                    }
                }
                rail[index] = best;
            }
        }
    }
}

fn semantic_cap_indices() -> Vec<usize> {
    // The shoulder/armscye joins are the two reflex garment corners. Other
    // semantic joins remain regular rail stations and belong to the convex CDT
    // boundary rather than requiring an explicit cap.
    semantic_cap_groups()
        .into_iter()
        .flat_map(|[start, end]| start..=end)
        .collect()
}

fn semantic_cap_groups() -> Vec<[usize; 2]> {
    vec![[0, 1], [27, 27], [29, 29], [39, 40], [180, 181]]
}

fn semantic_cap_vertex_count() -> usize {
    semantic_cap_groups()
        .into_iter()
        .map(|group| usize::from(matches!(group, [39, 40] | [180, 181])) + 1)
        .sum()
}

fn cap_group_score(outer: &[[f32; 2]], rail: &[[f32; 2]], group: [usize; 2]) -> f32 {
    let anchor = (group[0] + outer.len() - 1) % outer.len();
    let mut triangles = Vec::new();
    for station in std::iter::once(anchor).chain(group[0]..=group[1]) {
        let next = (station + 1) % outer.len();
        triangles.push([outer[station], outer[next], rail[station]]);
        triangles.push([outer[next], rail[next], rail[station]]);
    }
    for station in group[0]..=group[1] {
        let next = (station + 1) % outer.len();
        triangles.push([rail[anchor], rail[station], rail[next]]);
    }
    let orientation = polygon_orientation(outer);
    triangles
        .into_iter()
        .map(|triangle| {
            let area = signed_triangle_area(triangle[0], triangle[1], triangle[2]) * orientation;
            if area <= 1e-12 {
                return 0.0;
            }
            let max_edge = distance(triangle[0], triangle[1])
                .max(distance(triangle[1], triangle[2]))
                .max(distance(triangle[2], triangle[0]));
            area / (max_edge * max_edge).max(1e-12)
        })
        .fold(f32::INFINITY, f32::min)
}

fn polygon_orientation(polygon: &[[f32; 2]]) -> f32 {
    polygon
        .iter()
        .copied()
        .zip(polygon.iter().copied().cycle().skip(1))
        .take(polygon.len())
        .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
        .sum::<f32>()
        .signum()
}

fn strip_quads_are_injective(outer: &[[f32; 2]], rail: &[[f32; 2]]) -> bool {
    invalid_strip_stations(outer, rail).is_empty()
}

fn invalid_strip_stations(outer: &[[f32; 2]], rail: &[[f32; 2]]) -> BTreeSet<usize> {
    let orientation = polygon_orientation(outer);
    (0..outer.len())
        .filter(|&station| {
            let next = (station + 1) % outer.len();
            // Orientation alone permits two neighboring inner stations to
            // converge almost exactly at a reflex corner. Keep the paired
            // rail edge at least a quarter of its semantic boundary edge:
            // the cap optimizer may change spacing, but not collapse a cell.
            // This relative condition is independent of wearer size/units
            // and is also applied when mapping the frozen chart to a morph.
            if distance(rail[station], rail[next]) < 0.25 * distance(outer[station], outer[next]) {
                return true;
            }
            let forward = [
                [outer[station], outer[next], rail[station]],
                [outer[next], rail[next], rail[station]],
            ];
            let reverse = [
                [outer[station], outer[next], rail[next]],
                [outer[station], rail[next], rail[station]],
            ];
            ![forward, reverse].into_iter().all(|faces| {
                faces.into_iter().all(|face| {
                    signed_triangle_area(face[0], face[1], face[2]) * orientation > 1e-12
                })
            })
        })
        .flat_map(|station| [station, (station + 1) % outer.len()])
        .collect()
}

fn chart_triangle_score(positions: &[[f32; 2]], face: [u32; 3]) -> f32 {
    let triangle = face.map(|index| positions[index as usize]);
    let area = signed_triangle_area(triangle[0], triangle[1], triangle[2]).abs();
    let max_edge = distance(triangle[0], triangle[1])
        .max(distance(triangle[1], triangle[2]))
        .max(distance(triangle[2], triangle[0]));
    area / (max_edge * max_edge).max(1e-12)
}

fn polygon_is_simple(polygon: &[[f32; 2]]) -> bool {
    let count = polygon.len();
    for i in 0..count {
        let a = polygon[i];
        let b = polygon[(i + 1) % count];
        for j in (i + 1)..count {
            if j == i || j == (i + 1) % count || (j + 1) % count == i {
                continue;
            }
            let c = polygon[j];
            let d = polygon[(j + 1) % count];
            if segments_intersect(a, b, c, d) {
                return false;
            }
        }
    }
    true
}

fn polygon_is_well_separated(polygon: &[[f32; 2]]) -> bool {
    if !polygon_is_simple(polygon) {
        return false;
    }
    let count = polygon.len();
    for i in 0..count {
        let a = polygon[i];
        let b = polygon[(i + 1) % count];
        for j in (i + 2)..count {
            if (j + 1) % count == i {
                continue;
            }
            let c = polygon[j];
            let d = polygon[(j + 1) % count];
            let clearance = point_segment_distance(a, c, d)
                .min(point_segment_distance(b, c, d))
                .min(point_segment_distance(c, a, b))
                .min(point_segment_distance(d, a, b));
            if clearance < 1e-6 {
                return false;
            }
        }
    }
    true
}

fn segments_intersect(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> bool {
    let ab_c = signed_triangle_area(a, b, c);
    let ab_d = signed_triangle_area(a, b, d);
    let cd_a = signed_triangle_area(c, d, a);
    let cd_b = signed_triangle_area(c, d, b);
    if ab_c * ab_d < -1e-12 && cd_a * cd_b < -1e-12 {
        return true;
    }
    let on_segment = |p: [f32; 2], q: [f32; 2], r: [f32; 2], area: f32| {
        area.abs() <= 1e-9
            && q[0] >= p[0].min(r[0]) - 1e-7
            && q[0] <= p[0].max(r[0]) + 1e-7
            && q[1] >= p[1].min(r[1]) - 1e-7
            && q[1] <= p[1].max(r[1]) + 1e-7
    };
    on_segment(a, c, b, ab_c)
        || on_segment(a, d, b, ab_d)
        || on_segment(c, a, d, cd_a)
        || on_segment(c, b, d, cd_b)
}

fn triangulate_candidates(
    candidates: &[Point2<f64>],
    constraints: &[[usize; 2]],
    boundary: &[[f32; 2]],
) -> (Vec<[f32; 2]>, Vec<[u32; 3]>) {
    triangulate_candidates_with_domain(candidates, constraints, boundary, false)
}

fn triangulate_candidates_with_domain(
    candidates: &[Point2<f64>],
    constraints: &[[usize; 2]],
    boundary: &[[f32; 2]],
    constraint_domain: bool,
) -> (Vec<[f32; 2]>, Vec<[u32; 3]>) {
    let triangulation = ConstrainedDelaunayTriangulation::<Point2<f64>>::bulk_load_cdt(
        candidates.to_vec(),
        constraints.to_vec(),
    )
    .expect("physical breastplate reference domain");
    let positions = triangulation
        .vertices()
        .map(|vertex| {
            let point = vertex.position();
            [point.x as f32, point.y as f32]
        })
        .collect::<Vec<_>>();
    // A rounded f32 centroid can cross a sub-ULP-thin boundary ear. Instead
    // flood the CDT dual graph from its infinite face, crossing only edges
    // which are NOT constrained. This retains the exact authored polygon:
    // neither a thin-face filter nor boundary-coordinate regularization.
    let mut outside = vec![false; triangulation.num_all_faces()];
    if constraint_domain {
        let mut adjacency = vec![Vec::new(); outside.len()];
        for edge in triangulation.directed_edges() {
            if !edge.as_undirected().is_constraint_edge() {
                adjacency[edge.face().fix().index()].push(edge.rev().face().fix().index());
            }
        }
        let outer = triangulation.outer_face().fix().index();
        outside[outer] = true;
        let mut pending = vec![outer];
        while let Some(face) = pending.pop() {
            for &other in &adjacency[face] {
                if !outside[other] {
                    outside[other] = true;
                    pending.push(other);
                }
            }
        }
    }
    let mut triangles = triangulation
        .inner_faces()
        .filter_map(|face| {
            let triangle = face.vertices().map(|vertex| vertex.fix().index() as u32);
            if constraint_domain {
                return (!outside[face.fix().index()]).then_some(triangle);
            }
            let centroid = triangle.iter().fold([0.0; 2], |mut center, index| {
                center[0] += positions[*index as usize][0] / 3.0;
                center[1] += positions[*index as usize][1] / 3.0;
                center
            });
            point_in_polygon(centroid, boundary).then_some(triangle)
        })
        .collect::<Vec<_>>();
    for triangle in &mut triangles {
        if signed_triangle_area(
            positions[triangle[0] as usize],
            positions[triangle[1] as usize],
            positions[triangle[2] as usize],
        ) < 0.0
        {
            triangle.swap(1, 2);
        }
    }
    // `bulk_load_cdt` is free to reorder vertex handles. The first
    // `boundary.len()` candidates nevertheless carry stable semantic IDs in
    // every downstream surface evaluator, so restore that order explicitly
    // and remap triangles before returning. Assuming insertion order here
    // connected semantic boundary vertices to unrelated interior samples as
    // soon as a boundary-layer candidate changed the bulk-load permutation.
    let mut old_to_new = vec![usize::MAX; positions.len()];
    let mut reordered = Vec::with_capacity(positions.len());
    let mut used = vec![false; positions.len()];
    for expected in boundary {
        let (old_index, position) = positions
            .iter()
            .enumerate()
            .filter(|(index, _)| !used[*index])
            .min_by(|(_, left), (_, right)| {
                let left_distance =
                    (left[0] - expected[0]).powi(2) + (left[1] - expected[1]).powi(2);
                let right_distance =
                    (right[0] - expected[0]).powi(2) + (right[1] - expected[1]).powi(2);
                left_distance.total_cmp(&right_distance)
            })
            .expect("CDT retains every semantic boundary vertex");
        used[old_index] = true;
        old_to_new[old_index] = reordered.len();
        reordered.push(*position);
    }
    for (old_index, position) in positions.iter().enumerate() {
        if !used[old_index] {
            old_to_new[old_index] = reordered.len();
            reordered.push(*position);
        }
    }
    for triangle in &mut triangles {
        *triangle = triangle.map(|index| old_to_new[index as usize] as u32);
    }
    (reordered, triangles)
}

fn canonical_boundary(
    parameters: CanonicalBoundaryParameters,
) -> (Vec<[f32; 2]>, Vec<SemanticBoundaryRange>) {
    let waist = [
        0.2003 * parameters.waist_width_scale.clamp(0.82, 1.18),
        0.0936,
    ];
    let mid = [0.2203, 0.41465];
    let shoulder = [0.087, 0.582];
    let neck = [
        0.03456 * parameters.neck_width_scale.clamp(0.75, 1.25),
        0.612,
    ];
    let neck_center = 0.5839;
    let mut boundary = Vec::with_capacity(BOUNDARY_VERTEX_COUNT);
    let mut ranges = Vec::new();
    for edge in BreastplateBoundaryEdge::ALL {
        let segments = edge.segments();
        ranges.push(SemanticBoundaryRange {
            edge,
            start: boundary.len(),
            segments,
        });
        let curve = |t: f32| match edge {
            BreastplateBoundaryEdge::Neck => {
                let centered = 2.0 * t - 1.0;
                [
                    -neck[0] + 2.0 * neck[0] * t,
                    neck_center
                        + (neck[1] - neck_center) * (2.0 * centered * centered - centered.powi(4)),
                ]
            }
            BreastplateBoundaryEdge::RightShoulder => cubic(
                neck,
                [neck[0] + (shoulder[0] - neck[0]) * 0.34, neck[1]],
                [shoulder[0], shoulder[1] + 0.040],
                shoulder,
                t,
            ),
            BreastplateBoundaryEdge::RightArmhole => {
                armhole_curve(shoulder, mid, parameters.armhole_cut, t)
            }
            BreastplateBoundaryEdge::RightSide => cubic(
                mid,
                [mid[0] + 0.004, mid[1] - 0.08],
                [waist[0] + 0.012, waist[1] + 0.08],
                waist,
                t,
            ),
            BreastplateBoundaryEdge::Waist => [
                waist[0] * (1.0 - 2.0 * t),
                waist[1] - 0.004 * (std::f32::consts::PI * t).sin(),
            ],
            BreastplateBoundaryEdge::LeftSide => mirror(cubic(
                waist,
                [waist[0] + 0.012, waist[1] + 0.08],
                [mid[0] + 0.004, mid[1] - 0.08],
                mid,
                t,
            )),
            BreastplateBoundaryEdge::LeftArmhole => mirror(armhole_curve(
                shoulder,
                mid,
                parameters.armhole_cut,
                1.0 - t,
            )),
            BreastplateBoundaryEdge::LeftShoulder => mirror(cubic(
                shoulder,
                [shoulder[0], shoulder[1] + 0.040],
                [neck[0] + (shoulder[0] - neck[0]) * 0.34, neck[1]],
                neck,
                t,
            )),
        };
        let dense = (0..=256)
            .map(|i| curve(i as f32 / 256.0))
            .collect::<Vec<_>>();
        let mut lengths = vec![0.0];
        for pair in dense.windows(2) {
            lengths.push(lengths.last().copied().unwrap() + distance(pair[0], pair[1]));
        }
        let total = *lengths.last().unwrap();
        for sample in 0..segments {
            let fraction = sample as f32 / segments as f32;
            let fraction = if matches!(
                edge,
                BreastplateBoundaryEdge::RightShoulder | BreastplateBoundaryEdge::LeftShoulder
            ) {
                fraction.powf(0.95)
            } else {
                fraction
            };
            let target = total * fraction;
            let upper = lengths.partition_point(|x| *x < target).min(256);
            let lower = upper.saturating_sub(1);
            let q = (target - lengths[lower]) / (lengths[upper] - lengths[lower]).max(1e-8);
            boundary.push([
                dense[lower][0] + (dense[upper][0] - dense[lower][0]) * q,
                dense[lower][1] + (dense[upper][1] - dense[lower][1]) * q,
            ]);
        }
    }
    debug_assert_eq!(boundary.len(), BOUNDARY_VERTEX_COUNT);
    (boundary, ranges)
}
fn cubic(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2], t: f32) -> [f32; 2] {
    let o = 1.0 - t;
    [
        o.powi(3) * a[0] + 3.0 * o * o * t * b[0] + 3.0 * o * t * t * c[0] + t.powi(3) * d[0],
        o.powi(3) * a[1] + 3.0 * o * o * t * b[1] + 3.0 * o * t * t * c[1] + t.powi(3) * d[1],
    ]
}

fn hermite(a: [f32; 2], b: [f32; 2], tangent_a: [f32; 2], tangent_b: [f32; 2], t: f32) -> [f32; 2] {
    let t2 = t * t;
    let t3 = t2 * t;
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    [
        h00 * a[0] + h10 * tangent_a[0] + h01 * b[0] + h11 * tangent_b[0],
        h00 * a[1] + h10 * tangent_a[1] + h01 * b[1] + h11 * tangent_b[1],
    ]
}

fn armhole_curve(shoulder: [f32; 2], mid_axillary: [f32; 2], extra_cut: f32, t: f32) -> [f32; 2] {
    let anterior = [shoulder[0] - 0.004 - extra_cut * 0.20, shoulder[1] - 0.040];
    let cut = [mid_axillary[0] - 0.043 - extra_cut, mid_axillary[1] + 0.035];
    let points = [shoulder, anterior, cut, mid_axillary];
    let tangents = [
        [0.0, -0.080],
        [(cut[0] - shoulder[0]) * 0.5, (cut[1] - shoulder[1]) * 0.5],
        [
            (mid_axillary[0] - anterior[0]) * 0.5,
            (mid_axillary[1] - anterior[1]) * 0.5,
        ],
        [0.012, -0.090],
    ];
    let scaled = t.clamp(0.0, 1.0) * 3.0;
    let segment = (scaled.floor() as usize).min(2);
    hermite(
        points[segment],
        points[segment + 1],
        tangents[segment],
        tangents[segment + 1],
        scaled - segment as f32,
    )
}
fn mirror(p: [f32; 2]) -> [f32; 2] {
    [-p[0], p[1]]
}

fn add_staggered_interior_points(c: &mut Vec<Point2<f64>>, b: &[[f32; 2]]) {
    let min = b
        .iter()
        .fold([f32::INFINITY; 2], |r, p| [r[0].min(p[0]), r[1].min(p[1])]);
    let max = b.iter().fold([f32::NEG_INFINITY; 2], |r, p| {
        [r[0].max(p[0]), r[1].max(p[1])]
    });
    let mut row = 0;
    let mut y = min[1] + INTERIOR_SPACING;
    while y < max[1] - INTERIOR_SPACING * 0.5 {
        let stagger = if row % 2 == 0 {
            0.0
        } else {
            INTERIOR_SPACING * 0.5
        };
        let mut x = min[0] + INTERIOR_SPACING + stagger;
        while x < max[0] - INTERIOR_SPACING * 0.5 {
            let p = [x, y];
            let margin = b
                .iter()
                .zip(b.iter().cycle().skip(1))
                .take(b.len())
                .map(|(a, z)| point_segment_distance(p, *a, *z))
                .fold(f32::INFINITY, f32::min);
            let separated = c.iter().all(|candidate| {
                let dx = candidate.x as f32 - p[0];
                let dy = candidate.y as f32 - p[1];
                dx * dx + dy * dy > (INTERIOR_SPACING * 0.48).powi(2)
            });
            if point_in_polygon(p, b) && margin > INTERIOR_SPACING * 0.32 && separated {
                c.push(Point2::new(x as f64, y as f64));
            }
            x += INTERIOR_SPACING;
        }
        row += 1;
        y += INTERIOR_SPACING * 3.0_f32.sqrt() * 0.5;
    }
}
fn harmonic_neighbors(points: &[[f32; 2]], triangles: &[[u32; 3]]) -> Vec<Vec<(usize, f32)>> {
    // Positive mean-value coordinates reproduce affine functions on each
    // interior one-ring. Inverse-distance averages did not reproduce the
    // reference chart and moved free sites even on a near-identity wearer.
    let mut rows = vec![BTreeMap::<usize, f64>::new(); points.len()];
    for triangle in triangles {
        for corner in 0..3 {
            let center = triangle[corner] as usize;
            let first = triangle[(corner + 1) % 3] as usize;
            let second = triangle[(corner + 2) % 3] as usize;
            let edge = |index: usize| {
                [
                    f64::from(points[index][0]) - f64::from(points[center][0]),
                    f64::from(points[index][1]) - f64::from(points[center][1]),
                ]
            };
            let a = edge(first);
            let b = edge(second);
            let lengths = [a[0].hypot(a[1]), b[0].hypot(b[1])];
            assert!(lengths.iter().all(|length| *length > 0.0));
            let cross = (a[0] * b[1] - a[1] * b[0]).abs();
            let dot = a[0] * b[0] + a[1] * b[1];
            let half_angle = 0.5 * cross.atan2(dot);
            let tangent = half_angle.tan();
            *rows[center].entry(first).or_default() += tangent / lengths[0];
            *rows[center].entry(second).or_default() += tangent / lengths[1];
        }
    }
    rows.into_iter()
        .map(|row| {
            let total = row.values().sum::<f64>();
            assert!(total.is_finite() && total > 0.0);
            row.into_iter()
                .map(|(index, weight)| (index, (weight / total) as f32))
                .collect()
        })
        .collect()
}
fn harmonic_extension<const D: usize>(
    boundary: &[[f32; D]],
    neighbors: &[Vec<(usize, f32)>],
) -> Vec<[f32; D]> {
    let mut values = vec![[0.0; D]; neighbors.len()];
    values[..boundary.len()].copy_from_slice(boundary);
    let center = boundary.iter().fold([0.0; D], |mut r, p| {
        for a in 0..D {
            r[a] += p[a] / boundary.len() as f32;
        }
        r
    });
    values[boundary.len()..].fill(center);
    for _ in 0..1200 {
        let old = values.clone();
        let mut delta = 0.0_f32;
        for v in boundary.len()..values.len() {
            let total = neighbors[v].iter().map(|x| x.1).sum::<f32>();
            for a in 0..D {
                values[v][a] = neighbors[v]
                    .iter()
                    .map(|(n, w)| old[*n][a] * w)
                    .sum::<f32>()
                    / total.max(1e-8);
                delta = delta.max((values[v][a] - old[v][a]).abs());
            }
        }
        if delta < 1e-7 {
            break;
        }
    }
    values
}

fn harmonic_extension_with_fixed<const D: usize>(
    fixed: &[Option<[f32; D]>],
    neighbors: &[Vec<(usize, f32)>],
    reference: &[[f32; D]],
) -> Vec<[f32; D]> {
    assert_eq!(fixed.len(), neighbors.len());
    assert_eq!(fixed.len(), reference.len());
    // Solve the semantic boundary displacement, never a fresh absolute
    // embedding from a centroid. Identity then reproduces the canonical
    // sites exactly; mean-value rows additionally reproduce affine motion.
    let targets = fixed;
    let fixed = targets
        .iter()
        .zip(reference)
        .map(|(target, origin)| {
            target.map(|point| std::array::from_fn(|axis| point[axis] - origin[axis]))
        })
        .collect::<Vec<_>>();
    let fixed_values = fixed.iter().flatten().copied().collect::<Vec<_>>();
    assert!(!fixed_values.is_empty());
    let center = fixed_values.iter().fold([0.0; D], |mut result, point| {
        for axis in 0..D {
            result[axis] += point[axis] / fixed_values.len() as f32;
        }
        result
    });
    let mut values = fixed
        .iter()
        .map(|point| point.unwrap_or(center))
        .collect::<Vec<_>>();
    for _ in 0..1200 {
        let old = values.clone();
        let mut delta = 0.0_f32;
        for vertex in 0..values.len() {
            if fixed[vertex].is_some() {
                continue;
            }
            let total = neighbors[vertex]
                .iter()
                .map(|neighbor| neighbor.1)
                .sum::<f32>();
            for axis in 0..D {
                values[vertex][axis] = neighbors[vertex]
                    .iter()
                    .map(|(neighbor, weight)| old[*neighbor][axis] * weight)
                    .sum::<f32>()
                    / total.max(1e-8);
                delta = delta.max((values[vertex][axis] - old[vertex][axis]).abs());
            }
        }
        if delta < 1e-7 {
            break;
        }
    }
    values
        .into_iter()
        .zip(reference)
        .zip(targets)
        .map(|((delta, origin), target)| {
            target.unwrap_or_else(|| std::array::from_fn(|axis| origin[axis] + delta[axis]))
        })
        .collect()
}

fn point_in_polygon(p: [f32; 2], q: &[[f32; 2]]) -> bool {
    let mut inside = false;
    for (a, b) in q.iter().zip(q.iter().cycle().skip(1)).take(q.len()) {
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
    }
    inside
}
fn point_segment_distance(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let d = sub2(b, a);
    let t = (dot2(sub2(p, a), d) / dot2(d, d).max(1e-8)).clamp(0.0, 1.0);
    distance(p, add2(a, [d[0] * t, d[1] * t]))
}
fn signed_triangle_area(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    cross2(sub2(b, a), sub2(c, a)) * 0.5
}
fn sub2(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn add2(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}
fn scale2(a: [f32; 2], scale: f32) -> [f32; 2] {
    [a[0] * scale, a[1] * scale]
}

fn chart_positions_for_source(
    index: usize,
    outer: &[[f32; 2]],
    inner: &[[f32; 2]],
    caps: &[[f32; 2]],
    deep: &[[f32; 2]],
) -> [f32; 2] {
    if index < BOUNDARY_VERTEX_COUNT {
        outer[index]
    } else if index < BOUNDARY_VERTEX_COUNT * 2 {
        inner[index - BOUNDARY_VERTEX_COUNT]
    } else if index < BOUNDARY_VERTEX_COUNT * 2 + caps.len() {
        caps[index - BOUNDARY_VERTEX_COUNT * 2]
    } else {
        deep[index - BOUNDARY_VERTEX_COUNT * 2 - caps.len()]
    }
}
fn dot2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}
fn cross2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}
fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    dot2(sub2(b, a), sub2(b, a)).sqrt()
}

#[cfg(test)]
#[path = "breastplate_harmonic_regression_tests.rs"]
mod harmonic_regression_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_structured_samples_follow_sources(topology: &CanonicalBreastplateTopology) {
        let count = topology.boundary_vertices.len();
        let identity =
            topology.semantic_domain_from_boundary(&topology.canonical_positions[..count]);
        assert_eq!(identity, topology.canonical_positions);
        let target_boundary = topology.canonical_positions[..count]
            .iter()
            .map(|point| [point[0] * 1.03, point[1] * 0.98 + 0.01])
            .collect::<Vec<_>>();
        let mapped = topology.semantic_domain_from_boundary(&target_boundary);
        for cap in &topology.structured_spoke_caps {
            for &sample in &cap.samples {
                let index = sample.vertex as usize;
                for points in [
                    &topology.canonical_positions,
                    &topology.chart_positions,
                    &mapped,
                ] {
                    assert_eq!(
                        points[index],
                        structured_sample_position(points, cap.source, sample)
                    );
                }
            }
        }
    }

    #[test]
    fn rail_and_deep_moves_update_derived_samples_and_rollback_the_entire_state() {
        let (boundary, _) = canonical_boundary(Default::default());
        let original = CanonicalBreastplateTopology::from_metric_chart_with_candidates(
            &boundary,
            &boundary,
            &[],
            0.5,
            false,
            |point| point,
        );
        assert_structured_samples_follow_sources(&original);
        let count = original.boundary_vertices.len();
        let mut rail_accepted = 0;
        let mut deep_accepted = 0;
        let mut rejected = 0;
        for deep in [false, true] {
            for station in [13, 14, 15] {
                let offset = original
                    .local_ring_stations
                    .iter()
                    .position(|&value| value == station)
                    .unwrap();
                for amount in [-0.06, 0.06, 1e8] {
                    let mut trial = original.clone();
                    let moved = if deep {
                        trial.relax_free_vertex(
                            count * 2 + semantic_cap_vertex_count() + offset,
                            0.0,
                            amount,
                        )
                    } else {
                        trial.relax_inner_rail_station(station, 0.0, amount)
                    };
                    assert_eq!(trial.indices, original.indices);
                    assert_eq!(
                        &trial.canonical_positions[..count],
                        &original.canonical_positions[..count]
                    );
                    if moved {
                        if deep {
                            deep_accepted += 1;
                        } else {
                            rail_accepted += 1;
                        }
                        assert_structured_samples_follow_sources(&trial);
                        assert!(
                            trial
                                .structured_spoke_caps
                                .iter()
                                .flat_map(|cap| &cap.samples)
                                .any(|sample| trial.canonical_positions[sample.vertex as usize]
                                    != original.canonical_positions[sample.vertex as usize])
                        );
                        assert!(moved_faces_are_valid(
                            &original.canonical_positions,
                            &trial.canonical_positions,
                            &trial.indices
                        ));
                    } else {
                        rejected += 1;
                        assert_eq!(trial.canonical_positions, original.canonical_positions);
                        assert_eq!(trial.chart_positions, original.chart_positions);
                        assert_eq!(trial.harmonic_neighbors, original.harmonic_neighbors);
                    }
                }
            }
        }
        assert!(rail_accepted > 0 && deep_accepted > 0 && rejected > 0);
    }

    #[test]
    fn failed_family_neck_fixture_requires_current_source_interpolation() {
        // Exact canonical source coordinates from failed family18. The derived
        // sample had stayed behind while quality search moved its source rails.
        let points = [
            [-0.004391718, 1.4272926],
            [0.0002565818, 1.4271655],
            [0.0047246506, 1.4268435],
            [-0.004698389, 1.4252524],
            [0.0002551077, 1.4250926],
            [0.0046984283, 1.4252524],
        ];
        let sample = StructuredSpokeSample {
            vertex: 6,
            tangent: 0.8,
            radial: 0.5,
        };
        let corrected = structured_sample_position(&points, [[0, 1, 2], [3, 4, 5]], sample);
        let stale = [-0.0009351979, 1.42546];
        assert!(corrected[1] - stale[1] > 0.00069);
        // Restoring the derived point moves it away from the final deep-rail
        // edge, rather than changing the underlying body field or its sources.
        let old_area = signed_triangle_area(points[3], points[4], stale).abs();
        let new_area = signed_triangle_area(points[3], points[4], corrected).abs();
        assert!(new_area > old_area * 2.5);
    }

    #[test]
    fn structured_dependency_refresh_preserves_order_and_unaffected_samples() {
        let mut topology = CanonicalBreastplateTopology::generate();
        let count = topology.boundary_vertices.len() as u32;
        let deep_start = count * 2 + semantic_cap_vertex_count() as u32;
        let deep_end = deep_start + topology.local_ring_stations.len() as u32;
        for cap in &topology.structured_spoke_caps {
            assert!(
                cap.source
                    .iter()
                    .flatten()
                    .all(|&source| source < count * 2 || (deep_start..deep_end).contains(&source))
            );
            assert!(cap.samples.iter().all(|sample| {
                cap.source
                    .iter()
                    .flatten()
                    .all(|&source| source < sample.vertex)
            }));
        }
        // A compact two-level dependency graph also verifies future earlier-
        // derived sources are propagated before their dependent is evaluated.
        topology.canonical_positions = vec![
            [0., 0.],
            [1., 0.],
            [2., 0.],
            [0., 1.],
            [1., 1.],
            [2., 1.],
            [0.5, 0.5],
            [0.5, 0.25],
            [0.7, 0.8],
        ];
        topology.chart_positions = topology.canonical_positions.clone();
        topology.structured_spoke_caps = vec![
            StructuredSpokeCap {
                source: [[0, 1, 2], [3, 4, 5]],
                rows: vec![],
                samples: vec![StructuredSpokeSample {
                    vertex: 6,
                    tangent: 0.5,
                    radial: 0.5,
                }],
            },
            StructuredSpokeCap {
                source: [[0, 1, 2], [6, 6, 6]],
                rows: vec![],
                samples: vec![StructuredSpokeSample {
                    vertex: 7,
                    tangent: 0.5,
                    radial: 0.5,
                }],
            },
            StructuredSpokeCap {
                source: [[0, 1, 2], [0, 1, 2]],
                rows: vec![],
                samples: vec![StructuredSpokeSample {
                    vertex: 8,
                    tangent: 0.5,
                    radial: 0.5,
                }],
            },
        ];
        let before = topology.canonical_positions.clone();
        topology.canonical_positions[3][1] += 0.1;
        topology.chart_positions[3][1] += 0.1;
        topology.refresh_structured_samples(&before, &before);
        for cap in &topology.structured_spoke_caps[..2] {
            let sample = cap.samples[0];
            assert_eq!(
                topology.canonical_positions[sample.vertex as usize],
                structured_sample_position(&topology.canonical_positions, cap.source, sample)
            );
        }
        assert_ne!(topology.canonical_positions[7], before[7]);
        assert_eq!(topology.canonical_positions[8], before[8]);
        assert_eq!(topology.chart_positions, topology.canonical_positions);
    }

    #[test]
    fn reference_precision_survives_accepted_quality_moves() {
        let (boundary, _) = canonical_boundary(Default::default());
        let mut topology = CanonicalBreastplateTopology::from_metric_chart_with_candidates(
            &boundary,
            &boundary,
            &[],
            0.5,
            false,
            |point| point,
        );
        let rail_moved = (0..BOUNDARY_VERTEX_COUNT).any(|station| {
            topology.relax_inner_rail_station_preserving_embedding(station, 0.02, 0.0)
        });
        let free_moved = (2 * BOUNDARY_VERTEX_COUNT..topology.canonical_positions.len())
            .any(|index| topology.relax_free_vertex_preserving_embedding(index, 0.02, 0.0));
        assert!(rail_moved && free_moved);
        let reference = &topology.canonical_positions;
        // This local residual catches stale pre-relaxation weights without
        // mistaking the bounded Jacobi iteration for an exact global solve.
        for vertex in 2 * BOUNDARY_VERTEX_COUNT..reference.len() {
            let average = topology.harmonic_neighbors[vertex].iter().fold(
                [0.0_f64; 2],
                |mut value, (neighbor, weight)| {
                    for axis in 0..2 {
                        value[axis] += f64::from(reference[*neighbor][axis]) * f64::from(*weight);
                    }
                    value
                },
            );
            for axis in 0..2 {
                assert!(
                    (average[axis] - f64::from(reference[vertex][axis])).abs() < 2e-6,
                    "vertex {vertex}, axis {axis}"
                );
            }
        }
        let fixed = reference
            .iter()
            .enumerate()
            .map(|(index, point)| (index < 2 * BOUNDARY_VERTEX_COUNT).then_some(*point))
            .collect::<Vec<_>>();
        assert_eq!(
            harmonic_extension_with_fixed(&fixed, &topology.harmonic_neighbors, reference),
            *reference
        );
        // Small but representable body changes must not take the identity
        // shortcut and silently replace the exact semantic boundary.
        let boundary = reference[..BOUNDARY_VERTEX_COUNT]
            .iter()
            .map(|p| [p[0] + 2e-7, p[1] - 2e-7])
            .collect::<Vec<_>>();
        assert_ne!(&boundary, &reference[..BOUNDARY_VERTEX_COUNT]);
        let mapped = topology.semantic_domain_from_boundary(&boundary);
        assert_eq!(&mapped[..BOUNDARY_VERTEX_COUNT], &boundary);
    }

    #[test]
    fn mean_value_boundary_extension_preserves_three_dimensional_affine_data() {
        let reference = [[-2., -1.], [3., -1.], [2., 2.], [-1., 3.], [0.2, 0.4]];
        let neighbors =
            harmonic_neighbors(&reference, &[[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]]);
        let affine = |p: [f32; 2]| [p[0] + 0.3 * p[1], 2.0 * p[1], 0.4 * p[0] - p[1] + 0.2];
        let boundary = reference[..4]
            .iter()
            .copied()
            .map(affine)
            .collect::<Vec<_>>();
        let mapped = harmonic_extension(&boundary, &neighbors);
        assert_eq!(&mapped[..4], &boundary);
        for (actual, expected) in mapped[4].iter().zip(affine(reference[4])) {
            assert!((actual - expected).abs() < 5e-7);
        }
        let reference3 = reference.map(affine);
        let targets = reference3
            .iter()
            .enumerate()
            .map(|(index, point)| {
                (index < 4).then_some([
                    point[0] * 0.713 + 0.015,
                    point[1] - 0.317,
                    point[2] * 1.113,
                ])
            })
            .collect::<Vec<_>>();
        let displaced = harmonic_extension_with_fixed(&targets, &neighbors, &reference3);
        for (actual, target) in displaced.iter().zip(targets).take(4) {
            assert_eq!(*actual, target.unwrap());
        }
    }

    #[test]
    fn mean_value_embedding_reproduces_identity_and_affine_maps() {
        let faces = [[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]];
        for center in [[0.2, 0.4], [0.45, 0.7]] {
            let reference = [[-2., -1.], [3., -1.], [2., 2.], [-1., 3.], center];
            let neighbors = harmonic_neighbors(&reference, &faces);
            let reproduced = neighbors[4]
                .iter()
                .fold([0.0; 2], |mut value, (index, weight)| {
                    for axis in 0..2 {
                        value[axis] += reference[*index][axis] * weight;
                    }
                    value
                });
            assert!(distance(reproduced, center) < 2e-7);
            let fixed = reference
                .iter()
                .enumerate()
                .map(|(i, p)| (i < 4).then_some(*p))
                .collect::<Vec<_>>();
            assert_eq!(
                harmonic_extension_with_fixed(&fixed, &neighbors, &reference),
                reference
            );
            let transform = |p: [f32; 2]| {
                [
                    1.2 * p[0] + 0.3 * p[1] + 0.1,
                    -0.2 * p[0] + 0.9 * p[1] - 0.2,
                ]
            };
            let fixed = fixed
                .into_iter()
                .map(|p| p.map(transform))
                .collect::<Vec<_>>();
            let mapped = harmonic_extension_with_fixed(&fixed, &neighbors, &reference);
            for (actual, source) in mapped.iter().zip(reference) {
                assert!(distance(*actual, transform(source)) < 5e-7);
            }
        }
    }
    #[test]
    fn paired_inner_rail_rejects_near_coincident_stations_at_any_scale() {
        for scale in [0.001, 1.0, 1000.0] {
            let outer =
                [[0., 0.], [1., 0.], [1., 1.], [0., 1.]].map(|p| [p[0] * scale, p[1] * scale]);
            let regular = [[0.1, 0.1], [0.9, 0.1], [0.9, 0.9], [0.1, 0.9]]
                .map(|p| [p[0] * scale, p[1] * scale]);
            assert!(invalid_strip_stations(&outer, &regular).is_empty());
            let mut collapsed = regular;
            collapsed[1] = [0.1001 * scale, 0.1 * scale];
            let invalid = invalid_strip_stations(&outer, &collapsed);
            assert!(invalid.contains(&0) && invalid.contains(&1));
        }
    }
    #[test]
    fn accepted_rail_relaxations_preserve_metric_spacing() {
        let (boundary, _) = canonical_boundary(Default::default());
        let reference = CanonicalBreastplateTopology::from_metric_chart_with_candidates(
            &boundary,
            &boundary,
            &[],
            0.5,
            true,
            |point| point,
        );
        let count = reference.boundary_vertices.len();
        let mut accepted = 0;
        let mut rejected = 0;
        for station in [0, 1, 28, 40, 68, 96, 124, 152, 180] {
            for tangent in [-1.0, -0.4, 0.0, 0.4, 1.0] {
                let mut trial = reference.clone();
                if trial.relax_inner_rail_station(station, tangent, 0.0) {
                    accepted += 1;
                    assert!(
                        invalid_strip_stations(
                            &trial.chart_positions[..count],
                            &trial.chart_positions[count..2 * count],
                        )
                        .is_empty()
                    );
                } else {
                    rejected += 1;
                    assert_eq!(trial.chart_positions, reference.chart_positions);
                    assert_eq!(trial.canonical_positions, reference.canonical_positions);
                }
                assert_eq!(trial.indices, reference.indices);
            }
        }
        assert!(
            accepted > 0 && rejected > 0,
            "accepted={accepted} rejected={rejected}; initial invalid={:?}",
            invalid_strip_stations(
                &reference.chart_positions[..count],
                &reference.chart_positions[count..2 * count]
            )
        );
    }
    #[test]
    fn angular_side_without_extra_ring_has_no_duplicate_transition_faces() {
        let (boundary, _) = canonical_boundary(Default::default());
        let topology = CanonicalBreastplateTopology::from_metric_chart_with_candidates(
            &boundary,
            &boundary,
            &[],
            0.5,
            false,
            |point| point,
        );
        assert!(
            topology
                .local_ring_stations
                .iter()
                .all(|station| (4..25).contains(station))
        );
        assert!(
            topology
                .indices
                .as_chunks::<3>()
                .0
                .iter()
                .all(|[a, b, c]| a != b && b != c && a != c)
        );
        assert!(inconsistent_oriented_edges(topology.indices.as_chunks::<3>().0).is_empty());
    }
    #[test]
    fn reference_is_deterministic_and_well_formed() {
        let a = CanonicalBreastplateTopology::generate();
        let b = CanonicalBreastplateTopology::generate();
        assert_eq!(a.canonical_positions, b.canonical_positions);
        assert_eq!(a.indices, b.indices);
        assert_eq!(a.reference_hash(), b.reference_hash());
        assert_eq!(a.reference_hash(), 0xdd57_67f7_da4e_1603);
        assert_eq!(a.boundary_vertices.len(), BOUNDARY_VERTEX_COUNT);
        assert_eq!(
            a.semantic_ranges.iter().map(|r| r.segments).sum::<usize>(),
            BOUNDARY_VERTEX_COUNT
        );
        let mut edges = std::collections::BTreeMap::new();
        for t in a.indices.as_chunks::<3>().0 {
            for (x, y) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                edges
                    .entry((x.min(y), x.max(y)))
                    .or_insert_with(Vec::new)
                    .push((x, y, *t));
            }
        }
        assert!(
            edges
                .values()
                .all(|uses| uses.len() == 1 || uses.len() == 2)
        );
        let inconsistent = edges
            .iter()
            .filter(|(_, uses)| uses.len() == 2 && uses[0].0 == uses[1].0)
            .collect::<Vec<_>>();
        assert!(inconsistent.is_empty(), "{inconsistent:?}");
    }
    #[test]
    fn parameter_boundary_is_exact_and_connectivity_fixed() {
        let t = CanonicalBreastplateTopology::generate();
        for p in [
            CanonicalBoundaryParameters {
                neck_width_scale: 0.75,
                armhole_cut: 0.11,
                waist_width_scale: 0.82,
            },
            CanonicalBoundaryParameters {
                neck_width_scale: 1.25,
                armhole_cut: -0.025,
                waist_width_scale: 1.18,
            },
        ] {
            let q = t.parameter_positions(p);
            let (b, _) = canonical_boundary(p);
            assert_eq!(&q[..BOUNDARY_VERTEX_COUNT], b);
            assert_eq!(q.len(), t.canonical_positions.len());
        }
    }
}
