# Game design and mod format

**Status: design, version 1 draft (2026-11-10).** Nothing in this section is implemented yet
unless a page says so; the [implementation plan](../implementation-plan/index.md) schedules it.
Pages written here are the reference the stages implement against: a key named here is the key
the engine will read.

## The goal

OpenInterceptor is a general engine for **space-combat strategy games**: a real-time strategy
layer (a star map, bases, research, an economy, factions) wrapped around a first-person space
combat simulator. Everything a game is made of is a **mod**: a folder of rules, campaign
definitions, scripts and assets.

Three mods define what "done" means:

| Mod | Kind | What it proves |
|---|---|---|
| [`interceptor`](interceptor-campaign.md) | campaign, needs the player's CD | The format can express **every** system of the original game: the full X-COM: Interceptor campaign is the primary test case. |
| [`default`](default-campaign.md) | campaign, original-free | The engine stands alone: a complete, smaller campaign — two factions competing for the resources of a few star systems — built only from content made for this project. It ships with the engine and is what runs when no CD is present. |
| [`interceptor-remastered`](remastered.md) | campaign with `parent = interceptor` | A mod can extend another campaign and override individual assets: sharper textures for modern screens and an upgraded fighter model with real raised and recessed geometry. |

## Pages

| Page | Contents |
|---|---|
| [The original game, system by system](original-analysis.md) | A functional analysis of X-COM: Interceptor: every system, its rules, and where its data lives. |
| [Mod structure](mod-structure.md) | Mod kinds, `parent`, folder layout, load order, ids, overrides, generated assets. |
| [Format: campaign and game flow](format-campaign.md) | The campaign ruleset: start state, calendar, phases, difficulty, victory and defeat. |
| [Format: strategy layer](format-strategy.md) | Star map, factions and relations, economy, research, bases and modules, ships, equipment, pilots. |
| [Logistics](logistics.md) | Planned for the default campaign: resources and deposits, mining facilities, research stations, manufacturing centers, supply convoys and ambushes, survey probes, construction stages; open questions. |
| [Format: space objects and models](format-objects.md) | Object descriptors, model sets (intact, damage, debris, construction states), mounts. |
| [Format: missions and events](format-missions.md) | Mission types and generation, objectives, outcomes; scripted campaign events. |
| [Format: behaviours and scripting](format-behaviour.md) | AI and object behaviours with difficulty variants; the condition/action language; scripts. |
| [Format: interface, text and media](format-interface.md) | Screen layouts and skins, cockpits, text and encyclopaedia, music, video. |
| [Coverage matrix](coverage.md) | Every original system → the format construct that expresses it → its simplified equivalent in the default campaign. |
| [The Interceptor campaign mod](interceptor-campaign.md) | How the `interceptor` mod defines the full original campaign from the CD. |
| [The default campaign](default-campaign.md) | Design of the original-free campaign, its content and how its assets are generated. |
| [Interceptor Remastered](remastered.md) | Asset recipes: upscaled textures and mesh patches built locally from the player's CD. |

## Design principles

1. **Data first, scripts second.** Everything the original game does must be expressible as
   rules and declarative definitions (with a small condition language). Scripts exist for
   mods that want behaviour the declarative layer cannot describe; the `interceptor` and
   `default` campaigns must not need them. This keeps campaigns checkable by `--check`,
   saves deterministic, and modding approachable.
2. **One format for both campaigns.** Every construct the original needs has a simpler use in
   the default campaign, and vice versa ([coverage](coverage.md)). A feature only one of them
   would use is a sign the format is shaped around a game rather than a genre.
3. **Extend what exists.** The established conventions stay: YAML rules files (mod format 2,
   checked against schemas) with `inherit` (including `inherit: original:<file>/<section>` to
   read the original INI data), layering by load order, `original:` references, per-asset
   overrides at fixed paths, OBJ/PNG/WAV for mod art. New content types are new top-level
   sections and folders, not a new syntax.
4. **No original content in the repository.** The `interceptor` mod is rules and references;
   the remastered mod is *recipes* that the engine applies to the player's own files.
5. **Everything is validated.** Every reference, condition and expression is checked at load
   (`--check`) with a message naming the file, line and key path.
