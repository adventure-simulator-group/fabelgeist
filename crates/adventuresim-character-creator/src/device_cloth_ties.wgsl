const TIE_TANGENT_STEP:f32=0.001;
const KNOT_LAYER_GAUGES:f32=1.5;
fn tie_vector(point:u32, offset:u32)->vec3<f32> {
    let first=FIRST_POINT+point*POINT_WORDS+offset;
    return vec3<f32>(fit[first],fit[first+1u],fit[first+2u]);
}
fn tie_path(kind:u32,t:f32,front:vec3<f32>,back:vec3<f32>,center:vec3<f32>,knot:vec3<f32>,outward:vec3<f32>)->vec3<f32> {
    let vertical=vec3<f32>(0.0,1.0,0.0);
    let sideways=host_cross(vertical,outward);
    if (kind==FRONT_BAND||kind==BACK_BAND) {
        let start=select(front,back,kind==BACK_BAND);
        let start_delta=start-center;let knot_delta=knot-center;
        let angle=atan2(start_delta.z,start_delta.x);
        let knot_angle=atan2(knot_delta.z,knot_delta.x);
        let turn=atan2(sin(knot_angle-angle),cos(knot_angle-angle));
        let direction=vec3<f32>(cos(angle+turn*t),0.0,sin(angle+turn*t));
        let origin=vec3<f32>(center.x,mix(start.y,knot.y,t),center.z);
        let nominal=mix(length(start_delta.xz),length(knot_delta.xz),t);
        let radius=max(nominal,cloth_body_support(origin,direction)+params.placement);
        let layer=select(0.0,KNOT_LAYER_GAUGES*params.gauge*t,kind==BACK_BAND);
        return origin+direction*radius+outward*layer;
    }
    if (kind==KNOT) {
        return knot+vertical*(sin(TAU*t)*params.width*0.5)
            +outward*(cos(TAU*t)*params.width*0.4);
    }
    let sign=select(1.0,-1.0,kind==LEFT_LOOP||kind==LEFT_TAIL);
    if (kind==LEFT_LOOP||kind==RIGHT_LOOP) {
        return knot+sideways*(sign*params.bow*sin(TAU*0.5*t))
            +vertical*(params.bow*0.35*sin(TAU*t))
            +outward*(params.width*0.5*sin(TAU*0.5*t)
                +params.gauge*KNOT_LAYER_GAUGES*(select(0.0,2.0,kind==RIGHT_LOOP)+t));
    }
    return knot+sideways*(sign*params.width*(0.25+t))
        -vertical*(params.tail*t)+outward*(params.width*0.25*t);
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x>=params.count) {return;}
    let word=id.x*TIE_WORDS;
    let front=tie_vector(tape[word],0u)+host_normalized(tie_vector(tape[word],3u))*params.placement;
    let back=tie_vector(tape[word+1u],0u)+host_normalized(tie_vector(tape[word+1u],3u))*params.placement;
    let normal=tie_vector(tape[word],3u)+tie_vector(tape[word+1u],3u);
    let kind=tape[word+2u];let t=bitcast<f32>(tape[word+3u]);
    let lane=bitcast<f32>(tape[word+4u]);
    let center=vec3<f32>(joints[params.neck*8u],(front.y+back.y)*0.5,joints[params.neck*8u+2u]);
    var outward=host_normalized(vec3<f32>(normal.x,0.0,normal.z));
    if (length(outward)<0.5) {outward=host_normalized(vec3<f32>((front.x+back.x)*0.5-center.x,0.0,(front.z+back.z)*0.5-center.z));}
    let radius=cloth_body_support(center,outward)+params.width+params.placement;
    let knot=center+outward*radius;
    let point=tie_path(kind,t,front,back,center,knot,outward);
    let before=tie_path(kind,max(0.0,t-TIE_TANGENT_STEP),front,back,center,knot,outward);
    let after=tie_path(kind,min(1.0,t+TIE_TANGENT_STEP),front,back,center,knot,outward);
    let tangent=host_normalized(after-before);
    var facing=outward;
    if (kind==FRONT_BAND||kind==BACK_BAND) {facing=host_normalized(vec3<f32>(point.x-center.x,0.0,point.z-center.z));}
    if (kind==KNOT) {facing=host_normalized(vec3<f32>(0.0,sin(TAU*t),0.0)+outward*cos(TAU*t));}
    let across=host_normalized(host_cross(facing,tangent));
    let position=point+across*(params.width*lane);
    let first=FIRST_POINT+(params.first+id.x)*POINT_WORDS;
    fit[first]=position.x;fit[first+1u]=position.y;fit[first+2u]=position.z;
    fit[first+3u]=facing.x;fit[first+4u]=facing.y;fit[first+5u]=facing.z;
}
