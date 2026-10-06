# Binary FBX header layout

The binary FBX model-file reader admits the file header's four-byte version into
a bespoke FbxVersion before selecting node-header layout. Versions below 7500
use three 32-bit words and a one-byte name length; versions at or above 7500 use
three 64-bit words and that same name length. HeaderFormat owns the
corresponding 13-byte and 25-byte record widths, including the null record that
terminates a node list.

The version is an open wire identity. Layout selection preserves the existing
cutoff for every four-byte value, including values outside the documented
7100–7700 range; it does not establish support for other features of those
versions. The public parser, decoded nodes, truncation diagnostics and format
rejection remain unchanged. Counts, offsets, array encoding and wider decode
errors remain separate migration responsibilities.

A regression fixture records the unchanged main parser's results for eight
versions around and beyond the cutoff, empty and terminated lists, scalar
properties, nested and sibling nodes, truncation and invalid input. These 59
observations exercise the public parser against independently encoded wire
records.
