//! Signed tetrahedral volume moments of the same solids emitted by the generator.
//!
//! Integration uses f64 to preserve small sections on long pike shafts. Assembled
//! components retain their authored overlaps; this is construction mass, not a
//! Boolean union or a reconstruction of unmodeled tangs and fastening hardware.
use crate::mesh::{RawMesh, construction::ConstructedWeapon};
use crate::{ComponentRole, DerivedMaterialMass, DerivedProperties, MaterialClass};

#[derive(Default)]
pub(crate) struct SolidMoments {
    pub volume_m3: f64,
    first_moment_m4: [f64; 3],
    transverse_moment_m5: f64,
}

impl SolidMoments {
    pub fn new(mesh: &RawMesh, grip: [f32; 3]) -> Self {
        let mut result = Self::default();
        for triangle in mesh.indices.as_chunks::<3>().0 {
            let [a, b, c] = triangle.map(|index| {
                let point = mesh.positions[index as usize];
                std::array::from_fn::<_, 3, _>(|axis| {
                    f64::from(point[axis]) - f64::from(grip[axis])
                })
            });
            let volume = (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0;
            result.volume_m3 += volume;
            let mut second = [0.0; 3];
            for axis in 0..3 {
                result.first_moment_m4[axis] += volume * (a[axis] + b[axis] + c[axis]) / 4.0;
                second[axis] = volume
                    * (a[axis].powi(2)
                        + b[axis].powi(2)
                        + c[axis].powi(2)
                        + a[axis] * b[axis]
                        + a[axis] * c[axis]
                        + b[axis] * c[axis])
                    / 10.0;
            }
            result.transverse_moment_m5 += second[1] + (second[0] + second[2]) * 0.5;
        }
        result
    }
}

impl ConstructedWeapon<'_> {
    pub fn properties(&self) -> DerivedProperties {
        let mut mass = 0.0;
        let mut first = 0.0;
        let mut inertia = 0.0;
        let mut minimum = f32::INFINITY;
        let mut maximum = f32::NEG_INFINITY;
        let mut head_minimum = f32::INFINITY;
        let mut head_maximum = f32::NEG_INFINITY;
        for part in &self.parts {
            let moments = SolidMoments::new(&part.mesh, self.grip);
            let density = f64::from(part.component.material.density_kg_m3());
            mass += moments.volume_m3 * density;
            first += moments.first_moment_m4[1] * density;
            inertia += moments.transverse_moment_m5 * density;
            for point in &part.mesh.positions {
                minimum = minimum.min(point[1]);
                maximum = maximum.max(point[1]);
                if part.component.role == ComponentRole::Head {
                    head_minimum = head_minimum.min(point[1]);
                    head_maximum = head_maximum.max(point[1]);
                }
            }
        }
        let grip_to_tip_m = maximum - self.grip[1];
        DerivedProperties {
            mass_kg: mass as f32,
            length_m: maximum - minimum,
            grip_to_tip_m,
            striking_head_length_m: if head_minimum.is_finite() {
                head_maximum - head_minimum
            } else {
                0.0
            },
            center_of_mass_from_grip_m: (first / mass) as f32,
            moment_of_inertia_kg_m2: inertia as f32,
            balance: ((inertia / mass).sqrt() / f64::from(grip_to_tip_m)) as f32,
        }
    }

    pub fn material_masses(&self) -> Vec<DerivedMaterialMass> {
        let mut masses = Vec::<DerivedMaterialMass>::new();
        for part in &self.parts {
            let material: MaterialClass = part.component.material;
            let mass_kg = (SolidMoments::new(&part.mesh, self.grip).volume_m3
                * f64::from(material.density_kg_m3())) as f32;
            if let Some(entry) = masses.iter_mut().find(|entry| entry.material == material) {
                entry.mass_kg += mass_kg;
            } else {
                masses.push(DerivedMaterialMass { material, mass_kg });
            }
        }
        masses
    }
}
