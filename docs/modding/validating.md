# Validating a mod

```sh
openinterceptor --check --mods ~/my-mods            # original-free mods
openinterceptor --check --mods ~/my-mods --data /path/to/interceptor   # an installed copy or the CD
openinterceptor --list  --mods ~/my-mods            # what loaded: packs, playable ships, scenarios
```

`--check` loads everything without opening a window and reports:

- every **file** that is not valid YAML or does not match its schema, with file, line and key
  path: `rules/ships.yaml:12: ships.comet.max_sped: unknown key (did you mean 'max_speed'?)`,
  `scenarios/t.yaml:2: unbalanced …`, a wrong type or unit (`range: 2.4kn` → "unknown unit 'kn'
  … (use m, km)");
- every rules definition that failed (`ships.x: unknown beam 'y'`, bad inherit, unknown
  faction or kind, missing `model`, cockpit without `image` or `model`, a behaviour whose
  condition does not check, an unknown difficulty level in `variants`);
- every ship model, debris model and texture that does not load, and unknown cockpit ids;
- every cockpit image and every referenced sound that does not load — since app 0.16.2 also the
  radio voice and chatter clips, the music's segments and stingers, and the voice sets' faces
  (`voices.<id>.face <file>: …`);
- every scenario that cannot spawn (with each playable ship for `player: choose`), including unknown object behaviours;
- with original data, every original mesh and texture that does not decode.

It ends with a summary line and exits with status 1 if there were errors, so it can run in a
script or CI. Warnings (such as a pack skipped for lack of original data) do not fail it.

The repository's own gate (`scripts/check.sh`) runs `--check --no-original` on every build, so
the `default` campaign — the template for mods — always validates. `--check` checks **every
campaign separately**, each with its own active set of packs (its parent chain, the addons that
target it and their dependencies), so an addon is validated with each campaign it applies to;
`--check --campaign <id>` checks just one.

Validation happens at the difficulty level being loaded (the default unless `--difficulty` is
given), so a typo inside one level's `variants` entry is reported whichever level is played.
For balance across levels, `--soak <scenario> --difficulty <level>` plays a battle without a
window and reports the player's fate; `--soak-pilot passive | ai | idle` chooses who flies the
player's craft (`idle` sits still: how long a defended object holds on its own).

**In your editor.** Point each file at its JSON Schema (`# yaml-language-server:
$schema=…`, see [Rulesets](rulesets.md#the-yaml-subset)) and most of these mistakes are
underlined as you type.
