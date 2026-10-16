fn cloth_vector(point:u32, offset:u32)->vec3<f32> {
    let first=FIRST_POINT+point*POINT_WORDS+offset;
    return vec3<f32>(fit[first],fit[first+1u],fit[first+2u]);
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x>=params.rows*params.columns) {return;}
    let column=id.x%params.columns;
    let row=id.x/params.columns+params.first_row;
    let a=samples[column*3u];let b=samples[column*3u+1u];
    let weight=bitcast<f32>(samples[column*3u+2u]);
    let base=mix3(cloth_vector(a,0u),cloth_vector(b,0u),weight);
    let fraction=f32(row)/f32(params.rows-1u+params.first_row);
    let height=params.bottom+params.height*fraction;
    let t=height/params.full_height;
    let along=f32(column)/f32(params.columns-1u);
    let release=t*t*(3.0-2.0*t);
    var center=vec3<f32>(0.0);
    for(var i=0u;i+1u<params.columns;i+=1u) {
        let sa=samples[i*3u];let sb=samples[i*3u+1u];
        let sw=bitcast<f32>(samples[i*3u+2u]);
        center+=mix3(cloth_vector(sa,0u),cloth_vector(sb,0u),sw);
    }
    center/=f32(params.columns-1u);
    let pleat=params.depth*0.5*(1.0-cos(TAU*params.pleats*along))*release;
    let delta=base-center;
    let radial=host_normalized(vec3<f32>(delta.x,0.0,delta.z));
    let taper=mix(1.0,params.top_scale,t);
    let origin=vec3<f32>(center.x,base.y+height,center.z);
    let support=cloth_body_support(origin,radial);
    let radius=max(length(vec2<f32>(delta.x,delta.z))*taper,support)+pleat;
    let point=origin+radial*radius;
    let first=FIRST_POINT+(params.first+id.x)*POINT_WORDS;
    fit[first]=point.x;fit[first+1u]=point.y;fit[first+2u]=point.z;
    fit[first+3u]=radial.x;fit[first+4u]=radial.y;fit[first+5u]=radial.z;
}
