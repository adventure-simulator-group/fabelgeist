//! An orbit camera, and the few matrices the renderer needs.
//!
//! Matrices are column-major `[[f32; 4]; 4]`, as WGSL's `mat4x4<f32>` reads
//! them, with depth from zero to one as wgpu's clip space has it.

pub type Mat4 = [[f32; 4]; 4];

pub fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn normalize(a: [f32; 3]) -> [f32; 3] {
    scale(a, 1.0 / dot(a, a).sqrt().max(1e-12))
}

pub fn multiply(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut m = [[0.0; 4]; 4];
    for (c, column) in m.iter_mut().enumerate() {
        for (r, value) in column.iter_mut().enumerate() {
            *value = (0..4).map(|k| a[k][r] * b[c][k]).sum();
        }
    }
    m
}

/// A right-handed view matrix looking from `eye` at `target`.
pub fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Mat4 {
    let f = normalize(sub(target, eye));
    let s = normalize(cross(f, up));
    let u = cross(s, f);
    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0],
    ]
}

pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fov_y * 0.5).tan();
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far / (near - far), -1.0],
        [0.0, 0.0, near * far / (near - far), 0.0],
    ]
}

pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
    [
        [2.0 / (right - left), 0.0, 0.0, 0.0],
        [0.0, 2.0 / (top - bottom), 0.0, 0.0],
        [0.0, 0.0, 1.0 / (near - far), 0.0],
        [
            -(right + left) / (right - left),
            -(top + bottom) / (top - bottom),
            near / (near - far),
            1.0,
        ],
    ]
}

/// Orbits a target: drag to turn, pan to move the target, wheel to come
/// closer.
#[derive(Clone, Debug)]
pub struct OrbitCamera {
    pub target: [f32; 3],
    /// Around the vertical, radians.
    pub yaw: f32,
    /// Above the horizon, radians.
    pub pitch: f32,
    pub distance: f32,
    pub fov_y: f32,
}

impl OrbitCamera {
    /// Framing a box, from above and to one side.
    pub fn framing(lo: [f32; 3], mut hi: [f32; 3]) -> Self {
        // Room for what will grow: a young scene is framed as if its plants
        // were already a third of its width tall.
        let width = (hi[0] - lo[0]).max(hi[2] - lo[2]);
        hi[1] = hi[1].max(lo[1] + width * 0.35);
        let center = scale(add(lo, hi), 0.5);
        let extent = dot(sub(hi, lo), sub(hi, lo)).sqrt().max(4.0);
        Self {
            target: [center[0], lo[1] + (hi[1] - lo[1]) * 0.3, center[2]],
            yaw: 0.7,
            pitch: 0.42,
            distance: extent * 1.0,
            fov_y: 40f32.to_radians(),
        }
    }

    pub fn eye(&self) -> [f32; 3] {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        add(self.target, scale([cp * sy, sp, cp * cy], self.distance))
    }

    /// Unit vectors: forward, right, up.
    pub fn basis(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let forward = normalize(sub(self.target, self.eye()));
        let right = normalize(cross(forward, [0.0, 1.0, 0.0]));
        (forward, right, cross(right, forward))
    }

    pub fn rotate(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx * 0.006;
        self.pitch = (self.pitch + dy * 0.006).clamp(-0.1, 1.5);
    }

    /// Move the target across the screen, by pixels of a view `height`
    /// pixels tall.
    pub fn pan(&mut self, dx: f32, dy: f32, height: f32) {
        let (_, right, up) = self.basis();
        let per_pixel = 2.0 * self.distance * (self.fov_y * 0.5).tan() / height.max(1.0);
        self.target = add(self.target, scale(right, -dx * per_pixel));
        self.target = add(self.target, scale(up, dy * per_pixel));
    }

    pub fn zoom(&mut self, wheel: f32) {
        self.distance = (self.distance * (wheel * 0.0015).exp()).clamp(1.0, 5000.0);
    }

    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let near = (self.distance * 0.005).max(0.05);
        let far = self.distance * 20.0 + 1000.0;
        multiply(
            &perspective(self.fov_y, aspect, near, far),
            &look_at(self.eye(), self.target, [0.0, 1.0, 0.0]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transform(m: &Mat4, p: [f32; 3]) -> [f32; 4] {
        let mut out = [0.0; 4];
        for (r, value) in out.iter_mut().enumerate() {
            *value = m[0][r] * p[0] + m[1][r] * p[1] + m[2][r] * p[2] + m[3][r];
        }
        out
    }

    #[test]
    fn the_target_lands_in_the_middle_of_the_view() {
        let camera = OrbitCamera::framing([-10.0, 0.0, -10.0], [10.0, 5.0, 10.0]);
        let clip = transform(&camera.view_projection(1.5), camera.target);
        assert!((clip[0] / clip[3]).abs() < 1e-4 && (clip[1] / clip[3]).abs() < 1e-4);
        let depth = clip[2] / clip[3];
        assert!((0.0..1.0).contains(&depth), "{depth}");
    }
}
