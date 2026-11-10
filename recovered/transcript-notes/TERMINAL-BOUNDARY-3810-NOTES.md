# Medication initialization follow-up

Selected revision `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`;
`REGISTRATION-TERMINAL-INIT-L13-3810.json` records the inspected sources.

- For an existing persisted medication definition, weapon initialization
  returns successfully before weapon-design logic because the definition is
  not a weapon. A missing definition still errors, and the outer generic
  addition wrapper suppresses that error.
- Stable personal object insertion requires quantity one, a carried location
  with a backing row, and no existing physical object for that new row.
  It inserts the object after these checks. Unique-ID and database behavior
  are not established by source inspection.
- Measured-item detection includes food, positive alcohol serving size, soap
  and smithing materials. The actual projected strong-poppy item fields and
  database state must be checked before deciding whether the generic reducer
  takes its measured or non-measured branch. Either branch must still be
  qualified end-to-end; no committed item or public access is claimed.
- These observations refine the existing conditional #947 inventory trace
  and terminal example; neither contract is declared closed.
