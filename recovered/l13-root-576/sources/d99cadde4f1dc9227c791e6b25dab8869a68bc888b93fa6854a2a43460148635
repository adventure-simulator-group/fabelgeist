//! Body-mounted fixtures fitted to canonical blade and grip surfaces.
use crate::construction::{ClearanceEnvelope, Detail, LoftEnd, Solid, Sweep};
use crate::mesh::native_part;
use crate::model::output::ModelBounds;
use crate::model::output::PartSource;
use crate::*;

fn holder_part(id: &str, material: Material, solid: Solid, _crease: f64) -> PartSource {
    PartSource::new(solid, material, id, id)
}
fn tube_centers(points: &[[f64; 3]], radius: f64, segments: usize) -> Result<Solid, GenerateError> {
    let mut path: Vec<_> = points.iter().map(|p| p.map(f64::from)).collect();
    path.push(path[0]);
    Solid::sweep(
        &path,
        &Sweep {
            width: radius * 2.0,
            depth: radius * 2.0,
            radial_segments: segments,
            ..Sweep::default()
        },
        Detail::High,
    )
    .map_err(GenerateError::Construction)
}
fn sheath_parts(
    blade: &FittedPart,
    design: &WeaponHolderDesign,
) -> Result<Vec<PartSource>, GenerateError> {
    let envelope =
        ClearanceEnvelope::new(&blade.positions, 32).map_err(GenerateError::Construction)?;
    let [base, tip] = envelope.extent();
    let clearance = metres(design.clearance);
    let wall = metres(design.wall_thickness);
    let cavity_tip = tip + clearance;
    let body = holder_part(
        "scabbard-body",
        design.body_material,
        envelope
            .shell(base, cavity_tip, clearance, wall, LoftEnd::Closed)
            .map_err(GenerateError::Construction)?,
        0.82,
    );
    let fitting = |name, lower, upper| {
        envelope
            .shell(lower, upper, clearance + wall, 0.002, LoftEnd::Open)
            .map(|solid| holder_part(name, design.fitting_material, solid, 0.82))
            .map_err(GenerateError::Construction)
    };
    let throat = fitting(
        "scabbard-throat",
        base,
        (base + metres(design.throat_length)).min(cavity_tip),
    )?;
    let chape = fitting(
        "scabbard-chape",
        (cavity_tip + wall - metres(design.chape_length)).max(base),
        cavity_tip + wall,
    )?;
    let hanger_half_width = metres(design.hanger_width) * 0.5;
    let hanger_half_height = metres(design.hanger_height) * 0.5;
    let mouth = bounds(&envelope.ring(base, clearance + wall));
    let hanger_center = [
        mouth.min[0] - hanger_half_width,
        base + hanger_half_height,
        (mouth.min[2] + mouth.max[2]) * 0.5,
    ];
    let hanger_centers = (0..28)
        .map(|index| {
            let angle = index as f64 / 28.0 * std::f64::consts::TAU;
            [
                hanger_center[0] + angle.cos() * hanger_half_width,
                hanger_center[1] + angle.sin() * hanger_half_height,
                hanger_center[2],
            ]
        })
        .collect::<Vec<_>>();
    let suspension = holder_part(
        "scabbard-suspension",
        design.body_material,
        tube_centers(&hanger_centers, metres(design.loop_bar_radius), 10)?,
        0.65,
    );
    Ok(vec![body, throat, chape, suspension])
}

fn haft_loop_parts(
    grip: &FittedPart,
    design: &WeaponHolderDesign,
) -> Result<Vec<PartSource>, GenerateError> {
    let center_x = (grip.bounds.min[0] + grip.bounds.max[0]) * 0.5;
    let center_z = (grip.bounds.min[2] + grip.bounds.max[2]) * 0.5;
    let grip_length = grip.bounds.max[1] - grip.bounds.min[1];
    let y = grip.bounds.min[1] + grip_length * (f64::from(design.loop_position.0) / 1000.0);
    let radius_x = (grip.bounds.max[0] - grip.bounds.min[0]) * 0.5
        + metres(design.clearance)
        + metres(design.loop_bar_radius);
    let radius_z = (grip.bounds.max[2] - grip.bounds.min[2]) * 0.5
        + metres(design.clearance)
        + metres(design.loop_bar_radius);
    let ring_centers = (0..24)
        .map(|index| {
            let angle = index as f64 / 24.0 * std::f64::consts::TAU;
            [
                center_x + angle.cos() * radius_x,
                y,
                center_z + angle.sin() * radius_z,
            ]
        })
        .collect::<Vec<_>>();
    let hanger_half_width = metres(design.hanger_width) * 0.5;
    let hanger_half_height = metres(design.hanger_height) * 0.5;
    let hanger_center = [center_x - radius_x - hanger_half_width, y, center_z];
    let hanger_centers = (0..28)
        .map(|index| {
            let angle = index as f64 / 28.0 * std::f64::consts::TAU;
            [
                hanger_center[0] + angle.cos() * hanger_half_width,
                hanger_center[1] + angle.sin() * hanger_half_height,
                hanger_center[2],
            ]
        })
        .collect::<Vec<_>>();
    Ok(vec![
        holder_part(
            "haft-frog",
            design.body_material,
            tube_centers(&ring_centers, metres(design.loop_bar_radius), 10)?,
            0.65,
        ),
        holder_part(
            "belt-loop",
            design.body_material,
            tube_centers(&hanger_centers, metres(design.loop_bar_radius), 10)?,
            0.65,
        ),
    ])
}

pub fn generate_holder(
    design: &WeaponHolderDesign,
) -> Result<GeneratedWeaponHolder, GenerateError> {
    Ok(HolderConstruction::new(design)?.render(design))
}
pub(crate) struct HolderConstruction {
    parts: Vec<PartSource>,
    extent: ModelBounds,
    grip: [f64; 3],
    pub(crate) derived: DerivedProperties,
}
impl HolderConstruction {
    pub(crate) fn new(design: &WeaponHolderDesign) -> Result<Self, GenerateError> {
        crate::validation::holder_identity(design).map_err(GenerateError::Invalid)?;
        let weapon = crate::model::Construction::new(&design.fitted_weapon.recipe)
            .map_err(GenerateError::Construction)?;
        let grip = weapon.physical.control_point;
        let parts = match design.kind {
            WeaponHolderKind::BladeSheath => {
                let blade_id = design
                    .fitted_weapon
                    .recipe
                    .components
                    .iter()
                    .find(|component| {
                        matches!(&component.shape, crate::recipe::Shape::LoftedBlade(_))
                    })
                    .map(|component| component.id.as_deref().expect("gameplay component ID"))
                    .ok_or(GenerateError::MissingHolderGeometry("blade"))?;
                let blade = weapon
                    .sources
                    .iter()
                    .find(|part| part.component_id == blade_id)
                    .ok_or(GenerateError::MissingHolderGeometry("blade"))?;
                sheath_parts(&FittedPart::new(blade), design)?
            }
            WeaponHolderKind::HaftLoop => {
                let grip_id = design
                    .fitted_weapon
                    .recipe
                    .components
                    .iter()
                    .find(|component| component.role == Some(ComponentRole::Grip))
                    .map(|component| component.id.as_deref().expect("gameplay component ID"))
                    .ok_or(GenerateError::MissingHolderGeometry("grip"))?;
                let grip_part = weapon
                    .sources
                    .iter()
                    .find(|part| part.component_id == grip_id)
                    .ok_or(GenerateError::MissingHolderGeometry("grip"))?;
                haft_loop_parts(&FittedPart::new(grip_part), design)?
            }
        };
        let all_positions = parts
            .iter()
            .flat_map(|part| part.solid.positions.iter().copied())
            .collect::<Vec<_>>();
        let extent = bounds(&all_positions);
        let physics = PhysicalProperties::from_sources(&parts, grip);
        Ok(Self {
            parts,
            grip,
            derived: DerivedProperties {
                mass_kg: physics.mass_kg as f32,
                length_m: (extent.max[1] - extent.min[1]) as f32,
                moment_of_inertia_kg_m2: physics.moment_of_inertia_kg_m2 as f32,
                center_of_mass_from_grip_m: physics.center_of_mass_from_grip_m as f32,
                balance: physics.balance as f32,
                ..DerivedProperties::default()
            },
            extent,
        })
    }
    fn render(self, design: &WeaponHolderDesign) -> GeneratedWeaponHolder {
        GeneratedWeaponHolder {
            design_hash: holder_design_hash(design),
            kind: design.kind,
            grip: self.grip.map(|n| n as f32),
            bounds: Bounds {
                min: self.extent.min.map(|n| n as f32),
                max: self.extent.max.map(|n| n as f32),
            },
            parts: self
                .parts
                .into_iter()
                .map(ModelPart::from_source)
                .map(native_part)
                .collect(),
            derived: self.derived,
        }
    }
}

fn metres(value: Millimeters) -> f64 {
    f64::from(value.0) / 1000.0
}
struct FittedPart {
    positions: Vec<[f64; 3]>,
    bounds: ModelBounds,
}
impl FittedPart {
    fn new(part: &PartSource) -> Self {
        let positions = part.solid.positions.clone();
        Self {
            bounds: bounds(&positions),
            positions,
        }
    }
}
fn bounds(points: &[[f64; 3]]) -> ModelBounds {
    let mut bounds = ModelBounds {
        min: [f64::INFINITY; 3],
        max: [f64::NEG_INFINITY; 3],
    };
    for p in points {
        for (i, coordinate) in p.iter().enumerate() {
            bounds.min[i] = bounds.min[i].min(*coordinate);
            bounds.max[i] = bounds.max[i].max(*coordinate);
        }
    }
    bounds
}
