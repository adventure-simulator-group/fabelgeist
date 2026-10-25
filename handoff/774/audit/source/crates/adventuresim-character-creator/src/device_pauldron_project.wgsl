fn cross2(a: vec2<f32>, b: vec2<f32>) -> f32 { return a.x * b.y - a.y * b.x; }
fn triangle_point(i: u32) -> vec3<f32> { return vec3<f32>(triangles[i * 3u], triangles[i * 3u + 1u], triangles[i * 3u + 2u]); }

// Depth and influence: extend only the physical nearest edge outside a
// triangle, fading to zero at the shared carrier's sampling reserve.
fn depth(point: vec3<f32>, face: u32, radius: f32) -> vec2<f32> {
    let p = array<vec3<f32>, 3>(triangle_point(face * 3u), triangle_point(face * 3u + 1u), triangle_point(face * 3u + 2u));
    let low = min(p[0].xy, min(p[1].xy, p[2].xy));
    let high = max(p[0].xy, max(p[1].xy, p[2].xy));
    if (any(point.xy < low - radius) || any(point.xy > high + radius)) { return vec2<f32>(0.0); }
    let a = p[1].xy - p[0].xy; let b = p[2].xy - p[0].xy;
    let denominator = cross2(a,b);
    if (abs(denominator) <= 1e-12) { return vec2<f32>(0.0); }
    let offset = point.xy - p[0].xy;
    let u = cross2(offset,b) / denominator; let v = cross2(a,offset) / denominator;
    if (u >= 0.0 && v >= 0.0 && u + v <= 1.0) {
        return vec2<f32>(p[0].z + u * (p[1].z - p[0].z) + v * (p[2].z - p[0].z), 1.0);
    }
    var nearest = radius * radius; var result = vec2<f32>(0.0);
    for (var i = 0u; i < 3u; i += 1u) {
        let first = p[i]; let second = p[(i+1u)%3u];
        let edge = second.xy - first.xy; let length2 = dot(edge,edge);
        if (length2 == 0.0) { continue; }
        let t = clamp(dot(point.xy - first.xy,edge) / length2, 0.0, 1.0);
        let delta = point.xy - mix(first.xy,second.xy,t); let distance2 = dot(delta,delta);
        if (distance2 < nearest) {
            nearest = distance2;
            let fade = sqrt(distance2) / radius;
            result = vec2<f32>(mix(first.z,second.z,t), 1.0 - fade * fade * (3.0 - 2.0 * fade));
        }
    }
    return result;
}

fn bounds(point: vec3<f32>, start: u32, end: u32, radius: f32, clearance: f32) -> vec2<f32> {
    var low = 1e30; var high = -1e30;
    for (var face = start; face < end; face += 1u) {
        let sample = depth(point,face,radius);
        if (sample.y > 0.0) {
            low = min(low, point.z + (sample.x - clearance - point.z) * sample.y);
            high = max(high, point.z + (sample.x + clearance - point.z) * sample.y);
        }
    }
    return vec2<f32>(low,high);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    let frame = frame_at(0u);
    let original = frame_point(frame,formed(i));
    var point = original;
    let body = bounds(point,0u,ends[0u],0.0,0.0);
    var midpoint = frame.origin.z;
    if (body.x <= body.y) { midpoint = (body.x + body.y) * 0.5; }
    let anterior = point.z > midpoint;
    if (body.x <= body.y) { point.z = select(min(point.z,body.x - params.body_clearance),max(point.z,body.y + params.body_clearance),anterior); }
    for (var group = 1u; group < params.groups; group += 1u) {
        let layer = bounds(point,ends[group - 1u],ends[group],reserve[0],params.plate_clearance);
        if (layer.x <= layer.y) { point.z = select(min(point.z,layer.x),max(point.z,layer.y),anterior); }
    }
    let d = point - original;
    let local = vec3<f32>(dot(frame.x,d),dot(frame.y,d),dot(frame.z,d));
    for (var k = 0u; k < 3u; k += 1u) { delta[i * 3u + k] = local[k]; }
}
