# Implementation status

Updated as each stage progresses. Stage pages hold the detail; this page is the summary.

| Stage | Status | Notes |
|---|---|---|
| [0 Foundation](implementation-plan/stage-0-foundation.md) | **Done** | Repository, SpecItOut manifest, gate, docs site; every asset type for stages 1–2 decoded and verified |
| [1 Flight simulator](implementation-plan/stage-1-flight.md) | **In progress** | See the checklist below |
| [2 Enemies and combat](implementation-plan/stage-2-combat.md) | Not started | |
| [3 Missions and scripting](implementation-plan/stage-3-missions.md) | Not started | Scripting language decision pending |
| [4 Strategy layer](implementation-plan/stage-4-strategy.md) | Not started | |
| [5 Presentation](implementation-plan/stage-5-presentation.md) | Not started | |
| [6 Modding toolkit](implementation-plan/stage-6-modding.md) | Not started | |
| [7 Platforms and release](implementation-plan/stage-7-release.md) | Not started | |

## Stage 0 results

- All **297** XMSH meshes in `xobjects1.dll` and `xobjects2.dll` parse, with 4 LODs each.
- The three player hulls and an alien saucer were rendered with their original textures by an
  independent prototype decoder, confirming the mesh, UV and texture layouts.
- XTXR textures (all sizes), cockpit TGAs (colour key), ADPCM sound effects and PCM music loops
  are understood. Balance data is plain INI.
- Unknowns that do not block stages 1–2: XSPT RLE details, a few mesh fields, the MFD animation
  file `mfdanim.mps`, the AIFF-C codec of the extracted CD audio.

## Stage 1 checklist

| Item | Status |
|---|---|
| PE resource reader, XMSH, XTXR, TGA, INI readers with unit tests | In progress |
| Packs, VFS, rules layering, scenarios | In progress |
| Rendering: LODs, lighting, starfield, sun | In progress |
| Cockpits A/B/C and HUD instruments | In progress |
| Keyboard, mouse, gamepad (macOS, Linux) | In progress |
| Flight model per hull | In progress |
| Beams, missiles, lock, targeting, damage | In progress |
| Wingmen formation | In progress |
| Testbed launcher, ship selection, model viewer | In progress |
| Sample original-free pack | In progress |
| Stage 1 [manual tests](testing/manual-tests.md) passed | Not yet |
