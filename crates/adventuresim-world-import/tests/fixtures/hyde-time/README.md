# Synthetic HYDE time fixtures

`fixtures.zip` contains generated NetCDF-4 inputs for the public importer
integration test. They use HYDE's grid dimensions but contain only one
populated cell and no real HYDE data. The test overlays each scenario's files
on the `negative` baseline, and supplies the common synthetic area grid.
The generation tools are not needed when running the test.

To regenerate, install Python `netCDF4` and NumPy and run the following from
this directory, choosing output paths that do not yet exist:

```sh
python generate.py --writer-site . --output /tmp/hyde-base
python generate_extra.py --writer-site . --base /tmp/hyde-base --output /tmp/hyde-extra
python generate_boundary.py --writer-site . --base /tmp/hyde-base --output /tmp/hyde-boundary
python generate_later.py --writer-site . --base /tmp/hyde-base --output /tmp/hyde-later
python pack.py --base /tmp/hyde-base --extra /tmp/hyde-extra --boundary /tmp/hyde-boundary --later /tmp/hyde-later --output /tmp/hyde-fixtures.zip
```

The checked-in fixture was generated with netCDF4 1.7.5.1.post1, netCDF-C
4.10.1, HDF5 2.2.0 and NumPy 2.3.5. Different library versions may produce
different binary files even when their decoded contents are equivalent.
Review the generated inputs and test results before replacing the archive.
