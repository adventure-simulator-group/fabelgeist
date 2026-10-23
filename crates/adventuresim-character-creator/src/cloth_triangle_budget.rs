//! Count every exported cloth triangle, including invisible construction.
use anyhow::{Result, bail};
use fabelgeist_armor::{ArmorComponentRole, BuiltPart, EQUIPMENT_TRIANGLE_BUDGET};

struct ComponentTriangles {
    role: ArmorComponentRole,
    triangles: usize,
}

pub(crate) fn check(part: &BuiltPart) -> Result<()> {
    let count = part.indices.len() / 3;
    if count < EQUIPMENT_TRIANGLE_BUDGET {
        return Ok(());
    }
    let mut roles = Vec::<ComponentTriangles>::new();
    for component in &part.components {
        let count = (component.indices.end - component.indices.start) / 3;
        if let Some(entry) = roles.iter_mut().find(|entry| entry.role == component.role) {
            entry.triangles += count;
        } else {
            roles.push(ComponentTriangles {
                role: component.role,
                triangles: count,
            });
        }
    }
    let detail = roles
        .iter()
        .map(|entry| format!("{:?}={}", entry.role, entry.triangles))
        .collect::<Vec<_>>()
        .join(", ");
    bail!(
        "cloth article has {count} triangles; it must stay below {EQUIPMENT_TRIANGLE_BUDGET} including hidden construction ({detail})"
    )
}
