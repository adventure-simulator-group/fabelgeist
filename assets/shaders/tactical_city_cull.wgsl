struct Building { transform: mat4x4<f32>, bounds: vec4<f32>, levels: vec4<u32>, uv_offset: vec4<f32> }
struct ViewParameters { clip_from_world: mat4x4<f32>, camera: vec4<f32>, projection: vec4<f32>, counts: vec4<u32>, policy: vec4<u32>, far_plane: vec4<f32> }
struct DrawArgs { vertices: u32, instances: atomic<u32>, first_vertex: u32, first_instance: u32 }
@group(0) @binding(0) var<uniform> params: ViewParameters;
@group(0) @binding(1) var<storage, read> buildings: array<Building>;
@group(0) @binding(2) var<storage, read_write> selection: array<u32>;
struct DrawRange { geometry: vec4<u32>, instances: vec4<u32> }
@group(0) @binding(3) var<storage, read> source: array<DrawRange>;
@group(0) @binding(4) var<storage, read_write> visible: array<vec2<u32>>;
@group(0) @binding(5) var<storage, read_write> draws: array<DrawArgs>;
@group(0) @binding(6) var<storage, read> owners: array<u32>;

@compute @workgroup_size(64)
fn select_buildings(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.counts.x { return; }
    let building = buildings[id.x];
    if building.levels.y == 2u { return; } // Canonical component transform, not a visibility owner.
    let out_index = params.counts.z * params.counts.x + id.x;
    let previous = select(0u, selection[out_index], params.projection.w > 0.5);
    selection[out_index] = 0u;
    if dot(params.far_plane, vec4(building.bounds.xyz, 1.0)) < -building.bounds.w * length(params.far_plane.xyz) { return; }
    // Static outdoor props retain the tactical distance policy. Shadow views
    // use their parent camera's position and their own caster frustum. Shared
    // point/spot shadows have no parent camera, so only their frustum applies.
    if params.policy.y == 0u && building.levels.y == 1u && distance(building.bounds.xyz, params.camera.xyz) >= bitcast<f32>(building.levels.w) { return; }
    let rows = transpose(params.clip_from_world);
    // Reverse-Z clip volume. Shadow casters use their light's view, not the
    // camera's frustum, so off-screen buildings may still cast visible shadows.
    let planes = array(rows[3]+rows[0], rows[3]-rows[0], rows[3]+rows[1], rows[3]-rows[1], rows[2], rows[3]-rows[2]);
    for (var i=0u; i<6u; i++) {
        // Directional shadow projections deliberately admit casters before
        // their near plane, matching Bevy's directional shadow visibility.
        if params.policy.x != 0u && params.camera.w > 0.5 && i == 5u { continue; }
        let plane = planes[i];
        if dot(plane, vec4(building.bounds.xyz, 1.0)) < -building.bounds.w * length(plane.xyz) { return; }
    }
    // Distant city shadows use the client-generated enclosure. Camera and
    // shadow history stay independent; off-camera casters remain eligible.
    if params.policy.x != 0u && (building.levels.x & 4u) != 0u {
        selection[out_index] = 4u;
        return;
    }
    let clip = params.clip_from_world * vec4(building.bounds.xyz, 1.0);
    let pixels = building.bounds.w * params.projection.x / max(abs(clip.w), 0.01);
    // Separate entry/exit thresholds prevent small camera movements from
    // alternating representations. History belongs to this retained view.
    var facade_threshold = params.projection.y;
    if previous == 4u { facade_threshold *= 1.1; }
    if previous == 2u { facade_threshold *= 0.9; }
    var level = 2u;
    if pixels > facade_threshold { level = 1u; }
    if pixels > params.projection.z && (building.levels.x & 1u) != 0u { level = 0u; }
    if (building.levels.x & (1u << level)) == 0u { level = 2u; }
    selection[out_index] = 1u << level;
}

const FACADE_OVERLAY_FLAG: u32 = 256u;
const LOD_LEVEL_MASK: u32 = 3u;

// Most ranges are rejected. One lane checks a whole range and reserves its
// selected clusters, rather than scheduling a mostly idle workgroup per range.
@compute @workgroup_size(64)
fn compact_ranges(@builtin(global_invocation_id) id: vec3<u32>,
                  @builtin(num_workgroups) groups: vec3<u32>) {
    let range = id.y * groups.x * 64u + id.x;
    if range >= arrayLength(&source) { return; }
    let entry = source[range];
    let job = entry.geometry;
    if params.policy.x != 0u && (job.w & FACADE_OVERLAY_FLAG) != 0u { return; }
    let cluster_count = entry.instances.y;
    for (var placement = job.x; placement < job.x + entry.instances.x; placement += 1u) {
        let owner = owners[placement];
        if (selection[params.counts.z * params.counts.x + owner] & (1u << (job.w & LOD_LEVEL_MASK))) == 0u { continue; }
        let output_start = atomicAdd(&draws[params.counts.w].instances, cluster_count);
        for (var cluster = 0u; cluster < cluster_count; cluster += 1u) {
            let index = params.counts.w * params.counts.y + output_start + cluster;
            visible[index] = vec2(range, placement * cluster_count + cluster);
        }
    }
}
