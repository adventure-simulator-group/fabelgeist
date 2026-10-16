# Parametric dress textile references

These recipes exercise gathered skirts, continuous upper sections, colored
border panels and repeating foliage. They are stylized studies. Their flat
ornament and regular folds do not establish faithful historical recreation.
Cloth physics, detailed bodices and separate accessories remain outside scope.

`pisa.json` uses the crimson velvet gown displayed at the Museo nazionale di
Palazzo Reale in Pisa, dated around 1560. The museum attributes it to Mastro
Agostino for Eleonora di Toledo or a lady of her court; ownership is uncertain.
It is an Italian comparison for the game's sixteenth-century German setting.
The photograph supports its long gathered skirt and border placement, but
does not resolve the exact ornamental motif.

- [Museum description](https://cultura.gov.it/luogo/museo-nazionale-di-palazzo-reale)
- [University of Pisa exhibition photographs](https://www.cfs.unipi.it/2022/10/18/inaugurata-al-museo-di-palazzo-reale-di-pisa-la-mostra-eleonora-di-toledo-e-cosimo-de-medici-coppia-nobilissima-et-bella/)

`cranach.json` uses Anne von Wiese's 2024 Cranach-style gown. This is a modern
reconstruction or impression of early-sixteenth-century Saxon court dress,
using historical and modern methods, not a surviving original. Its photograph
provides a clear target for a burgundy gathered skirt, a large botanical field,
a dark divider, a smaller patterned band and a broad dark hem.

- [Maker's photographs and construction account](https://annevonwiese.com/16th-c-cranach-style-gown-of-2024/)

Both files contain `SkirtDesign` recipes. Given a saved native review body,
export one through the same fitter used by the creator and runtime:

```sh
cargo run --manifest-path crates/adventuresim-character-creator/Cargo.toml \
  --no-default-features --example skirt_review -- BODY.json OUTPUT \
  --design crates/fabelgeist-armor/review/textiles/cranach.json
```

The export includes the generated mesh, native fabric coordinates and baked
parametric color maps. Appearance layers are independent of puff geometry.
Change layer intervals, colors and repeats to reuse a border at a sleeve cuff
or on another garment with the same proximal-to-distal chart convention.
No reference photograph is distributed with these recipes.
