# Stage 7 — Modding toolkit and Interceptor Remastered

**Version 0.8.0. Status: planned.** The tools that make mods easy, and the proof that a mod can
extend a campaign.

Design: [mod structure](../../specification/design/mod-structure.md), [Interceptor Remastered](../../definition/campaigns/remastered.md).

## Work items

| # | Item | Module |
|---|---|---|
| 7.1 | `--check --campaign <id>` grown into the full validator: every definition, reference, condition and text key; JSON report; per-addon checks | `check` |
| 7.2 | **Recipes**: the generated-asset system (cache keyed by recipe, version and input hash; `--build-recipes`; build on first use with progress) | `recipe` (new) |
| 7.3 | `upscale` generator: de-dither, edge-directed ×2 steps, sharpen, alpha re-application, wrap-aware borders | `recipe::upscale` |
| 7.4 | `mesh-patch` generator: inset, extrude, bevel, cut along UV lines, subdivide, relief from texture, LOD copy; input mesh check | `recipe::meshpatch` |
| 7.5 | `recolour` and `composite` generators | `recipe` |
| 7.6 | Viewer: patch hot-reload and before/after comparison; model-set state browser | `app::viewer` |
| 7.7 | **`interceptor-remastered`**: `parent = interceptor`; ×4 textures and ×2 images; the enhanced X-1A Lightning II mesh patch | `packs/interceptor-remastered` |
| 7.8 | `default`: balance pass, complete encyclopaedia | `resources/default` |
| 7.9 | Modding guide finished: tutorials from "change one number" through "an addon" and "a campaign that extends another" to "a new campaign", built on the default campaign | docs |

## Mod and cutscene designer (MD4-MD6)

From the [mod designer, cutscene designer and game bundling](../../specification/design/designer.md) design (decisions D1–D5
made 2026-11-11). The designer is a **separate program in the same repository** (D5): a second
binary, `openinterceptor-designer`, sharing the engine library, built with macroquad's UI module
and the engine renderer (D1), so the build produces two artifacts, the game and the designer.

| # | Item | Module | Requirements |
|---|---|---|---|
| MD4 | **Designer application shell**: the second binary and build artifact; pack wizard with base selection and folder skeleton; pack tree, schema-driven forms and the YAML view; undo; saving that keeps hand-written files; validation on save; play in the testbed; export as folder or zip | `designer` (new binary) | [R-MOD-021 – R-MOD-024](../../specification/requirements/modding.md#r-mod-021-designer-application), [R-MOD-028](../../specification/requirements/modding.md#r-mod-028-validate-on-save-play-in-the-testbed), [R-MOD-030](../../specification/requirements/modding.md#r-mod-030-package-a-pack) |
| MD5 | **Asset import** with dependencies (including conversion from original formats by the extractor), external editor launch, live reload, "where used", provenance recording in `pack.yaml` | `designer`, `extract` | [R-MOD-025 – R-MOD-027](../../specification/requirements/modding.md#r-mod-025-import-an-asset-with-its-dependencies), [R-MOD-029](../../specification/requirements/modding.md#r-mod-029-provenance-of-imported-assets) |
| MD6 | **Cutscene Designer**: outline, 3D map, preview, timeline with keyframes, cameras and cuts, sounds, captions, scrubbing with snapshots every 2 s | `designer` | [R-MOD-031](../../specification/requirements/modding.md#r-mod-031-cutscene-designer-workspace), [R-MOD-032](../../specification/requirements/modding.md#r-mod-032-seeking-with-snapshots) |

**Exit**: the remastered campaign plays the whole Interceptor campaign with its generated assets;
recipe outputs bit-identical across runs and machines; validator catches every seeded error in a
test mod.
