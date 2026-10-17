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
fn sleeve_vector(index:u32)->vec3<f32> {
    let word=FIRST_POINT+index*POINT_WORDS;
    return vec3<f32>(fit[word],fit[word+1u],fit[word+2u]);
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x>=params.count) {return;}
    let row=id.x/params.columns;let column=id.x%params.columns;
    let frame=frame_at(0u);
    let shoulder=sleeve_joint(params.shoulder);let elbow=sleeve_joint(params.elbow);
    let wrist=sleeve_joint(params.wrist);
    let upper_axis=host_normalized(elbow-shoulder);
    let length_upper=length(elbow-shoulder);let length_lower=length(wrist-elbow);
    let total=length_upper+length_lower;
    let reach=total*params.reach;
    let cuff_start=max(0.0,1.0-params.cuff_length/reach);
    var t=1.0;
    if (row<BODY_ROWS) {t=stations[row]*cuff_start;}
    if (row==BODY_ROWS) {t=cuff_start;}
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
    let angle=angles[column];let radial=up*cos(angle)+across*sin(angle);
    let gather=smoothstep(0.80*cuff_start,cuff_start,t);
    let radius=mix(params.radius,params.cuff_radius,gather);
    var point=center+radial*radius;
    // Preserve the cut boundary at the attachment; release into an even volume.
    let source=host_point(frame,sleeve_vector(boundary[column]));
    let release=smoothstep(0.0,0.18,t);
    point=mix3(source,center+radial*radius,release);
    let local=host_local(frame,point);
    let normal=vec3<f32>(host_dot(frame.x,radial),host_dot(frame.y,radial),host_dot(frame.z,radial));
    let first=FIRST_POINT+(params.first+id.x)*POINT_WORDS;
    fit[first]=local.x;fit[first+1u]=local.y;fit[first+2u]=local.z;
    fit[first+3u]=normal.x;fit[first+4u]=normal.y;fit[first+5u]=normal.z;
}
