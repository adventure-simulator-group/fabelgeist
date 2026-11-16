# Landmass source

This directory contains the published Landmass 0.9.3 crate. The local
`Cargo.toml` changes its `geo` dependency from 0.31 to 0.33.1 so that Cargo can
resolve it with the workspace's existing geometry dependencies. Remove this
patch when a compatible upstream release is available and validated.

Source: <https://github.com/andriyDev/landmass>
