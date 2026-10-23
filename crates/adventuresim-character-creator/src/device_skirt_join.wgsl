fn body_point(index:u32)->PatchPoint {
    let source=body_source(index);
    if (params.loose==0u) {return source;}
    let word=index*NECK_WORDS;
    let a=body_source(neckline[word]);let b=body_source(neckline[word+1u]);
    let neck=mix3(a.point,b.point,bitcast<f32>(neckline[word+2u]));
    let frame=frame_at(0u);
    let local=host_local(frame,source.point);let neck_local=host_local(frame,neck);
    if (neck_local.y<=frame.half_extents.y) {atomicOr(&waist_status[0],1u);return source;}
    let down=clamp((neck_local.y-local.y)/(neck_local.y-frame.half_extents.y),0.0,1.0);
    if (down<=0.0) {return source;}
    // Gathered cloth opens immediately below its bound edge, then approaches
    // the free fall with zero radial slope. A zero-slope start follows the
    // chest first and produces a padded ledge when the fullness releases.
    let retained=1.0-clamp(down/params.fullness_release,0.0,1.0);
    let share=select(1.0-retained*retained*retained*retained,1.0,neckline[word+3u]!=0u);
    let axes=vec2<f32>(enclosure[0u],enclosure[1u]);
    let direction=host_normalized(vec3<f32>(local.x,0.0,local.z));
    let hanging=1.0/length(direction.xz/axes)+params.ease;
    let radius=max(length(local.xz),mix(length(neck_local.xz),hanging,share));
    let radial=host_normalized(frame.x*local.x+frame.z*local.z);
    return PatchPoint(frame.origin+frame.y*local.y+radial*radius,host_normalized(mix3(source.normal,radial,share)));
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.count) { return; }
    if (id.x >= params.sources && id.x < params.points) { return; }
    var result: PatchPoint;
    if (id.x < params.points) { result = body_point(id.x); }
    else {
        let start = (id.x - params.points) * 3u;
        let a = body_point(samples[start]);
        let b = body_point(samples[start + 1u]);
        let blend = bitcast<f32>(samples[start + 2u]);
        result = PatchPoint(mix3(a.point, b.point, blend), host_normalized(mix3(a.normal, b.normal, blend)));
    }
    let frame = frame_at(0u);
    let point = host_local(frame, result.point);
    // A pointed waist must leave a positive hanging span on every refit.
    // Reject this wearer/design combination instead of clipping the waist.
    if (point.y <= -frame.half_extents.y) { atomicOr(&waist_status[0], 1u); }
    let normal = vec3<f32>(host_dot(frame.x, result.normal), host_dot(frame.y, result.normal), host_dot(frame.z, result.normal));
    let start = FIRST_POINT + id.x * OUTPUT_WORDS;
    fit[start] = point.x; fit[start + 1u] = point.y; fit[start + 2u] = point.z;
    fit[start + 3u] = normal.x; fit[start + 4u] = normal.y; fit[start + 5u] = normal.z;
}
