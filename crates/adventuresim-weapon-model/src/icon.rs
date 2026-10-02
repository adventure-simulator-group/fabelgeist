//! Deterministic studio-lit orthographic portraits of generated weapon meshes.

use std::collections::HashMap;

use thiserror::Error;
mod render;
mod screen;
use screen::{occupied_bounds, raw_coordinates, relative_screen};
mod studio;
pub use studio::environment_hdr;

use crate::{
    ComponentRole, GenerateError, GeneratedWeapon, GeneratedWeaponHolder, MeshPart, WeaponDesign,
    WeaponHolderDesign, WeaponHolderKind, generate, generate_holder,
};

/// Bump whenever projection, framing, or rasterization changes.
pub const ICON_RENDERER_VERSION: u16 = 5;
const MAX_HEAD_ZOOM: f32 = 2.0;
const SYNTHETIC_HEAD_LENGTH_M: f32 = 0.12;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WeaponIconLayout {
    /// Guard centered; complete handle extends upper-right and blade exits lower-left.
    HiltFocus,
    /// Head root centered; complete head extends upper-left and shaft exits lower-right.
    HeadFocus,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconBounds {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl IconBounds {
    fn empty() -> Self {
        Self {
            min: [f32::INFINITY; 2],
            max: [f32::NEG_INFINITY; 2],
        }
    }

    fn include(&mut self, point: [f32; 2]) {
        for (axis, value) in point.into_iter().enumerate() {
            self.min[axis] = self.min[axis].min(value);
            self.max[axis] = self.max[axis].max(value);
        }
    }

    pub fn center(self) -> [f32; 2] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
        ]
    }

    fn is_finite(self) -> bool {
        self.min.into_iter().chain(self.max).all(f32::is_finite)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WeaponIconSpec {
    pub size: u16,
    pub supersampling: u8,
}

impl Default for WeaponIconSpec {
    fn default() -> Self {
        Self {
            size: 96,
            supersampling: 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeaponIcon {
    pub size: u16,
    pub rgba: Vec<u8>,
    pub layout: WeaponIconLayout,
    /// True when the base layout is horizontally mirrored, as for scabbards.
    pub mirrored: bool,
    /// Normalized semantic framing anchor: guard, head root, sheath throat, or loop center.
    pub framing_anchor: [f32; 2],
    /// Head-layout magnification relative to fitting the full head/socket assembly.
    pub head_zoom: f32,
    /// Normalized bounds of the hilt/head assembly that controls framing.
    pub focus_bounds: IconBounds,
    /// Normalized bounds of all occupied output pixels.
    pub occupied_bounds: IconBounds,
}

impl WeaponIcon {
    pub fn encode_png(&self) -> Result<Vec<u8>, IconError> {
        studio::encode_png(self.size, self.rgba.clone())
    }
}

#[derive(Debug, Error)]
pub enum IconError {
    #[error("icon dimensions or supersampling are invalid")]
    InvalidSpec,
    #[error("weapon generation failed: {0}")]
    Generate(#[from] GenerateError),
    #[error("weapon has no usable icon focus geometry")]
    MissingFocus,
    #[error("weapon icon rasterization failed")]
    Rasterization,
    #[error("weapon icon PNG encoding failed: {0}")]
    Png(String),
}

/// Presentation classification is derived from component semantics, not catalog IDs.
/// This keeps arbitrary modular polearm heads in the head-focused family.
pub fn icon_layout(design: &WeaponDesign) -> WeaponIconLayout {
    let has_guard = design
        .recipe
        .components
        .iter()
        .any(|component| component.role == Some(ComponentRole::Guard));
    let has_sword_blade = design
        .recipe
        .components
        .iter()
        .any(|component| matches!(component.shape, crate::recipe::Shape::LoftedBlade(_)));
    if has_guard && has_sword_blade {
        WeaponIconLayout::HiltFocus
    } else {
        WeaponIconLayout::HeadFocus
    }
}

pub fn generate_icon(design: &WeaponDesign, spec: WeaponIconSpec) -> Result<WeaponIcon, IconError> {
    let generated = generate(design)?;
    let layout = icon_layout(design);
    let projection = Projection::new(design, &generated, layout)?;
    rasterize_icon(&generated.parts, spec, projection)
}

/// Render a fitted scabbard or haft loop from its independently persisted recipe.
///
/// Blade sheaths use the same diagonal as their weapon, with the throat centered
/// and the long body intentionally cropped toward the lower-left. Compact haft
/// loops fit the complete loop-and-hanger assembly in the opposite diagonal.
pub fn generate_holder_icon(
    design: &WeaponHolderDesign,
    spec: WeaponIconSpec,
) -> Result<WeaponIcon, IconError> {
    let generated = generate_holder(design)?;
    let projection = Projection::holder(&generated)?;
    rasterize_icon(&generated.parts, spec, projection)
}

fn rasterize_icon(
    parts: &[MeshPart],
    spec: WeaponIconSpec,
    projection: Projection,
) -> Result<WeaponIcon, IconError> {
    let (rgba, alpha) = render::render(parts, spec, &projection)?;
    let output_size = usize::from(spec.size);
    let occupied_bounds = occupied_bounds(&alpha, output_size).ok_or(IconError::Rasterization)?;
    Ok(WeaponIcon {
        size: spec.size,
        rgba,
        layout: projection.layout,
        mirrored: projection.mirror_x,
        framing_anchor: projection.framing_anchor,
        head_zoom: projection.head_zoom,
        focus_bounds: projection.focus_bounds,
        occupied_bounds,
    })
}

struct Projection {
    layout: WeaponIconLayout,
    lateral_center: f32,
    axial_center: f32,
    target: [f32; 2],
    scale: f32,
    mirror_x: bool,
    flip_lateral: bool,
    framing_anchor: [f32; 2],
    head_zoom: f32,
    focus_bounds: IconBounds,
}

impl Projection {
    fn new(
        design: &WeaponDesign,
        generated: &GeneratedWeapon,
        layout: WeaponIconLayout,
    ) -> Result<Self, IconError> {
        let roles: HashMap<&str, ComponentRole> = design
            .recipe
            .components
            .iter()
            .filter_map(|component| component.id.as_deref().zip(component.role))
            .collect();
        let mut focus = Vec::new();
        let mut principal_head = Vec::new();
        for part in &generated.parts {
            let role = roles.get(part.component_id.as_str()).copied();
            let selected = match layout {
                WeaponIconLayout::HiltFocus => matches!(
                    role,
                    Some(ComponentRole::Grip | ComponentRole::Guard | ComponentRole::Structure)
                ),
                WeaponIconLayout::HeadFocus => {
                    matches!(role, Some(ComponentRole::Head | ComponentRole::Socket))
                }
            };
            if selected {
                focus.extend(part.positions.iter().copied());
            }
            if layout == WeaponIconLayout::HeadFocus && role == Some(ComponentRole::Head) {
                principal_head.extend(part.positions.iter().copied());
            }
        }
        if principal_head.is_empty() && layout == WeaponIconLayout::HeadFocus {
            synthetic_head_focus(generated, &mut focus, &mut principal_head);
        }
        if focus.is_empty() {
            return Err(IconError::MissingFocus);
        }

        let anchor = framing_anchor(design, generated, &roles, layout)?;
        let [lateral_center, axial_center] = raw_coordinates(anchor, layout);
        let target = [0.5, 0.5];
        let safe = IconBounds {
            min: [0.02, 0.02],
            max: [0.98, 0.98],
        };
        let base_scale = fit_scale(
            &focus,
            [lateral_center, axial_center],
            layout,
            false,
            false,
            target,
            safe,
        )? * 0.96;
        let (scale, head_zoom, framed_focus) = match layout {
            WeaponIconLayout::HiltFocus => (base_scale, 1.0, focus),
            WeaponIconLayout::HeadFocus => {
                let head_scale = fit_scale(
                    &principal_head,
                    [lateral_center, axial_center],
                    layout,
                    false,
                    false,
                    target,
                    safe,
                )? * 0.96;
                let zoom = (head_scale / base_scale).clamp(1.0, MAX_HEAD_ZOOM);
                (base_scale * zoom, zoom, principal_head)
            }
        };
        let mut projection = Self {
            layout,
            lateral_center,
            axial_center,
            target,
            scale,
            mirror_x: false,
            flip_lateral: false,
            framing_anchor: target,
            head_zoom,
            focus_bounds: IconBounds::empty(),
        };
        for point in framed_focus {
            projection.focus_bounds.include(projection.point(point));
        }
        Ok(projection)
    }

    fn holder(generated: &GeneratedWeaponHolder) -> Result<Self, IconError> {
        let (layout, mirror_x, flip_lateral, target, focus, anchor) = match generated.kind {
            WeaponHolderKind::BladeSheath => {
                let focus = generated
                    .parts
                    .iter()
                    .filter(|part| {
                        matches!(
                            part.component_id.as_str(),
                            "scabbard-throat" | "scabbard-suspension"
                        )
                    })
                    .flat_map(|part| part.positions.iter().copied())
                    .collect::<Vec<_>>();
                let throat = generated
                    .parts
                    .iter()
                    .find(|part| part.component_id == "scabbard-throat")
                    .ok_or(IconError::MissingFocus)?;
                (
                    WeaponIconLayout::HiltFocus,
                    true,
                    false,
                    [0.24, 0.24],
                    focus,
                    bounds_center(throat.bounds),
                )
            }
            WeaponHolderKind::HaftLoop => {
                let focus = generated
                    .parts
                    .iter()
                    .flat_map(|part| part.positions.iter().copied())
                    .collect::<Vec<_>>();
                (
                    WeaponIconLayout::HeadFocus,
                    false,
                    false,
                    [0.5, 0.5],
                    focus,
                    bounds_center(generated.bounds),
                )
            }
        };
        if focus.is_empty() {
            return Err(IconError::MissingFocus);
        }
        let [lateral_center, axial_center] = raw_coordinates(anchor, layout);
        let safe = IconBounds {
            min: [0.02, 0.02],
            max: [0.98, 0.98],
        };
        let scale = fit_scale(
            &focus,
            [lateral_center, axial_center],
            layout,
            mirror_x,
            flip_lateral,
            target,
            safe,
        )? * 0.96;
        let mut projection = Self {
            layout,
            lateral_center,
            axial_center,
            target,
            scale,
            mirror_x,
            flip_lateral,
            framing_anchor: target,
            head_zoom: 1.0,
            focus_bounds: IconBounds::empty(),
        };
        for point in focus {
            projection.focus_bounds.include(projection.point(point));
        }
        Ok(projection)
    }

    fn point(&self, point: [f32; 3]) -> [f32; 2] {
        let relative = relative_screen(
            raw_coordinates(point, self.layout),
            [self.lateral_center, self.axial_center],
            self.layout,
            self.mirror_x,
            self.flip_lateral,
        );
        [
            self.target[0] + relative[0] * self.scale,
            self.target[1] + relative[1] * self.scale,
        ]
    }
}

fn bounds_center(bounds: crate::Bounds) -> [f32; 3] {
    std::array::from_fn(|axis| (bounds.min[axis] + bounds.max[axis]) * 0.5)
}

fn fit_scale(
    points: &[[f32; 3]],
    center: [f32; 2],
    layout: WeaponIconLayout,
    mirror_x: bool,
    flip_lateral: bool,
    target: [f32; 2],
    safe: IconBounds,
) -> Result<f32, IconError> {
    let mut scale = f32::INFINITY;
    for point in points {
        let relative = relative_screen(
            raw_coordinates(*point, layout),
            center,
            layout,
            mirror_x,
            flip_lateral,
        );
        for axis in 0..2 {
            if relative[axis] < -1.0e-6 {
                scale = scale.min((target[axis] - safe.min[axis]) / -relative[axis]);
            } else if relative[axis] > 1.0e-6 {
                scale = scale.min((safe.max[axis] - target[axis]) / relative[axis]);
            }
        }
    }
    if !scale.is_finite() || scale <= 0.0 {
        return Err(IconError::MissingFocus);
    }
    Ok(scale)
}

fn framing_anchor(
    design: &WeaponDesign,
    generated: &GeneratedWeapon,
    roles: &HashMap<&str, ComponentRole>,
    layout: WeaponIconLayout,
) -> Result<[f32; 3], IconError> {
    let named_anchor = |name: &str| {
        generated
            .anchors
            .iter()
            .find(|anchor| anchor.name == name)
            .map(|anchor| anchor.position)
    };
    match layout {
        WeaponIconLayout::HiltFocus => {
            let main_guard = generated
                .parts
                .iter()
                .filter(|part| roles.get(part.component_id.as_str()) == Some(&ComponentRole::Guard))
                .max_by(|left, right| {
                    let left_span = (left.bounds.max[0] - left.bounds.min[0])
                        + (left.bounds.max[2] - left.bounds.min[2]);
                    let right_span = (right.bounds.max[0] - right.bounds.min[0])
                        + (right.bounds.max[2] - right.bounds.min[2]);
                    left_span.total_cmp(&right_span)
                })
                .ok_or(IconError::MissingFocus)?;
            named_anchor(&format!("{}.base", main_guard.component_id))
                .ok_or(IconError::MissingFocus)
        }
        WeaponIconLayout::HeadFocus => {
            let socket_roots = design
                .recipe
                .components
                .iter()
                .filter(|component| {
                    component.role == Some(ComponentRole::Socket)
                        && matches!(
                            component.shape,
                            crate::recipe::Shape::Socket(_) | crate::recipe::Shape::Sleeve(_)
                        )
                })
                .filter_map(|component| named_anchor(&format!("{}.top", component.id.as_deref()?)))
                .collect::<Vec<_>>();
            if let Some(root) = socket_roots
                .into_iter()
                .max_by(|left, right| left[1].total_cmp(&right[1]))
            {
                return Ok(root);
            }
            let mut bases = design
                .recipe
                .components
                .iter()
                .filter(|component| component.role == Some(ComponentRole::Head))
                .filter_map(|component| named_anchor(&format!("{}.base", component.id.as_deref()?)))
                .collect::<Vec<_>>();
            if bases.is_empty() {
                let top = design
                    .recipe
                    .components
                    .iter()
                    .filter(|component| {
                        matches!(
                            component.role,
                            Some(ComponentRole::Grip | ComponentRole::Structure)
                        )
                    })
                    .filter_map(|component| {
                        named_anchor(&format!("{}.top", component.id.as_deref()?))
                    })
                    .max_by(|left, right| left[1].total_cmp(&right[1]));
                if let Some(mut top) = top {
                    let span = generated.bounds.max[1] - generated.bounds.min[1];
                    top[1] -= SYNTHETIC_HEAD_LENGTH_M.min(span * 0.2);
                    bases.push(top);
                }
            }
            let count = bases.len() as f32;
            if count == 0.0 {
                return Err(IconError::MissingFocus);
            }
            Ok(bases.into_iter().fold([0.0; 3], |mut sum, point| {
                for axis in 0..3 {
                    sum[axis] += point[axis] / count;
                }
                sum
            }))
        }
    }
}

fn synthetic_head_focus(
    generated: &GeneratedWeapon,
    focus: &mut Vec<[f32; 3]>,
    principal_head: &mut Vec<[f32; 3]>,
) {
    let span = generated.bounds.max[1] - generated.bounds.min[1];
    let focus_cutoff = generated.bounds.max[1] - span * 0.2;
    let head_cutoff = generated.bounds.max[1] - SYNTHETIC_HEAD_LENGTH_M.min(span * 0.2);
    for point in generated
        .parts
        .iter()
        .flat_map(|part| part.positions.iter().copied())
        .filter(|point| point[1] >= focus_cutoff)
    {
        focus.push(point);
        focus.push([point[0], focus_cutoff, point[2]]);
        if point[1] >= head_cutoff {
            principal_head.push(point);
            principal_head.push([point[0], head_cutoff, point[2]]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_design;

    fn roles(design: &WeaponDesign) -> HashMap<&str, ComponentRole> {
        design
            .recipe
            .components
            .iter()
            .filter_map(|component| component.id.as_deref().zip(component.role))
            .collect()
    }

    fn named(generated: &GeneratedWeapon, name: &str) -> [f32; 3] {
        generated
            .anchors
            .iter()
            .find(|anchor| anchor.name == name)
            .unwrap_or_else(|| panic!("missing anchor {name}"))
            .position
    }

    #[test]
    fn semantic_icon_anchors_are_guard_center_and_head_root() {
        let sword = default_design("longsword").unwrap();
        let generated = generate(&sword).unwrap();
        assert_eq!(
            framing_anchor(
                &sword,
                &generated,
                &roles(&sword),
                WeaponIconLayout::HiltFocus
            )
            .unwrap(),
            named(&generated, "guard.base")
        );

        let polearm = default_design("halberd").unwrap();
        let generated = generate(&polearm).unwrap();
        assert_eq!(
            framing_anchor(
                &polearm,
                &generated,
                &roles(&polearm),
                WeaponIconLayout::HeadFocus
            )
            .unwrap(),
            named(&generated, "socket.top")
        );
    }
}
