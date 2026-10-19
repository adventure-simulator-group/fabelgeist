#define_import_path adventuresim_environment::tree_pose

// GPU wire boundary: root xyz / XZ scale, then height scale / sin yaw / cos yaw.
struct TreePose {
    origin_horizontal: vec4<f32>,
    vertical_yaw: vec4<f32>,
};

// GPU ABI: pose, branch/leaf spans, then four root-distance margin endpoints.
// Four aligned vec4s: 64 bytes, matching PlacementRecord in trees/upload.rs.
struct TreeInstance {
    pose: TreePose,
    records: vec4<u32>,
    visibility: vec4<f32>,
};

// The indexed grid supplies a local integer address; Eidolon supplies the
// placement ordinal. Do not use builtin(vertex_index), which includes the
// Bevy mesh allocator's base-vertex offset.
struct TreeVertex {
    @location(0) address: u32,
    @location(10) i_index: u32,
};

fn rotate(pose: TreePose, point: vec3<f32>) -> vec3<f32> {
    let s = pose.vertical_yaw.y;
    let c = pose.vertical_yaw.z;
    return vec3<f32>(c * point.x + s * point.z, point.y, -s * point.x + c * point.z);
}

fn place_tree(pose: TreePose, point: vec3<f32>) -> vec3<f32> {
    let scale = vec3<f32>(pose.origin_horizontal.w, pose.vertical_yaw.x, pose.origin_horizontal.w);
    return pose.origin_horizontal.xyz + rotate(pose, point * scale);
}

fn tree_normal(pose: TreePose, normal: vec3<f32>) -> vec3<f32> {
    let scale = vec3<f32>(pose.origin_horizontal.w, pose.vertical_yaw.x, pose.origin_horizontal.w);
    return normalize(rotate(pose, normal / scale));
}

// The same quantized range kernel as Bevy 0.19 mesh_functions.wgsl, with the
// production tactical_tree_impostor.wgsl crisp acceptance of levels -7..8.
// Keep the quotient and WGSL round: CPU midpoint thresholds change the rule.
fn tree_visible(tree: TreeInstance, lod_eye: vec3<f32>) -> bool {
    let range = tree.visibility;
    if range.w == 0.0 { return true; }
    let camera_distance = length(lod_eye - tree.pose.origin_horizontal.xyz);
    // These outside/interior cases also avoid overflow or division of very
    // large quotients: only the active positive-width margin needs rounding.
    if camera_distance <= range.x || camera_distance >= range.w { return false; }
    if camera_distance >= range.y && camera_distance < range.z { return true; }
    let far_margin = camera_distance >= range.z;
    let offset = select(-16, 0, far_margin);
    let bounds = select(range.xy, range.zw, far_margin);
    let level = i32(round((camera_distance - bounds.x) / (bounds.y - bounds.x) * 16.0));
    let quantized = offset + clamp(level, 0, 16);
    return quantized > -8 && quantized <= 8;
}
