//! Shared manufacturing constraints, evaluated before geometry allocation.
use super::*;
mod dimensions;
mod graphs;
mod melee;
mod ranged;
mod shields;

const MAX_COMPONENTS: usize = 96;
const MAX_AUTHORED_STATIONS: usize = 512;
const MAX_OUTLINE_STATIONS: usize = 2048;
const MAX_RECIPE_BYTES: usize = 65536;
const MAX_WORLD_METRES: f64 = 20.0;
const MIN_MANUFACTURED_METRES: f64 = 0.000001;
const MAX_SAMPLING_REQUEST: u16 = 256;
const MAX_ROUND_GRIP_RADIUS: f64 = 0.022;
const MAX_SWORD_GRIP_WIDTH: f64 = 0.038;
const MAX_SWORD_GRIP_THICKNESS: f64 = 0.028;

/// Stable authority error classifications; the accompanying field names belong
/// to the schema, never to mutable labels or renderer text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RecipeError {
    Budget,
    Dimension,
    Proportion,
    Attachment,
    Profile,
    Grip,
    Clearance,
    SelfIntersection,
    MissingNode,
    Facing,
}
impl std::fmt::Display for RecipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Facing => {
                "working-end facing dependencies need distinct directional components and no cycles"
            }
            Self::SelfIntersection => "construction outline or centerline intersects itself",
            Self::MissingNode => "guard construction references a missing node",
            Self::Budget => "recipe exceeds its bounded construction budget",
            Self::Dimension => {
                "manufactured dimensions must be positive and within the supported scale"
            }
            Self::Proportion => "component proportions cannot form the declared construction",
            Self::Attachment => "component placement must use one valid parent declaration",
            Self::Profile => "profile needs increasing stations and positive radii",
            Self::Grip => "grip exceeds anatomical dimensions",
            Self::Clearance => "moving or fitted components lack required clearance",
        })
    }
}
impl std::error::Error for RecipeError {}
type Checked = Result<(), RecipeError>;
fn require(condition: bool, error: RecipeError) -> Checked {
    if condition { Ok(()) } else { Err(error) }
}
fn positive(value: f64) -> Checked {
    require(
        (MIN_MANUFACTURED_METRES..=MAX_WORLD_METRES).contains(&value),
        RecipeError::Dimension,
    )
}
fn nonnegative(value: f64) -> Checked {
    require(
        (0.0..=MAX_WORLD_METRES).contains(&value),
        RecipeError::Dimension,
    )
}
fn proportion(condition: bool) -> Checked {
    require(condition, RecipeError::Proportion)
}
fn clearance(condition: bool) -> Checked {
    require(condition, RecipeError::Clearance)
}
fn bounded(value: f64) -> Checked {
    require(value.abs() <= MAX_WORLD_METRES, RecipeError::Dimension)
}
enum ProfileEnds {
    Open,
    Poles,
}
fn profile(points: &[[Metres; 2]], ends: ProfileEnds) -> Checked {
    require(
        (2..=MAX_AUTHORED_STATIONS).contains(&points.len()),
        RecipeError::Budget,
    )?;
    for (index, point) in points.iter().enumerate() {
        bounded(point[0].get())?;
        if matches!(ends, ProfileEnds::Poles) && (index == 0 || index == points.len() - 1) {
            nonnegative(point[1].get())?;
        } else {
            positive(point[1].get())?;
        }
    }
    require(
        points.iter().any(|p| p[1].get() > MIN_MANUFACTURED_METRES),
        RecipeError::Profile,
    )?;
    require(
        points.windows(2).all(|p| p[1][0].get() > p[0][0].get()),
        RecipeError::Profile,
    )
}
impl Recipe {
    /// Validate the canonical recipe before constructing, integrating or storing
    /// it. JSON decoding alone only establishes field types and finite numbers.
    pub fn validate(&self) -> Checked {
        require(
            self.components.len() <= MAX_COMPONENTS
                && (!self.components.is_empty() || self.shaft.is_some()),
            RecipeError::Budget,
        )?;
        require(
            serde_json::to_vec(self)
                .map_err(|_| RecipeError::Budget)?
                .len()
                <= MAX_RECIPE_BYTES,
            RecipeError::Budget,
        )?;
        if let Some(p) = &self.shaft {
            positive(p.length.get())?;
            positive(p.radius.get())?;
            let scale = p
                .bottom_scale
                .map_or(1.0, Ratio::get)
                .max(p.top_scale.map_or(0.92, Ratio::get));
            require(
                p.radius.get() * scale <= MAX_ROUND_GRIP_RADIUS + 1e-9,
                RecipeError::Grip,
            )?;
            if let Some(n) = p.segments {
                require(
                    (3..=MAX_SAMPLING_REQUEST).contains(&n.0),
                    RecipeError::Budget,
                )?;
            }
        }
        if let Some(clearance) = self.grip_clearance {
            nonnegative(clearance.get())?;
        }
        let mut identifiers = std::collections::BTreeSet::new();
        for (index, component) in self.components.iter().enumerate() {
            let id = component.resolved_id(index);
            require(
                !id.trim().is_empty()
                    && id.len() <= 128
                    && id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_'))
                    && id != "weapon"
                    && !(id == "shaft" && self.shaft.is_some())
                    && identifiers.insert(id),
                RecipeError::Attachment,
            )?;
            component.validate()?;
        }
        facing::directions(self).map_err(|_| RecipeError::Facing)?;
        Ok(())
    }
}
impl Component {
    fn validate(&self) -> Checked {
        let parents = usize::from(self.mount.is_some())
            + usize::from(self.attach.is_some())
            + usize::from(self.stretch_between.is_some());
        require(parents == 1, RecipeError::Attachment)?;
        if let Some(p) = self.offset {
            for n in p {
                bounded(n.get())?;
            }
        }
        if let Some(rotation) = self.rotation {
            for angle in rotation {
                require(angle.get().abs() <= 3600.0, RecipeError::Dimension)?;
            }
        }
        if let Some(p) = &self.attach {
            require(self.offset.is_none(), RecipeError::Attachment)?;
            if let Some(n) = p.overlap {
                require((0.0..=0.15).contains(&n.get()), RecipeError::Attachment)?;
            }
            if let Some(offset) = p.offset {
                for n in offset {
                    bounded(n.get())?;
                }
            }
        }
        dimensions::check(&self.shape)?;
        graphs::check(&self.shape)?;
        ranged::check(&self.shape)?;
        melee::check(&self.shape)?;
        shields::check(self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
