// Shared course profile: axial runs distal to proximal; seam is axial fraction.
fn textile_fullness(axial: f32, seam: f32, role: f32, profile: vec4<f32>, constriction: f32) -> f32 {
    let course = 1.0 / profile.x;
    let puff = min(floor(axial / course), profile.x - 1.0);
    let proximal = puff / max(profile.x - 1.0, 1.0);
    let scale = select(mix(profile.z, 1.0, proximal), 1.0, profile.x == 1.0);
    var edge = 0.0;
    if (role == 1.0) { edge = course * constriction * 0.5; }
    let phase = clamp(((axial + seam) - puff * course - edge) / (course - 2.0 * edge), 0.0, 1.0);
    var fullness = profile.y * scale * pow(max(sin(PI * phase), 0.0), profile.w);
    if (role == 2.0) { fullness = 0.0; }
    return fullness;
}
