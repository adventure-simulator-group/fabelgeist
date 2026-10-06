# Euler composition orders

MHR (Momentum Human Rig) builds rest rotations from Euler angles in a named
composition order. Use the bespoke `math::EulerOrder` type with
`quat_from_euler` for radians or `quat_from_euler_degrees` for degrees. The
angles always occupy native `[x, y, z]` coordinates; the order selects which
rotation is applied first, second, and third. `EulerOrder::Xyz`, for example,
applies X first and produces `Rz * Ry * Rx`.

The six variants each apply every coordinate axis once. Arbitrary integer
arrays, repeated axes, and out-of-range axes are absent from the Rust API.
This restriction replaces the former raw-array helper contract. Angles,
quaternions, and matrices retain their existing native floating-point arrays.
The order projects to coordinate indexes privately where the quaternion
kernel indexes those arrays; callers retain the named order.

## FBX interpretation

`EulerOrder::from_fbx_code` interprets the native FBX `RotationOrder` integer
once at the rig loader boundary. Codes 1 through 5 select XZY, YZX, YXZ, ZXY,
and ZYX respectively. Code 0 selects XYZ. Every other code also selects XYZ,
including negative or unknown values and spherical code 6. Spherical rotation
remains unsupported. This inherited fallback produces an effective composition
order; it neither validates nor retains the original wire code.

The character loader composes local rotation in that effective order and
pre-rotation in XYZ, then bakes `pre_rotation * local_rotation` into the joint.
The quaternion arithmetic, floating-point operation order, degree conversion,
and joint traversal remain the same. Focused tests cover all six orders, native
code fallbacks, exact floating-point observations, and a public character load
from an authored binary FBX mesh and skin fixture.
