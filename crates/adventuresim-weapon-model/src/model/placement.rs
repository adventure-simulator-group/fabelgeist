//! Recipe-local attachment frames, shaft fitting and grip placement.
use super::*;
use std::collections::BTreeMap;
mod anchors;
mod fitting;
use super::grip::grip_point;
use anchors::*;
use fitting::*;

pub(super) struct ResolvedComponent {
    pub(super) component: Component,
    pub(super) id: String,
    pub(super) label: String,
    pub(super) offset: Point,
    pub(super) rotation: Point,
    pub(super) shaft_contact: Option<f64>,
    pub(super) grip_width: Option<f64>,
    pub(super) grip_seat_radius: Option<f64>,
}
pub(super) struct Resolved {
    pub(super) output: ResolvedRecipe,
    pub(super) components: Vec<ResolvedComponent>,
    pub(super) grip: Point,
}

fn metres(point: Point) -> Result<[Metres; 3], String> {
    Ok([
        Metres::new(point[0])?,
        Metres::new(point[1])?,
        Metres::new(point[2])?,
    ])
}

pub(super) fn resolve(recipe: &Recipe) -> Result<Resolved, String> {
    let grip_width = recipe.components.iter().find_map(|c| {
        if c.id.as_deref() == Some(GRIP_COMPONENT_ID) {
            match &c.shape {
                Shape::OvalGrip(p) => Some(p.width.get()),
                Shape::SlabGrip(p) => Some(p.width.get()),
                _ => None,
            }
        } else {
            None
        }
    });

    let mut recipe = recipe.clone();
    crate::recipe::facing::apply(&mut recipe)?;
    let mut graph = PlacementGraph {
        frames: BTreeMap::new(),
        rotations: BTreeMap::new(),
        components: Vec::new(),
        grip_width,
        shaft: recipe.shaft.clone(),
    };
    graph.frames.insert(WEAPON_ROOT_FRAME.into(), [0.0; 3]);
    if let Some(shaft) = &recipe.shaft {
        graph.frames.insert(SHAFT_BOTTOM_FRAME.into(), [0.0; 3]);
        graph
            .frames
            .insert(SHAFT_TOP_FRAME.into(), [0.0, shaft.length.get(), 0.0]);
    }
    for (index, component) in recipe.components.iter_mut().enumerate() {
        graph.insert(index, component)?;
    }
    for component in &graph.components {
        mounts::attachment_contact(component, &graph.components, recipe.shaft.as_ref())?;
    }
    if let (Some(clearance), Some(base), Some(top)) = (
        recipe.grip_clearance,
        graph.frames.get(GRIP_BASE_FRAME),
        graph.frames.get(GRIP_TOP_FRAME),
    ) && clearance.get() > magnitude(sub(*top, *base))
    {
        return Err("hand control point must remain within the modeled grip".into());
    }
    let grip = grip_point(&recipe, &graph.frames);
    graph.frames.insert(WEAPON_GRIP_FRAME.into(), grip);
    Ok(Resolved {
        output: ResolvedRecipe {
            recipe,
            frames: graph.frames,
            errors: Vec::new(),
        },
        components: graph.components,
        grip,
    })
}

struct PlacementGraph {
    frames: BTreeMap<String, Point>,
    rotations: BTreeMap<String, Point>,
    components: Vec<ResolvedComponent>,
    grip_width: Option<f64>,
    shaft: Option<Shaft>,
}
impl PlacementGraph {
    fn insert(&mut self, index: usize, component: &mut Component) -> Result<(), String> {
        let Self {
            frames,
            rotations,
            components,
            grip_width,
            shaft,
        } = self;

        let id = component.resolved_id(index);
        component.id = Some(id.clone());
        let rotation = component.rotation.map_or([0.0; 3], |r| r.map(Degrees::get));
        let local = component.offset.map_or([0.0; 3], |p| p.map(Metres::get));
        if let Shape::Pommel(p) = &mut component.shape
            && let Some(profile) = &mut p.profile
        {
            for point in profile {
                point[0] = Metres::new(point[0].get() * p.length_scale.map_or(1.0, Ratio::get))?;
                point[1] = Metres::new(point[1].get() * p.width_scale.map_or(1.0, Ratio::get))?;
            }
        }
        let range = component.shape.range()?;
        let (mut offset, shaft_contact) =
            mounted(component, shaft.as_ref(), frames, rotation, local, range)?;
        offset = attached(component, frames, rotations, rotation, local, range, offset)?;
        if let Shape::Box(p) = &component.shape
            && p.fit_shaft_side == Some(true)
        {
            let shaft = shaft.as_ref().ok_or("side fitting requires shaft")?;
            offset[0] = if local[0] < 0.0 { -1.0 } else { 1.0 }
                * (shaft.radius.get() * shaft.top_scale.map_or(0.92, Ratio::get)
                    + p.size[0].get() / 2.0
                    - 0.002);
        }
        if let Shape::GuardAssembly(p) = &mut component.shape {
            guard_nodes::resolve(p, frames, offset, rotation)?;
        }
        let grip_seat_radius = seating(component, components, offset, rotation);
        component.offset = Some(metres(offset)?);
        let range = component.shape.range()?;
        register(component, &id, range, rotation, offset, frames, rotations)?;
        let resolved_component = ResolvedComponent {
            component: component.clone(),
            id,
            label: component.label.clone().unwrap_or_default(),
            offset,
            rotation,
            shaft_contact,
            grip_width: *grip_width,
            grip_seat_radius,
        };
        components.push(resolved_component);

        Ok(())
    }
}
