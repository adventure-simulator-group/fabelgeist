//! The body measurements and design presets that ship with this crate.
//!
//! The reference project keeps these in its `assets/` directory and reads them
//! from disk. They are bundled here as well so that a front-end can offer the
//! presets without knowing where the crate lives on disk; the files themselves
//! are still there for the CLI to read.

/// A bundled YAML file: its stem and its contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedYaml {
    pub name: &'static str,
    pub yaml: &'static str,
}

/// Body measurement sets (the `body:` files).
pub const BODIES: &[NamedYaml] = &[
    NamedYaml {
        name: "mean_all",
        yaml: include_str!("../assets/bodies/mean_all.yaml"),
    },
    NamedYaml {
        name: "mean_female",
        yaml: include_str!("../assets/bodies/mean_female.yaml"),
    },
    NamedYaml {
        name: "mean_male",
        yaml: include_str!("../assets/bodies/mean_male.yaml"),
    },
    NamedYaml {
        name: "f_smpl_average_A40",
        yaml: include_str!("../assets/bodies/f_smpl_average_A40.yaml"),
    },
    NamedYaml {
        name: "m_smpl_average_A40",
        yaml: include_str!("../assets/bodies/m_smpl_average_A40.yaml"),
    },
];

/// Design presets (the `design:` files).
pub const DESIGNS: &[NamedYaml] = &[
    NamedYaml {
        name: "t-shirt",
        yaml: include_str!("../assets/design_params/t-shirt.yaml"),
    },
    NamedYaml {
        name: "default",
        yaml: include_str!("../assets/design_params/default.yaml"),
    },
];

/// Look a bundled file up by stem.
pub fn find(set: &'static [NamedYaml], name: &str) -> Option<&'static NamedYaml> {
    set.iter().find(|entry| entry.name == name)
}
