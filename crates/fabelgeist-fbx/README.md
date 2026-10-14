# Binary FBX property tags

The binary FBX model-file reader admits each one-byte property tag into a
bespoke FbxPropertyTag before dispatch. The same owner names all thirteen
supported tag codes and distinguishes scalar numbers and Booleans, numeric and
Boolean arrays, strings, and raw byte payloads. Tag identity remains typed
through both property selection and array selection.

Tag admission is open. Unknown native bytes reach the existing dispatch
rejection and retain the same character diagnostic. The type does not admit
decoded payloads or change scalar widths, array decoding, Boolean truth rules,
string encoding, read ordering or parser errors. Version/header layouts, array
encoding, counts, offsets and structured errors remain separate migration
responsibilities.

A regression fixture records 284 observations from the unchanged main public
parser. Independently encoded properties cover all 256 native tag bytes, all
thirteen known payload representations, missing and short payloads, and mixed
forward/reverse property sequences. Comparisons retain exact floating-point bit
patterns, invalid string bytes and the distinct string/raw property variants.
