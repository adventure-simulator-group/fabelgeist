#define_import_path adventuresim_environment::tree_pose

// GPU wire boundary: root xyz / XZ scale, then height scale / sin yaw / cos yaw.
struct TreePose {
    origin_horizontal: vec4<f32>,
    vertical_yaw: vec4<f32>,
};

// GPU ABI: pose followed by actual branch start/count and leaf start/count.
struct TreeInstance {
    pose: TreePose,
    records: vec4<u32>,
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
