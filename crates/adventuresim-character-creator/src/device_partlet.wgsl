@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x>=params.rows*params.columns) {return;}
    let column=id.x%params.columns;
    let row=id.x/params.columns+params.first_row;
    let a=samples[column*SAMPLE_WORDS];let b=samples[column*SAMPLE_WORDS+1u];
    let weight=bitcast<f32>(samples[column*SAMPLE_WORDS+2u]);
    let base=mix3(cloth_vector(a,0u),cloth_vector(b,0u),weight);
    let fraction=f32(row)/f32(params.rows-1u+params.first_row);
    let height=select(params.bottom+params.height*fraction,row_heights[id.x/params.columns],params.first_row>0u);
    let t=height/params.full_height;
    let along=f32(column)/f32(params.columns-1u);
    let frame=collar_frame(column);
    let center=frame.center;let seam_length=frame.seam_length;
    var delta=base-center;
    var base_height=base.y;
    if (params.oval_depth>0.0) {
        let oval=frame.oval;
        let angle=bitcast<f32>(samples[column*SAMPLE_WORDS+3u]);
        let transition=clamp(height/params.taper_height,0.0,1.0);
        let seat=seating[params.columns]+seating[params.columns+1u]*cos(angle)+seating[params.columns+2u]*sin(angle)+seating[params.columns+3u]*cos(2.0*angle)+seating[params.columns+4u]*sin(2.0*angle);
        delta=oval;
        base_height=seat;
        if (transition<1.0) {
            let body_normal=host_normalized(mix3(cloth_vector(a,3u),cloth_vector(b,3u),weight));
            let upright=vec3<f32>(0.0,1.0,0.0);
            let end=vec3<f32>(center.x+oval.x,seat+params.taper_height,center.z+oval.z);
            let chord=end-base;
            var departure=host_normalized(upright-body_normal*body_normal.y);
            if (host_dot(departure,chord)<0.0) {departure=-departure;}
            let tangent_length=length(chord);
            let t2=transition*transition;let t3=t2*transition;
            var point=base*(2.0*t3-3.0*t2+1.0)+departure*tangent_length*(t3-2.0*t2+transition)
                +end*(-2.0*t3+3.0*t2)+upright*tangent_length*(t3-t2);
            point.y=max(point.y,cloth_body_height(point,point.y));
            delta=point-center;
            base_height=point.y-height;
        }
    }
    let radial=host_normalized(vec3<f32>(delta.x,0.0,delta.z));
    let taper=select(mix(1.0,params.top_scale,clamp(height/params.taper_height,0.0,1.0)),1.0,params.oval_depth>0.0);
    let origin=vec3<f32>(center.x,base_height+height,center.z);
    var support=0.0;
    if (params.oval_depth==0.0) {support=cloth_body_support(origin,radial);}
    var free_fraction=0.0;
    if(params.full_height>params.free_start){
        free_fraction=clamp((height-params.free_start)/(params.full_height-params.free_start),0.0,1.0);
    }
    let turn=free_fraction*free_fraction;
    let radius=max(length(vec2<f32>(delta.x,delta.z))*taper,support)+params.flare*turn*turn;
    let point=origin+radial*radius;
    let first=FIRST_POINT+(params.first+id.x)*POINT_WORDS;
    fit[first]=point.x;fit[first+1u]=point.y;fit[first+2u]=point.z;
    var normal=radial;
    if (params.oval_depth>0.0) {
        let body_normal=host_normalized(mix3(cloth_vector(a,3u),cloth_vector(b,3u),weight));
        let transition=clamp(height/params.taper_height,0.0,1.0);
        let blend=transition*transition*(3.0-2.0*transition);
        normal=host_normalized(mix3(body_normal,radial,blend));
    }
    fit[first+3u]=normal.x;fit[first+4u]=normal.y;fit[first+5u]=normal.z;
}
