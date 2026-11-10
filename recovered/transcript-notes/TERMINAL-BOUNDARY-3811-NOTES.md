# Strong tincture projection follow-up

Selected revision `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.
Registrations: `REGISTRATION-TERMINAL-PROJECTION-L13-3811.json`,
`REGISTRATION-TERMINAL-DEFAULTS-L13-3811.json`,
`REGISTRATION-TERMINAL-FOOD-LOOKUP-L13-3811.json`, and the prior authored
catalog registration `REGISTRATION-TERMINAL-POPPIES-L13-3808.json`.

The selected authored strong tincture entry has kind medication and omits
capabilities. `ItemDefinition` defaults omitted capabilities; the projection
starts from `Item::default()`, assigns medication kind, and sets alcohol
serving and repairability only if the relevant capabilities exist. Therefore
a freshly projected definition has medication kind, zero alcohol serving and
no repairability. The selected food lookup is catalog-based; whether this ID
has an independent food definition remains separately to be qualified. Soap
and smithing identity checks likewise require exact comparison.

This source projection does not establish that an existing database has that
row, that initialization was run on it, that item creation committed, or that
the generic reducer is available to a particular client. No native execution
or contract closure is claimed.
