# Strategic interface construction

This guide defines the construction language for the in-world strategic
interface. Its first reference flow is Goslar: settlement map, public square,
market, inn, church, armoury and residences. The setting is 1544 within the
Göttingen and Harz region. Character selection and tactical combat have their
own presentation requirements.

The interface interprets local construction for navigation and readable
controls. It does not reconstruct the exact appearance or contents of a
particular building in 1544. Historical evidence and design choices are
distinguished below.

## Shared construction

Frames should explain how a surface is supported. Posts carry lintels; rails
divide fitted panels; brackets support projections; masonry openings have
visible depth. Use a consistent direction for highlights and cast shadows.
Recesses, projecting ledges and attached signs should remain distinguishable.

Timber, plaster, stone and iron provide the shared material vocabulary. Vary
the width, spacing and depth of structural members before changing ornament.
Keep decorative carving on the members that could carry it: brackets, rails,
spandrels, sign surrounds and panel edges. Continuous patterns must terminate
at joints or fit within a bounded panel.

The repeated top frieze keeps one height and pattern scale across places at
each breakpoint. Different materials and motifs should not make the header
or adjoining reading surfaces jump. Preserve the place navigation's horizontal
position within a settlement, including history and live page replacement.

Richness belongs to a place's function and means. A merchant's carved lintel,
an inn's fitted timber panels and an armoury's worked iron can all be elaborate
without sharing a church window. Repeated architectural motifs must still
leave a plain, uninterrupted surface for reading and editing.

A Place Facade depicts the place. Its separate semantic icon identifies the
service. Keep both independent of the active state, available actions and
time-of-day lighting. A facade is navigation artwork, not a photograph or a
promise that the rendered scene reproduces a surviving building.

## Treatments by place

- **Map:** a fitted timber frame and inset map surface, with controls seated
  on its rails. Keep terrain, routes, labels and spatial interactions clear.
- **Public square:** open civic framing, substantial stone edges and timber
  notice or service boards. Projecting ledges and repeated bays connect it to
  the surrounding buildings without enclosing the square in a church portal.
- **Market and merchant hall:** repeated bays, moulded lintels, carved
  brackets and bounded weave or foliate friezes. Trading controls belong to
  counter-like surfaces and fitted panels. This is the richest commercial
  treatment in the initial flow.
- **Inn:** close timber framing, deeper panel recesses, a substantial sign
  surround and carved rails. Warm material tones and enclosed proportions
  suggest shared domestic rooms and a sheltered courtyard.
- **Church:** stone reveals, taller openings and bounded tracery, with painted
  timber panels where appropriate. Reserve the strongest pointed and foiled
  forms for this treatment; keep text clear of them.
- **Armoury:** heavy framing, iron straps, fasteners and recessed storage-like
  panels. Put ornamental effort into the ironwork and joinery. Preserve the
  forge renderer's opening and interaction area.
- **Residences:** smaller domestic bays, fitted panels and carved timber
  edges. Wealth may increase the depth of mouldings and the amount of relief
  carving. A residence does not automatically inherit an inn's service sign
  or a church's tracery.

These are design treatments grounded in the references below. Exact paint
colours, decorative patterns, furnishings and control metaphors remain
authored interpretations.

## Composition, readability and interaction

Keep architectural family, component skin, material, service, interaction state
and lighting as separate presentation inputs. The family provides regional
construction rules; the skin fits those rules to a component; the material
provides its surface and edge treatment. Service identity selects meaningful
signs and place treatments. Selected, focused, disabled and pressed states must
not require separate copies of the architectural artwork.

Text and controls sit on dark, opaque local surfaces, including when the
surrounding scene is bright. Material texture and lighting must not reduce
their contrast. Use the established body and heading typography for reading,
labels and numbers. Restrict blackletter to existing display roles; it is not
a substitute for readable small text. The fonts are interface choices, not
claims about lettering on a particular sixteenth-century building.

Actions retain Tactile Buttons and Instant Tooltips, visible labels, accessible
names, keyboard focus and non-colour state cues. Ornament cannot intercept
pointer events or add meaningless stops to keyboard navigation. On narrow
screens, reduce decorative margins and structural thickness before reducing
text size or hiding a necessary control. Keep navigation and scrolling usable
at narrow widths and browser zoom.

Changing presentation belongs on `#strategic-page` or its descendants because
soft navigation replaces that region. Live refresh also replaces sidebars,
and dialogue updates replace the contents of their description and NPC
regions. Structural surrounds must survive those operations without copying
stale state. Preserve existing route targets, IDs, form actions and focus
restoration identities.

The fullscreen Bevy canvas remains outside the replaceable page region. Keep
one canvas, Wasm instance, WebGPU device and asset cache for the lifetime of
the strategic document. Scene changes and HTML navigation must not restart
them. Bevy owns continuous 3D presentation and spatial interaction; HTML owns
document panels, forms, dialogue and accessibility-critical controls. Both
continue through existing authoritative strategic state and typed commands.

The ornamental family is enabled only for Goslar (`viabundus-2337`). Other
settlements retain their existing frames pending a local review. Regional
settlement headers use inland scenery instead of assigning coastal scenery
from an identifier hash. Narrow layouts stack regions in source and keyboard
order, with bounded scrolling for long merchant stock lists.

Review the complete Goslar flow at desktop and narrow widths before extending
the treatment to more settlements. Check text contrast, keyboard operation,
long labels, clipping, navigation, live updates and persistent canvas identity
alongside the distinct appearance of each place. Store screenshots and review
transcripts in ignored output directories, not beside this design contract.

## Historical references and their limits

- [Goslar's World Heritage masterplan][goslar-plan], pages 13 and 22, describes
  low timber miners' houses around 1500, patrician houses combining a hall
  with a stone heated chamber, and the Alte Ausspann's four-sided courtyard
  with buildings partly from around 1500. It supports differences of scale,
  material and enclosure; it does not establish the UI's furniture details.
- [The Brusttuch heritage record][brusttuch] dates the house to 1521–1526 and
  describes masonry lower storeys, a projecting timber upper storey,
  keel-arched window heads and relief carving on brackets, posts and
  spandrels. The [heritage authority's publication][brusttuch-study] connects
  this unusually rich decoration to its owner's wealth and education. It is
  a local precedent for elaborate secular construction, not a typical inn.
- [Goslar's council chamber][council], furnished between 1505 and 1520, has
  painted panels on walls, ceiling and window recesses. It supports rich
  fitted civic interiors without making every commercial room ceremonial.
- [The Kaiserworth guildhall][kaiserworth] dates to the late fifteenth
  century. Its arcades and surviving early figures support a prosperous
  merchant treatment. The present emperor statues date to 1684 and do not
  belong in a 1544 reconstruction.
- [Göttingen's St. Jacobi][jacobi] provides local medieval church fabric and
  a folding altar completed in 1402. Its present choir glazing dates to
  1900–1901. Use the older structural evidence, not the whole modern interior.
- [The Goslar Zwinger][zwinger], built in 1517, provides evidence for massive
  local sandstone defensive construction. It does not document an armoury's
  racks, furniture or internal arrangement.

## Artwork and reuse

### Pigments and expense

Painted decoration uses a hierarchy of materials, not an equally available
rainbow. Earth red, ochre, lime white and carbon black supply the ordinary
colour vocabulary. The merchant hall uses azurite blue; the inn uses a
verdigris-inspired green. Small vermilion details punctuate the vine frieze.
The sanctuary's tiny blue lozenges and yellow outlines refer to lapis lazuli
and orpiment. Broad sanctuary paint remains earth red and azurite. Yellow
painted edges elsewhere represent ochre or lead-tin yellow, not ubiquitous
gold leaf. Timber and iron keep their separate material colours.

These allocations and RGB values are design interpretations, not measured
1544 colours or estimates of household purchasing power. Pigment, binding
medium, underpainting and particle size all affect appearance. The frieze
vectors have transparent grounds so CSS can fit the same carving to each
place's paint without changing its geometry, padding or pattern scale.

The [Domvorhalle conservation report][dom-pigments], p. 152, identifies
cinnabar, lapis lazuli, orpiment and gilding in Goslar's medieval first paint
layer. That establishes earlier local use, not prevalence or an unchanged
appearance in 1544. The [Brusttuch restoration report][brusttuch-paint],
pp. 240–241, says stripping erased its original facade colour evidence;
the restoration drew on a nineteenth-century scheme.

[Heydenreich's Cranach study][cranach-materials], table 5 and pp. 156–157,
175, provides contemporary German workshop evidence. Accounts price yellow
ochre at about 0.66–0.76 groschen per pound, lead-tin yellow at about
1.75–3, and vermilion at about 9–10.5. These are workshop purchases across
different years and grades, not a Goslar retail price list. Azurite varied
in grade; coarse material could produce strongly coloured matte backgrounds.
Ultramarine appears only in a restricted period of the surveyed Cranach work.
Its exceptional status should remain visible through limited application.

[The Annunciation analysis][cranach-annunciation] identifies the earth colours,
azurite, lead white, lead-tin yellow and vermilion used as references here.
[Holy Trinity analysis][cranach-green] identifies verdigris in green paint.
[Denninger's Hildesheim ceiling study][hildesheim-pigments] supplies older
regional evidence for charcoal, lime white, ochre and copper green.

### Authored details

The [Harz artwork attribution](static/styles/harz/ATTRIBUTION.md) records the
original geometric facades, friezes, tracery and ironwork used by this
construction language. CSS supplies adaptive surfaces, joints and depth.
Existing Game Icons retain their own attribution and remain semantic signs.

Freely available architectural SVGs were searched before authoring these
details. Public-domain geometric [trefoil][trefoil] and
[quatrefoil][quatrefoil] files are available, but neither reproduces a local
building. Their existence does not make them appropriate across every place.
Original geometry lets the mouldings, joints and patterns fit the actual
components without importing generic decoration. None of those external SVGs
is incorporated into the Harz artwork.

The older timber-framed PNGs have mixed provenance documentation. Their
repository history alone does not establish an external licence, a historical
source or a generation method. Preserve the narrower claims recorded in the
Harz attribution rather than extending the provenance of the documented
project-native masks to all older assets.

[goslar-plan]: https://www.goslar.de/fileadmin/media-goslar/stadt/welterbe/masterplan_altstadt_goslar.pdf
[brusttuch]: https://denkmalatlas.niedersachsen.de/viewer/piresolver?id=36527867
[brusttuch-study]: https://denkmalpflege.niedersachsen.de/themen/welterbe/renaissance-in-holz-das-brusttuch-in-goslar-142312.html
[council]: https://www.meingoslar.de/erleben-und-geniessen/museen/huldigungssaal
[kaiserworth]: https://zinnfigurenmuseum.goslar.de/marktplatz
[jacobi]: https://www.kirche-goettingen-mitte.de/unsere-kirchen/st-jacobi/geschichte-und-bau-st-jacobi
[zwinger]: https://zwinger.de/geschichte-des-zwingers-zu-goslar/
[trefoil]: https://commons.wikimedia.org/wiki/File:Trefoil-Architectural.svg
[quatrefoil]: https://commons.wikimedia.org/wiki/File:Quatrefoil-Architectural-Square.svg
[dom-pigments]: https://www.icomos.de/data/pdf/xix-0421-0922-01.pdf#page=154
[brusttuch-paint]: https://niemeyer-verlag.de/wp-content/uploads/Denkmalpflege_2011_4.pdf
[cranach-materials]: https://lucascranach.org/application/files/3016/7885/6148/Heydenreich_2007_Lucas_Cranach_the_Elder.pdf
[cranach-annunciation]: https://lucascranach.org/en/PRIVATE_NONE-P509/
[cranach-green]: https://lucascranach.org/en/DE_SKD_SKS_SAV2200/
[hildesheim-pigments]: https://www.tandfonline.com/doi/abs/10.1179/sic.1969.009
