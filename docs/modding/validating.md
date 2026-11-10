# Validating a mod

```sh
openinterceptor --check --mods ~/my-mods            # original-free mods
openinterceptor --check --mods ~/my-mods --data /path/to/interceptor-cd
openinterceptor --list  --mods ~/my-mods            # what loaded: packs, playable ships, scenarios
```

`--check` loads everything without opening a window and reports:

- every rules definition that failed (`[ship:x]: unknown beam 'y'`, bad inherit, unknown
  faction or kind, missing `Model`, bad missile count, cockpit without `Image`);
- every ship model, debris model and texture that does not load, and unknown cockpit ids;
- every cockpit image and every referenced sound that does not load;
- every scenario that cannot spawn (with each playable ship for `player = choose`);
- with original data, every original mesh and texture that does not decode.

It ends with a summary line and exits with status 1 if there were errors, so it can run in a
script or CI. Warnings (such as a pack skipped for lack of original data) do not fail it.

The repository's own gate (`scripts/check.sh`) runs `--check --no-original` on every build, so
the `default` campaign — the template for mods — always validates. `--check` checks **every
campaign separately**, each with its own active set of packs (its parent chain, the addons that
target it and their dependencies), so an addon is validated with each campaign it applies to;
`--check --campaign <id>` checks just one.
