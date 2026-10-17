fn cloth_vector(point:u32, offset:u32)->vec3<f32> {
    let first=FIRST_POINT+point*POINT_WORDS+offset;
    return vec3<f32>(fit[first],fit[first+1u],fit[first+2u]);
}
fn collar_sample(column:u32)->vec3<f32> {
    return mix3(cloth_vector(samples[column*SAMPLE_WORDS],0u),cloth_vector(samples[column*SAMPLE_WORDS+1u],0u),bitcast<f32>(samples[column*SAMPLE_WORDS+2u]));
}
struct CollarFrame { center:vec3<f32>, seam_length:f32, oval:vec3<f32>, }
fn collar_frame(column:u32)->CollarFrame {
    var center=vec3<f32>(0.0);
    var seam_length=0.0;
    for(var i=0u;i+1u<params.columns;i+=1u) {
        let sa=samples[i*SAMPLE_WORDS];let sb=samples[i*SAMPLE_WORDS+1u];
        let sw=bitcast<f32>(samples[i*SAMPLE_WORDS+2u]);
        let seam_point=mix3(cloth_vector(sa,0u),cloth_vector(sb,0u),sw);
        center+=seam_point;
        seam_length+=length(collar_sample(i+1u)-seam_point);
    }
    center/=f32(params.columns-1u);
    if (params.oval_depth>0.0) {
        // Native rig boundary: eight floats per joint, position in XYZ.
        center.x=joints[params.neck*8u];center.z=joints[params.neck*8u+2u];
    }
    var oval=vec3<f32>(0.0);
    if (params.oval_depth>0.0) {
        let width=seam_length*params.top_scale/params.oval_length;
        let angle=bitcast<f32>(samples[column*SAMPLE_WORDS+3u]);
        let lateral=vec3<f32>(params.lateral_x,0.0,params.lateral_z);
        let anterior=vec3<f32>(params.anterior_x,0.0,params.anterior_z);
        oval=lateral*(width*cos(angle))+anterior*(width*params.oval_depth*sin(angle));
    }
    return CollarFrame(center,seam_length,oval);
}
