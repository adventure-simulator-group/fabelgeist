# Browser tactical case sites

The browser launcher uses a settlement case site unless a wilderness site is
requested explicitly. Choose the site for the scene being tested:

```text
python scripts/dev_stack.py tactical-play browser 24930 \
  --scene-input path/to/forest-scene.json --case-site wilderness
```

`--case-site settlement` requests the town context. This option belongs to
browser mode; other modes reject it before starting processes. Existing mode
defaults still apply when the option is omitted.

A wilderness case site is two kilometres from its synthetic host settlement.
It does not request the town's population or business snapshot, and it keeps
the ordinary player loadout. The scene input supplies the actual rendered
terrain and geography; the synthetic host offset does not relocate that input.
The session metadata records the effective case-site kind. Mission claims,
transient server authority and isolated database ownership use the same
supervised workflow for both site kinds.
