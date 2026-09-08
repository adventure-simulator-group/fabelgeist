//! Diagnostic surface-metric boundary layer with freely triangulated corners.
//! The authored boundary is the only frozen cycle. Derived layer sites are not
//! forced into N+i quads, radial spokes, or historical cap templates.
use super::*;

#[derive(Clone, Debug)]
pub(super) struct MetricSite {
    edge: usize,
    fraction: f32,
    width_ratio: f64,
}

#[derive(Clone, Debug)]
pub(super) struct MetricPanel {
    pub(super) pins: Vec<(usize, MetricSite)>,
    refinement_sites: Vec<usize>,
}

fn distance3(a: [f32; 3], b: [f32; 3]) -> f64 {
    (0..3)
        .map(|i| (f64::from(a[i]) - f64::from(b[i])).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn validate_disk_boundary(
    boundary_count: usize,
    vertex_count: usize,
    triangles: &[[u32; 3]],
) -> Result<(), String> {
    let mut incidence = BTreeMap::<(u32, u32), usize>::new();
    let mut used = BTreeSet::new();
    for t in triangles {
        used.extend(t.iter().copied());
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            *incidence.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let boundary_edges = (0..boundary_count as u32)
        .map(|i| {
            let j = (i + 1) % boundary_count as u32;
            (i.min(j), i.max(j))
        })
        .collect::<BTreeSet<_>>();
    for edge in &boundary_edges {
        if incidence.get(edge) != Some(&1) {
            return Err(format!(
                "Metric panel true boundary edge {edge:?} has incidence {:?}, expected1",
                incidence.get(edge)
            ));
        }
    }
    for (edge, count) in &incidence {
        if !boundary_edges.contains(edge) && *count != 2 {
            return Err(format!(
                "Metric panel non-boundary edge {edge:?} has incidence {count}, expected2"
            ));
        }
    }
    if used.len() != vertex_count
        || vertex_count as isize - incidence.len() as isize + triangles.len() as isize != 1
    {
        return Err("Metric panel has orphan vertices or non-disk Euler characteristic".into());
    }
    Ok(())
}

fn metric(
    q: [f32; 2],
    boundary: &[[f32; 2]],
    field: &impl Fn([f32; 2]) -> Result<[f32; 3], String>,
) -> Result<[[f64; 2]; 2], String> {
    let mut j = [[0.0; 3]; 2];
    let h = 0.0001;
    let center = field(q)?;
    for axis in 0..2 {
        let mut lo = q;
        lo[axis] -= h;
        let mut hi = q;
        hi[axis] += h;
        // One-sided differences at a true trim avoid sampling the field's
        // clamped coronal endpoint. Interior differences remain centered.
        let (a, b, width) = match (
            point_in_polygon(lo, boundary),
            point_in_polygon(hi, boundary),
        ) {
            (true, false) => (field(lo)?, center, h),
            (false, true) => (center, field(hi)?, h),
            _ => (field(lo)?, field(hi)?, 2.0 * h),
        };
        for k in 0..3 {
            j[axis][k] = f64::from(b[k] - a[k]) / f64::from(width);
        }
    }
    let g = std::array::from_fn(|a| {
        std::array::from_fn(|b| (0..3).map(|k| j[a][k] * j[b][k]).sum::<f64>())
    });
    if g[0][0] * g[1][1] - g[0][1] * g[1][0] <= 1e-12 || g.iter().flatten().any(|x| !x.is_finite())
    {
        return Err(format!("Singular metric panel Jacobian at {q:?}: {g:?}"));
    }
    Ok(g)
}

fn conormal(g: [[f64; 2]; 2], t: [f64; 2], orientation: f64) -> [f64; 2] {
    let covector = [-t[1] * orientation, t[0] * orientation];
    let determinant = g[0][0] * g[1][1] - g[0][1] * g[1][0];
    let v = [
        (g[1][1] * covector[0] - g[0][1] * covector[1]) / determinant,
        (-g[1][0] * covector[0] + g[0][0] * covector[1]) / determinant,
    ];
    let norm = (v[0] * covector[0] + v[1] * covector[1]).sqrt();
    [v[0] / norm, v[1] / norm]
}

fn site_position(
    site: &MetricSite,
    boundary: &[[f32; 2]],
    field: &impl Fn([f32; 2]) -> Result<[f32; 3], String>,
) -> Result<[f32; 2], String> {
    let a = boundary[site.edge];
    let b = boundary[(site.edge + 1) % boundary.len()];
    let tangent = [f64::from(b[0] - a[0]), f64::from(b[1] - a[1])];
    let orientation = f64::from(polygon_orientation(boundary));
    let width = distance3(field(a)?, field(b)?) * site.width_ratio;
    let mut q = add2(a, scale2(sub2(b, a), site.fraction));
    for _ in 0..8 {
        let velocity = conormal(metric(q, boundary, field)?, tangent, orientation);
        q = std::array::from_fn(|i| (f64::from(q[i]) + width * velocity[i] / 8.0) as f32);
        if !point_in_polygon(q, boundary) {
            return Err(format!(
                "Metric inward site leaves material: edge={} fraction={} ratio={} q={q:?}",
                site.edge, site.fraction, site.width_ratio
            ));
        }
    }
    Ok(q)
}

impl CanonicalBreastplateTopology {
    pub fn is_metric_panel(&self) -> bool {
        self.metric_panel.is_some()
    }

    pub fn is_metric_site(&self, index: usize) -> bool {
        self.metric_panel
            .as_ref()
            .is_some_and(|p| p.pins.iter().any(|(id, _)| *id == index))
    }

    pub fn quality_move_stencil(&self, index: usize) -> Vec<[f32; 2]> {
        if self.is_metric_site(index) {
            // A small two-parameter stencil: along-edge fraction and relative
            // physical inset width. Includes coupled moves, unlike the old
            // chart-cardinal search which cannot move a stored metric recipe.
            (-2..=2)
                .flat_map(|along| {
                    (-2..=2).filter_map(move |width| {
                        (along != 0 || width != 0)
                            .then_some([along as f32 * 0.1, width as f32 * 0.25])
                    })
                })
                .collect()
        } else {
            vec![[-0.06, 0.0], [0.0, -0.06], [0.0, 0.06], [0.06, 0.0]]
        }
    }

    pub fn quality_search_scales(&self) -> &'static [f32] {
        if self.is_metric_panel() {
            &[1.0, 0.5, 0.25, 0.125]
        } else {
            &[1.0]
        }
    }

    /// Move a derived site's generating recipe, never an authored boundary.
    /// Subsequent wearers reevaluate this SAME edge/fraction/width definition
    /// in their own full metric. The reference and every dependent mapping
    /// therefore describe the accepted recipe rather than a stale pinned point.
    pub fn relax_metric_site_recipe(
        &mut self,
        index: usize,
        fraction_delta: f32,
        width_fraction: f32,
        field: impl Fn([f32; 2]) -> Result<[f32; 3], String>,
    ) -> bool {
        let Some(panel) = &self.metric_panel else {
            return false;
        };
        let Some(offset) = panel.pins.iter().position(|(id, _)| *id == index) else {
            return false;
        };
        let mut site = panel.pins[offset].1.clone();
        site.fraction += fraction_delta;
        site.width_ratio *= 1.0 + f64::from(width_fraction);
        if !(0.0..1.0).contains(&site.fraction)
            || site.fraction == 0.0
            || !site.width_ratio.is_finite()
            || site.width_ratio <= 0.0
        {
            return false;
        }
        let count = self.boundary_vertices.len();
        let Ok(point) = site_position(&site, &self.canonical_positions[..count], &field) else {
            return false;
        };
        if point == self.canonical_positions[index] {
            return false;
        }
        let mut updated = self.canonical_positions.clone();
        updated[index] = point;
        if !moved_faces_are_valid(&self.canonical_positions, &updated, &self.indices) {
            return false;
        }
        self.metric_panel.as_mut().unwrap().pins[offset].1 = site;
        self.canonical_positions = updated;
        self.chart_positions[index] = point;
        self.harmonic_neighbors =
            harmonic_neighbors(&self.canonical_positions, self.indices.as_chunks::<3>().0);
        true
    }

    pub fn from_surface_metric_panel(
        boundary: &[[f32; 2]],
        extra: &[[f32; 2]],
        field: impl Fn([f32; 2]) -> Result<[f32; 3], String>,
    ) -> Result<Self, String> {
        if boundary.len() != BOUNDARY_VERTEX_COUNT {
            return Err("Metric panel boundary count changed".into());
        }
        let mut sites = Vec::<(MetricSite, [f32; 2], [f32; 3])>::new();
        for edge in 0..boundary.len() {
            let length = distance3(
                field(boundary[edge])?,
                field(boundary[(edge + 1) % boundary.len()])?,
            );
            let subdivisions = (length / 0.012).ceil().clamp(1.0, 4.0) as usize;
            for step in 0..subdivisions {
                let mut site = MetricSite {
                    edge,
                    fraction: (step as f32 + 0.5) / subdivisions as f32,
                    width_ratio: 0.70 / subdivisions as f64,
                };
                // Near a narrow reflex turn, use a shorter admissible site, not
                // a collapsed mandatory rail. This optional derived site may
                // be omitted on the reference; the chosen layout then freezes.
                let mut found = None;
                for scale in [1.0, 0.75, 0.5] {
                    site.width_ratio = 0.70 * scale / subdivisions as f64;
                    match site_position(&site, boundary, &field) {
                        Ok(q) => {
                            found = Some(q);
                            break;
                        }
                        Err(e) if e.starts_with("Metric inward site leaves material") => {}
                        Err(e) => return Err(e),
                    }
                }
                let Some(q) = found else {
                    eprintln!(
                        "metric panel omitted optional reflex site: edge={edge} fraction={}",
                        site.fraction
                    );
                    continue;
                };
                let p = field(q)?;
                let separation = (length / subdivisions as f64 * 0.35).min(0.005);
                if sites
                    .iter()
                    .all(|(_, _, old)| distance3(*old, p) > separation)
                {
                    sites.push((site, q, p));
                }
            }
        }
        let mut candidates = boundary
            .iter()
            .chain(sites.iter().map(|(_, q, _)| q))
            .map(|q| Point2::new(f64::from(q[0]), f64::from(q[1])))
            .collect::<Vec<_>>();
        let mut interior = Vec::new();
        add_staggered_interior_points(&mut interior, boundary);
        for q in interior {
            let p2 = [q.x as f32, q.y as f32];
            if !point_in_polygon(p2, boundary) {
                continue;
            }
            let p = field(p2)?;
            if sites.iter().any(|(_, _, old)| distance3(*old, p) < 0.0035) {
                continue;
            }
            // Keep the interior cloud out of the actual derived boundary layer.
            let mut near = false;
            for edge in 0..boundary.len() {
                let a = boundary[edge];
                let b = boundary[(edge + 1) % boundary.len()];
                if point_segment_distance(p2, a, b) < 0.003 {
                    near = true;
                    break;
                }
            }
            if !near {
                candidates.push(q);
            }
        }
        // Explicit quality refinement is not background sampling: do not
        // silently discard near-boundary requests using cloud spacing rules.
        for &q in extra {
            if !q.iter().all(|x| x.is_finite()) || !point_in_polygon(q, boundary) {
                return Err(format!(
                    "Metric panel refinement request outside material: {q:?}"
                ));
            }
            candidates.push(Point2::new(q[0] as f64, q[1] as f64));
        }
        let constraints = (0..boundary.len())
            .map(|i| [i, (i + 1) % boundary.len()])
            .collect::<Vec<_>>();
        let (positions, triangles) =
            triangulate_candidates_with_domain(&candidates, &constraints, boundary, true);
        validate_disk_boundary(boundary.len(), positions.len(), &triangles)?;
        let refinement_sites = extra
            .iter()
            .map(|q| {
                positions
                    .iter()
                    .position(|p| p == q)
                    .ok_or_else(|| format!("CDT lost requested metric refinement site: {q:?}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        eprintln!(
            "metric panel CDT: {} metric sites, {} requested refinement sites retained, {} total vertices",
            sites.len(),
            refinement_sites.len(),
            positions.len()
        );
        let mut pins = Vec::new();
        for (site, q, _) in sites {
            let index = positions
                .iter()
                .position(|p| *p == q)
                .ok_or("CDT lost metric site")?;
            pins.push((index, site));
        }
        pins.sort_by_key(|(i, _)| *i);
        let neighbors = harmonic_neighbors(&positions, &triangles);
        Ok(Self {
            canonical_positions: positions.clone(),
            chart_positions: positions,
            indices: triangles.into_iter().flatten().collect(),
            boundary_vertices: (0..boundary.len() as u32).collect(),
            semantic_ranges: Self::semantic_layout(),
            harmonic_neighbors: neighbors,
            local_ring_stations: Vec::new(),
            structured_spoke_caps: Vec::new(),
            metric_panel: Some(MetricPanel {
                pins,
                refinement_sites,
            }),
        })
    }

    pub fn metric_domain_from_boundary(
        &self,
        boundary: &[[f32; 2]],
        field: impl Fn([f32; 2]) -> Result<[f32; 3], String>,
    ) -> Result<Vec<[f32; 2]>, String> {
        let panel = self.metric_panel.as_ref().ok_or("Not a metric panel")?;
        if boundary.len() != self.boundary_vertices.len() {
            return Err("Metric mapping boundary count differs".into());
        }
        let mut fixed = vec![None; self.canonical_positions.len()];
        for (i, p) in boundary.iter().enumerate() {
            fixed[i] = Some(*p);
        }
        for (index, site) in &panel.pins {
            fixed[*index] = Some(site_position(site, boundary, &field)?);
        }
        // Identical boundary coordinates do not imply an identical metric:
        // depth-only morphs can move inward sites while leaving the trim fixed.
        if fixed
            .iter()
            .zip(&self.canonical_positions)
            .all(|(target, p)| target.is_none_or(|q| q == *p))
        {
            return Ok(self.canonical_positions.clone());
        }
        Ok(harmonic_extension_with_fixed(
            &fixed,
            &self.harmonic_neighbors,
            &self.canonical_positions,
        ))
    }

    pub fn metric_layout_json(&self) -> String {
        let Some(panel) = &self.metric_panel else {
            return "null".into();
        };
        let roles=(0..self.canonical_positions.len()).map(|i| {
            if i<self.boundary_vertices.len() {return format!("{{\"kind\":\"boundary\",\"station\":{i}}}");}
            if let Some((_,s))=panel.pins.iter().find(|(id,_)|*id==i) {
                return format!("{{\"kind\":\"metric-edge-site\",\"edge\":{},\"fraction\":{},\"width_ratio\":{}}}",s.edge,s.fraction,s.width_ratio);
            }
            "{\"kind\":\"interior\"}".into()
        }).collect::<Vec<_>>().join(",");
        let constraints = (0..self.boundary_vertices.len())
            .map(|i| {
                format!(
                    "{{\"edge\":[{i},{}],\"reason\":\"true-boundary\"}}",
                    (i + 1) % self.boundary_vertices.len()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"kind\":\"full-metric-sites-cdt-v1\",\"roles\":[{roles}],\"constraints\":[{constraints}],\"reference_sites\":{:?},\"refinement_sites\":{:?}}}",
            self.canonical_positions, panel.refinement_sites
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn affine(q: [f32; 2]) -> Result<[f32; 3], String> {
        Ok([2.0 * q[0] + 0.8 * q[1], 0.7 * q[1], 0.4 * q[0] - 0.3 * q[1]])
    }

    #[test]
    fn full_metric_conormal_is_physically_orthogonal_and_unit_length() {
        let polygon = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
        let g = metric([0.0, 0.0], &polygon, &affine).unwrap();
        assert!(g[0][1].abs() > 1.0);
        for t in [[1.0, 0.0], [0.0, 1.0], [0.4, 0.7]] {
            let v = conormal(g, t, 1.0);
            let dot =
                t[0] * (g[0][0] * v[0] + g[0][1] * v[1]) + t[1] * (g[1][0] * v[0] + g[1][1] * v[1]);
            let norm =
                v[0] * (g[0][0] * v[0] + g[0][1] * v[1]) + v[1] * (g[1][0] * v[0] + g[1][1] * v[1]);
            assert!(dot.abs() < 1e-12);
            assert!((norm - 1.0).abs() < 1e-12);
        }
        let site = MetricSite {
            edge: 0,
            fraction: 0.5,
            width_ratio: 0.1,
        };
        let q = site_position(&site, &polygon, &affine).unwrap();
        let expected = distance3(affine(polygon[0]).unwrap(), affine(polygon[1]).unwrap()) * 0.1;
        let actual = distance3(affine([0.0, -1.0]).unwrap(), affine(q).unwrap());
        assert!((actual - expected).abs() < 0.0003);
    }

    #[test]
    fn metric_panel_preserves_semantics_identity_roles_and_disk_incidence() {
        let (boundary, _) = canonical_boundary(Default::default());
        let topology =
            CanonicalBreastplateTopology::from_surface_metric_panel(&boundary, &[], affine)
                .unwrap();
        let second =
            CanonicalBreastplateTopology::from_surface_metric_panel(&boundary, &[], affine)
                .unwrap();
        assert_eq!(topology.reference_hash(), second.reference_hash());
        assert_eq!(
            &topology.canonical_positions[..boundary.len()],
            boundary.as_slice()
        );
        assert!(topology.inner_rail_vertex(0).is_none());
        assert!(topology.cap_vertex(0).is_none());
        assert_eq!(
            topology
                .metric_domain_from_boundary(&boundary, affine)
                .unwrap(),
            topology.canonical_positions
        );
        let mut incidence = BTreeMap::new();
        for triangle in topology.indices.as_chunks::<3>().0 {
            assert!(
                signed_triangle_area(
                    topology.canonical_positions[triangle[0] as usize],
                    topology.canonical_positions[triangle[1] as usize],
                    topology.canonical_positions[triangle[2] as usize]
                ) > 0.0
            );
            for (a, b) in [
                (triangle[0], triangle[1]),
                (triangle[1], triangle[2]),
                (triangle[2], triangle[0]),
            ] {
                *incidence.entry((a.min(b), a.max(b))).or_insert(0usize) += 1;
            }
        }
        for ((a, b), count) in &incidence {
            let boundary_edge = (*a as usize + 1) % boundary.len() == *b as usize
                || (*b as usize + 1) % boundary.len() == *a as usize;
            let is_outer =
                *a < boundary.len() as u32 && *b < boundary.len() as u32 && boundary_edge;
            assert_eq!(*count, if is_outer { 1 } else { 2 });
        }
        let roles = topology.metric_layout_json();
        assert!(roles.contains("metric-edge-site"));
        assert_eq!(
            topology.canonical_positions.len() as isize - incidence.len() as isize
                + (topology.indices.len() / 3) as isize,
            1
        );
        let mut changed = topology.clone();
        let positions = changed
            .canonical_positions
            .iter()
            .map(|q| affine(*q).unwrap())
            .collect::<Vec<_>>();
        changed.optimize_physical_edges(&[positions]);
        assert_eq!(roles, changed.metric_layout_json());
    }

    #[test]
    fn same_boundary_with_changed_depth_metric_recomputes_sites_without_remeshing() {
        let (boundary, _) = canonical_boundary(Default::default());
        let topology =
            CanonicalBreastplateTopology::from_surface_metric_panel(&boundary, &[], affine)
                .unwrap();
        let changed =
            |q: [f32; 2]| Ok([2.0 * q[0] + 0.8 * q[1], 0.7 * q[1], 1.8 * q[0] - 0.3 * q[1]]);
        let mapped = topology
            .metric_domain_from_boundary(&boundary, changed)
            .unwrap();
        assert_eq!(mapped.len(), topology.canonical_positions.len());
        assert_eq!(&mapped[..boundary.len()], boundary.as_slice());
        assert!(
            topology
                .metric_panel
                .as_ref()
                .unwrap()
                .pins
                .iter()
                .any(|(i, _)| distance(mapped[*i], topology.canonical_positions[*i]) > 0.0001)
        );
        for (i, site) in &topology.metric_panel.as_ref().unwrap().pins {
            assert_eq!(
                mapped[*i],
                site_position(site, &boundary, &changed).unwrap()
            );
        }
    }

    #[test]
    fn explicit_near_boundary_refinement_is_retained_and_invalid_request_is_error() {
        let (boundary, _) = canonical_boundary(Default::default());
        let original =
            CanonicalBreastplateTopology::from_surface_metric_panel(&boundary, &[], affine)
                .unwrap();
        let (_, site) = &original.metric_panel.as_ref().unwrap().pins[0];
        let origin = add2(
            boundary[site.edge],
            scale2(
                sub2(
                    boundary[(site.edge + 1) % boundary.len()],
                    boundary[site.edge],
                ),
                site.fraction,
            ),
        );
        let end = site_position(site, &boundary, &affine).unwrap();
        let extra = add2(origin, scale2(sub2(end, origin), 0.05));
        assert!(
            point_segment_distance(
                extra,
                boundary[site.edge],
                boundary[(site.edge + 1) % boundary.len()]
            ) < 0.003
        );
        let refined =
            CanonicalBreastplateTopology::from_surface_metric_panel(&boundary, &[extra], affine)
                .unwrap();
        assert!(refined.canonical_positions.contains(&extra));
        assert_eq!(
            refined
                .metric_panel
                .as_ref()
                .unwrap()
                .refinement_sites
                .len(),
            1
        );
        assert!(
            CanonicalBreastplateTopology::from_surface_metric_panel(
                &boundary,
                &[[100.0, 100.0]],
                affine
            )
            .is_err()
        );
    }

    #[test]
    fn constraint_flood_excludes_microscopic_exterior_ears_without_moving_boundary() {
        // A clockwise near-vertical edge with two one-ULP inward stations.
        // f32 centroid classification includes its microscopic exterior ear.
        // A scale/offset variant verifies this is topology, not an area cutoff.
        for (scale, offset) in [(1.0, 0.0), (2.0, 0.5)] {
            let boundary = [
                [-0.27387866, 1.1024563],
                [-0.27387860, 1.1119058],
                [-0.27387860, 1.1213498],
                [-0.27387866, 1.1307884],
                [0.1, 1.1307884],
                [0.1, 1.1024563],
            ]
            .map(|q| q.map(|v| v * scale + offset));
            let candidates = boundary
                .iter()
                .map(|q| Point2::new(q[0] as f64, q[1] as f64))
                .collect::<Vec<_>>();
            let constraints = (0..boundary.len())
                .map(|i| [i, (i + 1) % boundary.len()])
                .collect::<Vec<_>>();
            let (positions, triangles) =
                triangulate_candidates_with_domain(&candidates, &constraints, &boundary, true);
            assert_eq!(&positions[..boundary.len()], boundary.as_slice());
            validate_disk_boundary(boundary.len(), positions.len(), &triangles).unwrap();
            assert!(triangles.iter().all(|t| t.iter().any(|i| *i >= 4)));
        }
        let bad = [[0, 1, 2], [0, 2, 3], [0, 3, 4], [0, 4, 5], [0, 2, 1]];
        assert!(validate_disk_boundary(6, 6, &bad).is_err());
    }

    #[test]
    fn derived_recipe_moves_reference_and_target_without_changing_boundary_or_ids() {
        let (boundary, _) = canonical_boundary(Default::default());
        let original =
            CanonicalBreastplateTopology::from_surface_metric_panel(&boundary, &[], affine)
                .unwrap();
        let mut changed = original.clone();
        let moved = original
            .metric_panel
            .as_ref()
            .unwrap()
            .pins
            .iter()
            .find_map(|(index, _)| {
                for (along, width) in [(0.03, 0.0), (-0.03, 0.0), (0.0, 0.03), (0.0, -0.03)] {
                    if changed.relax_metric_site_recipe(*index, along, width, affine) {
                        return Some(*index);
                    }
                }
                None
            })
            .unwrap();
        assert_ne!(original.reference_hash(), changed.reference_hash());
        assert_ne!(original.metric_layout_json(), changed.metric_layout_json());
        assert_eq!(original.indices, changed.indices);
        assert_eq!(
            &changed.canonical_positions[..boundary.len()],
            boundary.as_slice()
        );
        assert_eq!(
            changed
                .metric_domain_from_boundary(&boundary, affine)
                .unwrap(),
            changed.canonical_positions
        );
        let target =
            |q: [f32; 2]| Ok([2.0 * q[0] + 0.8 * q[1], 0.7 * q[1], 1.8 * q[0] - 0.3 * q[1]]);
        let mapped = changed
            .metric_domain_from_boundary(&boundary, target)
            .unwrap();
        let (_, recipe) = changed
            .metric_panel
            .as_ref()
            .unwrap()
            .pins
            .iter()
            .find(|(i, _)| *i == moved)
            .unwrap();
        assert_eq!(
            mapped[moved],
            site_position(recipe, &boundary, &target).unwrap()
        );
        let snapshot = changed.metric_layout_json();
        let positions = changed.canonical_positions.clone();
        assert!(!changed.relax_metric_site_recipe(moved, 2.0, 0.0, affine));
        assert!(!changed.relax_metric_site_recipe(moved, 0.0, -2.0, affine));
        assert!(!changed.relax_metric_site_recipe(0, 0.01, 0.01, affine));
        assert_eq!(snapshot, changed.metric_layout_json());
        assert_eq!(positions, changed.canonical_positions);
    }

    #[test]
    fn bounded_refinement_preserves_coarse_stencil_order_and_adds_small_legal_recipe_steps() {
        let (boundary, _) = canonical_boundary(Default::default());
        let topology =
            CanonicalBreastplateTopology::from_surface_metric_panel(&boundary, &[], affine)
                .unwrap();
        let index = topology.metric_panel.as_ref().unwrap().pins[0].0;
        let coarse = topology.quality_move_stencil(index);
        assert_eq!(topology.quality_search_scales(), &[1.0, 0.5, 0.25, 0.125]);
        assert_eq!(coarse.len(), 24);
        assert_eq!(
            coarse
                .iter()
                .map(|v| v.map(|x| (x * topology.quality_search_scales()[0]).to_bits()))
                .collect::<Vec<_>>(),
            coarse
                .iter()
                .map(|v| v.map(f32::to_bits))
                .collect::<Vec<_>>()
        );
        let fine = coarse
            .iter()
            .map(|v| v.map(|x| x * 0.25))
            .collect::<Vec<_>>();
        assert!(fine.contains(&[0.025, 0.0]));
        assert!(!coarse.contains(&[0.025, 0.0]));
        assert_eq!(
            CanonicalBreastplateTopology::generate().quality_search_scales(),
            &[1.0]
        );
    }

    #[test]
    #[ignore = "requires BREASTPLATE_METRIC_PANEL_FIXTURE pointing at a real profile.mesh.json"]
    fn actual_metric_panel_candidates_replay_exact_boundary_incidence() {
        let path = std::env::var_os("BREASTPLATE_METRIC_PANEL_FIXTURE").unwrap();
        let data: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let points: Vec<[f32; 2]> = serde_json::from_value(data["domain_arc_y"].clone()).unwrap();
        let count = data["boundary_count"].as_u64().unwrap() as usize;
        let input_faces: Vec<u32> = serde_json::from_value(data["indices"].clone()).unwrap();
        let candidates = points
            .iter()
            .map(|q| Point2::new(q[0] as f64, q[1] as f64))
            .collect::<Vec<_>>();
        let constraints = (0..count).map(|i| [i, (i + 1) % count]).collect::<Vec<_>>();
        let (positions, triangles) =
            triangulate_candidates_with_domain(&candidates, &constraints, &points[..count], true);
        validate_disk_boundary(count, positions.len(), &triangles).unwrap();
        assert_eq!(&positions[..count], &points[..count]);
        assert_eq!(positions.len(), points.len());
        eprintln!(
            "exact candidate replay: {} -> {} faces; {} vertices and {} boundary samples unchanged",
            input_faces.len() / 3,
            triangles.len(),
            positions.len(),
            count
        );
    }
}
