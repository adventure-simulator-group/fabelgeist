use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};
use fabelgeist_math::Vec3;

use super::*;
use crate::tests::{Random, brute_force_overlaps, scattered_boxes, sphere_mesh};
use crate::{Aabb, morton};

async fn build(bounds: &[Aabb]) -> Result<(WgpuContext, GpuBvh)> {
    let context = WgpuContext::new().await?;
    let kernels = BvhKernels::new(&context)?;
    let mut bvh = GpuBvh::new(&context, kernels, bounds.len().max(1) as u32)?;

    let packed = pack_bounds(bounds);
    let buffer = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&packed),
        BufferDefinition::storage().with_usage(BufferUse::CopySource),
    )?;
    bvh.build(&context, &buffer, bounds.len() as u32)?;
    Ok((context, bvh))
}

/// Walk the hierarchy on the host exactly as the WGSL traversal does, and
/// check every invariant it depends on.
fn check_structure(nodes: &[HostNode], right_children: &[u32], indices: &[u32], count: usize) {
    assert_eq!(nodes.len(), 2 * count - 1, "wrong node count");

    let mut sorted = indices.to_vec();
    sorted.sort_unstable();
    assert_eq!(
        sorted,
        (0..count as u32).collect::<Vec<_>>(),
        "indices are not a permutation of the primitives"
    );

    let mut reached = vec![false; count];
    let mut visited = vec![false; nodes.len()];
    let mut stack = vec![0usize];

    while let Some(index) = stack.pop() {
        assert!(!visited[index], "node {index} is reachable twice");
        visited[index] = true;

        let node = nodes[index];
        assert!(!node.bounds.is_empty(), "node {index} has empty bounds");

        if node.is_leaf() {
            assert!(index >= count - 1, "leaf {index} is in the internal range");
            let slot = node.left_or_first as usize;
            assert_eq!(
                slot,
                index - (count - 1),
                "leaf {index} points at the wrong slot"
            );
            reached[indices[slot] as usize] = true;
        } else {
            assert!(
                index < count - 1,
                "internal node {index} is in the leaf range"
            );
            let left = node.left_or_first as usize;
            let right = right_children[index] as usize;
            assert!(
                left < nodes.len() && right < nodes.len(),
                "child out of range"
            );
            assert_ne!(left, right, "node {index} has the same child twice");
            stack.push(left);
            stack.push(right);
        }
    }

    assert!(visited.iter().all(|&v| v), "some node is unreachable");
    assert!(reached.iter().all(|&r| r), "some primitive is unreachable");
}

/// The traversal in `shaders::traversal_source`, run on the host. Testing the
/// hierarchy through the same walk the GPU does is the point: a tree that
/// looks fine but that the traversal cannot walk is still broken.
fn query_aabb(
    nodes: &[HostNode],
    right_children: &[u32],
    indices: &[u32],
    query: &Aabb,
) -> Vec<u32> {
    let mut found = Vec::new();
    if nodes.is_empty() {
        return found;
    }
    let mut stack = vec![0usize];
    while let Some(index) = stack.pop() {
        let node = nodes[index];
        if !node.bounds.overlaps(query) {
            continue;
        }
        if node.is_leaf() {
            found.push(indices[node.left_or_first as usize]);
        } else {
            stack.push(node.left_or_first as usize);
            stack.push(right_children[index] as usize);
        }
    }
    found.sort_unstable();
    found
}

#[tokio::test]
async fn builds_a_sound_hierarchy() -> Result<()> {
    for count in [2usize, 3, 5, 17, 256, 257, 5000] {
        let bounds = scattered_boxes(count, 53 + count as u32);
        let (context, bvh) = build(&bounds).await?;

        let nodes = bvh.read_nodes(&context).await?;
        let indices = bvh.read_indices(&context).await?;
        let right_children = bvh.read_right_children(&context).await?;
        check_structure(&nodes, &right_children, &indices, count);
    }
    Ok(())
}

#[tokio::test]
async fn a_single_primitive_is_its_own_root() -> Result<()> {
    let bounds = vec![Aabb::new(Vec3::splat(-1.0), Vec3::splat(2.0))];
    let (context, bvh) = build(&bounds).await?;

    let nodes = bvh.read_nodes(&context).await?;
    assert_eq!(nodes.len(), 1);
    assert!(nodes[0].is_leaf());
    assert!((nodes[0].bounds.min - Vec3::splat(-1.0)).length() < 1e-5);
    assert!((nodes[0].bounds.max - Vec3::splat(2.0)).length() < 1e-5);
    Ok(())
}

#[tokio::test]
async fn node_bounds_contain_their_children() -> Result<()> {
    let bounds = scattered_boxes(3000, 59);
    let (context, bvh) = build(&bounds).await?;
    let nodes = bvh.read_nodes(&context).await?;
    let right_children = bvh.read_right_children(&context).await?;
    let indices = bvh.read_indices(&context).await?;
    let count = bounds.len();

    for (index, node) in nodes.iter().enumerate() {
        if node.is_leaf() {
            let primitive = bounds[indices[node.left_or_first as usize] as usize];
            assert!(
                node.bounds.union(primitive) == node.bounds,
                "leaf {index} does not contain its primitive"
            );
        } else {
            let left = nodes[node.left_or_first as usize].bounds;
            let right = nodes[right_children[index] as usize].bounds;
            assert!(
                node.bounds.union(left).union(right) == node.bounds,
                "node {index} does not contain its children"
            );
        }
    }

    // And the root contains everything.
    let all = bounds.iter().fold(Aabb::EMPTY, |acc, &b| acc.union(b));
    let root = nodes[0].bounds;
    assert!(
        (root.min - all.min).length() < 1e-3 && (root.max - all.max).length() < 1e-3,
        "root {root:?} is not the scene bounds {all:?}"
    );
    let _ = count;
    Ok(())
}

#[tokio::test]
async fn traversal_finds_every_overlap() -> Result<()> {
    let bounds = scattered_boxes(4000, 61);
    let (context, bvh) = build(&bounds).await?;
    let nodes = bvh.read_nodes(&context).await?;
    let right_children = bvh.read_right_children(&context).await?;
    let indices = bvh.read_indices(&context).await?;

    let mut random = Random::new(67);
    for _ in 0..100 {
        let center = random.point(-12.0, 12.0);
        let half = Vec3::splat(random.range(0.1, 4.0));
        let query = Aabb::new(center - half, center + half);

        assert_eq!(
            query_aabb(&nodes, &right_children, &indices, &query),
            brute_force_overlaps(&bounds, &query),
            "the GPU hierarchy and brute force disagree"
        );
    }
    Ok(())
}

/// Duplicate Morton codes are what the index tie-break in `delta` exists for.
/// Without it the hierarchy degenerates and the traversal starts to overflow
/// its stack, which shows up as missing primitives rather than as an error.
#[tokio::test]
async fn handles_coincident_primitives() -> Result<()> {
    let count = 2000;
    let bounds = vec![Aabb::new(Vec3::splat(0.0), Vec3::splat(1.0)); count];
    let (context, bvh) = build(&bounds).await?;

    let nodes = bvh.read_nodes(&context).await?;
    let right_children = bvh.read_right_children(&context).await?;
    let indices = bvh.read_indices(&context).await?;
    check_structure(&nodes, &right_children, &indices, count);

    let query = Aabb::new(Vec3::splat(0.5), Vec3::splat(0.5));
    assert_eq!(
        query_aabb(&nodes, &right_children, &indices, &query).len(),
        count,
        "every coincident box overlaps the query"
    );
    Ok(())
}

/// Every primitive on one plane gives a zero extent on one axis, which the
/// Morton pass must not divide by.
#[tokio::test]
async fn handles_a_flat_scene() -> Result<()> {
    let mut random = Random::new(71);
    let bounds: Vec<Aabb> = (0..1000)
        .map(|_| {
            let x = random.range(-5.0, 5.0);
            let z = random.range(-5.0, 5.0);
            Aabb::new(Vec3::new(x, 0.0, z), Vec3::new(x + 0.1, 0.0, z + 0.1))
        })
        .collect();

    let (context, bvh) = build(&bounds).await?;
    let nodes = bvh.read_nodes(&context).await?;
    let right_children = bvh.read_right_children(&context).await?;
    let indices = bvh.read_indices(&context).await?;
    check_structure(&nodes, &right_children, &indices, bounds.len());
    assert!(nodes.iter().all(|n| n.bounds.min.is_finite()));
    Ok(())
}

/// A refit keeps the topology and recomputes the boxes; the result must still
/// answer every query, and must match the moved geometry rather than the old.
///
/// Repeated, and deliberately so. This is also the crate's guard against a
/// missing barrier between dependent dispatches: a refit is three different
/// kernels taking turns on one buffer through atomics, which is the shape that
/// breaks when  stops re-registering resources per dispatch.
/// One round of it caught that bug about two times in three, so it runs eight
/// -- enough that a regression cannot slip past on a lucky run.
#[tokio::test]
async fn refit_tracks_moved_primitives() -> Result<()> {
    let bounds = scattered_boxes(2000, 73);
    let (context, mut bvh) = build(&bounds).await?;

    for round in 0..8 {
        let mut random = Random::new(79 + round * 7);
        let moved: Vec<Aabb> = bounds
            .iter()
            .map(|b| {
                let shift = random.point(-0.4, 0.4);
                Aabb::new(b.min + shift, b.max + shift)
            })
            .collect();

        let packed = pack_bounds(&moved);
        let buffer = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&packed),
            BufferDefinition::storage().with_usage(BufferUse::CopySource),
        )?;
        let mut batch = KernelBatch::new(&context);
        bvh.record_refit(&mut batch, &buffer)?;
        batch.submit();

        let nodes = bvh.read_nodes(&context).await?;
        let right_children = bvh.read_right_children(&context).await?;
        let indices = bvh.read_indices(&context).await?;

        let mut random = Random::new(83);
        for _ in 0..100 {
            let center = random.point(-12.0, 12.0);
            let half = Vec3::splat(random.range(0.5, 4.0));
            let query = Aabb::new(center - half, center + half);
            assert_eq!(
                query_aabb(&nodes, &right_children, &indices, &query),
                brute_force_overlaps(&moved, &query),
                "round {round}: the refit hierarchy disagrees with brute force"
            );
        }
    }
    Ok(())
}

/// The whole reason the GPU build exists: real mesh geometry, rebuilt from a
/// buffer. Checked against the host tree's answers, not just against itself.
#[tokio::test]
async fn agrees_with_the_host_tree_on_a_mesh() -> Result<()> {
    let (positions, triangles) = sphere_mesh(24, 48, 1.5);
    let bounds = crate::triangle_bounds(&positions, &triangles);

    let host = crate::Bvh::build(&bounds);
    let (context, gpu) = build(&bounds).await?;
    let nodes = gpu.read_nodes(&context).await?;
    let right_children = gpu.read_right_children(&context).await?;
    let indices = gpu.read_indices(&context).await?;

    let mut random = Random::new(89);
    for _ in 0..150 {
        let center = random.point(-2.0, 2.0);
        let half = Vec3::splat(random.range(0.05, 0.6));
        let query = Aabb::new(center - half, center + half);

        let mut from_host = Vec::new();
        host.query_aabb(&bounds, &query, |index| from_host.push(index));
        from_host.sort_unstable();

        assert_eq!(
            query_aabb(&nodes, &right_children, &indices, &query),
            from_host,
            "the two hierarchies disagree"
        );
    }
    Ok(())
}

#[tokio::test]
async fn rejects_more_primitives_than_it_was_built_for() -> Result<()> {
    let context = WgpuContext::new().await?;
    let kernels = BvhKernels::new(&context)?;
    let mut bvh = GpuBvh::new(&context, kernels, 10)?;
    let buffer = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[0.0f32; 8]),
        BufferDefinition::storage(),
    )?;
    let mut batch = KernelBatch::new(&context);
    assert!(bvh.record_build(&mut batch, &buffer, 100).is_err());
    Ok(())
}

/// The WGSL Morton encoder and the Rust one must agree, or a hierarchy built
/// on the GPU cannot be reasoned about with the host code.
#[tokio::test]
async fn wgsl_morton_matches_the_host() -> Result<()> {
    let context = WgpuContext::new().await?;
    let code = format!(
        r#"
@group(0) @binding(0) var<storage, read> input: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;
{}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let index = global_id.x;
    if (index >= arrayLength(&output)) {{ return; }}
    output[index] = bvh_morton(input[index].xyz);
}}
"#,
        super::shaders::MORTON
    );
    let kernel = Kernel::new(&context, code)?;

    let mut random = Random::new(97);
    let points: Vec<[f32; 4]> = (0..500)
        .map(|_| [random.unit(), random.unit(), random.unit(), 0.0])
        // The endpoints are where a clamp is either right or off by one.
        .chain([[0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 0.0]])
        .collect();

    let input = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&points),
        BufferDefinition::storage(),
    )?;
    let output = Buffer::new(
        &context,
        (points.len() as u64 * 4).into(),
        BufferDefinition::storage().with_usage(BufferUse::CopySource),
    )?;

    let mut parameters = PassParameters::new();
    parameters.insert("input", input);
    parameters.insert("output", output.clone());
    kernel.run(
        &context,
        parameters,
        kernel.groups_for((points.len() as u32).into()),
    )?;

    let from_gpu: Vec<u32> = output.read(&context).await?;
    for (point, &gpu_code) in points.iter().zip(&from_gpu) {
        let host = morton::encode_unit(point[0], point[1], point[2]);
        assert_eq!(
            host, gpu_code,
            "Morton codes differ for {point:?}: host {host:#x}, GPU {gpu_code:#x}"
        );
    }
    Ok(())
}
