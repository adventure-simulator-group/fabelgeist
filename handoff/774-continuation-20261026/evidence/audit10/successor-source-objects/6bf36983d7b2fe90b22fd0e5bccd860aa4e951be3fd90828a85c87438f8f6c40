fn chart_origin() -> vec3<f32> { return vec3<f32>(0.0); }
fn chart_offset(u: f32, axial: f32) -> f32 {
    return design[8u]*2.5*(params.value2-axial)/(params.value2-params.value1);
}
fn shaped_point(angle: f32, across: f32, axial: f32) -> vec3<f32> {
    var base = mix(frames[16u],frames[17u],axial)
        - pow(1.0-across,10.0)*design[4u]*pow(axial,3.0)
        + design[5u]*pow(abs(2.0*across-1.0),8.0)*pow(1.0-axial,8.0);
    if (design[6u]>0.0 && params.value3>=design[6u]) { base-=design[7u]; }
    let slope = design[3u]*pow(axial,4.0);
    var low = base; var high = base;
    if (slope>0.0) {
        var extent = frames[17u]-frames[16u];
        for (var i=0u;i<4u;i++) {
            high=base+extent;
            if (high-base-slope*abs(carrier_point(angle,high).x)>=0.0) { break; }
            extent*=2.0;
        }
        // Explicit failure survives later shell evaluation and readback.
        if (high-base-slope*abs(carrier_point(angle,high).x)<0.0) { return invalid_chart_point(); }
        for (var i=0u;i<24u;i++) {
            let middle=(low+high)*0.5;
            if (middle-base-slope*abs(carrier_point(angle,middle).x)<0.0) { low=middle; } else { high=middle; }
        }
    }
    let height=(low+high)*0.5;
    let point=carrier_point(angle,height);
    return vec3<f32>(point.x,height,point.y);
}
fn inner_angle(axial: f32) -> f32 {
    let sign=params.value0;
    let gap=design[2u]+chart_offset(0.0,axial);
    var low=-design[0u]; var high=min(design[1u],PI*0.5);
    if (sign*shaped_point(sign*high,0.0,axial).x<=gap) { invalid_chart_point(); return 0.0; }
    if (sign*shaped_point(sign*low,0.0,axial).x < gap) {
        for (var i=0u;i<24u;i++) {
            let middle=(low+high)*0.5;
            if (sign*shaped_point(sign*middle,0.0,axial).x<gap) { low=middle; } else { high=middle; }
        }
        low=high;
    }
    return low;
}
fn chart_point(u: f32,v: f32) -> vec3<f32> {
    let axial=mix(params.value1,params.value2,v);
    let sign=params.value0;
    let across=select(1.0-u,u,sign>0.0);
    let row=u32(round(v*f32(COURSE_ROWS)));
    let low=frames[u32(frames[45u])+u32(params.value3)*(COURSE_ROWS+1u)+row];
    let point=shaped_point(sign*mix(low,design[1u],across),across,axial);
    if (point.y<frames[18u] || point.y>frames[19u]) { return invalid_chart_point(); }
    let section=carrier_section((point.y-frames[16u])/(frames[17u]-frames[16u]));
    if (any(section.zw<=vec2<f32>(0.0))) { return invalid_chart_point(); }
    return point;
}
