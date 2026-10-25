# Settlement buildings in Central Germany, 1544

This implementation note distinguishes a plausible regional inventory from a
claim that any particular settlement contained every listed institution. It is
not a census. Population catchments in the code are gameplay tuning, not sourced
historical capacities. The catalogue is authoritative for current thresholds.

## Demand and identity

Use parish churches for the repeated 100–500-person catchment. A cathedral is an
institutional designation, not the next size of parish church. Monasteries,
colleges, castles, mints and religious communities require dated local evidence.
Reformation-era occupation and reuse matter: a former religious building need
not still have its original occupants or function in 1544.

Derive specialist premises from the existing economy. An available weaponsmith
needs a weaponsmith's workshop; selling basic weapons through a village smith
does not establish a separate specialist. Housing capacity and service capacity
remain distinct. Count repeatable premises until their combined service
catchment covers population, but keep singular civic institutions singular.
Neighbourhood services need distributed sites; no sequence of capacity rolls
should be presented as a measured historical population.

## Regional inventory

- Domestic and agricultural: cottages, urban workshop-dwellings, merchant
  houses,
  hall houses, manor houses, lodging houses, barns, threshing floors, stables,
  cart sheds, byres, pigsties, hay stores and granaries. Small household
  outbuildings can share a plot rather than becoming separate institutions.
- Food and drink: bakehouses, brewhouses, malt houses, butchers, communal ovens,
  grain mills, fishmongers, inns and wine taverns. Markets also need stalls and
  open space, not only monumental market halls.
- General crafts: smiths, carpenters, joiners, wheelwrights, coopers, cobblers,
  saddlers, harness makers, rope makers, chandlers, potters, stonecutters,
  glaziers, locksmiths, cutlers and goldsmiths. Many share the workshop-house
  structural family; business identity need not imply a wholly unique shell.
- Textiles and hides: weavers, tailors, cloth finishers, fulling mills, dye
  houses,
  tanneries, furriers and cloth stores. Put water-dependent and unpleasant
  processes on verified water sites and keep drying/work yards legible.
- Trade and administration: warehouses, grain stores, weigh houses, customs and
  toll houses, guildhalls, town halls, council chambers, counting rooms,
  municipal
  prisons and watch houses. Road gates and bridges are infrastructure with their
  own geometry, not arbitrary shop buildings.
- Religion, care and learning: parish churches, chapels, rectories, collegiate
  churches and cathedrals, monasteries and convents, hospitals and almshouses,
  bathhouses, schools, university colleges, apothecaries, printers, bookbinders
  and booksellers. Synagogues and ritual baths need dated community evidence.
- Regional production: water-powered grain, fulling, paper and saw mills; post
  windmills; horse mills; brick and tile kilns; lime kilns; glasshouses; ore
  smelters; assay houses; salt-boiling houses; charcoal sheds and mining service
  buildings. Water, wind, fuel and mineral availability are site constraints.
- Power and defense: castles, keeps, fortified manor houses, arsenals, powder
  stores, gatehouses, wall towers and executioner/knacker dwellings. Not every
  town needs every one; fortification layout and institutional authority matter.

The code gives first-class identities to 65 service/institution uses plus
ordinary dwellings. Closely related household outbuildings and crafts above can
use the same room programme rather than multiplying nearly identical enums.

## Reference anchors

The following sources support the regional typology, not numerical catchments.

- [Erfurt's architectural inventory](https://www.erfurt-tourismus.de/en/all-about-erfurt/places-of-interest/)
  records churches, monastic ranges, merchant houses and woad storage. These
  support a mixed city silhouette and specialised storage buildings.
- [Erfurt's historic religious skyline](https://www.germany.travel/en/cities-culture/erfurt.html)
  distinguishes numerous parish churches, chapels and monastic institutions.
- [Cranach Foundation chronology](https://www.cranach-stiftung.de/de/lucas-cranach-der-aeltere.html)
  documents the Wittenberg workshop properties, pharmacy privilege and printing
  activity around the Reformation. This anchors mixed-use premises and the
  specialist book/pharmacy economy before 1544.
- [Cranach houses and the book trade](https://www.cranach-stiftung.de/en/cranach-houses-in-wittenberg.html)
  records the 1533 book-trade consortium and 1534 Luther Bible distribution.
- [Erfurt municipal weigh house](https://www.via-regia.org/via_regia/geschichte/einzelthemen/thueringen/erfurt1e.php)
  describes a building erected in 1354 and the use of adjacent properties for
  storage and the weighmaster's dwelling in the fifteenth century.
- [Hessenpark's milling history](https://www.hessenpark.de/lexikon/handwerk/vorfuehrungen/muellerei/)
  distinguishes water mills and post windmills. Museum buildings of later dates
  should not be copied wholesale into 1544.
- [Erfurt's account of expulsion](https://juedisches-leben.erfurt.de/jl/de/mittelalter/index.html)
  dates the forced departure of its Jewish community to 1453. A surviving
  medieval
  synagogue cannot justify generating an active 1544 community there.

## Further variation requiring separate authored evidence

Terrain-following streets, market rights, bridge/toll sites, historic wall
circuits, regional wall/roof materials, parcel subdivision and building age
would add further city identity. Courtyard annexes, gardens, animal pens, ovens,
signs, loading equipment and trade machinery would make purposes visible
outdoors. Water mills require a millrace and power-site model before automatic
placement; post windmills need a distinct elevated mechanical structure.
Historical institutions should be supplied by reviewed 1544 settlement evidence.
These constraints should be implemented directly, without legacy schema paths.
