# fabelgeist-numpy-storage

NumPy `.npy` and `.npz` reading for Burn, in pure Rust — no NumPy, no C zlib.

```rust
use burn::tensor::Device;
use fabelgeist_numpy_storage::{Npz, NpzArrayName};

let archive = Npz::open("weights.npz")?;
for name in archive.keys() {
    println!("{name}: {:?}", archive.array(&name)?.shape);
}
let weights = archive.array(&NpzArrayName::new("layer0"))?.to_tensor::<2>(&Device::default())?;
```

- Archives are memory-mapped; members may be stored or deflated, and zip64 is
  handled (NumPy switches to it past 2 GB).
- `uncompressed_size` reports a member's decoded size without decoding it.
- Element types: `f4`, `f8`, `i4`, `i8`, `u1`, `b1`.
  Values are widened on
  access with `to_f32`, `to_i64` or `to_bool`, or uploaded straight to a device
  with `to_tensor` / `to_int_tensor`, so a `.npz` written as `float64` still
  loads into an f32 tensor.
- Fortran-ordered arrays are rejected rather than silently transposed.

Archive lookup uses `ArchiveMemberName` for exact parsed filenames and
`NpzArrayName` for array keys. `contains` returns `ArchiveMemberPresence`. Array
keys match a full member name or that name with one trailing `.npy` removed; the
first matching member in central-directory order wins. Query spelling is
unchanged, including case, paths, empty names and NUL. Parsed filenames retain
the reader's lossy UTF-8 decoding, so distinct invalid byte spellings can
collide. Names do not validate path safety or membership. Explicit native text
adapters are available for presentation and external string APIs.

Used by `fabelgeist-mhr` for Momentum Human Rig (MHR) pose-corrective tensors.
