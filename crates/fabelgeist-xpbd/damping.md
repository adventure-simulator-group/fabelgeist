# Per-second velocity damping

`dynamics::DampingRate` is the shared bespoke type for exponential particle
velocity drag per second. Solver settings, shell materials, fabric presets and
creator drape stages retain it through storage and forwarding. A positive rate
reduces velocity; a negative rate amplifies it.

Use `DampingRate::per_second(0.6)` for a native literal or `DampingRate::from`
when decoding a native scalar. Both admit every f32 bit pattern, including
signed zeros, negatives, infinities and NaNs. Native equality, ordering and
scalar Debug remain unchanged. Admission does not impose a global minimum.

Shell materials require a finite nonnegative rate, before checking positive
thickness. Creator stages require a finite rate within the inclusive range
0 to 20 per second. Their validation order and diagnostics remain unchanged.
These are local consumer policies rather than restrictions on constructing a
rate. The sewing preset remains 8 per second; fabric presets keep their authored
rates.

Projection with `f32::from(rate)` occurs directly at the shader uniform, the
independent host-reference equation, scalar serde and egui ports. The shader
keeps its existing gravity, exponential drag and speed-limiting operation order.
Creator JSON still uses a scalar: nonfinite values serialize as null, which the
existing f32 deserializer rejects. The slider projects its typed endpoints and
value for egui, then admits the edited scalar once.

Dimensionless animation-preview damping is a separate role. Duration, gravity,
speed, spring damping and other material properties retain their own contracts.
This type introduces no new solver admission, numerical policy or render claim.
