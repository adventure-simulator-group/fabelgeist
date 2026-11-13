# Guard replay command correction

The earlier `GUARD-REPLAY-ADDITION.md` omits the interpreter and installed
relative directory from its inventory-generator command. From the root of an
installed evidence checkout, use:

```sh
python3 -B delegated/post8b-followups/OBJECT-ALTERNATE-INVENTORY.py \
  --objects <absolute-object-directory> --output <new-private-absolute-file>
```

The guard compares the supplied alternate to the supplied inventory before its
child checks; concurrent replacement after that comparison is not excluded.
Git can follow on-disk alternates from any participating object store,
including the explicitly supplied alternate. Hashing an `info/alternates`
pointer as a file does not inventory the pointed-to store. The particular
minimal alternate recorded in this audit had an empty `info` directory when
inventoried, while the predecessor checkouts still rely on separately held
shared stores. The original replay note remains preserved.
