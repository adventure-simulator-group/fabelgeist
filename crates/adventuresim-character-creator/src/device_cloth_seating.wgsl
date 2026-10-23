// Closest surface support after procedural interpolation, in canonical metres.
fn cloth_closest_edge(point:vec3<f32>,a:vec3<f32>,b:vec3<f32>)->vec3<f32> {
    let edge=b-a;let squared=host_dot(edge,edge);
    if (squared==0.0) {return a;}
    return a+edge*clamp(host_dot(point-a,edge)/squared,0.0,1.0);
}
fn cloth_seated_point(point:vec3<f32>,clearance:f32)->vec3<f32> {
    var distance=3.402823466e+38;var closest=point;var normal=vec3<f32>(0.0);
    for (var triangle=0u;triangle<params.seating_face_count;triangle++) {
        let a=positions_at(seating_faces[triangle*3u]);
        let b=positions_at(seating_faces[triangle*3u+1u]);
        let c=positions_at(seating_faces[triangle*3u+2u]);
        let cross=host_cross(b-a,c-a);let squared=host_dot(cross,cross);
        if(squared==0.0){continue;}
        let projected=point-cross*(host_dot(point-a,cross)/squared);
        let first=host_dot(host_cross(b-a,projected-a),cross);
        let second=host_dot(host_cross(c-b,projected-b),cross);
        let third=host_dot(host_cross(a-c,projected-c),cross);
        var candidate=projected;
        if(first<0.0||second<0.0||third<0.0){
            candidate=cloth_closest_edge(point,a,b);
            let bc=cloth_closest_edge(point,b,c);let ca=cloth_closest_edge(point,c,a);
            if(length(bc-point)<length(candidate-point)){candidate=bc;}
            if(length(ca-point)<length(candidate-point)){candidate=ca;}
        }
        let gap=host_dot(point-candidate,point-candidate);
        if(gap<distance){
            distance=gap;closest=candidate;
            normal=cross/sqrt(squared);
        }
    }
    let gap=host_dot(point-closest,normal);
    return point+normal*max(clearance-gap,0.0);
}

// Concave body edges can expose another adjacent face after the first move.
const CLOTH_POINT_SEATING_STEPS:u32=3u;
fn cloth_fitted_point(point:vec3<f32>,clearance:f32)->vec3<f32> {
    var seated=point;
    for(var step=0u;step<CLOTH_POINT_SEATING_STEPS;step++){seated=cloth_seated_point(seated,clearance);}
    return seated;
}
