# The Interceptor campaign mod

**Status: design; authored across Stages 2–6, playable start to finish at the end of
[Stage 5](../implementation-plan/stage-5-campaign.md).** The existing `interceptor` pack grows
into a `kind = campaign` mod that plays the whole original campaign from the player's CD. It is
the **primary test case** of the engine: if a system of the original cannot be expressed, the
format is incomplete.

## What the mod contains — and what it does not

The mod contains **only this project's own work**: rules that inherit original sections,
definitions authored from the analysis, and references to original files. It never contains
original text, art, audio or extracted tables of the original's content beyond the names and
numbers needed to reference them (mesh names, section names, values the rules restate).

```text
packs/interceptor/
  pack.ini                     kind = campaign, requires_original = yes, format = 1
  campaign/
    campaign.ini               [campaign], [start:interceptor], endings
    doomsday.ini               [arc:doomsday], [plan:doomsday], warnings
    score.ini                  [score:interceptor] (MissionScores by name), reports
  strategy/
    map.ini                    [map:frontier] generated; [starclass:]×9; [body:]; travel
    factions.ini               xcom, alien, corporate (funding), pirates, cult, arms dealer, gsc
    economy.ini                resources, income, expenses, market, factory, delivery, storage
    research.ini               71 topics: inherit original:GameSettings.ini/R_*, + Unlocks, Appears
    salvage.ini                13 sets: inherit the *Goodies sections
    bases.ini                  [basetype:xcom], [basetype:alien], templates
    modules.ini                68 modules: inherit M_*, + Effects, OnDestroyed, model sets
    items.ini                  beams, launchers, missiles, mines, equipment: inherit originals
    pilots.ini                 [pilots]: ranks inherit RankAdvancement*, formulas authored
    strategist.ini             [strategist:alien]: Think.ini values by reference + rules
  rules/                       Stage 1–2 combat rules (ships, flight, cockpits, sounds)
  models/
    modelsets.ini              21 class model sets, 11 station sets, module sets with textures
    debris.ini                 the 28 debris sets
    effects.ini                explosions, shock waves, shield rings
  behaviours/                  per-class AI with Beginner…Superhuman variants
  missions/                    27 mission types, simulator types, the final mission
  events/                      side missions, messages, news triggers, reports
  interface/                   screens laid out with original wrapper art; cockpits A/B/C
  text/en/                     imports: original:LangFile1.txt, starcorp1, Names1, news1, short1
  scenarios/                   testbed arenas (Stage 1, kept)
```

## Import, reference, author

| Kind of content | How the mod gets it |
|---|---|
| Numbers in `GameSettings.ini`, `Think.ini`, `soundfx.ini`, `MPSettings.INI` | `inherit = original:<file>/<section>` and per-key references (`think:<section>/<key>`); never copied |
| Text | `Text = original:LangFile1.txt` and the other text files, imported at load under `orig.*` keys; the mod's own text files only map ids to those keys |
| Models, textures, images, sounds, music, video | `original:` references and override paths (Stage 1) |
| Tables the executable holds (class → mesh, debris sets, module texture triples, star visuals) | Written as rules naming the original meshes and textures — facts needed to reference the data, recorded once in the mod |
| Rules the executable holds (mission compositions, AI, story, yields, formulas) | **Authored** from the [analysis](original-analysis.md); each definition's comment cites its source (guide chapter, CD string, play-test) and confidence |

## Id mapping

Original section names become readable ids, with the original name kept as the inheritance
source:

```ini
[research:tri_titanium]
inherit = original:GameSettings.ini/R_<topic>   ; Level, FileSize, Prereq* (section name illustrative)
Size = {FileSize} * 1000                 ; the original's unit is k TQ
Requires = research(wraith_examination)  ; numeric Prereq ids resolved to readable ids
Unlocks = item:tri_titanium_armor, hull:firestar       ; AUTH: guide ch. 3
Name = @orig.R_<topic>.found              ; from short1.txt
```

Numeric tech ids in prerequisite keys (`Prereq1 = 12`) are translated by a mapping table in the
mod (`[techids]`), checked by `--check` against the original file so a mistake is reported, not
silently wrong.

## Fidelity process

1. **Stage 2** — combat behaviours: each alien class's behaviour tuned against recorded play of
   the original (where possible) and the guide's descriptions; difficulty variants from the
   guide's list of what each level changes.
2. **Stage 3** — mission types: each of the 27 types authored; compositions from the guide's
   legible tables, else estimates marked *derived*; checked by playing each in the testbed.
3. **Stages 4–5** — the strategy layer, then the arc and the strategist; a **full campaign
   play-through** at Veteran is Stage 5's exit test, with a log comparing milestone dates
   (first message, first Firestar, Doomsday progress) against the guide's expectations.
4. The [open items](coverage.md#open-items-data-the-original-keeps-in-code) are closed or
   documented as *derived* estimates in the mod.
