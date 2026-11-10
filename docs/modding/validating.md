# Validating a mod

```sh
openinterceptor --check                   # all packs, with original data if found
openinterceptor --check --no-original     # only original-free packs
openinterceptor --check --pack mods/my-campaign
```

`--check` loads every pack without opening a window and reports:

- `pack.ini` errors and missing dependencies,
- rule sections that reference unknown ids (a ship naming a missing beam),
- models and textures that cannot be found or parsed,
- scenarios whose objects reference unknown classes,
- keys the engine does not know (warnings, to catch typos).

It exits non-zero on any error, so it can run in a mod's own CI.
