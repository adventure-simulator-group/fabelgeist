use super::*;

#[test]
fn every_usage_combination_matches_the_original_descriptor_policy() {
    let cases: Vec<[u32; 2]> = serde_json::from_str(include_str!(
        "../../../../../tests/fixtures/buffer_usage.json"
    ))
    .unwrap();
    let roles = [
        BufferUse::Uniform,
        BufferUse::Storage,
        BufferUse::Vertex,
        BufferUse::Index,
        BufferUse::Indirect,
        BufferUse::CopySource,
        BufferUse::CopyDestination,
        BufferUse::HostWrite,
        BufferUse::HostRead,
    ];
    for [mask, expected] in cases {
        let mut definition = BufferDefinition::new();
        for (slot, role) in roles.into_iter().enumerate() {
            if mask & (1 << slot) != 0 {
                definition = definition.with_usage(role);
            }
        }
        let flags = match definition.usage {
            UsageSelection::Unspecified => GENERAL_USAGE,
            UsageSelection::Explicit(flags) => flags,
        };
        assert_eq!(flags.bits(), expected);
    }
}

#[test]
fn labelled_clones_retain_exact_spelling_and_explicit_mapping_selection() {
    let label = "  DEVICE\0mémoire 🦀  ";
    let definition = BufferDefinition::map_read()
        .with_usage(BufferUse::CopyDestination)
        .with_label(label.into());
    let cloned = definition.clone();
    assert_eq!(<&str>::from(cloned.label.as_ref().unwrap()), label);
    assert!(matches!(cloned.usage, UsageSelection::Explicit(flags)
        if flags == wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST));
    assert!(
        matches!(BufferDefinition::default().usage, UsageSelection::Explicit(flags)
        if flags == GENERAL_USAGE)
    );
}
