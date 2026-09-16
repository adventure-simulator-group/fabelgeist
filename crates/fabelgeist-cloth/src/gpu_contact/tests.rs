//! The host contact tests, run through the GPU solve.
use super::*;
use crate::ccd::{self, Pair};
use fabelgeist_xpbd::particles::pack;

struct Case<'a> {
    start: &'a [Vec3],
    end: &'a [Vec3],
    masses: &'a [f32],
    faces: &'a [[u32; 3]],
    seams: &'a [[u32; 2]],
    velocities: Option<&'a [Vec3]>,
    obstacle: Option<(&'a [Vec3], &'a [[u32; 3]], f32)>,
    thickness: f32,
    iterations: u32,
}

impl Default for Case<'_> {
    fn default() -> Self {
        Self {
            start: &[],
            end: &[],
            masses: &[],
            faces: &[],
            seams: &[],
            velocities: None,
            obstacle: None,
            thickness: 0.005,
            iterations: 4,
        }
    }
}

struct Outcome {
    positions: Vec<Vec3>,
    velocities: Vec<Vec3>,
    contacts: u32,
}

async fn project(case: Case<'_>) -> Result<Outcome> {
    let context = WgpuContext::new().await?;
    let cache = KernelCache::new();
    let particles = Particles::from_positions(&context, case.start, case.masses)?;
    particles
        .positions
        .write(&context, &pack(case.end, case.masses))?;
    if let Some(velocities) = case.velocities {
        particles
            .velocities
            .write(&context, &pack(velocities, &vec![0.0; velocities.len()]))?;
    }
    let mut contacts = GpuSurfaceContacts::new(
        &context,
        &cache,
        case.start.len() as u32,
        case.faces,
        case.seams,
    )?;
    if let Some((positions, faces, clearance)) = case.obstacle {
        contacts.set_static_surface(&context, positions, faces, clearance)?;
    }
    contacts.project(&context, &particles, case.thickness, case.iterations, false)?;
    Ok(Outcome {
        positions: particles.read_positions(&context).await?,
        velocities: particles.read_velocities(&context).await?,
        contacts: contacts.contact_count(&context).await?,
    })
}

fn p(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

#[tokio::test]
async fn vertex_crossing_triangle_interior_is_returned_to_approach_side() -> Result<()> {
    let end = [
        p(-1., 0., -1.),
        p(0., 0., 1.),
        p(1., 0., -1.),
        p(0., -0.1, 0.),
    ];
    let mut start = end;
    start[3].y = 0.1;
    let outcome = project(Case {
        start: &start,
        end: &end,
        masses: &[0., 0., 0., 1.],
        faces: &[[0, 1, 2]],
        iterations: 2,
        ..Default::default()
    })
    .await?;
    assert!(
        outcome.positions[3].y >= 0.0049,
        "{:?}",
        outcome.positions[3]
    );
    assert_eq!(&outcome.positions[..3], &end[..3]);
    Ok(())
}

#[tokio::test]
async fn triangle_interior_crossing_preserves_slide_and_pins() -> Result<()> {
    let start = [
        p(-1., 0., -1.),
        p(0., 0., 1.),
        p(1., 0., -1.),
        p(0., 0.1, 0.),
    ];
    let mut end = start;
    end[3] = p(0.02, -0.1, 0.);
    let mut velocity = [Vec3::default(); 4];
    velocity[3] = p(1., -20., 0.);
    let outcome = project(Case {
        start: &start,
        end: &end,
        masses: &[0., 0., 0., 1.],
        faces: &[[0, 1, 2]],
        velocities: Some(&velocity),
        ..Default::default()
    })
    .await?;
    assert!(outcome.contacts > 0);
    assert!(outcome.positions[3].y >= 0.0049);
    assert!((outcome.positions[3].x - end[3].x).abs() < 1e-5);
    assert!((outcome.velocities[3].x - 1.).abs() < 1e-4);
    assert!(
        outcome.velocities[3].y >= -1e-4,
        "{:?}",
        outcome.velocities[3]
    );
    assert_eq!(&outcome.positions[..3], &start[..3]);
    Ok(())
}

#[tokio::test]
async fn adjacent_faces_are_not_inflated() -> Result<()> {
    let points = [p(0., 0., 0.), p(1., 0., 0.), p(0., 0., 1.), p(1., 0., 1.)];
    let outcome = project(Case {
        start: &points,
        end: &points,
        masses: &[1.; 4],
        faces: &[[0, 1, 2], [1, 3, 2]],
        thickness: 0.01,
        iterations: 2,
        ..Default::default()
    })
    .await?;
    assert_eq!(outcome.contacts, 0);
    assert_eq!(outcome.positions, points);
    Ok(())
}

#[tokio::test]
async fn edge_interiors_separate_without_close_endpoints() -> Result<()> {
    let points = [
        p(-1., 0., 0.),
        p(1., 0., 0.),
        p(-1., 0., -1.),
        p(0., 0.001, -1.),
        p(0., 0.001, 1.),
        p(1., 0.001, 1.),
    ];
    let outcome = project(Case {
        start: &points,
        end: &points,
        masses: &[1.; 6],
        faces: &[[0, 1, 2], [3, 4, 5]],
        thickness: 0.01,
        iterations: 8,
        ..Default::default()
    })
    .await?;
    let q = &outcome.positions;
    assert!(outcome.contacts > 0);
    let gap = ccd::proximity(
        Pair::EdgeEdge,
        [q[0], q[1], q[3], q[4]],
        [points[0], points[1], points[3], points[4]],
    )
    .distance;
    assert!(gap > 0.008, "edges are {gap} apart");
    Ok(())
}

#[tokio::test]
async fn swept_edge_bounds_find_fast_crossings_and_preserve_pinned_edges() -> Result<()> {
    let start = [
        p(-1.0, 0.0, 0.0),
        p(1.0, 0.0, 0.0),
        p(-1.0, 0.0, -1.0),
        p(0.0, 10.0, -1.0),
        p(0.0, 10.0, 1.0),
        p(1.0, 10.0, 1.0),
    ];
    let mut end = start;
    for q in &mut end[3..] {
        q.y = -10.0;
    }
    let outcome = project(Case {
        start: &start,
        end: &end,
        masses: &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        faces: &[[0, 1, 2], [3, 4, 5]],
        thickness: 0.004,
        iterations: 8,
        ..Default::default()
    })
    .await?;
    let q = &outcome.positions;
    assert_eq!(&q[..3], &start[..3]);
    let remaining = ccd::sweep(
        Pair::EdgeEdge,
        [start[0], start[1], start[3], start[4]],
        [q[0], q[1], q[3], q[4]],
        0.001,
    );
    assert!(
        remaining.is_none(),
        "edge still crosses: {remaining:?}; positions {q:?}"
    );
    Ok(())
}

#[tokio::test]
async fn translating_body_pushes_a_stationary_vertex_to_the_approach_side() -> Result<()> {
    let start = [
        p(-1.0, -1.0, -1.0),
        p(1.0, -1.0, -1.0),
        p(0.0, 1.0, -1.0),
        p(0.0, 0.0, 0.0),
    ];
    let mut end = start;
    for q in &mut end[..3] {
        q.z = 1.0;
    }
    let outcome = project(Case {
        start: &start,
        end: &end,
        masses: &[0.0, 0.0, 0.0, 1.0],
        faces: &[[0, 1, 2]],
        thickness: 0.004,
        ..Default::default()
    })
    .await?;
    assert!(
        outcome.positions[3].z >= 1.0039,
        "body passed through vertex: {:?}",
        outcome.positions[3]
    );
    assert_eq!(&outcome.positions[..3], &end[..3]);
    Ok(())
}

#[tokio::test]
async fn sewn_panel_copies_do_not_repel_each_other() -> Result<()> {
    let points = [
        p(0., 0., 0.),
        p(1., 0., 0.),
        p(0., 1., 0.),
        p(1., 0., 0.),
        p(1., 1., 0.),
        p(0., 1., 0.),
    ];
    let outcome = project(Case {
        start: &points,
        end: &points,
        masses: &[1.; 6],
        faces: &[[0, 1, 2], [3, 4, 5]],
        seams: &[[1, 3], [2, 5]],
        ..Default::default()
    })
    .await?;
    assert_eq!(outcome.contacts, 0);
    assert_eq!(outcome.positions, points);
    Ok(())
}

#[tokio::test]
async fn fixed_triangle_interior_blocks_a_cloth_face_without_vertex_overlap() -> Result<()> {
    let start = [p(-1.0, -1.0, 0.1), p(1.0, -1.0, 0.1), p(0.0, 1.0, 0.1)];
    let end = start.map(|q| p(q.x, q.y, -0.1));
    let obstacle = [p(-0.1, -0.1, 0.0), p(0.1, -0.1, 0.0), p(0.0, 0.1, 0.0)];
    let outcome = project(Case {
        start: &start,
        end: &end,
        masses: &[1.0; 3],
        faces: &[[0, 1, 2]],
        obstacle: Some((&obstacle, &[[0, 1, 2]], 0.003)),
        thickness: 0.003,
        iterations: 8,
        ..Default::default()
    })
    .await?;
    let q = &outcome.positions;
    assert!(outcome.contacts > 0);
    let normal = (q[1] - q[0]).cross(q[2] - q[0]);
    for point in obstacle {
        let signed = (point - q[0]).dot(normal) / normal.length();
        assert!(
            signed <= -0.0025,
            "obstacle vertex {point:?} is {signed} from the cloth"
        );
    }
    Ok(())
}

#[tokio::test]
async fn resting_cloth_reaches_body_ease_without_inflating_self_contacts() -> Result<()> {
    let start = [p(-0.1, -0.1, 0.002), p(0.1, -0.1, 0.002), p(0., 0.1, 0.002)];
    let body = [p(-1., -1., 0.), p(1., -1., 0.), p(0., 1., 0.)];
    let outcome = project(Case {
        start: &start,
        end: &start,
        masses: &[1.; 3],
        faces: &[[0, 1, 2]],
        obstacle: Some((&body, &[[0, 1, 2]], 0.005)),
        thickness: 0.0006,
        ..Default::default()
    })
    .await?;
    assert!(outcome.contacts > 0);
    assert!(
        outcome.positions.iter().all(|q| q.z >= 0.00499),
        "{:?}",
        outcome.positions
    );
    Ok(())
}

#[tokio::test]
async fn removing_the_obstacle_stops_its_contacts() -> Result<()> {
    let context = WgpuContext::new().await?;
    let cache = KernelCache::new();
    let start = [p(-0.1, 0.1, -0.1), p(0.1, 0.1, -0.1), p(0.0, 0.1, 0.1)];
    let end: Vec<_> = start.iter().map(|q| p(q.x, -0.1, q.z)).collect();
    let body = [p(-1., 0., -1.), p(0., 0., 1.), p(1., 0., -1.)];
    let particles = Particles::from_positions(&context, &start, &[1.; 3])?;
    let mut contacts = GpuSurfaceContacts::new(&context, &cache, 3, &[[0, 1, 2]], &[])?;
    contacts.set_static_surface(&context, &body, &[[0, 1, 2]], 0.003)?;
    particles.positions.write(&context, &pack(&end, &[1.; 3]))?;
    contacts.project(&context, &particles, 0.003, 4, false)?;
    let blocked = particles.read_positions(&context).await?;
    assert!(blocked.iter().all(|q| q.y >= 0.0029), "{blocked:?}");

    contacts.set_static_surface(&context, &[], &[], 0.0)?;
    particles.positions.write(&context, &pack(&end, &[1.; 3]))?;
    contacts.project(&context, &particles, 0.003, 4, false)?;
    assert_eq!(contacts.contact_count(&context).await?, 0);
    assert_eq!(particles.read_positions(&context).await?, end);
    Ok(())
}

#[test]
fn seam_groups_join_every_copy_of_a_vertex() {
    let groups = seam_groups(6, &[[1, 3], [3, 5], [0, 4]]);
    assert_eq!(groups[1], groups[3]);
    assert_eq!(groups[1], groups[5]);
    assert_eq!(groups[0], groups[4]);
    assert_ne!(groups[0], groups[1]);
    assert_ne!(groups[2], groups[1]);
}
