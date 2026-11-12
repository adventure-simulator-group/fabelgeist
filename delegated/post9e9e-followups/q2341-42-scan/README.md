# q2341/42 evidence replay and privacy scan receipt

`PACKET-ANCHOR.json` binds the source packet. The standalone replay explicitly
reports original-byte, source-store, transport and private-path checks that it
cannot repeat without inputs. The full replay used the product-source Git
object store, original transport metadata and separately held private key; it
executed only evidence tooling. The final regex and restricted-corpus scans are
bounded and do not cover unknown encodings, secrets or external Git history.
The same-machine key and private corpus are recovery dependencies, not
independent public custody. The receipt itself is scanned and its removed-path
digest exposure checked separately; that later check is not recursively
included in this receipt.
