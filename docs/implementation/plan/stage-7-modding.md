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

**Exit**: the remastered campaign plays the whole Interceptor campaign with its generated assets;
recipe outputs bit-identical across runs and machines; validator catches every seeded error in a
test mod.
