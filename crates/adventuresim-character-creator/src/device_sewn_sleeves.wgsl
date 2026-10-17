// Keep the attachment cap supported while releasing into the authored volume.
const SLEEVE_ATTACHMENT_RELEASE:f32=0.18;
// Cubic joint spans preserve a continuous tangent through the elbow.
fn sleeve_curve(a:vec3<f32>,b:vec3<f32>,ta:vec3<f32>,tb:vec3<f32>,t:f32)->vec3<f32> {
    let t2=t*t;let t3=t2*t;
    return a*(2.0*t3-3.0*t2+1.0)+ta*(t3-2.0*t2+t)+b*(-2.0*t3+3.0*t2)+tb*(t3-t2);
}
fn sleeve_tangent(a:vec3<f32>,b:vec3<f32>,ta:vec3<f32>,tb:vec3<f32>,t:f32)->vec3<f32> {
    let t2=t*t;
    return host_normalized(a*(6.0*t2-6.0*t)+ta*(3.0*t2-4.0*t+1.0)+b*(-6.0*t2+6.0*t)+tb*(3.0*t2-2.0*t));
}
fn sleeve_joint(index:u32)->vec3<f32> {
    return vec3<f32>(joints[index*8u],joints[index*8u+1u],joints[index*8u+2u]);
}
fn sleeve_vector(index:u32,offset:u32)->vec3<f32> {
    let word=FIRST_POINT+index*POINT_WORDS+offset;
    return vec3<f32>(fit[word],fit[word+1u],fit[word+2u]);
}
struct SleevePoint {point:vec3<f32>,normal:vec3<f32>};
fn frill_fold(angle:f32)->f32 {
    let phase=fract(angle/TAU*params.fold_count+1.0);
    if (params.fold_profile==0u) {return (1.0-cos(TAU*phase))*0.5;}
    let mouth=params.fold_mouth;let valley=params.fold_valley;
    var local=phase/mouth;var high=1.0-valley;
    if (phase>=mouth) {local=(phase-mouth)/(1.0-mouth);high=-valley;}
    return valley+high*sqrt(max(1.0-pow(2.0*local-1.0,2.0),0.0));
}
fn sleeve_point(raw:u32)->SleevePoint {
    let record=raw*GRID_WORDS;
    let source_row=grid[record];let angle=bitcast<f32>(grid[record+1u]);
    let start=boundary[grid[record+2u]];let end=boundary[grid[record+3u]];
    let blend=bitcast<f32>(grid[record+4u]);
    let frame=frame_at(0u);
    let source=host_point(frame,mix3(sleeve_vector(start,0u),sleeve_vector(end,0u),blend));
    if (source_row==0u) {
        let local=host_normalized(mix3(sleeve_vector(start,3u),sleeve_vector(end,3u),blend));
        return SleevePoint(source,frame.x*local.x+frame.y*local.y+frame.z*local.z);
    }
    let row=source_row-1u;
    let shoulder=sleeve_joint(params.shoulder);let elbow=sleeve_joint(params.elbow);
    let wrist=sleeve_joint(params.wrist);
    let upper_axis=host_normalized(elbow-shoulder);
    let length_upper=length(elbow-shoulder);let length_lower=length(wrist-elbow);
    let total=length_upper+length_lower;
    let reach=total*params.reach;
    if (reach<=params.cuff_length+params.frill_length) {atomicOr(&sleeve_status[0],1u);return SleevePoint(vec3<f32>(0.0),vec3<f32>(0.0));}
    let gather_end=1.0-params.frill_length/reach;
    let cuff_start=gather_end-params.cuff_length/reach;
    var t=gather_end;
    if (row<BODY_ROWS) {t=stations[row]*cuff_start;}
    if (row==BODY_ROWS) {t=cuff_start;}
    if (row==BODY_ROWS+2u) {t=mix(gather_end,1.0,0.5);}
    if (row==BODY_ROWS+3u) {t=1.0;}
    let distance=t*reach;
    let lower_axis=host_normalized(wrist-elbow);
    let bend=host_normalized(upper_axis+lower_axis);
    let before_elbow=distance<length_upper;
    let a=select(elbow,shoulder,before_elbow);
    let b=select(wrist,elbow,before_elbow);
    let span=select(length_lower,length_upper,before_elbow);
    let local_t=select((distance-length_upper)/length_lower,distance/length_upper,before_elbow);
    let ta=select(bend,upper_axis,before_elbow)*span;
    let tb=select(lower_axis,bend,before_elbow)*span;
    let center=sleeve_curve(a,b,ta,tb,local_t);
    let axis=sleeve_tangent(a,b,ta,tb,local_t);
    let up=host_normalized(vec3<f32>(0.0,1.0,0.0)-axis*axis.y);
    let across=host_cross(axis,up);
    let radial=up*cos(angle)+across*sin(angle);
    let gather=smoothstep(0.80*cuff_start,cuff_start,t);
    let cap_fullness=mix(1.0,params.proximal_fullness,smoothstep(0.0,SLEEVE_ATTACHMENT_RELEASE,t));
    let fullness=mix(cap_fullness,1.0,smoothstep(SLEEVE_ATTACHMENT_RELEASE,params.fullness_release,t));
    var radius=mix(params.radius*fullness,params.cuff_radius,gather);
    if (params.frill_length>0.0) {
        let flare=params.frill_flare+params.fold_depth*(frill_fold(angle)-0.5);
        radius+=flare*smoothstep(gather_end,1.0,t);
    }
    var point=center+radial*radius;
    // Preserve the cut boundary at the attachment; release into an even volume.
    let release=smoothstep(0.0,SLEEVE_ATTACHMENT_RELEASE,t);
    point=mix3(source,center+radial*radius,release);
    return SleevePoint(point,radial);
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x>=params.count) {return;}
    let word=id.x*SOURCE_WORDS;
    var point=vec3<f32>(0.0);var normal=vec3<f32>(0.0);
    for (var corner=0u;corner<3u;corner++) {
        let sample=sleeve_point(pattern[word+corner]);let weight=bitcast<f32>(pattern[word+6u+corner]);
        point+=sample.point*weight;normal+=sample.normal*weight;
    }
    let frame=frame_at(0u);let local=host_local(frame,point);normal=host_normalized(normal);
    let n=vec3<f32>(host_dot(frame.x,normal),host_dot(frame.y,normal),host_dot(frame.z,normal));
    let first=FIRST_POINT+(params.first+id.x)*POINT_WORDS;
    fit[first]=local.x;fit[first+1u]=local.y;fit[first+2u]=local.z;
    fit[first+3u]=n.x;fit[first+4u]=n.y;fit[first+5u]=n.z;
}
