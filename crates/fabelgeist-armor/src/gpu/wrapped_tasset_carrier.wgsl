// Fit metadata: bottom/top, support low/high/count, layer low/high/count,
// center depth, clearance, gauge, flare, course count; controls at word 32.
fn cross2(a: vec2<f32>, b: vec2<f32>) -> f32 { return a.x*b.y-a.y*b.x; }
fn hull_point(node: u32, corner: u32) -> vec2<f32> {
    let at = HULL_START + node * HULL_WORDS + 1u + corner*2u;
    return vec2<f32>(frames[at], frames[at+1u]);
}
fn hull_radius(node: u32, center: vec2<f32>, direction: vec2<f32>) -> f32 {
    let count = u32(frames[HULL_START + node * HULL_WORDS]);
    var radius = 0.012;
    for (var i = 0u; i < count; i++) {
        let a = hull_point(node,i) - center;
        let edge = hull_point(node,(i+1u)%count) - hull_point(node,i);
        let determinant = cross2(direction,edge);
        if (abs(determinant) < 1e-8) { continue; }
        let distance = cross2(a,edge)/determinant;
        let along = cross2(a,direction)/determinant;
        if (along >= 0.0 && along <= 1.0) { radius = max(radius,distance); }
    }
    return radius;
}
fn envelope(height: f32, center: vec2<f32>, direction: vec2<f32>, low: f32, high: f32, count: u32, first: u32) -> f32 {
    let coordinate = clamp((height-low)/(high-low),0.0,1.0)*f32(count-1u);
    let index = min(u32(coordinate), count-2u);
    return mix(hull_radius(first+index,center,direction),hull_radius(first+index+1u,center,direction),coordinate-f32(index));
}
fn carrier_section(axial: f32) -> vec4<f32> {
    let weights = vec3<f32>((1.0-axial)*(1.0-axial),2.0*axial*(1.0-axial),axial*axial);
    var section = vec4<f32>(0.0);
    for (var i = 0u; i < 3u; i++) {
        let at = 32u+i*4u;
        section += weights[i]*vec4<f32>(frames[at],frames[at+1u],frames[at+2u],frames[at+3u]);
    }
    return section;
}
fn carrier_point(angle: f32, height: f32) -> vec2<f32> {
    let axial = (height-frames[16u])/(frames[17u]-frames[16u]);
    let section = carrier_section(axial);
    let direction = vec2<f32>(sin(angle),cos(angle));
    // Bracket searches may continue beyond the anatomical span. Signed radii
    // are mathematically valid there; the final chart rejects nonpositive ones.
    // This equivalent product form also has a finite limit at degenerate
    // exploratory sections. Such probes must not poison the final fit status.
    let radii = abs(section.zw);
    let denominator = length(direction*radii.yx);
    var radius = max(radii.x,radii.y);
    if (denominator>0.0) { radius = radii.x*radii.y/denominator; }
    radius += frames[44u];
    if (height >= frames[18u] && height <= frames[19u]) {
        radius = max(radius,envelope(height,section.xy,direction,frames[18u],frames[19u],u32(frames[20u]),0u));
    }
    radius = radius * max(1.0+frames[30u]*(1.0-axial),1.0)+frames[28u]+frames[29u]+0.004;
    var point = section.xy + direction*radius;
    if (frames[25u] > 0.0) {
        let center = vec2<f32>(0.0,frames[27u]);
        let distance = length(point-center);
        let ray = (point-center)/max(distance,1e-8);
        let required = envelope(height,center,ray,frames[23u],frames[24u],u32(frames[25u]),u32(frames[20u]))+frames[28u]+frames[29u];
        let transition = (frames[17u]-frames[16u])/frames[31u];
        let blend = smoothstep(0.0,1.0,clamp((height-frames[23u]+transition)/transition,0.0,1.0));
        point += ray*max(required-distance,0.0)*blend;
    }
    return point;
}
