# bevy_line_boil

A Bevy plugin that applies a classic cartoon "line boil" effect via turbulent vertex displacement. Creates a hand-drawn animation look by jittering vertices at fixed frame intervals.

## Features

- Screen-space vertex displacement for that hand-drawn 2D wobble
- Time-quantized animation (frame-held effect like classic cartoons)
- Smooth spatial coherence (nearby vertices move together)
- Works with any glTF model using StandardMaterial
- Configurable intensity, frame rate, and noise frequency

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_line_boil = "0.1"
```

Then in your app:

```rust
use bevy::prelude::*;
use bevy_line_boil::{LineBoil, LineBoilPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LineBoilPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a model with line boil effect
    commands.spawn((
        SceneRoot(asset_server.load("character.glb#Scene0")),
        LineBoil::subtle(),
    ));
}
```

## Presets

- `LineBoil::subtle()` - Very gentle wobble (intensity=0.008, frame_rate=8, noise=6)
- `LineBoil::aggressive()` - More pronounced effect (intensity=0.04, frame_rate=4, noise=12)

## Custom Configuration

```rust
LineBoil::new()
    .with_intensity(0.02)      // How far vertices move
    .with_frame_rate(6.0)      // FPS for time quantization (lower = more "held")
    .with_noise_frequency(10.0) // Turbulence scale
    .with_seed(42.0)           // Variation between entities
```

## Compatibility

| bevy_line_boil | Bevy |
|----------------|------|
| 0.1            | 0.17 |

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
