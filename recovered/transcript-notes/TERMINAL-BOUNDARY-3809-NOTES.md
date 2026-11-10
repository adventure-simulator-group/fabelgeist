# Terminal medication inventory acquisition follow-up

Selected revision `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.
Prospective registrations: `REGISTRATION-TERMINAL-INVENTORY-L13-3809.json`
and `REGISTRATION-TERMINAL-INVENTORY-TAIL-L13-3809.json`. Catalog and
administration evidence are preserved at checkpoints 3808 and 3805.

- `change_inventory_item` is a reducer whose inspected entry checks for a
  living character. For a non-durable, non-measured item with no matching row
  and a positive quantity, it calls `add_inventory_item`. The wrapper
  suppresses checked-helper errors, so a successful reducer return alone does
  not prove a row was created.
- If the persisted item definition exists with medication kind, the checked
  helper treats it as individual and inserts one quantity-one row per unit,
  together with a stable personal object. Thus adding one strong poppy item
  on the empty-row branch is a source-derived candidate for satisfying the
  quantity-one requirement of standalone administration. It depends on
  successful initialization and commit, a usable persisted definition,
  reducer exposure and authorization; none was tested in a native runtime.
- If a matching non-measured row already exists, the reducer instead first
  attempts to increment its quantity. That branch does not establish the
  quantity-one administration precondition. Keep it distinct from the selected
  empty-row scenario and the ordinary brewed-component workflow.
- The previously tracked inventory contract #947 remains independent; this
  trace is not an additional unique finding or a claim of access or repair.
