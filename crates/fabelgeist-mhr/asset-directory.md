# Native asset-directory selection

`Mhr::from_files` accepts the directory containing `compact_v6_1.model`, or its
parent with that file under `assets/`. The direct definition wins when both
locations are files. Selection keeps the supplied native path spelling and
performs no canonicalization. Invalid level-of-detail configuration is rejected
before this lookup.

If neither `Path::is_file` test succeeds, loading returns the bespoke
`MhrAssetDirectoryError::Missing` through its existing outer `anyhow` result.
The error retains the original native PathBuf as diagnostic provenance.
Its Display text is unchanged, and it has no source cause. Callers can downcast
the outer error to inspect that classification; Debug exposes the structured
class and native path.

```rust,no_run
use fabelgeist_mhr::{Mhr, MhrAssetDirectoryError, MhrConfig};

let device = Default::default();
if let Err(error) = Mhr::from_files("body-assets", MhrConfig::default(), &device) {
    if let Some(MhrAssetDirectoryError::Missing { native_directory }) =
        error.downcast_ref::<MhrAssetDirectoryError>()
    {
        eprintln!("No model definition found under {}", native_directory.display());
    }
}
```

Native Path/PathBuf, joins, file predicates and selected paths are standard
filesystem representation ports used immediately by the existing loader.
They provide no checked directory, resource locator or storage identity.
Rejected paths keep their OS representation, including non-UTF-8 spelling.
The diagnostic formats that path through the standard library's display port.

`is_file` treats metadata errors as false. A path that cannot be inspected can
therefore produce the same missing-assets rejection; no inspection-error cause
or stricter path admission is added. Definition symlinks follow the standard
file predicate. Downstream file reads retain their original I/O sources and
loading/parsing contexts. URI and byte loading use their existing separate
paths.

This contract covers native directory selection only. Full asset reading,
format decoding, checked layouts and model evaluation retain their own owners.
There is no claim of successful tensor upload, rendering or real-asset loading
from directory-selection behavior alone.
