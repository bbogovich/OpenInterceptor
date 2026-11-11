# Stage 8 — Platforms and release

**Version 1.0.0. Status: planned.**

- Ships three campaigns: `default` (*Perils of the Void*, the no-CD experience), `interceptor`
  and `interceptor-remastered` (both need the player's CD).
- macOS `.app` (engine + original-free packs); optional local-only variant with the player's
  data in `dist/`, never published.
- Linux tarball and `.desktop`; AppImage considered.
- Windows build and XInput gamepads.
- Release process and versioning. Multiplayer: deferred, possibly never.

## Game bundling (MD7)

From the [mod designer, cutscene designer and game bundling](../../specification/design/designer.md) design (decisions D1–D5
made 2026-11-11).

| # | Item | Module | Requirements |
|---|---|---|---|
| MD7 | **Product bundling**: the build's two artifacts, the game and the designer (D5); `product.yaml`; branded product builds from the game artifact (`tools/build_bundle.py --product`, or the designer's "Package game") with exactly the listed packs, starting in the product's campaign; release mode without the testbed, viewer and developer tools; players' own mods still load on top; the provenance guard (no `requires_original` pack, no converted file); the full validator first | `tools/build_bundle.py`, `designer` | [R-PLT-007 – R-PLT-010](../../specification/requirements/platforms.md#r-plt-007-product-file), [R-MOD-029](../../specification/requirements/modding.md#r-mod-029-provenance-of-imported-assets) |
