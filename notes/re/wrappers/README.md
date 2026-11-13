# Reverse-engineering notes: the original's strategy screens

Facts only — ids, coordinates, addresses and string *keys* read from the player's own
`Interceptor.exe` and `WrappersDLL.dll` (2026-11-12). No original art, text or code is
here. Each fact is marked verified (read from the code or tables) or derived.

- `catalogue.md` — every WrappersDLL image id, grouped by screen and role.
- `screens.yaml` — each screen and dialog: background, controls, rectangles (640×480),
  state image ids (`spr:N` = the exe's sprite table), actions, callbacks.
- `navigation.md` — which screen opens which; menu commands; time controls.
- `re-notes.md` — how the exe builds a screen (factory, widget constructors, sprite
  table, callbacks), with addresses for re-reading at load time.
- `tools/` — the Python (stdlib only) decoders used. They hard-code the old machine's
  scratch paths: edit them before reuse, and run them on assets extracted to a temporary
  folder outside the repository (`openinterceptor --extract <dir>`). Never commit their
  outputs (disassembly listings, images).

Used by `packs/interceptor/rules/screens.yaml` (Stage 6.9, issues #98–#112).
