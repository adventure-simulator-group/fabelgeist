# Terminal medication availability follow-up

Selected revision `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.
Prospective registrations: `REGISTRATION-TERMINAL-POPPIES-L13-3808.json`
and `REGISTRATION-TERMINAL-HERBAL-L13-3808.json`.

- The authored item catalog contains `strong_poppy_tincture` as a medication
  item with strong potency tag. Catalog presence is not proof of player
  acquisition, an individual inventory row, or authority to administer it.
- The inspected poppy tincture preparation process instead pins the ordinary
  `poppy_tincture` profile, with pinned potency clamped between 0.1 and 1.0.
  Its administration reducer accepts 1 to 1,000 milliunits of the tincture,
  scales that fraction by component potency, and calls the oral intervention
  component path. This is not evidence that this process creates the strong
  standalone medication item or supplies its 8,000-milliunit dose.
- Additional item production, inventory insertion and ordinary access paths
  require qualification. The conditional strong-profile numerical example
  remains conditional; the ordinary tincture path has distinct strength,
  dose and carrier semantics. No runtime or closure claim is made.
