use super::*;

/// Deterministic points in the unit cube, xorshift32 so that a failure is
/// reproducible from the test name alone.
fn points(count: usize, seed: u32) -> Vec<[f32; 3]> {
    let mut state = seed | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state as f32 / u32::MAX as f32
    };
    (0..count).map(|_| [next(), next(), next()]).collect()
}

/// A latitude-longitude sphere of radius one around the origin.
fn sphere(rows: u32, columns: u32) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let mut positions = Vec::new();
    for row in 0..=rows {
        let polar = std::f32::consts::PI * row as f32 / rows as f32;
        for column in 0..columns {
            let azimuth = std::f32::consts::TAU * column as f32 / columns as f32;
            positions.push([
                polar.sin() * azimuth.cos(),
                polar.cos(),
                polar.sin() * azimuth.sin(),
            ]);
        }
    }
    let mut triangles = Vec::new();
    for row in 0..rows {
        for column in 0..columns {
            let a = row * columns + column;
            let b = row * columns + (column + 1) % columns;
            let c = a + columns;
            let d = b + columns;
            triangles.push([a, c, b]);
            triangles.push([b, c, d]);
        }
    }
    (positions, triangles)
}

fn squared(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}

fn buffer<T: bytemuck::NoUninit>(context: &WgpuContext, data: &[T]) -> Result<Buffer> {
    Buffer::from_slice(context, data, BufferDefinition::storage().with_copy_src())
}

#[tokio::test]
async fn nearest_points_match_a_host_search() -> Result<()> {
    let context = WgpuContext::new().await?;
    let query = MeshQuery::new(&context)?;
    // More targets than one tile, and a count that is not a multiple of one.
    let targets = points(1_300, 7);
    let queries = points(700, 11);
    let hits = QueryHits::new(&context, queries.len() as u32)?;
    let positions = buffer(&context, &targets)?;
    let mut batch = KernelBatch::new(&context);
    query.record_nearest_points(
        &mut batch,
        &buffer(&context, &queries)?,
        queries.len() as u32,
        PointTargets {
            positions: &positions,
            candidates: None,
            count: targets.len() as u32,
        },
        &hits,
    )?;
    batch.submit();
    let found = hits.read(&context, queries.len() as u32).await?;
    for (point, hit) in queries.iter().zip(found) {
        let expected = (0..targets.len())
            .min_by(|a, b| squared(*point, targets[*a]).total_cmp(&squared(*point, targets[*b])))
            .unwrap();
        assert_eq!(hit.target, Some(expected as u32));
        // The device may fuse the sum's multiply-adds: one rounding, not three.
        let distance = squared(*point, targets[expected]);
        assert!((hit.distance - distance).abs() <= distance * 1e-6);
    }
    Ok(())
}

#[tokio::test]
async fn candidates_narrow_a_point_search() -> Result<()> {
    let context = WgpuContext::new().await?;
    let query = MeshQuery::new(&context)?;
    let targets = points(600, 3);
    let queries = points(100, 5);
    let candidates: Vec<u32> = (0..600).filter(|i| i % 3 == 1).collect();
    let hits = QueryHits::new(&context, queries.len() as u32)?;
    let positions = buffer(&context, &targets)?;
    let candidate_buffer = buffer(&context, &candidates)?;
    let mut batch = KernelBatch::new(&context);
    query.record_nearest_points(
        &mut batch,
        &buffer(&context, &queries)?,
        queries.len() as u32,
        PointTargets {
            positions: &positions,
            candidates: Some(&candidate_buffer),
            count: candidates.len() as u32,
        },
        &hits,
    )?;
    batch.submit();
    for (point, hit) in queries.iter().zip(hits.read(&context, 100).await?) {
        let expected = candidates
            .iter()
            .min_by(|a, b| {
                squared(*point, targets[**a as usize])
                    .total_cmp(&squared(*point, targets[**b as usize]))
            })
            .unwrap();
        assert_eq!(hit.target, Some(*expected));
    }
    Ok(())
}

#[tokio::test]
async fn ties_go_to_the_lowest_index() -> Result<()> {
    let context = WgpuContext::new().await?;
    let query = MeshQuery::new(&context)?;
    // Four copies of the same point, the first at index 1.
    let targets: [[f32; 3]; 5] = [[5.0, 5.0, 5.0], [0.0; 3], [0.0; 3], [1.0; 3], [0.0; 3]];
    let hits = QueryHits::new(&context, 1)?;
    let positions = buffer(&context, &targets)?;
    let mut batch = KernelBatch::new(&context);
    query.record_nearest_points(
        &mut batch,
        &buffer(&context, &[[0.1f32, 0.0, 0.0]])?,
        1,
        PointTargets {
            positions: &positions,
            candidates: None,
            count: targets.len() as u32,
        },
        &hits,
    )?;
    batch.submit();
    assert_eq!(hits.read(&context, 1).await?[0].target, Some(1));
    Ok(())
}

#[tokio::test]
async fn closest_points_lie_on_the_surface() -> Result<()> {
    let context = WgpuContext::new().await?;
    let query = MeshQuery::new(&context)?;
    let (positions, triangles) = sphere(24, 48);
    // Queries inside and outside the sphere.
    let queries: Vec<[f32; 3]> = points(500, 9)
        .into_iter()
        .map(|p| p.map(|x| 3.0 * x - 1.5))
        .collect();
    let hits = QueryHits::new(&context, queries.len() as u32)?;
    let position_buffer = buffer(&context, &positions)?;
    let triangle_buffer = buffer(&context, &triangles)?;
    let mut batch = KernelBatch::new(&context);
    query.record_closest_triangles(
        &mut batch,
        &buffer(&context, &queries)?,
        queries.len() as u32,
        TriangleTargets {
            positions: &position_buffer,
            triangles: &triangle_buffer,
            candidates: None,
            count: triangles.len() as u32,
        },
        &hits,
    )?;
    batch.submit();
    for (point, hit) in queries.iter().zip(hits.read(&context, 500).await?) {
        let triangle = triangles[hit.target.unwrap() as usize];
        let closest: [f32; 3] = std::array::from_fn(|axis| {
            (0..3)
                .map(|corner| hit.weights[corner] * positions[triangle[corner] as usize][axis])
                .sum()
        });
        assert!((squared(*point, closest) - hit.distance).abs() < 1e-5);
        assert!(hit.weights.iter().all(|w| (-1e-6..=1.0 + 1e-6).contains(w)));
        // A tessellated unit sphere: the surface is about one unit out.
        let radius = squared(*point, [0.0; 3]).sqrt();
        assert!((hit.distance.sqrt() - (radius - 1.0).abs()).abs() < 0.01);
    }
    Ok(())
}

#[tokio::test]
async fn rays_find_the_first_and_last_crossing() -> Result<()> {
    let context = WgpuContext::new().await?;
    let query = MeshQuery::new(&context)?;
    let (positions, triangles) = sphere(32, 64);
    let position_buffer = buffer(&context, &positions)?;
    let triangle_buffer = buffer(&context, &triangles)?;
    // From outside, straight through the centre.
    let origins = [[-3.0f32, 0.1, 0.05], [0.02, -4.0, 0.03], [5.0, 5.0, 5.0]];
    let directions = [[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
    let origin_buffer = buffer(&context, &origins)?;
    let direction_buffer = buffer(&context, &directions)?;
    for (crossing, expected) in [(Crossing::First, [2.0, 3.0]), (Crossing::Last, [4.0, 5.0])] {
        let hits = QueryHits::new(&context, 3)?;
        let mut batch = KernelBatch::new(&context);
        query.record_ray_triangles(
            &mut batch,
            Rays {
                origins: &origin_buffer,
                directions: &direction_buffer,
                count: 3,
                span: [0.0, 100.0],
                crossing,
                parallel_epsilon: 1e-9,
            },
            TriangleTargets {
                positions: &position_buffer,
                triangles: &triangle_buffer,
                candidates: None,
                count: triangles.len() as u32,
            },
            &hits,
        )?;
        batch.submit();
        let found = hits.read(&context, 3).await?;
        for (hit, distance) in found.iter().zip(expected) {
            assert!(
                (hit.distance - distance).abs() < 0.01,
                "{crossing:?}: {hit:?}"
            );
            assert!(hit.target.is_some());
        }
        assert_eq!(found[2].target, None, "a ray pointing away misses");
    }
    Ok(())
}
