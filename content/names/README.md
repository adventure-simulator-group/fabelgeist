# Personal-name catalog

This is the external name catalog for the German population around 1544. YAML
uses the JSON-compatible subset. The world-schema build merges, validates, and
embeds it; runtime code does not read these files.

## Files

- `given_names.yaml` and `surnames.yaml` define stable identities and forms.
- `repertoires/` contains eligible IDs and relative `frequency` scores.
- `provenance/sources.yaml` contains bibliographic and population metadata.
- `provenance/observations.yaml` contains recorded spellings and raw counts.
- `provenance/derivations.yaml` links observations to repertoire entries and
  documents each frequency calibration.

Game-facing records contain no provenance fields. Build validation requires
exactly one derivation for every eligible repertoire entry.

## Selection and frequencies

Generation selects by culture, broad religious naming tradition, and birth
period. MVP settlements use German and Western Christian. Lutheranism does not
receive a separate modifier. Region, class, and education are accepted
generation context but do not alter the production repertoire without
comparable evidence.

Repertoire frequencies are relative sampling scores, not probabilities or raw
counts. Numeric observations use a square-root-smoothed integer score;
attestations without a comparable count receive score one. Family scores are
independent of form scores, so adding a variant does not increase family
prevalence. A family-level score may be curated from its attested forms when
the source publishes no aggregate.

The primary Leipzig baseline is deliberately given most production mass: its
male family scores are multiplied by seven and its female scores by three.
The Kremer long tail and broader comparator additions retain their own scores,
so an isolated attestation widens the catalog without becoming commonplace.
This yields 86.3% primary mass for men and 88.5% for women in the German
repertoire. These multipliers are curation policy, not claims about the size
of the Leipzig samples; raw observations remain in provenance.

Kremer/Kietz's rural Leipzig tables provide the primary given-name baseline.
Fambach, the 1495 tax records, Nuremberg 1497, Hamburg, Leipzig council books,
Munich's 1563 summary, and local inscriptions are comparators or form evidence;
their unlike counts are not pooled. The small Nuremberg and 1495 additions are
deliberately assigned the minimum nonzero repertoire score: they widen the
German repertoire without pretending that tax entries are national weights.
Hornburg's citizen book is the principal open local surname corpus and
represents urban male admissions, not a census.

The 1495 Baden-Württemberg tax-roll surname index adds a broader German
comparator set. It reports that surnames were already broadly inherited, but
it is still a tax sample outside the MVP region. Repeated unqualified entries
were normalized into 33 additional commoner identities; explicitly marked
occupational, locative, prepended, feminine-only, and unresolved entries were
left out. Their source counts are retained in provenance; raw counts from the
two corpora are never pooled. In the production repertoire these comparator
identities receive the minimum score one, so even a repeated Franck or Gewder
entry widens the tail without turning the regional tax sample into national
surname probabilities.

Hornburg's counted surname identities are the production core: their existing
sqrt-smoothed scores are multiplied by fifteen. The two feminine-form
comparators (`Pfeiffer` and `Schneider`) and all 1495 additions remain at
minimum score. The resulting surname selection is 86.7% Hornburg core and
13.3% comparator tail; the multiplier changes production mass, not any raw
source count.

The Nuremberg 1497 surname index supplies a second, independently preserved
comparator slice. Forty-one patronymic and descriptive identities are included;
the source's occupational, locative, prepended, feminine-only, and
uncategorized lists remain out of the random repertoire. These entries also
receive score one, so the two tax corpora broaden the tail without being
silently pooled.

The everyday assumption that Johannes was more common than Johann is not
supported. Hans, Johann, and Johannes share one family, with Hans dominant in
the everyday register; documentary forms remain separate. No immediate
Lutheran naming break is modeled.

## Normalization boundaries

Recorded spellings and normalization decisions belong in provenance;
player-facing forms belong in the catalog. Shared forms may link multiple
families: `Henne` links Johannes and Heinrich but is not randomly generated
without a defensible within-family frequency. Feminine surname forms are
explicit entries; code never appends `-in`. Farm names, household membership,
occupational bynames, and noble dynasties are separate concepts. Scholarly
aliases are intentionally outside this MVP catalog.

## Regional audit queue

The print-only Göttingen tax lists, Göttingen citizen admissions, and Goslar
house book are recorded as unused leads in `provenance/sources.yaml`:

- [Göttingen tax lists](https://www.regionalgeschichte.de/detailview?no=0834)
- [Göttingen citizen admissions][göttingen-admissions]
- [Goslar house book](https://www.regionalgeschichte.de/detailview?no=1261)

[göttingen-admissions]:
  https://stadtarchiv.goettingen.de/portal/seiten/familienforschung-900001294-25480.html
